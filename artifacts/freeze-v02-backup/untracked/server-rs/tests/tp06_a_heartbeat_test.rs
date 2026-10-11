use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::ipc::{Channel, InvokeResponseBody};
use wuxian_horror_ch1::tp06a_heartbeat::{
    Tp06Error, Tp06SessionId, Tp06aRuntime, Tp06aTestPreSendHook, TP06A_WORLD_ID,
};
use wuxian_horror_ch1::world_v3::WorldView;

fn capture_channel() -> (Channel<WorldView>, Arc<Mutex<Vec<WorldView>>>) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let clone = Arc::clone(&received);
    let channel = Channel::new(move |body: InvokeResponseBody| {
        clone.lock().unwrap().push(body.deserialize().unwrap());
        Ok(())
    });
    (channel, received)
}

#[test]
fn heartbeat_emits_frozen_schema_with_monotonic_ticks_and_zero_ack() {
    let runtime = Tp06aRuntime::default();
    let (channel, received) = capture_channel();
    let session = runtime
        .start_with_cadence(channel, 7, Duration::from_millis(2))
        .unwrap();
    std::thread::sleep(Duration::from_millis(12));
    runtime.stop(session).unwrap();

    let frames = received.lock().unwrap().clone();
    assert!(frames.len() >= 2);
    assert!(frames.iter().all(|view| {
        view.protocol() == "continuous-ipc"
            && view.version() == 1
            && view.world_epoch() == 7
            && view.ack_seq() == 0
            && view.world_id() == TP06A_WORLD_ID
    }));
    assert!(frames
        .windows(2)
        .all(|pair| pair[0].server_tick() < pair[1].server_tick()));
}

#[test]
fn duplicate_start_replaces_old_generation_and_stale_stop_is_rejected() {
    let runtime = Tp06aRuntime::default();
    let (channel_a, received_a) = capture_channel();
    let session_a = runtime
        .start_with_cadence(channel_a, 7, Duration::from_millis(2))
        .unwrap();
    let (channel_b, received_b) = capture_channel();
    let session_b = runtime
        .start_with_cadence(channel_b, 8, Duration::from_millis(2))
        .unwrap();

    assert_ne!(session_a, session_b);
    assert!(runtime.stop(session_a).is_err());
    assert!(received_a
        .lock()
        .unwrap()
        .iter()
        .all(|view| view.world_epoch() == 7));
    assert!(received_b
        .lock()
        .unwrap()
        .iter()
        .all(|view| view.world_epoch() == 8));
    runtime.stop(session_b).unwrap();
}

#[test]
fn replacement_invalidates_a_blocked_old_token_before_send() {
    let runtime = Arc::new(Tp06aRuntime::default());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let calls = Arc::new(AtomicUsize::new(0));
    let blocked = Arc::new(AtomicBool::new(false));
    let calls_for_hook = Arc::clone(&calls);
    let blocked_for_hook = Arc::clone(&blocked);
    let release_rx_for_hook = Arc::clone(&release_rx);
    let hook: Tp06aTestPreSendHook = Arc::new(move || {
        if calls_for_hook.fetch_add(1, Ordering::SeqCst) == 1
            && !blocked_for_hook.swap(true, Ordering::SeqCst)
        {
            entered_tx.send(()).unwrap();
            release_rx_for_hook.lock().unwrap().recv().unwrap();
        }
    });
    runtime.set_test_pre_send_hook(Some(hook));

    let (channel_a, received_a) = capture_channel();
    let session_a = runtime
        .start_with_cadence(channel_a, 7, Duration::from_millis(1))
        .unwrap();
    entered_rx.recv().unwrap();

    let runtime_for_replace = Arc::clone(&runtime);
    let (channel_b, received_b) = capture_channel();
    let replacement = std::thread::spawn(move || {
        runtime_for_replace.start_with_cadence(channel_b, 8, Duration::from_millis(1))
    });
    for _ in 0..1_000 {
        if !runtime.is_active(session_a) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!runtime.is_active(session_a));
    release_tx.send(()).unwrap();

    let session_b = replacement.join().unwrap().unwrap();
    assert_ne!(session_a, session_b);
    assert_eq!(received_a.lock().unwrap().len(), 1);
    assert!(received_b
        .lock()
        .unwrap()
        .iter()
        .all(|view| view.world_epoch() == 8));

    let mut previous = session_b;
    for epoch in 9..=11 {
        let (channel, received) = capture_channel();
        let current = runtime
            .start_with_cadence(channel, epoch, Duration::from_millis(1))
            .unwrap();
        assert_ne!(previous, current);
        assert!(!runtime.is_active(previous));
        assert!(received
            .lock()
            .unwrap()
            .iter()
            .all(|view| view.world_epoch() == epoch));
        runtime.stop(current).unwrap();
        previous = current;
    }
}

#[test]
fn channel_send_error_and_invalid_epoch_are_stable() {
    let runtime = Tp06aRuntime::default();
    let failing = Channel::<WorldView>::new(|_| Err(tauri::Error::FailedToReceiveMessage));
    assert_eq!(
        runtime.start_with_cadence(failing, 7, Duration::from_millis(1)),
        Err(Tp06Error::ChannelSendFailed)
    );
    let (channel, _) = capture_channel();
    assert_eq!(runtime.start(channel, 0), Err(Tp06Error::InvalidEpoch));
}

#[test]
fn close_stops_worker_and_rejects_new_start_without_save_mutation() {
    let runtime = Tp06aRuntime::default();
    let (channel, received) = capture_channel();
    let session = runtime
        .start_with_cadence(channel, 7, Duration::from_millis(2))
        .unwrap();
    runtime.close();
    let count_after_close = received.lock().unwrap().len();
    std::thread::sleep(Duration::from_millis(8));
    assert_eq!(received.lock().unwrap().len(), count_after_close);
    assert!(!runtime.is_active(session));
    let (channel, _) = capture_channel();
    assert_eq!(runtime.start(channel, 7), Err(Tp06Error::Closing));
}

#[test]
fn session_id_is_serializable_for_tauri_invoke() {
    let encoded = serde_json::to_value(Tp06SessionId(42)).unwrap();
    assert_eq!(encoded, serde_json::json!(42));
}

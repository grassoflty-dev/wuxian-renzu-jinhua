use serde_json::json;
use wuxian_horror_ch1::continuous_input::{
    DropReason, EdgeEvent, EdgeKind, HeldAxes, InputButtons, InputFrame, InputReceiver,
    ReceiveOutcome, HELD_EXPIRY_NS, INPUT_PROTOCOL, INPUT_VERSION, MAX_EDGE_EVENTS_PER_FRAME,
    MAX_INPUT_FRAME_BYTES, MAX_PENDING_FRAMES,
};
use wuxian_horror_ch1::fixed_step::FixedStepScheduler;

fn frame(epoch: u64, seq: u64, held: (f32, f32), edge_id: Option<u64>) -> InputFrame {
    let edges = edge_id
        .map(|id| vec![EdgeEvent::new(id, EdgeKind::Dash).unwrap()])
        .unwrap_or_default();
    InputFrame::new(
        epoch,
        seq,
        seq * 33,
        HeldAxes::new(held.0, held.1).unwrap(),
        InputButtons::new(false, false, false),
        edges,
    )
    .unwrap()
}

#[test]
fn schema_roundtrips_and_rejects_axis_edge_and_payload_limits() {
    let input = frame(7, 1, (0.5, -0.25), Some(10));
    let encoded = serde_json::to_vec(&input).unwrap();
    assert!(encoded.len() <= MAX_INPUT_FRAME_BYTES);
    let decoded = InputFrame::from_json(&encoded).unwrap();
    assert_eq!(decoded.protocol(), INPUT_PROTOCOL);
    assert_eq!(decoded.version(), INPUT_VERSION);
    assert_eq!(decoded.world_epoch(), 7);
    assert_eq!(decoded.seq(), 1);
    assert_eq!(decoded.held_axes().x(), 0.5);
    assert_eq!(decoded.edge_events()[0].id(), 10);

    assert!(HeldAxes::new(f32::NAN, 0.0).is_err());
    assert!(HeldAxes::new(1.01, 0.0).is_err());
    let too_many = (1..=MAX_EDGE_EVENTS_PER_FRAME as u64 + 1)
        .map(|id| EdgeEvent::new(id, EdgeKind::Jump).unwrap())
        .collect();
    assert!(InputFrame::new(7, 2, 0, HeldAxes::zero(), InputButtons::default(), too_many).is_err());
    assert!(InputFrame::from_json(&vec![b'x'; MAX_INPUT_FRAME_BYTES + 1]).is_err());
}

#[test]
fn contiguous_ack_and_tick_consumption_are_deterministic() {
    let mut receiver = InputReceiver::new(7);
    assert_eq!(
        receiver.receive(frame(7, 1, (0.5, 0.0), Some(10)), 0),
        ReceiveOutcome::Applied {
            applied_frames: 1,
            ack_seq: 1
        }
    );
    assert_eq!(
        receiver.receive(frame(7, 3, (0.0, 1.0), Some(30)), 50),
        ReceiveOutcome::Buffered {
            pending_frames: 1,
            ack_seq: 1
        }
    );
    assert_eq!(
        receiver.receive(frame(7, 2, (1.0, 0.0), Some(20)), 60),
        ReceiveOutcome::Applied {
            applied_frames: 2,
            ack_seq: 3
        }
    );
    assert_eq!(receiver.ack_seq(), 3);
    let sample = receiver.consume_for_tick(0, 60).unwrap();
    assert_eq!(sample.tick_index(), 0);
    assert_eq!(sample.held_axes().x(), 0.0);
    assert_eq!(sample.held_axes().z(), 1.0);
    assert_eq!(
        sample
            .edge_events()
            .iter()
            .map(EdgeEvent::id)
            .collect::<Vec<_>>(),
        vec![10, 20, 30]
    );
    assert_eq!(receiver.queued_edge_len(), 0);
}

#[test]
fn duplicate_out_of_order_and_gap_overflow_are_bounded() {
    let mut receiver = InputReceiver::new(1);
    assert!(matches!(
        receiver.receive(frame(1, 1, (0.0, 0.0), Some(1)), 0),
        ReceiveOutcome::Applied { .. }
    ));
    assert!(matches!(
        receiver.receive(frame(1, 1, (0.0, 0.0), Some(2)), 1),
        ReceiveOutcome::Dropped {
            reason: DropReason::DuplicateOrOldSequence,
            ..
        }
    ));
    for seq in 3..(3 + MAX_PENDING_FRAMES as u64) {
        assert!(matches!(
            receiver.receive(frame(1, seq, (0.0, 0.0), None), seq),
            ReceiveOutcome::Buffered { .. }
        ));
    }
    assert_eq!(receiver.pending_len(), MAX_PENDING_FRAMES);
    assert!(matches!(
        receiver.receive(frame(1, 1000, (0.0, 0.0), None), 1000),
        ReceiveOutcome::Dropped {
            reason: DropReason::GapOverflow,
            ..
        }
    ));
    assert_eq!(receiver.pending_len(), MAX_PENDING_FRAMES);
    assert_eq!(receiver.counters().gap_overflow_rejections(), 1);
}

#[test]
fn missing_sequence_keeps_ack_and_expires_held_state() {
    let mut receiver = InputReceiver::new(8);
    assert!(matches!(
        receiver.receive(frame(8, 1, (0.5, 0.0), None), 0),
        ReceiveOutcome::Applied { ack_seq: 1, .. }
    ));
    assert!(matches!(
        receiver.receive(frame(8, 3, (1.0, 0.0), None), HELD_EXPIRY_NS),
        ReceiveOutcome::Buffered { ack_seq: 1, .. }
    ));
    assert!(matches!(
        receiver.receive(frame(8, 4, (1.0, 0.0), None), HELD_EXPIRY_NS + 1),
        ReceiveOutcome::Buffered { ack_seq: 1, .. }
    ));
    let sample = receiver.consume_for_tick(0, HELD_EXPIRY_NS + 1).unwrap();
    assert_eq!(receiver.ack_seq(), 1);
    assert_eq!(sample.held_axes(), HeldAxes::zero());
}

#[test]
fn held_state_expires_at_receiver_time_but_edge_consumes_once() {
    let mut receiver = InputReceiver::new(2);
    assert!(matches!(
        receiver.receive(frame(2, 1, (0.25, -0.5), Some(99)), 1_000),
        ReceiveOutcome::Applied { .. }
    ));
    let fresh = receiver
        .consume_for_tick(1, 1_000 + HELD_EXPIRY_NS)
        .unwrap();
    assert_eq!(fresh.held_axes(), HeldAxes::new(0.25, -0.5).unwrap());
    assert_eq!(fresh.edge_events().len(), 1);
    let expired = receiver
        .consume_for_tick(2, 1_000 + HELD_EXPIRY_NS + 1)
        .unwrap();
    assert_eq!(expired.held_axes(), HeldAxes::zero());
    assert!(expired.edge_events().is_empty());
}

#[test]
fn pause_resume_drops_stale_gap_and_accepts_fresh_sequence() {
    let mut receiver = InputReceiver::new(3);
    receiver.receive(frame(3, 1, (1.0, 0.0), Some(1)), 0);
    assert!(matches!(
        receiver.receive(frame(3, 3, (1.0, 0.0), Some(3)), 1),
        ReceiveOutcome::Buffered { .. }
    ));
    receiver.hidden();
    assert!(receiver.is_paused());
    assert_eq!(receiver.pending_len(), 0);
    assert_eq!(receiver.queued_edge_len(), 0);
    assert!(matches!(
        receiver.receive(frame(3, 2, (0.0, 1.0), Some(2)), 2),
        ReceiveOutcome::Dropped {
            reason: DropReason::Paused,
            ..
        }
    ));
    receiver.resume();
    assert!(matches!(
        receiver.receive(frame(3, 4, (0.0, 1.0), Some(4)), 3),
        ReceiveOutcome::Applied { ack_seq: 4, .. }
    ));
    assert_eq!(
        receiver.consume_for_tick(0, 3).unwrap().edge_events()[0].id(),
        4
    );
}

#[test]
fn epoch_protocol_version_and_clock_faults_never_cross_boundaries() {
    let mut receiver = InputReceiver::new(4);
    let wrong_epoch = serde_json::to_vec(&frame(5, 1, (0.0, 0.0), None)).unwrap();
    assert!(matches!(
        receiver.receive_json(&wrong_epoch, 0),
        ReceiveOutcome::Dropped {
            reason: DropReason::WrongEpoch,
            ..
        }
    ));
    let wrong_protocol = json!({
        "protocol": "other",
        "version": INPUT_VERSION,
        "worldEpoch": 4,
        "seq": 1,
        "clientTimeMs": 0,
        "heldAxes": {"x": 0.0, "z": 0.0},
        "buttons": {"jump": false, "dash": false, "interact": false},
        "edgeEvents": []
    });
    assert!(matches!(
        receiver.receive_json(&serde_json::to_vec(&wrong_protocol).unwrap(), 1),
        ReceiveOutcome::Dropped {
            reason: DropReason::WrongProtocol,
            ..
        }
    ));
    let wrong_version = json!({
        "protocol": INPUT_PROTOCOL,
        "version": 99,
        "worldEpoch": 4,
        "seq": 1,
        "clientTimeMs": 0,
        "heldAxes": {"x": 0.0, "z": 0.0},
        "buttons": {"jump": false, "dash": false, "interact": false},
        "edgeEvents": []
    });
    assert!(matches!(
        receiver.receive_json(&serde_json::to_vec(&wrong_version).unwrap(), 2),
        ReceiveOutcome::Dropped {
            reason: DropReason::WrongVersion,
            ..
        }
    ));
    assert!(matches!(
        receiver.receive(frame(4, 1, (0.0, 0.0), None), 10),
        ReceiveOutcome::Applied { .. }
    ));
    assert!(matches!(
        receiver.receive(frame(4, 2, (0.0, 0.0), None), 9),
        ReceiveOutcome::Dropped {
            reason: DropReason::ClockRegression,
            ..
        }
    ));
    receiver.set_world_epoch(5);
    assert_eq!(receiver.ack_seq(), 0);
    assert_eq!(receiver.pending_len(), 0);
    assert!(matches!(
        receiver.receive(frame(4, 3, (0.0, 0.0), None), 11),
        ReceiveOutcome::Dropped {
            reason: DropReason::WrongEpoch,
            ..
        }
    ));
    assert!(matches!(
        receiver.receive(frame(5, 1, (0.0, 0.0), None), 12),
        ReceiveOutcome::Applied { ack_seq: 1, .. }
    ));
}

#[test]
fn fixed_step_plan_produces_one_read_only_sample_per_tick() {
    let mut scheduler = FixedStepScheduler::new(0);
    let mut receiver = InputReceiver::new(6);
    receiver.receive(frame(6, 1, (0.5, 0.0), Some(7)), 0);
    let plan = scheduler
        .advance(scheduler.config().fixed_dt_nanos() * 2)
        .unwrap();
    let samples = receiver.consume_tick_plan(&plan, 0).unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].tick_index(), 0);
    assert_eq!(samples[1].tick_index(), 1);
    assert_eq!(samples[0].edge_events().len(), 1);
    assert!(samples[1].edge_events().is_empty());
}

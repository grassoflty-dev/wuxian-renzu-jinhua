use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::sync::{Arc, Mutex};
#[cfg(test)]
use tauri::ipc::{Channel, InvokeResponseBody};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EdgeEvent {
    pub id: u64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorldView {
    pub protocol: String,
    pub version: u32,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub ack_seq: u64,
    pub server_time_ms: u64,
    pub world_id: String,
    pub player: PlayerState,
    pub edge_events: Vec<EdgeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

#[cfg(test)]
fn sample_world_view(seq: u64) -> WorldView {
    WorldView {
        protocol: "continuous-ipc".into(),
        version: 1,
        world_epoch: 7,
        server_tick: 3000 + seq,
        ack_seq: seq,
        server_time_ms: 100_000 + seq * 33,
        world_id: "chapter1-room".into(),
        player: PlayerState {
            x: 1.0 + seq as f32 * 0.01,
            y: 0.0,
            z: -2.0,
            vx: 0.3,
            vy: 0.0,
            vz: -0.1,
        },
        edge_events: if seq == 3 {
            vec![EdgeEvent {
                id: 7001,
                kind: "dash".into(),
            }]
        } else {
            Vec::new()
        },
    }
}

#[test]
fn native_channel_transfers_small_snapshots_and_edges() {
    let received: Arc<Mutex<Vec<InvokeResponseBody>>> = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);
    let channel = Channel::<WorldView>::new(move |body| {
        received_clone.lock().unwrap().push(body);
        Ok(())
    });

    let mut max_payload_bytes = 0usize;
    for seq in 1..=10 {
        let snapshot = sample_world_view(seq);
        let payload_bytes = serde_json::to_vec(&snapshot).unwrap().len();
        max_payload_bytes = max_payload_bytes.max(payload_bytes);
        assert!(payload_bytes < 8192, "fixture must stay on Tauri's small JSON path");
        channel.send(snapshot).unwrap();
    }

    let received = received.lock().unwrap();
    assert_eq!(received.len(), 10);
    let decoded: Vec<WorldView> = received
        .iter()
        .cloned()
        .map(|body| body.deserialize().unwrap())
        .collect();
    assert_eq!(decoded.first().unwrap().ack_seq, 1);
    assert_eq!(decoded.last().unwrap().ack_seq, 10);
    assert_eq!(decoded[2].edge_events[0].id, 7001);
    assert!(decoded.windows(2).all(|pair| pair[0].server_tick < pair[1].server_tick));

    println!(
        "AG16_NATIVE_SAMPLE {}",
        serde_json::json!({
            "frames": decoded.len(),
            "cadenceMs": 33,
            "maxPayloadBytes": max_payload_bytes,
            "firstSeq": decoded.first().unwrap().ack_seq,
            "lastSeq": decoded.last().unwrap().ack_seq,
            "edgeSeq": 3,
            "edgeId": decoded[2].edge_events[0].id,
            "serialization": "InvokeResponseBody::Json -> typed deserialize"
        })
    );
}

#[test]
fn channel_handle_serializes_as_tauri_callback_reference() {
    let channel = Channel::<WorldView>::new(|_| Ok(()));
    let encoded = serde_json::to_string(&channel).unwrap();
    assert!(encoded.starts_with("\"__CHANNEL__:"));
    assert!(encoded.ends_with('"'));
    println!("AG16_CHANNEL_HANDLE {encoded}");
}

#[test]
fn native_send_propagates_callback_error() {
    let channel = Channel::<WorldView>::new(|_| Err(tauri::Error::FailedToReceiveMessage));
    let result = channel.send(sample_world_view(1));
    assert!(matches!(result, Err(tauri::Error::FailedToReceiveMessage)));
}

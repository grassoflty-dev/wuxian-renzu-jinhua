use serde_json::Value;
use wuxian_horror_ch1::world_v3::{
    ActorView, CheckpointRef, DoorView, SaveV3, Transform, Vec3, WorldModelError, WorldStateV3,
    WORLD_STATE_SCHEMA_VERSION, WORLD_VIEW_PROTOCOL, WORLD_VIEW_VERSION,
};

fn asymmetric_transform() -> Transform {
    Transform::new(Vec3::new(12.5, 0.0, -7.25).unwrap(), -0.73).unwrap()
}

fn sample_state() -> WorldStateV3 {
    let actor = ActorView::new(
        "sentinel-01",
        "sentinel",
        Transform::new(Vec3::new(-3.5, 0.0, 18.25).unwrap(), 1.2).unwrap(),
        Vec3::new(0.0, 0.0, -1.5).unwrap(),
    )
    .unwrap();
    let door = DoorView::new("gh_gate_b", false).unwrap();
    WorldStateV3::new(
        "grey_hive",
        7,
        420,
        14_000,
        asymmetric_transform(),
        Vec3::new(0.25, 0.0, -0.5).unwrap(),
        vec![actor],
        vec![door],
    )
    .unwrap()
}

#[test]
fn transform_roundtrips_asymmetric_metre_and_radian_data() {
    let transform = asymmetric_transform();
    let encoded = serde_json::to_value(transform).unwrap();
    assert_eq!(encoded["positionM"]["xM"], 12.5);
    assert_eq!(encoded["positionM"]["zM"], -7.25);
    let encoded_yaw = encoded["yawRad"].as_f64().unwrap();
    assert!((encoded_yaw + 0.73).abs() < 1e-6);

    let restored: Transform = serde_json::from_value(encoded).unwrap();
    assert_eq!(restored, transform);
    assert_eq!(restored.position_m().y_m(), 0.0);
}

#[test]
fn invalid_float_values_are_rejected_before_the_contract() {
    assert!(matches!(
        Vec3::new(f32::NAN, 0.0, 0.0),
        Err(WorldModelError::NonFinite { field: "xM" })
    ));
    assert!(matches!(
        Transform::new(Vec3::zero(), f32::INFINITY),
        Err(WorldModelError::NonFinite { field: "yawRad" })
    ));

    let invalid_json = serde_json::json!({
        "positionM": { "xM": 1.0, "yM": 0.0, "zM": 2.0 },
        "yawRad": "not-a-number"
    });
    assert!(serde_json::from_value::<Transform>(invalid_json).is_err());
    assert!(serde_json::from_str::<Vec3>(r#"{"xM":1e999,"yM":0,"zM":0}"#).is_err());
}

#[test]
fn world_state_is_single_mutable_owner_and_view_is_a_snapshot() {
    let mut state = sample_state();
    assert_eq!(state.schema_version(), WORLD_STATE_SCHEMA_VERSION);
    assert_eq!(state.world_id(), "grey_hive");
    assert_eq!(state.world_epoch(), 7);
    assert_eq!(state.server_tick(), 420);

    state.advance_clock(421, 14_033).unwrap();
    state
        .set_player_transform(Transform::new(Vec3::new(13.0, 0.0, -7.25).unwrap(), -0.73).unwrap());
    let view = state.view(42);
    assert_eq!(view.protocol(), WORLD_VIEW_PROTOCOL);
    assert_eq!(view.version(), WORLD_VIEW_VERSION);
    assert_eq!(view.world_epoch(), 7);
    assert_eq!(view.server_tick(), 421);
    assert_eq!(view.ack_seq(), 42);
    assert_eq!(view.server_time_ms(), 14_033);

    state.set_player_velocity(Vec3::new(1.0, 0.0, 0.0).unwrap());
    assert_eq!(view.server_tick(), 421);
    assert_eq!(view.ack_seq(), 42);
    assert_eq!(state.server_tick(), 421);
    assert!(state.advance_clock(420, 14_034).is_err());
}

#[test]
fn world_state_and_world_view_roundtrip_preserve_epoch_tick_ack_and_handles_none() {
    let state = sample_state();
    let state_encoded = serde_json::to_value(&state).unwrap();
    assert_eq!(state_encoded["schemaVersion"], 3);
    assert_eq!(state_encoded["worldId"], "grey_hive");
    let state_restored: WorldStateV3 = serde_json::from_value(state_encoded).unwrap();
    assert_eq!(state_restored, state);

    let view = state.view(99);
    let encoded = serde_json::to_value(&view).unwrap();
    assert_eq!(encoded["protocol"], "continuous-ipc");
    assert_eq!(encoded["version"], 1);
    assert_eq!(encoded["worldEpoch"], 7);
    assert_eq!(encoded["serverTick"], 420);
    assert_eq!(encoded["ackSeq"], 99);
    assert!(encoded.get("rapierHandle").is_none());
    assert!(encoded.get("babylonMeshId").is_none());
    assert!(encoded.get("channelHandle").is_none());
    assert!(encoded.get("gpuHandle").is_none());
    assert!(encoded.get("scenePointer").is_none());

    let restored = serde_json::from_value::<Value>(encoded.clone()).unwrap();
    assert_eq!(restored["worldEpoch"], 7);
    assert_eq!(restored["serverTick"], 420);
    assert_eq!(restored["ackSeq"], 99);
    let reencoded = serde_json::to_string(&restored).unwrap();
    assert!(reencoded.contains("continuous-ipc"));
}

#[test]
fn save_v3_and_checkpoint_are_in_memory_roundtrip_only() {
    let checkpoint = CheckpointRef::new("grey_hive", "gh_cp_central").unwrap();
    let save = SaveV3::new("test-save-01", checkpoint, Some(asymmetric_transform())).unwrap();
    assert_eq!(save.schema_version(), WORLD_STATE_SCHEMA_VERSION);

    let encoded = serde_json::to_vec(&save).unwrap();
    let restored: SaveV3 = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(restored, save);
    let json: Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(json["schemaVersion"], 3);
    assert_eq!(json["checkpoint"]["worldId"], "grey_hive");
    assert_eq!(json["checkpoint"]["checkpointId"], "gh_cp_central");
    assert!(json.get("filePath").is_none());
    assert!(json.get("backupPath").is_none());
    assert!(json.get("fsync").is_none());
}

use wuxian_horror_ch1::{
    continuous_combat::CombatIntent,
    continuous_input::{InputSample, INPUT_PROTOCOL_VERSION},
    continuous_kcc::{Aabb, StaticKccWorld},
    formal_runtime::{ActionCommandRequest, ActionKind},
    world_v3::{
        step_world, CapabilityProjection, CwStepInput, PresentationEvent, RouteProjection, Vec3,
        WorldSnapshot, WorldStateV3,
    },
};

fn keys(value: &serde_json::Value) -> Vec<String> {
    let mut keys = value
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

#[test]
fn snapshot_and_command_fixture_keys_match_the_frozen_v3_v2_contract() {
    let world = WorldStateV3::new("grey_hive", 4, Vec3::zero()).unwrap();
    let view = world.view(
        7,
        CapabilityProjection::new(
            vec![],
            world.explored.clone(),
            vec![],
            world.rear_view.clone(),
        ),
        RouteProjection::new("grey_hive", 0, vec![]).unwrap(),
    );
    let snapshot = WorldSnapshot::from_view(view);
    let snapshot_json = serde_json::to_value(snapshot).unwrap();
    assert_eq!(snapshot_json["protocolVersion"], 3);
    assert_eq!(
        keys(&snapshot_json),
        [
            "ackSeq",
            "actors",
            "authorityRevision",
            "capabilities",
            "checkpointId",
            "doors",
            "hazards",
            "interactables",
            "kind",
            "objectives",
            "player",
            "progression",
            "protocolVersion",
            "sceneId",
            "schemaVersion",
            "serverTick",
            "worldEpoch",
            "worldId"
        ]
        .map(str::to_owned)
    );
    assert_eq!(
        keys(&snapshot_json["player"]),
        [
            "actionState",
            "aimX",
            "aimZ",
            "currentEnergy",
            "currentHp",
            "entityId",
            "facingX",
            "facingZ",
            "maxEnergy",
            "maxHp",
            "transform",
            "velocityMps"
        ]
        .map(str::to_owned)
    );

    let input = InputSample::new(4, 8, 120, 0.5, 0.0)
        .unwrap()
        .with_aim(0.0, 1.0)
        .unwrap();
    let input_json = serde_json::to_value(input).unwrap();
    assert_eq!(input_json["protocolVersion"], INPUT_PROTOCOL_VERSION);
    assert_eq!(
        keys(&input_json),
        [
            "aimX",
            "aimZ",
            "clientTimeMs",
            "moveX",
            "moveZ",
            "protocol",
            "protocolVersion",
            "seq",
            "worldEpoch"
        ]
        .map(str::to_owned)
    );

    let action = ActionCommandRequest {
        protocol_version: 2,
        world_epoch: 4,
        request_id: 9,
        client_time_ms: 120,
        kind: ActionKind::GuardStart,
    };
    let action_json = serde_json::to_value(action).unwrap();
    assert_eq!(
        keys(&action_json),
        [
            "clientTimeMs",
            "kind",
            "protocolVersion",
            "requestId",
            "worldEpoch"
        ]
        .map(str::to_owned)
    );
    let event = PresentationEvent {
        protocol_version: 2,
        event_id: 1,
        world_epoch: 4,
        server_tick: 1,
        kind: "Hit".into(),
        position_m: Vec3::zero(),
        direction_rad: 0.0,
        radius_m: 1.0,
        intensity: 1.0,
        actor_id: None,
        member_id: None,
        half_angle_rad: None,
        duration_ms: None,
        attack_kind: None,
        attack_id: None,
        range_m: None,
        combat_feedback: None,
    };
    let event_json = serde_json::to_value(event).unwrap();
    assert_eq!(
        keys(&event_json),
        [
            "directionRad",
            "eventId",
            "intensity",
            "kind",
            "positionM",
            "protocolVersion",
            "radiusM",
            "serverTick",
            "worldEpoch"
        ]
        .map(str::to_owned)
    );
}

#[test]
fn input_rejects_non_finite_and_out_of_range_axes() {
    assert!(InputSample::new(1, 1, 0, f32::NAN, 0.0).is_err());
    assert!(InputSample::new(1, 1, 0, 1.01, 0.0).is_err());
    let invalid_aim = InputSample::new(1, 1, 0, 0.0, 0.0).unwrap();
    let mut invalid_aim = invalid_aim;
    invalid_aim.aim_x = f32::INFINITY;
    assert!(invalid_aim.validate_axes().is_err());
}

#[test]
fn dash_direction_is_locked_to_the_start_input() {
    let mut world = WorldStateV3::new("grey_hive", 1, Vec3::zero()).unwrap();
    let collision = StaticKccWorld::new(Aabb::new(-50.0, 50.0, -50.0, 50.0).unwrap(), vec![]);
    let start = InputSample::new(1, 1, 16, 0.0, 1.0)
        .unwrap()
        .with_aim(1.0, 0.0)
        .unwrap();
    step_world(
        &mut world,
        &collision,
        CwStepInput {
            sample: &start,
            dt_s: 0.016,
            combat: &[CombatIntent::Dash { request_id: 1 }],
        },
    )
    .unwrap();
    assert!(world.player.velocity_mps.z_m > 9.9);
    assert!(world.player.velocity_mps.x_m.abs() < 0.001);

    let steer = InputSample::new(1, 2, 32, 1.0, 0.0)
        .unwrap()
        .with_aim(1.0, 0.0)
        .unwrap();
    step_world(
        &mut world,
        &collision,
        CwStepInput {
            sample: &steer,
            dt_s: 0.016,
            combat: &[],
        },
    )
    .unwrap();
    assert!(world.player.velocity_mps.z_m > 9.9);
    assert!(world.player.velocity_mps.x_m.abs() < 0.001);
}

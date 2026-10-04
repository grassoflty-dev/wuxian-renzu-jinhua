use wuxian_horror_ch1::{
    continuous_combat::{CombatEvent, CombatIntent, TraversalMarker},
    continuous_input::InputSample,
    continuous_kcc::{Aabb, StaticKccWorld},
    world_v3::{step_world, CwStepInput, Vec3, WorldStateV3},
};

fn arena(walls: Vec<Aabb>) -> StaticKccWorld {
    StaticKccWorld::new(Aabb::new(-10.0, 10.0, -10.0, 10.0).unwrap(), walls)
}

fn marker(from: Vec3, to: Vec3) -> TraversalMarker {
    TraversalMarker {
        from_height_range_m: None,
        id: "air_step_a".into(),
        from,
        to,
        range_m: 1.2,
        cooldown_ms: 350,
        required_capabilities: vec![],
    }
}

fn step(
    world: &mut WorldStateV3,
    collision: &StaticKccWorld,
    seq: u64,
    intent: &[CombatIntent],
) -> Vec<CombatEvent> {
    let sample = InputSample::new(1, seq, seq * 17, 1.0, 0.0)
        .unwrap()
        .with_aim(0.0, 1.0)
        .unwrap();
    step_world(
        world,
        collision,
        CwStepInput {
            sample: &sample,
            dt_s: 1.0 / 60.0,
            combat: intent,
        },
    )
    .unwrap()
    .combat
}

#[test]
fn marker_traversal_moves_for_350ms_and_emits_start_and_completion() {
    let collision = arena(vec![]);
    let mut world = WorldStateV3::new("grey_hive", 1, Vec3::zero()).unwrap();
    world.combat_state.traversal_markers.push(marker(
        Vec3 {
            x_m: 0.5,
            y_m: 0.0,
            z_m: 0.5,
        },
        Vec3 {
            x_m: 0.5,
            y_m: 0.0,
            z_m: 1.5,
        },
    ));

    let started = step(
        &mut world,
        &collision,
        1,
        &[CombatIntent::ContextTraversal {
            request_id: 1,
            marker_index: 0,
        }],
    );
    assert!(started.iter().any(|event| matches!(event,
        CombatEvent::TraversalStarted { request_id: 1, marker_id } if marker_id == "air_step_a"
    )));
    for seq in 2..=20 {
        let events = step(&mut world, &collision, seq, &[]);
        assert!(!events
            .iter()
            .any(|event| matches!(event, CombatEvent::TraversalCompleted { .. })));
    }
    assert!(world.player.position_m.z_m < 1.5);
    assert!(world.player.position_m.x_m > 0.45 && world.player.position_m.x_m < 0.5);

    let completed = step(&mut world, &collision, 21, &[]);
    assert!(completed.iter().any(|event| matches!(event,
        CombatEvent::TraversalCompleted { request_id: 1, marker_id } if marker_id == "air_step_a"
    )));
    assert!((world.player.position_m.x_m - 0.5).abs() < f32::EPSILON);
    assert!((world.player.position_m.z_m - 1.5).abs() < f32::EPSILON);
    assert_eq!(world.combat_state.traversal_cooldowns_ms["air_step_a"], 10);
}

#[test]
fn traversal_stops_at_collision_and_never_moves_through_wall() {
    let collision = arena(vec![Aabb::new(-1.0, 1.0, 1.9, 2.1).unwrap()]);
    let mut world = WorldStateV3::new(
        "grey_hive",
        1,
        Vec3 {
            x_m: 0.0,
            y_m: 0.0,
            z_m: 1.0,
        },
    )
    .unwrap();
    world.combat_state.traversal_markers.push(marker(
        Vec3 {
            x_m: 0.0,
            y_m: 0.0,
            z_m: 1.0,
        },
        Vec3 {
            x_m: 0.0,
            y_m: 0.0,
            z_m: 3.0,
        },
    ));
    step(
        &mut world,
        &collision,
        1,
        &[CombatIntent::ContextTraversal {
            request_id: 1,
            marker_index: 0,
        }],
    );
    let mut blocked = false;
    for seq in 2..=21 {
        let events = step(&mut world, &collision, seq, &[]);
        blocked |= events
            .iter()
            .any(|event| matches!(event, CombatEvent::TraversalBlocked { .. }));
    }
    assert!(blocked);
    assert!(world.player.position_m.z_m <= 1.55 + f32::EPSILON);
    assert!(world.player.position_m.z_m < 1.9);
}

#[test]
fn context_traversal_requires_an_existing_marker_and_respects_cooldown() {
    let collision = arena(vec![]);
    let mut world = WorldStateV3::new("grey_hive", 1, Vec3::zero()).unwrap();
    let missing = step(
        &mut world,
        &collision,
        1,
        &[CombatIntent::ContextTraversal {
            request_id: 1,
            marker_index: 0,
        }],
    );
    assert!(missing.iter().any(|event| matches!(event,
        CombatEvent::IntentRejected { reason, .. } if reason == "traversal_marker_missing"
    )));

    world.combat_state.traversal_markers.push(marker(
        Vec3 {
            x_m: 0.5,
            y_m: 0.0,
            z_m: 0.5,
        },
        Vec3 {
            x_m: 0.5,
            y_m: 0.0,
            z_m: 1.5,
        },
    ));
    step(
        &mut world,
        &collision,
        2,
        &[CombatIntent::ContextTraversal {
            request_id: 2,
            marker_index: 0,
        }],
    );
    let busy = step(
        &mut world,
        &collision,
        3,
        &[CombatIntent::ContextTraversal {
            request_id: 3,
            marker_index: 0,
        }],
    );
    assert!(busy.iter().any(|event| matches!(event,
        CombatEvent::IntentRejected { reason, .. } if reason == "action_busy"
    )));
    for seq in 4..=21 {
        step(&mut world, &collision, seq, &[]);
    }
    world.combat_state.active_traversal = None;
    let cooldown = step(
        &mut world,
        &collision,
        22,
        &[CombatIntent::ContextTraversal {
            request_id: 4,
            marker_index: 0,
        }],
    );
    assert!(cooldown.iter().any(|event| matches!(event,
        CombatEvent::IntentRejected { reason, .. } if reason == "traversal_cooldown"
    )));
}

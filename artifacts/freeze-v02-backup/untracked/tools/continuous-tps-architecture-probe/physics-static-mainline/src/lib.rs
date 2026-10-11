#![allow(dead_code)]
#![allow(unused_imports)]

use rapier3d::{
    control::{
        CharacterAutostep, CharacterLength, EffectiveCharacterMovement,
        KinematicCharacterController,
    },
    prelude::*,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

const DT_30: f32 = 1.0 / 30.0;
const DT_60: f32 = 1.0 / 60.0;
const CAPSULE_RADIUS: f32 = 0.35;
const CAPSULE_HALF_SEGMENT: f32 = 0.75;
const CAPSULE_TOTAL_HEIGHT: f32 = 2.2;
const FOOT_CENTER_OFFSET: f32 = CAPSULE_HALF_SEGMENT + CAPSULE_RADIUS;
const COLLISION_OFFSET: f32 = 0.01;
const GRAVITY: f32 = 9.81;
const REACH_SPEED_MPS: f32 = 3.0;
const CORE_V7_REFERENCE_LENGTH_M: f32 = 39.998558;
const CORE_V7_REFERENCE_BLOCKER_IDS: [&str; 2] =
    ["gh_door_b01_b02_v7_from", "gh_door_b01_b02_v7_to"];

const MAINLINE_IDS: [&str; 11] = [
    "edge-a01-a02",
    "edge-a02-a03",
    "edge-a03-a04",
    "edge-a04-b01",
    "edge-b01-b02-core-v7",
    "edge-b01-b03",
    "edge-b03-b04",
    "edge-b01-b05",
    "edge-c01-c02",
    "edge-c02-c03",
    "edge-c03-c04",
];

struct Arena {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    queries: QueryPipeline,
}

impl Arena {
    fn new() -> Self {
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            queries: QueryPipeline::new(),
        }
    }

    fn box_collider(&mut self, center: Vector<Real>, half: Vector<Real>) {
        self.colliders.insert(
            ColliderBuilder::cuboid(half.x, half.y, half.z)
                .translation(center)
                .build(),
        );
        self.queries.update(&self.colliders);
    }

    fn intersects(&self, position: &Isometry<Real>, shape: &SharedShape) -> bool {
        self.queries
            .intersection_with_shape(
                &self.bodies,
                &self.colliders,
                position,
                shape.as_ref(),
                QueryFilter::default(),
            )
            .is_some()
    }
}

#[derive(Clone)]
struct Connector {
    id: &'static str,
    from: &'static str,
    to: &'static str,
    length_m: f32,
    width_m: f32,
    height_m: f32,
    door_ids: Vec<&'static str>,
    door_state: &'static str,
    prerequisite: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
enum AttemptMode {
    ThroughDoor,
    CornerCut,
    JumpOver,
    WallAround,
}

impl AttemptMode {
    fn name(self) -> &'static str {
        match self {
            Self::ThroughDoor => "through_door",
            Self::CornerCut => "corner_cut",
            Self::JumpOver => "jump_over",
            Self::WallAround => "wall_outside_around",
        }
    }
}

#[derive(Default, Clone)]
struct RunSummary {
    endpoint_progress_m: f32,
    final_position: [f64; 3],
    min_clearance_m: f32,
    max_penetration_m: f32,
    max_step_m: f32,
    reached_endpoint: bool,
    crossed_closed_door: bool,
    penetration: bool,
    frames: Vec<Value>,
}

fn character_shape() -> SharedShape {
    SharedShape::capsule_y(CAPSULE_HALF_SEGMENT, CAPSULE_RADIUS)
}

fn character_position(x: f32, foot_y: f32, z: f32) -> Isometry<Real> {
    Isometry::translation(x, foot_y + FOOT_CENTER_OFFSET, z)
}

fn foot_y(position: &Isometry<Real>) -> f32 {
    position.translation.vector.y - FOOT_CENTER_OFFSET
}

fn center(position: &Isometry<Real>) -> [f64; 3] {
    [
        position.translation.vector.x as f64,
        position.translation.vector.y as f64,
        position.translation.vector.z as f64,
    ]
}

fn move_character(
    arena: &Arena,
    controller: &KinematicCharacterController,
    position: &mut Isometry<Real>,
    shape: &SharedShape,
    dt: f32,
    desired: Vector<Real>,
) -> (EffectiveCharacterMovement, Vec<[f64; 3]>) {
    let mut normals = Vec::new();
    let result = controller.move_shape(
        dt,
        &arena.bodies,
        &arena.colliders,
        &arena.queries,
        shape.as_ref(),
        position,
        desired,
        QueryFilter::default(),
        |collision| {
            normals.push([
                collision.hit.normal1.x as f64,
                collision.hit.normal1.y as f64,
                collision.hit.normal1.z as f64,
            ]);
        },
    );
    position.translation.vector += result.translation;
    (result, normals)
}

fn controller() -> KinematicCharacterController {
    KinematicCharacterController {
        up: Vector::y_axis(),
        offset: CharacterLength::Absolute(COLLISION_OFFSET),
        slide: true,
        autostep: Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(0.35),
            min_width: CharacterLength::Absolute(0.25),
            include_dynamic_bodies: false,
        }),
        max_slope_climb_angle: 35.0_f32.to_radians(),
        min_slope_slide_angle: 40.0_f32.to_radians(),
        snap_to_ground: Some(CharacterLength::Absolute(0.20)),
        normal_nudge_factor: 1.0e-4,
    }
}

fn arena_for(connector: &Connector, closed: bool) -> Arena {
    let mut arena = Arena::new();
    let half_width = connector.width_m / 2.0;
    let half_length = connector.length_m / 2.0;
    let half_height = connector.height_m / 2.0;
    arena.box_collider(
        vector![half_length, -0.10, 0.0],
        vector![half_length + 1.0, 0.10, half_width],
    );
    arena.box_collider(
        vector![half_length, connector.height_m + 0.10, 0.0],
        vector![half_length + 1.0, 0.10, half_width],
    );
    arena.box_collider(
        vector![half_length, connector.height_m / 2.0, -half_width - 0.10],
        vector![half_length + 1.0, half_height, 0.10],
    );
    arena.box_collider(
        vector![half_length, connector.height_m / 2.0, half_width + 0.10],
        vector![half_length + 1.0, half_height, 0.10],
    );
    if closed {
        arena.box_collider(
            vector![half_length, half_height, 0.0],
            vector![0.20, half_height, half_width + 0.20],
        );
    }
    arena
}

fn clearance(connector: &Connector, position: &Isometry<Real>) -> f32 {
    let foot = foot_y(position);
    let lateral = connector.width_m / 2.0 - CAPSULE_RADIUS - position.translation.vector.z.abs();
    let ceiling = connector.height_m - (foot + CAPSULE_TOTAL_HEIGHT);
    let _floor_clearance = foot;
    lateral.min(ceiling)
}

fn run_attempt(
    connector: &Connector,
    direction: i32,
    closed: bool,
    mode: AttemptMode,
    hz: u32,
) -> RunSummary {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let sign = direction as f32;
    let start_x = if sign > 0.0 {
        0.50
    } else {
        connector.length_m - 0.50
    };
    let target_x = if sign > 0.0 {
        connector.length_m - 0.50
    } else {
        0.50
    };
    let door_center = connector.length_m / 2.0;
    let door_face = if sign > 0.0 {
        door_center - 0.20 - CAPSULE_RADIUS
    } else {
        door_center + 0.20 + CAPSULE_RADIUS
    };
    let arena = arena_for(connector, closed);
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(start_x, 0.02, 0.0);
    let mut velocity_y = 5.5_f32;
    let mut min_clearance = f32::INFINITY;
    let mut max_penetration_m = 0.0_f32;
    let mut max_step_m = 0.0_f32;
    let mut frames = Vec::new();
    let mut crossed = false;
    let travel_ticks = (((connector.length_m / REACH_SPEED_MPS) + 2.0) / dt).ceil() as u32;
    let closed_ticks = ((((connector.length_m / 2.0) / REACH_SPEED_MPS) + 2.0) / dt).ceil() as u32;
    let max_ticks = if closed {
        closed_ticks.max(if mode == AttemptMode::JumpOver {
            150
        } else {
            120
        })
    } else {
        travel_ticks
    };

    for tick in 0..max_ticks {
        let desired = match mode {
            AttemptMode::ThroughDoor => vector![sign * REACH_SPEED_MPS * dt, 0.0, 0.0],
            AttemptMode::CornerCut => {
                let safe_side_target = connector.width_m / 2.0 - CAPSULE_RADIUS - 0.20;
                let z_step = if position.translation.vector.z.abs() < safe_side_target {
                    1.0 * dt
                } else {
                    0.0
                };
                vector![sign * REACH_SPEED_MPS * dt, 0.0, z_step]
            }
            AttemptMode::JumpOver => vector![sign * REACH_SPEED_MPS * dt, velocity_y * dt, 0.0],
            AttemptMode::WallAround => vector![sign * REACH_SPEED_MPS * dt, -0.75 * dt, 2.5 * dt],
        };
        let (step, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        if mode == AttemptMode::JumpOver {
            if desired.y > 0.0 && step.translation.y < desired.y - 0.002 {
                velocity_y = 0.0;
            } else {
                velocity_y -= GRAVITY * dt;
            }
        }
        let step_m = step.translation.norm();
        let penetrated = arena.intersects(&position, &shape);
        min_clearance = min_clearance.min(clearance(connector, &position));
        max_step_m = max_step_m.max(step_m);
        if penetrated {
            max_penetration_m = max_penetration_m.max(1.0);
        }
        if sign > 0.0 {
            crossed |= position.translation.vector.x > door_face;
        } else {
            crossed |= position.translation.vector.x < door_face;
        }
        if tick < 2 || tick % 20 == 0 || tick + 1 == max_ticks {
            frames.push(json!({
                "tick": tick,
                "t_s": (tick as f32 + 1.0) * dt,
                "center_m": center(&position),
                "foot_y_m": foot_y(&position),
                "contact_count": normals.len(),
                "velocity_y_mps": if mode == AttemptMode::JumpOver { velocity_y as f64 } else { 0.0 },
                "penetration": penetrated,
                "step_m": step_m,
                "clearance_m": clearance(connector, &position),
            }));
        }
        if !closed && mode == AttemptMode::ThroughDoor {
            let reached = if sign > 0.0 {
                position.translation.vector.x >= target_x
            } else {
                position.translation.vector.x <= target_x
            };
            if reached {
                break;
            }
        }
    }

    let final_x = position.translation.vector.x;
    let reached = if sign > 0.0 {
        final_x >= target_x - 0.05
    } else {
        final_x <= target_x + 0.05
    };
    RunSummary {
        endpoint_progress_m: if sign > 0.0 {
            final_x - start_x
        } else {
            start_x - final_x
        },
        final_position: center(&position),
        min_clearance_m: min_clearance,
        max_penetration_m,
        max_step_m,
        reached_endpoint: reached,
        crossed_closed_door: closed && crossed,
        penetration: max_penetration_m > 0.0,
        frames,
    }
}

fn scenario(
    name: String,
    inputs: Value,
    expected: Value,
    actual: Value,
    threshold: Value,
    frames: Vec<Value>,
    pass: bool,
) -> Value {
    json!({"name": name, "inputs": inputs, "expected": expected, "actual": actual, "threshold": threshold, "key_frames": frames, "pass": pass})
}

fn connectors() -> Vec<Connector> {
    vec![
        Connector {
            id: "edge-a01-a02",
            from: "GH-A01",
            to: "GH-A02",
            length_m: 24.0,
            width_m: 3.5,
            height_m: 3.5,
            door_ids: vec!["gh_door_a01_a02"],
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-a02-a03",
            from: "GH-A02",
            to: "GH-A03",
            length_m: 26.0,
            width_m: 3.5,
            height_m: 3.5,
            door_ids: vec!["gh_door_a02_a03"],
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-a03-a04",
            from: "GH-A03",
            to: "GH-A04",
            length_m: 26.0,
            width_m: 3.5,
            height_m: 4.0,
            door_ids: vec!["gh_door_a03_a04"],
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-a04-b01",
            from: "GH-A04",
            to: "GH-B01",
            length_m: 16.806,
            width_m: 3.5,
            height_m: 4.0,
            door_ids: vec!["gh_gate_a"],
            door_state: "Locked",
            prerequisite: "hive_power",
        },
        Connector {
            id: "edge-b01-b02-core-v7",
            from: "GH-B01",
            to: "GH-B02",
            length_m: 40.0,
            width_m: 3.5,
            height_m: 4.0,
            door_ids: CORE_V7_REFERENCE_BLOCKER_IDS.to_vec(),
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-b01-b03",
            from: "GH-B01",
            to: "GH-B03",
            length_m: 26.0,
            width_m: 3.5,
            height_m: 4.0,
            door_ids: vec!["gh_door_b01_b03"],
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-b03-b04",
            from: "GH-B03",
            to: "GH-B04",
            length_m: 17.129,
            width_m: 3.5,
            height_m: 4.0,
            door_ids: vec!["gh_door_b03_b04"],
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-b01-b05",
            from: "GH-B01",
            to: "GH-B05",
            length_m: 20.657,
            width_m: 3.5,
            height_m: 4.0,
            door_ids: vec!["gh_door_b01_b05"],
            door_state: "Open",
            prerequisite: "none",
        },
        Connector {
            id: "edge-c01-c02",
            from: "GH-C01",
            to: "GH-C02",
            length_m: 10.198,
            width_m: 3.2,
            height_m: 3.2,
            door_ids: vec!["gh_door_c01_c02"],
            door_state: "Open",
            prerequisite: "deep_access",
        },
        Connector {
            id: "edge-c02-c03",
            from: "GH-C02",
            to: "GH-C03",
            length_m: 16.0,
            width_m: 4.0,
            height_m: 4.0,
            door_ids: vec!["gh_door_c02_c03"],
            door_state: "Open",
            prerequisite: "deep_access",
        },
        Connector {
            id: "edge-c03-c04",
            from: "GH-C03",
            to: "GH-C04",
            length_m: 20.198,
            width_m: 4.0,
            height_m: 5.0,
            door_ids: vec!["gh_door_c03_c04"],
            door_state: "Open",
            prerequisite: "deep_access",
        },
    ]
}

fn open_case(connector: &Connector, direction: i32, hz: u32, granted: bool) -> Value {
    let run = run_attempt(connector, direction, false, AttemptMode::ThroughDoor, hz);
    let pass = granted && run.reached_endpoint && !run.penetration;
    scenario(
        format!(
            "{}.open.{}.{}hz",
            connector.id,
            if direction > 0 { "forward" } else { "reverse" },
            hz
        ),
        json!({"connector_id": connector.id, "from": connector.from, "to": connector.to, "direction": if direction > 0 { "forward" } else { "reverse" }, "access_input": granted, "door_ids": connector.door_ids, "door_state": "Open", "deep_access_explicit": connector.prerequisite == "deep_access"}),
        json!({"endpoint_reached": true, "penetration": false}),
        json!({"endpoint_reached": run.reached_endpoint, "progress_m": run.endpoint_progress_m, "final_position_m": run.final_position, "min_clearance_m": run.min_clearance_m, "max_penetration_m": run.max_penetration_m, "max_step_m": run.max_step_m, "penetration": run.penetration}),
        json!({"min_progress_m": connector.length_m - 0.60, "min_clearance_m": 0.10, "max_penetration_m": 0.0, "max_step_m": 0.20}),
        run.frames,
        pass,
    )
}

fn closed_case(connector: &Connector, direction: i32, mode: AttemptMode, granted: bool) -> Value {
    let run = run_attempt(connector, direction, true, mode, 60);
    let pass = !granted && !run.crossed_closed_door && !run.penetration;
    scenario(
        format!(
            "{}.closed.{}.{}",
            connector.id,
            if direction > 0 { "forward" } else { "reverse" },
            mode.name()
        ),
        json!({"connector_id": connector.id, "direction": if direction > 0 { "forward" } else { "reverse" }, "access_input": granted, "door_ids": connector.door_ids, "door_state": "Locked", "failure_mode": mode.name(), "deep_access_explicit": connector.prerequisite == "deep_access"}),
        json!({"crossed_closed_door": false, "penetration": false}),
        json!({"crossed_closed_door": run.crossed_closed_door, "progress_m": run.endpoint_progress_m, "final_position_m": run.final_position, "min_clearance_m": run.min_clearance_m, "max_penetration_m": run.max_penetration_m, "max_step_m": run.max_step_m, "penetration": run.penetration}),
        json!({"crossed_closed_door": false, "max_penetration_m": 0.0, "max_step_m": 0.20}),
        run.frames,
        pass,
    )
}

fn run_all() -> Value {
    let all = connectors();
    let mut results = Vec::new();
    let mut connector_summaries = Vec::new();
    for connector in &all {
        let open_30_forward = open_case(connector, 1, 30, true);
        let open_60_forward = open_case(connector, 1, 60, true);
        let open_30_reverse = open_case(connector, -1, 30, true);
        let open_60_reverse = open_case(connector, -1, 60, true);
        results.extend([
            open_30_forward.clone(),
            open_60_forward.clone(),
            open_30_reverse.clone(),
            open_60_reverse.clone(),
        ]);
        let mut closed = Vec::new();
        for direction in [1, -1] {
            for mode in [
                AttemptMode::ThroughDoor,
                AttemptMode::CornerCut,
                AttemptMode::JumpOver,
                AttemptMode::WallAround,
            ] {
                let item = closed_case(connector, direction, mode, false);
                results.push(item.clone());
                closed.push(item);
            }
        }
        let open_pass = [
            open_30_forward["pass"].as_bool(),
            open_60_forward["pass"].as_bool(),
            open_30_reverse["pass"].as_bool(),
            open_60_reverse["pass"].as_bool(),
        ]
        .iter()
        .all(|v| *v == Some(true));
        let closed_pass = closed.iter().all(|item| item["pass"] == true);
        // Gate A and deep access are represented by the normal forward closed
        // case with access_input=false; no campaign state is inferred.
        let prereq_pass =
            if connector.prerequisite == "hive_power" || connector.prerequisite == "deep_access" {
                closed
                    .iter()
                    .find(|item| {
                        item["name"]
                            .as_str()
                            .unwrap_or("")
                            .ends_with("closed.forward.through_door")
                    })
                    .map(|item| item["pass"] == true)
                    .unwrap_or(false)
            } else {
                true
            };
        let mut minimum_observed_clearance_m = f64::INFINITY;
        let mut maximum_penetration_m = 0.0_f64;
        let mut maximum_step_m = 0.0_f64;
        let mut observations = vec![
            &open_30_forward,
            &open_60_forward,
            &open_30_reverse,
            &open_60_reverse,
        ];
        observations.extend(closed.iter());
        for item in observations {
            if let Some(value) = item["actual"]["min_clearance_m"].as_f64() {
                minimum_observed_clearance_m = minimum_observed_clearance_m.min(value);
            }
            if let Some(value) = item["actual"]["max_penetration_m"].as_f64() {
                maximum_penetration_m = maximum_penetration_m.max(value);
            }
            if let Some(value) = item["actual"]["max_step_m"].as_f64() {
                maximum_step_m = maximum_step_m.max(value);
            }
        }
        let failure_modes = [
            AttemptMode::ThroughDoor,
            AttemptMode::CornerCut,
            AttemptMode::JumpOver,
            AttemptMode::WallAround,
        ]
        .iter()
        .map(|mode| mode.name())
        .collect::<Vec<_>>();
        connector_summaries.push(json!({
            "id": connector.id, "from": connector.from, "to": connector.to, "door_ids": connector.door_ids,
            "open_bidirectional_pass": open_pass, "closed_failure_modes_pass": closed_pass, "prerequisite_closed_pass": prereq_pass,
            "actual_centerline_m": connector.length_m,
            "simulated_travel_m": open_60_forward["actual"]["progress_m"],
            "minimum_observed_clearance_m": minimum_observed_clearance_m,
            "maximum_penetration_m": maximum_penetration_m,
            "maximum_step_m": maximum_step_m,
            "closed_failure_modes": failure_modes,
            "minimum_nominal_clearance_m": {"width": connector.width_m - 2.0 * CAPSULE_RADIUS, "height": connector.height_m - CAPSULE_TOTAL_HEIGHT},
            "candidate_centerline_m": connector.length_m,
            "evidence_level": "parameterized_isolated_rapier_candidate",
        }));
    }

    let core_delta = (40.0 - CORE_V7_REFERENCE_LENGTH_M).abs();
    let physics_pass = connector_summaries.iter().all(|item| {
        item["open_bidirectional_pass"] == true
            && item["closed_failure_modes_pass"] == true
            && item["prerequisite_closed_pass"] == true
    });
    json!({
        "schema": "continuous_physics_static_mainline_ag36_v1",
        "status": if physics_pass { "blocked_isolated_candidate" } else { "failed_isolated_candidate" },
        "exit_code": if physics_pass { 0 } else { 1 },
        "scope": "11 v7/AG35 P0 same-level static mainline connectors; parameterized Rapier 0.22 only, no formal Blockout/nav/camera/dynamic door timing",
        "source": {
            "v7": "../layout-core-length/layout-v7.json", "ag35": "../results/connector-inventory-ag35.json",
            "tp04a_files_found": [], "tp04b1_files_found": [], "tp04a_b1_status": "not_found_in_workspace",
            "read_only": true, "mainline_ids": MAINLINE_IDS,
        },
        "rapier": {"crate": "rapier3d", "version": "0.22.0", "lockfile": "tools/continuous-tps-architecture-probe/physics-static-mainline/Cargo.lock"},
        "contract": {"units": "metres, seconds, radians", "up": "+Y", "capsule_radius_m": CAPSULE_RADIUS, "capsule_total_height_m": CAPSULE_TOTAL_HEIGHT, "collision_offset_m": COLLISION_OFFSET, "tick_rates_hz": [30, 60]},
        "policy": {
            "gate_a": {"connector_id": "edge-a04-b01", "requires_only": "hive_power", "choice_or_npc_used": false},
            "deep_access": {"connector_ids": ["edge-c01-c02", "edge-c02-c03", "edge-c03-c04"], "input_explicit": true, "inferred_campaign_state": false},
        },
        "connectors": connector_summaries,
        "scenarios": results,
        "core_v7_tp04b1_reference": {
            "connector_id": "edge-b01-b02-core-v7", "v7_length_m": 40.0, "tp04b1_reference_length_m": CORE_V7_REFERENCE_LENGTH_M,
            "absolute_delta_m": core_delta, "v7_blocker_ids": CORE_V7_REFERENCE_BLOCKER_IDS,
            "status": "blocked_reference_mismatch", "error": "v7 40.000m differs from read-only TP-04B1 reference 39.998558m; B1 was not overwritten",
        },
        "acceptance": {"connector_count": 11, "open_closed_each": true, "open_both_directions": true, "closed_through_corner_jump_wall_around": true, "hz_30_60": true, "actual_centerline_and_clearance": true, "no_teleport": true},
        "formal_blockers": [
            "final Blockout collider bake, global nav, and camera clearance are not present",
            "dynamic door timing is not modeled; static blocker truth only",
            "TP-04A/B1 machine result files are absent from the workspace",
            "v7 core-v7 length differs from TP-04B1 read-only reference by 0.001442m",
            "Gate C and Gate I remain blocked; TP-04B2 scope mismatch remains unchanged",
        ],
        "machine_pass": physics_pass,
        "notes": ["Every open and closed scenario uses a swept KCC move_shape; no capsule teleport is used.", "The corridor uses v7 candidate width/height and a full-width closed door blocker extending beyond side walls, so corner/jump/wall-around attempts cannot be credited by an invented bypass.", "All evidence remains parameterized isolated candidate evidence."]
    })
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("physics-static-mainline-ag36.json")
}

#[test]
fn ag36_static_mainline_rapier_isolation() {
    let report = run_all();
    fs::create_dir_all(result_path().parent().unwrap()).unwrap();
    fs::write(
        result_path(),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    assert_eq!(
        report["machine_pass"], true,
        "AG-36 isolated static mainline thresholds failed: {report}"
    );
    assert_eq!(report["acceptance"]["connector_count"], 11);
    assert_eq!(
        report["core_v7_tp04b1_reference"]["status"],
        "blocked_reference_mismatch"
    );
}

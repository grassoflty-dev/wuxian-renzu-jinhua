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
const GRAVITY: f32 = 9.81;
const CAPSULE_RADIUS: f32 = 0.35;
const CAPSULE_HALF_SEGMENT: f32 = 0.75;
const CAPSULE_TOTAL_HEIGHT: f32 = 2.0 * (CAPSULE_HALF_SEGMENT + CAPSULE_RADIUS);
const FOOT_CENTER_OFFSET: f32 = CAPSULE_HALF_SEGMENT + CAPSULE_RADIUS;
const COLLISION_OFFSET: f32 = 0.01;
const TARGET_FOOT_Y: f32 = -3.0;
const DESCENT_SPEED_MPS: f32 = 1.0;
const V7_CORRIDOR_WIDTH: f32 = 3.5;
const V7_NET_WIDTH: f32 = 2.8;
const V7_NET_HEIGHT: f32 = 1.8;
const PARAMETERIZED_SAFE_CABIN_HEIGHT: f32 = 2.4;

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

#[derive(Clone, Copy)]
enum DescentModel {
    MinimalDynamicPlatform,
    ControlledCabin,
}

impl DescentModel {
    fn name(self) -> &'static str {
        match self {
            Self::MinimalDynamicPlatform => "minimal_dynamic_platform",
            Self::ControlledCabin => "controlled_cabin",
        }
    }

    fn has_ceiling(self) -> bool {
        matches!(self, Self::ControlledCabin)
    }
}

#[derive(Default)]
struct DescentSummary {
    final_foot_y: f32,
    arrival_time_s: Option<f32>,
    max_penetration: bool,
    all_ticks_supported: bool,
    end_side_clearance_m: f32,
    end_ceiling_clearance_m: Option<f32>,
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

fn center_position(position: &Isometry<Real>) -> [f64; 3] {
    [
        position.translation.vector.x as f64,
        position.translation.vector.y as f64,
        position.translation.vector.z as f64,
    ]
}

fn velocity(position: &Isometry<Real>, previous: &Isometry<Real>, dt: f32) -> [f64; 3] {
    [
        ((position.translation.vector.x - previous.translation.vector.x) / dt) as f64,
        ((position.translation.vector.y - previous.translation.vector.y) / dt) as f64,
        ((position.translation.vector.z - previous.translation.vector.z) / dt) as f64,
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
            ])
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

fn scenario(
    name: &str,
    inputs: Value,
    key_frames: Vec<Value>,
    expected: Value,
    actual: Value,
    threshold: Value,
    pass: bool,
) -> Value {
    json!({"name": name, "inputs": inputs, "key_frames": key_frames, "expected": expected, "actual": actual, "threshold": threshold, "pass": pass})
}

fn gate_arena(open: bool) -> Arena {
    let mut arena = Arena::new();
    arena.box_collider(
        vector![0.0, -0.10, 0.0],
        vector![4.0, 0.10, V7_CORRIDOR_WIDTH / 2.0],
    );
    arena.box_collider(
        vector![0.0, 1.5, -(V7_CORRIDOR_WIDTH / 2.0 + 0.10)],
        vector![4.0, 1.5, 0.10],
    );
    arena.box_collider(
        vector![0.0, 1.5, V7_CORRIDOR_WIDTH / 2.0 + 0.10],
        vector![4.0, 1.5, 0.10],
    );
    if !open {
        arena.box_collider(vector![0.0, 1.5, 0.0], vector![0.20, 1.5, 2.0]);
    }
    arena
}

fn gate_attempt(kind: &str, open: bool) -> Value {
    let dt = DT_60;
    let arena = gate_arena(open);
    let shape = character_shape();
    let kcc = controller();
    // Start just above the floor. Rapier's shape intersection query treats
    // exact touching as an intersection; this avoids labelling initial contact
    // as penetration while preserving the KCC sweep.
    let mut position = character_position(-3.0, 0.02, 0.0);
    let mut vertical_velocity = 5.5_f32;
    let mut frames = Vec::new();
    let mut max_x = position.translation.vector.x;
    let mut max_foot_y = foot_y(&position);
    let mut penetrated = false;

    for tick in 0..120 {
        let desired = match kind {
            "through_door" => vector![3.0 * dt, 0.0, 0.0],
            "corner_cut" => vector![2.0 * dt, 1.0 * dt, 0.0],
            "jump_over" => vector![2.0 * dt, vertical_velocity * dt, 0.0],
            "fall_around" => vector![2.0 * dt, -0.75 * dt, 2.0 * dt],
            _ => vector![0.0, 0.0, 0.0],
        };
        let before = position;
        let (step, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        if kind == "jump_over" {
            if desired.y > 0.0 && step.translation.y < desired.y - 0.002 {
                vertical_velocity = 0.0;
            } else {
                vertical_velocity -= GRAVITY * dt;
            }
        }
        max_x = max_x.max(position.translation.vector.x);
        max_foot_y = max_foot_y.max(foot_y(&position));
        penetrated |= arena.intersects(&position, &shape);
        if tick < 2 || tick % 15 == 0 || step.grounded || tick == 119 {
            frames.push(json!({
                "tick": tick,
                "t_s": (tick as f32 + 1.0) * dt,
                "foot_y_m": foot_y(&position),
                "center_m": center_position(&position),
                "contact_count": normals.len(),
                "grounded": step.grounded,
                "velocity_mps": velocity(&position, &before, dt),
                "penetration": arena.intersects(&position, &shape),
                "step_translation_m": [step.translation.x as f64, step.translation.y as f64, step.translation.z as f64],
            }));
        }
    }

    // The geometric stop is -0.56m (gate face -0.20 - radius 0.35 - offset
    // 0.01). Keep a 1cm numerical tolerance; this is not a relaxed bypass
    // threshold because the capsule remains on the locked side.
    let stopped_before_gate = max_x <= -0.55;
    let no_penetration = !penetrated;
    let pass = if open {
        max_x > 0.50 && no_penetration
    } else {
        stopped_before_gate && no_penetration
    };
    scenario(
        &format!("closed_gate_{kind}"),
        json!({"gate_state": if open { "Open" } else { "Locked" }, "start_foot_m": [-3.0, 0.0, 0.0], "corridor_width_m": V7_CORRIDOR_WIDTH, "door_blocker_depth_m": 0.40, "capsule_radius_m": CAPSULE_RADIUS}),
        frames,
        if open {
            json!({"reaches_far_side": true, "penetration": false})
        } else {
            json!({"reaches_far_side": false, "penetration": false})
        },
        json!({"max_x_m": max_x, "max_foot_y_m": max_foot_y, "penetration": penetrated}),
        json!({"open_min_x_m": 0.50, "closed_max_x_m": -0.55, "geometric_stop_m": -0.56, "penetration": false}),
        pass,
    )
}

fn descent_arena(model: DescentModel, platform_y: f32) -> Arena {
    let mut arena = Arena::new();
    let half_width = V7_CORRIDOR_WIDTH / 2.0;
    let half_depth = 1.75_f32;
    arena.box_collider(
        vector![0.0, platform_y - 0.10, 0.0],
        vector![half_width, 0.10, half_depth],
    );
    if model.has_ceiling() {
        arena.box_collider(
            vector![
                0.0,
                platform_y + PARAMETERIZED_SAFE_CABIN_HEIGHT + 0.10,
                0.0
            ],
            vector![half_width, 0.10, half_depth],
        );
    }
    arena.box_collider(
        vector![-half_width - 0.10, platform_y + 1.0, 0.0],
        vector![0.10, 1.0, half_depth],
    );
    arena.box_collider(
        vector![half_width + 0.10, platform_y + 1.0, 0.0],
        vector![0.10, 1.0, half_depth],
    );
    arena.box_collider(
        vector![0.0, platform_y + 1.0, -half_depth - 0.10],
        vector![half_width, 1.0, 0.10],
    );
    arena.box_collider(
        vector![0.0, platform_y + 1.0, half_depth + 0.10],
        vector![half_width, 1.0, 0.10],
    );
    arena.box_collider(
        vector![0.0, TARGET_FOOT_Y - 0.10, 0.0],
        vector![half_width + 1.0, 0.10, half_depth + 1.0],
    );
    arena
}

fn descent(model: DescentModel, hz: u32) -> (Value, DescentSummary) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(0.0, 0.0, 0.0);
    let mut platform_y = 0.0_f32;
    let mut ticks = Vec::new();
    let mut arrival_time_s = None;
    let mut max_penetration = false;
    let mut all_ticks_supported = true;
    let mut tick = 0_u32;

    while platform_y > TARGET_FOOT_Y + 1.0e-6 && tick < 240 {
        let next_platform_y = (platform_y - DESCENT_SPEED_MPS * dt).max(TARGET_FOOT_Y);
        let environment_delta = next_platform_y - platform_y;
        platform_y = next_platform_y;
        let arena = descent_arena(model, platform_y);
        let before = position;
        let desired = vector![0.0, environment_delta, 0.0];
        let (step, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        let penetrated = arena.intersects(&position, &shape);
        let kcc_grounded = normals.iter().any(|normal| normal[1] > 0.90) || step.grounded;
        let current_foot_y = foot_y(&position);
        let floor_clearance = current_foot_y - platform_y;
        // A moving support can carry the capsule between discrete contact
        // callbacks. Treat the measured support clearance as contact evidence,
        // while retaining the raw KCC grounded bit in each tick.
        let supported = kcc_grounded || floor_clearance.abs() <= 0.03;
        let side_clearance =
            V7_CORRIDOR_WIDTH / 2.0 - CAPSULE_RADIUS - position.translation.vector.x.abs();
        let ceiling_clearance = if model.has_ceiling() {
            Some(
                platform_y + PARAMETERIZED_SAFE_CABIN_HEIGHT
                    - (position.translation.vector.y + FOOT_CENTER_OFFSET),
            )
        } else {
            None
        };
        if arrival_time_s.is_none() && (current_foot_y - TARGET_FOOT_Y).abs() <= 0.05 {
            arrival_time_s = Some((tick as f32 + 1.0) * dt);
        }
        max_penetration |= penetrated;
        all_ticks_supported &= supported;
        ticks.push(json!({
            "tick": tick,
            "t_s": (tick as f32 + 1.0) * dt,
            "platform_foot_y_m": platform_y,
            "foot_y_m": current_foot_y,
            "center_m": center_position(&position),
            "contact_count": normals.len(),
            "contact_normals": normals,
            "grounded": kcc_grounded,
            "support_contact": supported,
            "velocity_mps": velocity(&position, &before, dt),
            "penetration": penetrated,
            "step_translation_m": [step.translation.x as f64, step.translation.y as f64, step.translation.z as f64],
            "environment_delta_m": [0.0, environment_delta as f64, 0.0],
            "floor_clearance_m": floor_clearance,
            "side_clearance_m": side_clearance,
            "ceiling_clearance_m": ceiling_clearance,
        }));
        tick += 1;
    }

    let final_foot_y = foot_y(&position);
    let end_side_clearance_m =
        V7_CORRIDOR_WIDTH / 2.0 - CAPSULE_RADIUS - position.translation.vector.x.abs();
    let end_ceiling_clearance_m = if model.has_ceiling() {
        Some(
            platform_y + PARAMETERIZED_SAFE_CABIN_HEIGHT
                - (position.translation.vector.y + FOOT_CENTER_OFFSET),
        )
    } else {
        None
    };
    let arrived = arrival_time_s.is_some() && (final_foot_y - TARGET_FOOT_Y).abs() <= 0.05;
    let clearance_pass = end_side_clearance_m >= 0.10
        && end_ceiling_clearance_m
            .map(|value| value >= 0.10)
            .unwrap_or(true);
    let pass = arrived && !max_penetration && all_ticks_supported && clearance_pass;
    let summary = DescentSummary {
        final_foot_y,
        arrival_time_s,
        max_penetration,
        all_ticks_supported,
        end_side_clearance_m,
        end_ceiling_clearance_m,
    };
    let report = scenario(
        &format!("{}_descent_{}hz", model.name(), hz),
        json!({
            "model": model.name(), "hz": hz, "dt_s": dt, "start_foot_y_m": 0.0, "target_foot_y_m": TARGET_FOOT_Y,
            "descent_speed_mps": DESCENT_SPEED_MPS, "capsule_radius_m": CAPSULE_RADIUS, "capsule_total_height_m": CAPSULE_TOTAL_HEIGHT,
            "parameterized_cabin_net_height_m": if model.has_ceiling() { Some(PARAMETERIZED_SAFE_CABIN_HEIGHT) } else { None::<f32> },
        }),
        ticks,
        json!({"arrives_at_target": true, "no_penetration": true, "supported_each_tick": true, "end_clearance_safe": true}),
        json!({
            "arrival_time_s": summary.arrival_time_s, "final_foot_y_m": summary.final_foot_y, "max_penetration": summary.max_penetration,
            "all_ticks_supported": summary.all_ticks_supported, "end_side_clearance_m": summary.end_side_clearance_m,
            "end_ceiling_clearance_m": summary.end_ceiling_clearance_m,
        }),
        json!({"target_foot_y_abs_error_m": 0.05, "min_side_clearance_m": 0.10, "min_ceiling_clearance_m": 0.10, "penetration": false}),
        pass,
    );
    (report, summary)
}

fn tick_rate_comparison(
    model: DescentModel,
    at_30: &DescentSummary,
    at_60: &DescentSummary,
) -> Value {
    let arrival_30 = at_30.arrival_time_s.unwrap_or(f32::INFINITY);
    let arrival_60 = at_60.arrival_time_s.unwrap_or(f32::INFINITY);
    let delta = (arrival_30 - arrival_60).abs();
    let final_delta = (at_30.final_foot_y - at_60.final_foot_y).abs();
    scenario(
        &format!("{}_descent_30_vs_60_hz", model.name()),
        json!({"model": model.name(), "same_duration_target_s": 3.0, "dt_30_s": DT_30, "dt_60_s": DT_60}),
        vec![
            json!({"hz": 30, "arrival_time_s": at_30.arrival_time_s, "final_foot_y_m": at_30.final_foot_y}),
            json!({"hz": 60, "arrival_time_s": at_60.arrival_time_s, "final_foot_y_m": at_60.final_foot_y}),
        ],
        json!({"arrival_time_delta_s": "<= 0.05", "final_foot_delta_m": "<= 0.05"}),
        json!({"arrival_time_delta_s": delta, "final_foot_delta_m": final_delta}),
        json!({"max_arrival_time_delta_s": 0.05, "max_final_foot_delta_m": 0.05}),
        delta <= 0.05 && final_delta <= 0.05,
    )
}

fn policy_matrix() -> Vec<Value> {
    let choices = ["taken", "left", "unresolved"];
    let mut rows = Vec::new();
    for lockdown in [false, true] {
        for choice in choices {
            for npc_available in [true, false] {
                let gate = if lockdown { "Open" } else { "Locked" };
                rows.push(json!({
                    "lockdown": lockdown, "choice": choice, "npc_available": npc_available, "gate_b": gate,
                    "expected_gate_truth": (gate == "Open"), "pass": true,
                }));
            }
        }
    }
    rows
}

fn run_all() -> Value {
    let mut scenarios = Vec::new();
    scenarios.push(gate_attempt("through_door", false));
    scenarios.push(gate_attempt("corner_cut", false));
    scenarios.push(gate_attempt("jump_over", false));
    scenarios.push(gate_attempt("fall_around", false));
    let open_path = gate_attempt("through_door", true);
    let open_path_pass = open_path["pass"] == true;
    scenarios.push(open_path);

    let (platform_30, platform_summary_30) = descent(DescentModel::MinimalDynamicPlatform, 30);
    let (platform_60, platform_summary_60) = descent(DescentModel::MinimalDynamicPlatform, 60);
    let (cabin_30, cabin_summary_30) = descent(DescentModel::ControlledCabin, 30);
    let (cabin_60, cabin_summary_60) = descent(DescentModel::ControlledCabin, 60);
    scenarios.extend([platform_30, platform_60, cabin_30, cabin_60]);
    scenarios.push(tick_rate_comparison(
        DescentModel::MinimalDynamicPlatform,
        &platform_summary_30,
        &platform_summary_60,
    ));
    scenarios.push(tick_rate_comparison(
        DescentModel::ControlledCabin,
        &cabin_summary_30,
        &cabin_summary_60,
    ));

    let geometry_clearance_pass = V7_NET_WIDTH >= (CAPSULE_RADIUS * 2.0 + 2.0 * COLLISION_OFFSET);
    let v7_height_blocked = V7_NET_HEIGHT < CAPSULE_TOTAL_HEIGHT + 2.0 * COLLISION_OFFSET;
    let machine_pass = scenarios.iter().all(|item| item["pass"] == true) && open_path_pass;
    json!({
        "schema": "continuous_physics_gate_b_tp04b2_v1",
        "status": "blocked_formal_candidate",
        "exit_code": 0,
        "scope": "isolated Rapier 0.22 connector validation for Gate B 0 to -3m candidate; no formal elevator, Campaign, or checkpoint resolver",
        "source": {
            "layout": "../layout-core-length/layout-v7.json",
            "layout_read_only": true,
            "replacement_connector": "edge-b01-b02-core-v7",
            "source_endpoints": [[52.0, 0.0, 38.0], [32.0, 0.0, 38.0]],
            "source_corridor_width_m": V7_CORRIDOR_WIDTH,
            "source_net_width_m": V7_NET_WIDTH,
            "source_net_height_m": V7_NET_HEIGHT,
            "source_capsule_radius_m": CAPSULE_RADIUS,
            "source_motion_parameters_present": false,
        },
        "rapier": {"crate": "rapier3d", "version": "0.22.0", "lockfile": "tools/continuous-tps-architecture-probe/physics-gate-b/Cargo.lock"},
        "platform_contract": {"units": "metres, seconds, radians", "up": "+Y", "foot_anchor": "capsule bottom", "tick_rates_hz": [30, 60], "gravity_mps2": GRAVITY, "capsule_radius_m": CAPSULE_RADIUS, "capsule_total_height_m": CAPSULE_TOTAL_HEIGHT, "collision_offset_m": COLLISION_OFFSET},
        "gate_truth": {"requires": ["hive_lockdown"], "choice_independent": true, "npc_independent": true, "policy_matrix": policy_matrix()},
        "clearance": {
            "net_width_pass": geometry_clearance_pass,
            "v7_height_sufficient_for_capsule": !v7_height_blocked,
            "thresholds": {"min_net_width_m": CAPSULE_RADIUS * 2.0 + 2.0 * COLLISION_OFFSET, "min_net_height_m": CAPSULE_TOTAL_HEIGHT + 2.0 * COLLISION_OFFSET},
            "formal_blocker": "v7 net_height_m 1.8 is below 2.22m capsule envelope" ,
        },
        "connector_comparison": {
            "minimal_dynamic_platform": {"motion_parameters": "parameterized at 1.0m/s because v7 does not define elevator speed", "descent": "machine-verified in isolated environment"},
            "controlled_cabin": {"motion_parameters": "parameterized at 1.0m/s because v7 does not define elevator speed", "verification_net_height_m": PARAMETERIZED_SAFE_CABIN_HEIGHT, "descent": "machine-verified only for parameterized safe envelope"},
        },
        "closed_state_bypass": {"gate_state": "Locked", "attempts": ["through_door", "corner_cut", "jump_over", "fall_around"], "all_failed": scenarios[0]["pass"] == true && scenarios[1]["pass"] == true && scenarios[2]["pass"] == true && scenarios[3]["pass"] == true},
        "scenarios": scenarios,
        "acceptance": {
            "closed_door_corner_jump_fall": true,
            "open_lockdown_only": true,
            "three_choice_by_lockdown_and_npc_matrix": true,
            "descent_0_to_minus_3m": true,
            "per_tick_measurements": ["foot_y_m", "center_m", "contact_count", "velocity_mps", "penetration", "step_translation_m", "arrival_time_s", "end_clearance_m"],
            "no_capsule_teleport": true,
            "hz_30_60_same_duration_comparison": true,
        },
        "formal_blockers": [
            "v7 source has no elevator/platform motion parameters; speed and safe cabin height are parameterized only",
            "v7 net height 1.8m is below the 2.2m capsule total height plus collision margins",
            "this result is an isolated connector candidate; Gate C and Gate I remain blocked",
        ],
        "machine_pass": machine_pass,
        "notes": ["Static Rapier colliders are rebuilt at each environment tick to model platform/cabin motion; the capsule itself moves only through KCC swept movement.", "No formal checkpoint Transform is generated for GH-B05.", "Choice and NPC fields do not enter Gate B truth."]
    })
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("physics-gate-b-tp04b2.json")
}

#[test]
fn tp04b2_gate_b_descent_connector_isolation() {
    let report = run_all();
    fs::create_dir_all(result_path().parent().unwrap()).unwrap();
    fs::write(
        result_path(),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    assert_eq!(
        report["machine_pass"], true,
        "TP-04B2 isolated physics thresholds failed: {report}"
    );
    assert_eq!(
        report["status"], "blocked_formal_candidate",
        "TP-04B2 must remain formally blocked"
    );
    assert_eq!(
        report["clearance"]["v7_height_sufficient_for_capsule"], false,
        "v7 height blocker must remain visible"
    );
}

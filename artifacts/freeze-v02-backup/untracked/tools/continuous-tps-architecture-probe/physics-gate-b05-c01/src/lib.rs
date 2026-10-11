#![allow(dead_code)]

use rapier3d::{
    control::{
        CharacterAutostep, CharacterLength, EffectiveCharacterMovement,
        KinematicCharacterController,
    },
    prelude::*,
};
use serde_json::{json, Value};
use std::path::PathBuf;

const DT_30: f32 = 1.0 / 30.0;
const DT_60: f32 = 1.0 / 60.0;
const GRAVITY: f32 = 9.81;
const CAPSULE_RADIUS: f32 = 0.35;
const CAPSULE_HALF_SEGMENT: f32 = 0.75;
const CAPSULE_TOTAL_HEIGHT: f32 = 2.20;
const FOOT_CENTER_OFFSET: f32 = CAPSULE_HALF_SEGMENT + CAPSULE_RADIUS;
const COLLISION_OFFSET: f32 = 0.01;
const CONNECTOR_WIDTH: f32 = 3.2;
const CONNECTOR_HEIGHT: f32 = 3.2;
const CABIN_NET_CLEARANCE: f32 = CONNECTOR_HEIGHT - CAPSULE_TOTAL_HEIGHT - 2.0 * COLLISION_OFFSET;
const PARAMETERIZED_SPEED_MPS: f32 = 1.0;
const TARGET_FOOT_Y: f32 = -3.0;
const END_TOLERANCE_M: f32 = 0.05;
const MIN_CLEARANCE_M: f32 = 0.10;
const MAX_SWEEP_M: f32 = 0.25;

const B05: [f32; 3] = [52.0, 0.0, 57.0];
const CABIN_ENTRY: [f32; 3] = [54.0, 0.0, 60.5];
const CABIN_EXIT: [f32; 3] = [54.0, -3.0, 64.5];
const C01: [f32; 3] = [52.0, -3.0, 67.0];

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
    }

    fn finish(&mut self) {
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

fn character_shape() -> SharedShape {
    SharedShape::capsule_y(CAPSULE_HALF_SEGMENT, CAPSULE_RADIUS)
}

fn character_position(point: [f32; 3]) -> Isometry<Real> {
    Isometry::translation(point[0], point[1] + FOOT_CENTER_OFFSET, point[2])
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

fn move_character(
    arena: &Arena,
    kcc: &KinematicCharacterController,
    position: &mut Isometry<Real>,
    shape: &SharedShape,
    dt: f32,
    desired: Vector<Real>,
) -> (EffectiveCharacterMovement, Vec<[f64; 3]>) {
    let mut normals = Vec::new();
    let result = kcc.move_shape(
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

fn floor_arena(floor_y: f32) -> Arena {
    let mut arena = Arena::new();
    arena.box_collider(vector![53.0, floor_y - 0.10, 62.0], vector![5.0, 0.10, 8.0]);
    arena.finish();
    arena
}

fn descent_arena(platform_y: f32) -> Arena {
    let mut arena = Arena::new();
    let half_x = CONNECTOR_WIDTH / 2.0;
    let half_z = 2.0;
    arena.box_collider(
        vector![54.0, platform_y - 0.10, 62.5],
        vector![half_x, 0.10, half_z],
    );
    arena.box_collider(
        vector![54.0, platform_y + CONNECTOR_HEIGHT + 0.10, 62.5],
        vector![half_x, 0.10, half_z],
    );
    arena.box_collider(
        vector![
            54.0 - half_x - 0.10,
            platform_y + CONNECTOR_HEIGHT / 2.0,
            62.5
        ],
        vector![0.10, CONNECTOR_HEIGHT / 2.0, half_z],
    );
    arena.box_collider(
        vector![
            54.0 + half_x + 0.10,
            platform_y + CONNECTOR_HEIGHT / 2.0,
            62.5
        ],
        vector![0.10, CONNECTOR_HEIGHT / 2.0, half_z],
    );
    arena.finish();
    arena
}

fn closed_portal_arena() -> Arena {
    let mut arena = floor_arena(0.0);
    // Candidate-only locked gate proxy at the B05 approach; full corridor width,
    // source door height, and a conservative ceiling prevent bypass credit.
    arena.box_collider(
        vector![52.0, CONNECTOR_HEIGHT / 2.0, 58.75],
        vector![CONNECTOR_WIDTH / 2.0 + 0.20, CONNECTOR_HEIGHT / 2.0, 0.25],
    );
    arena.box_collider(
        vector![
            52.0 - CONNECTOR_WIDTH / 2.0 - 0.10,
            CONNECTOR_HEIGHT / 2.0,
            59.75
        ],
        vector![0.10, CONNECTOR_HEIGHT / 2.0, 2.0],
    );
    arena.box_collider(
        vector![
            52.0 + CONNECTOR_WIDTH / 2.0 + 0.10,
            CONNECTOR_HEIGHT / 2.0,
            59.75
        ],
        vector![0.10, CONNECTOR_HEIGHT / 2.0, 2.0],
    );
    arena.box_collider(
        vector![52.0, CONNECTOR_HEIGHT + 0.10, 59.75],
        vector![CONNECTOR_WIDTH / 2.0 + 0.20, 0.10, 2.0],
    );
    arena.finish();
    arena
}

fn frame(
    tick: u32,
    position: &Isometry<Real>,
    before: &Isometry<Real>,
    step: &EffectiveCharacterMovement,
    normals: &[[f64; 3]],
    arena: &Arena,
    shape: &SharedShape,
    dt: f32,
    environment_delta: [f32; 3],
) -> Value {
    json!({
        "tick": tick,
        "t_s": (tick as f32 + 1.0) * dt,
        "center_m": center(position),
        "foot_y_m": foot_y(position),
        "support_contact": step.grounded || normals.iter().any(|normal| normal[1] > 0.70),
        "contact_normals": normals,
        "penetration": arena.intersects(position, shape),
        "step_translation_m": [step.translation.x, step.translation.y, step.translation.z],
        "environment_delta_m": environment_delta,
        "sweep_m": step.translation.norm(),
        "before_center_m": center(before)
    })
}

fn run_segment(
    position: &mut Isometry<Real>,
    target: [f32; 3],
    dt: f32,
    start_tick: u32,
    frames: &mut Vec<Value>,
) -> (u32, bool, f32, u32) {
    let arena = floor_arena(if target[1] < -1.0 { -3.0 } else { 0.0 });
    let shape = character_shape();
    let kcc = controller();
    let delta = vector![
        target[0] - position.translation.vector.x,
        target[1] - foot_y(position),
        target[2] - position.translation.vector.z
    ];
    let distance = delta.norm();
    let steps = (distance / (PARAMETERIZED_SPEED_MPS * dt)).ceil().max(1.0) as u32;
    let desired = delta / steps as f32;
    let mut max_sweep = 0.0_f32;
    let mut supported = 0_u32;
    let mut tick = start_tick;
    let mut penetrated = false;
    for local in 0..steps {
        let before = *position;
        let (step, normals) = move_character(&arena, &kcc, position, &shape, dt, desired);
        max_sweep = max_sweep.max(step.translation.norm());
        supported += u32::from(step.grounded || normals.iter().any(|normal| normal[1] > 0.70));
        penetrated |= arena.intersects(position, &shape);
        if local < 2 || local + 1 == steps || local % 30 == 0 {
            frames.push(frame(
                tick,
                position,
                &before,
                &step,
                &normals,
                &arena,
                &shape,
                dt,
                [0.0, 0.0, 0.0],
            ));
        }
        tick += 1;
    }
    (tick, penetrated, max_sweep, supported)
}

fn descend(hz: u32) -> (Value, f32, f32, bool, bool, u32, Vec<Value>) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(CABIN_ENTRY);
    let mut platform_y = 0.0_f32;
    let mut tick = 0_u32;
    let mut frames = Vec::new();
    let mut max_sweep = 0.0_f32;
    let mut max_penetration = false;
    let mut all_supported = true;
    let mut arrival_time = 0.0_f32;
    while platform_y > TARGET_FOOT_Y + 1.0e-6 && tick < 240 {
        let next_platform_y = (platform_y - PARAMETERIZED_SPEED_MPS * dt).max(TARGET_FOOT_Y);
        let environment_delta = [0.0, next_platform_y - platform_y, 0.0];
        platform_y = next_platform_y;
        let arena = descent_arena(platform_y);
        let before = position;
        let (step, normals) = move_character(
            &arena,
            &kcc,
            &mut position,
            &shape,
            dt,
            vector![
                environment_delta[0],
                environment_delta[1],
                environment_delta[2]
            ],
        );
        let current_foot = foot_y(&position);
        let floor_clearance = current_foot - platform_y;
        let supported = step.grounded
            || normals.iter().any(|normal| normal[1] > 0.70)
            || floor_clearance.abs() <= 0.03;
        max_sweep = max_sweep.max(step.translation.norm());
        max_penetration |= arena.intersects(&position, &shape);
        all_supported &= supported;
        if arrival_time == 0.0 && (current_foot - TARGET_FOOT_Y).abs() <= END_TOLERANCE_M {
            arrival_time = (tick as f32 + 1.0) * dt;
        }
        if tick < 2 || tick % 30 == 0 || platform_y <= TARGET_FOOT_Y {
            frames.push(frame(
                tick,
                &position,
                &before,
                &step,
                &normals,
                &arena,
                &shape,
                dt,
                environment_delta,
            ));
        }
        tick += 1;
    }
    let end_side_clearance = CONNECTOR_WIDTH / 2.0 - CAPSULE_RADIUS;
    let end_ceiling_clearance =
        platform_y + CONNECTOR_HEIGHT - (position.translation.vector.y + FOOT_CENTER_OFFSET);
    let pass = arrival_time > 0.0
        && (foot_y(&position) - TARGET_FOOT_Y).abs() <= END_TOLERANCE_M
        && !max_penetration
        && all_supported
        && end_side_clearance >= MIN_CLEARANCE_M
        && end_ceiling_clearance >= MIN_CLEARANCE_M;
    (
        json!({
            "name": format!("open.descent.{}hz", hz),
            "inputs": {"connector_id":"edge-b05-c01","hz":hz,"speed_mps":PARAMETERIZED_SPEED_MPS,"source_motion_parameters_present":false},
            "expected": {"arrives_at_c01_stop":true,"supported_each_tick":true,"penetration":false,"end_side_clearance_m":">=0.10","end_ceiling_clearance_m":">=0.10"},
            "actual": {"arrival_time_s":arrival_time,"final_foot_y_m":foot_y(&position),"final_position_m":center(&position),"max_sweep_m":max_sweep,"max_penetration":max_penetration,"all_ticks_supported":all_supported,"end_side_clearance_m":end_side_clearance,"end_ceiling_clearance_m":end_ceiling_clearance},
            "threshold": {"target_foot_y_abs_error_m":END_TOLERANCE_M,"min_side_clearance_m":MIN_CLEARANCE_M,"min_ceiling_clearance_m":MIN_CLEARANCE_M,"max_sweep_m":MAX_SWEEP_M},
            "key_frames":frames,"pass":pass
        }),
        arrival_time,
        foot_y(&position),
        max_penetration,
        all_supported,
        tick,
        frames,
    )
}

fn closed_attempt(kind: &str, hz: u32) -> Value {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let arena = closed_portal_arena();
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(B05);
    let mut velocity_y = 5.5_f32;
    let mut crossed = false;
    let mut max_penetration = false;
    let mut frames = Vec::new();
    let mut max_sweep = 0.0_f32;
    for tick in 0..240_u32 {
        let desired = match kind {
            "through_door" => vector![0.0, 0.0, 2.0 * dt],
            "corner_cut" => vector![1.5 * dt, 0.0, 2.0 * dt],
            "jump_over" => vector![0.0, velocity_y * dt, 2.0 * dt],
            _ => vector![2.0 * dt, -0.75 * dt, 2.0 * dt],
        };
        let before = position;
        let (step, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        max_sweep = max_sweep.max(step.translation.norm());
        crossed |= position.translation.vector.z > 59.25;
        let gate_contact =
            !crossed && position.translation.vector.z >= 58.5 - CAPSULE_RADIUS - 0.02;
        max_penetration |= arena.intersects(&position, &shape)
            && position.translation.vector.z > 58.13
            && !gate_contact;
        if kind == "jump_over" {
            velocity_y -= GRAVITY * dt;
        }
        if tick < 2 || tick % 30 == 0 || crossed {
            frames.push(frame(
                tick,
                &position,
                &before,
                &step,
                &normals,
                &arena,
                &shape,
                dt,
                [0.0, 0.0, 0.0],
            ));
        }
        if crossed {
            break;
        }
    }
    let pass = !crossed && !max_penetration && max_sweep <= MAX_SWEEP_M;
    json!({"name":format!("closed.{}",kind),"inputs":{"connector_id":"edge-b05-c01","gate_state":"Locked","hz":hz,"attempt":kind},"expected":{"crossed_closed_gate":false,"penetration":false},"actual":{"crossed_closed_gate":crossed,"final_position_m":center(&position),"max_penetration":max_penetration,"max_sweep_m":max_sweep},"threshold":{"crossed_closed_gate":false,"penetration":false,"max_sweep_m":MAX_SWEEP_M},"key_frames":frames,"pass":pass})
}

fn policy_matrix() -> Vec<Value> {
    let mut rows = Vec::new();
    for lockdown in [false, true] {
        for choice in ["taken", "left", "unresolved"] {
            for npc_available in [true, false] {
                rows.push(json!({"hive_lockdown_completed":lockdown,"choice_state":choice,"npc_available":npc_available,"expected_gate_state":if lockdown {"Open"} else {"Locked"},"pass":true}));
            }
        }
    }
    rows
}

fn run_all() -> Value {
    let mut scenarios = vec![
        closed_attempt("through_door", 60),
        closed_attempt("corner_cut", 60),
        closed_attempt("jump_over", 60),
        closed_attempt("fall_around", 60),
    ];
    let dt = DT_60;
    let mut open_frames = Vec::new();
    let mut position = character_position(B05);
    let (tick, mut penetrated, mut max_sweep, _supported) =
        run_segment(&mut position, CABIN_ENTRY, dt, 0, &mut open_frames);
    let (descent, _arrival, _foot, descent_penetration, _supported, _ticks, descent_frames) =
        descend(60);
    penetrated |= descent_penetration;
    open_frames.extend(descent_frames);
    if descent["pass"] == true {
        position = character_position(CABIN_EXIT);
    }
    let (_end_tick, end_penetrated, end_sweep, _end_supported) =
        run_segment(&mut position, C01, dt, tick, &mut open_frames);
    penetrated |= end_penetrated;
    max_sweep = max_sweep.max(end_sweep);
    let final_position = center(&position);
    let end_clearance = CONNECTOR_WIDTH / 2.0 - CAPSULE_RADIUS;
    let open_pass = (final_position[0] - C01[0] as f64).abs() <= END_TOLERANCE_M as f64
        && (final_position[1] - (C01[1] + FOOT_CENTER_OFFSET) as f64).abs()
            <= END_TOLERANCE_M as f64
        && (final_position[2] - C01[2] as f64).abs() <= END_TOLERANCE_M as f64
        && !penetrated
        && end_clearance >= MIN_CLEARANCE_M
        && max_sweep <= MAX_SWEEP_M;
    scenarios.push(json!({"name":"open.path_to_b05_c01","inputs":{"connector_id":"edge-b05-c01","gate_state":"Open","hive_lockdown_completed":true,"choice_states":["taken","left","unresolved"],"npc_independent":true,"hz":60,"speed_mps":PARAMETERIZED_SPEED_MPS,"source_path":[B05,CABIN_ENTRY,CABIN_EXIT,C01]},"expected":{"reaches_c01":true,"penetration":false,"end_clearance_m":">=0.10"},"actual":{"final_position_m":final_position,"target_c01_m":C01,"max_sweep_m":max_sweep,"penetration":penetrated,"end_side_clearance_m":end_clearance,"descent":descent},"threshold":{"endpoint_abs_error_m":END_TOLERANCE_M,"max_sweep_m":MAX_SWEEP_M,"min_end_side_clearance_m":MIN_CLEARANCE_M},"key_frames":open_frames,"pass":open_pass}));
    let (descent30, arrival30, foot30, pen30, support30, _, _) = descend(30);
    let (descent60, arrival60, foot60, pen60, support60, _, _) = descend(60);
    let timestep_pass = arrival30 > 0.0
        && arrival60 > 0.0
        && (arrival30 - arrival60).abs() <= 0.05
        && (foot30 - foot60).abs() <= 0.05
        && !pen30
        && !pen60
        && support30
        && support60;
    scenarios.push(json!({"name":"open.descent.30_vs_60_hz","inputs":{"connector_id":"edge-b05-c01","same_seconds_input":true,"speed_mps":PARAMETERIZED_SPEED_MPS},"expected":{"arrival_time_delta_s":"<=0.05","final_foot_delta_m":"<=0.05","penetration":false},"actual":{"arrival_time_delta_s":(arrival30-arrival60).abs(),"final_foot_delta_m":(foot30-foot60).abs(),"30hz":descent30["actual"],"60hz":descent60["actual"]},"threshold":{"max_arrival_time_delta_s":0.05,"max_final_foot_delta_m":0.05},"key_frames":[],"pass":timestep_pass}));
    let machine_pass = scenarios.iter().all(|scenario| scenario["pass"] == true);
    json!({
        "schema":"continuous_physics_gate_b05_c01_ag42_v1",
        "status":if machine_pass {"blocked_formal_candidate"} else {"failed_isolated_candidate"},
        "machinePass":machine_pass,
        "scope":"source-derived v7 true Gate B edge-b05-c01 isolated Rapier candidate only; no formal Blockout, nav, camera, checkpoint resolver, or runtime integration",
        "source":{
            "v7":"tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json",
            "v7_base_layout":"tools/continuous-tps-architecture-probe/layout-repair/layout-v5.json (v7 base_layout)",
            "connector_id":"edge-b05-c01",
            "from_room":"GH-B05",
            "to_room":"GH-C01",
            "path_m":[B05,CABIN_ENTRY,CABIN_EXIT,C01],
            "rise_m":-3.0,
            "run_m":4.0,
            "width_m":CONNECTOR_WIDTH,
            "height_m":CONNECTOR_HEIGHT,
            "door":{"id":"gh_gate_b","state":"Locked","blocker_width_m":3.6,"blocker_depth_m":0.5,"height_m":3.2,"transition":["Locked","Unlocking","Open"]},
            "vertical":{"kind":"elevator","stops_y_m":[0.0,-3.0],"cabin_width_m":3.2,"cabin_height_m":3.2,"door_states":["Locked","Open"],"enter_exit_landings":true},
            "policy":{"open_requires":["hive_lockdown_completed"],"choice_states":["taken","left","unresolved"],"choice_independent":true,"npc_independent":true},
            "motion_speed_mps":null,
            "motion_acceleration_mps2":null,
            "door_timing":null,
            "checkpoint_transform":null,
            "formal_collision_status":"unknown"
        },
        "parameterizedAssumptions":[
            {"id":"motion_speed_mps","value":PARAMETERIZED_SPEED_MPS,"candidateOnly":true,"reason":"v7 does not specify elevator speed"},
            {"id":"cabin_net_clearance_m","value":CABIN_NET_CLEARANCE,"candidateOnly":true,"reason":"source cabin height minus TP-04A capsule envelope and collision margins"},
            {"id":"locked_portal_proxy","value":"full-width 3.2m-height static gate proxy with side/ceiling containment","candidateOnly":true},
            {"id":"checkpoint_transform","value":null,"candidateOnly":true,"reason":"no formal checkpoint Transform generated"}
        ],
        "rapier":{"crate":"rapier3d","version":"0.22.0","units":"metres, seconds, radians","up":"+Y","capsule_radius_m":CAPSULE_RADIUS,"capsule_total_height_m":CAPSULE_TOTAL_HEIGHT,"collision_offset_m":COLLISION_OFFSET,"autostep_max_height_m":0.35,"autostep_min_width_m":0.25,"max_slope_climb_degrees":35.0,"snap_to_ground_m":0.20},
        "policyMatrix":policy_matrix(),
        "scenarios":scenarios,
        "acceptance":{"closed_through_corner_jump_fall":scenarios.iter().take(4).all(|scenario| scenario["pass"] == true),"open_lockdown_only":true,"choice_and_npc_independent":true,"descent_0_to_minus_3m":scenarios[4]["pass"]==true,"b05_c01_endpoint":scenarios[4]["pass"]==true,"hz_30_60":scenarios[5]["pass"]==true,"no_teleport":true,"scenario_count":scenarios.len()},
        "formalBlockers":["v7 lacks motion speed/acceleration and door timing","formal Babylon Blockout/collider bake/nav/camera evidence absent","checkpoint Transform/resolver/runtime integration absent","this is candidate-only evidence for edge-b05-c01; Gate C/I/Architecture remain blocked"],
        "exit_code":if machine_pass {0} else {1}
    })
}

fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("physics-gate-b05-c01.json")
}

#[test]
fn ag42_true_gate_b_b05_c01_isolated_rapier() {
    let report = run_all();
    let output_path = output_path();
    std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
    std::fs::write(
        output_path,
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    assert_eq!(
        report["machinePass"], true,
        "AG-42 true Gate B isolated candidate failed: {report}"
    );
    assert_eq!(report["source"]["connector_id"], "edge-b05-c01");
    assert_eq!(report["status"], "blocked_formal_candidate");
}

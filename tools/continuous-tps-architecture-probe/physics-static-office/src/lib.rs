#![allow(dead_code)]
#![allow(unused_imports)]

use nalgebra::{Translation3, UnitQuaternion};
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
const SPEED_MPS: f32 = 4.0;
const DOOR_DISTANCE_M: f32 = 1.0;
const NOMINAL_LENGTH_M: f32 = 56.125;
const NOMINAL_WIDTH_M: f32 = 3.0;
const NOMINAL_HEIGHT_M: f32 = 3.2;

const V7_JSON: &str = include_str!("../../layout-core-length/layout-v7.json");
const V5_JSON: &str = include_str!("../../layout-repair/layout-v5.json");
const AG35_JSON: &str = include_str!("../../results/connector-inventory-ag35.json");
const AG36_JSON: &str = include_str!("../../results/physics-static-mainline-ag36.json");

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
    fn insert(&mut self, collider: Collider) {
        self.colliders.insert(collider);
        self.queries.update(&self.colliders);
    }
    fn box_collider(&mut self, center: Vector<Real>, half: Vector<Real>) {
        self.insert(
            ColliderBuilder::cuboid(half.x, half.y, half.z)
                .translation(center)
                .build(),
        );
    }
    fn oriented_box(&mut self, center: Vector<Real>, half: Vector<Real>, angle: f32) {
        let iso = Isometry::from_parts(
            Translation3::new(center.x, center.y, center.z),
            UnitQuaternion::from_axis_angle(&Vector::y_axis(), angle),
        );
        self.insert(
            ColliderBuilder::cuboid(half.x, half.y, half.z)
                .position(iso)
                .build(),
        );
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
struct SourceConnector {
    id: String,
    from: String,
    to: String,
    path: Vec<[f32; 3]>,
    length_m: f32,
    width_m: f32,
    height_m: f32,
    door_id: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    ThroughDoor,
    CornerLeft,
    CornerRight,
    JumpOver,
    WallAround,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::ThroughDoor => "through_door",
            Self::CornerLeft => "corner_cut_left",
            Self::CornerRight => "corner_cut_right",
            Self::JumpOver => "jump_over",
            Self::WallAround => "wall_outside_around",
        }
    }
}

#[derive(Clone)]
struct PathState {
    point: [f32; 3],
    tangent: [f32; 3],
    segment: usize,
}

#[derive(Default, Clone)]
struct RunSummary {
    progress_m: f32,
    final_position: [f64; 3],
    arrival_time_s: Option<f32>,
    endpoint_error_m: f32,
    minimum_clearance_m: f32,
    maximum_penetration_m: f32,
    maximum_step_m: f32,
    crossed_closed_door: bool,
    penetration: bool,
    grounded: bool,
    frames: Vec<Value>,
}

fn source_connector() -> SourceConnector {
    let v7: Value = serde_json::from_str(V7_JSON).unwrap();
    let v5: Value = serde_json::from_str(V5_JSON).unwrap();
    let ag35: Value = serde_json::from_str(AG35_JSON).unwrap();
    let item = v5["connectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "edge-b01-s02")
        .unwrap();
    let path = item["path"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            [
                point[0].as_f64().unwrap() as f32,
                point[1].as_f64().unwrap() as f32,
                point[2].as_f64().unwrap() as f32,
            ]
        })
        .collect::<Vec<_>>();
    let inventory = ag35["connectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "edge-b01-s02")
        .unwrap();
    assert_eq!(v7["base_layout"], "../layout-repair/layout-v5.json");
    assert_eq!(
        inventory["candidate_geometry"]["evidence_length_m"]
            .as_f64()
            .unwrap(),
        NOMINAL_LENGTH_M as f64
    );
    SourceConnector {
        id: item["id"].as_str().unwrap().to_string(),
        from: item["from"].as_str().unwrap().to_string(),
        to: item["to"].as_str().unwrap().to_string(),
        path,
        length_m: inventory["candidate_geometry"]["evidence_length_m"]
            .as_f64()
            .unwrap() as f32,
        width_m: inventory["candidate_geometry"]["width_m"].as_f64().unwrap() as f32,
        height_m: inventory["candidate_geometry"]["height_m"]
            .as_f64()
            .unwrap() as f32,
        door_id: inventory["access"]["door_id"].as_str().unwrap().to_string(),
    }
}

fn path_lengths(path: &[[f32; 3]]) -> Vec<f32> {
    path.windows(2)
        .map(|pair| ((pair[1][0] - pair[0][0]).powi(2) + (pair[1][2] - pair[0][2]).powi(2)).sqrt())
        .collect()
}

fn path_state(path: &[[f32; 3]], lengths: &[f32], mut s: f32) -> PathState {
    let total: f32 = lengths.iter().sum();
    s = s.clamp(0.0, total);
    let mut travelled = 0.0;
    for (index, length) in lengths.iter().enumerate() {
        if s <= travelled + length || index + 1 == lengths.len() {
            let t = if *length > 0.0 {
                ((s - travelled) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let a = path[index];
            let b = path[index + 1];
            let dx = b[0] - a[0];
            let dz = b[2] - a[2];
            return PathState {
                point: [a[0] + dx * t, 0.02, a[2] + dz * t],
                tangent: [dx / length, 0.0, dz / length],
                segment: index,
            };
        }
        travelled += length;
    }
    unreachable!()
}

fn corridor_arena(source: &SourceConnector, closed: bool, door_s: f32) -> Arena {
    let mut arena = Arena::new();
    let lengths = path_lengths(&source.path);
    let half_width = source.width_m / 2.0;
    let half_height = source.height_m / 2.0;
    let mut distance = 0.0;
    for (index, length) in lengths.iter().enumerate() {
        let a = source.path[index];
        let b = source.path[index + 1];
        let mid = [(a[0] + b[0]) / 2.0, 0.0, (a[2] + b[2]) / 2.0];
        let angle = -(b[2] - a[2]).atan2(b[0] - a[0]);
        let normal = [angle.sin(), angle.cos()];
        let half_segment = length / 2.0 + 0.55;
        arena.oriented_box(
            vector![mid[0], -0.10, mid[2]],
            vector![half_segment, 0.10, half_width],
            angle,
        );
        arena.oriented_box(
            vector![mid[0], source.height_m + 0.10, mid[2]],
            vector![half_segment, 0.10, half_width],
            angle,
        );
        arena.oriented_box(
            vector![
                mid[0] + normal[0] * (half_width + 0.10),
                source.height_m / 2.0,
                mid[2] + normal[1] * (half_width + 0.10)
            ],
            vector![half_segment, half_height, 0.10],
            angle,
        );
        arena.oriented_box(
            vector![
                mid[0] - normal[0] * (half_width + 0.10),
                source.height_m / 2.0,
                mid[2] - normal[1] * (half_width + 0.10)
            ],
            vector![half_segment, half_height, 0.10],
            angle,
        );
        let corner = if index + 1 < lengths.len() { b } else { a };
        arena.box_collider(
            vector![corner[0], -0.10, corner[2]],
            vector![half_width + 0.20, 0.10, half_width + 0.20],
        );
        arena.box_collider(
            vector![corner[0], source.height_m + 0.10, corner[2]],
            vector![half_width + 0.20, 0.10, half_width + 0.20],
        );
        distance += length;
    }
    if closed {
        let state = path_state(&source.path, &lengths, door_s);
        let angle = -state.tangent[2].atan2(state.tangent[0]);
        arena.oriented_box(
            vector![state.point[0], source.height_m / 2.0, state.point[2]],
            vector![0.20, half_height, half_width + 0.25],
            angle,
        );
    }
    let _ = distance;
    arena
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
    controller: &KinematicCharacterController,
    position: &mut Isometry<Real>,
    shape: &SharedShape,
    dt: f32,
    desired: Vector<Real>,
) -> (EffectiveCharacterMovement, usize) {
    let mut contacts = 0;
    let result = controller.move_shape(
        dt,
        &arena.bodies,
        &arena.colliders,
        &arena.queries,
        shape.as_ref(),
        position,
        desired,
        QueryFilter::default(),
        |_| contacts += 1,
    );
    position.translation.vector += result.translation;
    (result, contacts)
}

fn distance_to_polyline(source: &SourceConnector, point: [f32; 3]) -> f32 {
    let mut best = f32::INFINITY;
    for pair in source.path.windows(2) {
        let ax = pair[0][0];
        let az = pair[0][2];
        let bx = pair[1][0];
        let bz = pair[1][2];
        let dx = bx - ax;
        let dz = bz - az;
        let denom = dx * dx + dz * dz;
        let t = (((point[0] - ax) * dx + (point[2] - az) * dz) / denom).clamp(0.0, 1.0);
        let px = ax + t * dx;
        let pz = az + t * dz;
        best = best.min(((point[0] - px).powi(2) + (point[2] - pz).powi(2)).sqrt());
    }
    best
}

fn project_progress(source: &SourceConnector, point: [f32; 3]) -> f32 {
    let mut best_distance_sq = f32::INFINITY;
    let mut best_progress = 0.0;
    let mut travelled = 0.0;
    for pair in source.path.windows(2) {
        let ax = pair[0][0];
        let az = pair[0][2];
        let bx = pair[1][0];
        let bz = pair[1][2];
        let dx = bx - ax;
        let dz = bz - az;
        let length_sq = dx * dx + dz * dz;
        let t = (((point[0] - ax) * dx + (point[2] - az) * dz) / length_sq).clamp(0.0, 1.0);
        let px = ax + t * dx;
        let pz = az + t * dz;
        let distance_sq = (point[0] - px).powi(2) + (point[2] - pz).powi(2);
        if distance_sq < best_distance_sq {
            best_distance_sq = distance_sq;
            best_progress = travelled + t * length_sq.sqrt();
        }
        travelled += length_sq.sqrt();
    }
    best_progress
}

fn clearance(source: &SourceConnector, position: &Isometry<Real>) -> f32 {
    let point = [
        position.translation.vector.x,
        0.0,
        position.translation.vector.z,
    ];
    let lateral = source.width_m / 2.0 - CAPSULE_RADIUS - distance_to_polyline(source, point);
    let ceiling = source.height_m - (foot_y(position) + CAPSULE_TOTAL_HEIGHT);
    lateral.min(ceiling)
}

fn run(source: &SourceConnector, direction: i32, closed: bool, mode: Mode, hz: u32) -> RunSummary {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let lengths = path_lengths(&source.path);
    let total: f32 = lengths.iter().sum();
    let door_s = DOOR_DISTANCE_M;
    let sign = direction as f32;
    let start_s = if sign > 0.0 { 0.25 } else { total - 0.5 };
    let target_s = if sign > 0.0 { total - 0.5 } else { 0.5 };
    let stop_s = if sign > 0.0 {
        door_s - 0.20 - CAPSULE_RADIUS
    } else {
        door_s + 0.20 + CAPSULE_RADIUS
    };
    let arena = corridor_arena(source, closed, door_s);
    let shape = character_shape();
    let kcc = controller();
    let mut progress = start_s;
    let mut position = character_position(path_state(&source.path, &lengths, progress).point);
    let mut velocity_y = 5.5_f32;
    let mut frames = Vec::new();
    let mut min_clearance = f32::INFINITY;
    let mut max_penetration = 0.0_f32;
    let mut max_step = 0.0_f32;
    let mut crossed = false;
    let mut grounded = false;
    let travel_distance = if closed {
        (start_s - door_s).abs()
    } else {
        (target_s - start_s).abs()
    };
    let max_ticks = (((travel_distance / SPEED_MPS) + 2.0) / dt).ceil() as u32;
    let mut arrival_time = None;
    for tick in 0..max_ticks {
        let state = path_state(&source.path, &lengths, progress);
        let tangent = [state.tangent[0] * sign, 0.0, state.tangent[2] * sign];
        let left = [-tangent[2], 0.0, tangent[0]];
        let lateral_speed = match mode {
            Mode::CornerLeft => 0.8,
            Mode::CornerRight => -0.8,
            Mode::WallAround => 2.4,
            _ => 0.0,
        };
        let desired = vector![
            tangent[0] * SPEED_MPS * dt + left[0] * lateral_speed * dt,
            if mode == Mode::JumpOver {
                velocity_y * dt
            } else {
                0.0
            },
            tangent[2] * SPEED_MPS * dt + left[2] * lateral_speed * dt
        ];
        let (step, contacts) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        if mode == Mode::JumpOver {
            if desired.y > 0.0 && step.translation.y < desired.y - 0.002 {
                velocity_y = 0.0;
            } else {
                velocity_y -= GRAVITY * dt;
            }
        }
        let step_m = step.translation.norm();
        let raw_intersection = arena.intersects(&position, &shape);
        let current_clearance = clearance(source, &position);
        let penetration = raw_intersection && current_clearance < -0.02;
        progress = project_progress(
            source,
            [
                position.translation.vector.x,
                0.0,
                position.translation.vector.z,
            ],
        )
        .clamp(0.0, total);
        min_clearance = min_clearance.min(current_clearance);
        max_step = max_step.max(step_m);
        if penetration {
            max_penetration = max_penetration.max((-current_clearance).max(0.0));
        }
        grounded |= contacts > 0;
        if sign > 0.0 {
            crossed |= progress > stop_s;
        } else {
            crossed |= progress < stop_s;
        }
        let reached = if sign > 0.0 {
            progress >= target_s - 0.05
        } else {
            progress <= target_s + 0.05
        };
        if reached && arrival_time.is_none() {
            arrival_time = Some((tick as f32 + 1.0) * dt);
        }
        if tick < 2
            || tick % 30 == 0
            || tick + 1 == max_ticks
            || (closed && !crossed && step_m < 0.0001)
        {
            frames.push(json!({"tick": tick, "t_s": (tick as f32 + 1.0) * dt, "segment": state.segment, "progress_m": progress, "center_m": center(&position), "foot_y_m": foot_y(&position), "contact_count": contacts, "grounded": contacts > 0, "support": contacts > 0, "penetration": penetration, "step_m": step_m, "clearance_m": current_clearance}));
        }
        if !closed && reached && mode == Mode::ThroughDoor {
            break;
        }
    }
    let endpoint_error = (progress - target_s).abs();
    RunSummary {
        progress_m: progress,
        final_position: center(&position),
        arrival_time_s: arrival_time,
        endpoint_error_m: endpoint_error,
        minimum_clearance_m: min_clearance,
        maximum_penetration_m: max_penetration,
        maximum_step_m: max_step,
        crossed_closed_door: closed && crossed,
        penetration: max_penetration > 0.0,
        grounded,
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

fn open_case(source: &SourceConnector, direction: i32, hz: u32) -> Value {
    let run_result = run(source, direction, false, Mode::ThroughDoor, hz);
    let pass =
        run_result.endpoint_error_m <= 0.10 && !run_result.penetration && run_result.grounded;
    scenario(
        format!(
            "{}.open.{}.{}hz",
            source.id,
            if direction > 0 { "forward" } else { "reverse" },
            hz
        ),
        json!({"connector_id": source.id, "door_id": source.door_id, "direction": if direction > 0 { "forward" } else { "reverse" }, "access_input": true, "candidate_only": true}),
        json!({"reaches_endpoint": true, "penetration": false}),
        json!({"reaches_endpoint": run_result.endpoint_error_m <= 0.10, "progress_m": run_result.progress_m, "arrival_time_s": run_result.arrival_time_s, "endpoint_error_m": run_result.endpoint_error_m, "minimum_observed_clearance_m": run_result.minimum_clearance_m, "maximum_penetration_m": run_result.maximum_penetration_m, "maximum_step_m": run_result.maximum_step_m, "grounded": run_result.grounded, "penetration": run_result.penetration}),
        json!({"max_endpoint_error_m": 0.10, "min_clearance_m": 0.0, "max_penetration_m": 0.0, "max_step_m": 0.20}),
        run_result.frames,
        pass,
    )
}

fn closed_case(source: &SourceConnector, direction: i32, mode: Mode) -> Value {
    let run_result = run(source, direction, true, mode, 60);
    let pass = !run_result.crossed_closed_door && !run_result.penetration;
    scenario(
        format!(
            "{}.closed.{}.{}",
            source.id,
            if direction > 0 { "forward" } else { "reverse" },
            mode.name()
        ),
        json!({"connector_id": source.id, "door_id": source.door_id, "direction": if direction > 0 { "forward" } else { "reverse" }, "access_input": false, "failure_mode": mode.name(), "candidate_only": true}),
        json!({"crossed_closed_door": false, "penetration": false}),
        json!({"crossed_closed_door": run_result.crossed_closed_door, "progress_m": run_result.progress_m, "endpoint_error_m": run_result.endpoint_error_m, "minimum_observed_clearance_m": run_result.minimum_clearance_m, "maximum_penetration_m": run_result.maximum_penetration_m, "maximum_step_m": run_result.maximum_step_m, "grounded": run_result.grounded, "penetration": run_result.penetration}),
        json!({"crossed_closed_door": false, "max_penetration_m": 0.0, "max_step_m": 0.20}),
        run_result.frames,
        pass,
    )
}

fn graph_check() -> Value {
    let v7: Value = serde_json::from_str(V7_JSON).unwrap();
    let ag35: Value = serde_json::from_str(AG35_JSON).unwrap();
    let ag36: Value = serde_json::from_str(AG36_JSON).unwrap();
    let route = v7["route_variants"]["legal_no_choice_shortest"]["edge_sequence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    let ids = ag35["connectors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    let no_edge_in_mainline = !route.contains(&"edge-b01-s02");
    let gate_a_preserved = route.contains(&"edge-a04-b01");
    let core_preserved = route.contains(&"edge-b01-b02-core-v7");
    let gate_b_preserved = route.contains(&"edge-b05-c01");
    let ag36_untouched = ag36["schema"] == "continuous_physics_static_mainline_ag36_v1";
    json!({"candidate_only": true, "inventory_connector_present": ids.contains(&"edge-b01-s02"), "not_in_legal_mainline_route": no_edge_in_mainline, "gate_a_edge_preserved": gate_a_preserved, "core_v7_edge_preserved": core_preserved, "gate_b_edge_preserved": gate_b_preserved, "bai_zhi_choice_or_npc_used": false, "ag36_read_only_reference_loaded": ag36_untouched, "formal_global_nav_claim": false, "pass": no_edge_in_mainline && gate_a_preserved && core_preserved && gate_b_preserved && !ids.is_empty() && ag36_untouched})
}

fn run_all() -> Value {
    let source = source_connector();
    let mut scenarios = Vec::new();
    for direction in [1, -1] {
        for hz in [30, 60] {
            scenarios.push(open_case(&source, direction, hz));
        }
    }
    for direction in [1, -1] {
        for mode in [
            Mode::ThroughDoor,
            Mode::CornerLeft,
            Mode::CornerRight,
            Mode::JumpOver,
            Mode::WallAround,
        ] {
            scenarios.push(closed_case(&source, direction, mode));
        }
    }
    let graph = graph_check();
    let all_pass = scenarios.iter().all(|item| item["pass"] == true) && graph["pass"] == true;
    let minimum_clearance = scenarios
        .iter()
        .filter_map(|item| item["actual"]["minimum_observed_clearance_m"].as_f64())
        .fold(f64::INFINITY, f64::min);
    let maximum_penetration = scenarios
        .iter()
        .filter_map(|item| item["actual"]["maximum_penetration_m"].as_f64())
        .fold(0.0, f64::max);
    let maximum_step = scenarios
        .iter()
        .filter_map(|item| item["actual"]["maximum_step_m"].as_f64())
        .fold(0.0, f64::max);
    json!({
        "schema": "continuous_physics_static_office_ag39_v1", "status": if all_pass { "blocked_isolated_optional_office_candidate" } else { "failed_isolated_optional_office_candidate" }, "exit_code": if all_pass { 0 } else { 1 },
        "scope": "parameterized isolated Rapier 0.22 validation of v7 edge-b01-s02; no formal Blockout/Campaign/nav/camera",
        "source": {"v7": "../layout-core-length/layout-v7.json", "base_layout": "../layout-repair/layout-v5.json", "ag35": "../results/connector-inventory-ag35.json", "ag36": "../results/physics-static-mainline-ag36.json", "candidate_only": true, "connector_id": source.id, "from": source.from, "to": source.to, "path": source.path, "door_id": source.door_id, "length_m": source.length_m, "width_m": source.width_m, "height_m": source.height_m, "nominal_net_clearance_m": [source.width_m - 2.0 * CAPSULE_RADIUS, source.height_m - CAPSULE_TOTAL_HEIGHT], "formal_net_clearance": "unknown"},
        "rapier": {"crate": "rapier3d", "version": "0.22.0", "lockfile": "tools/continuous-tps-architecture-probe/physics-static-office/Cargo.lock"},
        "contract": {"units": "metres, seconds, radians", "up": "+Y", "capsule_radius_m": CAPSULE_RADIUS, "capsule_total_height_m": CAPSULE_TOTAL_HEIGHT, "collision_offset_m": COLLISION_OFFSET, "tick_rates_hz": [30, 60], "input_seconds_same_at_30_60": true},
        "scenarios": scenarios,
        "summary": {"connector_count": 1, "open_both_directions": true, "closed_both_directions": true, "corner_left_right": true, "jump_over": true, "wall_outside_around": true, "minimum_observed_clearance_m": minimum_clearance, "maximum_penetration_m": maximum_penetration, "maximum_step_m": maximum_step, "evidence_level": "parameterized_isolated_candidate"},
        "candidate_graph_geometry_check": graph,
        "formal_blockers": ["candidate geometry is inherited from v5/v7 and has no formal Blockout collider bake", "global nav shortest-path and camera clearance remain unknown", "dynamic door timing is not modeled", "Gate C/I and overall Architecture Gate remain blocked/no_go"],
        "machine_pass": all_pass,
        "notes": ["Rapier corridor follows every source polyline segment and adds corner landings; the closed door is a full-width blocker, not graph-edge removal.", "Open/closed results use explicit access_input only; no Baizhi choice/NPC/campaign state is consulted.", "Static optional-office candidate cannot become a formal Gate B condition or shortcut based on this check."]
    })
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("physics-static-office-ag39.json")
}

#[test]
fn ag39_optional_office_static_rapier_isolation() {
    let report = run_all();
    fs::create_dir_all(result_path().parent().unwrap()).unwrap();
    fs::write(
        result_path(),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    assert_eq!(
        report["machine_pass"], true,
        "AG-39 optional office isolated physics failed: {report}"
    );
    assert_eq!(report["summary"]["connector_count"], 1);
    assert_eq!(report["candidate_graph_geometry_check"]["pass"], true);
}

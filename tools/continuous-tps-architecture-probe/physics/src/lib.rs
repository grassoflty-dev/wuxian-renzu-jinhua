use rapier3d::{
    control::{CharacterAutostep, CharacterLength, EffectiveCharacterMovement, KinematicCharacterController},
    prelude::*,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

const DT_60: f32 = 1.0 / 60.0;
const DT_30: f32 = 1.0 / 30.0;
const GRAVITY: f32 = 9.81;
const RADIUS: f32 = 0.35;
const HALF_HEIGHT: f32 = 0.75;
const FOOT_CENTER_OFFSET: f32 = HALF_HEIGHT + RADIUS;
const OFFSET: f32 = 0.01;

struct Arena {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    queries: QueryPipeline,
}

impl Arena {
    fn new() -> Self {
        Self { bodies: RigidBodySet::new(), colliders: ColliderSet::new(), queries: QueryPipeline::new() }
    }
    fn box_collider(&mut self, center: Vector<Real>, half: Vector<Real>) {
        self.colliders.insert(ColliderBuilder::cuboid(half.x, half.y, half.z).translation(center).build());
        self.queries.update(&self.colliders);
    }
    fn room(&mut self) {
        self.box_collider(vector![0.0, -0.10, 0.0], vector![5.0, 0.10, 5.0]);
        self.box_collider(vector![-5.10, 1.50, 0.0], vector![0.10, 1.50, 5.0]);
        self.box_collider(vector![5.10, 1.50, 0.0], vector![0.10, 1.50, 5.0]);
        self.box_collider(vector![0.0, 1.50, -5.10], vector![5.0, 1.50, 0.10]);
        self.box_collider(vector![0.0, 1.50, 5.10], vector![5.0, 1.50, 0.10]);
    }
    fn intersects(&self, position: &Isometry<Real>, shape: &SharedShape) -> bool {
        self.queries.intersection_with_shape(&self.bodies, &self.colliders, position, shape.as_ref(), QueryFilter::default()).is_some()
    }
}

fn character_shape() -> SharedShape { SharedShape::capsule_y(HALF_HEIGHT, RADIUS) }
fn character_position(x: f32, foot_y: f32, z: f32) -> Isometry<Real> { Isometry::translation(x, foot_y + FOOT_CENTER_OFFSET, z) }
fn foot_y(position: &Isometry<Real>) -> f32 { position.translation.vector.y - FOOT_CENTER_OFFSET }
fn vec3(value: Vector<Real>) -> [f64; 3] { [value.x as f64, value.y as f64, value.z as f64] }
fn sample_position(position: &Isometry<Real>, grounded: bool, collisions: usize) -> Value {
    json!({"position_m": vec3(position.translation.vector), "foot_y_m": foot_y(position), "grounded": grounded, "collision_count": collisions})
}

fn move_character(arena: &Arena, controller: &KinematicCharacterController, position: &mut Isometry<Real>, shape: &SharedShape, dt: f32, desired: Vector<Real>) -> (EffectiveCharacterMovement, Vec<[f64; 3]>) {
    let mut normals = Vec::new();
    let result = controller.move_shape(dt, &arena.bodies, &arena.colliders, &arena.queries, shape.as_ref(), position, desired, QueryFilter::default(), |collision| {
        // normal1 is the world-space character-facing contact normal for a
        // fixed collider; normal2 points back toward the moving capsule.
        normals.push(vec3(collision.hit.normal1.into_inner()));
    });
    position.translation.vector += result.translation;
    (result, normals)
}

fn base_controller() -> KinematicCharacterController {
    KinematicCharacterController {
        up: Vector::y_axis(), offset: CharacterLength::Absolute(OFFSET), slide: true,
        autostep: Some(CharacterAutostep { max_height: CharacterLength::Absolute(0.35), min_width: CharacterLength::Absolute(0.25), include_dynamic_bodies: false }),
        max_slope_climb_angle: 35.0_f32.to_radians(), min_slope_slide_angle: 40.0_f32.to_radians(),
        snap_to_ground: Some(CharacterLength::Absolute(0.20)), normal_nudge_factor: 1.0e-4,
    }
}

fn result(name: &str, inputs: Value, key_frames: Vec<Value>, expected: Value, actual: Value, threshold: Value, pass: bool) -> Value {
    json!({"name": name, "inputs": inputs, "key_frames": key_frames, "expected": expected, "actual": actual, "threshold": threshold, "pass": pass})
}

fn grounded() -> Value {
    let mut arena = Arena::new(); arena.room();
    let shape = character_shape(); let controller = base_controller(); let mut position = character_position(0.0, OFFSET, 0.0);
    let mut grounded = false; let mut normals = Vec::new();
    for tick in 0..8 { let desired = if tick == 0 { vector![0.0, -0.20, 0.0] } else { vector![0.0, 0.0, 0.0] }; let (step, hits) = move_character(&arena, &controller, &mut position, &shape, DT_60, desired); grounded = step.grounded; normals.extend(hits); }
    let penetration = arena.intersects(&position, &shape);
    let contact_grounded = foot_y(&position).abs() <= 0.02 && normals.iter().any(|normal| normal[1] > 0.90);
    let pass = contact_grounded && !penetration;
    result("grounded_floor_contact", json!({"room_m": [10.0, 10.0], "start_foot_m": [0.0, 0.0], "dt_s": DT_60}), vec![sample_position(&position, contact_grounded, normals.len())], json!({"grounded": true, "foot_y_m": 0.0, "penetration": false, "ground_normal": [0.0, 1.0, 0.0]}), json!({"kcc_grounded": grounded, "contact_grounded": contact_grounded, "foot_y_m": foot_y(&position), "penetration": penetration, "collision_normals": normals}), json!({"foot_y_abs_m": 0.02, "ground_normal_y_min": 0.90}), pass)
}

fn wall_and_slide() -> (Value, Value) {
    let mut arena = Arena::new(); arena.room(); arena.box_collider(vector![2.0, 1.0, 0.0], vector![0.10, 1.0, 4.0]);
    let shape = character_shape(); let controller = base_controller();
    let mut frontal = character_position(0.0, 0.0, 0.0);
    let (frontal_step, frontal_normals) = move_character(&arena, &controller, &mut frontal, &shape, DT_60, vector![4.0, 0.0, 0.0]);
    let frontal_intersection = arena.intersects(&frontal, &shape);
    let frontal_pass = frontal.translation.x <= 2.0 - 0.10 - RADIUS + 0.03 && !frontal_intersection;
    let frontal_result = result("frontal_wall_stop", json!({"wall_center_x_m": 2.0, "wall_half_x_m": 0.10, "desired_translation_m": [4.0, 0.0, 0.0], "dt_s": DT_60}), vec![sample_position(&frontal, frontal_step.grounded, frontal_normals.len())], json!({"x_m": "stops before wall", "penetration": false, "normal": [-1.0, 0.0, 0.0]}), json!({"position_m": vec3(frontal.translation.vector), "penetration": frontal_intersection, "collision_normals": frontal_normals}), json!({"max_x_m": 1.58}), frontal_pass);

    let mut sliding = character_position(0.0, 0.0, 0.0);
    let (sliding_step, sliding_normals) = move_character(&arena, &controller, &mut sliding, &shape, DT_60, vector![4.0, 0.0, 3.0]);
    let sliding_intersection = arena.intersects(&sliding, &shape);
    let sliding_pass = sliding.translation.x <= 1.58 && sliding.translation.z > 2.0 && !sliding_intersection;
    let sliding_result = result("diagonal_wall_slide", json!({"wall_center_x_m": 2.0, "desired_translation_m": [4.0, 0.0, 3.0], "dt_s": DT_60}), vec![sample_position(&sliding, sliding_step.grounded, sliding_normals.len())], json!({"x_m": "stops before wall", "z_m": "continues along wall", "penetration": false}), json!({"position_m": vec3(sliding.translation.vector), "penetration": sliding_intersection, "collision_normals": sliding_normals}), json!({"max_x_m": 1.58, "min_z_m": 2.0}), sliding_pass);
    (frontal_result, sliding_result)
}

fn doorway() -> (Value, Value) {
    let mut arena = Arena::new(); arena.room();
    arena.box_collider(vector![0.0, 1.5, -3.0], vector![0.10, 1.5, 2.0]);
    arena.box_collider(vector![0.0, 1.5, 3.0], vector![0.10, 1.5, 2.0]);
    let shape = character_shape(); let controller = base_controller();
    let mut open = character_position(-3.0, 0.0, 0.0);
    let (open_step, open_normals) = move_character(&arena, &controller, &mut open, &shape, DT_60, vector![6.0, 0.0, 0.0]);
    let open_intersection = arena.intersects(&open, &shape); let open_pass = open.translation.x > 2.0 && !open_intersection;
    let open_result = result("doorway_pass", json!({"door_center_x_m": 0.0, "opening_z_m": [-1.0, 1.0], "start_m": [-3.0, 0.0, 0.0], "desired_translation_m": [6.0, 0.0, 0.0]}), vec![sample_position(&open, open_step.grounded, open_normals.len())], json!({"crosses_wall": true, "penetration": false}), json!({"position_m": vec3(open.translation.vector), "penetration": open_intersection, "collision_normals": open_normals}), json!({"min_x_m": 2.0}), open_pass);

    let mut blocked = character_position(-3.0, 0.0, 1.50);
    let (blocked_step, blocked_normals) = move_character(&arena, &controller, &mut blocked, &shape, DT_60, vector![6.0, 0.0, 0.0]);
    let blocked_intersection = arena.intersects(&blocked, &shape); let blocked_pass = blocked.translation.x < -0.40 && !blocked_intersection;
    let blocked_result = result("doorframe_block", json!({"door_center_x_m": 0.0, "start_m": [-3.0, 0.0, 1.50], "desired_translation_m": [6.0, 0.0, 0.0]}), vec![sample_position(&blocked, blocked_step.grounded, blocked_normals.len())], json!({"crosses_wall": false, "penetration": false}), json!({"position_m": vec3(blocked.translation.vector), "penetration": blocked_intersection, "collision_normals": blocked_normals}), json!({"max_x_m": -0.40}), blocked_pass);
    (open_result, blocked_result)
}

fn steps() -> (Value, Value) {
    fn run(height: f32) -> (Isometry<Real>, bool, Vec<[f64; 3]>, bool) {
        let mut arena = Arena::new(); arena.room(); arena.box_collider(vector![1.5, height / 2.0, 0.0], vector![0.30, height / 2.0, 1.5]);
        let shape = character_shape(); let controller = base_controller(); let mut position = character_position(0.0, 0.0, 0.0);
        let (step, normals) = move_character(&arena, &controller, &mut position, &shape, DT_60, vector![3.0, 0.0, 0.0]);
        (position, step.grounded, normals, arena.intersects(&position, &shape))
    }
    let (small, small_grounded, small_normals, small_penetration) = run(0.30);
    let small_pass = small.translation.x > 2.0 && foot_y(&small) > 0.20 && !small_penetration;
    let small_result = result("small_step_autostep", json!({"step_height_m": 0.30, "autostep_max_height_m": 0.35, "desired_translation_m": [3.0, 0.0, 0.0]}), vec![sample_position(&small, small_grounded, small_normals.len())], json!({"crosses": true, "foot_y_m": "> 0.20", "penetration": false}), json!({"position_m": vec3(small.translation.vector), "foot_y_m": foot_y(&small), "penetration": small_penetration, "collision_normals": small_normals}), json!({"min_x_m": 2.0, "min_foot_y_m": 0.20}), small_pass);
    let (large, large_grounded, large_normals, large_penetration) = run(0.80);
    let large_pass = large.translation.x < 1.20 && !large_penetration;
    let large_result = result("large_step_block", json!({"step_height_m": 0.80, "autostep_max_height_m": 0.35, "desired_translation_m": [3.0, 0.0, 0.0]}), vec![sample_position(&large, large_grounded, large_normals.len())], json!({"crosses": false, "penetration": false}), json!({"position_m": vec3(large.translation.vector), "foot_y_m": foot_y(&large), "penetration": large_penetration, "collision_normals": large_normals}), json!({"max_x_m": 1.20}), large_pass);
    (small_result, large_result)
}

fn wedge(arena: &mut Arena, rise: f32) {
    let vertices = vec![point![0.0, 0.0, -2.0], point![4.0, 0.0, -2.0], point![4.0, 0.0, 2.0], point![0.0, 0.0, 2.0], point![0.0, 0.0, -2.0], point![4.0, rise, -2.0], point![4.0, rise, 2.0], point![0.0, 0.0, 2.0]];
    let indices = vec![[0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7], [0, 1, 5], [0, 5, 4], [1, 2, 6], [1, 6, 5], [2, 3, 7], [2, 7, 6], [3, 0, 4], [3, 4, 7]];
    let _ = indices;
    arena.colliders.insert(ColliderBuilder::convex_hull(&vertices).expect("ramp convex hull").build()); arena.queries.update(&arena.colliders);
}

fn slope_tests() -> (Value, Value) {
    fn run(angle_degrees: f32) -> (Isometry<Real>, bool, Vec<[f64; 3]>, bool) {
        let mut arena = Arena::new(); arena.room(); let rise = (angle_degrees.to_radians().tan() * 4.0).min(5.0); wedge(&mut arena, rise);
        let shape = character_shape(); let controller = base_controller(); let mut position = character_position(-1.5, 0.0, 0.0);
        let (step, normals) = move_character(&arena, &controller, &mut position, &shape, DT_60, vector![2.8, 0.0, 0.0]);
        (position, step.grounded, normals, arena.intersects(&position, &shape))
    }
    let (shallow, shallow_grounded, shallow_normals, shallow_penetration) = run(20.0);
    let shallow_pass = shallow.translation.x > 0.8 && !shallow_normals.iter().any(|n| n[1] < 0.7) && !shallow_penetration;
    let shallow_result = result("slope_20deg_climb", json!({"slope_degrees": 20.0, "max_slope_climb_degrees": 35.0, "desired_translation_m": [2.8, 0.0, 0.0]}), vec![sample_position(&shallow, shallow_grounded, shallow_normals.len())], json!({"crosses": true, "ground_normal_y": "> 0.7", "penetration": false}), json!({"position_m": vec3(shallow.translation.vector), "grounded": shallow_grounded, "penetration": shallow_penetration, "collision_normals": shallow_normals}), json!({"min_x_m": 0.8, "min_ground_normal_y": 0.7}), shallow_pass);
    let (steep, steep_grounded, steep_normals, steep_penetration) = run(50.0);
    let steep_pass = steep.translation.x < 0.60 && !steep_normals.iter().any(|n| n[1] > 0.7) && !steep_penetration;
    let steep_result = result("slope_50deg_rejected", json!({"slope_degrees": 50.0, "max_slope_climb_degrees": 35.0, "desired_translation_m": [2.8, 0.0, 0.0]}), vec![sample_position(&steep, steep_grounded, steep_normals.len())], json!({"crosses": false, "ground_normal_y": "not a walkable floor", "penetration": false}), json!({"position_m": vec3(steep.translation.vector), "grounded": steep_grounded, "penetration": steep_penetration, "collision_normals": steep_normals}), json!({"max_x_m": 0.60, "max_ground_normal_y": 0.7}), steep_pass);
    (shallow_result, steep_result)
}

fn jump() -> Value {
    let mut arena = Arena::new(); arena.room(); arena.box_collider(vector![0.0, 2.7, 0.0], vector![4.0, 0.1, 4.0]);
    let shape = character_shape(); let controller = base_controller(); let mut position = character_position(0.0, 0.0, 0.0); let mut velocity = 5.5_f32;
    let mut head_hit = false; let mut landed = false; let mut apex = foot_y(&position); let mut frames = Vec::new();
    for tick in 0..120 {
        let desired_y = velocity * DT_60; let before = position;
        let (step, normals) = move_character(&arena, &controller, &mut position, &shape, DT_60, vector![0.0, desired_y, 0.0]);
        if desired_y > 0.0 && step.translation.y < desired_y - 0.002 { head_hit = true; velocity = 0.0; } else { velocity -= GRAVITY * DT_60; }
        apex = apex.max(foot_y(&position)); if step.grounded && tick > 2 && velocity <= 0.0 { landed = true; }
        if tick < 5 || head_hit || landed || tick % 10 == 0 { frames.push(json!({"tick": tick, "before_y_m": foot_y(&before), "after_y_m": foot_y(&position), "velocity_y_mps": velocity, "translation_y_m": step.translation.y, "grounded": step.grounded, "collision_normals": normals})); }
        if landed && tick > 10 { break; }
    }
    let penetration = arena.intersects(&position, &shape); let pass = apex > 0.35 && head_hit && landed && !penetration;
    result("jump_apex_land_and_ceiling", json!({"dt_s": DT_60, "initial_vertical_velocity_mps": 5.5, "gravity_mps2": GRAVITY, "ceiling_underside_y_m": 2.6}), frames, json!({"apex_foot_y_m": "> 0.35", "head_collision": true, "landed": true, "penetration": false}), json!({"apex_foot_y_m": apex, "head_collision": head_hit, "landed": landed, "final_position_m": vec3(position.translation.vector), "penetration": penetration}), json!({"min_apex_foot_y_m": 0.35}), pass)
}

fn dash() -> (Value, Value) {
    let mut clear_arena = Arena::new(); clear_arena.room(); let shape = character_shape(); let controller = base_controller(); let mut clear = character_position(-3.0, 0.0, 0.0);
    let (clear_step, clear_normals) = move_character(&clear_arena, &controller, &mut clear, &shape, DT_60, vector![5.0, 0.0, 0.0]); let clear_intersection = clear_arena.intersects(&clear, &shape); let clear_pass = clear.translation.x > 1.9 && !clear_intersection;
    let clear_result = result("dash_clear_path", json!({"start_m": [-3.0, 0.0, 0.0], "dash_translation_m": [5.0, 0.0, 0.0], "dt_s": DT_60}), vec![sample_position(&clear, clear_step.grounded, clear_normals.len())], json!({"reaches_target": true, "penetration": false}), json!({"position_m": vec3(clear.translation.vector), "penetration": clear_intersection, "collision_normals": clear_normals}), json!({"min_x_m": 1.9}), clear_pass);

    let mut wall_arena = Arena::new(); wall_arena.room(); wall_arena.box_collider(vector![0.0, 1.5, 0.0], vector![0.10, 1.5, 4.0]); let mut wall = character_position(-3.0, 0.0, 0.0);
    let (wall_step, wall_normals) = move_character(&wall_arena, &controller, &mut wall, &shape, DT_60, vector![6.0, 0.0, 0.0]); let wall_intersection = wall_arena.intersects(&wall, &shape); let wall_pass = wall.translation.x <= -0.46 && !wall_intersection;
    let wall_result = result("dash_wall_sweep", json!({"start_m": [-3.0, 0.0, 0.0], "wall_x_m": 0.0, "dash_translation_m": [6.0, 0.0, 0.0], "dt_s": DT_60}), vec![sample_position(&wall, wall_step.grounded, wall_normals.len())], json!({"reaches_target": false, "stops_before_wall": true, "penetration": false}), json!({"position_m": vec3(wall.translation.vector), "penetration": wall_intersection, "collision_normals": wall_normals}), json!({"max_x_m": -0.46}), wall_pass);
    (clear_result, wall_result)
}

fn timestep_compare() -> Value {
    fn run(dt: f32) -> (Isometry<Real>, Vec<Value>) {
        let mut arena = Arena::new(); arena.room(); let shape = character_shape(); let controller = base_controller(); let mut position = character_position(-3.0, 0.0, 0.0); let mut frames = Vec::new(); let mut elapsed = 0.0;
        while elapsed < 1.0 - 1e-6 {
            let (step, normals) = move_character(&arena, &controller, &mut position, &shape, dt, vector![2.0 * dt, 0.0, 0.0]); elapsed += dt;
            if frames.len() < 2 || elapsed > 0.99 { frames.push(json!({"t_s": elapsed, "position_m": vec3(position.translation.vector), "grounded": step.grounded, "collision_normals": normals})); }
        }
        (position, frames)
    }
    let (at_30, frames_30) = run(DT_30); let (at_60, frames_60) = run(DT_60); let delta = (at_30.translation.vector - at_60.translation.vector).norm();
    result("fixed_input_30_vs_60_hz", json!({"velocity_mps": [2.0, 0.0, 0.0], "duration_s": 1.0, "dt_30_s": DT_30, "dt_60_s": DT_60}), vec![json!({"hz": 30, "frames": frames_30}), json!({"hz": 60, "frames": frames_60})], json!({"same_input": true, "position_delta_m": "<= 0.02"}), json!({"final_30_m": vec3(at_30.translation.vector), "final_60_m": vec3(at_60.translation.vector), "position_delta_m": delta}), json!({"max_position_delta_m": 0.02}), delta <= 0.02)
}

fn run_all() -> Value {
    let (wall, slide) = wall_and_slide(); let (door, frame) = doorway(); let (small_step, large_step) = steps(); let (slope_pass, slope_steep) = slope_tests(); let (dash_clear, dash_wall) = dash();
    let scenarios = vec![grounded(), wall, slide, door, frame, small_step, large_step, slope_pass, slope_steep, jump(), dash_clear, dash_wall, timestep_compare()];
    let all_pass = scenarios.iter().all(|item| item["pass"] == true);
    json!({
        "schema": "continuous_physics_probe_ag06_v1", "status": if all_pass { "isolated_poc_passed" } else { "isolated_poc_failed" }, "exit_code": if all_pass { 0 } else { 1 },
        "rapier": {"crate": "rapier3d", "version": "0.22.0", "lockfile": "tools/continuous-tps-architecture-probe/physics/Cargo.lock"},
        "platform_contract": {"units": "metres, seconds, radians", "up": "+Y", "foot_anchor": "capsule bottom at y=0", "gravity_mps2": GRAVITY, "capsule_radius_m": RADIUS, "capsule_half_segment_m": HALF_HEIGHT, "capsule_total_height_m": 2.0 * (HALF_HEIGHT + RADIUS), "tick_rates_hz": [30, 60], "slope_climb_degrees": 35.0, "autostep_max_height_m": 0.35, "autostep_min_width_m": 0.25, "collision_offset_m": OFFSET},
        "room_contract": {"size_m": [10.0, 10.0], "perimeter_height_m": 3.0, "floor_y_m": 0.0},
        "scenarios": scenarios,
        "notes": ["Static colliders only; no Campaign/Tauri/TPS integration.", "Dash uses one swept KCC move_shape call, not a teleport.", "All positions and thresholds are measured in the isolated Rapier harness."]
    })
}

fn result_path() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("results").join("physics-ag06.json") }

#[test]
fn ag06_rapier_kcc_isolated_poc() {
    let report = run_all();
    fs::create_dir_all(result_path().parent().unwrap()).unwrap();
    fs::write(result_path(), format!("{}\n", serde_json::to_string_pretty(&report).unwrap())).unwrap();
    assert_eq!(report["exit_code"], 0, "AG-06 isolated physics scenarios failed: {report}");
}

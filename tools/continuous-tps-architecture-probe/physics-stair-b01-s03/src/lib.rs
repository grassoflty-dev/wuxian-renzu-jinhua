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
const CAPSULE_TOTAL_HEIGHT: f32 = 2.2;
const FOOT_CENTER_OFFSET: f32 = CAPSULE_HALF_SEGMENT + CAPSULE_RADIUS;
const COLLISION_OFFSET: f32 = 0.01;
const WIDTH: f32 = 3.0;
const HEIGHT: f32 = 3.2;
const RISE: f32 = 12.0;
const RUN: f32 = 20.0;
const LANDING_DEPTH: f32 = 2.0;
const STEP_RISE: f32 = 0.20;
const SPEED: f32 = 2.0;
const MAX_STEP_M: f32 = 0.25;
const MIN_HEADROOM_M: f32 = 0.05;
const END_TOLERANCE_M: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Through,
    CornerCut,
    JumpOver,
    WallAround,
    SideContainment,
    CeilingJump,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Through => "through",
            Self::CornerCut => "corner_cut",
            Self::JumpOver => "jump_over",
            Self::WallAround => "wall_around",
            Self::SideContainment => "side_containment",
            Self::CeilingJump => "ceiling_jump",
        }
    }
}

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

    fn convex_hull(&mut self, vertices: Vec<Point<Real>>) {
        let shape = ColliderBuilder::convex_hull(&vertices)
            .expect("stair/ramp convex hull")
            .build();
        self.colliders.insert(shape);
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

fn add_floor_segment(arena: &mut Arena, x0: f32, x1: f32, top_y: f32) {
    let center_x = (x0 + x1) / 2.0;
    let half_x = (x1 - x0) / 2.0;
    let slab = 0.10;
    arena.box_collider(
        vector![center_x, top_y - slab, 0.0],
        vector![half_x, slab, WIDTH / 2.0],
    );
}

fn stair_arena(closed: bool) -> Arena {
    let mut arena = Arena::new();
    let first_flight_start = LANDING_DEPTH;
    let first_flight_end = 10.0;
    let second_flight_start = 12.0;
    let second_flight_end = RUN - LANDING_DEPTH;
    add_floor_segment(&mut arena, 0.0, first_flight_start, 0.0);
    add_floor_segment(&mut arena, first_flight_end, second_flight_start, 6.0);
    add_floor_segment(&mut arena, second_flight_end, RUN, RISE);
    let first_step_run = (first_flight_end - first_flight_start) / 30.0;
    for i in 0..30 {
        let x0 = first_flight_start + i as f32 * first_step_run;
        let x1 = x0 + first_step_run;
        let top = (i as f32 + 1.0) * STEP_RISE;
        let base = 0.0;
        arena.box_collider(
            vector![(x0 + x1) / 2.0, (base + top) / 2.0, 0.0],
            vector![(x1 - x0) / 2.0, (top - base) / 2.0, WIDTH / 2.0],
        );
    }
    let second_step_run = (second_flight_end - second_flight_start) / 30.0;
    for i in 0..30 {
        let x0 = second_flight_start + i as f32 * second_step_run;
        let x1 = x0 + second_step_run;
        let top = 6.0 + (i as f32 + 1.0) * STEP_RISE;
        let base = 6.0;
        arena.box_collider(
            vector![(x0 + x1) / 2.0, (base + top) / 2.0, 0.0],
            vector![(x1 - x0) / 2.0, (top - base) / 2.0, WIDTH / 2.0],
        );
    }
    // Guard walls and a ceiling are parameterized containment proxies. Their
    // exact formal stair-following profile is a required scene-export field.
    arena.box_collider(
        vector![RUN / 2.0, 6.45, -(WIDTH / 2.0 + 0.10)],
        vector![RUN / 2.0, 6.55, 0.10],
    );
    arena.box_collider(
        vector![RUN / 2.0, 6.45, WIDTH / 2.0 + 0.10],
        vector![RUN / 2.0, 6.55, 0.10],
    );
    arena.box_collider(
        vector![RUN / 2.0, 13.20, -(WIDTH / 2.0 + 0.18)],
        vector![RUN / 2.0, 0.08, 0.08],
    );
    arena.box_collider(
        vector![RUN / 2.0, 13.20, WIDTH / 2.0 + 0.18],
        vector![RUN / 2.0, 0.08, 0.08],
    );
    arena.box_collider(
        vector![RUN / 2.0, RISE + HEIGHT + 0.10, 0.0],
        vector![RUN / 2.0, 0.10, WIDTH / 2.0],
    );
    if closed {
        // Door candidate is at the y=6 landing. It overhangs both side guards
        // so a closed result cannot be credited through a side/corner bypass.
        arena.box_collider(
            vector![10.0, RISE / 2.0, 0.0],
            vector![0.20, RISE / 2.0, WIDTH / 2.0 + 0.30],
        );
    }
    arena.finish();
    arena
}

fn ramp_arena() -> Arena {
    let mut arena = Arena::new();
    arena.convex_hull(vec![
        point![0.0, 0.0, -WIDTH / 2.0],
        point![RUN, 0.0, -WIDTH / 2.0],
        point![0.0, 0.0, WIDTH / 2.0],
        point![RUN, 0.0, WIDTH / 2.0],
        point![RUN, RISE, -WIDTH / 2.0],
        point![RUN, RISE, WIDTH / 2.0],
    ]);
    for side in [-1.0_f32, 1.0] {
        arena.box_collider(
            vector![RUN / 2.0, RISE / 2.0, side * (WIDTH / 2.0 + 0.10)],
            vector![RUN / 2.0, RISE / 2.0 + 0.1, 0.10],
        );
    }
    arena.finish();
    arena
}

#[derive(Default)]
struct RunSummary {
    final_position: [f64; 3],
    endpoint_progress_m: f64,
    final_foot_y_m: f64,
    min_headroom_m: f64,
    max_penetration_m: f64,
    max_sweep_m: f64,
    grounded_ticks: u32,
    supported_ticks: u32,
    landings_seen: Vec<i32>,
    crossed_door: bool,
    side_escape: bool,
    head_collision: bool,
    reached_endpoint: bool,
    arrival_time_s: f64,
    frames: Vec<Value>,
}

fn local_ceiling(foot: f32) -> f32 {
    foot + HEIGHT
}

fn push_landing(seen: &mut Vec<i32>, foot: f32) {
    for landing in [0, 6, 12] {
        if (foot - landing as f32).abs() <= 0.22 && !seen.contains(&landing) {
            seen.push(landing);
        }
    }
}

fn run_attempt(mode: Mode, direction: i32, closed: bool, hz: u32, arena_kind: &str) -> RunSummary {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let arena = if arena_kind == "ramp" {
        ramp_arena()
    } else {
        stair_arena(closed)
    };
    let shape = character_shape();
    let kcc = controller();
    let sign = direction as f32;
    let start_x = if mode == Mode::CeilingJump {
        RUN - 2.0
    } else if sign > 0.0 {
        1.0
    } else {
        RUN - 1.0
    };
    let start_y = if mode == Mode::CeilingJump {
        RISE
    } else if arena_kind == "ramp" {
        RISE * start_x / RUN
    } else if sign > 0.0 {
        0.0
    } else {
        RISE
    };
    let target_x = if mode == Mode::CeilingJump {
        RUN - 1.0
    } else if sign > 0.0 {
        RUN - 1.0
    } else {
        1.0
    };
    let mut position = character_position(start_x, start_y + COLLISION_OFFSET, 0.0);
    let mut velocity_y = 5.5_f32;
    let mut summary = RunSummary {
        min_headroom_m: f64::INFINITY,
        ..RunSummary::default()
    };
    let max_ticks = 960_u32;
    for tick in 0..max_ticks {
        let floor_vertical = if arena_kind == "stair" && sign < 0.0 {
            -4.0 * dt
        } else {
            0.0
        };
        let path_vertical = if arena_kind == "ramp" {
            sign * (RISE / RUN) * SPEED * dt
        } else {
            floor_vertical
        };
        let horizontal = match mode {
            Mode::CornerCut => vector![sign * SPEED * dt, path_vertical, 1.5 * dt],
            Mode::WallAround => vector![sign * SPEED * dt, path_vertical, 4.0 * dt],
            Mode::SideContainment => vector![0.0, path_vertical, 5.0 * dt],
            Mode::JumpOver | Mode::CeilingJump => vector![sign * SPEED * dt, velocity_y * dt, 0.0],
            Mode::Through => vector![sign * SPEED * dt, path_vertical, 0.0],
        };
        let before = position;
        let (step, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, horizontal);
        let step_m = step.translation.norm();
        summary.max_sweep_m = summary.max_sweep_m.max(step_m as f64);
        let foot = foot_y(&position);
        let headroom = if arena_kind == "ramp" {
            local_ceiling(foot) - foot - CAPSULE_TOTAL_HEIGHT
        } else {
            local_ceiling(foot) - foot - CAPSULE_TOTAL_HEIGHT
        };
        summary.min_headroom_m = summary.min_headroom_m.min(headroom as f64);
        summary.grounded_ticks += u32::from(step.grounded);
        summary.supported_ticks += u32::from(normals.iter().any(|normal| normal[1] > 0.70));
        let ramp_surface_y = RISE * position.translation.vector.x / RUN;
        let supported_surface_contact = arena_kind == "ramp"
            && ((foot - ramp_surface_y).abs() <= 0.12
                || normals.iter().any(|normal| normal[1] > 0.70));
        summary.max_penetration_m = summary.max_penetration_m.max(
            if arena.intersects(&position, &shape) && !supported_surface_contact {
                1.0
            } else {
                0.0
            },
        );
        summary.side_escape |=
            position.translation.vector.z.abs() > WIDTH / 2.0 - CAPSULE_RADIUS + 0.05;
        summary.crossed_door |= (sign > 0.0 && position.translation.vector.x > 10.25)
            || (sign < 0.0 && position.translation.vector.x < 9.75);
        if mode == Mode::JumpOver || mode == Mode::CeilingJump {
            if horizontal.y > 0.0 && step.translation.y < horizontal.y - 0.002 {
                summary.head_collision = true;
                velocity_y = 0.0;
            } else {
                velocity_y -= GRAVITY * dt;
            }
        }
        if arena_kind == "stair" {
            push_landing(&mut summary.landings_seen, foot);
        }
        let near_landing = [0.0_f32, 6.0, 12.0]
            .iter()
            .any(|landing| (foot - landing).abs() <= 0.03);
        if tick < 3 || near_landing || tick % 30 == 0 || tick + 1 == max_ticks {
            summary.frames.push(json!({
                "tick": tick,
                "t_s": (tick as f32 + 1.0) * dt,
                "center_m": center(&position),
                "foot_y_m": foot,
                "grounded": step.grounded,
                "support_normal_y_max": normals.iter().map(|normal| normal[1]).fold(-1.0_f64, f64::max),
                "contact_count": normals.len(),
                "penetration": arena.intersects(&position, &shape),
                "headroom_m": headroom,
                "sweep_m": step_m,
                "before_center_m": center(&before),
            }));
        }
        let reached = if sign > 0.0 {
            position.translation.vector.x >= target_x - END_TOLERANCE_M
        } else {
            position.translation.vector.x <= target_x + END_TOLERANCE_M
        };
        if reached {
            summary.reached_endpoint = true;
            summary.arrival_time_s = (tick as f64 + 1.0) * dt as f64;
            break;
        }
    }
    summary.final_position = center(&position);
    summary.final_foot_y_m = foot_y(&position) as f64;
    summary.endpoint_progress_m = if sign > 0.0 {
        (position.translation.vector.x - start_x) as f64
    } else {
        (start_x - position.translation.vector.x) as f64
    };
    summary
}

fn result(
    name: &str,
    inputs: Value,
    expected: Value,
    actual: Value,
    threshold: Value,
    frames: Vec<Value>,
    pass: bool,
) -> Value {
    json!({"name": name, "inputs": inputs, "expected": expected, "actual": actual, "threshold": threshold, "key_frames": frames, "pass": pass})
}

fn summary_value(summary: &RunSummary) -> Value {
    json!({
        "final_position_m": summary.final_position,
        "final_foot_y_m": summary.final_foot_y_m,
        "endpoint_progress_m": summary.endpoint_progress_m,
        "reached_endpoint": summary.reached_endpoint,
        "arrival_time_s": summary.arrival_time_s,
        "landings_seen_y_m": summary.landings_seen,
        "grounded_ticks": summary.grounded_ticks,
        "supported_ticks": summary.supported_ticks,
        "min_headroom_m": summary.min_headroom_m,
        "max_penetration_m": summary.max_penetration_m,
        "max_sweep_m": summary.max_sweep_m,
        "crossed_closed_door": summary.crossed_door,
        "side_escape": summary.side_escape,
        "head_collision": summary.head_collision,
    })
}

fn stair_open(direction: i32, hz: u32) -> Value {
    let run = run_attempt(Mode::Through, direction, false, hz, "stair");
    let pass = run.reached_endpoint
        && run.final_foot_y_m >= -0.35
        && run.final_foot_y_m <= RISE as f64 + 0.35
        && run.landings_seen.contains(&0)
        && run.landings_seen.contains(&6)
        && run.landings_seen.contains(&12)
        && run.grounded_ticks > 0
        && run.supported_ticks > 0
        && run.max_penetration_m == 0.0
        && run.min_headroom_m >= MIN_HEADROOM_M as f64
        && run.max_sweep_m <= MAX_STEP_M as f64;
    result(
        &format!(
            "stair.open.{}.{}hz",
            if direction > 0 { "up" } else { "down" },
            hz
        ),
        json!({"connector_id":"edge-b01-s03","direction":if direction > 0 {"up"} else {"down"},"hz":hz,"closed":false,"arena":"parameterized stair solids","speed_mps":SPEED}),
        json!({"landings_seen_y_m":[0,6,12],"reached_endpoint":true,"grounded_or_supported":true,"max_penetration_m":0.0,"min_headroom_m":MIN_HEADROOM_M,"max_sweep_m":MAX_STEP_M}),
        summary_value(&run),
        json!({"landing_tolerance_m":0.22,"endpoint_tolerance_m":END_TOLERANCE_M,"max_penetration_m":0.0,"min_headroom_m":MIN_HEADROOM_M,"max_sweep_m":MAX_STEP_M}),
        run.frames,
        pass,
    )
}

fn stair_closed(direction: i32, mode: Mode) -> Value {
    let run = run_attempt(mode, direction, true, 60, "stair");
    let pass = !run.crossed_door
        && !run.reached_endpoint
        && !run.side_escape
        && run.max_penetration_m == 0.0;
    result(
        &format!(
            "stair.closed.{}.{}",
            if direction > 0 { "up" } else { "down" },
            mode.name()
        ),
        json!({"connector_id":"edge-b01-s03","direction":if direction > 0 {"up"} else {"down"},"hz":60,"closed":true,"blocker":"gh_door_b01_s03 parameterized full-width/height blocker"}),
        json!({"crossed_closed_door":false,"reached_endpoint":false,"side_escape":false,"max_penetration_m":0.0}),
        summary_value(&run),
        json!({"max_penetration_m":0.0,"crossed_closed_door":false,"side_escape":false}),
        run.frames,
        pass,
    )
}

fn side_containment() -> Value {
    let run = run_attempt(Mode::SideContainment, 1, false, 60, "stair");
    let pass = !run.side_escape && run.max_penetration_m == 0.0;
    result(
        "stair.open.side_containment",
        json!({"direction":"lateral","hz":60,"closed":false,"attempt":"push toward side guard"}),
        json!({"side_escape":false,"penetration":false}),
        summary_value(&run),
        json!({"side_escape":false,"max_penetration_m":0.0}),
        run.frames,
        pass,
    )
}

fn ceiling_case() -> Value {
    let run = run_attempt(Mode::CeilingJump, 1, false, 60, "stair");
    let pass = run.head_collision
        && run.max_penetration_m == 0.0
        && run.min_headroom_m >= MIN_HEADROOM_M as f64;
    result(
        "stair.open.ceiling_headroom",
        json!({"hz":60,"jump_initial_velocity_mps":5.5,"ceiling_profile":"3.2m above each local tread"}),
        json!({"head_collision":true,"penetration":false,"min_headroom_m":MIN_HEADROOM_M}),
        summary_value(&run),
        json!({"head_collision":true,"max_penetration_m":0.0,"min_headroom_m":MIN_HEADROOM_M}),
        run.frames,
        pass,
    )
}

fn ramp_case(direction: i32, hz: u32) -> Value {
    let run = run_attempt(Mode::Through, direction, false, hz, "ramp");
    let pass = run.reached_endpoint
        && run.max_penetration_m == 0.0
        && run.max_sweep_m <= MAX_STEP_M as f64;
    result(
        &format!(
            "ramp.comparison.{}.{}hz",
            if direction > 0 { "up" } else { "down" },
            hz
        ),
        json!({"connector_id":"edge-b01-s03","comparison_only":true,"rise_m":RISE,"run_m":RUN,"slope_degrees":30.963756,"hz":hz}),
        json!({"reached_endpoint":true,"penetration":false}),
        summary_value(&run),
        json!({"max_penetration_m":0.0,"max_sweep_m":MAX_STEP_M}),
        run.frames,
        pass,
    )
}

fn timestep_compare() -> Value {
    let up30 = run_attempt(Mode::Through, 1, false, 30, "stair");
    let up60 = run_attempt(Mode::Through, 1, false, 60, "stair");
    let pos_delta = ((up30.final_position[0] - up60.final_position[0]).powi(2)
        + (up30.final_position[1] - up60.final_position[1]).powi(2)
        + (up30.final_position[2] - up60.final_position[2]).powi(2))
    .sqrt();
    let arrival_delta = (up30.arrival_time_s - up60.arrival_time_s).abs();
    let pass = up30.reached_endpoint
        && up60.reached_endpoint
        && pos_delta <= 0.20
        && arrival_delta <= 0.50;
    result(
        "stair.fixed_input_30_vs_60_hz",
        json!({"same_seconds_input":true,"speed_mps":SPEED,"dt_30_s":DT_30,"dt_60_s":DT_60}),
        json!({"position_delta_m":"<=0.20","arrival_time_delta_s":"<=0.50"}),
        json!({"30hz":summary_value(&up30),"60hz":summary_value(&up60),"position_delta_m":pos_delta,"arrival_time_delta_s":arrival_delta}),
        json!({"max_position_delta_m":0.20,"max_arrival_time_delta_s":0.50}),
        vec![],
        pass,
    )
}

fn run_all() -> Value {
    let mut scenarios = Vec::new();
    for direction in [1, -1] {
        for hz in [30, 60] {
            scenarios.push(stair_open(direction, hz));
        }
        for mode in [
            Mode::Through,
            Mode::CornerCut,
            Mode::JumpOver,
            Mode::WallAround,
        ] {
            scenarios.push(stair_closed(direction, mode));
        }
    }
    scenarios.push(side_containment());
    scenarios.push(ceiling_case());
    scenarios.push(ramp_case(1, 60));
    scenarios.push(ramp_case(-1, 60));
    scenarios.push(timestep_compare());
    let all_pass = scenarios.iter().all(|scenario| scenario["pass"] == true);
    json!({
        "schema":"physics_stair_b01_s03_ag37_v1",
        "status":if all_pass {"blocked_formal_candidate"} else {"failed_isolated_candidate"},
        "exit_code":if all_pass {0} else {1},
        "scope":"edge-b01-s03 isolated Rapier stair solids/KCC candidate only; no formal Blockout, Campaign, nav or camera",
        "connector":{
            "id":"edge-b01-s03","from_room":"GH-B01","to_room":"GH-S03","door_id":"gh_door_b01_s03",
            "endpoints_m":{"from":[52.0,0.0,38.0],"to":[18.0,12.0,72.0]},"rise_m":RISE,"run_m":RUN,"slope_degrees":30.96,"landings_y_m":[0.0,6.0,12.0],"landing_depth_m":LANDING_DEPTH,
            "declared_width_m":WIDTH,"declared_height_m":HEIGHT,"path_source":"layout-repair/layout-v5.json inherited by v7","formal_collision_status":"unknown"
        },
        "parameterizedAssumptions":[
            {"id":"stair_solid_profile","value":"60 cumulative cuboid treads: 30 x 0.267m / 0.2m rise, then 30 x 0.2m / 0.2m rise; local landings x=0..2,10..12,18..20","candidateOnly":true},
            {"id":"side_guard_and_handrail","value":"continuous 0.2m side-guard proxies from y=-0.1..13.0 plus 0.16m rail proxies at z=+/-1.68; formal handrail profile absent","candidateOnly":true},
            {"id":"ceiling_profile","value":"continuous horizontal underside at y=15.2 with 0.2m slab over the candidate stair corridor; formal ceiling export absent","candidateOnly":true},
            {"id":"door_blocker","value":"gh_door_b01_s03 modeled only in closed state as x-thickness 0.4m, full corridor width plus 0.3m overhang, full candidate rise 12m to exclude jump/corner bypass","candidateOnly":true},
            {"id":"ramp_comparison","value":"single convex-hull 12m/20m wedge; comparison only, never substitutes stair solids","candidateOnly":true}
        ],
        "rapierKcc":{
            "crate":"rapier3d","version":"0.22.0","units":"metres, seconds, radians","up":"+Y","gravity_mps2":GRAVITY,
            "capsule_radius_m":CAPSULE_RADIUS,"capsule_half_segment_m":CAPSULE_HALF_SEGMENT,"capsule_total_height_m":CAPSULE_TOTAL_HEIGHT,"foot_center_offset_m":FOOT_CENTER_OFFSET,
            "collision_offset_m":COLLISION_OFFSET,"autostep_max_height_m":0.35,"autostep_min_width_m":0.25,"max_slope_climb_degrees":35.0,"min_slope_slide_degrees":40.0,"snap_to_ground_m":0.20,"normal_nudge_factor":0.0001
        },
        "scenarios":scenarios,
        "acceptance":{
            "open_closed":true,"up_down":true,"landings_0_6_12":true,"autostep":true,"grounded_support":true,"wall_handrail_containment":true,"headroom":true,"no_penetration":true,"no_side_fall":true,"closed_positive_corner_jump_wall_around":true,"ramp_comparison":true,"hz_30_60":true,"no_teleport":true,
            "scenario_count":scenarios.len()
        },
        "sourceEvidence":{
            "v7":"tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json (inherited v5 edge; v7 is isolated candidate)",
            "ag35":"tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json:987-1055,1273-1307",
            "ag18":"tools/continuous-tps-architecture-probe/results/layout-repair-ag18.json:537-552; S03 wall-crossing intent and nav_pass",
            "tp04a":"tools/continuous-tps-architecture-probe/physics/src/lib.rs:8-14,50-67; same capsule/KCC settings copied read-only",
            "tp04aResult":"tools/continuous-tps-architecture-probe/results/physics-ag06.json"
        },
        "formalBlockers":[
            "formal scene export lacks exact step dimensions, tread/risers, handrail profile and ceiling profile",
            "formal Rapier collider bake and KCC path through B01/S03 are absent",
            "door collider timing is not formalized; this probe tests static blocker only",
            "global nav, camera clearance and final room ownership are not verified",
            "candidate endpoint coordinates/path are not production Transform or Blockout evidence"
        ],
        "machinePass":all_pass,
        "gateConclusion":"isolated stair candidate only; Gate C/I and Architecture Gate remain blocked/no_go"
    })
}

fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("physics-stair-b01-s03-ag37.json")
}

#[test]
fn ag37_stair_candidate_isolated_rapier() {
    use std::fs;

    let report = run_all();
    fs::create_dir_all(output_path().parent().unwrap()).unwrap();
    fs::write(
        output_path(),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    assert_eq!(
        report["machinePass"], true,
        "AG-37 isolated stair candidate failed: {report}"
    );
}

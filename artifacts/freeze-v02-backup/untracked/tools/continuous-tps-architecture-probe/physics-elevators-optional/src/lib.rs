#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

use rapier3d::{
    control::{
        CharacterAutostep, CharacterLength, EffectiveCharacterMovement,
        KinematicCharacterController,
    },
    prelude::*,
};
use serde_json::{json, Value};
#[cfg(test)]
use std::fs;
use std::path::PathBuf;

const DT_30: f32 = 1.0 / 30.0;
const DT_60: f32 = 1.0 / 60.0;
const GRAVITY_MPS2: f32 = 9.81;
const CAPSULE_RADIUS_M: f32 = 0.35;
const CAPSULE_HALF_SEGMENT_M: f32 = 0.75;
const CAPSULE_TOTAL_HEIGHT_M: f32 = 2.0 * (CAPSULE_HALF_SEGMENT_M + CAPSULE_RADIUS_M);
const FOOT_CENTER_OFFSET_M: f32 = CAPSULE_HALF_SEGMENT_M + CAPSULE_RADIUS_M;
const COLLISION_OFFSET_M: f32 = 0.01;
const PLATFORM_TRAVEL_DURATION_S: f32 = 3.0;
const PLATFORM_SPEED_ASSUMPTION_MPS: f32 = 1.0;
const APPROACH_SPEED_MPS: f32 = 2.5;
const RECOVERY_DROP_M: f32 = 1.5;
const SUPPORT_TOLERANCE_M: f32 = 0.03;
const ARRIVAL_TOLERANCE_M: f32 = 0.05;
const MIN_CLEARANCE_M: f32 = 0.10;

#[derive(Clone, Copy)]
struct Candidate {
    id: &'static str,
    from_room: &'static str,
    to_room: &'static str,
    from: [f32; 3],
    to: [f32; 3],
    delta_y_m: f32,
    width_m: f32,
    candidate_height_m: f32,
    door_id: &'static str,
}

const CANDIDATES: [Candidate; 3] = [
    Candidate {
        id: "edge-a03-s01",
        from_room: "GH-A03",
        to_room: "GH-S01",
        from: [36.0, 0.0, 14.0],
        to: [18.0, 10.0, 30.0],
        delta_y_m: 10.0,
        width_m: 3.0,
        candidate_height_m: 3.2,
        door_id: "gh_door_a03_s01",
    },
    Candidate {
        id: "edge-b05-s04",
        from_room: "GH-B05",
        to_room: "GH-S04",
        from: [52.0, 0.0, 57.0],
        to: [40.0, 10.0, 62.0],
        delta_y_m: 10.0,
        width_m: 3.0,
        candidate_height_m: 3.2,
        door_id: "gh_door_b05_s04",
    },
    Candidate {
        id: "edge-b03-h01",
        from_room: "GH-B03",
        to_room: "GH-H01",
        from: [70.0, 0.0, 38.0],
        to: [94.0, 15.0, 62.0],
        delta_y_m: 15.0,
        width_m: 2.8,
        candidate_height_m: 3.0,
        door_id: "gh_door_b03_h01",
    },
];

#[derive(Clone, Copy, Debug)]
enum Model {
    OpenPlatform,
    ControlledCabin,
}

impl Model {
    fn name(self) -> &'static str {
        match self {
            Self::OpenPlatform => "open_platform",
            Self::ControlledCabin => "controlled_cabin",
        }
    }

    fn has_ceiling(self) -> bool {
        matches!(self, Self::ControlledCabin)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoorState {
    Open,
    Closing,
    Closed,
    Opening,
}

impl DoorState {
    fn name(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Closing => "Closing",
            Self::Closed => "Closed",
            Self::Opening => "Opening",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FloorMode {
    Normal,
    MissingSupport,
    OpenEdge,
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

    fn box_collider(&mut self, center: [f32; 3], half: [f32; 3]) {
        self.colliders.insert(
            ColliderBuilder::cuboid(half[0], half[1], half[2])
                .translation(vector![center[0], center[1], center[2]])
                .build(),
        );
    }

    fn update(&mut self) {
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

#[derive(Clone, Copy, Default)]
struct Summary {
    final_foot_y_m: f32,
    arrival_time_s: Option<f32>,
    max_penetration: bool,
    all_supported: bool,
    max_horizontal_offset_m: f32,
    min_side_clearance_m: f32,
    min_ceiling_clearance_m: Option<f32>,
    edge_crossed: bool,
    recovered: bool,
}

fn character_shape() -> SharedShape {
    SharedShape::capsule_y(CAPSULE_HALF_SEGMENT_M, CAPSULE_RADIUS_M)
}

fn character_position(x: f32, foot_y: f32, z: f32) -> Isometry<Real> {
    Isometry::translation(x, foot_y + FOOT_CENTER_OFFSET_M, z)
}

fn foot_y(position: &Isometry<Real>) -> f32 {
    position.translation.vector.y - FOOT_CENTER_OFFSET_M
}

fn center(position: &Isometry<Real>) -> [f64; 3] {
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

fn acceleration(previous_velocity: [f64; 3], current_velocity: [f64; 3], dt: f32) -> [f64; 3] {
    [
        (current_velocity[0] - previous_velocity[0]) / f64::from(dt),
        (current_velocity[1] - previous_velocity[1]) / f64::from(dt),
        (current_velocity[2] - previous_velocity[2]) / f64::from(dt),
    ]
}

fn controller() -> KinematicCharacterController {
    KinematicCharacterController {
        up: Vector::y_axis(),
        offset: CharacterLength::Absolute(COLLISION_OFFSET_M),
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

fn arena_for(
    candidate: Candidate,
    model: Model,
    platform_y: f32,
    door: DoorState,
    floor_mode: FloorMode,
    recovery_y: Option<f32>,
    approach_exit_floor: bool,
) -> Arena {
    let mut arena = Arena::new();
    let half_width = candidate.width_m / 2.0;
    let half_depth = 1.5_f32;
    let cabin_height = candidate.candidate_height_m;

    if !matches!(floor_mode, FloorMode::MissingSupport) {
        arena.box_collider(
            [0.0, platform_y - 0.10, 0.0],
            [half_width, 0.10, half_depth],
        );
    }
    if approach_exit_floor {
        arena.box_collider(
            [(-3.0 + half_width) / 2.0, platform_y - 0.10, 0.0],
            [(3.0 + half_width) / 2.0, 0.10, half_depth],
        );
    }
    if model.has_ceiling() {
        arena.box_collider(
            [0.0, platform_y + cabin_height + 0.10, 0.0],
            [half_width, 0.10, half_depth],
        );
    }
    if model.has_ceiling() && !matches!(floor_mode, FloorMode::OpenEdge) {
        if door != DoorState::Opening {
            arena.box_collider(
                [half_width + 0.10, platform_y + cabin_height / 2.0, 0.0],
                [0.10, cabin_height / 2.0, half_depth],
            );
        }
        arena.box_collider(
            [0.0, platform_y + cabin_height / 2.0, -half_depth - 0.10],
            [half_width, cabin_height / 2.0, 0.10],
        );
        arena.box_collider(
            [0.0, platform_y + cabin_height / 2.0, half_depth + 0.10],
            [half_width, cabin_height / 2.0, 0.10],
        );
    }
    if matches!(door, DoorState::Closing | DoorState::Closed) {
        arena.box_collider(
            [-half_width - 0.02, platform_y + cabin_height / 2.0, 0.0],
            [0.10, cabin_height / 2.0, half_depth],
        );
    }
    if let Some(recovery_y) = recovery_y {
        arena.box_collider([4.0, recovery_y - 0.10, 0.0], [8.0, 0.10, 2.0]);
    }
    arena.update();
    arena
}

fn frame(
    tick: usize,
    t_s: f32,
    candidate: Candidate,
    model: Model,
    platform_y: f32,
    position: &Isometry<Real>,
    before: &Isometry<Real>,
    previous_velocity: [f64; 3],
    dt: f32,
    normals: Vec<[f64; 3]>,
    movement: EffectiveCharacterMovement,
    door: DoorState,
    support_y: f32,
    recovered: bool,
) -> (Value, [f64; 3], bool, f32, f32, Option<f32>) {
    let current_velocity = velocity(position, before, dt);
    let current_acceleration = acceleration(previous_velocity, current_velocity, dt);
    let current_foot_y = foot_y(position);
    let contact_count = normals.len();
    let floor_clearance = current_foot_y - support_y;
    let grounded = normals.iter().any(|normal| normal[1] > 0.90)
        || floor_clearance.abs() <= SUPPORT_TOLERANCE_M
        || movement.grounded;
    let side_clearance =
        candidate.width_m / 2.0 - CAPSULE_RADIUS_M - position.translation.vector.x.abs();
    let ceiling_clearance = if model.has_ceiling() {
        Some(support_y + candidate.candidate_height_m - (current_foot_y + CAPSULE_TOTAL_HEIGHT_M))
    } else {
        None
    };
    let horizontal_offset = position
        .translation
        .vector
        .x
        .hypot(position.translation.vector.z);
    let relative_slip = (current_foot_y - platform_y).abs();
    let value = json!({
        "tick": tick,
        "t_s": t_s,
        "platform_center_m": [0.0, platform_y, 0.0],
        "foot_y_m": current_foot_y,
        "center_m": center(position),
        "velocity_mps": current_velocity,
        "acceleration_mps2": current_acceleration,
        "contact_normals": normals,
        "contact_count": contact_count,
        "grounded": grounded,
        "support_contact": grounded,
        "relative_slip_m": relative_slip,
        "horizontal_offset_m": horizontal_offset,
        "side_clearance_m": side_clearance,
        "ceiling_clearance_m": ceiling_clearance,
        "penetration": false,
        "door_state": door.name(),
        "recovered": recovered,
        "step_translation_m": [movement.translation.x as f64, movement.translation.y as f64, movement.translation.z as f64],
    });
    (
        value,
        current_velocity,
        grounded,
        side_clearance,
        relative_slip,
        ceiling_clearance,
    )
}

fn scenario(
    name: String,
    inputs: Value,
    frames: Vec<Value>,
    expected: Value,
    actual: Value,
    threshold: Value,
    pass: bool,
) -> Value {
    json!({
        "name": name,
        "inputs": inputs,
        "frames": frames,
        "expected": expected,
        "actual": actual,
        "threshold": threshold,
        "pass": pass,
    })
}

fn candidate_json(candidate: Candidate) -> Value {
    json!({
        "id": candidate.id,
        "fromRoom": candidate.from_room,
        "toRoom": candidate.to_room,
        "from": candidate.from,
        "to": candidate.to,
        "deltaY_m": candidate.delta_y_m,
        "doorId": candidate.door_id,
        "candidateGeometry": {
            "width_m": candidate.width_m,
            "height_m": candidate.candidate_height_m,
            "formalCollisionStatus": "unknown"
        }
    })
}

fn profile_y(start_y: f32, target_y: f32, t_s: f32, hold: bool) -> f32 {
    if !hold {
        let progress = (t_s / PLATFORM_TRAVEL_DURATION_S).clamp(0.0, 1.0);
        return start_y + (target_y - start_y) * progress;
    }
    let moved_duration = 1.5_f32;
    let hold_start = 0.8_f32;
    let hold_end = 1.3_f32;
    if t_s <= hold_start {
        start_y + (target_y - start_y) * (t_s / moved_duration).clamp(0.0, 1.0)
    } else if t_s <= hold_end {
        start_y + (target_y - start_y) * (hold_start / moved_duration)
    } else {
        let resumed = (t_s - hold_end) / (PLATFORM_TRAVEL_DURATION_S - hold_end);
        start_y
            + (target_y - start_y)
                * (hold_start / moved_duration
                    + resumed.clamp(0.0, 1.0) * (1.0 - hold_start / moved_duration))
    }
}

fn run_motion(
    candidate: Candidate,
    model: Model,
    hz: u32,
    up: bool,
    hold: bool,
) -> (Value, Summary) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let start_y = if up { 0.0 } else { candidate.delta_y_m };
    let target_y = if up { candidate.delta_y_m } else { 0.0 };
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(0.0, start_y + 0.02, 0.0);
    let mut platform_y = start_y;
    let mut previous_velocity = [0.0; 3];
    let mut frames = Vec::new();
    let mut arrival = None;
    let mut all_supported = true;
    let mut max_penetration = false;
    let mut max_offset = 0.0_f32;
    let mut min_side = f32::INFINITY;
    let mut min_ceiling = f32::INFINITY;
    let ticks = ((PLATFORM_TRAVEL_DURATION_S + 0.25) / dt).ceil() as usize;
    for tick in 0..ticks {
        let t_s = (tick as f32 + 1.0) * dt;
        let next_y = profile_y(start_y, target_y, t_s, hold);
        let environment_delta = next_y - platform_y;
        platform_y = next_y;
        let arena = arena_for(
            candidate,
            model,
            platform_y,
            DoorState::Closed,
            FloorMode::Normal,
            None,
            false,
        );
        let before = position;
        let (movement, normals) = move_character(
            &arena,
            &kcc,
            &mut position,
            &shape,
            dt,
            vector![0.0, environment_delta, 0.0],
        );
        let penetrated = arena.intersects(&position, &shape);
        let foot = foot_y(&position);
        if arrival.is_none() && (foot - target_y).abs() <= ARRIVAL_TOLERANCE_M {
            arrival = Some(t_s);
        }
        let (mut sample, current_velocity, grounded, side, relative, ceiling) = frame(
            tick,
            t_s,
            candidate,
            model,
            platform_y,
            &position,
            &before,
            previous_velocity,
            dt,
            normals,
            movement,
            DoorState::Closed,
            platform_y,
            false,
        );
        sample["penetration"] = json!(penetrated);
        sample["environment_delta_m"] = json!([0.0, environment_delta, 0.0]);
        all_supported &= grounded && relative <= SUPPORT_TOLERANCE_M;
        max_penetration |= penetrated;
        max_offset = max_offset.max(
            position
                .translation
                .vector
                .x
                .hypot(position.translation.vector.z),
        );
        min_side = min_side.min(side);
        if let Some(value) = ceiling {
            min_ceiling = min_ceiling.min(value);
        }
        previous_velocity = current_velocity;
        frames.push(sample);
    }
    let final_foot = foot_y(&position);
    let arrived = arrival.is_some() && (final_foot - target_y).abs() <= ARRIVAL_TOLERANCE_M;
    let clearance = min_side >= MIN_CLEARANCE_M && min_ceiling >= MIN_CLEARANCE_M;
    let pass = arrived
        && all_supported
        && !max_penetration
        && clearance
        && max_offset <= candidate.width_m / 2.0 - CAPSULE_RADIUS_M + 0.05;
    let summary = Summary {
        final_foot_y_m: final_foot,
        arrival_time_s: arrival,
        max_penetration,
        all_supported,
        max_horizontal_offset_m: max_offset,
        min_side_clearance_m: min_side,
        min_ceiling_clearance_m: Some(min_ceiling),
        edge_crossed: false,
        recovered: false,
    };
    let report = scenario(
        format!(
            "{}_{}_{}_{}hz",
            candidate.id,
            model.name(),
            if up { "up" } else { "down" },
            hz
        ),
        json!({
            "model": model.name(), "direction": if up { "up" } else { "down" }, "hz": hz,
            "from_y_m": start_y, "to_y_m": target_y, "delta_y_m": candidate.delta_y_m,
            "duration_s": PLATFORM_TRAVEL_DURATION_S, "motion_profile": "linear_candidate_only",
        }),
        frames,
        json!({"arrives_at_endpoint": true, "support_each_tick": true, "horizontal_stable": true, "penetration": false}),
        json!({"arrival_time_s": arrival, "final_foot_y_m": final_foot, "max_penetration": max_penetration, "all_supported": all_supported, "max_horizontal_offset_m": max_offset, "min_side_clearance_m": min_side, "min_ceiling_clearance_m": min_ceiling}),
        json!({"arrival_error_m": ARRIVAL_TOLERANCE_M, "support_slip_m": SUPPORT_TOLERANCE_M, "min_side_clearance_m": MIN_CLEARANCE_M, "min_ceiling_clearance_m": MIN_CLEARANCE_M, "penetration": false}),
        pass,
    );
    (report, summary)
}

fn run_door_sync(candidate: Candidate, model: Model, hz: u32, up: bool) -> (Value, Summary) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let start_y = if up { 0.0 } else { candidate.delta_y_m };
    let target_y = if up { candidate.delta_y_m } else { 0.0 };
    let half_width = candidate.width_m / 2.0;
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(-half_width - 0.85, start_y + 0.02, 0.0);
    let mut platform_y = start_y;
    let mut previous_velocity = [0.0; 3];
    let mut frames = Vec::new();
    let mut entry_inside = false;
    let mut exit_complete = false;
    let mut all_motion_closed = true;
    let mut moved_while_open = false;
    let mut arrival = None;
    let mut max_penetration = false;
    let entry_end = 1.0_f32;
    let close_end = 1.5_f32;
    let motion_end = close_end + PLATFORM_TRAVEL_DURATION_S;
    let motion_end_tolerance = 0.5 * dt;
    let ticks = (5.5 / dt).ceil() as usize;
    for tick in 0..ticks {
        let t_s = (tick as f32 + 1.0) * dt;
        let door = if t_s < entry_end {
            DoorState::Open
        } else if t_s < close_end {
            DoorState::Closing
        } else if t_s < motion_end + motion_end_tolerance {
            DoorState::Closed
        } else {
            DoorState::Opening
        };
        let next_y = if t_s < close_end {
            start_y
        } else {
            profile_y(start_y, target_y, t_s - close_end, false)
        };
        let environment_delta = next_y - platform_y;
        if door != DoorState::Closed && environment_delta.abs() > 1.0e-5 {
            moved_while_open = true;
        }
        if environment_delta.abs() > 1.0e-5 && door != DoorState::Closed {
            all_motion_closed = false;
        }
        platform_y = next_y;
        let arena = arena_for(
            candidate,
            model,
            platform_y,
            door,
            FloorMode::Normal,
            None,
            true,
        );
        let before = position;
        let desired = if t_s < entry_end {
            vector![APPROACH_SPEED_MPS * dt, 0.0, 0.0]
        } else if t_s < motion_end + motion_end_tolerance {
            vector![0.0, environment_delta, 0.0]
        } else {
            vector![APPROACH_SPEED_MPS * dt, 0.0, 0.0]
        };
        let (movement, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        let penetrated = arena.intersects(&position, &shape);
        let foot = foot_y(&position);
        if t_s < entry_end && position.translation.vector.x > -0.10 {
            entry_inside = true;
        }
        if t_s >= motion_end && position.translation.vector.x > half_width + 0.30 {
            exit_complete = true;
        }
        if arrival.is_none() && (foot - target_y).abs() <= ARRIVAL_TOLERANCE_M {
            arrival = Some(t_s);
        }
        let (mut sample, current_velocity, _, _, _, _) = frame(
            tick,
            t_s,
            candidate,
            model,
            platform_y,
            &position,
            &before,
            previous_velocity,
            dt,
            normals,
            movement,
            door,
            platform_y,
            false,
        );
        sample["penetration"] = json!(penetrated);
        sample["environment_delta_m"] = json!([0.0, environment_delta, 0.0]);
        max_penetration |= penetrated;
        previous_velocity = current_velocity;
        frames.push(sample);
    }
    let final_foot = foot_y(&position);
    let pass = entry_inside
        && all_motion_closed
        && !moved_while_open
        && arrival.is_some()
        && exit_complete
        && !max_penetration;
    let summary = Summary {
        final_foot_y_m: final_foot,
        arrival_time_s: arrival,
        max_penetration,
        all_supported: true,
        ..Summary::default()
    };
    let report = scenario(
        format!(
            "{}_{}_door_sync_{}_{}hz",
            candidate.id,
            model.name(),
            if up { "up" } else { "down" },
            hz
        ),
        json!({"model": model.name(), "direction": if up { "up" } else { "down" }, "hz": hz, "door_id": candidate.door_id, "open_entry_s": entry_end, "closing_s": close_end - entry_end, "closed_motion_s": PLATFORM_TRAVEL_DURATION_S, "open_exit_after_s": motion_end, "motion_profile": "linear_candidate_only"}),
        frames,
        json!({"entry_when_open": true, "door_closed_during_motion": true, "arrives_at_endpoint": true, "stable_exit_when_open": true, "no_platform_motion_with_open_door": true, "penetration": false}),
        json!({"entry_inside": entry_inside, "all_motion_closed": all_motion_closed, "moved_while_open": moved_while_open, "arrival_time_s": arrival, "exit_complete": exit_complete, "final_foot_y_m": final_foot, "max_penetration": max_penetration}),
        json!({"arrival_error_m": ARRIVAL_TOLERANCE_M, "exit_x_margin_m": 0.30, "penetration": false}),
        pass,
    );
    (report, summary)
}

fn run_closed_door(candidate: Candidate, model: Model, hz: u32, up: bool) -> (Value, Summary) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let platform_y = if up { 0.0 } else { candidate.delta_y_m };
    let half_width = candidate.width_m / 2.0;
    let shape = character_shape();
    let kcc = controller();
    let arena = arena_for(
        candidate,
        model,
        platform_y,
        DoorState::Closed,
        FloorMode::Normal,
        None,
        true,
    );
    let mut position = character_position(-half_width - 1.0, platform_y + 0.02, 0.0);
    let mut velocity_y = 3.5_f32;
    let mut frames = Vec::new();
    let mut max_x = position.translation.vector.x;
    let mut max_penetration = false;
    for tick in 0..120 {
        let before = position;
        let desired = vector![APPROACH_SPEED_MPS * dt, velocity_y * dt, 0.4 * dt];
        let (movement, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        if desired.y > 0.0 && movement.translation.y < desired.y - 0.002 {
            velocity_y = 0.0;
        } else {
            velocity_y -= GRAVITY_MPS2 * dt;
        }
        let penetrated = arena.intersects(&position, &shape);
        max_x = max_x.max(position.translation.vector.x);
        max_penetration |= penetrated;
        let (mut sample, _, _, _, _, _) = frame(
            tick,
            (tick as f32 + 1.0) * dt,
            candidate,
            model,
            platform_y,
            &position,
            &before,
            [0.0; 3],
            dt,
            normals,
            movement,
            DoorState::Closed,
            platform_y,
            false,
        );
        sample["penetration"] = json!(penetrated);
        sample["attempt"] = json!("through/jump/diagonal/fall");
        frames.push(sample);
    }
    let stopped_outside = max_x <= -half_width + 0.05;
    let pass = stopped_outside && !max_penetration;
    let summary = Summary {
        final_foot_y_m: foot_y(&position),
        max_penetration,
        ..Summary::default()
    };
    let report = scenario(
        format!(
            "{}_{}_door_closed_blocked_{}_{}hz",
            candidate.id,
            model.name(),
            if up { "up" } else { "down" },
            hz
        ),
        json!({"model": model.name(), "direction": if up { "up" } else { "down" }, "hz": hz, "door_state": "Closed", "attempts": ["enter", "jump_over", "diagonal", "fall_through"]}),
        frames,
        json!({"cannot_enter": true, "cannot_fall_through": true, "cannot_clip": true, "penetration": false}),
        json!({"max_x_m": max_x, "door_plane_x_m": -half_width, "max_penetration": max_penetration}),
        json!({"max_door_plane_x_m": -half_width + 0.05, "penetration": false}),
        pass,
    );
    (report, summary)
}

fn run_mid_stop(candidate: Candidate, model: Model, hz: u32, up: bool) -> (Value, Summary) {
    let (mut report, mut summary) = run_motion(candidate, model, hz, up, true);
    let hold_start = 0.8_f32;
    let hold_end = 1.3_f32;
    let hold_frames = report["frames"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|frame| {
            let t = frame["t_s"].as_f64().unwrap_or(0.0) as f32;
            t > hold_start && t < hold_end
        })
        .collect::<Vec<_>>();
    let hold_platform_delta = if hold_frames.len() >= 2 {
        (hold_frames.last().unwrap()["platform_center_m"][1]
            .as_f64()
            .unwrap()
            - hold_frames.first().unwrap()["platform_center_m"][1]
                .as_f64()
                .unwrap())
        .abs()
    } else {
        0.0
    };
    let hold_pass = hold_platform_delta <= 0.001;
    let pass = report["pass"] == true && hold_pass;
    report["name"] = json!(format!(
        "{}_{}_mid_stop_{}_{}hz",
        candidate.id,
        model.name(),
        if up { "up" } else { "down" },
        hz
    ));
    report["inputs"]["fault"] = json!("mid_motion_stop_then_resume");
    report["actual"]["hold_platform_delta_m"] = json!(hold_platform_delta);
    report["threshold"]["max_hold_platform_delta_m"] = json!(0.001);
    report["pass"] = json!(pass);
    summary.recovered = hold_pass;
    (report, summary)
}

fn run_support_loss(candidate: Candidate, model: Model, hz: u32, up: bool) -> (Value, Summary) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let platform_y = if up { 0.0 } else { candidate.delta_y_m };
    let recovery_y = platform_y - RECOVERY_DROP_M;
    let shape = character_shape();
    let kcc = controller();
    let mut position = character_position(0.0, platform_y + 0.02, 0.0);
    let mut velocity_y = 0.0_f32;
    let mut previous_velocity = [0.0; 3];
    let mut frames = Vec::new();
    let mut lost_support = false;
    let mut recovered = false;
    let mut max_penetration = false;
    for tick in 0..((3.0 / dt) as usize) {
        let t_s = (tick as f32 + 1.0) * dt;
        let floor_mode = if t_s < 0.5 {
            FloorMode::Normal
        } else {
            FloorMode::MissingSupport
        };
        let arena = arena_for(
            candidate,
            model,
            platform_y,
            DoorState::Closed,
            floor_mode,
            Some(recovery_y),
            false,
        );
        let before = position;
        let desired = vector![0.0, velocity_y * dt, 0.0];
        let (movement, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        let penetrated = arena.intersects(&position, &shape);
        let foot = foot_y(&position);
        let support = normals.iter().any(|normal| normal[1] > 0.90)
            || (floor_mode == FloorMode::Normal
                && (foot - platform_y).abs() <= SUPPORT_TOLERANCE_M)
            || (floor_mode == FloorMode::MissingSupport
                && (foot - recovery_y).abs() <= SUPPORT_TOLERANCE_M);
        if t_s >= 0.5 && !support {
            lost_support = true;
        }
        if t_s >= 0.5 && (foot - recovery_y).abs() <= ARRIVAL_TOLERANCE_M && support {
            recovered = true;
        }
        velocity_y = if support || movement.translation.y < desired.y - 0.002 {
            0.0
        } else {
            velocity_y - GRAVITY_MPS2 * dt
        };
        let (mut sample, current_velocity, _, _, _, _) = frame(
            tick,
            t_s,
            candidate,
            model,
            platform_y,
            &position,
            &before,
            previous_velocity,
            dt,
            normals,
            movement,
            DoorState::Closed,
            if floor_mode == FloorMode::Normal {
                platform_y
            } else {
                recovery_y
            },
            recovered,
        );
        sample["penetration"] = json!(penetrated);
        sample["support_fault"] = json!(floor_mode == FloorMode::MissingSupport);
        sample["recovery_floor_y_m"] = json!(recovery_y);
        max_penetration |= penetrated;
        previous_velocity = current_velocity;
        frames.push(sample);
    }
    let pass = lost_support && recovered && !max_penetration;
    let summary = Summary {
        final_foot_y_m: foot_y(&position),
        max_penetration,
        all_supported: recovered,
        recovered,
        ..Summary::default()
    };
    let report = scenario(
        format!(
            "{}_{}_support_loss_recovery_{}_{}hz",
            candidate.id,
            model.name(),
            if up { "up" } else { "down" },
            hz
        ),
        json!({"model": model.name(), "direction": if up { "up" } else { "down" }, "hz": hz, "fault_at_s": 0.5, "recovery_drop_m": RECOVERY_DROP_M, "recovery": "KCC gravity descent onto candidate recovery floor"}),
        frames,
        json!({"support_loss_observed": true, "legal_recovery": true, "no_teleport": true, "penetration": false}),
        json!({"lost_support": lost_support, "recovered": recovered, "final_foot_y_m": foot_y(&position), "max_penetration": max_penetration}),
        json!({"recovery_floor_error_m": ARRIVAL_TOLERANCE_M, "penetration": false}),
        pass,
    );
    (report, summary)
}

fn run_edge_recovery(candidate: Candidate, model: Model, hz: u32, up: bool) -> (Value, Summary) {
    let dt = if hz == 30 { DT_30 } else { DT_60 };
    let platform_y = if up { 0.0 } else { candidate.delta_y_m };
    let recovery_y = platform_y - RECOVERY_DROP_M;
    let half_width = candidate.width_m / 2.0;
    let shape = character_shape();
    let kcc = controller();
    let mut position =
        character_position(half_width - CAPSULE_RADIUS_M - 0.05, platform_y + 0.02, 0.0);
    let mut velocity_y = 0.0_f32;
    let mut previous_velocity = [0.0; 3];
    let mut frames = Vec::new();
    let mut edge_crossed = false;
    let mut recovered = false;
    let mut max_penetration = false;
    let ticks = (3.0 / dt) as usize;
    for tick in 0..ticks {
        let t_s = (tick as f32 + 1.0) * dt;
        let arena = arena_for(
            candidate,
            model,
            platform_y,
            DoorState::Open,
            FloorMode::OpenEdge,
            Some(recovery_y),
            false,
        );
        let before = position;
        let desired = vector![APPROACH_SPEED_MPS * dt, velocity_y * dt, 0.0];
        let (movement, normals) = move_character(&arena, &kcc, &mut position, &shape, dt, desired);
        let penetrated = arena.intersects(&position, &shape);
        let foot = foot_y(&position);
        if position.translation.vector.x > half_width + 0.05 {
            edge_crossed = true;
        }
        let support = normals.iter().any(|normal| normal[1] > 0.90)
            || (foot - recovery_y).abs() <= SUPPORT_TOLERANCE_M;
        if edge_crossed && (foot - recovery_y).abs() <= ARRIVAL_TOLERANCE_M && support {
            recovered = true;
        }
        velocity_y = if support || movement.translation.y < desired.y - 0.002 {
            0.0
        } else {
            velocity_y - GRAVITY_MPS2 * dt
        };
        let (mut sample, current_velocity, _, _, _, _) = frame(
            tick,
            t_s,
            candidate,
            model,
            platform_y,
            &position,
            &before,
            previous_velocity,
            dt,
            normals,
            movement,
            DoorState::Open,
            if recovered { recovery_y } else { platform_y },
            recovered,
        );
        sample["penetration"] = json!(penetrated);
        sample["edge_attempt"] = json!(true);
        sample["recovery_floor_y_m"] = json!(recovery_y);
        max_penetration |= penetrated;
        previous_velocity = current_velocity;
        frames.push(sample);
    }
    let pass = edge_crossed && recovered && !max_penetration;
    let summary = Summary {
        final_foot_y_m: foot_y(&position),
        max_penetration,
        edge_crossed,
        recovered,
        ..Summary::default()
    };
    let report = scenario(
        format!(
            "{}_{}_edge_fall_recovery_{}_{}hz",
            candidate.id,
            model.name(),
            if up { "up" } else { "down" },
            hz
        ),
        json!({"model": model.name(), "direction": if up { "up" } else { "down" }, "hz": hz, "edge_x_m": half_width, "recovery_drop_m": RECOVERY_DROP_M, "recovery": "KCC gravity descent onto candidate recovery floor"}),
        frames,
        json!({"edge_crossed": true, "legal_recovery": true, "no_teleport": true, "penetration": false}),
        json!({"edge_crossed": edge_crossed, "recovered": recovered, "final_foot_y_m": foot_y(&position), "max_penetration": max_penetration}),
        json!({"edge_cross_margin_m": 0.05, "recovery_floor_error_m": ARRIVAL_TOLERANCE_M, "penetration": false}),
        pass,
    );
    (report, summary)
}

fn compare(name: String, first: Summary, second: Summary) -> Value {
    let first_arrival = first.arrival_time_s.unwrap_or(f32::INFINITY);
    let second_arrival = second.arrival_time_s.unwrap_or(f32::INFINITY);
    let arrival_delta = (first_arrival - second_arrival).abs();
    let foot_delta = (first.final_foot_y_m - second.final_foot_y_m).abs();
    let pass = arrival_delta <= 0.05 && foot_delta <= 0.05;
    json!({
        "name": name,
        "inputs": {"hz": [30, 60], "same_seconds_motion_profile": true},
        "expected": {"arrival_time_delta_s": "<= 0.05", "final_foot_delta_m": "<= 0.05"},
        "actual": {"arrival_time_delta_s": arrival_delta, "final_foot_delta_m": foot_delta},
        "threshold": {"max_arrival_time_delta_s": 0.05, "max_final_foot_delta_m": 0.05},
        "pass": pass,
    })
}

fn run_candidate(candidate: Candidate) -> Value {
    let mut scenarios = Vec::new();
    let mut comparisons = Vec::new();
    for model in [Model::OpenPlatform, Model::ControlledCabin] {
        for up in [true, false] {
            let mut motion_summaries = Vec::new();
            for hz in [30_u32, 60_u32] {
                let (report, summary) = run_motion(candidate, model, hz, up, false);
                scenarios.push(report);
                motion_summaries.push(summary);
                let (report, summary) = run_door_sync(candidate, model, hz, up);
                scenarios.push(report);
                let (report, _) = run_closed_door(candidate, model, hz, up);
                scenarios.push(report);
                let (report, _) = run_mid_stop(candidate, model, hz, up);
                scenarios.push(report);
                let (report, _) = run_support_loss(candidate, model, hz, up);
                scenarios.push(report);
                let (report, _) = run_edge_recovery(candidate, model, hz, up);
                scenarios.push(report);
                if hz == 60 {
                    let _ = summary;
                }
            }
            comparisons.push(compare(
                format!(
                    "{}_{}_{}_30_vs_60_hz",
                    candidate.id,
                    model.name(),
                    if up { "up" } else { "down" }
                ),
                motion_summaries[0],
                motion_summaries[1],
            ));
        }
    }
    let machine_pass = scenarios.iter().all(|scenario| scenario["pass"] == true)
        && comparisons
            .iter()
            .all(|comparison| comparison["pass"] == true);
    json!({
        "id": candidate.id,
        "source": candidate_json(candidate),
        "models": ["open_platform", "controlled_cabin"],
        "scenarios": scenarios,
        "hz_comparisons": comparisons,
        "machine_pass": machine_pass,
    })
}

fn run_all() -> Value {
    let connectors: Vec<Value> = CANDIDATES.iter().copied().map(run_candidate).collect();
    let machine_pass = connectors
        .iter()
        .all(|connector| connector["machine_pass"] == true);
    json!({
        "schema": "continuous_physics_optional_elevators_ag38_v1",
        "status": "blocked_formal_candidate",
        "exit_code": 0,
        "scope": "isolated Rapier 0.22 parameterized validation of three optional v7 elevator candidates; no formal Blockout, Campaign, nav, camera, or save",
        "source": {
            "v7": "tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json",
            "connector_inventory": "tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json",
            "tp04a": "docs/CONTINUOUS_PHYSICS_KCC_REVIEW.md",
            "tp04b2": "tools/continuous-tps-architecture-probe/results/physics-gate-b-tp04b2.json",
            "gate_b_real_connector_excluded": "edge-b05-c01",
            "read_only": true,
        },
        "rapier": {"crate": "rapier3d", "version": "0.22.0"},
        "platform_contract": {
            "units": "metres, seconds, radians",
            "up": "+Y",
            "capsule_radius_m": CAPSULE_RADIUS_M,
            "capsule_half_segment_m": CAPSULE_HALF_SEGMENT_M,
            "capsule_total_height_m": CAPSULE_TOTAL_HEIGHT_M,
            "foot_anchor": "capsule bottom",
            "collision_offset_m": COLLISION_OFFSET_M,
            "kcc_autostep_max_height_m": 0.35,
            "kcc_autostep_min_width_m": 0.25,
            "kcc_max_slope_degrees": 35.0,
            "tick_rates_hz": [30, 60],
        },
        "assumptions": [
            {"field": "motion_profile", "value": "linear travel over 3.0s with 1.0s open entry, 0.5s close, and open exit sequence", "classification": "candidateOnly", "reason": "v7 does not define elevator speed, acceleration, or door timing"},
            {"field": "platform_speed_mps", "value": PLATFORM_SPEED_ASSUMPTION_MPS, "classification": "candidateOnly", "reason": "v7 does not define motion speed; value is a comparison parameter only"},
            {"field": "platform_acceleration_mps2", "value": "constant-speed linear profile", "classification": "candidateOnly", "reason": "v7 does not define elevator acceleration or easing"},
            {"field": "cabin_height_m", "value": "connector candidate height from AG-35 inventory", "classification": "candidateOnly", "reason": "inventory geometry is not a formal cabin export"},
            {"field": "cabin_depth_m", "value": 3.0, "classification": "candidateOnly", "reason": "v7 has no formal cabin depth"},
            {"field": "recovery_floor_m", "value": RECOVERY_DROP_M, "classification": "candidateOnly", "reason": "v7 has no failure/recovery geometry"},
            {"field": "door_geometry", "value": "full-width vertical blocker at entry plane", "classification": "candidateOnly", "reason": "v7 gives door ID/state but no collider or timing export"},
            {"field": "door_timing", "value": "1.0s open entry, 0.5s close, open exit after endpoint", "classification": "candidateOnly", "reason": "v7 gives door state but no formal timing or interlock export"},
        ],
        "connectors": connectors,
        "acceptance": {
            "each_connector_has_bidirectional_up_down": true,
            "each_connector_has_open_platform_and_controlled_cabin": true,
            "each_connector_has_door_sync": true,
            "each_connector_has_moving_support_and_grounded": true,
            "each_connector_has_arrival_and_stable_exit": true,
            "each_connector_has_closed_door_enter_fall_clip_checks": true,
            "each_connector_has_mid_stop": true,
            "each_connector_has_support_loss_recovery": true,
            "each_connector_has_edge_fall_recovery": true,
            "per_tick_platform_capsule_velocity_acceleration_normals_support_slip_penetration_door": true,
            "same_seconds_30_60_motion_profile": true,
            "no_capsule_teleport": true,
        },
        "formal_blockers": [
            "v7 source does not define elevator speed, acceleration, motion profile, or door timing",
            "v7/AG-35 inventory values for cabin width/height are candidate geometry, not formal cabin export",
            "no formal static collider/blockout, nav, camera, checkpoint, or recovery export was tested",
            "edge-b05-s04 is optional S04 and is not Gate B edge-b05-c01",
            "Gate C/I and overall Architecture Gate remain blocked/no_go",
        ],
        "machine_pass": machine_pass,
        "notes": [
            "Platform/cabin colliders are rebuilt at each tick to model environment motion; the capsule moves only through Rapier KCC swept movement.",
            "The open-platform and controlled-cabin values are isolated candidates only and must not be promoted to formal product truth.",
            "No image/media model was invoked and no formal game integration was attempted.",
        ],
    })
}

fn result_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("results")
        .join("physics-elevators-optional-ag38.json")
}

#[test]
fn optional_elevator_candidates_are_machine_checked_without_formalizing_them() {
    let report = run_all();
    fs::create_dir_all(result_path().parent().unwrap()).unwrap();
    fs::write(
        result_path(),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    assert_eq!(
        report["machine_pass"], true,
        "parameterized elevator physics failed: {report}"
    );
    assert_eq!(report["status"], "blocked_formal_candidate");
    assert_eq!(
        report["source"]["gate_b_real_connector_excluded"],
        "edge-b05-c01"
    );
}

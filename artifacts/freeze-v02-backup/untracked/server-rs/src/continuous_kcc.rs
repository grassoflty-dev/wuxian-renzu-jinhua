//! Rapier-backed continuous KCC adapter.
//!
//! Rapier types and static collider storage stay private to this module. The
//! public boundary uses TP-02 metre/radian Transform and Vec3 values only.
//! This is a static-geometry adapter for TP-04A; it is not wired to runtime,
//! Campaign, Tauri, navigation, or the Grey Hive map.

use crate::world_v3::{Transform, Vec3, WorldModelError};
use rapier3d::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterController},
    parry::query::ShapeCastOptions,
    prelude::*,
};

pub const CAPSULE_HALF_SEGMENT_M: f32 = 0.75;
pub const CAPSULE_RADIUS_M: f32 = 0.35;
pub const FOOT_CENTER_OFFSET_M: f32 = 1.10;
pub const COLLISION_OFFSET_M: f32 = 0.01;
pub const SNAP_DISTANCE_M: f32 = 0.20;
pub const GRAVITY_MPS2: f32 = 9.81;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KccError {
    NonFinite(&'static str),
    InvalidDt,
    InvalidGravity,
    InvalidGeometry,
    Transform(WorldModelError),
}

impl std::fmt::Display for KccError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite(field) => write!(formatter, "{field} must be finite"),
            Self::InvalidDt => formatter.write_str("dt must be positive and finite"),
            Self::InvalidGravity => formatter.write_str("gravity must be finite and non-negative"),
            Self::InvalidGeometry => formatter.write_str("static geometry is invalid"),
            Self::Transform(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for KccError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KccState {
    pub foot_transform: Transform,
    pub velocity_mps: Vec3,
}

impl KccState {
    pub fn new(foot_transform: Transform, velocity_mps: Vec3) -> Self {
        Self {
            foot_transform,
            velocity_mps,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionInput {
    desired_translation_m: [f32; 3],
    gravity_mps2: f32,
    dt_s: f32,
}

impl MotionInput {
    pub fn new(
        desired_translation_m: [f32; 3],
        gravity_mps2: f32,
        dt_s: f32,
    ) -> Result<Self, KccError> {
        if desired_translation_m.iter().any(|value| !value.is_finite()) {
            return Err(KccError::NonFinite("desiredTranslationM"));
        }
        if !gravity_mps2.is_finite() || gravity_mps2 < 0.0 {
            return Err(KccError::InvalidGravity);
        }
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return Err(KccError::InvalidDt);
        }
        Ok(Self {
            desired_translation_m,
            gravity_mps2,
            dt_s,
        })
    }

    pub fn stationary(dt_s: f32) -> Result<Self, KccError> {
        Self::new([0.0, 0.0, 0.0], 0.0, dt_s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KccStep {
    pub state: KccState,
    pub translation_m: Vec3,
    pub requested_translation_m: Vec3,
    pub kcc_grounded: bool,
    pub contact_grounded: bool,
    pub grounded: bool,
    pub foot_distance_m: Option<f32>,
    pub ground_normal: Option<[f32; 3]>,
    pub collision_normals: Vec<[f32; 3]>,
    pub head_collision: bool,
    pub landed: bool,
    pub penetration: bool,
}

struct StaticRapierWorld {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    queries: QueryPipeline,
}

pub struct StaticKccWorld {
    rapier: StaticRapierWorld,
    controller: KinematicCharacterController,
}

fn finite_array(values: &[f32; 3], field: &'static str) -> Result<(), KccError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(KccError::NonFinite(field))
    }
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
        snap_to_ground: Some(CharacterLength::Absolute(SNAP_DISTANCE_M)),
        normal_nudge_factor: 1.0e-4,
    }
}

fn character_shape() -> SharedShape {
    SharedShape::capsule_y(CAPSULE_HALF_SEGMENT_M, CAPSULE_RADIUS_M)
}

fn foot_to_isometry(transform: &Transform) -> Isometry<Real> {
    let position = transform.position_m();
    Isometry::translation(
        position.x_m(),
        position.y_m() + FOOT_CENTER_OFFSET_M,
        position.z_m(),
    )
}

fn foot_from_isometry(position: &Isometry<Real>, yaw_rad: f32) -> Result<Transform, KccError> {
    let vector = position.translation.vector;
    let foot = Vec3::new(vector.x, vector.y - FOOT_CENTER_OFFSET_M, vector.z)
        .map_err(KccError::Transform)?;
    Transform::new(foot, yaw_rad).map_err(KccError::Transform)
}

fn normal_array(normal: Vector<Real>) -> [f32; 3] {
    [normal.x, normal.y, normal.z]
}

impl StaticKccWorld {
    pub fn new() -> Self {
        Self {
            rapier: StaticRapierWorld {
                bodies: RigidBodySet::new(),
                colliders: ColliderSet::new(),
                queries: QueryPipeline::new(),
            },
            controller: controller(),
        }
    }

    pub fn room_10m() -> Self {
        let mut world = Self::new();
        world.add_box_unchecked([0.0, -0.10, 0.0], [5.0, 0.10, 5.0]);
        world.add_box_unchecked([-5.10, 1.50, 0.0], [0.10, 1.50, 5.0]);
        world.add_box_unchecked([5.10, 1.50, 0.0], [0.10, 1.50, 5.0]);
        world.add_box_unchecked([0.0, 1.50, -5.10], [5.0, 1.50, 0.10]);
        world.add_box_unchecked([0.0, 1.50, 5.10], [5.0, 1.50, 0.10]);
        world.refresh();
        world
    }

    pub fn add_box(
        &mut self,
        center_m: [f32; 3],
        half_extents_m: [f32; 3],
    ) -> Result<(), KccError> {
        finite_array(&center_m, "centerM")?;
        finite_array(&half_extents_m, "halfExtentsM")?;
        if half_extents_m.iter().any(|value| *value <= 0.0) {
            return Err(KccError::InvalidGeometry);
        }
        self.add_box_unchecked(center_m, half_extents_m);
        self.refresh();
        Ok(())
    }

    pub fn add_convex_hull(&mut self, vertices_m: &[[f32; 3]]) -> Result<(), KccError> {
        if vertices_m.len() < 4 {
            return Err(KccError::InvalidGeometry);
        }
        let mut vertices = Vec::with_capacity(vertices_m.len());
        for vertex in vertices_m {
            finite_array(vertex, "verticesM")?;
            vertices.push(point![vertex[0], vertex[1], vertex[2]]);
        }
        let collider = ColliderBuilder::convex_hull(&vertices)
            .ok_or(KccError::InvalidGeometry)?
            .build();
        self.rapier.colliders.insert(collider);
        self.refresh();
        Ok(())
    }

    fn add_box_unchecked(&mut self, center_m: [f32; 3], half_extents_m: [f32; 3]) {
        self.rapier.colliders.insert(
            ColliderBuilder::cuboid(half_extents_m[0], half_extents_m[1], half_extents_m[2])
                .translation(vector![center_m[0], center_m[1], center_m[2]])
                .build(),
        );
    }

    fn refresh(&mut self) {
        self.rapier.queries.update(&self.rapier.colliders);
    }

    pub fn intersects(&self, state: &KccState) -> bool {
        let position = foot_to_isometry(&state.foot_transform);
        // Treat the configured KCC skin as non-penetrating clearance. Rapier's
        // intersection query reports a touching contact as an intersection;
        // shrinking the probe by the same skin avoids false positives on
        // walkable ramps while still detecting penetration beyond the adapter's
        // collision offset.
        let probe_shape = SharedShape::capsule_y(
            CAPSULE_HALF_SEGMENT_M - COLLISION_OFFSET_M,
            CAPSULE_RADIUS_M - COLLISION_OFFSET_M,
        );
        self.rapier
            .queries
            .intersection_with_shape(
                &self.rapier.bodies,
                &self.rapier.colliders,
                &position,
                probe_shape.as_ref(),
                QueryFilter::default(),
            )
            .is_some()
    }

    pub fn move_foot(&self, state: &KccState, input: MotionInput) -> Result<KccStep, KccError> {
        if !state.velocity_mps.x_m().is_finite()
            || !state.velocity_mps.y_m().is_finite()
            || !state.velocity_mps.z_m().is_finite()
        {
            return Err(KccError::NonFinite("velocityMps"));
        }
        let mut position = foot_to_isometry(&state.foot_transform);
        let requested_y = state.velocity_mps.y_m() * input.dt_s;
        let requested = vector![
            input.desired_translation_m[0],
            requested_y,
            input.desired_translation_m[2]
        ];
        let shape = character_shape();
        let mut collision_normals = Vec::new();
        let effective = self.controller.move_shape(
            input.dt_s,
            &self.rapier.bodies,
            &self.rapier.colliders,
            &self.rapier.queries,
            shape.as_ref(),
            &position,
            requested,
            QueryFilter::default(),
            |collision| collision_normals.push(normal_array(collision.hit.normal1.into_inner())),
        );
        position.translation.vector += effective.translation;

        let probe = self.rapier.queries.cast_shape(
            &self.rapier.bodies,
            &self.rapier.colliders,
            &position,
            &vector![0.0, -1.0, 0.0],
            shape.as_ref(),
            ShapeCastOptions::with_max_time_of_impact(SNAP_DISTANCE_M),
            QueryFilter::default(),
        );
        let (foot_distance_m, probe_normal) = probe
            .map(|(_, hit)| {
                (
                    Some(hit.time_of_impact.max(0.0)),
                    Some(normal_array(hit.normal1.into_inner())),
                )
            })
            .unwrap_or((None, None));
        let upward_collision = collision_normals
            .iter()
            .filter(|normal| normal[1] >= 0.70)
            .max_by(|left, right| left[1].total_cmp(&right[1]))
            .copied();
        let ground_normal = probe_normal.or(upward_collision);
        let contact_grounded = foot_distance_m
            .is_some_and(|distance| distance <= SNAP_DISTANCE_M + 1.0e-4)
            && ground_normal.is_some_and(|normal| normal[1] >= 0.70);
        let head_collision =
            state.velocity_mps.y_m() > 0.0 && effective.translation.y < requested_y - 0.002;
        let mut vertical_velocity = if head_collision {
            0.0
        } else {
            state.velocity_mps.y_m() - input.gravity_mps2 * input.dt_s
        };
        if contact_grounded && vertical_velocity < 0.0 {
            vertical_velocity = 0.0;
        }
        let foot_transform = foot_from_isometry(&position, state.foot_transform.yaw_rad())?;
        let velocity_mps = Vec3::new(
            state.velocity_mps.x_m(),
            vertical_velocity,
            state.velocity_mps.z_m(),
        )
        .map_err(KccError::Transform)?;
        let next_state = KccState::new(foot_transform, velocity_mps);
        let penetration = self.intersects(&next_state);
        Ok(KccStep {
            state: next_state,
            translation_m: Vec3::new(
                effective.translation.x,
                effective.translation.y,
                effective.translation.z,
            )
            .map_err(KccError::Transform)?,
            requested_translation_m: Vec3::new(requested.x, requested.y, requested.z)
                .map_err(KccError::Transform)?,
            kcc_grounded: effective.grounded,
            contact_grounded,
            grounded: contact_grounded,
            foot_distance_m,
            ground_normal,
            collision_normals,
            head_collision,
            landed: contact_grounded && state.velocity_mps.y_m() <= 0.0,
            penetration,
        })
    }
}

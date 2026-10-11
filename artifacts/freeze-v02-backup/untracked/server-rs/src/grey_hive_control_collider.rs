//! Isolated Rapier collider slice for the AG-23 Grey Hive B01↔B02 channel.
//!
//! This module deliberately exposes only a small static-geometry probe. It
//! does not enter Campaign, WorldState, SaveV3, Babylon, or any formal map.

use crate::continuous_kcc::{KccError, KccState, MotionInput, StaticKccWorld};
use crate::world_v3::{Transform, Vec3, WorldModelError};

pub const CONTROL_CHANNEL_WIDTH_M: f32 = 3.5;
pub const CONTROL_CHANNEL_HEIGHT_M: f32 = 4.0;
pub const CONTROL_CHANNEL_MAX_STEP_M: f32 = 0.5;
pub const CONTROL_CHANNEL_BASE_LENGTH_M: f32 = 28.0;
pub const CONTROL_CHANNEL_V7_LENGTH_M: f32 = 40.0;

pub const V7_WAYPOINTS: &[[f32; 3]] = &[
    [52.0, 0.0, 38.0],
    [44.0, 0.0, 32.0],
    [44.0, 0.0, 26.0],
    [32.0, 0.0, 26.0],
    [32.0, 0.0, 32.0],
    [32.0, 0.0, 38.0],
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlColliderError {
    Kcc(KccError),
    InvalidPath,
}

impl std::fmt::Display for ControlColliderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kcc(error) => error.fmt(formatter),
            Self::InvalidPath => formatter.write_str("control collider path is invalid"),
        }
    }
}

impl std::error::Error for ControlColliderError {}

impl From<KccError> for ControlColliderError {
    fn from(error: KccError) -> Self {
        Self::Kcc(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlRun {
    pub requested_distance_m: f32,
    pub actual_distance_m: f32,
    pub reached_goal: bool,
    pub blocked: bool,
    pub penetration: bool,
    pub frames: usize,
    pub final_state: KccState,
    pub collision_normals: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShortestPathMeasurement {
    pub centerline_m: f32,
    pub shortest_complete_m: Option<f32>,
    pub candidates: Vec<ControlRun>,
}

pub struct ControlColliderSlice {
    world: StaticKccWorld,
    gate_b_open: bool,
}

fn state_at(position: [f32; 3]) -> Result<KccState, ControlColliderError> {
    let transform = Transform::new(Vec3::new(position[0], position[1], position[2])?, 0.0)?;
    Ok(KccState::new(transform, Vec3::new(0.0, 0.0, 0.0)?))
}

fn path_length(path: &[[f32; 3]]) -> f32 {
    path.windows(2)
        .map(|pair| {
            let dx = pair[1][0] - pair[0][0];
            let dy = pair[1][1] - pair[0][1];
            let dz = pair[1][2] - pair[0][2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

fn add_segment_wall(
    world: &mut StaticKccWorld,
    first: [f32; 3],
    second: [f32; 3],
    side: f32,
) -> Result<(), ControlColliderError> {
    let dx = second[0] - first[0];
    let dz = second[2] - first[2];
    let length = (dx * dx + dz * dz).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err(ControlColliderError::InvalidPath);
    }
    let normal_x = -dz / length;
    let normal_z = dx / length;
    let trim = 0.50_f32.min(length * 0.25);
    let tangent_x = dx / length;
    let tangent_z = dz / length;
    let first = [
        first[0] + tangent_x * trim,
        first[1],
        first[2] + tangent_z * trim,
    ];
    let second = [
        second[0] - tangent_x * trim,
        second[1],
        second[2] - tangent_z * trim,
    ];
    let inner = CONTROL_CHANNEL_WIDTH_M * 0.5;
    let outer = inner + 0.25;
    let vertices = [
        [
            first[0] + normal_x * side * inner,
            0.0,
            first[2] + normal_z * side * inner,
        ],
        [
            second[0] + normal_x * side * inner,
            0.0,
            second[2] + normal_z * side * inner,
        ],
        [
            second[0] + normal_x * side * outer,
            0.0,
            second[2] + normal_z * side * outer,
        ],
        [
            first[0] + normal_x * side * outer,
            0.0,
            first[2] + normal_z * side * outer,
        ],
        [
            first[0] + normal_x * side * inner,
            CONTROL_CHANNEL_HEIGHT_M,
            first[2] + normal_z * side * inner,
        ],
        [
            second[0] + normal_x * side * inner,
            CONTROL_CHANNEL_HEIGHT_M,
            second[2] + normal_z * side * inner,
        ],
        [
            second[0] + normal_x * side * outer,
            CONTROL_CHANNEL_HEIGHT_M,
            second[2] + normal_z * side * outer,
        ],
        [
            first[0] + normal_x * side * outer,
            CONTROL_CHANNEL_HEIGHT_M,
            first[2] + normal_z * side * outer,
        ],
    ];
    world
        .add_convex_hull(&vertices)
        .map_err(ControlColliderError::Kcc)
}

impl ControlColliderSlice {
    pub fn open() -> Result<Self, ControlColliderError> {
        Self::new(true)
    }

    pub fn closed() -> Result<Self, ControlColliderError> {
        Self::new(false)
    }

    pub fn gate_b_open(&self) -> bool {
        self.gate_b_open
    }

    pub fn run_forward(&self) -> Result<ControlRun, ControlColliderError> {
        let start = state_at(V7_WAYPOINTS[0])?;
        self.run_path(&start, V7_WAYPOINTS)
    }

    pub fn run_reverse(&self) -> Result<ControlRun, ControlColliderError> {
        let mut reverse = V7_WAYPOINTS.to_vec();
        reverse.reverse();
        let start = state_at(reverse[0])?;
        self.run_path(&start, &reverse)
    }

    pub fn shortest_open_path() -> Result<ShortestPathMeasurement, ControlColliderError> {
        let candidates: Vec<Vec<[f32; 3]>> = vec![
            V7_WAYPOINTS.to_vec(),
            vec![
                V7_WAYPOINTS[0],
                V7_WAYPOINTS[2],
                V7_WAYPOINTS[3],
                V7_WAYPOINTS[4],
                V7_WAYPOINTS[5],
            ],
            vec![
                V7_WAYPOINTS[0],
                V7_WAYPOINTS[1],
                V7_WAYPOINTS[2],
                V7_WAYPOINTS[3],
                V7_WAYPOINTS[5],
            ],
            vec![
                V7_WAYPOINTS[0],
                V7_WAYPOINTS[2],
                V7_WAYPOINTS[3],
                V7_WAYPOINTS[5],
            ],
            vec![V7_WAYPOINTS[0], V7_WAYPOINTS[5]],
        ];
        let centerline_m = path_length(V7_WAYPOINTS);
        let mut results = Vec::new();
        for path in candidates {
            let slice = Self::open()?;
            let start = state_at(path[0])?;
            results.push(slice.run_path(&start, &path)?);
        }
        let shortest_complete_m = results
            .iter()
            .filter(|run| run.reached_goal && !run.penetration)
            .map(|run| run.actual_distance_m)
            .min_by(f32::total_cmp);
        Ok(ShortestPathMeasurement {
            centerline_m,
            shortest_complete_m,
            candidates: results,
        })
    }

    fn new(gate_b_open: bool) -> Result<Self, ControlColliderError> {
        let mut world = StaticKccWorld::new();
        world
            .add_box([42.0, -0.10, 33.0], [18.0, 0.10, 17.0])
            .map_err(ControlColliderError::Kcc)?;

        for pair in V7_WAYPOINTS.windows(2) {
            add_segment_wall(&mut world, pair[0], pair[1], -1.0)?;
            add_segment_wall(&mut world, pair[0], pair[1], 1.0)?;
        }

        // The old straight B01↔B02 route is a solid blocker in the candidate
        // graph. The v7 path exits the B01 landing below this wall and returns
        // to B02 through the service channel.
        world
            .add_box([42.0, 2.0, 38.0], [8.0, 2.0, 0.20])
            .map_err(ControlColliderError::Kcc)?;

        // Closed-door variant: the same channel remains physically present,
        // but the B02 approach is blocked by an explicit door collider.
        if !gate_b_open {
            world
                .add_box([32.0, 2.0, 32.0], [2.0, 2.0, 0.20])
                .map_err(ControlColliderError::Kcc)?;
        }
        Ok(Self { world, gate_b_open })
    }

    fn run_path(
        &self,
        initial: &KccState,
        path: &[[f32; 3]],
    ) -> Result<ControlRun, ControlColliderError> {
        if path.len() < 2
            || path
                .iter()
                .any(|point| point.iter().any(|value| !value.is_finite()))
        {
            return Err(ControlColliderError::InvalidPath);
        }
        let requested_distance_m = path_length(path);
        let mut state = *initial;
        let mut actual_distance_m = 0.0;
        let mut frames = 0;
        let mut blocked = false;
        let mut penetration = false;
        let mut collision_normals = Vec::new();

        'segments: for pair in path.windows(2) {
            let dx = pair[1][0] - pair[0][0];
            let dy = pair[1][1] - pair[0][1];
            let dz = pair[1][2] - pair[0][2];
            let length = (dx * dx + dy * dy + dz * dz).sqrt();
            let steps = (length / CONTROL_CHANNEL_MAX_STEP_M).ceil() as usize;
            if steps == 0 {
                return Err(ControlColliderError::InvalidPath);
            }
            let desired = [dx / steps as f32, dy / steps as f32, dz / steps as f32];
            for _ in 0..steps {
                let result = self
                    .world
                    .move_foot(&state, MotionInput::new(desired, 0.0, 1.0 / 60.0)?)?;
                let actual = result.translation_m;
                actual_distance_m += (actual.x_m() * actual.x_m()
                    + actual.y_m() * actual.y_m()
                    + actual.z_m() * actual.z_m())
                .sqrt();
                collision_normals.extend(result.collision_normals);
                penetration |= result.penetration;
                frames += 1;
                state = result.state;
                if (actual.x_m() - desired[0]).abs()
                    + (actual.y_m() - desired[1]).abs()
                    + (actual.z_m() - desired[2]).abs()
                    > 0.08
                {
                    blocked = true;
                    break 'segments;
                }
            }
        }

        let target = path[path.len() - 1];
        let position = state.foot_transform.position_m();
        let reached_goal = (position.x_m() - target[0]).abs() <= 0.02
            && (position.y_m() - target[1]).abs() <= 0.05
            && (position.z_m() - target[2]).abs() <= 0.02;
        Ok(ControlRun {
            requested_distance_m,
            actual_distance_m,
            reached_goal,
            blocked,
            penetration,
            frames,
            final_state: state,
            collision_normals,
        })
    }
}

impl From<WorldModelError> for ControlColliderError {
    fn from(error: WorldModelError) -> Self {
        Self::Kcc(KccError::Transform(error))
    }
}

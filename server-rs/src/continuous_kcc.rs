use crate::world_v3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl Aabb {
    pub fn new(min_x: f32, max_x: f32, min_z: f32, max_z: f32) -> Result<Self, KccError> {
        if ![min_x, max_x, min_z, max_z].iter().all(|v| v.is_finite())
            || min_x >= max_x
            || min_z >= max_z
        {
            return Err(KccError::InvalidWorld);
        }
        Ok(Self {
            min_x,
            max_x,
            min_z,
            max_z,
        })
    }
    fn contains_circle(&self, x: f32, z: f32, radius: f32) -> bool {
        x + radius > self.min_x
            && x - radius < self.max_x
            && z + radius > self.min_z
            && z - radius < self.max_z
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticKccWorld {
    pub bounds: Aabb,
    pub walls: Vec<Aabb>,
}
impl StaticKccWorld {
    pub fn new(bounds: Aabb, walls: Vec<Aabb>) -> Self {
        Self { bounds, walls }
    }

    /// Fixed first-flow Grey Hive collision blockout. The Gate A slab is
    /// included only while Rust progression has not restored power.
    pub fn grey_hive_first_flow(gate_open: bool) -> Self {
        let bounds = Aabb::new(-8.4, 8.4, -3.4, 20.4).expect("constant Grey Hive bounds");
        let mut walls = vec![
            // Power room shell: x=-4..4, z=-3..6.
            Aabb::new(-4.2, -3.8, -3.0, 6.0).unwrap(),
            Aabb::new(3.8, 4.2, -3.0, 6.0).unwrap(),
            Aabb::new(-4.0, 4.0, -3.2, -2.8).unwrap(),
            // The north wall leaves only the contracted 3m passage open.
            Aabb::new(-8.2, -1.5, 5.8, 6.2).unwrap(),
            Aabb::new(1.5, 8.2, 5.8, 6.2).unwrap(),
            // Sealed sides of the narrow approach to Gate A.
            Aabb::new(-1.7, -1.3, 6.0, 10.0).unwrap(),
            Aabb::new(1.3, 1.7, 6.0, 10.0).unwrap(),
            // South boundary of the Central Shaft entrance, open only to the passage.
            Aabb::new(-8.2, -1.5, 9.8, 10.2).unwrap(),
            Aabb::new(1.5, 8.2, 9.8, 10.2).unwrap(),
            // Central Shaft entrance shell: x=-8..8, z=10..20.
            Aabb::new(-8.2, -7.8, 10.0, 20.0).unwrap(),
            Aabb::new(7.8, 8.2, 10.0, 20.0).unwrap(),
            Aabb::new(-8.0, 8.0, 19.8, 20.2).unwrap(),
        ];
        if !gate_open {
            walls.push(Aabb::new(-1.4, 1.4, 7.75, 8.25).unwrap());
        }
        Self { bounds, walls }
    }

    pub fn can_occupy(&self, position: Vec3, radius_m: f32) -> bool {
        position.x_m.is_finite()
            && position.y_m.is_finite()
            && position.z_m.is_finite()
            && radius_m.is_finite()
            && radius_m > 0.0
            && position.y_m >= 0.0
            && position.y_m <= 3.0
            && position.x_m - radius_m >= self.bounds.min_x
            && position.x_m + radius_m <= self.bounds.max_x
            && position.z_m - radius_m >= self.bounds.min_z
            && position.z_m + radius_m <= self.bounds.max_z
            && !self
                .walls
                .iter()
                .any(|wall| wall.contains_circle(position.x_m, position.z_m, radius_m))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KccBody {
    pub position_m: Vec3,
    pub velocity_mps: Vec3,
    pub radius_m: f32,
    pub grounded: bool,
    pub move_speed_mps: f32,
    pub jump_speed_mps: f32,
    pub gravity_mps2: f32,
    pub dash_speed_mps: f32,
    pub dash_remaining_ms: u64,
    pub dash_cooldown_remaining_ms: u64,
}

impl KccBody {
    pub fn new(position_m: Vec3) -> Self {
        Self {
            position_m,
            velocity_mps: Vec3::zero(),
            radius_m: 0.35,
            grounded: true,
            move_speed_mps: 4.0,
            jump_speed_mps: 5.5,
            gravity_mps2: 16.0,
            dash_speed_mps: 10.0,
            dash_remaining_ms: 0,
            dash_cooldown_remaining_ms: 0,
        }
    }
    pub fn try_jump(&mut self) -> bool {
        if !self.grounded {
            false
        } else {
            self.velocity_mps.y_m = self.jump_speed_mps;
            self.grounded = false;
            true
        }
    }
    pub fn try_dash(&mut self) -> bool {
        if self.dash_remaining_ms > 0 || self.dash_cooldown_remaining_ms > 0 {
            false
        } else {
            self.dash_remaining_ms = 180;
            self.dash_cooldown_remaining_ms = 700;
            true
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollisionEvent {
    pub axis: String,
    pub position_m: Vec3,
}

pub fn step_kcc(
    body: &mut KccBody,
    world: &StaticKccWorld,
    axes: (f32, f32),
    dt_s: f32,
) -> Result<Vec<CollisionEvent>, KccError> {
    if !dt_s.is_finite() || dt_s <= 0.0 || dt_s > 0.1 {
        return Err(KccError::InvalidDelta);
    }
    let dt_ms = (dt_s * 1000.0).round() as u64;
    body.dash_cooldown_remaining_ms = body.dash_cooldown_remaining_ms.saturating_sub(dt_ms);
    let speed = if body.dash_remaining_ms > 0 {
        body.dash_speed_mps
    } else {
        body.move_speed_mps
    };
    body.dash_remaining_ms = body.dash_remaining_ms.saturating_sub(dt_ms);
    body.velocity_mps.x_m = axes.0 * speed;
    body.velocity_mps.z_m = axes.1 * speed;
    if !body.grounded {
        body.velocity_mps.y_m -= body.gravity_mps2 * dt_s;
    }
    let mut collisions = vec![];
    let mut next_x = body.position_m.x_m + body.velocity_mps.x_m * dt_s;
    if next_x - body.radius_m < world.bounds.min_x
        || next_x + body.radius_m > world.bounds.max_x
        || world
            .walls
            .iter()
            .any(|w| w.contains_circle(next_x, body.position_m.z_m, body.radius_m))
    {
        next_x = body.position_m.x_m;
        body.velocity_mps.x_m = 0.0;
        collisions.push(CollisionEvent {
            axis: "x".into(),
            position_m: body.position_m,
        });
    }
    body.position_m.x_m = next_x;
    let mut next_z = body.position_m.z_m + body.velocity_mps.z_m * dt_s;
    if next_z - body.radius_m < world.bounds.min_z
        || next_z + body.radius_m > world.bounds.max_z
        || world
            .walls
            .iter()
            .any(|w| w.contains_circle(body.position_m.x_m, next_z, body.radius_m))
    {
        next_z = body.position_m.z_m;
        body.velocity_mps.z_m = 0.0;
        collisions.push(CollisionEvent {
            axis: "z".into(),
            position_m: body.position_m,
        });
    }
    body.position_m.z_m = next_z;
    body.position_m.y_m += body.velocity_mps.y_m * dt_s;
    if body.position_m.y_m <= 0.0 {
        body.position_m.y_m = 0.0;
        body.velocity_mps.y_m = 0.0;
        body.grounded = true;
    }
    Ok(collisions)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KccError {
    InvalidWorld,
    InvalidDelta,
}

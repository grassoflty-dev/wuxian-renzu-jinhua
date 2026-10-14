use wuxian_horror_ch1::{
    effects::TerrainTag,
    player_rules::{EffectivePlayerRules, MovementMode, MovementContext, FULL_DAMAGE_BPS},
};
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

    fn contains_point(&self, x: f32, z: f32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }
}

const WATER_WALK_SPEED_FACTOR: f32 = 0.6;

#[derive(Clone, Debug, PartialEq)]
pub struct StaticKccWorld {
    pub bounds: Aabb,
    pub walls: Vec<Aabb>,
    walkable_polygons: Vec<Vec<[f32; 2]>>,
    terrain_regions: Vec<KccTerrainRegion>,
    moving_supports: Vec<crate::moving_support::MovingSupportDefinition>,
    standing_decks: Vec<crate::moving_support::StandingDeckDefinition>,
    movement_slow_zones: Vec<Aabb>,
    movement_mode: MovementMode,
    effective_rules: EffectivePlayerRules,
}

#[derive(Clone,Copy)]
enum SupportContact<'a> {
    Moving(&'a crate::moving_support::MovingSupportDefinition),
    Standing(&'a crate::moving_support::StandingDeckDefinition),
}
impl SupportContact<'_> {
    fn id(&self)->&str {match self {Self::Moving(s)=>&s.id,Self::Standing(s)=>&s.id}}
    fn rank(&self)->u8 {match self {Self::Standing(_)=>0,Self::Moving(_)=>1}}
    fn supports(&self,position:Vec3,radius:f32)->bool {match self {Self::Moving(s)=>s.supports(position,radius),Self::Standing(s)=>s.supports(position,radius)}}
    fn height_at(&self,time:u64)->Result<f32,KccError> {match self {
        Self::Moving(s)=>s.pose_at(time).map(|p|p.height_m).map_err(|_|KccError::InvalidWorld),
        Self::Standing(s)=>Ok(s.height_m),
    }}
}

#[derive(Clone, Debug, PartialEq)]
pub struct KccTerrainRegion {
    pub id: String,
    pub tag: TerrainTag,
    pub polygon: Vec<[f32; 2]>,
    pub surface_velocity_mps: Option<[f32; 2]>,
}
impl StaticKccWorld {
    pub fn new(bounds: Aabb, walls: Vec<Aabb>) -> Self {
        Self {
            bounds,
            walls,
            walkable_polygons: Vec::new(),
            terrain_regions: Vec::new(),
            moving_supports: Vec::new(),
            standing_decks: Vec::new(),
            movement_slow_zones: Vec::new(),
            movement_mode: MovementMode::Ground,
            effective_rules: EffectivePlayerRules::default(),
        }
    }

    pub fn with_walkable_polygons(mut self, polygons: Vec<Vec<[f32; 2]>>) -> Self {
        self.walkable_polygons = polygons;
        self
    }

    pub fn with_terrain_regions(mut self, regions: Vec<KccTerrainRegion>) -> Self {
        self.terrain_regions = regions;
        self
    }

    pub fn with_moving_supports(mut self, definitions: Vec<crate::moving_support::MovingSupportDefinition>) -> Result<Self,KccError> {
        crate::moving_support::validate_support_catalog(&definitions,&self.standing_decks,self.bounds).map_err(|_|KccError::InvalidWorld)?;
        self.moving_supports=definitions;
        Ok(self)
    }

    /// Actors in this bounded model remain on their authored floor or static
    /// deck. They neither board moving lifts nor acquire unsupported flight.
    /// Floor motion retains each controller's existing collision checks.
    pub fn stable_actor_footprint(&self, position: Vec3, radius: f32) -> bool {
        [position.x_m,position.y_m,position.z_m,radius].iter().all(|n|n.is_finite())
            && radius>0.0 && (0.0..=3.0).contains(&position.y_m) && if position.y_m.abs()<=0.001 {true} else {
                self.can_occupy(position,radius) && self.standing_decks.iter().any(|deck|
                    (deck.height_m-position.y_m).abs()<=0.001 && deck.supports(position,radius))
            }
    }

    pub fn has_moving_supports(&self) -> bool { !self.moving_supports.is_empty() }
    pub fn has_support_surfaces(&self)->bool {self.has_moving_supports() || !self.standing_decks.is_empty()}
    pub fn with_standing_decks(mut self,definitions:Vec<crate::moving_support::StandingDeckDefinition>)->Result<Self,KccError> {
        crate::moving_support::validate_support_catalog(&self.moving_supports,&definitions,self.bounds)
            .map_err(|_|KccError::InvalidWorld)?;
        self.standing_decks=definitions;Ok(self)
    }
    fn support_contacts(&self)->impl Iterator<Item=SupportContact<'_>> {
        self.standing_decks.iter().map(SupportContact::Standing)
            .chain(self.moving_supports.iter().map(SupportContact::Moving))
    }
    fn supporting_contact(&self,position:Vec3,radius:f32,now_ms:u64)->Option<SupportContact<'_>> {
        self.support_contacts().filter(|s|s.supports(position,radius)
            && s.height_at(now_ms).is_ok_and(|height|(height-position.y_m).abs()<=0.001))
            // A coplanar stationary deck cannot pull its rider down through its
            // solid top when an overlapping docked lift begins to descend.
            .min_by(|a,b|a.rank().cmp(&b.rank()).then_with(||a.id().cmp(b.id())))
    }


    pub fn validate_saved_support_frame(&self, saved: Option<&crate::moving_support::SupportFrame>,
        epoch: u64, now_ms: u64) -> bool {
        match saved {
            Some(value) => self.has_support_surfaces()
                && crate::moving_support::validate_frame_with_standing(value,&self.moving_supports,&self.standing_decks,epoch,now_ms).is_ok(),
            None => !self.has_support_surfaces(),
        }
    }

    pub fn support_rider_view(&self,body:&KccBody,now_ms:u64,traversing:bool)->Result<crate::moving_support::SupportRiderView,KccError> {
        use crate::moving_support::{SupportRiderMode as Mode,SupportRiderView};
        if ![body.position_m.x_m,body.position_m.y_m,body.position_m.z_m,body.radius_m].iter().all(|n|n.is_finite())
            || body.radius_m<=0.0
            || !(0.0..=3.0).contains(&body.position_m.y_m) {return Err(KccError::InvalidWorld);}
        let (mode,support_id)=if traversing {(Mode::Traversal,None)} else if !body.grounded {(Mode::Airborne,None)}
            else if let Some(support)=self.supporting_contact(body.position_m,body.radius_m,now_ms) {(Mode::Surface,Some(support.id().to_owned()))}
            else if body.position_m.y_m.abs()<=0.001 {(Mode::Floor,None)} else {return Err(KccError::InvalidWorld);};
        Ok(SupportRiderView{position_m:body.position_m,radius_m:body.radius_m,mode,support_id})
    }

    pub fn support_frame(&self, world_epoch: u64, now_ms: u64) -> Result<crate::moving_support::SupportFrame,KccError> {
        crate::moving_support::frame_with_standing(&self.moving_supports,&self.standing_decks,world_epoch,now_ms).map_err(|_|KccError::InvalidWorld)
    }

    /// Save restoration must not create a grounded body floating between floors.
    pub fn valid_saved_support_contact(&self, body: &KccBody, now_ms: u64) -> bool {
        if self.has_support_surfaces() && !(crate::moving_support::FLOOR_M..=crate::moving_support::CEILING_M)
            .contains(&body.position_m.y_m) {return false;}
        if !self.has_support_surfaces() || !body.grounded || body.position_m.y_m.abs()<=0.001 {return true;}
        self.supporting_contact(body.position_m,body.radius_m,now_ms).is_some()
    }

    pub fn with_movement_rules(
        mut self,
        rules: EffectivePlayerRules,
        mode: MovementMode,
    ) -> Self {
        self.effective_rules = rules;
        self.movement_mode = mode;
        self
    }

    pub fn set_movement_rules(&mut self, rules: EffectivePlayerRules, mode: MovementMode) {
        self.effective_rules = rules;
        self.movement_mode = mode;
    }

    /// Shared resolved hazard resistance for authoritative actor attacks.
    pub(crate) fn remaining_hazard_damage_bps(&self, tag: crate::effects::HazardTag) -> u16 {
        self.effective_rules.hazards.remaining_damage_bps(tag)
    }

    pub fn map_terrain_tags(&mut self, mut resolve: impl FnMut(&str, TerrainTag) -> TerrainTag) {
        for region in &mut self.terrain_regions {
            region.tag = resolve(&region.id, region.tag);
        }
    }

    /// Read-only resolved terrain under a finite planar footprint. This does not
    /// consult or grant player movement capabilities, or modify pump state.
    pub fn intersects_terrain_at(&self, position: Vec3, radius_m: f32, tags: &[TerrainTag]) -> bool {
        [position.x_m, position.y_m, position.z_m, radius_m].into_iter().all(f32::is_finite)
            && radius_m > 0.0 && crate::continuous_combat::floor_height_overlap(position.y_m)
            && self.terrain_regions.iter().any(|region| {
                tags.contains(&region.tag) && region.polygon.len() >= 3
                    && region.polygon.iter().flatten().all(|v| v.is_finite())
                    && circle_intersects_polygon(position.x_m, position.z_m, radius_m, &region.polygon)
            })
    }

    pub fn terrain_tag(&self, region_id: &str) -> Option<TerrainTag> {
        self.terrain_regions
            .iter()
            .find(|region| region.id == region_id)
            .map(|region| region.tag)
    }

    pub fn with_movement_slow_zones(mut self, zones: Vec<Aabb>) -> Self {
        self.movement_slow_zones = zones;
        self
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
        Self::new(bounds, walls)
    }

    pub fn can_occupy(&self, position: Vec3, radius_m: f32) -> bool {
        self.can_occupy_with_rules(position, radius_m, self.movement_mode, &self.effective_rules)
    }

    pub fn can_occupy_with_rules(
        &self,
        position: Vec3,
        radius_m: f32,
        mode: MovementMode,
        rules: &EffectivePlayerRules,
    ) -> bool {
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
            && (self.walkable_polygons.is_empty()
                || self.walkable_polygons.iter().any(|polygon| {
                    circle_inside_polygon(position.x_m, position.z_m, radius_m, polygon)
                }))
            && !self
                .walls
                .iter()
                .any(|wall| wall.contains_circle(position.x_m, position.z_m, radius_m))
            && !self.terrain_regions.iter().any(|region| {
                circle_intersects_polygon(position.x_m, position.z_m, radius_m, &region.polygon)
                    && !rules
                        .resolve_movement(
                            mode,
                            MovementContext {
                                terrain: Some(region.tag),
                                ..MovementContext::default()
                            },
                        )
                        .allowed
            })
    }
}

fn point_in_polygon(x: f32, z: f32, polygon: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        if (a[1] > z) != (b[1] > z) && x < (b[0] - a[0]) * (z - a[1]) / (b[1] - a[1]) + a[0] {
            inside = !inside;
        }
    }
    inside
}

fn distance_to_segment_sq(x: f32, z: f32, a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dz = b[1] - a[1];
    let length_sq = dx * dx + dz * dz;
    let t = if length_sq <= f32::EPSILON {
        0.0
    } else {
        (((x - a[0]) * dx + (z - a[1]) * dz) / length_sq).clamp(0.0, 1.0)
    };
    (x - (a[0] + t * dx)).powi(2) + (z - (a[1] + t * dz)).powi(2)
}

pub(crate) fn circle_inside_polygon(x: f32, z: f32, radius: f32, polygon: &[[f32; 2]]) -> bool {
    polygon.len() >= 3
        && point_in_polygon(x, z, polygon)
        && (0..polygon.len()).all(|index| {
            distance_to_segment_sq(x, z, polygon[index], polygon[(index + 1) % polygon.len()])
                >= radius * radius
        })
}

fn circle_intersects_polygon(x: f32, z: f32, radius: f32, polygon: &[[f32; 2]]) -> bool {
    point_in_polygon(x, z, polygon)
        || (0..polygon.len()).any(|index| {
            distance_to_segment_sq(x, z, polygon[index], polygon[(index + 1) % polygon.len()])
                <= radius * radius
        })
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
        self.try_dash_for(180, 700)
    }

    pub fn try_dash_for(&mut self, active_ms: u64, cooldown_ms: u64) -> bool {
        if self.dash_remaining_ms > 0 || self.dash_cooldown_remaining_ms > 0 {
            false
        } else if active_ms == 0 {
            false
        } else {
            self.dash_remaining_ms = active_ms;
            self.dash_cooldown_remaining_ms = cooldown_ms;
            true
        }
    }

    pub fn try_dash_for_distance(
        &mut self,
        active_ms: u64,
        cooldown_ms: u64,
        distance_m: f32,
    ) -> bool {
        if !distance_m.is_finite()
            || distance_m <= 0.0
            || !self.try_dash_for(active_ms, cooldown_ms)
        {
            return false;
        }
        self.dash_speed_mps = distance_m * 1000.0 / active_ms as f32;
        true
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
    step_kcc_with_surface_motion(body, world, axes, dt_s, true)
}

/// Context traversal already owns its exact authored path for this tick.
/// Ordinary walking/dashing receives support velocity only while grounded.
pub fn step_kcc_with_surface_motion(
    body: &mut KccBody, world: &StaticKccWorld, axes: (f32, f32), dt_s: f32,
    allow_surface_motion: bool,
) -> Result<Vec<CollisionEvent>, KccError> {
    step_kcc_internal(body,world,axes,dt_s,allow_surface_motion,None)
}

pub fn step_kcc_at_world_time(
    body: &mut KccBody, world: &StaticKccWorld, axes: (f32,f32), dt_s: f32,
    allow_surface_motion: bool, now_ms: u64,
) -> Result<Vec<CollisionEvent>,KccError> {
    step_kcc_internal(body,world,axes,dt_s,allow_surface_motion,Some(now_ms))
}

fn step_kcc_internal(
    body: &mut KccBody, world: &StaticKccWorld, axes: (f32,f32), dt_s: f32,
    allow_surface_motion: bool, now_ms: Option<u64>,
) -> Result<Vec<CollisionEvent>,KccError> {
    if !dt_s.is_finite() || dt_s <= 0.0 || dt_s > 0.1 {
        return Err(KccError::InvalidDelta);
    }
    if !world.moving_supports.is_empty() && now_ms.is_none() {return Err(KccError::InvalidWorld);}
    let dt_ms=(dt_s*1000.0).round() as u64;
    let now_ms=now_ms.unwrap_or(0);
    let next_ms=now_ms.checked_add(dt_ms).ok_or(KccError::InvalidDelta)?;
    let previous_y=body.position_m.y_m;
    let mut carrying=if allow_surface_motion && body.grounded {
        world.supporting_contact(body.position_m,body.radius_m,now_ms)
    } else {None};
    for region in &world.terrain_regions {
        if let Some(velocity) = region.surface_velocity_mps {
            if region.tag != TerrainTag::Conveyor || !velocity.iter().all(|v| v.is_finite())
                || !(0.1..=4.0).contains(&velocity[0].hypot(velocity[1])) {
                return Err(KccError::InvalidWorld);
            }
        } else if region.tag == TerrainTag::Conveyor {
            return Err(KccError::InvalidWorld);
        }
    }
    // A point has one supporting surface. Stable ID order resolves overlaps,
    // never summing regions into an accidental speed boost.
    let surface = if allow_surface_motion && carrying.is_none() && body.grounded && body.position_m.y_m.abs()<=0.001 {
        world.terrain_regions.iter()
            .filter(|region| region.surface_velocity_mps.is_some()
                && point_in_polygon(body.position_m.x_m, body.position_m.z_m, &region.polygon)
                && !world.effective_rules.resolve_movement(world.movement_mode,
                    MovementContext { terrain: Some(region.tag), ..MovementContext::default() })
                    .terrain_penalty_ignored)
            .min_by(|a,b| a.id.cmp(&b.id))
            .and_then(|region| region.surface_velocity_mps).unwrap_or([0.0, 0.0])
    } else { [0.0, 0.0] };
    if allow_surface_motion && world.has_support_surfaces() && body.grounded && previous_y>0.001 && carrying.is_none() {
        body.grounded=false;
    }
    if let Some(support)=carrying {
        let mut target=support.height_at(next_ms)?;
        if matches!(support,SupportContact::Moving(_)) && target<previous_y {
            if let Some(deck)=world.standing_decks.iter().filter(|deck|deck.supports(body.position_m,body.radius_m)
                && deck.height_m>=target && deck.height_m<=previous_y)
                .max_by(|a,b|a.height_m.total_cmp(&b.height_m).then_with(||b.id.cmp(&a.id))) {
                target=deck.height_m;carrying=Some(SupportContact::Standing(deck));
            }
        }
        let mut candidate=body.position_m;candidate.y_m=target;
        if !world.can_occupy(candidate,body.radius_m) {return Err(KccError::InvalidWorld);}
        body.velocity_mps.y_m=if matches!(carrying,Some(SupportContact::Standing(_))) {0.0} else {(target-previous_y)/dt_s};
        body.position_m.y_m=target;
    }
    body.dash_cooldown_remaining_ms = body.dash_cooldown_remaining_ms.saturating_sub(dt_ms);
    let dash_was_active = body.dash_remaining_ms > 0;
    let horizontal_dt_s = if dash_was_active {
        dt_ms.min(body.dash_remaining_ms) as f32 / 1000.0
    } else {
        dt_s
    };
    let terrain_multiplier_bps = world
        .terrain_regions
        .iter()
        .filter(|region| crate::continuous_combat::floor_height_overlap(body.position_m.y_m)
            && point_in_polygon(body.position_m.x_m, body.position_m.z_m, &region.polygon))
        .map(|region| {
            world
                .effective_rules
                .resolve_movement(
                    world.movement_mode,
                    MovementContext {
                        terrain: Some(region.tag),
                        ..MovementContext::default()
                    },
                )
                .movement_multiplier_bps
        })
        .min()
        .unwrap_or(FULL_DAMAGE_BPS);
    let legacy_water_zone = world
        .movement_slow_zones
        .iter()
        .any(|zone| crate::continuous_combat::floor_height_overlap(body.position_m.y_m)
            && zone.contains_point(body.position_m.x_m, body.position_m.z_m));
    let multiplier_bps = if legacy_water_zone {
        terrain_multiplier_bps.min((WATER_WALK_SPEED_FACTOR * FULL_DAMAGE_BPS as f32) as u16)
    } else {
        terrain_multiplier_bps
    };
    let speed = if dash_was_active {
        body.dash_speed_mps
    } else {
        body.move_speed_mps * f32::from(multiplier_bps) / f32::from(FULL_DAMAGE_BPS)
    };
    body.dash_remaining_ms = body.dash_remaining_ms.saturating_sub(dt_ms);
    if dash_was_active && body.dash_remaining_ms == 0 {
        body.dash_speed_mps = 10.0;
    }
    body.velocity_mps.x_m = axes.0 * speed + surface[0];
    body.velocity_mps.z_m = axes.1 * speed + surface[1];
    if !allow_surface_motion {body.velocity_mps.y_m=0.0;}
    if !body.grounded && allow_surface_motion {
        body.velocity_mps.y_m -= body.gravity_mps2 * dt_s;
    }
    let mut collisions = vec![];
    let mut next_x = body.position_m.x_m + axes.0 * speed * horizontal_dt_s + surface[0] * dt_s;
    let mut x_candidate = body.position_m;
    x_candidate.x_m = next_x;
    if !motion_segment_clear(world, body.position_m, x_candidate, body.radius_m, surface != [0.0, 0.0] || world.has_support_surfaces()) {
        next_x = body.position_m.x_m;
        body.velocity_mps.x_m = 0.0;
        collisions.push(CollisionEvent {
            axis: "x".into(),
            position_m: body.position_m,
        });
    }
    body.position_m.x_m = next_x;
    let mut next_z = body.position_m.z_m + axes.1 * speed * horizontal_dt_s + surface[1] * dt_s;
    let mut z_candidate = body.position_m;
    z_candidate.z_m = next_z;
    if !motion_segment_clear(world, body.position_m, z_candidate, body.radius_m, surface != [0.0, 0.0] || world.has_support_surfaces()) {
        next_z = body.position_m.z_m;
        body.velocity_mps.z_m = 0.0;
        collisions.push(CollisionEvent {
            axis: "z".into(),
            position_m: body.position_m,
        });
    }
    body.position_m.z_m = next_z;
    // Authored traversal has already installed this tick's exact 3D point.
    if !allow_surface_motion {return Ok(collisions);}
    let retained=carrying.is_some_and(|support|support.supports(body.position_m,body.radius_m));
    if carrying.is_some() && !retained {
        // Carry and free motion cover one tick, not two stacked vertical steps.
        body.position_m.y_m=previous_y;
        body.grounded=false;
        body.velocity_mps.y_m-=body.gravity_mps2*dt_s;
    }
    if !retained {
        body.position_m.y_m += body.velocity_mps.y_m * dt_s;
        if allow_surface_motion {
            let landing=world.support_contacts().filter(|support|support.supports(body.position_m,body.radius_m))
                .filter_map(|support|Some((support.height_at(now_ms).ok()?,support.height_at(next_ms).ok()?)))
                .filter(|(old,next)|previous_y>=*old-0.001 && body.position_m.y_m<=*next)
                .max_by(|a,b|a.1.total_cmp(&b.1));
            if let Some((_,height))=landing {
                body.position_m.y_m=height;body.velocity_mps.y_m=0.0;body.grounded=true;
            }
        }
        if body.position_m.y_m<=crate::moving_support::FLOOR_M {
            body.position_m.y_m=crate::moving_support::FLOOR_M;body.velocity_mps.y_m=0.0;body.grounded=true;
        }
        if world.has_support_surfaces() && body.position_m.y_m>crate::moving_support::CEILING_M {
            body.position_m.y_m=crate::moving_support::CEILING_M;body.velocity_mps.y_m=0.0;body.grounded=false;
            collisions.push(CollisionEvent{axis:"ceiling".into(),position_m:body.position_m});
        }
    }
    Ok(collisions)
}

// Sweep carried movement so even dash plus a fast belt cannot tunnel a thin wall.
// Non-surface movement retains its existing collision path in this bounded slice.
fn motion_segment_clear(world: &StaticKccWorld, from: Vec3, to: Vec3, radius: f32, swept: bool) -> bool {
    let distance = (to.x_m - from.x_m).hypot(to.z_m - from.z_m);
    let steps = if swept { (distance / 0.05).ceil().max(1.0) as usize } else { 1 };
    (1..=steps).all(|step| {
        let t = step as f32 / steps as f32;
        world.can_occupy(Vec3 { x_m: from.x_m + (to.x_m - from.x_m) * t,
            y_m: from.y_m, z_m: from.z_m + (to.z_m - from.z_m) * t }, radius)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KccError {
    InvalidWorld,
    InvalidDelta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_zone_slows_walking_only_while_inside_it() {
        let world = StaticKccWorld::new(Aabb::new(-10.0, 10.0, -10.0, 10.0).unwrap(), vec![])
            .with_movement_slow_zones(vec![Aabb::new(0.0, 5.0, 0.0, 5.0).unwrap()]);
        let mut body = KccBody::new(Vec3::new(-2.0, 0.0, 2.0).unwrap());
        step_kcc(&mut body, &world, (1.0, 0.0), 0.05).unwrap();
        assert_eq!(body.velocity_mps.x_m, 4.0);

        body.position_m.x_m = 2.0;
        step_kcc(&mut body, &world, (1.0, 0.0), 0.05).unwrap();
        assert_eq!(body.velocity_mps.x_m, 2.4);

        body.position_m.x_m = 6.0;
        step_kcc(&mut body, &world, (1.0, 0.0), 0.05).unwrap();
        assert_eq!(body.velocity_mps.x_m, 4.0);

        body.position_m.x_m = 2.0;
        assert!(body.try_dash());
        step_kcc(&mut body, &world, (1.0, 0.0), 0.05).unwrap();
        assert_eq!(body.velocity_mps.x_m, 10.0);
    }
}

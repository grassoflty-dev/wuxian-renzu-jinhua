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
    player_escape_walls: Vec<(String,Aabb)>,
    walkable_polygons: Vec<Vec<[f32; 2]>>,
    terrain_regions: Vec<KccTerrainRegion>,
    moving_supports: Vec<crate::moving_support::MovingSupportDefinition>,
    standing_decks: Vec<crate::moving_support::StandingDeckDefinition>,
    shaft_step: Option<ShaftStep>,
    gate_shaft_upper_union: bool,
    gate_decon_union: bool,
    beacon_exit_union: bool,
    hive_route_union: bool,
    mh_shore_support: Option<[Aabb; 3]>,
    cw_early_union: bool,
    shaft_lockdown_union: bool,
    actor_scene_bounds: Option<Aabb>,
    movement_slow_zones: Vec<Aabb>,
    movement_mode: MovementMode,
    effective_rules: EffectivePlayerRules,
}

/// One owner-validated Shaft movement interval. No other scene can construct
/// this through the generic support APIs.
#[derive(Clone,Debug,PartialEq)]
struct ShaftStep {
    before:crate::formal_runtime::shaft_lift::ShaftOwner,
    after:crate::formal_runtime::shaft_lift::ShaftOwner,
    /// Shaft local origin in this KCC's coordinates.
    offset:[f32;2],
}
impl ShaftStep {
    fn local(&self,mut p:Vec3)->Vec3 {
        p.x_m-=self.offset[0];p.z_m-=self.offset[1];p
    }
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
            player_escape_walls: Vec::new(),
            walkable_polygons: Vec::new(),
            terrain_regions: Vec::new(),
            moving_supports: Vec::new(),
            standing_decks: Vec::new(),
            shaft_step: None,
            gate_shaft_upper_union: false,
            gate_decon_union: false,
            beacon_exit_union: false,
            hive_route_union: false,
            mh_shore_support: None,
            cw_early_union: false,
            shaft_lockdown_union: false,
            actor_scene_bounds: None,
            movement_slow_zones: Vec::new(),
            movement_mode: MovementMode::Ground,
            effective_rules: EffectivePlayerRules::default(),
        }
    }

    /// Exact rectangular union for the validated Grey Hive Power/Gate pair.
    /// Reject unsupported per-scene terrain/supports instead of silently dropping them.
    pub(crate) fn join_grey_hive_pair(&self, other: &Self, other_x: f32) -> Result<Self, KccError> {
        if ![20.0_f32, -20.0].contains(&other_x)
            || self.bounds != Aabb::new(0.0,20.0,0.0,14.0)?
            || other.bounds != Aabb::new(0.0,20.0,0.0,14.0)?
            || !self.walkable_polygons.is_empty() || !other.walkable_polygons.is_empty()
            || !self.terrain_regions.is_empty() || !other.terrain_regions.is_empty()
            || !self.moving_supports.is_empty() || !other.moving_supports.is_empty()
            || !self.standing_decks.is_empty() || !other.standing_decks.is_empty()
            || self.shaft_step.is_some() || other.shaft_step.is_some()
            || !self.movement_slow_zones.is_empty() || !other.movement_slow_zones.is_empty()
        { return Err(KccError::InvalidWorld); }
        let mut joined = self.clone();
        joined.bounds = Aabb::new(other_x.min(0.0), 20.0 + other_x.max(0.0), 0.0, 14.0)?;
        for wall in &other.walls {
            joined.walls.push(Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?);
        }
        Ok(joined)
    }

    /// Exact union of the selected Gate A and Shaft upper floor rectangles.
    /// Both callers must supply their already-authorized scene KCCs. The
    /// non-rectangular outline excludes the two strips outside Gate A.
    pub(crate) fn join_gate_shaft_upper(&self,other:&Self,self_is_gate:bool,
        now_ms:u64,next_ms:u64)->Result<Self,KccError> {
        let (gate,shaft)=if self_is_gate {(self,other)} else {(other,self)};
        if gate.bounds!=Aabb::new(0.0,20.0,0.0,14.0)?
            || shaft.bounds!=Aabb::new(0.0,24.0,0.0,16.0)?
            || gate.shaft_step.is_some()
            || gate.movement_mode!=shaft.movement_mode || gate.effective_rules!=shaft.effective_rules
            || !gate.walkable_polygons.is_empty() || !shaft.walkable_polygons.is_empty()
            || !gate.terrain_regions.is_empty() || !shaft.terrain_regions.is_empty()
            || !gate.moving_supports.is_empty() || !shaft.moving_supports.is_empty()
            || !gate.standing_decks.is_empty() || !shaft.standing_decks.is_empty()
            || !gate.movement_slow_zones.is_empty() || !shaft.movement_slow_zones.is_empty()
            || next_ms<now_ms || next_ms-now_ms>100 {return Err(KccError::InvalidWorld);}
        let mut joined=self.clone();
        joined.gate_shaft_upper_union=true;
        joined.actor_scene_bounds=Some(self.bounds);
        let (offset_x,offset_z)=if self_is_gate {(20.0,-1.0)} else {(-20.0,1.0)};
        joined.bounds=if self_is_gate {Aabb::new(0.0,44.0,-1.0,15.0)?}
            else {Aabb::new(-20.0,24.0,0.0,16.0)?};
        joined.walkable_polygons=vec![if self_is_gate {
            vec![[0.0,0.0],[20.0,0.0],[20.0,-1.0],[44.0,-1.0],[44.0,15.0],[20.0,15.0],[20.0,14.0],[0.0,14.0]]
        } else {
            vec![[-20.0,1.0],[0.0,1.0],[0.0,0.0],[24.0,0.0],[24.0,16.0],[0.0,16.0],[0.0,15.0],[-20.0,15.0]]
        }];
        for wall in &other.walls {
            joined.walls.push(Aabb::new(wall.min_x+offset_x,wall.max_x+offset_x,
                wall.min_z+offset_z,wall.max_z+offset_z)?);
        }
        for (id,wall) in &other.player_escape_walls {
            joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+offset_x,wall.max_x+offset_x,
                wall.min_z+offset_z,wall.max_z+offset_z)?));
        }
        if self_is_gate {
            joined.shaft_step=shaft.shaft_step.clone();
            if let Some(step)=&mut joined.shaft_step {
                step.offset=[20.0,-1.0];
                step.before.clock.rebase_world_time(now_ms).map_err(|_|KccError::InvalidWorld)?;
                step.after.clock.rebase_world_time(next_ms).map_err(|_|KccError::InvalidWorld)?;
            }
        }
        Ok(joined)
    }

    pub(crate) fn is_gate_shaft_upper_union(&self)->bool {self.gate_shaft_upper_union}

    /// Exact two-rectangle union for the selected Gate B / Decon pair only.
    pub(crate) fn join_gate_decon(&self,other:&Self,other_x:f32)->Result<Self,KccError> {
        if ![24.0_f32,-24.0].contains(&other_x) || self.bounds!=Aabb::new(0.0,24.0,0.0,16.0)?
            || other.bounds!=Aabb::new(0.0,24.0,0.0,16.0)? || self.movement_mode!=other.movement_mode
            || self.effective_rules!=other.effective_rules || !self.walkable_polygons.is_empty() || !other.walkable_polygons.is_empty()
            || !self.terrain_regions.is_empty() || !other.terrain_regions.is_empty() || !self.moving_supports.is_empty()
            || !other.moving_supports.is_empty() || !self.standing_decks.is_empty() || !other.standing_decks.is_empty()
            || self.shaft_step.is_some() || other.shaft_step.is_some() || !self.movement_slow_zones.is_empty()
            || !other.movement_slow_zones.is_empty() {return Err(KccError::InvalidWorld);}
        let mut joined=self.clone();joined.gate_decon_union=true;joined.actor_scene_bounds=Some(self.bounds);
        joined.bounds=Aabb::new(other_x.min(0.0),24.0+other_x.max(0.0),0.0,16.0)?;
        for wall in &other.walls {joined.walls.push(Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?);}
        for (id,wall) in &other.player_escape_walls {joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?));}
        Ok(joined)
    }
    pub(crate) fn is_gate_decon_union(&self)->bool {self.gate_decon_union}

    /// The Beacon/Exit pair has the same two validated 24x16 support
    /// rectangles as Gate B/Decon, but is a distinct owner and has no gate.
    /// Both caller-provided worlds must already reflect their selected variants.
    pub(crate) fn join_beacon_exit(&self,other:&Self,other_x:f32)->Result<Self,KccError> {
        let mut joined=self.join_gate_decon(other,other_x)?;
        joined.gate_decon_union=false;
        joined.beacon_exit_union=true;
        Ok(joined)
    }
    pub(crate) fn is_beacon_exit_union(&self)->bool {self.beacon_exit_union}

    /// Exact two-room support union for one selected same-floor Hive edge.
    /// All collision and legacy-escape walls remain in their original world
    /// positions after translation; actor movement stays within its own room.
    pub(crate) fn join_hive_route(&self,other:&Self,other_x:f32,width:f32,depth:f32)->Result<Self,KccError> {
        if ![(20.0_f32,14.0_f32),(24.0,16.0)].contains(&(width,depth))
            || ![width,-width].contains(&other_x)
            || self.bounds!=Aabb::new(0.0,width,0.0,depth)? || other.bounds!=self.bounds
            || self.movement_mode!=other.movement_mode || self.effective_rules!=other.effective_rules
            || !self.walkable_polygons.is_empty() || !other.walkable_polygons.is_empty()
            || !self.terrain_regions.is_empty() || !other.terrain_regions.is_empty()
            || !self.moving_supports.is_empty() || !other.moving_supports.is_empty()
            || !self.standing_decks.is_empty() || !other.standing_decks.is_empty()
            || self.shaft_step.is_some() || other.shaft_step.is_some()
            || !self.movement_slow_zones.is_empty() || !other.movement_slow_zones.is_empty() {
            return Err(KccError::InvalidWorld);
        }
        let mut joined=self.clone();
        joined.hive_route_union=true;
        joined.actor_scene_bounds=Some(self.bounds);
        joined.bounds=Aabb::new(other_x.min(0.0),width+other_x.max(0.0),0.0,depth)?;
        for wall in &other.walls {
            joined.walls.push(Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?);
        }
        for (id,wall) in &other.player_escape_walls {
            joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?));
        }
        Ok(joined)
    }
    pub(crate) fn is_hive_route_union(&self)->bool {self.hive_route_union}

    /// Selected Clockworks first-floor pair; default bounds remain the floor.
    /// Terrain and actual surface-velocity regions retain their source coordinates.
    pub(crate) fn join_cw_early(&self,other:&Self,other_x:f32)->Result<Self,KccError> {
        let bounds=Aabb::new(0.0,24.0,0.0,16.0)?;
        if ![-24.0_f32,24.0].contains(&other_x) || self.bounds!=bounds || other.bounds!=bounds
            || self.movement_mode!=other.movement_mode || self.effective_rules!=other.effective_rules
            || !self.walkable_polygons.is_empty() || !other.walkable_polygons.is_empty()
            || !self.moving_supports.is_empty() || !other.moving_supports.is_empty()
            || !self.standing_decks.is_empty() || !other.standing_decks.is_empty()
            || self.shaft_step.is_some() || other.shaft_step.is_some()
            || self.mh_shore_support.is_some() || other.mh_shore_support.is_some()
            || self.cw_early_union || other.cw_early_union {return Err(KccError::InvalidWorld);}
        let mut joined=self.clone();joined.hive_route_union=true;joined.cw_early_union=true;
        joined.actor_scene_bounds=Some(bounds);
        joined.bounds=Aabb::new(other_x.min(0.0),24.0+other_x.max(0.0),0.0,16.0)?;
        for wall in &other.walls {joined.walls.push(Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?);}
        for (id,wall) in &other.player_escape_walls {joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?));}
        for region in &other.terrain_regions {let mut shifted=region.clone();for p in &mut shifted.polygon {p[0]+=other_x;}joined.terrain_regions.push(shifted);}
        for zone in &other.movement_slow_zones {joined.movement_slow_zones.push(Aabb::new(zone.min_x+other_x,zone.max_x+other_x,zone.min_z,zone.max_z)?);}
        Ok(joined)
    }

    /// Boiler/Gear ground-only composition; the caller binds exact scene identities
    /// and grounded Floor contact before this private union is constructed.
    pub(crate) fn join_cw_gear_ground(&self,other:&Self,other_x:f32,self_is_gear:bool)->Result<Self,KccError> {
        let bounds=Aabb::new(0.0,24.0,0.0,16.0)?;
        let (gear,boiler)=if self_is_gear {(self,other)} else {(other,self)};
        if ![-24.0_f32,24.0].contains(&other_x) || self.bounds!=bounds || other.bounds!=bounds
            || self.movement_mode!=other.movement_mode || self.effective_rules!=other.effective_rules
            || !self.walkable_polygons.is_empty() || !other.walkable_polygons.is_empty()
            || self.shaft_step.is_some() || other.shaft_step.is_some()
            || self.mh_shore_support.is_some() || other.mh_shore_support.is_some()
            || self.cw_early_union || other.cw_early_union
            || !boiler.moving_supports.is_empty() || !boiler.standing_decks.is_empty()
            || gear.moving_supports.len()!=1 || gear.standing_decks.len()!=2
            || gear.moving_supports[0].id!="cw_gear_main_lift"
            || !gear.standing_decks.iter().any(|d|d.id=="cw_gear_upper_dock")
            || !gear.standing_decks.iter().any(|d|d.id=="cw_gear_furnace_landing") {return Err(KccError::InvalidWorld);}
        let mut joined=self.clone();joined.hive_route_union=true;joined.cw_early_union=true;
        joined.actor_scene_bounds=Some(bounds);
        joined.bounds=Aabb::new(other_x.min(0.0),24.0+other_x.max(0.0),0.0,16.0)?;
        for wall in &other.walls {joined.walls.push(Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?);}
        for (id,wall) in &other.player_escape_walls {joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?));}
        for region in &other.terrain_regions {let mut shifted=region.clone();for p in &mut shifted.polygon {p[0]+=other_x;}joined.terrain_regions.push(shifted);}
        for zone in &other.movement_slow_zones {joined.movement_slow_zones.push(Aabb::new(zone.min_x+other_x,zone.max_x+other_x,zone.min_z,zone.max_z)?);}
        for support in &other.moving_supports {let mut shifted=support.clone();for p in &mut shifted.polygon {p[0]+=other_x;}joined.moving_supports.push(shifted);}
        for deck in &other.standing_decks {let mut shifted=deck.clone();for p in &mut shifted.polygon {p[0]+=other_x;}joined.standing_decks.push(shifted);}
        crate::moving_support::validate_support_catalog(&joined.moving_supports,&joined.standing_decks,joined.bounds).map_err(|_|KccError::InvalidWorld)?;
        Ok(joined)
    }

    /// One pre-kill Arena ingress exception is private to the player; the wall
    /// remains in the normal actor collision collection and standalone target.
    pub(crate) fn join_cw_later_ingress(&self,other:&Self,other_x:f32,west:bool)->Result<Self,KccError> {
        let mut joined=self.join_cw_early(other,other_x)?;
        let (id,local)=if west {("cw_arena_shutter_west_blocker",Aabb::new(0.5,1.0,6.0,9.5)?)}
            else {("cw_arena_shutter_east_blocker",Aabb::new(23.0,23.5,6.0,9.5)?)};
        if other.walls.iter().filter(|w|**w==local).count()!=1 {return Err(KccError::InvalidWorld);}
        let shifted=Aabb::new(local.min_x+other_x,local.max_x+other_x,local.min_z,local.max_z)?;
        if joined.player_escape_walls.iter().any(|(_,w)|*w==shifted) {return Err(KccError::InvalidWorld);}
        joined.player_escape_walls.push((id.into(),shifted));
        Ok(joined)
    }
    /// Two explicitly validated shore floors and their real, matching doorway.
    /// Original colliders, terrain, water slow zones and actor bounds remain owned.
    pub(crate) fn join_mh_shore(&self, other:&Self, other_x:f32)->Result<Self,KccError> {
        let bounds=Aabb::new(0.0,24.0,0.0,16.0)?;
        let floor=vec![[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]];
        if ![-24.0_f32,24.0].contains(&other_x) || self.bounds!=bounds || other.bounds!=bounds
            || self.movement_mode!=other.movement_mode || self.effective_rules!=other.effective_rules
            || self.walkable_polygons!=vec![floor.clone()] || other.walkable_polygons!=vec![floor]
            || self.mh_shore_support.is_some() || other.mh_shore_support.is_some()
            || !self.moving_supports.is_empty() || !other.moving_supports.is_empty()
            || !self.standing_decks.is_empty() || !other.standing_decks.is_empty()
            || self.shaft_step.is_some() || other.shaft_step.is_some() {return Err(KccError::InvalidWorld);}
        let mut joined=self.clone();
        joined.hive_route_union=true;
        joined.actor_scene_bounds=Some(bounds);
        joined.bounds=Aabb::new(other_x.min(0.0),24.0+other_x.max(0.0),0.0,16.0)?;
        let own=Aabb::new(0.5,23.5,0.5,15.5)?;
        let target=Aabb::new(other_x+0.5,other_x+23.5,0.5,15.5)?;
        let bridge=if other_x>0.0 {Aabb::new(23.5,24.5,6.0,9.5)?} else {Aabb::new(-0.5,0.5,6.0,9.5)?};
        joined.mh_shore_support=Some([own,bridge,target]);
        joined.walkable_polygons.extend(other.walkable_polygons.iter().map(|poly|poly.iter().map(|p|[p[0]+other_x,p[1]]).collect::<Vec<_>>()));
        for wall in &other.walls {joined.walls.push(Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?);}
        for (id,wall) in &other.player_escape_walls {joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+other_x,wall.max_x+other_x,wall.min_z,wall.max_z)?));}
        for region in &other.terrain_regions {let mut shifted=region.clone();for p in &mut shifted.polygon {p[0]+=other_x;}joined.terrain_regions.push(shifted);}
        for zone in &other.movement_slow_zones {joined.movement_slow_zones.push(Aabb::new(zone.min_x+other_x,zone.max_x+other_x,zone.min_z,zone.max_z)?);}
        Ok(joined)
    }

    /// Selected Shaft east / Lockdown west upper-floor union. Shaft support
    /// remains owned in Shaft coordinates; actors stay in their source scene.
    pub(crate) fn join_shaft_lockdown_upper(&self,other:&Self,self_is_shaft:bool,
        now:u64,next:u64)->Result<Self,KccError> {
        let (shaft,lockdown)=if self_is_shaft {(self,other)} else {(other,self)};
        if shaft.bounds!=Aabb::new(0.0,24.0,0.0,16.0)? || lockdown.bounds!=Aabb::new(0.0,24.0,0.0,16.0)?
            || lockdown.shaft_step.is_some() || shaft.movement_mode!=lockdown.movement_mode
            || shaft.effective_rules!=lockdown.effective_rules || !shaft.walkable_polygons.is_empty() || !lockdown.walkable_polygons.is_empty()
            || !shaft.terrain_regions.is_empty() || !lockdown.terrain_regions.is_empty() || !shaft.moving_supports.is_empty()
            || !lockdown.moving_supports.is_empty() || !shaft.standing_decks.is_empty() || !lockdown.standing_decks.is_empty()
            || !shaft.movement_slow_zones.is_empty() || !lockdown.movement_slow_zones.is_empty()
            || next<now || next-now>100 {return Err(KccError::InvalidWorld);}
        let offset:f32=if self_is_shaft {24.0} else {-24.0};
        let mut joined=self.clone();joined.shaft_lockdown_union=true;joined.actor_scene_bounds=Some(self.bounds);
        joined.bounds=Aabb::new(offset.min(0.0),24.0+offset.max(0.0),0.0,16.0)?;
        for wall in &other.walls {joined.walls.push(Aabb::new(wall.min_x+offset,wall.max_x+offset,wall.min_z,wall.max_z)?);}
        for (id,wall) in &other.player_escape_walls {joined.player_escape_walls.push((id.clone(),Aabb::new(wall.min_x+offset,wall.max_x+offset,wall.min_z,wall.max_z)?));}
        if !self_is_shaft {
            joined.shaft_step=shaft.shaft_step.clone();
            if let Some(step)=&mut joined.shaft_step {
                step.offset=[-24.0,0.0];
                step.before.clock.rebase_world_time(now).map_err(|_|KccError::InvalidWorld)?;
                step.after.clock.rebase_world_time(next).map_err(|_|KccError::InvalidWorld)?;
            }
        }
        Ok(joined)
    }
    pub(crate) fn is_shaft_lockdown_union(&self)->bool {self.shaft_lockdown_union}
    pub(crate) fn retime_lockdown_shaft(&mut self,now:u64,next:u64)->Result<(),KccError> {
        if !self.shaft_lockdown_union || next<now || next-now>100 {return Err(KccError::InvalidWorld);}
        if let Some(step)=&mut self.shaft_step {
            if step.offset!=[-24.0,0.0] {return Err(KccError::InvalidWorld);}
            step.before=step.after.clone();
            step.before.clock.rebase_world_time(now).map_err(|_|KccError::InvalidWorld)?;
            step.after.clock.rebase_world_time(next).map_err(|_|KccError::InvalidWorld)?;
        }
        Ok(())
    }

    /// While a Gate A footprint still straddles the seam, keep the last
    /// verified unloaded pose stationary while restricting movement inward.
    pub(crate) fn retime_unloaded_shaft(&mut self,now:u64,next:u64)->Result<(),KccError> {
        if !self.gate_shaft_upper_union || next<now || next-now>100 {return Err(KccError::InvalidWorld);}
        if let Some(step)=&mut self.shaft_step {
            if step.offset!=[20.0,-1.0] {return Err(KccError::InvalidWorld);}
            step.before=step.after.clone();
            step.before.clock.rebase_world_time(now).map_err(|_|KccError::InvalidWorld)?;
            step.after.clock.rebase_world_time(next).map_err(|_|KccError::InvalidWorld)?;
        }
        Ok(())
    }

    pub fn with_walkable_polygons(mut self, polygons: Vec<Vec<[f32; 2]>>) -> Self {
        self.walkable_polygons = polygons;
        self
    }

    /// Installs only a pair of already-admitted authority states for one tick.
    /// This does not admit a scene, fabricate a clock, or grant readiness.
    pub(crate) fn with_shaft_step(mut self,scene:&crate::scene_runtime::SceneDefinition,
        before:crate::formal_runtime::shaft_lift::ShaftOwner,
        after:crate::formal_runtime::shaft_lift::ShaftOwner)->Result<Self,KccError> {
        let geometry=if before.geometry.service_v2 {
            crate::formal_runtime::shaft_lower_service::candidate_geometry(scene)
        } else {crate::formal_runtime::shaft_lift::ShaftGeometry::from_scene(scene)};
        let Some(geometry)=geometry else{return Err(KccError::InvalidWorld);};
        let Some(dt)=after.clock.world_ms().checked_sub(before.clock.world_ms()) else {return Err(KccError::InvalidWorld);};
        if before.geometry!=geometry || after.geometry!=geometry || dt>100 || (dt==0 && before!=after)
            || !self.moving_supports.is_empty() || !self.standing_decks.is_empty() {
            return Err(KccError::InvalidWorld);
        }
        self.shaft_step=Some(ShaftStep{before,after,offset:[0.0,0.0]});Ok(self)
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
            && radius>0.0 && self.actor_scene_bounds.is_none_or(|b|position.x_m-radius>=b.min_x
                && position.x_m+radius<=b.max_x && position.z_m-radius>=b.min_z && position.z_m+radius<=b.max_z)
            && (0.0..=3.0).contains(&position.y_m) && if position.y_m.abs()<=0.001 {true} else {
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
        if let Some(step)=&self.shaft_step {
            if step.after.clock.world_ms()!=now_ms || !(-4.0..=3.0).contains(&body.position_m.y_m) {return false;}
            return !body.grounded || step.after.floor_contact(step.local(body.position_m),body.radius_m)
                || step.after.surface_contact(step.local(body.position_m),body.radius_m);
        }
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

    pub(crate) fn add_dense_wall(&mut self, id: String, wall: Aabb, escape: bool) {
        self.walls.push(wall);
        if escape { self.player_escape_walls.push((id,wall)); }
    }
    pub(crate) fn player_escape_ids(&self) -> Vec<String> {
        self.player_escape_walls.iter().map(|(id,_)| id.clone()).collect()
    }
    pub(crate) fn can_player_occupy(&self, position: Vec3, radius_m: f32) -> bool {
        self.can_occupy_impl(position,radius_m,self.movement_mode,&self.effective_rules,true)
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
        self.can_occupy_impl(position,radius_m,mode,rules,false)
    }

    fn can_occupy_impl(&self, position: Vec3, radius_m: f32, mode: MovementMode, rules: &EffectivePlayerRules, allow_player_escape: bool) -> bool {
        let shaft_valid=self.shaft_step.as_ref().is_none_or(|step| {
            let local=step.local(position);
            step.before.new_obstacle_at(local,radius_m)==Some(false)
                && step.after.new_obstacle_at(local,radius_m)==Some(false)
                && (position.y_m>=0.0 || step.before.geometry.lower_floor_contains(local.x_m,local.z_m,radius_m))
        });
        let service_lower=self.shaft_step.as_ref().is_some_and(|step| {
            let local=step.local(position);position.y_m<0.0 && step.before.geometry.service_v2
                && step.after.geometry.service_v2
                && crate::formal_runtime::shaft_lower_service::covers_disc(local.x_m,local.z_m,radius_m)
        });
        position.x_m.is_finite()
            && position.y_m.is_finite()
            && position.z_m.is_finite()
            && radius_m.is_finite()
            && radius_m > 0.0
            && position.y_m >= if self.shaft_step.is_some() {-4.0} else {0.0}
            && position.y_m <= 3.0
            && (service_lower || (position.x_m - radius_m >= self.bounds.min_x
            && position.x_m + radius_m <= self.bounds.max_x
            && position.z_m - radius_m >= self.bounds.min_z
            && position.z_m + radius_m <= self.bounds.max_z))
            && (allow_player_escape || self.actor_scene_bounds.is_none_or(|b|
                position.x_m-radius_m>=b.min_x && position.x_m+radius_m<=b.max_x
                    && position.z_m-radius_m>=b.min_z && position.z_m+radius_m<=b.max_z))
            && shaft_valid
            && (position.y_m<0.0 && self.shaft_step.is_some() || self.mh_shore_support.as_ref().is_some_and(|parts| shore_disc_covered(parts,position.x_m,position.z_m,radius_m))
                || self.mh_shore_support.is_none() && (self.walkable_polygons.is_empty()
                || self.walkable_polygons.iter().any(|polygon| {
                    circle_inside_polygon(position.x_m, position.z_m, radius_m, polygon)
                })))
            && (position.y_m<0.0 && self.shaft_step.is_some() || !self
                .walls
                .iter()
                .any(|wall| wall.contains_circle(position.x_m, position.z_m, radius_m)
                    && !(allow_player_escape && self.player_escape_walls.iter().any(|(_,escape)| escape==wall))))
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

// Exact coverage of a disc by the three adjoining axis-aligned shore strips.
// For each x interval, its widest disc cross-section occurs nearest the centre.
fn shore_disc_covered(parts:&[Aabb;3],x:f32,z:f32,r:f32)->bool {
    if ![x,z,r].into_iter().all(f32::is_finite) || r<=0.0 {return false;}
    let mut strips=*parts;
    strips.sort_by(|a,b|a.min_x.total_cmp(&b.min_x));
    let mut cursor=x-r;let end=x+r;
    for strip in strips {
        if strip.max_x<cursor {continue;}
        if strip.min_x>cursor {return false;}
        let right=strip.max_x.min(end);
        let nearest=x.clamp(cursor,right);
        let half=(r*r-(nearest-x)*(nearest-x)).max(0.0).sqrt();
        if z-half<strip.min_z || z+half>strip.max_z {return false;}
        cursor=right;
        if cursor>=end {return true;}
    }
    false
}

// The owner's x-then-z sweep is axis-aligned. Its minimum distance to every
// exterior edge of the three-strip support is exact, including concave corners.
fn shore_segment_covered(parts:&[Aabb;3],from:Vec3,to:Vec3,r:f32)->bool {
    if !shore_disc_covered(parts,from.x_m,from.z_m,r) || !shore_disc_covered(parts,to.x_m,to.z_m,r)
        || (from.x_m!=to.x_m && from.z_m!=to.z_m) {return false;}
    let mut strips=*parts;strips.sort_by(|a,b|a.min_x.total_cmp(&b.min_x));
    let (l,b,rr)=(strips[0],strips[1],strips[2]);
    let vertices=[(l.min_x,l.min_z),(l.max_x,l.min_z),(l.max_x,b.min_z),(b.max_x,b.min_z),
        (rr.min_x,rr.min_z),(rr.max_x,rr.min_z),(rr.max_x,rr.max_z),(rr.min_x,rr.max_z),
        (b.max_x,b.max_z),(l.max_x,b.max_z),(l.max_x,l.max_z),(l.min_x,l.max_z)];
    let (sx0,sx1)=(from.x_m.min(to.x_m),from.x_m.max(to.x_m));
    let (sz0,sz1)=(from.z_m.min(to.z_m),from.z_m.max(to.z_m));
    for i in 0..vertices.len() {
        let a=vertices[i];let b=vertices[(i+1)%vertices.len()];
        let (ex0,ex1)=(a.0.min(b.0),a.0.max(b.0));let (ez0,ez1)=(a.1.min(b.1),a.1.max(b.1));
        let dx=(ex0-sx1).max(sx0-ex1).max(0.0);let dz=(ez0-sz1).max(sz0-ez1).max(0.0);
        if dx*dx+dz*dz<r*r {return false;}
    }
    true
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
    if let Some(step)=&world.shaft_step {
        if step.before.clock.world_ms()!=now_ms || step.after.clock.world_ms()!=next_ms {
            return Err(KccError::InvalidWorld);
        }
    }
    let shaft_carried=world.shaft_step.as_ref().is_some_and(|step|allow_surface_motion && body.grounded
        && step.before.surface_contact(step.local(body.position_m),body.radius_m));
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
    let surface = if allow_surface_motion && carrying.is_none() && !shaft_carried && body.grounded && body.position_m.y_m.abs()<=0.001 {
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
        if !world.can_player_occupy(candidate,body.radius_m) {return Err(KccError::InvalidWorld);}
        body.velocity_mps.y_m=if matches!(carrying,Some(SupportContact::Standing(_))) {0.0} else {(target-previous_y)/dt_s};
        body.position_m.y_m=target;
    }
    if shaft_carried {
        let step=world.shaft_step.as_ref().ok_or(KccError::InvalidWorld)?;
        let target=step.after.clock.pose().height_m;
        let mut candidate=body.position_m;candidate.y_m=target;
        if !world.can_player_occupy(candidate,body.radius_m) {return Err(KccError::InvalidWorld);}
        body.velocity_mps.y_m=(target-previous_y)/dt_s;
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
    if !motion_segment_clear(world, body.position_m, x_candidate, body.radius_m, surface != [0.0, 0.0] || world.has_support_surfaces() || world.shaft_step.is_some() || world.gate_decon_union || world.beacon_exit_union || world.shaft_lockdown_union || world.mh_shore_support.is_some() || world.cw_early_union) {
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
    if !motion_segment_clear(world, body.position_m, z_candidate, body.radius_m, surface != [0.0, 0.0] || world.has_support_surfaces() || world.shaft_step.is_some() || world.gate_decon_union || world.beacon_exit_union || world.shaft_lockdown_union || world.mh_shore_support.is_some() || world.cw_early_union) {
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
    if let Some(step)=&world.shaft_step {
        if shaft_carried {
            if step.after.cabin_contains(step.local(body.position_m),body.radius_m) {
                body.grounded=true;
                return Ok(collisions);
            }
            if !step.after.floor_contact(step.local(body.position_m),body.radius_m) {return Err(KccError::InvalidWorld);}
            body.velocity_mps.y_m=0.0;body.grounded=true;
            return Ok(collisions);
        }
        if body.grounded && !step.after.floor_contact(step.local(body.position_m),body.radius_m)
            && !step.after.surface_contact(step.local(body.position_m),body.radius_m) {
            body.grounded=false;body.velocity_mps.y_m-=body.gravity_mps2*dt_s;
        }
        if !body.grounded {
            let proposed=body.position_m.y_m+body.velocity_mps.y_m*dt_s;
            let mut landing=None;
            for height in [0.0_f32,step.after.clock.pose().height_m,-4.0] {
                let mut candidate=body.position_m;candidate.y_m=height;
                if previous_y>=height-0.001 && proposed<=height
                    && (step.after.floor_contact(step.local(candidate),body.radius_m) || step.after.surface_contact(step.local(candidate),body.radius_m))
                    && world.can_player_occupy(candidate,body.radius_m) {
                    landing=Some(landing.map_or(height,|old:f32|old.max(height)));
                }
            }
            if let Some(height)=landing {body.position_m.y_m=height;body.velocity_mps.y_m=0.0;body.grounded=true;}
            else {body.position_m.y_m=proposed;if proposed < -4.0 || proposed > 3.0 {return Err(KccError::InvalidWorld);}}
        }
        return Ok(collisions);
    }
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
    if let Some(step)=&world.shaft_step {
        if from.y_m<0.0 && step.before.geometry.service_v2 {
            let (a,b)=(step.local(from),step.local(to));
            let lower_hold=step.before.clock.pose().phase==crate::moving_support::SupportPhase::LowerHold && step.after.clock.pose().phase==crate::moving_support::SupportPhase::LowerHold;
            if !crate::formal_runtime::shaft_lower_service::candidate_sweep_geometry(&step.before.geometry,a,b,radius,lower_hold)
                || !crate::formal_runtime::shaft_lower_service::candidate_sweep_geometry(&step.after.geometry,a,b,radius,lower_hold) {return false;}
        }
    }    if world.mh_shore_support.as_ref().is_some_and(|parts|!shore_segment_covered(parts,from,to,radius)) {return false;}
    let distance = (to.x_m - from.x_m).hypot(to.z_m - from.z_m);
    let steps = if swept { (distance / 0.05).ceil().max(1.0) as usize } else { 1 };
    (1..=steps).all(|step| {
        let t = step as f32 / steps as f32;
        world.can_player_occupy(Vec3 { x_m: from.x_m + (to.x_m - from.x_m) * t,
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
    fn cw_early_union_preserves_conveyor_contact_and_actor_bounds() {
        let bounds=Aabb::new(0.0,24.0,0.0,16.0).unwrap();
        let own=StaticKccWorld::new(bounds,vec![Aabb::new(23.5,24.0,0.5,6.0).unwrap(),Aabb::new(23.5,24.0,9.5,15.5).unwrap()]);
        let target=StaticKccWorld::new(bounds,vec![Aabb::new(0.0,0.5,0.5,6.0).unwrap(),Aabb::new(0.0,0.5,9.5,15.5).unwrap()])
            .with_terrain_regions(vec![KccTerrainRegion{id:"cw_conveyor_belt".into(),tag:TerrainTag::Conveyor,polygon:vec![[6.0,6.0],[18.0,6.0],[18.0,10.0],[6.0,10.0]],surface_velocity_mps:Some([-1.2,0.0])}]);
        let joined=own.join_cw_early(&target,24.0).unwrap();
        assert!(joined.can_player_occupy(Vec3::new(24.0,0.0,8.0).unwrap(),0.3));
        assert!(!joined.can_occupy(Vec3::new(24.0,0.0,8.0).unwrap(),0.3));
        assert!(!joined.can_player_occupy(Vec3::new(24.0,0.0,6.1).unwrap(),0.3));
        let mut local=KccBody::new(Vec3::new(10.0,0.0,8.0).unwrap());
        let mut translated=KccBody::new(Vec3::new(34.0,0.0,8.0).unwrap());
        step_kcc(&mut local,&target,(0.0,0.0),0.05).unwrap();step_kcc(&mut translated,&joined,(0.0,0.0),0.05).unwrap();
        assert_eq!(local.velocity_mps.x_m,translated.velocity_mps.x_m);assert!(translated.velocity_mps.x_m<0.0);
        let mut air=KccBody::new(Vec3::new(34.0,1.0,8.0).unwrap());air.grounded=false;
        step_kcc(&mut air,&joined,(0.0,0.0),0.05).unwrap();assert_eq!(air.velocity_mps.x_m,0.0);
        assert!(target.join_cw_early(&own,-24.0).unwrap().can_player_occupy(Vec3::new(0.0,0.0,8.0).unwrap(),0.3));
    }

    #[test]
    fn mh_shore_union_covers_complete_disc_and_keeps_boundaries() {
        let bounds=Aabb::new(0.0,24.0,0.0,16.0).unwrap();
        let floor=vec![vec![[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]];
        let own=StaticKccWorld::new(bounds,vec![Aabb::new(23.5,24.0,0.5,6.0).unwrap(),Aabb::new(23.5,24.0,9.5,15.5).unwrap()]).with_walkable_polygons(floor.clone());
        let target=StaticKccWorld::new(bounds,vec![Aabb::new(0.0,0.5,0.5,6.0).unwrap(),Aabb::new(0.0,0.5,9.5,15.5).unwrap()]).with_walkable_polygons(floor.clone())
            .with_terrain_regions(vec![KccTerrainRegion{id:"water".into(),tag:TerrainTag::WaterShallow,polygon:vec![[13.0,5.0],[19.0,5.0],[19.0,11.0],[13.0,11.0]],surface_velocity_mps:None}])
            .with_movement_slow_zones(vec![Aabb::new(13.0,19.0,5.0,11.0).unwrap()]);
        let joined=own.join_mh_shore(&target,24.0).unwrap();
        for x in [23.3,23.5,24.0,24.5,24.7] {assert!(joined.can_player_occupy(Vec3::new(x,0.0,8.0).unwrap(),0.3),"x={x}");}
        assert!(!joined.can_player_occupy(Vec3::new(24.0,0.0,6.2).unwrap(),0.3));
        assert!(!joined.can_player_occupy(Vec3::new(24.0,0.0,9.3).unwrap(),0.3));
        let a=Vec3::new(23.0,0.0,6.2).unwrap();let b=Vec3::new(25.0,0.0,6.2).unwrap();
        assert!(joined.can_player_occupy(a,0.3) && joined.can_player_occupy(b,0.3));
        assert!(!shore_segment_covered(joined.mh_shore_support.as_ref().unwrap(),a,b,0.3));
        assert!(!joined.can_occupy(Vec3::new(24.0,0.0,8.0).unwrap(),0.3));
        assert!(joined.intersects_terrain_at(Vec3::new(40.0,0.0,8.0).unwrap(),0.3,&[TerrainTag::WaterShallow]));
        let mut local=KccBody::new(Vec3::new(16.0,0.0,8.0).unwrap());
        let mut translated=KccBody::new(Vec3::new(40.0,0.0,8.0).unwrap());
        step_kcc(&mut local,&target,(1.0,0.0),0.05).unwrap();step_kcc(&mut translated,&joined,(1.0,0.0),0.05).unwrap();
        assert_eq!(local.velocity_mps.x_m,translated.velocity_mps.x_m);
        let reverse=target.join_mh_shore(&own,-24.0).unwrap();
        assert!(reverse.can_player_occupy(Vec3::new(0.0,0.0,8.0).unwrap(),0.3));
        assert!(own.join_mh_shore(&StaticKccWorld::new(bounds,vec![]),24.0).is_err());
    }

    #[test]
    fn hive_route_union_preserves_apertures_walls_and_actor_room_bounds() {
        for (width,depth,low,high) in [(20.0,14.0,5.5,8.5),(24.0,16.0,6.5,9.5)] {
            let own=StaticKccWorld::new(Aabb::new(0.0,width,0.0,depth).unwrap(),vec![
                Aabb::new(width-0.5,width,0.0,low).unwrap(),
                Aabb::new(width-0.5,width,high,depth).unwrap(),
            ]);
            let target=StaticKccWorld::new(Aabb::new(0.0,width,0.0,depth).unwrap(),vec![
                Aabb::new(0.0,0.5,0.0,low).unwrap(),
                Aabb::new(0.0,0.5,high,depth).unwrap(),
            ]);
            let joined=own.join_hive_route(&target,width,width,depth).unwrap();
            let middle=(low+high)/2.0;
            assert!(joined.can_player_occupy(Vec3::new(width-0.1,0.0,middle).unwrap(),0.35));
            assert!(!joined.can_occupy(Vec3::new(width-0.1,0.0,middle).unwrap(),0.35));
            assert!(!joined.can_player_occupy(Vec3::new(width-0.1,0.0,low-0.1).unwrap(),0.35));
            let reverse=target.join_hive_route(&own,-width,width,depth).unwrap();
            assert!(reverse.can_player_occupy(Vec3::new(0.1,0.0,middle).unwrap(),0.35));
            assert!(!reverse.can_occupy(Vec3::new(0.1,0.0,middle).unwrap(),0.35));
            assert!(!reverse.can_player_occupy(Vec3::new(0.1,0.0,low-0.1).unwrap(),0.35));
        }
    }

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

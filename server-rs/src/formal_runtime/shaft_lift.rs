//! Shaft-only two-stop clock. A blocked boarding threshold holds the current
//! endpoint; neither world time nor a rider position is rewritten.
use crate::moving_support::{SupportFrame,SupportPhase,SupportPose};
use sha2::{Digest,Sha256};

pub(super) const SUPPORT_ID:&str="gh_shaft_two_stop_lift";
// Reuses the existing maintained Gear Shaft travel and dwell timing.
const TRAVEL_MS:u64=4_000;
const HOLD_MS:u64=2_000;
const PERIOD_MS:u64=2*(TRAVEL_MS+HOLD_MS);

#[derive(Clone,Debug,PartialEq)]
pub(crate) struct ShaftClock { effective_ms:u64, world_ms:u64, paused:bool }

impl ShaftClock {
    pub(crate) fn world_ms(&self)->u64 {self.world_ms}
    /// Unloaded Shaft time is paused; rebase only the external clock on return.
    pub(crate) fn rebase_world_time(&mut self,world_ms:u64)->Result<(),&'static str> {
        if world_ms<self.world_ms {return Err("E_SHAFT_CLOCK_REVERSED");}
        self.world_ms=world_ms;
        Ok(())
    }
    /// New installation begins at the upper landing without relocating a body.
    pub(crate) fn at_upper(world_ms:u64)->Self {
        Self {effective_ms:HOLD_MS+TRAVEL_MS,world_ms,paused:false}
    }

    /// Reconstruct the paused cyclic phase from the actual saved support pose;
    /// world time remains the original saved authority time.
    pub(crate) fn restore(world_ms:u64,saved:&SupportPose)->Result<Self,&'static str> {
        if saved.support_id!=SUPPORT_ID {return Err("E_SHAFT_SUPPORT_ID");}
        let elapsed=u64::from(saved.phase_elapsed_ms);
        let effective_ms=match saved.phase {
            SupportPhase::LowerHold if elapsed<HOLD_MS => elapsed,
            SupportPhase::Rising if elapsed<TRAVEL_MS => HOLD_MS+elapsed,
            SupportPhase::UpperHold if elapsed<HOLD_MS => HOLD_MS+TRAVEL_MS+elapsed,
            SupportPhase::Falling if elapsed<TRAVEL_MS => 2*HOLD_MS+TRAVEL_MS+elapsed,
            _=>return Err("E_SHAFT_SUPPORT_PHASE"),
        };
        let paused=matches!(saved.phase,SupportPhase::Rising|SupportPhase::Falling) && saved.velocity_mps==0.0;
        let clock=Self{effective_ms,world_ms,paused};
        if clock.pose()!=*saved {return Err("E_SHAFT_SUPPORT_POSE");}
        Ok(clock)
    }

    pub(crate) fn pose(&self)->SupportPose {
        let t=self.effective_ms%PERIOD_MS;
        let (phase,height_m,velocity_mps,elapsed)=if t<HOLD_MS {
            (SupportPhase::LowerHold,-4.0,0.0,t)
        } else if t<HOLD_MS+TRAVEL_MS {
            let elapsed=t-HOLD_MS;
            (SupportPhase::Rising,-4.0+4.0*(elapsed as f32/TRAVEL_MS as f32),1.0,elapsed)
        } else if t<2*HOLD_MS+TRAVEL_MS {
            (SupportPhase::UpperHold,0.0,0.0,t-HOLD_MS-TRAVEL_MS)
        } else {
            let elapsed=t-2*HOLD_MS-TRAVEL_MS;
            (SupportPhase::Falling,-4.0*(elapsed as f32/TRAVEL_MS as f32),-1.0,elapsed)
        };
        SupportPose{support_id:SUPPORT_ID.into(),phase,height_m,velocity_mps:if self.paused {0.0}else{velocity_mps},phase_elapsed_ms:elapsed as u32}
    }

    /// Caller supplies the authoritative actual-radius threshold occupancy or
    /// swept crossing for this step. This method never changes movement itself.
    pub(crate) fn advance(&mut self,next_world_ms:u64,threshold_blocked:bool)->Result<(),&'static str> {
        let dt=next_world_ms.checked_sub(self.world_ms).ok_or("E_SHAFT_CLOCK_REVERSED")?;
        if dt>100 {return Err("E_SHAFT_CLOCK_STEP");} // Existing KCC maximum step.
        let t=self.effective_ms%PERIOD_MS;
        let hold_end=if t<HOLD_MS {Some(HOLD_MS)} else if (HOLD_MS+TRAVEL_MS..2*HOLD_MS+TRAVEL_MS).contains(&t) {Some(2*HOLD_MS+TRAVEL_MS)} else {None};
        let advance=if threshold_blocked {hold_end.map_or(0,|end|dt.min(end-1-t))} else {dt};
        self.paused=threshold_blocked && hold_end.is_none();
        self.effective_ms=(t+advance)%PERIOD_MS;
        self.world_ms=next_world_ms;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocked_threshold_keeps_the_upper_landing_stationary() {
        let mut clock=ShaftClock::at_upper(10_000);
        for i in 1..=300 {clock.advance(10_000+i*17,true).unwrap();}
        let pose=clock.pose();
        assert_eq!(pose.phase,SupportPhase::UpperHold);
        assert_eq!(pose.height_m,0.0);
        clock.advance(15_117,false).unwrap();
        assert_eq!(clock.pose().phase,SupportPhase::Falling);
    }
    #[test]
    fn saved_paused_pose_recovers_without_rewinding_world_time() {
        let mut clock=ShaftClock::at_upper(1_000);
        for i in 1..=130 {clock.advance(1_000+i*17,false).unwrap();}
        clock.advance(3_227,true).unwrap();
        let saved=clock.pose();
        let mut restored=ShaftClock::restore(3_227,&saved).unwrap();
        assert_eq!(restored.pose(),saved);
        restored.advance(3_244,false).unwrap();
        assert_eq!(restored.pose().phase,SupportPhase::Falling);
    }
    #[test]
    fn unexpected_obstruction_freezes_motion_and_restores_its_direction() {
        let mut clock=ShaftClock::at_upper(0);
        for i in 1..=120 {clock.advance(i*17,false).unwrap();}
        assert_eq!(clock.pose().phase,SupportPhase::Falling);
        let height=clock.pose().height_m;
        clock.advance(2057,true).unwrap();
        let saved=clock.pose();
        assert_eq!(saved.height_m,height);
        assert_eq!(saved.velocity_mps,0.0);
        let mut restored=ShaftClock::restore(2057,&saved).unwrap();
        restored.advance(2074,false).unwrap();
        assert!(restored.pose().height_m<height);
    }
    #[test]
    fn swept_real_radius_cannot_skip_a_thin_threshold() {
        let area=crate::continuous_kcc::Aabb::new(10.0,14.0,5.0,5.15).unwrap();
        let a=crate::world_v3::Vec3::new(12.0,0.0,4.0).unwrap();
        let b=crate::world_v3::Vec3::new(12.0,0.0,6.0).unwrap();
        assert!(swept_obstructs(a,b,0.35,area).unwrap());
        let far=crate::world_v3::Vec3::new(8.0,0.0,4.0).unwrap();
        assert!(!swept_obstructs(far,far,0.3,area).unwrap());
        assert!(swept_obstructs(a,b,f32::NAN,area).is_err());
    }
    #[test]
    fn malformed_saved_pose_is_rejected() {
        let mut pose=ShaftClock::at_upper(5).pose();
        pose.height_m=-3.0;
        assert!(ShaftClock::restore(5,&pose).is_err());
    }
}

/// Conservative swept circular-footprint obstruction. Expanding the rectangle
/// by the real body radius may hold at a corner; it never clears an intersecting
/// body or alters that body's collision or path.
pub(super) fn swept_obstructs(
    from:crate::world_v3::Vec3,to:crate::world_v3::Vec3,
    radius:f32,area:crate::continuous_kcc::Aabb,
)->Result<bool,&'static str> {
    if ![from.x_m,from.y_m,from.z_m,to.x_m,to.y_m,to.z_m,radius,
        area.min_x,area.max_x,area.min_z,area.max_z].iter().all(|v|v.is_finite()) || radius<=0.0
        || area.min_x>=area.max_x || area.min_z>=area.max_z {return Err("E_SHAFT_OBSTRUCTION_INPUT");}
    let bounds=[(area.min_x-radius,area.max_x+radius),(area.min_z-radius,area.max_z+radius)];
    let start=[from.x_m,from.z_m]; let delta=[to.x_m-from.x_m,to.z_m-from.z_m];
    let mut lo=0.0_f32; let mut hi=1.0_f32;
    for axis in 0..2 {
        if delta[axis]==0.0 {
            if start[axis]<bounds[axis].0 || start[axis]>bounds[axis].1 {return Ok(false);}
        } else {
            let a=(bounds[axis].0-start[axis])/delta[axis];
            let b=(bounds[axis].1-start[axis])/delta[axis];
            lo=lo.max(a.min(b));hi=hi.min(a.max(b));
            if lo>hi {return Ok(false);}
        }
    }
    Ok(lo<=1.0 && hi>=0.0)
}

/// Snapshot-only visual contract. Geometry comes from the same admitted scene;
/// gate flags are derived from the actual clock, never from the client.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct ShaftLiftView {
    pub schema_version:u32,
    pub world_id:String,
    pub scene_id:String,
    pub world_epoch:u64,
    pub server_tick:u64,
    pub authority_revision:u64,
    pub server_time_ms:u64,
    pub scene_source_sha256:String,
    pub pose:SupportPose,
    pub upper_gate_open:bool,
    pub lower_gate_open:bool,
    pub cabin_gate_open:bool,
    pub legacy_floor_active:bool,
}

impl ShaftClock {
    pub(super) fn project(&self,scene:&crate::scene_runtime::SceneDefinition,
        epoch:u64,tick:u64,revision:u64,world_ms:u64,legacy_floor_active:bool)->Option<ShaftLiftView> {
        if scene.world_id!="grey_hive" || scene.scene_id!="gh_central_shaft" || epoch==0 || self.world_ms!=world_ms
            || scene.verified_source_sha256.len()!=64 || !scene.verified_source_sha256.bytes().all(|v|v.is_ascii_hexdigit()&&!v.is_ascii_uppercase()) {return None;}
        let supports:Vec<_>=scene.moving_supports.iter().filter(|d|d.id==SUPPORT_ID).collect();
        let [support]=supports.as_slice() else {return None;};
        if support.polygon!=[[1.5,1.0],[5.5,1.0],[5.5,5.0],[1.5,5.0]] || support.lower_m!=-4.0 || support.upper_m!=0.0
            || support.travel_ms!=4_000 || support.endpoint_hold_ms!=2_000 {return None;}
        let pose=self.pose();
        if legacy_floor_active && pose.phase!=SupportPhase::UpperHold {return None;}
        let upper_gate_open=pose.phase==SupportPhase::UpperHold;
        let lower_gate_open=pose.phase==SupportPhase::LowerHold;
        Some(ShaftLiftView{schema_version:1,world_id:scene.world_id.clone(),scene_id:scene.scene_id.clone(),
            world_epoch:epoch,server_tick:tick,authority_revision:revision,server_time_ms:world_ms,
            scene_source_sha256:scene.verified_source_sha256.clone(),pose,upper_gate_open,lower_gate_open,
            cabin_gate_open:upper_gate_open||lower_gate_open,legacy_floor_active})
    }
}

#[derive(Clone,Debug,PartialEq)]
pub(crate) struct ShaftWall { pub id:String, pub area:crate::continuous_kcc::Aabb, pub lower:f32, pub upper:f32 }
#[derive(Clone,Debug,PartialEq)]
pub(crate) struct ShaftGeometry {
    pub(crate) service_v2:bool,
    pub well:crate::continuous_kcc::Aabb,
    pub lower_floor:crate::continuous_kcc::Aabb,
    pub walls:Vec<ShaftWall>,
    pub upper_gate:crate::continuous_kcc::Aabb,
    pub lower_gate:crate::continuous_kcc::Aabb,
    pub cabin_gate:crate::continuous_kcc::Aabb,
    pub cabin_rails:Vec<ShaftWall>,
}

/// Owner-side state exists only for an admitted active Shaft epoch.
#[derive(Clone,Debug,PartialEq)]
pub(crate) struct ShaftOwner {
    pub geometry:ShaftGeometry,
    pub clock:ShaftClock,
}
impl ShaftOwner {
    /// Whole-entry admission only: every actual live disc must be clear of the
    /// proposed opening and new obstacles. The caller separately checks the
    /// original legacy KCC before deciding to defer or activate.
    pub(crate) fn clear_at_entry(&self,world:&crate::world_v3::WorldStateV3)->Result<bool,&'static str> {
        if world.world_id!="grey_hive" || world.scene_id!="gh_central_shaft" || !world.sentinels.is_empty()
            || !world.player.position_m.y_m.is_finite() || !(0.0..=3.0).contains(&world.player.position_m.y_m)
            || !world.player.velocity_mps.y_m.is_finite() {
            return Err("E_SHAFT_LEGACY_CONTACT");
        }
        if world.player.grounded && (world.player.position_m.y_m!=0.0 || world.player.velocity_mps.y_m!=0.0) {
            return Err("E_SHAFT_LEGACY_CONTACT");
        }
        if !world.player.grounded {return Ok(false);}
        let clear=|position:crate::world_v3::Vec3,radius:f32| -> Result<bool,&'static str> {
            if position.y_m!=0.0 {return Err("E_SHAFT_LEGACY_CONTACT");}
            Ok(!swept_obstructs(position,position,radius,self.geometry.well)?
                && !self.new_obstacle_at(position,radius).ok_or("E_SHAFT_INTERLOCK_INPUT")?)
        };
        if !clear(world.player.position_m,world.player.radius_m)? {return Ok(false);}
        for actor in &world.generic_actors {
            if !actor.validate() {return Err("E_SHAFT_INTERLOCK_ACTOR");}
            if actor.hp>0 && !clear(actor.position_m,actor.body_radius_m())? {return Ok(false);}
        }
        Ok(true)
    }

    /// Construct the interlock from actual pre/post owner states. Unknown or
    /// changed actor sets fail closed; the Shaft has no authored sentinel and
    /// this path never assumes one has a radius it does not expose.
    pub(crate) fn advance_from_worlds(&mut self,before:&crate::world_v3::WorldStateV3,
        after:&crate::world_v3::WorldStateV3)->Result<(),&'static str> {
        if before.world_id!="grey_hive" || before.scene_id!="gh_central_shaft"
            || after.world_id!=before.world_id || after.scene_id!=before.scene_id
            || after.revision.world_epoch!=before.revision.world_epoch || before.server_time_ms!=self.clock.world_ms()
            || !before.sentinels.is_empty() || !after.sentinels.is_empty()
            || before.player.radius_m!=after.player.radius_m {return Err("E_SHAFT_INTERLOCK_CONTEXT");}
        let mut actors=Vec::new();let mut seen=std::collections::BTreeSet::new();
        for a in &before.generic_actors {
            if !a.validate() || !seen.insert(a.entity_id.as_str()) {return Err("E_SHAFT_INTERLOCK_ACTOR");}
            let rows:Vec<_>=after.generic_actors.iter().filter(|b|b.entity_id==a.entity_id).collect();
            if rows.len()!=1 || !rows[0].validate() || rows[0].entity_type!=a.entity_type
                || rows[0].controller_variant!=a.controller_variant || rows[0].body_radius_m()!=a.body_radius_m() {
                return Err("E_SHAFT_INTERLOCK_ACTOR");
            }
            if a.hp>0 || rows[0].hp>0 {actors.push((a.position_m,rows[0].position_m,a.body_radius_m()));}
        }
        if seen.len()!=after.generic_actors.len() {return Err("E_SHAFT_INTERLOCK_ACTOR");}
        self.advance(after.server_time_ms,(before.player.position_m,after.player.position_m,before.player.radius_m),&actors)
    }

    pub(crate) fn cabin_contains(&self,p:crate::world_v3::Vec3,radius:f32)->bool {
        let a=self.geometry.well;
        [p.x_m,p.y_m,p.z_m,radius].iter().all(|v|v.is_finite()) && radius>0.0
            && p.x_m-radius>=a.min_x && p.x_m+radius<=a.max_x
            && p.z_m-radius>=a.min_z && p.z_m+radius<=a.max_z
    }

    /// Floor and cabin contacts use the real foot height and the entire disc.
    /// No body may be snapped to either landing merely from horizontal overlap.
    pub(crate) fn surface_contact(&self,p:crate::world_v3::Vec3,radius:f32)->bool {
        self.cabin_contains(p,radius)
            && (p.y_m-self.clock.pose().height_m).abs()<=0.001
    }
    pub(crate) fn floor_contact(&self,p:crate::world_v3::Vec3,radius:f32)->bool {
        if ![p.x_m,p.y_m,p.z_m,radius].iter().all(|v|v.is_finite()) || radius<=0.0 {return false;}
        let well=self.geometry.well;
        let dx=p.x_m-p.x_m.clamp(well.min_x,well.max_x);
        let dz=p.z_m-p.z_m.clamp(well.min_z,well.max_z);
        if p.y_m.abs()<=0.001 {return self.clock.pose().phase==SupportPhase::UpperHold
            || dx*dx+dz*dz>=radius*radius;}
        if (p.y_m+4.0).abs()>0.001 {return false;}
        let a=self.geometry.lower_floor;
        let ordinary=p.x_m-radius>=a.min_x && p.x_m+radius<=a.max_x
            && p.z_m-radius>=a.min_z && p.z_m+radius<=a.max_z;
        ordinary || (self.geometry.service_v2 && super::shaft_lower_service::covers_disc(p.x_m,p.z_m,radius)) || (self.clock.pose().phase==SupportPhase::LowerHold
            && self.geometry.lower_floor_contains(p.x_m,p.z_m,radius))
    }

    /// New Shaft obstacles only. Existing scene collision remains a separate
    /// mandatory KCC check. No actor receives a new-piece exemption.
    pub(crate) fn new_obstacle_at(&self,position:crate::world_v3::Vec3,radius:f32)->Option<bool> {
        if ![position.x_m,position.y_m,position.z_m,radius].iter().all(|v|v.is_finite()) || radius<=0.0 {return None;}
        let hit=|area:crate::continuous_kcc::Aabb| {
            let dx=position.x_m-position.x_m.clamp(area.min_x,area.max_x);
            let dz=position.z_m-position.z_m.clamp(area.min_z,area.max_z);
            dx*dx+dz*dz<radius*radius
        };
        if self.geometry.walls.iter().any(|wall|position.y_m>=wall.lower && position.y_m<wall.upper
            && hit(wall.area)) {return Some(true);}
        let pose=self.clock.pose();
        if self.geometry.cabin_rails.iter().any(|wall|position.y_m>=pose.height_m+wall.lower
            && position.y_m<pose.height_m+wall.upper && hit(wall.area)) {
            return Some(true);
        }
        let upper_open=pose.phase==SupportPhase::UpperHold;
        let lower_open=pose.phase==SupportPhase::LowerHold;
        if (!upper_open && (0.0..2.4).contains(&position.y_m) && hit(self.geometry.upper_gate))
            || (!lower_open && (-4.0..-1.6).contains(&position.y_m) && hit(self.geometry.lower_gate))
            || (!(upper_open || lower_open) && position.y_m>=pose.height_m && position.y_m<pose.height_m+1.1
                && hit(self.geometry.cabin_gate)) {return Some(true);}
        Some(false)
    }

    pub(crate) fn new_at_upper(scene:&crate::scene_runtime::SceneDefinition,world_ms:u64)->Option<Self> {
        Some(Self{geometry:ShaftGeometry::from_scene(scene)?,clock:ShaftClock::at_upper(world_ms)})
    }

    /// Build both presentation records from one validated authority state.
    /// The caller remains responsible for actual KCC and readiness admission.
    pub(crate) fn views(&self,scene:&crate::scene_runtime::SceneDefinition,
        epoch:u64,tick:u64,revision:u64,world_ms:u64,body:&crate::continuous_kcc::KccBody,
        traversing:bool,upper_portal:bool,upper_east_portal:bool)->Option<(ShaftLiftView,crate::moving_support::SupportSceneView)> {
        use crate::moving_support::{SupportRiderMode as Mode,SupportRiderView,SupportSceneView};
        let selected=if self.geometry.service_v2 {super::shaft_lower_service::candidate_geometry(scene)} else {ShaftGeometry::from_scene(scene)};
        if selected.as_ref()!=Some(&self.geometry) {return None;}
        let p=body.position_m;let r=body.radius_m;
        let seam=upper_portal && !traversing && body.grounded && p.y_m==0.0 && p.x_m>=0.0 && p.x_m<=r
            && (7.0..=9.0).contains(&p.z_m) && p.z_m-r>=6.5 && p.z_m+r<=9.5;
        let east_seam=upper_east_portal && !traversing && body.grounded && p.y_m==0.0 && p.x_m<=24.0 && p.x_m>=24.0-r
            && (7.0..=9.0).contains(&p.z_m) && p.z_m-r>=6.5 && p.z_m+r<=9.5;
        let service_lower=self.geometry.service_v2 && p.y_m<0.0 && super::shaft_lower_service::covers_disc(p.x_m,p.z_m,r);
        if ![p.x_m,p.y_m,p.z_m,r].iter().all(|n|n.is_finite()) || r<=0.0 || !(-4.0..=3.0).contains(&p.y_m)
            || (!service_lower && ((p.x_m-r<0.0 && !seam) || (p.x_m+r>24.0 && !east_seam) || p.z_m-r<0.0 || p.z_m+r>16.0)) {return None;}
        let visual=self.clock.project(scene,epoch,tick,revision,world_ms,false)?;
        let (mode,support_id)=if traversing {(Mode::Traversal,None)} else if !body.grounded {(Mode::Airborne,None)}
            else if self.surface_contact(p,r) {
                (Mode::Surface,Some(SUPPORT_ID.to_owned()))
            } else if self.floor_contact(p,r) {
                (Mode::Floor,None)
            } else {return None;};
        let support=SupportSceneView{schema_version:1,world_id:scene.world_id.clone(),scene_id:scene.scene_id.clone(),
            world_epoch:epoch,server_tick:tick,authority_revision:revision,server_time_ms:world_ms,
            scene_source_sha256:scene.verified_source_sha256.clone(),poses:vec![visual.pose.clone()],
            rider:SupportRiderView{position_m:p,radius_m:r,mode,support_id}};
        Some((visual,support))
    }

    /// Use the existing support-frame wire format and catalog fingerprint.
    /// The pose is the owner's interlocked clock, not a fresh world-time pose.
    pub(crate) fn frame(&self,scene:&crate::scene_runtime::SceneDefinition,epoch:u64)->Result<SupportFrame,&'static str> {
        if self.geometry.service_v2 {return super::shaft_lower_service::candidate_frame(self,scene,epoch);}
        if epoch==0 || ShaftGeometry::from_scene(scene).as_ref()!=Some(&self.geometry) {
            return Err("E_SHAFT_FRAME_SCENE");
        }
        let bytes=serde_json::to_vec(&scene.moving_supports).map_err(|_|"E_SHAFT_FRAME_CATALOG")?;
        Ok(SupportFrame{world_epoch:epoch,server_time_ms:self.clock.world_ms,
            catalog_sha256:format!("{:x}",Sha256::digest(bytes)),shaft_lower_recipe:None,poses:vec![self.clock.pose()]})
    }

    /// Exact scene/catalog and pose validation precedes installing restored
    /// authority; caller separately validates legacy-floor protection/contact.
    pub(crate) fn restore_frame(scene:&crate::scene_runtime::SceneDefinition,frame:&SupportFrame,
        epoch:u64,world_ms:u64)->Result<Self,&'static str> {
        if frame.shaft_lower_recipe.is_some() {
            if !super::shaft_lower_service::ADMISSION_READY {return Err("E_SHAFT_SERVICE_NOT_READY");}
            return super::shaft_lower_service::candidate_from_frame(scene,frame,epoch,world_ms);
        }
        let geometry=ShaftGeometry::from_scene(scene).ok_or("E_SHAFT_FRAME_SCENE")?;
        if epoch==0 || frame.world_epoch!=epoch || frame.server_time_ms!=world_ms || frame.poses.len()!=1 {
            return Err("E_SHAFT_FRAME_CONTEXT");
        }
        let clock=ShaftClock::restore(world_ms,&frame.poses[0])?;
        let owner=Self{geometry,clock};
        if owner.frame(scene,epoch)?!=*frame {
            return Err("E_SHAFT_FRAME_MISMATCH");
        }
        Ok(owner)
    }

    pub(crate) fn advance(
        &mut self,next_world_ms:u64,
        player:(crate::world_v3::Vec3,crate::world_v3::Vec3,f32),
        actors:&[(crate::world_v3::Vec3,crate::world_v3::Vec3,f32)],
    )->Result<(),&'static str> {
        let mut next=self.clone();
        let occupied=next.geometry.departure_obstructed(player,actors)?;
        next.clock.advance(next_world_ms,occupied)?;
        *self=next;
        Ok(())
    }
}

/// Shape-only save parsing. The exact catalog and contact are checked again
/// against the verified scene before a restored owner may be installed.
pub(crate) fn saved_frame_shape(frame:&SupportFrame,epoch:u64,world_ms:u64)->bool {
    frame.shaft_lower_recipe.as_ref().is_none_or(|r|r.is_known())
        && epoch>0 && frame.world_epoch==epoch && frame.server_time_ms==world_ms && frame.poses.len()==1
        && frame.catalog_sha256.len()==64
        && frame.catalog_sha256.bytes().all(|v|v.is_ascii_hexdigit()&&!v.is_ascii_uppercase())
        && ShaftClock::restore(world_ms,&frame.poses[0]).is_ok()
}
const LAYOUT:&str=include_str!("../../../apps/godot_playable/data/gh_central_shaft_lift.local.json");
/// Admit only the one authored Shaft support; this does not activate it.
/// Runtime construction and presentation must both be ready before use.
pub(crate) fn source_support_valid(scene:&crate::scene_runtime::SceneDefinition)->bool {
    if scene.world_id!="grey_hive" || scene.scene_id!="gh_central_shaft"
        || scene.bounds_m.x!=0.0 || scene.bounds_m.z!=0.0
        || scene.bounds_m.width!=24.0 || scene.bounds_m.depth!=16.0
        || scene.moving_supports.len()!=1 || !scene.standing_decks.is_empty() {return false;}
    let s=&scene.moving_supports[0];
    s.id==SUPPORT_ID && s.polygon==[[1.5,1.0],[5.5,1.0],[5.5,5.0],[1.5,5.0]]
        && s.lower_m == -4.0 && s.upper_m==0.0 && s.travel_ms==4_000 && s.endpoint_hold_ms==2_000
}
fn rectangle(value:&serde_json::Value)->Option<crate::continuous_kcc::Aabb> {
    let p=value.as_array()?; if p.len()!=4 {return None;}
    let pair=|i:usize|->Option<(f32,f32)>{let a=p[i].as_array()?;if a.len()!=2{return None;}Some((a[0].as_f64()? as f32,a[1].as_f64()? as f32))};
    let (x0,z0)=pair(0)?;let (x1,z1)=pair(1)?;let (x2,z2)=pair(2)?;let (x3,z3)=pair(3)?;
    if x1!=x2 || x0!=x3 || z0!=z1 || z2!=z3 {return None;}
    crate::continuous_kcc::Aabb::new(x0,x2,z0,z2).ok()
}
impl ShaftGeometry {
    /// Only the exact compiled Shaft support plus the R2 local design admits
    /// this geometry. The legacy scene without it remains unchanged.
    pub(crate) fn from_scene(scene:&crate::scene_runtime::SceneDefinition)->Option<Self> {
        if !source_support_valid(scene) || scene.verified_source_sha256.len()!=64 {return None;}
        let d:serde_json::Value=serde_json::from_str(LAYOUT).ok()?;
        if d.get("schema")?.as_str()? != "gh-shaft-lift/v1" || d.get("worldId")?.as_str()? != scene.world_id
            || d.get("sceneId")?.as_str()? != scene.scene_id || d.get("anchorXZ")? != &serde_json::json!([3.5,3]) {return None;}
        let support=d.get("support")?;
        if support.get("id")?.as_str()?!=SUPPORT_ID || support.get("polygon")? != &serde_json::json!([[1.5,1],[5.5,1],[5.5,5],[1.5,5]])
            || support.get("lowerM")?.as_i64()? != -4 || support.get("upperM")?.as_i64()? != 0
            || support.get("travelMs")?.as_u64()? != 4_000 || support.get("endpointHoldMs")?.as_u64()? != 2_000 {return None;}
        let well=rectangle(support.get("polygon")?)?;
        let lower_floor=rectangle(d.get("lowerFloor")?)?;
        if lower_floor!=crate::continuous_kcc::Aabb::new(1.5,19.5,5.0,14.0).ok()? {return None;}
        let rows=d.get("walls")?.as_array()?;if rows.len()!=18{return None;}
        let mut ids=std::collections::BTreeSet::new();let mut walls=Vec::new();
        for row in rows {
            let id=row.get("id")?.as_str()?;if !ids.insert(id.to_owned()) {return None;}
            let area=rectangle(row.get("polygon")?)?;let heights=row.get("heightBandM")?.as_array()?;
            if heights.len()!=2{return None;}let lower=heights[0].as_f64()? as f32;let upper=heights[1].as_f64()? as f32;
            if !lower.is_finite() || !upper.is_finite() || lower < -4.0 || upper > 2.4 || lower>=upper {return None;}
            walls.push(ShaftWall{id:id.to_owned(),area,lower,upper});
        }
        let gates=d.get("landingGates")?.as_array()?;if gates.len()!=2{return None;}
        let gate=|id:&str|->Option<crate::continuous_kcc::Aabb>{let a:Vec<_>=gates.iter().filter(|x|x.get("id").and_then(|v|v.as_str())==Some(id)).collect();if a.len()!=1{return None;}rectangle(a[0].get("polygon")?)};
        let upper_gate=gate("upper_landing_gate")?;let lower_gate=gate("lower_landing_gate")?;
        let cabin_gate=rectangle(d.get("cabinGate")?.get("polygon")?)?;
        let rail_rows=d.get("cabinRails")?.as_array()?;
        if rail_rows.len()!=3 {return None;}
        let mut cabin_rails=Vec::new();
        for id in ["cabin_west_rail","cabin_east_rail","cabin_north_rail"] {
            let matches:Vec<_>=rail_rows.iter().filter(|row|row.get("id").and_then(|v|v.as_str())==Some(id)).collect();
            if matches.len()!=1 || matches[0].get("relativeHeightBandM")?!=&serde_json::json!([0,1.1]) {return None;}
            cabin_rails.push(ShaftWall{id:id.to_owned(),area:rectangle(matches[0].get("polygon")?)?,lower:0.0,upper:1.1});
        }
        if upper_gate!=crate::continuous_kcc::Aabb::new(1.5,5.5,5.0,5.15).ok()? || lower_gate!=upper_gate
            || cabin_gate!=crate::continuous_kcc::Aabb::new(1.6,5.4,4.9,5.0).ok()? {return None;}
        if cabin_rails[0].area!=crate::continuous_kcc::Aabb::new(1.5,1.6,1.0,5.0).ok()?
            || cabin_rails[1].area!=crate::continuous_kcc::Aabb::new(5.4,5.5,1.0,5.0).ok()?
            || cabin_rails[2].area!=crate::continuous_kcc::Aabb::new(1.6,5.4,1.0,1.1).ok()? {return None;}
        Some(Self{service_v2:false,well,lower_floor,walls,upper_gate,lower_gate,cabin_gate,cabin_rails})
    }

    /// Conservatively hold the support when a real noncarried actor intersects
    /// the well or landing threshold, or when the player's actual swept disc
    /// intersects the threshold. The caller supplies authoritative positions;
    /// this function neither invents a body radius nor changes movement.
    pub(crate) fn departure_obstructed(
        &self, player:(crate::world_v3::Vec3,crate::world_v3::Vec3,f32),
        actors:&[(crate::world_v3::Vec3,crate::world_v3::Vec3,f32)],
    )->Result<bool,&'static str> {
        if swept_obstructs(player.0,player.1,player.2,self.upper_gate)? {return Ok(true);}
        for &(from,to,radius) in actors {
            if swept_obstructs(from,to,radius,self.well)?
                || swept_obstructs(from,to,radius,self.upper_gate)? {return Ok(true);}
        }
        Ok(false)
    }

    /// A grounded lower-level disc must fit wholly inside the lower floor or
    /// the connected cabin approach. The real radius is supplied by the KCC.
    pub(crate) fn lower_floor_contains(&self, x:f32,z:f32,radius:f32)->bool {
        if ![x,z,radius].iter().all(|v|v.is_finite()) || radius<=0.0 {return false;}
        if self.service_v2 && super::shaft_lower_service::covers_disc(x,z,radius) {return true;}
        let fits=|a:crate::continuous_kcc::Aabb| x-radius>=a.min_x && x+radius<=a.max_x
            && z-radius>=a.min_z && z+radius<=a.max_z;
        fits(self.lower_floor) || fits(crate::continuous_kcc::Aabb {
            min_x:self.well.min_x,max_x:self.well.max_x,
            min_z:self.well.min_z,max_z:self.lower_floor.max_z,
        })
    }

    /// Shaft-only contract: select authored height bands by authoritative foot
    /// height and retain the actual KCC radius. This does not model a capsule
    /// head or ceiling. Callers must also check original scene colliders.
    pub(crate) fn intersects_wall(&self,x:f32,z:f32,radius:f32,foot_y:f32)->Option<bool> {
        if ![x,z,radius,foot_y].iter().all(|v|v.is_finite()) || radius<=0.0 {return None;}
        Some(self.walls.iter().any(|wall| {
            if foot_y<wall.lower || foot_y>=wall.upper {return false;}
            let dx=x-x.clamp(wall.area.min_x,wall.area.max_x);
            let dz=z-z.clamp(wall.area.min_z,wall.area.max_z);
            dx*dx+dz*dz < radius*radius
        }))
    }
}

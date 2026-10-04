//! Generic vertical support timing. Immutable authored geometry plus the saved
//! authoritative world clock determine every pose; no client phase is trusted.
use crate::{continuous_kcc::Aabb, world_v3::Vec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use sha2::{Digest,Sha256};

pub const FLOOR_M: f32 = 0.0;
pub const CEILING_M: f32 = 3.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MovingSupportDefinition {
    pub id: String,
    pub polygon: Vec<[f32; 2]>,
    pub lower_m: f32,
    pub upper_m: f32,
    pub travel_ms: u32,
    pub endpoint_hold_ms: u32,
}

/// Immutable raised standing surface. It is not a zero-distance moving lift.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct StandingDeckDefinition {
    pub id: String,
    pub polygon: Vec<[f32;2]>,
    pub height_m: f32,
}
impl StandingDeckDefinition {
    pub fn validate(&self,bounds:Aabb)->Result<(),&'static str> {
        validate_geometry(&self.id,&self.polygon,bounds)?;
        if !self.height_m.is_finite() || !(FLOOR_M..=CEILING_M).contains(&self.height_m) {
            return Err("E_STANDING_DECK_HEIGHT_INVALID");
        }
        Ok(())
    }
    pub fn supports(&self,position:Vec3,radius:f32)->bool {
        radius.is_finite() && radius>0.0 && [position.x_m,position.y_m,position.z_m].iter().all(|n|n.is_finite())
            && crate::continuous_kcc::circle_inside_polygon(position.x_m,position.z_m,radius,&self.polygon)
    }
}

/// Authored inclusive footpoint range. All non-default values are explicit
/// content tuning, never inferred from a renderer or a client position request.
#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
#[serde(transparent)]
pub struct HeightRange(pub [f32;2]);
impl HeightRange {
    pub fn validate(self)->Result<(),&'static str> {
        if !self.0.iter().all(|n|n.is_finite()) || self.0[0]<FLOOR_M
            || self.0[1]>CEILING_M || self.0[0]>self.0[1] {return Err("E_HEIGHT_RANGE_INVALID");}
        Ok(())
    }
    pub fn contains(self,height:f32)->bool {
        self.validate().is_ok() && height.is_finite() && (self.0[0]..=self.0[1]).contains(&height)
    }
    pub fn midpoint(self)->f32 {(self.0[0]+self.0[1])*0.5}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportPhase { LowerHold, Rising, UpperHold, Falling, Stationary }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SupportPose {
    pub support_id: String,
    pub phase: SupportPhase,
    pub height_m: f32,
    pub velocity_mps: f32,
    pub phase_elapsed_ms: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SupportFrame {
    pub world_epoch: u64,
    pub server_time_ms: u64,
    pub catalog_sha256: String,
    pub poses: Vec<SupportPose>,
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes()
        .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c))
}

fn validate_geometry(id:&str,polygon:&[[f32;2]],bounds:Aabb)->Result<(),&'static str> {
    if Aabb::new(bounds.min_x,bounds.max_x,bounds.min_z,bounds.max_z).is_err() {return Err("E_SUPPORT_BOUNDS_INVALID");}
    if !valid_id(id) || !(3..=128).contains(&polygon.len()) {return Err("E_SUPPORT_DEFINITION_INVALID");}
    let mut winding=0.0_f32;
    for (index,point) in polygon.iter().enumerate() {
        if !point.iter().all(|n|n.is_finite()) || point[0]<bounds.min_x || point[0]>bounds.max_x
            || point[1]<bounds.min_z || point[1]>bounds.max_z || polygon[..index].contains(point) {
            return Err("E_SUPPORT_POLYGON_INVALID");
        }
        let a=*point;let b=polygon[(index+1)%polygon.len()];let c=polygon[(index+2)%polygon.len()];
        let cross=(b[0]-a[0])*(c[1]-b[1])-(b[1]-a[1])*(c[0]-b[0]);
        if !cross.is_finite() || cross.abs()<1e-6 || (winding!=0.0 && cross.signum()!=winding) {
            return Err("E_SUPPORT_POLYGON_INVALID");
        }
        winding=cross.signum();
        for (other,point) in polygon.iter().enumerate() {
            if other==index || other==(index+1)%polygon.len() {continue;}
            let side=(b[0]-a[0])*(point[1]-a[1])-(b[1]-a[1])*(point[0]-a[0]);
            if !side.is_finite() || side*winding<=1e-6 {return Err("E_SUPPORT_POLYGON_INVALID");}
        }
    }
    Ok(())
}
fn geometry_bounds(polygon:&[[f32;2]])->Result<Aabb,&'static str> {
    Aabb::new(polygon.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),
        polygon.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max),
        polygon.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min),
        polygon.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max))
        .map_err(|_|"E_SUPPORT_POLYGON_INVALID")
}

impl MovingSupportDefinition {
    pub fn validate(&self, bounds: Aabb) -> Result<(), &'static str> {
        validate_geometry(&self.id,&self.polygon,bounds)?;
        if !self.lower_m.is_finite() || !self.upper_m.is_finite()
            || self.lower_m < FLOOR_M || self.upper_m > CEILING_M || self.upper_m-self.lower_m < 0.1
            || !(250..=60_000).contains(&self.travel_ms) || self.endpoint_hold_ms > 30_000 {
            return Err("E_SUPPORT_DEFINITION_INVALID");
        }
        Ok(())
    }

    /// Millisecond arithmetic is bounded before conversion; even a large saved
    /// world time cannot overflow the finite period or create accumulated drift.
    pub fn pose_at(&self, now_ms: u64) -> Result<SupportPose, &'static str> {
        if !valid_id(&self.id) || !self.lower_m.is_finite() || !self.upper_m.is_finite() || self.lower_m<FLOOR_M
            || self.upper_m>CEILING_M || self.upper_m-self.lower_m<0.1
            || !(250..=60_000).contains(&self.travel_ms) || self.endpoint_hold_ms>30_000 {
            return Err("E_SUPPORT_TIMING_INVALID");
        }
        let travel=u64::from(self.travel_ms);
        let hold=u64::from(self.endpoint_hold_ms);
        let period=2*(travel+hold);
        let time=now_ms%period;
        let distance=self.upper_m-self.lower_m;
        let speed=distance*1000.0/self.travel_ms as f32;
        let (phase,height_m,velocity_mps,elapsed)=if time<hold {
            (SupportPhase::LowerHold,self.lower_m,0.0,time)
        } else if time<hold+travel {
            let elapsed=time-hold;
            (SupportPhase::Rising,self.lower_m+distance*(elapsed as f32/self.travel_ms as f32),speed,elapsed)
        } else if time<2*hold+travel {
            (SupportPhase::UpperHold,self.upper_m,0.0,time-hold-travel)
        } else {
            let elapsed=time-2*hold-travel;
            (SupportPhase::Falling,self.upper_m-distance*(elapsed as f32/self.travel_ms as f32),-speed,elapsed)
        };
        Ok(SupportPose {support_id:self.id.clone(),phase,height_m,velocity_mps,phase_elapsed_ms:elapsed as u32})
    }

    /// Full circular foot contact, not merely a center point over the edge.
    pub fn supports(&self, position: Vec3, radius: f32) -> bool {
        radius.is_finite() && radius>0.0 && [position.x_m,position.y_m,position.z_m].iter().all(|v|v.is_finite())
            && crate::continuous_kcc::circle_inside_polygon(position.x_m,position.z_m,radius,&self.polygon)
    }
}

pub fn validate_catalog(definitions:&[MovingSupportDefinition],bounds:Aabb)->Result<(),&'static str> {
    validate_support_catalog(definitions,&[],bounds)
}
pub fn validate_support_catalog(moving:&[MovingSupportDefinition],standing:&[StandingDeckDefinition],bounds:Aabb)->Result<(),&'static str> {
    if moving.len()+standing.len()>64 {return Err("E_SUPPORT_CATALOG_LIMIT");}
    let mut ids=BTreeSet::new();
    for definition in moving {
        definition.validate(bounds)?;
        if !ids.insert(&definition.id) {return Err("E_SUPPORT_DUPLICATE_ID");}
    }
    for definition in standing {
        definition.validate(bounds)?;
        if !ids.insert(&definition.id) {return Err("E_SUPPORT_DUPLICATE_ID");}
    }
    Ok(())
}
pub fn frame(definitions:&[MovingSupportDefinition],world_epoch:u64,server_time_ms:u64)->Result<SupportFrame,&'static str> {
    frame_with_standing(definitions,&[],world_epoch,server_time_ms)
}
pub fn frame_with_standing(moving:&[MovingSupportDefinition],standing:&[StandingDeckDefinition],world_epoch:u64,server_time_ms:u64)->Result<SupportFrame,&'static str> {
    if world_epoch==0 || moving.len()+standing.len()>64 {return Err("E_SUPPORT_FRAME_MISMATCH");}
    let mut seen=BTreeSet::new();
    for definition in moving {
        definition.validate(geometry_bounds(&definition.polygon)?)?;
        if !seen.insert(&definition.id) {return Err("E_SUPPORT_DUPLICATE_ID");}
    }
    for definition in standing {
        definition.validate(geometry_bounds(&definition.polygon)?)?;
        if !seen.insert(&definition.id) {return Err("E_SUPPORT_DUPLICATE_ID");}
    }
    let mut poses=moving.iter().map(|d|d.pose_at(server_time_ms)).collect::<Result<Vec<_>,_>>()?;
    poses.extend(standing.iter().map(|d|SupportPose{support_id:d.id.clone(),phase:SupportPhase::Stationary,
        height_m:d.height_m,velocity_mps:0.0,phase_elapsed_ms:0}));
    poses.sort_by(|a,b|a.support_id.cmp(&b.support_id));
    let mut canonical_moving=moving.iter().collect::<Vec<_>>();canonical_moving.sort_by(|a,b|a.id.cmp(&b.id));
    let mut canonical_standing=standing.iter().collect::<Vec<_>>();canonical_standing.sort_by(|a,b|a.id.cmp(&b.id));
    // Existing moving-only saves retain their exact prior catalog fingerprint.
    let bytes=if standing.is_empty() {serde_json::to_vec(&canonical_moving)} else {
        serde_json::to_vec(&serde_json::json!({"movingSupports":canonical_moving,"standingDecks":canonical_standing}))
    }.map_err(|_|"E_SUPPORT_DEFINITION_INVALID")?;
    Ok(SupportFrame{world_epoch,server_time_ms,catalog_sha256:format!("{:x}",Sha256::digest(bytes)),poses})
}
pub fn validate_frame(value:&SupportFrame,definitions:&[MovingSupportDefinition],world_epoch:u64,server_time_ms:u64)->Result<(),&'static str> {
    validate_frame_with_standing(value,definitions,&[],world_epoch,server_time_ms)
}
pub fn validate_frame_with_standing(value:&SupportFrame,moving:&[MovingSupportDefinition],standing:&[StandingDeckDefinition],world_epoch:u64,server_time_ms:u64)->Result<(),&'static str> {
    if world_epoch==0 || value!=&frame_with_standing(moving,standing,world_epoch,server_time_ms)? {
        return Err("E_SUPPORT_FRAME_MISMATCH");
    }
    Ok(())
}

/// Shape-only persistence validation. Exact source/timing equality is checked
/// separately by the canonical scene KCC before any restored state is installed.
pub fn validate_saved_frame_shape(value: &SupportFrame) -> Result<(), &'static str> {
    if value.world_epoch==0 || value.poses.is_empty() || value.poses.len()>64
        || value.catalog_sha256.len()!=64 || !value.catalog_sha256.bytes().all(|c|c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
        return Err("E_SUPPORT_FRAME_MISMATCH");
    }
    let mut last: Option<&str> = None;
    for pose in &value.poses {
        if !valid_id(&pose.support_id) || last.is_some_and(|id| id>=pose.support_id.as_str())
            || !pose.height_m.is_finite() || !(FLOOR_M..=CEILING_M).contains(&pose.height_m)
            || !pose.velocity_mps.is_finite() || pose.velocity_mps.abs()>12.0
            || pose.phase_elapsed_ms>60_000
            || match pose.phase {
                SupportPhase::Stationary => pose.velocity_mps!=0.0 || pose.phase_elapsed_ms!=0,
                SupportPhase::LowerHold | SupportPhase::UpperHold => pose.velocity_mps!=0.0 || pose.phase_elapsed_ms>30_000,
                SupportPhase::Rising => pose.velocity_mps<=0.0,
                SupportPhase::Falling => pose.velocity_mps>=0.0,
            } { return Err("E_SUPPORT_FRAME_MISMATCH"); }
        last=Some(&pose.support_id);
    }
    Ok(())
}

/// Ephemeral atomic presentation contract. This is not a persisted save record.
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SupportSceneView {
    pub schema_version:u32,
    pub world_id:String,
    pub scene_id:String,
    pub world_epoch:u64,
    pub server_tick:u64,
    pub authority_revision:u64,
    pub server_time_ms:u64,
    /// Raw compiler scene bytes, bound to the existing verified Web manifest.
    /// Unlike the saved support-only fingerprint, this is presentation identity.
    pub scene_source_sha256:String,
    pub poses:Vec<SupportPose>,
    pub rider:SupportRiderView,
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SupportRiderView {
    pub position_m:Vec3,
    pub radius_m:f32,
    pub mode:SupportRiderMode,
    pub support_id:Option<String>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum SupportRiderMode { Surface,Floor,Airborne,Traversal }

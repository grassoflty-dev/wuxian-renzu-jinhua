//! New charge numbers are development tuning. Legacy light/heavy values and
//! their independent attack_serial cadence are deliberately untouched.
use super::*;
use crate::continuous_kcc::StaticKccWorld;
pub const WINDUP_MS:u64=900;
pub const ACTIVE_MS:u64=600;
pub const COOLDOWN_MS:u64=6000;
pub const SPEED:f32=6.0;
pub const MAX_TRAVEL:f32=3.6;
// Total player-center contact radius, also used verbatim by the public warning.
pub const HIT_RADIUS:f32=0.6;
const BODY_RADIUS:f32=0.3;
const MAX_SERIAL:u64=9_007_199_254_740_991;

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ChargeController {
    pub schema_version:u32,
    pub cooldown_remaining_ms:u64,
    pub charge_serial:u64,
    #[serde(deserialize_with="required_attack")]
    pub attack:Option<ChargeAttack>,
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ChargeAttack {
    pub origin_m:Vec3,
    pub direction_m:Vec3,
    pub travel_m:f32,
    pub elapsed_ms:u64,
    pub hit_resolved:bool,
}
fn required_attack<'de,D:serde::Deserializer<'de>>(d:D)->Result<Option<ChargeAttack>,D::Error> {Option::<ChargeAttack>::deserialize(d)}
impl Default for ChargeController {
    fn default()->Self {Self{schema_version:1,cooldown_remaining_ms:0,charge_serial:0,attack:None}}
}
pub(super) fn finite(p:Vec3)->bool {[p.x_m,p.y_m,p.z_m].iter().all(|v|v.is_finite())}
fn point(a:&ChargeAttack,ms:u64)->Vec3 {
    let distance=(SPEED*ms as f32/1000.0).min(a.travel_m);
    Vec3{x_m:a.origin_m.x_m+a.direction_m.x_m*distance,y_m:a.origin_m.y_m,z_m:a.origin_m.z_m+a.direction_m.z_m*distance}
}
fn close(a:Vec3,b:Vec3)->bool { (a.x_m-b.x_m).abs()<0.001 && (a.y_m-b.y_m).abs()<0.001 && (a.z_m-b.z_m).abs()<0.001 }
impl Sentinel {
    pub fn enable_charge(&mut self) {if self.charge_controller.is_none() {
        if self.state==SentinelState::Hit && self.state_remaining_ms>120 {self.state=SentinelState::Stagger;}
        self.charge_controller=Some(ChargeController::default());}}
    pub(super) fn cancel_charge(&mut self) {
        if let Some(c)=&mut self.charge_controller {if c.attack.take().is_some() {c.cooldown_remaining_ms=COOLDOWN_MS;}}
    }
    pub fn validate(&self)->bool {
        if self.entity_id.trim().is_empty() || !finite(self.position_m) || (self.hp==0 && self.active) {return false;}
        let Some(c)=&self.charge_controller else {
            return !matches!(self.state,SentinelState::ChargeWindup|SentinelState::Charge|SentinelState::Stagger);
        };
        if c.schema_version!=1 || c.cooldown_remaining_ms>COOLDOWN_MS || c.charge_serial>MAX_SERIAL
            || self.attack_serial>MAX_SERIAL || self.state_remaining_ms>COOLDOWN_MS
            || (self.hp==0 && (self.state!=SentinelState::Death || self.state_remaining_ms!=0))
            || (self.hp>0 && self.state==SentinelState::Death) {return false;}
        let attacking=matches!(self.state,SentinelState::ChargeWindup|SentinelState::Charge);
        if attacking!=c.attack.is_some() {return false;}
        let Some(a)=&c.attack else {
            let max=match self.state {SentinelState::Chase|SentinelState::Death=>0,SentinelState::Attack=>250,
                SentinelState::HeavyAttack=>400,SentinelState::Recover=>500,SentinelState::Hit=>120,SentinelState::Stagger=>900,_=>return false};
            return self.state_remaining_ms<=max && (c.cooldown_remaining_ms==0 || c.charge_serial>0);
        };
        if !finite(a.origin_m)||!finite(a.direction_m)||a.direction_m.y_m!=0.0
            || (a.direction_m.x_m.hypot(a.direction_m.z_m)-1.0).abs()>0.0001
            || !a.travel_m.is_finite() || a.travel_m<=0.0 || a.travel_m>MAX_TRAVEL
            || c.charge_serial==0 || c.cooldown_remaining_ms!=0 {return false;}
        match self.state {
            SentinelState::ChargeWindup=>self.state_remaining_ms>0 && self.state_remaining_ms<=WINDUP_MS
                && a.elapsed_ms==0 && !a.hit_resolved && close(self.position_m,a.origin_m),
            SentinelState::Charge=>a.elapsed_ms<ACTIVE_MS && (a.elapsed_ms>0 || !a.hit_resolved) && self.state_remaining_ms==ACTIVE_MS-a.elapsed_ms
                && close(self.position_m,point(a,a.elapsed_ms)),
            _=>false,
        }
    }
    pub fn charge_geometry_valid(&self,kcc:&StaticKccWorld)->bool {
        self.validate() && self.charge_controller.as_ref().and_then(|c|c.attack.as_ref()).is_none_or(|a|
            close(supported_endpoint(a.origin_m,point(a,ACTIVE_MS),kcc),point(a,ACTIVE_MS)))
    }
}
fn lerp(a:Vec3,b:Vec3,t:f32)->Vec3 {Vec3{x_m:a.x_m+(b.x_m-a.x_m)*t,y_m:a.y_m,z_m:a.z_m+(b.z_m-a.z_m)*t}}
// At most 5 cm between tested actor discs (30 cm radius), so even a thin wall
// cannot fall between samples. Static deck/terrain eligibility is checked too.
fn supported_endpoint(from:Vec3,to:Vec3,kcc:&StaticKccWorld)->Vec3 {
    let steps=(horizontal_distance(from,to)/0.05).ceil().max(1.0) as usize;
    let mut endpoint=from;
    for n in 0..=steps {let p=lerp(from,to,n as f32/steps as f32);
        if !kcc.can_occupy(p,BODY_RADIUS)||!kcc.stable_actor_footprint(p,BODY_RADIUS) {break;}
        endpoint=p;
    }
    endpoint
}
fn line_clear(from:Vec3,to:Vec3,kcc:&StaticKccWorld)->bool {
    let steps=(horizontal_distance(from,to)/0.025).ceil().max(1.0) as usize;
    (0..=steps).all(|n|kcc.can_occupy(lerp(from,to,n as f32/steps as f32),0.05))
}
fn contact(from:Vec3,to:Vec3,player:Vec3,kcc:&StaticKccWorld)->bool {
    if !crate::continuous_combat::combat_vertical_overlap(from,player) {return false;}
    let dx=to.x_m-from.x_m; let dz=to.z_m-from.z_m;let len=dx*dx+dz*dz;
    let t=if len>0.0 {((player.x_m-from.x_m)*dx+(player.z_m-from.z_m)*dz)/len} else {0.0};
    let nearest=lerp(from,to,t.clamp(0.0,1.0));
    horizontal_distance(nearest,player)<=HIT_RADIUS && line_clear(nearest,player,kcc)
}
fn changed(s:&Sentinel)->SentinelEvent {SentinelEvent::StateChanged{sentinel_id:s.entity_id.clone(),state:s.state}}
fn recover(s:&mut Sentinel) {s.cancel_charge();s.state=SentinelState::Recover;s.state_remaining_ms=500;}
pub(super) fn tick_charge(s:&mut Sentinel,player:Vec3,dt_ms:u64,kcc:&StaticKccWorld)->Option<(Vec<SentinelEvent>,Vec<CombatEvent>)> {
    let controller=s.charge_controller.as_mut()?;
    controller.cooldown_remaining_ms=controller.cooldown_remaining_ms.saturating_sub(dt_ms);
    if s.state==SentinelState::ChargeWindup {
        s.state_remaining_ms=s.state_remaining_ms.saturating_sub(dt_ms);
        if s.state_remaining_ms>0 {return Some((vec![],vec![]));}
        s.state=SentinelState::Charge;s.state_remaining_ms=ACTIVE_MS;
        return Some((vec![changed(s)],vec![]));
    }
    if s.state==SentinelState::Charge {
        let mut a=controller.attack.clone().unwrap();let elapsed=(a.elapsed_ms+dt_ms).min(ACTIVE_MS);
        let from=s.position_m;let intended=point(&a,elapsed);let endpoint=supported_endpoint(from,intended,kcc);
        let hit=!a.hit_resolved && contact(from,endpoint,player,kcc);a.hit_resolved|=hit;
        a.elapsed_ms=elapsed;s.position_m=endpoint;s.state_remaining_ms=ACTIVE_MS-elapsed;
        controller.attack=Some(a.clone());let serial=controller.charge_serial;
        let mut events=vec![SentinelEvent::Moved{sentinel_id:s.entity_id.clone(),position_m:endpoint}];
        let mut combat=vec![];
        if hit {events.push(SentinelEvent::DamagedPlayer{sentinel_id:s.entity_id.clone(),damage:16,attack_serial:serial});
            combat.push(CombatEvent::PlayerDamaged{source_id:s.entity_id.clone(),damage:16,contact:Some(crate::continuous_combat::CombatContact::new(from,player,None))});}
        if !close(endpoint,intended)||elapsed==ACTIVE_MS||horizontal_distance(a.origin_m,endpoint)+0.001>=a.travel_m {
            recover(s);events.push(changed(s));
        }
        return Some((events,combat));
    }
    let distance=horizontal_distance(s.position_m,player);
    if s.state==SentinelState::Chase && s.state_remaining_ms==0 && controller.cooldown_remaining_ms==0
        && (2.5..=6.5).contains(&distance) && crate::continuous_combat::combat_vertical_overlap(s.position_m,player)
        && line_clear(s.position_m,player,kcc) {
        let Some(serial)=controller.charge_serial.checked_add(1).filter(|s|*s<=MAX_SERIAL) else {return Some((vec![],vec![]));};
        let direction=Vec3{x_m:(player.x_m-s.position_m.x_m)/distance,y_m:0.0,z_m:(player.z_m-s.position_m.z_m)/distance};
        let end=Vec3{x_m:s.position_m.x_m+direction.x_m*MAX_TRAVEL,y_m:s.position_m.y_m,z_m:s.position_m.z_m+direction.z_m*MAX_TRAVEL};
        let travel=horizontal_distance(s.position_m,supported_endpoint(s.position_m,end,kcc)).min(MAX_TRAVEL);
        if travel>0.001 {controller.charge_serial=serial;controller.attack=Some(ChargeAttack{origin_m:s.position_m,direction_m:direction,travel_m:travel,elapsed_ms:0,hit_resolved:false});
            s.state=SentinelState::ChargeWindup;s.state_remaining_ms=WINDUP_MS;return Some((vec![changed(s)],vec![]));}
    }
    None
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SentinelWarningView {pub shape:String,pub origin_m:Vec3,pub direction_rad:f32,pub radius_m:f32,pub range_m:f32}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SentinelEncounterView {pub actor_id:String,pub phase:String,pub remaining_ms:u64,pub attack_serial:u64,pub position_m:Vec3,pub warning:Option<SentinelWarningView>}
impl Sentinel {
    pub fn encounter_view(&self,hz:u32)->Option<SentinelEncounterView> {
        if !self.validate() || hz==0 {return None;}
        // Deactivate is not death. Keep its saved phase/timer frozen, but do
        // not publish a live threat or counterplay countdown while inactive.
        if !self.active && self.hp>0 {
            return Some(SentinelEncounterView{actor_id:self.entity_id.clone(),phase:"inactive".into(),remaining_ms:0,
                attack_serial:self.attack_serial,position_m:self.position_m,warning:None});
        }
        let phase=match self.state {SentinelState::Chase=>"chase",SentinelState::Attack=>"light_windup",SentinelState::HeavyAttack=>"heavy_windup",SentinelState::ChargeWindup=>"charge_windup",SentinelState::Charge=>"charge",SentinelState::Recover=>"recover",SentinelState::Hit=>"hit",SentinelState::Stagger=>"stagger",SentinelState::Death=>"dead"};
        let warning=match self.state {
            SentinelState::Attack|SentinelState::HeavyAttack=>{let radius=if self.state==SentinelState::Attack {1.8}else{2.2};Some(SentinelWarningView{shape:"circle".into(),origin_m:self.position_m,direction_rad:0.0,radius_m:radius,range_m:0.0})},
            SentinelState::ChargeWindup|SentinelState::Charge=>{let a=self.charge_controller.as_ref()?.attack.as_ref()?;Some(SentinelWarningView{shape:"corridor".into(),origin_m:a.origin_m,direction_rad:a.direction_m.x_m.atan2(a.direction_m.z_m),radius_m:HIT_RADIUS,range_m:a.travel_m})},_=>None};
        let serial=if matches!(self.state,SentinelState::ChargeWindup|SentinelState::Charge) {self.charge_controller.as_ref()?.charge_serial} else {self.attack_serial};
        let step=(1000.0/hz as f32).round() as u64;
        let ticks=self.state_remaining_ms.div_ceil(step.max(1)).max(u64::from(matches!(self.state,SentinelState::Attack|SentinelState::HeavyAttack)));
        let remaining=(ticks*1000).div_ceil(hz as u64);
        Some(SentinelEncounterView{actor_id:self.entity_id.clone(),phase:phase.into(),remaining_ms:remaining,attack_serial:serial,position_m:self.position_m,warning})
    }
}

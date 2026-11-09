//! Unadmitted lower-service recipe and exact planar support predicates.
//! This module alone does not select a physical, visual, save or information variant.
use std::sync::OnceLock;
use serde_json::Value;
// Kept false until Continue staging, client frame commitment and all remaining
// consumers are bound together. A client cannot opt into this candidate early.
pub(super) const ADMISSION_READY:bool=true;

impl crate::moving_support::ShaftLowerRecipeIdentity {
 pub(crate) fn for_scene(scene:&crate::scene_runtime::SceneDefinition)->Option<Self> {
  candidate_geometry(scene)?;Some(Self{recipe_version:2,scene_source_sha256:scene.verified_source_sha256.clone()})
 }
 pub(crate) fn matches_scene(&self,scene:&crate::scene_runtime::SceneDefinition)->bool {
  self.recipe_version==2 && Self::for_scene(scene).as_ref()==Some(self)
 }
}
#[derive(Clone,Copy,Debug,PartialEq)]
struct Rect { x0:f64,z0:f64,x1:f64,z1:f64 }
impl Rect {
 fn parse(v:&Value)->Option<Self> {let a=v.as_array()?;if a.len()!=4{return None;}
  let r=Self{x0:a[0].as_f64()?,z0:a[1].as_f64()?,x1:a[2].as_f64()?,z1:a[3].as_f64()?};
  ([r.x0,r.z0,r.x1,r.z1].iter().all(|n|n.is_finite())&&r.x0<r.x1&&r.z0<r.z1).then_some(r)}
 fn interior(self,x:f64,z:f64)->bool{x>self.x0&&x<self.x1&&z>self.z0&&z<self.z1}
}
#[derive(Debug)]
struct Recipe { floors:Vec<Rect>, walls:Vec<(String,Rect)> }
const EXPECTED_WALLS:&[(&str,[f64;4])]=&[
 ("connector_north_a",[19.75,5.0,23.25,5.25]),
 ("connector_north_turn",[23.25,5.25,23.5,6.5]),
 ("connector_north_b",[24.0,6.25,26.0,6.5]),
 ("connector_south_a",[19.75,9.25,23.0,9.5]),
 ("connector_south_b",[23.0,9.5,26.0,9.75]),
 ("sector_north",[26.0,1.0,50.0,1.25]),
 ("sector_south",[26.0,20.75,50.0,21.0]),
 ("sector_east",[49.75,1.25,50.0,20.75]),
 ("sector_west_north",[26.0,1.25,26.25,6.5]),
 ("sector_west_south",[26.0,9.5,26.25,20.75]),
 ("north_bays_front_0",[26.25,6.75,29.0,7.0]),
 ("north_bays_front_1",[31.0,6.75,37.0,7.0]),
 ("north_bays_front_2",[39.0,6.75,45.0,7.0]),
 ("north_bays_front_last",[47.0,6.75,49.75,7.0]),
 ("north_divider_34",[33.875,1.25,34.125,6.75]),
 ("north_divider_42",[41.875,1.25,42.125,6.75]),
 ("middle_bays_front_0",[29.0,10.0,31.0,10.25]),
 ("middle_bays_front_1",[33.0,10.0,37.0,10.25]),
 ("middle_bays_front_2",[39.0,10.0,43.0,10.25]),
 ("middle_bays_front_last",[45.0,10.0,47.0,10.25]),
 ("middle_bays_rear_0",[29.0,13.75,31.0,14.0]),
 ("middle_bays_rear_1",[33.0,13.75,37.0,14.0]),
 ("middle_bays_rear_2",[39.0,13.75,43.0,14.0]),
 ("middle_bays_rear_last",[45.0,13.75,47.0,14.0]),
 ("middle_divider_35",[34.875,10.25,35.125,13.75]),
 ("middle_divider_41",[40.875,10.25,41.125,13.75]),
 ("middle_west",[28.75,10.0,29.0,14.0]),
 ("middle_east",[47.0,10.0,47.25,14.0]),
 ("south_bays_front_0",[26.25,17.0,29.0,17.25]),
 ("south_bays_front_1",[31.0,17.0,37.0,17.25]),
 ("south_bays_front_2",[39.0,17.0,45.0,17.25]),
 ("south_bays_front_last",[47.0,17.0,49.75,17.25]),
 ("south_divider_34",[33.875,17.25,34.125,20.75]),
 ("south_divider_42",[41.875,17.25,42.125,20.75]),
];
static RECIPE:OnceLock<Option<Recipe>>=OnceLock::new();
fn recipe()->Option<&'static Recipe>{RECIPE.get_or_init(||{
 let v:Value=serde_json::from_str(include_str!("../../../apps/godot_playable/data/gh_shaft_lower_service_v2.local.json")).ok()?;
 if v["schema"]!="gh-lower-service/v2"||v["status"]!="candidate-not-admitted"||v["scene"]!="gh_central_shaft"
  ||v["source_sha"]!="6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832"||v["floor_y"]!=-4{return None;}
 let floors:Vec<_>=v["floor_union_rects"].as_array()?.iter().map(Rect::parse).collect::<Option<_>>()?;
 let expected=[[1.5,5.,19.5,14.],[19.5,5.25,23.5,9.25],[23.,6.5,26.25,9.5],[26.,1.,50.,21.]];
 if floors.len()!=expected.len()||floors.iter().zip(expected).any(|(a,b)|[a.x0,a.z0,a.x1,a.z1]!=b){return None;}
 if v["old_owner_cut"]!=serde_json::json!([19.5,5.25,19.75,7.75])
  ||v["retained_spans"]!=serde_json::json!([[19.5,4.75,19.75,5.25],[19.5,7.75,19.75,14.25]]){return None;}
 let rows=v["new_walls"].as_array()?;if rows.len()!=34{return None;}
 let mut ids=std::collections::BTreeSet::new();let mut walls=Vec::new();for (row,(expected_id,expected_rect)) in rows.iter().zip(EXPECTED_WALLS) {
  let id=row["id"].as_str()?;if id.is_empty()||!ids.insert(id)||row["band"]!=serde_json::json!([-4,-1.6]){return None;}
  let rect=Rect::parse(&row["rect_xz"])?;if id!=*expected_id || [rect.x0,rect.z0,rect.x1,rect.z1]!=*expected_rect{return None;}walls.push((id.to_owned(),rect));
 }
 Some(Recipe{floors,walls})
}).as_ref()}

/// Build only a candidate: no runtime owner or save selects this variant here.
/// The lower_east cut is accepted only against the exact retained old owner.
pub(crate) fn candidate_geometry(scene:&crate::scene_runtime::SceneDefinition)->Option<super::shaft_lift::ShaftGeometry> {
 if scene.verified_source_sha256!="6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832"{return None;}
 let pack=recipe()?;let mut geometry=super::shaft_lift::ShaftGeometry::from_scene(scene)?;
 let indices:Vec<_>=geometry.walls.iter().enumerate().filter(|(_,w)|w.id=="lower_east").map(|(i,_)|i).collect();
 let [index]=indices.as_slice() else{return None;};let old=&geometry.walls[*index];
 if old.area!=crate::continuous_kcc::Aabb::new(19.5,19.75,4.75,14.25).ok()? || old.lower!= -4. || old.upper!= -1.6{return None;}
 geometry.walls.remove(*index);geometry.service_v2=true;
 let wall=|id:&str,r:Rect|Some(super::shaft_lift::ShaftWall{id:id.into(),area:crate::continuous_kcc::Aabb::new(r.x0 as f32,r.x1 as f32,r.z0 as f32,r.z1 as f32).ok()?,lower:-4.,upper:-1.6});
 geometry.walls.push(wall("lower_east_retained_north",Rect{x0:19.5,z0:4.75,x1:19.75,z1:5.25})?);
 geometry.walls.push(wall("lower_east_retained_south",Rect{x0:19.5,z0:7.75,x1:19.75,z1:14.25})?);
 for (id,r) in &pack.walls {if geometry.walls.iter().any(|w|w.id==*id){return None;}geometry.walls.push(wall(id,*r)?);}
 // The old ordinary collision is upper-only in the existing KCC. These two
 // explicitly NEW lower-band boundary owners preserve its authored aperture.
 if scene.collision.len()!=8{return None;}
 for (source,id,rect) in [
  ("shaft_wall_east_north","sector_aperture_lower_north",Rect{x0:23.5,z0:0.5,x1:24.,z1:6.5}),
  ("shaft_wall_east_south","sector_aperture_lower_south",Rect{x0:23.5,z0:9.5,x1:24.,z1:15.5})] {
  let mut matches=scene.collision.iter().filter(|w|w.id==source);let existing=matches.next()?;
  if matches.next().is_some() || existing.polygon!=vec![[rect.x0 as f32,rect.z0 as f32],[rect.x1 as f32,rect.z0 as f32],[rect.x1 as f32,rect.z1 as f32],[rect.x0 as f32,rect.z1 as f32]] {return None;}
  if geometry.walls.iter().any(|w|w.id==id){return None;}geometry.walls.push(wall(id,rect)?);
 }
 Some(geometry)
}
/// Prepare, but never install, an exact candidate from an already valid old
/// Shaft owner. The caller must separately coordinate target frame readiness.
pub(super) fn candidate_from_old(scene:&crate::scene_runtime::SceneDefinition,old:&super::shaft_lift::ShaftOwner,
 world:&crate::world_v3::WorldStateV3)->Option<super::shaft_lift::ShaftOwner> {
 if old.geometry.service_v2 || old.clock.world_ms()!=world.server_time_ms
  || super::shaft_lift::ShaftGeometry::from_scene(scene).as_ref()!=Some(&old.geometry)
  || !old.clear_at_entry(world).ok()? {return None;}
 Some(super::shaft_lift::ShaftOwner{geometry:candidate_geometry(scene)?,clock:old.clock.clone()})
}
/// Encode a provisional owner without changing the established support
/// catalog fingerprint. This does not write a Save or install the owner.
pub(crate) fn candidate_frame(owner:&super::shaft_lift::ShaftOwner,scene:&crate::scene_runtime::SceneDefinition,epoch:u64)->Result<crate::moving_support::SupportFrame,&'static str>{
 let geometry=candidate_geometry(scene).ok_or("E_SHAFT_SERVICE_SCENE")?;
 if owner.geometry!=geometry{return Err("E_SHAFT_SERVICE_GEOMETRY");}
 let old=super::shaft_lift::ShaftOwner{geometry:super::shaft_lift::ShaftGeometry::from_scene(scene).ok_or("E_SHAFT_SERVICE_SCENE")?,clock:owner.clock.clone()};
 let mut frame=old.frame(scene,epoch)?;frame.shaft_lower_recipe=Some(crate::moving_support::ShaftLowerRecipeIdentity::for_scene(scene).ok_or("E_SHAFT_SERVICE_SCENE")?);Ok(frame)
}
/// Prepare a candidate with the exact saved catalog and time before any
/// bounds validation; callers must not install it before matching client and
/// information preflight, and must separately validate all live bodies.
pub(crate) fn candidate_from_frame(scene:&crate::scene_runtime::SceneDefinition,frame:&crate::moving_support::SupportFrame,epoch:u64,time:u64)->Result<super::shaft_lift::ShaftOwner,&'static str>{
 let id=frame.shaft_lower_recipe.as_ref().ok_or("E_SHAFT_SERVICE_IDENTITY")?;
 if !id.matches_scene(scene){return Err("E_SHAFT_SERVICE_IDENTITY");}
 let mut legacy=frame.clone();legacy.shaft_lower_recipe=None;
 let mut owner=super::shaft_lift::ShaftOwner::restore_frame(scene,&legacy,epoch,time)?;
 owner.geometry=candidate_geometry(scene).ok_or("E_SHAFT_SERVICE_SCENE")?;Ok(owner)
}
// Information candidates only. Membership is not an observation, permission,
// visit, or authority to render a body. Full admission remains a separate step.
const REGION_PARTS:&[(&str,&[[f64;4]])]=&[
 ("pipe_fittings",&[[26.25,1.25,33.875,6.75],[29.,6.75,31.,7.]]),
 ("cable_sorting",&[[34.125,1.25,41.875,6.75],[37.,6.75,39.,7.]]),
 ("filter_storage",&[[42.125,1.25,49.75,6.75],[45.,6.75,47.,7.]]),
 ("seal_assembly",&[[29.,10.25,34.875,13.75],[31.,10.,33.,10.25],[31.,13.75,33.,14.]]),
 ("tool_check",&[[35.125,10.25,40.875,13.75],[37.,10.,39.,10.25],[37.,13.75,39.,14.]]),
 ("equipment_service",&[[41.125,10.25,47.,13.75],[43.,10.,45.,10.25],[43.,13.75,45.,14.]]),
 ("turnover_storage",&[[26.25,17.25,33.875,20.75],[29.,17.,31.,17.25]]),
 ("records_sorting",&[[34.125,17.25,41.875,20.75],[37.,17.,39.,17.25]]),
 ("shift_recess",&[[42.125,17.25,49.75,20.75],[45.,17.,47.,17.25]]),
 ("distribution_spine",&[[26.25,7.,49.75,10.]]),
 ("west_return_link",&[[26.25,10.,28.75,14.]]),
 ("east_return_link",&[[47.25,10.,49.75,14.]]),
 ("rear_return_spine",&[[26.25,14.,49.75,17.]]),
 ("connector",&[[19.5,5.25,19.75,7.75],[19.75,5.25,23.25,9.25],
  [23.25,6.5,23.5,9.25],[23.5,6.5,26.25,9.5],[23.,9.25,23.5,9.5]]),
];
/// Complete candidate part rows only; producing rows does not record a visit.
pub(crate) fn candidate_rows(parent:&str)->Option<Vec<crate::world_v3::ExploredRoom>> {
 let short=parent.strip_prefix("grey_hive.gh_central_shaft.lower_service_v2.")?;
 let (_,parts)=REGION_PARTS.iter().find(|(id,_)|*id==short)?;
 Some(parts.iter().enumerate().map(|(i,p)|crate::world_v3::ExploredRoom{
  room_id:format!("{parent}:part{i}"),outline_m:vec![
   crate::world_v3::Vec3{x_m:p[0] as f32,y_m:-4.,z_m:p[1] as f32},crate::world_v3::Vec3{x_m:p[2] as f32,y_m:-4.,z_m:p[1] as f32},
   crate::world_v3::Vec3{x_m:p[2] as f32,y_m:-4.,z_m:p[3] as f32},crate::world_v3::Vec3{x_m:p[0] as f32,y_m:-4.,z_m:p[3] as f32}] }).collect())
}
pub(crate) fn region_at(x:f64,z:f64)->Option<&'static str>{
 if !x.is_finite()||!z.is_finite(){return None;}let mut found=None;
 for (id,parts) in REGION_PARTS {if parts.iter().any(|p|Rect{x0:p[0],z0:p[1],x1:p[2],z1:p[3]}.interior(x,z)) {
  if found.is_some(){return None;}found=Some(*id);
 }}found
}
pub(super) fn selected(state:&super::RuntimeState,scene:&crate::scene_runtime::SceneDefinition)->bool {
 state.world.world_id=="grey_hive" && state.world.scene_id=="gh_central_shaft" && state.world.explored.world_id=="grey_hive"
  && state.pending_entry.is_none() && state.pending_exit.is_none() && state.world.player_hp>0 && state.last_owner_error.is_none()
  && state.shaft_owner.as_ref().is_some_and(|o|o.geometry.service_v2 && candidate_geometry(scene).as_ref()==Some(&o.geometry))
  && crate::moving_support::ShaftLowerRecipeIdentity::for_scene(scene).as_ref()==state.world_persistent_v1.shaft_recipe_format_v2.as_ref()
}
pub(super) fn history_ready(state:&super::RuntimeState,scene:&crate::scene_runtime::SceneDefinition,key:Option<&str>)->bool {
 selected(state,scene) && key.is_some_and(|k|k.starts_with("shaft_lower_"))
}
pub(super) fn history_rows(state:&super::RuntimeState)->Vec<crate::world_v3::ExploredRoom> {
 let mut out=Vec::new();
 let Some(archive)=state.world_persistent_v1.lower_service_map_v1.as_ref().filter(|a|a.validate() && Some(&a.recipe)==state.world_persistent_v1.shaft_recipe_format_v2.as_ref()) else{return out;};
 for (short,_) in REGION_PARTS {
  let id=format!("grey_hive.gh_central_shaft.lower_service_v2.{short}");let Some(expected)=candidate_rows(&id) else{continue;};
  let prefix=format!("{id}:part");let active:Vec<_>=state.world.explored.rooms.iter().filter(|r|r.room_id.starts_with(&prefix)).collect();
  let valid_active=active.len()==expected.len() && expected.iter().all(|e|active.iter().filter(|a|a.room_id==e.room_id).count()==1 && active.iter().any(|a|*a==e));
  let archived=archive.records.iter().find(|r|r.parent_id==id);
  if archived.is_some_and(|r|r.rows==expected) && (valid_active || active.is_empty()) {out.extend(expected);}
 }
 out
}
/// Records one fully qualified, actually occupied parent after a committed
/// lower-floor step. No preview, pause, surface, or unselected owner may write.
pub(super) fn observe(state:&mut super::RuntimeState,scene:&super::SceneRuntime,support:Option<&crate::moving_support::SupportSceneView>)->bool {
 if state.paused || state.pending_entry.is_some() || state.pending_exit.is_some() || state.last_owner_error.is_some()
  || !state.shaft_owner.as_ref().is_some_and(|o|o.geometry.service_v2)
  || !super::shaft_floor_region_eligible(state,scene,support,-4.) || state.world.explored.world_id!="grey_hive" {return false;}
 let p=state.world.player.position_m;let Some(short)=region_at(f64::from(p.x_m),f64::from(p.z_m)) else{return false;};
 let parent=format!("grey_hive.gh_central_shaft.lower_service_v2.{short}");let Some(rows)=candidate_rows(&parent) else{return false;};
 let prefix=format!("{parent}:part");if state.world.explored.rooms.iter().any(|r|r.room_id.starts_with(&prefix) && !rows.iter().any(|e|e.room_id==r.room_id)){return false;}
 let Some(identity)=crate::moving_support::ShaftLowerRecipeIdentity::for_scene(scene.current_scene()) else{return false;};
 if state.world_persistent_v1.shaft_recipe_format_v2.as_ref()!=Some(&identity){return false;}
 for row in &rows {let found:Vec<_>=state.world.explored.rooms.iter().filter(|r|r.room_id==row.room_id).collect();
  if found.len()>1 || found.first().is_some_and(|r|*r!=row){return false;}}
 let mut persistent=state.world_persistent_v1.clone();
 let archive=persistent.lower_service_map_v1.get_or_insert_with(||crate::world_persistent_v1::LowerServiceMapArchive{schema_version:1,recipe:identity.clone(),records:vec![]});
 if archive.recipe!=identity{return false;}
 if let Some(existing)=archive.records.iter().find(|r|r.parent_id==parent) {if existing.rows!=rows{return false;}}
 else {archive.records.push(crate::world_persistent_v1::LowerServiceMapRecord{parent_id:parent.clone(),rows:rows.clone()});}
 if persistent.validate().is_err(){return false;}
 for row in rows {if !state.world.explored.rooms.iter().any(|r|r.room_id==row.room_id){state.world.explored.rooms.push(row);}}
 state.world.explored.player_position_m=p;state.world_persistent_v1=persistent;
 state.dense_region_previous=Some((scene.scene_id.clone(),scene.world_epoch,parent));true
}
// Reject a disc or axis-aligned swept disc that intersects any uncovered cell
// of the exact rectangle union. Cell partition edges come only from the pack.
fn covered_sweep(parts:&[Rect],from:[f64;2],to:[f64;2],radius:f64)->bool {
 if parts.is_empty()||![from[0],from[1],to[0],to[1],radius].iter().all(|n|n.is_finite())||radius<=0.
  ||(from[0]!=to[0]&&from[1]!=to[1]){return false;}
 let (x0,x1)=(from[0].min(to[0])-radius,from[0].max(to[0])+radius);
 let (z0,z1)=(from[1].min(to[1])-radius,from[1].max(to[1])+radius);
 let mut xs=vec![x0,x1];let mut zs=vec![z0,z1];
 for p in parts {for x in [p.x0,p.x1] {if x>x0&&x<x1{xs.push(x);}}for z in [p.z0,p.z1] {if z>z0&&z<z1{zs.push(z);}}}
 xs.sort_by(f64::total_cmp);xs.dedup();zs.sort_by(f64::total_cmp);zs.dedup();
 let (sx0,sx1)=(from[0].min(to[0]),from[0].max(to[0]));let (sz0,sz1)=(from[1].min(to[1]),from[1].max(to[1]));
 for x in xs.windows(2){for z in zs.windows(2){let mx=(x[0]+x[1])/2.;let mz=(z[0]+z[1])/2.;
  if parts.iter().any(|p|p.interior(mx,mz)){continue;}
  let dx=(x[0]-sx1).max(sx0-x[1]).max(0.);let dz=(z[0]-sz1).max(sz0-z[1]).max(0.);
  if dx*dx+dz*dz<radius*radius{return false;}
 }}true
}

pub(crate) fn covers_disc(x:f32,z:f32,r:f32)->bool {
 recipe().is_some_and(|p|covered_sweep(&p.floors,[f64::from(x),f64::from(z)],[f64::from(x),f64::from(z)],f64::from(r)))
}
pub(crate) fn covers_axis_sweep(from:crate::world_v3::Vec3,to:crate::world_v3::Vec3,r:f32,lower_hold:bool)->bool {
 recipe().is_some_and(|p|{let mut floors=p.floors.clone();
  if lower_hold {floors.push(Rect{x0:1.5,z0:1.,x1:5.5,z1:5.});}
  covered_sweep(&floors,[f64::from(from.x_m),f64::from(from.z_m)],[f64::from(to.x_m),f64::from(to.z_m)],f64::from(r))})
}
/// Pure candidate-only lower-floor sweep: selected support and all explicit
/// height-banded candidate walls are checked together. Never installs an owner.
pub(crate) fn candidate_sweep_geometry(geometry:&super::shaft_lift::ShaftGeometry,from:crate::world_v3::Vec3,to:crate::world_v3::Vec3,radius:f32,lower_hold:bool)->bool {
 if !geometry.service_v2 || !covers_axis_sweep(from,to,radius,lower_hold){return false;}
 let (x0,x1)=(f64::from(from.x_m.min(to.x_m)),f64::from(from.x_m.max(to.x_m)));
 let (z0,z1)=(f64::from(from.z_m.min(to.z_m)),f64::from(from.z_m.max(to.z_m)));let radius=f64::from(radius);
 !geometry.walls.iter().filter(|w|w.lower<=-4. && w.upper> -4.).any(|w|{
  let dx=(f64::from(w.area.min_x)-x1).max(x0-f64::from(w.area.max_x)).max(0.);
  let dz=(f64::from(w.area.min_z)-z1).max(z0-f64::from(w.area.max_z)).max(0.);
  dx*dx+dz*dz<radius*radius
 })
}
#[cfg(test)]
fn candidate_lower_sweep(scene:&crate::scene_runtime::SceneDefinition,from:[f64;2],to:[f64;2],radius:f64)->bool {
 let Some(geometry)=candidate_geometry(scene) else{return false;};
 let from=crate::world_v3::Vec3{x_m:from[0] as f32,y_m:-4.,z_m:from[1] as f32};
 let to=crate::world_v3::Vec3{x_m:to[0] as f32,y_m:-4.,z_m:to[1] as f32};
 candidate_sweep_geometry(&geometry,from,to,radius as f32,false)
}
#[cfg(test)]
mod tests {use super::*;
 #[test] fn lower_service_frame_identity_is_strict(){
  let mut frame=crate::moving_support::SupportFrame{world_epoch:1,server_time_ms:0,catalog_sha256:"0".repeat(64),shaft_lower_recipe:None,
   poses:vec![super::super::shaft_lift::ShaftClock::at_upper(0).pose()]};
  let old=serde_json::to_value(&frame).unwrap();assert!(old.get("shaftLowerRecipe").is_none());
  assert!(super::super::shaft_lift::saved_frame_shape(&frame,1,0));
  let mut null=old.clone();null["shaftLowerRecipe"]=serde_json::Value::Null;
  assert!(serde_json::from_value::<crate::moving_support::SupportFrame>(null).is_err());
  frame.shaft_lower_recipe=Some(crate::moving_support::ShaftLowerRecipeIdentity{recipe_version:2,scene_source_sha256:"6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832".into()});
  let encoded=serde_json::to_value(&frame).unwrap();assert_eq!(serde_json::from_value::<crate::moving_support::SupportFrame>(encoded).unwrap(),frame);
  // Exact JSON.stringify output from the client ready-proof fixture.
  let wire=r#"{"shaftLowerService":{"recipeVersion":2,"sceneSourceSha256":"6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832"}}"#;
  let value:serde_json::Value=serde_json::from_str(wire).unwrap();
  let decoded:crate::moving_support::ShaftLowerRecipeIdentity=serde_json::from_value(value["shaftLowerService"].clone()).unwrap();
  assert_eq!(Some(decoded),frame.shaft_lower_recipe);assert!(serde_json::from_str::<crate::moving_support::ShaftLowerRecipeIdentity>(r#"{"recipeVersion":2.0,"sceneSourceSha256":"wrong"}"#).is_err());
  assert!(super::super::shaft_lift::saved_frame_shape(&frame,1,0));
  let mut wrong=frame.clone();wrong.shaft_lower_recipe.as_mut().unwrap().recipe_version=3;assert!(!super::super::shaft_lift::saved_frame_shape(&wrong,1,0));
  let mut wrong=frame.clone();wrong.shaft_lower_recipe.as_mut().unwrap().scene_source_sha256="0".repeat(64);assert!(!super::super::shaft_lift::saved_frame_shape(&wrong,1,0));
  let mut scene:crate::scene_runtime::SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_central_shaft.json")).unwrap();
  scene.verified_source_sha256=frame.shaft_lower_recipe.as_ref().unwrap().scene_source_sha256.clone();
  assert_eq!(super::super::shaft_lift::ShaftOwner::restore_frame(&scene,&frame,1,0).unwrap_err(),"E_SHAFT_FRAME_MISMATCH");
 } #[test] fn lower_service_candidate_uses_exact_union_and_concave_sweep(){
  let r=recipe().expect("qualified candidate data");assert_eq!(r.walls.len(),34);
  assert_eq!(REGION_PARTS.len(),14);assert_eq!(REGION_PARTS.iter().map(|(_,p)|p.len()).sum::<usize>(),30);
  let mut ids=std::collections::BTreeSet::new();for (id,parts) in REGION_PARTS {assert!(ids.insert(*id));
   for p in *parts {let part=Rect{x0:p[0],z0:p[1],x1:p[2],z1:p[3]};
    assert!(r.floors.iter().any(|f|part.x0>=f.x0&&part.x1<=f.x1&&part.z0>=f.z0&&part.z1<=f.z1),"{id}");
    assert!(!r.walls.iter().any(|(_,w)|part.x0<w.x1&&part.x1>w.x0&&part.z0<w.z1&&part.z1>w.z0),"{id}");
   }
  }
  assert_eq!(region_at(30.,4.),Some("pipe_fittings"));assert_eq!(region_at(30.,7.),None);
  let mut scene:crate::scene_runtime::SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_central_shaft.json")).unwrap();
  scene.verified_source_sha256="6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832".into();
  let old=super::super::shaft_lift::ShaftGeometry::from_scene(&scene).unwrap();let candidate=candidate_geometry(&scene).unwrap();
  let bounds=crate::continuous_kcc::Aabb::new(0.,24.,0.,16.).unwrap();
  let old_owner=super::super::shaft_lift::ShaftOwner{geometry:old.clone(),clock:super::super::shaft_lift::ShaftClock::at_upper(0)};
  let provisional=super::super::shaft_lift::ShaftOwner{geometry:candidate.clone(),clock:super::super::shaft_lift::ShaftClock::at_upper(0)};
  let frame=candidate_frame(&provisional,&scene,1).unwrap();assert_eq!(provisional.frame(&scene,1).unwrap(),frame);assert_eq!(candidate_from_frame(&scene,&frame,1,0).unwrap(),provisional);assert_eq!(super::super::shaft_lift::ShaftOwner::restore_frame(&scene,&frame,1,0).unwrap(),provisional);
  assert!(candidate_from_frame(&scene,&frame,2,0).is_err());assert!(candidate_from_frame(&scene,&frame,1,1).is_err());
  let mut bad=frame.clone();bad.catalog_sha256="0".repeat(64);assert!(candidate_from_frame(&scene,&bad,1,0).is_err());
  let mut bad=frame.clone();bad.shaft_lower_recipe.as_mut().unwrap().recipe_version=3;assert!(candidate_from_frame(&scene,&bad,1,0).is_err());
  let mut world=crate::world_v3::WorldStateV3::new("grey_hive",1,crate::world_v3::Vec3::new(10.,0.,4.).unwrap()).unwrap();world.scene_id="gh_central_shaft".into();
  assert!(candidate_from_old(&scene,&old_owner,&world).is_some());
  world.player.position_m=crate::world_v3::Vec3::new(3.5,0.,3.).unwrap();assert!(candidate_from_old(&scene,&old_owner,&world).is_none());
  let new_owner=super::super::shaft_lift::ShaftOwner{geometry:candidate.clone(),clock:super::super::shaft_lift::ShaftClock::at_upper(0)};
  let original=crate::continuous_kcc::StaticKccWorld::new(bounds,vec![]).with_shaft_step(&scene,old_owner.clone(),old_owner).unwrap();
  let expanded=crate::continuous_kcc::StaticKccWorld::new(bounds,vec![]).with_shaft_step(&scene,new_owner.clone(),new_owner).unwrap();
  let lower=crate::world_v3::Vec3::new(27.,-4.,8.).unwrap();let upper=crate::world_v3::Vec3::new(27.,0.,8.).unwrap();
  assert!(!original.can_player_occupy(lower,0.35));assert!(expanded.can_player_occupy(lower,0.35));assert!(!expanded.can_player_occupy(upper,0.35));
  let body=crate::continuous_kcc::KccBody::new(lower);
  let (view,support)=provisional.views(&scene,1,0,0,0,&body,false,false,false).unwrap();
  assert_eq!(view.scene_source_sha256,scene.verified_source_sha256);assert_eq!(support.rider.mode,crate::moving_support::SupportRiderMode::Floor);
  assert!((super::super::shaft_lift::ShaftOwner{geometry:old.clone(),clock:super::super::shaft_lift::ShaftClock::at_upper(0)}).views(&scene,1,0,0,0,&body,false,false,false).is_none());
  assert!(expanded.can_player_occupy(crate::world_v3::Vec3::new(32.,-4.,12.).unwrap(),0.35));
  assert!(!expanded.can_player_occupy(crate::world_v3::Vec3::new(29.5,-4.,10.125).unwrap(),0.35));
  let before=super::super::shaft_lift::ShaftOwner{geometry:candidate.clone(),clock:super::super::shaft_lift::ShaftClock::at_upper(0)};
  let p=crate::world_v3::Vec3::new(23.9,-4.,8.).unwrap();let mut after=before.clone();after.advance(50,(p,p,0.35),&[]).unwrap();
  let moving=crate::continuous_kcc::StaticKccWorld::new(bounds,vec![]).with_shaft_step(&scene,before,after).unwrap();
  let mut body=crate::continuous_kcc::KccBody::new(p);crate::continuous_kcc::step_kcc(&mut body,&moving,(1.,0.),0.05).unwrap();
  assert!(body.position_m.x_m>24.0);assert_eq!(body.position_m.y_m,-4.0);
  let mut reverse=crate::continuous_kcc::KccBody::new(body.position_m);crate::continuous_kcc::step_kcc(&mut reverse,&moving,(-1.,0.),0.05).unwrap();
  assert!(reverse.position_m.x_m<24.0);assert_eq!(reverse.position_m.y_m,-4.0);
  let identity=crate::moving_support::ShaftLowerRecipeIdentity::for_scene(&scene).unwrap();assert!(identity.matches_scene(&scene));
  let encoded=serde_json::to_value(&identity).unwrap();assert_eq!(serde_json::from_value::<crate::moving_support::ShaftLowerRecipeIdentity>(encoded).unwrap(),identity);
  assert!(serde_json::from_value::<crate::moving_support::ShaftLowerRecipeIdentity>(serde_json::json!({"recipeVersion":2,"sceneSourceSha256":identity.scene_source_sha256,"unknown":true})).is_err());
  assert_eq!(old.walls.len(),18);assert_eq!(candidate.walls.len(),55);assert_eq!(candidate.well,old.well);
  assert!(candidate_lower_sweep(&scene,[18.5,6.5],[21.25,6.5],0.35));
  assert!(candidate_lower_sweep(&scene,[21.25,8.],[27.,8.],0.35));
  assert!(!candidate_lower_sweep(&scene,[21.25,6.5],[23.4,6.5],0.35));
  for wall in old.walls.iter().filter(|w|w.id!="lower_east") {assert!(candidate.walls.iter().any(|w|w==wall));}
  for (a,b) in [([21.25,6.5],[21.25,8.]),([27.625,8.],[48.5,8.]),([48.5,8.],[48.5,15.5]),
   ([48.5,15.5],[27.625,15.5]),([27.625,15.5],[27.625,8.]),([32.,8.],[32.,15.5]),
   ([38.,8.],[38.,15.5]),([44.,8.],[44.,15.5]),([30.,8.],[30.,4.]),([38.,8.],[38.,4.]),
   ([46.,8.],[46.,4.]),([30.,15.5],[30.,19.]),([38.,15.5],[38.,19.]),([46.,15.5],[46.,19.])]
   {assert!(candidate_lower_sweep(&scene,a,b,0.35),"{a:?} -> {b:?}");}
  assert!(!candidate_lower_sweep(&scene,[28.,8.],[28.,4.],0.35));
  assert!(!candidate.walls.iter().any(|w|w.id=="lower_east"));
  scene.verified_source_sha256="0".repeat(64);assert!(candidate_geometry(&scene).is_none());
  assert!(covered_sweep(&r.floors,[18.5,6.5],[21.25,6.5],0.35));
  assert!(covered_sweep(&r.floors,[21.25,8.],[27.,8.],0.35));
  assert!(covered_sweep(&r.floors,[27.,8.],[48.,8.],0.35));
  assert!(!covered_sweep(&r.floors,[23.4,5.4],[23.4,6.4],0.35));
  assert!(!covered_sweep(&r.floors,[25.,5.],[25.,6.],0.35));
  assert!(!covered_sweep(&r.floors,[20.,6.],[21.,7.],0.35));
 }
}

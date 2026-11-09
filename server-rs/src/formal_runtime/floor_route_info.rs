//! Source-qualified information geometry only; no physical or save-schema changes.
use std::sync::OnceLock;
use serde_json::Value;
use crate::{scene_runtime::SceneDefinition,world_v3::{ExploredRoom,Vec3}};

const SOURCE:&str=include_str!("../../../apps/godot_playable/data/gh_floor_route_regions.local.json");
static DATA:OnceLock<Option<Value>>=OnceLock::new();
fn data()->Option<&'static Value>{DATA.get_or_init(||{
 let d:Value=serde_json::from_str(SOURCE).ok()?;
 (d["schema"].as_str()==Some("gh-floor-route-info/v1") && d["variants"].as_object()?.len()==13).then_some(d)
}).as_ref()}

// `shaft` is supplied only from the actual owner/deferred qualification.
pub(super) fn variant(scene:&SceneDefinition,open:bool,shaft:Option<bool>)->Option<&'static str>{
 if scene.world_id!="grey_hive"{return None;}
 let key=match (scene.scene_id.as_str(),scene.verified_source_sha256.as_str()) {
  ("gh_entry_maintenance","5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b")=>"entry",
  ("gh_central_shaft","6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832")=>match shaft? {false=>"shaft_deferred",true=>if open{"shaft_active_open"}else{"shaft_active_closed"}},
  ("gh_lockdown","23e5d5d9f37f6f44b1f8b9105aab656424778ef5edf0e775bbc364143fe1b30c")=>if open{"lockdown_open"}else{"lockdown_closed"},
  ("gh_lockdown","718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850")=>if open{"lockdown_legacy_open"}else{"lockdown_legacy_closed"},
  ("gh_gate_b","a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1")=>if open{"gate_b_open"}else{"gate_b_closed"},
  ("gh_sentinel_arena","3e3e3a52dca003ec13c7b3698317f6a32627091eeda833a49896910e521f45bb")=>"sentinel",
  _=>return None,
 };
 let owners=data()?["variants"][key]["owners"].as_array()?;
 let static_owners:Vec<_>=owners.iter().filter(|o|o["role"].as_str()==Some("collision_owner"))
  .filter(|o|!matches!(o["id"].as_str(),Some("well_west"|"well_east"|"well_north"|"upper_landing_gate"))).collect();
 if static_owners.len()!=scene.collision.len(){return None;}
 for owner in static_owners {
  let id=owner["id"].as_str()?;let matches:Vec<_>=scene.collision.iter().filter(|r|r.id==id).collect();
  let rect=owner["rect_x0_z0_x1_z1"].as_array()?;
  if matches.len()!=1 || rect.len()!=4 || matches[0].requires_event.as_deref()!=owner["requiresEvent"].as_str()
    || matches[0].requires_actor_first_kill.is_some(){return None;}
  let v:Option<Vec<f32>>=rect.iter().map(|x|x.as_f64().map(|n|n as f32)).collect();let r=v?;
  if matches[0].polygon!=vec![[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]]{return None;}
 }
 Some(key)
}
// The caller must first prove actual owner/view Floor contact at Y=-4. This
// reuses the same source/owner check as the active Shaft, never a height guess.
pub(super) fn lower_variant(scene:&SceneDefinition,open:bool)->Option<&'static str>{
 variant(scene,false,Some(true))?;
 let key=if open {"shaft_lower_open"} else {"shaft_lower_closed"};
 let row=&data()?["variants"][key];
 if row["scene"].as_str()!=Some("gh_central_shaft") || row["provider"].as_str()!=Some("grey_hive.gh_central_shaft.lower_info_v1")
  || row["floor_y"].as_f64()!=Some(-4.0) || row["parents"].as_array()?.len()!=5 {return None;}
 Some(key)
}
fn parents(key:&str)->Option<&'static Vec<Value>>{data()?["variants"][key]["parents"].as_array()}
pub(super) fn parts(key:&str,id:&str)->Option<Vec<[[f32;2];4]>>{
 let row=parents(key)?.iter().find(|r|r["id"].as_str()==Some(id)&&r["available"].as_bool()==Some(true))?;
 row["parts_xz"].as_array()?.iter().map(|p|{let p=p.as_array()?;if p.len()!=4{return None;}let mut out=[[0.0;2];4];
  for(i,v)in p.iter().enumerate(){let a=v.as_array()?;if a.len()!=2{return None;}out[i]=[a[0].as_f64()? as f32,a[1].as_f64()? as f32];}Some(out)}).collect()
}
pub(super) fn occupied(key:&str,p:Vec3,previous:Option<&str>)->Option<String>{
 let floor=data()?["variants"][key]["floor_y"].as_f64()? as f32;
 if !p.x_m.is_finite()||!p.z_m.is_finite()||p.y_m!=floor{return None;}
 let matches:Vec<String>=parents(key)?.iter().filter_map(|r|{let id=r["id"].as_str()?;
  parts(key,id)?.iter().any(|p0|p.x_m>=p0[0][0]&&p.x_m<=p0[2][0]&&p.z_m>=p0[0][1]&&p.z_m<=p0[2][1]).then(||id.to_owned())}).collect();
 if key.starts_with("shaft_lower_") {
  if let Some(id)=matches.iter().find(|id|parts(key,id).is_some_and(|regions|regions.iter().any(|r|
   p.x_m>r[0][0]&&p.x_m<r[2][0]&&p.z_m>r[0][1]&&p.z_m<r[2][1]))) {return Some(id.clone());}
 }
 previous.filter(|id|matches.iter().any(|m|m==id)).map(str::to_owned).or_else(||matches.into_iter().next())
}
fn memory_key(key:&str)->&str{match key{"shaft_active_closed"=>"shaft_active_open","shaft_lower_closed"=>"shaft_lower_open","lockdown_closed"=>"lockdown_open","lockdown_legacy_closed"=>"lockdown_legacy_open","gate_b_closed"=>"gate_b_open",_=>key}}
pub(super) fn rows(key:&str,id:&str)->Option<Vec<ExploredRoom>>{Some(parts(memory_key(key),id)?.iter().enumerate().map(|(i,p)|ExploredRoom{
 room_id:format!("{id}:part{i}"),outline_m:p.iter().map(|v|Vec3{x_m:v[0],y_m:if key.starts_with("shaft_lower_"){-4.0}else{0.0},z_m:v[1]}).collect()}).collect())}
pub(super) fn stored_regions(key:&str,rooms:&[ExploredRoom])->Vec<String>{let Some(p)=parents(key)else{return Vec::new();};p.iter().filter_map(|r|{
 if r["available"].as_bool()!=Some(true){return None;}let id=r["id"].as_str()?;let expected=rows(key,id)?;
 expected.iter().all(|e|rooms.iter().filter(|r|r.room_id==e.room_id && r.outline_m==e.outline_m).count()==1 && rooms.iter().filter(|r|r.room_id==e.room_id).count()==1).then(||id.to_owned())}).collect()}

#[cfg(test)]
mod tests {
 use super::*;
 fn fixture(raw:&str,sha:&str)->SceneDefinition {
  let mut scene:SceneDefinition=serde_json::from_str(raw).unwrap();
  // Test a supplied registry identity without hashing any runtime artifact.
  scene.verified_source_sha256=sha.into();scene
 }
 #[test]
 fn strict_source_owners_and_complete_rows() {
  let mut scene=fixture(include_str!("../../../content/scenes/compiled/gh_entry_maintenance.json"),"5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b");
  assert_eq!(variant(&scene,false,None),Some("entry"));
  let id="grey_hive.gh_entry_maintenance.info_v1.entry_transit";
  let p=Vec3{x_m:10.0,y_m:0.0,z_m:7.0};assert_eq!(occupied("entry",p,None).as_deref(),Some(id));
  let expected=rows("entry",id).unwrap();assert_eq!(stored_regions("entry",&expected),vec![id.to_owned()]);
  let mut duplicate=expected.clone();duplicate.push(expected[0].clone());assert!(stored_regions("entry",&duplicate).is_empty());
  let mut malformed=expected.clone();malformed[0].outline_m[0].x_m+=1.0;assert!(stored_regions("entry",&malformed).is_empty());
  scene.verified_source_sha256="wrong".into();assert_eq!(variant(&scene,false,None),None);
  scene.verified_source_sha256="5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b".into();scene.collision[0].polygon[0][0]+=1.0;assert_eq!(variant(&scene,false,None),None);
 }
 #[test]
 fn gate_passage_requires_open_and_actual_complete_visit() {
  let scene=fixture(include_str!("../../../content/scenes/compiled/gh_gate_b.json"),"a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1");
  assert_eq!(variant(&scene,false,None),Some("gate_b_closed"));assert_eq!(variant(&scene,true,None),Some("gate_b_open"));
  let id="grey_hive.gh_gate_b.info_v1.seal_passage";let saved=rows("gate_b_open",id).unwrap();
  assert!(stored_regions("gate_b_closed",&saved).is_empty());assert_eq!(stored_regions("gate_b_open",&saved),vec![id.to_owned()]);
  assert!(stored_regions("gate_b_open",&[]).is_empty());
 }
 #[test]
 fn shaft_floor_mode_rejects_surface_lower_air_and_traversal() {
  use crate::moving_support::SupportRiderMode::*;
  assert!(super::super::shaft_upper_floor_flags(Floor,0.0,true,false));
  assert!(!super::super::shaft_upper_floor_flags(Surface,0.0,true,false));
  assert!(!super::super::shaft_upper_floor_flags(Floor,-4.0,true,false));
  assert!(!super::super::shaft_upper_floor_flags(Airborne,0.0,false,false));
  assert!(!super::super::shaft_upper_floor_flags(Traversal,0.0,true,true));
 }
 #[test]
 fn lower_owner_view_and_five_parent_memory_are_layer_bound() {
  use crate::moving_support::{SupportPhase,SupportRiderMode};
  let mut scene=fixture(include_str!("../../../content/scenes/compiled/gh_central_shaft.json"),"6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832");
  let mut owner=super::super::shaft_lift::ShaftOwner::new_at_upper(&scene,0).unwrap();
  let mut pose=owner.clock.pose();pose.phase=SupportPhase::LowerHold;pose.height_m=-4.0;pose.velocity_mps=0.0;pose.phase_elapsed_ms=0;
  owner.clock=super::super::shaft_lift::ShaftClock::restore(0,&pose).unwrap();
  let mut body=crate::continuous_kcc::KccBody::new(Vec3{x_m:10.5,y_m:-4.0,z_m:10.0});body.grounded=true;
  let (_,view)=owner.views(&scene,1,0,0,0,&body,false,false,false).unwrap();
  assert_eq!(view.rider.mode,SupportRiderMode::Floor);
  assert!(super::super::shaft_floor_contact_flags(true,false,view.rider.mode,-4.0,-4.0,true,false));
  assert!(!super::super::shaft_floor_contact_flags(false,true,view.rider.mode,-4.0,-4.0,true,false));
  let mut cabin=body.clone();cabin.position_m=Vec3{x_m:3.5,y_m:-4.0,z_m:3.0};
  assert_eq!(owner.views(&scene,1,0,0,0,&cabin,false,false,false).unwrap().1.rider.mode,SupportRiderMode::Surface);
  assert!(!super::super::shaft_floor_contact_flags(true,false,SupportRiderMode::Surface,-4.0,-4.0,true,false));
  body.grounded=false;assert_eq!(owner.views(&scene,1,0,0,0,&body,false,false,false).unwrap().1.rider.mode,SupportRiderMode::Airborne);
  assert_eq!(owner.views(&scene,1,0,0,0,&body,true,false,false).unwrap().1.rider.mode,SupportRiderMode::Traversal);
  assert_eq!(lower_variant(&scene,true),Some("shaft_lower_open"));
  scene.verified_source_sha256="wrong".into();assert!(lower_variant(&scene,true).is_none());
  let data=parents("shaft_lower_open").unwrap();assert_eq!(data.len(),5);
  let mut all=Vec::new();
  for parent in data {let id=parent["id"].as_str().unwrap();let rows=rows("shaft_lower_open",id).unwrap();assert!(rows.iter().all(|r|r.outline_m.iter().all(|p|p.y_m==-4.0)));all.extend(rows);}
  assert_eq!(all.len(),15);assert_eq!(stored_regions("shaft_lower_open",&all).len(),5);
  let mut duplicate=all.clone();duplicate.push(all[0].clone());assert_eq!(stored_regions("shaft_lower_open",&duplicate).len(),4);
  let mut missing=all.clone();missing.remove(0);assert_eq!(stored_regions("shaft_lower_open",&missing).len(),4);
  let p=Vec3{x_m:3.5,y_m:-4.0,z_m:5.075};assert!(occupied("shaft_lower_open",p,None).is_some());assert!(occupied("shaft_lower_closed",p,None).is_none());
  assert!(occupied("shaft_lower_open",Vec3{y_m:0.0,..p},None).is_none());
  assert!(stored_regions("shaft_lower_open",&[]).is_empty());
 }
}

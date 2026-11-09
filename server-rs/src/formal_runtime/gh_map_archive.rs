//! Qualified Grey Hive history only; never current visibility or movement authority.
use super::*;
use crate::world_persistent_v1::{GhMapKnowledgeArchiveV2,GhMapKnowledgeRecord};
use crate::world_v3::ExploredRoom;
use std::sync::OnceLock;
static CATALOG:OnceLock<Vec<GhMapKnowledgeRecord>>=OnceLock::new();
fn sha(scene:&str,legacy:bool)->Option<&'static str>{Some(match scene {
 "gh_power_room"=>"3d05edeca4a119bc7f2db7aa094041a7ad61380b00015a613fda787025e67854",
 "gh_gate_a"=>"3c62f3c35a2c9f2e935707e7a93c34a8d6f5696ed730b8487c4efe0b6d1ee484",
 "gh_bio_isolation"=>"bf531fd9d49ee875653a8048130dd0c6e1d9b2170c1848ad48c49f2cb35ce0db",
 "gh_deep_decon"=>"a53fd5c62a4540a375b51f7286798c7bab1a5c73aa37e98d3f4d6bad7b24f1c0",
 "gh_beacon"=>"e6434c598904d7fe36f4f48ebc67559e6d484eb535393a7b447bc99cbf86c766",
 "gh_exit"=>"b32ad8d1204e9eaf45512b0e2d5ea22e08f5bd09ec0b5e31980e41336c45ee09",
 "gh_entry_maintenance"=>"4bf7a69a72e714a8de36c683f8b04fa7b1c89bbb9a358b2735dc7d2775915eaf",
 "gh_central_shaft"=>"6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832",
 "gh_lockdown" if legacy=>"718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850",
 "gh_lockdown"=>"23e5d5d9f37f6f44b1f8b9105aab656424778ef5edf0e775bbc364143fe1b30c",
 "gh_gate_b"=>"a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1",
 "gh_sentinel_arena"=>"3e3e3a52dca003ec13c7b3698317f6a32627091eeda833a49896910e521f45bb",_=>return None})}
fn record(scene:&str,family:&str,layer:&str,frame:&str,id:&str,rows:Vec<ExploredRoom>,legacy:bool)->Option<GhMapKnowledgeRecord>{
 if rows.is_empty()||rows.len()>12||rows.iter().any(|r|r.outline_m.len()!=4){return None;}
 Some(GhMapKnowledgeRecord{world_id:"grey_hive".into(),scene_id:scene.into(),scene_source_sha256:sha(scene,legacy)?.into(),provider_family:family.into(),layer_id:layer.into(),frame_id:frame.into(),parent_id:id.into(),rows})
}
fn build()->Vec<GhMapKnowledgeRecord>{
 let mut out=Vec::new();
 if let Ok(v)=serde_json::from_str::<serde_json::Value>(include_str!("../../../apps/godot_playable/data/gh_service_wing_dense.local.json")) {
  if let Some(regions)=v["regions"].as_array(){for p in regions {if let (Some(id),Some(scene))=(p["id"].as_str(),p["sceneId"].as_str()) {
   if let Some(poly)=dense_service_wing::shared_region_polygon(id){let row=ExploredRoom{room_id:id.into(),outline_m:poly.iter().map(|p|Vec3{x_m:p[0],y_m:0.0,z_m:p[1]}).collect()};
    if dense_service_wing::valid_stored_room(&row){if let Some(r)=record(scene,"grey_hive.service_wing.dense_info_v1","upper_y0","gh_upper_shared_v1",id,vec![row],false){out.push(r);}}}
  }}}
 }
 for (scene,family,regions) in [("gh_bio_isolation","grey_hive.gh_bio_isolation.dense_info_v1",bio_info::regions()),("gh_deep_decon","grey_hive.gh_deep_decon.dense_info_v1",decon_info::regions())] {
  if let Some(regions)=regions {for p in regions {if let Some(id)=p["id"].as_str(){let rows=if scene=="gh_bio_isolation"{bio_info::rows(id)}else{decon_info::rows(id)};if let Some(rows)=rows {if let Some(r)=record(scene,family,"upper_y0","gh_upper_shared_v1",id,rows,false){out.push(r);}}}}}
 }
 if let Ok(v)=serde_json::from_str::<serde_json::Value>(include_str!("../../../apps/godot_playable/data/gh_beacon_exit_regions.local.json")) {
  for(scene,key)in [("gh_beacon","beacon"),("gh_exit","exit_open")] {if let Some(regions)=v["variants"][key]["regions"].as_array(){for p in regions {if let Some(id)=p["id"].as_str(){if let Some(rows)=beacon_exit_info::rows(scene,id){if let Some(r)=record(scene,"grey_hive.beacon_exit.info_v1","upper_y0","gh_beacon_exit_pair_v1",id,rows,false){out.push(r);}}}}}}
 }
 if let Ok(v)=serde_json::from_str::<serde_json::Value>(include_str!("../../../apps/godot_playable/data/gh_floor_route_regions.local.json")) {
  for key in ["entry","shaft_deferred","shaft_active_open","shaft_lower_open","lockdown_open","lockdown_legacy_open","gate_b_open","sentinel"] {
   let var=&v["variants"][key];let (Some(scene),Some(family),Some(parents))=(var["scene"].as_str(),var["provider"].as_str(),var["parents"].as_array()) else{continue;};
   let lower=key=="shaft_lower_open";for p in parents {if let Some(id)=p["id"].as_str(){if let Some(rows)=floor_route_info::rows(key,id){if let Some(r)=record(scene,family,if lower{"lower_y_minus4"}else{"upper_y0"},if lower{"gh_shaft_lower_local_v1"}else{"gh_upper_shared_v1"},id,rows,key=="lockdown_legacy_open"){out.push(r);}}}}
  }
 }
 out
}
pub(super) fn catalog()->&'static [GhMapKnowledgeRecord]{CATALOG.get_or_init(build)}
#[cfg(test)]
pub(crate) fn test_record()->GhMapKnowledgeRecord{catalog()[0].clone()}
fn same_owner(a:&GhMapKnowledgeRecord,b:&GhMapKnowledgeRecord)->bool{a.world_id==b.world_id&&a.scene_id==b.scene_id&&a.provider_family==b.provider_family&&a.layer_id==b.layer_id&&a.frame_id==b.frame_id&&a.parent_id==b.parent_id}
fn same_identity(a:&GhMapKnowledgeRecord,b:&GhMapKnowledgeRecord)->bool{same_owner(a,b)&&a.scene_source_sha256==b.scene_source_sha256}
pub(crate) fn valid_recognized_record(r:&GhMapKnowledgeRecord)->bool{catalog().iter().find(|e|same_identity(e,r)).is_none_or(|e|e.rows==r.rows)}
fn complete(expected:&GhMapKnowledgeRecord,rooms:&[ExploredRoom])->bool{expected.rows.iter().all(|e|rooms.iter().filter(|r|r.room_id==e.room_id).count()==1&&rooms.iter().any(|r|r==e))}
fn allowed_import(state:&RuntimeState,r:&GhMapKnowledgeRecord,current_key:Option<&str>)->bool {
 let rooms=&state.world.explored.rooms;let open=completed_world_events(state,"grey_hive");
 match r.scene_id.as_str(){
  "gh_power_room"|"gh_gate_a"=>r.rows.len()==1&&dense_service_wing::valid_stored_room(&r.rows[0])&&complete(r,rooms),
  "gh_bio_isolation"=>bio_info::stored_regions(rooms).contains(&r.parent_id),
  "gh_deep_decon"=>decon_info::stored_regions(rooms).contains(&r.parent_id),
  "gh_beacon"|"gh_exit"=>beacon_exit_info::stored_regions(&r.scene_id,open.contains("hive_extraction"),rooms).contains(&r.parent_id),
  _=>{let key=match r.provider_family.as_str(){
   "grey_hive.gh_entry_maintenance.info_v1"=>"entry",
   "grey_hive.gh_central_shaft.upper_active_info_v1"=>if state.world.shaft_upper_map_observation.as_ref().and_then(|o|o.trusted_family())==Some("shaft_active")||current_key.is_some_and(|k|k.starts_with("shaft_active_")){"shaft_active_open"}else{return false;},
   "grey_hive.gh_central_shaft.upper_deferred_info_v1"=>if current_key==Some("shaft_deferred"){"shaft_deferred"}else{return false;},
   "grey_hive.gh_central_shaft.lower_info_v1"=>"shaft_lower_open",
   "grey_hive.gh_lockdown.dense_info_v1"=>if state.world.lockdown_map_observation.as_ref().and_then(|o|o.trusted_family())==Some("lockdown")||current_key.is_some_and(|k|k=="lockdown_open"||k=="lockdown_closed"){"lockdown_open"}else{return false;},
   "grey_hive.gh_lockdown.legacy_info_v1"=>if state.world.lockdown_map_observation.as_ref().and_then(|o|o.trusted_family())==Some("lockdown_legacy")||current_key.is_some_and(|k|k.starts_with("lockdown_legacy_")){"lockdown_legacy_open"}else{return false;},
   "grey_hive.gh_gate_b.info_v1"=>"gate_b_open","grey_hive.gh_sentinel_arena.info_v1"=>"sentinel",_=>return false};
   floor_route_info::stored_regions(key,rooms).contains(&r.parent_id)
  }
 }
}
fn insert(archive:&mut GhMapKnowledgeArchiveV2,r:GhMapKnowledgeRecord)->Result<(),String>{
 if archive.records.iter().any(|old|same_identity(old,&r)){return Ok(());}
 let stale=archive.records.iter().filter(|old|same_owner(old,&r)).count();if archive.records.len()-stale>=85{return Err("E_GH_MAP_ARCHIVE_CAPACITY".into());}
 archive.records.retain(|old|!same_owner(old,&r));archive.records.push(r);
 archive.records.sort_by(|a,b|(&a.world_id,&a.scene_id,&a.scene_source_sha256,&a.provider_family,&a.layer_id,&a.frame_id,&a.parent_id).cmp(&(&b.world_id,&b.scene_id,&b.scene_source_sha256,&b.provider_family,&b.layer_id,&b.frame_id,&b.parent_id)));Ok(())
}
fn empty()->GhMapKnowledgeArchiveV2{GhMapKnowledgeArchiveV2{schema_version:2,records:vec![],shaft_upper_observation:None,lockdown_observation:None}}
/// One already-qualified successful occupied parent from a committed step.
pub(super) fn observe(state:&mut RuntimeState,scene:&SceneRuntime,key:Option<&str>){
 let Some((id_scene,epoch,id))=state.dense_region_previous.as_ref() else{return;};let d=scene.current_scene();
 if state.paused||state.pending_entry.is_some()||state.pending_exit.is_some()||state.world.world_id!="grey_hive"||d.world_id!="grey_hive"||state.world.scene_id!=d.scene_id||*id_scene!=d.scene_id||*epoch!=scene.world_epoch||scene.world_epoch!=state.world.revision.world_epoch{return;}
 let admitted=if let Some(k)=key {floor_route_info::parts(k,id).is_some()}else{dense_service_wing::admitted(d)||bio_info::admitted(d)||decon_info::admitted(d)||beacon_exit_info::admitted(d)};
 if !admitted{return;}
 let expected=catalog().iter().find(|r|r.scene_id==d.scene_id&&r.scene_source_sha256==d.verified_source_sha256&&r.parent_id==*id
  && (key.is_none() || key.is_some_and(|k| floor_route_info::rows(k,id).as_ref()==Some(&r.rows))) && complete(r,&state.world.explored.rooms));
 let Some(expected)=expected else{return;};let mut archive=state.world_persistent_v1.map_knowledge_archive_v2.clone().unwrap_or_else(empty);
 if insert(&mut archive,expected.clone()).is_err(){return;}
 if key.is_some_and(|k|k.starts_with("shaft_active_")){archive.shaft_upper_observation=state.world.shaft_upper_map_observation.clone();}
 if key.is_some_and(|k|k.starts_with("lockdown_")){archive.lockdown_observation=state.world.lockdown_map_observation.clone();}
 state.world_persistent_v1.map_knowledge_archive_v2=Some(archive);
}
/// A departure candidate imports only complete typed history already accepted by providers.
pub(super) fn import_history(state:&mut RuntimeState,scene:&SceneRuntime)->Result<(),String>{
 if state.world.world_id!="grey_hive"||state.world.explored.world_id!="grey_hive"||scene.current_scene().world_id!="grey_hive"||scene.current_scene().scene_id!=state.world.scene_id||scene.world_epoch!=state.world.revision.world_epoch{return Ok(());}
 let lift=shaft_region_support_view(state,scene.current_scene());let key=floor_route_region_key(state,scene,lift.as_ref());
 let mut archive=state.world_persistent_v1.map_knowledge_archive_v2.clone().unwrap_or_else(empty);let mut added=false;
 for r in catalog(){if allowed_import(state,r,key){insert(&mut archive,r.clone())?;added=true;}}
 if added {
  if state.world.shaft_upper_map_observation.as_ref().and_then(|o|o.trusted_family())==Some("shaft_active"){archive.shaft_upper_observation=state.world.shaft_upper_map_observation.clone();}
  if state.world.lockdown_map_observation.as_ref().and_then(|o|o.trusted_family()).is_some(){archive.lockdown_observation=state.world.lockdown_map_observation.clone();}
  state.world_persistent_v1.map_knowledge_archive_v2=Some(archive);
 }Ok(())
}
/// Read-only history may be displayed while paused and never supplies a physical layer; the committed frame is
/// qualified by the existing scene/provider and, for Shaft, its floor key.
pub(super) fn committed_frame(state:&RuntimeState,d:&crate::scene_runtime::SceneDefinition,key:Option<&str>)->bool{
 if state.world.world_id!="grey_hive"||state.world.explored.world_id!="grey_hive"||state.world.scene_id!=d.scene_id||d.world_id!="grey_hive"
  ||state.pending_entry.is_some()||state.pending_exit.is_some()||state.world.player_hp==0||state.last_owner_error.is_some(){return false;}
 if d.scene_id=="gh_central_shaft" {return key.is_some_and(|k|k.starts_with("shaft_"));}
 if let Some(k)=key {let open=completed_world_events(state,"grey_hive").contains("hive_lockdown");return floor_route_info::variant(d,open,None)==Some(k);}
 dense_service_wing::admitted(d)||bio_info::admitted(d)||decon_info::admitted(d)||beacon_exit_info::admitted(d)
}
/// Read-only union; conflicting active rows are never replaced.
pub(super) fn history_rows(state:&RuntimeState)->Vec<ExploredRoom>{
 let mut rooms=state.world.explored.rooms.clone();if state.world.world_id!="grey_hive"||state.world.explored.world_id!="grey_hive"{return rooms;}
 if let Some(archive)=&state.world_persistent_v1.map_knowledge_archive_v2 {for r in &archive.records {
  if !catalog().iter().any(|e|same_identity(e,r)&&e.rows==r.rows){continue;}
  for row in &r.rows {if !rooms.iter().any(|current|current.room_id==row.room_id){rooms.push(row.clone());}}
 }}rooms
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn gh_archive_catalog_is_bounded_strict_and_stale_opaque(){
  let c=catalog();assert_eq!(c.len(),85);let mut ids=std::collections::BTreeSet::new();
  for r in c {assert!(ids.insert((&r.scene_id,&r.scene_source_sha256,&r.provider_family,&r.parent_id)));assert!(!r.rows.is_empty()&&r.rows.len()<=12);assert!(r.rows.iter().all(|p|p.outline_m.len()==4));assert!(valid_recognized_record(r));}
  let r=c[0].clone();let mut p=crate::world_persistent_v1::WorldPersistentState::default();
  assert!(serde_json::to_value(&p).unwrap().get("mapKnowledgeArchiveV2").is_none());
  let mut null=serde_json::to_value(&p).unwrap();null["mapKnowledgeArchiveV2"]=serde_json::Value::Null;assert!(serde_json::from_value::<crate::world_persistent_v1::WorldPersistentState>(null).is_err());
  let mut a=empty();insert(&mut a,r.clone()).unwrap();p.map_knowledge_archive_v2=Some(a);assert!(p.validate().is_ok());
  let round:crate::world_persistent_v1::WorldPersistentState=serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();assert_eq!(round,p);
  let mut bad=p.clone();bad.map_knowledge_archive_v2.as_mut().unwrap().records[0].rows[0].outline_m[0].x_m+=0.25;assert!(bad.validate().is_err());
  let mut dup=p.clone();dup.map_knowledge_archive_v2.as_mut().unwrap().records.push(r);assert!(dup.validate().is_err());
  let mut stale=p;stale.map_knowledge_archive_v2.as_mut().unwrap().records[0].scene_source_sha256="0".repeat(64);assert!(stale.validate().is_ok());
  let mut state=FormalRuntime::initial_state(1).unwrap();state.world.world_id="grey_hive".into();state.world.explored.world_id="grey_hive".into();state.world_persistent_v1=stale;
  assert!(history_rows(&state).is_empty());
 }
 #[test]
 fn gh_history_projection_requires_committed_provider_and_no_new_visit(){
  let mut state=FormalRuntime::initial_state(1).unwrap();
  state.world=WorldStateV3::new("grey_hive",1,Vec3::new(8.,0.,10.).unwrap()).unwrap();state.world.scene_id="gh_bio_isolation".into();
  state.paused=false;state.pending_entry=None;state.pending_exit=None;
  let expected=catalog().iter().find(|r|r.scene_id=="gh_bio_isolation").unwrap().clone();let mut a=empty();insert(&mut a,expected.clone()).unwrap();state.world_persistent_v1.map_knowledge_archive_v2=Some(a);
  let mut d:crate::scene_runtime::SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_bio_isolation.json")).unwrap();d.verified_source_sha256=expected.scene_source_sha256.clone();assert!(bio_info::admitted(&d));
  let rev=state.world.revision;crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Grant{capability_id:crate::capability_v1::CAP_LOCAL_MAP.into()},rev).unwrap();
  crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Select{capability_ids:vec![crate::capability_v1::CAP_LOCAL_MAP.into()]},rev).unwrap();
  let build=state.progression_v6.clone();state.install_build(build).unwrap();
  let mut out=crate::world_v3::CapabilityProjection::new(vec![],state.world.explored.clone(),vec![],crate::world_v3::RearViewAuthorization::denied());
  live_rules::project_information(&state,Some(&d),None,None,&mut out);
  assert_eq!(out.map_topology_authorized,Some(true));assert_eq!(out.explored_map.rooms.len(),expected.rows.len());
  assert_eq!(out.map_current_room_ids,Some(vec![]));assert_eq!(out.map_explored_room_ids.as_ref().unwrap().len(),expected.rows.len());assert!(state.world.explored.rooms.is_empty());
  state.paused=true;let before=state.world_persistent_v1.clone();live_rules::project_information(&state,Some(&d),None,None,&mut out);assert_eq!(out.map_topology_authorized,Some(true));assert_eq!(out.explored_map.rooms.len(),expected.rows.len());assert_eq!(state.world_persistent_v1,before);
  state.paused=false;state.world.world_id="return_station".into();live_rules::project_information(&state,Some(&d),None,None,&mut out);assert_eq!(out.map_topology_authorized,Some(false));
 }
 #[test]
 fn gh_archive_capacity_supersedes_only_qualified_same_owner(){
  let mut a=empty();for r in catalog(){insert(&mut a,r.clone()).unwrap();}assert_eq!(a.records.len(),85);
  for r in &mut a.records {r.scene_source_sha256="0".repeat(64);}assert!(a.validate());
  let chosen=catalog()[0].clone();insert(&mut a,chosen.clone()).unwrap();assert_eq!(a.records.len(),85);
  assert_eq!(a.records.iter().filter(|r|r.scene_source_sha256!="0".repeat(64)).count(),1);assert!(a.records.contains(&chosen));
  let mut unknown=empty();for i in 0..85 {let mut r=chosen.clone();r.parent_id=format!("future:{i}");for (n,row) in r.rows.iter_mut().enumerate(){row.room_id=format!("future:{i}:part{n}");}r.scene_source_sha256="0".repeat(64);unknown.records.push(r);}
  assert!(unknown.validate());let before=unknown.clone();assert_eq!(insert(&mut unknown,chosen).unwrap_err(),"E_GH_MAP_ARCHIVE_CAPACITY");assert_eq!(unknown,before);
 }
 #[test]
 fn gh_archive_observed_variants_are_typed_and_null_is_invalid(){
  let mut a=empty();let mut unknown=crate::world_v3::ObservedMapVariant::from_floor_route_key("shaft_active_open").unwrap();unknown.scene_source_sha256="0".repeat(64);
  a.shaft_upper_observation=Some(unknown.clone());assert!(a.validate());assert!(unknown.trusted_family().is_none());
  let round:GhMapKnowledgeArchiveV2=serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();assert_eq!(round,a);
  let mut value=serde_json::to_value(&a).unwrap();value["shaftUpperObservation"]=serde_json::Value::Null;assert!(serde_json::from_value::<GhMapKnowledgeArchiveV2>(value).is_err());
 }
 #[test]
 fn gh_closed_exit_history_does_not_open_passage(){
  let mut state=FormalRuntime::initial_state(1).unwrap();state.world=WorldStateV3::new("grey_hive",1,Vec3::new(5.,0.,8.).unwrap()).unwrap();
  state.world.scene_id="gh_exit".into();state.paused=false;state.pending_entry=None;state.pending_exit=None;
  let passage=catalog().iter().find(|r|r.parent_id=="grey_hive.beacon_exit.info_v1.gh_exit.seal_passage").unwrap().clone();
  let mut a=empty();insert(&mut a,passage.clone()).unwrap();state.world_persistent_v1.map_knowledge_archive_v2=Some(a);
  let mut d:crate::scene_runtime::SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_exit.json")).unwrap();d.verified_source_sha256=passage.scene_source_sha256.clone();assert!(beacon_exit_info::admitted(&d));
  let rev=state.world.revision;crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Grant{capability_id:crate::capability_v1::CAP_LOCAL_MAP.into()},rev).unwrap();
  crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Select{capability_ids:vec![crate::capability_v1::CAP_LOCAL_MAP.into()]},rev).unwrap();let build=state.progression_v6.clone();state.install_build(build).unwrap();
  let mut out=crate::world_v3::CapabilityProjection::new(vec![],state.world.explored.clone(),vec![],crate::world_v3::RearViewAuthorization::denied());
  live_rules::project_information(&state,Some(&d),None,None,&mut out);
  assert!(out.explored_map.rooms.iter().all(|r|!r.room_id.starts_with(&passage.parent_id)));
  assert_eq!(state.world_persistent_v1.map_knowledge_archive_v2.as_ref().unwrap().records.len(),1);
 }
 #[test]
 fn gh_lower_history_projection_never_mixes_upper(){
  let mut state=FormalRuntime::initial_state(1).unwrap();state.world=WorldStateV3::new("grey_hive",1,Vec3::new(10.5,-4.,10.).unwrap()).unwrap();
  state.world.scene_id="gh_central_shaft".into();state.paused=false;state.pending_entry=None;state.pending_exit=None;
  let lower=catalog().iter().find(|r|r.provider_family=="grey_hive.gh_central_shaft.lower_info_v1").unwrap().clone();let upper=catalog().iter().find(|r|r.scene_id=="gh_bio_isolation").unwrap().clone();
  let mut a=empty();insert(&mut a,lower.clone()).unwrap();insert(&mut a,upper).unwrap();state.world_persistent_v1.map_knowledge_archive_v2=Some(a);
  let mut d:crate::scene_runtime::SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_central_shaft.json")).unwrap();d.verified_source_sha256=lower.scene_source_sha256.clone();
  let rev=state.world.revision;crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Grant{capability_id:crate::capability_v1::CAP_LOCAL_MAP.into()},rev).unwrap();
  crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Select{capability_ids:vec![crate::capability_v1::CAP_LOCAL_MAP.into()]},rev).unwrap();let build=state.progression_v6.clone();state.install_build(build).unwrap();
  let mut out=crate::world_v3::CapabilityProjection::new(vec![],state.world.explored.clone(),vec![],crate::world_v3::RearViewAuthorization::denied());
  live_rules::project_information(&state,Some(&d),None,Some("shaft_lower_open"),&mut out);
  assert!(!out.explored_map.rooms.is_empty());assert!(out.explored_map.rooms.iter().all(|r|r.room_id.starts_with(&lower.parent_id)&&r.outline_m.iter().all(|p|p.y_m == -4.0)));
  live_rules::project_information(&state,Some(&d),None,None,&mut out);assert_eq!(out.map_topology_authorized,Some(false));
 }
 #[test]
 fn gh_history_import_requires_complete_rows_and_real_variant_tag(){
  let mut state=FormalRuntime::initial_state(1).unwrap();state.world=WorldStateV3::new("grey_hive",1,Vec3::new(2.,0.,2.).unwrap()).unwrap();
  let bio=catalog().iter().find(|r|r.scene_id=="gh_bio_isolation" && r.rows.len()>1).unwrap();
  state.world.explored.rooms=bio.rows[..bio.rows.len()-1].to_vec();assert!(!allowed_import(&state,bio,None));
  state.world.explored.rooms=bio.rows.clone();assert!(allowed_import(&state,bio,None));state.world.explored.rooms.push(bio.rows[0].clone());assert!(!allowed_import(&state,bio,None));
  let upper=catalog().iter().find(|r|r.provider_family=="grey_hive.gh_central_shaft.upper_active_info_v1").unwrap();state.world.explored.rooms=upper.rows.clone();
  assert!(!allowed_import(&state,upper,None));state.world.shaft_upper_map_observation=Some(crate::world_v3::ObservedMapVariant::from_floor_route_key("shaft_active_open").unwrap());assert!(allowed_import(&state,upper,None));
  state.world.shaft_upper_map_observation.as_mut().unwrap().scene_source_sha256="0".repeat(64);assert!(!allowed_import(&state,upper,None));
  let deferred=catalog().iter().find(|r|r.provider_family=="grey_hive.gh_central_shaft.upper_deferred_info_v1").unwrap();state.world.explored.rooms=deferred.rows.clone();
  assert!(!allowed_import(&state,deferred,None));assert!(allowed_import(&state,deferred,Some("shaft_deferred")));
 }
}

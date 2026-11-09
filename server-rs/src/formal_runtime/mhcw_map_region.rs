//! Exact MH/CW information regions. No collision, Save schema, or mechanics.
use std::sync::OnceLock;
use serde_json::Value;
use crate::{scene_runtime::SceneDefinition,world_v3::{ExploredRoom,Vec3}};
const SOURCE:&str=include_str!("mhcw_map_regions.json");
static DATA:OnceLock<Option<Value>>=OnceLock::new();
fn data()->Option<&'static Value>{DATA.get_or_init(||{
    let value:Value=serde_json::from_str(SOURCE).ok()?;let groups=value.as_array()?;
    if groups.len()!=19 || groups.iter().map(|g|g["parents"].as_array().map_or(0,Vec::len)).sum::<usize>()!=38{return None;}
    let mut ids=std::collections::BTreeSet::new();
    for g in groups {if !ids.insert(g["provider_id"].as_str()?) {return None;}
        for parent in g["parents"].as_array()? {if !ids.insert(parent["parent_id"].as_str()?) || parent["parts"].as_array()?.len()!=1 {return None;}
            if !ids.insert(parent["parts"][0]["row_id"].as_str()?) {return None;}}
    } Some(value)
}).as_ref()}
fn points(part:&Value)->Option<Vec<[f32;2]>>{
    let points=part["polygon_xz"].as_array()?;if points.len()!=4{return None;}
    points.iter().map(|p|{let a=p.as_array()?;if a.len()!=2{return None;}let x=a[0].as_f64()? as f32;let z=a[1].as_f64()? as f32;
        (x.is_finite()&&z.is_finite()).then_some([x,z])}).collect()
}
pub(super) fn group(scene:&SceneDefinition,layer:&str)->Option<&'static Value>{
    let g=data()?.as_array()?.iter().find(|g|g["world_id"].as_str()==Some(scene.world_id.as_str())
        && g["scene_id"].as_str()==Some(scene.scene_id.as_str()) && g["layer_id"].as_str()==Some(layer)
        && g["selected_sha_declared"].as_str()==Some(scene.verified_source_sha256.as_str()))?;
    if scene.bounds_m.x!=0.0 || scene.bounds_m.z!=0.0 || scene.bounds_m.width!=24.0 || scene.bounds_m.depth!=16.0{return None;}
    for parent in g["parents"].as_array()? {
        let polygon=points(&parent["parts"][0])?;
        if scene.world_id=="mist_harbor" {
            let id=parent["source_region_id"].as_str()?;
            if scene.exploration_regions.iter().filter(|r|r.id==id && r.polygon==polygon).count()!=1{return None;}
        } else if layer=="gear_static_decks_y2" {
            let id=parent["source_region_id"].as_str()?;
            if scene.standing_decks.iter().filter(|d|d.id==id && d.height_m==2.0 && d.polygon==polygon).count()!=1{return None;}
        } else if layer=="ground_y0" && polygon!=vec![[0.0,0.0],[24.0,0.0],[24.0,16.0],[0.0,16.0]] {return None;}
    }
    Some(g)
}
pub(super) fn occupied(scene:&SceneDefinition,layer:&str,p:Vec3,previous:Option<&str>,support:Option<&str>)->Option<String>{
    let g=group(scene,layer)?;if !p.x_m.is_finite() || !p.y_m.is_finite() || !p.z_m.is_finite() || p.y_m!=g["floor_y"].as_f64()? as f32{return None;}
    let mut matches=Vec::new();for parent in g["parents"].as_array()? {
        if layer=="gear_static_decks_y2" && parent["source_region_id"].as_str()!=support {continue;}
        let polygon=points(&parent["parts"][0])?;
        if super::point_in_or_on_polygon(p.x_m,p.z_m,&polygon){matches.push(parent["parent_id"].as_str()?.to_owned());}
    }
    previous.filter(|p|matches.iter().any(|m|m==p)).map(str::to_owned).or_else(||matches.into_iter().next())
}
pub(super) fn rows(g:&Value,id:&str)->Option<Vec<ExploredRoom>>{
    let parent=g["parents"].as_array()?.iter().find(|v|v["parent_id"].as_str()==Some(id))?;
    parent["parts"].as_array()?.iter().map(|part|{
        let outline=part["outline_m"].as_array()?.iter().map(|p|{let a=p.as_array()?;if a.len()!=3{return None;}Vec3::new(a[0].as_f64()? as f32,a[1].as_f64()? as f32,a[2].as_f64()? as f32).ok()}).collect::<Option<Vec<_>>>()?;
        Some(ExploredRoom{room_id:part["row_id"].as_str()?.into(),outline_m:outline})
    }).collect()
}
pub(super) fn stored(g:&Value,rooms:&[ExploredRoom])->Vec<String>{
    let Some(parents)=g["parents"].as_array() else{return Vec::new();};parents.iter().filter_map(|p|{
        let id=p["parent_id"].as_str()?;let expected=rows(g,id)?;
        expected.iter().all(|e|rooms.iter().filter(|r|r.room_id==e.room_id).count()==1 && rooms.iter().any(|r|r==e)).then(||id.to_owned())
    }).collect()
}
pub(super) fn groups()->Option<&'static [Value]>{Some(data()?.as_array()?.as_slice())}
fn record(g:&Value,id:&str)->Option<crate::world_persistent_v1::MapKnowledgeRecord>{
 Some(crate::world_persistent_v1::MapKnowledgeRecord{
  world_id:g["world_id"].as_str()?.into(),scene_id:g["scene_id"].as_str()?.into(),scene_source_sha256:g["selected_sha_declared"].as_str()?.into(),
  provider_family:g["provider_id"].as_str()?.into(),layer_id:g["layer_id"].as_str()?.into(),frame_id:g["frame_id"].as_str()?.into(),
  parent_id:id.into(),rows:rows(g,id)?,
 })
}
fn same_identity(a:&crate::world_persistent_v1::MapKnowledgeRecord,b:&crate::world_persistent_v1::MapKnowledgeRecord)->bool{
 a.world_id==b.world_id && a.scene_id==b.scene_id && a.scene_source_sha256==b.scene_source_sha256 && a.provider_family==b.provider_family
  && a.layer_id==b.layer_id && a.frame_id==b.frame_id && a.parent_id==b.parent_id
}
fn same_owner(a:&crate::world_persistent_v1::MapKnowledgeRecord,b:&crate::world_persistent_v1::MapKnowledgeRecord)->bool{
 a.world_id==b.world_id && a.scene_id==b.scene_id && a.provider_family==b.provider_family
  && a.layer_id==b.layer_id && a.frame_id==b.frame_id && a.parent_id==b.parent_id
}
/// Known identity with altered geometry is malformed; unknown identities remain opaque.
pub(crate) fn valid_recognized_record(r:&crate::world_persistent_v1::MapKnowledgeRecord)->bool{
 let Some(all)=groups() else{return false;};
 for g in all {if let Some(expected)=record(g,&r.parent_id){if same_identity(&expected,r){return expected.rows==r.rows;}}}
 true
}
pub(super) fn archive_parent(persistent:&mut crate::world_persistent_v1::WorldPersistentState,g:&Value,id:&str)->Result<(),String>{
 let r=record(g,id).ok_or("E_MAP_ARCHIVE_SOURCE")?;
 let archive=persistent.map_knowledge_archive_v1.get_or_insert_with(||crate::world_persistent_v1::MapKnowledgeArchiveV1{schema_version:1,records:vec![]});
 if archive.records.iter().any(|old|same_identity(old,&r)){return Ok(());}
 // Only a newly qualified actual observation/import supersedes stale SHA
 // records for this exact owner; other opaque records remain untouched.
 let stale=archive.records.iter().filter(|old|same_owner(old,&r)).count();
 if archive.records.len()-stale>=38 {return Err("E_MAP_ARCHIVE_CAPACITY".into());}
 archive.records.retain(|old|!same_owner(old,&r));
 archive.records.push(r);archive.records.sort_by(|a,b|(&a.world_id,&a.scene_id,&a.scene_source_sha256,&a.provider_family,&a.layer_id,&a.frame_id,&a.parent_id)
  .cmp(&(&b.world_id,&b.scene_id,&b.scene_source_sha256,&b.provider_family,&b.layer_id,&b.frame_id,&b.parent_id)));
 Ok(())
}
/// Import only rows already accepted as complete by the existing strict provider.
pub(super) fn archive_valid_history(state:&mut super::RuntimeState)->Result<(),String>{
 if state.world.explored.world_id!=state.world.world_id{return Ok(());}
 let mut pending=Vec::new();for g in groups().ok_or("E_MAP_ARCHIVE_SOURCE")? {
  if g["world_id"].as_str()!=Some(state.world.world_id.as_str()){continue;}
  for id in stored(g,&state.world.explored.rooms){pending.push((g,id));}
 }
 let mut candidate=state.world_persistent_v1.clone();for(g,id)in pending {archive_parent(&mut candidate,g,&id)?;}
 state.world_persistent_v1=candidate;Ok(())
}
/// Add recognized archive rows only if no active conflicting row has that ID.
pub(super) fn history_rows(state:&super::RuntimeState,g:&Value)->Vec<ExploredRoom>{
 let mut combined=state.world.explored.rooms.clone();
 if let Some(archive)=&state.world_persistent_v1.map_knowledge_archive_v1 {
  for saved in &archive.records {let Some(expected)=record(g,&saved.parent_id) else{continue;};
   if !same_identity(saved,&expected)||saved.rows!=expected.rows {continue;}
   for row in &saved.rows {if !combined.iter().any(|r|r.room_id==row.room_id){combined.push(row.clone());}}
  }
 } combined
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_validates_current_records_and_keeps_unknown_opaque() {
        let group=groups().unwrap().iter().find(|g|g["scene_id"]=="mh_fog_pier").unwrap();
        let id=group["parents"][0]["parent_id"].as_str().unwrap();
        let mut persistent=crate::world_persistent_v1::WorldPersistentState::default();
        let absent=serde_json::to_value(&persistent).unwrap();assert!(absent.get("mapKnowledgeArchiveV1").is_none());
        let mut null=absent.clone();null["mapKnowledgeArchiveV1"]=serde_json::Value::Null;
        assert!(serde_json::from_value::<crate::world_persistent_v1::WorldPersistentState>(null).is_err());
        archive_parent(&mut persistent,group,id).unwrap();archive_parent(&mut persistent,group,id).unwrap();
        assert_eq!(persistent.map_knowledge_archive_v1.as_ref().unwrap().records.len(),1);assert!(persistent.validate().is_ok());
        let round:crate::world_persistent_v1::WorldPersistentState=serde_json::from_value(serde_json::to_value(&persistent).unwrap()).unwrap();assert_eq!(round,persistent);
        let mut stale=persistent.clone();stale.map_knowledge_archive_v1.as_mut().unwrap().records[0].scene_source_sha256="0".repeat(64);assert!(stale.validate().is_ok());
        let mut invalid=persistent.clone();invalid.map_knowledge_archive_v1.as_mut().unwrap().records[0].rows[0].outline_m[0].x_m+=0.125;assert!(invalid.validate().is_err());
        let mut duplicate=persistent.clone();let rec=duplicate.map_knowledge_archive_v1.as_ref().unwrap().records[0].clone();duplicate.map_knowledge_archive_v1.as_mut().unwrap().records.push(rec);assert!(duplicate.validate().is_err());
        let mut state=super::super::FormalRuntime::initial_state(1).unwrap();state.world_persistent_v1=stale;
        assert!(stored(group,&history_rows(&state,group)).is_empty());state.world_persistent_v1=persistent;
        assert_eq!(stored(group,&history_rows(&state,group)),vec![id.to_owned()]);
    }
    #[test]
    fn departure_imports_only_complete_typed_history() {
        let groups=groups().unwrap();let a=groups.iter().find(|g|g["scene_id"]=="mh_fog_pier").unwrap();
        let b=groups.iter().find(|g|g["scene_id"]=="mh_tidal_warehouse").unwrap();
        let ai=a["parents"][0]["parent_id"].as_str().unwrap();let bi=b["parents"][0]["parent_id"].as_str().unwrap();
        let mut state=super::super::FormalRuntime::initial_state(1).unwrap();
        state.world=crate::world_v3::WorldStateV3::new("mist_harbor",1,Vec3::new(1.,0.,1.).unwrap()).unwrap();
        state.world.explored.rooms.extend(rows(a,ai).unwrap());let mut bad=rows(b,bi).unwrap();bad[0].outline_m[0].x_m+=0.25;state.world.explored.rooms.extend(bad);
        state.world_persistent_v1.mist_harbor.explored_region_ids.push("mh_tw_west_loading".into());
        let original=state.world.explored.rooms.clone();archive_valid_history(&mut state).unwrap();
        let archive=state.world_persistent_v1.map_knowledge_archive_v1.as_ref().unwrap();assert_eq!(archive.records.len(),1);assert_eq!(archive.records[0].parent_id,ai);
        assert_eq!(state.world.explored.rooms,original);archive_valid_history(&mut state).unwrap();assert_eq!(state.world_persistent_v1.map_knowledge_archive_v1.as_ref().unwrap().records.len(),1);
    }
    #[test]
    fn qualified_observation_supersedes_only_same_owner_stale_record_at_capacity() {
        let mut persistent=crate::world_persistent_v1::WorldPersistentState::default();
        for g in groups().unwrap(){for p in g["parents"].as_array().unwrap(){archive_parent(&mut persistent,g,p["parent_id"].as_str().unwrap()).unwrap();}}
        let archive=persistent.map_knowledge_archive_v1.as_mut().unwrap();assert_eq!(archive.records.len(),38);
        for r in &mut archive.records {r.scene_source_sha256="0".repeat(64);}
        assert!(persistent.validate().is_ok());
        let g=groups().unwrap().iter().find(|g|g["scene_id"]=="mh_fog_pier").unwrap();let id=g["parents"][0]["parent_id"].as_str().unwrap();
        let mut state=super::super::FormalRuntime::initial_state(1).unwrap();state.world_persistent_v1=persistent;
        assert!(stored(g,&history_rows(&state,g)).is_empty());
        archive_parent(&mut state.world_persistent_v1,g,id).unwrap();let archive=state.world_persistent_v1.map_knowledge_archive_v1.as_ref().unwrap();
        assert_eq!(archive.records.len(),38);assert_eq!(archive.records.iter().filter(|r|r.scene_source_sha256=="0".repeat(64)).count(),37);
        assert_eq!(stored(g,&history_rows(&state,g)),vec![id.to_owned()]);assert!(state.world_persistent_v1.validate().is_ok());
    }
    #[test]
    fn full_opaque_archive_blocks_departure_import_without_losing_rows() {
        let group=groups().unwrap().iter().find(|g|g["scene_id"]=="mh_fog_pier").unwrap();
        let id=group["parents"][0]["parent_id"].as_str().unwrap();let mut state=super::super::FormalRuntime::initial_state(1).unwrap();
        state.world=crate::world_v3::WorldStateV3::new("mist_harbor",1,Vec3::new(1.,0.,1.).unwrap()).unwrap();
        state.world.explored.rooms=rows(group,id).unwrap();let mut opaque=Vec::new();
        for i in 0..38 {let mut item=record(group,id).unwrap();item.parent_id=format!("future:{i}");item.rows[0].room_id=format!("future:{i}:part0");item.scene_source_sha256="0".repeat(64);opaque.push(item);}
        state.world_persistent_v1.map_knowledge_archive_v1=Some(crate::world_persistent_v1::MapKnowledgeArchiveV1{schema_version:1,records:opaque});
        assert!(state.world_persistent_v1.validate().is_ok());let before=state.world_persistent_v1.clone();let rooms=state.world.explored.rooms.clone();
        assert_eq!(archive_valid_history(&mut state).unwrap_err(),"E_MAP_ARCHIVE_CAPACITY");
        assert_eq!(state.world_persistent_v1,before);assert_eq!(state.world.explored.rooms,rooms);
    }
    #[test]
    fn exact_eighteen_scenes_and_complete_parent_rows() {
        let all=groups().unwrap();let mut count=0;
        for g in all {
            let name=g["scene_id"].as_str().unwrap();
            let raw=match name {
            "mh_fog_pier"=>include_str!("../../../content/scenes/compiled/mh_fog_pier.json"),
            "mh_tidal_warehouse"=>include_str!("../../../content/scenes/compiled/mh_tidal_warehouse.json"),
            "mh_signal_yard"=>include_str!("../../../content/scenes/compiled/mh_signal_yard.json"),
            "mh_drowned_quay"=>include_str!("../../../content/scenes/compiled/mh_drowned_quay.json"),
            "mh_breakwater"=>include_str!("../../../content/scenes/compiled/mh_breakwater.json"),
            "mh_pump_station"=>include_str!("../../../content/scenes/compiled/mh_pump_station.json"),
            "mh_resonance_tower"=>include_str!("../../../content/scenes/compiled/mh_resonance_tower.json"),
            "mh_warden_arena"=>include_str!("../../../content/scenes/compiled/mh_warden_arena.json"),
            "mh_extraction"=>include_str!("../../../content/scenes/compiled/mh_extraction.json"),
            "cw_entry_foundry"=>include_str!("../../../content/scenes/compiled/cw_entry_foundry.json"),
            "cw_pressure_hall"=>include_str!("../../../content/scenes/compiled/cw_pressure_hall.json"),
            "cw_conveyor_bridge"=>include_str!("../../../content/scenes/compiled/cw_conveyor_bridge.json"),
            "cw_boiler_chamber"=>include_str!("../../../content/scenes/compiled/cw_boiler_chamber.json"),
            "cw_gear_shaft"=>include_str!("../../../content/scenes/compiled/cw_gear_shaft.json"),
            "cw_furnace_heart"=>include_str!("../../../content/scenes/compiled/cw_furnace_heart.json"),
            "cw_forged_guard_arena"=>include_str!("../../../content/scenes/compiled/cw_forged_guard_arena.json"),
            "cw_regulator_core"=>include_str!("../../../content/scenes/compiled/cw_regulator_core.json"),
            "cw_shutdown_exit"=>include_str!("../../../content/scenes/compiled/cw_shutdown_exit.json"),
                _=>panic!("unexpected scene"),
            };
            let mut scene:SceneDefinition=serde_json::from_str(raw).unwrap();
            scene.verified_source_sha256=g["selected_sha_declared"].as_str().unwrap().into();
            let layer=g["layer_id"].as_str().unwrap();let selected=group(&scene,layer).expect("exact provider");
            for p in selected["parents"].as_array().unwrap() {
                let id=p["parent_id"].as_str().unwrap();let expected=rows(selected,id).unwrap();
                assert_eq!(stored(selected,&expected),vec![id.to_owned()]);
                let mut duplicate=expected.clone();duplicate.push(expected[0].clone());assert!(stored(selected,&duplicate).is_empty());
                let mut malformed=expected.clone();malformed[0].outline_m[0].x_m+=0.125;assert!(stored(selected,&malformed).is_empty());
                count+=1;
            }
            scene.verified_source_sha256="0".repeat(64);assert!(group(&scene,layer).is_none());
        }
        assert_eq!(count,38);
    }
}

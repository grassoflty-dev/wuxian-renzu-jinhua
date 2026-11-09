//! View-only known seam junctions. No visit, progression or movement effects.
use super::*;
use crate::player_rules::{KnowledgeChannel,KnowledgeLevel};
use crate::world_v3::{CapabilityProjection,KnownMapJunction};

fn endpoint(scene:&str,east:bool)->Option<&'static str>{
 Some(match (scene,east) {
  ("mh_fog_pier",true)=>"mh_fp_warehouse_approach",("mh_tidal_warehouse",false)=>"mh_tw_west_loading",
  ("mh_tidal_warehouse",true)=>"mh_tw_beacon_bay",("mh_signal_yard",false)=>"mh_sy_west_yard",
  ("mh_signal_yard",true)=>"mh_sy_quay_approach",("mh_drowned_quay",false)=>"mh_dq_west_quay",
  ("mh_drowned_quay",true)=>"mh_dq_east_quay",("mh_breakwater",false)=>"mh_bw_west_seawall",
  ("mh_breakwater",true)=>"mh_bw_east_beacon",("mh_pump_station",false)=>"mh_ps_intake",
  ("mh_pump_station",true)=>"mh_ps_east_channel",("mh_resonance_tower",false)=>"mh_rt_entry",
  ("mh_resonance_tower",true)=>"mh_rt_signal_section",("mh_warden_arena",false)=>"mh_wa_entry",
  ("mh_warden_arena",true)=>"mh_wa_exit",("mh_extraction",false)=>"mh_ex_arrival",
  _=>return None,
 })
}
fn parent<'a>(group:&'a serde_json::Value,scene:&str,east:bool)->Option<&'a serde_json::Value>{
 let rows=group["parents"].as_array()?;
 if group["world_id"].as_str()==Some("mist_harbor") {
  let key=endpoint(scene,east)?;let mut m=rows.iter().filter(|r|r["source_region_id"].as_str()==Some(key));let one=m.next()?;if m.next().is_some(){None}else{Some(one)}
 } else if group["world_id"].as_str()==Some("clockworks") && rows.len()==1 && rows[0]["source_region_id"].is_null() {Some(&rows[0])} else {None}
}
fn covered(parent:&serde_json::Value,portal:&crate::scene_runtime::TransitionDefinition)->bool {
 let Some(poly)=parent["parts"][0]["polygon_xz"].as_array() else{return false;};if poly.len()!=4 || portal.polygon.len()!=4{return false;}
 let values:Option<Vec<(f64,f64)>>=poly.iter().map(|v|Some((v[0].as_f64()?,v[1].as_f64()?))).collect();let Some(v)=values else{return false;};
 let (minx,maxx,minz,maxz)=v.iter().fold((f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY),|a,p|(a.0.min(p.0),a.1.max(p.0),a.2.min(p.1),a.3.max(p.1)));
 let (px0,px1,pz0,pz1)=portal.polygon.iter().fold((f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY),|a,p|(a.0.min(f64::from(p[0])),a.1.max(f64::from(p[0])),a.2.min(f64::from(p[1])),a.3.max(f64::from(p[1]))));
 minx<px1 && maxx>px0 && minz<=pz0 && maxz>=pz1
}
fn portal<'a>(from:&'a crate::scene_runtime::SceneDefinition,to:&str)->Option<&'a crate::scene_runtime::TransitionDefinition>{
 let mut matches=from.transitions.iter().filter(|t|t.to_scene_id==to);let first=matches.next()?;if matches.next().is_some(){None}else{Some(first)}
}
fn pair(state:&RuntimeState,source:&SceneRuntime,target:&SceneRuntime,east:bool,frame:&str,layer:&str,displayed:&std::collections::BTreeSet<String>)->Option<(KnownMapJunction,String,String)> {
 let a=source.current_scene();let b=target.current_scene();
 let mut all=BTreeSet::new();for id in ["mist_beacon_east","mist_signal","clockworks_valves","clockworks_core"]{all.insert(id.to_owned());}
 let verify=|from:&SceneRuntime,to:&str|if a.world_id=="mist_harbor"{from.mh_shore_connection_verified(to,&all,true)}else{from.cw_early_connection_verified(to,&all,true)};
 if !verify(source,&b.scene_id)||!verify(target,&a.scene_id){return None;}
 let ga=mhcw_map_region::group(a,layer)?;let gb=mhcw_map_region::group(b,layer)?;
 if ga["frame_id"].as_str()!=Some(frame)||gb["frame_id"].as_str()!=Some(frame)||ga["layer_id"].as_str()!=Some(layer)||gb["layer_id"].as_str()!=Some(layer){return None;}
 let pa=parent(ga,&a.scene_id,east)?;let pb=parent(gb,&b.scene_id,!east)?;
 let ia=pa["parent_id"].as_str()?;let ib=pb["parent_id"].as_str()?;
 if !mhcw_map_region::stored(ga,&mhcw_map_region::history_rows(state,ga)).iter().any(|id|id==ia) || !mhcw_map_region::stored(gb,&mhcw_map_region::history_rows(state,gb)).iter().any(|id|id==ib){return None;}
 let ra=mhcw_map_region::rows(ga,ia)?;let rb=mhcw_map_region::rows(gb,ib)?;if ra.len()!=1||rb.len()!=1||!displayed.contains(&ra[0].room_id)||!displayed.contains(&rb[0].room_id){return None;}
 let ta=portal(a,&b.scene_id)?;let tb=portal(b,&a.scene_id)?;
 if !covered(pa,ta)||!covered(pb,tb)||ga["parents"].as_array()?.iter().filter(|p|covered(p,ta)).count()!=1
    ||gb["parents"].as_array()?.iter().filter(|p|covered(p,tb)).count()!=1{return None;}
 let ax=ga["origin_xz"][0].as_f64()?;let az=ga["origin_xz"][1].as_f64()?;let bx=gb["origin_xz"][0].as_f64()?;let bz=gb["origin_xz"][1].as_f64()?;
 if az!=bz || if east {bx!=ax+f64::from(a.bounds_m.width)} else {ax!=bx+f64::from(b.bounds_m.width)} {return None;}
 let z0=ta.polygon.iter().map(|p|f64::from(p[1])).fold(f64::INFINITY,f64::min);let z1=ta.polygon.iter().map(|p|f64::from(p[1])).fold(f64::NEG_INFINITY,f64::max);
 let rz0=tb.polygon.iter().map(|p|f64::from(p[1])).fold(f64::INFINITY,f64::min);let rz1=tb.polygon.iter().map(|p|f64::from(p[1])).fold(f64::NEG_INFINITY,f64::max);
 if z0!=rz0||z1!=rz1||z0>=z1{return None;}
 Some((KnownMapJunction{from_room_id:ra[0].room_id.clone(),to_room_id:rb[0].room_id.clone(),x:if east {bx}else{ax},z:az+(z0+z1)/2.0,replaced_connection_ids:Vec::new()},ta.id.clone(),tb.id.clone()))
}
pub(super) fn project(state:&RuntimeState,scene:&SceneRuntime,current:&serde_json::Value,out:&mut CapabilityProjection){
 out.map_known_junctions=None;
 let Some(layer)=current["layer_id"].as_str() else{return;};
 if out.map_topology_authorized!=Some(true)||state.effective_rules_v6.map.level(KnowledgeChannel::Connections)==KnowledgeLevel::None || layer!="ground_y0" || state.world.explored.world_id!=state.world.world_id{return;}
 let Some(frame)=current["frame_id"].as_str() else{return;};
 let displayed:std::collections::BTreeSet<String>=out.explored_map.rooms.iter().map(|r|r.room_id.clone()).collect();let mut result=Vec::new();let mut used=std::collections::BTreeSet::new();
 for east in [false,true] {let mut cursor=scene.clone();for _ in 0..9 {
  let target_id=if state.world.world_id=="mist_harbor" {cursor.mh_shore_neighbor(east)}else if state.world.world_id=="clockworks" {cursor.cw_early_neighbor(east)}else{None}.map(|s|s.scene_id.clone());
  let Some(target_id)=target_id else{break;};let next=if state.world.world_id=="mist_harbor"{cursor.mh_shore_candidate(&target_id)}else{cursor.cw_early_candidate(&target_id)};let Some(next)=next else{break;};
  if let Some((mut junction,forward,reverse))=pair(state,&cursor,&next,east,frame,layer,&displayed){
   let key=if junction.from_room_id<junction.to_room_id {(junction.from_room_id.clone(),junction.to_room_id.clone())}else{(junction.to_room_id.clone(),junction.from_room_id.clone())};
   if used.insert(key){junction.replaced_connection_ids=out.explored_map.connections.iter().filter(|c| (c.connection_id==forward||c.connection_id==reverse) && ((c.from_room_id==junction.from_room_id&&c.to_room_id==junction.to_room_id)||(c.from_room_id==junction.to_room_id&&c.to_room_id==junction.from_room_id))).map(|c|c.connection_id.clone()).collect();result.push(junction);}
  }
  cursor=next;
 }}
 if !result.is_empty(){out.map_known_junctions=Some(result);}
}

/// Current-frame reference only; the two directions have different history.
pub(super) fn project_height_reference(state:&RuntimeState,scene:&SceneRuntime,out:&mut CapabilityProjection) {
 out.map_known_height_references=None;
 let d=scene.current_scene();
 if state.world.world_id!="clockworks" || d.world_id!="clockworks" || scene.world_epoch!=state.world.revision.world_epoch
  || out.map_topology_authorized!=Some(true)
  || !matches!(state.effective_rules_v6.map.level(KnowledgeChannel::Connections),KnowledgeLevel::ExploredOnly|KnowledgeLevel::Known|KnowledgeLevel::Full)
  || state.world_persistent_v1.map_knowledge_archive_v1.as_ref().is_some_and(|a|!a.validate()) {return;}
 let Some(contact)=mhcw_history_contact(state,d) else{return;};
 let gear_id="mhcw_v1:clockworks:cw_gear_shaft:upper_static:cw_gear_furnace_landing";
 let ground_id="mhcw_v1:clockworks:cw_gear_shaft:ground:facility_floor";
 let furnace_id="mhcw_v1:clockworks:cw_furnace_heart:ground:facility_floor";
 let forward=match d.scene_id.as_str() {
  "cw_gear_shaft" if contact.layer=="gear_static_decks_y2" && contact.region==gear_id=>true,
  "cw_furnace_heart" if contact.layer=="ground_y0" && contact.region==furnace_id=>false,
  _=>return,
 };
 let (Some(gear),Some(furnace))=(scene.cw_height_map_scene("cw_gear_shaft"),scene.cw_height_map_scene("cw_furnace_heart")) else{return;};
 if gear.verified_source_sha256!="91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9"
  || furnace.verified_source_sha256!="f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5" {return;}
 let f:Vec<_>=gear.transitions.iter().filter(|t|t.id=="cw_gear_to_furnace_heart").collect();
 let r:Vec<_>=furnace.transitions.iter().filter(|t|t.id=="cw_furnace_heart_return_to_gear_shaft").collect();
 if f.len()!=1 || r.len()!=1 || f[0].to_scene_id!="cw_furnace_heart" || f[0].spawn_id!="cw_furnace_heart_spawn"
  || f[0].polygon!=vec![[22.,6.5],[24.,6.5],[24.,9.5],[22.,9.5]] || f[0].height_range_m.map(|h|h.0)!=Some([1.9,2.1])
  || r[0].to_scene_id!="cw_gear_shaft" || r[0].spawn_id!="cw_gear_shaft_spawn"
  || r[0].polygon!=vec![[0.5,6.5],[2.5,6.5],[2.5,9.5],[0.5,9.5]] || r[0].height_range_m.is_some() {return;}
 if gear.spawns.iter().filter(|s|s.id=="cw_gear_shaft_spawn"&&s.kind=="player"&&s.position==[3.,0.,8.]).count()!=1
  || furnace.spawns.iter().filter(|s|s.id=="cw_furnace_heart_spawn"&&s.kind=="player").count()!=1 {return;}
 let (Some(gear_group),Some(furnace_group))=(mhcw_map_region::group(gear,if forward {"gear_static_decks_y2"}else{"ground_y0"}),mhcw_map_region::group(furnace,"ground_y0")) else{return;};
 if gear_group["frame_id"].as_str()!=Some("cw_early_v1") || furnace_group["frame_id"].as_str()!=Some("cw_later_base_v1") {return;}
 let gear_parent=if forward {gear_id}else{ground_id};
 if !mhcw_map_region::stored(gear_group,&mhcw_map_region::history_rows(state,gear_group)).iter().any(|x|x==gear_parent)
  || !mhcw_map_region::stored(furnace_group,&mhcw_map_region::history_rows(state,furnace_group)).iter().any(|x|x==furnace_id) {return;}
 let (group,parent,kind,frame,x,z,offset)=if forward {(gear_group,gear_id,"gear_to_furnace","cw_early_v1",119.0,8.0,96.0)}else{(furnace_group,furnace_id,"furnace_return_to_gear_ground","cw_later_base_v1",1.5,8.0,0.0)};
 let Some(rows)=mhcw_map_region::rows(group,parent) else{return;};if rows.len()!=1{return;}
 let row=&rows[0];if !out.map_explored_room_ids.as_ref().is_some_and(|ids|ids.contains(&row.room_id)){return;}
 let mut displayed=out.explored_map.rooms.iter().filter(|r|r.room_id==row.room_id);
 let Some(shown)=displayed.next() else{return;};
 if displayed.next().is_some() || shown.outline_m.len()!=row.outline_m.len()
  || !shown.outline_m.iter().zip(&row.outline_m).all(|(a,b)|a.x_m==(f64::from(b.x_m)+offset) as f32 && a.z_m==b.z_m && a.y_m==b.y_m) {return;}
 out.map_known_height_references=Some(vec![crate::world_v3::KnownHeightMapReference{kind:kind.into(),scene_source_sha256:d.verified_source_sha256.clone(),frame_id:frame.into(),current_room_id:row.room_id.clone(),x,z}]);
}
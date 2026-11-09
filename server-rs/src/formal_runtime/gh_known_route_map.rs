//! Neutral, view-only Grey Hive seam markers. They confer no traversal rights.
use super::*;
use crate::player_rules::{KnowledgeChannel, KnowledgeLevel};
use crate::world_v3::{CapabilityProjection, KnownMapJunction};

struct Edge { a: &'static str, b: &'static str, portal: &'static str, pa: &'static str, pb: &'static str }
const EDGES: &[Edge] = &[
 Edge{a:"gh_entry_maintenance",b:"gh_power_room",portal:"gh_entry_to_power_room",pa:"grey_hive.gh_entry_maintenance.info_v1.entry_transit",pb:"corr_power_w"},
 Edge{a:"gh_power_room",b:"gh_gate_a",portal:"gh_power_to_gate_a",pa:"corr_power_e",pb:"corr_gate_w"},
 Edge{a:"gh_gate_a",b:"gh_central_shaft",portal:"gh_gate_a_to_shaft",pa:"corr_gate_e",pb:"grey_hive.gh_central_shaft.upper_active_info_v1.cross_transit"},
 Edge{a:"gh_central_shaft",b:"gh_lockdown",portal:"gh_shaft_to_lockdown",pa:"grey_hive.gh_central_shaft.upper_active_info_v1.cross_transit",pb:"grey_hive.gh_lockdown.dense_info_v1.west_transit"},
 Edge{a:"gh_lockdown",b:"gh_bio_isolation",portal:"gh_lockdown_to_bio_isolation",pa:"grey_hive.gh_lockdown.dense_info_v1.east_transit",pb:"grey_hive.gh_bio_isolation.dense_info_v1.main_transit"},
 Edge{a:"gh_bio_isolation",b:"gh_gate_b",portal:"gh_bio_to_gate_b",pa:"grey_hive.gh_bio_isolation.dense_info_v1.main_transit",pb:"grey_hive.gh_gate_b.info_v1.west_approach"},
 Edge{a:"gh_gate_b",b:"gh_deep_decon",portal:"gh_gate_b_to_deep_decon",pa:"grey_hive.gh_gate_b.info_v1.east_approach",pb:"grey_hive.gh_deep_decon.dense_info_v1.main_transit"},
 Edge{a:"gh_deep_decon",b:"gh_sentinel_arena",portal:"gh_deep_decon_to_sentinel_arena",pa:"grey_hive.gh_deep_decon.dense_info_v1.main_transit",pb:"grey_hive.gh_sentinel_arena.info_v1.west_approach"},
 Edge{a:"gh_sentinel_arena",b:"gh_beacon",portal:"gh_sentinel_arena_to_beacon",pa:"grey_hive.gh_sentinel_arena.info_v1.east_approach",pb:"grey_hive.beacon_exit.info_v1.gh_beacon.operational_hall"},
 Edge{a:"gh_beacon",b:"gh_exit",portal:"gh_beacon_to_exit",pa:"grey_hive.beacon_exit.info_v1.gh_beacon.operational_hall",pb:"grey_hive.beacon_exit.info_v1.gh_exit.operational_hall"},
];
fn origin(scene:&str)->Option<(f64,f64)> {Some(match scene {
 "gh_entry_maintenance"=>(-20.,0.),"gh_power_room"=>(0.,0.),"gh_gate_a"=>(20.,0.),
 "gh_central_shaft"=>(40.,-1.),"gh_lockdown"=>(64.,-1.),"gh_bio_isolation"=>(88.,-1.),
 "gh_gate_b"=>(112.,-1.),"gh_deep_decon"=>(136.,-1.),"gh_sentinel_arena"=>(160.,-1.),
 "gh_beacon"=>(184.,-1.),"gh_exit"=>(208.,-1.),_=>return None})}
fn rows_origin(scene:&str)->Option<(f64,f64)> {match scene {
 "gh_power_room"|"gh_gate_a"=>Some((0.,0.)),"gh_beacon"|"gh_exit"=>Some((184.,-1.)),_=>origin(scene)}}
fn rectangle_contains(row:&crate::world_v3::ExploredRoom,x:f64,z:f64)->bool {
 if row.outline_m.len()!=4{return false;}
 let (x0,x1,z0,z1)=row.outline_m.iter().fold((f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY),|a,p|
  (a.0.min(f64::from(p.x_m)),a.1.max(f64::from(p.x_m)),a.2.min(f64::from(p.z_m)),a.3.max(f64::from(p.z_m))));
 x0<=x && x<=x1 && z0<=z && z<=z1
}
fn endpoint_unchecked(state:&RuntimeState,scene:&crate::scene_runtime::SceneDefinition,parent:&str,x:f64,z:f64,out:&CapabilityProjection)->Option<String>{
 let mut records=gh_map_archive::catalog().iter().filter(|r|r.scene_id==scene.scene_id && r.scene_source_sha256==scene.verified_source_sha256 && r.layer_id=="upper_y0" && r.parent_id==parent);
 let r=records.next()?;if records.next().is_some() {return None;}
 let history=gh_map_archive::history_rows(state);let authorized=out.map_explored_room_ids.as_ref()?;let (ox,oz)=rows_origin(&scene.scene_id)?;
 let mut seam=None;
 for row in &r.rows {
  if history.iter().filter(|v|v.room_id==row.room_id && *v==row).count()!=1 || history.iter().filter(|v|v.room_id==row.room_id).count()!=1 || !authorized.contains(&row.room_id) {return None;}
  let mut displayed=out.explored_map.rooms.iter().filter(|v|v.room_id==row.room_id);let d=displayed.next()?;if displayed.next().is_some()||d.outline_m.len()!=row.outline_m.len(){return None;}
  if !d.outline_m.iter().zip(&row.outline_m).all(|(a,b)|a.x_m==(f64::from(b.x_m)+ox) as f32 && a.z_m==(f64::from(b.z_m)+oz) as f32 && a.y_m==b.y_m){return None;}
  if rectangle_contains(d,x,z) {if seam.is_some(){return None;} seam=Some(d.room_id.clone());}
 }
 seam
}
fn endpoint(state:&RuntimeState,scene:&crate::scene_runtime::SceneDefinition,parent:&str,x:f64,z:f64,out:&CapabilityProjection)->Option<String>{
 let selected=endpoint_unchecked(state,scene,parent,x,z,out)?;
 // Reject an ambiguous ownership at the exact portal anchor.
 let mut owners=std::collections::BTreeSet::new();
 for r in gh_map_archive::catalog().iter().filter(|r|r.scene_id==scene.scene_id && r.scene_source_sha256==scene.verified_source_sha256 && r.layer_id=="upper_y0") {
  if endpoint_unchecked(state,scene,&r.parent_id,x,z,out).is_some(){owners.insert(r.parent_id.as_str());}
 }
 (owners.len()==1 && owners.contains(parent)).then_some(selected)
}
pub(super) fn project(state:&RuntimeState,scene:&SceneRuntime,key:Option<&str>,out:&mut CapabilityProjection){
 let current=scene.current_scene();
 if scene.world_epoch!=state.world.revision.world_epoch || !gh_map_archive::committed_frame(state,current,key)
  || key.is_some_and(|k|k.starts_with("shaft_lower_")) || out.map_topology_authorized!=Some(true)
  || state.effective_rules_v6.map.level(KnowledgeChannel::Connections)==KnowledgeLevel::None {return;}
 let mut result=Vec::new();
 for e in EDGES {
  let (Some(a),Some(b),Some((ax,az)),Some((bx,bz)))=(scene.gh_map_scene(e.a),scene.gh_map_scene(e.b),origin(e.a),origin(e.b)) else{continue;};
  let mut ts=a.transitions.iter().filter(|t|t.id==e.portal && t.to_scene_id==e.b);let Some(t)=ts.next() else{continue;};if ts.next().is_some()||t.polygon.len()!=4 {continue;}
  let seam=ax+f64::from(a.bounds_m.width);if seam!=bx {continue;}
  let z0=t.polygon.iter().map(|p|f64::from(p[1])+az).fold(f64::INFINITY,f64::min);let z1=t.polygon.iter().map(|p|f64::from(p[1])+az).fold(f64::NEG_INFINITY,f64::max);
  // Where a reciprocal portal is authored, it must describe the same opening.
  let reverse:Vec<_>=b.transitions.iter().filter(|v|v.to_scene_id==e.a).collect();
  if reverse.len()>1 {continue;}
  if let Some(v)=reverse.first() {
   if v.polygon.len()!=4 {continue;}
   let rz0=v.polygon.iter().map(|p|f64::from(p[1])+bz).fold(f64::INFINITY,f64::min);
   let rz1=v.polygon.iter().map(|p|f64::from(p[1])+bz).fold(f64::NEG_INFINITY,f64::max);
   if rz0!=z0 || rz1!=z1 {continue;}
  }
  let px=t.polygon.iter().map(|p|f64::from(p[0])+ax).fold(f64::NEG_INFINITY,f64::max);if px!=seam||z0>=z1||az!=bz && (z0-bz<0. || z1-bz>f64::from(b.bounds_m.depth)){continue;}let z=(z0+z1)/2.;
  let (Some(from),Some(to))=(endpoint(state,a,e.pa,seam,z,out),endpoint(state,b,e.pb,seam,z,out)) else{continue;};if from==to{continue;}
  let replaced_connection_ids=out.explored_map.connections.iter().filter(|c|(c.connection_id==e.portal || reverse.first().is_some_and(|v|v.id==c.connection_id)) && ((c.from_room_id==from&&c.to_room_id==to)||(c.from_room_id==to&&c.to_room_id==from))).map(|c|c.connection_id.clone()).collect();
  result.push(KnownMapJunction{from_room_id:from,to_room_id:to,x:seam,z,replaced_connection_ids});
 }
 if !result.is_empty(){out.map_known_junctions=Some(result);}
}

/// A known vertical relation is presentation only; it does not imply a
/// currently available cabin, a traversable gate, or a newly observed room.
pub(super) fn project_vertical(state:&RuntimeState,scene:&SceneRuntime,key:Option<&str>,out:&mut CapabilityProjection) {
 out.map_known_vertical_links=None;
 let d=scene.current_scene();
 if d.world_id!="grey_hive" || d.scene_id!="gh_central_shaft"
  || d.verified_source_sha256!="6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832"
  || scene.world_epoch!=state.world.revision.world_epoch || state.shaft_deferred || state.shaft_owner.is_none()
  || !gh_map_archive::committed_frame(state,d,key) || out.map_topology_authorized!=Some(true)
  || !matches!(state.effective_rules_v6.map.level(KnowledgeChannel::Connections),KnowledgeLevel::ExploredOnly|KnowledgeLevel::Known|KnowledgeLevel::Full) {return;}
 let support=shaft_region_support_view(state,d);
 if floor_route_region_key(state,scene,support.as_ref())!=key {return;}
 let upper=match key {Some("shaft_active_open"|"shaft_active_closed")=>true,Some("shaft_lower_open"|"shaft_lower_closed")=>false,_=>return};
 if state.world_persistent_v1.map_knowledge_archive_v2.as_ref().is_some_and(|a|!a.validate()) {return;}
 // Unknown or contradictory observed variant tags cannot establish upper history.
 let tags=[state.world.shaft_upper_map_observation.as_ref(),state.world_persistent_v1.map_knowledge_archive_v2.as_ref().and_then(|a|a.shaft_upper_observation.as_ref())];
 if !tags.iter().any(|t|t.is_some()) || tags.iter().flatten().any(|t|t.trusted_family()!=Some("shaft_active")) {return;}
 let up="grey_hive.gh_central_shaft.upper_active_info_v1.north_service";
 let low="grey_hive.gh_central_shaft.lower_info_v1.front_landing_corridor";
 let history=gh_map_archive::history_rows(state);
 if !floor_route_info::stored_regions("shaft_active_open",&history).iter().any(|v|v==up)
  || !floor_route_info::stored_regions("shaft_lower_open",&history).iter().any(|v|v==low) {return;}
 let (parent,index,other,ox,oz,layer)=if upper {(up,if key==Some("shaft_active_closed"){6}else{3},format!("{low}:part0"),40.0,-1.0,"upper_y0")}
  else {(low,if key==Some("shaft_lower_closed"){1}else{0},format!("{up}:part3"),0.0,0.0,"lower_y_minus4")};
 // Current gate geometry may split one historical parent into more display
 // pieces. Validate the actual selected pieces without changing stored rows.
 let Some(parts)=floor_route_info::parts(key.unwrap(),parent) else{return;};
 if parts.len()!=match key {Some("shaft_active_closed")=>7,Some("shaft_active_open")=>5,Some("shaft_lower_closed")=>5,Some("shaft_lower_open")=>4,_=>return} {return;}
 for (i,part) in parts.iter().enumerate() {
  let id=format!("{parent}:part{i}");
  let mut displayed=out.explored_map.rooms.iter().filter(|r|r.room_id==id);
  let Some(shown)=displayed.next() else{return;};
  if displayed.next().is_some() || shown.outline_m.len()!=part.len()
   || !shown.outline_m.iter().zip(part).all(|(a,b)|a.x_m==(f64::from(b[0])+ox) as f32 && a.z_m==(f64::from(b[1])+oz) as f32 && a.y_m==if upper {0.0}else{-4.0}) {return;}
 }
 let x=(3.5_f32+ox as f32) as f64;let z=(5.15_f32+oz as f32) as f64;
 let current=format!("{parent}:part{index}");
 let mut display=out.explored_map.rooms.iter().filter(|r|r.room_id==current);
 if !display.next().is_some_and(|r|rectangle_contains(r,x,z)) || display.next().is_some(){return;}
 out.map_known_vertical_links=Some(vec![crate::world_v3::KnownVerticalMapLink{support_id:"gh_shaft_two_stop_lift".into(),scene_source_sha256:d.verified_source_sha256.clone(),
  current_layer_id:layer.into(),current_room_id:current,other_room_id:other,x,z}]);
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn gh_known_route_declared_parents_are_canonical_upper_groups() {
  for edge in EDGES {for (scene,id) in [(edge.a,edge.pa),(edge.b,edge.pb)] {
   assert_eq!(gh_map_archive::catalog().iter().filter(|r|r.scene_id==scene && r.parent_id==id && r.layer_id=="upper_y0").count(),1,"{scene}: {id}");
  }}
 }
}

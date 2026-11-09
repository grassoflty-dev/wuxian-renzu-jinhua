//! Decon information regions, separate from physical collision and Save schema.
use std::sync::OnceLock;
use serde_json::Value;
use crate::{scene_runtime::SceneDefinition,world_v3::{ExploredRoom,Vec3}};

const SOURCE:&str=include_str!("../../../apps/godot_playable/data/gh_decon_regions.local.json");
const SHA:&str="a53fd5c62a4540a375b51f7286798c7bab1a5c73aa37e98d3f4d6bad7b24f1c0";
static DATA:OnceLock<Option<Value>>=OnceLock::new();
fn data()->Option<&'static Value> {
 DATA.get_or_init(|| {
  let v:Value=serde_json::from_str(SOURCE).ok()?;
  if v["schema"].as_str()!=Some("gh-decon-info/v1") || v["regions"].as_array()?.len()!=9
   || v["owners"].as_array()?.len()!=25 || v["fragments"]["rows"].as_array()?.len()!=111 {return None;}
  if v["regions"].as_array()?.iter().map(|r|r["polygon_parts_xz"].as_array().map_or(0,Vec::len)).sum::<usize>()!=47{return None;}
  Some(v)
 }).as_ref()
}
pub(super) fn admitted(scene:&SceneDefinition)->bool {
 let Some(d)=data() else{return false;};
 if scene.world_id!="grey_hive" || scene.scene_id!="gh_deep_decon"
  || scene.verified_source_sha256!=SHA || scene.collision.len()!=25 {return false;}
 d["owners"].as_array().unwrap().iter().all(|owner| {
  let Some(id)=owner["id"].as_str() else{return false;};
  let rows:Vec<_>=scene.collision.iter().filter(|r|r.id==id).collect();
  let Some(rect)=owner["rect_x0_z0_x1_z1"].as_array() else{return false;};
  if rows.len()!=1 || rect.len()!=4 || rows[0].requires_event.is_some()
   || rows[0].requires_actor_first_kill.is_some() {return false;}
  let values:Option<Vec<f32>>=rect.iter().map(|n|n.as_f64().map(|x|x as f32)).collect();
  let Some(r)=values else{return false;};
  rows[0].polygon==vec![[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]]
 })
}
pub(super) fn regions()->Option<&'static Vec<Value>> {data()?["regions"].as_array()}
pub(super) fn parts(id:&str)->Option<Vec<[[f32;2];4]>> {
 let row=regions()?.iter().find(|r|r["id"].as_str()==Some(id))?;
 row["polygon_parts_xz"].as_array()?.iter().map(|part| {
  let p=part.as_array()?; if p.len()!=4{return None;}
  let mut out=[[0.0;2];4];
  for (i,v) in p.iter().enumerate() {let a=v.as_array()?;if a.len()!=2{return None;}
   out[i]=[a[0].as_f64()? as f32,a[1].as_f64()? as f32];}
  Some(out)
 }).collect()
}
pub(super) fn occupied(scene:&SceneDefinition,p:Vec3,previous:Option<&str>)->Option<String> {
 if !admitted(scene) || !p.x_m.is_finite() || !p.z_m.is_finite() || p.y_m!=0.0{return None;}
 // A point strictly inside a collision is never an information region.
 for wall in &scene.collision {
  let r=&wall.polygon;
  if p.x_m>r[0][0] && p.x_m<r[2][0] && p.z_m>r[0][1] && p.z_m<r[2][1] {return None;}
 }
 let matches:Vec<String>=regions()?.iter().filter_map(|row| {
  let id=row["id"].as_str()?;
  parts(id)?.iter().any(|r|p.x_m>=r[0][0] && p.x_m<=r[2][0] && p.z_m>=r[0][1] && p.z_m<=r[2][1]).then(||id.to_owned())
 }).collect();
 previous.filter(|id|matches.iter().any(|x|x==id)).map(str::to_owned).or_else(||matches.into_iter().next())
}
fn part_id(id:&str,n:usize)->String {format!("{id}:part{n}")}
pub(super) fn rows(id:&str)->Option<Vec<ExploredRoom>> {
 Some(parts(id)?.iter().enumerate().map(|(n,p)|ExploredRoom {
  room_id:part_id(id,n),
  outline_m:p.iter().map(|v|Vec3{x_m:v[0],y_m:0.0,z_m:v[1]}).collect(),
 }).collect())
}
pub(super) fn stored_regions(rooms:&[ExploredRoom])->Vec<String> {
 let Some(all)=regions() else{return Vec::new();};
 all.iter().filter_map(|region| {
  let id=region["id"].as_str()?;let expected=rows(id)?;
  expected.iter().all(|e|rooms.iter().filter(|r|r.room_id==e.room_id && r.outline_m==e.outline_m).count()==1
   && rooms.iter().filter(|r|r.room_id==e.room_id).count()==1).then(||id.to_owned())
 }).collect()
}

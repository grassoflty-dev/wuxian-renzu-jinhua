//! Scene-qualified information geometry; gameplay remains scene-local.
use std::sync::OnceLock;
use serde_json::Value;
use crate::{scene_runtime::SceneDefinition,world_v3::{ExploredRoom,Vec3}};
const SOURCE:&str=include_str!("../../../apps/godot_playable/data/gh_beacon_exit_regions.local.json");
const BEACON:&str="e6434c598904d7fe36f4f48ebc67559e6d484eb535393a7b447bc99cbf86c766";
const EXIT:&str="b32ad8d1204e9eaf45512b0e2d5ea22e08f5bd09ec0b5e31980e41336c45ee09";
static DATA:OnceLock<Option<Value>>=OnceLock::new();
fn data()->Option<&'static Value> {
 DATA.get_or_init(|| {
  let v:Value=serde_json::from_str(SOURCE).ok()?;
  if v["schema"].as_str()!=Some("gh-beacon-exit-info/v1")
   || v["owners"]["gh_beacon"].as_array()?.len()!=13 || v["owners"]["gh_exit"].as_array()?.len()!=17
   || v["variants"]["beacon"]["regions"].as_array()?.len()!=4
   || v["variants"]["exit_open"]["regions"].as_array()?.len()!=7
   || v["variants"]["exit_closed"]["regions"].as_array()?.len()!=7 {return None;}
  Some(v)
 }).as_ref()
}
pub(super) fn admitted(scene:&SceneDefinition)->bool {
 let Some(d)=data() else{return false;};
 let expected=match scene.scene_id.as_str() {"gh_beacon"=>BEACON,"gh_exit"=>EXIT,_=>return false};
 if scene.world_id!="grey_hive" || scene.verified_source_sha256!=expected{return false;}
 let Some(owners)=d["owners"][&scene.scene_id].as_array() else{return false;};
 if scene.collision.len()!=owners.len(){return false;}
 owners.iter().all(|o| {
  let Some(id)=o["id"].as_str() else{return false;};
  let rows:Vec<_>=scene.collision.iter().filter(|r|r.id==id).collect();
  let Some(rect)=o["rect_x0_z0_x1_z1"].as_array() else{return false;};
  if rows.len()!=1 || rect.len()!=4 || rows[0].requires_actor_first_kill.is_some()
   || rows[0].requires_event.as_deref()!=o["requiresEvent"].as_str(){return false;}
  let values:Option<Vec<f32>>=rect.iter().map(|n|n.as_f64().map(|v|v as f32)).collect();
  let Some(r)=values else{return false;};
  rows[0].polygon==vec![[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]]
 })
}
fn variant(scene:&str,open:bool)->Option<&'static Value> {
 let key=if scene=="gh_beacon" {"beacon"} else if scene=="gh_exit" {if open {"exit_open"} else {"exit_closed"}} else {return None;};
 Some(&data()?["variants"][key])
}
pub(super) fn parts(scene:&str,id:&str,open:bool,pair:bool)->Option<Vec<[[f32;2];4]>> {
 let region=variant(scene,open)?["regions"].as_array()?.iter().find(|r|r["id"].as_str()==Some(id) && r["available"].as_bool()==Some(true))?;
 let key=if pair {"polygon_parts_pair_xz"} else {"polygon_parts_xz"};
 region[key].as_array()?.iter().map(|part| {
  let p=part.as_array()?;if p.len()!=4{return None;}let mut out=[[0.0;2];4];
  for(i,v)in p.iter().enumerate(){let a=v.as_array()?;if a.len()!=2{return None;}
   out[i]=[a[0].as_f64()? as f32,a[1].as_f64()? as f32];}Some(out)
 }).collect()
}
pub(super) fn occupied(scene:&SceneDefinition,p:Vec3,previous:Option<&str>,open:bool)->Option<String> {
 if !admitted(scene) || !p.x_m.is_finite() || !p.z_m.is_finite() || p.y_m!=0.0{return None;}
 for wall in &scene.collision {
  if wall.requires_event.as_deref()==Some("hive_extraction") && open {continue;}
  let r=&wall.polygon;
  if p.x_m>r[0][0] && p.x_m<r[2][0] && p.z_m>r[0][1] && p.z_m<r[2][1]{return None;}
 }
 let rows=variant(&scene.scene_id,open)?["regions"].as_array()?;
 let matches:Vec<String>=rows.iter().filter_map(|row| {
  let id=row["id"].as_str()?;
  parts(&scene.scene_id,id,open,false)?.iter().any(|r|p.x_m>=r[0][0] && p.x_m<=r[2][0] && p.z_m>=r[0][1] && p.z_m<=r[2][1]).then(||id.to_owned())
 }).collect();
 previous.filter(|id|matches.iter().any(|x|x==id)).map(str::to_owned).or_else(||matches.into_iter().next())
}
pub(super) fn rows(scene:&str,id:&str)->Option<Vec<ExploredRoom>> {
 Some(parts(scene,id,true,true)?.iter().enumerate().map(|(n,p)|ExploredRoom {
  room_id:format!("{id}:part{n}"),outline_m:p.iter().map(|v|Vec3{x_m:v[0],y_m:0.0,z_m:v[1]}).collect()
 }).collect())
}
pub(super) fn stored_regions(scene:&str,open:bool,rooms:&[ExploredRoom])->Vec<String> {
 let Some(variant)=variant(scene,open) else{return Vec::new();};
 let Some(regions)=variant["regions"].as_array() else{return Vec::new();};
 regions.iter().filter_map(|r| {
  if r["available"].as_bool()!=Some(true){return None;}let id=r["id"].as_str()?;
  let expected=rows(scene,id)?;
  expected.iter().all(|e|rooms.iter().filter(|r|r.room_id==e.room_id && r.outline_m==e.outline_m).count()==1
   && rooms.iter().filter(|r|r.room_id==e.room_id).count()==1).then(||id.to_owned())
 }).collect()
}

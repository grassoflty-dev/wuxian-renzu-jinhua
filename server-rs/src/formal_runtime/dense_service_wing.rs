//! Authored same-floor partitions. The original scene bytes and Save format remain unchanged.
use std::sync::OnceLock;
use crate::{continuous_kcc::{Aabb,StaticKccWorld},scene_runtime::SceneDefinition,world_v3::Vec3};
const SOURCE: &str = include_str!("../../../apps/godot_playable/data/gh_service_wing_dense.local.json");
#[derive(Clone)] struct Wall { id:String, scene:String, rect:Aabb }
static WALLS: OnceLock<Option<Vec<Wall>>> = OnceLock::new();
fn catalog() -> Option<&'static Vec<Wall>> {
    WALLS.get_or_init(|| {
        let data: serde_json::Value = serde_json::from_str(SOURCE).ok()?;
        if data.get("schema")?.as_str()? != "gh-service-wing-dense/v1" || data.get("worldId")?.as_str()? != "grey_hive" { return None; }
        let rows=data.get("walls")?.as_array()?;
        if rows.len()!=24 { return None; }
        let mut walls=Vec::new(); let mut ids=std::collections::BTreeSet::new();
        for row in rows {
            let id=row.get("id")?.as_str()?; let scene=row.get("sceneId")?.as_str()?;
            if !id.starts_with("dense_") || !ids.insert(id.to_owned()) || !["gh_power_room","gh_gate_a"].contains(&scene) { return None; }
            let a=row.get("rect")?.as_array()?; if a.len()!=4 {return None;}
            let x=a[0].as_f64()? as f32; let z=a[1].as_f64()? as f32; let xx=a[2].as_f64()? as f32; let zz=a[3].as_f64()? as f32;
            if x<0.0 || z<0.0 || xx>20.0 || zz>14.0 || (z<8.5 && zz>5.5) { return None; }
            walls.push(Wall{id:id.to_owned(),scene:scene.to_owned(),rect:Aabb::new(x,xx,z,zz).ok()?});
        }
        if walls.iter().filter(|w|w.scene=="gh_power_room").count()!=16 || walls.iter().filter(|w|w.scene=="gh_gate_a").count()!=8 { return None; }
        Some(walls)
    }).as_ref()
}
pub(super) fn admitted(scene:&SceneDefinition)->bool {
    scene.world_id=="grey_hive" && match scene.scene_id.as_str() {
        "gh_power_room" => scene.verified_source_sha256=="3d05edeca4a119bc7f2db7aa094041a7ad61380b00015a613fda787025e67854",
        "gh_gate_a" => scene.verified_source_sha256=="3c62f3c35a2c9f2e935707e7a93c34a8d6f5696ed730b8487c4efe0b6d1ee484",
        _=>false,
    }
}
pub(super) fn install(kcc:&mut StaticKccWorld,scene:&SceneDefinition,player:Option<(Vec3,f32)>) -> bool {
    if !admitted(scene) { return false; }
    let Some(rows)=catalog() else { return false; };
    for wall in rows.iter().filter(|w|w.scene==scene.scene_id) {
        let escape=player.is_some_and(|(p,r)| r.is_finite() && r>0.0 && p.x_m+r>wall.rect.min_x && p.x_m-r<wall.rect.max_x && p.z_m+r>wall.rect.min_z && p.z_m-r<wall.rect.max_z);
        kcc.add_dense_wall(wall.id.clone(),wall.rect,escape);
    }
    true
}

// The room polygons are authored presentation boundaries, not line-of-sight or save data.
// Validate the exact admitted layout before returning a unique occupied region.
const REGIONS: [(&str,&str,[f32;4]);16] = [
 ("power_nw","gh_power_room",[0.5,0.5,8.0,5.5]),("power_nm","gh_power_room",[8.0,0.5,14.0,5.5]),("power_ne","gh_power_room",[14.0,0.5,19.5,5.5]),
 ("power_sw","gh_power_room",[0.5,8.5,8.0,13.5]),("power_sm","gh_power_room",[8.0,8.5,14.0,13.5]),("power_se","gh_power_room",[14.0,8.5,19.5,13.5]),
 ("gate_nw","gh_gate_a",[20.5,0.5,29.5,5.5]),("gate_sw","gh_gate_a",[20.5,8.5,29.5,13.5]),("gate_ne","gh_gate_a",[30.5,0.5,39.5,5.5]),("gate_se","gh_gate_a",[30.5,8.5,39.5,13.5]),
 ("corr_power_w","gh_power_room",[0.0,5.5,8.0,8.5]),("corr_power_m","gh_power_room",[8.0,5.5,14.0,8.5]),("corr_power_e","gh_power_room",[14.0,5.5,20.0,8.5]),
 ("corr_gate_w","gh_gate_a",[20.0,5.5,29.5,8.5]),("corr_gate_threshold","gh_gate_a",[29.5,5.5,30.5,8.5]),("corr_gate_e","gh_gate_a",[30.5,5.5,40.0,8.5]),
];
static REGIONS_VALID: OnceLock<bool> = OnceLock::new();
fn regions_valid() -> bool {
 *REGIONS_VALID.get_or_init(|| {
  let Some(data)=serde_json::from_str::<serde_json::Value>(SOURCE).ok() else{return false;};
  let Some(rows)=data.get("regions").and_then(|v|v.as_array()) else{return false;};
  if catalog().is_none() || rows.len()!=REGIONS.len() {return false;}
  REGIONS.iter().all(|(id,scene,r)| {
   let matches:Vec<_>=rows.iter().filter(|v|v.get("id").and_then(|x|x.as_str())==Some(*id)).collect();
   if matches.len()!=1 || matches[0].get("sceneId").and_then(|x|x.as_str())!=Some(*scene) {return false;}
   let expected=[[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]];
   let Some(poly)=matches[0].get("polygon").and_then(|v|v.as_array()) else{return false;};
   poly.len()==4 && poly.iter().zip(expected).all(|(point,expected)| {
    let Some(pair)=point.as_array() else{return false;};
    pair.len()==2 && pair[0].as_f64()==Some(expected[0] as f64) && pair[1].as_f64()==Some(expected[1] as f64)
   })
  })
 })
}
pub(super) fn shared_region_polygon(id:&str)->Option<[[f32;2];4]> {
 if !regions_valid() {return None;}
 let r=REGIONS.iter().find(|row|row.0==id)?.2;
 Some([[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]])
}
pub(super) fn valid_stored_room(room:&crate::world_v3::ExploredRoom)->bool {
 let Some(poly)=shared_region_polygon(&room.room_id) else{return false;};
 room.outline_m.len()==4 && room.outline_m.iter().zip(poly).all(|(p,xy)|
  p.x_m==xy[0] && p.y_m==0.0 && p.z_m==xy[1])
}
pub(super) fn region_polygon(scene:&SceneDefinition,id:&str)->Option<[[f32;2];4]> {
 if !admitted(scene) || !regions_valid() {return None;}
 let row=REGIONS.iter().find(|row|row.0==id && row.1==scene.scene_id)?;
 let offset=if scene.scene_id=="gh_gate_a" {20.0} else {0.0};
 let r=row.2;
 Some([[r[0]-offset,r[1]],[r[2]-offset,r[1]],[r[2]-offset,r[3]],[r[0]-offset,r[3]]])
}
pub(super) fn occupied(scene:&SceneDefinition,pos:Vec3,previous:Option<&str>,gate_open:bool)->Option<&'static str> {
 if !admitted(scene) || !regions_valid() || !pos.x_m.is_finite() || !pos.y_m.is_finite() || !pos.z_m.is_finite() || pos.y_m!=0.0 {return None;}
 let x=pos.x_m+if scene.scene_id=="gh_gate_a" {20.0}else{0.0};
 let includes=|row:&&(&str,&str,[f32;4])| row.1==scene.scene_id && (row.0!="corr_gate_threshold" || gate_open) && x>=row.2[0] && x<=row.2[2] && pos.z_m>=row.2[1] && pos.z_m<=row.2[3];
 if let Some(prior)=previous {if let Some(row)=REGIONS.iter().find(|row|row.0==prior && includes(row)){return Some(row.0);}}
 REGIONS.iter().filter(|row|includes(row)).map(|row|row.0).min()
}

//! Generic support fixtures only; no authored Gear Shaft or reward admission.
use wuxian_horror_ch1::{
    continuous_kcc::{step_kcc,step_kcc_at_world_time,Aabb,KccBody,KccError,KccTerrainRegion,StaticKccWorld},
    effects::TerrainTag,
    moving_support::{self,MovingSupportDefinition,SupportPhase},
    world_v3::Vec3,
};
fn support(id:&str)->MovingSupportDefinition { MovingSupportDefinition{id:id.into(),polygon:vec![[2.,2.],[8.,2.],[8.,8.],[2.,8.]],lower_m:0.,upper_m:2.,travel_ms:1000,endpoint_hold_ms:250} }
fn bounds()->Aabb {Aabb::new(0.,10.,0.,10.).unwrap()}
fn world(defs:Vec<MovingSupportDefinition>)->StaticKccWorld {StaticKccWorld::new(bounds(),vec![]).with_moving_supports(defs).unwrap()}
fn body(x:f32,y:f32,z:f32)->KccBody {KccBody::new(Vec3::new(x,y,z).unwrap())}
fn near(a:f32,b:f32) {assert!((a-b).abs()<0.0001,"{a} != {b}");}
fn step(body:&mut KccBody,world:&StaticKccWorld,axis:(f32,f32),time:u64) {step_kcc_at_world_time(body,world,axis,0.1,true,time).unwrap();}

#[test]
fn deterministic_phase_covers_endpoints_zero_hold_and_large_saved_clock() {
    use SupportPhase::*;let s=support("lift");
    for (time,phase,height) in [(0,LowerHold,0.),(249,LowerHold,0.),(250,Rising,0.),(750,Rising,1.),(1250,UpperHold,2.),(1499,UpperHold,2.),(1500,Falling,2.),(2000,Falling,1.),(2500,LowerHold,0.)] {
        let pose=s.pose_at(time).unwrap();assert_eq!(pose.phase,phase);near(pose.height_m,height);
    }
    assert_eq!(s.pose_at(u64::MAX),s.pose_at(u64::MAX%2500));
    for travel_ms in [250,60000] {let mut s=s.clone();s.endpoint_hold_ms=0;s.travel_ms=travel_ms;s.validate(bounds()).unwrap();assert_eq!(s.pose_at(0).unwrap().phase,Rising);assert_eq!(s.pose_at(travel_ms as u64).unwrap().phase,Falling);near(s.pose_at(2*travel_ms as u64).unwrap().height_m,0.);}
}

#[test]
fn grounded_rider_follows_saved_clock_without_accumulated_drift() {
    let s=support("lift");let w=world(vec![s.clone()]);let mut p=body(5.,0.,5.);
    for time in (0..50_000).step_by(100) {step(&mut p,&w,(0.,0.),time);near(p.position_m.y_m,s.pose_at(time+100).unwrap().height_m);assert!(p.grounded);}
}

#[test]
fn boarding_requires_full_foot_contact_and_does_not_snap_from_below() {
    let w=world(vec![support("lift")]);let mut p=body(5.,1.1,5.);p.grounded=false;p.velocity_mps.y_m=-1.;step(&mut p,&w,(0.,0.),750);near(p.position_m.y_m,1.2);assert!(p.grounded);
    let mut below=body(5.,0.,5.);step(&mut below,&w,(0.,0.),750);near(below.position_m.y_m,0.);
    let mut edge=body(2.34,1.,5.);step(&mut edge,&w,(0.,0.),750);assert!(!edge.grounded);assert!(edge.position_m.y_m<1.);
}

#[test]
fn dismount_uses_one_gravity_step_and_jump_does_not_keep_platform_contact() {
    let w=world(vec![support("lift")]);
    for (time,height,expected) in [(750,1.,1.04),(1750,1.5,1.14)] {let mut p=body(7.6,height,5.);step(&mut p,&w,(1.,0.),time);assert!(!p.grounded);near(p.position_m.y_m,expected);}
    let mut jumped=body(5.,1.,5.);assert!(jumped.try_jump());step(&mut jumped,&w,(0.,0.),750);assert!(!jumped.grounded);near(jumped.position_m.y_m,1.39);
}

#[test]
fn support_scene_dash_sweeps_thin_walls_and_world_bounds() {
    let mut s=support("lift");s.polygon=vec![[1.,2.],[8.,2.],[8.,8.],[1.,8.]];
    let w=StaticKccWorld::new(bounds(),vec![Aabb::new(2.7,2.75,1.,9.).unwrap()]).with_moving_supports(vec![s]).unwrap();
    let mut p=body(2.2,1.,5.);assert!(p.try_dash());let events=step_kcc_at_world_time(&mut p,&w,(1.,0.),0.1,true,750).unwrap();near(p.position_m.x_m,2.2);assert!(events.iter().any(|e|e.axis=="x"));
    let w=world(vec![support("lift")]);let mut p=body(9.6,0.,5.);assert!(p.try_dash());step(&mut p,&w,(1.,0.),750);near(p.position_m.x_m,9.6);
}

#[test]
fn support_owns_contact_over_floor_conveyor_and_overlaps_have_stable_ids() {
    let s=support("a");let mut b=support("b");b.travel_ms=2000;
    for defs in [vec![s.clone(),b.clone()],vec![b,s.clone()]] {let mut p=body(5.,0.,5.);step(&mut p,&world(defs),(0.,0.),250);near(p.position_m.y_m,0.2);}
    let w=world(vec![s]).with_terrain_regions(vec![KccTerrainRegion{id:"belt".into(),tag:TerrainTag::Conveyor,polygon:vec![[2.,2.],[8.,2.],[8.,8.],[2.,8.]],surface_velocity_mps:Some([2.,0.])}]);
    let mut p=body(5.,0.,5.);step(&mut p,&w,(0.,0.),0);near(p.position_m.x_m,5.);step(&mut p,&w,(0.,0.),250);near(p.position_m.x_m,5.);near(p.position_m.y_m,0.2);
}

#[test]
fn ceiling_floor_and_authored_traversal_remain_bounded() {
    let w=world(vec![support("lift")]);let mut p=body(5.,2.99,5.);p.grounded=false;p.velocity_mps.y_m=5.;let events=step_kcc_at_world_time(&mut p,&w,(0.,0.),0.1,true,0).unwrap();near(p.position_m.y_m,3.);near(p.velocity_mps.y_m,0.);assert!(events.iter().any(|e|e.axis=="ceiling"));
    let mut p=body(1.,0.02,1.);p.grounded=false;p.velocity_mps.y_m=-5.;step(&mut p,&w,(0.,0.),0);near(p.position_m.y_m,0.);assert!(p.grounded);
    let mut p=body(5.,1.2,5.);p.grounded=false;p.velocity_mps.y_m=-5.;step_kcc_at_world_time(&mut p,&w,(0.,0.),0.1,false,750).unwrap();near(p.position_m.y_m,1.2);near(p.velocity_mps.y_m,0.);
}

#[test]
fn missing_time_and_overflow_reject_before_body_or_timers_change() {
    let w=world(vec![support("lift")]);let mut p=body(5.,0.,5.);assert!(p.try_dash());let before=p.clone();assert_eq!(step_kcc(&mut p,&w,(0.,0.),0.1),Err(KccError::InvalidWorld));assert_eq!(p,before);assert_eq!(step_kcc_at_world_time(&mut p,&w,(0.,0.),0.1,true,u64::MAX),Err(KccError::InvalidDelta));assert_eq!(p,before);
}

#[test]
fn malformed_catalogs_are_rejected_without_admitting_geometry() {
    let s=support("valid");let mut variants=vec![];
    for (lower,upper,travel,hold) in [(-0.1,2.,1000,250),(0.,3.1,1000,250),(0.,0.09,1000,250),(0.,2.,249,250),(0.,2.,60001,250),(0.,2.,1000,30001),(f32::NAN,2.,1000,250)] {let mut v=s.clone();v.lower_m=lower;v.upper_m=upper;v.travel_ms=travel;v.endpoint_hold_ms=hold;variants.push(v);}
    for polygon in [vec![[2.,2.],[8.,8.],[8.,2.],[2.,8.]],vec![[2.,2.],[5.,2.],[8.,2.],[8.,8.],[2.,8.]],vec![[2.,2.],[8.,2.],[4.,4.],[8.,8.],[2.,8.]],vec![[2.,2.],[8.,2.],[8.,8.],[2.,2.]],vec![[2.,2.],[11.,2.],[8.,8.],[2.,8.]],vec![[f32::NAN,2.],[8.,2.],[8.,8.],[2.,8.]]] {let mut v=s.clone();v.polygon=polygon;variants.push(v);}
    let mut bad=s.clone();bad.id="".into();variants.push(bad);
    for v in variants {assert!(world_result(vec![v]).is_err());}
    assert!(world_result(vec![s.clone(),s.clone()]).is_err());assert!(world_result((0..65).map(|i|{let mut v=s.clone();v.id=format!("lift-{i}");v}).collect()).is_err());
}
fn world_result(defs:Vec<MovingSupportDefinition>)->Result<StaticKccWorld,KccError> {StaticKccWorld::new(bounds(),vec![]).with_moving_supports(defs)}

#[test]
fn persisted_phase_rejects_stale_malformed_duplicate_or_changed_catalog() {
    let defs=vec![support("lift")];let value=moving_support::frame(&defs,7,750).unwrap();moving_support::validate_saved_frame_shape(&value).unwrap();moving_support::validate_frame(&value,&defs,7,750).unwrap();
    let mut variants=vec![];
    let mut v=value.clone();v.world_epoch+=1;variants.push(v);let mut v=value.clone();v.server_time_ms+=1;variants.push(v);let mut v=value.clone();v.poses.push(v.poses[0].clone());variants.push(v);let mut v=value.clone();v.poses.clear();variants.push(v);let mut v=value.clone();v.poses[0].height_m=f32::NAN;variants.push(v);let mut v=value.clone();v.poses[0].phase=SupportPhase::Falling;variants.push(v);let mut v=value.clone();v.poses[0].phase_elapsed_ms+=1;variants.push(v);let mut v=value.clone();v.poses[0].velocity_mps+=1.;variants.push(v);
    for v in variants {assert!(moving_support::validate_frame(&v,&defs,7,750).is_err());}
    assert!(moving_support::validate_frame(&value,&[],7,750).is_err());
    let held=moving_support::frame(&defs,7,0).unwrap();let mut changed=defs.clone();changed[0].travel_ms=2000;assert!(moving_support::validate_frame(&held,&changed,7,0).is_err(),"catalog fingerprint rejects changed timing even at identical endpoint pose");
    let w=world(defs);assert!(!w.validate_saved_support_frame(None,7,750));assert!(w.validate_saved_support_frame(Some(&value),7,750));assert!(!world(vec![]).validate_saved_support_frame(Some(&value),7,750));
}

#[test]
fn reversed_catalog_order_has_identical_canonical_fingerprint_and_frame() {
    let a=support("a");let mut b=support("b");b.travel_ms=2000;
    assert_eq!(moving_support::frame(&[a.clone(),b.clone()],4,1250).unwrap(),moving_support::frame(&[b,a],4,1250).unwrap());
}

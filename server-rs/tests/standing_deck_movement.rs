//! Generic physics fixtures; 2m deck/1m overlap are explicit test tuning,
//! not canonical Gear Shaft dimensions or authored campaign admission.
use sha2::{Digest,Sha256};
use wuxian_horror_ch1::{continuous_kcc::{Aabb,KccBody,StaticKccWorld,step_kcc_at_world_time},
    moving_support::{self,MovingSupportDefinition,StandingDeckDefinition,HeightRange,SupportPhase},world_v3::Vec3};
fn lift()->MovingSupportDefinition {MovingSupportDefinition{id:"a_lift".into(),polygon:vec![[1.,1.],[4.,1.],[4.,4.],[1.,4.]],lower_m:0.,upper_m:2.,travel_ms:1000,endpoint_hold_ms:500}}
fn deck()->StandingDeckDefinition {StandingDeckDefinition{id:"z_deck".into(),polygon:vec![[3.,1.],[7.,1.],[7.,4.],[3.,4.]],height_m:2.}}
fn world()->StaticKccWorld {StaticKccWorld::new(Aabb::new(0.,10.,0.,10.).unwrap(),vec![])}
fn body(x:f32,y:f32)->KccBody {KccBody::new(Vec3::new(x,y,2.).unwrap())}
fn step(p:&mut KccBody,w:&StaticKccWorld,x:f32,time:u64) {step_kcc_at_world_time(p,w,(x,0.),0.1,true,time).unwrap();}
fn near(a:f32,b:f32) {assert!((a-b).abs()<0.0001,"{a} != {b}");}

#[test]
fn matched_dock_transfers_onto_static_deck_and_does_not_descend_with_lift() {
    let w=world().with_moving_supports(vec![lift()]).unwrap().with_standing_decks(vec![deck()]).unwrap();let mut p=body(2.3,2.);
    for time in [1500,1600,1700] {step(&mut p,&w,1.,time);near(p.position_m.y_m,2.);assert!(p.grounded);}
    // Both footprints now cover the entire foot; stationary support wins even
    // though the moving lift's ID sorts first.
    for time in (1800..3000).step_by(100) {step(&mut p,&w,0.,time);near(p.position_m.y_m,2.);assert!(p.grounded);}
}

#[test]
fn dock_transfer_at_engine_fixed_step_never_drops_between_overlapping_surfaces() {
    let w=world().with_moving_supports(vec![lift()]).unwrap().with_standing_decks(vec![deck()]).unwrap();let mut p=body(2.3,2.);let mut time=1500;
    for _ in 0..24 {step_kcc_at_world_time(&mut p,&w,(1.,0.),1./60.,true,time).unwrap();time+=17;near(p.position_m.y_m,2.);assert!(p.grounded);}
    assert!(p.position_m.x_m>3.8);
}

#[test]
fn walking_under_or_jumping_from_lower_floor_does_not_snap_onto_two_meter_deck() {
    let w=world().with_standing_decks(vec![deck()]).unwrap();let mut p=body(4.,0.);step(&mut p,&w,0.,0);near(p.position_m.y_m,0.);
    assert!(p.try_jump());let mut max_y=0.0_f32;
    for time in (0..1500).step_by(100) {step(&mut p,&w,0.,time);max_y=max_y.max(p.position_m.y_m);}
    assert!(max_y<1.0);near(p.position_m.y_m,0.);assert!(p.grounded);
    assert!(p.try_dash());step(&mut p,&w,1.,1500);near(p.position_m.y_m,0.);
}

#[test]
fn walk_and_dash_off_deck_release_contact_then_land_on_recovery_floor() {
    let w=world().with_standing_decks(vec![deck()]).unwrap();
    for dash in [false,true] {let mut p=body(6.6,2.);if dash {assert!(p.try_dash());}step(&mut p,&w,1.,0);assert!(!p.grounded);near(p.position_m.y_m,1.84);near(p.velocity_mps.y_m,-1.6);
        for time in (100..2000).step_by(100) {step(&mut p,&w,0.,time);}near(p.position_m.y_m,0.);assert!(p.grounded);
    }
}

#[test]
fn missed_dock_does_not_step_up_from_a_lower_moving_pose() {
    let w=world().with_moving_supports(vec![lift()]).unwrap().with_standing_decks(vec![deck()]).unwrap();let mut p=body(3.5,1.2);
    step(&mut p,&w,1.,2400);assert!(p.position_m.y_m<2.);assert!(p.position_m.y_m<=1.2);
}

#[test]
fn descending_lift_cannot_carry_player_through_static_deck_top() {
    let mut moving=lift();moving.polygon=vec![[1.,1.],[8.,1.],[8.,4.],[1.,4.]];let mut standing=deck();standing.height_m=1.;
    let w=world().with_moving_supports(vec![moving]).unwrap().with_standing_decks(vec![standing]).unwrap();let mut p=body(4.,1.2);
    step(&mut p,&w,0.,2400);near(p.position_m.y_m,1.);assert!(p.grounded);step(&mut p,&w,0.,2500);near(p.position_m.y_m,1.);
}

#[test]
fn static_contact_and_catalog_validation_rejects_malformed_or_duplicate_authority() {
    for height in [-0.1,3.1,f32::NAN,f32::INFINITY] {let mut d=deck();d.height_m=height;assert!(world().with_standing_decks(vec![d]).is_err());}
    let d=deck();assert!(world().with_standing_decks(vec![d.clone(),d.clone()]).is_err());
    let mut same_id=d.clone();same_id.id=lift().id;assert!(world().with_moving_supports(vec![lift()]).unwrap().with_standing_decks(vec![same_id.clone()]).is_err());assert!(world().with_standing_decks(vec![same_id]).unwrap().with_moving_supports(vec![lift()]).is_err());
    let w=world().with_standing_decks(vec![d]).unwrap();assert!(w.valid_saved_support_contact(&body(4.,2.),0));assert!(!w.valid_saved_support_contact(&body(4.,1.),0));assert!(!w.valid_saved_support_contact(&body(2.99,2.),0));
    let mut air=body(4.,1.);air.grounded=false;assert!(w.valid_saved_support_contact(&air,0));
}

#[test]
fn moving_only_fingerprint_is_unchanged_and_static_geometry_is_exactly_bound() {
    let moving=vec![lift()];let old_hash=format!("{:x}",Sha256::digest(serde_json::to_vec(&moving).unwrap()));let old=moving_support::frame(&moving,7,1500).unwrap();assert_eq!(old.catalog_sha256,old_hash);assert_eq!(old,moving_support::frame_with_standing(&moving,&[],7,1500).unwrap());
    let standing=vec![deck()];let value=moving_support::frame_with_standing(&moving,&standing,7,1500).unwrap();assert_ne!(value.catalog_sha256,old_hash);let stationary=value.poses.iter().find(|p|p.support_id=="z_deck").unwrap();assert_eq!(stationary.phase,SupportPhase::Stationary);near(stationary.velocity_mps,0.);assert_eq!(stationary.phase_elapsed_ms,0);moving_support::validate_saved_frame_shape(&value).unwrap();
    let mut changed=standing.clone();changed[0].polygon[0][0]=3.1;assert!(moving_support::validate_frame_with_standing(&value,&moving,&changed,7,1500).is_err());assert!(moving_support::validate_frame_with_standing(&value,&moving,&[],7,1500).is_err());
    let mut forged=value.clone();forged.poses.iter_mut().find(|p|p.phase==SupportPhase::Stationary).unwrap().phase_elapsed_ms=1;assert!(moving_support::validate_saved_frame_shape(&forged).is_err());
}

#[test]
fn explicit_height_ranges_are_finite_bounded_and_inclusive() {
    let band=HeightRange([1.9,2.1]);assert!(band.contains(1.9));assert!(band.contains(2.1));assert!(!band.contains(0.));assert!(!band.contains(f32::NAN));
    for band in [HeightRange([-0.1,1.]),HeightRange([0.,3.1]),HeightRange([2.,1.]),HeightRange([f32::NAN,2.])] {assert!(band.validate().is_err());}
    for raw in ["[]","[0]","[0,1,2]","{\"minM\":0,\"maxM\":1}"] {assert!(serde_json::from_str::<HeightRange>(raw).is_err());}
}

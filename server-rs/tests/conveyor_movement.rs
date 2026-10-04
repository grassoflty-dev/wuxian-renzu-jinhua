use wuxian_horror_ch1::{
    continuous_kcc::{step_kcc, step_kcc_with_surface_motion, Aabb, KccBody, KccError, KccTerrainRegion, StaticKccWorld},
    effects::{EffectLifetime, EffectResolver, EffectSource, EffectSourceKind, EffectSpec, StackRule, TerrainPenaltyEffect, TerrainTag},
    player_rules::{BoundaryTag, EffectivePlayerRules, MovementContext, MovementMode},
    world_v3::Vec3,
};

fn belt(id: &str, velocity: [f32; 2]) -> KccTerrainRegion {
    KccTerrainRegion { id: id.into(), tag: TerrainTag::Conveyor,
        polygon: vec![[2.0,2.0],[8.0,2.0],[8.0,8.0],[2.0,8.0]],
        surface_velocity_mps: Some(velocity) }
}
fn world(velocity: [f32;2]) -> StaticKccWorld {
    StaticKccWorld::new(Aabb::new(0.0,10.0,0.0,10.0).unwrap(),vec![])
        .with_terrain_regions(vec![belt("belt",velocity)])
}
fn body(x: f32, z: f32) -> KccBody { KccBody::new(Vec3::new(x,0.0,z).unwrap()) }
fn near(a: f32,b: f32) { assert!((a-b).abs()<0.0001,"{a} != {b}"); }

#[test]
fn conveyor_is_additive_velocity_for_idle_with_and_against_motion() {
    for (axis, expected) in [(0.0,-1.2),(1.0,2.8),(-1.0,-5.2)] {
        let mut player=body(5.0,5.0);
        step_kcc(&mut player,&world([-1.2,0.0]),(axis,0.0),0.1).unwrap();
        near(player.velocity_mps.x_m,expected);
        near(player.position_m.x_m,5.0+expected*0.1);
        near(player.position_m.z_m,5.0);
    }
}

#[test]
fn outside_or_airborne_bodies_receive_no_carried_velocity() {
    let mut outside=body(1.0,1.0);
    step_kcc(&mut outside,&world([1.2,0.0]),(0.0,0.0),0.1).unwrap();
    near(outside.position_m.x_m,1.0);
    let mut airborne=body(5.0,5.0); airborne.grounded=false; airborne.position_m.y_m=1.0;
    step_kcc(&mut airborne,&world([1.2,0.0]),(0.0,0.0),0.1).unwrap();
    near(airborne.position_m.x_m,5.0);
}

#[test]
fn effect_resolver_ignores_only_surface_penalty_and_cannot_unlock_boundaries() {
    let source=EffectSource { source_kind:EffectSourceKind::Equipment,source_id:"test.hover".into(),
        instance_id:"test.hover:1".into(),lifetime:EffectLifetime::Equipped,ui_category:None,
        effects:vec![EffectSpec::TerrainPenaltyIgnore(TerrainPenaltyEffect {tag:TerrainTag::Conveyor,stack:StackRule::Any})] };
    let rules=EffectResolver::resolve(&[source]).unwrap();
    let supported=world([1.2,0.0]).with_movement_rules(rules.clone(),MovementMode::Hover);
    let mut player=body(5.0,5.0);
    step_kcc(&mut player,&supported,(0.0,0.0),0.1).unwrap(); near(player.position_m.x_m,5.0);
    step_kcc(&mut player,&supported,(1.0,0.0),0.1).unwrap(); near(player.position_m.x_m,5.4);
    for boundary in [BoundaryTag::World,BoundaryTag::Quest] {
        assert!(!rules.resolve_movement(MovementMode::Hover,MovementContext {boundary:Some(boundary),..Default::default()}).allowed);
    }
    assert!(!supported.can_occupy(Vec3::new(11.0,0.0,5.0).unwrap(),0.35));
    // Merely selecting a future mode is not an equipment/capability grant.
    let no_effect=world([1.2,0.0]).with_movement_rules(EffectivePlayerRules::default(),MovementMode::Hover);
    let mut player=body(5.0,5.0);step_kcc(&mut player,&no_effect,(0.0,0.0),0.1).unwrap();near(player.position_m.x_m,5.12);
}

#[test]
fn belt_plus_dash_is_swept_against_thin_walls_and_world_boundaries() {
    let wall=StaticKccWorld::new(Aabb::new(0.0,10.0,0.0,10.0).unwrap(),vec![Aabb::new(2.75,2.80,1.0,9.0).unwrap()])
        .with_terrain_regions(vec![belt("fast",[4.0,0.0])]);
    let mut player=body(2.0,5.0);assert!(player.try_dash());
    let events=step_kcc(&mut player,&wall,(1.0,0.0),0.1).unwrap();
    assert!(!events.is_empty());near(player.position_m.x_m,2.0);near(player.velocity_mps.x_m,0.0);
    let mut edge=belt("edge",[4.0,0.0]);edge.polygon=vec![[8.0,2.0],[10.0,2.0],[10.0,8.0],[8.0,8.0]];
    let boundary=StaticKccWorld::new(Aabb::new(0.0,10.0,0.0,10.0).unwrap(),vec![]).with_terrain_regions(vec![edge]);
    let mut player=body(9.6,5.0);step_kcc(&mut player,&boundary,(0.0,0.0),0.1).unwrap();near(player.position_m.x_m,9.6);
}

#[test]
fn malformed_motion_rejects_before_touching_body_or_dash_timers() {
    for velocity in [Some([f32::NAN,0.0]),Some([f32::INFINITY,0.0]),Some([0.0,0.0]),Some([4.1,0.0]),None] {
        let mut region=belt("invalid",[1.0,0.0]);region.surface_velocity_mps=velocity;
        let world=StaticKccWorld::new(Aabb::new(0.0,10.0,0.0,10.0).unwrap(),vec![]).with_terrain_regions(vec![region]);
        let mut player=body(5.0,5.0);assert!(player.try_dash());let before=player.clone();
        assert_eq!(step_kcc(&mut player,&world,(0.0,0.0),0.1),Err(KccError::InvalidWorld));assert_eq!(player,before);
    }
}

#[test]
fn overlapping_support_is_stable_id_order_never_added_into_speed_boost() {
    let a=belt("a",[2.0,0.0]);let b=belt("b",[-3.0,0.0]);
    for regions in [vec![a.clone(),b.clone()],vec![b,a]] {
        let world=StaticKccWorld::new(Aabb::new(0.0,10.0,0.0,10.0).unwrap(),vec![]).with_terrain_regions(regions);
        let mut player=body(5.0,5.0);step_kcc(&mut player,&world,(0.0,0.0),0.1).unwrap();near(player.position_m.x_m,5.2);
    }
}

#[test]
fn authored_traversal_can_hold_exact_path_and_belt_resumes_after_release() {
    let mut player=body(5.0,5.0);
    for _ in 0..7 {step_kcc_with_surface_motion(&mut player,&world([1.2,0.0]),(0.0,0.0),0.05,false).unwrap();}
    near(player.position_m.x_m,5.0);
    step_kcc(&mut player,&world([1.2,0.0]),(0.0,0.0),0.05).unwrap();near(player.position_m.x_m,5.06);
}

#[test]
fn steady_conveyor_transport_is_fixed_step_deterministic() {
    let mut left=body(5.0,5.0);let mut right=left.clone();let world=world([1.2,0.0]);
    for _ in 0..10 {step_kcc(&mut left,&world,(0.0,0.0),0.05).unwrap();}
    for _ in 0..30 {step_kcc(&mut right,&world,(0.0,0.0),1.0/60.0).unwrap();}
    near(left.position_m.x_m,right.position_m.x_m);near(left.position_m.x_m,5.6);
}

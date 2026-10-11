use wuxian_horror_ch1::continuous_kcc::{
    KccError, KccState, MotionInput, StaticKccWorld, CAPSULE_RADIUS_M, COLLISION_OFFSET_M,
};
use wuxian_horror_ch1::world_v3::{Transform, Vec3};

fn state(x_m: f32, foot_y_m: f32, z_m: f32, velocity_y_mps: f32) -> KccState {
    KccState::new(
        Transform::new(Vec3::new(x_m, foot_y_m, z_m).unwrap(), 0.0).unwrap(),
        Vec3::new(0.0, velocity_y_mps, 0.0).unwrap(),
    )
}

fn motion(translation: [f32; 3], gravity_mps2: f32, dt_s: f32) -> MotionInput {
    MotionInput::new(translation, gravity_mps2, dt_s).unwrap()
}

fn assert_clear(world: &StaticKccWorld, step: &wuxian_horror_ch1::continuous_kcc::KccStep) {
    assert!(!step.penetration, "KCC result penetrates static geometry");
    assert!(
        !world.intersects(&step.state),
        "KCC state intersects static geometry"
    );
}

#[test]
fn ag06_grounded_floor_has_explicit_contact_evidence() {
    let world = StaticKccWorld::room_10m();
    let step = world
        .move_foot(
            &state(0.0, COLLISION_OFFSET_M, 0.0, -0.20),
            motion([0.0; 3], 0.0, 1.0 / 60.0),
        )
        .unwrap();
    assert!(step.contact_grounded);
    assert!(step.grounded);
    assert!(step.foot_distance_m.unwrap() <= 0.02);
    assert!(step.ground_normal.unwrap()[1] >= 0.90);
    assert_clear(&world, &step);
}

#[test]
fn ag06_frontal_wall_stops_at_capsule_boundary() {
    let mut world = StaticKccWorld::room_10m();
    world.add_box([2.0, 1.0, 0.0], [0.10, 1.0, 4.0]).unwrap();

    let step = world
        .move_foot(
            &state(0.0, 0.0, 0.0, 0.0),
            motion([4.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(step.state.foot_transform.position_m().x_m() <= 1.58);
    assert_clear(&world, &step);
}

#[test]
fn ag06_diagonal_wall_slide_preserves_forward_component() {
    let mut world = StaticKccWorld::room_10m();
    world.add_box([2.0, 1.0, 0.0], [0.10, 1.0, 4.0]).unwrap();

    let step = world
        .move_foot(
            &state(0.0, 0.0, 0.0, 0.0),
            motion([4.0, 0.0, 3.0], 0.0, 1.0),
        )
        .unwrap();
    let position = step.state.foot_transform.position_m();

    assert!(position.x_m() <= 1.58);
    assert!(position.z_m() > 2.0);
    assert_clear(&world, &step);
}

#[test]
fn ag06_doorway_passes_and_frame_blocks() {
    let mut world = StaticKccWorld::room_10m();
    world.add_box([0.0, 1.5, -3.0], [0.10, 1.5, 2.0]).unwrap();
    world.add_box([0.0, 1.5, 3.0], [0.10, 1.5, 2.0]).unwrap();

    let pass = world
        .move_foot(
            &state(-3.0, 0.0, 0.0, 0.0),
            motion([6.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(pass.state.foot_transform.position_m().x_m() > 2.0);
    assert_clear(&world, &pass);

    let blocked = world
        .move_foot(
            &state(-3.0, 0.0, 1.5, 0.0),
            motion([6.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(blocked.state.foot_transform.position_m().x_m() < -0.40);
    assert_clear(&world, &blocked);
}

#[test]
fn ag06_small_step_autosteps_and_large_step_blocks() {
    let mut small = StaticKccWorld::room_10m();
    small.add_box([1.5, 0.15, 0.0], [0.30, 0.15, 1.5]).unwrap();
    let small_step = small
        .move_foot(
            &state(0.0, 0.0, 0.0, 0.0),
            motion([3.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(small_step.state.foot_transform.position_m().x_m() > 2.0);
    assert!(small_step.state.foot_transform.position_m().y_m() > 0.20);
    assert_clear(&small, &small_step);

    let mut large = StaticKccWorld::room_10m();
    large.add_box([1.5, 0.40, 0.0], [0.30, 0.40, 1.5]).unwrap();
    let large_step = large
        .move_foot(
            &state(0.0, 0.0, 0.0, 0.0),
            motion([3.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(large_step.state.foot_transform.position_m().x_m() < 1.20);
    assert_clear(&large, &large_step);
}

fn slope_vertices(angle_degrees: f32) -> Vec<[f32; 3]> {
    let rise_m = (angle_degrees.to_radians().tan() * 4.0).min(5.0);
    vec![
        [0.0, 0.0, -2.0],
        [4.0, 0.0, -2.0],
        [4.0, 0.0, 2.0],
        [0.0, 0.0, 2.0],
        [0.0, 0.0, -2.0],
        [4.0, rise_m, -2.0],
        [4.0, rise_m, 2.0],
        [0.0, 0.0, 2.0],
    ]
}

#[test]
fn ag06_walkable_twenty_degree_slope_climbs() {
    let mut world = StaticKccWorld::room_10m();
    world.add_convex_hull(&slope_vertices(20.0)).unwrap();
    let step = world
        .move_foot(
            &state(-1.5, 0.0, 0.0, 0.0),
            motion([2.8, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(step.state.foot_transform.position_m().x_m() > 0.8);
    assert!(step.ground_normal.unwrap()[1] >= 0.70);
    assert_clear(&world, &step);
}

#[test]
fn ag06_fifty_degree_slope_is_not_walkable() {
    let mut world = StaticKccWorld::room_10m();
    world.add_convex_hull(&slope_vertices(50.0)).unwrap();
    let step = world
        .move_foot(
            &state(-1.5, 0.0, 0.0, 0.0),
            motion([2.8, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(step.state.foot_transform.position_m().x_m() < 0.60);
    assert!(step.kcc_grounded);
    assert!(!step.grounded);
    assert!(step
        .collision_normals
        .iter()
        .all(|normal| normal[1] <= 0.70));
    assert_clear(&world, &step);
}

#[test]
fn ag06_jump_hits_head_and_lands_without_penetration() {
    let mut world = StaticKccWorld::room_10m();
    world.add_box([0.0, 2.7, 0.0], [4.0, 0.10, 4.0]).unwrap();
    let mut current = state(0.0, 0.0, 0.0, 5.5);
    let mut apex: f32 = 0.0;
    let mut head_collision = false;
    let mut landed = false;

    for _ in 0..120 {
        let step = world
            .move_foot(&current, motion([0.0; 3], 9.81, 1.0 / 60.0))
            .unwrap();
        apex = apex.max(step.state.foot_transform.position_m().y_m());
        head_collision |= step.head_collision;
        landed |= step.landed;
        assert_clear(&world, &step);
        current = step.state;
    }

    assert!(apex > 0.35);
    assert!(head_collision);
    assert!(landed);
}

#[test]
fn ag06_dash_clear_reaches_target() {
    let world = StaticKccWorld::room_10m();
    let step = world
        .move_foot(
            &state(-3.0, 0.0, 0.0, 0.0),
            motion([5.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(step.state.foot_transform.position_m().x_m() >= 1.9);
    assert_clear(&world, &step);
}

#[test]
fn ag06_dash_wall_sweep_stops_before_wall() {
    let mut world = StaticKccWorld::room_10m();
    world.add_box([0.0, 1.5, 0.0], [0.10, 1.5, 4.0]).unwrap();
    let step = world
        .move_foot(
            &state(-3.0, 0.0, 0.0, 0.0),
            motion([6.0, 0.0, 0.0], 0.0, 1.0),
        )
        .unwrap();
    assert!(step.state.foot_transform.position_m().x_m() <= -0.46);
    assert_clear(&world, &step);
}

#[test]
fn ag06_fixed_input_is_stable_at_thirty_and_sixty_hz() {
    fn run(dt_s: f32) -> f32 {
        let world = StaticKccWorld::room_10m();
        let mut current = state(-3.0, 0.0, 0.0, 0.0);
        let frames = (1.0 / dt_s).round() as usize;
        for _ in 0..frames {
            current = world
                .move_foot(&current, motion([2.0 * dt_s, 0.0, 0.0], 0.0, dt_s))
                .unwrap()
                .state;
        }
        current.foot_transform.position_m().x_m()
    }

    assert!((run(1.0 / 30.0) - run(1.0 / 60.0)).abs() <= 0.02);
}

#[test]
fn foot_anchor_round_trips_without_center_offset_drift() {
    let world = StaticKccWorld::room_10m();
    let current = state(0.5, 0.37, -0.5, 0.0);
    let step = world
        .move_foot(&current, MotionInput::stationary(1.0 / 60.0).unwrap())
        .unwrap();
    assert!((step.state.foot_transform.position_m().y_m() - 0.37).abs() <= 0.02);
}

#[test]
fn invalid_input_and_geometry_are_rejected() {
    assert!(matches!(
        MotionInput::new([f32::NAN, 0.0, 0.0], 9.81, 1.0 / 60.0),
        Err(KccError::NonFinite("desiredTranslationM"))
    ));
    assert!(matches!(
        MotionInput::new([0.0; 3], 9.81, 0.0),
        Err(KccError::InvalidDt)
    ));
    assert!(matches!(
        MotionInput::new([0.0; 3], -1.0, 1.0 / 60.0),
        Err(KccError::InvalidGravity)
    ));

    let mut world = StaticKccWorld::new();
    assert!(matches!(
        world.add_box([0.0, 0.0, 0.0], [0.0, 1.0, 1.0]),
        Err(KccError::InvalidGeometry)
    ));
    assert!(matches!(
        world.add_box([f32::NAN, 0.0, 0.0], [1.0, 1.0, 1.0]),
        Err(KccError::NonFinite("centerM"))
    ));
    assert!(world
        .add_convex_hull(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
        .is_err());
}

#[test]
fn capsule_dimensions_match_ag06_contract() {
    assert!((CAPSULE_RADIUS_M * 2.0 - 0.70).abs() <= f32::EPSILON);
    assert!((CAPSULE_RADIUS_M * 2.0 + 1.50 - 2.20).abs() <= f32::EPSILON);
}

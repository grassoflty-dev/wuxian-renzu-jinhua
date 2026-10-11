use wuxian_horror_ch1::grey_hive_control_collider::{
    ControlColliderSlice, CONTROL_CHANNEL_BASE_LENGTH_M, CONTROL_CHANNEL_MAX_STEP_M,
    CONTROL_CHANNEL_V7_LENGTH_M, CONTROL_CHANNEL_WIDTH_M, V7_WAYPOINTS,
};

fn path_length(path: &[[f32; 3]]) -> f32 {
    path.windows(2)
        .map(|pair| {
            let dx = pair[1][0] - pair[0][0];
            let dy = pair[1][1] - pair[0][1];
            let dz = pair[1][2] - pair[0][2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

#[test]
fn v7_control_channel_contract_is_40m_and_sampled() {
    assert!((path_length(V7_WAYPOINTS) - CONTROL_CHANNEL_V7_LENGTH_M).abs() <= 0.001);
    assert!((CONTROL_CHANNEL_V7_LENGTH_M - CONTROL_CHANNEL_BASE_LENGTH_M - 12.0).abs() <= 0.001);
    assert!((CONTROL_CHANNEL_WIDTH_M - 3.5).abs() <= f32::EPSILON);
    assert!(CONTROL_CHANNEL_MAX_STEP_M <= 0.5);
    assert!(V7_WAYPOINTS.iter().all(|point| point[1] == 0.0));
}

#[test]
fn open_channel_runs_forward_and_reverse_without_penetration() {
    let slice = ControlColliderSlice::open().unwrap();
    assert!(slice.gate_b_open());

    let forward = slice.run_forward().unwrap();
    assert!(forward.reached_goal, "forward final state: {forward:?}");
    assert!(
        !forward.blocked,
        "forward should not be blocked: {forward:?}"
    );
    assert!(!forward.penetration, "forward penetrated: {forward:?}");
    assert!((forward.requested_distance_m - 40.0).abs() <= 0.001);
    assert!((forward.actual_distance_m - 40.0).abs() <= 0.08);

    let reverse = slice.run_reverse().unwrap();
    assert!(reverse.reached_goal, "reverse final state: {reverse:?}");
    assert!(
        !reverse.blocked,
        "reverse should not be blocked: {reverse:?}"
    );
    assert!(!reverse.penetration, "reverse penetrated: {reverse:?}");
    assert!((reverse.requested_distance_m - 40.0).abs() <= 0.001);
    assert!((reverse.actual_distance_m - 40.0).abs() <= 0.08);
}

#[test]
fn closed_door_blocks_b02_approach_without_penetration() {
    let slice = ControlColliderSlice::closed().unwrap();
    assert!(!slice.gate_b_open());
    let forward = slice.run_forward().unwrap();
    assert!(forward.blocked, "closed door did not block: {forward:?}");
    assert!(
        !forward.reached_goal,
        "closed door reached B02: {forward:?}"
    );
    assert!(!forward.penetration, "closed door penetrated: {forward:?}");
    assert!(forward.actual_distance_m < forward.requested_distance_m - 0.05);
}

#[test]
fn shortest_open_probe_rejects_old_direct_edge_and_reports_cut_candidates() {
    let measurement = ControlColliderSlice::shortest_open_path().unwrap();
    assert!((measurement.centerline_m - 40.0).abs() <= 0.001);
    let shortest = measurement
        .shortest_complete_m
        .expect("at least one open candidate must reach B02");
    assert!(
        shortest >= 39.0,
        "real collider cut shortened v7 too far: {measurement:?}"
    );
    assert!(shortest <= 40.0 + 0.08);
    assert_eq!(measurement.candidates.len(), 5);
    assert!(measurement.candidates.last().unwrap().blocked);
    assert!(measurement.candidates.last().unwrap().penetration == false);
}

#[test]
fn open_slice_has_no_unexpected_penetration_in_candidate_runs() {
    let measurement = ControlColliderSlice::shortest_open_path().unwrap();
    assert!(measurement.candidates.iter().all(|run| !run.penetration));
    assert!(measurement.candidates.iter().any(|run| run.reached_goal));
}

use wuxian_horror_ch1::world_progression::*;
use wuxian_horror_ch1::world_v3::WorldRevision;

fn revision(n: u64) -> WorldRevision {
    WorldRevision::new(1, n, n).expect("valid revision")
}

fn assert_rejected(result: RouteResult, expected: RouteRejectCode) {
    match result {
        RouteResult::Rejected { code, .. } => assert_eq!(code, expected),
        RouteResult::Applied { .. } => panic!("expected rejection"),
    }
}

fn apply(state: &mut RouteState, command: RouteCommand, n: &mut u64) -> Vec<RouteEvent> {
    *n += 1;
    match apply_route_command(state, command, revision(*n)) {
        RouteResult::Applied { events, .. } => events,
        RouteResult::Rejected { code, message } => panic!("unexpected {code:?}: {message}"),
    }
}

fn complete_world(state: &mut RouteState, world: &str, events: &[&str], seq: &mut u64) {
    apply(state, RouteCommand::Enter { world_id: world.into(), request_id: format!("enter-{world}-{seq}") }, seq);
    for event in events {
        apply(state, RouteCommand::Progress { event_id: (*event).into(), request_id: format!("progress-{event}-{seq}") }, seq);
    }
    apply(state, RouteCommand::Complete { world_id: world.into(), request_id: format!("complete-{world}-{seq}") }, seq);
}

#[test]
fn rejects_unknown_skip_and_uncleared_revisit_without_mutation() {
    let mut state = RouteState::new();
    let before = serde_json::to_string(&state).unwrap();
    assert_rejected(apply_route_command(&mut state, RouteCommand::Enter { world_id: "unknown".into(), request_id: "unknown-1".into() }, revision(1)), RouteRejectCode::UnknownWorld);
    assert_eq!(serde_json::to_string(&state).unwrap(), before);
    assert_rejected(apply_route_command(&mut state, RouteCommand::Enter { world_id: WORLD_MIST_HARBOR.into(), request_id: "skip-1".into() }, revision(2)), RouteRejectCode::InvalidWorldOrder);
    assert_rejected(apply_route_command(&mut state, RouteCommand::Revisit { world_id: WORLD_GREY_HIVE.into(), request_id: "revisit-early".into() }, revision(3)), RouteRejectCode::FirstClearRequired);
    assert_rejected(apply_route_command(&mut state, RouteCommand::Complete { world_id: WORLD_GREY_HIVE.into(), request_id: "complete-early".into() }, revision(4)), RouteRejectCode::RequiredProgressMissing);
}

#[test]
fn normal_three_world_progression_and_clockworks_tasks() {
    let mut state = RouteState::new();
    let mut seq = 0;
    complete_world(&mut state, WORLD_GREY_HIVE, &["hive_power", "hive_lockdown", "hive_extraction"], &mut seq);
    complete_world(&mut state, WORLD_MIST_HARBOR, &["mist_beacon_west", "mist_beacon_east", "mist_signal"], &mut seq);
    complete_world(&mut state, WORLD_CLOCKWORKS, &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], &mut seq);
    assert!(state.progress.iter().all(|p| p.completed));
    let tasks = post_clear_task_candidates(&state, WORLD_CLOCKWORKS).unwrap();
    assert!(tasks.len() >= 3);
    assert_eq!(migrate_world_id("clockwork_city").unwrap(), WORLD_CLOCKWORKS);
    assert_rejected(apply_route_command(&mut state, RouteCommand::Enter { world_id: "clockwork_unknown".into(), request_id: "bad".into() }, revision(999)), RouteRejectCode::UnknownWorld);
}

#[test]
fn completion_reward_and_save_roundtrip_are_idempotent() {
    let mut state = RouteState::new();
    let mut seq = 0;
    complete_world(&mut state, WORLD_GREY_HIVE, &["hive_power", "hive_lockdown", "hive_extraction"], &mut seq);
    let completion_request = "complete-grey_hive-4".to_string();
    let events = apply(&mut state, RouteCommand::Complete { world_id: WORLD_GREY_HIVE.into(), request_id: completion_request }, &mut seq);
    assert!(matches!(events.as_slice(), [RouteEvent::DuplicateIgnored { .. }]));

    let reward = RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "grey-first-clear".into(), policy: RewardPolicy::FirstClear, request_id: "reward-1".into(), visit_id: None, cycle_id: None };
    apply(&mut state, reward.clone(), &mut seq);
    let ledger_len = state.reward_ledger.len();
    let duplicate = apply(&mut state, reward, &mut seq);
    assert!(matches!(duplicate.as_slice(), [RouteEvent::DuplicateIgnored { .. }]));
    assert_eq!(state.reward_ledger.len(), ledger_len);

    let json = serde_json::to_string(&state).unwrap();
    let mut restored: RouteState = serde_json::from_str(&json).unwrap();
    let duplicate_after_restore = apply(&mut restored, RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "grey-first-clear".into(), policy: RewardPolicy::FirstClear, request_id: "reward-1".into(), visit_id: None, cycle_id: None }, &mut seq);
    assert!(matches!(duplicate_after_restore.as_slice(), [RouteEvent::DuplicateIgnored { .. }]));
    assert_eq!(restored.reward_ledger.len(), ledger_len);
}

#[test]
fn all_reward_classes_enforce_real_transaction_keys() {
    let mut state = RouteState::new();
    let mut seq = 0;
    complete_world(&mut state, WORLD_GREY_HIVE, &["hive_power", "hive_lockdown", "hive_extraction"], &mut seq);
    for command in [
        RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "first".into(), policy: RewardPolicy::FirstClear, request_id: "rf".into(), visit_id: None, cycle_id: None },
        RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "once".into(), policy: RewardPolicy::OneTime, request_id: "ro".into(), visit_id: None, cycle_id: None },
        RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "visit".into(), policy: RewardPolicy::Revisit, request_id: "rv".into(), visit_id: Some(1), cycle_id: None },
        RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "cycle".into(), policy: RewardPolicy::Renewable, request_id: "rr".into(), visit_id: None, cycle_id: Some(1) },
    ] { apply(&mut state, command, &mut seq); }
    assert_eq!(state.reward_ledger.len(), 4);
    assert_rejected(apply_route_command(&mut state, RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "visit".into(), policy: RewardPolicy::Revisit, request_id: "rv-new".into(), visit_id: Some(1), cycle_id: None }, revision(100)), RouteRejectCode::RewardAlreadyGranted);
    assert_rejected(apply_route_command(&mut state, RouteCommand::GrantReward { world_id: WORLD_GREY_HIVE.into(), reward_id: "bad".into(), policy: RewardPolicy::Renewable, request_id: "bad-key".into(), visit_id: None, cycle_id: None }, revision(101)), RouteRejectCode::RewardKeyMalformed);
}

#[test]
fn revisit_preserves_first_results_and_gate_b_ignores_choice() {
    for choice in [HiveChoice::Taken, HiveChoice::Left, HiveChoice::Unresolved] {
        let mut state = RouteState::new();
        let mut seq = 0;
        apply(&mut state, RouteCommand::Progress { event_id: "hive_power".into(), request_id: format!("power-{choice:?}") }, &mut seq);
        apply(&mut state, RouteCommand::Progress { event_id: "hive_lockdown".into(), request_id: format!("lock-{choice:?}") }, &mut seq);
        assert!(gate_b_ready(&state));
        if choice != HiveChoice::Unresolved { assert!(state.set_hive_choice_once(choice)); }
        apply(&mut state, RouteCommand::Progress { event_id: "hive_extraction".into(), request_id: format!("exit-{seq}") }, &mut seq);
        apply(&mut state, RouteCommand::Complete { world_id: WORLD_GREY_HIVE.into(), request_id: format!("complete-{seq}") }, &mut seq);
        assert!(state.set_first_mainline_result(WORLD_GREY_HIVE, "first-result"));
        assert!(state.record_sentinel_first_kill(WORLD_GREY_HIVE));
        apply(&mut state, RouteCommand::Revisit { world_id: WORLD_GREY_HIVE.into(), request_id: format!("revisit-{seq}") }, &mut seq);
        assert!(!state.set_first_mainline_result(WORLD_GREY_HIVE, "rewrite"));
        assert!(!state.record_sentinel_first_kill(WORLD_GREY_HIVE));
        assert_eq!(state.progress[0].first_mainline_result.as_deref(), Some("first-result"));
        assert!(state.progress[0].sentinel_first_kill_recorded);
        assert!(gate_b_ready(&state));
    }
}

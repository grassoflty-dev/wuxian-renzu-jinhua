//! Freeze v0.2 authoritative route progression and reward ledger.
//!
//! This module owns no shared wire DTOs. It consumes the projections and
//! revision identity owned by `world_v3` and is composed later by
//! `formal_runtime`.

use crate::world_v3::{RouteProjection, WorldProgressProjection, WorldRevision};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const ROUTE_SCHEMA_VERSION: u32 = 1;
pub const WORLD_GREY_HIVE: &str = "grey_hive";
pub const WORLD_MIST_HARBOR: &str = "mist_harbor";
pub const WORLD_CLOCKWORKS: &str = "clockworks";
pub const GATE_B_EVENT: &str = "hive_lockdown";

const CATALOG_JSON: &str = include_str!("../data/world_progression_v1.json");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RevisitPolicy {
    Never,
    AfterFirstClearLimited { max_revisits: u32 },
    TaskCandidatesLimited { max_revisits: u32 },
    RenewableByCycle { max_revisits_per_cycle: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RewardPolicy {
    FirstClear,
    OneTime,
    Revisit,
    Renewable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HiveChoice {
    Taken,
    Left,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldProgress {
    pub world_id: String,
    pub completed: bool,
    pub first_completion: bool,
    pub visit_id: u64,
    pub cycle_id: u64,
    pub revisit_count: u32,
    pub completed_events: Vec<String>,
    pub first_mainline_result: Option<String>,
    pub hive_choice: HiveChoice,
    pub sentinel_first_kill_recorded: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardLedgerEntry {
    pub world_id: String,
    pub reward_id: String,
    pub policy: RewardPolicy,
    pub request_id: String,
    pub visit_id: Option<u64>,
    pub cycle_id: Option<u64>,
    pub granted_at_revision: WorldRevision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteState {
    pub schema_version: u32,
    pub current_world_id: String,
    pub progress: Vec<WorldProgress>,
    pub reward_ledger: Vec<RewardLedgerEntry>,
    pub event_seq: u64,
    #[serde(default)]
    processed_request_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteCommand {
    Enter { world_id: String, request_id: String },
    Progress { event_id: String, request_id: String },
    Complete { world_id: String, request_id: String },
    Revisit { world_id: String, request_id: String },
    /// Leave the current world back to the hub without completing it.
    /// Human decision 2026-11-10: walking back out of the Grey Hive abandons
    /// the run instead of silently keeping it open.
    Abandon { world_id: String, request_id: String },
    GrantReward {
        world_id: String,
        reward_id: String,
        policy: RewardPolicy,
        request_id: String,
        visit_id: Option<u64>,
        cycle_id: Option<u64>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteEvent {
    Entered { world_id: String, visit_id: u64 },
    Progressed { world_id: String, event_id: String },
    Completed { world_id: String, first_completion: bool },
    Revisited { world_id: String, visit_id: u64, revisit_count: u32 },
    Abandoned { world_id: String },
    RewardGranted { world_id: String, reward_id: String },
    DuplicateIgnored { request_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteRejectCode {
    EmptyId,
    UnknownWorld,
    InvalidWorldOrder,
    WorldNotCurrent,
    RequiredProgressMissing,
    FirstClearRequired,
    RevisitLimitReached,
    RewardKeyMalformed,
    RewardAlreadyGranted,
    RunNotAbandonable,
    InvalidCatalog,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RouteResult {
    Applied { events: Vec<RouteEvent>, projection: RouteProjection },
    Rejected { code: RouteRejectCode, message: String },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Catalog {
    schema_version: u32,
    worlds: Vec<WorldDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldDefinition {
    world_id: String,
    name: String,
    order: usize,
    revisit_policy: RevisitPolicy,
    required_events: Vec<String>,
    post_clear_task_candidates: Vec<String>,
}

impl RouteState {
    pub fn new() -> Self {
        let progress = [WORLD_GREY_HIVE, WORLD_MIST_HARBOR, WORLD_CLOCKWORKS]
            .into_iter()
            .map(|world_id| WorldProgress {
                world_id: world_id.to_string(),
                completed: false,
                first_completion: false,
                visit_id: 0,
                cycle_id: 1,
                revisit_count: 0,
                completed_events: Vec::new(),
                first_mainline_result: None,
                hive_choice: HiveChoice::Unresolved,
                sentinel_first_kill_recorded: false,
            })
            .collect();
        Self {
            schema_version: ROUTE_SCHEMA_VERSION,
            current_world_id: WORLD_GREY_HIVE.to_string(),
            progress,
            reward_ledger: Vec::new(),
            event_seq: 0,
            processed_request_ids: Vec::new(),
        }
    }

    pub fn set_first_mainline_result(&mut self, world_id: &str, result: &str) -> bool {
        if result.trim().is_empty() { return false; }
        let Some(progress) = self.progress.iter_mut().find(|p| p.world_id == world_id) else { return false; };
        if progress.first_mainline_result.is_some() { return false; }
        progress.first_mainline_result = Some(result.to_string());
        true
    }

    pub fn set_hive_choice_once(&mut self, choice: HiveChoice) -> bool {
        let progress = self.progress.iter_mut().find(|p| p.world_id == WORLD_GREY_HIVE).expect("frozen world exists");
        if progress.hive_choice != HiveChoice::Unresolved { return false; }
        progress.hive_choice = choice;
        true
    }

    pub fn record_sentinel_first_kill(&mut self, world_id: &str) -> bool {
        let Some(progress) = self.progress.iter_mut().find(|p| p.world_id == world_id) else { return false; };
        if progress.sentinel_first_kill_recorded { return false; }
        progress.sentinel_first_kill_recorded = true;
        true
    }
}

impl Default for RouteState {
    fn default() -> Self { Self::new() }
}

pub fn migrate_world_id(id: &str) -> Result<&str, RouteRejectCode> {
    match id {
        WORLD_GREY_HIVE | WORLD_MIST_HARBOR | WORLD_CLOCKWORKS => Ok(id),
        "clockwork_city" => Ok(WORLD_CLOCKWORKS),
        _ => Err(RouteRejectCode::UnknownWorld),
    }
}

pub fn gate_b_ready(state: &RouteState) -> bool {
    state.progress.iter().find(|p| p.world_id == WORLD_GREY_HIVE)
        .is_some_and(|p| p.completed_events.iter().any(|event| event == GATE_B_EVENT))
}

pub fn post_clear_task_candidates(state: &RouteState, world_id: &str) -> Result<Vec<String>, RouteRejectCode> {
    let id = migrate_world_id(world_id)?;
    let progress = state.progress.iter().find(|p| p.world_id == id).ok_or(RouteRejectCode::UnknownWorld)?;
    if !progress.completed { return Err(RouteRejectCode::FirstClearRequired); }
    let catalog = catalog().map_err(|_| RouteRejectCode::InvalidCatalog)?;
    Ok(catalog.worlds.into_iter().find(|w| w.world_id == id).expect("validated catalog").post_clear_task_candidates)
}

pub fn can_revisit_world(state: &RouteState, world_id: &str) -> bool {
    let Ok(world_id) = migrate_world_id(world_id) else {
        return false;
    };
    let Some(progress) = state.progress.iter().find(|p| p.world_id == world_id) else {
        return false;
    };
    if !progress.completed {
        return false;
    }
    let Ok(catalog) = catalog() else {
        return false;
    };
    let Some(definition) = catalog.worlds.iter().find(|w| w.world_id == world_id) else {
        return false;
    };
    let limit = match definition.revisit_policy {
        RevisitPolicy::Never => 0,
        RevisitPolicy::AfterFirstClearLimited { max_revisits }
        | RevisitPolicy::TaskCandidatesLimited { max_revisits } => max_revisits,
        RevisitPolicy::RenewableByCycle { max_revisits_per_cycle } => max_revisits_per_cycle,
    };
    progress.revisit_count < limit
}

pub fn apply_route_command(state: &mut RouteState, command: RouteCommand, revision: WorldRevision) -> RouteResult {
    let request_id = match &command {
        RouteCommand::Enter { request_id, .. } | RouteCommand::Progress { request_id, .. }
        | RouteCommand::Complete { request_id, .. } | RouteCommand::Revisit { request_id, .. }
        | RouteCommand::Abandon { request_id, .. }
        | RouteCommand::GrantReward { request_id, .. } => request_id,
    }.clone();
    if request_id.trim().is_empty() { return rejected(RouteRejectCode::EmptyId, "requestId is required"); }
    if state.processed_request_ids.iter().any(|id| id == &request_id) {
        return applied(state, vec![RouteEvent::DuplicateIgnored { request_id }]);
    }
    let catalog = match catalog() { Ok(c) => c, Err(message) => return rejected(RouteRejectCode::InvalidCatalog, &message) };
    let outcome = match command {
        RouteCommand::Enter { world_id, .. } => enter(state, &catalog, &world_id),
        RouteCommand::Progress { event_id, .. } => progress(state, &catalog, &event_id),
        RouteCommand::Complete { world_id, .. } => complete(state, &catalog, &world_id),
        RouteCommand::Revisit { world_id, .. } => revisit(state, &catalog, &world_id),
        RouteCommand::Abandon { world_id, .. } => abandon(state, &catalog, &world_id),
        RouteCommand::GrantReward { world_id, reward_id, policy, request_id, visit_id, cycle_id } => {
            grant_reward(state, &world_id, &reward_id, policy, request_id, visit_id, cycle_id, revision)
        }
    };
    match outcome {
        Ok(events) => {
            state.processed_request_ids.push(request_id);
            state.event_seq = state.event_seq.saturating_add(1);
            applied(state, events)
        }
        Err((code, message)) => rejected(code, &message),
    }
}

pub fn project_route(state: &RouteState) -> RouteProjection {
    RouteProjection {
        schema_version: ROUTE_SCHEMA_VERSION,
        current_world_id: state.current_world_id.clone(),
        event_seq: state.event_seq,
        worlds: state.progress.iter().map(|p| WorldProgressProjection {
            world_id: p.world_id.clone(), completed: p.completed, first_completion: p.first_completion,
            visit_id: p.visit_id, cycle_id: p.cycle_id, revisit_count: p.revisit_count,
            completed_events: p.completed_events.clone(),
            hive_choice: if p.world_id == WORLD_GREY_HIVE { Some(p.hive_choice.clone()) } else { None },
            pressure_valve_receipts: None,
        }).collect(),
    }
}

fn enter(state: &mut RouteState, catalog: &Catalog, requested: &str) -> Result<Vec<RouteEvent>, (RouteRejectCode, String)> {
    let world_id = migrate_world_id(requested).map_err(|c| (c, format!("unknown world: {requested}")))?;
    let target = catalog.worlds.iter().find(|w| w.world_id == world_id).expect("validated catalog");
    if target.order > 0 {
        let prior = &catalog.worlds[target.order - 1].world_id;
        if !state.progress.iter().find(|p| &p.world_id == prior).is_some_and(|p| p.completed) {
            return Err((RouteRejectCode::InvalidWorldOrder, format!("{prior} must be completed first")));
        }
    }
    state.current_world_id = world_id.to_string();
    let p = state.progress.iter_mut().find(|p| p.world_id == world_id).expect("frozen world exists");
    if p.visit_id == 0 { p.visit_id = 1; }
    Ok(vec![RouteEvent::Entered { world_id: world_id.to_string(), visit_id: p.visit_id }])
}

fn progress(state: &mut RouteState, catalog: &Catalog, event_id: &str) -> Result<Vec<RouteEvent>, (RouteRejectCode, String)> {
    if event_id.trim().is_empty() { return Err((RouteRejectCode::EmptyId, "eventId is required".into())); }
    // The hub is not a catalog world: reject instead of panicking when no run is
    // in progress (walking back to the hub now returns current_world_id to it).
    let Some(definition) = catalog.worlds.iter().find(|w| w.world_id == state.current_world_id) else {
        return Err((RouteRejectCode::WorldNotCurrent, format!("no run is in progress for {}", state.current_world_id)));
    };
    if !definition.required_events.iter().any(|id| id == event_id) {
        return Err((RouteRejectCode::RequiredProgressMissing, format!("event is not valid for {}", state.current_world_id)));
    }
    let p = state.progress.iter_mut().find(|p| p.world_id == state.current_world_id).expect("frozen world exists");
    if !p.completed_events.iter().any(|id| id == event_id) { p.completed_events.push(event_id.to_string()); }
    Ok(vec![RouteEvent::Progressed { world_id: state.current_world_id.clone(), event_id: event_id.to_string() }])
}

fn complete(state: &mut RouteState, catalog: &Catalog, requested: &str) -> Result<Vec<RouteEvent>, (RouteRejectCode, String)> {
    let world_id = migrate_world_id(requested).map_err(|c| (c, format!("unknown world: {requested}")))?;
    if world_id != state.current_world_id { return Err((RouteRejectCode::WorldNotCurrent, "only current world can complete".into())); }
    let definition = catalog.worlds.iter().find(|w| w.world_id == world_id).expect("validated catalog");
    let p = state.progress.iter_mut().find(|p| p.world_id == world_id).expect("frozen world exists");
    if p.completed { return Ok(vec![RouteEvent::Completed { world_id: world_id.to_string(), first_completion: false }]); }
    if !definition.required_events.iter().all(|required| p.completed_events.iter().any(|actual| actual == required)) {
        return Err((RouteRejectCode::RequiredProgressMissing, "required mainline progress is incomplete".into()));
    }
    p.completed = true;
    p.first_completion = true;
    Ok(vec![RouteEvent::Completed { world_id: world_id.to_string(), first_completion: true }])
}

fn revisit(state: &mut RouteState, catalog: &Catalog, requested: &str) -> Result<Vec<RouteEvent>, (RouteRejectCode, String)> {
    let world_id = migrate_world_id(requested).map_err(|c| (c, format!("unknown world: {requested}")))?;
    let definition = catalog.worlds.iter().find(|w| w.world_id == world_id).expect("validated catalog");
    let p = state.progress.iter_mut().find(|p| p.world_id == world_id).expect("frozen world exists");
    if !p.completed { return Err((RouteRejectCode::FirstClearRequired, "first clear is required before revisit".into())); }
    let limit = match definition.revisit_policy {
        RevisitPolicy::Never => 0,
        RevisitPolicy::AfterFirstClearLimited { max_revisits } | RevisitPolicy::TaskCandidatesLimited { max_revisits } => max_revisits,
        RevisitPolicy::RenewableByCycle { max_revisits_per_cycle } => max_revisits_per_cycle,
    };
    if p.revisit_count >= limit { return Err((RouteRejectCode::RevisitLimitReached, "revisit limit reached".into())); }
    p.revisit_count += 1;
    p.visit_id = p.visit_id.saturating_add(1);
    state.current_world_id = world_id.to_string();
    Ok(vec![RouteEvent::Revisited { world_id: world_id.to_string(), visit_id: p.visit_id, revisit_count: p.revisit_count }])
}

/// Walk back out of the current world into the hub without completing it.
///
/// Human decision 2026-11-10: leaving through the maintenance door abandons the
/// run. Completion records and the reward ledger are never touched; only the
/// run-scoped event progress is cleared so the next entry is a new run. Map
/// knowledge lives outside `RouteState` and is therefore preserved by design.
fn abandon(state: &mut RouteState, catalog: &Catalog, requested: &str) -> Result<Vec<RouteEvent>, (RouteRejectCode, String)> {
    let world_id = migrate_world_id(requested).map_err(|c| (c, format!("unknown world: {requested}")))?;
    if catalog.worlds.iter().all(|w| w.world_id != world_id) {
        return Err((RouteRejectCode::UnknownWorld, format!("unknown world: {requested}")));
    }
    if world_id != state.current_world_id {
        return Err((RouteRejectCode::WorldNotCurrent, "only the current world can be abandoned".into()));
    }
    let p = state.progress.iter_mut().find(|p| p.world_id == world_id).expect("frozen world exists");
    if p.completed { return Err((RouteRejectCode::RunNotAbandonable, "a completed world cannot be abandoned".into())); }
    p.completed_events.clear();
    state.current_world_id = "return_station".to_string();
    Ok(vec![RouteEvent::Abandoned { world_id: world_id.to_string() }])
}

fn grant_reward(state: &mut RouteState, requested: &str, reward_id: &str, policy: RewardPolicy, request_id: String, visit_id: Option<u64>, cycle_id: Option<u64>, revision: WorldRevision) -> Result<Vec<RouteEvent>, (RouteRejectCode, String)> {
    let world_id = migrate_world_id(requested).map_err(|c| (c, format!("unknown world: {requested}")))?;
    if reward_id.trim().is_empty() { return Err((RouteRejectCode::EmptyId, "rewardId is required".into())); }
    let malformed = match policy { RewardPolicy::FirstClear | RewardPolicy::OneTime => visit_id.is_some() || cycle_id.is_some(), RewardPolicy::Revisit => visit_id.is_none() || cycle_id.is_some(), RewardPolicy::Renewable => cycle_id.is_none() || visit_id.is_some() };
    if malformed { return Err((RouteRejectCode::RewardKeyMalformed, "reward key does not match policy".into())); }
    let duplicate = state.reward_ledger.iter().any(|entry| entry.world_id == world_id && entry.reward_id == reward_id && entry.policy == policy && match policy { RewardPolicy::FirstClear | RewardPolicy::OneTime => true, RewardPolicy::Revisit => entry.visit_id == visit_id, RewardPolicy::Renewable => entry.cycle_id == cycle_id });
    if duplicate { return Err((RouteRejectCode::RewardAlreadyGranted, "reward key was already granted".into())); }
    if policy == RewardPolicy::FirstClear && !state.progress.iter().find(|p| p.world_id == world_id).is_some_and(|p| p.completed) { return Err((RouteRejectCode::FirstClearRequired, "first-clear reward requires completion".into())); }
    state.reward_ledger.push(RewardLedgerEntry { world_id: world_id.to_string(), reward_id: reward_id.to_string(), policy, request_id, visit_id, cycle_id, granted_at_revision: revision });
    Ok(vec![RouteEvent::RewardGranted { world_id: world_id.to_string(), reward_id: reward_id.to_string() }])
}

fn catalog() -> Result<Catalog, String> {
    let catalog: Catalog = serde_json::from_str(CATALOG_JSON).map_err(|e| format!("catalog parse failed: {e}"))?;
    let expected = [WORLD_GREY_HIVE, WORLD_MIST_HARBOR, WORLD_CLOCKWORKS];
    if catalog.schema_version != ROUTE_SCHEMA_VERSION || catalog.worlds.len() != expected.len() { return Err("catalog schema/world count mismatch".into()); }
    let mut ids = HashSet::new();
    for (index, world) in catalog.worlds.iter().enumerate() {
        if world.world_id != expected[index] || world.order != index || world.name.trim().is_empty() || !ids.insert(&world.world_id) || world.required_events.is_empty() { return Err("catalog identity/order invalid".into()); }
    }
    Ok(catalog)
}

fn applied(state: &RouteState, events: Vec<RouteEvent>) -> RouteResult { RouteResult::Applied { events, projection: project_route(state) } }
fn rejected(code: RouteRejectCode, message: &str) -> RouteResult { RouteResult::Rejected { code, message: message.to_string() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn revision() -> WorldRevision { WorldRevision::new(1, 0, 1).expect("valid revision") }
    fn gh(state: &RouteState) -> &WorldProgress { state.progress.iter().find(|p| p.world_id == WORLD_GREY_HIVE).unwrap() }

    fn entered_run() -> RouteState {
        let mut state = RouteState::new();
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Enter { world_id: WORLD_GREY_HIVE.into(), request_id: "enter".into() }, revision()),
            RouteResult::Applied { .. }));
        for (index, event) in ["hive_power", "hive_lockdown"].into_iter().enumerate() {
            assert!(matches!(
                apply_route_command(&mut state, RouteCommand::Progress { event_id: event.into(), request_id: format!("progress-{index}") }, revision()),
                RouteResult::Applied { .. }));
        }
        state
    }

    #[test]
    fn abandon_voids_the_run_and_returns_the_route_to_the_hub() {
        let mut state = entered_run();
        state.set_hive_choice_once(HiveChoice::Taken);
        state.record_sentinel_first_kill(WORLD_GREY_HIVE);
        let before_rewards = state.reward_ledger.clone();
        let result = apply_route_command(&mut state, RouteCommand::Abandon { world_id: WORLD_GREY_HIVE.into(), request_id: "leave".into() }, revision());
        match result {
            RouteResult::Applied { ref events, .. } => {
                assert_eq!(events, &vec![RouteEvent::Abandoned { world_id: WORLD_GREY_HIVE.into() }]);
            }
            RouteResult::Rejected { .. } => panic!("abandon must be accepted for an in-progress run"),
        }
        assert_eq!(state.current_world_id, "return_station", "the hub must become the current world");
        let progress = gh(&state);
        assert!(progress.completed_events.is_empty(), "the run's progress is void");
        assert!(!progress.completed, "abandoning never completes the world");
        assert!(!progress.first_completion, "abandoning never grants a first completion");
        assert_eq!(progress.hive_choice, HiveChoice::Taken, "decisions are knowledge, not run progress");
        assert!(progress.sentinel_first_kill_recorded, "first-kill records are knowledge, not run progress");
        assert_eq!(state.reward_ledger, before_rewards, "no reward may be granted by leaving");
        assert_eq!(project_route(&state).current_world_id, "return_station");
    }

    #[test]
    fn abandon_then_enter_starts_a_new_run() {
        let mut state = entered_run();
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Abandon { world_id: WORLD_GREY_HIVE.into(), request_id: "leave".into() }, revision()),
            RouteResult::Applied { .. }));
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Enter { world_id: WORLD_GREY_HIVE.into(), request_id: "re-enter".into() }, revision()),
            RouteResult::Applied { events, .. } if matches!(events.as_slice(), [RouteEvent::Entered { world_id, .. }] if world_id == WORLD_GREY_HIVE)));
        let progress = gh(&state);
        assert!(progress.completed_events.is_empty(), "a re-entry after abandon carries no events");
        assert_eq!(state.current_world_id, WORLD_GREY_HIVE);
        // The next event progresses the new run normally.
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Progress { event_id: "hive_power".into(), request_id: "power-again".into() }, revision()),
            RouteResult::Applied { events, .. } if matches!(events.as_slice(), [RouteEvent::Progressed { event_id, .. }] if event_id == "hive_power")));
        assert_eq!(gh(&state).completed_events, vec!["hive_power".to_string()]);
    }

    #[test]
    fn abandon_is_rejected_for_the_wrong_or_a_completed_world() {
        let mut state = entered_run();
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Abandon { world_id: WORLD_MIST_HARBOR.into(), request_id: "wrong-world".into() }, revision()),
            RouteResult::Rejected { code: RouteRejectCode::WorldNotCurrent, .. }));
        assert_eq!(gh(&state).completed_events, vec!["hive_power".to_string(), "hive_lockdown".to_string()]);
        let mut completed = entered_run();
        assert!(matches!(
            apply_route_command(&mut completed, RouteCommand::Progress { event_id: "hive_extraction".into(), request_id: "extract".into() }, revision()),
            RouteResult::Applied { .. }));
        assert!(matches!(
            apply_route_command(&mut completed, RouteCommand::Complete { world_id: WORLD_GREY_HIVE.into(), request_id: "complete".into() }, revision()),
            RouteResult::Applied { events, .. } if matches!(events.as_slice(), [RouteEvent::Completed { first_completion: true, .. }])));
        assert!(matches!(
            apply_route_command(&mut completed, RouteCommand::Abandon { world_id: WORLD_GREY_HIVE.into(), request_id: "leave-completed".into() }, revision()),
            RouteResult::Rejected { code: RouteRejectCode::RunNotAbandonable, .. }));
        assert!(gh(&completed).completed && gh(&completed).first_completion);
        assert_eq!(completed.current_world_id, WORLD_GREY_HIVE);
    }

    #[test]
    fn abandon_is_idempotent_for_a_replayed_request_id() {
        let mut state = entered_run();
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Abandon { world_id: WORLD_GREY_HIVE.into(), request_id: "leave".into() }, revision()),
            RouteResult::Applied { .. }));
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Abandon { world_id: WORLD_GREY_HIVE.into(), request_id: "leave".into() }, revision()),
            RouteResult::Applied { events, .. } if matches!(events.as_slice(), [RouteEvent::DuplicateIgnored { request_id }] if request_id == "leave")));
        assert_eq!(state.current_world_id, "return_station");
    }

    #[test]
    fn progress_at_the_hub_is_rejected_instead_of_panicking() {
        let mut state = entered_run();
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Abandon { world_id: WORLD_GREY_HIVE.into(), request_id: "leave".into() }, revision()),
            RouteResult::Applied { .. }));
        assert!(matches!(
            apply_route_command(&mut state, RouteCommand::Progress { event_id: "hive_power".into(), request_id: "hub-progress".into() }, revision()),
            RouteResult::Rejected { code: RouteRejectCode::WorldNotCurrent, .. }));
    }
}

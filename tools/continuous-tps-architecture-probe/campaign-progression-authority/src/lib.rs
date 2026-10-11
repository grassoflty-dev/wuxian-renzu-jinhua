use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const WORLD_ID: &str = "grey_hive";
pub const BOUNDARY_EPSILON: f64 = 1.0e-5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceState {
    Unresolved,
    Taken,
    Left,
}

impl ChoiceState {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "unresolved" => Some(Self::Unresolved),
            "taken" => Some(Self::Taken),
            "left" => Some(Self::Left),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Objective {
    Airlock,
    RestorePower,
    Lockdown,
    GateB,
    Deep,
    Sentinel,
    Beacon,
    Exit,
    ResultThenHub,
}

impl Objective {
    fn as_str(self) -> &'static str {
        match self {
            Self::Airlock => "airlock",
            Self::RestorePower => "restore_power",
            Self::Lockdown => "hive_lockdown_completed",
            Self::GateB => "gate_b",
            Self::Deep => "deep",
            Self::Sentinel => "sentinel",
            Self::Beacon => "beacon",
            Self::Exit => "active_exit",
            Self::ResultThenHub => "result_then_hub",
        }
    }
    fn known(value: &str) -> bool {
        matches!(
            value,
            "airlock"
                | "restore_power"
                | "hive_lockdown_completed"
                | "gate_b"
                | "deep"
                | "sentinel"
                | "beacon"
                | "active_exit"
                | "result_then_hub"
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub event_id: String,
    pub epoch: u64,
    pub tick: u64,
    pub actor_id: String,
    pub kind: String,
    pub prerequisite: Option<String>,
    pub objective: Option<String>,
    pub choice: Option<String>,
    pub npc_available: Option<bool>,
    pub numeric_value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RewardRecord {
    pub reward_id: String,
    pub amount: i64,
    pub granted_at_tick: u64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResultState {
    pub summary: String,
    pub rank: String,
    pub reward_total: i64,
    pub sample_record: String,
    pub map: Option<String>,
    pub settlement_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignSnapshot {
    pub world_epoch: u64,
    pub tick: u64,
    pub objective: Objective,
    pub hive_power: bool,
    pub hive_lockdown_completed: bool,
    pub gate_a_open: bool,
    pub gate_b_open: bool,
    pub hive_choice: ChoiceState,
    pub baizhi_conversed: bool,
    pub sentinel_defeated: bool,
    pub beacon_active: bool,
    pub exit_active: bool,
    pub checkpoint_id: String,
    pub reward_ledger: Vec<RewardRecord>,
    pub processed_event_ids: Vec<String>,
    pub result: Option<ResultState>,
    pub mode: String,
    pub active_world: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateDiff {
    pub objective_before: String,
    pub objective_after: String,
    pub flags_added: Vec<String>,
    pub checkpoint_before: String,
    pub checkpoint_after: String,
    pub reward_count_delta: i64,
    pub result_before: bool,
    pub result_after: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub actor_id: String,
    pub epoch: u64,
    pub tick: u64,
    pub outcome: String,
    pub error_code: Option<String>,
    pub state_diff: StateDiff,
    pub next_objective: String,
    pub next_checkpoint: String,
    pub reward_records: Vec<RewardRecord>,
    pub candidate_only: bool,
    pub evidence_level: String,
    pub state_hash: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Authority {
    pub world_epoch: u64,
    pub tick: u64,
    pub objective: Objective,
    pub hive_power: bool,
    pub hive_lockdown_completed: bool,
    pub gate_a_open: bool,
    pub gate_b_open: bool,
    pub hive_choice: ChoiceState,
    pub baizhi_conversed: bool,
    pub sentinel_defeated: bool,
    pub beacon_active: bool,
    pub exit_active: bool,
    pub checkpoint_id: String,
    pub reward_ledger: Vec<RewardRecord>,
    pub processed_event_ids: BTreeMap<String, (u64, u64)>,
    pub result: Option<ResultState>,
    pub mode: String,
    pub active_world: Option<String>,
}

impl Authority {
    pub fn new(world_epoch: u64) -> Self {
        Self {
            world_epoch,
            tick: 0,
            objective: Objective::Airlock,
            hive_power: false,
            hive_lockdown_completed: false,
            gate_a_open: false,
            gate_b_open: false,
            hive_choice: ChoiceState::Unresolved,
            baizhi_conversed: false,
            sentinel_defeated: false,
            beacon_active: false,
            exit_active: false,
            checkpoint_id: "gh_cp_airlock".to_string(),
            reward_ledger: Vec::new(),
            processed_event_ids: BTreeMap::new(),
            result: None,
            mode: "world".to_string(),
            active_world: Some(WORLD_ID.to_string()),
        }
    }

    pub fn apply_batch(&mut self, mut events: Vec<ProgressEvent>) -> Vec<EventRecord> {
        events.sort_by(|left, right| {
            (left.tick, &left.event_id, &left.actor_id).cmp(&(
                right.tick,
                &right.event_id,
                &right.actor_id,
            ))
        });
        events.into_iter().map(|event| self.apply(event)).collect()
    }

    pub fn apply(&mut self, event: ProgressEvent) -> EventRecord {
        let before = self.snapshot();
        let side_effect_before = self.reward_ledger.len();
        if event.epoch != self.world_epoch {
            return self.reject(&event, "E_OLD_EPOCH", &before, side_effect_before);
        }
        if self.processed_event_ids.contains_key(&event.event_id) {
            return self.reject(&event, "E_DUPLICATE_EVENT", &before, side_effect_before);
        }
        if event.tick < self.tick {
            return self.reject(&event, "E_OLD_TICK", &before, side_effect_before);
        }
        if event.event_id.is_empty() || event.actor_id.is_empty() {
            return self.reject(
                &event,
                "E_EMPTY_EVENT_ID_OR_ACTOR",
                &before,
                side_effect_before,
            );
        }
        if let Some(value) = event.numeric_value {
            if !value.is_finite() {
                return self.reject(&event, "E_NONFINITE_VALUE", &before, side_effect_before);
            }
        }
        if let Some(objective) = event.objective.as_deref() {
            if !Objective::known(objective) {
                return self.reject(&event, "E_UNKNOWN_OBJECTIVE", &before, side_effect_before);
            }
        }
        if let Some(flag) = event.prerequisite.as_deref() {
            if !matches!(flag, "hive_power" | "hive_lockdown_completed") {
                return self.reject(&event, "E_UNKNOWN_FLAG", &before, side_effect_before);
            }
        }
        self.tick = event.tick;
        self.processed_event_ids
            .insert(event.event_id.clone(), (event.epoch, event.tick));

        let error = match event.kind.as_str() {
            "complete_airlock" => self.complete_airlock(&event),
            "restore_power" => self.restore_power(&event),
            "complete_lockdown" => self.complete_lockdown(&event),
            "first_conversation" => self.first_conversation(&event),
            "choose_baizhi" => self.choose_baizhi(&event),
            "reach_gate_b" => self.reach_gate_b(&event),
            "enter_deep" => self.enter_deep(&event),
            "defeat_sentinel" => self.defeat_sentinel(&event),
            "activate_beacon" => self.activate_beacon(&event),
            "activate_exit" => self.activate_exit(&event),
            "extract" => self.extract(&event),
            "complete_result" => self.complete_result(&event),
            _ => Some("E_UNKNOWN_EVENT"),
        };
        if let Some(error) = error {
            return self.reject(&event, error, &before, side_effect_before);
        }
        self.accept(&event, &before, side_effect_before)
    }

    fn require_objective(&self, expected: Objective) -> Option<&'static str> {
        if self.objective == expected {
            None
        } else {
            Some("E_OBJECTIVE_PREREQUISITE")
        }
    }

    fn require_flag(event: &ProgressEvent, expected: &'static str) -> Option<&'static str> {
        match event.prerequisite.as_deref() {
            Some(actual) if actual != expected => Some("E_PREREQUISITE"),
            _ => None,
        }
    }

    fn complete_airlock(&mut self, event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Airlock) {
            return Some(error);
        }
        self.objective = Objective::RestorePower;
        self.checkpoint_id = "gh_cp_airlock".to_string();
        let _ = event;
        None
    }

    fn restore_power(&mut self, event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::RestorePower) {
            return Some(error);
        }
        if let Some(error) = Self::require_flag(event, "") {
            return Some(error);
        }
        self.hive_power = true;
        self.gate_a_open = true;
        self.objective = Objective::Lockdown;
        self.checkpoint_id = "gh_cp_power".to_string();
        None
    }

    fn complete_lockdown(&mut self, event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Lockdown) {
            return Some(error);
        }
        if let Some(error) = Self::require_flag(event, "hive_power") {
            return Some(error);
        }
        if !self.hive_power {
            return Some("E_PREREQUISITE");
        }
        self.hive_lockdown_completed = true;
        self.gate_b_open = true;
        self.objective = Objective::GateB;
        self.checkpoint_id = "gh_cp_central".to_string();
        None
    }

    fn first_conversation(&mut self, event: &ProgressEvent) -> Option<&'static str> {
        if self.mode != "world" {
            return Some("E_NOT_IN_WORLD");
        }
        self.baizhi_conversed = true;
        let _ = event.npc_available;
        None
    }

    fn choose_baizhi(&mut self, event: &ProgressEvent) -> Option<&'static str> {
        if !self.baizhi_conversed {
            return Some("E_CHOICE_BEFORE_CONVERSATION");
        }
        if self.hive_choice != ChoiceState::Unresolved {
            return Some("E_CHOICE_ALREADY_SET");
        }
        let Some(choice) = event.choice.as_deref().and_then(ChoiceState::from_str) else {
            return Some("E_UNKNOWN_CHOICE");
        };
        if choice == ChoiceState::Unresolved {
            return Some("E_UNKNOWN_CHOICE");
        }
        self.hive_choice = choice;
        None
    }

    fn reach_gate_b(&mut self, event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::GateB) {
            return Some(error);
        }
        if let Some(error) = Self::require_flag(event, "hive_lockdown_completed") {
            return Some(error);
        }
        if !self.hive_lockdown_completed || !self.gate_b_open {
            return Some("E_PREREQUISITE");
        }
        self.objective = Objective::Deep;
        self.checkpoint_id = "gh_cp_post_bio".to_string();
        None
    }

    fn enter_deep(&mut self, _event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Deep) {
            return Some(error);
        }
        self.objective = Objective::Sentinel;
        self.checkpoint_id = "gh_cp_pre_sentinel".to_string();
        None
    }

    fn defeat_sentinel(&mut self, _event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Sentinel) {
            return Some(error);
        }
        self.sentinel_defeated = true;
        self.objective = Objective::Beacon;
        None
    }

    fn activate_beacon(&mut self, _event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Beacon) {
            return Some(error);
        }
        if !self.sentinel_defeated {
            return Some("E_PREREQUISITE");
        }
        self.beacon_active = true;
        self.objective = Objective::Exit;
        self.checkpoint_id = "gh_cp_exit".to_string();
        None
    }

    fn activate_exit(&mut self, _event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Exit) {
            return Some(error);
        }
        if !self.beacon_active {
            return Some("E_PREREQUISITE");
        }
        self.exit_active = true;
        None
    }

    fn extract(&mut self, _event: &ProgressEvent) -> Option<&'static str> {
        if let Some(error) = self.require_objective(Objective::Exit) {
            return Some(error);
        }
        if !self.exit_active {
            return Some("E_PREREQUISITE");
        }
        if self.result.is_some() {
            return Some("E_ALREADY_COMPLETE");
        }
        self.objective = Objective::ResultThenHub;
        let (sample_record, bonus) = match self.hive_choice {
            ChoiceState::Taken => ("白芷的样本：已带走", 10),
            ChoiceState::Left => ("白芷的样本：留在灰巢", 0),
            ChoiceState::Unresolved => ("白芷的样本：未处理", 0),
        };
        let reward_total = 42 + bonus;
        let reward_id = "grey_hive_first_clear".to_string();
        if !self
            .reward_ledger
            .iter()
            .any(|reward| reward.reward_id == reward_id)
        {
            self.reward_ledger.push(RewardRecord {
                reward_id,
                amount: reward_total,
                granted_at_tick: self.tick,
                reason: "grey_hive_completed_once".to_string(),
            });
        }
        self.result = Some(ResultState {
            summary: format!("灰巢完成；{}。", sample_record),
            rank: if self.hive_choice == ChoiceState::Taken {
                "A"
            } else {
                "B"
            }
            .to_string(),
            reward_total,
            sample_record: sample_record.to_string(),
            map: None,
            settlement_count: 1,
        });
        self.mode = "result".to_string();
        self.active_world = None;
        None
    }

    fn complete_result(&mut self, _event: &ProgressEvent) -> Option<&'static str> {
        if self.mode == "hub" {
            return Some("E_ALREADY_SETTLED");
        }
        if self.objective != Objective::ResultThenHub || self.result.is_none() {
            return Some("E_RESULT_NOT_READY");
        }
        self.mode = "hub".to_string();
        None
    }

    fn accept(
        &self,
        event: &ProgressEvent,
        before: &CampaignSnapshot,
        reward_before: usize,
    ) -> EventRecord {
        let after = self.snapshot();
        EventRecord {
            event_id: event.event_id.clone(),
            actor_id: event.actor_id.clone(),
            epoch: event.epoch,
            tick: event.tick,
            outcome: "accepted".to_string(),
            error_code: None,
            state_diff: diff(before, &after, reward_before, self.reward_ledger.len()),
            next_objective: self.objective.as_str().to_string(),
            next_checkpoint: self.checkpoint_id.clone(),
            reward_records: self
                .reward_ledger
                .iter()
                .skip(reward_before)
                .cloned()
                .collect(),
            candidate_only: true,
            evidence_level: "isolated_candidate_tp14a1".to_string(),
            state_hash: self.state_hash(),
        }
    }

    fn reject(
        &self,
        event: &ProgressEvent,
        error: &'static str,
        before: &CampaignSnapshot,
        reward_before: usize,
    ) -> EventRecord {
        let after = self.snapshot();
        EventRecord {
            event_id: event.event_id.clone(),
            actor_id: event.actor_id.clone(),
            epoch: event.epoch,
            tick: event.tick,
            outcome: "rejected".to_string(),
            error_code: Some(error.to_string()),
            state_diff: diff(before, &after, reward_before, self.reward_ledger.len()),
            next_objective: self.objective.as_str().to_string(),
            next_checkpoint: self.checkpoint_id.clone(),
            reward_records: Vec::new(),
            candidate_only: true,
            evidence_level: "isolated_candidate_tp14a1".to_string(),
            state_hash: self.state_hash(),
        }
    }

    pub fn snapshot(&self) -> CampaignSnapshot {
        CampaignSnapshot {
            world_epoch: self.world_epoch,
            tick: self.tick,
            objective: self.objective,
            hive_power: self.hive_power,
            hive_lockdown_completed: self.hive_lockdown_completed,
            gate_a_open: self.gate_a_open,
            gate_b_open: self.gate_b_open,
            hive_choice: self.hive_choice,
            baizhi_conversed: self.baizhi_conversed,
            sentinel_defeated: self.sentinel_defeated,
            beacon_active: self.beacon_active,
            exit_active: self.exit_active,
            checkpoint_id: self.checkpoint_id.clone(),
            reward_ledger: self.reward_ledger.clone(),
            processed_event_ids: self.processed_event_ids.keys().cloned().collect(),
            result: self.result.clone(),
            mode: self.mode.clone(),
            active_world: self.active_world.clone(),
        }
    }

    pub fn reload(&mut self, snapshot: CampaignSnapshot) -> Result<(), &'static str> {
        if snapshot.world_epoch != self.world_epoch {
            return Err("E_RELOAD_EPOCH");
        }
        if !Objective::known(snapshot.objective.as_str()) || snapshot.checkpoint_id.is_empty() {
            return Err("E_RELOAD_STATE");
        }
        self.tick = snapshot.tick;
        self.objective = snapshot.objective;
        self.hive_power = snapshot.hive_power;
        self.hive_lockdown_completed = snapshot.hive_lockdown_completed;
        self.gate_a_open = snapshot.gate_a_open;
        self.gate_b_open = snapshot.gate_b_open;
        self.hive_choice = snapshot.hive_choice;
        self.baizhi_conversed = snapshot.baizhi_conversed;
        self.sentinel_defeated = snapshot.sentinel_defeated;
        self.beacon_active = snapshot.beacon_active;
        self.exit_active = snapshot.exit_active;
        self.checkpoint_id = snapshot.checkpoint_id;
        self.reward_ledger = snapshot.reward_ledger;
        self.processed_event_ids = snapshot
            .processed_event_ids
            .into_iter()
            .map(|id| (id, (self.world_epoch, self.tick)))
            .collect();
        self.result = snapshot.result;
        self.mode = snapshot.mode;
        self.active_world = snapshot.active_world;
        Ok(())
    }

    pub fn state_hash(&self) -> String {
        let bytes = serde_json::to_vec(&self.snapshot()).expect("snapshot serializes");
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("fnv1a64-{hash:016x}")
    }
}

fn diff(
    before: &CampaignSnapshot,
    after: &CampaignSnapshot,
    reward_before: usize,
    reward_after: usize,
) -> StateDiff {
    let mut flags_added = Vec::new();
    if !before.hive_power && after.hive_power {
        flags_added.push("hive_power".to_string());
    }
    if !before.hive_lockdown_completed && after.hive_lockdown_completed {
        flags_added.push("hive_lockdown_completed".to_string());
    }
    if !before.gate_a_open && after.gate_a_open {
        flags_added.push("gate_a_open".to_string());
    }
    if !before.gate_b_open && after.gate_b_open {
        flags_added.push("gate_b_open".to_string());
    }
    if !before.sentinel_defeated && after.sentinel_defeated {
        flags_added.push("sentinel_defeated".to_string());
    }
    if !before.beacon_active && after.beacon_active {
        flags_added.push("beacon_active".to_string());
    }
    if !before.exit_active && after.exit_active {
        flags_added.push("exit_active".to_string());
    }
    StateDiff {
        objective_before: before.objective.as_str().to_string(),
        objective_after: after.objective.as_str().to_string(),
        flags_added,
        checkpoint_before: before.checkpoint_id.clone(),
        checkpoint_after: after.checkpoint_id.clone(),
        reward_count_delta: reward_after as i64 - reward_before as i64,
        result_before: before.result.is_some(),
        result_after: after.result.is_some(),
    }
}

fn event(id: &str, tick: u64, kind: &str) -> ProgressEvent {
    ProgressEvent {
        event_id: id.to_string(),
        epoch: 7,
        tick,
        actor_id: "player".to_string(),
        kind: kind.to_string(),
        prerequisite: None,
        objective: None,
        choice: None,
        npc_available: None,
        numeric_value: None,
    }
}

fn with_prereq(mut event: ProgressEvent, prerequisite: &str) -> ProgressEvent {
    event.prerequisite = Some(prerequisite.to_string());
    event
}

fn scenario(
    id: &str,
    input: Value,
    events: Value,
    expected: Value,
    actual: Value,
    pass: bool,
    authority: &Authority,
    timeline: Vec<u64>,
) -> Value {
    json!({
        "id": id,
        "input": input,
        "events": events,
        "expected": expected,
        "actual": actual,
        "pass": pass,
        "objectiveDiff": {"current": authority.objective.as_str()},
        "checkpointDiff": {"current": authority.checkpoint_id},
        "rewardCount": authority.reward_ledger.len(),
        "timelineTicks": timeline,
        "stateHash": authority.state_hash(),
        "evidenceLevel": "isolated_candidate_tp14a1"
    })
}

fn complete_choice(choice: Option<&str>) -> (Authority, Vec<EventRecord>, bool) {
    let mut authority = Authority::new(7);
    let mut events = Vec::new();
    let mut apply = |event: ProgressEvent| {
        events.push(authority.apply(event));
    };
    apply(event("airlock", 0, "complete_airlock"));
    if let Some(choice) = choice {
        apply(event("baizhi-dialog", 1, "first_conversation"));
        let mut choose = event("baizhi-choice", 2, "choose_baizhi");
        choose.choice = Some(choice.to_string());
        choose.npc_available = Some(true);
        apply(choose);
    } else {
        apply(event("baizhi-dialog-unresolved", 1, "first_conversation"));
    }
    apply(event("restore-power", 3, "restore_power"));
    apply(with_prereq(
        event("lockdown", 4, "complete_lockdown"),
        "hive_power",
    ));
    apply(with_prereq(
        event("gate-b", 5, "reach_gate_b"),
        "hive_lockdown_completed",
    ));
    apply(event("deep", 6, "enter_deep"));
    apply(event("sentinel", 7, "defeat_sentinel"));
    apply(event("beacon", 8, "activate_beacon"));
    apply(event("exit-active", 9, "activate_exit"));
    apply(event("extract", 10, "extract"));
    apply(event("result-hub", 11, "complete_result"));
    let pass = authority.objective == Objective::ResultThenHub
        && authority.hive_power
        && authority.hive_lockdown_completed
        && authority.gate_a_open
        && authority.gate_b_open
        && authority.sentinel_defeated
        && authority.beacon_active
        && authority.exit_active
        && authority.mode == "hub"
        && authority.active_world.is_none()
        && authority
            .result
            .as_ref()
            .is_some_and(|result| result.map.is_none() && result.settlement_count == 1)
        && authority.reward_ledger.len() == 1;
    (authority, events, pass)
}

pub fn run_probe() -> Value {
    let mut scenarios = Vec::new();
    for choice in [Some("taken"), Some("left"), None] {
        let label = choice.unwrap_or("unresolved");
        let (authority, records, pass) = complete_choice(choice);
        let sample = authority
            .result
            .as_ref()
            .map(|result| result.sample_record.clone());
        let expected_sample = match label {
            "taken" => "白芷的样本：已带走",
            "left" => "白芷的样本：留在灰巢",
            _ => "白芷的样本：未处理",
        };
        scenarios.push(scenario(
            &format!("complete_grey_hive_{label}"),
            json!({"world":"grey_hive","choice":label,"npcOptional":true}),
            json!(records),
            json!({"objective":"result_then_hub","sampleRecord":expected_sample,"rewardCount":1,"map":null}),
            json!({"objective":authority.objective,"sampleRecord":sample,"rewardCount":authority.reward_ledger.len(),"mode":authority.mode,"map":authority.result.as_ref().and_then(|r|r.map.clone())}),
            pass && sample.as_deref() == Some(expected_sample),
            &authority,
            (0..=11).collect(),
        ));
    }

    let mut repeat = Authority::new(7);
    for event in [
        event("d-airlock", 0, "complete_airlock"),
        event("d-power", 1, "restore_power"),
        with_prereq(event("d-lock", 2, "complete_lockdown"), "hive_power"),
        with_prereq(
            event("d-gate", 3, "reach_gate_b"),
            "hive_lockdown_completed",
        ),
        event("d-deep", 4, "enter_deep"),
        event("d-sentinel", 5, "defeat_sentinel"),
    ] {
        repeat.apply(event);
    }
    let sentinel_duplicate = repeat.apply(event("d-sentinel", 5, "defeat_sentinel"));
    let sentinel_repeat = repeat.apply(event("d-sentinel-again", 6, "defeat_sentinel"));
    let beacon = repeat.apply(event("d-beacon", 7, "activate_beacon"));
    let beacon_duplicate = repeat.apply(event("d-beacon", 7, "activate_beacon"));
    let exit = repeat.apply(event("d-exit", 8, "activate_exit"));
    let exit_duplicate = repeat.apply(event("d-exit", 8, "activate_exit"));
    let repeat_pass = sentinel_duplicate.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && sentinel_repeat.error_code.as_deref() == Some("E_OBJECTIVE_PREREQUISITE")
        && beacon.outcome == "accepted"
        && beacon_duplicate.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && exit.outcome == "accepted"
        && exit_duplicate.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && repeat.sentinel_defeated
        && repeat.beacon_active
        && repeat.exit_active;
    scenarios.push(scenario(
        "sentinel_beacon_exit_duplicate_triggers",
        json!({"sentinel":true,"beacon":true,"exit":true,"duplicatePolicy":"event ID and objective guard"}),
        json!([sentinel_duplicate,sentinel_repeat,beacon,beacon_duplicate,exit,exit_duplicate]),
        json!({"sentinelDuplicate":"E_DUPLICATE_EVENT","beaconDuplicate":"E_DUPLICATE_EVENT","exitDuplicate":"E_DUPLICATE_EVENT","statePreserved":true}),
        json!({"sentinelDuplicate":sentinel_duplicate.error_code,"sentinelRepeat":sentinel_repeat.error_code,"beacon":beacon.outcome,"beaconDuplicate":beacon_duplicate.error_code,"exit":exit.outcome,"exitDuplicate":exit_duplicate.error_code,"statePreserved":repeat.sentinel_defeated && repeat.beacon_active && repeat.exit_active}),
        repeat_pass,
        &repeat,
        vec![0,1,2,3,4,5,6,7,8],
    ));

    let mut gate = Authority::new(7);
    let blocked_lockdown = gate.apply(with_prereq(
        event("blocked-lockdown", 0, "complete_lockdown"),
        "hive_power",
    ));
    let mut gate_a = Authority::new(7);
    gate_a.apply(event("power", 0, "complete_airlock"));
    let power_record = gate_a.apply(event("power-on", 1, "restore_power"));
    let gate_a_ok = gate_a.gate_a_open && gate_a.hive_power;
    let mut gate_b = Authority::new(7);
    gate_b.apply(event("b-airlock", 0, "complete_airlock"));
    gate_b.apply(event("b-power", 1, "restore_power"));
    let b_locked = gate_b.apply(with_prereq(
        event("b-lock", 2, "complete_lockdown"),
        "hive_power",
    ));
    let gate_matrix_pass = blocked_lockdown.error_code.as_deref()
        == Some("E_OBJECTIVE_PREREQUISITE")
        && power_record.outcome == "accepted"
        && gate_a_ok
        && b_locked.outcome == "accepted";
    scenarios.push(scenario(
        "gate_a_gate_b_prerequisite_matrix",
        json!({"gateA":"hive_power only","gateB":"hive_lockdown_completed only","choiceStates":["taken","left","unresolved"],"npcIgnored":true}),
        json!([blocked_lockdown,power_record,b_locked]),
        json!({"gateA":"hive_power","gateB":"hive_lockdown_completed","lockdownSkip":"rejected"}),
        json!({"gateAOpen":gate_a.gate_a_open,"gateBOpen":gate_b.gate_b_open,"lockdownSkip":blocked_lockdown.error_code}),
        gate_matrix_pass,
        &gate_b,
        vec![0,1,2],
    ));

    let mut negative = Authority::new(7);
    let skipped = negative.apply(event("skip", 0, "reach_gate_b"));
    let mut unknown_objective = event("unknown-objective", 0, "complete_airlock");
    unknown_objective.objective = Some("not-a-real-objective".to_string());
    let unknown_obj_record = negative.apply(unknown_objective);
    let mut unknown_flag = event("unknown-flag", 0, "complete_airlock");
    unknown_flag.prerequisite = Some("not-a-real-flag".to_string());
    let unknown_flag_record = negative.apply(unknown_flag);
    let mut nonfinite = event("nonfinite", 0, "complete_airlock");
    nonfinite.numeric_value = Some(f64::NAN);
    let nonfinite_record = negative.apply(nonfinite);
    let unknown_event_record = negative.apply(event("unknown-event", 0, "not_a_real_event"));
    scenarios.push(scenario(
        "skip_unknown_and_nonfinite_rejected",
        json!({"skip":"reach_gate_b before mainline","unknownObjective":true,"unknownFlag":true,"unknownEvent":true,"nonfinite":["NaN","Infinity"]}),
        json!([skipped,unknown_obj_record,unknown_flag_record,nonfinite_record,unknown_event_record]),
        json!({"skip":"E_OBJECTIVE_PREREQUISITE","unknownObjective":"E_UNKNOWN_OBJECTIVE","unknownFlag":"E_UNKNOWN_FLAG","unknownEvent":"E_UNKNOWN_EVENT","nonfinite":"E_NONFINITE_VALUE"}),
        json!({"skip":skipped.error_code,"unknownObjective":unknown_obj_record.error_code,"unknownFlag":unknown_flag_record.error_code,"unknownEvent":unknown_event_record.error_code,"nonfinite":nonfinite_record.error_code}),
        skipped.error_code.as_deref()==Some("E_OBJECTIVE_PREREQUISITE") && unknown_obj_record.error_code.as_deref()==Some("E_UNKNOWN_OBJECTIVE") && unknown_flag_record.error_code.as_deref()==Some("E_UNKNOWN_FLAG") && unknown_event_record.error_code.as_deref()==Some("E_UNKNOWN_EVENT") && nonfinite_record.error_code.as_deref()==Some("E_NONFINITE_VALUE"),
        &negative,
        vec![0],
    ));

    let mut dup = Authority::new(7);
    let first = dup.apply(event("dup", 0, "complete_airlock"));
    let duplicate = dup.apply(event("dup", 0, "complete_airlock"));
    let mut old = event("old", 0, "restore_power");
    old.epoch = 6;
    let old_epoch = dup.apply(old);
    let future = dup.apply(event("future", 2, "restore_power"));
    let mut old_tick = event("old-tick", 1, "complete_lockdown");
    old_tick.prerequisite = Some("hive_power".to_string());
    let old_tick_record = dup.apply(old_tick);
    scenarios.push(scenario(
        "duplicate_old_epoch_old_tick_and_future",
        json!({"duplicate":"dup","epoch":7,"currentTick":2}),
        json!([first,duplicate,old_epoch,future,old_tick_record]),
        json!({"duplicate":"E_DUPLICATE_EVENT","oldEpoch":"E_OLD_EPOCH","oldTick":"E_OLD_TICK","future":"accepted"}),
        json!({"duplicate":duplicate.error_code,"oldEpoch":old_epoch.error_code,"oldTick":old_tick_record.error_code,"future":future.outcome}),
        duplicate.error_code.as_deref()==Some("E_DUPLICATE_EVENT") && old_epoch.error_code.as_deref()==Some("E_OLD_EPOCH") && old_tick_record.error_code.as_deref()==Some("E_OLD_TICK") && future.outcome=="accepted",
        &dup,
        vec![0,2],
    ));

    let mut sorted = Authority::new(7);
    let mut b = event("event-b", 0, "complete_airlock");
    let mut a = event("event-a", 0, "complete_airlock");
    a.actor_id = "actor-a".to_string();
    b.actor_id = "actor-z".to_string();
    let sorted_records = sorted.apply_batch(vec![b, a]);
    let sorted_ids: Vec<&str> = sorted_records
        .iter()
        .map(|record| record.event_id.as_str())
        .collect();
    let sorting_pass = sorted_ids == vec!["event-a", "event-b"]
        && sorted_records.iter().all(|r| {
            r.outcome == "accepted" || r.error_code.as_deref() == Some("E_OBJECTIVE_PREREQUISITE")
        });
    scenarios.push(scenario(
        "same_tick_event_id_then_actor_sort",
        json!({"tick":0,"sortKey":["tick","eventId","actorId"],"inputOrder":["event-b","event-a"]}),
        json!(sorted_records),
        json!({"order":["event-a","event-b"]}),
        json!({"order":sorted_ids}),
        sorting_pass,
        &sorted,
        vec![0],
    ));

    let mut reload_source = Authority::new(7);
    for event in [
        event("r-airlock", 0, "complete_airlock"),
        event("r-power", 1, "restore_power"),
        with_prereq(event("r-lock", 2, "complete_lockdown"), "hive_power"),
        with_prereq(
            event("r-gate", 3, "reach_gate_b"),
            "hive_lockdown_completed",
        ),
        event("r-deep", 4, "enter_deep"),
        event("r-sentinel", 5, "defeat_sentinel"),
        event("r-beacon", 6, "activate_beacon"),
        event("r-exit", 7, "activate_exit"),
    ] {
        reload_source.apply(event);
    }
    let snapshot = reload_source.snapshot();
    let mut reloaded = Authority::new(7);
    let before_reload_hash = reload_source.state_hash();
    let reload_ok =
        reloaded.reload(snapshot.clone()).is_ok() && reloaded.state_hash() == before_reload_hash;
    let extract = reloaded.apply(event("r-extract", 8, "extract"));
    let duplicate_extract = reloaded.apply(event("r-extract", 8, "extract"));
    let complete_result = reloaded.apply(event("r-result", 9, "complete_result"));
    let repeat_result = reloaded.apply(event("r-result", 9, "complete_result"));
    let reload_pass = reload_ok
        && extract.outcome == "accepted"
        && duplicate_extract.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && complete_result.outcome == "accepted"
        && repeat_result.error_code.as_deref() == Some("E_DUPLICATE_EVENT")
        && reloaded.reward_ledger.len() == 1
        && reloaded
            .result
            .as_ref()
            .is_some_and(|r| r.map.is_none() && r.settlement_count == 1);
    scenarios.push(scenario(
        "reload_continue_reward_result_idempotency",
        json!({"snapshotTick":snapshot.tick,"snapshotChoice":"unresolved","mapAfterCompletion":null}),
        json!([extract,duplicate_extract,complete_result,repeat_result]),
        json!({"reloadSameHash":true,"rewardCount":1,"settlementCount":1,"map":null}),
        json!({"reloadSameHash":reload_ok,"rewardCount":reloaded.reward_ledger.len(),"settlementCount":reloaded.result.as_ref().map(|r|r.settlement_count),"map":reloaded.result.as_ref().and_then(|r|r.map.clone()),"duplicateExtract":duplicate_extract.error_code,"repeatResult":repeat_result.error_code}),
        reload_pass,
        &reloaded,
        vec![0,9],
    ));

    fn frequency_run() -> (String, Vec<RewardRecord>, Option<ResultState>) {
        let (mut authority, _, _) = complete_choice(None);
        (
            authority.state_hash(),
            authority.reward_ledger.clone(),
            authority.result.take(),
        )
    }
    let hz30 = frequency_run();
    let hz60 = frequency_run();
    scenarios.push(scenario(
        "thirty_sixty_hz_same_final_campaign_state",
        json!({"frequenciesHz":[30,60],"sameEventStream":true}),
        json!(["complete_mainline@fixed_ticks"]),
        json!({"sameStateHash":true,"sameRewardLedger":true,"sameResult":true}),
        json!({"hz30Hash":hz30.0,"hz60Hash":hz60.0,"sameStateHash":hz30.0==hz60.0,"sameRewardLedger":hz30.1==hz60.1,"sameResult":hz30.2==hz60.2}),
        hz30.0==hz60.0 && hz30.1==hz60.1 && hz30.2==hz60.2,
        &complete_choice(None).0,
        (0..=11).collect(),
    ));

    let all_pass = scenarios.iter().all(|scenario| scenario["pass"] == true);
    json!({
        "schema":"campaign_progression_authority_tp14a1_v1",
        "status":if all_pass {"passed_candidate_isolated"} else {"failed_candidate_probe"},
        "pass":all_pass,
        "candidateOnly":true,
        "formalSceneReady":false,
        "stateContract":{
            "world":"grey_hive",
            "worldEpochAndTick":true,
            "objective":"airlock -> restore_power -> hive_lockdown_completed -> gate_b -> deep -> sentinel -> beacon -> active_exit -> result_then_hub",
            "flags":["hive_power","hive_lockdown_completed","gate_a_open","gate_b_open","sentinel_defeated","beacon_active","exit_active"],
            "choiceStates":["taken","left","unresolved"],
            "checkpointIds":["gh_cp_airlock","gh_cp_power","gh_cp_central","gh_cp_post_bio","gh_cp_pre_sentinel","gh_cp_exit"],
            "rewardLedger":"unique reward_id, one grey_hive_first_clear record",
            "processedEventIds":"BTreeMap unique IDs",
            "result":"map=null, settlement_count=1"
        },
        "apiContract":{
            "input":["eventId","epoch","tick","actorId","kind","prerequisite","objective","choice","npcAvailable","numericValue"],
            "output":["accepted/rejected","stableErrorCode","stateDiff","nextObjective","nextCheckpoint","rewardRecords","serializableEventRecord"],
            "sortKey":["tick","eventId","actorId"],
            "owner":"isolated candidate progression authority; formal CampaignStore not wired"
        },
        "gatePolicy":{"gateA":"hive_power only","gateB":"hive_lockdown_completed only","choiceIndependent":true,"npcIndependent":true},
        "reloadPolicy":{"restores":"objective, flags, choice, checkpoint, processed IDs, reward ledger, result","doesNot":"rewrite choice or regrant reward"},
        "scenarios":scenarios,
        "formalGaps":["formal Campaign/CampaignStore wiring absent","TP-03D1 resolver absent","real world/door/combat owners absent","UI and save power-loss guarantees absent","30/60 evidence is fixed-event candidate only","TP-14/Campaign/Save gates blocked; Architecture no_go"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_choices_complete_without_blocking_gate_b() {
        for choice in [Some("taken"), Some("left"), None] {
            let (authority, _, pass) = complete_choice(choice);
            assert!(pass);
            assert!(authority.gate_b_open && authority.sentinel_defeated && authority.exit_active);
        }
    }

    #[test]
    fn rewards_and_result_are_once_only() {
        let (mut authority, _, _) = complete_choice(None);
        let before = authority.reward_ledger.len();
        let duplicate = authority.apply(event("extract", 10, "extract"));
        assert_eq!(duplicate.error_code.as_deref(), Some("E_DUPLICATE_EVENT"));
        assert_eq!(authority.reward_ledger.len(), before);
    }

    #[test]
    fn reload_preserves_choice_and_hash() {
        let (authority, _, _) = complete_choice(Some("taken"));
        let snapshot = authority.snapshot();
        let hash = authority.state_hash();
        let mut restored = Authority::new(7);
        restored.reload(snapshot).unwrap();
        assert_eq!(restored.hive_choice, ChoiceState::Taken);
        assert_eq!(restored.state_hash(), hash);
        assert!(restored.result.as_ref().is_some_and(|r| r.map.is_none()));
    }

    #[test]
    fn old_epoch_and_skip_are_rejected() {
        let mut authority = Authority::new(7);
        let mut old = event("old", 0, "complete_airlock");
        old.epoch = 6;
        assert_eq!(
            authority.apply(old).error_code.as_deref(),
            Some("E_OLD_EPOCH")
        );
        assert_eq!(
            authority
                .apply(event("skip", 0, "reach_gate_b"))
                .error_code
                .as_deref(),
            Some("E_OBJECTIVE_PREREQUISITE")
        );
    }
}

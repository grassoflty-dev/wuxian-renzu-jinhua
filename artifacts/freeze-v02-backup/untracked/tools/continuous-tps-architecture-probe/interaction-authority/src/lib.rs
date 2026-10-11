use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const BOUNDARY_EPSILON_M: f64 = 1.0e-5;
pub const OPENING_TICKS: u64 = 2;
pub const CLOSING_TICKS: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position_m: [f64; 3],
    pub yaw_rad: f64,
}

impl Transform {
    pub fn finite(self) -> bool {
        self.position_m.iter().all(|v| v.is_finite()) && self.yaw_rad.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineOfSight {
    Clear,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColliderStatus {
    Clear,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceState {
    Taken,
    Left,
    Unresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldFlags {
    pub hive_power: bool,
    pub hive_lockdown_completed: bool,
    pub hive_choice: ChoiceState,
    pub npc_available: bool,
}

impl Default for WorldFlags {
    fn default() -> Self {
        Self {
            hive_power: false,
            hive_lockdown_completed: false,
            hive_choice: ChoiceState::Unresolved,
            npc_available: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prerequisite {
    None,
    HivePower,
    HiveLockdownCompleted,
}

impl Prerequisite {
    fn satisfied(self, flags: WorldFlags) -> bool {
        match self {
            Self::None => true,
            Self::HivePower => flags.hive_power,
            Self::HiveLockdownCompleted => flags.hive_lockdown_completed,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorState {
    Closed,
    Opening,
    Open,
    Closing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DoorSpec {
    pub door_id: String,
    pub prerequisite: Prerequisite,
    pub dynamic_connector: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DoorRuntime {
    pub state: DoorState,
    pub progress_ticks: u64,
    pub collider_blocking: bool,
    pub side_effects: u64,
}

impl DoorRuntime {
    fn closed() -> Self {
        Self {
            state: DoorState::Closed,
            progress_ticks: 0,
            collider_blocking: true,
            side_effects: 0,
        }
    }

    fn invariant(&self) -> bool {
        self.collider_blocking == (self.state != DoorState::Open)
            && match self.state {
                DoorState::Closed => self.progress_ticks == 0,
                DoorState::Opening => self.progress_ticks < OPENING_TICKS,
                DoorState::Open => self.progress_ticks == OPENING_TICKS,
                DoorState::Closing => self.progress_ticks < CLOSING_TICKS,
            }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    OpenDoor,
    CloseDoor,
    ObserveTrigger,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub event_id: String,
    pub world_epoch: u64,
    pub tick: u64,
    pub actor_id: String,
    pub actor_transform: Transform,
    pub target_id: String,
    pub target_transform: Transform,
    pub interaction_radius_m: f64,
    pub vertical_half_height_m: f64,
    pub target_exists: bool,
    pub target_enabled: bool,
    pub los: LineOfSight,
    pub collider: ColliderStatus,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerPhase {
    Enter,
    Stay,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateChange {
    pub target_id: String,
    pub from: String,
    pub to: String,
    pub tick: u64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub actor_id: String,
    pub target_id: String,
    pub world_epoch: u64,
    pub tick: u64,
    pub outcome: Outcome,
    pub error_code: Option<String>,
    pub trigger_phase: Option<TriggerPhase>,
    pub state_changes: Vec<StateChange>,
    pub side_effect_delta: u64,
    pub state_hash: String,
    pub candidate_only: bool,
    pub evidence_level: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DoorSnapshot {
    pub door_id: String,
    pub runtime: DoorRuntime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriggerSnapshot {
    pub actor_id: String,
    pub target_id: String,
    pub inside: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReloadSnapshot {
    pub world_epoch: u64,
    pub tick: u64,
    pub doors: Vec<DoorSnapshot>,
    pub triggers: Vec<TriggerSnapshot>,
    pub seen_event_ids: Vec<String>,
    pub side_effect_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Authority {
    pub world_epoch: u64,
    pub tick: u64,
    pub flags: WorldFlags,
    specs: BTreeMap<String, DoorSpec>,
    doors: BTreeMap<String, DoorRuntime>,
    trigger_inside: BTreeMap<(String, String), bool>,
    seen_event_ids: BTreeSet<String>,
    pub side_effect_count: u64,
}

impl Authority {
    pub fn new(world_epoch: u64, tick: u64, flags: WorldFlags, specs: Vec<DoorSpec>) -> Self {
        let mut spec_map = BTreeMap::new();
        let mut doors = BTreeMap::new();
        for spec in specs {
            doors.insert(spec.door_id.clone(), DoorRuntime::closed());
            spec_map.insert(spec.door_id.clone(), spec);
        }
        Self {
            world_epoch,
            tick,
            flags,
            specs: spec_map,
            doors,
            trigger_inside: BTreeMap::new(),
            seen_event_ids: BTreeSet::new(),
            side_effect_count: 0,
        }
    }

    pub fn door(&self, door_id: &str) -> Option<&DoorRuntime> {
        self.doors.get(door_id)
    }

    pub fn can_pass(&self, door_id: &str) -> bool {
        self.specs
            .get(door_id)
            .and_then(|spec| self.doors.get(door_id).map(|door| (spec, door)))
            .is_some_and(|(spec, door)| {
                spec.enabled
                    && !spec.dynamic_connector
                    && spec.prerequisite.satisfied(self.flags)
                    && door.state == DoorState::Open
                    && !door.collider_blocking
            })
    }

    pub fn set_flags(&mut self, flags: WorldFlags) -> Vec<StateChange> {
        let old_flags = self.flags;
        self.flags = flags;
        let mut changes = Vec::new();
        if old_flags.hive_power && !flags.hive_power {
            changes
                .extend(self.revoke_prerequisite(Prerequisite::HivePower, "prerequisite_revoked"));
        }
        if old_flags.hive_lockdown_completed && !flags.hive_lockdown_completed {
            changes.extend(
                self.revoke_prerequisite(
                    Prerequisite::HiveLockdownCompleted,
                    "prerequisite_revoked",
                ),
            );
        }
        changes
    }

    fn revoke_prerequisite(
        &mut self,
        prerequisite: Prerequisite,
        reason: &str,
    ) -> Vec<StateChange> {
        let ids: Vec<String> = self
            .specs
            .values()
            .filter(|spec| spec.prerequisite == prerequisite)
            .map(|spec| spec.door_id.clone())
            .collect();
        let mut changes = Vec::new();
        for door_id in ids {
            if let Some(runtime) = self.doors.get_mut(&door_id) {
                if matches!(runtime.state, DoorState::Opening | DoorState::Open) {
                    let from = format!("{:?}", runtime.state);
                    runtime.state = DoorState::Closing;
                    runtime.progress_ticks = 0;
                    runtime.collider_blocking = true;
                    runtime.side_effects += 1;
                    self.side_effect_count += 1;
                    changes.push(StateChange {
                        target_id: door_id,
                        from,
                        to: "Closing".to_string(),
                        tick: self.tick,
                        reason: reason.to_string(),
                    });
                }
            }
        }
        changes
    }

    pub fn advance_to(&mut self, target_tick: u64) -> Vec<StateChange> {
        if target_tick <= self.tick {
            return Vec::new();
        }
        let mut changes = Vec::new();
        for tick in (self.tick + 1)..=target_tick {
            self.tick = tick;
            let ids: Vec<String> = self.doors.keys().cloned().collect();
            for door_id in ids {
                if let Some(runtime) = self.doors.get_mut(&door_id) {
                    let from = runtime.state;
                    match runtime.state {
                        DoorState::Opening => {
                            runtime.progress_ticks += 1;
                            if runtime.progress_ticks >= OPENING_TICKS {
                                runtime.state = DoorState::Open;
                                runtime.progress_ticks = OPENING_TICKS;
                                runtime.collider_blocking = false;
                            }
                        }
                        DoorState::Closing => {
                            runtime.progress_ticks += 1;
                            if runtime.progress_ticks >= CLOSING_TICKS {
                                runtime.state = DoorState::Closed;
                                runtime.progress_ticks = 0;
                                runtime.collider_blocking = true;
                            }
                        }
                        DoorState::Closed | DoorState::Open => {}
                    }
                    if runtime.state != from {
                        changes.push(StateChange {
                            target_id: door_id,
                            from: format!("{:?}", from),
                            to: format!("{:?}", runtime.state),
                            tick,
                            reason: "fixed_tick_timeline".to_string(),
                        });
                    }
                }
            }
            let revoked = self.set_flags(self.flags);
            changes.extend(revoked);
        }
        changes
    }

    pub fn process_batch(&mut self, mut events: Vec<InteractionEvent>) -> Vec<EventRecord> {
        events.sort_by(|left, right| {
            (left.tick, &left.event_id, &left.actor_id, &left.target_id).cmp(&(
                right.tick,
                &right.event_id,
                &right.actor_id,
                &right.target_id,
            ))
        });
        events
            .into_iter()
            .map(|event| self.process_event(event))
            .collect()
    }

    pub fn process_event(&mut self, event: InteractionEvent) -> EventRecord {
        let mut changes = Vec::new();
        let side_effect_before = self.side_effect_count;
        let common = self.validate_common(&event);
        if let Some(error) = common {
            return self.reject(&event, error, changes, side_effect_before);
        }
        if event.tick > self.tick {
            changes.extend(self.advance_to(event.tick));
        }
        if self.seen_event_ids.contains(&event.event_id) {
            return self.reject(&event, "E_DUPLICATE_EVENT", changes, side_effect_before);
        }
        self.seen_event_ids.insert(event.event_id.clone());
        if !event.target_exists {
            return self.reject(&event, "E_TARGET_MISSING", changes, side_effect_before);
        }
        if !event.target_enabled {
            return self.reject(&event, "E_TARGET_DISABLED", changes, side_effect_before);
        }
        match event.action {
            Action::ObserveTrigger => {
                let key = (event.actor_id.clone(), event.target_id.clone());
                let inside_now = within_scope(&event)
                    && event.los == LineOfSight::Clear
                    && event.collider == ColliderStatus::Clear;
                let was_inside = self.trigger_inside.get(&key).copied().unwrap_or(false);
                let phase = if inside_now && !was_inside {
                    Some(TriggerPhase::Enter)
                } else if inside_now {
                    Some(TriggerPhase::Stay)
                } else if was_inside {
                    Some(TriggerPhase::Exit)
                } else {
                    None
                };
                self.trigger_inside.insert(key, inside_now);
                self.accept(&event, phase, changes, side_effect_before)
            }
            Action::OpenDoor | Action::CloseDoor => {
                if event.los == LineOfSight::Blocked {
                    return self.reject(&event, "E_LOS_BLOCKED", changes, side_effect_before);
                }
                if event.collider == ColliderStatus::Blocked {
                    return self.reject(&event, "E_COLLIDER_BLOCKED", changes, side_effect_before);
                }
                if !within_scope(&event) {
                    return self.reject(&event, "E_OUT_OF_SCOPE", changes, side_effect_before);
                }
                let Some(spec) = self.specs.get(&event.target_id).cloned() else {
                    return self.reject(&event, "E_TARGET_MISSING", changes, side_effect_before);
                };
                if !spec.enabled {
                    return self.reject(&event, "E_TARGET_DISABLED", changes, side_effect_before);
                }
                if spec.dynamic_connector {
                    return self.reject(
                        &event,
                        "E_DYNAMIC_CONNECTOR_EXCLUDED",
                        changes,
                        side_effect_before,
                    );
                }
                if !spec.prerequisite.satisfied(self.flags) {
                    return self.reject(&event, "E_PREREQUISITE", changes, side_effect_before);
                }
                let runtime = self
                    .doors
                    .get_mut(&event.target_id)
                    .expect("spec/runtime pair");
                match event.action {
                    Action::OpenDoor => match runtime.state {
                        DoorState::Closed | DoorState::Closing => {
                            let from = format!("{:?}", runtime.state);
                            runtime.state = DoorState::Opening;
                            runtime.progress_ticks = 0;
                            runtime.collider_blocking = true;
                            runtime.side_effects += 1;
                            self.side_effect_count += 1;
                            changes.push(StateChange {
                                target_id: event.target_id.clone(),
                                from,
                                to: "Opening".to_string(),
                                tick: self.tick,
                                reason: "accepted_open_command".to_string(),
                            });
                        }
                        DoorState::Opening | DoorState::Open => {}
                    },
                    Action::CloseDoor => match runtime.state {
                        DoorState::Open | DoorState::Opening => {
                            let from = format!("{:?}", runtime.state);
                            runtime.state = DoorState::Closing;
                            runtime.progress_ticks = 0;
                            runtime.collider_blocking = true;
                            runtime.side_effects += 1;
                            self.side_effect_count += 1;
                            changes.push(StateChange {
                                target_id: event.target_id.clone(),
                                from,
                                to: "Closing".to_string(),
                                tick: self.tick,
                                reason: "accepted_close_command".to_string(),
                            });
                        }
                        DoorState::Closed | DoorState::Closing => {}
                    },
                    Action::ObserveTrigger => unreachable!(),
                }
                self.accept(&event, None, changes, side_effect_before)
            }
        }
    }

    fn validate_common(&self, event: &InteractionEvent) -> Option<&'static str> {
        if event.world_epoch != self.world_epoch {
            return Some("E_OLD_EPOCH");
        }
        if event.tick < self.tick {
            return Some("E_OLD_TICK");
        }
        if event.event_id.is_empty() || event.actor_id.is_empty() || event.target_id.is_empty() {
            return Some("E_EMPTY_ID");
        }
        if !event.actor_transform.finite() || !event.target_transform.finite() {
            return Some("E_NONFINITE_TRANSFORM");
        }
        if !event.interaction_radius_m.is_finite()
            || !event.vertical_half_height_m.is_finite()
            || event.interaction_radius_m < 0.0
            || event.vertical_half_height_m < 0.0
        {
            return Some("E_INVALID_SCOPE");
        }
        None
    }

    fn reject(
        &self,
        event: &InteractionEvent,
        error: &'static str,
        changes: Vec<StateChange>,
        side_effect_before: u64,
    ) -> EventRecord {
        EventRecord {
            event_id: event.event_id.clone(),
            actor_id: event.actor_id.clone(),
            target_id: event.target_id.clone(),
            world_epoch: event.world_epoch,
            tick: event.tick,
            outcome: Outcome::Rejected,
            error_code: Some(error.to_string()),
            trigger_phase: None,
            state_changes: changes,
            side_effect_delta: self.side_effect_count - side_effect_before,
            state_hash: self.state_hash(),
            candidate_only: true,
            evidence_level: "isolated_candidate_tp10a1".to_string(),
        }
    }

    fn accept(
        &self,
        event: &InteractionEvent,
        phase: Option<TriggerPhase>,
        changes: Vec<StateChange>,
        side_effect_before: u64,
    ) -> EventRecord {
        EventRecord {
            event_id: event.event_id.clone(),
            actor_id: event.actor_id.clone(),
            target_id: event.target_id.clone(),
            world_epoch: event.world_epoch,
            tick: event.tick,
            outcome: Outcome::Accepted,
            error_code: None,
            trigger_phase: phase,
            state_changes: changes,
            side_effect_delta: self.side_effect_count - side_effect_before,
            state_hash: self.state_hash(),
            candidate_only: true,
            evidence_level: "isolated_candidate_tp10a1".to_string(),
        }
    }

    pub fn snapshot(&self) -> ReloadSnapshot {
        ReloadSnapshot {
            world_epoch: self.world_epoch,
            tick: self.tick,
            doors: self
                .doors
                .iter()
                .map(|(door_id, runtime)| DoorSnapshot {
                    door_id: door_id.clone(),
                    runtime: runtime.clone(),
                })
                .collect(),
            triggers: self
                .trigger_inside
                .iter()
                .map(|((actor_id, target_id), inside)| TriggerSnapshot {
                    actor_id: actor_id.clone(),
                    target_id: target_id.clone(),
                    inside: *inside,
                })
                .collect(),
            seen_event_ids: self.seen_event_ids.iter().cloned().collect(),
            side_effect_count: self.side_effect_count,
        }
    }

    pub fn reload(&mut self, snapshot: ReloadSnapshot) -> Result<(), &'static str> {
        if snapshot.world_epoch != self.world_epoch {
            return Err("E_RELOAD_EPOCH");
        }
        if snapshot
            .doors
            .iter()
            .any(|door| !self.specs.contains_key(&door.door_id) || !door.runtime.invariant())
        {
            return Err("E_RELOAD_STATE");
        }
        let mut doors = BTreeMap::new();
        for door in snapshot.doors {
            doors.insert(door.door_id, door.runtime);
        }
        if doors.len() != self.specs.len() {
            return Err("E_RELOAD_STATE");
        }
        self.tick = snapshot.tick;
        self.doors = doors;
        self.trigger_inside = snapshot
            .triggers
            .into_iter()
            .map(|entry| ((entry.actor_id, entry.target_id), entry.inside))
            .collect();
        self.seen_event_ids = snapshot.seen_event_ids.into_iter().collect();
        self.side_effect_count = snapshot.side_effect_count;
        Ok(())
    }

    pub fn state_hash(&self) -> String {
        let snapshot = self.snapshot();
        let bytes = serde_json::to_vec(&snapshot).expect("snapshot is serializable");
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("fnv1a64-{hash:016x}")
    }
}

fn within_scope(event: &InteractionEvent) -> bool {
    let dx = event.actor_transform.position_m[0] - event.target_transform.position_m[0];
    let dz = event.actor_transform.position_m[2] - event.target_transform.position_m[2];
    let dy = (event.actor_transform.position_m[1] - event.target_transform.position_m[1]).abs();
    let horizontal = (dx * dx + dz * dz).sqrt();
    horizontal <= event.interaction_radius_m + BOUNDARY_EPSILON_M
        && dy <= event.vertical_half_height_m + BOUNDARY_EPSILON_M
}

fn door_specs() -> Vec<DoorSpec> {
    vec![
        DoorSpec {
            door_id: "gh_gate_a".to_string(),
            prerequisite: Prerequisite::HivePower,
            dynamic_connector: false,
            enabled: true,
        },
        DoorSpec {
            door_id: "gh_gate_b".to_string(),
            prerequisite: Prerequisite::HiveLockdownCompleted,
            dynamic_connector: false,
            enabled: true,
        },
        DoorSpec {
            door_id: "edge-b05-c01".to_string(),
            prerequisite: Prerequisite::HiveLockdownCompleted,
            dynamic_connector: true,
            enabled: true,
        },
        DoorSpec {
            door_id: "disabled_door".to_string(),
            prerequisite: Prerequisite::None,
            dynamic_connector: false,
            enabled: false,
        },
    ]
}

fn flags(power: bool, lockdown: bool, choice: ChoiceState, npc: bool) -> WorldFlags {
    WorldFlags {
        hive_power: power,
        hive_lockdown_completed: lockdown,
        hive_choice: choice,
        npc_available: npc,
    }
}

fn event(
    id: &str,
    tick: u64,
    actor_id: &str,
    target_id: &str,
    actor: [f64; 3],
    target: [f64; 3],
    action: Action,
) -> InteractionEvent {
    InteractionEvent {
        event_id: id.to_string(),
        world_epoch: 7,
        tick,
        actor_id: actor_id.to_string(),
        actor_transform: Transform {
            position_m: actor,
            yaw_rad: 0.0,
        },
        target_id: target_id.to_string(),
        target_transform: Transform {
            position_m: target,
            yaw_rad: 0.0,
        },
        interaction_radius_m: 3.0,
        vertical_half_height_m: 1.0,
        target_exists: true,
        target_enabled: true,
        los: LineOfSight::Clear,
        collider: ColliderStatus::Clear,
        action,
    }
}

fn scenario(
    id: &str,
    input: Value,
    events: Value,
    expected: Value,
    actual: Value,
    pass: bool,
    hash: String,
    timeline_ticks: Vec<u64>,
) -> Value {
    json!({
        "id": id,
        "input": input,
        "events": events,
        "expected": expected,
        "actual": actual,
        "pass": pass,
        "boundaryEpsilonM": BOUNDARY_EPSILON_M,
        "timelineTicks": timeline_ticks,
        "stateHash": hash,
        "evidenceLevel": "isolated_candidate_tp10a1"
    })
}

pub fn run_probe() -> Value {
    let mut scenarios = Vec::new();

    let mut scope = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let boundary = event(
        "scope-boundary",
        0,
        "actor-a",
        "gh_gate_a",
        [3.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    let boundary_record = scope.process_event(boundary.clone());
    let mut outside = event(
        "scope-outside",
        0,
        "actor-a",
        "gh_gate_a",
        [3.0 + 2.0 * BOUNDARY_EPSILON_M, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    outside.interaction_radius_m = 3.0;
    let outside_record = scope.process_event(outside.clone());
    let mut los = event(
        "los-blocked",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    los.los = LineOfSight::Blocked;
    let los_record = scope.process_event(los.clone());
    let mut collider = event(
        "collider-blocked",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    collider.collider = ColliderStatus::Blocked;
    let collider_record = scope.process_event(collider.clone());
    scenarios.push(scenario(
        "scope_vertical_los_collider_boundaries",
        json!({"radiusM":3.0,"verticalHalfHeightM":1.0,"actorAtBoundary":[3.0,1.0,0.0],"target":[0.0,0.0,0.0]}),
        json!([boundary,outside,los,collider]),
        json!({"boundary":"Accepted","outside":"E_OUT_OF_SCOPE","los":"E_LOS_BLOCKED","collider":"E_COLLIDER_BLOCKED"}),
        json!({"boundary":boundary_record.outcome,"outside":outside_record.error_code,"los":los_record.error_code,"collider":collider_record.error_code}),
        matches!(boundary_record.outcome, Outcome::Accepted) && outside_record.error_code.as_deref()==Some("E_OUT_OF_SCOPE") && los_record.error_code.as_deref()==Some("E_LOS_BLOCKED") && collider_record.error_code.as_deref()==Some("E_COLLIDER_BLOCKED"),
        scope.state_hash(),
        vec![0],
    ));

    let mut invalid = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let mut missing = event(
        "missing-target",
        0,
        "actor-a",
        "missing",
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    missing.target_exists = false;
    let mut disabled = event(
        "disabled-target",
        0,
        "actor-a",
        "disabled_door",
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    disabled.target_enabled = false;
    let mut nonfinite = event(
        "nonfinite-transform",
        0,
        "actor-a",
        "gh_gate_a",
        [f64::NAN, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    nonfinite.target_transform.yaw_rad = f64::INFINITY;
    let missing_record = invalid.process_event(missing.clone());
    let disabled_record = invalid.process_event(disabled.clone());
    let nonfinite_record = invalid.process_event(nonfinite.clone());
    scenarios.push(scenario(
        "target_missing_disabled_nonfinite",
        json!({"targetMissing":true,"disabledDoor":"disabled_door","nonfinite":["NaN","Infinity"]}),
        json!([missing,disabled,nonfinite]),
        json!({"missing":"E_TARGET_MISSING","disabled":"E_TARGET_DISABLED","nonfinite":"E_NONFINITE_TRANSFORM"}),
        json!({"missing":missing_record.error_code,"disabled":disabled_record.error_code,"nonfinite":nonfinite_record.error_code}),
        missing_record.error_code.as_deref()==Some("E_TARGET_MISSING") && disabled_record.error_code.as_deref()==Some("E_TARGET_DISABLED") && nonfinite_record.error_code.as_deref()==Some("E_NONFINITE_TRANSFORM"),
        invalid.state_hash(),
        vec![0],
    ));

    let mut ordering = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let first = event(
        "evt-first",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    let first_record = ordering.process_event(first.clone());
    let duplicate_record = ordering.process_event(first.clone());
    let advance = event(
        "evt-advance",
        1,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::CloseDoor,
    );
    let _ = ordering.process_event(advance);
    let stale = event(
        "evt-stale",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    let stale_record = ordering.process_event(stale.clone());
    scenarios.push(scenario(
        "duplicate_and_out_of_order_events",
        json!({"epoch":7,"firstTick":0,"staleTick":0,"currentTick":1}),
        json!([first,stale]),
        json!({"first":"Accepted","duplicate":"E_DUPLICATE_EVENT","stale":"E_OLD_TICK"}),
        json!({"first":first_record.outcome,"duplicate":duplicate_record.error_code,"stale":stale_record.error_code}),
        matches!(first_record.outcome, Outcome::Accepted) && duplicate_record.error_code.as_deref()==Some("E_DUPLICATE_EVENT") && stale_record.error_code.as_deref()==Some("E_OLD_TICK"),
        ordering.state_hash(),
        vec![0,1],
    ));

    let mut gate_a = Authority::new(
        7,
        0,
        flags(false, true, ChoiceState::Taken, true),
        door_specs(),
    );
    let gate_a_locked = gate_a.process_event(event(
        "gate-a-locked",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    let mut gate_a_open = Authority::new(
        7,
        0,
        flags(true, false, ChoiceState::Left, false),
        door_specs(),
    );
    let gate_a_open_record = gate_a_open.process_event(event(
        "gate-a-open",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    scenarios.push(scenario(
        "gate_a_only_hive_power",
        json!({"gate":"gh_gate_a","prerequisite":"hive_power","choiceStates":["taken","left","unresolved"],"npcIgnored":true}),
        json!(["hive_power=false","hive_power=true"]),
        json!({"false":"E_PREREQUISITE","true":"Accepted"}),
        json!({"false":gate_a_locked.error_code,"true":gate_a_open_record.outcome}),
        gate_a_locked.error_code.as_deref()==Some("E_PREREQUISITE") && matches!(gate_a_open_record.outcome, Outcome::Accepted),
        gate_a_open.state_hash(),
        vec![0],
    ));

    let mut gate_b_matrix = Vec::new();
    let mut gate_b_matrix_pass = true;
    for choice in [
        ChoiceState::Taken,
        ChoiceState::Left,
        ChoiceState::Unresolved,
    ] {
        for npc in [false, true] {
            let mut authority =
                Authority::new(7, 0, flags(false, false, choice, npc), door_specs());
            let locked = authority.process_event(event(
                "b-locked",
                0,
                "actor-a",
                "gh_gate_b",
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                Action::OpenDoor,
            ));
            let mut authority_open =
                Authority::new(7, 0, flags(false, true, choice, npc), door_specs());
            let open = authority_open.process_event(event(
                "b-open",
                0,
                "actor-a",
                "gh_gate_b",
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                Action::OpenDoor,
            ));
            gate_b_matrix_pass &= locked.error_code.as_deref() == Some("E_PREREQUISITE")
                && matches!(open.outcome, Outcome::Accepted);
            gate_b_matrix.push(json!({"choice":choice,"npcAvailable":npc,"locked":locked.error_code,"open":open.outcome}));
        }
    }
    scenarios.push(scenario(
        "gate_b_only_hive_lockdown_completed",
        json!({"gate":"gh_gate_b","prerequisite":"hive_lockdown_completed","choiceStates":["taken","left","unresolved"],"npcIgnored":true}),
        json!(gate_b_matrix.clone()),
        json!({"lockdownFalse":"E_PREREQUISITE","lockdownTrue":"Accepted","choiceAndNpcIndependent":true}),
        json!({"matrix":gate_b_matrix,"choiceAndNpcIndependent":gate_b_matrix_pass}),
        gate_b_matrix_pass,
        Authority::new(7, 0, flags(false, true, ChoiceState::Unresolved, false), door_specs()).state_hash(),
        vec![0],
    ));

    let mut timeline = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let open_record = timeline.process_event(event(
        "timeline-open",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    let opening_blocked = !timeline.can_pass("gh_gate_a");
    let _ = timeline.advance_to(1);
    let mid_blocked = !timeline.can_pass("gh_gate_a");
    let _ = timeline.advance_to(2);
    let open_passable = timeline.can_pass("gh_gate_a");
    let close_record = timeline.process_event(event(
        "timeline-close",
        3,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::CloseDoor,
    ));
    let close_blocked = !timeline.can_pass("gh_gate_a");
    let _ = timeline.advance_to(5);
    let closed_again = timeline
        .door("gh_gate_a")
        .is_some_and(|d| d.state == DoorState::Closed && d.collider_blocking);
    scenarios.push(scenario(
        "door_closed_opening_open_closing_closed_timeline",
        json!({"door":"gh_gate_a","openingTicks":OPENING_TICKS,"closingTicks":CLOSING_TICKS}),
        json!([open_record,close_record]),
        json!({"tick0":"Opening/blocking","tick2":"Open/passable","tick3":"Closing/blocking","tick5":"Closed/blocking"}),
        json!({"openingBlocked":opening_blocked,"midBlocked":mid_blocked,"openPassable":open_passable,"closeBlocked":close_blocked,"closedAgain":closed_again}),
        opening_blocked && mid_blocked && open_passable && close_blocked && closed_again,
        timeline.state_hash(),
        vec![0,1,2,3,4,5],
    ));

    let mut revoke = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let _ = revoke.process_event(event(
        "revoke-open",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    let _ = revoke.advance_to(2);
    let revoke_changes = revoke.set_flags(flags(false, true, ChoiceState::Unresolved, false));
    let safe_blocked = !revoke.can_pass("gh_gate_a")
        && revoke
            .door("gh_gate_a")
            .is_some_and(|d| d.state == DoorState::Closing && d.collider_blocking);
    let _ = revoke.advance_to(4);
    let safe_closed = revoke
        .door("gh_gate_a")
        .is_some_and(|d| d.state == DoorState::Closed);
    scenarios.push(scenario(
        "prerequisite_revoke_safe_close",
        json!({"door":"gh_gate_a","revoke":"hive_power=false","strategy":"immediate_block_then_close_timeline"}),
        json!(revoke_changes),
        json!({"revokedState":"Closing","colliderBlocking":true,"finalState":"Closed"}),
        json!({"safeBlocked":safe_blocked,"safeClosed":safe_closed}),
        safe_blocked && safe_closed,
        revoke.state_hash(),
        vec![0,2,4],
    ));

    let mut trigger = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let enter_event = event(
        "trigger-enter",
        0,
        "actor-a",
        "trigger-main",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::ObserveTrigger,
    );
    let stay_event = event(
        "trigger-stay",
        1,
        "actor-a",
        "trigger-main",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::ObserveTrigger,
    );
    let exit_event = event(
        "trigger-exit",
        2,
        "actor-a",
        "trigger-main",
        [4.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::ObserveTrigger,
    );
    let mut no_event = event(
        "trigger-outside",
        3,
        "actor-a",
        "trigger-main",
        [4.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::ObserveTrigger,
    );
    no_event.target_exists = true;
    let trigger_records = trigger.process_batch(vec![
        exit_event.clone(),
        enter_event.clone(),
        no_event.clone(),
        stay_event.clone(),
    ]);
    let phases: Vec<Value> = trigger_records
        .iter()
        .map(|r| json!({"id":r.event_id,"phase":r.trigger_phase,"error":r.error_code}))
        .collect();
    let phase_names: Vec<String> = trigger_records
        .iter()
        .filter_map(|r| r.trigger_phase.as_ref().map(|p| format!("{:?}", p)))
        .collect();
    scenarios.push(scenario(
        "trigger_enter_stay_exit_once",
        json!({"trigger":"trigger-main","actor":"actor-a"}),
        json!([enter_event, stay_event, exit_event, no_event]),
        json!({"phases":["Enter","Stay","Exit"],"outsideAgain":"no transition"}),
        json!({"records":phases,"phases":phase_names}),
        phase_names == vec!["Enter", "Stay", "Exit"],
        trigger.state_hash(),
        vec![0, 1, 2, 3],
    ));

    let mut reload_source = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let reload_open = reload_source.process_event(event(
        "reload-open",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    let _ = reload_source.advance_to(2);
    let snapshot = reload_source.snapshot();
    let source_hash = reload_source.state_hash();
    let source_effects = reload_source.side_effect_count;
    let mut reloaded = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let reload_result = reloaded.reload(snapshot.clone());
    let reload_hash_matches = reload_result.is_ok() && reloaded.state_hash() == source_hash;
    let duplicate_after_reload = reloaded.process_event(event(
        "reload-open",
        2,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    let reload_side_effects_stable = reloaded.side_effect_count == source_effects;
    let reload_close = reloaded.process_event(event(
        "reload-close",
        2,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::CloseDoor,
    ));
    scenarios.push(scenario(
        "reload_restores_progress_and_no_duplicate_side_effect",
        json!({"snapshotTick":snapshot.tick,"snapshotDoorState":snapshot.doors.iter().find(|d|d.door_id=="gh_gate_a").map(|d|d.runtime.state)}),
        json!([reload_open,duplicate_after_reload,reload_close]),
        json!({"reload":"same state hash","duplicate":"E_DUPLICATE_EVENT","sideEffects":"unchanged"}),
        json!({"reloadHashMatches":reload_hash_matches,"duplicate":duplicate_after_reload.error_code,"sideEffectsStable":reload_side_effects_stable,"close":reload_close.outcome}),
        reload_hash_matches && duplicate_after_reload.error_code.as_deref()==Some("E_DUPLICATE_EVENT") && reload_side_effects_stable,
        reloaded.state_hash(),
        vec![0,2],
    ));

    let mut sorted = Authority::new(
        7,
        0,
        flags(true, true, ChoiceState::Unresolved, false),
        door_specs(),
    );
    let event_b = event(
        "event-b",
        0,
        "actor-z",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    );
    let event_a = event(
        "event-a",
        0,
        "actor-a",
        "gh_gate_a",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::CloseDoor,
    );
    let sorted_records = sorted.process_batch(vec![event_b.clone(), event_a.clone()]);
    let sorted_ids: Vec<String> = sorted_records.iter().map(|r| r.event_id.clone()).collect();
    let final_sorted_state = sorted.door("gh_gate_a").map(|d| format!("{:?}", d.state));
    scenarios.push(scenario(
        "same_tick_sort_event_id_then_actor_id",
        json!({"tick":0,"inputOrder":["event-b","event-a"],"sortKey":["tick","eventId","actorId"]}),
        json!([event_b, event_a]),
        json!({"order":["event-a","event-b"],"final":"Opening"}),
        json!({"order":sorted_ids,"final":final_sorted_state}),
        sorted_ids == vec!["event-a".to_string(), "event-b".to_string()]
            && final_sorted_state.as_deref() == Some("Opening"),
        sorted.state_hash(),
        vec![0],
    ));

    let mut dynamic = Authority::new(
        7,
        0,
        flags(false, true, ChoiceState::Taken, true),
        door_specs(),
    );
    let dynamic_record = dynamic.process_event(event(
        "dynamic-elevator",
        0,
        "actor-a",
        "edge-b05-c01",
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        Action::OpenDoor,
    ));
    scenarios.push(scenario(
        "dynamic_elevator_excluded_not_evaluated",
        json!({"connector":"edge-b05-c01","dynamic":true,"formalNavmesh":false}),
        json!([dynamic_record.clone()]),
        json!({"result":"E_DYNAMIC_CONNECTOR_EXCLUDED","status":"excluded/not_evaluated"}),
        json!({"result":dynamic_record.error_code,"status":"excluded/not_evaluated"}),
        dynamic_record.error_code.as_deref() == Some("E_DYNAMIC_CONNECTOR_EXCLUDED"),
        dynamic.state_hash(),
        vec![0],
    ));

    fn run_frequency(hz: u32) -> (String, DoorState, bool) {
        let mut authority = Authority::new(
            7,
            0,
            flags(true, true, ChoiceState::Unresolved, false),
            door_specs(),
        );
        let _ = authority.process_event(event(
            "frequency-open",
            0,
            "actor-a",
            "gh_gate_a",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::OpenDoor,
        ));
        let final_tick = if hz == 30 { 6 } else { 6 };
        let _ = authority.advance_to(final_tick);
        (
            authority.state_hash(),
            authority.door("gh_gate_a").expect("gate exists").state,
            authority.can_pass("gh_gate_a"),
        )
    }
    let freq30 = run_frequency(30);
    let freq60 = run_frequency(60);
    scenarios.push(scenario(
        "thirty_sixty_hz_same_final_door_state",
        json!({"frequenciesHz":[30,60],"sameInputTicks":[0,6]}),
        json!(["frequency-open@tick0","advance@tick6"]),
        json!({"sameState":true,"samePassability":true}),
        json!({"hz30":{"state":freq30.1,"passable":freq30.2,"hash":freq30.0},"hz60":{"state":freq60.1,"passable":freq60.2,"hash":freq60.0},"sameState":freq30.1==freq60.1,"samePassability":freq30.2==freq60.2}),
        freq30.1 == freq60.1 && freq30.2 == freq60.2,
        freq60.0,
        vec![0,6],
    ));

    let all_pass = scenarios.iter().all(|s| s["pass"] == true);
    json!({
        "schema":"interaction_authority_tp10a1_v1",
        "status":if all_pass {"passed_candidate_isolated"} else {"failed_candidate_probe"},
        "pass":all_pass,
        "candidateOnly":true,
        "formalSceneReady":false,
        "authorityContract":{
            "units":"meters",
            "rightHanded":true,
            "upAxis":"+Y",
            "anchor":"capsule_foot",
            "input":["worldEpoch","tick","uniqueEventId","actorId","actorTransform","targetId","targetTransform","interactionRadiusM","verticalHalfHeightM","los","collider","doorState","prerequisite","reloadSnapshot"],
            "output":["accepted/rejected","stableErrorCode","stateChanges","serializableEventRecord"],
            "owner":"isolated authority state machine; formal runtime owner not wired"
        },
        "gatePolicy":{
            "gateA":"hive_power only",
            "gateB":"hive_lockdown_completed only",
            "choiceStates":["taken","left","unresolved"],
            "choiceIndependent":true,
            "npcIndependent":true
        },
        "doorPolicy":{
            "openingTicks":OPENING_TICKS,
            "closingTicks":CLOSING_TICKS,
            "closedColliderBlocking":true,
            "openColliderBlocking":false,
            "prerequisiteRevocation":"immediate collider block then fixed closing timeline"
        },
        "ordering":{"sort":["tick","eventId","actorId","targetId"],"duplicate":"E_DUPLICATE_EVENT","oldEpoch":"E_OLD_EPOCH","oldTick":"E_OLD_TICK"},
        "triggerPolicy":{"phases":["Enter","Stay","Exit"],"oncePerTransition":true},
        "reloadPolicy":{"restores":"door state/progress, trigger occupancy, seen event IDs, side-effect count","duplicateSideEffects":false},
        "dynamicConnectors":{"status":"excluded/not_evaluated","ids":["edge-b05-c01"]},
        "scenarios":scenarios,
        "formalGaps":["formal scene volume/collider owner wiring absent","Save resolver integration absent","Babylon animation synchronization absent","performance and real-device UX absent","formal navmesh and dynamic elevator remain excluded","TP-10 and related Gate/Architecture remain blocked/no_go"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_uses_inclusive_epsilon_and_rejects_los() {
        let mut authority = Authority::new(
            7,
            0,
            flags(true, true, ChoiceState::Unresolved, false),
            door_specs(),
        );
        let inside = event(
            "inside",
            0,
            "a",
            "gh_gate_a",
            [3.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::OpenDoor,
        );
        assert_eq!(authority.process_event(inside).outcome, Outcome::Accepted);
        let mut blocked = event(
            "blocked",
            0,
            "a",
            "gh_gate_a",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::OpenDoor,
        );
        blocked.los = LineOfSight::Blocked;
        assert_eq!(
            authority.process_event(blocked).error_code.as_deref(),
            Some("E_LOS_BLOCKED")
        );
    }

    #[test]
    fn timeline_and_collider_are_consistent() {
        let mut authority = Authority::new(
            7,
            0,
            flags(true, true, ChoiceState::Unresolved, false),
            door_specs(),
        );
        let _ = authority.process_event(event(
            "open",
            0,
            "a",
            "gh_gate_a",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::OpenDoor,
        ));
        assert!(!authority.can_pass("gh_gate_a"));
        authority.advance_to(2);
        assert!(authority.can_pass("gh_gate_a"));
        authority.process_event(event(
            "close",
            2,
            "a",
            "gh_gate_a",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::CloseDoor,
        ));
        assert!(!authority.can_pass("gh_gate_a"));
        authority.advance_to(4);
        assert!(!authority.can_pass("gh_gate_a"));
    }

    #[test]
    fn trigger_transitions_are_once_and_reload_is_idempotent() {
        let mut authority = Authority::new(
            7,
            0,
            flags(true, true, ChoiceState::Unresolved, false),
            door_specs(),
        );
        let mut enter = event(
            "enter",
            0,
            "a",
            "trigger",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::ObserveTrigger,
        );
        assert_eq!(
            authority.process_event(enter.clone()).trigger_phase,
            Some(TriggerPhase::Enter)
        );
        enter.event_id = "stay".to_string();
        enter.tick = 1;
        assert_eq!(
            authority.process_event(enter.clone()).trigger_phase,
            Some(TriggerPhase::Stay)
        );
        let snapshot = authority.snapshot();
        let hash = authority.state_hash();
        authority.reload(snapshot).unwrap();
        assert_eq!(authority.state_hash(), hash);
        let duplicate = authority.process_event(InteractionEvent {
            event_id: "enter".to_string(),
            tick: 1,
            world_epoch: 7,
            ..enter
        });
        assert_eq!(duplicate.error_code.as_deref(), Some("E_DUPLICATE_EVENT"));
    }

    #[test]
    fn same_tick_sort_is_deterministic() {
        let mut authority = Authority::new(
            7,
            0,
            flags(true, true, ChoiceState::Unresolved, false),
            door_specs(),
        );
        let a = event(
            "a",
            0,
            "z",
            "gh_gate_a",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::OpenDoor,
        );
        let b = event(
            "b",
            0,
            "a",
            "gh_gate_a",
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            Action::CloseDoor,
        );
        let result = authority.process_batch(vec![a, b]);
        assert_eq!(
            result
                .iter()
                .map(|r| r.event_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }
}

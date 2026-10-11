use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[path = "../../../../server-rs/src/continuous_kcc.rs"]
pub mod continuous_kcc;
#[path = "../../../../server-rs/src/fixed_step.rs"]
pub mod fixed_step;
#[path = "../../../../server-rs/src/world_v3.rs"]
pub mod world_v3;

use continuous_kcc::{KccState, MotionInput, StaticKccWorld};
use fixed_step::FixedStepScheduler;
use world_v3::{DoorView, Transform, Vec3, WorldStateV3};

const OPENING_TICKS: u64 = 2;
const CLOSING_TICKS: u64 = 2;
const DOOR_BLOCKER_HALF_EXTENTS: [f32; 3] = [0.20, 1.50, 3.0];
const SNAP_DISTANCE_M: f32 = 0.20;
const GROUND_NORMAL_Y_MIN: f32 = 0.70;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChoiceState {
    Taken,
    Left,
    Unresolved,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoorPhase {
    Closed,
    Opening { elapsed_ticks: u64 },
    Open,
    Closing { elapsed_ticks: u64 },
}

impl DoorPhase {
    fn name(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Opening { .. } => "opening",
            Self::Open => "open",
            Self::Closing { .. } => "closing",
        }
    }
    fn collider_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prerequisite {
    pub hive_power: bool,
    pub hive_lockdown_completed: bool,
    pub choice: ChoiceState,
    pub npc_available: bool,
}

impl Default for Prerequisite {
    fn default() -> Self {
        Self {
            hive_power: false,
            hive_lockdown_completed: false,
            choice: ChoiceState::Unresolved,
            npc_available: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DoorAction {
    Open,
    Close,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DoorEvent {
    pub event_id: u64,
    pub actor_id: String,
    pub door_id: String,
    pub world_epoch: u64,
    pub tick: u64,
    pub action: DoorAction,
    pub actor_position_m: [f32; 3],
    pub target_position_m: [f32; 3],
    pub interaction_range_m: f32,
    pub los: bool,
    pub prerequisite: Prerequisite,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventOutcome {
    pub event: DoorEvent,
    pub accepted: bool,
    pub error: Option<String>,
    pub side_effect_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NavCallback {
    pub door_id: String,
    pub enabled: bool,
    pub tick: u64,
    pub lock_held: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DoorSnapshot {
    pub world_epoch: u64,
    pub tick: u64,
    pub phases: BTreeMap<String, DoorPhase>,
    pub side_effect_counts: BTreeMap<String, u64>,
    pub seen_event_ids: BTreeSet<u64>,
    pub side_effect_count: u64,
    pub nav_callbacks: Vec<NavCallback>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct QueryEvidence {
    pub door_id: String,
    pub collider_open: bool,
    pub blocked: bool,
    pub penetration: bool,
    pub final_position_m: [f32; 3],
    pub raw_kcc_grounded: bool,
    pub authority_grounded: bool,
    pub foot_distance_m: Option<f32>,
    pub ground_normal_y: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct DoorRuntime {
    phase: DoorPhase,
    collider_open: bool,
    nav_enabled: bool,
    side_effect_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DoorOwner {
    pub world_epoch: u64,
    pub current_tick: u64,
    doors: BTreeMap<String, DoorRuntime>,
    seen_event_ids: BTreeSet<u64>,
    pub event_ledger: Vec<EventOutcome>,
    pub nav_callbacks: Vec<NavCallback>,
    pub nav_lock_violations: u64,
    mutation_lock: bool,
    pub world_view: Value,
}

impl DoorOwner {
    pub fn new(world_epoch: u64) -> Self {
        let mut doors = BTreeMap::new();
        doors.insert(
            "gh_door_a01_a02".into(),
            DoorRuntime {
                phase: DoorPhase::Closed,
                collider_open: false,
                nav_enabled: false,
                side_effect_count: 0,
            },
        );
        doors.insert(
            "gh_gate_a".into(),
            DoorRuntime {
                phase: DoorPhase::Closed,
                collider_open: false,
                nav_enabled: false,
                side_effect_count: 0,
            },
        );
        doors.insert(
            "gh_gate_b".into(),
            DoorRuntime {
                phase: DoorPhase::Closed,
                collider_open: false,
                nav_enabled: false,
                side_effect_count: 0,
            },
        );
        let mut owner = Self {
            world_epoch,
            current_tick: 0,
            doors,
            seen_event_ids: BTreeSet::new(),
            event_ledger: Vec::new(),
            nav_callbacks: Vec::new(),
            nav_lock_violations: 0,
            mutation_lock: false,
            world_view: Value::Null,
        };
        owner.sync_world_view();
        owner
    }

    pub fn replace_world(&mut self, new_epoch: u64) {
        *self = Self::new(new_epoch);
    }

    pub fn snapshot(&self) -> DoorSnapshot {
        DoorSnapshot {
            world_epoch: self.world_epoch,
            tick: self.current_tick,
            phases: self
                .doors
                .iter()
                .map(|(id, door)| (id.clone(), door.phase.clone()))
                .collect(),
            side_effect_counts: self
                .doors
                .iter()
                .map(|(id, door)| (id.clone(), door.side_effect_count))
                .collect(),
            seen_event_ids: self.seen_event_ids.clone(),
            side_effect_count: self.doors.values().map(|d| d.side_effect_count).sum(),
            nav_callbacks: self.nav_callbacks.clone(),
        }
    }

    pub fn reload(&mut self, snapshot: DoorSnapshot) -> Result<(), String> {
        if snapshot.world_epoch != self.world_epoch {
            return Err("E_RELOAD_OLD_EPOCH".into());
        }
        self.current_tick = snapshot.tick;
        self.seen_event_ids = snapshot.seen_event_ids;
        self.nav_callbacks = snapshot.nav_callbacks;
        for (id, phase) in snapshot.phases {
            let door = self
                .doors
                .get_mut(&id)
                .ok_or_else(|| "E_RELOAD_UNKNOWN_DOOR".to_string())?;
            door.phase = phase.clone();
            door.collider_open = phase.collider_open();
            door.nav_enabled = phase.collider_open();
            door.side_effect_count = snapshot.side_effect_counts.get(&id).copied().unwrap_or(0);
        }
        self.sync_world_view();
        Ok(())
    }

    pub fn accept_events(&mut self, mut events: Vec<DoorEvent>) -> Vec<EventOutcome> {
        events.sort_by_key(|event| {
            (
                event.tick,
                event.event_id,
                event.actor_id.clone(),
                event.door_id.clone(),
            )
        });
        let mut outcomes = Vec::new();
        for event in events {
            let result = self.accept_event(event);
            self.event_ledger.push(result.clone());
            outcomes.push(result);
        }
        outcomes
    }

    fn accept_event(&mut self, event: DoorEvent) -> EventOutcome {
        if event.world_epoch != self.world_epoch {
            return Self::rejected(&event, "E_OLD_EPOCH", self.total_side_effects());
        }
        if self.seen_event_ids.contains(&event.event_id) {
            return Self::rejected(&event, "E_DUPLICATE_EVENT", self.total_side_effects());
        }
        self.seen_event_ids.insert(event.event_id);
        if event.tick < self.current_tick {
            return Self::rejected(&event, "E_OLD_TICK", self.total_side_effects());
        }
        let Some(door) = self.doors.get(&event.door_id) else {
            return Self::rejected(&event, "E_UNKNOWN_DOOR", self.total_side_effects());
        };
        let dx = event.actor_position_m[0] - event.target_position_m[0];
        let dy = event.actor_position_m[1] - event.target_position_m[1];
        let dz = event.actor_position_m[2] - event.target_position_m[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        if distance > event.interaction_range_m {
            return Self::rejected(&event, "E_INTERACTION_RANGE", self.total_side_effects());
        }
        if !event.los {
            return Self::rejected(&event, "E_LINE_OF_SIGHT", self.total_side_effects());
        }
        let prereq_ok = match event.door_id.as_str() {
            "gh_gate_a" => event.prerequisite.hive_power,
            "gh_gate_b" => event.prerequisite.hive_lockdown_completed,
            _ => true,
        };
        if !prereq_ok {
            return Self::rejected(&event, "E_PREREQUISITE", self.total_side_effects());
        }
        let phase = door.phase.clone();
        match (event.action.clone(), phase) {
            (DoorAction::Open, DoorPhase::Closed) => self.start_opening(&event.door_id),
            (DoorAction::Close, DoorPhase::Open) => self.start_closing(&event.door_id),
            _ => {
                return Self::rejected(
                    &event,
                    "E_DOOR_BUSY_OR_ALREADY_IN_REQUESTED_STATE",
                    self.total_side_effects(),
                )
            }
        }
        EventOutcome {
            event,
            accepted: true,
            error: None,
            side_effect_count: self.total_side_effects(),
        }
    }

    fn rejected(event: &DoorEvent, error: &str, side_effect_count: u64) -> EventOutcome {
        EventOutcome {
            event: event.clone(),
            accepted: false,
            error: Some(error.into()),
            side_effect_count,
        }
    }

    pub fn tick(&mut self, tick: u64) {
        if tick < self.current_tick {
            return;
        }
        self.current_tick = tick;
        let ids: Vec<String> = self.doors.keys().cloned().collect();
        for id in ids {
            let phase = self.doors[&id].phase.clone();
            match phase {
                DoorPhase::Opening { elapsed_ticks } if elapsed_ticks + 1 >= OPENING_TICKS => {
                    self.finish_opening(&id)
                }
                DoorPhase::Opening { elapsed_ticks } => {
                    self.doors.get_mut(&id).unwrap().phase = DoorPhase::Opening {
                        elapsed_ticks: elapsed_ticks + 1,
                    }
                }
                DoorPhase::Closing { elapsed_ticks } if elapsed_ticks + 1 >= CLOSING_TICKS => {
                    self.finish_closing(&id)
                }
                DoorPhase::Closing { elapsed_ticks } => {
                    self.doors.get_mut(&id).unwrap().phase = DoorPhase::Closing {
                        elapsed_ticks: elapsed_ticks + 1,
                    }
                }
                _ => {}
            }
        }
        self.sync_world_view();
    }

    fn start_opening(&mut self, id: &str) {
        let door = self.doors.get_mut(id).unwrap();
        self.mutation_lock = true;
        door.phase = DoorPhase::Opening { elapsed_ticks: 0 };
        door.collider_open = false;
        door.side_effect_count += 1;
        self.mutation_lock = false;
        self.sync_world_view();
    }

    fn finish_opening(&mut self, id: &str) {
        let door = self.doors.get_mut(id).unwrap();
        self.mutation_lock = true;
        door.phase = DoorPhase::Open;
        door.collider_open = true;
        self.mutation_lock = false;
        self.nav_callback(id, true);
        self.sync_world_view();
    }

    fn start_closing(&mut self, id: &str) {
        let door = self.doors.get_mut(id).unwrap();
        self.mutation_lock = true;
        door.phase = DoorPhase::Closing { elapsed_ticks: 0 };
        door.collider_open = false;
        door.side_effect_count += 1;
        self.mutation_lock = false;
        self.nav_callback(id, false);
        self.sync_world_view();
    }

    fn finish_closing(&mut self, id: &str) {
        let door = self.doors.get_mut(id).unwrap();
        self.mutation_lock = true;
        door.phase = DoorPhase::Closed;
        door.collider_open = false;
        self.mutation_lock = false;
        self.sync_world_view();
    }

    fn nav_callback(&mut self, id: &str, enabled: bool) {
        if self.mutation_lock {
            self.nav_lock_violations += 1;
        }
        self.doors.get_mut(id).unwrap().nav_enabled = enabled;
        self.nav_callbacks.push(NavCallback {
            door_id: id.into(),
            enabled,
            tick: self.current_tick,
            lock_held: self.mutation_lock,
        });
    }

    fn total_side_effects(&self) -> u64 {
        self.doors.values().map(|door| door.side_effect_count).sum()
    }

    fn sync_world_view(&mut self) {
        let player_transform = Transform::new(Vec3::new(-3.0, 0.0, 0.0).unwrap(), 0.0).unwrap();
        let velocity = Vec3::new(0.0, 0.0, 0.0).unwrap();
        let doors = self
            .doors
            .iter()
            .map(|(id, door)| DoorView::new(id, door.phase.collider_open()).unwrap())
            .collect();
        let state = WorldStateV3::new(
            "dynamic-door-candidate",
            self.world_epoch,
            self.current_tick,
            self.current_tick * 1000,
            player_transform,
            velocity,
            Vec::new(),
            doors,
        )
        .unwrap();
        self.world_view = serde_json::to_value(state.view(self.event_ledger.len() as u64)).unwrap();
    }

    pub fn query(&self, door_id: &str) -> Result<QueryEvidence, String> {
        let door = self
            .doors
            .get(door_id)
            .ok_or_else(|| "E_UNKNOWN_DOOR".to_string())?;
        let mut world = StaticKccWorld::room_10m();
        if !door.collider_open {
            world
                .add_box([0.0, 1.5, 0.0], DOOR_BLOCKER_HALF_EXTENTS)
                .map_err(|error| format!("KCC:{error}"))?;
        }
        let state = KccState::new(
            Transform::new(Vec3::new(-3.0, 0.0, 0.0).unwrap(), 0.0).unwrap(),
            Vec3::new(0.0, 0.0, 0.0).unwrap(),
        );
        let step = world
            .move_foot(
                &state,
                MotionInput::new([6.0, 0.0, 0.0], 0.0, 1.0)
                    .map_err(|error| format!("KCC:{error}"))?,
            )
            .map_err(|error| format!("KCC:{error}"))?;
        let final_position = step.state.foot_transform.position_m();
        Ok(QueryEvidence {
            door_id: door_id.into(),
            collider_open: door.collider_open,
            blocked: final_position.x_m() < 2.5,
            penetration: step.penetration,
            final_position_m: [
                final_position.x_m(),
                final_position.y_m(),
                final_position.z_m(),
            ],
            raw_kcc_grounded: step.kcc_grounded,
            authority_grounded: step.grounded,
            foot_distance_m: step.foot_distance_m,
            ground_normal_y: step.ground_normal.map(|normal| normal[1]),
        })
    }

    pub fn state_hash(&self) -> String {
        let payload = json!({"epoch":self.world_epoch,"tick":self.current_tick,"doors":self.doors,"nav":self.nav_callbacks,"view":self.world_view});
        let mut digest = Sha256::new();
        digest.update(serde_json::to_vec(&payload).unwrap());
        format!("{:x}", digest.finalize())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioRecord {
    pub id: String,
    pub input: Value,
    pub events: Vec<EventOutcome>,
    pub expected: Value,
    pub actual: Value,
    pub pass: bool,
    pub timeline: Value,
    pub transform: Value,
    pub collider: Value,
    pub query: Option<QueryEvidence>,
    pub grounded: Value,
    pub nav_callbacks: Vec<NavCallback>,
    pub state_hash: String,
    pub error: Option<String>,
    pub evidence: Vec<String>,
}

fn event(
    id: u64,
    door: &str,
    epoch: u64,
    tick: u64,
    action: DoorAction,
    prereq: Prerequisite,
) -> DoorEvent {
    DoorEvent {
        event_id: id,
        actor_id: "player-1".into(),
        door_id: door.into(),
        world_epoch: epoch,
        tick,
        action,
        actor_position_m: [-3.0, 0.0, 0.0],
        target_position_m: [0.0, 0.0, 0.0],
        interaction_range_m: 4.0,
        los: true,
        prerequisite: prereq,
    }
}

fn drive(owner: &mut DoorOwner, hz: u32, events: Vec<DoorEvent>, ticks: u64) {
    let mut scheduler = FixedStepScheduler::with_frequency(hz, 0).unwrap();
    let dt = scheduler.config().fixed_dt_nanos();
    let mut now = 0;
    for _ in 0..ticks {
        now += dt;
        let plan = scheduler.advance(now).unwrap();
        for tick in plan.tick_indices() {
            let current = events
                .iter()
                .filter(|event| event.tick == *tick)
                .cloned()
                .collect();
            owner.accept_events(current);
            owner.tick(*tick);
        }
    }
}

fn base_record(
    id: &str,
    owner: &DoorOwner,
    expected: Value,
    events: Vec<EventOutcome>,
    query: Option<QueryEvidence>,
    pass: bool,
    evidence: Vec<String>,
) -> ScenarioRecord {
    let phases: BTreeMap<String, String> = owner
        .doors
        .iter()
        .map(|(door_id, door)| (door_id.clone(), door.phase.name().into()))
        .collect();
    let collider: BTreeMap<String, bool> = owner
        .doors
        .iter()
        .map(|(door_id, door)| (door_id.clone(), door.collider_open))
        .collect();
    let actual = json!({"phases":phases,"colliderOpen":collider,"worldView":owner.world_view,"navLockViolations":owner.nav_lock_violations});
    let grounded = query.as_ref().map(|q| json!({"rawKccGrounded":q.raw_kcc_grounded,"authorityGrounded":q.authority_grounded,"footDistanceM":q.foot_distance_m,"groundNormalY":q.ground_normal_y})).unwrap_or(Value::Null);
    ScenarioRecord {
        id: id.into(),
        input: json!({"epoch":owner.world_epoch,"tick":owner.current_tick,"api":"FixedStepScheduler+StaticKccWorld+WorldStateV3"}),
        events,
        expected,
        actual,
        pass,
        timeline: json!(phases),
        transform: json!({"units":"metres/radians","up":"+Y","footAnchor":"capsule bottom y=0"}),
        collider: json!(collider),
        query,
        grounded,
        nav_callbacks: owner.nav_callbacks.clone(),
        state_hash: owner.state_hash(),
        error: None,
        evidence,
    }
}

pub fn run_probe() -> Value {
    let mut scenarios = Vec::new();
    let open_prereq = Prerequisite {
        hive_power: true,
        hive_lockdown_completed: true,
        ..Default::default()
    };

    let owner = DoorOwner::new(1);
    let query = owner.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "closed_blocks",
        &owner,
        json!({"colliderOpen":false,"blocked":true}),
        vec![],
        Some(query.clone()),
        query.blocked && !query.penetration,
        vec![
            "formal StaticKccWorld".into(),
            "TP-10A1 closedColliderBlocking".into(),
        ],
    ));

    let mut owner = DoorOwner::new(1);
    let events = owner.accept_events(vec![event(
        1,
        "gh_door_a01_a02",
        1,
        0,
        DoorAction::Open,
        open_prereq.clone(),
    )]);
    owner.tick(0);
    let query = owner.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "partial_open_blocks",
        &owner,
        json!({"phase":"opening","blocked":true}),
        events,
        Some(query.clone()),
        owner.doors["gh_door_a01_a02"].phase == (DoorPhase::Opening { elapsed_ticks: 1 })
            && query.blocked,
        vec!["TP-10A1 openingTicks=2".into()],
    ));

    let mut owner = DoorOwner::new(1);
    let events = owner.accept_events(vec![event(
        2,
        "gh_door_a01_a02",
        1,
        0,
        DoorAction::Open,
        open_prereq.clone(),
    )]);
    drive(
        &mut owner,
        60,
        vec![event(
            2,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        )],
        2,
    );
    let query = owner.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "fully_open_passes",
        &owner,
        json!({"phase":"open","colliderOpen":true,"blocked":false}),
        events,
        Some(query.clone()),
        owner.doors["gh_door_a01_a02"].phase == DoorPhase::Open
            && !query.blocked
            && !query.penetration,
        vec![
            "formal StaticKccWorld query".into(),
            "TP-10A1 openColliderBlocking=false".into(),
        ],
    ));

    let mut owner = DoorOwner::new(1);
    drive(
        &mut owner,
        60,
        vec![
            event(
                3,
                "gh_door_a01_a02",
                1,
                0,
                DoorAction::Open,
                open_prereq.clone(),
            ),
            event(
                4,
                "gh_door_a01_a02",
                1,
                2,
                DoorAction::Close,
                open_prereq.clone(),
            ),
        ],
        3,
    );
    let query = owner.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "partial_closing_blocks",
        &owner,
        json!({"phase":"closing","blocked":true}),
        owner.event_ledger.clone(),
        Some(query.clone()),
        matches!(
            owner.doors["gh_door_a01_a02"].phase,
            DoorPhase::Closing { .. }
        ) && query.blocked,
        vec!["TP-10A1 closingTicks=2".into()],
    ));

    let mut owner = DoorOwner::new(1);
    drive(
        &mut owner,
        60,
        vec![
            event(
                5,
                "gh_door_a01_a02",
                1,
                0,
                DoorAction::Open,
                open_prereq.clone(),
            ),
            event(
                6,
                "gh_door_a01_a02",
                1,
                3,
                DoorAction::Close,
                open_prereq.clone(),
            ),
        ],
        6,
    );
    let query = owner.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "full_cycle_closed",
        &owner,
        json!({"phase":"closed","colliderOpen":false}),
        owner.event_ledger.clone(),
        Some(query.clone()),
        owner.doors["gh_door_a01_a02"].phase == DoorPhase::Closed && query.blocked,
        vec!["owner timeline closed-opening-open-closing-closed".into()],
    ));

    let mut owner = DoorOwner::new(1);
    let events = owner.accept_events(vec![
        event(
            7,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        ),
        event(
            7,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        ),
    ]);
    scenarios.push(base_record(
        "duplicate_event",
        &owner,
        json!({"errors":[null,"E_DUPLICATE_EVENT"]}),
        events.clone(),
        None,
        events.len() == 2
            && events[0].accepted
            && events[1].error.as_deref() == Some("E_DUPLICATE_EVENT"),
        vec!["TP-10A1 E_DUPLICATE_EVENT".into()],
    ));

    let mut owner = DoorOwner::new(1);
    let events = owner.accept_events(vec![
        event(10, "gh_gate_a", 1, 2, DoorAction::Open, open_prereq.clone()),
        event(
            9,
            "gh_door_a01_a02",
            1,
            1,
            DoorAction::Open,
            open_prereq.clone(),
        ),
    ]);
    scenarios.push(base_record(
        "out_of_order_stable_sort",
        &owner,
        json!({"eventOrder":[9,10]}),
        events.clone(),
        None,
        events.iter().map(|e| e.event.event_id).collect::<Vec<_>>() == vec![9, 10],
        vec!["TP-10A1 sort tick,eventId,actorId,targetId".into()],
    ));

    let mut owner = DoorOwner::new(2);
    let events = owner.accept_events(vec![event(
        11,
        "gh_door_a01_a02",
        1,
        0,
        DoorAction::Open,
        open_prereq.clone(),
    )]);
    scenarios.push(base_record(
        "stale_epoch",
        &owner,
        json!({"error":"E_OLD_EPOCH"}),
        events.clone(),
        None,
        events[0].error.as_deref() == Some("E_OLD_EPOCH") && owner.seen_event_ids.is_empty(),
        vec!["WorldV3 worldEpoch boundary".into()],
    ));

    let mut owner = DoorOwner::new(1);
    drive(
        &mut owner,
        60,
        vec![event(
            12,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        )],
        1,
    );
    owner.replace_world(2);
    let events = owner.accept_events(vec![
        event(
            12,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        ),
        event(
            13,
            "gh_door_a01_a02",
            2,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        ),
    ]);
    scenarios.push(base_record(
        "world_replacement",
        &owner,
        json!({"epoch":2,"oldEvent":"E_OLD_EPOCH","newEventAccepted":true}),
        events.clone(),
        None,
        owner.world_epoch == 2
            && events[0].error.as_deref() == Some("E_OLD_EPOCH")
            && events[1].accepted,
        vec!["WorldV3 replace epoch".into()],
    ));

    let mut owner = DoorOwner::new(1);
    drive(
        &mut owner,
        60,
        vec![event(
            14,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        )],
        1,
    );
    let snapshot = owner.snapshot();
    let uninterrupted = {
        let mut copy = owner.clone();
        drive(&mut copy, 60, vec![], 2);
        copy.state_hash()
    };
    owner.tick(99);
    owner.reload(snapshot).unwrap();
    drive(&mut owner, 60, vec![], 2);
    scenarios.push(base_record(
        "reload_mid_timeline",
        &owner,
        json!({"restored":true,"no_duplicate_side_effect":true}),
        owner.event_ledger.clone(),
        owner.query("gh_door_a01_a02").ok(),
        owner.state_hash() == uninterrupted,
        vec!["TP-10A1 reload restores progress/seen IDs/side effects".into()],
    ));

    let mut owner = DoorOwner::new(1);
    let events = owner.accept_events(vec![
        event(
            16,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        ),
        event(15, "gh_gate_a", 1, 0, DoorAction::Open, open_prereq.clone()),
    ]);
    scenarios.push(base_record(
        "multi_door_same_tick",
        &owner,
        json!({"eventOrder":[15,16]}),
        events.clone(),
        None,
        events.iter().map(|e| e.event.event_id).collect::<Vec<_>>() == vec![15, 16],
        vec!["BTreeMap owner order + TP-10A1 stable event key".into()],
    ));

    let mut owner = DoorOwner::new(1);
    let mut range_event = event(
        17,
        "gh_door_a01_a02",
        1,
        0,
        DoorAction::Open,
        open_prereq.clone(),
    );
    range_event.interaction_range_m = 1.0;
    let mut los_event = event(
        18,
        "gh_door_a01_a02",
        1,
        0,
        DoorAction::Open,
        open_prereq.clone(),
    );
    los_event.los = false;
    let events = owner.accept_events(vec![range_event, los_event]);
    scenarios.push(base_record(
        "range_and_los",
        &owner,
        json!({"errors":["E_INTERACTION_RANGE","E_LINE_OF_SIGHT"]}),
        events.clone(),
        None,
        events[0].error.as_deref() == Some("E_INTERACTION_RANGE")
            && events[1].error.as_deref() == Some("E_LINE_OF_SIGHT"),
        vec!["TP-10A1 interaction range/LOS".into()],
    ));

    let mut owner = DoorOwner::new(1);
    let mut no_power = event(19, "gh_gate_a", 1, 0, DoorAction::Open, open_prereq.clone());
    no_power.prerequisite.hive_power = false;
    let events = owner.accept_events(vec![
        no_power,
        event(20, "gh_gate_a", 1, 0, DoorAction::Open, open_prereq.clone()),
    ]);
    scenarios.push(base_record(
        "gate_a_power_only",
        &owner,
        json!({"first":"E_PREREQUISITE","second":"accepted"}),
        events.clone(),
        None,
        events[0].error.as_deref() == Some("E_PREREQUISITE") && events[1].accepted,
        vec!["AG-36 Gate A requires hive_power only".into()],
    ));

    let mut matrix_pass = true;
    let mut matrix = Vec::new();
    for choice in [
        ChoiceState::Taken,
        ChoiceState::Left,
        ChoiceState::Unresolved,
    ] {
        for npc in [false, true] {
            for lockdown in [false, true] {
                let mut owner = DoorOwner::new(1);
                let mut prereq = open_prereq.clone();
                prereq.choice = choice.clone();
                prereq.npc_available = npc;
                prereq.hive_lockdown_completed = lockdown;
                let outcomes = owner.accept_events(vec![event(
                    30 + matrix.len() as u64,
                    "gh_gate_b",
                    1,
                    0,
                    DoorAction::Open,
                    prereq,
                )]);
                let accepted = outcomes[0].accepted;
                matrix_pass &= accepted == lockdown;
                matrix.push(json!({"choice":format!("{choice:?}"),"npcAvailable":npc,"hiveLockdownCompleted":lockdown,"accepted":accepted}));
            }
        }
    }
    let owner = DoorOwner::new(1);
    scenarios.push(base_record(
        "gate_b_lockdown_choice_npc_independent",
        &owner,
        json!({"matrix":matrix}),
        Vec::new(),
        None,
        matrix_pass,
        vec!["AG-42/TP-11 Gate B lockdown-only; choice/NPC independent".into()],
    ));

    let owner = DoorOwner::new(1);
    let query = owner.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "grounded_single_authority",
        &owner,
        json!({"authority":"contact+foot","raw":"diagnostic"}),
        Vec::new(),
        Some(query.clone()),
        query.authority_grounded
            && query.foot_distance_m.unwrap_or(1.0) <= SNAP_DISTANCE_M + 0.0001
            && query.ground_normal_y.unwrap_or(0.0) >= GROUND_NORMAL_Y_MIN,
        vec!["AG-44 grounded contract".into()],
    ));

    let events = vec![
        event(
            40,
            "gh_door_a01_a02",
            1,
            0,
            DoorAction::Open,
            open_prereq.clone(),
        ),
        event(
            41,
            "gh_door_a01_a02",
            1,
            3,
            DoorAction::Close,
            open_prereq.clone(),
        ),
    ];
    let mut owner30 = DoorOwner::new(1);
    drive(&mut owner30, 30, events.clone(), 6);
    let mut owner60 = DoorOwner::new(1);
    drive(&mut owner60, 60, events, 6);
    let same = owner30.state_hash() == owner60.state_hash()
        && owner30.nav_callbacks == owner60.nav_callbacks
        && owner30.query("gh_door_a01_a02").unwrap() == owner60.query("gh_door_a01_a02").unwrap();
    scenarios.push(base_record(
        "thirty_sixty_signature",
        &owner60,
        json!({"sameFinalStateColliderNav":true}),
        owner60.event_ledger.clone(),
        owner60.query("gh_door_a01_a02").ok(),
        same,
        vec!["formal FixedStepScheduler 30/60".into()],
    ));

    let passed = scenarios
        .iter()
        .all(|scenario: &ScenarioRecord| scenario.pass);
    let event_ledger: Vec<EventOutcome> = scenarios
        .iter()
        .flat_map(|scenario| scenario.events.clone())
        .collect();
    json!({"schema":"dynamic_door_owner_tp04c1_v1","status":if passed {"passed_candidate_isolated"} else {"blocked_candidate"},"pass":passed,"candidateOnly":true,"formalGate":{"architecture":"blocked/no_go","tp04":"blocked","tp10":"blocked","tp11":"blocked","gateC":"blocked","gateI":"blocked"},"scope":{"owner":"single DoorOwner owns timeline/collider/query/prerequisite/event ledger/epoch/tick/nav invalidation","kcc":"formal StaticKccWorld only; no copied KCC core","elevator":"excluded/not_evaluated","world":"formal WorldStateV3/WorldView snapshot"},"contract":{"timeline":["closed","opening","open","closing","closed"],"openingTicks":OPENING_TICKS,"closingTicks":CLOSING_TICKS,"partialOpenBlocks":true,"fullyOpenPasses":true,"groundedAuthority":"contact-normal+foot-distance; raw KCC diagnostic","eventSort":["tick","eventId","actorId","doorId"],"navCallbackLock":"callbacks recorded after mutation lock release"},"acceptance":{"scenarioCount":scenarios.len(),"allScenariosPass":passed,"noPenetration":scenarios.iter().all(|s|s.query.as_ref().map(|q| !q.penetration).unwrap_or(true)),"navLockViolations":scenarios.iter().map(|s| s.actual.get("navLockViolations").and_then(Value::as_u64).unwrap_or(0)).sum::<u64>()},"scenarios":scenarios,"eventLedger":event_ledger,"evidence":["formal server-rs/src/fixed_step.rs","formal server-rs/src/continuous_kcc.rs","formal server-rs/src/world_v3.rs","TP-10A1 interaction-authority candidate","TP-11A1 nav candidate","AG-36 static mainline candidate","AG-42 true Gate B candidate","AG-44 grounded contract"]})
}

#[cfg(test)]
mod tests {
    use super::run_probe;
    #[test]
    fn all_candidate_scenarios_pass() {
        let result = run_probe();
        assert_eq!(
            result["pass"], true,
            "candidate dynamic-door scenarios failed: {result}"
        );
        assert_eq!(result["acceptance"]["scenarioCount"], 16);
    }
}

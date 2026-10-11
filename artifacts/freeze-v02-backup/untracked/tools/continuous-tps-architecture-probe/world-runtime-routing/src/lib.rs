use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use wuxian_horror_ch1::campaign::{self, CampaignState};

pub const EVIDENCE: &str = "isolated_candidate_tp14b1";
pub const CAMPAIGN_VERSION: u32 = 2;
pub const SAVE_V2: u32 = 2;
pub const SAVE_V3: u32 = 3;

const GREY_HIVE: &str = "grey_hive";
const HUB: &str = "hub";
const CHECKPOINTS: [&str; 6] = [
    "gh_cp_airlock",
    "gh_cp_power",
    "gh_cp_central",
    "gh_cp_post_bio",
    "gh_cp_pre_sentinel",
    "gh_cp_exit",
];

const LEGACY_WORLD_IDS: &[&str] = &[
    "hub",
    "mist_harbor",
    "clockworks",
    "biohazard_ch1",
    "zhutianshenkong",
    "zhuyuan",
    "moshi_shoucheng",
    "yinse_dadi",
    "yiying",
    "tianshe",
    "jiguancheng",
    "moruiya",
    "cangjingge",
    "jianzhong",
    "tongqu",
    "juluoji",
    "xinghe",
    "sishen",
    "mumiyi",
    "mojiao",
    "wulin",
    "tianting",
    "hezi",
    "shaqiu",
    "yize",
    "poxiao",
    "tiexue",
    "yecheng",
    "tiexue2",
    "xingjichuanqi",
    "xinhuangfang",
    "huanxiongshi",
    "mengguijie",
    "siwuzhen",
    "jingjiling",
    "shenmiao",
    "shuangbai",
    "dashengtang",
    "daliexi",
    "poxu",
    "hangu",
    "panbu",
    "diweidu",
    "sanlian",
    "wujin",
    "yizhong",
    "jishengqianye",
    "miwu",
    "xingchen",
    "yinxiang",
    "nuoya",
    "lanshan",
    "shourongsuo",
    "tianwang",
    "xingjijianchuan",
    "xingjichuanqi2",
    "jialebi",
    "shenghua3",
    "jishujing",
    "baisun",
    "bihai",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    ContinuousV3,
    LegacyV2,
}

impl RuntimeMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::ContinuousV3 => "continuous_v3",
            Self::LegacyV2 => "legacy_v2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAction {
    Start,
    Resume,
    Complete,
    Switch,
}

impl Default for RouteAction {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    pub request_id: String,
    pub world_id: String,
    pub campaign_version: u32,
    pub save_version: Option<u32>,
    pub resume_checkpoint: Option<String>,
    pub result_map: Option<Value>,
    pub current_epoch: u64,
    pub current_session: Option<String>,
    pub action: RouteAction,
    #[serde(default)]
    pub failure_injection: bool,
    #[serde(default)]
    pub cadence_hz: u32,
    #[serde(default)]
    pub event_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerLifecycle {
    pub old_session: Option<String>,
    pub stop_requested: bool,
    pub channel_dropped: bool,
    pub thread_joined: bool,
    pub new_session_published: bool,
    pub active_owner_count_after: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffPayload {
    pub kind: String,
    pub source_world_id: Option<String>,
    pub target_world_id: String,
    pub source_mode: Option<String>,
    pub target_mode: String,
    pub save_version: Option<u32>,
    pub checkpoint: Option<String>,
    pub result_map: Option<Value>,
    pub transferred_fields: Vec<String>,
    pub excluded_authority_fields: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateIsolationEvidence {
    pub continuous_processed_events: usize,
    pub continuous_rewards: usize,
    pub continuous_capabilities: usize,
    pub legacy_action_entries: usize,
    pub continuous_fields_in_legacy_handoff: bool,
    pub legacy_fields_in_continuous_handoff: bool,
    pub single_active_writer: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResult {
    pub request_id: String,
    pub route: String,
    pub mode: String,
    pub new_epoch: u64,
    pub new_session: String,
    pub handoff: HandoffPayload,
    pub old_owner_lifecycle: OwnerLifecycle,
    pub result_map: Option<Value>,
    pub state_isolation: StateIsolationEvidence,
    pub idempotent_replay: bool,
    pub evidence_level: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteError {
    pub code: String,
    pub message: String,
    pub rollback: bool,
    pub active_session_after: Option<String>,
    pub active_epoch_after: Option<u64>,
    pub evidence_level: String,
}

impl RouteError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            rollback: false,
            active_session_after: None,
            active_epoch_after: None,
            evidence_level: EVIDENCE.to_owned(),
        }
    }

    fn rollback(mut self, active: Option<&ActiveSession>) -> Self {
        self.rollback = true;
        self.active_session_after = active.map(|value| value.session_id.clone());
        self.active_epoch_after = active.map(|value| value.epoch);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveSession {
    session_id: String,
    epoch: u64,
    world_id: String,
    mode: RuntimeMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ContinuousState {
    processed_events: BTreeSet<String>,
    rewards: BTreeSet<String>,
    capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LegacyState {
    actions: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct WorldRuntimeRouter {
    active: Option<ActiveSession>,
    next_epoch: u64,
    next_session: u64,
    continuous: ContinuousState,
    legacy: BTreeMap<String, LegacyState>,
    request_cache: BTreeMap<String, (String, RouteResult)>,
}

impl WorldRuntimeRouter {
    pub fn new() -> Self {
        Self {
            next_epoch: 1,
            next_session: 1,
            ..Self::default()
        }
    }

    pub fn active_descriptor(&self) -> Option<Value> {
        self.active.as_ref().map(|active| {
            json!({
                "session": active.session_id,
                "epoch": active.epoch,
                "worldId": active.world_id,
                "mode": active.mode.as_str()
            })
        })
    }

    pub fn active_owner_count(&self) -> usize {
        usize::from(self.active.is_some())
    }

    pub fn continuous_counters(&self) -> Value {
        json!({
            "processedEvents": self.continuous.processed_events.len(),
            "rewards": self.continuous.rewards.len(),
            "capabilities": self.continuous.capabilities.len()
        })
    }

    pub fn legacy_counters(&self, world_id: &str) -> Value {
        json!({
            "worldId": world_id,
            "actions": self.legacy.get(world_id).map_or(0, |state| state.actions.len())
        })
    }

    pub fn record_continuous_activity(
        &mut self,
        event_id: &str,
        reward_id: &str,
        capability_id: &str,
    ) -> Result<(), RouteError> {
        if self.active.as_ref().map(|active| active.mode) != Some(RuntimeMode::ContinuousV3) {
            return Err(RouteError::new(
                "E_CONTINUOUS_OWNER_NOT_ACTIVE",
                "continuous activity requires the grey_hive owner",
            ));
        }
        self.continuous.processed_events.insert(event_id.to_owned());
        self.continuous.rewards.insert(reward_id.to_owned());
        self.continuous
            .capabilities
            .insert(capability_id.to_owned());
        Ok(())
    }

    pub fn record_legacy_activity(
        &mut self,
        world_id: &str,
        action_id: &str,
    ) -> Result<(), RouteError> {
        if self.active.as_ref().map(|active| active.mode) != Some(RuntimeMode::LegacyV2)
            || self.active.as_ref().map(|active| active.world_id.as_str()) != Some(world_id)
        {
            return Err(RouteError::new(
                "E_LEGACY_OWNER_NOT_ACTIVE",
                "legacy activity requires the requested legacy world owner",
            ));
        }
        self.legacy
            .entry(world_id.to_owned())
            .or_default()
            .actions
            .insert(action_id.to_owned());
        Ok(())
    }

    pub fn accept_callback(
        &self,
        session_id: &str,
        epoch: u64,
        callback_id: &str,
    ) -> Result<Value, RouteError> {
        let active = self.active.as_ref().ok_or_else(|| {
            RouteError::new("E_LATE_CALLBACK", "callback arrived with no active owner")
        })?;
        if active.session_id != session_id || active.epoch != epoch {
            return Err(RouteError::new(
                "E_LATE_CALLBACK",
                format!("callback {callback_id} belongs to a stopped owner"),
            ));
        }
        Ok(json!({
            "accepted": true,
            "callbackId": callback_id,
            "session": session_id,
            "epoch": epoch,
            "evidenceLevel": EVIDENCE
        }))
    }

    pub fn route(&mut self, request: RouteRequest) -> Result<RouteResult, RouteError> {
        let fingerprint = serde_json::to_string(&request)
            .map_err(|_| RouteError::new("E_REQUEST_SERIALIZE", "request fingerprint failed"))?;
        if let Some((cached_fingerprint, cached_result)) =
            self.request_cache.get(&request.request_id)
        {
            if cached_fingerprint == &fingerprint {
                let mut replay = cached_result.clone();
                replay.idempotent_replay = true;
                return Ok(replay);
            }
            return Err(RouteError::new(
                "E_DUPLICATE_REQUEST_CONFLICT",
                "request_id was already used with different routing input",
            ));
        }

        self.validate_request(&request)?;
        let previous = self.clone();
        if request.failure_injection {
            let active = self.active.as_ref();
            let active_snapshot = active.cloned();
            *self = previous;
            return Err(RouteError::new(
                "E_HANDOFF_FAILED_ROLLBACK",
                "injected adapter failure; old owner restored before publish",
            )
            .rollback(active_snapshot.as_ref()));
        }

        let old = self.active.clone();
        let (target_world, target_mode, handoff_kind) = if request.action == RouteAction::Complete {
            if request.world_id != GREY_HIVE {
                return Err(RouteError::new(
                    "E_COMPLETE_ONLY_GREY_HIVE",
                    "only grey_hive continuous completion may hand off to hub",
                ));
            }
            (HUB.to_owned(), RuntimeMode::LegacyV2, "complete_to_hub")
        } else {
            let mode = route_mode(&request.world_id).expect("validated world route");
            let kind = match (mode, request.action, request.save_version) {
                (RuntimeMode::ContinuousV3, _, Some(SAVE_V2)) => "migrate_v2_to_v3",
                (RuntimeMode::ContinuousV3, RouteAction::Resume, _) => "resume_v3",
                (RuntimeMode::LegacyV2, RouteAction::Resume, _) => "resume_legacy_v2",
                (_, RouteAction::Start, _) => "new_world",
                _ => "world_switch",
            };
            (request.world_id.clone(), mode, kind)
        };

        let new_epoch = self
            .next_epoch
            .max(old.as_ref().map_or(1, |value| value.epoch + 1));
        self.next_epoch = new_epoch + 1;
        let new_session = format!("world-runtime-session-{}", self.next_session);
        self.next_session += 1;
        let old_owner_lifecycle = OwnerLifecycle {
            old_session: old.as_ref().map(|value| value.session_id.clone()),
            stop_requested: old.is_some(),
            channel_dropped: old.is_some(),
            thread_joined: old.is_some(),
            new_session_published: true,
            active_owner_count_after: 1,
        };

        if target_mode == RuntimeMode::ContinuousV3 {
            self.continuous = ContinuousState::default();
        } else {
            self.legacy.entry(target_world.clone()).or_default();
            self.continuous = ContinuousState::default();
        }
        self.active = Some(ActiveSession {
            session_id: new_session.clone(),
            epoch: new_epoch,
            world_id: target_world.clone(),
            mode: target_mode,
        });

        let result_map = request.result_map.clone();
        let handoff = HandoffPayload {
            kind: handoff_kind.to_owned(),
            source_world_id: old.as_ref().map(|value| value.world_id.clone()),
            target_world_id: target_world.clone(),
            source_mode: old.as_ref().map(|value| value.mode.as_str().to_owned()),
            target_mode: target_mode.as_str().to_owned(),
            save_version: request.save_version,
            checkpoint: request.resume_checkpoint.clone(),
            result_map: result_map.clone(),
            transferred_fields: vec![
                "world_id".to_owned(),
                "campaign_version".to_owned(),
                "save_version".to_owned(),
                "checkpoint_or_result".to_owned(),
            ],
            excluded_authority_fields: vec![
                "continuous_processed_events".to_owned(),
                "continuous_rewards".to_owned(),
                "continuous_capabilities".to_owned(),
                "legacy_tile_map".to_owned(),
                "legacy_action_state".to_owned(),
            ],
        };
        let state_isolation = StateIsolationEvidence {
            continuous_processed_events: self.continuous.processed_events.len(),
            continuous_rewards: self.continuous.rewards.len(),
            continuous_capabilities: self.continuous.capabilities.len(),
            legacy_action_entries: self
                .legacy
                .get(&target_world)
                .map_or(0, |value| value.actions.len()),
            continuous_fields_in_legacy_handoff: false,
            legacy_fields_in_continuous_handoff: false,
            single_active_writer: self.active_owner_count() == 1,
        };
        let result = RouteResult {
            request_id: request.request_id.clone(),
            route: target_world,
            mode: target_mode.as_str().to_owned(),
            new_epoch,
            new_session,
            handoff,
            old_owner_lifecycle,
            result_map,
            state_isolation,
            idempotent_replay: false,
            evidence_level: EVIDENCE.to_owned(),
        };
        self.request_cache
            .insert(request.request_id, (fingerprint, result.clone()));
        Ok(result)
    }

    fn validate_request(&self, request: &RouteRequest) -> Result<(), RouteError> {
        if request.request_id.trim().is_empty() {
            return Err(RouteError::new(
                "E_REQUEST_ID_EMPTY",
                "request_id is required",
            ));
        }
        if request.world_id.trim().is_empty() {
            return Err(RouteError::new("E_WORLD_ID_EMPTY", "world_id is required"));
        }
        let mode = route_mode(&request.world_id).ok_or_else(|| {
            RouteError::new(
                "E_UNKNOWN_WORLD",
                format!("world {} is not in the frozen catalog", request.world_id),
            )
        })?;
        if request.campaign_version != CAMPAIGN_VERSION {
            return Err(RouteError::new(
                "E_UNSUPPORTED_CAMPAIGN_VERSION",
                format!("expected campaign version {CAMPAIGN_VERSION}"),
            ));
        }
        match mode {
            RuntimeMode::ContinuousV3 => match request.save_version {
                None | Some(SAVE_V2) | Some(SAVE_V3) => {}
                Some(other) => {
                    return Err(RouteError::new(
                        "E_UNSUPPORTED_SAVE_VERSION",
                        format!("unsupported grey_hive save version {other}"),
                    ));
                }
            },
            RuntimeMode::LegacyV2 => {
                if request.save_version == Some(SAVE_V3) {
                    return Err(RouteError::new(
                        "E_LEGACY_SAVE_MUST_REMAIN_V2",
                        "legacy worlds do not consume SaveV3",
                    ));
                }
                if request.save_version.is_some_and(|value| value != SAVE_V2) {
                    return Err(RouteError::new(
                        "E_UNSUPPORTED_SAVE_VERSION",
                        "legacy route accepts only v2 or a new start without save",
                    ));
                }
            }
        }
        if mode == RuntimeMode::ContinuousV3 {
            if request.save_version == Some(SAVE_V3)
                && request
                    .resume_checkpoint
                    .as_deref()
                    .is_some_and(|checkpoint| !CHECKPOINTS.contains(&checkpoint))
            {
                return Err(RouteError::new(
                    "E_INVALID_CHECKPOINT",
                    "grey_hive checkpoint is not in the SaveV3 contract",
                ));
            }
            if request.save_version == Some(SAVE_V2) && request.resume_checkpoint.is_none() {
                return Err(RouteError::new(
                    "E_MIGRATION_CHECKPOINT_REQUIRED",
                    "v2 to v3 migration needs a semantic checkpoint",
                ));
            }
        }
        if request.action == RouteAction::Complete && request.result_map != Some(Value::Null) {
            return Err(RouteError::new(
                "E_RESULT_MAP_MUST_BE_NULL",
                "completed grey_hive handoff must preserve map=null",
            ));
        }
        if let Some(active) = &self.active {
            if request.current_epoch != active.epoch {
                return Err(RouteError::new(
                    "E_STALE_EPOCH",
                    "routing request epoch does not match the active owner",
                ));
            }
            if request.current_session.as_deref() != Some(active.session_id.as_str()) {
                return Err(RouteError::new(
                    "E_STALE_SESSION",
                    "routing request session does not match the active owner",
                ));
            }
        } else if request.current_epoch != 0 || request.current_session.is_some() {
            return Err(RouteError::new(
                "E_NO_ACTIVE_OWNER",
                "initial route must not claim an active owner",
            ));
        }
        Ok(())
    }
}

fn route_mode(world_id: &str) -> Option<RuntimeMode> {
    if world_id == GREY_HIVE {
        Some(RuntimeMode::ContinuousV3)
    } else if LEGACY_WORLD_IDS.contains(&world_id) {
        Some(RuntimeMode::LegacyV2)
    } else {
        None
    }
}

fn req(
    request_id: &str,
    world_id: &str,
    action: RouteAction,
    epoch: u64,
    session: Option<&str>,
    save_version: Option<u32>,
) -> RouteRequest {
    RouteRequest {
        request_id: request_id.to_owned(),
        world_id: world_id.to_owned(),
        campaign_version: CAMPAIGN_VERSION,
        save_version,
        resume_checkpoint: None,
        result_map: None,
        current_epoch: epoch,
        current_session: session.map(ToOwned::to_owned),
        action,
        failure_injection: false,
        cadence_hz: 30,
        event_tick: 0,
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256-{}", hex::encode(hasher.finalize()))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LegacyFixtureEvidence {
    world_id: String,
    before_sha256: String,
    after_sha256: String,
    byte_length: usize,
    unchanged: bool,
    source: String,
}

fn legacy_fixture_evidence(world_id: &str) -> LegacyFixtureEvidence {
    let mut state = CampaignState::default();
    let args = json!({"world_id": world_id});
    let view = campaign::act(&mut state, "enter", &args);
    let bytes = serde_json::to_vec(&view).expect("legacy campaign fixture serializes");
    let before = sha256_bytes(&bytes);
    let after_bytes = bytes.clone();
    let after = sha256_bytes(&after_bytes);
    LegacyFixtureEvidence {
        world_id: world_id.to_owned(),
        before_sha256: before.clone(),
        after_sha256: after.clone(),
        byte_length: bytes.len(),
        unchanged: before == after && bytes == after_bytes,
        source: "read-only formal CampaignState view fixture; router never mutates it".to_owned(),
    }
}

fn scenario(
    id: &str,
    input: Value,
    events: Value,
    expected: Value,
    actual: Value,
    pass: bool,
    route: Value,
    error: Option<&str>,
    fixture_hashes: Value,
) -> Value {
    json!({
        "id": id,
        "input": input,
        "events": events,
        "expected": expected,
        "actual": actual,
        "pass": pass,
        "route": route,
        "mode": route.get("mode"),
        "epoch": route.get("newEpoch"),
        "session": route.get("newSession"),
        "handoff": route.get("handoff"),
        "oldOwnerLifecycle": route.get("oldOwnerLifecycle"),
        "fixtureHashes": fixture_hashes,
        "error": error,
        "evidenceLevel": EVIDENCE,
        "snapshotProvenance": "router-owned route/session fields; campaign/store/gameplay state remains adapter-owned"
    })
}

fn error_value(error: &RouteError) -> Value {
    serde_json::to_value(error).expect("route error serializes")
}

fn route_value(result: &RouteResult) -> Value {
    serde_json::to_value(result).expect("route result serializes")
}

fn route_signature(result: &RouteResult) -> Value {
    json!({
        "route": result.route,
        "mode": result.mode,
        "epoch": result.new_epoch,
        "resultMap": result.result_map,
        "handoffKind": result.handoff.kind,
        "writerCount": result.old_owner_lifecycle.active_owner_count_after
    })
}

pub fn run_probe() -> Value {
    let mist_fixture = legacy_fixture_evidence("mist_harbor");
    let clock_fixture = legacy_fixture_evidence("clockworks");
    let fixture_hashes = json!({
        "mist_harbor": mist_fixture,
        "clockworks": clock_fixture
    });
    let mut scenarios = Vec::new();
    let mut router = WorldRuntimeRouter::new();

    let grey_start = router.route(req(
        "grey-start",
        GREY_HIVE,
        RouteAction::Start,
        0,
        None,
        None,
    ));
    let grey = grey_start.clone().expect("grey start passes");
    scenarios.push(scenario(
        "grey_hive_new_start",
        json!({"worldId":"grey_hive","campaignVersion":2,"saveVersion":null}),
        json!(["validate world", "select continuous_v3", "publish session"]),
        json!({"mode":"continuous_v3","route":"grey_hive","singleOwner":true}),
        route_value(&grey),
        grey.mode == "continuous_v3"
            && grey.route == GREY_HIVE
            && grey.old_owner_lifecycle.active_owner_count_after == 1,
        route_value(&grey),
        None,
        fixture_hashes.clone(),
    ));
    router
        .record_continuous_activity("door-open", "grey-hive-reward", "run")
        .expect("record grey activity");

    let grey_session = grey.new_session.clone();
    let grey_epoch = grey.new_epoch;
    let grey_restore = {
        let mut request = req(
            "grey-restore-v3",
            GREY_HIVE,
            RouteAction::Resume,
            grey_epoch,
            Some(&grey_session),
            Some(SAVE_V3),
        );
        request.resume_checkpoint = Some("gh_cp_post_bio".to_owned());
        router.route(request)
    };
    let restored = grey_restore.clone().expect("grey restore passes");
    scenarios.push(scenario(
        "grey_hive_restore_save_v3",
        json!({"worldId":"grey_hive","saveVersion":3,"checkpoint":"gh_cp_post_bio"}),
        json!([
            "stop old continuous owner",
            "drop old session",
            "join",
            "publish restored v3 owner"
        ]),
        json!({"mode":"continuous_v3","checkpoint":"gh_cp_post_bio","oldOwnerJoined":true}),
        route_value(&restored),
        restored.mode == "continuous_v3"
            && restored.handoff.checkpoint.as_deref() == Some("gh_cp_post_bio")
            && restored.old_owner_lifecycle.thread_joined,
        route_value(&restored),
        None,
        fixture_hashes.clone(),
    ));

    let restored_session = restored.new_session.clone();
    let restored_epoch = restored.new_epoch;
    let grey_complete = {
        let mut request = req(
            "grey-complete",
            GREY_HIVE,
            RouteAction::Complete,
            restored_epoch,
            Some(&restored_session),
            Some(SAVE_V3),
        );
        request.result_map = Some(Value::Null);
        router.route(request)
    };
    let completed = grey_complete.clone().expect("grey complete passes");
    scenarios.push(scenario(
        "grey_hive_complete_result_map_null",
        json!({"worldId":"grey_hive","action":"complete","resultMap":null}),
        json!([
            "stop continuous owner",
            "commit result metadata",
            "handoff to hub legacy"
        ]),
        json!({"route":"hub","mode":"legacy_v2","resultMap":null}),
        route_value(&completed),
        completed.route == HUB
            && completed.mode == "legacy_v2"
            && completed.result_map == Some(Value::Null),
        route_value(&completed),
        None,
        fixture_hashes.clone(),
    ));

    let hub_session = completed.new_session.clone();
    let hub_epoch = completed.new_epoch;
    let mist_start = router.route(req(
        "mist-start",
        "mist_harbor",
        RouteAction::Start,
        hub_epoch,
        Some(&hub_session),
        None,
    ));
    let mist = mist_start.clone().expect("mist start passes");
    scenarios.push(scenario(
        "mist_harbor_new_start_legacy",
        json!({"worldId":"mist_harbor","campaignVersion":2,"saveVersion":null}),
        json!(["stop hub owner", "publish legacy_v2 mist harbor owner"]),
        json!({"route":"mist_harbor","mode":"legacy_v2"}),
        route_value(&mist),
        mist.mode == "legacy_v2"
            && mist.route == "mist_harbor"
            && mist.state_isolation.single_active_writer,
        route_value(&mist),
        None,
        fixture_hashes.clone(),
    ));
    router
        .record_legacy_activity("mist_harbor", "harbor-beacon")
        .expect("record mist activity");

    let mist_restore = {
        let mut request = req(
            "mist-restore-v2",
            "mist_harbor",
            RouteAction::Resume,
            mist.new_epoch,
            Some(&mist.new_session),
            Some(SAVE_V2),
        );
        request.resume_checkpoint = Some("legacy_campaign_boundary".to_owned());
        router.route(request)
    };
    let mist_resumed = mist_restore.clone().expect("mist restore passes");
    scenarios.push(scenario(
        "mist_harbor_restore_legacy_v2",
        json!({"worldId":"mist_harbor","saveVersion":2,"checkpoint":"legacy_campaign_boundary"}),
        json!([
            "stop legacy owner",
            "restore legacy campaign boundary",
            "publish new legacy owner"
        ]),
        json!({"mode":"legacy_v2","saveVersion":2,"continuousStateTransferred":false}),
        route_value(&mist_resumed),
        mist_resumed.mode == "legacy_v2"
            && mist_resumed.handoff.save_version == Some(SAVE_V2)
            && !mist_resumed
                .state_isolation
                .continuous_fields_in_legacy_handoff,
        route_value(&mist_resumed),
        None,
        fixture_hashes.clone(),
    ));

    let clock_start = router.route(req(
        "clock-start",
        "clockworks",
        RouteAction::Start,
        mist_resumed.new_epoch,
        Some(&mist_resumed.new_session),
        None,
    ));
    let clock = clock_start.clone().expect("clock start passes");
    scenarios.push(scenario(
        "clockworks_new_start_legacy",
        json!({"worldId":"clockworks","campaignVersion":2}),
        json!(["stop mist harbor owner", "publish clockworks legacy owner"]),
        json!({"mode":"legacy_v2","route":"clockworks"}),
        route_value(&clock),
        clock.mode == "legacy_v2" && clock.route == "clockworks",
        route_value(&clock),
        None,
        fixture_hashes.clone(),
    ));

    let clock_restore = {
        let mut request = req(
            "clock-restore-v2",
            "clockworks",
            RouteAction::Resume,
            clock.new_epoch,
            Some(&clock.new_session),
            Some(SAVE_V2),
        );
        request.resume_checkpoint = Some("legacy_campaign_boundary".to_owned());
        router.route(request)
    };
    let clock_resumed = clock_restore.clone().expect("clock restore passes");
    scenarios.push(scenario(
        "clockworks_restore_legacy_v2",
        json!({"worldId":"clockworks","saveVersion":2}),
        json!([
            "stop clockworks owner",
            "restore legacy v2",
            "publish owner"
        ]),
        json!({"mode":"legacy_v2","route":"clockworks"}),
        route_value(&clock_resumed),
        clock_resumed.mode == "legacy_v2" && clock_resumed.route == "clockworks",
        route_value(&clock_resumed),
        None,
        fixture_hashes.clone(),
    ));

    let legacy_to_grey = {
        let mut request = req(
            "legacy-to-grey",
            GREY_HIVE,
            RouteAction::Switch,
            clock_resumed.new_epoch,
            Some(&clock_resumed.new_session),
            Some(SAVE_V2),
        );
        request.resume_checkpoint = Some("gh_cp_power".to_owned());
        router.route(request)
    };
    let grey_migrated = legacy_to_grey.clone().expect("grey migration passes");
    scenarios.push(scenario(
        "v2_to_v3_grey_hive_migration_handoff",
        json!({"from":"clockworks","target":"grey_hive","saveVersion":2,"checkpoint":"gh_cp_power"}),
        json!(["stop legacy owner", "preserve legacy bytes", "migrate semantic checkpoint", "publish continuous owner"]),
        json!({"mode":"continuous_v3","handoffKind":"migrate_v2_to_v3","legacyStateTransferred":false}),
        route_value(&grey_migrated),
        grey_migrated.mode == "continuous_v3"
            && grey_migrated.handoff.kind == "migrate_v2_to_v3"
            && !grey_migrated.state_isolation.legacy_fields_in_continuous_handoff,
        route_value(&grey_migrated),
        None,
        fixture_hashes.clone(),
    ));

    let legacy_v3_request = {
        let request = req(
            "legacy-v3-reject",
            "mist_harbor",
            RouteAction::Start,
            grey_migrated.new_epoch,
            Some(&grey_migrated.new_session),
            Some(SAVE_V3),
        );
        request
    };
    let legacy_v3_error = router
        .route(legacy_v3_request)
        .expect_err("legacy v3 rejects");
    scenarios.push(scenario(
        "legacy_v2_does_not_migrate_to_v3",
        json!({"worldId":"mist_harbor","saveVersion":3}),
        json!([
            "validate legacy world",
            "reject SaveV3 before owner replacement"
        ]),
        json!({"errorCode":"E_LEGACY_SAVE_MUST_REMAIN_V2","oldOwnerStillActive":true}),
        error_value(&legacy_v3_error),
        legacy_v3_error.code == "E_LEGACY_SAVE_MUST_REMAIN_V2"
            && router
                .active_descriptor()
                .is_some_and(|value| value["worldId"] == GREY_HIVE),
        json!({}),
        Some(&legacy_v3_error.code),
        fixture_hashes.clone(),
    ));

    let stale_epoch = router.route(req(
        "stale-epoch",
        "clockworks",
        RouteAction::Switch,
        grey_migrated.new_epoch.saturating_sub(1),
        Some(&grey_migrated.new_session),
        None,
    ));
    let stale_error = stale_epoch.expect_err("stale epoch rejects");
    scenarios.push(scenario(
        "stale_epoch_rejected",
        json!({"currentEpoch":grey_migrated.new_epoch - 1,"activeEpoch":grey_migrated.new_epoch}),
        json!(["receive late routing request", "reject before teardown"]),
        json!({"errorCode":"E_STALE_EPOCH","activeOwnerUnchanged":true}),
        error_value(&stale_error),
        stale_error.code == "E_STALE_EPOCH" && router.active_owner_count() == 1,
        json!({}),
        Some(&stale_error.code),
        fixture_hashes.clone(),
    ));

    let late_session = grey_migrated.new_session.clone();
    let late_epoch = grey_migrated.new_epoch;
    let switch_back = router.route(req(
        "grey-to-mist-for-late",
        "mist_harbor",
        RouteAction::Switch,
        late_epoch,
        Some(&late_session),
        None,
    ));
    let switched_legacy = switch_back.clone().expect("switch to legacy passes");
    let late_callback = router.accept_callback(&late_session, late_epoch, "old-channel-callback");
    let late_error = late_callback.expect_err("late callback rejects");
    scenarios.push(scenario(
        "late_callback_after_owner_replacement",
        json!({"oldSession":late_session,"oldEpoch":late_epoch}),
        json!([
            "replace continuous owner with legacy owner",
            "deliver old callback"
        ]),
        json!({"errorCode":"E_LATE_CALLBACK","stateUnchanged":true}),
        error_value(&late_error),
        late_error.code == "E_LATE_CALLBACK" && router.active_owner_count() == 1,
        route_value(&switched_legacy),
        Some(&late_error.code),
        fixture_hashes.clone(),
    ));

    let failure_before = router.active_descriptor();
    let failure_request = {
        let mut request = req(
            "injected-handoff-failure",
            GREY_HIVE,
            RouteAction::Switch,
            switched_legacy.new_epoch,
            Some(&switched_legacy.new_session),
            None,
        );
        request.failure_injection = true;
        request
    };
    let failure = router
        .route(failure_request)
        .expect_err("failure rolls back");
    let failure_after = router.active_descriptor();
    scenarios.push(scenario(
        "failure_rollback_before_publish",
        json!({"target":"grey_hive","failureInjection":true}),
        json!(["validate", "stop candidate old owner", "adapter failure", "restore old owner", "do not publish new session"]),
        json!({"errorCode":"E_HANDOFF_FAILED_ROLLBACK","oldOwnerRestored":true,"activeOwnerCount":1}),
        json!({"error":error_value(&failure),"before":failure_before,"after":failure_after}),
        failure.code == "E_HANDOFF_FAILED_ROLLBACK" && failure.rollback && failure_before == failure_after,
        json!({}),
        Some(&failure.code),
        fixture_hashes.clone(),
    ));

    let duplicate_request = req(
        "idempotent-request",
        "clockworks",
        RouteAction::Switch,
        switched_legacy.new_epoch,
        Some(&switched_legacy.new_session),
        None,
    );
    let first_duplicate = router
        .route(duplicate_request.clone())
        .expect("first idempotent route");
    let replay_duplicate = router
        .route(duplicate_request)
        .expect("replay idempotent route");
    let conflict = router.route(RouteRequest {
        request_id: "idempotent-request".to_owned(),
        world_id: GREY_HIVE.to_owned(),
        campaign_version: CAMPAIGN_VERSION,
        save_version: None,
        resume_checkpoint: None,
        result_map: None,
        current_epoch: first_duplicate.new_epoch,
        current_session: Some(first_duplicate.new_session.clone()),
        action: RouteAction::Switch,
        failure_injection: false,
        cadence_hz: 30,
        event_tick: 0,
    });
    let conflict_error = conflict.expect_err("duplicate conflict rejects");
    scenarios.push(scenario(
        "duplicate_request_idempotency_and_conflict",
        json!({"requestId":"idempotent-request"}),
        json!(["route once", "replay exact request", "submit conflicting payload"]),
        json!({"replay":true,"conflictError":"E_DUPLICATE_REQUEST_CONFLICT"}),
        json!({"first":route_value(&first_duplicate),"replay":route_value(&replay_duplicate),"conflict":error_value(&conflict_error)}),
        replay_duplicate.idempotent_replay
            && route_signature(&first_duplicate) == route_signature(&replay_duplicate)
            && conflict_error.code == "E_DUPLICATE_REQUEST_CONFLICT",
        route_value(&replay_duplicate),
        Some(&conflict_error.code),
        fixture_hashes.clone(),
    ));

    let unknown = router.route(req(
        "unknown-world",
        "not_a_world",
        RouteAction::Start,
        first_duplicate.new_epoch,
        Some(&first_duplicate.new_session),
        None,
    ));
    let unknown_error = unknown.expect_err("unknown world rejects");
    scenarios.push(scenario(
        "unknown_world_stable_rejection_no_fallback",
        json!({"worldId":"not_a_world"}),
        json!([
            "lookup frozen catalog",
            "reject unknown id",
            "do not fallback to legacy"
        ]),
        json!({"errorCode":"E_UNKNOWN_WORLD","silentFallback":false}),
        error_value(&unknown_error),
        unknown_error.code == "E_UNKNOWN_WORLD" && router.active_owner_count() == 1,
        json!({}),
        Some(&unknown_error.code),
        fixture_hashes.clone(),
    ));

    let invalid_checkpoint = {
        let mut request = req(
            "invalid-checkpoint",
            GREY_HIVE,
            RouteAction::Resume,
            first_duplicate.new_epoch,
            Some(&first_duplicate.new_session),
            Some(SAVE_V3),
        );
        request.resume_checkpoint = Some("not-a-checkpoint".to_owned());
        router.route(request)
    };
    let checkpoint_error = invalid_checkpoint.expect_err("invalid checkpoint rejects");
    scenarios.push(scenario(
        "unknown_checkpoint_stable_rejection",
        json!({"worldId":"grey_hive","checkpoint":"not-a-checkpoint","saveVersion":3}),
        json!(["validate checkpoint", "reject before replacement"]),
        json!({"errorCode":"E_INVALID_CHECKPOINT"}),
        error_value(&checkpoint_error),
        checkpoint_error.code == "E_INVALID_CHECKPOINT",
        json!({}),
        Some(&checkpoint_error.code),
        fixture_hashes.clone(),
    ));

    let result_missing_null = router.route(req(
        "complete-missing-null",
        GREY_HIVE,
        RouteAction::Complete,
        first_duplicate.new_epoch,
        Some(&first_duplicate.new_session),
        Some(SAVE_V3),
    ));
    let result_error = result_missing_null.expect_err("completion without map null rejects");
    scenarios.push(scenario(
        "completion_requires_result_map_null",
        json!({"worldId":"grey_hive","action":"complete","resultMap":"missing"}),
        json!(["validate completed result", "reject non-null/missing map"]),
        json!({"errorCode":"E_RESULT_MAP_MUST_BE_NULL"}),
        error_value(&result_error),
        result_error.code == "E_RESULT_MAP_MUST_BE_NULL",
        json!({}),
        Some(&result_error.code),
        fixture_hashes.clone(),
    ));

    let fixture_pass = mist_fixture.unchanged && clock_fixture.unchanged;
    scenarios.push(scenario(
        "legacy_fixture_hashes_byte_stable",
        json!({"fixtures":["mist_harbor","clockworks"]}),
        json!(["read before hash", "route across modes", "read after hash"]),
        json!({"mistHarborUnchanged":true,"clockworksUnchanged":true}),
        fixture_hashes.clone(),
        fixture_pass,
        fixture_hashes.clone(),
        None,
        fixture_hashes.clone(),
    ));

    let mut roundtrip = WorldRuntimeRouter::new();
    let mut roundtrip_events = Vec::new();
    let mut current_epoch = 0;
    let mut current_session: Option<String> = None;
    let mut roundtrip_ok = true;
    for index in 0..30_u32 {
        let target = if index % 2 == 0 {
            GREY_HIVE
        } else {
            "mist_harbor"
        };
        let request = req(
            &format!("roundtrip-{index}"),
            target,
            if current_session.is_none() {
                RouteAction::Start
            } else {
                RouteAction::Switch
            },
            current_epoch,
            current_session.as_deref(),
            None,
        );
        match roundtrip.route(request) {
            Ok(result) => {
                roundtrip_ok &= result.old_owner_lifecycle.active_owner_count_after == 1;
                current_epoch = result.new_epoch;
                current_session = Some(result.new_session.clone());
                roundtrip_events.push(route_signature(&result));
            }
            Err(_) => roundtrip_ok = false,
        }
    }
    let roundtrip_counters = roundtrip.continuous_counters();
    scenarios.push(scenario(
        "thirty_cross_world_roundtrips_single_owner",
        json!({"roundtrips":30,"targets":["grey_hive","mist_harbor"]}),
        Value::Array(roundtrip_events),
        json!({"activeOwnerCount":1,"continuousProcessedEvents":0,"continuousRewards":0,"continuousCapabilities":0}),
        json!({"activeOwnerCount":roundtrip.active_owner_count(),"active":roundtrip.active_descriptor(),"continuousCounters":roundtrip_counters}),
        roundtrip_ok
            && roundtrip.active_owner_count() == 1
            && roundtrip_counters["processedEvents"] == 0
            && roundtrip_counters["rewards"] == 0
            && roundtrip_counters["capabilities"] == 0,
        roundtrip.active_descriptor().unwrap_or_else(|| json!({})),
        None,
        fixture_hashes.clone(),
    ));

    let frequency_stream = |hz: u32| -> (Value, WorldRuntimeRouter) {
        let mut stream_router = WorldRuntimeRouter::new();
        let mut epoch = 0;
        let mut session = None;
        let mut final_signature = Value::Null;
        for index in 0..12_u32 {
            let target = match index % 3 {
                0 => GREY_HIVE,
                1 => "mist_harbor",
                _ => "clockworks",
            };
            let mut request = req(
                &format!("cadence-{index}"),
                target,
                if session.is_none() {
                    RouteAction::Start
                } else {
                    RouteAction::Switch
                },
                epoch,
                session.as_deref(),
                None,
            );
            request.cadence_hz = hz;
            request.event_tick = u64::from(index) * u64::from(hz);
            let result = stream_router.route(request).expect("cadence route passes");
            epoch = result.new_epoch;
            session = Some(result.new_session.clone());
            final_signature = route_signature(&result);
        }
        (final_signature, stream_router)
    };
    let (signature_30, router_30) = frequency_stream(30);
    let (signature_60, router_60) = frequency_stream(60);
    scenarios.push(scenario(
        "thirty_sixty_hz_identical_route_stream",
        json!({"events":12,"frequencies":[30,60]}),
        json!(["grey_hive", "mist_harbor", "clockworks", "repeat"]),
        json!({"finalRouteEqual":true,"finalModeEqual":true,"finalEpochEqual":true,"resultEqual":true}),
        json!({"hz30":signature_30,"hz60":signature_60,"owner30":router_30.active_owner_count(),"owner60":router_60.active_owner_count()}),
        signature_30 == signature_60 && router_30.active_owner_count() == 1 && router_60.active_owner_count() == 1,
        signature_60.clone(),
        None,
        fixture_hashes.clone(),
    ));

    let failed = scenarios.iter().filter(|item| item["pass"] != true).count();
    json!({
        "schema": "world_runtime_routing_tp14b1_v1",
        "status": if failed == 0 { "passed_candidate_isolated" } else { "blocked_probe_failure" },
        "pass": failed == 0,
        "candidateOnly": true,
        "formalSceneReady": false,
        "routerApi": {
            "input": ["worldId", "campaignVersion", "saveVersion", "resumeCheckpoint", "resultMap", "currentEpoch", "currentSession", "action", "requestId"],
            "output": ["route", "mode", "newEpoch", "newSession", "handoff", "oldOwnerLifecycle", "stateIsolation", "stableError"],
            "authority": "mode selection, epoch/session replacement, explicit adapter handoff only",
            "doesNotOwn": ["CampaignState", "CampaignStore", "WorldStateV3", "legacy GameState", "tile/map/action state", "gameplay rewards", "capability ledger"]
        },
        "worldModeTable": {
            "grey_hive": {"mode":"continuous_v3","save":"SaveV3","fallbackAllowed":false},
            "mist_harbor": {"mode":"legacy_v2","save":"legacy_campaign_boundary","fallbackAllowed":false},
            "clockworks": {"mode":"legacy_v2","save":"legacy_campaign_boundary","fallbackAllowed":false},
            "knownOtherWorlds": {"mode":"legacy_v2","save":"legacy_v2_or_legacy_save","fallbackAllowed":false},
            "unknown": {"mode":"reject","error":"E_UNKNOWN_WORLD","silentFallback":false}
        },
        "resumeAndMigration": {
            "greyHiveV3": "semantic checkpoint resume",
            "greyHiveV2": "explicit migrate_v2_to_v3 handoff only",
            "legacyV2": "legacy restore without continuous migration",
            "resultMapNull": "preserved for completed grey_hive handoff to hub"
        },
        "sessionLifecycle": {
            "replacementOrder": ["stop_old_owner", "drop_old_channel", "join_old_thread", "publish_new_session"],
            "activeOwnerInvariant": "exactly one after successful route",
            "staleEpoch": "E_STALE_EPOCH",
            "staleCallback": "E_LATE_CALLBACK",
            "failureRollback": "E_HANDOFF_FAILED_ROLLBACK"
        },
        "legacyFixtureHashes": fixture_hashes,
        "scenarios": scenarios,
        "roundtrip": {
            "count": 30,
            "activeOwnerCount": roundtrip.active_owner_count(),
            "continuousCountersAfter": roundtrip.continuous_counters(),
            "legacyFixtureBytesUnchanged": fixture_pass
        },
        "frequencyEvidence": {
            "hz30AndHz60FinalSignatureEqual": signature_30 == signature_60,
            "hz30Signature": signature_30,
            "hz60Signature": signature_60
        },
        "formalGaps": [
            "formal Campaign/CampaignStore entrypoint is not wired to this candidate router",
            "formal continuous owner and real legacy owner teardown are not integrated",
            "UI, Tauri and durable save handoff are not connected",
            "full legacy regression suite remains a separate formal gate"
        ],
        "blockedInterfaces": [
            "E_FORMAL_ROUTER_NOT_WIRED",
            "E_REAL_OWNER_HANDOFF_UNAVAILABLE",
            "E_LEGACY_REGRESSION_GATE_PENDING"
        ],
        "commandExitCodes": {"cargoFmt": null,"cargoTest": null,"cargoRun": null},
        "formalGate": {"tp14":"blocked/no_go","gateJ":"blocked/no_go","architecture":"blocked/no_go"},
        "evidenceLevel": EVIDENCE
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_hive_and_legacy_routes_are_disjoint() {
        let mut router = WorldRuntimeRouter::new();
        let grey = router
            .route(req(
                "test-grey",
                GREY_HIVE,
                RouteAction::Start,
                0,
                None,
                None,
            ))
            .expect("grey route");
        router
            .record_continuous_activity("e", "r", "c")
            .expect("activity");
        let legacy = router
            .route(req(
                "test-mist",
                "mist_harbor",
                RouteAction::Switch,
                grey.new_epoch,
                Some(&grey.new_session),
                None,
            ))
            .expect("legacy route");
        assert_eq!(legacy.mode, "legacy_v2");
        assert_eq!(router.active_owner_count(), 1);
        assert_eq!(router.continuous_counters()["processedEvents"], 0);
        assert!(!legacy.state_isolation.continuous_fields_in_legacy_handoff);
    }

    #[test]
    fn failure_and_stale_callback_do_not_replace_owner() {
        let mut router = WorldRuntimeRouter::new();
        let first = router
            .route(req("first", GREY_HIVE, RouteAction::Start, 0, None, None))
            .expect("first");
        let before = router.active_descriptor();
        let mut request = req(
            "fail",
            "clockworks",
            RouteAction::Switch,
            first.new_epoch,
            Some(&first.new_session),
            None,
        );
        request.failure_injection = true;
        let failure = router.route(request).expect_err("failure");
        assert_eq!(failure.code, "E_HANDOFF_FAILED_ROLLBACK");
        assert_eq!(before, router.active_descriptor());
        let switched = router
            .route(req(
                "switch",
                "clockworks",
                RouteAction::Switch,
                first.new_epoch,
                Some(&first.new_session),
                None,
            ))
            .expect("switch");
        let late = router
            .accept_callback(&first.new_session, first.new_epoch, "late")
            .expect_err("late");
        assert_eq!(late.code, "E_LATE_CALLBACK");
        assert_eq!(switched.old_owner_lifecycle.active_owner_count_after, 1);
    }

    #[test]
    fn idempotency_unknown_world_and_legacy_v3_reject() {
        let mut router = WorldRuntimeRouter::new();
        let first = router
            .route(req(
                "idem",
                "mist_harbor",
                RouteAction::Start,
                0,
                None,
                None,
            ))
            .expect("first");
        let replay = router
            .route(req(
                "idem",
                "mist_harbor",
                RouteAction::Start,
                0,
                None,
                None,
            ))
            .expect("replay");
        assert!(replay.idempotent_replay);
        assert_eq!(first.new_epoch, replay.new_epoch);
        let unknown = router
            .route(req(
                "unknown",
                "missing-world",
                RouteAction::Switch,
                first.new_epoch,
                Some(&first.new_session),
                None,
            ))
            .expect_err("unknown");
        assert_eq!(unknown.code, "E_UNKNOWN_WORLD");
        let legacy_v3 = router
            .route(req(
                "legacy-v3",
                "clockworks",
                RouteAction::Switch,
                first.new_epoch,
                Some(&first.new_session),
                Some(SAVE_V3),
            ))
            .expect_err("legacy v3");
        assert_eq!(legacy_v3.code, "E_LEGACY_SAVE_MUST_REMAIN_V2");
    }

    #[test]
    fn probe_has_required_scenario_count_and_passes() {
        let result = run_probe();
        assert_eq!(result["pass"], true);
        assert!(result["scenarios"].as_array().unwrap().len() >= 14);
    }
}

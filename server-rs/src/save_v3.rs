//! Validated persistence DTO for the formal Continuous World owner.
use crate::{
    capability_v1::CapabilityState,
    continuous_combat::CombatState,
    continuous_kcc::KccBody,
    formal_runtime::RuntimeState,
    sentinel_ai::{Sentinel, SentinelState},
    world_progression::{RouteState, WORLD_GREY_HIVE},
    world_v3::{ExploredMap, RearViewAuthorization, Vec3, WorldRevision, WorldStateV3},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const SAVE_SCHEMA_VERSION: u32 = 4;
pub const SAVE_FILE_NAME: &str = "formal-save-v3.json";
pub const CONTENT_VERSION: &str = "grey-hive-first-flow/1";
const PRIOR_CONTENT_VERSION: &str = "grey-hive-open-room-blockout/1";
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerStateV3 {
    pub position_m: Vec3,
    pub velocity_mps: Vec3,
    pub radius_m: f32,
    pub grounded: bool,
    pub move_speed_mps: f32,
    pub jump_speed_mps: f32,
    pub gravity_mps2: f32,
    pub dash_speed_mps: f32,
    pub dash_remaining_ms: u64,
    pub dash_cooldown_remaining_ms: u64,
    pub current_hp: u32,
    pub max_hp: u32,
    #[serde(default = "starting_energy")]
    pub current_energy: u32,
    #[serde(default = "starting_energy")]
    pub max_energy: u32,
}

fn starting_energy() -> u32 {
    100
}

impl From<&KccBody> for PlayerStateV3 {
    fn from(body: &KccBody) -> Self {
        Self {
            position_m: body.position_m,
            velocity_mps: body.velocity_mps,
            radius_m: body.radius_m,
            grounded: body.grounded,
            move_speed_mps: body.move_speed_mps,
            jump_speed_mps: body.jump_speed_mps,
            gravity_mps2: body.gravity_mps2,
            dash_speed_mps: body.dash_speed_mps,
            dash_remaining_ms: body.dash_remaining_ms,
            dash_cooldown_remaining_ms: body.dash_cooldown_remaining_ms,
            current_hp: 0,
            max_hp: 0,
            current_energy: starting_energy(),
            max_energy: starting_energy(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SentinelStateV3 {
    pub entity_id: String,
    pub position_m: Vec3,
    pub hp: u32,
    pub state: SentinelState,
    pub state_remaining_ms: u64,
    pub attack_serial: u64,
    pub active: bool,
}
impl From<&Sentinel> for SentinelStateV3 {
    fn from(value: &Sentinel) -> Self {
        Self {
            entity_id: value.entity_id.clone(),
            position_m: value.position_m,
            hp: value.hp,
            state: value.state,
            state_remaining_ms: value.state_remaining_ms,
            attack_serial: value.attack_serial,
            active: value.active,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationProvenance {
    pub source_format: String,
    pub source_version: u32,
    pub source_world_id: String,
    pub method: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveV3 {
    pub schema_version: u32,
    pub content_version: String,
    pub world_id: String,
    pub checkpoint_id: String,
    pub revision: WorldRevision,
    pub server_time_ms: u64,
    pub player: PlayerStateV3,
    pub sentinels: Vec<SentinelStateV3>,
    pub combat_handled_request_ids: Vec<u64>,
    pub explored: ExploredMap,
    pub rear_view: RearViewAuthorization,
    pub last_input_seq: u64,
    pub last_client_time_ms: u64,
    pub last_received_seq: u64,
    pub last_received_client_time_ms: u64,
    pub last_damaged_at_ms: Option<u64>,
    pub last_combat_at_ms: Option<u64>,
    pub capabilities: CapabilityState,
    pub route: RouteState,
    pub migration_provenance: Option<MigrationProvenance>,
}

impl SaveV3 {
    pub(crate) fn capture(state: &RuntimeState) -> Result<Self, String> {
        if state.world.sentinels.iter().any(|s|s.charge_controller.is_some()) {return Err("E_SAVE_LEGACY_SENTINEL_UNSUPPORTED".into());}
        let mut player = PlayerStateV3::from(&state.world.player);
        player.current_hp = state.world.player_hp;
        player.max_hp = state.world.player_max_hp;
        player.current_energy = state.world.player_energy;
        player.max_energy = state.world.player_max_energy;
        let save = Self {
            schema_version: SAVE_SCHEMA_VERSION,
            content_version: CONTENT_VERSION.into(),
            world_id: state.world.world_id.clone(),
            checkpoint_id: if route_has_power(&state.route) {
                "gh_cp_power".into()
            } else {
                "gh_cp_airlock".into()
            },
            revision: state.world.revision,
            server_time_ms: state.world.server_time_ms,
            player,
            sentinels: state.world.sentinels.iter().map(Into::into).collect(),
            combat_handled_request_ids: state.world.combat_state.handled_request_ids(),
            explored: state.world.explored.clone(),
            rear_view: state.world.rear_view.clone(),
            last_input_seq: state.world.last_input_seq,
            last_client_time_ms: state.world.last_client_time_ms,
            last_received_seq: state.last_received_seq,
            last_received_client_time_ms: state.last_client_time_ms,
            last_damaged_at_ms: state.last_damaged_at_ms,
            last_combat_at_ms: state.last_combat_at_ms,
            capabilities: state.capabilities.clone(),
            route: state.route.clone(),
            migration_provenance: None,
        };
        save.validate()?;
        Ok(save)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if !matches!(self.schema_version, 3 | SAVE_SCHEMA_VERSION) {
            return Err("E_SAVE_VERSION_UNSUPPORTED".into());
        }
        if !matches!(
            self.content_version.as_str(),
            CONTENT_VERSION | PRIOR_CONTENT_VERSION
        ) {
            return Err("E_SAVE_CONTENT_VERSION_UNSUPPORTED".into());
        }
        if self.world_id != WORLD_GREY_HIVE
            || self.explored.world_id != self.world_id
            || self.capabilities.world_id != self.world_id
        {
            return Err("E_SAVE_WORLD_MISMATCH".into());
        }
        let legacy_checkpoint = self.content_version == PRIOR_CONTENT_VERSION
            && self.checkpoint_id == "gh_cp_open_room_blockout";
        let first_flow_checkpoint =
            matches!(self.checkpoint_id.as_str(), "gh_cp_airlock" | "gh_cp_power");
        if !legacy_checkpoint && !first_flow_checkpoint {
            return Err("E_SAVE_CHECKPOINT_UNSUPPORTED".into());
        }
        if self.revision.world_epoch == 0
            || self.revision.authority_revision < self.revision.server_tick
            || self.last_input_seq != self.last_received_seq
            || self.last_client_time_ms != self.last_received_client_time_ms
            || self.player.max_hp == 0
            || self.player.current_hp > self.player.max_hp
            || self.player.max_energy == 0
            || self.player.current_energy > self.player.max_energy
            || self.capabilities.schema_version != crate::capability_v1::CAPABILITY_SCHEMA_VERSION
            || self.route.schema_version != crate::world_progression::ROUTE_SCHEMA_VERSION
            || self.route.current_world_id != self.world_id
            || !self
                .route
                .progress
                .iter()
                .any(|p| p.world_id == self.world_id)
        {
            return Err("E_SAVE_STATE_INVALID".into());
        }
        let f = [
            self.player.position_m.x_m,
            self.player.position_m.y_m,
            self.player.position_m.z_m,
            self.player.velocity_mps.x_m,
            self.player.velocity_mps.y_m,
            self.player.velocity_mps.z_m,
            self.player.radius_m,
            self.player.move_speed_mps,
            self.player.jump_speed_mps,
            self.player.gravity_mps2,
            self.player.dash_speed_mps,
            self.explored.player_position_m.x_m,
            self.explored.player_position_m.y_m,
            self.explored.player_position_m.z_m,
            self.capabilities.regeneration.hp_per_second,
            self.capabilities.regeneration.regen_cap_fraction,
        ];
        if f.iter().any(|v| !v.is_finite())
            || self.player.radius_m <= 0.0
            || [
                self.player.move_speed_mps,
                self.player.jump_speed_mps,
                self.player.gravity_mps2,
                self.player.dash_speed_mps,
            ]
            .iter()
            .any(|v| *v < 0.0)
            || !crate::continuous_kcc::StaticKccWorld::grey_hive_first_flow(route_has_power(
                &self.route,
            ))
            .can_occupy(self.player.position_m, self.player.radius_m)
        {
            return Err("E_SAVE_NUMERIC_INVALID".into());
        }
        let mut capability_ids = BTreeSet::new();
        if self
            .capabilities
            .grants
            .iter()
            .any(|g| !capability_ids.insert(g.capability_id.as_str()))
            || self
                .capabilities
                .selected
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != self.capabilities.selected.len()
        {
            return Err("E_SAVE_DUPLICATE_CAPABILITY".into());
        }
        if self.capabilities.grants.iter().any(|g| {
            !matches!(
                g.capability_id.as_str(),
                crate::capability_v1::CAP_LOCAL_MAP
                    | crate::capability_v1::CAP_REAR_VIEW
                    | crate::capability_v1::CAP_ENEMY_VITALS
                    | crate::capability_v1::CAP_REGENERATION
            )
        }) || self.capabilities.selected.iter().any(|id| {
            !self
                .capabilities
                .grants
                .iter()
                .any(|g| &g.capability_id == id)
        }) || self.sentinels.iter().any(|s| {
            matches!(s.state, SentinelState::ChargeWindup | SentinelState::Charge | SentinelState::Stagger)
                || s.entity_id.trim().is_empty()
                || ![s.position_m.x_m, s.position_m.y_m, s.position_m.z_m]
                    .iter()
                    .all(|v| v.is_finite())
        }) {
            return Err("E_SAVE_ID_OR_ENTITY_INVALID".into());
        }
        let mut sentinel_ids = BTreeSet::new();
        if self
            .sentinels
            .iter()
            .any(|s| !sentinel_ids.insert(s.entity_id.as_str()))
            || self.sentinels.iter().any(|s| s.hp == 0 && s.active)
        {
            return Err("E_SAVE_SENTINEL_INVALID".into());
        }
        let mut route_world_ids = BTreeSet::new();
        if self.route.progress.len() != 3
            || self.route.progress.iter().any(|p| {
                !matches!(
                    p.world_id.as_str(),
                    WORLD_GREY_HIVE
                        | crate::world_progression::WORLD_MIST_HARBOR
                        | crate::world_progression::WORLD_CLOCKWORKS
                ) || !route_world_ids.insert(p.world_id.as_str())
            })
        {
            return Err("E_SAVE_ROUTE_INVALID".into());
        }
        let mut route_requests = BTreeSet::new();
        if self.route.reward_ledger.iter().any(|entry| {
            entry.request_id.trim().is_empty() || !route_requests.insert(entry.request_id.as_str())
        }) {
            return Err("E_SAVE_REWARD_LEDGER_INVALID".into());
        }
        self.capabilities
            .regeneration
            .validate()
            .map_err(|_| "E_SAVE_CAPABILITY_INVALID".to_string())?;
        if self.rear_view != self.capabilities.rear_view_authorization {
            return Err("E_SAVE_AUTHORIZATION_MISMATCH".into());
        }
        Ok(())
    }

    pub(crate) fn restore(&self) -> Result<RuntimeState, String> {
        self.validate()?;
        let epoch = self
            .revision
            .world_epoch
            .checked_add(1)
            .ok_or("E_WORLD_EPOCH_EXHAUSTED")?;
        let mut world = WorldStateV3::new(self.world_id.clone(), epoch, self.player.position_m)
            .map_err(|e| format!("E_SAVE_WORLD_RESTORE: {e:?}"))?;
        world.revision = WorldRevision {
            world_epoch: epoch,
            ..self.revision
        };
        world.server_time_ms = self.server_time_ms;
        world.player.position_m = self.player.position_m;
        world.player.velocity_mps = self.player.velocity_mps;
        world.player.radius_m = self.player.radius_m;
        world.player.grounded = self.player.grounded;
        world.player.move_speed_mps = self.player.move_speed_mps;
        world.player.jump_speed_mps = self.player.jump_speed_mps;
        world.player.gravity_mps2 = self.player.gravity_mps2;
        world.player.dash_speed_mps = self.player.dash_speed_mps;
        world.player.dash_remaining_ms = self.player.dash_remaining_ms;
        world.player.dash_cooldown_remaining_ms = self.player.dash_cooldown_remaining_ms;
        world.player_hp = self.player.current_hp;
        world.player_max_hp = self.player.max_hp;
        world.player_energy = self.player.current_energy;
        world.player_max_energy = self.player.max_energy;
        world.sentinels = self
            .sentinels
            .iter()
            .map(|s| Sentinel {
                entity_id: s.entity_id.clone(),
                position_m: s.position_m,
                hp: s.hp,
                state: s.state,
                state_remaining_ms: s.state_remaining_ms,
                attack_serial: s.attack_serial,
                active: s.active,
                charge_controller: None,
            })
            .collect();
        world.combat_state =
            CombatState::from_handled_request_ids(self.combat_handled_request_ids.iter().copied())
                .map_err(|_| "E_SAVE_DUPLICATE_COMBAT_REQUEST")?;
        world.explored = self.explored.clone();
        world.rear_view = self.rear_view.clone();
        world.last_input_seq = self.last_input_seq;
        world.last_client_time_ms = self.last_client_time_ms;
        let mut capabilities = self.capabilities.clone();
        rebase_capability_revisions(&mut capabilities, epoch)?;
        let stepper = crate::fixed_step::FixedStepClock::new(
            crate::fixed_step::FixedStepConfig::new(60, 1)
                .map_err(|e| format!("E_SAVE_CLOCK: {e:?}"))?,
        );
        // The scheduler starts clean; authority time and input sequence remain persisted.
        Ok(RuntimeState {
            world,
            world_persistent_v1: crate::world_persistent_v1::WorldPersistentState::default(),
            route: self.route.clone(),
            capabilities,
            build_commands: Default::default(),
            progression_v6: crate::formal_runtime::build_v6::PlayerProgressionV6::default(),
            effect_sources_v6: Vec::new(),
            effective_rules_v6: crate::player_rules::EffectivePlayerRules::default(),
            kcc: crate::continuous_kcc::StaticKccWorld::grey_hive_first_flow(route_has_power(
                &self.route,
            )),
            stepper,
            latest_sample: crate::continuous_input::InputSample::new(
                epoch,
                self.last_received_seq,
                self.last_received_client_time_ms,
                0.0,
                0.0,
            )
            .map_err(|error| format!("E_SAVE_INPUT: {error:?}"))?,
            latest_input_received_at: std::time::Instant::now(),
            pending_combat: Vec::new(),
            presentation_events: Vec::new(),
            next_presentation_event_id: 1,
            sound_cues: Vec::new(),
            next_sound_cue_id: 1,
            last_received_seq: self.last_received_seq,
            last_client_time_ms: self.last_received_client_time_ms,
            last_damaged_at_ms: self.last_damaged_at_ms,
            last_combat_at_ms: self.last_combat_at_ms,
            paused: false,
            entry_generation: 0,
            pending_entry: None,
            entry_pause_requested: false,
            last_lifecycle_sequence: 0,
            last_owner_error: None,
        })
    }
}

fn route_has_power(route: &RouteState) -> bool {
    route
        .progress
        .iter()
        .find(|progress| progress.world_id == WORLD_GREY_HIVE)
        .is_some_and(|progress| {
            progress
                .completed_events
                .iter()
                .any(|event| event == "hive_power")
        })
}

fn rebase_capability_revisions(state: &mut CapabilityState, epoch: u64) -> Result<(), String> {
    let mut value =
        serde_json::to_value(&*state).map_err(|e| format!("E_SAVE_CAPABILITY_ENCODE: {e}"))?;
    let object = value.as_object_mut().ok_or("E_SAVE_CAPABILITY_INVALID")?;
    for key in ["currentRevision", "lastTickRevision"] {
        if let Some(revision) = object.get_mut(key).filter(|v| !v.is_null()) {
            revision["worldEpoch"] = serde_json::json!(epoch);
        }
    }
    *state = serde_json::from_value(value).map_err(|_| "E_SAVE_CAPABILITY_INVALID")?;
    Ok(())
}

pub fn save_path(root: &Path) -> PathBuf {
    root.join(SAVE_FILE_NAME)
}
pub fn has_valid_save(root: &Path) -> bool {
    read_save(root)
        .and_then(|save| save.restore().map(|_| save))
        .is_ok()
}

/// Legacy grid saves have no authenticated continuous-world checkpoint mapping.
/// Only their format header is inspected to select a rejection code. No legacy
/// story module is loaded and no caller can obtain an accepted/migrated save.
/// Unlike the former full DTO decode, unrelated old payload fields do not affect
/// rejection classification. Every input is still rejected without any writes.
pub fn inspect_legacy_grid_v2(raw: &str) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct LegacyHeader {
        #[serde(default)]
        save_version: u32,
        #[serde(default = "legacy_world")]
        world_id: String,
    }
    fn legacy_world() -> String { "biohazard_ch1".to_owned() }
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| "E_LEGACY_SAVE_CORRUPT".to_string())?;
    if !value.is_object() { return Err("E_LEGACY_SAVE_CORRUPT".into()); }
    let old: LegacyHeader =
        serde_json::from_str(raw).map_err(|_| "E_LEGACY_SAVE_CORRUPT".to_string())?;
    if old.save_version != 2 {
        return Err("E_LEGACY_SAVE_VERSION_UNSUPPORTED".into());
    }
    if old.world_id != "biohazard_ch1" {
        return Err("E_LEGACY_SAVE_WORLD_UNSUPPORTED".into());
    }
    Err("E_V2_GRID_NO_CHECKPOINT_MAPPING: historical GameState v2 has no authenticated Grey Hive semantic checkpoint mapping; original retained".into())
}

#[cfg(test)]
mod legacy_header_rejection_tests {
    use super::inspect_legacy_grid_v2;
    #[test]
    fn every_legacy_header_is_rejected_without_payload_migration() {
        for raw in [r#"{"save_version":2,"world_id":"biohazard_ch1"}"#,
                    r#"{"save_version":2}"#,
                    r#"{"save_version":2,"world_id":"biohazard_ch1","px":123,"py":456}"#] {
            let unchanged = raw.to_owned();
            assert!(inspect_legacy_grid_v2(raw).unwrap_err().starts_with("E_V2_GRID_NO_CHECKPOINT_MAPPING"));
            assert_eq!(raw, unchanged);
        }
        assert_eq!(inspect_legacy_grid_v2("{}").unwrap_err(), "E_LEGACY_SAVE_VERSION_UNSUPPORTED");
        assert_eq!(inspect_legacy_grid_v2(r#"{"save_version":2,"world_id":"other"}"#).unwrap_err(), "E_LEGACY_SAVE_WORLD_UNSUPPORTED");
        for raw in ["not json", "[]", "null", r#"{"save_version":"2"}"#, r#"{"save_version":2,"world_id":null}"#] {
            assert_eq!(inspect_legacy_grid_v2(raw).unwrap_err(), "E_LEGACY_SAVE_CORRUPT");
        }
    }
}

pub fn read_save(root: &Path) -> Result<SaveV3, String> {
    let path = save_path(root);
    let mut file = File::open(&path).map_err(|_| "E_NO_SAVE".to_string())?;
    let size = file
        .metadata()
        .map_err(|e| format!("E_SAVE_READ: {e}"))?
        .len();
    if size > 8 * 1024 * 1024 {
        return Err("E_SAVE_TOO_LARGE".into());
    }
    let mut raw = String::with_capacity(size as usize);
    file.read_to_string(&mut raw)
        .map_err(|e| format!("E_SAVE_READ: {e}"))?;
    let save: SaveV3 = serde_json::from_str(&raw).map_err(|_| "E_SAVE_CORRUPT".to_string())?;
    save.validate()?;
    Ok(save)
}

pub fn write_save(root: &Path, save: &SaveV3) -> Result<(), String> {
    save.validate()?;
    fs::create_dir_all(root).map_err(|e| format!("E_SAVE_DIR: {e}"))?;
    let target = save_path(root);
    let token = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let temp = root.join(format!(
        ".formal-save-v3.{}.{}.tmp",
        std::process::id(),
        token
    ));
    let backup = root.join(format!(
        "{SAVE_FILE_NAME}.bak.{}.{}",
        std::process::id(),
        token
    ));
    let bytes = serde_json::to_vec_pretty(save).map_err(|e| format!("E_SAVE_ENCODE: {e}"))?;
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("E_SAVE_TEMP: {e}"))?;
        file.write_all(&bytes)
            .map_err(|e| format!("E_SAVE_WRITE: {e}"))?;
        file.sync_all().map_err(|e| format!("E_SAVE_SYNC: {e}"))?;
        let mut written = String::new();
        File::open(&temp)
            .and_then(|mut f| f.read_to_string(&mut written))
            .map_err(|e| format!("E_SAVE_VERIFY_READ: {e}"))?;
        let check: SaveV3 =
            serde_json::from_str(&written).map_err(|_| "E_SAVE_VERIFY".to_string())?;
        check.validate()?;
        if check != *save {
            return Err("E_SAVE_VERIFY_MISMATCH".into());
        }
        if target.exists() {
            let _ = fs::remove_file(&backup);
            fs::rename(&target, &backup).map_err(|e| format!("E_SAVE_BACKUP: {e}"))?;
        }
        if let Err(error) = fs::rename(&temp, &target) {
            if backup.exists() {
                let _ = fs::rename(&backup, &target);
            }
            return Err(format!("E_SAVE_COMMIT: {error}"));
        }
        let persisted = read_save(root)?;
        if persisted != *save {
            return Err("E_SAVE_POSTWRITE_MISMATCH".into());
        }
        Ok(())
    })();
    if temp.exists() {
        let _ = fs::remove_file(temp);
    }
    write_result
}

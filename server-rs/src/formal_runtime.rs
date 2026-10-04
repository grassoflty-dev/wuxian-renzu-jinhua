//! Single Rust owner for the Freeze v0.2 continuous runtime and route state.
//!
//! Tauri commands submit validated input or route intent to this owner. The
//! browser cannot choose a timestep, mutate world state, or advance route data.

use crate::{
    capability_v1::{
        apply_command_at_revision, choose_first_enhancement, project_capabilities_with_rules,
        tick_regeneration_authorized, CapabilityCommand, CapabilityState, CapabilityTickInput,
        RegenerationConfig, CAPABILITY_SCHEMA_VERSION, CAP_ACOUSTIC_MAPPING, CAP_AIR_STEP,
        CAP_ENEMY_VITALS, CAP_LOCAL_MAP, CAP_REAR_VIEW, CAP_REGENERATION,
    },
    continuous_combat::{horizontal_distance, CombatIntent, TraversalMarker},
    continuous_input::{InputSample, INPUT_PROTOCOL, INPUT_PROTOCOL_VERSION},
    continuous_kcc::StaticKccWorld,
    effects::CapabilityPermission,
    fixed_step::{FixedStepClock, FixedStepConfig},
    scene_registry::{valid_id, WorldRegistry},
    scene_runtime::{SceneEvent, SceneRuntime},
    world_progression::{
        apply_route_command, project_route, RewardPolicy, RouteCommand, RouteResult, RouteState,
    },
    world_v3::{
        apply_effects_atomically, step_world_held_input, ActorView, CommandReceipt,
        ContinuousWorldError, CwStepInput, DoorView, EnemyVitals, PresentationEvent,
        RouteProjection, SoundCueEvent, Transform, Vec3, WorldStateV3, WorldView,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "../tests/support/entry_readiness.rs"]
pub(crate) mod entry_test_support;

pub mod build_ui;
pub mod build_v6;
mod capability_sources;
#[cfg(test)]
mod world_gate_dedup_tests;
#[cfg(test)]
mod sentinel_charge_tests;
mod clockworks_core;
mod clockworks_campaign;
pub(crate) mod clockworks_controls;
pub(crate) mod clockworks_roster;
pub(crate) mod gear_shaft_support;
pub(crate) mod grey_hive_roster;
pub(crate) mod grey_hive_beacon;
pub(crate) mod swarm_roster;
pub(crate) mod tidebound_roster;
pub(crate) mod signal_wraith_roster;
mod signal_wraith_presentation;
mod encounter_hazards;
mod ordinary_enemy_presentation;
mod combat_presentation;
pub(crate) mod environmental_hazards;
mod live_rules;
pub(crate) mod warden_encounter;
#[cfg(feature = "deterministic-replay")]
mod replay;

const FIXED_HZ: u32 = 60;
const MAX_STEPS_PER_FRAME: u32 = 1;
const MAX_PREFLIGHT_SAVE_BYTES: u64 = 8 * 1024 * 1024;
// Legacy GH migration blockout only. A loaded SceneRuntime never resolves IDs or coordinates here.
pub const POWER_CONSOLE_ID: &str = "gh_power_console";
const GATE_A_ID: &str = "gh_gate_a";
const POWER_EVENT_ID: &str = "hive_power";
const SENTINEL_ENTITY_TYPE: &str = "enemy.grey_hive.sentinel";
const SENTINEL_SCENE_ID: &str = "gh_sentinel_arena";
const SENTINEL_SPAWN_ID: &str = "gh_sentinel_arena_sentinel_01";
const CLOCKWORKS_ELITE_ENTITY_TYPE: &str = "enemy.clockworks.forged_guard_elite";
const CLOCKWORKS_ELITE_SCENE_ID: &str = "cw_forged_guard_arena";
const CLOCKWORKS_REGULATOR_ENTITY_TYPE: &str = "enemy.clockworks.prime_regulator";
const CLOCKWORKS_REGULATOR_SCENE_ID: &str = "cw_regulator_core";

fn is_clockworks_elite_arena_exit(
    source_world: &str,
    source_scene_id: &str,
    state_world_id: &str,
    state_scene_id: &str,
    transition_id: &str,
) -> bool {
    source_world == "clockworks"
        && source_scene_id == CLOCKWORKS_ELITE_SCENE_ID
        && state_world_id == "clockworks"
        && state_scene_id == CLOCKWORKS_ELITE_SCENE_ID
        && matches!(
            transition_id,
            "cw_arena_return_to_furnace_heart" | "cw_arena_to_regulator_core"
        )
}
const CLOCKWORKS_ELITE_SPAWN_ID: &str = "cw_forged_guard_elite";
const CLOCKWORKS_REGULATOR_SPAWN_ID: &str = "cw_prime_regulator";
// Temporary development HP; final combat balance is still pending.
const SENTINEL_DEVELOPMENT_HP: u32 = 100;
const POWER_CONSOLE_POSITION: Vec3 = Vec3 {
    x_m: 2.0,
    y_m: 0.0,
    z_m: 3.0,
};
const GATE_A_POSITION: Vec3 = Vec3 {
    x_m: 0.0,
    y_m: 0.0,
    z_m: 8.0,
};
const POWER_INTERACTION_RANGE_M: f32 = 2.5;
const CAPABILITY_DATA: &str = include_str!("../data/capability_v1.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityData {
    schema_version: u32,
    canonical_capability_ids: Vec<String>,
    regeneration: RegenerationConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TraversalSceneDefinition {
    schema_version: u32,
    world_id: String,
    scene_id: String,
    bounds_m: TraversalSceneBounds,
    #[serde(default)]
    collision: Vec<TraversalCollisionPolygon>,
    logic: TraversalSceneLogic,
}

#[derive(Deserialize)]
struct TraversalSceneBounds {
    width: f32,
    depth: f32,
}

#[derive(Deserialize)]
struct TraversalCollisionPolygon {
    polygon: Vec<[f32; 2]>,
}

#[derive(Deserialize)]
struct TraversalSceneLogic {
    traversal: Vec<TraversalMarkerDefinition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TraversalMarkerDefinition {
    #[serde(default)]
    from_height_range_m: Option<crate::moving_support::HeightRange>,
    id: String,
    from: [f32; 3],
    to: [f32; 3],
    range_m: f32,
    cooldown_ms: u64,
    #[serde(default)]
    required_capabilities: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CombatIntentRequest {
    Attack {
        request_id: u64,
    },
    /// Legacy v0.2 migration input only. Formal Action-v2 traversal never maps through Jump.
    Jump {
        request_id: u64,
    },
    Dash {
        request_id: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionCommandRequest {
    pub protocol_version: u32,
    pub world_epoch: u64,
    pub request_id: u64,
    pub client_time_ms: u64,
    pub kind: ActionKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionKind {
    PrimaryAttack,
    Dash,
    Pulse,
    GuardStart,
    GuardEnd,
    /// Accepted as a v2 migration alias; it has no gameplay effect yet.
    Guard,
    Pierce,
    Interact,
    ContextTraversal,
}

impl From<CombatIntentRequest> for CombatIntent {
    fn from(value: CombatIntentRequest) -> Self {
        match value {
            CombatIntentRequest::Attack { request_id } => Self::Attack { request_id },
            CombatIntentRequest::Jump { request_id } => Self::Jump { request_id },
            CombatIntentRequest::Dash { request_id } => Self::Dash { request_id },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RouteCommandRequest {
    Enter {
        world_id: String,
        request_id: String,
    },
    Progress {
        event_id: String,
        request_id: String,
    },
    Complete {
        world_id: String,
        request_id: String,
    },
    Revisit {
        world_id: String,
        request_id: String,
    },
    GrantReward {
        world_id: String,
        reward_id: String,
        policy: RewardPolicy,
        request_id: String,
        visit_id: Option<u64>,
        cycle_id: Option<u64>,
    },
}

/// Browser commands can select/activate owned capabilities, but cannot grant
/// capabilities or provide a rear-view authorization DTO.
#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CapabilityCommandRequest {
    Select {
        capability_ids: Vec<String>,
    },
    Activate {
        capability_id: String,
        request_id: u64,
    },
}

impl From<CapabilityCommandRequest> for CapabilityCommand {
    fn from(value: CapabilityCommandRequest) -> Self {
        match value {
            CapabilityCommandRequest::Select { capability_ids } => Self::Select { capability_ids },
            CapabilityCommandRequest::Activate {
                capability_id,
                request_id,
            } => Self::Activate {
                capability_id,
                request_id,
            },
        }
    }
}

impl RouteCommandRequest {
    fn into_route_command(self) -> RouteCommand {
        match self {
            Self::Enter {
                world_id,
                request_id,
            } => RouteCommand::Enter {
                world_id,
                request_id,
            },
            Self::Progress {
                event_id,
                request_id,
            } => RouteCommand::Progress {
                event_id,
                request_id,
            },
            Self::Complete {
                world_id,
                request_id,
            } => RouteCommand::Complete {
                world_id,
                request_id,
            },
            Self::Revisit {
                world_id,
                request_id,
            } => RouteCommand::Revisit {
                world_id,
                request_id,
            },
            Self::GrantReward {
                world_id,
                reward_id,
                policy,
                request_id,
                visit_id,
                cycle_id,
            } => RouteCommand::GrantReward {
                world_id,
                reward_id,
                policy,
                request_id,
                visit_id,
                cycle_id,
            },
        }
    }

    fn target_world(&self) -> Option<&str> {
        match self {
            Self::Enter { world_id, .. }
            | Self::Complete { world_id, .. }
            | Self::Revisit { world_id, .. }
            | Self::GrantReward { world_id, .. } => Some(world_id),
            Self::Progress { .. } => None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormalRouteResponse {
    pub applied: bool,
    pub error_code: Option<String>,
    pub events: Vec<String>,
    pub view: WorldView,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormalInteractionResponse {
    pub applied: bool,
    pub already_applied: bool,
    pub error_code: Option<String>,
    pub events: Vec<String>,
    pub view: WorldView,
    pub receipt: CommandReceipt,
}

#[derive(Clone)]
pub(crate) struct RuntimeState {
    pub(crate) world: WorldStateV3,
    pub(crate) world_persistent_v1: crate::world_persistent_v1::WorldPersistentState,
    pub(crate) route: RouteState,
    pub(crate) capabilities: CapabilityState,
    pub(crate) progression_v6: build_v6::PlayerProgressionV6,
    pub(crate) build_commands: build_ui::BuildCommandState,
    pub(crate) effect_sources_v6: Vec<crate::effects::EffectSource>,
    pub(crate) effective_rules_v6: crate::player_rules::EffectivePlayerRules,
    pub(crate) kcc: StaticKccWorld,
    pub(crate) stepper: FixedStepClock,
    pub(crate) latest_sample: InputSample,
    pub(crate) latest_input_received_at: Instant,
    pub(crate) pending_combat: Vec<CombatIntent>,
    pub(crate) presentation_events: Vec<PresentationEvent>,
    pub(crate) next_presentation_event_id: u64,
    pub(crate) sound_cues: Vec<SoundCueEvent>,
    pub(crate) next_sound_cue_id: u64,
    pub(crate) last_received_seq: u64,
    pub(crate) last_client_time_ms: u64,
    pub(crate) last_damaged_at_ms: Option<u64>,
    pub(crate) last_combat_at_ms: Option<u64>,
    pub(crate) paused: bool,
    pub(crate) entry_generation: u64,
    pub(crate) pending_entry: Option<crate::world_v3::SceneEntryToken>,
    pub(crate) entry_pause_requested: bool,
    pub(crate) last_lifecycle_sequence: u64,
    pub(crate) last_owner_error: Option<String>,
}

/// An exact live journey identity for asynchronous lifecycle commands.
/// Continue/New/scene/world transitions always advance the owner epoch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionContext {
    pub world_id: String,
    pub scene_id: String,
    pub world_epoch: u64,
}

/// Managed by Tauri as the sole formal runtime owner.
pub struct FormalRuntime {
    state: Arc<Mutex<RuntimeState>>,
    scene_runtime: Arc<Mutex<Option<SceneRuntime>>>,
    scene_registry: Mutex<Option<WorldRegistry>>,
    save_root: PathBuf,
    active_slot: Mutex<Option<String>>,
    stop_owner: Arc<AtomicBool>,
    owner_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    #[cfg(feature = "deterministic-replay")]
    input_replay: bool,
}

impl FormalRuntime {
    pub fn new() -> Result<Self, String> {
        let save_root =
            std::env::temp_dir().join(format!("wuxian-formal-runtime-test-{}", std::process::id()));
        Self::new_with_save_dir(save_root)
    }

    pub fn new_with_save_dir(save_root: PathBuf) -> Result<Self, String> {
        let state = Arc::new(Mutex::new(Self::initial_state(1)?));
        let scene_runtime = Arc::new(Mutex::new(None));
        let stop_owner = Arc::new(AtomicBool::new(false));
        let owner_state = Arc::clone(&state);
        let owner_scene_runtime = Arc::clone(&scene_runtime);
        let owner_stop = Arc::clone(&stop_owner);
        let owner_handle = std::thread::Builder::new()
            .name("wuxian-simulation-60hz".into())
            .spawn(move || simulation_owner_loop(owner_state, owner_scene_runtime, owner_stop))
            .map_err(|error| format!("E_OWNER_START: {error}"))?;
        Ok(Self {
            state,
            scene_runtime,
            scene_registry: Mutex::new(None),
            save_root,
            active_slot: Mutex::new(None),
            stop_owner,
            owner_handle: Mutex::new(Some(owner_handle)),
            #[cfg(feature = "deterministic-replay")]
            input_replay: false,
        })
    }

    pub fn has_save(&self) -> bool {
        self.has_default_save() || self.list_save_slots().iter().any(|slot| slot.valid)
    }

    /// Read-only default-save probe. Match Continue's winning source without
    /// allowing a corrupt newer file to advertise a valid older fallback.
    pub fn has_default_save(&self) -> bool {
        if self.reject_dead_save(None).is_err() || self.reject_staged_clockworks_save(None).is_err() {
            return false;
        }
        let save = if crate::save_v6::path_present(&crate::save_v6::save_path(&self.save_root)) {
            crate::save_v6::read_save(&self.save_root)
        } else if crate::save_v6::path_present(&crate::save_v5::save_path(&self.save_root)) {
            crate::save_v5::read_save(&self.save_root).and_then(crate::save_v6::SaveV6::from_v5)
        } else {
            crate::save_v5::read_v4(&self.save_root)
                .and_then(|legacy| crate::save_v5::from_v4(&legacy))
                .and_then(crate::save_v6::SaveV6::from_v5)
        };
        save.and_then(|save| {
            let state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
            self.prepare_default_save(&save, &state).map(|_| ())
        }).is_ok()
    }

    pub fn list_save_slots(&self) -> Vec<crate::save_slots::SlotSummary> {
        crate::save_slots::list_slots(&self.save_root).into_iter().map(|mut slot| {
            if slot.current_hp == Some(0) {
                slot.valid = false;
                slot.error_code = Some("E_SAVE_PLAYER_DEAD".into());
            } else if slot.slot_id == "legacy-save-v3" && slot.valid && !self.has_default_save() {
                // This virtual slot continues the winning default source, not
                // the legacy file directly. Do not bypass a blocked newer file.
                slot.valid = false;
                slot.error_code = Some("E_SAVE_DEFAULT_UNAVAILABLE".into());
            }
            slot
        }).collect()
    }

    /// Server-owned test/content grant. The client supplies only a catalog ID, never effects.
    pub fn grant_trusted_build_item(&self, item_id: &str) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let mut candidate = state.progression_v6.clone();
        candidate.grant_trusted_item(item_id)?;
        state.install_build_command(candidate)
    }

    pub fn equip_build_item(&self, item_id: &str) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let mut candidate = state.progression_v6.clone();
        candidate.equip(item_id)?;
        state.install_build_command(candidate)
    }

    pub fn unequip_build_slot(&self, slot: &str) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let mut candidate = state.progression_v6.clone();
        candidate.unequip(slot)?;
        state.install_build_command(candidate)
    }

    /// Internal trusted progression entry point. No Tauri/public acquisition command exposes it.
    pub fn grant_trusted_build_bloodline(&self, id: &str) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let mut candidate = state.progression_v6.clone();
        candidate.grant_trusted_bloodline(id)?;
        state.install_build_command(candidate)
    }

    /// Explicit trusted replacement resets the previous lineage's tier and proficiency.
    pub fn replace_trusted_build_bloodline(
        &self,
        expected_id: &str,
        id: &str,
    ) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let mut candidate = state.progression_v6.clone();
        candidate.replace_trusted_bloodline(expected_id, id)?;
        state.install_build_command(candidate)
    }

    pub fn remove_trusted_build_bloodline(&self, expected_id: &str) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let mut candidate = state.progression_v6.clone();
        candidate.remove_trusted_bloodline(expected_id)?;
        state.install_build_command(candidate)
    }

    pub fn build_snapshot(
        &self,
    ) -> Result<
        (
            build_v6::PlayerProgressionV6,
            Vec<crate::effects::EffectSource>,
            crate::player_rules::EffectivePlayerRules,
        ),
        String,
    > {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let mut progression = state.progression_v6.clone();
        progression.sync_capabilities(&state.capabilities);
        let (sources, rules) = progression
            .resolve_rules_with_capabilities(&state.capabilities, &state.world.rear_view)?;
        Ok((progression, sources, rules))
    }

    pub fn save_slot(
        &self,
        slot_id: &str,
        display_name: &str,
        create: bool,
    ) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let save = crate::save_v6::SaveV6::capture(&state, scene.as_ref())?;
        if create {
            crate::save_slots::create_slot_v6(&self.save_root, slot_id, display_name, &save)?;
        } else {
            crate::save_slots::overwrite_slot_v6(&self.save_root, slot_id, display_name, &save)?;
        }
        *self
            .active_slot
            .lock()
            .map_err(|_| "E_SLOT_LOCK_POISONED".to_string())? = Some(slot_id.into());
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    fn reject_dead_save(&self, slot_id: Option<&str>) -> Result<(), String> {
        let paths = match slot_id {
            None | Some("legacy-save-v3") => vec![
                crate::save_v6::save_path(&self.save_root),
                crate::save_v5::save_path(&self.save_root),
                self.save_root.join(crate::save_v3::SAVE_FILE_NAME),
            ],
            Some(id) => {
                crate::save_slots::validate_slot_id(id)?;
                let root = self.save_root.join("slots").join(id);
                vec![root.join("slot-v6.json"), root.join("slot-v5.json"), root.join("slot-v4.json")]
            }
        };
        for path in paths {
            if !crate::save_v6::path_present(&path) { continue; }
            let bytes = crate::save_v6::read_bounded(&path, "E_SAVE", "E_NO_SAVE")?;
            if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                // Slot -> SaveV6 -> SaveV5, or legacy plain SaveV3/V5.
                for _ in 0..3 {
                    if value.get("player").and_then(|player| player.get("currentHp"))
                        .and_then(serde_json::Value::as_u64) == Some(0) {
                        return Err("E_SAVE_PLAYER_DEAD".into());
                    }
                    let Some(nested) = value.get("save").cloned() else { break; };
                    value = nested;
                }
            }
            // Never select an older profile behind the actual winning source.
            break;
        }
        Ok(())
    }

    fn reject_staged_clockworks_save(&self, slot_id: Option<&str>) -> Result<(), String> {
        let restricted_production = self
            .scene_registry
            .lock()
            .map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?
            .as_ref()
            .is_some_and(|registry| registry.is_staged_clockworks_production()
                || registry.is_complete_clockworks_production());
        if !restricted_production {
            return Ok(());
        }

        let candidates = match slot_id {
            None | Some("legacy-save-v3") => vec![
                (crate::save_v6::save_path(&self.save_root), true),
                (crate::save_v5::save_path(&self.save_root), false),
                (self.save_root.join(crate::save_v3::SAVE_FILE_NAME), false),
            ],
            Some(slot_id) => {
                crate::save_slots::validate_slot_id(slot_id)?;
                let slot_root = self.save_root.join("slots").join(slot_id);
                vec![
                    (slot_root.join("slot-v6.json"), true),
                    (slot_root.join("slot-v5.json"), true),
                    (slot_root.join("slot-v4.json"), true),
                ]
            }
        };

        for (path, _) in candidates {
            if !path.is_file() {
                continue;
            }
            if std::fs::metadata(&path)
                .map(|metadata| metadata.len() > MAX_PREFLIGHT_SAVE_BYTES)
                .unwrap_or(true)
            {
                return Ok(());
            }
            let Ok(file) = std::fs::File::open(&path) else {
                return Ok(());
            };
            let mut bytes = Vec::new();
            if file
                .take(MAX_PREFLIGHT_SAVE_BYTES + 1)
                .read_to_end(&mut bytes)
                .is_err()
                || bytes.len() as u64 > MAX_PREFLIGHT_SAVE_BYTES
            {
                return Ok(());
            }
            let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                return Ok(());
            };
            // File and slot wrappers can nest SaveV6 and SaveV5. Inspect the
            // winning source before migration; never rewrite an unknown scene.
            let mut save = Some(&value);
            for _ in 0..3 {
                if save.is_some_and(|value| value.get("worldId").is_some()) { break; }
                save = save.and_then(|value| value.get("save"));
            }
            if let Some(world_id) = save
                .and_then(|value| value.get("worldId"))
                .and_then(serde_json::Value::as_str)
            {
                let scene_id = save
                    .and_then(|value| value.get("sceneId"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                self.reject_staged_clockworks_world(world_id, scene_id)?;
            }
            break;
        }
        Ok(())
    }

    fn reject_staged_clockworks_world(&self, world_id: &str, scene_id: &str) -> Result<(), String> {
        let registry = self.scene_registry.lock().map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?;
        if world_id == "clockworks"
            && registry.as_ref().is_some_and(|registry| registry.is_complete_clockworks_production()
                && registry.scene(scene_id).is_none_or(|scene| scene.world_id != "clockworks"))
        {
            return Err("E_SAVE_SCENE_UNKNOWN".into());
        }
        drop(registry);
        if world_id == "clockworks"
            && scene_id != CLOCKWORKS_REGULATOR_SCENE_ID
            && self
                .scene_registry
                .lock()
                .map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?
                .as_ref()
                .is_some_and(WorldRegistry::is_staged_clockworks_production)
        {
            return Err("E_SAVE_STAGED_WORLD_UNAVAILABLE".into());
        }
        Ok(())
    }

    pub fn continue_slot(&self, slot_id: &str) -> Result<WorldView, String> {
        self.reject_dead_save(Some(slot_id))?;
        self.reject_staged_clockworks_save(Some(slot_id))?;
        let save = if slot_id == "legacy-save-v3" {
            crate::save_v6::read_or_migrate(&self.save_root)?
        } else {
            crate::save_slots::read_slot_v6(&self.save_root, slot_id)?.1
        };
        self.reject_staged_clockworks_world(&save.save.world_id, &save.save.scene_id)?;
        if save.save.player.current_hp == 0 { return Err("E_SAVE_PLAYER_DEAD".into()); }
        let mut restored = save.restore_state()?;
        restored.world.last_input_seq = 0;
        restored.world.last_client_time_ms = 0;
        restored.world.combat_state = Default::default();
        restored.last_received_seq = 0;
        restored.last_client_time_ms = 0;
        restored.latest_sample =
            InputSample::new(restored.world.revision.world_epoch, 0, 0, 0.0, 0.0)
                .map_err(|error| format!("E_SAVE_INPUT: {error:?}"))?;
        restored.pending_combat.clear();
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        Self::rebase_restored_entry(&mut restored, &state)?;
        let mut scene_runtime = None;
        if save.save.scene_id != save.save.world_id {
            let registry = self
                .scene_registry
                .lock()
                .map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?;
            let registry = registry.as_ref().ok_or("E_SAVE_SCENE_REGISTRY_REQUIRED")?;
            scene_runtime = Some(restore_scene_state(
                registry.clone(),
                &save.save,
                restored.world.revision.world_epoch,
            )?);
            clockworks_controls::restore_scene(
                scene_runtime.as_mut().expect("runtime just restored"),
                &restored.world_persistent_v1.clockworks,
            )?;
            let scene = scene_runtime
                .as_ref()
                .expect("runtime just restored")
                .current_scene();
            if scene.world_id != save.save.world_id {
                return Err("E_SAVE_SCENE_WORLD_MISMATCH".into());
            }
            let spawned_legacy_actors = restored.world.generic_actors.is_empty();
            restore_generic_actors(&mut restored.world, scene,
                &mut restored.world_persistent_v1,
                scene_runtime.as_ref().unwrap().is_complete_clockworks_production())?;
            if spawned_legacy_actors {
                reset_clockworks_regulator_encounter_for_new_boss(
                    &mut restored.world_persistent_v1.clockworks,
                    scene,
                    &restored.world,
                )?;
            }
            clockworks_core::restore_regulator_progression(
                &mut restored.world,
                scene,
                &restored.world_persistent_v1.clockworks,
            )?;
            restore_clockworks_elite_progression(
                &mut restored.world,
                scene,
                restored
                    .world_persistent_v1
                    .clockworks
                    .forged_guard_elite_first_kill,
            )?;
            restore_sentinels(&mut restored.world, scene)?;
            enable_native_sentinel_charge(&mut restored.world, scene, scene_runtime.as_ref().unwrap().is_complete_clockworks_production())?;
            restored.kcc = scene_kcc_for_state(
                scene,
                &completed_world_events(&restored, &save.save.world_id),
                &restored,
            )?;
            if !restored.world.actors_have_stable_support(&restored.kcc) {
                return Err("E_SAVE_ACTOR_SUPPORT_INVALID".into());
            }
            let upgraded_floor = gear_shaft_support::restore_candidate(&mut restored, &save.save, scene,
                scene_runtime.as_ref().unwrap().is_complete_clockworks_production())?;
            if !upgraded_floor && !restored.kcc.validate_saved_support_frame(save.save.moving_support_frame.as_ref(),
                save.save.revision.world_epoch, save.save.server_time_ms) {
                return Err("E_SAVE_SUPPORT_FRAME_INVALID".into());
            }
            if !restored.kcc.valid_saved_support_contact(&restored.world.player, restored.world.server_time_ms) {
                return Err("E_SAVE_SUPPORT_CONTACT_INVALID".into());
            }
            if !restored.kcc.can_occupy(
                restored.world.player.position_m,
                restored.world.player.radius_m,
            ) {
                return Err("E_SAVE_PLAYER_POSITION_INVALID".into());
            }
            restored.world.combat_state.traversal_markers =
                scene_runtime.as_ref().unwrap().traversals();
        }
        if scene_runtime.is_none() && save.save.moving_support_frame.is_some() {
            return Err("E_SAVE_SUPPORT_FRAME_INVALID".into());
        }
        if let Some(scene) = scene_runtime.as_ref() {
            update_mist_harbor_exploration(&mut restored, scene.current_scene(), false)?;
            environmental_hazards::prepare_scene(&mut restored, scene.current_scene())?;
            warden_encounter::prepare_scene(&mut restored, scene.current_scene())?;
            ordinary_enemy_presentation::prepare_restored(&mut restored, scene);
        }
        // Saved epochs may repeat on repeated Continue. Keep the ephemeral Build command
        // revision monotonic within this owner without changing any saved authority records.
        restored.build_commands.revision = state
            .build_commands
            .revision
            .checked_add(1)
            .filter(|value| *value <= build_ui::MAX_SAFE_REVISION)
            .ok_or("E_BUILD_REVISION_EXHAUSTED")?;
        // A held Continue clears horizontal intent, while gravity remains exact
        // for an airborne save in a source-validated moving-support scene.
        let support_vertical_velocity = restored.kcc.has_support_surfaces()
            .then_some(restored.world.player.velocity_mps.y_m);
        Self::prepare_entry(&mut restored, state.entry_generation)?;
        if let Some(vertical) = support_vertical_velocity {
            restored.world.player.velocity_mps.y_m = vertical;
            // Combat's ephemeral dash direction is deliberately not restored.
            restored.world.player.dash_remaining_ms = 0;
        }
        let mut scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let mut slot = self.active_slot.lock().map_err(|_| "E_SLOT_LOCK_POISONED")?;
        *state = restored;
        *scene = scene_runtime;
        *slot = if slot_id == "legacy-save-v3" { None } else { Some(slot_id.into()) };
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    pub fn return_to_hub_slot(
        &self,
        slot_id: &str,
        display_name: &str,
        create: bool,
    ) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() {
            state.pending_entry = None;
            return self.project_locked_scene_view(&state);
        }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let backup = state.clone();
        if !state.paused {
            state
                .world
                .bump_authority_revision()
                .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
            state.paused = true;
        }
        let result = (|| {
            let scene = self
                .scene_runtime
                .lock()
                .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
            let save = crate::save_v6::SaveV6::capture(&state, scene.as_ref())?;
            if create {
                crate::save_slots::create_slot_v6(&self.save_root, slot_id, display_name, &save)
            } else {
                crate::save_slots::overwrite_slot_v6(&self.save_root, slot_id, display_name, &save)
            }
        })();
        if let Err(error) = result {
            *state = backup;
            return Err(error);
        }
        *self
            .active_slot
            .lock()
            .map_err(|_| "E_SLOT_LOCK_POISONED".to_string())? = Some(slot_id.into());
        state.pending_entry = None;
        self.project_locked_scene_view(&state)
    }

    pub fn save(&self) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        let scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let save = crate::save_v6::SaveV6::capture(&state, scene.as_ref())?;
        crate::save_v6::write_save(&self.save_root, &save)?;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    pub fn continue_saved(&self) -> Result<WorldView, String> {
        self.reject_dead_save(None)?;
        self.reject_staged_clockworks_save(None)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let save = crate::save_v6::read_or_migrate(&self.save_root)?;
        let (restored, scene_runtime) = self.prepare_default_save(&save, &state)?;
        let mut scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        *state = restored;
        *scene = scene_runtime;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    /// Build and validate a candidate without installing it or writing save files.
    /// Availability and explicit Continue must use exactly the same restore rules.
    fn prepare_default_save(&self, save: &crate::save_v6::SaveV6, state: &RuntimeState)
        -> Result<(RuntimeState, Option<SceneRuntime>), String>
    {
        self.reject_staged_clockworks_world(&save.save.world_id, &save.save.scene_id)?;
        if save.save.player.current_hp == 0 { return Err("E_SAVE_PLAYER_DEAD".into()); }
        let mut restored = save.restore_state()?;
        // Save v3 retains the last session for validation, but the new epoch
        // starts a fresh browser input and combat request-id domain.
        restored.world.last_input_seq = 0;
        restored.world.last_client_time_ms = 0;
        restored.world.combat_state = Default::default();
        restored.last_received_seq = 0;
        restored.last_client_time_ms = 0;
        restored.latest_sample =
            InputSample::new(restored.world.revision.world_epoch, 0, 0, 0.0, 0.0)
                .map_err(|error| format!("E_SAVE_INPUT: {error:?}"))?;
        restored.pending_combat.clear();
        Self::rebase_restored_entry(&mut restored, state)?;
        let mut scene_runtime = None;
        if save.save.scene_id != save.save.world_id {
            let registry = self
                .scene_registry
                .lock()
                .map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?;
            let registry = registry.as_ref().ok_or("E_SAVE_SCENE_REGISTRY_REQUIRED")?;
            scene_runtime = Some(restore_scene_state(
                registry.clone(),
                &save.save,
                restored.world.revision.world_epoch,
            )?);
            clockworks_controls::restore_scene(
                scene_runtime.as_mut().expect("runtime just restored"),
                &restored.world_persistent_v1.clockworks,
            )?;
            let scene = scene_runtime
                .as_ref()
                .expect("runtime just restored")
                .current_scene();
            if scene.world_id != save.save.world_id {
                return Err("E_SAVE_SCENE_WORLD_MISMATCH".into());
            }
            let spawned_legacy_actors = restored.world.generic_actors.is_empty();
            restore_generic_actors(&mut restored.world, scene,
                &mut restored.world_persistent_v1,
                scene_runtime.as_ref().unwrap().is_complete_clockworks_production())?;
            if spawned_legacy_actors {
                reset_clockworks_regulator_encounter_for_new_boss(
                    &mut restored.world_persistent_v1.clockworks,
                    scene,
                    &restored.world,
                )?;
            }
            clockworks_core::restore_regulator_progression(
                &mut restored.world,
                scene,
                &restored.world_persistent_v1.clockworks,
            )?;
            restore_clockworks_elite_progression(
                &mut restored.world,
                scene,
                restored
                    .world_persistent_v1
                    .clockworks
                    .forged_guard_elite_first_kill,
            )?;
            restore_sentinels(&mut restored.world, scene)?;
            enable_native_sentinel_charge(&mut restored.world, scene, scene_runtime.as_ref().unwrap().is_complete_clockworks_production())?;
            restored.kcc = scene_kcc_for_state(
                scene,
                &completed_world_events(&restored, &save.save.world_id),
                &restored,
            )?;
            if !restored.world.actors_have_stable_support(&restored.kcc) {
                return Err("E_SAVE_ACTOR_SUPPORT_INVALID".into());
            }
            let upgraded_floor = gear_shaft_support::restore_candidate(&mut restored, &save.save, scene,
                scene_runtime.as_ref().unwrap().is_complete_clockworks_production())?;
            if !upgraded_floor && !restored.kcc.validate_saved_support_frame(save.save.moving_support_frame.as_ref(),
                save.save.revision.world_epoch, save.save.server_time_ms) {
                return Err("E_SAVE_SUPPORT_FRAME_INVALID".into());
            }
            if !restored.kcc.valid_saved_support_contact(&restored.world.player, restored.world.server_time_ms) {
                return Err("E_SAVE_SUPPORT_CONTACT_INVALID".into());
            }
            if !restored.kcc.can_occupy(
                restored.world.player.position_m,
                restored.world.player.radius_m,
            ) {
                return Err("E_SAVE_PLAYER_POSITION_INVALID".into());
            }
            restored.world.combat_state.traversal_markers =
                scene_runtime.as_ref().unwrap().traversals();
        }
        if scene_runtime.is_none() && save.save.moving_support_frame.is_some() {
            return Err("E_SAVE_SUPPORT_FRAME_INVALID".into());
        }
        if let Some(scene) = scene_runtime.as_ref() {
            update_mist_harbor_exploration(&mut restored, scene.current_scene(), false)?;
            environmental_hazards::prepare_scene(&mut restored, scene.current_scene())?;
            warden_encounter::prepare_scene(&mut restored, scene.current_scene())?;
            ordinary_enemy_presentation::prepare_restored(&mut restored, scene);
        }
        // Saved epochs may repeat on repeated Continue. Keep the ephemeral Build command
        // revision monotonic within this owner without changing any saved authority records.
        restored.build_commands.revision = state
            .build_commands
            .revision
            .checked_add(1)
            .filter(|value| *value <= build_ui::MAX_SAFE_REVISION)
            .ok_or("E_BUILD_REVISION_EXHAUSTED")?;
        // A held Continue clears horizontal intent, while gravity remains exact
        // for an airborne save in a source-validated moving-support scene.
        let support_vertical_velocity = restored.kcc.has_support_surfaces()
            .then_some(restored.world.player.velocity_mps.y_m);
        Self::prepare_entry(&mut restored, state.entry_generation)?;
        if let Some(vertical) = support_vertical_velocity {
            restored.world.player.velocity_mps.y_m = vertical;
            // Combat's ephemeral dash direction is deliberately not restored.
            restored.world.player.dash_remaining_ms = 0;
        }
        Ok((restored, scene_runtime))
    }

    fn initial_state(world_epoch: u64) -> Result<RuntimeState, String> {
        let world = WorldStateV3::new("grey_hive", world_epoch, crate::world_v3::Vec3::zero())
            .map_err(|error| format!("E_RUNTIME_INIT: {error:?}"))?;
        let mut route = RouteState::new();
        match apply_route_command(
            &mut route,
            RouteCommand::Enter {
                world_id: "grey_hive".into(),
                request_id: "formal-runtime-bootstrap-grey-hive".into(),
            },
            world.revision,
        ) {
            RouteResult::Applied { .. } => {}
            RouteResult::Rejected { code, message } => {
                return Err(format!("E_RUNTIME_INIT: {code:?}: {message}"))
            }
        }
        let config = FixedStepConfig::new(FIXED_HZ, MAX_STEPS_PER_FRAME)
            .map_err(|error| format!("E_RUNTIME_INIT: {error:?}"))?;
        let stepper = FixedStepClock::new(config);
        let capability_data: CapabilityData = serde_json::from_str(CAPABILITY_DATA)
            .map_err(|error| format!("E_RUNTIME_INIT: invalid capability config: {error}"))?;
        let expected_ids = [
            CAP_LOCAL_MAP,
            CAP_REAR_VIEW,
            CAP_ENEMY_VITALS,
            CAP_REGENERATION,
            CAP_ACOUSTIC_MAPPING,
            CAP_AIR_STEP,
        ];
        if capability_data.schema_version != CAPABILITY_SCHEMA_VERSION
            || capability_data.canonical_capability_ids
                != expected_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
        {
            return Err("E_RUNTIME_INIT: capability schema/IDs mismatch".into());
        }
        let capabilities = CapabilityState::new("grey_hive", capability_data.regeneration)
            .map_err(|error| format!("E_RUNTIME_INIT: capability config rejected: {error:?}"))?;
        Ok(RuntimeState {
            world,
            world_persistent_v1: crate::world_persistent_v1::WorldPersistentState::default(),
            route,
            capabilities,
            progression_v6: build_v6::PlayerProgressionV6::default(),
            build_commands: build_ui::BuildCommandState::default(),
            effect_sources_v6: Vec::new(),
            effective_rules_v6: crate::player_rules::EffectivePlayerRules::default(),
            kcc: StaticKccWorld::grey_hive_first_flow(false),
            stepper,
            latest_sample: InputSample::new(world_epoch, 0, 0, 0.0, 0.0)
                .map_err(|error| format!("E_RUNTIME_INIT: {error:?}"))?,
            latest_input_received_at: Instant::now(),
            pending_combat: Vec::new(),
            presentation_events: Vec::new(),
            next_presentation_event_id: 1,
            sound_cues: Vec::new(),
            next_sound_cue_id: 1,
            last_received_seq: 0,
            last_client_time_ms: 0,
            last_damaged_at_ms: None,
            last_combat_at_ms: None,
            paused: false,
            entry_generation: 0,
            pending_entry: None,
            entry_pause_requested: false,
            last_lifecycle_sequence: 0,
            last_owner_error: None,
        })
    }

    /// Start a fresh in-memory Grey Hive blockout. This is a New/reset action,
    /// not a persisted Save v3 restore.
    pub fn reset_new(&self) -> Result<WorldView, String> {
        let production_registry = self
            .scene_registry
            .lock()
            .map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?
            .clone();
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let next_epoch = state
            .world
            .revision
            .world_epoch
            .checked_add(1)
            .filter(|epoch| *epoch <= build_ui::MAX_SAFE_REVISION)
            .ok_or_else(|| "E_WORLD_EPOCH_EXHAUSTED".to_string())?;
        let mut fresh = Self::initial_state(next_epoch)?;
        if production_registry.as_ref().is_some_and(WorldRegistry::is_complete_clockworks_production) {
            fresh.world_persistent_v1.grey_hive.beacon = Some(crate::world_persistent_v1::GreyHiveBeaconState::new_uncollected());
        }
        fresh.build_commands.revision = state
            .build_commands
            .revision
            .checked_add(1)
            .filter(|value| *value <= build_ui::MAX_SAFE_REVISION)
            .ok_or("E_BUILD_REVISION_EXHAUSTED")?;
        let prepared_scene = match production_registry {
            Some(registry) if registry.scene("rs_core_room").is_some() =>
                Some(Self::prepare_scene_registry(&mut fresh, registry, "rs_core_room")?),
            Some(registry) if registry.scene("gh_entry_maintenance").is_some() =>
                Some(Self::prepare_scene_registry(&mut fresh, registry, "gh_entry_maintenance")?),
            _ => None,
        };
        Self::prepare_entry(&mut fresh, state.entry_generation)?;
        let mut scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let mut slot = self.active_slot.lock().map_err(|_| "E_SLOT_LOCK_POISONED")?;
        *state = fresh;
        *scene = prepared_scene;
        *slot = None;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    pub fn snapshot(&self) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if let Some(error) = &state.last_owner_error {
            return Err(format!("E_SIMULATION_OWNER: {error}"));
        }
        let scene_runtime = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        Ok(project_scene_view(&state, scene_runtime.as_ref()))
    }

    fn project_locked_scene_view(&self, state: &RuntimeState) -> Result<WorldView, String> {
        let scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        Ok(project_scene_view(state, scene.as_ref()))
    }

    /// Install a complete compiler-produced campaign into the authoritative runtime.
    /// Asset/entity catalogs are supplied by trusted Rust startup code, never by browser input.
    pub fn load_scene_registry<I, S>(
        &self,
        scene_json: I,
        start_scene_id: &str,
        admitted_assets: &std::collections::BTreeSet<String>,
        known_entity_types: &std::collections::BTreeSet<String>,
    ) -> Result<WorldView, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let registry = WorldRegistry::load(scene_json, admitted_assets, known_entity_types)
            .map_err(|error| error.to_string())?;
        self.install_scene_registry(registry, start_scene_id)
    }

    /// Install the intentionally partial, production Grey Hive scene slice.
    pub fn load_grey_hive_slice_registry<I, S>(
        &self,
        scene_json: I,
        start_scene_id: &str,
        admitted_assets: &std::collections::BTreeSet<String>,
        known_entity_types: &std::collections::BTreeSet<String>,
    ) -> Result<WorldView, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let registry =
            WorldRegistry::load_grey_hive_slice(scene_json, admitted_assets, known_entity_types)
                .map_err(|error| error.to_string())?;
        self.install_scene_registry(registry, start_scene_id)
    }

    /// Install the strict native production bundle. Fresh journeys start at
    /// Return Station; cross-world gates are authored and progression-gated.
    pub(crate) fn load_production_scene_registry<I, S>(
        &self,
        scene_json: I,
        admitted_assets: &std::collections::BTreeSet<String>,
        known_entity_types: &std::collections::BTreeSet<String>,
    ) -> Result<WorldView, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let registry = WorldRegistry::load_complete_clockworks_production(
            scene_json,
            admitted_assets,
            known_entity_types,
        )
        .map_err(|error| error.to_string())?;
        self.install_scene_registry(registry, "rs_core_room")
    }

    /// Enter the one supported cross-world route. The target is derived only
    /// from the active native scene and fixed gate id; callers cannot select a
    /// world or scene. The command is bound to the active world epoch and gate.
    pub fn use_world_gate(
        &self,
        gate_id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if !valid_id(request_id) {
            return Err("E_WORLD_GATE_REQUEST_INVALID".into());
        }
        if state.world.revision.world_epoch != world_epoch {
            return Err("E_WORLD_GATE_STALE_EPOCH".into());
        }
        let mut active = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let active_scene = active.as_ref().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        let source = active_scene.current_scene();
        if source.world_id != state.world.world_id || source.scene_id != state.world.scene_id {
            return Err("E_WORLD_GATE_SCENE_MISMATCH".into());
        }

        let (target_scene_id, marker_id, marker_kind) = match (source.world_id.as_str(), gate_id) {
            ("return_station", "rs_world_gate_to_gh") if source.scene_id == "rs_core_room" => (
                "gh_entry_maintenance",
                "rs_world_gate_marker",
                "world_gate_marker",
            ),
            ("return_station", "rs_world_gate_to_mh") if source.scene_id == "rs_core_room" => {
                let gh = state
                    .route
                    .progress
                    .iter()
                    .find(|progress| progress.world_id == "grey_hive")
                    .ok_or("E_WORLD_GATE_GH_PROGRESS_MISSING")?;
                if !gh.completed
                    || !gh.first_completion
                    || !gh
                        .completed_events
                        .iter()
                        .any(|event| event == "hive_extraction")
                {
                    return Err("E_WORLD_GATE_GH_EXTRACTION_REQUIRED".into());
                }
                (
                    "mh_fog_pier",
                    "rs_mh_world_gate_marker",
                    "world_gate_marker",
                )
            }
            ("return_station", "rs_world_gate_to_cw")
                if source.scene_id == "rs_core_room" && active_scene.is_complete_clockworks_production() =>
            {
                if !clockworks_campaign::mist_harbor_extracted(&state) {
                    return Err("E_WORLD_GATE_MH_EXTRACTION_REQUIRED".into());
                }
                if !clockworks_campaign::entry_eligible(&state) {
                    return Err("E_WORLD_GATE_UNAVAILABLE".into());
                }
                ("cw_entry_foundry", "rs_cw_world_gate_marker", "world_gate_marker")
            }
            ("clockworks", "cw_shutdown_return_to_rs")
                if source.scene_id == "cw_shutdown_exit" && active_scene.is_complete_clockworks_production()
                    && state.route.current_world_id == "clockworks" =>
            {
                if !clockworks_campaign::completed(&state) {
                    return Err("E_CLOCKWORKS_SETTLEMENT_REQUIRED".into());
                }
                ("rs_core_room", "cw_shutdown_exit_portal_staged", "exit_portal_staged_marker")
            }
            ("grey_hive", "gh_extraction_return_to_rs") if source.scene_id == "gh_exit" => {
                let gh = state
                    .route
                    .progress
                    .iter()
                    .find(|progress| progress.world_id == "grey_hive")
                    .ok_or("E_WORLD_GATE_GH_PROGRESS_MISSING")?;
                if !gh.completed
                    || !gh.first_completion
                    || !gh
                        .completed_events
                        .iter()
                        .any(|event| event == "hive_extraction")
                {
                    return Err("E_WORLD_GATE_GH_EXTRACTION_REQUIRED".into());
                }
                (
                    "rs_core_room",
                    "gh_exit_extraction_console",
                    "extraction_console",
                )
            }
            ("mist_harbor", "mh_extraction_return_to_rs")
                if source.scene_id == "mh_extraction"
                    && state.route.current_world_id == "mist_harbor" =>
            {
                let mh = state
                    .route
                    .progress
                    .iter()
                    .find(|progress| progress.world_id == "mist_harbor")
                    .ok_or("E_WORLD_GATE_MH_PROGRESS_MISSING")?;
                let required = ["mist_beacon_west", "mist_beacon_east", "mist_signal"];
                if !required
                    .iter()
                    .all(|required| mh.completed_events.iter().any(|event| event == required))
                {
                    return Err("E_MH_EXTRACTION_REQUIRED".into());
                }
                if !state.world_persistent_v1.mist_harbor.warden_defeated { return Err("E_MH_WARDEN_DEFEAT_REQUIRED".into()); }
                ("rs_core_room", "mh_extraction_exit", "world_exit")
            }
            _ => return Err("E_WORLD_GATE_UNAVAILABLE".into()),
        };
        let marker = source
            .interactions
            .iter()
            .find(|interaction| interaction.id == marker_id && interaction.kind == marker_kind)
            .ok_or("E_WORLD_GATE_MARKER_MISSING")?;
        let distance = {
            let position = state.world.player.position_m;
            let dx = position.x_m - marker.position[0];
            let dy = position.y_m - marker.position[1];
            let dz = position.z_m - marker.position[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        };
        if !distance.is_finite() || distance > 2.5
            || marker.height_range_m.is_some_and(|range|!range.contains(state.world.player.position_m.y_m)) {
            return Err("E_WORLD_GATE_OUT_OF_RANGE".into());
        }
        let registry = self
            .scene_registry
            .lock()
            .map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?
            .clone()
            .ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        let next_epoch = world_epoch
            .checked_add(1)
            .filter(|epoch| *epoch <= build_ui::MAX_SAFE_REVISION)
            .ok_or("E_WORLD_GATE_EPOCH_EXHAUSTED")?;
        let mut candidate = SceneRuntime::new(registry, target_scene_id, next_epoch)
            .map_err(|error| error.to_string())?;
        clockworks_controls::restore_scene(&mut candidate, &state.world_persistent_v1.clockworks)?;
        let target = candidate.current_scene().clone();
        let spawn = target
            .spawns
            .iter()
            .find(|spawn| spawn.kind == "player")
            .ok_or("E_SCENE_PLAYER_SPAWN")?;

        let mut next_route = state.route.clone();
        if target.world_id == "grey_hive" {
            let gh = next_route
                .progress
                .iter()
                .find(|progress| progress.world_id == "grey_hive")
                .ok_or("E_WORLD_GATE_GH_PROGRESS_MISSING")?;
            let revisiting = gh.completed;
            let command = if revisiting {
                RouteCommand::Revisit {
                    world_id: "grey_hive".into(),
                    request_id: format!("world-gate-revisit-{request_id}"),
                }
            } else {
                RouteCommand::Enter {
                    world_id: "grey_hive".into(),
                    request_id: format!("world-gate-enter-{request_id}"),
                }
            };
            match apply_route_command(&mut next_route, command, state.world.revision) {
                RouteResult::Applied { events, .. } => {
                    let expected = if revisiting {
                        matches!(events.as_slice(), [crate::world_progression::RouteEvent::Revisited { world_id, .. }]
                            if world_id == "grey_hive")
                    } else {
                        matches!(events.as_slice(), [crate::world_progression::RouteEvent::Entered { world_id, .. }]
                            if world_id == "grey_hive")
                    };
                    // DuplicateIgnored is an acknowledgement, not a new visit.
                    // Never install a destination at a fresh epoch without the
                    // exact route transaction this gate requested.
                    if !expected { return Err("E_WORLD_GATE_ROUTE_NOT_APPLIED".into()); }
                }
                RouteResult::Rejected { code, message } => {
                    return Err(format!("E_WORLD_GATE_ROUTE: {code:?}: {message}"));
                }
            }
        } else if target.world_id == "mist_harbor" {
            let mh = next_route
                .progress
                .iter()
                .find(|progress| progress.world_id == "mist_harbor")
                .ok_or("E_WORLD_GATE_MH_PROGRESS_MISSING")?;
            let revisiting = mh.completed;
            let command = if revisiting {
                if !crate::world_progression::can_revisit_world(&next_route, "mist_harbor") {
                    return Err("E_WORLD_GATE_UNAVAILABLE".into());
                }
                RouteCommand::Revisit {
                    world_id: "mist_harbor".into(),
                    request_id: format!("world-gate-revisit-{request_id}"),
                }
            } else {
                RouteCommand::Enter {
                    world_id: "mist_harbor".into(),
                    request_id: format!("world-gate-enter-{request_id}"),
                }
            };
            match apply_route_command(&mut next_route, command, state.world.revision) {
                RouteResult::Applied { events, .. } => {
                    let expected = if revisiting {
                        matches!(events.as_slice(), [crate::world_progression::RouteEvent::Revisited { world_id, .. }]
                            if world_id == "mist_harbor")
                    } else {
                        matches!(events.as_slice(), [crate::world_progression::RouteEvent::Entered { world_id, .. }]
                            if world_id == "mist_harbor")
                    };
                    // DuplicateIgnored is an acknowledgement, not a new visit.
                    // Never install a destination at a fresh epoch without the
                    // exact route transaction this gate requested.
                    if !expected { return Err("E_WORLD_GATE_ROUTE_NOT_APPLIED".into()); }
                }
                RouteResult::Rejected { code, message } => {
                    return Err(format!("E_WORLD_GATE_ROUTE: {code:?}: {message}"));
                }
            }
        } else if target.world_id == "clockworks" {
            next_route = clockworks_campaign::prepare_entry_route(&state, request_id)?;
        } else if source.world_id == "mist_harbor" {
            let mh = next_route
                .progress
                .iter()
                .find(|progress| progress.world_id == "mist_harbor")
                .ok_or("E_WORLD_GATE_MH_PROGRESS_MISSING")?;
            if !mh.completed {
                match apply_route_command(
                    &mut next_route,
                    RouteCommand::Complete {
                        world_id: "mist_harbor".into(),
                        request_id: format!("world-gate-complete-{request_id}"),
                    },
                    state.world.revision,
                ) {
                    RouteResult::Applied { ref events, .. }
                        if candidate.is_complete_clockworks_production()
                            && !matches!(events.as_slice(),
                                [crate::world_progression::RouteEvent::Completed { world_id, first_completion: true }]
                                    if world_id == "mist_harbor") =>
                    {
                        return Err("E_WORLD_GATE_COMPLETE_REJECTED".into());
                    }
                    RouteResult::Applied { .. } => {}
                    RouteResult::Rejected { code, message } => {
                        return Err(format!("E_WORLD_GATE_COMPLETE: {code:?}: {message}"));
                    }
                }
            }
        }
        let mut world = WorldStateV3::new(
            &target.world_id,
            next_epoch,
            vec3_from_array(spawn.position),
        )
        .map_err(|error| format!("E_SCENE_WORLD: {error:?}"))?;
        world.revision = crate::world_v3::WorldRevision {
            world_epoch: next_epoch,
            ..state.world.revision
        };
        world
            .bump_authority_revision()
            .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
        world.server_time_ms = state.world.server_time_ms;
        world.rear_view = state.world.rear_view.clone();
        world.scene_id = target.scene_id.clone();
        world.generic_actors = spawn_generic_actors(&target)?;
        warden_encounter::prepare_world(&mut world, &target, state.world_persistent_v1.mist_harbor.warden_defeated)?;
        world.sentinels = spawn_authored_sentinels(&target)?;
        world.player_hp = state.world.player_hp;
        world.player_max_hp = state.world.player_max_hp;
        world.player_energy = state.world.player_energy;
        world.player_max_energy = state.world.player_max_energy;
        world.combat_state.traversal_markers = candidate.traversals();
        let kcc = scene_kcc_for_state(
            &target,
            &route_completed_events(&next_route, &target.world_id),
            &state,
        )?;
        let mut capabilities = state.capabilities.clone();
        rebase_capability_revision_epoch(&mut capabilities, next_epoch)?;
        capabilities.world_id = target.world_id.clone();
        capabilities.explored_map = world.explored.clone();
        let latest_sample = InputSample::new(next_epoch, 0, 0, 0.0, 0.0)
            .map_err(|error| format!("E_SCENE_INPUT: {error:?}"))?;
        let stepper = FixedStepClock::new(state.stepper.config);
        active
            .as_ref()
            .ok_or("E_SCENE_REGISTRY_NOT_LOADED")?
            .clone()
            .claim_command_request(request_id, world_epoch)
            .map_err(|error| format!("E_WORLD_GATE_{error:?}"))?;

        let mut next_state = state.clone();
        clockworks_roster::mark_installed(&mut next_state.world_persistent_v1.clockworks,
            &target, candidate.is_complete_clockworks_production())?;
        gear_shaft_support::mark_installed(&mut next_state.world_persistent_v1.clockworks,
            &target, candidate.is_complete_clockworks_production())?;
        grey_hive_roster::mark_installed(&mut next_state.world_persistent_v1.grey_hive,
            &target, candidate.is_complete_clockworks_production())?;
        swarm_roster::mark_installed(&mut next_state.world_persistent_v1.grey_hive,
            &target, candidate.is_complete_clockworks_production())?;
        tidebound_roster::mark_installed(&mut next_state.world_persistent_v1.mist_harbor,
            &target, candidate.is_complete_clockworks_production())?;
        signal_wraith_roster::install(&mut next_state, &target, candidate.is_complete_clockworks_production())?;
        enable_native_sentinel_charge(&mut next_state.world, &target, candidate.is_complete_clockworks_production())?;
        reset_clockworks_regulator_encounter_for_new_boss(
            &mut next_state.world_persistent_v1.clockworks,
            &target,
            &world,
        )?;
        clockworks_core::restore_regulator_progression(
            &mut world,
            &target,
            &next_state.world_persistent_v1.clockworks,
        )?;
        next_state.world = world;
        next_state.route = next_route;
        next_state.kcc = kcc;
        next_state.install_capability_state(next_state.world.clone(), capabilities)?;
        if target.world_id == "mist_harbor" {
            update_mist_harbor_exploration(&mut next_state, &target, true)?;
        }
        environmental_hazards::prepare_scene(&mut next_state, &target)?;
        next_state.stepper = stepper;
        next_state.latest_sample = latest_sample;
        next_state.last_received_seq = 0;
        next_state.last_client_time_ms = 0;
        next_state.pending_combat.clear();
        next_state.presentation_events.clear();
        next_state.next_presentation_event_id = 1;
        next_state.sound_cues.clear();
        next_state.next_sound_cue_id = 1;
        Self::prepare_entry(&mut next_state, state.entry_generation)?;
        *state = next_state;
        *active = Some(candidate);
        Ok(project_scene_view(&state, active.as_ref()))
    }

    /// Read the authoritative campaign projection from the one authored
    /// Return Station mission terminal without mutating campaign progress.
    pub fn mission_terminal_status(
        &self,
        terminal_id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if state.world.revision.world_epoch != world_epoch {
            return Err("E_MISSION_TERMINAL_STALE_EPOCH".into());
        }
        if !valid_id(request_id) {
            return Err("E_MISSION_TERMINAL_REQUEST_INVALID".into());
        }
        if terminal_id != "rs_mission_terminal_marker"
            || state.world.world_id != "return_station"
            || state.world.scene_id != "rs_core_room"
        {
            return Err("E_MISSION_TERMINAL_UNAVAILABLE".into());
        }

        let mut scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let active = scene.as_mut().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        let definition = active.current_scene();
        if definition.world_id != "return_station"
            || definition.scene_id != "rs_core_room"
            || active.world_epoch != world_epoch
        {
            return Err("E_MISSION_TERMINAL_SCENE_MISMATCH".into());
        }
        let marker = definition
            .interactions
            .iter()
            .find(|item| item.id == terminal_id && item.kind == "mission_terminal_marker")
            .ok_or("E_MISSION_TERMINAL_MARKER_MISSING")?;
        let position = state.world.player.position_m;
        let dx = position.x_m - marker.position[0];
        let dy = position.y_m - marker.position[1];
        let dz = position.z_m - marker.position[2];
        if (dx * dx + dy * dy + dz * dz).sqrt() > 2.5
            || marker.height_range_m.is_some_and(|range|!range.contains(position.y_m)) {
            return Err("E_MISSION_TERMINAL_OUT_OF_RANGE".into());
        }
        active
            .claim_command_request(request_id, world_epoch)
            .map_err(|error| format!("E_MISSION_TERMINAL_{error:?}"))?;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    /// Read the authoritative capability projection from the authored
    /// Return Station capability terminal without changing capability state.
    pub fn capability_terminal_status(
        &self,
        terminal_id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if state.world.revision.world_epoch != world_epoch {
            return Err("E_CAPABILITY_TERMINAL_STALE_EPOCH".into());
        }
        if !valid_id(request_id) {
            return Err("E_CAPABILITY_TERMINAL_REQUEST_INVALID".into());
        }
        if terminal_id != "rs_capability_terminal_marker"
            || state.world.world_id != "return_station"
            || state.world.scene_id != "rs_core_room"
        {
            return Err("E_CAPABILITY_TERMINAL_UNAVAILABLE".into());
        }

        let mut scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let active = scene.as_mut().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        let definition = active.current_scene();
        if definition.world_id != "return_station"
            || definition.scene_id != "rs_core_room"
            || active.world_epoch != world_epoch
        {
            return Err("E_CAPABILITY_TERMINAL_SCENE_MISMATCH".into());
        }
        let marker = definition
            .interactions
            .iter()
            .find(|item| item.id == terminal_id && item.kind == "capability_terminal_marker")
            .ok_or("E_CAPABILITY_TERMINAL_MARKER_MISSING")?;
        let position = state.world.player.position_m;
        let dx = position.x_m - marker.position[0];
        let dy = position.y_m - marker.position[1];
        let dz = position.z_m - marker.position[2];
        if (dx * dx + dy * dy + dz * dz).sqrt() > 2.5
            || marker.height_range_m.is_some_and(|range|!range.contains(position.y_m)) {
            return Err("E_CAPABILITY_TERMINAL_OUT_OF_RANGE".into());
        }
        active
            .claim_command_request(request_id, world_epoch)
            .map_err(|error| format!("E_CAPABILITY_TERMINAL_{error:?}"))?;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    /// Rest at the one authored Return Station terminal, then persist the
    /// resulting authoritative state through the existing SaveV5 store.
    pub fn save_rest_terminal(
        &self,
        terminal_id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if state.world.revision.world_epoch != world_epoch {
            return Err("E_REST_TERMINAL_STALE_EPOCH".into());
        }
        if !valid_id(request_id) {
            return Err("E_REST_TERMINAL_REQUEST_INVALID".into());
        }
        if terminal_id != "rs_save_rest_terminal_marker"
            || state.world.world_id != "return_station"
            || state.world.scene_id != "rs_core_room"
        {
            return Err("E_REST_TERMINAL_UNAVAILABLE".into());
        }
        if state.world.combat_state.active_action.is_some()
            || state.world.combat_state.active_traversal.is_some()
            || state.world.sentinels.iter().any(|sentinel| sentinel.active)
        {
            return Err("E_REST_TERMINAL_COMBAT_ACTIVE".into());
        }

        let mut scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let active = scene.as_mut().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        let definition = active.current_scene();
        if definition.world_id != "return_station"
            || definition.scene_id != "rs_core_room"
            || active.world_epoch != world_epoch
        {
            return Err("E_REST_TERMINAL_SCENE_MISMATCH".into());
        }
        let marker = definition
            .interactions
            .iter()
            .find(|item| item.id == terminal_id && item.kind == "save_rest_terminal_marker")
            .ok_or("E_REST_TERMINAL_MARKER_MISSING")?;
        let position = state.world.player.position_m;
        let dx = position.x_m - marker.position[0];
        let dy = position.y_m - marker.position[1];
        let dz = position.z_m - marker.position[2];
        if (dx * dx + dy * dy + dz * dz).sqrt() > 2.5
            || marker.height_range_m.is_some_and(|range|!range.contains(position.y_m)) {
            return Err("E_REST_TERMINAL_OUT_OF_RANGE".into());
        }
        active
            .claim_command_request(request_id, world_epoch)
            .map_err(|error| format!("E_REST_TERMINAL_{error:?}"))?;

        let mut rested = state.clone();
        rested.world.player_hp = rested.world.player_max_hp;
        rested.world.player_energy = rested.world.player_max_energy;
        rested
            .world
            .bump_authority_revision()
            .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
        let save = crate::save_v6::SaveV6::capture(&rested, Some(active))?;
        crate::save_v6::write_save(&self.save_root, &save)?;
        *state = rested;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    fn install_scene_registry(
        &self,
        registry: WorldRegistry,
        start_scene_id: &str,
    ) -> Result<WorldView, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        let mut candidate = state.clone();
        let runtime = Self::prepare_scene_registry(&mut candidate, registry.clone(), start_scene_id)?;
        let mut scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let mut installed = self.scene_registry.lock().map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?;
        *state = candidate;
        *scene = Some(runtime);
        *installed = Some(registry);
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    /// All destination construction is isolated until the caller commits both owners.
    fn prepare_scene_registry(
        mut state: &mut RuntimeState,
        registry: WorldRegistry,
        start_scene_id: &str,
    ) -> Result<SceneRuntime, String> {
        let mut runtime =
            SceneRuntime::new(registry, start_scene_id, 1).map_err(|e| e.to_string())?;
        let scene = runtime.current_scene().clone();
        let spawn = scene
            .spawns
            .iter()
            .find(|s| s.kind == "player")
            .ok_or("E_SCENE_PLAYER_SPAWN")?;
        let environment = environmental_hazards::prepared_scene(&state, &scene)?;
        let kcc = scene_kcc_for_state(
            &scene,
            &completed_world_events(&state, &scene.world_id),
            &state,
        )?;
        let mut world = WorldStateV3::new(
            &scene.world_id,
            state.world.revision.world_epoch,
            vec3_from_array(spawn.position),
        )
        .map_err(|e| format!("E_SCENE_WORLD: {e:?}"))?;
        world.revision = state.world.revision;
        world.server_time_ms = state.world.server_time_ms;
        world.rear_view = state.world.rear_view.clone();
        world.scene_id = scene.scene_id.clone();
        world.generic_actors = spawn_generic_actors(&scene)?;
        warden_encounter::prepare_world(&mut world, &scene, state.world_persistent_v1.mist_harbor.warden_defeated)?;
        reset_clockworks_regulator_encounter_for_new_boss(
            &mut state.world_persistent_v1.clockworks,
            &scene,
            &world,
        )?;
        clockworks_core::restore_regulator_progression(
            &mut world,
            &scene,
            &state.world_persistent_v1.clockworks,
        )?;
        world.sentinels = spawn_authored_sentinels(&scene)?;
        world.player_hp = state.world.player_hp;
        world.player_max_hp = state.world.player_max_hp;
        world.player_energy = state.world.player_energy;
        world.player_max_energy = state.world.player_max_energy;
        let mut next_world = world;
        next_world.combat_state.traversal_markers = runtime.traversals();
        let mut capabilities = state.capabilities.clone();
        capabilities.world_id = scene.world_id.clone();
        capabilities.explored_map = next_world.explored.clone();
        state.install_capability_state(next_world, capabilities)?;
        state.kcc = kcc;
        state.world_persistent_v1.environment = environment;
        update_mist_harbor_exploration(&mut state, &scene, true)?;
        state.latest_sample = InputSample::new(state.world.revision.world_epoch, 0, 0, 0.0, 0.0)
            .map_err(|e| format!("E_SCENE_INPUT: {e:?}"))?;
        state.stepper = FixedStepClock::new(state.stepper.config);
        state.last_received_seq = 0;
        state.last_client_time_ms = 0;
        state.pending_combat.clear();
        state.presentation_events.clear();
        state.next_presentation_event_id = 1;
        state.sound_cues.clear();
        state.next_sound_cue_id = 1;
        runtime.world_epoch = state.world.revision.world_epoch;
        clockworks_roster::mark_installed(&mut state.world_persistent_v1.clockworks,
            &scene, runtime.is_complete_clockworks_production())?;
        gear_shaft_support::mark_installed(&mut state.world_persistent_v1.clockworks,
            &scene, runtime.is_complete_clockworks_production())?;
        grey_hive_roster::mark_installed(&mut state.world_persistent_v1.grey_hive,
            &scene, runtime.is_complete_clockworks_production())?;
        swarm_roster::mark_installed(&mut state.world_persistent_v1.grey_hive,
            &scene, runtime.is_complete_clockworks_production())?;
        tidebound_roster::mark_installed(&mut state.world_persistent_v1.mist_harbor,
            &scene, runtime.is_complete_clockworks_production())?;
        signal_wraith_roster::install(&mut state, &scene, runtime.is_complete_clockworks_production())?;
        enable_native_sentinel_charge(&mut state.world, &scene, runtime.is_complete_clockworks_production())?;
        clockworks_controls::restore_scene(&mut runtime, &state.world_persistent_v1.clockworks)?;
        Ok(runtime)
    }

    pub fn activate_scene_trigger(
        &self,
        id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        let mut guard = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let mut candidate = guard.as_ref().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?.clone();
        let mut next_state = state.clone();
        let events = candidate
            .trigger(
                id,
                request_id,
                world_epoch,
                next_state.world.player.position_m,
            )
            .map_err(|e| e.to_string())?;
        apply_scene_progress(&mut next_state, &candidate, &events, request_id)?;
        refresh_scene_kcc(&mut next_state, &candidate)?;
        next_state
            .world
            .bump_authority_revision()
            .map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
        for event in &events {
            emit_scene_presentation(&mut next_state, event);
        }
        *state = next_state;
        *guard = Some(candidate);
        Ok(project_scene_view(&state, guard.as_ref()))
    }

    pub fn activate_scene_checkpoint(
        &self,
        id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        let mut guard = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let mut candidate = guard.as_ref().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?.clone();
        let event = candidate
            .checkpoint(id, request_id, world_epoch, state.world.player.position_m)
            .map_err(|e| e.to_string())?;
        state.world.checkpoint_id = candidate.checkpoint_id.clone();
        state
            .world
            .bump_authority_revision()
            .map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
        emit_scene_presentation(&mut state, &event);
        *guard = Some(candidate);
        Ok(project_scene_view(&state, guard.as_ref()))
    }

    pub fn transition_scene(
        &self,
        id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<WorldView, String> {
        let mut live_state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        let mut state = live_state.clone();
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        let mut guard = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let mut candidate = guard.as_ref().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?.clone();
        let source_world = candidate.current_scene().world_id.clone();
        let source_scene_id = candidate.current_scene().scene_id.clone();
        if is_clockworks_elite_arena_exit(
            &source_world,
            &source_scene_id,
            &state.world.world_id,
            &state.world.scene_id,
            id,
        ) && !state
            .world_persistent_v1
            .clockworks
            .forged_guard_elite_first_kill
        {
            return Err("E_SCENE_CLOCKWORKS_ELITE_FIRST_KILL_REQUIRED".into());
        }
        let completed_events = completed_world_events(&state, &source_world);
        let transition = candidate
            .transition_with_progression(
                id,
                request_id,
                world_epoch,
                state.world.player.position_m,
                &completed_events,
            )
            .map_err(|e| e.to_string())?;
        let is_registered_sentinel_beacon_transition = source_world == "grey_hive"
            && source_scene_id == SENTINEL_SCENE_ID
            && state.world.world_id == "grey_hive"
            && state.world.scene_id == SENTINEL_SCENE_ID
            && id == "gh_sentinel_arena_to_beacon";
        if is_registered_sentinel_beacon_transition
            && !state.world_persistent_v1.grey_hive.sentinel_first_kill
        {
            return Err("E_SCENE_SENTINEL_FIRST_KILL_REQUIRED".into());
        }
        if source_world == "mist_harbor" && source_scene_id == warden_encounter::SCENE_ID
            && id == "mh_warden_arena_to_extraction" && !state.world_persistent_v1.mist_harbor.warden_defeated {
            return Err("E_MH_WARDEN_DEFEAT_REQUIRED".into());
        }
        let mut route = state.route.clone();
        if matches!(
            transition.target_world_id.as_str(),
            "grey_hive" | "mist_harbor" | "clockworks"
        ) && route.current_world_id != transition.target_world_id
        {
            match apply_route_command(
                &mut route,
                RouteCommand::Enter {
                    world_id: transition.target_world_id.clone(),
                    request_id: format!("scene-enter-{request_id}"),
                },
                state.world.revision,
            ) {
                RouteResult::Applied { .. } => {}
                RouteResult::Rejected { code, message } => {
                    return Err(format!("E_SCENE_ROUTE: {code:?}: {message}"))
                }
            }
        }
        let prior = state.world.clone();
        let mut world = WorldStateV3::new(
            &transition.target_world_id,
            transition.next_epoch,
            transition.position,
        )
        .map_err(|e| format!("E_SCENE_WORLD: {e:?}"))?;
        world.revision = crate::world_v3::WorldRevision {
            world_epoch: transition.next_epoch,
            ..prior.revision
        };
        world.server_time_ms = state.world.server_time_ms;
        world.rear_view = state.world.rear_view.clone();
        world.scene_id = transition.target_scene_id.clone();
        world.checkpoint_id = candidate.checkpoint_id.clone();
        world.player_hp = prior.player_hp;
        world.player_max_hp = prior.player_max_hp;
        world.player_energy = prior.player_energy;
        world.player_max_energy = prior.player_max_energy;
        world.combat_state.traversal_markers = candidate.traversals();
        let scene = candidate.current_scene().clone();
        let environment = environmental_hazards::prepared_scene(&state, &scene)?;
        world.generic_actors = spawn_generic_actors(&scene)?;
        warden_encounter::prepare_world(&mut world, &scene, state.world_persistent_v1.mist_harbor.warden_defeated)?;
        reset_clockworks_regulator_encounter_for_new_boss(
            &mut state.world_persistent_v1.clockworks,
            &scene,
            &world,
        )?;
        clockworks_core::restore_regulator_progression(
            &mut world,
            &scene,
            &state.world_persistent_v1.clockworks,
        )?;
        restore_clockworks_elite_progression(
            &mut world,
            &scene,
            state
                .world_persistent_v1
                .clockworks
                .forged_guard_elite_first_kill,
        )?;
        world.sentinels = spawn_authored_sentinels(&scene)?;
        let target_events = route_completed_events(&route, &scene.world_id);
        let kcc = scene_kcc_for_state(&scene, &target_events, &state)?;
        let mut capabilities = state.capabilities.clone();
        rebase_capability_revision_epoch(&mut capabilities, transition.next_epoch)?;
        capabilities.world_id = transition.target_world_id.clone();
        capabilities.explored_map = world.explored.clone();
        state.install_capability_state(world, capabilities)?;
        state.route = route;
        state.kcc = kcc;
        state.world_persistent_v1.environment = environment;
        update_mist_harbor_exploration(&mut state, &scene, true)?;
        state.stepper = FixedStepClock::new(state.stepper.config);
        state.latest_sample = InputSample::new(transition.next_epoch, 0, 0, 0.0, 0.0)
            .map_err(|e| format!("E_SCENE_INPUT: {e:?}"))?;
        state.last_received_seq = 0;
        state.last_client_time_ms = 0;
        state.pending_combat.clear();
        state.presentation_events.clear();
        state.next_presentation_event_id = 1;
        state.sound_cues.clear();
        state.next_sound_cue_id = 1;
        clockworks_roster::mark_installed(&mut state.world_persistent_v1.clockworks,
            &scene, candidate.is_complete_clockworks_production())?;
        gear_shaft_support::mark_installed(&mut state.world_persistent_v1.clockworks,
            &scene, candidate.is_complete_clockworks_production())?;
        grey_hive_roster::mark_installed(&mut state.world_persistent_v1.grey_hive,
            &scene, candidate.is_complete_clockworks_production())?;
        swarm_roster::mark_installed(&mut state.world_persistent_v1.grey_hive,
            &scene, candidate.is_complete_clockworks_production())?;
        tidebound_roster::mark_installed(&mut state.world_persistent_v1.mist_harbor,
            &scene, candidate.is_complete_clockworks_production())?;
        signal_wraith_roster::install(&mut state, &scene, candidate.is_complete_clockworks_production())?;
        enable_native_sentinel_charge(&mut state.world, &scene, candidate.is_complete_clockworks_production())?;
        clockworks_controls::restore_scene(&mut candidate, &state.world_persistent_v1.clockworks)?;
        emit_scene_presentation(&mut state, &transition.event);
        Self::prepare_entry(&mut state, live_state.entry_generation)?;
        *live_state = state;
        *guard = Some(candidate);
        Ok(project_scene_view(&live_state, guard.as_ref()))
    }

    /// Validated loading boundary for compiler-produced traversal markers.
    /// SceneRuntime will own scene selection later.
    pub fn load_traversal_scene_definition(&self, scene_json: &str) -> Result<(), String> {
        let scene: TraversalSceneDefinition = serde_json::from_str(scene_json)
            .map_err(|error| format!("E_TRAVERSAL_SCENE_SCHEMA: {error}"))?;
        if scene.schema_version != 1
            || scene.world_id.trim().is_empty()
            || scene.scene_id.trim().is_empty()
            || !scene.bounds_m.width.is_finite()
            || !scene.bounds_m.depth.is_finite()
            || scene.bounds_m.width <= 0.0
            || scene.bounds_m.depth <= 0.0
        {
            return Err("E_TRAVERSAL_SCENE_SCHEMA".into());
        }
        let (world_id, markers) = validate_traversal_scene(scene)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.world.world_id != world_id {
            return Err("E_TRAVERSAL_SCENE_WORLD".into());
        }
        if state.world.combat_state.active_action.is_some()
            || state.world.combat_state.active_traversal.is_some()
            || !state.pending_combat.is_empty()
        {
            return Err("E_TRAVERSAL_SCENE_BUSY".into());
        }
        state.world.combat_state.traversal_markers = markers;
        state.world.combat_state.traversal_cooldowns_ms.clear();
        Ok(())
    }

    pub fn presentation_events_since(
        &self,
        world_epoch: u64,
        after_event_id: u64,
    ) -> Result<Vec<PresentationEvent>, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if world_epoch != state.world.revision.world_epoch {
            return Err("E_EVENT_STALE_EPOCH".into());
        }
        if let Some(first) = state.presentation_events.first() {
            if after_event_id != 0 && after_event_id < first.event_id.saturating_sub(1) {
                return Err("E_EVENT_GAP_RESYNC_REQUIRED".into());
            }
        }
        Ok(state
            .presentation_events
            .iter()
            .filter(|event| event.event_id > after_event_id)
            .cloned()
            .collect())
    }

    pub fn sound_cues_since(
        &self,
        world_epoch: u64,
        after_event_id: u64,
    ) -> Result<Vec<SoundCueEvent>, String> {
        let state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if world_epoch != state.world.revision.world_epoch {
            return Err("E_SOUND_CUE_STALE_EPOCH".into());
        }
        if after_event_id >= state.next_sound_cue_id {
            return Err("E_SOUND_CUE_CURSOR_AHEAD".into());
        }
        if let Some(first) = state.sound_cues.first() {
            if after_event_id < first.event_id.saturating_sub(1) {
                return Err("E_SOUND_CUE_GAP_RESYNC_REQUIRED".into());
            }
        }
        Ok(state
            .sound_cues
            .iter()
            .filter(|event| event.event_id > after_event_id)
            .cloned()
            .collect())
    }

    /// Rust validates the fixed actor id and actual authoritative player
    /// position; the browser cannot submit a route-progress event directly.
    pub fn interact(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> Result<FormalInteractionResponse, String> {
        let loaded_epoch = {
            let scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
            if scene.as_ref().is_some_and(|runtime| runtime.is_complete_clockworks_production())
                && (grey_hive_beacon::is_beacon_id(actor_id) || actor_id == "gh_exit_extraction_console") {
                return Err("E_INTERACTION_SOURCE_EPOCH_REQUIRED".into());
            }
            if scene.as_ref().is_some_and(|runtime| runtime.is_complete_clockworks_production()
                && runtime.current_scene().world_id == "clockworks")
            {
                return Err("E_INTERACTION_SOURCE_EPOCH_REQUIRED".into());
            }
            scene.as_ref().map(|runtime| runtime.world_epoch)
        };
        if let Some(epoch) = loaded_epoch {
            return self.activate_scene_interaction(actor_id, request_id, epoch);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let mut scene_guard = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Ok(interaction_rejected(
                &state,
                scene_guard.as_ref(),
                "E_RUNTIME_PAUSED",
            ));
        }
        if let Some(active) = scene_guard.as_ref() {
            let mut candidate = active.clone();
            let events = match candidate.interact(
                actor_id,
                request_id,
                state.world.revision.world_epoch,
                state.world.player.position_m,
            ) {
                Ok(events) => events,
                Err(error) => {
                    return Ok(interaction_rejected(
                        &state,
                        scene_guard.as_ref(),
                        &error.to_string(),
                    ))
                }
            };
            if let Err(error) = apply_scene_progress(&mut state, &candidate, &events, request_id) {
                return Ok(interaction_rejected(&state, scene_guard.as_ref(), &error));
            }
            state
                .world
                .bump_authority_revision()
                .map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
            for event in &events {
                emit_scene_presentation(&mut state, event);
            }
            *scene_guard = Some(candidate);
            let view = project_scene_view(&state, scene_guard.as_ref());
            return Ok(FormalInteractionResponse {
                applied: true,
                already_applied: false,
                error_code: None,
                events: events.iter().map(|e| format!("{e:?}")).collect(),
                receipt: CommandReceipt::outcome(request_id, true, false, None, view.clone()),
                view,
            });
        }
        if actor_id != POWER_CONSOLE_ID {
            return Ok(interaction_rejected(
                &state,
                scene_guard.as_ref(),
                "E_INTERACTION_ACTOR_UNKNOWN",
            ));
        }
        if request_id.trim().is_empty() || request_id.len() > 128 {
            return Ok(interaction_rejected(
                &state,
                scene_guard.as_ref(),
                "E_INTERACTION_REQUEST_INVALID",
            ));
        }
        let player = state.world.player.position_m;
        let dx = player.x_m - POWER_CONSOLE_POSITION.x_m;
        let dy = player.y_m - POWER_CONSOLE_POSITION.y_m;
        let dz = player.z_m - POWER_CONSOLE_POSITION.z_m;
        if dx * dx + dy * dy + dz * dz > POWER_INTERACTION_RANGE_M * POWER_INTERACTION_RANGE_M {
            return Ok(interaction_rejected(
                &state,
                scene_guard.as_ref(),
                "E_INTERACTION_OUT_OF_RANGE",
            ));
        }
        if has_progress(&state, POWER_EVENT_ID) {
            let view = project_view(&state);
            return Ok(FormalInteractionResponse {
                applied: false,
                already_applied: true,
                error_code: None,
                events: vec![],
                receipt: CommandReceipt::outcome(request_id, false, true, None, view.clone()),
                view,
            });
        }

        let mut candidate = state.route.clone();
        let result = apply_route_command(
            &mut candidate,
            RouteCommand::Progress {
                event_id: POWER_EVENT_ID.into(),
                request_id: request_id.to_owned(),
            },
            state.world.revision,
        );
        match result {
            RouteResult::Applied { events, .. } => {
                state
                    .world
                    .bump_authority_revision()
                    .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
                state.route = candidate;
                state.kcc = StaticKccWorld::grey_hive_first_flow(true);
                state.refresh_build_movement_rules();
                emit_presentation(&mut state, "PowerActivated", POWER_CONSOLE_POSITION, 1.0);
                emit_presentation(&mut state, "DoorOpened", GATE_A_POSITION, 1.0);
                let view = project_view(&state);
                Ok(FormalInteractionResponse {
                    applied: true,
                    already_applied: false,
                    error_code: None,
                    events: events
                        .into_iter()
                        .map(|event| format!("{event:?}"))
                        .collect(),
                    receipt: CommandReceipt::outcome(request_id, true, false, None, view.clone()),
                    view,
                })
            }
            RouteResult::Rejected { code, message } => {
                let error_code = Some(format!("{code:?}: {message}"));
                let view = project_view(&state);
                Ok(FormalInteractionResponse {
                    applied: false,
                    already_applied: false,
                    receipt: CommandReceipt::outcome(
                        request_id,
                        false,
                        false,
                        error_code.clone(),
                        view.clone(),
                    ),
                    error_code,
                    events: vec![],
                    view,
                })
            }
        }
    }

    /// Epoch-bound Action-v2 path for authored scene interactions.
    pub fn activate_scene_interaction(
        &self,
        interaction_id: &str,
        request_id: &str,
        world_epoch: u64,
    ) -> Result<FormalInteractionResponse, String> {
        self.activate_scene_interaction_mode(interaction_id, request_id, world_epoch, false)
    }

    /// New environment controls require the caller's source epoch; legacy interact cannot rebind them.
    pub fn activate_environment_control(
        &self, interaction_id: &str, request_id: &str, world_epoch: u64,
    ) -> Result<FormalInteractionResponse, String> {
        self.activate_scene_interaction_mode(interaction_id, request_id, world_epoch, true)
    }

    fn activate_scene_interaction_mode(
        &self, interaction_id: &str, request_id: &str, world_epoch: u64, environment_only: bool,
    ) -> Result<FormalInteractionResponse, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        let mut guard = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Ok(interaction_rejected(
                &state,
                guard.as_ref(),
                "E_RUNTIME_PAUSED",
            ));
        }
        let Some(active) = guard.as_ref() else {
            return Ok(interaction_rejected(
                &state,
                guard.as_ref(),
                "E_SCENE_REGISTRY_NOT_LOADED",
            ));
        };
        if active.is_complete_clockworks_production()
            && (world_epoch != state.world.revision.world_epoch || world_epoch != active.world_epoch)
        {
            return Ok(interaction_rejected(&state, guard.as_ref(), "E_SCENE_RUNTIME_StaleEpoch"));
        }
        if !active.current_scene().height_allows(interaction_id,state.world.player.position_m.y_m) {
            return Ok(interaction_rejected(&state,guard.as_ref(),"E_SCENE_RUNTIME_OutOfRange"));
        }
        let environment_control = active.current_scene().interactions.iter().find(|item| item.id == interaction_id)
            .is_some_and(|item| item.kind == "environment_control" && item.environment_control.is_some());
        if environment_only {
            if world_epoch != state.world.revision.world_epoch || world_epoch != active.world_epoch {
                return Ok(interaction_rejected(&state, guard.as_ref(), "E_SCENE_RUNTIME_StaleEpoch"));
            }
            if active.current_scene().scene_id != state.world.scene_id || active.current_scene().world_id != state.world.world_id {
                return Ok(interaction_rejected(&state, guard.as_ref(), "E_ENV_CONTROL_SCENE_MISMATCH"));
            }
            if !environment_control {
                return Ok(interaction_rejected(&state, guard.as_ref(), "E_ENV_CONTROL_KIND_REQUIRED"));
            }
        } else if environment_control {
            return Ok(interaction_rejected(&state, guard.as_ref(), "E_ENV_CONTROL_EPOCH_COMMAND_REQUIRED"));
        }
        if active.is_complete_clockworks_production() && grey_hive_beacon::is_beacon_id(interaction_id) {
            let mut next_state = state.clone(); let mut candidate = active.clone();
            return match grey_hive_beacon::interact(&mut next_state, &mut candidate, interaction_id, request_id, world_epoch) {
                Ok(response) => { *state = next_state; *guard = Some(candidate); Ok(response) }
                Err(error) => Ok(interaction_rejected(&state, guard.as_ref(), &error)),
            };
        }
        if let Err(error) =
            clockworks_core::validate_core_interaction(&state, active, interaction_id)
        {
            return Ok(interaction_rejected(&state, guard.as_ref(), &error));
        }
        if state.world.world_id == "clockworks"
            && state.world.scene_id == "cw_shutdown_exit"
            && active.current_scene().world_id == "clockworks"
            && active.current_scene().scene_id == "cw_shutdown_exit"
            && interaction_id == "cw_air_step_grant_staged"
        {
            return Ok(interaction_rejected(
                &state,
                guard.as_ref(),
                "E_CLOCKWORKS_AIR_STEP_MARKER_STAGED",
            ));
        }
        let is_regulator_valve = interaction_id == "cw_regulator_valve_furnace_link_staged";
        let regulator_valve = is_regulator_valve
            && active.world_epoch == state.world.revision.world_epoch
            && registered_clockworks_regulator_roster(active.current_scene(), &state.world);
        if is_regulator_valve && !regulator_valve {
            return Ok(interaction_rejected(
                &state,
                guard.as_ref(),
                "E_CW_VALVE_SCOPE_INVALID",
            ));
        }
        if regulator_valve {
            let clockworks = &state.world_persistent_v1.clockworks;
            if !clockworks.regulator_phase2_active || state.world.generic_actors[0].hp == 0 {
                return Ok(interaction_rejected(
                    &state,
                    guard.as_ref(),
                    "E_CW_VALVE_PHASE_INACTIVE",
                ));
            }
            if state.world.server_time_ms < clockworks.regulator_valve_cooldown_until_ms {
                return Ok(interaction_rejected(
                    &state,
                    guard.as_ref(),
                    "E_CW_VALVE_COOLDOWN",
                ));
            }
        }
        let pump_control_position = active
            .current_scene()
            .interactions
            .iter()
            .find(|item| item.id == interaction_id && item.kind == "pump_control")
            .map(|item| vec3_from_array(item.position));
        let mut candidate = active.clone();
        let reconfirmation = match clockworks_controls::reconfirm_interaction(
            &state, &mut candidate, interaction_id, request_id, world_epoch,
        ) {
            Ok(events) => events,
            Err(error) => return Ok(interaction_rejected(&state, guard.as_ref(), &error)),
        };
        let control_reconfirmed = reconfirmation.is_some();
        let reconfirmed = match reconfirmation {
            Some(events) => Some(events),
            None => match clockworks_campaign::reconfirm_shutdown(
                &state, &mut candidate, interaction_id, request_id, world_epoch,
            ) {
                Ok(events) => events,
                Err(error) => return Ok(interaction_rejected(&state, guard.as_ref(), &error)),
            },
        };
        let reconfirmed = match reconfirmed {
            Some(events) => Some(events),
            None => match grey_hive_beacon::reconfirm_extraction(&state, &mut candidate, interaction_id, request_id, world_epoch) {
                Ok(events) => events,
                Err(error) => return Ok(interaction_rejected(&state, guard.as_ref(), &error)),
            },
        };
        let interacted = match reconfirmed {
            Some(events) => Ok(events),
            None => candidate.interact(
                interaction_id, request_id, world_epoch, state.world.player.position_m,
            ),
        };
        let events = match interacted {
            Ok(events) => events,
            Err(error) => {
                return Ok(interaction_rejected(&state, guard.as_ref(), &error.to_string()))
            }
        };
        if let Some(pump_position) = pump_control_position {
            let mut next_state = state.clone();
            let east_beacon_complete = completed_world_events(&next_state, "mist_harbor")
                .contains(crate::world_persistent_v1::MIST_BEACON_EAST_EVENT_ID);
            let result = next_state.world_persistent_v1.mist_harbor.pump.start(
                &next_state.world.world_id,
                east_beacon_complete,
                next_state.world.server_time_ms,
            );
            match result {
                Ok(crate::world_persistent_v1::PumpStartResult::Started) => {
                    refresh_scene_kcc(&mut next_state, &candidate)?;
                    next_state
                        .world
                        .bump_authority_revision()
                        .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
                    emit_presentation(&mut next_state, "PumpStarted", pump_position, 1.0);
                    *state = next_state;
                    *guard = Some(candidate);
                    let view = project_scene_view(&state, guard.as_ref());
                    return Ok(FormalInteractionResponse {
                        applied: true,
                        already_applied: false,
                        error_code: None,
                        events: events.iter().map(|event| format!("{event:?}")).collect(),
                        receipt: CommandReceipt::outcome(
                            request_id,
                            true,
                            false,
                            None,
                            view.clone(),
                        ),
                        view,
                    });
                }
                Ok(_) => {
                    *guard = Some(candidate);
                    let view = project_scene_view(&state, guard.as_ref());
                    return Ok(FormalInteractionResponse {
                        applied: false,
                        already_applied: true,
                        error_code: None,
                        events: Vec::new(),
                        receipt: CommandReceipt::outcome(
                            request_id,
                            false,
                            true,
                            None,
                            view.clone(),
                        ),
                        view,
                    });
                }
                Err(error) => {
                    return Ok(interaction_rejected(
                        &state,
                        guard.as_ref(),
                        &format!("E_PUMP_{error:?}"),
                    ));
                }
            }
        }
        let control_receipt = match clockworks_controls::validated_receipt(&state, active, &candidate, &events, control_reconfirmed) {
            Ok(receipt) => receipt,
            Err(error) => return Ok(interaction_rejected(&state, guard.as_ref(), &error)),
        };
        let mut next_state = state.clone();
        if let Err(error) = apply_scene_progress(&mut next_state, &candidate, &events, request_id) {
            return Ok(interaction_rejected(&state, guard.as_ref(), &error));
        }
        clockworks_controls::record_receipt(&mut next_state, control_receipt);
        if regulator_valve {
            let now = next_state.world.server_time_ms;
            let valve = candidate
                .current_scene()
                .interactions
                .iter()
                .find(|item| item.id == interaction_id)
                .ok_or("E_CW_VALVE_MARKER_MISSING")?;
            let cooldown = valve.cooldown_ms.unwrap_or(8_000).max(8_000);
            let cooled_until = now.saturating_add(6_000);
            let clockworks = &mut next_state.world_persistent_v1.clockworks;
            clockworks.regulator_heat_cooled_until_ms = cooled_until;
            clockworks.regulator_heat_warning_until_ms = cooled_until.saturating_add(600);
            clockworks.regulator_heat_next_damage_at_ms =
                clockworks.regulator_heat_warning_until_ms;
            clockworks.regulator_valve_cooldown_until_ms = now.saturating_add(cooldown);
        }
        if let Err(error) = environmental_hazards::apply_control(&mut next_state, candidate.current_scene(), interaction_id) {
            return Ok(interaction_rejected(&state, guard.as_ref(), &error));
        }
        if let Err(error) = refresh_scene_kcc(&mut next_state, &candidate) {
            return Ok(interaction_rejected(&state, guard.as_ref(), &error));
        }
        next_state
            .world
            .bump_authority_revision()
            .map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
        for event in &events {
            emit_scene_presentation(&mut next_state, event);
        }
        emit_authored_signal_ping(&mut next_state, &candidate, &events)?;
        *state = next_state;
        *guard = Some(candidate);
        let view = project_scene_view(&state, guard.as_ref());
        Ok(FormalInteractionResponse {
            applied: true,
            already_applied: false,
            error_code: None,
            events: events.iter().map(|e| format!("{e:?}")).collect(),
            receipt: CommandReceipt::outcome(request_id, true, false, None, view.clone()),
            view,
        })
    }

    /// Submit client intent to the mailbox. Only the Rust owner advances time.
    /// The response waits for application so legacy command receipts still
    /// acknowledge the exact sequence they accepted.
    pub fn submit_input(
        &self,
        sample: InputSample,
        combat: Vec<CombatIntentRequest>,
    ) -> Result<WorldView, String> {
        if sample.protocol != INPUT_PROTOCOL || sample.protocol_version != INPUT_PROTOCOL_VERSION {
            return Err("E_INPUT_PROTOCOL".into());
        }
        sample
            .validate_axes()
            .map_err(|error| format!("E_INPUT_INVALID: {error:?}"))?;
        let sample_aim_x = sample.aim_x;
        let sample_aim_z = sample.aim_z;
        let mut sample = InputSample::new(
            sample.world_epoch,
            sample.seq,
            sample.client_time_ms,
            sample.move_x,
            sample.move_z,
        )
        .and_then(|sample| sample.with_aim(sample_aim_x, sample_aim_z))
        .map_err(|error| format!("E_INPUT_INVALID: {error:?}"))?;

        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if sample.world_epoch != state.world.revision.world_epoch {
            return Err("E_INPUT_STALE_EPOCH".into());
        }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if sample.seq <= state.last_received_seq || sample.seq <= state.latest_sample.seq {
            return Err("E_INPUT_STALE_SEQUENCE".into());
        }
        if sample.client_time_ms < state.last_client_time_ms {
            return Err("E_INPUT_TIME_REGRESSION".into());
        }
        if sample.aim_x * sample.aim_x + sample.aim_z * sample.aim_z < 0.04 {
            sample.aim_x = state.latest_sample.aim_x;
            sample.aim_z = state.latest_sample.aim_z;
        }

        let wanted_seq = sample.seq;
        let wanted_epoch = sample.world_epoch;
        state.latest_sample = sample;
        state.latest_input_received_at = Instant::now();
        state
            .pending_combat
            .extend(combat.into_iter().map(CombatIntent::from));
        #[cfg(feature = "deterministic-replay")]
        if self.input_replay {
            return self.advance_input_replay(&mut state);
        }
        drop(state);

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let state = self
                .state
                .lock()
                .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
            if state.world.revision.world_epoch != wanted_epoch {
                return Err("E_INPUT_STALE_EPOCH".into());
            }
            if let Some(error) = &state.last_owner_error {
                return Err(format!("E_SIMULATION_OWNER: {error}"));
            }
            if state.last_received_seq >= wanted_seq {
                let scene = self
                    .scene_runtime
                    .lock()
                    .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
                return Ok(project_scene_view(&state, scene.as_ref()));
            }
            if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
                return Err("E_RUNTIME_PAUSED".into());
            }
            if Instant::now() >= deadline {
                return Err("E_OWNER_RESPONSE_TIMEOUT".into());
            }
            drop(state);
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn submit_action(&self, action: ActionCommandRequest) -> Result<WorldView, String> {
        #[cfg(feature = "deterministic-replay")]
        if self.input_replay {
            return Err("E_INPUT_REPLAY_ACTION_UNSUPPORTED".into());
        }
        if action.protocol_version != 2 {
            return Err("E_ACTION_PROTOCOL".into());
        }
        if action.request_id == 0 || action.request_id > build_ui::MAX_SAFE_REVISION {
            return Err("E_ACTION_REQUEST_INVALID".into());
        }
        let intent = match action.kind {
            ActionKind::PrimaryAttack => CombatIntent::ActionAttack {
                request_id: action.request_id,
            },
            ActionKind::Dash => CombatIntent::ActionDash {
                request_id: action.request_id,
            },
            ActionKind::Pulse => CombatIntent::Pulse {
                request_id: action.request_id,
            },
            ActionKind::GuardStart | ActionKind::Guard => CombatIntent::GuardStart {
                request_id: action.request_id,
            },
            ActionKind::GuardEnd => CombatIntent::GuardEnd {
                request_id: action.request_id,
            },
            ActionKind::Pierce => CombatIntent::Pierce {
                request_id: action.request_id,
            },
            ActionKind::ContextTraversal => CombatIntent::ContextTraversal {
                request_id: action.request_id,
                marker_index: 0,
            },
            _ => return Err("E_ACTION_NOT_IMPLEMENTED".into()),
        };
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if action.world_epoch != state.world.revision.world_epoch {
            return Err("E_ACTION_STALE_EPOCH".into());
        }
        if action.client_time_ms < state.last_client_time_ms {
            return Err("E_ACTION_TIME_REGRESSION".into());
        }
        let handled_ids = state.world.combat_state.handled_request_ids();
        if handled_ids.contains(&action.request_id)
            || state
                .pending_combat
                .iter()
                .any(|pending| pending.request_id() == action.request_id)
        {
            return Err("E_ACTION_DUPLICATE".into());
        }
        let greatest_seen = handled_ids
            .iter()
            .copied()
            .chain(
                state
                    .pending_combat
                    .iter()
                    .map(|pending| pending.request_id()),
            )
            .max()
            .unwrap_or(0);
        if action.request_id <= greatest_seen {
            return Err("E_ACTION_STALE_REQUEST".into());
        }
        let intent = if matches!(intent, CombatIntent::ContextTraversal { .. }) {
            if state.world.combat_state.active_action.is_some()
                || state.world.combat_state.active_traversal.is_some()
                || !state.pending_combat.is_empty()
                || state.world.player.dash_remaining_ms > 0
            {
                return Err("E_ACTION_BUSY".into());
            }
            let markers = &state.world.combat_state.traversal_markers;
            if markers.is_empty() {
                return Err("E_TRAVERSAL_NO_MARKER".into());
            }
            let in_range = markers
                .iter()
                .enumerate()
                .filter_map(|(index, marker)| {
                    let dx = state.world.player.position_m.x_m - marker.from.x_m;
                    let dz = state.world.player.position_m.z_m - marker.from.z_m;
                    let distance = (dx * dx + dz * dz).sqrt();
                    (distance <= marker.range_m && marker.from_height_range_m
                        .is_none_or(|range|range.contains(state.world.player.position_m.y_m))).then_some((index, distance, marker))
                })
                .collect::<Vec<_>>();
            if in_range.is_empty() {
                return Err("E_TRAVERSAL_OUT_OF_RANGE".into());
            }
            let eligible = in_range
                .iter()
                .filter(|(_, _, marker)| {
                    marker.required_capabilities.iter().all(|required| {
                        if required == CAP_AIR_STEP {
                            state
                                .effective_rules_v6
                                .capability_permissions
                                .contains(&CapabilityPermission::AuthoredAirStep)
                        } else {
                            // Other authored marker requirements remain acquisition requirements,
                            // not a new selection policy for Map/Acoustic or a route bypass.
                            state
                                .capabilities
                                .grants
                                .iter()
                                .any(|grant| &grant.capability_id == required)
                        }
                    })
                })
                .min_by(|left, right| left.1.total_cmp(&right.1));
            let Some((marker_index, _, marker)) = eligible else {
                return Err("E_TRAVERSAL_CAPABILITY".into());
            };
            if state
                .world
                .combat_state
                .traversal_cooldowns_ms
                .get(&marker.id)
                .copied()
                .unwrap_or(0)
                > 0
            {
                return Err("E_TRAVERSAL_COOLDOWN".into());
            }
            CombatIntent::ContextTraversal {
                request_id: action.request_id,
                marker_index: *marker_index,
            }
        } else {
            intent
        };
        let epoch = state.world.revision.world_epoch;
        let target_tick = state.world.revision.server_tick.saturating_add(1);
        state.pending_combat.push(intent);
        state.last_client_time_ms = action.client_time_ms;
        drop(state);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let state = self
                .state
                .lock()
                .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
            if state.world.revision.world_epoch != epoch {
                return Err("E_ACTION_STALE_EPOCH".into());
            }
            if let Some(error) = &state.last_owner_error {
                return Err(format!("E_SIMULATION_OWNER: {error}"));
            }
            if state.world.revision.server_tick >= target_tick {
                return self.project_locked_scene_view(&state);
            }
            if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
                return Err("E_RUNTIME_PAUSED".into());
            }
            if Instant::now() >= deadline {
                return Err("E_OWNER_RESPONSE_TIMEOUT".into());
            }
            drop(state);
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn apply_route_command(
        &self,
        request: RouteCommandRequest,
    ) -> Result<FormalRouteResponse, String> {
        if matches!(&request, RouteCommandRequest::Progress { event_id, .. } if event_id == "clockworks_core")
        {
            return Err("E_CLOCKWORKS_CORE_SOURCE_INVALID".into());
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        if let Some(target) = request.target_world() {
            let canonical = crate::world_progression::migrate_world_id(target)
                .map_err(|code| format!("E_ROUTE_REJECTED: {code:?}"))?;
            if canonical != state.world.world_id {
                return Err("E_WORLD_NOT_LOADED".into());
            }
        }
        let strict_native = self.scene_registry.lock().map_err(|_| "E_SCENE_REGISTRY_LOCK_POISONED")?
            .as_ref().is_some_and(WorldRegistry::is_complete_clockworks_production);
        if strict_native && (matches!(&request, RouteCommandRequest::Progress { event_id, .. } if event_id.starts_with("hive_"))
            || matches!(&request, RouteCommandRequest::Enter { world_id, .. } | RouteCommandRequest::Complete { world_id, .. }
                | RouteCommandRequest::Revisit { world_id, .. } if world_id == "grey_hive")) {
            return Err("E_GREY_HIVE_AUTHORED_COMMAND_REQUIRED".into());
        }
        if strict_native
            && (matches!(&request, RouteCommandRequest::Progress { event_id, .. }
                if event_id.starts_with("mist_"))
                || matches!(&request,
                    RouteCommandRequest::Enter { world_id, .. }
                    | RouteCommandRequest::Complete { world_id, .. }
                    | RouteCommandRequest::Revisit { world_id, .. }
                    if world_id == "mist_harbor"))
        {
            return Err("E_MIST_HARBOR_AUTHORED_COMMAND_REQUIRED".into());
        }
        if strict_native
            && (matches!(&request, RouteCommandRequest::Progress { event_id, .. }
                if event_id.starts_with("clockworks_"))
                || matches!(&request,
                    RouteCommandRequest::Enter { world_id, .. }
                    | RouteCommandRequest::Complete { world_id, .. }
                    | RouteCommandRequest::Revisit { world_id, .. }
                    if matches!(world_id.as_str(), "clockworks" | "clockwork_city")))
        {
            return Err("E_CLOCKWORKS_AUTHORED_COMMAND_REQUIRED".into());
        }
        let mut candidate = state.route.clone();
        let result = crate::world_progression::apply_route_command(
            &mut candidate,
            request.into_route_command(),
            state.world.revision,
        );
        let (applied, error_code, events) = match result {
            RouteResult::Applied { events, .. } => {
                state
                    .world
                    .bump_authority_revision()
                    .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
                state.route = candidate;
                let scene_guard = self
                    .scene_runtime
                    .lock()
                    .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
                if let Some(scene) = scene_guard.as_ref() {
                    refresh_scene_kcc(&mut state, scene)?;
                } else {
                    state.kcc =
                        StaticKccWorld::grey_hive_first_flow(has_progress(&state, POWER_EVENT_ID));
                    state.refresh_build_movement_rules();
                }
                (
                    true,
                    None,
                    events
                        .into_iter()
                        .map(|event| format!("{event:?}"))
                        .collect(),
                )
            }
            RouteResult::Rejected { code, message } => {
                (false, Some(format!("{code:?}: {message}")), Vec::new())
            }
        };
        let scene_guard = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        Ok(FormalRouteResponse {
            applied,
            error_code,
            events,
            view: project_scene_view(&state, scene_guard.as_ref()),
        })
    }

    pub fn apply_capability_command(
        &self,
        request: CapabilityCommandRequest,
    ) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.paused {
            return Err("E_RUNTIME_PAUSED".into());
        }
        let mut candidate = state.capabilities.clone();
        let effects =
            apply_command_at_revision(&mut candidate, request.into(), state.world.revision)
                .map_err(|error| format!("E_CAPABILITY_REJECTED: {error:?}"))?;
        candidate
            .set_explored_map(state.world.explored.clone())
            .map_err(|error| format!("E_CAPABILITY_MAP: {error:?}"))?;
        let mut world = state.world.clone();
        if !effects.is_empty() {
            apply_effects_atomically(&mut world, &effects)
                .map_err(|error| format!("E_WORLD_EFFECT: {error:?}"))?;
        }
        world
            .bump_authority_revision()
            .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
        state.install_capability_state(world, candidate)?;
        self.project_locked_scene_view(&state)
    }

    pub fn choose_first_enhancement(&self, capability_id: &str) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if !state
            .route
            .progress
            .iter()
            .find(|progress| progress.world_id == crate::world_progression::WORLD_GREY_HIVE)
            .is_some_and(|progress| progress.completed)
        {
            return Err("E_CAPABILITY_REQUIRES_GH_FIRST_CLEAR".into());
        }
        if !state.paused {
            return Err("E_ENHANCEMENT_REQUIRES_FORMAL_PAUSE".into());
        }
        let mut candidate = state.capabilities.clone();
        let effects = choose_first_enhancement(&mut candidate, capability_id, state.world.revision)
            .map_err(|error| format!("E_CAPABILITY_REJECTED: {error:?}"))?;
        candidate
            .set_explored_map(state.world.explored.clone())
            .map_err(|error| format!("E_CAPABILITY_MAP: {error:?}"))?;
        let mut world = state.world.clone();
        if !effects.is_empty() {
            apply_effects_atomically(&mut world, &effects)
                .map_err(|error| format!("E_WORLD_EFFECT: {error:?}"))?;
        }
        world
            .bump_authority_revision()
            .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
        state.install_capability_state(world, candidate)?;
        self.project_locked_scene_view(&state)
    }

    fn rebase_restored_entry(restored: &mut RuntimeState, live: &RuntimeState) -> Result<(), String> {
        // restore_state already maps saved_epoch to saved_epoch + 1. Preserve that
        // contract while also excluding every earlier journey in this owner.
        let epoch = live.world.revision.world_epoch.max(restored.world.revision.world_epoch.saturating_sub(1))
            .checked_add(1).filter(|epoch| *epoch <= build_ui::MAX_SAFE_REVISION)
            .ok_or("E_WORLD_EPOCH_EXHAUSTED")?;
        restored.world.revision.world_epoch = epoch;
        rebase_capability_revision_epoch(&mut restored.capabilities, epoch)?;
        restored.latest_sample.world_epoch = epoch;
        Ok(())
    }

    fn prepare_entry(state: &mut RuntimeState, prior_generation: u64) -> Result<(), String> {
        let generation = prior_generation.checked_add(1)
            .filter(|generation| *generation <= build_ui::MAX_SAFE_REVISION)
            .ok_or("E_SCENE_ENTRY_GENERATION_EXHAUSTED")?;
        state.world.bump_authority_revision().map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
        state.entry_generation = generation;
        state.entry_pause_requested = false;
        state.pending_entry = Some(crate::world_v3::SceneEntryToken {
            generation,
            world_id: state.world.world_id.clone(),
            scene_id: state.world.scene_id.clone(),
            world_epoch: state.world.revision.world_epoch,
        });
        state.paused = true;
        state.pending_combat.clear();
        state.latest_sample = InputSample::new(state.world.revision.world_epoch, 0, 0, 0.0, 0.0)
            .map_err(|error| format!("E_SCENE_INPUT: {error:?}"))?;
        state.world.player.velocity_mps = Vec3::zero();
        state.stepper = FixedStepClock::new(state.stepper.config);
        Ok(())
    }

    /// Only a completed first frame for the exact prepared destination may release the owner.
    /// The generation protects repeated Continue even when the saved epoch is identical.
    pub fn scene_ready(&self, token: &crate::world_v3::SceneEntryToken, remain_paused: bool) -> Result<WorldView, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
        if state.pending_entry.as_ref() != Some(token)
            || token.world_id != state.world.world_id
            || token.scene_id != state.world.scene_id
            || token.world_epoch != state.world.revision.world_epoch {
            return Err("E_SCENE_ENTRY_STALE_TOKEN".into());
        }
        let mut candidate = state.clone();
        candidate.world.bump_authority_revision().map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
        candidate.pending_entry = None;
        candidate.paused = remain_paused || candidate.entry_pause_requested;
        candidate.entry_pause_requested = false;
        candidate.stepper = FixedStepClock::new(candidate.stepper.config);
        candidate.latest_input_received_at = Instant::now();
        let scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        *state = candidate;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    pub fn pause(&self) -> Result<WorldView, String> {
        self.set_paused(true)
    }

    pub fn resume(&self) -> Result<WorldView, String> {
        self.set_paused(false)
    }

    pub fn pause_context(&self, context: &SessionContext) -> Result<WorldView, String> {
        self.set_paused_in_context(true, Some(context), None)
    }

    pub fn resume_context(&self, context: &SessionContext) -> Result<WorldView, String> {
        self.set_paused_in_context(false, Some(context), None)
    }

    pub fn pause_context_ordered(&self, context: &SessionContext, sequence: u64) -> Result<WorldView, String> {
        self.set_paused_in_context(true, Some(context), Some(sequence))
    }

    pub fn resume_context_ordered(&self, context: &SessionContext, sequence: u64) -> Result<WorldView, String> {
        self.set_paused_in_context(false, Some(context), Some(sequence))
    }

    fn set_paused(&self, paused: bool) -> Result<WorldView, String> {
        self.set_paused_in_context(paused, None, None)
    }

    fn set_paused_in_context(&self, paused: bool, context: Option<&SessionContext>, sequence: Option<u64>) -> Result<WorldView, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if context.is_some_and(|context| context.world_id != state.world.world_id
            || context.scene_id != state.world.scene_id
            || context.world_epoch != state.world.revision.world_epoch) {
            return Err("E_LIFECYCLE_STALE_CONTEXT".into());
        }
        if sequence.is_some_and(|sequence| sequence == 0 || sequence > build_ui::MAX_SAFE_REVISION
            || sequence <= state.last_lifecycle_sequence) {
            return Err("E_LIFECYCLE_STALE_COMMAND".into());
        }
        if state.world.player_hp == 0 {
            if !paused { return Err("E_RUNTIME_DEAD".into()); }
            return self.project_locked_scene_view(&state);
        }
        if !paused && state.pending_entry.is_some() {
            return Err("E_SCENE_ENTRY_NOT_READY".into());
        }
        let held_guard = state.world.combat_state.active_action.as_ref()
            .is_some_and(|action| action.kind == crate::continuous_combat::CombatActionKind::Guard);
        let retained_input = state.latest_sample.move_x != 0.0 || state.latest_sample.move_z != 0.0
            || state.world.player.velocity_mps != Vec3::zero()
            || !state.pending_combat.is_empty() || held_guard;
        let mut candidate = state.clone();
        if let Some(sequence) = sequence { candidate.last_lifecycle_sequence = sequence; }
        if paused && context.is_some() && candidate.pending_entry.is_some() {
            // A stopped/failed frontend may have an older ready(false) still in transit.
            // That token can be acknowledged, but cannot override this newer hold.
            candidate.entry_pause_requested = true;
        }
        if state.paused != paused || paused && retained_input {
            candidate.world.bump_authority_revision()
                .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
            candidate.paused = paused;
            candidate.stepper = FixedStepClock::new(candidate.stepper.config);
        }
        if paused {
            // Pause is also an atomic input release. No stale movement or queued
            // combat intent may execute when this same journey resumes.
            candidate.latest_sample.move_x = 0.0;
            candidate.latest_sample.move_z = 0.0;
            candidate.world.player.velocity_mps = Vec3::zero();
            candidate.pending_combat.clear();
            if held_guard {
                candidate.world.combat_state.active_action = None;
                candidate.world.combat_state.action_locks_facing = false;
            }
        }
        let scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        if (paused && context.is_some()) || (state.paused && !paused) {
            if let Some(scene) = scene.as_ref() {
                // A new contextual hold refreshes the current phase before the
                // frontend validates its cue-bearing frame. Timers remain held.
                // A real resume also publishes fresh cues; an already-running
                // idempotent resume does not replay them.
                ordinary_enemy_presentation::prepare_restored(&mut candidate, scene);
            }
        }
        *state = candidate;
        Ok(project_scene_view(&state, scene.as_ref()))
    }

    pub fn return_to_hub(&self) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if state.pending_entry.is_some() {
            state.pending_entry = None;
            return self.project_locked_scene_view(&state);
        }
        if state.world.player_hp == 0 { return self.project_locked_scene_view(&state); }
        let backup = state.clone();
        if !state.paused {
            state
                .world
                .bump_authority_revision()
                .map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
            state.paused = true;
        }
        let scene = match self.scene_runtime.lock() {
            Ok(scene) => scene,
            Err(_) => {
                *state = backup;
                return Err("E_SCENE_RUNTIME_LOCK_POISONED".into());
            }
        };
        let save = match crate::save_v6::SaveV6::capture(&state, scene.as_ref()) {
            Ok(save) => save,
            Err(error) => {
                *state = backup;
                return Err(error);
            }
        };
        if let Err(error) = crate::save_v6::write_save(&self.save_root, &save) {
            *state = backup;
            return Err(error);
        }
        state.pending_entry = None;
        Ok(project_scene_view(&state, scene.as_ref()))
    }
}

impl Drop for FormalRuntime {
    fn drop(&mut self) {
        self.stop_owner.store(true, Ordering::Release);
        if let Ok(mut handle) = self.owner_handle.lock() {
            if let Some(handle) = handle.take() {
                let _ = handle.join();
            }
        }
    }
}

fn wait_until(deadline: Instant) {
    if let Some(delay) = deadline.checked_duration_since(Instant::now()) {
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn simulation_owner_loop(
    state: Arc<Mutex<RuntimeState>>,
    scene_runtime: Arc<Mutex<Option<SceneRuntime>>>,
    stop: Arc<AtomicBool>,
) {
    let interval = Duration::from_secs_f64(1.0 / FIXED_HZ as f64);
    let mut deadline = Instant::now() + interval;
    while !stop.load(Ordering::Acquire) {
        wait_until(deadline);
        if stop.load(Ordering::Acquire) {
            break;
        }
        if let Ok(mut state) = state.lock() {
            if !state.paused && state.world.player_hp > 0 && state.last_owner_error.is_none() {
                let backup = state.clone();
                let scene_guard = scene_runtime.lock();
                let result = match scene_guard {
                    Ok(scene) => advance_owner_step_with_scene(&mut state, scene.as_ref()),
                    Err(_) => Err("E_SCENE_RUNTIME_LOCK_POISONED".into()),
                };
                if let Err(error) = result {
                    *state = backup;
                    state.last_owner_error = Some(error);
                }
            }
        }
        deadline += interval;
        if deadline <= Instant::now() {
            deadline = Instant::now() + interval;
        }
    }
}

fn advance_owner_step(state: &mut RuntimeState) -> Result<(), String> {
    advance_owner_step_with_scene(state, None)
}

fn advance_owner_step_with_scene(
    state: &mut RuntimeState,
    scene: Option<&SceneRuntime>,
) -> Result<(), String> {
    advance_owner_step_with_input_age(state, scene, None)
}

fn advance_owner_step_with_input_age(
    state: &mut RuntimeState,
    scene: Option<&SceneRuntime>,
    replay_input_age: Option<Duration>,
) -> Result<(), String> {
    if state.world.player_hp == 0 { return Ok(()); }
    let dt_s = state.stepper.config.dt_s();
    let steps = state
        .stepper
        .push_frame(dt_s)
        .map_err(|error| format!("E_TICK_SCHEDULER: {error:?}"))?;
    if steps != 1 {
        return Err("E_TICK_SCHEDULER: expected one step".into());
    }
    let registered_sentinel_scene = scene.is_some_and(|scene| {
        let current = scene.current_scene();
        current.world_id == "grey_hive"
            && current.scene_id == SENTINEL_SCENE_ID
            && state.world.world_id == "grey_hive"
            && state.world.scene_id == SENTINEL_SCENE_ID
    });
    let registered_sentinel_roster_is_unique =
        state.world.sentinels.len() == 1 && state.world.sentinels[0].entity_id == SENTINEL_SPAWN_ID;
    let registered_sentinel_was_alive = registered_sentinel_scene
        && registered_sentinel_roster_is_unique
        && state.world.sentinels.iter().any(|sentinel| {
            sentinel.entity_id == SENTINEL_SPAWN_ID && sentinel.hp > 0 && sentinel.active
        });
    let registered_clockworks_elite_scene = scene.is_some_and(|scene| {
        registered_clockworks_elite_roster(scene.current_scene(), &state.world)
    });
    let registered_clockworks_elite_was_alive =
        registered_clockworks_elite_scene && state.world.generic_actors[0].hp > 0;
    let registered_clockworks_regulator_scene = scene.is_some_and(|scene| {
        registered_clockworks_regulator_roster(scene.current_scene(), &state.world)
    });
    let registered_clockworks_regulator_was_alive = registered_clockworks_regulator_scene
        && state.world.generic_actors[0].hp > 0
        && state.world.generic_actors[0].validate();
    let mut sample = state.latest_sample.clone();
    // The live path samples wall-clock age here, exactly as before. Only the
    // feature-gated input replay supplies the age of its scheduled sample.
    let input_age = replay_input_age.unwrap_or_else(|| state.latest_input_received_at.elapsed());
    if input_age > Duration::from_millis(250) {
        sample.move_x = 0.0;
        sample.move_z = 0.0;
    }
    let intents = std::mem::take(&mut state.pending_combat);
    let regulator_combat_proof = clockworks_core::capture_regulator_combat(state, scene);
    let warden_combat_proof = warden_encounter::capture_combat(state, scene);
    let output = step_world_held_input(
        &mut state.world,
        &state.kcc,
        CwStepInput {
            sample: &sample,
            dt_s,
            combat: &intents,
        },
    )
    .map_err(|error| format!("E_WORLD_STEP: {error:?}"))?;
    clockworks_core::record_regulator_combat_defeat(
        state,
        scene,
        regulator_combat_proof,
        &output.combat,
    );
    warden_encounter::record_defeat(state, scene, warden_combat_proof, &output.combat);
    warden_encounter::present_events(state, scene, &output.actor_runtime);
    ordinary_enemy_presentation::present_events(state, scene, &output.actor_runtime);
    if let Some(scene) = scene { environmental_hazards::advance(state, scene)?; }
    if registered_clockworks_regulator_scene {
        advance_regulator_environment(state, scene.expect("registered scene"))?;
    }
    let registered_sentinel_hit = output.combat.iter().any(|event| match event {
        crate::continuous_combat::CombatEvent::AttackHit { target_id, .. }
        | crate::continuous_combat::CombatEvent::ActionImpact { target_id, .. } => {
            target_id == SENTINEL_SPAWN_ID
        }
        _ => false,
    });
    let registered_sentinel_died = registered_sentinel_was_alive
        && registered_sentinel_hit
        && state.world.sentinels.iter().any(|sentinel| {
            sentinel.entity_id == SENTINEL_SPAWN_ID && sentinel.hp == 0 && !sentinel.active
        });
    if registered_sentinel_died {
        state
            .world_persistent_v1
            .grey_hive
            .mark_sentinel_first_kill();
        if let Some(position) = state
            .world
            .sentinels
            .iter()
            .find(|sentinel| sentinel.entity_id == SENTINEL_SPAWN_ID)
            .map(|sentinel| sentinel.position_m)
        {
            emit_presentation(state, "SentinelDeath", position, 1.0);
        }
    }
    let registered_clockworks_elite_hit = output.combat.iter().any(|event| match event {
        crate::continuous_combat::CombatEvent::AttackHit { target_id, .. }
        | crate::continuous_combat::CombatEvent::ActionImpact { target_id, .. } => {
            target_id == CLOCKWORKS_ELITE_SPAWN_ID
        }
        _ => false,
    });
    let registered_clockworks_elite_died = registered_clockworks_elite_was_alive
        && registered_clockworks_elite_hit
        && state.world.generic_actors.len() == 1
        && state.world.generic_actors[0].entity_id == CLOCKWORKS_ELITE_SPAWN_ID
        && state.world.generic_actors[0].entity_type == CLOCKWORKS_ELITE_ENTITY_TYPE
        && state.world.generic_actors[0].hp == 0
        && state.world.generic_actors[0].validate();
    if registered_clockworks_elite_died {
        state
            .world_persistent_v1
            .clockworks
            .mark_forged_guard_elite_first_kill();
        if let Some(scene) = scene {
            refresh_scene_kcc(state, scene)?;
        }
        emit_presentation(
            state,
            "ForgedGuardEliteDeath",
            state.world.generic_actors[0].position_m,
            1.0,
        );
    }
    if registered_clockworks_elite_was_alive {
        for event in &output.actor_runtime {
            match event {
                crate::world_v3::ActorRuntimeEvent::AttackWindup {
                    actor_id,
                    kind: crate::world_v3::ActorAttackKind::PressureWave,
                    origin_m,
                    radius_m,
                    ..
                } if actor_id == CLOCKWORKS_ELITE_SPAWN_ID => emit_presentation_sized(
                    state,
                    "ForgedGuardPressureWindup",
                    *origin_m,
                    *radius_m,
                    1.0,
                ),
                crate::world_v3::ActorRuntimeEvent::AttackImpact {
                    actor_id,
                    kind: crate::world_v3::ActorAttackKind::PressureWave,
                    origin_m,
                    radius_m,
                    ..
                } if actor_id == CLOCKWORKS_ELITE_SPAWN_ID => emit_presentation_sized(
                    state,
                    "ForgedGuardPressureImpact",
                    *origin_m,
                    *radius_m,
                    1.0,
                ),
                _ => {}
            }
        }
    }
    if registered_clockworks_regulator_was_alive {
        for event in &output.actor_runtime {
            match event {
                crate::world_v3::ActorRuntimeEvent::AttackWindup {
                    actor_id,
                    kind: crate::world_v3::ActorAttackKind::PressureWave,
                    origin_m,
                    radius_m,
                    ..
                } if actor_id == CLOCKWORKS_REGULATOR_SPAWN_ID => emit_presentation_sized(
                    state,
                    "PrimeRegulatorPressureWindup",
                    *origin_m,
                    *radius_m,
                    1.0,
                ),
                crate::world_v3::ActorRuntimeEvent::AttackImpact {
                    actor_id,
                    kind: crate::world_v3::ActorAttackKind::PressureWave,
                    origin_m,
                    radius_m,
                    ..
                } if actor_id == CLOCKWORKS_REGULATOR_SPAWN_ID => emit_presentation_sized(
                    state,
                    "PrimeRegulatorPressureImpact",
                    *origin_m,
                    *radius_m,
                    1.0,
                ),
                _ => {}
            }
        }
    }
    if registered_sentinel_was_alive {
        for event in &output.sentinel {
            let (sentinel_id, kind) = match event {
                crate::sentinel_ai::SentinelEvent::StateChanged {
                    sentinel_id,
                    state: crate::sentinel_ai::SentinelState::Attack,
                } => (sentinel_id, "SentinelAttackWindup"),
                crate::sentinel_ai::SentinelEvent::StateChanged {
                    sentinel_id,
                    state: crate::sentinel_ai::SentinelState::HeavyAttack,
                } => (sentinel_id, "SentinelHeavyWindup"),
                crate::sentinel_ai::SentinelEvent::StateChanged { sentinel_id,
                    state: crate::sentinel_ai::SentinelState::ChargeWindup } => (sentinel_id, "SentinelChargeWindup"),
                _ => continue,
            };
            if sentinel_id != SENTINEL_SPAWN_ID {
                continue;
            }
            if let Some(position) = state
                .world
                .sentinels
                .iter()
                .find(|sentinel| sentinel.entity_id == SENTINEL_SPAWN_ID)
                .map(|sentinel| sentinel.position_m)
            {
                emit_presentation(state, kind, position, 1.0);
                if let Some(view)=state.world.sentinels.iter().find(|s|s.entity_id==*sentinel_id)
                    .and_then(|s|s.encounter_view(state.stepper.config.hz)) {
                    if let (Some(warning),Some(event))=(view.warning,state.presentation_events.last_mut()) {
                        event.position_m=warning.origin_m;event.radius_m=warning.radius_m;event.direction_rad=warning.direction_rad;
                        event.duration_ms=Some(view.remaining_ms);event.actor_id=Some(sentinel_id.clone());
                        event.attack_id=Some(view.attack_serial);event.range_m=Some(warning.range_m);
                        event.attack_kind=Some(if kind=="SentinelChargeWindup" {"sentinel_charge"} else if kind=="SentinelHeavyWindup" {"sentinel_heavy"} else {"sentinel_light"}.into());
                    }
                }
            }
        }
    }
    if let Some(scene) = scene {
        update_mist_harbor_exploration(state, scene.current_scene(), false)?;
    }
    combat_presentation::present_events(state, &output.combat);
    let now_ms = state.world.server_time_ms;
    if state
        .world_persistent_v1
        .resolve_at(now_ms)
        .map_err(|error| format!("E_PUMP_STATE: {error:?}"))?
    {
        state.kcc.map_terrain_tags(|region_id, authored_tag| {
            state.world_persistent_v1.resolve_terrain_tag(
                &state.world.world_id,
                region_id,
                authored_tag,
            )
        });
        if state.world.world_id == crate::world_persistent_v1::MIST_HARBOR_ID
            && state.world.scene_id == "mh_pump_station"
        {
            emit_presentation(
                state,
                "PumpDrained",
                vec3_from_array(crate::world_persistent_v1::PUMP_CONTROL_POSITION_M),
                1.0,
            );
        }
    }
    if output.combat.iter().any(|event| {
        matches!(
            event,
            crate::continuous_combat::CombatEvent::PlayerDamaged { .. }
        )
    }) {
        state.last_damaged_at_ms = Some(now_ms);
    }
    if output.combat.iter().any(crate::continuous_combat::CombatEvent::counts_as_combat_activity) {
        state.last_combat_at_ms = Some(now_ms);
    }
    state
        .capabilities
        .set_explored_map(state.world.explored.clone())
        .map_err(|error| format!("E_CAPABILITY_MAP: {error:?}"))?;
    if state.world.player_hp > 0 {
    let regeneration_authorized = state
        .effective_rules_v6
        .capability_permissions
        .contains(&CapabilityPermission::DelayedRegeneration);
    let capability_output = tick_regeneration_authorized(
        &mut state.capabilities,
        CapabilityTickInput {
            revision: output.revision,
            now_ms,
            dt_s,
            current_hp: state.world.player_hp,
            max_hp: state.world.player_max_hp,
            last_damaged_at_ms: state.last_damaged_at_ms,
            last_combat_at_ms: state.last_combat_at_ms,
            combat_events: &output.combat,
        },
        regeneration_authorized,
    )
    .map_err(|error| format!("E_CAPABILITY_TICK: {error:?}"))?;
    if !capability_output.effects.is_empty() {
        apply_effects_atomically(&mut state.world, &capability_output.effects)
            .map_err(|error: ContinuousWorldError| format!("E_WORLD_EFFECT: {error:?}"))?;
    }
    }
    state.last_received_seq = sample.seq;
    state.last_client_time_ms = state.last_client_time_ms.max(sample.client_time_ms);
    if state.world.player_hp == 0 {
        state.paused = true;
        state.pending_entry = None;
        state.pending_combat.clear();
        state.latest_sample.move_x = 0.0;
        state.latest_sample.move_z = 0.0;
        state.world.player.velocity_mps = Vec3::zero();
        emit_presentation(state, "PlayerDeath", state.world.player.position_m, 1.0);
    }
    Ok(())
}

fn validate_traversal_scene(
    scene: TraversalSceneDefinition,
) -> Result<(String, Vec<TraversalMarker>), String> {
    const PLAYER_RADIUS_M: f32 = 0.35;
    const KNOWN_CAPABILITIES: [&str; 5] = [
        CAP_LOCAL_MAP,
        CAP_REAR_VIEW,
        CAP_ENEMY_VITALS,
        CAP_REGENERATION,
        CAP_AIR_STEP,
    ];
    let width = scene.bounds_m.width;
    let depth = scene.bounds_m.depth;
    let collisions = scene
        .collision
        .into_iter()
        .map(|entry| entry.polygon)
        .collect::<Vec<_>>();
    for polygon in &collisions {
        if polygon.len() < 3
            || polygon.iter().any(|point| {
                !point[0].is_finite()
                    || !point[1].is_finite()
                    || point[0] < 0.0
                    || point[0] > width
                    || point[1] < 0.0
                    || point[1] > depth
            })
        {
            return Err("E_TRAVERSAL_SCENE_COLLISION".into());
        }
    }
    let mut ids = std::collections::BTreeSet::new();
    let mut markers = Vec::with_capacity(scene.logic.traversal.len());
    for data in scene.logic.traversal {
        if data.id.is_empty()
            || data.id.len() > 128
            || !data
                .id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
            || !ids.insert(data.id.clone())
        {
            return Err("E_TRAVERSAL_MARKER_ID".into());
        }
        if !data.range_m.is_finite()
            || (data.range_m - 1.2).abs() > f32::EPSILON
            || data.cooldown_ms != 350
        {
            return Err("E_TRAVERSAL_TUNING".into());
        }
        let mut required = std::collections::BTreeSet::new();
        for capability in &data.required_capabilities {
            if !KNOWN_CAPABILITIES.contains(&capability.as_str()) {
                return Err("E_TRAVERSAL_CAPABILITY_UNKNOWN".into());
            }
            if !required.insert(capability) {
                return Err("E_TRAVERSAL_CAPABILITY_DUPLICATE".into());
            }
        }
        if data.from_height_range_m.is_some_and(|range|!range.contains(data.from[1])) {
            return Err("E_TRAVERSAL_HEIGHT_RANGE".into());
        }
        let from = Vec3 {
            x_m: data.from[0],
            y_m: data.from[1],
            z_m: data.from[2],
        };
        let to = Vec3 {
            x_m: data.to[0],
            y_m: data.to[1],
            z_m: data.to[2],
        };
        if ![from.x_m, from.y_m, from.z_m, to.x_m, to.y_m, to.z_m]
            .iter()
            .all(|value| value.is_finite())
            || from.y_m.abs() > f32::EPSILON
            || to.y_m.abs() > f32::EPSILON
            || !traversal_position_clear(from, PLAYER_RADIUS_M, width, depth, &collisions)
            || !traversal_position_clear(to, PLAYER_RADIUS_M, width, depth, &collisions)
        {
            return Err("E_TRAVERSAL_DESTINATION_BLOCKED".into());
        }
        let distance = horizontal_distance(from, to);
        let samples = (distance / 0.1).ceil().max(1.0) as usize;
        if (0..=samples).any(|step| {
            let progress = step as f32 / samples as f32;
            let point = Vec3 {
                x_m: from.x_m + (to.x_m - from.x_m) * progress,
                y_m: 0.0,
                z_m: from.z_m + (to.z_m - from.z_m) * progress,
            };
            !traversal_position_clear(point, PLAYER_RADIUS_M, width, depth, &collisions)
        }) {
            return Err("E_TRAVERSAL_PATH_BLOCKED".into());
        }
        markers.push(TraversalMarker {
            from_height_range_m: data.from_height_range_m,
            id: data.id,
            from,
            to,
            range_m: data.range_m,
            cooldown_ms: data.cooldown_ms,
            required_capabilities: data.required_capabilities,
        });
    }
    Ok((scene.world_id, markers))
}

fn traversal_position_clear(
    position: Vec3,
    radius: f32,
    width: f32,
    depth: f32,
    collisions: &[Vec<[f32; 2]>],
) -> bool {
    if position.x_m < radius
        || position.x_m > width - radius
        || position.z_m < radius
        || position.z_m > depth - radius
    {
        return false;
    }
    !collisions.iter().any(|polygon| {
        let mut inside = false;
        for index in 0..polygon.len() {
            let [ax, az] = polygon[index];
            let [bx, bz] = polygon[(index + 1) % polygon.len()];
            if (az > position.z_m) != (bz > position.z_m) {
                let crossing = ax + (position.z_m - az) * (bx - ax) / (bz - az);
                if position.x_m < crossing {
                    inside = !inside;
                }
            }
            let dx = bx - ax;
            let dz = bz - az;
            let denom = dx * dx + dz * dz;
            let t = if denom == 0.0 {
                0.0
            } else {
                (((position.x_m - ax) * dx + (position.z_m - az) * dz) / denom).clamp(0.0, 1.0)
            };
            let nearest_x = ax + t * dx;
            let nearest_z = az + t * dz;
            if (position.x_m - nearest_x).powi(2) + (position.z_m - nearest_z).powi(2)
                <= radius * radius
            {
                return true;
            }
        }
        inside
    })
}

fn emit_presentation(state: &mut RuntimeState, kind: &str, position_m: Vec3, intensity: f32) {
    emit_presentation_sized(state, kind, position_m, 1.0, intensity);
}

fn emit_presentation_sized(
    state: &mut RuntimeState,
    kind: &str,
    position_m: Vec3,
    radius_m: f32,
    intensity: f32,
) {
    let event = PresentationEvent {
        protocol_version: 2,
        event_id: state.next_presentation_event_id,
        world_epoch: state.world.revision.world_epoch,
        server_tick: state.world.revision.server_tick,
        kind: kind.into(),
        position_m,
        direction_rad: 0.0,
        radius_m,
        intensity,
        actor_id: None,
        member_id: None,
        half_angle_rad: None,
        duration_ms: None,
        attack_kind: None,
        attack_id: None,
        range_m: None,
        combat_feedback: None,
    };
    state.next_presentation_event_id = state.next_presentation_event_id.saturating_add(1);
    state.presentation_events.push(event);
    if state.presentation_events.len() > 256 {
        state.presentation_events.remove(0);
    }
}

fn emit_authored_signal_ping(
    state: &mut RuntimeState,
    scene: &SceneRuntime,
    events: &[SceneEvent],
) -> Result<(), String> {
    let definition = scene.current_scene();
    if state.world.world_id != "mist_harbor"
        || state.route.current_world_id != "mist_harbor"
        || definition.world_id != "mist_harbor"
        || state.world.scene_id != definition.scene_id
    {
        return Ok(());
    }
    let expected = match definition.scene_id.as_str() {
        "mh_tidal_warehouse" => ("mh_west_beacon", "mist_beacon_west"),
        "mh_breakwater" => ("mh_east_beacon", "mist_beacon_east"),
        "mh_resonance_tower" => ("mh_signal_console_staged", "mist_signal"),
        _ => return Ok(()),
    };
    if !events.iter().any(|event| {
        matches!(event,
        SceneEvent::Interaction { id, event_id: Some(route_event) }
            if id == expected.0 && route_event == expected.1)
    }) || !route_completed_events(&state.route, "mist_harbor").contains(expected.1)
    {
        return Ok(());
    }
    let mut sources = definition
        .interactions
        .iter()
        .filter(|item| item.id == expected.0 && item.event.as_deref() == Some(expected.1));
    let source = sources.next().ok_or("E_SOUND_CUE_SOURCE_MISSING")?;
    if sources.next().is_some() || !source.position.iter().all(|value| value.is_finite()) {
        return Err("E_SOUND_CUE_SOURCE_INVALID".into());
    }
    let mapped = state
        .effective_rules_v6
        .capability_permissions
        .contains(&CapabilityPermission::AcousticMapping);
    let (direction_rad, distance_m) = if mapped {
        let dx = source.position[0] - state.world.player.position_m.x_m;
        let dz = source.position[2] - state.world.player.position_m.z_m;
        (Some(dx.atan2(dz)), Some((dx * dx + dz * dz).sqrt()))
    } else {
        (None, None)
    };
    let next_id = state
        .next_sound_cue_id
        .checked_add(1)
        .ok_or("E_SOUND_CUE_ID_EXHAUSTED")?;
    let event = SoundCueEvent {
        protocol_version: 1,
        event_id: state.next_sound_cue_id,
        world_epoch: state.world.revision.world_epoch,
        server_tick: state.world.revision.server_tick,
        world_id: definition.world_id.clone(),
        scene_id: definition.scene_id.clone(),
        kind: "signal_ping".into(),
        direction_rad,
        distance_m,
    };
    state.next_sound_cue_id = next_id;
    state.sound_cues.push(event);
    if state.sound_cues.len() > 256 {
        state.sound_cues.remove(0);
    }
    Ok(())
}

fn update_mist_harbor_exploration(
    state: &mut RuntimeState,
    scene: &crate::scene_runtime::SceneDefinition,
    mark_spawn: bool,
) -> Result<(), String> {
    if scene.world_id != crate::world_persistent_v1::MIST_HARBOR_ID {
        return Ok(());
    }
    let position = state.world.player.position_m;
    if mark_spawn {
        if let Some(spawn) = scene.spawns.iter().find(|spawn| spawn.kind == "player") {
            let [x, _, z] = spawn.position;
            let mut containing = scene
                .exploration_regions
                .iter()
                .filter(|region| point_in_or_on_polygon(x, z, &region.polygon))
                .collect::<Vec<_>>();
            containing.sort_by(|a, b| {
                polygon_edge_distance(x, z, &b.polygon)
                    .total_cmp(&polygon_edge_distance(x, z, &a.polygon))
            });
            if let Some(region) = containing.first() {
                state
                    .world_persistent_v1
                    .mark_explored(&region.id)
                    .map_err(|error| format!("E_EXPLORATION_STATE: {error:?}"))?;
            }
        }
    }
    for region in &scene.exploration_regions {
        if point_in_or_on_polygon(position.x_m, position.z_m, &region.polygon)
            && polygon_edge_distance(position.x_m, position.z_m, &region.polygon) + 1e-5 >= 0.35
        {
            state
                .world_persistent_v1
                .mark_explored(&region.id)
                .map_err(|error| format!("E_EXPLORATION_STATE: {error:?}"))?;
        }
    }
    let rooms = scene
        .exploration_regions
        .iter()
        .filter(|region| state.world_persistent_v1.is_explored(&region.id))
        .map(|region| crate::world_v3::ExploredRoom {
            room_id: region.id.clone(),
            outline_m: region
                .polygon
                .iter()
                .map(|point| vec3_from_array([point[0], 0.0, point[1]]))
                .collect(),
        })
        .collect();
    state.world.explored = crate::world_v3::ExploredMap::new(
        scene.world_id.clone(),
        position,
        rooms,
        Vec::new(),
        Vec::new(),
    )
    .map_err(|error| format!("E_EXPLORATION_MAP: {error:?}"))?;
    state
        .capabilities
        .set_explored_map(state.world.explored.clone())
        .map_err(|error| format!("E_CAPABILITY_MAP: {error:?}"))?;
    Ok(())
}

fn polygon_edge_distance(x: f32, z: f32, polygon: &[[f32; 2]]) -> f32 {
    if polygon.len() < 2 {
        return 0.0;
    }
    polygon
        .iter()
        .enumerate()
        .map(|(index, [ax, az])| {
            let [bx, bz] = polygon[(index + 1) % polygon.len()];
            let dx = bx - ax;
            let dz = bz - az;
            let length_sq = dx * dx + dz * dz;
            let t = if length_sq > 0.0 {
                (((x - ax) * dx + (z - az) * dz) / length_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let ex = x - (ax + t * dx);
            let ez = z - (az + t * dz);
            (ex * ex + ez * ez).sqrt()
        })
        .fold(f32::INFINITY, f32::min)
}

fn project_view(state: &RuntimeState) -> WorldView {
    let enemy_vitals = project_enemy_vitals(state);
    let mut capabilities = project_capabilities_with_rules(
        &state.capabilities,
        state.world.explored.clone(),
        enemy_vitals,
        state.world.rear_view.clone(),
        &state.effective_rules_v6,
    );
    live_rules::project_information(state, None, &mut capabilities);
    if state.world.world_id == "mist_harbor" {
        capabilities.acoustic_mapping_authorized = Some(state.effective_rules_v6.capability_permissions.contains(&CapabilityPermission::AcousticMapping));
    }
    let progression: RouteProjection = project_route(&state.route);
    let mut view = state
        .world
        .view(state.world.last_input_seq, capabilities, progression);
    view.build = build_ui::project_build(state);
    view.entry_token = state.pending_entry.clone();
    if state.world.world_id == crate::world_persistent_v1::MIST_HARBOR_ID {
        view.mist_harbor_pump = Some(crate::world_v3::MistHarborPumpProjection {
            state: match state.world_persistent_v1.mist_harbor.pump.state {
                crate::world_persistent_v1::PumpStatus::Ready => {
                    crate::world_v3::MistHarborPumpState::Ready
                }
                crate::world_persistent_v1::PumpStatus::Draining => {
                    crate::world_v3::MistHarborPumpState::Draining
                }
                crate::world_persistent_v1::PumpStatus::Drained => {
                    crate::world_v3::MistHarborPumpState::Drained
                }
            },
        });
    }
    let aim_x = state.latest_sample.aim_x;
    let aim_z = state.latest_sample.aim_z;
    let aim_length = (aim_x * aim_x + aim_z * aim_z).sqrt();
    if aim_length >= 0.2 {
        view.player.aim_x = aim_x / aim_length;
        view.player.aim_z = aim_z / aim_length;
        if !state.world.combat_state.action_locks_facing {
            view.player.facing_x = view.player.aim_x;
            view.player.facing_z = view.player.aim_z;
            view.player.transform.yaw_rad = view.player.aim_x.atan2(view.player.aim_z);
        }
    }
    view.player.action_state = if state.world.player_hp == 0 {
        "death".into()
    } else if state.world.player.dash_remaining_ms > 0 {
        "dash"
    } else {
        state.world.combat_state.action_state()
    }
    .into();
    view.actors.push(ActorView {
        entity_id: POWER_CONSOLE_ID.into(),
        entity_type: "prop.grey_hive.power_console".into(),
        actor_kind: "power_console".into(),
        transform: Transform {
            position_m: POWER_CONSOLE_POSITION,
            yaw_rad: 0.0,
        },
        active: true, members: None, signal_perception: None,
    });
    let gate_open = has_progress(state, POWER_EVENT_ID);
    view.doors.push(DoorView {
        door_id: GATE_A_ID.into(),
        transform: Transform {
            position_m: GATE_A_POSITION,
            yaw_rad: 0.0,
        },
        open: gate_open,
        locked: !gate_open,
    });
    view
}

fn project_enemy_vitals(state: &RuntimeState) -> Vec<EnemyVitals> {
    if !state
        .effective_rules_v6
        .capability_permissions
        .contains(&CapabilityPermission::EnemyVitalsBasic)
    {
        return Vec::new();
    }

    let sentinel_scene =
        state.world.world_id == "grey_hive" && state.world.scene_id == SENTINEL_SCENE_ID;
    let mut seen_ids = BTreeSet::new();
    let mut duplicate_ids = BTreeSet::new();
    for entity_id in state
        .world
        .generic_actors
        .iter()
        .map(|actor| actor.entity_id.as_str())
        .chain(
            state
                .world
                .sentinels
                .iter()
                .filter(|_| sentinel_scene)
                .map(|sentinel| sentinel.entity_id.as_str()),
        )
        .filter(|entity_id| !entity_id.trim().is_empty())
    {
        if !seen_ids.insert(entity_id.to_owned()) {
            duplicate_ids.insert(entity_id.to_owned());
        }
    }

    let mut vitals = state
        .world
        .generic_actors
        .iter()
        .filter_map(|actor| {
            if !actor.validate() || actor.hp == 0 {
                return None;
            }
            let max_hp = actor.max_hp()?;
            EnemyVitals::new(actor.entity_id.clone(), actor.hp, max_hp).ok()
        })
        .collect::<Vec<_>>();

    if sentinel_scene {
        vitals.extend(state.world.sentinels.iter().filter_map(|sentinel| {
            if sentinel.entity_id != SENTINEL_SPAWN_ID
                || !sentinel.active
                || sentinel.hp == 0
                || sentinel.hp > SENTINEL_DEVELOPMENT_HP
            {
                return None;
            }
            EnemyVitals::new(
                sentinel.entity_id.clone(),
                sentinel.hp,
                SENTINEL_DEVELOPMENT_HP,
            )
            .ok()
        }));
    }

    vitals.retain(|vital| !duplicate_ids.contains(&vital.entity_id));
    vitals
}

fn project_scene_view(state: &RuntimeState, scene: Option<&SceneRuntime>) -> WorldView {
    let mut view = project_view(state);
    let Some(scene) = scene else {
        return view;
    };
    let definition = scene.current_scene();
    view.grey_hive_beacon = grey_hive_beacon::project(state, scene);
    view.sentinel_encounter = project_sentinel_encounter(state, scene);
    if state.kcc.has_support_surfaces() {
        // Every field comes from the same locked authority state. Invalid
        // projection is omitted, causing support-scene readiness to fail closed.
        view.support_scene=(|| {
            let frame=state.kcc.support_frame(state.world.revision.world_epoch,state.world.server_time_ms).ok()?;
            let rider=state.kcc.support_rider_view(&state.world.player,state.world.server_time_ms,
                state.world.combat_state.active_traversal.is_some()).ok()?;
            Some(crate::moving_support::SupportSceneView{schema_version:1,world_id:definition.world_id.clone(),scene_id:definition.scene_id.clone(),
                world_epoch:state.world.revision.world_epoch,server_tick:state.world.revision.server_tick,
                authority_revision:state.world.revision.authority_revision,server_time_ms:state.world.server_time_ms,
                scene_source_sha256:definition.verified_source_sha256.clone(),poses:frame.poses,rider})
        })();
    }
    view.boss_encounter = warden_encounter::project(state, scene);
    live_rules::project_information(state, Some(definition), &mut view.capabilities);
    view.world_id = definition.world_id.clone();
    view.scene_id = definition.scene_id.clone();
    view.checkpoint_id = scene.checkpoint_id.clone();
    view.actors
        .retain(|actor| actor.entity_id != POWER_CONSOLE_ID);
    let completed_events = completed_world_events(state, &definition.world_id);
    view.doors = definition
        .doors
        .iter()
        .map(|door| {
            let open = scene.door_is_open(&door.id, &completed_events);
            DoorView {
                door_id: door.id.clone(),
                transform: Transform {
                    position_m: vec3_from_array(door.position),
                    yaw_rad: 0.0,
                },
                open: open.unwrap_or(false),
                locked: open.is_some_and(|is_open| !is_open),
            }
        })
        .collect();
    let gh_extraction_complete = definition.world_id == "grey_hive"
        && definition.scene_id == "gh_exit"
        && state.route.progress.iter().any(|progress| {
            progress.world_id == "grey_hive"
                && progress.completed
                && progress.first_completion
                && progress
                    .completed_events
                    .iter()
                    .any(|event| event == "hive_extraction")
        });
    let gh_first_clear = state.route.progress.iter().any(|progress| {
        progress.world_id == "grey_hive"
            && progress.completed
            && progress.first_completion
            && progress
                .completed_events
                .iter()
                .any(|event| event == "hive_extraction")
    });
    let mh_progress = state
        .route
        .progress
        .iter()
        .find(|progress| progress.world_id == "mist_harbor");
    let mh_first_clear = mh_progress.is_some_and(|progress| progress.completed);
    let mh_required_events_complete = mh_progress.is_some_and(|progress| {
        ["mist_beacon_west", "mist_beacon_east", "mist_signal"]
            .iter()
            .all(|required| {
                progress
                    .completed_events
                    .iter()
                    .any(|event| event == required)
            })
    });
    view.interactables = definition
        .interactions
        .iter()
        .filter_map(|item| {
            if matches!(
                item.kind.as_str(),
                "future_extraction_marker"
                    | "world_completion_staged_marker"
                    | "return_station_revisit_staged_marker"
            ) || (definition.scene_id == "mh_extraction" && item.id == "mh_extraction_exit")
                || (definition.scene_id == "cw_shutdown_exit" && item.id == "cw_shutdown_exit_portal_staged")
            {
                return None;
            }
            let world_gate_is_open = match (definition.scene_id.as_str(), item.id.as_str()) {
                ("rs_core_room", "rs_world_gate_marker") => {
                    let grey_hive = state
                        .route
                        .progress
                        .iter()
                        .find(|progress| progress.world_id == "grey_hive");
                    grey_hive.is_some_and(|progress| {
                        !progress.completed
                            || crate::world_progression::can_revisit_world(
                                &state.route,
                                "grey_hive",
                            )
                    })
                }
                ("rs_core_room", "rs_mh_world_gate_marker") => {
                    gh_first_clear
                        && (!mh_first_clear
                            || crate::world_progression::can_revisit_world(
                                &state.route,
                                "mist_harbor",
                            ))
                }
                ("rs_core_room", "rs_cw_world_gate_marker") => {
                    scene.is_complete_clockworks_production() && clockworks_campaign::entry_eligible(state)
                }
                _ => true,
            };
            if item.id == "rs_world_gate_marker" && !world_gate_is_open
                || item.id == "rs_mh_world_gate_marker" && !world_gate_is_open
                || item.id == "rs_cw_world_gate_marker" && !world_gate_is_open
            {
                return None;
            }
            Some(crate::world_v3::InteractableView {
                entity_id: item.id.clone(),
                kind: match (definition.scene_id.as_str(), item.id.as_str()) {
                    ("rs_core_room", "rs_world_gate_marker") => "world_gate".into(),
                    ("rs_core_room", "rs_mh_world_gate_marker") => "world_gate".into(),
                    ("rs_core_room", "rs_cw_world_gate_marker") => "world_gate".into(),
                    ("rs_core_room", "rs_mission_terminal_marker") => "mission_terminal".into(),
                    ("rs_core_room", "rs_capability_terminal_marker") => {
                        "capability_terminal".into()
                    }
                    ("rs_core_room", "rs_save_rest_terminal_marker") => "save_rest_terminal".into(),
                    _ => item.kind.clone(),
                },
                transform: Transform {
                    position_m: vec3_from_array(item.position),
                    yaw_rad: 0.0,
                },
                active: if scene.is_complete_clockworks_production() && grey_hive_beacon::is_beacon_id(&item.id) {
                    grey_hive_beacon::available(state, scene, &item.id)
                } else { definition.height_allows(&item.id,state.world.player.position_m.y_m)
                    && (!scene.object_activated(&item.id)
                    || clockworks_controls::reconfirmation_available(state, scene, &item.id)
                    || grey_hive_beacon::extraction_reconfirmation_available(state, scene, &item.id)
                    || (item.id == clockworks_campaign::SHUTDOWN_ID
                        && scene.is_complete_clockworks_production()
                        && clockworks_campaign::shutdown_available(state, scene)))
                    && (item.id != clockworks_campaign::SHUTDOWN_ID
                        || !scene.is_complete_clockworks_production()
                        || clockworks_campaign::shutdown_available(state, scene))
                    && (item.environment_control.is_none() || environmental_hazards::control_available(state, definition, item))
                    && (item.id != clockworks_core::CORE_CONSOLE_ID
                        || clockworks_core::core_console_available(state, scene))
                    && (item.id != "cw_regulator_valve_furnace_link_staged"
                        || (registered_clockworks_regulator_roster(definition, &state.world)
                            && state.world.generic_actors[0].hp > 0
                            && state.world_persistent_v1.clockworks.regulator_phase2_active
                            && state.world.server_time_ms
                                >= state
                                    .world_persistent_v1
                                    .clockworks
                                    .regulator_valve_cooldown_until_ms))
                    && (item.id != "rs_world_gate_marker" || (world_gate_is_open && !state.paused))
                    && (item.id != "rs_mh_world_gate_marker"
                        || (world_gate_is_open && !state.paused))
                    && (item.id != "rs_cw_world_gate_marker"
                        || (world_gate_is_open && !state.paused))
                    && (item.kind != "pump_control"
                        || (state.world_persistent_v1.mist_harbor.pump.state
                            == crate::world_persistent_v1::PumpStatus::Ready
                            && completed_events
                                .contains(crate::world_persistent_v1::MIST_BEACON_EAST_EVENT_ID)))
                    && (!state.paused
                        || definition.scene_id != "rs_core_room"
                        || !matches!(
                            item.id.as_str(),
                            "rs_world_gate_marker"
                                | "rs_mission_terminal_marker"
                                | "rs_capability_terminal_marker"
                                | "rs_save_rest_terminal_marker"
                        ))
                    && !(gh_extraction_complete && item.id == "gh_exit_extraction_console")
                    && (!scene.is_complete_clockworks_production() || item.id != "gh_exit_extraction_console" || grey_hive_beacon::owned(state)) },
            })
        })
        .collect();
    if definition.world_id == "clockworks" && definition.scene_id == "cw_shutdown_exit"
        && scene.is_complete_clockworks_production() && clockworks_campaign::completed(state)
    {
        if let Some(marker) = definition.interactions.iter()
            .find(|item| item.id == "cw_shutdown_exit_portal_staged")
        {
            view.interactables.push(crate::world_v3::InteractableView {
                entity_id: "cw_shutdown_return_to_rs".into(), kind: "world_gate".into(),
                transform: Transform { position_m: vec3_from_array(marker.position), yaw_rad: 0.0 },
                active: !state.paused && state.world.player_hp > 0
                    && definition.height_allows(&marker.id,state.world.player.position_m.y_m),
            });
        }
    }
    if gh_extraction_complete {
        if let Some(marker) = definition
            .interactions
            .iter()
            .find(|interaction| interaction.id == "gh_exit_extraction_console")
        {
            view.interactables.push(crate::world_v3::InteractableView {
                entity_id: "gh_extraction_return_to_rs".into(),
                kind: "world_gate".into(),
                transform: Transform {
                    position_m: vec3_from_array(marker.position),
                    yaw_rad: 0.0,
                },
                active: !state.paused && definition.height_allows(&marker.id,state.world.player.position_m.y_m),
            });
        }
    }
    if definition.world_id == "mist_harbor"
        && definition.scene_id == "mh_extraction"
        && state.route.current_world_id == "mist_harbor"
        && mh_required_events_complete
        && state.world_persistent_v1.mist_harbor.warden_defeated
    {
        if let Some(marker) = definition
            .interactions
            .iter()
            .find(|interaction| interaction.id == "mh_extraction_exit")
        {
            view.interactables.push(crate::world_v3::InteractableView {
                entity_id: "mh_extraction_return_to_rs".into(),
                kind: "world_gate".into(),
                transform: Transform {
                    position_m: vec3_from_array(marker.position),
                    yaw_rad: 0.0,
                },
                active: !state.paused && definition.height_allows(&marker.id,state.world.player.position_m.y_m),
            });
        }
    }
    let route_interactables = scene.route_interactables(
        state.world.player.position_m,
        &completed_events,
        !state.paused,
    );
    let elite_exits_unlocked = state
        .world_persistent_v1
        .clockworks
        .forged_guard_elite_first_kill;
    view.interactables
        .extend(route_interactables.into_iter().map(|mut item| {
            if definition.world_id == "mist_harbor" && definition.scene_id == warden_encounter::SCENE_ID
                && item.entity_id == "mh_warden_arena_to_extraction" && !state.world_persistent_v1.mist_harbor.warden_defeated { item.active = false; }

            if !elite_exits_unlocked
                && is_clockworks_elite_arena_exit(
                    &definition.world_id,
                    &definition.scene_id,
                    &state.world.world_id,
                    &state.world.scene_id,
                    &item.entity_id,
                )
            {
                item.active = false;
            }
            item
        }));
    let regulator_live = scene.world_epoch == state.world.revision.world_epoch
        && registered_clockworks_regulator_roster(definition, &state.world)
        && state.world.generic_actors[0].hp > 0;
    view.hazards = definition
        .hazards
        .iter()
        .map(|item| {
            if let Some(view) = environmental_hazards::project(state, definition, item) { return view; }
            if let Some(view) = encounter_hazards::project_hazard(
                item,
                &state.world_persistent_v1.clockworks,
                state.world.server_time_ms,
                regulator_live,
            ) {
                return view;
            }
            crate::world_v3::HazardView {
                height_range_m: item.height_range_m,
                environment: None,
                entity_id: item.id.clone(),
                kind: item.kind.clone(),
                transform: Transform {
                    position_m: {let mut p=polygon_center(&item.polygon);p.y_m=item.presentation_height();p},
                    yaw_rad: 0.0,
                },
                active: if definition.world_id == "mist_harbor"
                    && definition.scene_id == "mh_signal_yard"
                    && item.id == "mh_signal_interference_region"
                    && item.kind == "signal_interference_zone"
                {
                    item.height_allows(state.world.player.position_m.y_m) && point_in_or_on_polygon(
                        state.world.player.position_m.x_m,
                        state.world.player.position_m.z_m,
                        &item.polygon,
                    )
                } else if let Some(active) = drowned_quay_water_occupancy(
                    &definition.world_id,
                    &definition.scene_id,
                    item,
                    state.world.player.position_m,
                ) {
                    active
                        && state.world_persistent_v1.mist_harbor.pump.state
                            != crate::world_persistent_v1::PumpStatus::Drained
                } else {
                    true
                },
                polygon_m: None,
                warning_remaining_ms: None,
                phase_active: None,
            }
        })
        .collect();
    view.objectives = scene
        .objectives()
        .into_iter()
        .map(
            |(objective_id, status, position)| crate::world_v3::ObjectiveView {
                objective_id,
                state: status,
                position_m: vec3_from_array(position),
            },
        )
        .collect();
    signal_wraith_presentation::project(state, scene, &mut view);
    view
}

fn vec3_from_array(position: [f32; 3]) -> Vec3 {
    Vec3 {
        x_m: position[0],
        y_m: position[1],
        z_m: position[2],
    }
}

fn spawn_generic_actors(
    scene: &crate::scene_runtime::SceneDefinition,
) -> Result<Vec<crate::world_v3::ActorRuntime>, String> {
    scene
        .spawns
        .iter()
        .filter(|spawn| {
            spawn.kind == "enemy" && spawn.entity_type.as_deref() != Some(SENTINEL_ENTITY_TYPE)
        })
        .map(|spawn| {
            let entity_type = spawn
                .entity_type
                .as_deref()
                .ok_or("E_ACTOR_PROFILE_UNKNOWN")?;
            let actor=crate::world_v3::ActorRuntime::spawn(
                &spawn.id,entity_type,vec3_from_array(spawn.position))?;
            if spawn.position[1]>0.001 && !scene.standing_position_valid(spawn.position,actor.body_radius_m()) {
                return Err("E_ACTOR_SUPPORT_INVALID".into());
            }
            Ok(actor)
        })
        .collect()
}

fn registered_clockworks_elite_roster(
    scene: &crate::scene_runtime::SceneDefinition,
    world: &WorldStateV3,
) -> bool {
    let authored = scene
        .spawns
        .iter()
        .filter(|spawn| {
            spawn.id == CLOCKWORKS_ELITE_SPAWN_ID
                || spawn.entity_type.as_deref() == Some(CLOCKWORKS_ELITE_ENTITY_TYPE)
        })
        .collect::<Vec<_>>();
    scene.world_id == "clockworks"
        && scene.scene_id == CLOCKWORKS_ELITE_SCENE_ID
        && world.world_id == "clockworks"
        && world.scene_id == CLOCKWORKS_ELITE_SCENE_ID
        && authored.len() == 1
        && authored[0].id == CLOCKWORKS_ELITE_SPAWN_ID
        && authored[0].kind == "enemy"
        && authored[0].entity_type.as_deref() == Some(CLOCKWORKS_ELITE_ENTITY_TYPE)
        && world.generic_actors.len() == 1
        && world.generic_actors[0].entity_id == CLOCKWORKS_ELITE_SPAWN_ID
        && world.generic_actors[0].entity_type == CLOCKWORKS_ELITE_ENTITY_TYPE
        && world.generic_actors[0].home_m == vec3_from_array(authored[0].position)
}

fn registered_clockworks_regulator_roster(
    scene: &crate::scene_runtime::SceneDefinition,
    world: &WorldStateV3,
) -> bool {
    let authored = scene
        .spawns
        .iter()
        .filter(|spawn| {
            spawn.id == CLOCKWORKS_REGULATOR_SPAWN_ID
                || spawn.entity_type.as_deref() == Some(CLOCKWORKS_REGULATOR_ENTITY_TYPE)
        })
        .collect::<Vec<_>>();
    scene.world_id == "clockworks"
        && scene.scene_id == CLOCKWORKS_REGULATOR_SCENE_ID
        && world.world_id == "clockworks"
        && world.scene_id == CLOCKWORKS_REGULATOR_SCENE_ID
        && authored.len() == 1
        && authored[0].id == CLOCKWORKS_REGULATOR_SPAWN_ID
        && authored[0].kind == "enemy"
        && authored[0].entity_type.as_deref() == Some(CLOCKWORKS_REGULATOR_ENTITY_TYPE)
        && world.generic_actors.len() == 1
        && world.generic_actors[0].entity_id == CLOCKWORKS_REGULATOR_SPAWN_ID
        && world.generic_actors[0].entity_type == CLOCKWORKS_REGULATOR_ENTITY_TYPE
        && world.generic_actors[0].home_m == vec3_from_array(authored[0].position)
}

fn reset_clockworks_regulator_encounter_for_new_boss(
    persistent: &mut crate::world_persistent_v1::ClockworksPersistentState,
    scene: &crate::scene_runtime::SceneDefinition,
    world: &WorldStateV3,
) -> Result<(), String> {
    if scene.world_id != "clockworks" || scene.scene_id != CLOCKWORKS_REGULATOR_SCENE_ID {
        return Ok(());
    }
    if !scene.spawns.iter().any(|spawn| {
        spawn.id == CLOCKWORKS_REGULATOR_SPAWN_ID
            || spawn.entity_type.as_deref() == Some(CLOCKWORKS_REGULATOR_ENTITY_TYPE)
    }) {
        return Ok(());
    }
    if !registered_clockworks_regulator_roster(scene, world) {
        return Err("E_CLOCKWORKS_REGULATOR_ROSTER_INVALID".into());
    }
    persistent.reset_regulator_encounter();
    Ok(())
}

fn advance_regulator_environment(
    state: &mut RuntimeState,
    scene_runtime: &SceneRuntime,
) -> Result<(), String> {
    if state.paused {
        return Ok(());
    }
    let scene = scene_runtime.current_scene();
    if scene_runtime.world_epoch != state.world.revision.world_epoch
        || !registered_clockworks_regulator_roster(scene, &state.world)
        || state.world.generic_actors[0].hp == 0
    {
        return Ok(());
    }
    let hazards = scene
        .hazards
        .iter()
        .filter(|hazard| {
            hazard.id == "cw_regulator_furnace_heat_zone" && hazard.kind == "heat_zone"
        })
        .collect::<Vec<_>>();
    if hazards.len() != 1 {
        return Err("E_CW_HEAT_ZONE_INVALID".into());
    }
    let hazard = hazards[0];
    let (Some(damage), Some(period_ms), Some(warning_ms)) =
        (hazard.damage, hazard.period_ms, hazard.warning_ms)
    else {
        return Err("E_CW_HEAT_TUNING_MISSING".into());
    };
    let now = state.world.server_time_ms;
    let max_hp = state.world.generic_actors[0]
        .max_hp()
        .ok_or("E_CW_REGULATOR_PROFILE_MISSING")?;
    let phase2_active = state.world_persistent_v1.clockworks.regulator_phase2_active;
    if !phase2_active && u64::from(state.world.generic_actors[0].hp) * 100 <= u64::from(max_hp) * 60
    {
        let clockworks = &mut state.world_persistent_v1.clockworks;
        clockworks.regulator_phase2_active = true;
        clockworks.regulator_heat_warning_until_ms = now.saturating_add(warning_ms);
        clockworks.regulator_heat_next_damage_at_ms = clockworks.regulator_heat_warning_until_ms;
    }
    if !state.world_persistent_v1.clockworks.regulator_phase3_active
        && u64::from(state.world.generic_actors[0].hp) * 100 <= u64::from(max_hp) * 30
    {
        state.world_persistent_v1.clockworks.regulator_phase3_active = true;
        state
            .world_persistent_v1
            .clockworks
            .regulator_phase3_started_at_ms = now;
    }
    if state.world_persistent_v1.clockworks.regulator_phase3_active {
        encounter_hazards::advance_machinery(
            state,
            &scene.hazards,
            state
                .world_persistent_v1
                .clockworks
                .regulator_phase3_started_at_ms,
        )?;
    }
    let clockworks = &state.world_persistent_v1.clockworks;
    if !clockworks.regulator_phase2_active
        || now < clockworks.regulator_heat_cooled_until_ms
        || now < clockworks.regulator_heat_next_damage_at_ms
        || !hazard.height_allows(state.world.player.position_m.y_m)
        || !point_in_or_on_polygon(
            state.world.player.position_m.x_m,
            state.world.player.position_m.z_m,
            &hazard.polygon,
        )
    {
        return Ok(());
    }
    encounter_hazards::apply_hazard_damage(
        state,
        damage,
        crate::effects::HazardTag::Heat,
        "ClockworksHeatDamage",
    );
    state
        .world_persistent_v1
        .clockworks
        .regulator_heat_next_damage_at_ms = now.saturating_add(period_ms);
    Ok(())
}

fn restore_clockworks_elite_progression(
    world: &mut WorldStateV3,
    scene: &crate::scene_runtime::SceneDefinition,
    first_kill_complete: bool,
) -> Result<(), String> {
    if scene.world_id != "clockworks" || scene.scene_id != CLOCKWORKS_ELITE_SCENE_ID {
        return Ok(());
    }
    if !registered_clockworks_elite_roster(scene, world) {
        return Err("E_CLOCKWORKS_ELITE_ROSTER_INVALID".into());
    }
    if first_kill_complete {
        world.generic_actors[0].take_damage(u32::MAX);
    }
    Ok(())
}

fn enable_native_sentinel_charge(world:&mut WorldStateV3, scene:&crate::scene_runtime::SceneDefinition, strict:bool)->Result<(),String> {
    if !strict || scene.world_id!="grey_hive" || scene.scene_id!=SENTINEL_SCENE_ID {return Ok(());}
    if world.sentinels.len()!=1 || world.sentinels[0].entity_id!=SENTINEL_SPAWN_ID {return Err("E_SAVE_SENTINEL_ROSTER_MISMATCH".into());}
    world.sentinels[0].enable_charge();
    if !world.sentinels[0].validate() {return Err("E_SAVE_SENTINEL_CHARGE_INVALID".into());}
    Ok(())
}
fn project_sentinel_encounter(state:&RuntimeState, scene:&SceneRuntime)->Option<crate::sentinel_ai::SentinelEncounterView> {
    let definition=scene.current_scene();
    if !scene.is_complete_clockworks_production() || definition.world_id!="grey_hive" || definition.scene_id!=SENTINEL_SCENE_ID
        || state.world.world_id!=definition.world_id || state.world.scene_id!=definition.scene_id
        || state.world.revision.world_epoch!=scene.world_epoch || state.world.sentinels.len()!=1 {return None;}
    let actor=&state.world.sentinels[0];
    if actor.entity_id!=SENTINEL_SPAWN_ID || actor.charge_controller.is_none() || !actor.validate() || !state.kcc.stable_actor_footprint(actor.position_m,0.3)
        || definition.spawns.iter().filter(|s|s.id==SENTINEL_SPAWN_ID && s.kind=="enemy" && s.entity_type.as_deref()==Some(SENTINEL_ENTITY_TYPE)).count()!=1 {return None;}
    actor.encounter_view(state.stepper.config.hz)
}

fn spawn_authored_sentinels(
    scene: &crate::scene_runtime::SceneDefinition,
) -> Result<Vec<crate::sentinel_ai::Sentinel>, String> {
    let authored = scene
        .spawns
        .iter()
        .filter(|spawn| {
            spawn.entity_type.as_deref() == Some(SENTINEL_ENTITY_TYPE)
                || spawn.id == SENTINEL_SPAWN_ID
        })
        .collect::<Vec<_>>();
    let in_sentinel_arena = scene.world_id == "grey_hive" && scene.scene_id == SENTINEL_SCENE_ID;
    if !in_sentinel_arena {
        return if authored.is_empty() {
            Ok(Vec::new())
        } else {
            Err("E_SENTINEL_ROSTER_SCENE_MISMATCH".into())
        };
    }
    if authored.len() != 1
        || authored[0].id != SENTINEL_SPAWN_ID
        || authored[0].kind != "enemy"
        || authored[0].entity_type.as_deref() != Some(SENTINEL_ENTITY_TYPE)
    {
        return Err("E_SENTINEL_ROSTER_INVALID".into());
    }
    let spawn = authored[0];
    Ok(vec![crate::sentinel_ai::Sentinel::new(
        &spawn.id,
        vec3_from_array(spawn.position),
        SENTINEL_DEVELOPMENT_HP,
    )])
}

fn restore_sentinels(
    world: &mut WorldStateV3,
    scene: &crate::scene_runtime::SceneDefinition,
) -> Result<(), String> {
    let authored = spawn_authored_sentinels(scene)?;
    if !world.sentinels.is_empty() && authored.is_empty() {
        return Err("E_SENTINEL_ROSTER_SCENE_MISMATCH".into());
    }
    if world.sentinels.is_empty() {
        // Older v5/v6 records predate the authored Sentinel actor.
        world.sentinels = authored;
        return Ok(());
    }
    if world.sentinels.len() != authored.len()
        || world
            .sentinels
            .iter()
            .zip(&authored)
            .any(|(saved, source)| saved.entity_id != source.entity_id || (saved.position_m.y_m-source.position_m.y_m).abs()>0.001)
    {
        return Err("E_SAVE_SENTINEL_ROSTER_MISMATCH".into());
    }
    Ok(())
}

fn restore_generic_actors(
    world: &mut WorldStateV3,
    scene: &crate::scene_runtime::SceneDefinition,
    persistent: &mut crate::world_persistent_v1::WorldPersistentState,
    strict_native: bool,
) -> Result<(), String> {
    if clockworks_roster::restore(world, scene, &mut persistent.clockworks, strict_native)? { return Ok(()); }
    if grey_hive_roster::restore(world, scene, &mut persistent.grey_hive, strict_native)? { return Ok(()); }
    if swarm_roster::restore(world, scene, &mut persistent.grey_hive, strict_native)? { return Ok(()); }
    let tidebound_restored = tidebound_roster::restore(world, scene, &mut persistent.mist_harbor, strict_native)?;
    if signal_wraith_roster::restore(world, scene, &mut persistent.mist_harbor, strict_native)? { return Ok(()); }
    if tidebound_restored { return Ok(()); }
    let authored = spawn_generic_actors(scene)?;
    if world.generic_actors.is_empty() {
        // Pre-extension v5 saves have no generic actor field.
        world.generic_actors = authored;
        return Ok(());
    }
    if world.generic_actors.len() != authored.len()
        || world
            .generic_actors
            .iter()
            .zip(&authored)
            .any(|(saved, source)| {
                saved.entity_id != source.entity_id
                    || saved.entity_type != source.entity_type
                    || saved.home_m != source.home_m
            })
    {
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    Ok(())
}

fn drowned_quay_water_occupancy(
    world_id: &str,
    scene_id: &str,
    hazard: &crate::scene_runtime::HazardDefinition,
    player_position: Vec3,
) -> Option<bool> {
    (world_id == "mist_harbor"
        && scene_id == "mh_drowned_quay"
        && hazard.id == "mh_water_depth_region"
        && hazard.kind == "water_depth_slowdown")
        .then(|| hazard.height_allows(player_position.y_m)
            && point_in_or_on_polygon(player_position.x_m, player_position.z_m, &hazard.polygon))
}

// A player's center on the authored edge is inside the interference zone.
fn point_in_or_on_polygon(x: f32, z: f32, polygon: &[[f32; 2]]) -> bool {
    if polygon.len() < 3 || !x.is_finite() || !z.is_finite() {
        return false;
    }
    let mut inside = false;
    for index in 0..polygon.len() {
        let [ax, az] = polygon[index];
        let [bx, bz] = polygon[(index + 1) % polygon.len()];
        if ![ax, az, bx, bz].iter().all(|value| value.is_finite()) {
            return false;
        }
        let dx = bx - ax;
        let dz = bz - az;
        let cross = (x - ax) * dz - (z - az) * dx;
        if cross.abs() <= 1e-5
            && x >= ax.min(bx) - 1e-5
            && x <= ax.max(bx) + 1e-5
            && z >= az.min(bz) - 1e-5
            && z <= az.max(bz) + 1e-5
        {
            return true;
        }
        if (az > z) != (bz > z) && x < ax + (z - az) * dx / dz {
            inside = !inside;
        }
    }
    inside
}

fn polygon_center(polygon: &[[f32; 2]]) -> Vec3 {
    let n = polygon.len().max(1) as f32;
    Vec3 {
        x_m: polygon.iter().map(|p| p[0]).sum::<f32>() / n,
        y_m: 0.0,
        z_m: polygon.iter().map(|p| p[1]).sum::<f32>() / n,
    }
}

fn scene_kcc(
    scene: &crate::scene_runtime::SceneDefinition,
    completed_events: &std::collections::BTreeSet<String>,
) -> Result<StaticKccWorld, String> {
    scene_kcc_with_first_kill(scene, completed_events, false)
}

fn scene_kcc_with_first_kill(
    scene: &crate::scene_runtime::SceneDefinition,
    completed_events: &std::collections::BTreeSet<String>,
    forged_guard_elite_first_kill: bool,
) -> Result<StaticKccWorld, String> {
    let bounds = &scene.bounds_m;
    let outer = crate::continuous_kcc::Aabb::new(
        bounds.x,
        bounds.x + bounds.width,
        bounds.z,
        bounds.z + bounds.depth,
    )
    .map_err(|e| format!("E_SCENE_KCC: {e:?}"))?;
    let mut walls = Vec::new();
    for collision in &scene.collision {
        if forged_guard_elite_first_kill
            && scene.world_id == "clockworks"
            && scene.scene_id == CLOCKWORKS_ELITE_SCENE_ID
            && collision.requires_actor_first_kill.as_deref() == Some(CLOCKWORKS_ELITE_SPAWN_ID)
        {
            continue;
        }
        if collision
            .requires_event
            .as_ref()
            .is_some_and(|event| completed_events.contains(event))
        {
            continue;
        }
        let min_x = collision
            .polygon
            .iter()
            .map(|p| p[0])
            .fold(f32::INFINITY, f32::min);
        let max_x = collision
            .polygon
            .iter()
            .map(|p| p[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_z = collision
            .polygon
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let max_z = collision
            .polygon
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        if max_x - min_x > f32::EPSILON && max_z - min_z > f32::EPSILON {
            walls.push(
                crate::continuous_kcc::Aabb::new(min_x, max_x, min_z, max_z)
                    .map_err(|e| format!("E_SCENE_KCC: {e:?}"))?,
            );
        }
    }
    let mut walkable = scene
        .walkable_polygons
        .iter()
        .map(|polygon| polygon.polygon.clone())
        .collect::<Vec<_>>();
    if walkable.is_empty() && scene.world_id == crate::world_persistent_v1::MIST_HARBOR_ID {
        walkable.push(vec![
            [bounds.x + 0.5, bounds.z + 0.5],
            [bounds.x + bounds.width - 0.5, bounds.z + 0.5],
            [bounds.x + bounds.width - 0.5, bounds.z + bounds.depth - 0.5],
            [bounds.x + 0.5, bounds.z + bounds.depth - 0.5],
        ]);
    }
    let mut terrain_regions = scene
        .terrain_regions
        .iter()
        .map(|region| crate::continuous_kcc::KccTerrainRegion {
            id: region.id.clone(),
            tag: region.tag,
            polygon: region.polygon.clone(),
            surface_velocity_mps: region.surface_velocity_mps,
        })
        .collect::<Vec<_>>();
    if scene.world_id == crate::world_persistent_v1::MIST_HARBOR_ID
        && !terrain_regions
            .iter()
            .any(|region| region.id == crate::world_persistent_v1::DROWNED_QUAY_WATER_ID)
    {
        for hazard in scene.hazards.iter().filter(|hazard| {
            hazard.id == crate::world_persistent_v1::DROWNED_QUAY_WATER_ID
                && hazard.kind == "water_depth_slowdown"
        }) {
            terrain_regions.push(crate::continuous_kcc::KccTerrainRegion {
                id: hazard.id.clone(),
                tag: crate::effects::TerrainTag::WaterShallow,
                polygon: hazard.polygon.clone(),
                surface_velocity_mps: None,
            });
        }
    }
    StaticKccWorld::new(outer, walls)
        .with_walkable_polygons(walkable)
        .with_terrain_regions(terrain_regions)
        .with_moving_supports(scene.moving_supports.clone())
        .and_then(|kcc|kcc.with_standing_decks(scene.standing_decks.clone()))
        .map_err(|error|format!("E_SCENE_SUPPORT: {error:?}"))
}

fn scene_kcc_for_state(
    scene: &crate::scene_runtime::SceneDefinition,
    completed_events: &std::collections::BTreeSet<String>,
    state: &RuntimeState,
) -> Result<StaticKccWorld, String> {
    let mut kcc = scene_kcc_with_first_kill(
        scene,
        completed_events,
        scene.world_id == "clockworks"
            && scene.scene_id == CLOCKWORKS_ELITE_SCENE_ID
            && state
                .world_persistent_v1
                .clockworks
                .forged_guard_elite_first_kill,
    )?
    .with_movement_rules(
        state.effective_rules_v6.clone(),
        crate::player_rules::MovementMode::Ground,
    );
    kcc.map_terrain_tags(|region_id, authored_tag| {
        state
            .world_persistent_v1
            .resolve_terrain_tag(&scene.world_id, region_id, authored_tag)
    });
    Ok(kcc)
}

fn restore_scene_state(
    registry: WorldRegistry,
    save: &crate::save_v5::SaveV5,
    epoch: u64,
) -> Result<SceneRuntime, String> {
    let mut runtime =
        SceneRuntime::new(registry, &save.scene_id, epoch).map_err(|_| "E_SAVE_SCENE_UNKNOWN")?;
    let Some(saved) = save
        .scene_states
        .iter()
        .find(|value| value.scene_id == save.scene_id)
    else {
        if save.checkpoint_id.is_some() {
            return Err("E_SAVE_SCENE_STATE_MISSING".into());
        }
        return Ok(runtime);
    };
    let definition = runtime.current_scene().clone();
    for (index, id) in saved.activated_ids.iter().enumerate() {
        if let Some(interaction) = definition.interactions.iter().find(|value| value.id == *id) {
            runtime
                .interact(
                    id,
                    &format!("restore-interaction-{index}"),
                    epoch,
                    vec3_from_array(interaction.position),
                )
                .map_err(|_| "E_SAVE_SCENE_STATE_INVALID")?;
        } else if let Some(trigger) = definition.triggers.iter().find(|value| value.id == *id) {
            let (x, z) = polygon_interior(&trigger.polygon).ok_or("E_SAVE_SCENE_STATE_INVALID")?;
            runtime
                .trigger(
                    id,
                    &format!("restore-trigger-{index}"),
                    epoch,
                    Vec3 {
                        x_m: x,
                        y_m: trigger.height_range_m.map_or(0.0,|range|range.midpoint()),
                        z_m: z,
                    },
                )
                .map_err(|_| "E_SAVE_SCENE_STATE_INVALID")?;
        } else {
            return Err("E_SAVE_SCENE_STATE_UNKNOWN_ID".into());
        }
    }
    let actual_events: BTreeSet<_> = definition
        .emitted_event_ids()
        .into_iter()
        .filter(|event| runtime.event_complete(event))
        .collect();
    let expected_events: BTreeSet<_> = saved.emitted_event_ids.iter().cloned().collect();
    if actual_events != expected_events {
        return Err("E_SAVE_SCENE_EVENT_MISMATCH".into());
    }
    if let Some(checkpoint) = &saved.checkpoint_id {
        let cp = definition
            .checkpoints
            .iter()
            .find(|value| value.id == *checkpoint)
            .ok_or("E_SAVE_CHECKPOINT_UNKNOWN")?;
        runtime
            .checkpoint(
                checkpoint,
                "restore-checkpoint",
                epoch,
                vec3_from_array(cp.position),
            )
            .map_err(|_| "E_SAVE_CHECKPOINT_INVALID")?;
    }
    if runtime.checkpoint_id != save.checkpoint_id {
        return Err("E_SAVE_CHECKPOINT_MISMATCH".into());
    }
    Ok(runtime)
}

fn polygon_interior(polygon: &[[f32; 2]]) -> Option<(f32, f32)> {
    if polygon.len() < 3 {
        return None;
    }
    let min_x = polygon
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = polygon
        .iter()
        .map(|point| point[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_z = polygon
        .iter()
        .map(|point| point[1])
        .fold(f32::INFINITY, f32::min);
    let max_z = polygon
        .iter()
        .map(|point| point[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let contains = |x: f32, z: f32| {
        let mut inside = false;
        let mut previous = polygon.len() - 1;
        for index in 0..polygon.len() {
            let (xi, zi) = (polygon[index][0], polygon[index][1]);
            let (xj, zj) = (polygon[previous][0], polygon[previous][1]);
            if (zi > z) != (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi {
                inside = !inside;
            }
            previous = index;
        }
        inside
    };
    for row in 1..20 {
        for column in 1..20 {
            let x = min_x + (max_x - min_x) * column as f32 / 20.0;
            let z = min_z + (max_z - min_z) * row as f32 / 20.0;
            if contains(x, z) {
                return Some((x, z));
            }
        }
    }
    None
}

fn apply_scene_progress(
    state: &mut RuntimeState,
    scene: &SceneRuntime,
    events: &[SceneEvent],
    request_id: &str,
) -> Result<(), String> {
    grey_hive_beacon::validate_extraction(state, scene, events)?;
    clockworks_core::validate_core_event(state, scene, events)?;
    clockworks_controls::validate_pressure_event(state, scene, events)?;
    if clockworks_campaign::apply_shutdown(state, scene, events, request_id)? {
        return Ok(());
    }
    let event_id = events.iter().find_map(|event| match event {
        SceneEvent::Interaction { event_id, .. } => event_id.as_deref(),
        SceneEvent::Trigger { event_id, .. } => Some(event_id.as_str()),
        _ => None,
    });
    let Some(event_id) = event_id else {
        return Ok(());
    };
    if clockworks_controls::reconfirming_completed_event(state, event_id) {
        return Ok(());
    }
    let definition = scene.current_scene();
    let world_id = &definition.world_id;
    let clockworks_shutdown = event_id == "clockworks_shutdown";
    let official_clockworks_shutdown = clockworks_shutdown
        && world_id == "clockworks"
        && definition.scene_id == "cw_shutdown_exit";
    if clockworks_shutdown
        && (!official_clockworks_shutdown
            || state.world.world_id != "clockworks"
            || state.world.scene_id != "cw_shutdown_exit"
            || state.route.current_world_id != "clockworks"
            || !definition.interactions.iter().any(|interaction| {
                interaction.id == "cw_master_shutdown_staged"
                    && interaction.kind == "terminal"
                    && interaction.event.as_deref() == Some("clockworks_shutdown")
            })
            || !events.iter().any(|event| {
                matches!(event,
                    SceneEvent::Interaction { id, event_id: Some(emitted) }
                        if id == "cw_master_shutdown_staged" && emitted == "clockworks_shutdown")
            }))
    {
        return Err("E_CLOCKWORKS_SHUTDOWN_SOURCE_INVALID".into());
    }
    if official_clockworks_shutdown {
        let prerequisites = completed_world_events(state, "clockworks");
        if !prerequisites.contains("clockworks_valves")
            || !prerequisites.contains("clockworks_core")
        {
            return Err("E_CLOCKWORKS_SHUTDOWN_PREREQUISITES_MISSING".into());
        }
        if prerequisites.contains("clockworks_shutdown") {
            return Err("E_CLOCKWORKS_SHUTDOWN_DUPLICATE_EVENT".into());
        }
    }
    if !scene.is_required_event(world_id, event_id) {
        return Ok(());
    }
    let official_tower_signal = event_id == "mist_signal"
        && world_id == "mist_harbor"
        && definition.scene_id == "mh_resonance_tower";
    if official_tower_signal
        && (state.world.world_id != "mist_harbor"
            || state.world.scene_id != "mh_resonance_tower"
            || state.route.current_world_id != "mist_harbor"
            || !events.iter().any(|event| {
                matches!(event,
                SceneEvent::Interaction { id, event_id: Some(id_event) }
                    if id == "mh_signal_console_staged" && id_event == "mist_signal")
            }))
    {
        return Err("E_ACOUSTIC_MAPPING_SOURCE_INVALID".into());
    }
    let grants_acoustic_mapping = official_tower_signal;
    let grants_air_step = official_clockworks_shutdown;
    if grants_acoustic_mapping && route_completed_events(&state.route, world_id).contains(event_id)
    {
        return Err("E_ACOUSTIC_MAPPING_DUPLICATE_EVENT".into());
    }
    let mut route = state.route.clone();
    let completes_grey_hive =
        world_id == crate::world_progression::WORLD_GREY_HIVE && event_id == "hive_extraction";
    if completes_grey_hive {
        let prerequisites = ["hive_power", "hive_lockdown"];
        let has_prerequisites = route
            .progress
            .iter()
            .find(|progress| progress.world_id.as_str() == world_id.as_str())
            .is_some_and(|progress| {
                prerequisites.iter().all(|required| {
                    progress
                        .completed_events
                        .iter()
                        .any(|actual| actual.as_str() == *required)
                })
            });
        if !has_prerequisites {
            return Err("E_SCENE_PROGRESS_PREREQUISITES_MISSING".into());
        }
    }
    let reconfirming_old_extraction = completes_grey_hive && scene.is_complete_clockworks_production()
        && route_completed_events(&route, "grey_hive").contains("hive_extraction");
    if !reconfirming_old_extraction {
    match apply_route_command(
        &mut route,
        RouteCommand::Progress {
            event_id: event_id.into(),
            request_id: format!("scene-event-{request_id}"),
        },
        state.world.revision,
    ) {
        RouteResult::Applied { events, .. }
            if events.iter().any(|event| {
                matches!(
                    event,
                    crate::world_progression::RouteEvent::DuplicateIgnored { .. }
                )
            }) =>
        {
            return Err("E_SCENE_PROGRESS_DUPLICATE".into());
        }
        RouteResult::Applied { .. } => {}
        RouteResult::Rejected { code, message } => {
            return Err(format!("E_SCENE_PROGRESS: {code:?}: {message}"));
        }
    }
    }
    if completes_grey_hive {
        match apply_route_command(
            &mut route,
            RouteCommand::Complete {
                world_id: world_id.clone(),
                request_id: format!("scene-complete-{request_id}"),
            },
            state.world.revision,
        ) {
            RouteResult::Applied { events, .. }
                if events.iter().any(|event| {
                    matches!(
                        event,
                        crate::world_progression::RouteEvent::DuplicateIgnored { .. }
                    )
                }) =>
            {
                return Err("E_SCENE_COMPLETE_DUPLICATE".into());
            }
            RouteResult::Applied { ref events, .. } if scene.is_complete_clockworks_production()
                && !matches!(events.as_slice(), [crate::world_progression::RouteEvent::Completed { world_id, first_completion: true }] if world_id == "grey_hive") => {
                return Err("E_SCENE_COMPLETE_RECEIPT_INVALID".into());
            }
            RouteResult::Applied { .. } => {}
            RouteResult::Rejected { code, message } => {
                return Err(format!("E_SCENE_COMPLETE: {code:?}: {message}"));
            }
        }
    }
    let mut capabilities = state.capabilities.clone();
    if grants_acoustic_mapping {
        if capabilities
            .grants
            .iter()
            .any(|grant| grant.capability_id == CAP_ACOUSTIC_MAPPING)
        {
            return Err("E_ACOUSTIC_MAPPING_DUPLICATE".into());
        }
        apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Grant {
                capability_id: CAP_ACOUSTIC_MAPPING.into(),
            },
            state.world.revision,
        )
        .map_err(|error| format!("E_ACOUSTIC_MAPPING_GRANT: {error:?}"))?;
        let mut selected = capabilities.selected.clone();
        selected.push(CAP_ACOUSTIC_MAPPING.into());
        apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Select {
                capability_ids: selected,
            },
            state.world.revision,
        )
        .map_err(|error| format!("E_ACOUSTIC_MAPPING_SELECT: {error:?}"))?;
    }
    if grants_air_step {
        if capabilities
            .grants
            .iter()
            .any(|grant| grant.capability_id == CAP_AIR_STEP)
        {
            return Err("E_CLOCKWORKS_AIR_STEP_DUPLICATE_GRANT".into());
        }
        apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Grant {
                capability_id: CAP_AIR_STEP.into(),
            },
            state.world.revision,
        )
        .map_err(|error| format!("E_CLOCKWORKS_AIR_STEP_GRANT: {error:?}"))?;
        let mut selected = capabilities.selected.clone();
        selected.push(CAP_AIR_STEP.into());
        apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Select {
                capability_ids: selected,
            },
            state.world.revision,
        )
        .map_err(|error| format!("E_CLOCKWORKS_AIR_STEP_SELECT: {error:?}"))?;
    }
    state.install_capability_state(state.world.clone(), capabilities)?;
    state.route = route;
    Ok(())
}

fn emit_scene_presentation(state: &mut RuntimeState, event: &SceneEvent) {
    let (kind, position, intensity) = match event {
        SceneEvent::Interaction { .. } => ("SceneInteraction", state.world.player.position_m, 1.0),
        SceneEvent::Trigger { .. } => ("SceneTrigger", state.world.player.position_m, 1.0),
        SceneEvent::Checkpoint { .. } => {
            ("CheckpointActivated", state.world.player.position_m, 1.0)
        }
        SceneEvent::Transition { .. } => ("SceneTransition", state.world.player.position_m, 1.0),
    };
    let _ = event;
    emit_presentation(state, kind, position, intensity);
}

fn has_progress(state: &RuntimeState, event_id: &str) -> bool {
    state
        .route
        .progress
        .iter()
        .find(|progress| progress.world_id == state.world.world_id)
        .is_some_and(|progress| {
            progress
                .completed_events
                .iter()
                .any(|event| event == event_id)
        })
}

fn completed_world_events(
    state: &RuntimeState,
    world_id: &str,
) -> std::collections::BTreeSet<String> {
    route_completed_events(&state.route, world_id)
}

fn refresh_scene_kcc(state: &mut RuntimeState, scene: &SceneRuntime) -> Result<(), String> {
    let definition = scene.current_scene();
    let events = completed_world_events(state, &definition.world_id);
    state.kcc = scene_kcc_for_state(definition, &events, state)?;
    Ok(())
}

fn route_completed_events(
    route: &RouteState,
    world_id: &str,
) -> std::collections::BTreeSet<String> {
    route
        .progress
        .iter()
        .find(|progress| progress.world_id == world_id)
        .map(|progress| progress.completed_events.iter().cloned().collect())
        .unwrap_or_default()
}

fn rebase_capability_revision_epoch(state: &mut CapabilityState, epoch: u64) -> Result<(), String> {
    let mut value = serde_json::to_value(&*state).map_err(|_| "E_CAPABILITY_REVISION_INVALID")?;
    let object = value
        .as_object_mut()
        .ok_or("E_CAPABILITY_REVISION_INVALID")?;
    for key in ["currentRevision", "lastTickRevision"] {
        if let Some(revision) = object.get_mut(key).filter(|revision| !revision.is_null()) {
            revision["worldEpoch"] = serde_json::json!(epoch);
        }
    }
    *state = serde_json::from_value(value).map_err(|_| "E_CAPABILITY_REVISION_INVALID")?;
    Ok(())
}

fn interaction_rejected(
    state: &RuntimeState,
    scene: Option<&SceneRuntime>,
    code: &str,
) -> FormalInteractionResponse {
    let view = project_scene_view(state, scene);
    FormalInteractionResponse {
        applied: false,
        already_applied: false,
        error_code: Some(code.into()),
        events: vec![],
        receipt: CommandReceipt::outcome(
            "interaction-rejected",
            false,
            false,
            Some(code.into()),
            view.clone(),
        ),
        view,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_support_capture_enforces_airborne_envelope_and_preserves_falling_velocity() {
        let mut state=FormalRuntime::initial_state(1).unwrap();
        let support=crate::moving_support::MovingSupportDefinition {id:"capture_lift".into(),
            polygon:vec![[1.,1.],[9.,1.],[9.,9.],[1.,9.]],lower_m:0.,upper_m:2.,travel_ms:1000,endpoint_hold_ms:250};
        state.kcc=crate::continuous_kcc::StaticKccWorld::new(
            crate::continuous_kcc::Aabb::new(0.,10.,0.,10.).unwrap(),vec![])
            .with_moving_supports(vec![support]).unwrap();
        state.world.player.grounded=false;
        state.world.player.velocity_mps.y_m=-2.;
        for y in [-0.1,3.0001,99.] {
            state.world.player.position_m=Vec3::new(5.,y,5.).unwrap();let before=state.world.player.clone();
            assert_eq!(crate::save_v5::SaveV5::capture(&state,None).unwrap_err(),"E_SAVE_SUPPORT_CONTACT_INVALID");
            assert_eq!(state.world.player,before);
        }
        for y in [0.,1.5,3.] {
            state.world.player.position_m=Vec3::new(5.,y,5.).unwrap();
            let saved=crate::save_v5::SaveV5::capture(&state,None).unwrap();
            assert_eq!(saved.player.position_m.y_m,y);assert_eq!(saved.player.velocity_mps.y_m,-2.);
        }
    }

    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    const ACTION_TEST_RS: &str =
        include_str!("../tests/fixtures/scene-runtime-v1/return_station.json");
    const ACTION_TEST_GH: &str = include_str!("../tests/fixtures/scene-runtime-v1/grey_hive.json");
    const ACTION_TEST_MH: &str =
        include_str!("../tests/fixtures/scene-runtime-v1/mist_harbor.json");
    const ACTION_TEST_CW: &str = include_str!("../tests/fixtures/scene-runtime-v1/clockworks.json");
    const CW_SHUTDOWN_ENTRY: &str =
        include_str!("../../content/scenes/compiled/cw_entry_foundry.json");
    const CW_SHUTDOWN_PRESSURE: &str =
        include_str!("../../content/scenes/compiled/cw_pressure_hall.json");
    const CW_SHUTDOWN_BRIDGE: &str =
        include_str!("../../content/scenes/compiled/cw_conveyor_bridge.json");
    const CW_SHUTDOWN_BOILER: &str =
        include_str!("../../content/scenes/compiled/cw_boiler_chamber.json");
    const CW_SHUTDOWN_SHAFT: &str =
        include_str!("../../content/scenes/compiled/cw_gear_shaft.json");
    const CW_SHUTDOWN_FURNACE: &str =
        include_str!("../../content/scenes/compiled/cw_furnace_heart.json");
    const CW_SHUTDOWN_ARENA: &str =
        include_str!("../../content/scenes/compiled/cw_forged_guard_arena.json");
    const CW_SHUTDOWN_CORE: &str =
        include_str!("../../content/scenes/compiled/cw_regulator_core.json");
    const CW_SHUTDOWN_EXIT: &str =
        include_str!("../../content/scenes/compiled/cw_shutdown_exit.json");
    const SENTINEL_ARENA_SCENE: &str =
        include_str!("../../content/scenes/compiled/gh_sentinel_arena.json");
    const SENTINEL_ARENA_TMJ: &str =
        include_str!("../../design/maps/grey_hive/gh_sentinel_arena.tmj");

    fn grant_test_capability(state: &mut RuntimeState, id: &str) {
        let mut capabilities = state.capabilities.clone();
        let effects = apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Grant {
                capability_id: id.into(),
            },
            state.world.revision,
        )
        .unwrap();
        let mut world = state.world.clone();
        apply_effects_atomically(&mut world, &effects).unwrap();
        state.install_capability_state(world, capabilities).unwrap();
    }

    fn grant_enemy_vitals(state: &mut RuntimeState) {
        grant_test_capability(state, CAP_ENEMY_VITALS);
    }

    fn runtime_for_action_scene(scene_id: &str) -> FormalRuntime {
        let runtime = FormalRuntime::new().unwrap();
        runtime
            .load_scene_registry(
                [
                    ACTION_TEST_RS,
                    ACTION_TEST_GH,
                    ACTION_TEST_MH,
                    ACTION_TEST_CW,
                ],
                scene_id,
                &BTreeSet::new(),
                &BTreeSet::new(),
            )
            .unwrap();
        runtime
    }

    pub(super) fn runtime_for_clockworks_shutdown(scene_id: &str) -> FormalRuntime {
        let runtime = FormalRuntime::new().unwrap();
        let clockworks = [
            CW_SHUTDOWN_ENTRY,
            CW_SHUTDOWN_PRESSURE,
            CW_SHUTDOWN_BRIDGE,
            CW_SHUTDOWN_BOILER,
            CW_SHUTDOWN_SHAFT,
            CW_SHUTDOWN_FURNACE,
            CW_SHUTDOWN_ARENA,
            CW_SHUTDOWN_CORE,
            CW_SHUTDOWN_EXIT,
        ]
        .into_iter()
        .map(|raw| {
            let mut scene: serde_json::Value = serde_json::from_str(raw).unwrap();
            scene["presentation"] = serde_json::json!({
                "cameraProfile": "oblique_default",
                "layers": [],
                "sprites": [],
            });
            scene["spawns"] = serde_json::Value::Array(
                scene["spawns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|spawn| spawn["kind"] == "player")
                    .cloned()
                    .collect(),
            );
            scene["collision"] = serde_json::Value::Array(
                scene["collision"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|collision| collision["requiresActorFirstKill"].is_null())
                    .cloned()
                    .collect(),
            );
            scene["doors"] = serde_json::json!([]);
            scene["vfxMarkers"] = serde_json::json!([]);
            serde_json::to_string(&scene).unwrap()
        })
        .collect::<Vec<_>>();
        let mut documents = vec![
            ACTION_TEST_RS.to_owned(),
            ACTION_TEST_GH.to_owned(),
            ACTION_TEST_MH.to_owned(),
            ACTION_TEST_CW.to_owned(),
        ];
        documents.extend(clockworks);
        runtime
            .load_scene_registry(documents, scene_id, &BTreeSet::new(), &BTreeSet::new())
            .unwrap();
        runtime.pause().unwrap();
        runtime.stop_owner.store(true, Ordering::SeqCst);
        runtime
            .owner_handle
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .join()
            .unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.paused = false;
            state.route.current_world_id = "clockworks".into();
            state.world.player.position_m = Vec3::new(8.0, 0.0, 6.0).unwrap();
        }
        runtime
    }

    fn seed_clockworks_prerequisites(runtime: &FormalRuntime, events: &[&str]) {
        let mut state = runtime.state.lock().unwrap();
        let revision = state.world.revision;
        for (index, event_id) in events.iter().enumerate() {
            let result = apply_route_command(
                &mut state.route,
                RouteCommand::Progress {
                    event_id: (*event_id).into(),
                    request_id: format!("shutdown-test-prerequisite-{index}"),
                },
                revision,
            );
            assert!(matches!(result, RouteResult::Applied { .. }));
        }
    }

    fn assert_clockworks_shutdown_uncommitted(runtime: &FormalRuntime) {
        let state = runtime.state.lock().unwrap();
        assert!(!route_completed_events(&state.route, "clockworks").contains("clockworks_shutdown"));
        assert!(!state
            .capabilities
            .grants
            .iter()
            .any(|grant| grant.capability_id == CAP_AIR_STEP));
        assert!(!state
            .capabilities
            .selected
            .iter()
            .any(|capability| capability == CAP_AIR_STEP));
        drop(state);
        let scene = runtime.scene_runtime.lock().unwrap();
        let scene = scene.as_ref().unwrap();
        assert!(!scene.object_activated("cw_master_shutdown_staged"));
        assert!(!scene.event_complete("clockworks_shutdown"));
    }

    #[test]
    fn official_clockworks_shutdown_terminal_atomically_grants_and_selects_air_step() {
        let runtime = runtime_for_clockworks_shutdown("cw_shutdown_exit");
        seed_clockworks_prerequisites(&runtime, &["clockworks_valves", "clockworks_core"]);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let response = runtime
            .activate_scene_interaction("cw_master_shutdown_staged", "shutdown-valid", epoch)
            .unwrap();
        assert!(response.applied, "{:?}", response.error_code);
        assert!(response
            .events
            .iter()
            .any(|event| event.contains("clockworks_shutdown")));
        {
            let state = runtime.state.lock().unwrap();
            let completed = route_completed_events(&state.route, "clockworks");
            assert!(completed.contains("clockworks_shutdown"));
            assert!(completed.contains("clockworks_valves"));
            assert!(completed.contains("clockworks_core"));
            assert!(state
                .capabilities
                .grants
                .iter()
                .any(|grant| grant.capability_id == CAP_AIR_STEP));
            assert!(state
                .capabilities
                .selected
                .iter()
                .any(|capability| capability == CAP_AIR_STEP));
        }

        runtime.save().unwrap();
        runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        {
            let state = runtime.state.lock().unwrap();
            assert!(
                route_completed_events(&state.route, "clockworks").contains("clockworks_shutdown")
            );
            assert!(state
                .capabilities
                .grants
                .iter()
                .any(|grant| grant.capability_id == CAP_AIR_STEP));
            assert!(state
                .capabilities
                .selected
                .iter()
                .any(|capability| capability == CAP_AIR_STEP));
        }

        let before_replay = runtime.snapshot().unwrap();
        let replay = runtime
            .activate_scene_interaction(
                "cw_master_shutdown_staged",
                "shutdown-replay",
                before_replay.world_epoch,
            )
            .unwrap();
        assert!(!replay.applied);
        assert!(replay.error_code.is_some());
        let after_replay = runtime.snapshot().unwrap();
        assert_eq!(
            after_replay.authority_revision,
            before_replay.authority_revision
        );
        assert!(after_replay
            .capabilities
            .items
            .iter()
            .any(|item| item.capability_id == CAP_AIR_STEP && item.granted && item.selected));
    }

    #[test]
    fn shutdown_rejections_leave_route_capability_and_scene_ledger_unchanged() {
        for prerequisites in [
            &[][..],
            &["clockworks_valves"][..],
            &["clockworks_core"][..],
        ] {
            let runtime = runtime_for_clockworks_shutdown("cw_shutdown_exit");
            seed_clockworks_prerequisites(&runtime, prerequisites);
            let epoch = runtime.snapshot().unwrap().world_epoch;
            let response = runtime
                .activate_scene_interaction(
                    "cw_master_shutdown_staged",
                    "shutdown-missing-prereq",
                    epoch,
                )
                .unwrap();
            assert!(!response.applied);
            assert_eq!(
                response.error_code.as_deref(),
                Some("E_CLOCKWORKS_SHUTDOWN_PREREQUISITES_MISSING")
            );
            assert_clockworks_shutdown_uncommitted(&runtime);
        }

        let runtime = runtime_for_clockworks_shutdown("cw_shutdown_exit");
        seed_clockworks_prerequisites(&runtime, &["clockworks_valves", "clockworks_core"]);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let staged_marker = runtime
            .activate_scene_interaction(
                "cw_air_step_grant_staged",
                "shutdown-other-air-step-marker",
                epoch,
            )
            .unwrap();
        assert_eq!(
            staged_marker.error_code.as_deref(),
            Some("E_CLOCKWORKS_AIR_STEP_MARKER_STAGED")
        );
        assert_clockworks_shutdown_uncommitted(&runtime);

        let runtime = runtime_for_clockworks_shutdown("cw_regulator_core");
        seed_clockworks_prerequisites(&runtime, &["clockworks_valves", "clockworks_core"]);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let wrong_scene = runtime
            .activate_scene_interaction("cw_master_shutdown_staged", "shutdown-wrong-scene", epoch)
            .unwrap();
        assert!(!wrong_scene.applied);
        assert_clockworks_shutdown_uncommitted(&runtime);

        for invalid_position in [false, true] {
            let runtime = runtime_for_clockworks_shutdown("cw_shutdown_exit");
            seed_clockworks_prerequisites(&runtime, &["clockworks_valves", "clockworks_core"]);
            let epoch = runtime.snapshot().unwrap().world_epoch;
            if invalid_position {
                runtime.state.lock().unwrap().world.player.position_m =
                    Vec3::new(80.0, 0.0, 80.0).unwrap();
            }
            let response = runtime
                .activate_scene_interaction(
                    "cw_master_shutdown_staged",
                    if invalid_position {
                        "shutdown-out-of-range"
                    } else {
                        "shutdown-stale-epoch"
                    },
                    if invalid_position { epoch } else { epoch + 1 },
                )
                .unwrap();
            assert!(!response.applied);
            assert_clockworks_shutdown_uncommitted(&runtime);
        }

        let runtime = runtime_for_clockworks_shutdown("cw_shutdown_exit");
        seed_clockworks_prerequisites(&runtime, &["clockworks_valves", "clockworks_core"]);
        runtime.state.lock().unwrap().route.current_world_id = "grey_hive".into();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let wrong_route = runtime
            .activate_scene_interaction("cw_master_shutdown_staged", "shutdown-wrong-route", epoch)
            .unwrap();
        assert_eq!(
            wrong_route.error_code.as_deref(),
            Some("E_CLOCKWORKS_SHUTDOWN_SOURCE_INVALID")
        );
        assert_clockworks_shutdown_uncommitted(&runtime);
    }

    fn runtime_for_sentinel_beacon_gate() -> FormalRuntime {
        let save_root = std::env::temp_dir().join(format!(
            "formal-sentinel-beacon-gate-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, SENTINEL_SCENE_ID)
            .unwrap();

        runtime.pause().unwrap();
        runtime.stop_owner.store(true, Ordering::SeqCst);
        runtime
            .owner_handle
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .join()
            .unwrap();
        runtime.state.lock().unwrap().paused = false;
        runtime
    }

    fn kill_registered_sentinel_for_beacon_test(runtime: &FormalRuntime) {
        let scene = runtime.scene_runtime.lock().unwrap();
        let mut state = runtime.state.lock().unwrap();
        let epoch = state.world.revision.world_epoch;
        state.world.player.position_m = Vec3::new(5.0, 0.0, 5.0).unwrap();
        assert_eq!(state.world.sentinels.len(), 1);
        assert_eq!(state.world.sentinels[0].entity_id, SENTINEL_SPAWN_ID);
        state.world.sentinels[0].position_m = Vec3::new(5.8, 0.0, 5.0).unwrap();
        state.world.sentinels[0].hp = 1;
        state.world.sentinels[0].active = true;
        state.world.sentinels[0].state = crate::sentinel_ai::SentinelState::Chase;
        state.world.sentinels[0].state_remaining_ms = 0;
        state.stepper = FixedStepClock::new(state.stepper.config);
        state.latest_sample = InputSample::new(epoch, 1, 16, 0.0, 0.0)
            .unwrap()
            .with_aim(1.0, 0.0)
            .unwrap();
        state.latest_input_received_at = Instant::now();
        state
            .pending_combat
            .push(crate::continuous_combat::CombatIntent::Attack { request_id: 1 });

        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert_eq!(state.world.sentinels[0].hp, 0);
        assert!(!state.world.sentinels[0].active);
        assert!(state.world_persistent_v1.grey_hive.sentinel_first_kill);
    }

    fn set_test_player_position(runtime: &FormalRuntime, position: Vec3) {
        runtime.state.lock().unwrap().world.player.position_m = position;
    }

    fn runtime_for_clockworks_elite_gate() -> FormalRuntime {
        let runtime = FormalRuntime::new_with_save_dir(std::env::temp_dir().join(format!(
            "formal-cw-elite-gate-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )))
        .unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, CLOCKWORKS_ELITE_SCENE_ID)
            .unwrap();
        runtime
    }

    fn runtime_for_clockworks_regulator_gate() -> FormalRuntime {
        let runtime = FormalRuntime::new_with_save_dir(std::env::temp_dir().join(format!(
            "formal-cw-regulator-gate-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )))
        .unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, CLOCKWORKS_REGULATOR_SCENE_ID)
            .unwrap();
        runtime
    }

    fn stop_regulator_test_owner(runtime: &FormalRuntime) {
        runtime.pause().unwrap();
        runtime.stop_owner.store(true, Ordering::SeqCst);
        runtime
            .owner_handle
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .join()
            .unwrap();
        runtime.state.lock().unwrap().paused = false;
    }

    #[test]
    fn regulator_phase3_motion_cooling_rules_pause_and_save_are_authoritative() {
        let runtime = runtime_for_clockworks_regulator_gate();
        stop_regulator_test_owner(&runtime);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let scene = scene_guard.as_ref().unwrap();
        let mut state = runtime.state.lock().unwrap();
        state.world.server_time_ms = 100;
        state.world.generic_actors[0].hp = 181;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert!(!state.world_persistent_v1.clockworks.regulator_phase3_active);
        state.world.generic_actors[0].hp = 180;
        state.world.player.position_m = Vec3::new(9.0, 0.0, 12.0).unwrap();
        advance_regulator_environment(&mut state, scene).unwrap();
        assert!(state.world_persistent_v1.clockworks.regulator_phase3_active);
        assert_eq!(
            state
                .world_persistent_v1
                .clockworks
                .regulator_phase3_started_at_ms,
            100
        );
        let view = project_scene_view(&state, Some(scene));
        let press = view
            .hazards
            .iter()
            .find(|h| h.kind == "moving_machinery")
            .unwrap();
        assert!(press.active);
        assert_eq!(press.warning_remaining_ms, Some(900));
        assert_eq!(state.world.player_hp, 100);
        state
            .world_persistent_v1
            .clockworks
            .regulator_heat_cooled_until_ms = 20_000;
        state.world.server_time_ms = 1_000;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(
            state.world.player_hp, 84,
            "coolant does not disable machinery"
        );
        state.world.server_time_ms = 1_100;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(
            state.world.player_hp, 84,
            "per-hazard cooldown prevents every-tick damage"
        );
        state.world.server_time_ms = 1_750;
        state.world.player.position_m = Vec3::new(10.5, 0.0, 12.0).unwrap();
        state
            .effective_rules_v6
            .hazards
            .remaining_damage_bps
            .insert(crate::effects::HazardTag::Machinery, 5_000);
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(state.world.player_hp, 76);
        state.world.server_time_ms = 2_500;
        state.world.player.position_m = Vec3::new(12.0, 0.0, 12.0).unwrap();
        state
            .effective_rules_v6
            .hazards
            .immune
            .insert(crate::effects::HazardTag::Machinery);
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(state.world.player_hp, 76);
        // Pause freezes authority time, geometry, and all phase clocks.
        state.paused = true;
        let paused = project_scene_view(&state, Some(scene));
        let clocks = state.world_persistent_v1.clockworks.clone();
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(state.world.server_time_ms, 2_500);
        assert_eq!(state.world_persistent_v1.clockworks, clocks);
        assert_eq!(
            serde_json::to_value(project_scene_view(&state, Some(scene)).hazards).unwrap(),
            serde_json::to_value(&paused.hazards).unwrap()
        );
        // Save V6 retains motion origin and cooldown map; Continue derives identical geometry.
        state.effective_rules_v6 = Default::default();
        state.route.current_world_id = "clockworks".into();
        let save = crate::save_v6::SaveV6::capture(&state, Some(scene)).unwrap();
        let restored = save.restore_state().unwrap();
        let mut continued_scene = scene.clone();
        continued_scene.world_epoch = restored.world.revision.world_epoch;
        assert_eq!(restored.world_persistent_v1.clockworks, clocks);
        assert_eq!(
            serde_json::to_value(project_scene_view(&restored, Some(&continued_scene)).hazards)
                .unwrap(),
            serde_json::to_value(&paused.hazards).unwrap()
        );
        state.world.generic_actors[0].hp = 0;
        state.paused = false;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(
            state.world_persistent_v1.clockworks, clocks,
            "death stops hazard clocks"
        );
        assert!(project_scene_view(&state, Some(scene))
            .hazards
            .iter()
            .filter(|h| h.phase_active.is_some())
            .all(|h| !h.active && h.phase_active == Some(false)));
    }

    #[test]
    fn regulator_phase3_rejects_wrong_identity_and_keeps_ground_lane_clear() {
        let runtime = runtime_for_clockworks_regulator_gate();
        stop_regulator_test_owner(&runtime);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let scene = scene_guard.as_ref().unwrap();
        let mut state = runtime.state.lock().unwrap();
        state.world.generic_actors[0].hp = 1;
        state.world.generic_actors[0].entity_id = "forged_regulator".into();
        advance_regulator_environment(&mut state, scene).unwrap();
        assert!(!state.world_persistent_v1.clockworks.regulator_phase3_active);
        state.world.generic_actors[0].entity_id = CLOCKWORKS_REGULATOR_SPAWN_ID.into();
        state.world.scene_id = "cw_furnace_heart".into();
        advance_regulator_environment(&mut state, scene).unwrap();
        assert!(!state.world_persistent_v1.clockworks.regulator_phase3_active);
        state.world.scene_id = CLOCKWORKS_REGULATOR_SCENE_ID.into();
        let mut stale_scene = scene.clone();
        stale_scene.world_epoch += 1;
        advance_regulator_environment(&mut state, &stale_scene).unwrap();
        assert!(!state.world_persistent_v1.clockworks.regulator_phase3_active);
        state.world.player.position_m = Vec3::new(10.0, 0.0, 8.0).unwrap();
        for time in (0..16_000).step_by(100) {
            state.world.server_time_ms = time;
            advance_regulator_environment(&mut state, scene).unwrap();
            let view = project_scene_view(&state, Some(scene));
            for hazard in view.hazards.iter().filter(|h| h.phase_active == Some(true)) {
                assert!(!point_in_or_on_polygon(
                    10.0,
                    8.0,
                    hazard.polygon_m.as_ref().unwrap()
                ));
            }
        }
        assert_eq!(
            state.world.player_hp, 100,
            "ordinary Ground has a safe main lane"
        );
    }

    #[test]
    fn regulator_phase2_heat_threshold_warning_damage_and_generic_rules_are_authoritative() {
        let runtime = runtime_for_clockworks_regulator_gate();
        stop_regulator_test_owner(&runtime);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let scene = scene_guard.as_ref().unwrap();
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = Vec3::new(4.0, 0.0, 8.0).unwrap();
        state.world.generic_actors[0].hp = 361;
        state.world.server_time_ms = 100;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert!(!state.world_persistent_v1.clockworks.regulator_phase2_active);

        state.world.generic_actors[0].hp = 360;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert!(state.world_persistent_v1.clockworks.regulator_phase2_active);
        assert_eq!(
            state
                .world_persistent_v1
                .clockworks
                .regulator_heat_warning_until_ms,
            700
        );
        assert_eq!(state.world.player_hp, 100);
        let view = project_scene_view(&state, Some(scene));
        let heat = view
            .hazards
            .iter()
            .find(|hazard| hazard.entity_id == "cw_regulator_furnace_heat_zone")
            .unwrap();
        assert_eq!(heat.phase_active, Some(true));
        assert_eq!(heat.warning_remaining_ms, Some(600));
        assert!(view.interactables.iter().any(|item| {
            item.entity_id == "cw_regulator_valve_furnace_link_staged" && item.active
        }));

        state.world.server_time_ms = 700;
        state.world.player.position_m = Vec3::new(18.0, 0.0, 5.0).unwrap();
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(state.world.player_hp, 88);

        state
            .effective_rules_v6
            .hazards
            .remaining_damage_bps
            .insert(crate::effects::HazardTag::Heat, 5_000);
        state.world.server_time_ms = 1_700;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(state.world.player_hp, 82);

        state
            .effective_rules_v6
            .hazards
            .immune
            .insert(crate::effects::HazardTag::Heat);
        state.world.server_time_ms = 2_700;
        advance_regulator_environment(&mut state, scene).unwrap();
        assert_eq!(state.world.player_hp, 82);

        state.effective_rules_v6 = Default::default();
        let mut durable_state = FormalRuntime::initial_state(1).unwrap();
        durable_state.world_persistent_v1.clockworks = state.world_persistent_v1.clockworks.clone();
        let save = crate::save_v6::SaveV6::capture(&durable_state, None).unwrap();
        let restored = save.restore_state().unwrap();
        assert_eq!(
            restored.world_persistent_v1.clockworks,
            state.world_persistent_v1.clockworks
        );
        assert!(!route_completed_events(&restored.route, "clockworks").contains("clockworks_core"));
    }

    #[test]
    fn regulator_save_continue_preserves_encounter_but_scene_revisit_starts_fresh_boss() {
        let runtime = runtime_for_clockworks_regulator_gate();
        stop_regulator_test_owner(&runtime);
        {
            let scene_guard = runtime.scene_runtime.lock().unwrap();
            let scene_runtime = scene_guard.as_ref().unwrap();
            let mut state = runtime.state.lock().unwrap();
            state.world.generic_actors[0].hp = 180;
            state.world.server_time_ms = 10_000;
            state.route.current_world_id = "clockworks".into();
            state.world.player.position_m = Vec3::new(18.0, 0.0, 5.0).unwrap();
            advance_regulator_environment(&mut state, scene_runtime).unwrap();
            assert!(state.world_persistent_v1.clockworks.regulator_phase2_active);
        }

        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(18.0, 0.0, 11.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "regulator-save-continue-cooling",
                epoch,
            )
            .unwrap();
        let (boss_hp, clockworks_state) = {
            let state = runtime.state.lock().unwrap();
            (
                state.world.generic_actors[0].hp,
                state.world_persistent_v1.clockworks.clone(),
            )
        };
        assert_eq!(boss_hp, 180);
        assert!(clockworks_state.regulator_phase2_active);
        assert!(clockworks_state.regulator_phase3_active);
        assert_eq!(clockworks_state.regulator_phase3_started_at_ms, 10_000);
        assert_eq!(clockworks_state.regulator_heat_warning_until_ms, 16_600);
        assert_eq!(clockworks_state.regulator_heat_next_damage_at_ms, 16_600);
        assert_eq!(clockworks_state.regulator_heat_cooled_until_ms, 16_000);
        assert_eq!(clockworks_state.regulator_valve_cooldown_until_ms, 18_000);

        // Continue restores both the saved actor and the encounter clocks. It must
        // not treat the saved Core scene as a newly spawned Boss.
        {
            let scene_guard = runtime.scene_runtime.lock().unwrap();
            let scene_runtime = scene_guard.as_ref().unwrap();
            let scene = scene_runtime.current_scene();
            let state = runtime.state.lock().unwrap();
            let save = crate::save_v6::SaveV6::capture(&state, scene_guard.as_ref()).unwrap();
            let mut continued = save.restore_state().unwrap();
            restore_generic_actors(&mut continued.world, scene, &mut continued.world_persistent_v1, false).unwrap();
            assert_eq!(continued.world.generic_actors[0].hp, boss_hp);
            assert_eq!(continued.world_persistent_v1.clockworks, clockworks_state);
            let mut continued_scene = scene_runtime.clone();
            continued_scene.world_epoch = continued.world.revision.world_epoch;
            let view = project_scene_view(&continued, Some(&continued_scene));
            let heat = view
                .hazards
                .iter()
                .find(|hazard| hazard.entity_id == "cw_regulator_furnace_heat_zone")
                .unwrap();
            assert!(heat.phase_active == Some(true));
            assert!(
                !heat.active,
                "the saved valve cooling window remains active"
            );
            assert_eq!(
                heat.warning_remaining_ms, None,
                "Continue must not expose the cooling window as a heat warning"
            );
            let valve = view
                .interactables
                .iter()
                .find(|item| item.entity_id == "cw_regulator_valve_furnace_link_staged")
                .unwrap();
            assert!(!valve.active, "the saved valve cooldown remains active");
        }

        // Return to the Arena, then create a new Core Boss instance.
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .clockworks
            .forged_guard_elite_first_kill = true;
        set_test_player_position(&runtime, Vec3::new(1.0, 0.0, 8.0).unwrap());
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        assert_eq!(
            runtime
                .transition_scene(
                    "cw_regulator_core_return_to_arena",
                    "regulator-phase2-return-arena",
                    epoch,
                ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
                .unwrap()
                .scene_id,
            CLOCKWORKS_ELITE_SCENE_ID
        );
        set_test_player_position(&runtime, Vec3::new(23.0, 0.0, 8.0).unwrap());
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        let revisited = runtime
            .transition_scene(
                "cw_arena_to_regulator_core",
                "regulator-phase2-revisit-core",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(revisited.scene_id, CLOCKWORKS_REGULATOR_SCENE_ID);
        assert_eq!(revisited.actors.len(), 1);
        let state = runtime.state.lock().unwrap();
        assert_eq!(state.world.generic_actors[0].hp, 600);
        let clockworks = &state.world_persistent_v1.clockworks;
        assert!(clockworks.forged_guard_elite_first_kill);
        assert!(!clockworks.regulator_phase2_active);
        assert!(!clockworks.regulator_phase3_active);
        assert_eq!(clockworks.regulator_phase3_started_at_ms, 0);
        assert!(clockworks.regulator_hazard_next_damage_at_ms.is_empty());
        assert!(!clockworks.regulator_defeated);
        assert_eq!(clockworks.regulator_heat_warning_until_ms, 0);
        assert_eq!(clockworks.regulator_heat_next_damage_at_ms, 0);
        assert_eq!(clockworks.regulator_heat_cooled_until_ms, 0);
        assert_eq!(clockworks.regulator_valve_cooldown_until_ms, 0);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let view = project_scene_view(&state, scene_guard.as_ref());
        let heat = view
            .hazards
            .iter()
            .find(|hazard| hazard.entity_id == "cw_regulator_furnace_heat_zone")
            .unwrap();
        assert_eq!(heat.phase_active, Some(false));
        assert!(!heat.active);
        let valve = view
            .interactables
            .iter()
            .find(|item| item.entity_id == "cw_regulator_valve_furnace_link_staged")
            .unwrap();
        assert!(
            !valve.active,
            "the valve stays phase-gated for the full-health Boss"
        );
        drop(scene_guard);
        drop(state);

        runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert_eq!(
            runtime.state.lock().unwrap().world_persistent_v1.clockworks,
            crate::world_persistent_v1::ClockworksPersistentState::default()
        );
    }

    #[test]
    fn regulator_heat_projection_separates_cooling_from_warning_and_damage() {
        let runtime = runtime_for_clockworks_regulator_gate();
        stop_regulator_test_owner(&runtime);
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.generic_actors[0].hp = 360;
            state.world.server_time_ms = 10_000;
            state.world.player.position_m = Vec3::new(18.0, 0.0, 11.0).unwrap();
            state.world_persistent_v1.clockworks.regulator_phase2_active = true;
        }
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let response = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-projection",
                epoch,
            )
            .unwrap();
        assert!(response.applied);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let scene = scene_guard.as_ref().unwrap();
        let mut state = runtime.state.lock().unwrap();
        for (now, active, warning) in [
            (10_000, false, None),
            (15_999, false, None),
            (16_000, true, Some(600)),
            (16_599, true, Some(1)),
            (16_600, true, None),
        ] {
            state.world.server_time_ms = now;
            let view = project_scene_view(&state, Some(scene));
            let heat = view
                .hazards
                .iter()
                .find(|hazard| hazard.entity_id == "cw_regulator_furnace_heat_zone")
                .unwrap();
            assert_eq!(heat.phase_active, Some(true));
            assert_eq!(heat.active, active, "heat activation at {now}");
            assert_eq!(heat.warning_remaining_ms, warning, "warning at {now}");
        }
        state.world.server_time_ms = 16_100;
        state.world.generic_actors[0].entity_id = "unregistered-regulator".into();
        let view = project_scene_view(&state, Some(scene));
        let heat = view
            .hazards
            .iter()
            .find(|hazard| hazard.entity_id == "cw_regulator_furnace_heat_zone")
            .unwrap();
        assert_eq!(heat.phase_active, Some(false));
        assert!(!heat.active);
        assert_eq!(heat.warning_remaining_ms, None);
    }

    #[test]
    fn regulator_valve_is_phase_gated_epoch_bound_repeatable_and_cooldown_limited() {
        let runtime = runtime_for_clockworks_regulator_gate();
        stop_regulator_test_owner(&runtime);
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.generic_actors[0].hp = 360;
            state.world.server_time_ms = 10_000;
            state.world.player.position_m = Vec3::new(18.0, 0.0, 11.0).unwrap();
        }
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let before_phase = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-before-phase",
                epoch,
            )
            .unwrap();
        assert_eq!(
            before_phase.error_code.as_deref(),
            Some("E_CW_VALVE_PHASE_INACTIVE")
        );
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .clockworks
            .regulator_phase2_active = true;
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(15.0, 0.0, 11.0).unwrap();
        let out_of_range = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-out-of-range",
                epoch,
            )
            .unwrap();
        assert_eq!(
            out_of_range.error_code.as_deref(),
            Some("E_SCENE_RUNTIME_OutOfRange")
        );
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(18.0, 0.0, 11.0).unwrap();
        let first = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-first",
                epoch,
            )
            .unwrap();
        assert!(first.applied, "{:?}", first.error_code);
        {
            let state = runtime.state.lock().unwrap();
            assert_eq!(
                state
                    .world_persistent_v1
                    .clockworks
                    .regulator_heat_cooled_until_ms,
                16_000
            );
            assert_eq!(
                state
                    .world_persistent_v1
                    .clockworks
                    .regulator_valve_cooldown_until_ms,
                18_000
            );
        }
        let repeated = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-repeat-early",
                epoch,
            )
            .unwrap();
        assert_eq!(repeated.error_code.as_deref(), Some("E_CW_VALVE_COOLDOWN"));
        let stale = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-stale",
                epoch + 1,
            )
            .unwrap();
        assert!(!stale.applied);
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.server_time_ms = 18_000;
        }
        let reused = runtime
            .activate_scene_interaction(
                "cw_regulator_valve_furnace_link_staged",
                "coolant-reused",
                epoch,
            )
            .unwrap();
        assert!(reused.applied, "{:?}", reused.error_code);
        assert!(
            !route_completed_events(&runtime.state.lock().unwrap().route, "clockworks")
                .contains("clockworks_core")
        );
    }

    #[test]
    fn regulator_heat_scene_schema_rejects_invalid_tuning_duplicate_ids_and_polygons() {
        let base: crate::scene_runtime::SceneDefinition =
            serde_json::from_str(CW_SHUTDOWN_CORE).unwrap();
        let assets = base
            .presentation
            .sprites
            .iter()
            .map(|sprite| sprite.asset_id.clone())
            .chain(base.presentation.background_asset.iter().cloned())
            .chain(base.doors.iter().map(|door| door.asset_id.clone()))
            .collect::<BTreeSet<_>>();
        let entities = BTreeSet::from([CLOCKWORKS_REGULATOR_ENTITY_TYPE.to_owned()]);
        let required = BTreeMap::from([(
            "clockworks".to_owned(),
            BTreeSet::from([
                "clockworks_valves".to_owned(),
                "clockworks_core".to_owned(),
                "clockworks_shutdown".to_owned(),
            ]),
        )]);
        assert!(
            base.validate(&assets, &entities, &required).is_ok(),
            "{:?}",
            base.validate(&assets, &entities, &required)
        );

        let mut invalid_damage = base.clone();
        invalid_damage.hazards[0].damage = Some(0);
        assert!(matches!(
            invalid_damage.validate(&assets, &entities, &required),
            Err(crate::scene_runtime::SceneRuntimeError::MalformedSceneDefinition)
        ));

        let mut invalid_period = base.clone();
        invalid_period.hazards[0].period_ms = Some(0);
        assert!(matches!(
            invalid_period.validate(&assets, &entities, &required),
            Err(crate::scene_runtime::SceneRuntimeError::MalformedSceneDefinition)
        ));

        let mut duplicate = base.clone();
        duplicate.hazards.push(duplicate.hazards[0].clone());
        assert!(matches!(
            duplicate.validate(&assets, &entities, &required),
            Err(crate::scene_runtime::SceneRuntimeError::DuplicateObjectId)
        ));

        let mut invalid_polygon = base.clone();
        invalid_polygon.hazards[0].polygon[1] = invalid_polygon.hazards[0].polygon[0];
        assert!(matches!(
            invalid_polygon.validate(&assets, &entities, &required),
            Err(crate::scene_runtime::SceneRuntimeError::InvalidPolygon)
        ));

        let mut overlaps_route = base;
        overlaps_route.hazards[0].polygon =
            vec![[15.5, 3.5], [20.5, 3.5], [20.5, 8.0], [15.5, 8.0]];
        assert!(matches!(
            overlaps_route.validate(&assets, &entities, &required),
            Err(crate::scene_runtime::SceneRuntimeError::InvalidPolygon)
        ));
    }

    #[test]
    fn registered_prime_regulator_emits_core_scoped_pressure_wave_events() {
        let runtime = runtime_for_clockworks_regulator_gate();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        for seq in 1..=110 {
            let mut state = runtime.state.lock().unwrap();
            let scene = runtime.scene_runtime.lock().unwrap();
            state.world.player.position_m = Vec3::new(17.5, 0.0, 8.0).unwrap();
            state.latest_sample = InputSample::new(epoch, seq, seq * 17, 0.0, 0.0).unwrap();
            state.latest_input_received_at = Instant::now();
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        }
        let events = runtime.presentation_events_since(epoch, 0).unwrap();
        let windup = events
            .iter()
            .find(|event| event.kind == "PrimeRegulatorPressureWindup")
            .expect("the registered core boss emits a Rust-authored warning");
        let impact = events
            .iter()
            .find(|event| event.kind == "PrimeRegulatorPressureImpact")
            .expect("the registered core boss emits its pressure impact");
        assert_eq!(windup.radius_m, 4.5);
        assert_eq!(impact.radius_m, 4.5);
        assert!(impact.server_tick > windup.server_tick);
        assert!(!events
            .iter()
            .any(|event| { event.kind == "clockworks_core" || event.kind == "ClockworksCore" }));
        assert_eq!(
            runtime
                .presentation_events_since(epoch.saturating_sub(1), 0)
                .unwrap_err(),
            "E_EVENT_STALE_EPOCH"
        );
        let new_journey = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(!new_journey
            .actors
            .iter()
            .any(|actor| actor.entity_id == CLOCKWORKS_REGULATOR_SPAWN_ID));
    }

    fn clockworks_elite_owner_attack(runtime: &FormalRuntime, seq: u64) {
        let mut state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        state.world.player.position_m = Vec3::new(11.2, 0.0, 7.0).unwrap();
        state.latest_sample =
            InputSample::new(state.world.revision.world_epoch, seq, seq * 16, 0.0, 0.0)
                .unwrap()
                .with_aim(1.0, 0.0)
                .unwrap();
        state.latest_input_received_at = Instant::now();
        state
            .pending_combat
            .push(crate::continuous_combat::CombatIntent::Attack { request_id: seq });
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
    }

    #[test]
    fn registered_clockworks_elite_emits_epoch_scoped_pressure_wave_events_after_windup() {
        let runtime = runtime_for_clockworks_elite_gate();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        for seq in 1..=40 {
            let mut state = runtime.state.lock().unwrap();
            let scene = runtime.scene_runtime.lock().unwrap();
            state.world.player.position_m = Vec3::new(14.5, 0.0, 7.0).unwrap();
            state.latest_sample = InputSample::new(epoch, seq, seq * 17, 0.0, 0.0).unwrap();
            state.latest_input_received_at = Instant::now();
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        }
        {
            let state = runtime.state.lock().unwrap();
            assert_eq!(
                state.world.player_hp, 100,
                "900 ms windup has not resolved yet"
            );
        }
        for seq in 41..=60 {
            let mut state = runtime.state.lock().unwrap();
            let scene = runtime.scene_runtime.lock().unwrap();
            state.latest_sample = InputSample::new(epoch, seq, seq * 17, 0.0, 0.0).unwrap();
            state.latest_input_received_at = Instant::now();
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        }
        let events = runtime.presentation_events_since(epoch, 0).unwrap();
        let windup = events
            .iter()
            .find(|event| event.kind == "ForgedGuardPressureWindup")
            .unwrap();
        let impact = events
            .iter()
            .find(|event| event.kind == "ForgedGuardPressureImpact")
            .unwrap();
        assert_eq!(windup.position_m, Vec3::new(12.0, 0.0, 7.0).unwrap());
        assert_eq!(
            impact.position_m, windup.position_m,
            "impact uses the captured wave origin"
        );
        assert_eq!(windup.radius_m, 3.0);
        assert_eq!(impact.radius_m, 3.0);
        assert!(impact.server_tick > windup.server_tick);
        assert_eq!(runtime.state.lock().unwrap().world.player_hp, 84);
        assert_eq!(
            runtime
                .presentation_events_since(epoch.saturating_sub(1), 0)
                .unwrap_err(),
            "E_EVENT_STALE_EPOCH"
        );
    }

    fn arena_shutter_route(
        scene: &crate::scene_runtime::SceneDefinition,
        blocker_id: &str,
    ) -> (Vec3, Vec3, f32, bool, f32, f32) {
        let blocker = scene
            .collision
            .iter()
            .find(|collision| collision.id == blocker_id)
            .unwrap_or_else(|| panic!("missing compiled Arena blocker {blocker_id}"));
        let min_x = blocker
            .polygon
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);
        let max_x = blocker
            .polygon
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_z = blocker
            .polygon
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min);
        let max_z = blocker
            .polygon
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let spawn = scene
            .spawns
            .iter()
            .find(|spawn| spawn.kind == "player")
            .expect("compiled Arena player spawn");
        let west = (min_x + max_x) * 0.5 < scene.bounds_m.x + scene.bounds_m.width * 0.5;
        let start = Vec3::new(spawn.position[0], 0.0, (min_z + max_z) * 0.5).unwrap();
        let radius = crate::continuous_kcc::KccBody::new(start).radius_m;
        let target_x = if west {
            scene.bounds_m.x + radius + 0.05
        } else {
            scene.bounds_m.x + scene.bounds_m.width - radius - 0.05
        };
        let target = Vec3::new(target_x, 0.0, start.z_m).unwrap();
        (
            start,
            target,
            if west { -1.0 } else { 1.0 },
            west,
            min_x,
            max_x,
        )
    }

    fn assert_shutter_blocks_kcc(
        world: &crate::continuous_kcc::StaticKccWorld,
        scene: &crate::scene_runtime::SceneDefinition,
        blocker_id: &str,
    ) {
        let (start, target, direction, west, min_x, max_x) = arena_shutter_route(scene, blocker_id);
        let mut body = crate::continuous_kcc::KccBody::new(start);
        assert!(
            world.can_occupy(start, body.radius_m),
            "inside route is legal: {blocker_id}"
        );
        assert!(
            !world.can_occupy(target, body.radius_m),
            "outside route is blocked while shutter is closed: {blocker_id}"
        );
        let mut collided_on_x = false;
        for _ in 0..256 {
            let collisions =
                crate::continuous_kcc::step_kcc(&mut body, world, (direction, 0.0), 0.05).unwrap();
            collided_on_x |= collisions.iter().any(|collision| collision.axis == "x");
        }
        assert!(
            collided_on_x,
            "closed shutter generates an X collision: {blocker_id}"
        );
        if west {
            assert!(
                body.position_m.x_m >= max_x + body.radius_m - 0.1,
                "closed west shutter prevents crossing its compiled polygon: {blocker_id}"
            );
        } else {
            assert!(
                body.position_m.x_m <= min_x - body.radius_m + 0.1,
                "closed east shutter prevents crossing its compiled polygon: {blocker_id}"
            );
        }
    }

    fn assert_shutter_allows_kcc_crossing(
        world: &crate::continuous_kcc::StaticKccWorld,
        scene: &crate::scene_runtime::SceneDefinition,
        blocker_id: &str,
    ) {
        let (start, target, direction, west, _, _) = arena_shutter_route(scene, blocker_id);
        let mut body = crate::continuous_kcc::KccBody::new(start);
        assert!(
            world.can_occupy(start, body.radius_m),
            "inside route is legal: {blocker_id}"
        );
        assert!(
            world.can_occupy(target, body.radius_m),
            "outside route is legal after kill: {blocker_id}"
        );
        for _ in 0..256 {
            if (body.position_m.x_m - target.x_m).abs() <= 0.05 {
                break;
            }
            crate::continuous_kcc::step_kcc(&mut body, world, (direction, 0.0), 0.05).unwrap();
        }
        assert!(
            (body.position_m.x_m - target.x_m).abs() <= 0.1,
            "player crosses the opened {} Arena shutter: {blocker_id}",
            if west { "west" } else { "east" }
        );
    }

    #[test]
    fn clockworks_elite_transition_requires_real_death_and_persists_across_arena_revisit() {
        let runtime = runtime_for_clockworks_elite_gate();
        let arena_definition = runtime
            .scene_runtime
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .current_scene()
            .clone();
        assert_eq!(runtime.state.lock().unwrap().kcc.walls.len(), 8);
        assert_eq!(
            scene_kcc(&arena_definition, &std::collections::BTreeSet::new())
                .unwrap()
                .walls
                .len(),
            8,
            "no-state scene collision construction keeps both shutters closed"
        );
        let initial_closed_kcc =
            scene_kcc(&arena_definition, &std::collections::BTreeSet::new()).unwrap();
        for blocker_id in [
            "cw_arena_shutter_west_blocker",
            "cw_arena_shutter_east_blocker",
        ] {
            assert_shutter_blocks_kcc(&initial_closed_kcc, &arena_definition, blocker_id);
        }
        assert_eq!(runtime.snapshot().unwrap().actors.len(), 1);
        assert_eq!(
            runtime.snapshot().unwrap().actors[0].entity_id,
            CLOCKWORKS_ELITE_SPAWN_ID
        );
        assert_eq!(
            runtime.snapshot().unwrap().actors[0].entity_type,
            "runtime2d.enemy.clockworks.forged_guard.v2"
        );

        // A prewritten dead state without a real owner-combat hit is not progress.
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.generic_actors[0].take_damage(u32::MAX);
        }
        {
            let mut state = runtime.state.lock().unwrap();
            let scene = runtime.scene_runtime.lock().unwrap();
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            assert_eq!(
                state.kcc.walls.len(),
                8,
                "a forged zero-HP actor does not open shutters"
            );
            for blocker_id in [
                "cw_arena_shutter_west_blocker",
                "cw_arena_shutter_east_blocker",
            ] {
                assert_shutter_blocks_kcc(&state.kcc, &arena_definition, blocker_id);
            }
            assert!(
                !state
                    .world_persistent_v1
                    .clockworks
                    .forged_guard_elite_first_kill
            );
            state.world.generic_actors[0] = crate::world_v3::ActorRuntime::spawn(
                CLOCKWORKS_ELITE_SPAWN_ID,
                CLOCKWORKS_ELITE_ENTITY_TYPE,
                Vec3::new(12.0, 0.0, 7.0).unwrap(),
            )
            .unwrap();
        }

        set_test_player_position(&runtime, Vec3::new(23.0, 0.0, 8.0).unwrap());
        let blocked_epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        assert_eq!(
            runtime
                .transition_scene(
                    "cw_arena_to_regulator_core",
                    "cw-elite-core-gate-retry",
                    blocked_epoch,
                )
                .unwrap_err(),
            "E_SCENE_CLOCKWORKS_ELITE_FIRST_KILL_REQUIRED"
        );
        for (exit_id, position) in [
            (
                "cw_arena_return_to_furnace_heart",
                Vec3::new(1.0, 0.0, 8.0).unwrap(),
            ),
            (
                "cw_arena_to_regulator_core",
                Vec3::new(23.0, 0.0, 8.0).unwrap(),
            ),
        ] {
            set_test_player_position(&runtime, position);
            let exit = runtime
                .snapshot()
                .unwrap()
                .interactables
                .into_iter()
                .find(|item| item.entity_id == exit_id)
                .unwrap_or_else(|| panic!("missing projected exit {exit_id}"));
            assert!(!exit.active, "{exit_id} must stay inactive before the kill");
            let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
            assert_eq!(
                runtime
                    .transition_scene(exit_id, "cw-elite-exit-blocked", epoch)
                    .unwrap_err(),
                "E_SCENE_CLOCKWORKS_ELITE_FIRST_KILL_REQUIRED"
            );
            assert_eq!(
                runtime.snapshot().unwrap().scene_id,
                CLOCKWORKS_ELITE_SCENE_ID
            );
        }
        assert_eq!(
            runtime.snapshot().unwrap().scene_id,
            CLOCKWORKS_ELITE_SCENE_ID
        );
        runtime.pause().unwrap();

        for seq in 1..=5 {
            clockworks_elite_owner_attack(&runtime, seq);
            let state = runtime.state.lock().unwrap();
            if seq < 5 {
                assert!(
                    !state
                        .world_persistent_v1
                        .clockworks
                        .forged_guard_elite_first_kill
                );
                assert!(state.world.generic_actors[0].hp > 0);
            } else {
                assert_eq!(state.world.generic_actors[0].hp, 0);
                assert!(state.world.generic_actors[0].validate());
                assert!(
                    state
                        .world_persistent_v1
                        .clockworks
                        .forged_guard_elite_first_kill
                );
                assert_eq!(
                    state.kcc.walls.len(),
                    6,
                    "the real kill refreshes KCC in the same owner tick"
                );
                for blocker_id in [
                    "cw_arena_shutter_west_blocker",
                    "cw_arena_shutter_east_blocker",
                ] {
                    assert_shutter_allows_kcc_crossing(&state.kcc, &arena_definition, blocker_id);
                }
            }
        }

        runtime.resume().unwrap();
        runtime.state.lock().unwrap().route.current_world_id = "clockworks".into();
        for (exit_id, position) in [
            (
                "cw_arena_return_to_furnace_heart",
                Vec3::new(1.0, 0.0, 8.0).unwrap(),
            ),
            (
                "cw_arena_to_regulator_core",
                Vec3::new(23.0, 0.0, 8.0).unwrap(),
            ),
        ] {
            set_test_player_position(&runtime, position);
            let exit = runtime
                .snapshot()
                .unwrap()
                .interactables
                .into_iter()
                .find(|item| item.entity_id == exit_id)
                .unwrap_or_else(|| panic!("missing projected exit {exit_id}"));
            assert!(exit.active, "{exit_id} must be active after the kill");
        }
        runtime
            .save_slot("cw-elite-kill", "CW Elite Kill", true)
            .unwrap();
        let saved = crate::save_slots::read_slot_v6(&runtime.save_root, "cw-elite-kill")
            .unwrap()
            .1;
        let restored = saved.restore_state().unwrap();
        assert!(
            restored
                .world_persistent_v1
                .clockworks
                .forged_guard_elite_first_kill
        );
        let restored_kcc = scene_kcc_for_state(
            &arena_definition,
            &std::collections::BTreeSet::new(),
            &restored,
        )
        .unwrap();
        assert_eq!(
            restored_kcc.walls.len(),
            6,
            "Save V6 restore keeps the shutters open"
        );
        for blocker_id in [
            "cw_arena_shutter_west_blocker",
            "cw_arena_shutter_east_blocker",
        ] {
            assert_shutter_allows_kcc_crossing(&restored_kcc, &arena_definition, blocker_id);
        }
        assert_eq!(restored.world.generic_actors[0].hp, 0);
        let old_persistence: crate::world_persistent_v1::WorldPersistentState =
            serde_json::from_value(serde_json::json!({"mistHarbor": {}})).unwrap();
        assert!(!old_persistence.clockworks.forged_guard_elite_first_kill);

        set_test_player_position(&runtime, Vec3::new(1.0, 0.0, 8.0).unwrap());
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        runtime
            .transition_scene(
                "cw_arena_return_to_furnace_heart",
                "cw-elite-return-furnace",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        set_test_player_position(&runtime, Vec3::new(23.0, 0.0, 8.0).unwrap());
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        let revisit = runtime
            .transition_scene("cw_furnace_heart_to_arena", "cw-elite-revisit-arena", epoch).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(revisit.actors.len(), 1);
        assert!(!revisit.actors[0].active);
        let revisit_kcc = runtime.state.lock().unwrap().kcc.clone();
        for blocker_id in [
            "cw_arena_shutter_west_blocker",
            "cw_arena_shutter_east_blocker",
        ] {
            assert_shutter_allows_kcc_crossing(&revisit_kcc, &arena_definition, blocker_id);
        }
        set_test_player_position(&runtime, Vec3::new(23.0, 0.0, 8.0).unwrap());
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        assert_eq!(
            runtime
                .transition_scene(
                    "cw_arena_to_regulator_core",
                    "cw-elite-core-gate-retry",
                    epoch,
                ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
                .unwrap()
                .scene_id,
            "cw_regulator_core"
        );
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .clockworks
                .forged_guard_elite_first_kill
        );
        assert_eq!(runtime.state.lock().unwrap().kcc.walls.len(), 6);

        runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        let state = runtime.state.lock().unwrap();
        assert!(
            !state
                .world_persistent_v1
                .clockworks
                .forged_guard_elite_first_kill
        );
        let new_journey_kcc = scene_kcc_for_state(
            &arena_definition,
            &std::collections::BTreeSet::new(),
            &state,
        )
        .unwrap();
        assert_eq!(
            new_journey_kcc.walls.len(),
            8,
            "New Journey closes both shutters"
        );
        for blocker_id in [
            "cw_arena_shutter_west_blocker",
            "cw_arena_shutter_east_blocker",
        ] {
            assert_shutter_blocks_kcc(&new_journey_kcc, &arena_definition, blocker_id);
        }
    }

    #[test]
    fn enemy_vitals_projection_is_capability_gated_and_never_serializes_exact_hp() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        let mut actor = crate::world_v3::ActorRuntime::spawn(
            "worker-01",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        actor.hp = 40;
        state.world.generic_actors.push(actor);

        assert!(project_view(&state).capabilities.enemy_vitals.is_empty());

        grant_enemy_vitals(&mut state);
        let snapshot = crate::world_v3::WorldSnapshot::from_view(project_view(&state));
        let json = serde_json::to_value(snapshot).unwrap();
        let enemy_vitals = json["capabilities"]["enemyVitals"].as_array().unwrap();
        assert_eq!(enemy_vitals.len(), 1);
        assert_eq!(enemy_vitals[0]["entityId"], "worker-01");
        assert_eq!(enemy_vitals[0]["tier"], "wounded");
        let keys = enemy_vitals[0]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        assert_eq!(keys, BTreeSet::from(["entityId", "tier"]));
        assert!(enemy_vitals[0].get("currentHp").is_none());
        assert!(enemy_vitals[0].get("maxHp").is_none());
    }

    #[test]
    fn enemy_vitals_projection_uses_existing_tier_boundaries_for_generic_actors() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        grant_enemy_vitals(&mut state);
        for (hp, expected) in [
            (50, crate::world_v3::VitalTier::Healthy),
            (40, crate::world_v3::VitalTier::Wounded),
            (30, crate::world_v3::VitalTier::SeverelyWounded),
            (12, crate::world_v3::VitalTier::Critical),
        ] {
            let mut actor = crate::world_v3::ActorRuntime::spawn(
                "tier-target",
                "grey_hive.infected_maintenance_worker",
                Vec3::zero(),
            )
            .unwrap();
            actor.hp = hp;
            state.world.generic_actors = vec![actor];
            assert_eq!(
                project_view(&state).capabilities.enemy_vitals[0].tier,
                expected
            );
        }
    }

    #[test]
    fn enemy_vitals_projection_limits_sentinel_to_its_registered_grey_hive_scene() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        grant_enemy_vitals(&mut state);
        state.world.scene_id = SENTINEL_SCENE_ID.into();
        state.world.sentinels = vec![crate::sentinel_ai::Sentinel::new(
            SENTINEL_SPAWN_ID,
            Vec3::zero(),
            75,
        )];
        let projected = project_view(&state).capabilities.enemy_vitals;
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].entity_id, SENTINEL_SPAWN_ID);
        assert_eq!(projected[0].tier, crate::world_v3::VitalTier::Wounded);

        state.world.scene_id = "gh_deep_decon".into();
        assert!(project_view(&state).capabilities.enemy_vitals.is_empty());

        state.world.scene_id = SENTINEL_SCENE_ID.into();
        state.world.sentinels[0].entity_id = "unknown-sentinel".into();
        assert!(project_view(&state).capabilities.enemy_vitals.is_empty());

        state.world.sentinels[0].entity_id = SENTINEL_SPAWN_ID.into();
        state.world.sentinels[0].hp = SENTINEL_DEVELOPMENT_HP + 1;
        assert!(project_view(&state).capabilities.enemy_vitals.is_empty());
    }

    #[test]
    fn enemy_vitals_projection_skips_dead_malformed_and_duplicate_actors() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        grant_enemy_vitals(&mut state);

        let mut dead = crate::world_v3::ActorRuntime::spawn(
            "dead",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        dead.take_damage(u32::MAX);
        let mut unknown = crate::world_v3::ActorRuntime::spawn(
            "unknown",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        unknown.entity_type = "unregistered.enemy".into();
        let mut over_max = crate::world_v3::ActorRuntime::spawn(
            "over-max",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        over_max.hp = 51;
        let duplicate_a = crate::world_v3::ActorRuntime::spawn(
            "duplicate",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        let duplicate_b = duplicate_a.clone();
        let valid = crate::world_v3::ActorRuntime::spawn(
            "valid",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        state.world.generic_actors = vec![dead, unknown, over_max, duplicate_a, duplicate_b, valid];

        let view = project_view(&state);
        let projected = &view.capabilities.enemy_vitals;
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].entity_id, "valid");
        assert!(view.actors.iter().any(|actor| actor.entity_id == "valid"));
        assert!(!view
            .actors
            .iter()
            .any(|actor| actor.entity_id == "unknown" || actor.entity_id == "over-max"));
    }

    #[test]
    fn sentinel_arena_uses_one_authored_dynamic_actor_on_scene_entry_paths() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();

        let arena_view = runtime
            .install_scene_registry(registry.clone(), SENTINEL_SCENE_ID)
            .unwrap();
        assert_eq!(arena_view.actors.len(), 1);
        assert_eq!(arena_view.actors[0].entity_id, SENTINEL_SPAWN_ID);
        assert_eq!(arena_view.actors[0].entity_type, SENTINEL_ENTITY_TYPE);
        assert_eq!(arena_view.actors[0].actor_kind, "sentinel");
        assert!(arena_view.actors[0].active);
        assert!(runtime
            .state
            .lock()
            .unwrap()
            .world
            .generic_actors
            .is_empty());

        let source: crate::scene_runtime::SceneDefinition =
            serde_json::from_str(SENTINEL_ARENA_SCENE).unwrap();
        assert!(spawn_generic_actors(&source).unwrap().is_empty());
        assert_eq!(spawn_authored_sentinels(&source).unwrap().len(), 1);
        assert!(!SENTINEL_ARENA_TMJ.contains("sentinel_static_base_record"));
        assert!(!SENTINEL_ARENA_TMJ.contains("sentinel_static_parts_record"));
        assert!(!source
            .presentation
            .sprites
            .iter()
            .any(|sprite| sprite.asset_id == "runtime2d.enemy.sentinel.parts.v1"));

        // The fixed world gate enters Grey Hive at its authored entry scene;
        // that scene correctly has no Sentinel and replaces the prior roster.
        runtime
            .install_scene_registry(registry.clone(), "rs_core_room")
            .unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3::new(20.0, 0.0, 8.0).unwrap();
        }
        let gate_epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        let gate_view = runtime
            .use_world_gate("rs_world_gate_to_gh", "sentinel-gate-entry", gate_epoch).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(gate_view.scene_id, "gh_entry_maintenance");
        assert!(!gate_view
            .actors
            .iter()
            .any(|actor| actor.entity_type == SENTINEL_ENTITY_TYPE));

        let deep_view = runtime
            .install_scene_registry(registry.clone(), "gh_deep_decon")
            .unwrap();
        assert_eq!(deep_view.actors.len(), 4);
        assert_eq!(deep_view.actors.iter().map(|actor| (actor.entity_id.as_str(), actor.entity_type.as_str())).collect::<std::collections::BTreeSet<_>>(), [
            ("gh_decon_worker_01", "runtime2d.enemy.infected_maintenance_worker.v1"),
            ("gh_decon_worker_02", "runtime2d.enemy.infected_maintenance_worker.v1"),
            ("gh_deep_decon_swarm_01", swarm_roster::SWARM),
            ("gh_deep_decon_swarm_02", swarm_roster::SWARM),
        ].into_iter().collect());
        assert!(deep_view
            .actors
            .iter()
            .all(|actor| actor.actor_kind == "enemy"));
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3::new(23.0, 0.0, 8.0).unwrap();
        }
        let transition_epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        let transitioned = runtime
            .transition_scene(
                "gh_deep_decon_to_sentinel_arena",
                "sentinel-scene-entry",
                transition_epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(transitioned.scene_id, SENTINEL_SCENE_ID);
        assert_eq!(transitioned.actors.len(), 1);
        assert_eq!(transitioned.actors[0].entity_id, SENTINEL_SPAWN_ID);
        assert_eq!(transitioned.actors[0].entity_type, SENTINEL_ENTITY_TYPE);
    }

    #[test]
    fn sentinel_ai_ticks_at_sixty_hz_deals_damage_and_stops_after_death() {
        let scene: crate::scene_runtime::SceneDefinition =
            serde_json::from_str(SENTINEL_ARENA_SCENE).unwrap();
        let mut world = WorldStateV3::new(
            "grey_hive",
            1,
            crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap(),
        )
        .unwrap();
        world.scene_id = SENTINEL_SCENE_ID.into();
        world.sentinels = spawn_authored_sentinels(&scene).unwrap();
        world.sentinels[0].position_m = crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
        let kcc = scene_kcc(&scene, &std::collections::BTreeSet::new()).unwrap();

        let mut damaged_player = false;
        for seq in 1..=90 {
            let sample =
                crate::continuous_input::InputSample::new(1, seq, seq * 16, 0.0, 0.0).unwrap();
            crate::world_v3::step_world(
                &mut world,
                &kcc,
                crate::world_v3::CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[],
                },
            )
            .unwrap();
            if world.player_hp < world.player_max_hp {
                damaged_player = true;
                break;
            }
        }
        assert!(
            damaged_player,
            "Sentinel AI did not damage the player at 60 Hz"
        );

        let mut hit_count = 0;
        for seq in 91..=94 {
            let sample = crate::continuous_input::InputSample::new(1, seq, seq * 16, 0.0, 0.0)
                .unwrap()
                .with_aim(1.0, 0.0)
                .unwrap();
            let output = crate::world_v3::step_world(
                &mut world,
                &kcc,
                crate::world_v3::CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[crate::continuous_combat::CombatIntent::Attack { request_id: seq }],
                },
            )
            .unwrap();
            hit_count += output
                .combat
                .iter()
                .filter(|event| {
                    matches!(event,
                    crate::continuous_combat::CombatEvent::AttackHit { target_id, .. }
                        if target_id == SENTINEL_SPAWN_ID)
                })
                .count();
        }
        assert_eq!(hit_count, 4);
        assert_eq!(world.sentinels[0].hp, 0);
        assert!(!world.sentinels[0].active);
        assert_eq!(
            world.sentinels[0].state,
            crate::sentinel_ai::SentinelState::Death
        );
        let player_hp_after_death = world.player_hp;
        for seq in 95..=154 {
            let sample =
                crate::continuous_input::InputSample::new(1, seq, seq * 16, 0.0, 0.0).unwrap();
            crate::world_v3::step_world(
                &mut world,
                &kcc,
                crate::world_v3::CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[],
                },
            )
            .unwrap();
        }
        assert_eq!(world.player_hp, player_hp_after_death);
    }

    #[test]
    fn sentinel_save_restore_preserves_roster_and_fails_closed() {
        let save_root = std::env::temp_dir().join(format!(
            "formal-sentinel-save-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry.clone(), SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.sentinels[0].take_damage(25);
        }
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let state_guard = runtime.state.lock().unwrap();
        let v5 = crate::save_v5::SaveV5::capture(&state_guard, scene_guard.as_ref()).unwrap();
        drop(state_guard);
        let arena = scene_guard.as_ref().unwrap().current_scene().clone();
        drop(scene_guard);

        let mut restored_v5 = v5.clone().restore_state().unwrap();
        restore_sentinels(&mut restored_v5.world, &arena).unwrap();
        assert_eq!(restored_v5.world.sentinels[0].hp, 75);
        assert_eq!(
            restored_v5.world.sentinels[0].state,
            crate::sentinel_ai::SentinelState::Hit
        );
        assert_eq!(restored_v5.world.sentinels[0].state_remaining_ms, 120);

        let mut legacy_v5 = v5.clone();
        legacy_v5.actors.clear();
        let mut restored_legacy = legacy_v5.restore_state().unwrap();
        restore_sentinels(&mut restored_legacy.world, &arena).unwrap();
        assert_eq!(restored_legacy.world.sentinels.len(), 1);
        assert_eq!(
            restored_legacy.world.sentinels[0].hp,
            SENTINEL_DEVELOPMENT_HP
        );

        runtime
            .save_slot("sentinel-hurt", "Sentinel Hurt", true)
            .unwrap();
        runtime.continue_slot("sentinel-hurt").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert_eq!(runtime.state.lock().unwrap().world.sentinels[0].hp, 75);

        {
            let mut state = runtime.state.lock().unwrap();
            state.world.sentinels[0].take_damage(100);
        }
        runtime
            .save_slot("sentinel-dead", "Sentinel Dead", true)
            .unwrap();
        runtime.continue_slot("sentinel-dead").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        let state = runtime.state.lock().unwrap();
        assert_eq!(state.world.sentinels[0].hp, 0);
        assert_eq!(
            state.world.sentinels[0].state,
            crate::sentinel_ai::SentinelState::Death
        );
        assert!(!state.world.sentinels[0].active);
        assert!(!state.world_persistent_v1.grey_hive.sentinel_first_kill);

        let mut wrong_id =
            WorldStateV3::new("grey_hive", 1, crate::world_v3::Vec3::zero()).unwrap();
        wrong_id.scene_id = SENTINEL_SCENE_ID.into();
        wrong_id.sentinels = vec![crate::sentinel_ai::Sentinel::new(
            "wrong-sentinel-id",
            crate::world_v3::Vec3::zero(),
            SENTINEL_DEVELOPMENT_HP,
        )];
        assert_eq!(
            restore_sentinels(&mut wrong_id, &arena).unwrap_err(),
            "E_SAVE_SENTINEL_ROSTER_MISMATCH"
        );
        let mut duplicate =
            WorldStateV3::new("grey_hive", 1, crate::world_v3::Vec3::zero()).unwrap();
        duplicate.scene_id = SENTINEL_SCENE_ID.into();
        duplicate.sentinels = vec![
            spawn_authored_sentinels(&arena).unwrap().remove(0),
            spawn_authored_sentinels(&arena).unwrap().remove(0),
        ];
        assert_eq!(
            restore_sentinels(&mut duplicate, &arena).unwrap_err(),
            "E_SAVE_SENTINEL_ROSTER_MISMATCH"
        );
        let deep_decon: crate::scene_runtime::SceneDefinition = serde_json::from_str(include_str!(
            "../../content/scenes/compiled/gh_deep_decon.json"
        ))
        .unwrap();
        assert_eq!(
            restore_sentinels(&mut duplicate, &deep_decon).unwrap_err(),
            "E_SENTINEL_ROSTER_SCENE_MISMATCH"
        );
    }

    #[test]
    fn sentinel_first_kill_tracks_actual_lethal_combat_and_persists() {
        let save_root = std::env::temp_dir().join(format!(
            "formal-sentinel-first-kill-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry.clone(), SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();

        for seq in 1..=3 {
            let scene = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap();
            state.world.sentinels[0].position_m =
                crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
            state.latest_sample =
                InputSample::new(state.world.revision.world_epoch, seq, seq * 16, 0.0, 0.0)
                    .unwrap()
                    .with_aim(1.0, 0.0)
                    .unwrap();
            state.latest_input_received_at = Instant::now();
            state
                .pending_combat
                .push(crate::continuous_combat::CombatIntent::Attack { request_id: seq });
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            assert_eq!(state.world.sentinels[0].hp, 100 - (seq as u32 * 25));
            assert!(!state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }

        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            let seq = 4;
            state.world.player.position_m = crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap();
            state.world.sentinels[0].position_m =
                crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
            state.latest_sample =
                InputSample::new(state.world.revision.world_epoch, seq, seq * 16, 0.0, 0.0)
                    .unwrap()
                    .with_aim(1.0, 0.0)
                    .unwrap();
            state.latest_input_received_at = Instant::now();
            state
                .pending_combat
                .push(crate::continuous_combat::CombatIntent::Attack { request_id: seq });
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            assert_eq!(state.world.sentinels[0].hp, 0);
            assert!(!state.world.sentinels[0].active);
            assert!(state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }

        // A later dead-state step cannot manufacture a new event; the fact is idempotent.
        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            state.pending_combat.clear();
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            assert!(state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }

        runtime
            .save_slot("sentinel-first-kill", "Sentinel First Kill", true)
            .unwrap();
        runtime.continue_slot("sentinel-first-kill").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .grey_hive
                .sentinel_first_kill
        );
        assert_eq!(runtime.state.lock().unwrap().world.sentinels[0].hp, 0);

        runtime.resume().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3::new(1.0, 0.0, 8.0).unwrap();
        }
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        runtime
            .transition_scene(
                "gh_sentinel_arena_return_to_deep_decon",
                "first-kill-exit",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3::new(23.0, 0.0, 8.0).unwrap();
        }
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        let reentry = runtime
            .transition_scene(
                "gh_deep_decon_to_sentinel_arena",
                "first-kill-reentry",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert!(reentry.actors[0].active);
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .grey_hive
                .sentinel_first_kill
        );

        runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(
            !runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .grey_hive
                .sentinel_first_kill
        );
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn sentinel_beacon_gate_preflights_invalid_requests_and_leaves_locked_request_unclaimed() {
        let runtime = runtime_for_sentinel_beacon_gate();
        let beacon_position = Vec3::new(23.0, 0.0, 8.0).unwrap();
        set_test_player_position(&runtime, beacon_position);
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;

        assert_eq!(
            runtime
                .transition_scene(
                    "gh_sentinel_arena_to_beacon",
                    "stale-beacon-request",
                    epoch.saturating_sub(1),
                )
                .unwrap_err(),
            "E_SCENE_RUNTIME_StaleEpoch"
        );
        assert_eq!(
            runtime
                .transition_scene("unknown-arena-transition", "unknown-id-request", epoch)
                .unwrap_err(),
            "E_SCENE_RUNTIME_UnsafeTransition"
        );
        set_test_player_position(&runtime, Vec3::new(10.0, 0.0, 10.0).unwrap());
        assert_eq!(
            runtime
                .transition_scene(
                    "gh_sentinel_arena_to_beacon",
                    "out-of-range-beacon-request",
                    epoch,
                )
                .unwrap_err(),
            "E_SCENE_RUNTIME_OutOfRange"
        );
        set_test_player_position(&runtime, beacon_position);

        let route_before = runtime.state.lock().unwrap().route.clone();
        let (
            world_id_before,
            scene_id_before,
            revision_before,
            position_before,
            sentinel_hp_before,
        ) = {
            let state = runtime.state.lock().unwrap();
            (
                state.world.world_id.clone(),
                state.world.scene_id.clone(),
                state.world.revision,
                state.world.player.position_m,
                state.world.sentinels[0].hp,
            )
        };
        let (runtime_scene_before, runtime_epoch_before) = {
            let scene = runtime.scene_runtime.lock().unwrap();
            let scene = scene.as_ref().unwrap();
            (scene.scene_id.clone(), scene.world_epoch)
        };

        assert_eq!(
            runtime
                .transition_scene(
                    "gh_sentinel_arena_to_beacon",
                    "beacon-gate-retry-after-kill",
                    epoch,
                )
                .unwrap_err(),
            "E_SCENE_SENTINEL_FIRST_KILL_REQUIRED"
        );
        {
            let state = runtime.state.lock().unwrap();
            assert_eq!(state.world.world_id, world_id_before);
            assert_eq!(state.world.scene_id, scene_id_before);
            assert_eq!(state.world.revision, revision_before);
            assert_eq!(state.world.player.position_m, position_before);
            assert_eq!(state.world.sentinels[0].hp, sentinel_hp_before);
            assert_eq!(state.route, route_before);
            assert!(!state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }
        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let scene = scene.as_ref().unwrap();
            assert_eq!(scene.scene_id, runtime_scene_before);
            assert_eq!(scene.world_epoch, runtime_epoch_before);
        }

        kill_registered_sentinel_for_beacon_test(&runtime);
        set_test_player_position(&runtime, beacon_position);
        let entered = runtime
            .transition_scene(
                "gh_sentinel_arena_to_beacon",
                "beacon-gate-retry-after-kill",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(entered.scene_id, "gh_beacon");
        assert_eq!(entered.world_epoch, epoch + 1);
    }

    #[test]
    fn sentinel_beacon_gate_allows_return_continue_and_revisit_after_real_first_kill() {
        let runtime = runtime_for_sentinel_beacon_gate();
        let return_position = Vec3::new(1.0, 0.0, 8.0).unwrap();
        let arena_entry_position = Vec3::new(23.0, 0.0, 8.0).unwrap();
        let beacon_position = Vec3::new(23.0, 0.0, 8.0).unwrap();
        let route_before = runtime.state.lock().unwrap().route.clone();

        set_test_player_position(&runtime, return_position);
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        runtime
            .transition_scene(
                "gh_sentinel_arena_return_to_deep_decon",
                "prekill-arena-return",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(
            runtime.state.lock().unwrap().world.scene_id,
            "gh_deep_decon"
        );

        set_test_player_position(&runtime, arena_entry_position);
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        runtime
            .transition_scene(
                "gh_deep_decon_to_sentinel_arena",
                "prekill-arena-revisit",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(
            runtime.state.lock().unwrap().world.scene_id,
            SENTINEL_SCENE_ID
        );

        kill_registered_sentinel_for_beacon_test(&runtime);
        runtime
            .save_slot("sentinel-beacon-gate", "Sentinel Beacon Gate", true)
            .unwrap();
        runtime.continue_slot("sentinel-beacon-gate").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .grey_hive
                .sentinel_first_kill
        );

        set_test_player_position(&runtime, return_position);
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        runtime
            .transition_scene(
                "gh_sentinel_arena_return_to_deep_decon",
                "postkill-arena-return",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        set_test_player_position(&runtime, arena_entry_position);
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        runtime
            .transition_scene(
                "gh_deep_decon_to_sentinel_arena",
                "postkill-arena-revisit",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .grey_hive
                .sentinel_first_kill
        );
        assert_eq!(runtime.state.lock().unwrap().route, route_before);

        set_test_player_position(&runtime, beacon_position);
        let epoch = runtime.state.lock().unwrap().world.revision.world_epoch;
        let entered = runtime
            .transition_scene(
                "gh_sentinel_arena_to_beacon",
                "postkill-beacon-after-revisit",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(entered.scene_id, "gh_beacon");
        assert_eq!(runtime.state.lock().unwrap().route, route_before);
    }

    #[test]
    fn registered_sentinel_windups_emit_once_for_authoritative_state_transitions() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();

        let scene = runtime.scene_runtime.lock().unwrap();
        let mut state = runtime.state.lock().unwrap();
        let sentinel_position = Vec3::new(5.8, 0.0, 5.0).unwrap();
        state.world.player.position_m = Vec3::new(5.0, 0.0, 5.0).unwrap();
        state.world.sentinels[0].position_m = sentinel_position;
        state.world.sentinels[0].state = crate::sentinel_ai::SentinelState::Chase;
        state.world.sentinels[0].state_remaining_ms = 0;
        state.world.sentinels[0].attack_serial = 0;
        state.presentation_events.clear();
        state.next_presentation_event_id = 1;
        let epoch = state.world.revision.world_epoch;

        state.latest_sample = InputSample::new(epoch, 1, 16, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        let light = state
            .presentation_events
            .iter()
            .find(|event| event.kind == "SentinelAttackWindup")
            .unwrap()
            .clone();
        assert_eq!(light.kind, "SentinelAttackWindup");
        assert_eq!(light.position_m, sentinel_position);
        assert_eq!(light.world_epoch, epoch);
        assert_eq!(light.server_tick, state.world.revision.server_tick);
        assert_eq!(light.event_id, 1);

        state.latest_sample = InputSample::new(epoch, 2, 32, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert_eq!(
            state
                .presentation_events
                .iter()
                .filter(|event| event.kind == "SentinelAttackWindup")
                .count(),
            1
        );

        state.world.sentinels[0].state = crate::sentinel_ai::SentinelState::Attack;
        state.world.sentinels[0].state_remaining_ms = 0;
        state.world.sentinels[0].attack_serial = 2;
        state.latest_sample = InputSample::new(epoch, 3, 48, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        let heavy = state
            .presentation_events
            .iter()
            .find(|event| event.kind == "SentinelHeavyWindup")
            .unwrap()
            .clone();
        assert_eq!(heavy.kind, "SentinelHeavyWindup");
        assert_eq!(heavy.position_m, sentinel_position);
        assert_eq!(heavy.world_epoch, epoch);
        assert_eq!(heavy.server_tick, state.world.revision.server_tick);
        assert_eq!(heavy.event_id, 2);

        state.latest_sample = InputSample::new(epoch, 4, 64, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert_eq!(
            state
                .presentation_events
                .iter()
                .filter(|event| event.kind == "SentinelHeavyWindup")
                .count(),
            1
        );
        assert_eq!(state.presentation_events[1].event_id, 2);
    }

    #[test]
    fn registered_sentinel_death_event_follows_the_lethal_combat_predicate_once() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();

        let scene = runtime.scene_runtime.lock().unwrap();
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = Vec3::new(5.0, 0.0, 5.0).unwrap();
        state.world.sentinels[0].position_m = Vec3::new(5.8, 0.0, 5.0).unwrap();
        state.world.sentinels[0].hp = 1;
        state.latest_sample = InputSample::new(state.world.revision.world_epoch, 1, 16, 0.0, 0.0)
            .unwrap()
            .with_aim(1.0, 0.0)
            .unwrap();
        state.latest_input_received_at = Instant::now();
        state
            .pending_combat
            .push(crate::continuous_combat::CombatIntent::Attack { request_id: 1 });
        state.presentation_events.clear();
        state.next_presentation_event_id = 7;
        let epoch = state.world.revision.world_epoch;

        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        let death = state
            .presentation_events
            .iter()
            .find(|event| event.kind == "SentinelDeath")
            .unwrap()
            .clone();
        assert_eq!(death.kind, "SentinelDeath");
        assert_eq!(death.position_m, state.world.sentinels[0].position_m);
        assert_eq!(death.world_epoch, epoch);
        assert_eq!(death.server_tick, state.world.revision.server_tick);
        assert_eq!(death.event_id, 7);
        assert!(state.world_persistent_v1.grey_hive.sentinel_first_kill);

        state.latest_sample = InputSample::new(epoch, 2, 32, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert_eq!(
            state
                .presentation_events
                .iter()
                .filter(|event| event.kind == "SentinelDeath")
                .count(),
            1
        );
    }

    #[test]
    fn sentinel_presentation_events_reject_noncombat_death_wrong_scene_and_wrong_id() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();

        let scene = runtime.scene_runtime.lock().unwrap();
        let mut state = runtime.state.lock().unwrap();
        let epoch = state.world.revision.world_epoch;
        state.world.player.position_m = Vec3::new(5.0, 0.0, 5.0).unwrap();
        state.world.sentinels[0].position_m = Vec3::new(5.8, 0.0, 5.0).unwrap();

        state.latest_sample = InputSample::new(epoch, 1, 16, 0.0, 0.0)
            .unwrap()
            .with_aim(1.0, 0.0)
            .unwrap();
        state.latest_input_received_at = Instant::now();
        state
            .pending_combat
            .push(crate::continuous_combat::CombatIntent::Attack { request_id: 1 });
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert!(!state
            .presentation_events
            .iter()
            .any(|event| event.kind == "SentinelDeath"));

        state.world.sentinels[0].take_damage(SENTINEL_DEVELOPMENT_HP);
        state.latest_sample = InputSample::new(epoch, 2, 32, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert!(!state
            .presentation_events
            .iter()
            .any(|event| event.kind == "SentinelDeath"));

        state.world.sentinels = vec![crate::sentinel_ai::Sentinel::new(
            "wrong-sentinel-id",
            Vec3::new(5.8, 0.0, 5.0).unwrap(),
            SENTINEL_DEVELOPMENT_HP,
        )];
        state.presentation_events.clear();
        state.latest_sample = InputSample::new(epoch, 3, 48, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert!(state.presentation_events.is_empty());

        state.world.sentinels = vec![crate::sentinel_ai::Sentinel::new(
            SENTINEL_SPAWN_ID,
            Vec3::new(5.8, 0.0, 5.0).unwrap(),
            SENTINEL_DEVELOPMENT_HP,
        )];
        state.world.scene_id = "gh_deep_decon".into();
        state.presentation_events.clear();
        state.latest_sample = InputSample::new(epoch, 4, 64, 0.0, 0.0).unwrap();
        state.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert!(state.presentation_events.is_empty());
    }

    #[test]
    fn sentinel_first_kill_rejects_wrong_spawn_scene_and_noncombat_death() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry.clone(), SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();

        // A lethal owner combat hit against another sentinel ID is not the registered kill.
        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            state.world.sentinels[0].entity_id = "other-sentinel".into();
            state.world.sentinels[0].take_damage(75);
            state.world.player.position_m = crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap();
            state.world.sentinels[0].position_m =
                crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
            state.latest_sample =
                InputSample::new(state.world.revision.world_epoch, 1, 16, 0.0, 0.0)
                    .unwrap()
                    .with_aim(1.0, 0.0)
                    .unwrap();
            state.latest_input_received_at = Instant::now();
            state
                .pending_combat
                .push(crate::continuous_combat::CombatIntent::Attack { request_id: 1 });
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            assert_eq!(state.world.sentinels[0].hp, 0);
            assert!(!state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }

        // Reset the arena actor and give it a lethal hit while the runtime state's scene is wrong.
        runtime
            .install_scene_registry(registry.clone(), SENTINEL_SCENE_ID)
            .unwrap();
        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            state.world.scene_id = "gh_deep_decon".into();
            state.world.sentinels[0].take_damage(75);
            state.world.player.position_m = crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap();
            state.world.sentinels[0].position_m =
                crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
            state.latest_sample =
                InputSample::new(state.world.revision.world_epoch, 1, 16, 0.0, 0.0)
                    .unwrap()
                    .with_aim(1.0, 0.0)
                    .unwrap();
            state.latest_input_received_at = Instant::now();
            state
                .pending_combat
                .push(crate::continuous_combat::CombatIntent::Attack { request_id: 1 });
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            assert_eq!(state.world.sentinels[0].hp, 0);
            assert!(!state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }

        // Directly applying damage is not an actual lethal combat step.
        runtime
            .install_scene_registry(registry, SENTINEL_SCENE_ID)
            .unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.sentinels[0].take_damage(100);
            assert_eq!(state.world.sentinels[0].hp, 0);
            assert!(!state.world_persistent_v1.grey_hive.sentinel_first_kill);
        }
    }

    #[test]
    fn sentinel_first_kill_tracks_lethal_pulse_action_impact() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        runtime
            .install_scene_registry(registry, SENTINEL_SCENE_ID)
            .unwrap();
        runtime.pause().unwrap();

        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap();
            state.world.sentinels[0].position_m =
                crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
            // Leave a single Pulse hit lethal; the lethal damage itself comes from Pulse.
            state.world.sentinels[0].take_damage(90);
            assert_eq!(state.world.sentinels[0].hp, 10);
        }

        for seq in 1..=20 {
            let scene = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            state.latest_sample =
                InputSample::new(state.world.revision.world_epoch, seq, seq * 16, 0.0, 0.0)
                    .unwrap()
                    .with_aim(1.0, 0.0)
                    .unwrap();
            state.latest_input_received_at = Instant::now();
            if seq == 1 {
                state
                    .pending_combat
                    .push(crate::continuous_combat::CombatIntent::Pulse { request_id: 301 });
            }
            advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
            if state.world.sentinels[0].hp == 0 {
                break;
            }
        }

        let state = runtime.state.lock().unwrap();
        assert_eq!(state.world.sentinels[0].hp, 0);
        assert!(!state.world.sentinels[0].active);
        assert!(state
            .presentation_events
            .iter()
            .any(|event| event.kind == "PulseHit"));
        assert!(state.world_persistent_v1.grey_hive.sentinel_first_kill);
    }

    #[test]
    fn save_v6_accepts_pre_first_kill_world_persistence_shape() {
        let save_root = std::env::temp_dir().join(format!(
            "formal-sentinel-first-kill-legacy-v6-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world_persistent_v1.mist_harbor.explored_region_ids =
                vec!["mh_fp_entry_berth".into()];
        }
        let save = {
            let state = runtime.state.lock().unwrap();
            crate::save_v6::SaveV6::capture(&state, None).unwrap()
        };
        let json = serde_json::to_value(&save).unwrap();
        assert!(json["worldPersistentV1"].get("greyHive").is_none());
        crate::save_v6::write_save(&save_root, &save).unwrap();
        let restored = crate::save_v6::read_save(&save_root).unwrap();
        assert!(!restored.world_persistent_v1.grey_hive.sentinel_first_kill);
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn mist_harbor_extraction_gate_stays_hidden_and_atomic_until_all_required_events() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            let mut route = RouteState::new();
            for command in [
                RouteCommand::Enter {
                    world_id: "grey_hive".into(),
                    request_id: "mh-lock-gh-enter".into(),
                },
                RouteCommand::Progress {
                    event_id: "hive_power".into(),
                    request_id: "mh-lock-gh-power".into(),
                },
                RouteCommand::Progress {
                    event_id: "hive_lockdown".into(),
                    request_id: "mh-lock-gh-lockdown".into(),
                },
                RouteCommand::Progress {
                    event_id: "hive_extraction".into(),
                    request_id: "mh-lock-gh-extraction".into(),
                },
                RouteCommand::Complete {
                    world_id: "grey_hive".into(),
                    request_id: "mh-lock-gh-complete".into(),
                },
                RouteCommand::Enter {
                    world_id: "mist_harbor".into(),
                    request_id: "mh-lock-mh-enter".into(),
                },
            ] {
                assert!(matches!(
                    apply_route_command(&mut route, command, state.world.revision),
                    RouteResult::Applied { .. }
                ));
            }
            state.route = route;
        }
        let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
        let view = runtime
            .install_scene_registry(registry, "mh_extraction")
            .unwrap();
        assert_eq!(view.world_id, "mist_harbor");
        assert_eq!(view.scene_id, "mh_extraction");
        assert!(!view
            .interactables
            .iter()
            .any(|item| item.entity_id == "mh_extraction_return_to_rs"));

        let rejected = crate::scene_route_commands::world_gate(
            &runtime,
            "mh_extraction_return_to_rs",
            "mh-lock-exit-before-events",
            view.world_epoch,
        )
        .unwrap();
        assert!(!rejected.applied);
        assert_eq!(
            rejected.error_code.as_deref(),
            Some("E_MH_EXTRACTION_REQUIRED")
        );
        assert_eq!(rejected.snapshot.scene_id, "mh_extraction");
        assert_eq!(rejected.snapshot.world_epoch, view.world_epoch);
        let mh = rejected
            .snapshot
            .progression
            .worlds
            .iter()
            .find(|world| world.world_id == "mist_harbor")
            .unwrap();
        assert!(!mh.completed);
        assert!(!mh.first_completion);
        assert!(mh.completed_events.is_empty());
    }

    fn action_request(
        epoch: u64,
        request_id: u64,
        client_time_ms: u64,
        kind: ActionKind,
    ) -> ActionCommandRequest {
        ActionCommandRequest {
            protocol_version: 2,
            world_epoch: epoch,
            request_id,
            client_time_ms,
            kind,
        }
    }

    fn advance_action_inputs(
        runtime: &FormalRuntime,
        epoch: u64,
        first_seq: u64,
        last_seq: u64,
        time_offset_ms: u64,
    ) {
        for seq in first_seq..=last_seq {
            runtime
                .submit_input(
                    InputSample::new(epoch, seq, time_offset_ms + seq * 17, 0.0, 0.0).unwrap(),
                    vec![],
                )
                .unwrap();
        }
    }

    #[test]
    fn qer_and_dash_states_project_with_energy_and_start_events_in_formal_gh_and_rs_scenes() {
        for (scene_id, world_id) in [
            ("rs_test_hub", "return_station"),
            ("gh_test_power", "grey_hive"),
        ] {
            for (request_id, kind, action_key, state_name, event_kind) in [
                (101, ActionKind::Pulse, "pulse", "pulse", "PulseCast"),
                (102, ActionKind::GuardStart, "guard", "guard", "GuardWindup"),
                (103, ActionKind::Pierce, "pierce", "pierce", "PierceStarted"),
                (104, ActionKind::Dash, "dash", "dash", "DashStarted"),
            ] {
                let runtime = runtime_for_action_scene(scene_id);
                let initial = runtime.snapshot().unwrap();
                assert_eq!(initial.world_id, world_id);
                assert_eq!(initial.scene_id, scene_id);
                assert_eq!(initial.player.action_state, "idle");
                let epoch = initial.world_epoch;
                let response = runtime
                    .submit_action(action_request(epoch, request_id, 0, kind))
                    .unwrap();
                assert_eq!(response.world_epoch, epoch);
                assert_eq!(
                    response.player.action_state, state_name,
                    "scene={scene_id} action={action_key}"
                );
                assert_eq!(
                    response.player.current_energy,
                    initial.player.current_energy
                        - crate::continuous_combat::combat_v1().actions[action_key].energy,
                    "scene={scene_id} action={action_key}"
                );
                let active_request_id = runtime
                    .state
                    .lock()
                    .unwrap()
                    .world
                    .combat_state
                    .active_action
                    .as_ref()
                    .map(|action| action.request_id);
                assert_eq!(
                    active_request_id,
                    Some(request_id),
                    "scene={scene_id} action={action_key}"
                );
                let events = runtime.presentation_events_since(epoch, 0).unwrap();
                let mut starts = events.iter().filter(|event| event.kind == event_kind);
                let started = starts.next().unwrap_or_else(|| {
                    panic!("scene={scene_id} action={action_key} should emit {event_kind}")
                });
                assert!(
                    starts.next().is_none(),
                    "scene={scene_id} action={action_key} must emit exactly one {event_kind}"
                );
                assert_eq!(started.world_epoch, epoch);
                // The owner authors the event when it applies the action. The
                // caller can observe a later snapshot after another owner tick.
                assert!(
                    initial.server_tick < started.server_tick
                        && started.server_tick <= response.server_tick,
                    "scene={scene_id} action={action_key}: initial={}, start={}, response={}",
                    initial.server_tick,
                    started.server_tick,
                    response.server_tick
                );

                advance_action_inputs(&runtime, epoch, 1, 6, 0);
                let held = runtime.snapshot().unwrap();
                assert_eq!(
                    held.player.action_state, state_name,
                    "scene={scene_id} action={action_key}"
                );
            }
        }
    }

    #[test]
    fn guard_hold_release_preserves_the_active_projection_then_returns_to_idle() {
        let runtime = runtime_for_action_scene("rs_test_hub");
        let initial = runtime.snapshot().unwrap();
        let epoch = initial.world_epoch;
        let started = runtime
            .submit_action(action_request(epoch, 201, 0, ActionKind::GuardStart))
            .unwrap();
        assert_eq!(started.player.action_state, "guard");
        let energy_after_start = initial.player.current_energy
            - crate::continuous_combat::combat_v1().actions["guard"].energy;
        assert_eq!(started.player.current_energy, energy_after_start);

        advance_action_inputs(&runtime, epoch, 1, 6, 0);
        let held = runtime.snapshot().unwrap();
        assert_eq!(held.player.action_state, "guard");
        let guard_started = runtime
            .presentation_events_since(epoch, 0)
            .unwrap()
            .into_iter()
            .find(|event| event.kind == "GuardStarted")
            .expect("guard hold should emit GuardStarted after windup");
        assert_eq!(guard_started.world_epoch, epoch);

        let released = runtime
            .submit_action(action_request(epoch, 202, 200, ActionKind::GuardEnd))
            .unwrap();
        assert_eq!(released.player.action_state, "guard");
        assert_eq!(released.player.current_energy, energy_after_start);
        let guard_ended = runtime
            .presentation_events_since(epoch, guard_started.event_id)
            .unwrap()
            .into_iter()
            .find(|event| event.kind == "GuardEnded")
            .expect("guard release should emit GuardEnded");
        assert_eq!(guard_ended.world_epoch, epoch);

        advance_action_inputs(&runtime, epoch, 7, 60, 200);
        let finished = runtime.snapshot().unwrap();
        assert_eq!(finished.player.action_state, "idle");
        assert_eq!(finished.player.current_energy, energy_after_start);
    }

    #[test]
    fn stale_actions_emit_nothing_and_energy_rejection_emits_only_authority_feedback() {
        let runtime = runtime_for_action_scene("gh_test_power");
        let initial = runtime.snapshot().unwrap();
        let epoch = initial.world_epoch;
        assert_eq!(
            runtime
                .submit_action(action_request(epoch + 1, 301, 0, ActionKind::Pulse))
                .unwrap_err(),
            "E_ACTION_STALE_EPOCH"
        );
        {
            runtime.state.lock().unwrap().world.player_energy = 0;
        }
        let rejected = runtime
            .submit_action(action_request(epoch, 302, 0, ActionKind::Pulse))
            .unwrap();
        assert_eq!(rejected.player.action_state, "idle");
        assert_eq!(rejected.player.current_energy, 0);
        let events = runtime.presentation_events_since(epoch, 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "ActionRejected");
        assert_eq!(events[0].combat_feedback.as_ref().unwrap().reason.as_deref(), Some("insufficient_energy"));
        assert_eq!(events[0].combat_feedback.as_ref().unwrap().request_id, Some(302));
        assert!(runtime
            .state
            .lock()
            .unwrap()
            .world
            .combat_state
            .active_action
            .is_none());
    }

    #[test]
    fn mist_harbor_exploration_uses_center_clearance_and_selected_map_projection() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        state.world =
            WorldStateV3::new("mist_harbor", 1, Vec3::new(1.0, 0.0, 2.0).unwrap()).unwrap();
        state.capabilities.world_id = "mist_harbor".into();
        state.capabilities.explored_map = state.world.explored.clone();
        let mut scene: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/scene-runtime-v1/mist_harbor.json"
        ))
        .unwrap();
        scene["explorationRegions"] = serde_json::json!([
            {"id":"mh_ps_intake","polygon":[[0,0],[5,0],[5,4],[0,4]]},
            {"id":"mh_ps_control_hall","polygon":[[5,0],[10,0],[10,4],[5,4]]}
        ]);
        let scene: crate::scene_runtime::SceneDefinition = serde_json::from_value(scene).unwrap();

        update_mist_harbor_exploration(&mut state, &scene, true).unwrap();
        assert_eq!(
            state.world_persistent_v1.mist_harbor.explored_region_ids,
            ["mh_ps_intake"]
        );
        state.world.player.position_m = Vec3::new(5.0, 0.0, 2.0).unwrap();
        update_mist_harbor_exploration(&mut state, &scene, false).unwrap();
        assert_eq!(
            state.world_persistent_v1.mist_harbor.explored_region_ids,
            ["mh_ps_intake"]
        );
        state.world.player.position_m = Vec3::new(5.34, 0.0, 2.0).unwrap();
        update_mist_harbor_exploration(&mut state, &scene, false).unwrap();
        assert_eq!(
            state.world_persistent_v1.mist_harbor.explored_region_ids,
            ["mh_ps_intake"]
        );
        state.world.player.position_m = Vec3::new(5.35, 0.0, 2.0).unwrap();
        update_mist_harbor_exploration(&mut state, &scene, false).unwrap();
        assert!(state.world_persistent_v1.is_explored("mh_ps_control_hall"));

        let walked_ids = state
            .world_persistent_v1
            .mist_harbor
            .explored_region_ids
            .clone();
        state
            .capabilities
            .grants
            .push(crate::capability_v1::CapabilityGrant {
                capability_id: CAP_ACOUSTIC_MAPPING.into(),
                granted_at_revision: state.world.revision,
            });
        state
            .capabilities
            .selected
            .push(CAP_ACOUSTIC_MAPPING.into());
        assert!(project_view(&state)
            .capabilities
            .explored_map
            .rooms
            .is_empty());
        assert_eq!(
            state.world_persistent_v1.mist_harbor.explored_region_ids,
            walked_ids
        );
        state.capabilities.selected.clear();

        state
            .capabilities
            .grants
            .push(crate::capability_v1::CapabilityGrant {
                capability_id: CAP_LOCAL_MAP.into(),
                granted_at_revision: state.world.revision,
            });
        let unselected = project_view(&state);
        assert!(unselected.capabilities.explored_map.rooms.is_empty());
        state.capabilities.selected.push(CAP_LOCAL_MAP.into());
        let build = state.progression_v6.clone();
        state.install_build(build).unwrap();
        let selected = project_view(&state);
        assert_eq!(selected.capabilities.explored_map.rooms.len(), 2);
    }

    #[test]
    fn pump_projection_is_authoritative_and_only_present_in_mist_harbor() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        state.world = WorldStateV3::new("mist_harbor", 1, Vec3::zero()).unwrap();
        state.capabilities.world_id = "mist_harbor".into();
        for (status, expected) in [
            (crate::world_persistent_v1::PumpStatus::Ready, "ready"),
            (crate::world_persistent_v1::PumpStatus::Draining, "draining"),
            (crate::world_persistent_v1::PumpStatus::Drained, "drained"),
        ] {
            state.world_persistent_v1.mist_harbor.pump.state = status;
            let snapshot = crate::world_v3::WorldSnapshot::from_view(project_view(&state));
            assert_eq!(
                serde_json::to_value(snapshot).unwrap()["mistHarborPump"]["state"],
                expected
            );
        }
        state.world.world_id = "grey_hive".into();
        state.capabilities.world_id = "grey_hive".into();
        let snapshot = crate::world_v3::WorldSnapshot::from_view(project_view(&state));
        assert!(serde_json::to_value(snapshot)
            .unwrap()
            .get("mistHarborPump")
            .is_none());
    }

    #[test]
    fn owner_tick_completes_pump_and_retags_both_water_regions() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        state.world =
            WorldStateV3::new("mist_harbor", 1, Vec3::new(17.0, 0.0, 7.0).unwrap()).unwrap();
        state.capabilities.world_id = "mist_harbor".into();
        state.capabilities.explored_map = state.world.explored.clone();
        state.kcc = StaticKccWorld::new(
            crate::continuous_kcc::Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(),
            vec![],
        )
        .with_terrain_regions(vec![
            crate::continuous_kcc::KccTerrainRegion {
                id: crate::world_persistent_v1::PUMP_EAST_WATER_ID.into(),
                tag: crate::effects::TerrainTag::WaterDeep,
                polygon: vec![[18.0, 5.5], [23.5, 5.5], [23.5, 10.5], [18.0, 10.5]],
                surface_velocity_mps: None,
            },
            crate::continuous_kcc::KccTerrainRegion {
                id: crate::world_persistent_v1::DROWNED_QUAY_WATER_ID.into(),
                tag: crate::effects::TerrainTag::WaterShallow,
                polygon: vec![[13.0, 5.0], [19.0, 5.0], [19.0, 11.0], [13.0, 11.0]],
                surface_velocity_mps: None,
            },
        ]);
        state.world_persistent_v1.mist_harbor.pump.state =
            crate::world_persistent_v1::PumpStatus::Draining;
        state
            .world_persistent_v1
            .mist_harbor
            .pump
            .drain_complete_at_world_time_ms = Some(17);
        advance_owner_step(&mut state).unwrap();
        assert!(state.world.server_time_ms >= 17);
        assert_eq!(
            state.world_persistent_v1.mist_harbor.pump.state,
            crate::world_persistent_v1::PumpStatus::Drained
        );
        assert_eq!(
            state
                .kcc
                .terrain_tag(crate::world_persistent_v1::PUMP_EAST_WATER_ID),
            Some(crate::effects::TerrainTag::WetFloor)
        );
        assert_eq!(
            state
                .kcc
                .terrain_tag(crate::world_persistent_v1::DROWNED_QUAY_WATER_ID),
            Some(crate::effects::TerrainTag::WetFloor)
        );
    }

    #[test]
    fn authored_gh_enemy_spawns_project_and_survive_v5_continue() {
        let save_root = std::env::temp_dir().join(format!(
            "formal-actor-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let view = runtime
            .install_scene_registry(registry, "gh_central_shaft")
            .unwrap();
        assert!(view.actors.iter().any(|actor| actor.entity_type
            == "runtime2d.enemy.infected_maintenance_worker.v1"
            && actor.active));
        assert!(view.actors.iter().any(|actor| actor.entity_type
            == "runtime2d.enemy.infected_security.v1"
            && actor.active));
        let (dead_id, injured_id, before) = {
            let mut state = runtime.state.lock().unwrap();
            let revision = state.world.revision;
            assert!(matches!(
                apply_route_command(
                    &mut state.route,
                    RouteCommand::Enter {
                        world_id: "grey_hive".into(),
                        request_id: "actor-test-enter".into(),
                    },
                    revision
                ),
                RouteResult::Applied { .. }
            ));
            let dead_id = state.world.generic_actors[0].entity_id.clone();
            let injured_id = state.world.generic_actors[1].entity_id.clone();
            state.world.generic_actors[0].take_damage(999);
            state.world.generic_actors[1].take_damage(10);
            (dead_id, injured_id, state.world.generic_actors.clone())
        };
        runtime.save().unwrap();
        let stored = crate::save_v6::read_save(&save_root).unwrap().save;
        assert_eq!(stored.generic_actors, before);
        let mut legacy = serde_json::to_value(&stored).unwrap();
        legacy.as_object_mut().unwrap().remove("genericActors");
        let legacy: crate::save_v5::SaveV5 = serde_json::from_value(legacy).unwrap();
        assert!(legacy.generic_actors.is_empty());
        assert!(legacy.validate().is_ok());
        let continued = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&continued).unwrap();
        let restored = continued.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&continued, view)).unwrap();
        assert!(
            !restored
                .actors
                .iter()
                .find(|actor| actor.entity_id == dead_id)
                .unwrap()
                .active
        );
        assert!(
            restored
                .actors
                .iter()
                .find(|actor| actor.entity_id == injured_id)
                .unwrap()
                .active
        );
        assert_eq!(continued.state.lock().unwrap().world.generic_actors, before);
        let restored_legacy = legacy.restore_state().unwrap();
        let registry = continued
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let scene = SceneRuntime::new(
            registry,
            "gh_central_shaft",
            restored_legacy.world.revision.world_epoch,
        )
        .unwrap();
        let mut old_world = restored_legacy.world;
        restore_generic_actors(&mut old_world, scene.current_scene(), &mut Default::default(), false).unwrap();
        assert!(!old_world.generic_actors.is_empty());
        std::fs::remove_dir_all(save_root).unwrap();
    }

    #[test]
    fn mist_harbor_authored_actors_load_replace_and_use_generic_combat() {
        let save_root = std::env::temp_dir().join(format!(
            "formal-mh-actors-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let expected = [
            ("mh_fog_pier", 2, 0),
            ("mh_tidal_warehouse", 3, 1),
            ("mh_signal_yard", 0, 3),
            ("mh_drowned_quay", 4, 0),
            ("mh_breakwater", 0, 2),
            ("mh_pump_station", 3, 0),
            ("mh_resonance_tower", 0, 4),
        ];
        let mut previous_ids = std::collections::BTreeSet::new();
        for (scene_id, drowned_count, wraith_count) in expected {
            let view = runtime
                .install_scene_registry(registry.clone(), scene_id)
                .unwrap();
            assert_eq!(view.scene_id, scene_id);
            let drowned = view
                .actors
                .iter()
                .filter(|actor| actor.entity_type == "runtime2d.enemy.mistharbor.drowned.v2")
                .collect::<Vec<_>>();
            let wraith = view
                .actors
                .iter()
                .filter(|actor| actor.entity_type == "enemy.mist_harbor.signal_wraith")
                .collect::<Vec<_>>();
            assert_eq!(drowned.len(), drowned_count, "{scene_id} Drowned roster");
            assert_eq!(wraith.len(), wraith_count, "{scene_id} Wraith roster");
            assert!(view.actors.iter().all(|actor| actor.active));
            let current_ids = view
                .actors
                .iter()
                .map(|actor| actor.entity_id.clone())
                .collect::<std::collections::BTreeSet<_>>();
            assert!(
                current_ids.is_disjoint(&previous_ids),
                "stale actors in {scene_id}"
            );
            previous_ids = current_ids;
        }

        let view = runtime
            .install_scene_registry(registry, "mh_fog_pier")
            .unwrap();
        assert_eq!(view.actors.len(), 2);
        let mut combat_world = runtime.state.lock().unwrap().world.clone();
        let mut drowned = combat_world.generic_actors[0].clone();
        drowned.position_m = crate::world_v3::Vec3::new(5.8, 0.0, 5.0).unwrap();
        drowned.home_m = drowned.position_m;
        combat_world.generic_actors = vec![drowned];
        combat_world.player.position_m = crate::world_v3::Vec3::new(5.0, 0.0, 5.0).unwrap();
        let kcc = crate::continuous_kcc::StaticKccWorld::new(
            crate::continuous_kcc::Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(),
            vec![],
        );
        for seq in 1..=3 {
            let sample = crate::continuous_input::InputSample::new(1, seq, seq, 0.0, 0.0).unwrap();
            let output = crate::world_v3::step_world(
                &mut combat_world,
                &kcc,
                crate::world_v3::CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[crate::continuous_combat::CombatIntent::Attack { request_id: seq }],
                },
            )
            .unwrap();
            assert!(output.combat.iter().any(|event| matches!(
                event,
                crate::continuous_combat::CombatEvent::AttackHit { target_id, .. }
                    if target_id == &combat_world.generic_actors[0].entity_id
            )));
        }
        assert_eq!(combat_world.generic_actors[0].hp, 0);
        assert!(!combat_world.generic_actors[0].view().active);
        if save_root.exists() {
            std::fs::remove_dir_all(save_root).unwrap();
        }
    }

    #[test]
    fn unknown_authored_enemy_profile_cannot_enter_runtime() {
        let mut scene: crate::scene_runtime::SceneDefinition = serde_json::from_str(include_str!(
            "../../content/scenes/compiled/gh_entry_maintenance.json"
        ))
        .unwrap();
        let enemy = scene
            .spawns
            .iter_mut()
            .find(|spawn| spawn.kind == "enemy")
            .unwrap();
        enemy.entity_type = Some("grey_hive.unknown_enemy".into());
        assert_eq!(
            spawn_generic_actors(&scene).unwrap_err(),
            "E_ACTOR_PROFILE_UNKNOWN"
        );
    }

    #[test]
    fn clockworks_scenes_preserve_guards_and_admit_exact_authored_ordinary_enemies() {
        let roster = [
            ("cw_entry_foundry", 2, 0, ""),
            ("cw_pressure_hall", 2, 2, clockworks_roster::DRONE),
            ("cw_conveyor_bridge", 3, 2, clockworks_roster::DRONE),
            ("cw_boiler_chamber", 0, 3, clockworks_roster::HOUND),
            ("cw_gear_shaft", 2, 3, clockworks_roster::DRONE),
            ("cw_furnace_heart", 2, 3, clockworks_roster::HOUND),
        ];
        for (scene_id, guard_count, added_count, added_type) in roster {
            let path = format!("../content/scenes/compiled/{scene_id}.json");
            let bytes = std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path),
            )
            .unwrap();
            let scene: crate::scene_runtime::SceneDefinition =
                serde_json::from_str(&bytes).unwrap();
            assert_eq!(scene.scene_id, scene_id);
            let actors = spawn_generic_actors(&scene).unwrap();
            assert_eq!(actors.len(), guard_count + added_count, "{scene_id} roster");
            assert!(actors[guard_count..].iter().all(|actor| actor.entity_type == added_type
                && actor.view().entity_type == added_type && actor.ordinary.is_some() && actor.validate()));
            assert!(actors[..guard_count].iter().all(|actor| {
                actor.entity_type == "enemy.clockworks.forged_guard"
                    && actor.view().entity_type == "runtime2d.enemy.clockworks.forged_guard.v2"
                    && (3.0..=12.0).contains(&actor.home_m.x_m)
                    && actor.home_m.z_m == 8.0
            }));
        }
        let regulator_scene: crate::scene_runtime::SceneDefinition = serde_json::from_str(
            include_str!("../../content/scenes/compiled/cw_regulator_core.json"),
        )
        .unwrap();
        let regulators = spawn_generic_actors(&regulator_scene).unwrap();
        assert_eq!(regulators.len(), 1);
        assert_eq!(regulators[0].entity_id, CLOCKWORKS_REGULATOR_SPAWN_ID);
        assert_eq!(regulators[0].entity_type, CLOCKWORKS_REGULATOR_ENTITY_TYPE);
        assert_eq!(regulators[0].hp, 600);
        assert!(regulators[0].view().active);
        assert_eq!(
            regulators[0].view().entity_type,
            "runtime2d.enemy.clockworks.forged_guard.v2"
        );
        let staged_scene: crate::scene_runtime::SceneDefinition = serde_json::from_str(
            include_str!("../../content/scenes/compiled/cw_entry_foundry.json"),
        )
        .unwrap();
        assert_eq!(staged_scene.world_id, "clockworks");
    }

    fn acoustic_fixture(label: &str) -> (FormalRuntime, std::path::PathBuf) {
        let save_root = std::env::temp_dir().join(format!(
            "formal-acoustic-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let scene = SceneRuntime::new(registry, "mh_resonance_tower", 1).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            let revision = state.world.revision;
            for (index, event_id) in ["hive_power", "hive_lockdown", "hive_extraction"]
                .iter()
                .enumerate()
            {
                assert!(matches!(
                    apply_route_command(
                        &mut state.route,
                        RouteCommand::Progress {
                            event_id: (*event_id).into(),
                            request_id: format!("setup-gh-{index}")
                        },
                        revision,
                    ),
                    RouteResult::Applied { .. }
                ));
            }
            assert!(matches!(
                apply_route_command(
                    &mut state.route,
                    RouteCommand::Complete {
                        world_id: "grey_hive".into(),
                        request_id: "setup-gh-complete".into()
                    },
                    revision,
                ),
                RouteResult::Applied { .. }
            ));
            assert!(matches!(
                apply_route_command(
                    &mut state.route,
                    RouteCommand::Enter {
                        world_id: "mist_harbor".into(),
                        request_id: "setup-mh-enter".into()
                    },
                    revision,
                ),
                RouteResult::Applied { .. }
            ));
            let mut mist_world =
                WorldStateV3::new("mist_harbor", 1, Vec3::new(16.0, 0.0, 8.0).unwrap()).unwrap();
            mist_world.revision = revision;
            state.world = mist_world;
            state.world.scene_id = "mh_resonance_tower".into();
            state.kcc = scene_kcc(scene.current_scene(), &BTreeSet::new()).unwrap();
            state.capabilities.world_id = "mist_harbor".into();
            state.capabilities.explored_map = state.world.explored.clone();
        }
        *runtime.scene_runtime.lock().unwrap() = Some(scene);
        (runtime, save_root)
    }

    #[test]
    fn authored_signal_ping_is_epoch_scoped_and_only_successful_interaction_emits() {
        let (runtime, save_root) = acoustic_fixture("sound-authority");
        let epoch = runtime.snapshot().unwrap().world_epoch;
        assert!(runtime.sound_cues_since(epoch, 0).unwrap().is_empty());
        assert_eq!(
            runtime.sound_cues_since(epoch + 1, 0).unwrap_err(),
            "E_SOUND_CUE_STALE_EPOCH"
        );
        let applied = runtime
            .activate_scene_interaction("mh_signal_console_staged", "sound-earned", epoch)
            .unwrap();
        assert!(applied.applied);
        let cues = runtime.sound_cues_since(epoch, 0).unwrap();
        assert_eq!(cues.len(), 1);
        let cue = &cues[0];
        assert_eq!(
            (cue.protocol_version, cue.event_id, cue.world_epoch),
            (1, 1, epoch)
        );
        assert_eq!(cue.kind, "signal_ping");
        assert_eq!(
            (cue.world_id.as_str(), cue.scene_id.as_str()),
            ("mist_harbor", "mh_resonance_tower")
        );
        assert_eq!(cue.direction_rad, Some(0.0));
        assert_eq!(cue.distance_m, Some(0.0));
        let wire = serde_json::to_value(cue).unwrap();
        assert_eq!(wire["protocolVersion"], 1);
        assert!(wire.get("positionM").is_none());
        assert!(wire.get("sourceId").is_none());
        let repeated = runtime
            .activate_scene_interaction("mh_signal_console_staged", "sound-repeated", epoch)
            .unwrap();
        assert!(!repeated.applied);
        assert_eq!(runtime.sound_cues_since(epoch, 0).unwrap(), cues);
        assert!(runtime
            .sound_cues_since(epoch, cue.event_id)
            .unwrap()
            .is_empty());
        assert_eq!(
            runtime
                .sound_cues_since(epoch, cue.event_id + 1)
                .unwrap_err(),
            "E_SOUND_CUE_CURSOR_AHEAD"
        );
        runtime.save().unwrap();
        let restored = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&restored).unwrap();
        let restored_view = restored.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&restored, view)).unwrap();
        assert!(restored
            .sound_cues_since(restored_view.world_epoch, 0)
            .unwrap()
            .is_empty());
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn beacon_signal_ping_uses_authored_position_and_freezes_mapping_at_emission() {
        let (runtime, save_root) = acoustic_fixture("sound-beacon");
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let warehouse = SceneRuntime::new(registry, "mh_tidal_warehouse", 1).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.scene_id = "mh_tidal_warehouse".into();
            state.world.player.position_m = Vec3::new(16.0, 0.0, 10.0).unwrap();
            state.kcc = scene_kcc(warehouse.current_scene(), &BTreeSet::new()).unwrap();
        }
        *runtime.scene_runtime.lock().unwrap() = Some(warehouse);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let marker = runtime
            .activate_scene_interaction("mh_west_beacon_log_marker", "sound-static", epoch)
            .unwrap();
        assert!(marker.applied);
        assert!(runtime.sound_cues_since(epoch, 0).unwrap().is_empty());
        let applied = runtime
            .activate_scene_interaction("mh_west_beacon", "sound-west", epoch)
            .unwrap();
        assert!(applied.applied, "{:?}", applied.error_code);
        let cues = runtime.sound_cues_since(epoch, 0).unwrap();
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].scene_id, "mh_tidal_warehouse");
        assert_eq!(cues[0].direction_rad, None);
        assert_eq!(cues[0].distance_m, None);
        // Later selection cannot expose the location of a cue already emitted.
        {
            let mut state = runtime.state.lock().unwrap();
            let revision = state.world.revision;
            state
                .capabilities
                .grants
                .push(crate::capability_v1::CapabilityGrant {
                    capability_id: CAP_ACOUSTIC_MAPPING.into(),
                    granted_at_revision: revision,
                });
            state
                .capabilities
                .selected
                .push(CAP_ACOUSTIC_MAPPING.into());
            let build = state.progression_v6.clone();
            state.install_build(build).unwrap();
        }
        assert_eq!(runtime.sound_cues_since(epoch, 0).unwrap(), cues);
        let duplicate = runtime
            .activate_scene_interaction("mh_west_beacon", "sound-west-again", epoch)
            .unwrap();
        assert!(!duplicate.applied);
        assert_eq!(runtime.sound_cues_since(epoch, 0).unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(save_root);

        let (mapped_runtime, mapped_root) = acoustic_fixture("sound-bearing");
        let registry = mapped_runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let warehouse = SceneRuntime::new(registry, "mh_tidal_warehouse", 1).unwrap();
        {
            let mut state = mapped_runtime.state.lock().unwrap();
            state.world.scene_id = "mh_tidal_warehouse".into();
            state.world.player.position_m = Vec3::new(16.0, 0.0, 10.0).unwrap();
            state.kcc = scene_kcc(warehouse.current_scene(), &BTreeSet::new()).unwrap();
            let revision = state.world.revision;
            state
                .capabilities
                .grants
                .push(crate::capability_v1::CapabilityGrant {
                    capability_id: CAP_ACOUSTIC_MAPPING.into(),
                    granted_at_revision: revision,
                });
            state
                .capabilities
                .selected
                .push(CAP_ACOUSTIC_MAPPING.into());
            let build = state.progression_v6.clone();
            state.install_build(build).unwrap();
        }
        *mapped_runtime.scene_runtime.lock().unwrap() = Some(warehouse);
        let mapped_epoch = mapped_runtime.snapshot().unwrap().world_epoch;
        assert!(
            mapped_runtime
                .activate_scene_interaction("mh_west_beacon", "sound-mapped-west", mapped_epoch,)
                .unwrap()
                .applied
        );
        let mapped_cue = mapped_runtime
            .sound_cues_since(mapped_epoch, 0)
            .unwrap()
            .remove(0);
        assert!((mapped_cue.direction_rad.unwrap() - std::f32::consts::FRAC_PI_2).abs() < 0.0001);
        assert_eq!(mapped_cue.distance_m, Some(1.0));
        let _ = std::fs::remove_dir_all(mapped_root);

        let (east_runtime, east_root) = acoustic_fixture("sound-east");
        let registry = east_runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let breakwater = SceneRuntime::new(registry, "mh_breakwater", 1).unwrap();
        {
            let mut state = east_runtime.state.lock().unwrap();
            state.world.scene_id = "mh_breakwater".into();
            state.world.player.position_m = Vec3::new(17.0, 0.0, 8.0).unwrap();
            state.kcc = scene_kcc(breakwater.current_scene(), &BTreeSet::new()).unwrap();
        }
        *east_runtime.scene_runtime.lock().unwrap() = Some(breakwater);
        let east_epoch = east_runtime.snapshot().unwrap().world_epoch;
        let east_applied = east_runtime
            .activate_scene_interaction("mh_east_beacon", "sound-east-beacon", east_epoch)
            .unwrap();
        assert!(east_applied.applied, "{:?}", east_applied.error_code);
        let east_cues = east_runtime.sound_cues_since(east_epoch, 0).unwrap();
        assert_eq!(east_cues.len(), 1);
        assert_eq!(east_cues[0].scene_id, "mh_breakwater");
        let _ = std::fs::remove_dir_all(east_root);
    }

    #[test]
    fn sound_cue_queue_detects_gaps_and_resets_with_new_epoch() {
        let (runtime, save_root) = acoustic_fixture("sound-gap");
        let epoch = runtime.snapshot().unwrap().world_epoch;
        {
            let mut state = runtime.state.lock().unwrap();
            for id in 1..=257 {
                state.sound_cues.push(SoundCueEvent {
                    protocol_version: 1,
                    event_id: id,
                    world_epoch: epoch,
                    server_tick: 0,
                    world_id: "mist_harbor".into(),
                    scene_id: "mh_resonance_tower".into(),
                    kind: "signal_ping".into(),
                    direction_rad: None,
                    distance_m: None,
                });
            }
            state.sound_cues.remove(0);
            state.next_sound_cue_id = 258;
        }
        assert_eq!(
            runtime.sound_cues_since(epoch, 0).unwrap_err(),
            "E_SOUND_CUE_GAP_RESYNC_REQUIRED"
        );
        assert_eq!(runtime.sound_cues_since(epoch, 1).unwrap().len(), 256);
        let fresh = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(fresh.world_epoch > epoch);
        assert_eq!(
            runtime.sound_cues_since(epoch, 0).unwrap_err(),
            "E_SOUND_CUE_STALE_EPOCH"
        );
        assert!(runtime
            .sound_cues_since(fresh.world_epoch, 0)
            .unwrap()
            .is_empty());
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn authored_mist_signal_grants_selected_acoustic_mapping_once_and_survives_continue() {
        let (runtime, save_root) = acoustic_fixture("roundtrip");
        let before = runtime.snapshot().unwrap();
        let unearned = runtime.apply_capability_command(CapabilityCommandRequest::Select {
            capability_ids: vec![CAP_ACOUSTIC_MAPPING.into()],
        });
        assert!(
            matches!(&unearned, Err(error) if error.contains("NotGranted")),
            "unexpected capability command result: {unearned:?}"
        );
        assert_eq!(
            runtime.snapshot().unwrap().progression.event_seq,
            before.progression.event_seq
        );

        let applied = runtime
            .activate_scene_interaction(
                "mh_signal_console_staged",
                "acoustic-earned",
                before.world_epoch,
            )
            .unwrap();
        assert!(applied.applied, "{:?}", applied.error_code);
        assert_eq!(
            applied.view.progression.event_seq,
            before.progression.event_seq + 1
        );
        assert!(applied
            .view
            .capabilities
            .items
            .iter()
            .any(|item| item.capability_id == CAP_ACOUSTIC_MAPPING
                && item.granted
                && item.selected));
        assert!(runtime
            .state
            .lock()
            .unwrap()
            .capabilities
            .first_enhancement_choice
            .is_none());
        let duplicate = runtime
            .activate_scene_interaction(
                "mh_signal_console_staged",
                "acoustic-repeated",
                before.world_epoch,
            )
            .unwrap();
        assert!(!duplicate.applied);
        assert_eq!(
            duplicate.view.progression.event_seq,
            applied.view.progression.event_seq
        );
        assert_eq!(
            duplicate.view.capabilities.items,
            applied.view.capabilities.items
        );

        runtime.save().unwrap();
        let saved = crate::save_v6::read_save(&save_root).unwrap().save;
        assert!(saved
            .capabilities
            .grants
            .iter()
            .any(|grant| grant.capability_id == CAP_ACOUSTIC_MAPPING));
        let mut forged = saved.clone();
        forged
            .progression
            .progress
            .iter_mut()
            .find(|progress| progress.world_id == "mist_harbor")
            .unwrap()
            .completed_events
            .retain(|event| event != "mist_signal");
        assert_eq!(
            forged.validate().unwrap_err(),
            "E_SAVE_CAPABILITY_ROUTE_INVALID"
        );

        let continued = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&continued).unwrap();
        let restored = continued.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&continued, view)).unwrap();
        assert_eq!(restored.scene_id, "mh_resonance_tower");
        assert!(restored
            .capabilities
            .items
            .iter()
            .any(|item| item.capability_id == CAP_ACOUSTIC_MAPPING
                && item.granted
                && item.selected));
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn acoustic_grant_rejects_wrong_source_or_world_without_partial_route_commit() {
        let (runtime, save_root) = acoustic_fixture("invalid-source");
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let tower = SceneRuntime::new(registry.clone(), "mh_resonance_tower", 1).unwrap();
        let yard = SceneRuntime::new(registry, "mh_signal_yard", 1).unwrap();
        let mut state = runtime.state.lock().unwrap().clone();
        let original_route = state.route.clone();
        let original_caps = state.capabilities.clone();
        let forged = [SceneEvent::Interaction {
            id: "other_console".into(),
            event_id: Some("mist_signal".into()),
        }];
        assert_eq!(
            apply_scene_progress(&mut state, &tower, &forged, "forged").unwrap_err(),
            "E_ACOUSTIC_MAPPING_SOURCE_INVALID"
        );
        assert_eq!(state.route, original_route);
        assert_eq!(state.capabilities, original_caps);
        let wrong_event = [SceneEvent::Interaction {
            id: "mh_signal_console_staged".into(),
            event_id: Some("mist_beacon_west".into()),
        }];
        let mut wrong_event_state = state.clone();
        apply_scene_progress(&mut wrong_event_state, &tower, &wrong_event, "wrong-event").unwrap();
        assert_eq!(wrong_event_state.capabilities, original_caps);
        let exact = [SceneEvent::Interaction {
            id: "mh_signal_console_staged".into(),
            event_id: Some("mist_signal".into()),
        }];
        let mut other_scene_state = state.clone();
        apply_scene_progress(&mut other_scene_state, &yard, &exact, "other-scene").unwrap();
        assert!(
            route_completed_events(&other_scene_state.route, "mist_harbor").contains("mist_signal")
        );
        assert_eq!(other_scene_state.capabilities, original_caps);
        assert_eq!(state.route, original_route);
        state.route.current_world_id = "grey_hive".into();
        assert_eq!(
            apply_scene_progress(&mut state, &tower, &exact, "not-mh").unwrap_err(),
            "E_ACOUSTIC_MAPPING_SOURCE_INVALID"
        );
        assert_eq!(state.capabilities, original_caps);
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn mist_signal_route_request_collision_keeps_scene_and_capability_unapplied() {
        let (runtime, save_root) = acoustic_fixture("route-collision");
        {
            let mut state = runtime.state.lock().unwrap();
            let revision = state.world.revision;
            assert!(matches!(
                apply_route_command(
                    &mut state.route,
                    RouteCommand::Progress {
                        event_id: "mist_beacon_west".into(),
                        request_id: "scene-event-collision".into(),
                    },
                    revision,
                ),
                RouteResult::Applied { .. }
            ));
        }
        let before = runtime.snapshot().unwrap();
        let rejected = runtime
            .activate_scene_interaction("mh_signal_console_staged", "collision", before.world_epoch)
            .unwrap();
        assert!(!rejected.applied);
        assert_eq!(
            rejected.error_code.as_deref(),
            Some("E_SCENE_PROGRESS_DUPLICATE")
        );
        assert_eq!(
            rejected.view.progression.event_seq,
            before.progression.event_seq
        );
        assert_eq!(rejected.view.capabilities.items, before.capabilities.items);
        assert_eq!(
            runtime.snapshot().unwrap().interactables,
            before.interactables
        );
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn authored_signal_interference_projects_authoritative_occupancy_only_in_signal_yard() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let signal_yard = SceneRuntime::new(registry.clone(), "mh_signal_yard", 1).unwrap();
        let drowned_quay = SceneRuntime::new(registry, "mh_drowned_quay", 1).unwrap();
        let authored = signal_yard.current_scene();
        let zone = authored
            .hazards
            .iter()
            .find(|hazard| hazard.id == "mh_signal_interference_region")
            .unwrap();
        assert_eq!(authored.world_id, "mist_harbor");
        assert_eq!(zone.kind, "signal_interference_zone");
        assert_eq!(
            zone.polygon,
            vec![[13.0, 5.0], [19.0, 5.0], [19.0, 10.0], [13.0, 10.0]]
        );

        let mut state = runtime.state.lock().unwrap();
        for (x, z, expected) in [
            (12.9, 7.5, false),
            (13.0, 7.5, true),
            (16.0, 7.5, true),
            (19.0, 10.0, true),
            (19.1, 10.0, false),
        ] {
            state.world.player.position_m = Vec3 {
                x_m: x,
                y_m: 0.0,
                z_m: z,
            };
            let view = project_scene_view(&state, Some(&signal_yard));
            let projected = view
                .hazards
                .iter()
                .find(|hazard| hazard.entity_id == zone.id)
                .unwrap();
            assert_eq!(projected.active, expected, "player center at ({x}, {z})");
            assert_eq!(projected.transform.position_m.x_m, 16.0);
            assert_eq!(projected.transform.position_m.z_m, 7.5);
        }

        state.world.player.position_m = Vec3 {
            x_m: 0.5,
            y_m: 0.0,
            z_m: 0.5,
        };
        let quay_view = project_scene_view(&state, Some(&drowned_quay));
        assert!(quay_view
            .hazards
            .iter()
            .any(|hazard| { hazard.entity_id == "mh_water_depth_region" && !hazard.active }));
    }

    #[test]
    fn authored_drowned_quay_water_view_tracks_player_center_and_exact_hazard_identity() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let registry = runtime
            .scene_registry
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        let drowned_quay = SceneRuntime::new(registry, "mh_drowned_quay", 1).unwrap();
        let authored = drowned_quay.current_scene();
        let water = authored
            .hazards
            .iter()
            .find(|hazard| hazard.id == "mh_water_depth_region")
            .unwrap();
        assert_eq!(authored.world_id, "mist_harbor");
        assert_eq!(water.kind, "water_depth_slowdown");
        assert_eq!(
            water.polygon,
            vec![[13.0, 5.0], [19.0, 5.0], [19.0, 11.0], [13.0, 11.0]]
        );

        let mut state = runtime.state.lock().unwrap();
        for (x, z, expected) in [
            (0.5, 0.5, false),
            (12.9, 8.0, false),
            (13.0, 8.0, true),
            (15.0, 8.0, true),
            (19.0, 11.0, true),
            (19.1, 11.0, false),
        ] {
            state.world.player.position_m = Vec3 {
                x_m: x,
                y_m: 0.0,
                z_m: z,
            };
            let view = project_scene_view(&state, Some(&drowned_quay));
            let projected = view
                .hazards
                .iter()
                .find(|hazard| hazard.entity_id == water.id)
                .unwrap();
            assert_eq!(projected.active, expected, "player center at ({x}, {z})");
        }

        let position = Vec3 {
            x_m: 15.0,
            y_m: 0.0,
            z_m: 8.0,
        };
        assert_eq!(
            drowned_quay_water_occupancy("clockworks", "mh_drowned_quay", water, position),
            None
        );
        assert_eq!(
            drowned_quay_water_occupancy("mist_harbor", "mh_signal_yard", water, position),
            None
        );
        let mut wrong_id = water.clone();
        wrong_id.id = "other_water_region".into();
        assert_eq!(
            drowned_quay_water_occupancy("mist_harbor", "mh_drowned_quay", &wrong_id, position),
            None
        );
        let mut wrong_kind = water.clone();
        wrong_kind.kind = "signal_interference_zone".into();
        assert_eq!(
            drowned_quay_water_occupancy("mist_harbor", "mh_drowned_quay", &wrong_kind, position),
            None
        );
    }

    #[test]
    fn authored_drowned_quay_water_depth_reaches_authoritative_kcc() {
        let scene: crate::scene_runtime::SceneDefinition = serde_json::from_str(include_str!(
            "../../content/scenes/compiled/mh_drowned_quay.json"
        ))
        .unwrap();
        assert!(scene.hazards.iter().any(|hazard| {
            hazard.id == "mh_water_depth_region" && hazard.kind == "water_depth_slowdown"
        }));
        let world = scene_kcc(&scene, &std::collections::BTreeSet::new()).unwrap();
        let mut body = crate::continuous_kcc::KccBody::new(
            crate::world_v3::Vec3::new(15.0, 0.0, 8.0).unwrap(),
        );
        crate::continuous_kcc::step_kcc(&mut body, &world, (1.0, 0.0), 0.05).unwrap();
        assert_eq!(body.velocity_mps.x_m, 2.4);
        let mut dry_body = crate::continuous_kcc::KccBody::new(
            crate::world_v3::Vec3::new(10.0, 0.0, 8.0).unwrap(),
        );
        crate::continuous_kcc::step_kcc(&mut dry_body, &world, (1.0, 0.0), 0.05).unwrap();
        assert_eq!(dry_body.velocity_mps.x_m, 4.0);
    }

    #[test]
    fn return_station_rest_terminal_is_gated_replay_safe_and_persists_healed_vitals() {
        let save_root = std::env::temp_dir().join(format!(
            "formal-rest-terminal-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let fresh = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert_eq!(fresh.scene_id, "rs_core_room");
        assert_eq!(fresh.world_id, "return_station");
        let terminal_id = "rs_save_rest_terminal_marker";

        let out_of_range = crate::scene_route_commands::save_rest_terminal(
            &runtime,
            terminal_id,
            "rest-out-of-range",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!out_of_range.applied);
        assert_eq!(
            out_of_range.error_code.as_deref(),
            Some("E_REST_TERMINAL_OUT_OF_RANGE")
        );

        let stale = crate::scene_route_commands::save_rest_terminal(
            &runtime,
            terminal_id,
            "rest-stale-epoch",
            fresh.world_epoch + 1,
        )
        .unwrap();
        assert!(!stale.applied);
        assert_eq!(
            stale.error_code.as_deref(),
            Some("E_REST_TERMINAL_STALE_EPOCH")
        );

        runtime.pause().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3 {
                x_m: 8.0,
                y_m: 0.0,
                z_m: 12.0,
            };
            state.world.player_hp = 37;
            state.world.player_energy = 21;
        }
        let paused = crate::scene_route_commands::save_rest_terminal(
            &runtime,
            terminal_id,
            "rest-paused",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!paused.applied);
        assert_eq!(paused.error_code.as_deref(), Some("E_RUNTIME_PAUSED"));

        runtime.resume().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state
                .world
                .sentinels
                .push(crate::sentinel_ai::Sentinel::new(
                    "test-sentinel",
                    crate::world_v3::Vec3::zero(),
                    10,
                ));
        }
        let combat = crate::scene_route_commands::save_rest_terminal(
            &runtime,
            terminal_id,
            "rest-combat",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!combat.applied);
        assert_eq!(
            combat.error_code.as_deref(),
            Some("E_REST_TERMINAL_COMBAT_ACTIVE")
        );
        runtime.state.lock().unwrap().world.sentinels.clear();

        let rested = crate::scene_route_commands::save_rest_terminal(
            &runtime,
            terminal_id,
            "rest-success",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(rested.applied, "{:?}", rested.error_code);
        assert_eq!(
            rested.snapshot.player.current_hp,
            rested.snapshot.player.max_hp
        );
        assert_eq!(
            rested.snapshot.player.current_energy,
            rested.snapshot.player.max_energy
        );
        let persisted = crate::save_v6::read_save(&save_root).unwrap().save;
        assert_eq!(persisted.player.current_hp, persisted.player.max_hp);
        assert_eq!(persisted.player.current_energy, persisted.player.max_energy);
        assert_eq!(persisted.scene_id, "rs_core_room");

        let replay = crate::scene_route_commands::save_rest_terminal(
            &runtime,
            terminal_id,
            "rest-success",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!replay.applied);
        assert!(replay
            .error_code
            .as_deref()
            .unwrap()
            .contains("DuplicateRequest"));

        drop(runtime);
        let continued = FormalRuntime::new_with_save_dir(save_root.clone()).unwrap();
        crate::production_scene_bootstrap::install(&continued).unwrap();
        let restored = continued.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&continued, view)).unwrap();
        assert_eq!(restored.scene_id, "rs_core_room");
        assert_eq!(restored.player.current_hp, restored.player.max_hp);
        assert_eq!(restored.player.current_energy, restored.player.max_energy);
        let old_epoch = crate::scene_route_commands::save_rest_terminal(
            &continued,
            terminal_id,
            "rest-old-session-epoch",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!old_epoch.applied);
        assert_eq!(
            old_epoch.error_code.as_deref(),
            Some("E_REST_TERMINAL_STALE_EPOCH")
        );

        {
            let mut state = continued.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3 {
                x_m: 20.0,
                y_m: 0.0,
                z_m: 8.0,
            };
        }
        let entered_gh = continued
            .use_world_gate(
                "rs_world_gate_to_gh",
                "rest-security-enter-gh",
                restored.world_epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&continued, view))
            .unwrap();
        assert_eq!(entered_gh.world_id, "grey_hive");
        let other_world = crate::scene_route_commands::save_rest_terminal(
            &continued,
            terminal_id,
            "rest-other-world",
            entered_gh.world_epoch,
        )
        .unwrap();
        assert!(!other_world.applied);
        assert_eq!(
            other_world.error_code.as_deref(),
            Some("E_REST_TERMINAL_UNAVAILABLE")
        );
        drop(continued);
        let _ = std::fs::remove_dir_all(save_root);
    }

    #[test]
    fn return_station_mission_terminal_reads_authoritative_route_without_mutating_it() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let fresh = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        let terminal_id = "rs_mission_terminal_marker";
        let out_of_range = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-out-of-range",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!out_of_range.applied);
        assert_eq!(
            out_of_range.error_code.as_deref(),
            Some("E_MISSION_TERMINAL_OUT_OF_RANGE")
        );
        let stale = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-stale-epoch",
            fresh.world_epoch + 1,
        )
        .unwrap();
        assert!(!stale.applied);
        assert_eq!(
            stale.error_code.as_deref(),
            Some("E_MISSION_TERMINAL_STALE_EPOCH")
        );

        runtime.pause().unwrap();
        let paused = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-paused",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!paused.applied);
        assert_eq!(paused.error_code.as_deref(), Some("E_RUNTIME_PAUSED"));
        runtime.resume().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3 {
                x_m: 8.0,
                y_m: 0.0,
                z_m: 4.0,
            };
        }

        let before = runtime.snapshot().unwrap();
        let status = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-status-once",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(status.applied);
        assert_eq!(status.command_id, "mission-status-once");
        assert_eq!(status.snapshot.progression, before.progression);
        assert_eq!(status.snapshot.progression.worlds.len(), 3);
        let gh = status
            .snapshot
            .progression
            .worlds
            .iter()
            .find(|world| world.world_id == "grey_hive")
            .unwrap();
        assert!(!gh.completed);
        assert!(!gh.first_completion);
        assert!(status.snapshot.interactables.iter().any(|item| {
            item.entity_id == "rs_world_gate_marker" && item.kind == "world_gate" && item.active
        }));
        let native_gates: Vec<_> = status
            .snapshot
            .interactables
            .iter()
            .filter(|item| item.kind == "world_gate")
            .map(|item| item.entity_id.as_str())
            .collect();
        assert_eq!(native_gates, vec!["rs_world_gate_marker"]);
        for marker_id in ["rs_gh_entry_marker", "rs_storage_inventory_marker"] {
            assert!(status
                .snapshot
                .interactables
                .iter()
                .any(|item| item.entity_id == marker_id && item.kind.ends_with("_marker")));
        }
        assert!(status.snapshot.interactables.iter().any(|item| {
            item.entity_id == "rs_capability_terminal_marker" && item.kind == "capability_terminal"
        }));
        assert!(status.snapshot.interactables.iter().any(|item| {
            item.entity_id == "rs_save_rest_terminal_marker" && item.kind == "save_rest_terminal"
        }));

        let replay = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-status-once",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!replay.applied);
        assert!(replay
            .error_code
            .as_deref()
            .unwrap()
            .contains("DuplicateRequest"));
        let next_journey = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        let old_session = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-old-session",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!old_session.applied);
        assert_eq!(
            old_session.error_code.as_deref(),
            Some("E_MISSION_TERMINAL_STALE_EPOCH")
        );
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3 {
                x_m: 20.0,
                y_m: 0.0,
                z_m: 8.0,
            };
        }
        let in_grey_hive = runtime
            .use_world_gate(
                "rs_world_gate_to_gh",
                "mission-enter-gh",
                next_journey.world_epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        let other_world = crate::scene_route_commands::mission_terminal_status(
            &runtime,
            terminal_id,
            "mission-other-world",
            in_grey_hive.world_epoch,
        )
        .unwrap();
        assert!(!other_world.applied);
        assert_eq!(
            other_world.error_code.as_deref(),
            Some("E_MISSION_TERMINAL_UNAVAILABLE")
        );
    }

    #[test]
    fn return_station_capability_terminal_is_read_only_epoch_bound_and_projection_backed() {
        let runtime = FormalRuntime::new().unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let fresh = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        let terminal_id = "rs_capability_terminal_marker";
        let initial = runtime.snapshot().unwrap();
        assert!(initial.interactables.iter().any(|item| {
            item.entity_id == terminal_id && item.kind == "capability_terminal" && item.active
        }));
        assert!(initial
            .capabilities
            .items
            .iter()
            .all(|item| !item.granted && !item.selected));

        let out_of_range = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-out-of-range",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!out_of_range.applied);
        assert_eq!(
            out_of_range.error_code.as_deref(),
            Some("E_CAPABILITY_TERMINAL_OUT_OF_RANGE")
        );
        let stale = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-stale-epoch",
            fresh.world_epoch + 1,
        )
        .unwrap();
        assert!(!stale.applied);
        assert_eq!(
            stale.error_code.as_deref(),
            Some("E_CAPABILITY_TERMINAL_STALE_EPOCH")
        );

        runtime.pause().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3 {
                x_m: 15.0,
                y_m: 0.0,
                z_m: 4.0,
            };
        }
        let paused = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-paused",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!paused.applied);
        assert_eq!(paused.error_code.as_deref(), Some("E_RUNTIME_PAUSED"));
        runtime.resume().unwrap();

        let before = runtime.snapshot().unwrap();
        let unearned = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-unearned",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(unearned.applied);
        assert_eq!(unearned.snapshot.capabilities, before.capabilities);
        assert!(unearned
            .snapshot
            .capabilities
            .items
            .iter()
            .all(|item| !item.granted && !item.selected));
        assert!(!unearned.snapshot.progression.worlds.iter().any(|world| {
            world.world_id == "grey_hive" && world.completed && world.first_completion
        }));

        {
            let mut state = runtime.state.lock().unwrap();
            let revision = state.world.revision;
            let grey_hive = state
                .route
                .progress
                .iter_mut()
                .find(|progress| progress.world_id == "grey_hive")
                .unwrap();
            grey_hive.completed = true;
            grey_hive.first_completion = true;
            state
                .capabilities
                .grants
                .push(crate::capability_v1::CapabilityGrant {
                    capability_id: crate::capability_v1::CAP_LOCAL_MAP.into(),
                    granted_at_revision: revision,
                });
            state.capabilities.selected = vec![crate::capability_v1::CAP_LOCAL_MAP.into()];
            state.capabilities.first_enhancement_choice =
                Some(crate::capability_v1::CAP_LOCAL_MAP.into());
            let build = state.progression_v6.clone();
            state.install_build(build).unwrap();
        }
        let acquired = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-earned",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(acquired.applied);
        let local_map = acquired
            .snapshot
            .capabilities
            .items
            .iter()
            .find(|item| item.capability_id == crate::capability_v1::CAP_LOCAL_MAP)
            .unwrap();
        assert!(local_map.granted && local_map.selected);
        assert!(acquired.snapshot.progression.worlds.iter().any(|world| {
            world.world_id == "grey_hive" && world.completed && world.first_completion
        }));
        let replay = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-earned",
            fresh.world_epoch,
        )
        .unwrap();
        assert!(!replay.applied);
        assert!(replay
            .error_code
            .as_deref()
            .unwrap()
            .contains("DuplicateRequest"));

        let next = runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = crate::world_v3::Vec3 {
                x_m: 20.0,
                y_m: 0.0,
                z_m: 8.0,
            };
        }
        let entered_gh = runtime
            .use_world_gate(
                "rs_world_gate_to_gh",
                "capability-enter-gh",
                next.world_epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        let other_world = crate::scene_route_commands::capability_terminal_status(
            &runtime,
            terminal_id,
            "capability-other-world",
            entered_gh.world_epoch,
        )
        .unwrap();
        assert!(!other_world.applied);
        assert_eq!(
            other_world.error_code.as_deref(),
            Some("E_CAPABILITY_TERMINAL_UNAVAILABLE")
        );
    }

    fn traversal_scene(from: [f32; 3], to: [f32; 3], required: &[&str]) -> String {
        serde_json::json!({
            "schemaVersion": 1,
            "worldId": "grey_hive",
            "sceneId": "gh_test",
            "boundsM": {"x": 0.0, "z": 0.0, "width": 10.0, "depth": 10.0},
            "collision": [],
            "logic": {"traversal": [{
                "id": "air_step_a", "from": from, "to": to,
                "rangeM": 1.2, "cooldownMs": 350,
                "requiredCapabilities": required,
            }]}
        })
        .to_string()
    }

    #[test]
    fn acquired_air_step_uses_authored_traversal_even_when_another_capability_is_selected() {
        let runtime = FormalRuntime::new().unwrap();
        runtime.pause().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = Vec3::new(1.0, 0.0, 1.0).unwrap();
            grant_test_capability(&mut state, CAP_AIR_STEP);
            grant_test_capability(&mut state, CAP_REGENERATION);
        }
        runtime
            .load_traversal_scene_definition(&traversal_scene(
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                &[CAP_AIR_STEP],
            ))
            .unwrap();
        runtime.resume().unwrap();
        let selected = runtime
            .apply_capability_command(CapabilityCommandRequest::Select {
                capability_ids: vec![CAP_REGENERATION.into()],
            })
            .unwrap();
        assert!(selected
            .capabilities
            .items
            .iter()
            .any(|item| item.capability_id == CAP_AIR_STEP && item.granted && !item.selected));
        assert!(runtime
            .state
            .lock()
            .unwrap()
            .effective_rules_v6
            .capability_permissions
            .contains(&CapabilityPermission::AuthoredAirStep));
        runtime
            .submit_action(ActionCommandRequest {
                protocol_version: 2,
                world_epoch: selected.world_epoch,
                request_id: 1,
                client_time_ms: 0,
                kind: ActionKind::ContextTraversal,
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let view = runtime.snapshot().unwrap();
            if view.player.transform.position_m == Vec3::new(2.0, 0.0, 1.0).unwrap() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "authored Air Step did not reach its destination"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        runtime.pause().unwrap();
        assert_eq!(
            runtime.snapshot().unwrap().player.transform.position_m,
            Vec3::new(2.0, 0.0, 1.0).unwrap()
        );
    }

    #[test]
    fn owner_advances_without_browser_input_and_pause_holds_time() {
        let runtime = FormalRuntime::new().unwrap();
        let before = runtime.snapshot().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        let running = runtime.snapshot().unwrap();
        assert!(running.server_tick >= before.server_tick + 3);
        assert_eq!(running.ack_seq, 0);
        runtime.pause().unwrap();
        let paused = runtime.snapshot().unwrap();
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(runtime.snapshot().unwrap().server_tick, paused.server_tick);
        runtime.resume().unwrap();
        std::thread::sleep(Duration::from_millis(60));
        assert!(runtime.snapshot().unwrap().server_tick > paused.server_tick);
    }

    #[test]
    fn presentation_events_have_stable_ids_independent_of_snapshots() {
        let runtime = FormalRuntime::new().unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime
            .submit_input(
                InputSample::new(epoch, 1, 1, 0.0, 0.0).unwrap(),
                vec![CombatIntentRequest::Attack { request_id: 1 }],
            )
            .unwrap();
        let events = runtime.presentation_events_since(epoch, 0).unwrap();
        assert!(events.iter().any(|event| event.kind == "AttackStarted"));
        let last_id = events.last().unwrap().event_id;
        assert!(runtime
            .presentation_events_since(epoch, last_id)
            .unwrap()
            .is_empty());
        assert_eq!(runtime.presentation_events_since(epoch, 0).unwrap(), events);
    }

    #[test]
    fn separate_action_mailbox_rejects_duplicates_and_applies_pulse() {
        let runtime = FormalRuntime::new().unwrap();
        let view = runtime
            .submit_action(ActionCommandRequest {
                protocol_version: 2,
                world_epoch: 1,
                request_id: 42,
                client_time_ms: 0,
                kind: ActionKind::PrimaryAttack,
            })
            .unwrap();
        assert!(view.server_tick > 0);
        assert_eq!(
            runtime
                .submit_action(ActionCommandRequest {
                    protocol_version: 2,
                    world_epoch: 1,
                    request_id: 42,
                    client_time_ms: 0,
                    kind: ActionKind::PrimaryAttack
                })
                .unwrap_err(),
            "E_ACTION_DUPLICATE"
        );
        // Let the first action finish its 330 ms windup/active/recovery timeline before
        // submitting another action; the action mailbox correctly rejects overlap.
        for sequence in 1..=22 {
            runtime
                .submit_input(
                    InputSample::new(1, sequence, sequence * 17, 0.0, 0.0).unwrap(),
                    vec![],
                )
                .unwrap();
        }
        let pulse = runtime
            .submit_action(ActionCommandRequest {
                protocol_version: 2,
                world_epoch: 1,
                request_id: 43,
                client_time_ms: 374,
                kind: ActionKind::Pulse,
            })
            .unwrap();
        assert_eq!(pulse.player.current_energy, 88);
    }

    #[test]
    fn v2_action_epoch_version_and_time_are_validated_before_enqueue() {
        let runtime = FormalRuntime::new().unwrap();
        let mut stale_epoch = ActionCommandRequest {
            protocol_version: 2,
            world_epoch: 2,
            request_id: 1,
            client_time_ms: 10,
            kind: ActionKind::Dash,
        };
        assert_eq!(
            runtime.submit_action(stale_epoch.clone()).unwrap_err(),
            "E_ACTION_STALE_EPOCH"
        );
        stale_epoch.world_epoch = 1;
        stale_epoch.protocol_version = 1;
        assert_eq!(
            runtime.submit_action(stale_epoch.clone()).unwrap_err(),
            "E_ACTION_PROTOCOL"
        );
        stale_epoch.protocol_version = 2;
        runtime.submit_action(stale_epoch).unwrap();
        let regressed = ActionCommandRequest {
            protocol_version: 2,
            world_epoch: 1,
            request_id: 2,
            client_time_ms: 9,
            kind: ActionKind::PrimaryAttack,
        };
        assert_eq!(
            runtime.submit_action(regressed).unwrap_err(),
            "E_ACTION_TIME_REGRESSION"
        );
    }

    #[test]
    fn v2_input_rejects_nan_range_and_stale_epoch_at_the_mailbox() {
        let runtime = FormalRuntime::new().unwrap();
        let mut not_finite = InputSample::new(1, 1, 1, 0.0, 0.0).unwrap();
        not_finite.move_x = f32::NAN;
        assert!(runtime
            .submit_input(not_finite, vec![])
            .unwrap_err()
            .starts_with("E_INPUT_INVALID"));
        let out_of_range = InputSample::new(1, 1, 1, 0.0, 0.0)
            .unwrap()
            .with_aim(1.01, 0.0)
            .unwrap_err();
        assert_eq!(format!("{out_of_range:?}"), "AxisOutOfRange");
        let stale_epoch = InputSample::new(2, 1, 1, 0.0, 0.0).unwrap();
        assert_eq!(
            runtime.submit_input(stale_epoch, vec![]).unwrap_err(),
            "E_INPUT_STALE_EPOCH"
        );
    }

    #[test]
    fn owner_projects_snapshot_advances_fixed_tick_and_composes_route() {
        let legacy_projection: crate::world_v3::WorldProgressProjection =
            serde_json::from_value(serde_json::json!({
                "worldId": "grey_hive", "completed": false, "firstCompletion": false,
                "visitId": 1, "cycleId": 1, "revisitCount": 0
            }))
            .unwrap();
        assert!(legacy_projection.completed_events.is_empty());
        let runtime = FormalRuntime::new().unwrap();
        let initial = runtime.snapshot().unwrap();
        assert_eq!(initial.world_id, "grey_hive");
        assert_eq!(initial.schema_version, "freeze-v02-interfaces/1.2");
        assert_eq!(initial.progression.current_world_id, "grey_hive");
        assert_eq!(initial.progression.worlds[0].visit_id, 1);
        assert!(initial.progression.worlds[0].completed_events.is_empty());
        assert!(initial.capabilities.items.iter().all(|item| !item.granted));

        let view = runtime
            .submit_input(InputSample::new(1, 1, 1, 0.0, 1.0).unwrap(), Vec::new())
            .unwrap();
        assert_eq!(view.server_tick, 1);
        assert_eq!(view.ack_seq, 1);
        assert!(view.player.transform.position_m.z_m > 0.0);

        let response = runtime
            .apply_route_command(RouteCommandRequest::Progress {
                event_id: "hive_power".into(),
                request_id: "integration-hive-power-1".into(),
            })
            .unwrap();
        assert!(response.applied);
        assert_eq!(response.view.progression.event_seq, 2);
        assert!(response.view.progression.worlds[0]
            .completed_events
            .contains(&"hive_power".to_string()));
        assert_eq!(response.view.server_tick, 1);

        let stale = runtime.submit_input(InputSample::new(1, 1, 1, 1.0, 0.0).unwrap(), Vec::new());
        assert_eq!(stale.unwrap_err(), "E_INPUT_STALE_SEQUENCE");
    }

    #[test]
    fn route_cannot_switch_to_a_world_without_loaded_geometry() {
        let runtime = FormalRuntime::new().unwrap();
        let error = runtime
            .apply_route_command(RouteCommandRequest::Enter {
                world_id: "mist_harbor".into(),
                request_id: "integration-premature-world-switch".into(),
            })
            .unwrap_err();
        assert_eq!(error, "E_WORLD_NOT_LOADED");
        assert_eq!(runtime.snapshot().unwrap().world_id, "grey_hive");
    }

    #[test]
    fn capability_first_choice_is_rejected_before_grey_hive_first_clear() {
        let runtime = FormalRuntime::new().unwrap();
        let before = runtime.snapshot().unwrap();
        assert_eq!(
            runtime.choose_first_enhancement(CAP_REAR_VIEW).unwrap_err(),
            "E_CAPABILITY_REQUIRES_GH_FIRST_CLEAR"
        );
        let after = runtime.snapshot().unwrap();
        assert_eq!(after.capabilities.items, before.capabilities.items);
        assert_eq!(after.capabilities.rear_view, before.capabilities.rear_view);
        assert_eq!(after.progression, before.progression);
    }

    #[test]
    fn action_v2_context_traversal_uses_loaded_marker_and_emits_events() {
        let runtime = FormalRuntime::new().unwrap();
        runtime
            .load_traversal_scene_definition(&traversal_scene(
                [0.5, 0.0, 0.5],
                [0.5, 0.0, 1.5],
                &[],
            ))
            .unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime
            .submit_action(ActionCommandRequest {
                protocol_version: 2,
                world_epoch: epoch,
                request_id: 50,
                client_time_ms: 0,
                kind: ActionKind::ContextTraversal,
            })
            .unwrap();
        assert_eq!(
            runtime
                .submit_action(ActionCommandRequest {
                    protocol_version: 2,
                    world_epoch: epoch,
                    request_id: 51,
                    client_time_ms: 0,
                    kind: ActionKind::ContextTraversal,
                })
                .unwrap_err(),
            "E_ACTION_BUSY"
        );
        assert_eq!(
            runtime
                .submit_action(ActionCommandRequest {
                    protocol_version: 2,
                    world_epoch: epoch,
                    request_id: 50,
                    client_time_ms: 0,
                    kind: ActionKind::ContextTraversal,
                })
                .unwrap_err(),
            "E_ACTION_DUPLICATE"
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut completed = false;
        while Instant::now() < deadline {
            let events = runtime.presentation_events_since(epoch, 0).unwrap();
            if events
                .iter()
                .any(|event| event.kind == "ContextTraversalCompleted")
            {
                completed = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            completed,
            "traversal should complete after its 350 ms movement"
        );
        let view = runtime.snapshot().unwrap();
        assert!((view.player.transform.position_m.x_m - 0.5).abs() < 0.001);
        assert!((view.player.transform.position_m.z_m - 1.5).abs() < 0.001);
        let events = runtime.presentation_events_since(epoch, 0).unwrap();
        assert!(events
            .iter()
            .any(|event| event.kind == "ContextTraversalStarted"));
        assert!(events
            .iter()
            .any(|event| event.kind == "ContextTraversalCompleted"));
    }

    #[test]
    fn context_traversal_fails_closed_without_marker_in_range_or_capability() {
        let runtime = FormalRuntime::new().unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let request = |request_id, client_time_ms| ActionCommandRequest {
            protocol_version: 2,
            world_epoch: epoch,
            request_id,
            client_time_ms,
            kind: ActionKind::ContextTraversal,
        };
        assert_eq!(
            runtime.submit_action(request(1, 0)).unwrap_err(),
            "E_TRAVERSAL_NO_MARKER"
        );
        assert!(runtime
            .snapshot()
            .unwrap()
            .capabilities
            .items
            .iter()
            .any(|item| { item.capability_id == CAP_AIR_STEP && !item.granted && !item.selected }));
        assert_eq!(
            runtime
                .apply_capability_command(CapabilityCommandRequest::Select {
                    capability_ids: vec![CAP_AIR_STEP.into()],
                })
                .unwrap_err(),
            "E_CAPABILITY_REJECTED: NotGranted"
        );

        runtime
            .load_traversal_scene_definition(&traversal_scene(
                [5.0, 0.0, 5.0],
                [5.5, 0.0, 5.0],
                &[],
            ))
            .unwrap();
        assert_eq!(
            runtime.submit_action(request(2, 0)).unwrap_err(),
            "E_TRAVERSAL_OUT_OF_RANGE"
        );

        runtime
            .load_traversal_scene_definition(&traversal_scene(
                [0.5, 0.0, 0.5],
                [0.5, 0.0, 1.5],
                &[CAP_AIR_STEP],
            ))
            .unwrap();
        assert_eq!(
            runtime.submit_action(request(3, 0)).unwrap_err(),
            "E_TRAVERSAL_CAPABILITY"
        );
        let mut state = runtime.state.lock().unwrap();
        let revision = state.world.revision;
        let mut capabilities = state.capabilities.clone();
        apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Grant {
                capability_id: CAP_AIR_STEP.into(),
            },
            revision,
        )
        .unwrap();
        apply_command_at_revision(
            &mut capabilities,
            CapabilityCommand::Select {
                capability_ids: vec![CAP_AIR_STEP.into()],
            },
            revision,
        )
        .unwrap();
        let world = state.world.clone();
        state.install_capability_state(world, capabilities).unwrap();
        drop(state);
        assert!(runtime
            .snapshot()
            .unwrap()
            .capabilities
            .items
            .iter()
            .any(|item| { item.capability_id == CAP_AIR_STEP && item.granted && item.selected }));
        assert!(runtime.submit_action(request(3, 0)).is_ok());
    }

    #[test]
    fn traversal_scene_loader_rejects_unknown_duplicate_and_unsafe_markers() {
        let runtime = FormalRuntime::new().unwrap();
        let unknown = traversal_scene([0.5, 0.0, 0.5], [0.5, 0.0, 1.5], &["mobility.unknown"]);
        assert_eq!(
            runtime
                .load_traversal_scene_definition(&unknown)
                .unwrap_err(),
            "E_TRAVERSAL_CAPABILITY_UNKNOWN"
        );
        let mut duplicate: serde_json::Value =
            serde_json::from_str(&traversal_scene([0.5, 0.0, 0.5], [0.5, 0.0, 1.5], &[])).unwrap();
        let duplicate_marker = duplicate["logic"]["traversal"][0].clone();
        duplicate["logic"]["traversal"]
            .as_array_mut()
            .unwrap()
            .push(duplicate_marker);
        assert_eq!(
            runtime
                .load_traversal_scene_definition(&duplicate.to_string())
                .unwrap_err(),
            "E_TRAVERSAL_MARKER_ID"
        );
        let blocked = serde_json::json!({
            "schemaVersion": 1, "worldId": "grey_hive", "sceneId": "gh_test",
            "boundsM": {"x": 0.0, "z": 0.0, "width": 10.0, "depth": 10.0},
            "collision": [{"id": "wall", "polygon": [[5.0, 5.0], [6.0, 5.0], [6.0, 6.0], [5.0, 6.0]]}],
            "logic": {"traversal": [{
                "id": "air_step_blocked", "from": [1.0, 0.0, 5.5], "to": [5.5, 0.0, 5.5],
                "rangeM": 1.2, "cooldownMs": 350, "requiredCapabilities": []
            }]}
        });
        assert_eq!(
            runtime
                .load_traversal_scene_definition(&blocked.to_string())
                .unwrap_err(),
            "E_TRAVERSAL_DESTINATION_BLOCKED"
        );
    }

    fn entry_readiness_runtime(label: &str) -> FormalRuntime {
        let root = std::env::temp_dir().join(format!("entry-readiness-{label}-{}-{}",
            std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
        runtime.pause().unwrap();
        runtime.stop_owner.store(true, Ordering::SeqCst);
        runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
        runtime
    }

    #[test]
    fn entry_readiness_new_is_frozen_until_exact_token_and_rejects_generic_resume() {
        let runtime = entry_readiness_runtime("new");
        let prepared = runtime.reset_new().unwrap();
        let token = prepared.entry_token.clone().expect("New prepares an entry token");
        assert_eq!(runtime.resume().unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
        assert_eq!(runtime.submit_input(InputSample::new(prepared.world_epoch, 1, 1, 1.0, 0.0).unwrap(), vec![]).unwrap_err(), "E_RUNTIME_PAUSED");
        let before = runtime.snapshot().unwrap();
        let mut wrong = token.clone();
        wrong.scene_id.push_str("_wrong");
        assert_eq!(runtime.scene_ready(&wrong, false).unwrap_err(), "E_SCENE_ENTRY_STALE_TOKEN");
        assert_eq!(runtime.snapshot().unwrap(), before);
        let ready = runtime.scene_ready(&token, false).unwrap();
        assert!(ready.entry_token.is_none());
        assert!(!runtime.state.lock().unwrap().paused);
        assert_eq!(runtime.scene_ready(&token, false).unwrap_err(), "E_SCENE_ENTRY_STALE_TOKEN");
    }

    #[test]
    fn entry_readiness_continue_rebases_epoch_and_generation() {
        let runtime = entry_readiness_runtime("continue");
        runtime.save_slot("living", "Living", true).unwrap();
        let first = runtime.continue_slot("living").unwrap();
        let first_token = first.entry_token.unwrap();
        let second = runtime.continue_slot("living").unwrap();
        let second_token = second.entry_token.unwrap();
        assert!(second_token.world_epoch > first_token.world_epoch);
        assert!(second_token.generation > first_token.generation);
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.scene_ready(&first_token, false).unwrap_err(), "E_SCENE_ENTRY_STALE_TOKEN");
        assert_eq!(runtime.snapshot().unwrap(), before);
        let ready = runtime.scene_ready(&second_token, true).unwrap();
        assert!(ready.entry_token.is_none());
        assert!(runtime.state.lock().unwrap().paused);
        runtime.resume().unwrap();
        assert!(!runtime.state.lock().unwrap().paused);
        assert_eq!(runtime.submit_input(InputSample::new(first_token.world_epoch, 900, 900, 1.0, 0.0).unwrap(), vec![]).unwrap_err(), "E_INPUT_STALE_EPOCH");
        assert_eq!(runtime.submit_action(ActionCommandRequest { protocol_version: 2, world_epoch: first_token.world_epoch,
            request_id: 900, client_time_ms: 900, kind: ActionKind::GuardEnd }).unwrap_err(), "E_ACTION_STALE_EPOCH");
        assert_eq!(runtime.use_world_gate("rs_world_gate_to_gh", "stale-scene", first_token.world_epoch).unwrap_err(), "E_WORLD_GATE_STALE_EPOCH");
        let live = runtime.snapshot().unwrap();
        runtime.state.lock().unwrap().world.revision.world_epoch = build_ui::MAX_SAFE_REVISION;
        let exhausted = runtime.snapshot().unwrap();
        assert_eq!(runtime.continue_slot("living").unwrap_err(), "E_WORLD_EPOCH_EXHAUSTED");
        assert_eq!(runtime.snapshot().unwrap(), exhausted);
        assert!(live.world_epoch < exhausted.world_epoch);
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }

    #[test]
    fn entry_readiness_owner_does_not_advance_during_loading() {
        let runtime = FormalRuntime::new().unwrap();
        let prepared = runtime.reset_new().unwrap();
        std::thread::sleep(Duration::from_millis(70));
        assert_eq!(runtime.snapshot().unwrap(), prepared);
        runtime.scene_ready(prepared.entry_token.as_ref().unwrap(), false).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        assert!(runtime.snapshot().unwrap().server_tick > prepared.server_tick);
    }

    #[test]
    fn entry_readiness_new_prepares_production_scene_atomically() {
        let runtime = entry_readiness_runtime("production-new");
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        let prepared = runtime.reset_new().unwrap();
        assert_eq!(prepared.world_id, "return_station");
        assert_eq!(prepared.scene_id, "rs_core_room");
        assert_eq!(prepared.entry_token.as_ref().unwrap().scene_id, "rs_core_room");
        runtime.state.lock().unwrap().entry_generation = build_ui::MAX_SAFE_REVISION;
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.reset_new().unwrap_err(), "E_SCENE_ENTRY_GENERATION_EXHAUSTED");
        assert_eq!(runtime.snapshot().unwrap(), before);
    }

    #[test]
    fn entry_readiness_transition_failure_preserves_both_live_owners() {
        let runtime = entry_readiness_runtime("transition");
        runtime.load_scene_registry([ACTION_TEST_RS, ACTION_TEST_GH, ACTION_TEST_MH, ACTION_TEST_CW],
            "gh_test_power", &BTreeSet::new(), &BTreeSet::new()).unwrap();
        runtime.resume().unwrap();
        runtime.state.lock().unwrap().entry_generation = build_ui::MAX_SAFE_REVISION;
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.transition_scene("gh_to_hub", "entry-transition", before.world_epoch).unwrap_err(),
            "E_SCENE_ENTRY_GENERATION_EXHAUSTED");
        assert_eq!(runtime.snapshot().unwrap(), before);
        runtime.state.lock().unwrap().entry_generation = 0;
        let entered = runtime.transition_scene("gh_to_hub", "entry-transition", before.world_epoch).unwrap();
        assert_eq!(entered.scene_id, "rs_test_hub");
        assert!(runtime.state.lock().unwrap().paused);
        runtime.scene_ready(entered.entry_token.as_ref().unwrap(), false).unwrap();
        assert!(!runtime.state.lock().unwrap().paused);
    }

    #[test]
    fn entry_readiness_world_gate_failure_preserves_request_and_route() {
        let runtime = entry_readiness_runtime("world-gate");
        crate::production_scene_bootstrap::install(&runtime).unwrap();
        runtime.resume().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = Vec3::new(20.0, 0.0, 8.0).unwrap();
            state.entry_generation = build_ui::MAX_SAFE_REVISION;
        }
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.use_world_gate("rs_world_gate_to_gh", "entry-gate", before.world_epoch).unwrap_err(),
            "E_SCENE_ENTRY_GENERATION_EXHAUSTED");
        assert_eq!(runtime.snapshot().unwrap(), before);
        runtime.state.lock().unwrap().entry_generation = 0;
        let entered = runtime.use_world_gate("rs_world_gate_to_gh", "entry-gate", before.world_epoch).unwrap();
        assert_eq!(entered.scene_id, "gh_entry_maintenance");
        assert!(runtime.state.lock().unwrap().paused);
        runtime.scene_ready(entered.entry_token.as_ref().unwrap(), false).unwrap();
        assert!(!runtime.state.lock().unwrap().paused);
    }

    #[test]
    fn entry_readiness_return_invalidates_pending_token_without_persisting_it() {
        let runtime = entry_readiness_runtime("return");
        runtime.save().unwrap();
        let original = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        let prepared = runtime.reset_new().unwrap();
        let token = prepared.entry_token.unwrap();
        let returned = runtime.return_to_hub().unwrap();
        assert!(returned.entry_token.is_none());
        assert!(runtime.state.lock().unwrap().paused);
        assert_eq!(runtime.scene_ready(&token, false).unwrap_err(), "E_SCENE_ENTRY_STALE_TOKEN");
        let saved = std::fs::read_to_string(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        assert_eq!(saved.as_bytes(), original.as_slice());
        assert!(!saved.contains("entryToken"));
        assert!(!saved.contains("entryGeneration"));
        assert!(!saved.contains("pendingEntry"));
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }

    #[test]
    fn entry_readiness_rejects_valid_build_enhancement_and_save_until_ready() {
        let runtime = entry_readiness_runtime("mutations");
        runtime.grant_trusted_build_item("rear_view_lens").unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.route.progress.iter_mut().find(|p| p.world_id == "grey_hive").unwrap().completed = true;
            let generation = state.entry_generation;
            FormalRuntime::prepare_entry(&mut state, generation).unwrap();
        }
        let before = runtime.snapshot().unwrap();
        let request = build_ui::BuildCommandRequest {
            request_id: "ready-equip".into(), world_epoch: before.world_epoch,
            expected_build_revision: before.build.as_ref().unwrap().revision,
            action: build_ui::BuildAction::Equip { item_id: "rear_view_lens".into(), expected_item_id: None },
        };
        let rejected = runtime.apply_build_command(request.clone()).unwrap();
        assert!(!rejected.applied && !rejected.already_applied);
        assert_eq!(rejected.error_code.as_deref(), Some("E_SCENE_ENTRY_NOT_READY"));
        assert_eq!(runtime.choose_first_enhancement(CAP_LOCAL_MAP).unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
        assert_eq!(runtime.save().unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
        assert_eq!(runtime.save_slot("blocked", "Blocked", true).unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert!(!runtime.save_root.exists());
        runtime.scene_ready(before.entry_token.as_ref().unwrap(), true).unwrap();
        assert!(runtime.apply_build_command(request).unwrap().applied);
        runtime.choose_first_enhancement(CAP_LOCAL_MAP).unwrap();
        assert!(runtime.snapshot().unwrap().entry_token.is_none());
    }

    #[test]
    fn death_safety_fatal_input_is_acknowledged_once_then_owner_and_mutations_freeze() {
        let root = std::env::temp_dir().join(format!("death-owner-{}-{}", std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        runtime.pause().unwrap();
        runtime.save().unwrap();
        runtime.save_slot("living", "Living", true).unwrap();
        let file = crate::save_v6::save_path(&root);
        let slot = root.join("slots/living/slot-v6.json");
        let saved = std::fs::read(&file).unwrap();
        let slot_saved = std::fs::read(&slot).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player_hp = 1;
            let mut sentinel = crate::sentinel_ai::Sentinel::new("fatal-test", state.world.player.position_m, 100);
            sentinel.state = crate::sentinel_ai::SentinelState::Attack;
            state.world.sentinels = vec![sentinel];
        }
        runtime.resume().unwrap();
        let fatal = runtime.submit_input(InputSample::new(1, 1, 1, 0.0, 0.0).unwrap(), vec![]).unwrap();
        assert_eq!(fatal.ack_seq, 1);
        assert_eq!(fatal.player.current_hp, 0);
        assert_eq!(fatal.player.action_state, "death");
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(runtime.snapshot().unwrap(), fatal);
        assert_eq!(runtime.presentation_events_since(1, 0).unwrap().iter().filter(|event| event.kind == "PlayerDeath").count(), 1);
        assert_eq!(runtime.submit_input(InputSample::new(1, 2, 2, 1.0, 0.0).unwrap(), vec![]).unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.submit_action(ActionCommandRequest { protocol_version: 2, world_epoch: 1,
            request_id: 22, client_time_ms: 2, kind: ActionKind::GuardEnd }).unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.resume().unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.save().unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.save_slot("living", "Living", false).unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.choose_first_enhancement(CAP_LOCAL_MAP).unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.grant_trusted_build_item("rear_view_lens").unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.return_to_hub_slot("living", "Living", false).unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.return_to_hub().unwrap(), fatal);
        assert_eq!(runtime.pause().unwrap(), fatal);
        assert_eq!(runtime.snapshot().unwrap(), fatal);
        assert_eq!(std::fs::read(file).unwrap(), saved);
        assert_eq!(std::fs::read(slot).unwrap(), slot_saved);
        let restored = runtime.continue_slot("living").unwrap();
        assert!(restored.player.current_hp > 0 && restored.entry_token.is_some());
        runtime.scene_ready(restored.entry_token.as_ref().unwrap(), true).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn death_safety_dead_v6_slot_is_truthfully_unavailable_and_continue_is_nonmutating() {
        let runtime = entry_readiness_runtime("dead-slot");
        runtime.save_slot("dead", "Old death", true).unwrap();
        let path = runtime.save_root.join("slots/dead/slot-v6.json");
        let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["save"]["save"]["player"]["currentHp"] = serde_json::json!(0);
        let dead = serde_json::to_vec_pretty(&value).unwrap();
        std::fs::write(&path, &dead).unwrap();
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.continue_slot("dead").unwrap_err(), "E_SAVE_PLAYER_DEAD");
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(std::fs::read(&path).unwrap(), dead);
        let slots = runtime.list_save_slots();
        assert_eq!(slots.len(), 1);
        assert!(!slots[0].valid);
        assert_eq!(slots[0].error_code.as_deref(), Some("E_SAVE_PLAYER_DEAD"));
        assert!(!runtime.has_save());
        assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }

    #[test]
    fn death_safety_old_dead_v5_save_is_rejected_before_any_migration_or_backup() {
        let runtime = entry_readiness_runtime("dead-legacy");
        runtime.save().unwrap();
        let current_path = crate::save_v6::save_path(&runtime.save_root);
        let current: serde_json::Value = serde_json::from_slice(&std::fs::read(&current_path).unwrap()).unwrap();
        let mut legacy = current["save"].clone();
        legacy["player"]["currentHp"] = serde_json::json!(0);
        let path = crate::save_v5::save_path(&runtime.save_root);
        let dead = serde_json::to_vec_pretty(&legacy).unwrap();
        std::fs::write(&path, &dead).unwrap();
        std::fs::remove_file(current_path).unwrap();
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.continue_saved().unwrap_err(), "E_SAVE_PLAYER_DEAD");
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(std::fs::read(&path).unwrap(), dead);
        assert_eq!(std::fs::read_dir(&runtime.save_root).unwrap().count(), 1);
        assert!(!runtime.has_save());
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }

    #[test]
    fn death_safety_dead_build_rejection_does_not_change_receipts_or_rules() {
        let runtime = entry_readiness_runtime("dead-build");
        runtime.grant_trusted_build_item("rear_view_lens").unwrap();
        runtime.state.lock().unwrap().world.player_hp = 0;
        let before = runtime.snapshot().unwrap();
        let result = runtime.apply_build_command(build_ui::BuildCommandRequest {
            request_id: "dead-equip".into(), world_epoch: before.world_epoch,
            expected_build_revision: before.build.as_ref().unwrap().revision,
            action: build_ui::BuildAction::Equip { item_id: "rear_view_lens".into(), expected_item_id: None },
        }).unwrap();
        assert!(!result.applied && !result.already_applied);
        assert_eq!(result.error_code.as_deref(), Some("E_RUNTIME_DEAD"));
        assert_eq!(runtime.snapshot().unwrap(), before);
    }

    fn lifecycle_context(view: &WorldView) -> SessionContext {
        SessionContext { world_id: view.world_id.clone(), scene_id: view.scene_id.clone(), world_epoch: view.world_epoch }
    }

    #[test]
    fn lifecycle_safety_pause_atomically_releases_movement_guard_and_pending_combat() {
        let runtime = entry_readiness_runtime("lifecycle-neutral");
        runtime.resume().unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            state.latest_sample = InputSample::new(state.world.revision.world_epoch, 8, 100, 1.0, -1.0).unwrap();
            state.world.player.velocity_mps = Vec3::new(3.0, 0.0, 2.0).unwrap();
            state.pending_combat.push(CombatIntent::ActionAttack { request_id: 9 });
            state.world.combat_state.active_action = Some(crate::continuous_combat::ActiveCombatAction {
                kind: crate::continuous_combat::CombatActionKind::Guard, request_id: 7,
                elapsed_ms: 120, impact_resolved: false, end_requested: false,
            });
            state.world.combat_state.action_locks_facing = true;
        }
        let before = runtime.snapshot().unwrap();
        let context = lifecycle_context(&before);
        let paused = runtime.pause_context(&context).unwrap();
        assert_eq!(paused.server_tick, before.server_tick);
        assert_eq!(paused.player.velocity_mps, Vec3::zero());
        assert_ne!(paused.player.action_state, "guard");
        assert_eq!(runtime.pause_context(&context).unwrap(), paused, "repeat pause is idempotent");
        runtime.resume_context(&context).unwrap();
        {
            let mut state = runtime.state.lock().unwrap();
            assert!(state.pending_combat.is_empty());
            assert!(state.world.combat_state.active_action.is_none());
            assert_eq!((state.latest_sample.move_x, state.latest_sample.move_z), (0.0, 0.0));
            let position = state.world.player.position_m;
            advance_owner_step(&mut state).unwrap();
            assert_eq!(state.world.player.position_m, position, "resume cannot replay held movement");
            assert!(state.world.combat_state.active_action.is_none(), "resume cannot replay queued attack");
        }
    }

    #[test]
    fn lifecycle_safety_stale_pause_and_resume_cannot_touch_new_journey() {
        let runtime = entry_readiness_runtime("lifecycle-stale");
        let old = lifecycle_context(&runtime.snapshot().unwrap());
        let prepared = runtime.reset_new().unwrap();
        runtime.scene_ready(prepared.entry_token.as_ref().unwrap(), false).unwrap();
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.pause_context(&old).unwrap_err(), "E_LIFECYCLE_STALE_CONTEXT");
        assert_eq!(runtime.resume_context(&old).unwrap_err(), "E_LIFECYCLE_STALE_CONTEXT");
        assert_eq!(runtime.snapshot().unwrap(), before);
        for context in [SessionContext { scene_id: "wrong".into(), ..lifecycle_context(&before) },
            SessionContext { world_id: "wrong".into(), ..lifecycle_context(&before) }] {
            assert_eq!(runtime.pause_context(&context).unwrap_err(), "E_LIFECYCLE_STALE_CONTEXT");
            assert_eq!(runtime.snapshot().unwrap(), before);
        }
        assert!(!runtime.state.lock().unwrap().paused);
    }

    #[test]
    fn lifecycle_safety_pause_keeps_entry_barrier_and_dead_state_immutable() {
        let runtime = entry_readiness_runtime("lifecycle-entry");
        let prepared = runtime.reset_new().unwrap();
        let context = lifecycle_context(&prepared);
        assert_eq!(runtime.pause_context(&context).unwrap(), prepared);
        assert_eq!(runtime.resume_context(&context).unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
        runtime.scene_ready(prepared.entry_token.as_ref().unwrap(), true).unwrap();
        runtime.state.lock().unwrap().world.player_hp = 0;
        let dead = runtime.snapshot().unwrap();
        assert_eq!(runtime.pause_context(&context).unwrap(), dead);
        assert_eq!(runtime.resume_context(&context).unwrap_err(), "E_RUNTIME_DEAD");
        assert_eq!(runtime.snapshot().unwrap(), dead);
    }

    #[test]
    fn lifecycle_safety_failed_pause_revision_keeps_retained_input_unchanged() {
        let runtime = entry_readiness_runtime("lifecycle-overflow");
        {
            let mut state = runtime.state.lock().unwrap();
            state.latest_sample.move_x = 1.0;
            state.world.revision.authority_revision = u64::MAX;
        }
        let before = runtime.snapshot().unwrap();
        assert!(runtime.pause_context(&lifecycle_context(&before)).unwrap_err().starts_with("E_WORLD_REVISION"));
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(runtime.state.lock().unwrap().latest_sample.move_x, 1.0);
    }

    #[test]
    fn lifecycle_safety_ordering_rejects_late_resume_after_newer_pause() {
        let runtime = entry_readiness_runtime("lifecycle-order");
        let context = lifecycle_context(&runtime.snapshot().unwrap());
        runtime.pause_context_ordered(&context, 2).unwrap();
        let held = runtime.snapshot().unwrap();
        assert_eq!(runtime.resume_context_ordered(&context, 1).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert_eq!(runtime.resume_context_ordered(&context, 2).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert_eq!(runtime.snapshot().unwrap(), held);
        assert!(runtime.state.lock().unwrap().paused);
        runtime.resume_context_ordered(&context, 3).unwrap();
        assert!(!runtime.state.lock().unwrap().paused);
        assert_eq!(runtime.pause_context_ordered(&context, 2).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert!(!runtime.state.lock().unwrap().paused);
    }

    #[test]
    fn lifecycle_safety_pending_entry_stop_hold_wins_over_delayed_ready_false() {
        let runtime = entry_readiness_runtime("lifecycle-ready-hold");
        let prepared = runtime.reset_new().unwrap();
        let context = lifecycle_context(&prepared);
        runtime.pause_context_ordered(&context, 2).unwrap();
        let released = runtime.scene_ready(prepared.entry_token.as_ref().unwrap(), false).unwrap();
        assert!(released.entry_token.is_none());
        assert!(runtime.state.lock().unwrap().paused, "an older ready(false) may not resume a stopped owner");
        assert_eq!(runtime.resume_context_ordered(&context, 1).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert!(runtime.state.lock().unwrap().paused);
        runtime.resume_context_ordered(&context, 3).unwrap();
        assert!(!runtime.state.lock().unwrap().paused);
    }

}

#[cfg(test)]
mod vertical_actor_capture_tests {
    use super::*;
    #[test]
    fn capture_rejects_off_deck_or_cross_plane_actor_before_serialization() {
        let mut state=FormalRuntime::initial_state(9).unwrap();
        state.world.player.position_m=Vec3{x_m:2.,y_m:0.,z_m:2.};state.world.sentinels.clear();
        state.kcc=crate::continuous_kcc::StaticKccWorld::new(crate::continuous_kcc::Aabb::new(0.,12.,0.,12.).unwrap(),vec![])
            .with_standing_decks(vec![crate::moving_support::StandingDeckDefinition{id:"capture_deck".into(),polygon:vec![[4.,4.],[6.,4.],[6.,6.],[4.,6.]],height_m:2.}]).unwrap();
        state.world.generic_actors=vec![crate::world_v3::ActorRuntime::spawn("capture_actor","grey_hive.infected_maintenance_worker",Vec3{x_m:5.,y_m:2.,z_m:5.}).unwrap()];
        crate::save_v5::SaveV5::capture(&state,None).unwrap();
        for position in [Vec3{x_m:8.,y_m:2.,z_m:5.},Vec3{x_m:5.,y_m:0.,z_m:5.},Vec3{x_m:5.,y_m:1.,z_m:5.}] {
            state.world.generic_actors[0].position_m=position;
            assert_eq!(crate::save_v5::SaveV5::capture(&state,None).unwrap_err(),"E_SAVE_ACTOR_SUPPORT_INVALID");
        }
    }
}

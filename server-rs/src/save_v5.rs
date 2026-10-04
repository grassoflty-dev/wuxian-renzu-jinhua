//! Strict multi-world Save v5 DTO and durable persistence.
use crate::{
    capability_v1::CapabilityState,
    continuous_combat::CombatState,
    continuous_input::InputSample,
    continuous_kcc::{KccBody, StaticKccWorld},
    fixed_step::{FixedStepClock, FixedStepConfig},
    formal_runtime::RuntimeState,
    save_v3::{MigrationProvenance, PlayerStateV3},
    sentinel_ai::Sentinel,
    world_progression::{RouteState, WORLD_CLOCKWORKS, WORLD_GREY_HIVE, WORLD_MIST_HARBOR},
    world_v3::{
        ActorRuntime, ExploredMap, RearViewAuthorization, Vec3, WorldRevision, WorldStateV3,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const SAVE_V5_SCHEMA_VERSION: u32 = 5;
pub const SAVE_V5_FILE_NAME: &str = "formal-save-v5.json";
pub const SAVE_V5_CONTENT_VERSION: &str = "campaign-content-v1";
pub const SAVE_V5_BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_SAVE_BYTES: u64 = 8 * 1024 * 1024;
const PROGRESSION_CATALOG: &str = include_str!("../data/world_progression_v1.json");
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryState {
    pub items: Vec<InventoryItemV5>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryItemV5 {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DialogFlags(pub BTreeSet<String>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorSaveV5 {
    pub actor_id: String,
    pub actor_type: String,
    pub position_m: Vec3,
    pub hp: u32,
    pub state: crate::sentinel_ai::SentinelState,
    pub state_remaining_ms: u64,
    pub attack_serial: u64,
    pub active: bool,
    #[serde(default, skip_serializing_if="Option::is_none", deserialize_with="deserialize_charge_controller")]
    pub charge_controller: Option<crate::sentinel_ai::ChargeController>,
}

fn deserialize_charge_controller<'de,D:serde::Deserializer<'de>>(d:D)->Result<Option<crate::sentinel_ai::ChargeController>,D::Error> {
    crate::sentinel_ai::ChargeController::deserialize(d).map(Some)
}
impl ActorSaveV5 {
    fn sentinel(&self)->Sentinel {Sentinel{entity_id:self.actor_id.clone(),position_m:self.position_m,hp:self.hp,state:self.state,
        state_remaining_ms:self.state_remaining_ms,attack_serial:self.attack_serial,active:self.active,charge_controller:self.charge_controller.clone()}}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneStateSave {
    pub scene_id: String,
    pub activated_ids: Vec<String>,
    pub emitted_event_ids: Vec<String>,
    pub checkpoint_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveV5 {
    pub schema_version: u32,
    pub build_version: String,
    pub content_version: String,
    pub world_id: String,
    pub scene_id: String,
    pub checkpoint_id: Option<String>,
    pub player: PlayerStateV3,
    pub actors: Vec<ActorSaveV5>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_actors: Vec<ActorRuntime>,
    pub combat_handled_request_ids: Vec<u64>,
    pub scene_states: Vec<SceneStateSave>,
    pub progression: RouteState,
    pub capabilities: CapabilityState,
    pub inventory: InventoryState,
    pub dialog_flags: DialogFlags,
    pub revision: WorldRevision,
    pub server_time_ms: u64,
    /// Optional authority projection for scenes with moving supports. It is
    /// checked against immutable scene geometry and the saved clock on Continue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_support_frame: Option<crate::moving_support::SupportFrame>,
    pub explored: ExploredMap,
    pub rear_view: RearViewAuthorization,
    pub last_input_seq: u64,
    pub last_client_time_ms: u64,
    pub last_damaged_at_ms: Option<u64>,
    pub last_combat_at_ms: Option<u64>,
    pub migration_provenance: Option<MigrationProvenance>,
}

impl SaveV5 {
    pub(crate) fn capture(
        state: &RuntimeState,
        scene: Option<&crate::scene_runtime::SceneRuntime>,
    ) -> Result<Self, String> {
        let mut player = PlayerStateV3::from(&state.world.player);
        player.current_hp = state.world.player_hp;
        player.max_hp = state.world.player_max_hp;
        player.current_energy = state.world.player_energy;
        player.max_energy = state.world.player_max_energy;
        let scene_id = scene
            .map(|value| value.scene_id.clone())
            .unwrap_or_else(|| state.world.scene_id.clone());
        let scene_states = scene
            .map(capture_scene_state)
            .transpose()?
            .into_iter()
            .collect();
        if !state.kcc.valid_saved_support_contact(&state.world.player, state.world.server_time_ms) {
            return Err("E_SAVE_SUPPORT_CONTACT_INVALID".into());
        }
        if !state
            .kcc
            .can_occupy(state.world.player.position_m, state.world.player.radius_m)
        {
            return Err("E_SAVE_PLAYER_POSITION_INVALID".into());
        }
        if !state.world.actors_have_stable_support(&state.kcc) {
            return Err("E_SAVE_ACTOR_SUPPORT_INVALID".into());
        }
        let save = Self {
            schema_version: SAVE_V5_SCHEMA_VERSION,
            build_version: SAVE_V5_BUILD_VERSION.into(),
            content_version: SAVE_V5_CONTENT_VERSION.into(),
            world_id: state.world.world_id.clone(),
            scene_id,
            checkpoint_id: scene
                .and_then(|value| value.checkpoint_id.clone())
                .or_else(|| state.world.checkpoint_id.clone()),
            player,
            actors: state
                .world
                .sentinels
                .iter()
                .map(|actor| ActorSaveV5 {
                    actor_id: actor.entity_id.clone(),
                    actor_type: "sentinel".into(),
                    position_m: actor.position_m,
                    hp: actor.hp,
                    state: actor.state,
                    state_remaining_ms: actor.state_remaining_ms,
                    attack_serial: actor.attack_serial,
                    active: actor.active,
                    charge_controller: actor.charge_controller.clone(),
                })
                .collect(),
            generic_actors: state.world.generic_actors.clone(),
            combat_handled_request_ids: state.world.combat_state.handled_request_ids(),
            scene_states,
            progression: state.route.clone(),
            capabilities: state.capabilities.clone(),
            inventory: InventoryState::default(),
            dialog_flags: DialogFlags::default(),
            revision: state.world.revision,
            server_time_ms: state.world.server_time_ms,
            moving_support_frame: if state.kcc.has_support_surfaces() {
                Some(state.kcc.support_frame(state.world.revision.world_epoch,state.world.server_time_ms)
                    .map_err(|_| "E_SAVE_SUPPORT_FRAME_INVALID")?)
            } else {None},
            explored: state.world.explored.clone(),
            rear_view: state.world.rear_view.clone(),
            last_input_seq: state.last_received_seq,
            last_client_time_ms: state.last_client_time_ms,
            last_damaged_at_ms: state.last_damaged_at_ms,
            last_combat_at_ms: state.last_combat_at_ms,
            migration_provenance: None,
        };
        save.validate()?;
        Ok(save)
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(frame) = &self.moving_support_frame {
            if frame.world_epoch != self.revision.world_epoch || frame.server_time_ms != self.server_time_ms
                || crate::moving_support::validate_saved_frame_shape(frame).is_err() {
                return Err("E_SAVE_SUPPORT_FRAME_INVALID".into());
            }
            if !(crate::moving_support::FLOOR_M..=crate::moving_support::CEILING_M)
                .contains(&self.player.position_m.y_m) {
                return Err("E_SAVE_SUPPORT_CONTACT_INVALID".into());
            }
        }
        if self.schema_version != SAVE_V5_SCHEMA_VERSION {
            return Err("E_SAVE_VERSION_UNSUPPORTED".into());
        }
        if self.build_version != SAVE_V5_BUILD_VERSION {
            return Err("E_SAVE_BUILD_UNSUPPORTED".into());
        }
        if self.content_version != SAVE_V5_CONTENT_VERSION {
            return Err("E_SAVE_CONTENT_UNSUPPORTED".into());
        }
        if !known_world(&self.world_id)
            || self.scene_id.trim().is_empty()
            || self.scene_id.len() > 128
            || !known_world(&self.progression.current_world_id)
            || self.world_id != "return_station"
                && self.progression.current_world_id != self.world_id
            || self.capabilities.world_id != self.world_id
            || self.explored.world_id != self.world_id
            || self.revision.world_epoch == 0
            || self.revision.authority_revision < self.revision.server_tick
            || self.player.max_hp == 0
            || self.player.current_hp > self.player.max_hp
            || self.player.max_energy == 0
            || self.player.current_energy > self.player.max_energy
            || self.capabilities.schema_version != crate::capability_v1::CAPABILITY_SCHEMA_VERSION
            || self.progression.schema_version != crate::world_progression::ROUTE_SCHEMA_VERSION
        {
            return Err("E_SAVE_STATE_INVALID".into());
        }
        let numeric = [
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
        ];
        if numeric.iter().any(|n| !n.is_finite())
            || self.player.radius_m <= 0.0
            || [
                self.player.move_speed_mps,
                self.player.jump_speed_mps,
                self.player.gravity_mps2,
                self.player.dash_speed_mps,
            ]
            .iter()
            .any(|n| *n < 0.0)
        {
            return Err("E_SAVE_NUMERIC_INVALID".into());
        }
        self.capabilities
            .regeneration
            .validate()
            .map_err(|_| "E_SAVE_CAPABILITY_INVALID".to_string())?;
        let mut world_ids = BTreeSet::new();
        if self.progression.progress.len() != 3
            || self.progression.progress.iter().any(|entry| {
                !known_campaign_world(&entry.world_id) || !world_ids.insert(entry.world_id.as_str())
            })
        {
            return Err("E_SAVE_ROUTE_INVALID".into());
        }
        if self.world_id != "return_station" && self.progression.current_world_id != self.world_id {
            return Err("E_SAVE_ROUTE_INVALID".into());
        }
        let catalog: serde_json::Value =
            serde_json::from_str(PROGRESSION_CATALOG).map_err(|_| "E_SAVE_ROUTE_INVALID")?;
        let definitions = catalog["worlds"].as_array().ok_or("E_SAVE_ROUTE_INVALID")?;
        for progress in &self.progression.progress {
            let definition = definitions
                .iter()
                .find(|entry| entry["worldId"] == progress.world_id)
                .ok_or("E_SAVE_ROUTE_INVALID")?;
            let required = definition["requiredEvents"]
                .as_array()
                .ok_or("E_SAVE_ROUTE_INVALID")?
                .iter()
                .map(|event| event.as_str().ok_or("E_SAVE_ROUTE_INVALID"))
                .collect::<Result<BTreeSet<_>, _>>()?;
            let completed = progress
                .completed_events
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            if completed.len() != progress.completed_events.len()
                || completed.iter().any(|event| !required.contains(event))
                || progress.completed && !required.is_subset(&completed)
                || progress.first_completion && !progress.completed
            {
                return Err("E_SAVE_ROUTE_INVALID".into());
            }
        }
        let mut reward_requests = BTreeSet::new();
        if self.progression.reward_ledger.iter().any(|entry| {
            entry.request_id.trim().is_empty() || !reward_requests.insert(entry.request_id.as_str())
        }) {
            return Err("E_SAVE_REWARD_LEDGER_INVALID".into());
        }
        let allowed_capabilities = [
            crate::capability_v1::CAP_LOCAL_MAP,
            crate::capability_v1::CAP_REAR_VIEW,
            crate::capability_v1::CAP_ENEMY_VITALS,
            crate::capability_v1::CAP_REGENERATION,
            crate::capability_v1::CAP_ACOUSTIC_MAPPING,
            crate::capability_v1::CAP_AIR_STEP,
        ];
        let mut capabilities = BTreeSet::new();
        if self.capabilities.grants.iter().any(|grant| {
            !allowed_capabilities.contains(&grant.capability_id.as_str())
                || !capabilities.insert(grant.capability_id.as_str())
        }) || self.capabilities.selected.iter().any(|id| {
            !self
                .capabilities
                .grants
                .iter()
                .any(|grant| &grant.capability_id == id)
        }) || self
            .capabilities
            .selected
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != self.capabilities.selected.len()
        {
            return Err("E_SAVE_CAPABILITY_INVALID".into());
        }
        if capabilities.contains(crate::capability_v1::CAP_ACOUSTIC_MAPPING)
            && !self.progression.progress.iter().any(|progress| {
                progress.world_id == "mist_harbor"
                    && progress
                        .completed_events
                        .iter()
                        .any(|event| event == "mist_signal")
            })
        {
            return Err("E_SAVE_CAPABILITY_ROUTE_INVALID".into());
        }
        if capabilities.contains(crate::capability_v1::CAP_AIR_STEP)
            && !self.progression.progress.iter().any(|progress| {
                progress.world_id == WORLD_CLOCKWORKS
                    && progress
                        .completed_events
                        .iter()
                        .any(|event| event == "clockworks_shutdown")
            })
        {
            return Err("E_SAVE_CAPABILITY_ROUTE_INVALID".into());
        }
        let mut actor_ids = BTreeSet::new();
        if self.actors.iter().any(|actor| {
            actor.actor_id.trim().is_empty()
                || actor.actor_type != "sentinel"
                || !actor_ids.insert(actor.actor_id.as_str())
                || !actor.sentinel().validate()
                || ![
                    actor.position_m.x_m,
                    actor.position_m.y_m,
                    actor.position_m.z_m,
                ]
                .iter()
                .all(|n| n.is_finite())
        }) {
            return Err("E_SAVE_ACTOR_INVALID".into());
        }
        if self
            .generic_actors
            .iter()
            .any(|actor| !actor.validate() || !actor_ids.insert(actor.entity_id.as_str()))
        {
            return Err("E_SAVE_ACTOR_INVALID".into());
        }
        if self
            .combat_handled_request_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            != self.combat_handled_request_ids.len()
        {
            return Err("E_SAVE_COMBAT_REQUEST_INVALID".into());
        }
        if self
            .inventory
            .items
            .iter()
            .any(|item| item.item_id.trim().is_empty() || item.quantity == 0)
            || self
                .inventory
                .items
                .iter()
                .map(|item| item.item_id.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != self.inventory.items.len()
            || self
                .dialog_flags
                .0
                .iter()
                .any(|flag| flag.trim().is_empty())
        {
            return Err("E_SAVE_CONTENT_INVALID".into());
        }
        if (self.scene_id == self.world_id && !self.scene_states.is_empty())
            || (self.scene_id != self.world_id && self.scene_states.len() != 1)
        {
            return Err("E_SAVE_SCENE_INVALID".into());
        }
        let mut scene_ids = BTreeSet::new();
        for scene in &self.scene_states {
            if scene.scene_id != self.scene_id
                || !scene_ids.insert(scene.scene_id.as_str())
                || scene.checkpoint_id != self.checkpoint_id
            {
                return Err("E_SAVE_SCENE_INVALID".into());
            }
            if unique_nonempty(&scene.activated_ids).is_err()
                || unique_nonempty(&scene.emitted_event_ids).is_err()
            {
                return Err("E_SAVE_SCENE_INVALID".into());
            }
        }
        if let Some(checkpoint) = &self.checkpoint_id {
            if checkpoint.trim().is_empty() || checkpoint.len() > 128 {
                return Err("E_SAVE_CHECKPOINT_INVALID".into());
            }
        }
        if let Some(provenance) = &self.migration_provenance {
            let valid_method = matches!(
                (
                    provenance.source_format.as_str(),
                    provenance.method.as_str()
                ),
                (
                    "formal-save-v3.json",
                    "validated-v4-copy; Grey Hive checkpoint mapping; original retained"
                ) | (
                    "slot-v4.json",
                    "validated-v4-slot-copy; Grey Hive checkpoint mapping; original retained"
                )
            );
            if !valid_method
                || provenance.source_version != 4
                || provenance.source_world_id != WORLD_GREY_HIVE
            {
                return Err("E_SAVE_MIGRATION_PROVENANCE_INVALID".into());
            }
        }
        Ok(())
    }

    pub(crate) fn restore_state(&self) -> Result<RuntimeState, String> {
        self.validate()?;
        let epoch = self
            .revision
            .world_epoch
            .checked_add(1)
            .ok_or("E_WORLD_EPOCH_EXHAUSTED")?;
        let mut world = WorldStateV3::new(self.world_id.clone(), epoch, self.player.position_m)
            .map_err(|error| format!("E_SAVE_WORLD_RESTORE: {error:?}"))?;
        world.scene_id = self.scene_id.clone();
        world.checkpoint_id = self.checkpoint_id.clone();
        world.revision = WorldRevision {
            world_epoch: epoch,
            ..self.revision
        };
        world.server_time_ms = self.server_time_ms;
        world.player = KccBody {
            position_m: self.player.position_m,
            velocity_mps: self.player.velocity_mps,
            radius_m: self.player.radius_m,
            grounded: self.player.grounded,
            move_speed_mps: self.player.move_speed_mps,
            jump_speed_mps: self.player.jump_speed_mps,
            gravity_mps2: self.player.gravity_mps2,
            dash_speed_mps: self.player.dash_speed_mps,
            dash_remaining_ms: self.player.dash_remaining_ms,
            dash_cooldown_remaining_ms: self.player.dash_cooldown_remaining_ms,
        };
        world.player_hp = self.player.current_hp;
        world.player_max_hp = self.player.max_hp;
        world.player_energy = self.player.current_energy;
        world.player_max_energy = self.player.max_energy;
        if self.world_id == WORLD_GREY_HIVE
            && self.scene_id == WORLD_GREY_HIVE
            && !StaticKccWorld::grey_hive_first_flow(route_has_power(&self.progression))
                .can_occupy(world.player.position_m, world.player.radius_m)
        {
            return Err("E_SAVE_NUMERIC_INVALID".into());
        }
        world.sentinels = self
            .actors
            .iter()
            .map(ActorSaveV5::sentinel)
            .collect();
        world.generic_actors = self.generic_actors.clone();
        world.combat_state =
            CombatState::from_handled_request_ids(self.combat_handled_request_ids.iter().copied())
                .map_err(|_| "E_SAVE_COMBAT_REQUEST_INVALID")?;
        world.explored = self.explored.clone();
        world.rear_view = self.rear_view.clone();
        let mut capabilities = self.capabilities.clone();
        rebase_capability_epoch(&mut capabilities, epoch)?;
        let stepper = FixedStepClock::new(
            FixedStepConfig::new(60, 1).map_err(|error| format!("E_SAVE_CLOCK: {error:?}"))?,
        );
        Ok(RuntimeState {
            world,
            world_persistent_v1: crate::world_persistent_v1::WorldPersistentState::default(),
            route: self.progression.clone(),
            capabilities,
            build_commands: Default::default(),
            progression_v6: crate::formal_runtime::build_v6::PlayerProgressionV6::default(),
            effect_sources_v6: Vec::new(),
            effective_rules_v6: crate::player_rules::EffectivePlayerRules::default(),
            kcc: StaticKccWorld::grey_hive_first_flow(route_has_power(&self.progression)),
            stepper,
            latest_sample: InputSample::new(epoch, 0, 0, 0.0, 0.0)
                .map_err(|error| format!("E_SAVE_INPUT: {error:?}"))?,
            latest_input_received_at: std::time::Instant::now(),
            pending_combat: Vec::new(),
            presentation_events: Vec::new(),
            next_presentation_event_id: 1,
            sound_cues: Vec::new(),
            next_sound_cue_id: 1,
            last_received_seq: 0,
            last_client_time_ms: 0,
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

fn capture_scene_state(
    scene: &crate::scene_runtime::SceneRuntime,
) -> Result<SceneStateSave, String> {
    let definition = scene.current_scene();
    let mut candidates: Vec<String> = definition
        .interactions
        .iter()
        .map(|item| item.id.clone())
        .chain(definition.triggers.iter().map(|item| item.id.clone()))
        .collect();
    candidates.sort();
    candidates.dedup();
    let activated_ids: Vec<_> = candidates
        .into_iter()
        .filter(|id| scene.object_activated(id))
        .collect();
    let emitted_event_ids = definition
        .emitted_event_ids()
        .into_iter()
        .filter(|id| scene.event_complete(id))
        .collect();
    Ok(SceneStateSave {
        scene_id: scene.scene_id.clone(),
        activated_ids,
        emitted_event_ids,
        checkpoint_id: scene.checkpoint_id.clone(),
    })
}

fn unique_nonempty(values: &[String]) -> Result<(), ()> {
    let mut ids = BTreeSet::new();
    if values
        .iter()
        .any(|value| value.trim().is_empty() || !ids.insert(value.as_str()))
    {
        Err(())
    } else {
        Ok(())
    }
}

pub(crate) fn same_schema_shape(actual: &serde_json::Value, known: &serde_json::Value) -> bool {
    match (actual, known) {
        (serde_json::Value::Object(actual), serde_json::Value::Object(known)) => {
            actual.len() == known.len()
                && actual.iter().all(|(key, value)| {
                    known
                        .get(key)
                        .is_some_and(|known_value| same_schema_shape(value, known_value))
                })
        }
        (serde_json::Value::Array(actual), serde_json::Value::Array(known)) => {
            actual.len() == known.len()
                && actual
                    .iter()
                    .zip(known)
                    .all(|(value, known_value)| same_schema_shape(value, known_value))
        }
        (serde_json::Value::Null, serde_json::Value::Null)
        | (serde_json::Value::Bool(_), serde_json::Value::Bool(_))
        | (serde_json::Value::Number(_), serde_json::Value::Number(_))
        | (serde_json::Value::String(_), serde_json::Value::String(_)) => true,
        _ => false,
    }
}

fn known_world(world: &str) -> bool {
    known_campaign_world(world) || world == "return_station"
}

fn known_campaign_world(world: &str) -> bool {
    matches!(
        world,
        WORLD_GREY_HIVE | WORLD_MIST_HARBOR | WORLD_CLOCKWORKS
    )
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

fn rebase_capability_epoch(state: &mut CapabilityState, epoch: u64) -> Result<(), String> {
    let mut value = serde_json::to_value(&*state).map_err(|_| "E_SAVE_CAPABILITY_INVALID")?;
    let object = value.as_object_mut().ok_or("E_SAVE_CAPABILITY_INVALID")?;
    for key in ["currentRevision", "lastTickRevision"] {
        if let Some(revision) = object.get_mut(key).filter(|value| !value.is_null()) {
            revision["worldEpoch"] = serde_json::json!(epoch);
        }
    }
    *state = serde_json::from_value(value).map_err(|_| "E_SAVE_CAPABILITY_INVALID")?;
    Ok(())
}

pub fn save_path(root: &Path) -> PathBuf {
    root.join(SAVE_V5_FILE_NAME)
}

pub fn read_save(root: &Path) -> Result<SaveV5, String> {
    let path = save_path(root);
    let mut file = File::open(path).map_err(|_| "E_NO_SAVE".to_string())?;
    let size = file
        .metadata()
        .map_err(|e| format!("E_SAVE_READ: {e}"))?
        .len();
    if size > MAX_SAVE_BYTES {
        return Err("E_SAVE_TOO_LARGE".into());
    }
    let mut raw = String::with_capacity(size as usize);
    file.read_to_string(&mut raw)
        .map_err(|e| format!("E_SAVE_READ: {e}"))?;
    let raw_value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "E_SAVE_CORRUPT".to_string())?;
    let save: SaveV5 =
        serde_json::from_value(raw_value.clone()).map_err(|_| "E_SAVE_CORRUPT".to_string())?;
    if !same_schema_shape(
        &raw_value,
        &serde_json::to_value(&save).map_err(|_| "E_SAVE_CORRUPT")?,
    ) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    save.validate()?;
    Ok(save)
}

pub fn read_or_migrate(root: &Path) -> Result<SaveV5, String> {
    if save_path(root).is_file() {
        return read_save(root);
    }
    if root.join(crate::save_v3::SAVE_FILE_NAME).is_file() {
        return migrate_v4(root);
    }
    Err("E_NO_SAVE".into())
}

pub fn has_valid_save(root: &Path) -> bool {
    read_save(root).is_ok() || read_v4(root).is_ok()
}

pub fn write_save(root: &Path, save: &SaveV5) -> Result<(), String> {
    save.validate()?;
    fs::create_dir_all(root).map_err(|e| format!("E_SAVE_DIR: {e}"))?;
    let target = save_path(root);
    let token = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let temp = root.join(format!(
        ".formal-save-v5.{}.{}.tmp",
        std::process::id(),
        token
    ));
    let backup = root.join(format!(
        ".formal-save-v5.{}.{}.bak",
        std::process::id(),
        token
    ));
    let bytes = serde_json::to_vec_pretty(save).map_err(|e| format!("E_SAVE_ENCODE: {e}"))?;
    let result = (|| {
        write_temp(&temp, &bytes)?;
        verify_file(&temp, save)?;
        if target.exists() {
            fs::rename(&target, &backup).map_err(|e| format!("E_SAVE_BACKUP: {e}"))?;
        }
        if let Err(error) = fs::rename(&temp, &target) {
            if backup.exists() {
                let _ = fs::rename(&backup, &target);
            }
            return Err(format!("E_SAVE_COMMIT: {error}"));
        }
        if let Err(error) = verify_file(&target, save) {
            let _ = fs::remove_file(&target);
            if backup.exists() {
                let _ = fs::rename(&backup, &target);
            }
            return Err(error);
        }
        sync_directory(root)?;
        if backup.exists() {
            fs::remove_file(&backup).map_err(|e| format!("E_SAVE_BACKUP_CLEANUP: {e}"))?;
        }
        sync_directory(root)?;
        Ok(())
    })();
    if temp.exists() {
        let _ = fs::remove_file(temp);
    }
    result
}

fn write_temp(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("E_SAVE_TEMP: {e}"))?;
    file.write_all(bytes)
        .map_err(|e| format!("E_SAVE_WRITE: {e}"))?;
    file.sync_all().map_err(|e| format!("E_SAVE_SYNC: {e}"))
}

fn verify_file(path: &Path, expected: &SaveV5) -> Result<(), String> {
    let mut raw = String::new();
    File::open(path)
        .and_then(|mut file| file.read_to_string(&mut raw))
        .map_err(|e| format!("E_SAVE_VERIFY_READ: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "E_SAVE_VERIFY".to_string())?;
    let saved: SaveV5 =
        serde_json::from_value(value.clone()).map_err(|_| "E_SAVE_VERIFY".to_string())?;
    if !same_schema_shape(
        &value,
        &serde_json::to_value(&saved).map_err(|_| "E_SAVE_VERIFY".to_string())?,
    ) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    saved.validate()?;
    if &saved != expected {
        return Err("E_SAVE_VERIFY_MISMATCH".into());
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("E_SAVE_DIR_SYNC: {e}"))?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub(crate) fn migrate_v4(root: &Path) -> Result<SaveV5, String> {
    let legacy = read_v4(root)?;
    let backup_path = root.join(format!("{}.bak", crate::save_v3::SAVE_FILE_NAME));
    if !backup_path.exists() {
        fs::copy(crate::save_v3::save_path(root), &backup_path)
            .map_err(|e| format!("E_SAVE_MIGRATION_BACKUP: {e}"))?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&backup_path)
            .and_then(|file| file.sync_all())
            .map_err(|e| format!("E_SAVE_MIGRATION_BACKUP_SYNC: {e}"))?;
        sync_directory(root)?;
    } else if fs::read(&backup_path).map_err(|e| format!("E_SAVE_MIGRATION_BACKUP_READ: {e}"))?
        != fs::read(crate::save_v3::save_path(root))
            .map_err(|e| format!("E_SAVE_MIGRATION_SOURCE_READ: {e}"))?
    {
        return Err("E_SAVE_MIGRATION_BACKUP_MISMATCH".into());
    }
    let save = from_v4(&legacy)?;
    save.validate()?;
    write_save(root, &save)?;
    Ok(save)
}

pub(crate) fn read_v4(root: &Path) -> Result<crate::save_v3::SaveV3, String> {
    let path = crate::save_v3::save_path(root);
    let mut file = File::open(path).map_err(|_| "E_NO_SAVE".to_string())?;
    let size = file
        .metadata()
        .map_err(|e| format!("E_SAVE_READ: {e}"))?
        .len();
    if size > MAX_SAVE_BYTES {
        return Err("E_SAVE_TOO_LARGE".into());
    }
    let mut raw = String::with_capacity(size as usize);
    file.read_to_string(&mut raw)
        .map_err(|e| format!("E_SAVE_READ: {e}"))?;
    let raw_value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "E_SAVE_CORRUPT".to_string())?;
    let save: crate::save_v3::SaveV3 =
        serde_json::from_value(raw_value.clone()).map_err(|_| "E_SAVE_CORRUPT".to_string())?;
    if save.schema_version != 4 {
        return Err("E_SAVE_MIGRATION_VERSION_UNSUPPORTED".into());
    }
    if !same_schema_shape(
        &raw_value,
        &serde_json::to_value(&save).map_err(|_| "E_SAVE_CORRUPT")?,
    ) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    save.validate()?;
    Ok(save)
}

pub(crate) fn from_v4(legacy: &crate::save_v3::SaveV3) -> Result<SaveV5, String> {
    legacy.validate()?;
    if legacy.schema_version != 4 || legacy.world_id != WORLD_GREY_HIVE {
        return Err("E_SAVE_MIGRATION_WORLD_UNSUPPORTED".into());
    }
    let scene_id = WORLD_GREY_HIVE;
    let checkpoint_id = if legacy.checkpoint_id == "gh_cp_power" {
        "gh_cp_power"
    } else {
        "gh_cp_airlock"
    };
    let save = SaveV5 {
        schema_version: SAVE_V5_SCHEMA_VERSION,
        build_version: SAVE_V5_BUILD_VERSION.into(),
        content_version: SAVE_V5_CONTENT_VERSION.into(),
        world_id: legacy.world_id.clone(),
        scene_id: scene_id.into(),
        checkpoint_id: Some(checkpoint_id.into()),
        player: legacy.player.clone(),
        actors: legacy
            .sentinels
            .iter()
            .map(|actor| ActorSaveV5 {
                actor_id: actor.entity_id.clone(),
                actor_type: "sentinel".into(),
                position_m: actor.position_m,
                hp: actor.hp,
                state: actor.state,
                state_remaining_ms: actor.state_remaining_ms,
                attack_serial: actor.attack_serial,
                active: actor.active,
                charge_controller: None,
            })
            .collect(),
        generic_actors: vec![],
        combat_handled_request_ids: legacy.combat_handled_request_ids.clone(),
        scene_states: vec![],
        progression: legacy.route.clone(),
        capabilities: legacy.capabilities.clone(),
        inventory: InventoryState::default(),
        dialog_flags: DialogFlags::default(),
        revision: legacy.revision,
        server_time_ms: legacy.server_time_ms,
        moving_support_frame: None,
        explored: legacy.explored.clone(),
        rear_view: legacy.rear_view.clone(),
        last_input_seq: legacy.last_input_seq,
        last_client_time_ms: legacy.last_client_time_ms,
        last_damaged_at_ms: legacy.last_damaged_at_ms,
        last_combat_at_ms: legacy.last_combat_at_ms,
        migration_provenance: Some(MigrationProvenance {
            source_format: "formal-save-v3.json".into(),
            source_version: legacy.schema_version,
            source_world_id: legacy.world_id.clone(),
            method: "validated-v4-copy; Grey Hive checkpoint mapping; original retained".into(),
        }),
    };
    save.validate()?;
    Ok(save)
}

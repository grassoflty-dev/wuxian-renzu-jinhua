//! Single Rust owner for the Freeze v0.2 continuous runtime and route state.
//!
//! Tauri commands submit validated input or route intent to this owner. The
//! browser cannot choose a timestep, mutate world state, or advance route data.

use crate::{
    capability_v1::{
        apply_command_at_revision, choose_first_enhancement, project_capabilities,
        tick_regeneration, CapabilityCommand, CapabilityState, CapabilityTickInput,
        RegenerationConfig, CAPABILITY_SCHEMA_VERSION, CAP_ENEMY_VITALS, CAP_LOCAL_MAP,
        CAP_REAR_VIEW, CAP_REGENERATION,
    },
    continuous_combat::CombatIntent,
    continuous_input::{InputSample, INPUT_PROTOCOL},
    continuous_kcc::StaticKccWorld,
    fixed_step::{FixedStepClock, FixedStepConfig},
    world_progression::{
        apply_route_command, project_route, RewardPolicy, RouteCommand, RouteResult, RouteState,
    },
    world_v3::{
        apply_effects_atomically, step_world, ActorView, ContinuousWorldError, CwStepInput,
        DoorView, EnemyVitals, RouteProjection, Transform, Vec3, WorldStateV3, WorldView,
    },
};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const FIXED_HZ: u32 = 20;
const MAX_STEPS_PER_FRAME: u32 = 1;
pub const POWER_CONSOLE_ID: &str = "gh_power_console";
const GATE_A_ID: &str = "gh_gate_a";
const POWER_EVENT_ID: &str = "hive_power";
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

#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CombatIntentRequest {
    Attack { request_id: u64 },
    Jump { request_id: u64 },
    Dash { request_id: u64 },
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
}

#[derive(Clone)]
pub(crate) struct RuntimeState {
    pub(crate) world: WorldStateV3,
    pub(crate) route: RouteState,
    pub(crate) capabilities: CapabilityState,
    pub(crate) kcc: StaticKccWorld,
    pub(crate) stepper: FixedStepClock,
    pub(crate) next_tick_at: Instant,
    pub(crate) last_received_seq: u64,
    pub(crate) last_client_time_ms: u64,
    pub(crate) last_damaged_at_ms: Option<u64>,
    pub(crate) last_combat_at_ms: Option<u64>,
}

/// Managed by Tauri as the sole formal runtime owner.
pub struct FormalRuntime {
    state: Mutex<RuntimeState>,
    save_root: PathBuf,
}

impl FormalRuntime {
    pub fn new() -> Result<Self, String> {
        let save_root =
            std::env::temp_dir().join(format!("wuxian-formal-runtime-test-{}", std::process::id()));
        Self::new_with_save_dir(save_root)
    }

    pub fn new_with_save_dir(save_root: PathBuf) -> Result<Self, String> {
        Ok(Self {
            state: Mutex::new(Self::initial_state(1)?),
            save_root,
        })
    }

    pub fn has_save(&self) -> bool {
        crate::save_v3::has_valid_save(&self.save_root)
    }

    pub fn save(&self) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let save = crate::save_v3::SaveV3::capture(&state)?;
        crate::save_v3::write_save(&self.save_root, &save)?;
        Ok(project_view(&state))
    }

    pub fn continue_saved(&self) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let save = crate::save_v3::read_save(&self.save_root)?;
        let restored = save.restore()?;
        *state = restored;
        Ok(project_view(&state))
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
            route,
            capabilities,
            kcc: StaticKccWorld::grey_hive_first_flow(false),
            stepper,
            next_tick_at: Instant::now(),
            last_received_seq: 0,
            last_client_time_ms: 0,
            last_damaged_at_ms: None,
            last_combat_at_ms: None,
        })
    }

    /// Start a fresh in-memory Grey Hive blockout. This is a New/reset action,
    /// not a persisted Save v3 restore.
    pub fn reset_new(&self) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        let next_epoch = state
            .world
            .revision
            .world_epoch
            .checked_add(1)
            .ok_or_else(|| "E_WORLD_EPOCH_EXHAUSTED".to_string())?;
        *state = Self::initial_state(next_epoch)?;
        Ok(project_view(&state))
    }

    pub fn snapshot(&self) -> Result<WorldView, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        Ok(project_view(&state))
    }

    /// Rust validates the fixed actor id and actual authoritative player
    /// position; the browser cannot submit a route-progress event directly.
    pub fn interact(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> Result<FormalInteractionResponse, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if actor_id != POWER_CONSOLE_ID {
            return Ok(interaction_rejected(&state, "E_INTERACTION_ACTOR_UNKNOWN"));
        }
        if request_id.trim().is_empty() || request_id.len() > 128 {
            return Ok(interaction_rejected(
                &state,
                "E_INTERACTION_REQUEST_INVALID",
            ));
        }
        let player = state.world.player.position_m;
        let dx = player.x_m - POWER_CONSOLE_POSITION.x_m;
        let dy = player.y_m - POWER_CONSOLE_POSITION.y_m;
        let dz = player.z_m - POWER_CONSOLE_POSITION.z_m;
        if dx * dx + dy * dy + dz * dz > POWER_INTERACTION_RANGE_M * POWER_INTERACTION_RANGE_M {
            return Ok(interaction_rejected(&state, "E_INTERACTION_OUT_OF_RANGE"));
        }
        if has_progress(&state, POWER_EVENT_ID) {
            return Ok(FormalInteractionResponse {
                applied: false,
                already_applied: true,
                error_code: None,
                events: vec![],
                view: project_view(&state),
            });
        }

        let mut candidate = state.route.clone();
        let result = apply_route_command(
            &mut candidate,
            RouteCommand::Progress {
                event_id: POWER_EVENT_ID.into(),
                request_id: request_id.into(),
            },
            state.world.revision,
        );
        match result {
            RouteResult::Applied { events, .. } => {
                state.route = candidate;
                state.kcc = StaticKccWorld::grey_hive_first_flow(true);
                Ok(FormalInteractionResponse {
                    applied: true,
                    already_applied: false,
                    error_code: None,
                    events: events
                        .into_iter()
                        .map(|event| format!("{event:?}"))
                        .collect(),
                    view: project_view(&state),
                })
            }
            RouteResult::Rejected { code, message } => Ok(FormalInteractionResponse {
                applied: false,
                already_applied: false,
                error_code: Some(format!("{code:?}: {message}")),
                events: vec![],
                view: project_view(&state),
            }),
        }
    }

    /// Accept one sequenced input sample and advance at most one fixed Rust tick.
    /// Calls arriving faster than 20 Hz wait for the next server tick deadline.
    pub fn submit_input(
        &self,
        sample: InputSample,
        combat: Vec<CombatIntentRequest>,
    ) -> Result<WorldView, String> {
        if sample.protocol != INPUT_PROTOCOL {
            return Err("E_INPUT_PROTOCOL".into());
        }
        let sample = InputSample::new(
            sample.world_epoch,
            sample.seq,
            sample.client_time_ms,
            sample.move_x,
            sample.move_z,
        )
        .map_err(|error| format!("E_INPUT_INVALID: {error:?}"))?;

        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if sample.world_epoch != state.world.revision.world_epoch {
            return Err("E_INPUT_STALE_EPOCH".into());
        }
        if sample.seq <= state.last_received_seq {
            return Err("E_INPUT_STALE_SEQUENCE".into());
        }
        if sample.client_time_ms < state.last_client_time_ms {
            return Err("E_INPUT_TIME_REGRESSION".into());
        }

        wait_until(state.next_tick_at);
        let backup = state.clone();
        let dt_s = state.stepper.config.dt_s();
        let intents: Vec<CombatIntent> = combat.into_iter().map(Into::into).collect();
        let stepped = (|| {
            let steps = state
                .stepper
                .push_frame(dt_s)
                .map_err(|error| format!("E_TICK_SCHEDULER: {error:?}"))?;
            if steps != 1 {
                return Err("E_TICK_SCHEDULER: fixed frame did not produce one step".into());
            }
            let RuntimeState { world, kcc, .. } = &mut *state;
            let output = step_world(
                world,
                kcc,
                CwStepInput {
                    sample: &sample,
                    dt_s,
                    combat: &intents,
                },
            )
            .map_err(|error| format!("E_WORLD_STEP: {error:?}"))?;
            let now_ms = state.world.server_time_ms;
            if output.combat.iter().any(|event| {
                matches!(
                    event,
                    crate::continuous_combat::CombatEvent::PlayerDamaged { .. }
                )
            }) {
                state.last_damaged_at_ms = Some(now_ms);
            }
            if !output.combat.is_empty() {
                state.last_combat_at_ms = Some(now_ms);
            }
            let explored_map = state.world.explored.clone();
            let current_hp = state.world.player_hp;
            let max_hp = state.world.player_max_hp;
            let last_damaged_at_ms = state.last_damaged_at_ms;
            let last_combat_at_ms = state.last_combat_at_ms;
            state
                .capabilities
                .set_explored_map(explored_map)
                .map_err(|error| format!("E_CAPABILITY_MAP: {error:?}"))?;
            let capability_output = tick_regeneration(
                &mut state.capabilities,
                CapabilityTickInput {
                    revision: output.revision,
                    now_ms,
                    dt_s,
                    current_hp,
                    max_hp,
                    last_damaged_at_ms,
                    last_combat_at_ms,
                    combat_events: &output.combat,
                },
            )
            .map_err(|error| format!("E_CAPABILITY_TICK: {error:?}"))?;
            if !capability_output.effects.is_empty() {
                apply_effects_atomically(&mut state.world, &capability_output.effects)
                    .map_err(|error: ContinuousWorldError| format!("E_WORLD_EFFECT: {error:?}"))?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = stepped {
            *state = backup;
            return Err(error);
        }
        state.last_received_seq = sample.seq;
        state.last_client_time_ms = sample.client_time_ms;
        state.next_tick_at = Instant::now() + Duration::from_secs_f32(dt_s);
        Ok(project_view(&state))
    }

    pub fn apply_route_command(
        &self,
        request: RouteCommandRequest,
    ) -> Result<FormalRouteResponse, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
        if let Some(target) = request.target_world() {
            let canonical = crate::world_progression::migrate_world_id(target)
                .map_err(|code| format!("E_ROUTE_REJECTED: {code:?}"))?;
            if canonical != state.world.world_id {
                return Err("E_WORLD_NOT_LOADED".into());
            }
        }
        let mut candidate = state.route.clone();
        let result = crate::world_progression::apply_route_command(
            &mut candidate,
            request.into_route_command(),
            state.world.revision,
        );
        let (applied, error_code, events) = match result {
            RouteResult::Applied { events, .. } => {
                state.route = candidate;
                state.kcc =
                    StaticKccWorld::grey_hive_first_flow(has_progress(&state, POWER_EVENT_ID));
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
        Ok(FormalRouteResponse {
            applied,
            error_code,
            events,
            view: project_view(&state),
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
        state.world = world;
        state.capabilities = candidate;
        Ok(project_view(&state))
    }

    pub fn choose_first_enhancement(&self, capability_id: &str) -> Result<WorldView, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "E_RUNTIME_LOCK_POISONED".to_string())?;
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
        state.world = world;
        state.capabilities = candidate;
        Ok(project_view(&state))
    }
}

fn wait_until(deadline: Instant) {
    if let Some(delay) = deadline.checked_duration_since(Instant::now()) {
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn project_view(state: &RuntimeState) -> WorldView {
    let enemy_vitals = Vec::<EnemyVitals>::new();
    let capabilities = project_capabilities(
        &state.capabilities,
        state.world.explored.clone(),
        enemy_vitals,
        state.world.rear_view.clone(),
    );
    let progression: RouteProjection = project_route(&state.route);
    let mut view = state
        .world
        .view(state.world.last_input_seq, capabilities, progression);
    view.actors.push(ActorView {
        entity_id: POWER_CONSOLE_ID.into(),
        actor_kind: "power_console".into(),
        transform: Transform {
            position_m: POWER_CONSOLE_POSITION,
            yaw_rad: 0.0,
        },
        active: true,
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

fn interaction_rejected(state: &RuntimeState, code: &str) -> FormalInteractionResponse {
    FormalInteractionResponse {
        applied: false,
        already_applied: false,
        error_code: Some(code.into()),
        events: vec![],
        view: project_view(state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_projects_snapshot_advances_fixed_tick_and_composes_route() {
        let runtime = FormalRuntime::new().unwrap();
        let initial = runtime.snapshot().unwrap();
        assert_eq!(initial.world_id, "grey_hive");
        assert_eq!(initial.schema_version, "freeze-v02-interfaces/1.2");
        assert_eq!(initial.progression.current_world_id, "grey_hive");
        assert_eq!(initial.progression.worlds[0].visit_id, 1);
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
    fn capability_first_choice_is_rust_granted_and_appears_in_projection() {
        let runtime = FormalRuntime::new().unwrap();
        let view = runtime.choose_first_enhancement(CAP_REAR_VIEW).unwrap();
        assert!(view.capabilities.rear_view.granted);
        assert!(view
            .capabilities
            .items
            .iter()
            .any(|item| item.capability_id == CAP_REAR_VIEW && item.granted));
        assert!(runtime.choose_first_enhancement(CAP_LOCAL_MAP).is_err());
    }
}

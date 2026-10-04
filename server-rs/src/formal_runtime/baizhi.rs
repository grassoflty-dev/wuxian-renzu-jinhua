//! Narrow, transient authority for the optional Bio dialogue. Saves retain only
//! the existing GH hive_choice. No actor, encounter, reward or route event is added.
use super::*;
use crate::world_progression::HiveChoice;
use std::collections::BTreeMap;

pub const ENTITY_ID: &str = "gh_bz_whitezhi_v1";
pub const ENTITY_TYPE: &str = "npc.baizhi";
pub const INTERACTION_ID: &str = "gh_bz_first_contact";
pub const POSITION: [f32; 3] = [8.0, 0.0, 10.5];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaizhiDialogueTicket {
    pub owner_id: String,
    pub generation: u64,
    pub world_id: String,
    pub scene_id: String,
    pub world_epoch: u64,
    pub entity_id: String,
    pub interaction_id: String,
    pub pause_command_sequence: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaizhiBeginRequest {
    pub context: SessionContext,
    pub entity_id: String,
    pub interaction_id: String,
    pub owner_id: String,
    pub generation: u64,
    pub command_sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaizhiProjection {
    pub schema_version: u32,
    pub choice: HiveChoice,
    pub available: bool,
    pub can_interact: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaizhiNpcProjection {
    pub entity_id: String,
    pub entity_type: String,
    pub position: [f32; 3],
    pub yaw_rad: f64,
    pub interactable: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaizhiCommandReceipt {
    pub receipt: CommandReceipt,
    pub ticket: Option<BaizhiDialogueTicket>,
    pub resumed: bool,
}
#[derive(Clone, Debug)]
struct RequestBinding { ticket: BaizhiDialogueTicket, choice: HiveChoice, succeeded: bool }
#[derive(Clone, Debug, Default)]
pub(crate) struct DialogueState {
    pub(crate) ticket: Option<BaizhiDialogueTicket>,
    pub(crate) revoked: bool,
    requests: BTreeMap<String, RequestBinding>,
    last_closed: Option<BaizhiDialogueTicket>,
}
impl DialogueState {
    pub(crate) fn owns_pause(&self) -> bool { self.ticket.is_some() && !self.revoked }
    pub(crate) fn revoke(&mut self) { if self.ticket.is_some() { self.revoked = true; } }
}
fn same_session(state: &RuntimeState, world: &str, scene: &str, epoch: u64) -> bool {
    state.world.world_id == world && state.world.scene_id == scene
        && state.world.revision.world_epoch == epoch && epoch > 0 && epoch <= build_ui::MAX_SAFE_REVISION
}
fn choice(state: &RuntimeState) -> Option<HiveChoice> {
    let mut rows = state.route.progress.iter().filter(|row| row.world_id == "grey_hive");
    let result = rows.next()?.hive_choice.clone();
    rows.next().is_none().then_some(result)
}
fn configured(state: &RuntimeState, scene: &SceneRuntime) -> bool {
    let d = scene.current_scene();
    if !same_session(state, "grey_hive", "gh_bio_isolation", scene.world_epoch)
        || d.world_id != "grey_hive" || d.scene_id != "gh_bio_isolation" { return false; }
    let npcs: Vec<_> = d.spawns.iter().filter(|spawn| spawn.kind == "npc"
        || spawn.id == ENTITY_ID || spawn.entity_type.as_deref() == Some(ENTITY_TYPE)).collect();
    let markers: Vec<_> = d.interactions.iter().filter(|item| item.id == INTERACTION_ID || item.kind == "npc_dialogue").collect();
    npcs.len() == 1 && npcs[0].id == ENTITY_ID && npcs[0].kind == "npc"
        && npcs[0].entity_type.as_deref() == Some(ENTITY_TYPE) && npcs[0].position == POSITION
        && markers.len() == 1 && markers[0].id == INTERACTION_ID && markers[0].kind == "npc_dialogue"
        && markers[0].position == POSITION && markers[0].event.is_none()
        && markers[0].range_m.unwrap_or(2.5) == 2.5 && choice(state).is_some()
}
fn safe_site(state: &RuntimeState, scene: &SceneRuntime) -> Result<(), String> {
    if !configured(state, scene) { return Err("E_BAIZHI_CONFIGURATION".into()); }
    if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
    if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
    if state.last_owner_error.is_some() { return Err("E_BAIZHI_OWNER_UNAVAILABLE".into()); }
    let p = state.world.player.position_m;
    let distance = ((p.x_m - POSITION[0]).powi(2) + (p.y_m - POSITION[1]).powi(2) + (p.z_m - POSITION[2]).powi(2)).sqrt();
    if !distance.is_finite() || distance > 2.5 || !scene.current_scene().height_allows(INTERACTION_ID, p.y_m) {
        return Err("E_BAIZHI_OUT_OF_RANGE".into());
    }
    Ok(())
}
pub(super) fn project(state: &RuntimeState, scene: &SceneRuntime, view: &mut WorldView) {
    if !configured(state, scene) { return; }
    let can_interact = !state.paused && !state.baizhi.owns_pause() && safe_site(state, scene).is_ok();
    view.baizhi = Some(BaizhiProjection { schema_version: 1, choice: choice(state).unwrap(), available: true, can_interact });
    view.npcs = Some(vec![BaizhiNpcProjection { entity_id: ENTITY_ID.into(), entity_type: ENTITY_TYPE.into(),
        position: POSITION, yaw_rad: std::f64::consts::PI, interactable: can_interact }]);
}
fn clear_inputs(state: &mut RuntimeState) {
    state.latest_sample.move_x = 0.0;
    state.latest_sample.move_z = 0.0;
    state.world.player.velocity_mps = Vec3::zero();
    state.pending_combat.clear();
    if state.world.combat_state.active_action.as_ref().is_some_and(|action|
        action.kind == crate::continuous_combat::CombatActionKind::Guard) {
        state.world.combat_state.active_action = None;
        state.world.combat_state.action_locks_facing = false;
    }
    state.stepper = FixedStepClock::new(state.stepper.config);
}
impl FormalRuntime {
    /// All preflight checks and the simulation hold share the sole owner mutex.
    pub fn baizhi_begin(&self, request: BaizhiBeginRequest) -> Result<BaizhiCommandReceipt, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if !same_session(&state, &request.context.world_id, &request.context.scene_id, request.context.world_epoch) {
            return Err("E_BAIZHI_STALE_CONTEXT".into());
        }
        if request.entity_id != ENTITY_ID || request.interaction_id != INTERACTION_ID || !valid_id(&request.owner_id)
            || request.generation == 0 || request.generation > build_ui::MAX_SAFE_REVISION {
            return Err("E_BAIZHI_REQUEST_INVALID".into());
        }
        if request.command_sequence == 0 || request.command_sequence > build_ui::MAX_SAFE_REVISION
            || request.command_sequence <= state.last_lifecycle_sequence { return Err("E_LIFECYCLE_STALE_COMMAND".into()); }
        if state.paused || state.baizhi.owns_pause() { return Err("E_RUNTIME_PAUSED".into()); }
        let scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let active = scene.as_ref().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        safe_site(&state, active)?;
        let ticket = BaizhiDialogueTicket { owner_id: request.owner_id, generation: request.generation,
            world_id: request.context.world_id, scene_id: request.context.scene_id, world_epoch: request.context.world_epoch,
            entity_id: request.entity_id, interaction_id: request.interaction_id, pause_command_sequence: request.command_sequence };
        let mut candidate = state.clone();
        candidate.world.bump_authority_revision().map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
        candidate.last_lifecycle_sequence = request.command_sequence;
        candidate.paused = true;
        candidate.enhancement_terminal = None;
        clear_inputs(&mut candidate);
        candidate.baizhi.ticket = Some(ticket.clone());
        candidate.baizhi.revoked = false;
        candidate.baizhi.last_closed = None;
        ordinary_enemy_presentation::prepare_restored(&mut candidate, active);
        *state = candidate;
        Ok(BaizhiCommandReceipt { receipt: CommandReceipt::from_view(
            format!("baizhi-begin:{}:{}", ticket.owner_id, ticket.generation), project_scene_view(&state, Some(active))),
            ticket: Some(ticket), resumed: false })
    }
    pub fn baizhi_commit(&self, ticket: &BaizhiDialogueTicket, request_id: &str, selected: HiveChoice) -> Result<BaizhiCommandReceipt, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if !valid_id(request_id) { return Err("E_BAIZHI_REQUEST_INVALID".into()); }
        // Payload identity precedes outcome checks. A failed write does not erase
        // the binding, while a successful replay never performs another write.
        if let Some(binding) = state.baizhi.requests.get(request_id) {
            if binding.ticket != *ticket || binding.choice != selected { return Err("E_BAIZHI_REQUEST_PAYLOAD_MISMATCH".into()); }
            if binding.succeeded { return Err("E_BAIZHI_DUPLICATE_REQUEST".into()); }
        }
        if !same_session(&state, &ticket.world_id, &ticket.scene_id, ticket.world_epoch) { return Err("E_BAIZHI_STALE_CONTEXT".into()); }
        if state.baizhi.ticket.as_ref() != Some(ticket) || !state.baizhi.owns_pause()
            || ticket.pause_command_sequence != state.last_lifecycle_sequence || !state.paused {
            return Err("E_BAIZHI_TICKET_INVALID".into());
        }
        let scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let active = scene.as_ref().ok_or("E_SCENE_REGISTRY_NOT_LOADED")?;
        state.baizhi.requests.entry(request_id.into()).or_insert_with(|| RequestBinding { ticket: ticket.clone(), choice: selected.clone(), succeeded: false });
        safe_site(&state, active)?;
        if selected == HiveChoice::Unresolved { return Err("E_BAIZHI_CHOICE_NOT_COMMITTABLE".into()); }
        if choice(&state) != Some(HiveChoice::Unresolved) { return Err("E_BAIZHI_TERMINAL_CHOICE".into()); }
        let mut candidate = state.clone();
        if !candidate.route.set_hive_choice_once(selected) { return Err("E_BAIZHI_TERMINAL_CHOICE".into()); }
        candidate.world.bump_authority_revision().map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
        let save = crate::save_v6::SaveV6::capture(&candidate, scene.as_ref())?;
        self.persist_current_progress(&save)?;
        candidate.baizhi.requests.get_mut(request_id).unwrap().succeeded = true;
        *state = candidate;
        Ok(BaizhiCommandReceipt { receipt: CommandReceipt::from_view(request_id, project_scene_view(&state, Some(active))),
            ticket: Some(ticket.clone()), resumed: false })
    }
    pub fn baizhi_close(&self, ticket: &BaizhiDialogueTicket) -> Result<BaizhiCommandReceipt, String> {
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        if !same_session(&state, &ticket.world_id, &ticket.scene_id, ticket.world_epoch) { return Err("E_BAIZHI_STALE_CONTEXT".into()); }
        let scene = self.scene_runtime.lock().map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let command = format!("baizhi-close:{}:{}", ticket.owner_id, ticket.generation);
        if state.baizhi.ticket.as_ref() != Some(ticket) {
            if state.baizhi.last_closed.as_ref() == Some(ticket) {
                return Ok(BaizhiCommandReceipt { receipt: CommandReceipt::from_view(command, project_scene_view(&state, scene.as_ref())),
                    ticket: None, resumed: !state.paused && state.last_lifecycle_sequence == ticket.pause_command_sequence });
            }
            return Err("E_BAIZHI_TICKET_INVALID".into());
        }
        let revoked = state.baizhi.revoked || state.last_lifecycle_sequence != ticket.pause_command_sequence;
        let mut candidate = state.clone();
        if !revoked {
            if !state.paused { return Err("E_BAIZHI_PAUSE_UNCONFIRMED".into()); }
            if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
            if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
            if state.last_owner_error.is_some() { return Err("E_BAIZHI_OWNER_UNAVAILABLE".into()); }
            candidate.world.bump_authority_revision().map_err(|e| format!("E_WORLD_REVISION: {e:?}"))?;
            candidate.paused = false;
            clear_inputs(&mut candidate);
            if let Some(active) = scene.as_ref() { ordinary_enemy_presentation::prepare_restored(&mut candidate, active); }
        }
        // No fallible operation follows release: failed resume retains the ticket.
        candidate.baizhi.ticket = None;
        candidate.baizhi.last_closed = Some(ticket.clone());
        candidate.baizhi.revoked = false;
        *state = candidate;
        Ok(BaizhiCommandReceipt { receipt: CommandReceipt::from_view(command, project_scene_view(&state, scene.as_ref())),
            ticket: None, resumed: !revoked })
    }
}

#[cfg(test)]
mod tests;

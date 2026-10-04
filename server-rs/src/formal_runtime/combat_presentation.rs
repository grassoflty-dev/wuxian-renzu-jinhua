//! Project resolved owner events only. Snapshot differences cannot establish hits.
use super::{emit_presentation, valid_id, RuntimeState};
use crate::continuous_combat::{CombatActionKind, CombatContact, CombatEvent};
use crate::world_v3::{CombatFeedback, CombatOutcome};

fn feedback(
    state: &mut RuntimeState,
    kind: &str,
    contact: Option<CombatContact>,
    outcome: CombatOutcome,
    target_id: Option<&str>,
    source_id: &str,
    reason: Option<&str>,
) {
    let Some(contact) = contact else { return };
    // Presentation has no permission to invent an actor, location, or request ID.
    if !valid_id(&state.world.world_id) || !valid_id(&state.world.scene_id)
        || !valid_id(source_id) || target_id.is_some_and(|id| !valid_id(id))
        || contact.request_id.is_some_and(|id| id == 0 || id > super::build_ui::MAX_SAFE_REVISION)
        || !contact.direction_rad.is_finite()
        || ![contact.position_m.x_m, contact.position_m.y_m, contact.position_m.z_m].iter().all(|n| n.is_finite()) {
        return;
    }
    if outcome == CombatOutcome::Rejected && contact.request_id.is_none() { return; }
    let enemy_id = if outcome == CombatOutcome::EnemyHit { target_id } else if source_id != "player" { Some(source_id) } else { None };
    if let Some(id) = enemy_id {
        let count = state.world.sentinels.iter().filter(|actor| actor.entity_id == id).count()
            + state.world.generic_actors.iter().filter(|actor| actor.entity_id == id).count();
        if count != 1 { return; }
    }
    if outcome == CombatOutcome::EnemyHit {
        if let Some(actor) = state.world.generic_actors.iter().find(|actor| Some(actor.entity_id.as_str()) == target_id) {
            if actor.entity_type == super::signal_wraith_roster::WRAITH
                && actor.controller_variant == 1
                && !super::signal_wraith_presentation::projection(state, actor).is_some_and(|projection| projection.precise) {
                return;
            }
        }
    }
    emit_presentation(state, kind, contact.position_m, 1.0);
    let event = state.presentation_events.last_mut().expect("just emitted");
    event.direction_rad = contact.direction_rad;
    event.duration_ms = Some(match outcome {
        CombatOutcome::EnemyHit => 220,
        CombatOutcome::PlayerHurt | CombatOutcome::Blocked => 260,
        CombatOutcome::Absorbed => 160,
        CombatOutcome::Rejected => 1000,
    });
    event.combat_feedback = Some(CombatFeedback {
        world_id: state.world.world_id.clone(),
        scene_id: state.world.scene_id.clone(),
        outcome,
        target_id: target_id.map(str::to_owned),
        source_id: Some(source_id.into()),
        request_id: contact.request_id,
        reason: reason.map(str::to_owned),
    });
}

pub(super) fn present_events(state: &mut RuntimeState, events: &[CombatEvent]) {
    let player_position = state.world.player.position_m;
    for event in events {
        match event {
            // Modern J emits ActionStarted at windup and ActionImpact at contact.
            // The compatibility AttackHit may arrive on a later tick; its own
            // origin flag prevents replaying a second start/hit then.
            CombatEvent::AttackHit { target_id, damage, modern: false, contact, .. } => {
                emit_presentation(state, "AttackStarted", player_position, 1.0);
                if *damage > 0 {
                    feedback(state, "Hit", *contact, CombatOutcome::EnemyHit, Some(target_id), "player", None);
                }
            }
            CombatEvent::AttackHit { .. } => {}
            CombatEvent::AttackMiss { modern, contact, .. } => {
                if !modern { emit_presentation(state, "AttackStarted", player_position, 0.7); }
                feedback(state, "ActionRejected", *contact, CombatOutcome::Rejected, None, "player", Some("no_target"));
            }
            CombatEvent::ActionMiss { contact, .. } => {
                feedback(state, "ActionRejected", Some(*contact), CombatOutcome::Rejected, None, "player", Some("no_target"));
            }
            CombatEvent::PlayerDamaged { source_id, damage, contact } => {
                if *damage > 0 {
                    feedback(state, "Damaged", *contact, CombatOutcome::PlayerHurt, Some("player"), source_id, None);
                }
            }
            CombatEvent::ActionStarted { action, .. } => {
                let kind = match action {
                    CombatActionKind::PrimaryAttack => "AttackStarted",
                    CombatActionKind::Dash => "DashStarted",
                    CombatActionKind::Pulse => "PulseCast",
                    CombatActionKind::Guard => "GuardWindup",
                    CombatActionKind::Pierce => "PierceStarted",
                };
                emit_presentation(state, kind, player_position, 1.0);
            }
            CombatEvent::ActionImpact { action, target_id, damage, contact, .. } => {
                let kind = match action {
                    CombatActionKind::PrimaryAttack => "Hit",
                    CombatActionKind::Pulse => "PulseHit",
                    CombatActionKind::Pierce => "PierceHit",
                    // Neither action resolves enemy damage in current tuning.
                    CombatActionKind::Dash | CombatActionKind::Guard => continue,
                };
                if *damage > 0 {
                    feedback(state, kind, *contact, CombatOutcome::EnemyHit, Some(target_id), "player", None);
                }
            }
            CombatEvent::GuardStarted { .. } => emit_presentation(state, "GuardStarted", player_position, 1.0),
            CombatEvent::GuardImpact { source_id, contact, .. } => {
                feedback(state, "GuardImpact", *contact, CombatOutcome::Blocked, Some("player"), source_id, None);
            }
            CombatEvent::GuardEnded { .. } => emit_presentation(state, "GuardEnded", player_position, 1.0),
            CombatEvent::TraversalStarted { .. } => emit_presentation(state, "ContextTraversalStarted", player_position, 1.0),
            CombatEvent::TraversalCompleted { .. } => emit_presentation(state, "ContextTraversalCompleted", player_position, 1.0),
            CombatEvent::TraversalBlocked { .. } => emit_presentation(state, "ContextTraversalBlocked", player_position, 0.5),
            CombatEvent::DamageAbsorbed { source_id, reason, contact } => {
                feedback(state, "DamageAbsorbed", *contact, CombatOutcome::Absorbed, Some("player"), source_id, Some(reason));
            }
            CombatEvent::IntentRejected { reason, contact, .. } => {
                feedback(state, "ActionRejected", *contact, CombatOutcome::Rejected, None, "player", Some(reason));
            }
        }
    }
}

#[cfg(test)]
mod tests;

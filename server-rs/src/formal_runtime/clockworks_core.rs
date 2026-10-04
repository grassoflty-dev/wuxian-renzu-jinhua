//! The Core objective is a console interaction after an authoritative Boss death.
//! Combat never emits the route event, and a content emitter alone cannot bypass
//! the encounter. Defeat survives scene visits; live encounter clocks do not.

use super::*;
use crate::continuous_combat::CombatEvent;
use crate::world_persistent_v1::ClockworksPersistentState;

pub(super) const CORE_CONSOLE_ID: &str = "cw_regulator_core_console_staged";
const CORE_EVENT_ID: &str = "clockworks_core";

/// Exists only between the before/after halves of one successful owner step.
pub(super) struct RegulatorCombatProof {
    epoch: u64,
    next_tick: u64,
}

fn registered_encounter(state: &RuntimeState, scene: &SceneRuntime) -> bool {
    scene.world_epoch == state.world.revision.world_epoch
        && state.route.current_world_id == "clockworks"
        && registered_clockworks_regulator_roster(scene.current_scene(), &state.world)
        && state.world.generic_actors[0].validate()
}

pub(super) fn capture_regulator_combat(
    state: &RuntimeState,
    scene: Option<&SceneRuntime>,
) -> Option<RegulatorCombatProof> {
    let scene = scene?;
    if !registered_encounter(state, scene)
        || state.world_persistent_v1.clockworks.regulator_defeated
        || state.world.generic_actors[0].hp == 0
    {
        return None;
    }
    Some(RegulatorCombatProof {
        epoch: state.world.revision.world_epoch,
        next_tick: state.world.revision.server_tick.checked_add(1)?,
    })
}

pub(super) fn record_regulator_combat_defeat(
    state: &mut RuntimeState,
    scene: Option<&SceneRuntime>,
    proof: Option<RegulatorCombatProof>,
    combat: &[CombatEvent],
) -> bool {
    let (Some(scene), Some(proof)) = (scene, proof) else {
        return false;
    };
    if proof.epoch != state.world.revision.world_epoch
        || proof.next_tick != state.world.revision.server_tick
        || !registered_encounter(state, scene)
        || state.world_persistent_v1.clockworks.regulator_defeated
        || state.world.generic_actors[0].hp != 0
        || !combat.iter().any(|event| match event {
            CombatEvent::AttackHit {
                target_id, damage, ..
            }
            | CombatEvent::ActionImpact {
                target_id, damage, ..
            } => target_id == CLOCKWORKS_REGULATOR_SPAWN_ID && *damage > 0,
            _ => false,
        })
    {
        return false;
    }
    // No phase prerequisite: a legitimate direct lethal hit must complete the
    // encounter even when it crosses every phase threshold in one owner step.
    state.world_persistent_v1.clockworks.regulator_defeated = true;
    state
        .world_persistent_v1
        .clockworks
        .reset_regulator_encounter();
    emit_presentation(
        state,
        "PrimeRegulatorDeath",
        state.world.generic_actors[0].position_m,
        1.0,
    );
    true
}

/// Called after actor restore/spawn, including old saves lacking actor records.
pub(super) fn restore_regulator_progression(
    world: &mut WorldStateV3,
    scene: &crate::scene_runtime::SceneDefinition,
    persistent: &ClockworksPersistentState,
) -> Result<(), String> {
    if scene.world_id != "clockworks" || scene.scene_id != CLOCKWORKS_REGULATOR_SCENE_ID {
        return Ok(());
    }
    // Generic scene fixtures without a Boss are not upgraded into encounters.
    if !scene.spawns.iter().any(|spawn| {
        spawn.id == CLOCKWORKS_REGULATOR_SPAWN_ID
            || spawn.entity_type.as_deref() == Some(CLOCKWORKS_REGULATOR_ENTITY_TYPE)
    }) {
        return Ok(());
    }
    if !registered_clockworks_regulator_roster(scene, world) || !world.generic_actors[0].validate()
    {
        return Err("E_CLOCKWORKS_REGULATOR_ROSTER_INVALID".into());
    }
    if persistent.regulator_defeated {
        world.generic_actors[0].take_damage(u32::MAX);
    }
    Ok(())
}

pub(super) fn validate_core_source(state: &RuntimeState, scene: &SceneRuntime) -> Result<(), String> {
    let definition = scene.current_scene();
    let consoles = definition
        .interactions
        .iter()
        .filter(|item| item.id == CORE_CONSOLE_ID || item.event.as_deref() == Some(CORE_EVENT_ID))
        .collect::<Vec<_>>();
    if !registered_encounter(state, scene)
        || consoles.len() != 1
        || consoles[0].id != CORE_CONSOLE_ID
        || consoles[0].kind != "terminal"
        || consoles[0].event.as_deref() != Some(CORE_EVENT_ID)
        || definition
            .triggers
            .iter()
            .any(|item| item.event == CORE_EVENT_ID)
        || definition
            .interaction_aggregates
            .iter()
            .any(|item| item.event == CORE_EVENT_ID)
    {
        return Err("E_CLOCKWORKS_CORE_SOURCE_INVALID".into());
    }
    if !state.world_persistent_v1.clockworks.regulator_defeated
        || state.world.generic_actors[0].hp != 0
    {
        return Err("E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED".into());
    }
    // Older saves may already carry route progress without the later durable
    // console receipt. They must physically reconfirm; never invent the receipt.
    if state.world_persistent_v1.clockworks.core_console_confirmed {
        return Err("E_CLOCKWORKS_CORE_DUPLICATE_EVENT".into());
    }
    Ok(())
}

/// Validate before cloning/activating the scene, so rejected F does not consume
/// the console, request ID, route event, or revision.
pub(super) fn validate_core_interaction(
    state: &RuntimeState,
    scene: &SceneRuntime,
    interaction_id: &str,
) -> Result<(), String> {
    if interaction_id != CORE_CONSOLE_ID {
        return Ok(());
    }
    validate_core_source(state, scene)?;
    Ok(())
}

pub(super) fn core_console_available(state: &RuntimeState, scene: &SceneRuntime) -> bool {
    validate_core_interaction(state, scene, CORE_CONSOLE_ID).is_ok()
}

/// Validate every Core-bearing event before generic event selection, including
/// forged extra events that could otherwise hide behind an unrelated first one.
pub(super) fn validate_core_event(
    state: &RuntimeState,
    scene: &SceneRuntime,
    events: &[SceneEvent],
) -> Result<(), String> {
    let core_related = events.iter().any(|event| match event {
        SceneEvent::Interaction { id, event_id } => {
            id == CORE_CONSOLE_ID || event_id.as_deref() == Some(CORE_EVENT_ID)
        }
        SceneEvent::Trigger { event_id, .. } => event_id == CORE_EVENT_ID,
        _ => false,
    });
    if !core_related {
        return Ok(());
    }
    validate_core_source(state, scene)?;
    if !matches!(events,
        [SceneEvent::Interaction { id, event_id: Some(event_id) }]
            if id == CORE_CONSOLE_ID && event_id == CORE_EVENT_ID)
        || !scene.object_activated(CORE_CONSOLE_ID)
        || !scene.event_complete(CORE_EVENT_ID)
    {
        return Err("E_CLOCKWORKS_CORE_SOURCE_INVALID".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn core_runtime() -> FormalRuntime {
        let runtime = FormalRuntime::new_with_save_dir(std::env::temp_dir().join(format!(
            "formal-cw-core-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        )))
        .unwrap();
        crate::production_scene_bootstrap::install(&runtime).unwrap();
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
        {
            let mut state = runtime.state.lock().unwrap();
            state.paused = false;
            state.route.current_world_id = "clockworks".into();
            state.world.player.position_m = Vec3::new(12.5, 0.0, 8.0).unwrap();
        }
        runtime
    }

    fn owner_attack(runtime: &FormalRuntime, seq: u64, count: u64) {
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = Vec3::new(12.5, 0.0, 8.0).unwrap();
        state.latest_sample =
            InputSample::new(state.world.revision.world_epoch, seq, seq * 17, 0.0, 0.0)
                .unwrap()
                .with_aim(1.0, 0.0)
                .unwrap();
        state.latest_input_received_at = Instant::now();
        for index in 0..count {
            state.pending_combat.push(CombatIntent::Attack {
                request_id: seq * 100 + index,
            });
        }
        advance_owner_step_with_scene(&mut state, scene_guard.as_ref()).unwrap();
    }

    fn assert_no_core_progress(runtime: &FormalRuntime) {
        let state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        assert!(!completed_world_events(&state, "clockworks").contains(CORE_EVENT_ID));
        assert!(!scene.as_ref().unwrap().event_complete(CORE_EVENT_ID));
        assert!(!scene.as_ref().unwrap().object_activated(CORE_CONSOLE_ID));
    }

    #[test]
    fn core_console_requires_real_defeat_and_commits_only_on_f() {
        let runtime = core_runtime();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let before = runtime.state.lock().unwrap().world.revision;
        let early = runtime
            .activate_scene_interaction(CORE_CONSOLE_ID, "core-before-death", epoch)
            .unwrap();
        assert!(!early.applied);
        assert_eq!(
            early.error_code.as_deref(),
            Some("E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED")
        );
        assert_eq!(runtime.state.lock().unwrap().world.revision, before);
        assert_no_core_progress(&runtime);
        assert!(
            !runtime
                .snapshot()
                .unwrap()
                .interactables
                .iter()
                .find(|item| item.entity_id == CORE_CONSOLE_ID)
                .unwrap()
                .active
        );

        owner_attack(&runtime, 1, 23);
        assert_eq!(runtime.state.lock().unwrap().world.generic_actors[0].hp, 25);
        assert!(
            !runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .clockworks
                .regulator_defeated
        );
        owner_attack(&runtime, 2, 1);
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .clockworks
                .regulator_defeated
        );
        assert_no_core_progress(&runtime);
        assert!(
            runtime
                .snapshot()
                .unwrap()
                .interactables
                .iter()
                .find(|item| item.entity_id == CORE_CONSOLE_ID)
                .unwrap()
                .active
        );

        let stale = runtime
            .activate_scene_interaction(CORE_CONSOLE_ID, "core-stale", epoch + 1)
            .unwrap();
        assert!(!stale.applied);
        assert_no_core_progress(&runtime);
        let response = runtime
            .activate_scene_interaction(CORE_CONSOLE_ID, "core-before-death", epoch)
            .unwrap();
        assert!(
            response.applied,
            "a rejected early F must not consume its request ID"
        );
        let committed_revision = runtime.state.lock().unwrap().world.revision;
        assert!(
            completed_world_events(&runtime.state.lock().unwrap(), "clockworks")
                .contains(CORE_EVENT_ID)
        );
        for request in ["core-before-death", "core-duplicate"] {
            assert!(
                !runtime
                    .activate_scene_interaction(CORE_CONSOLE_ID, request, epoch)
                    .unwrap()
                    .applied
            );
            assert_eq!(
                runtime.state.lock().unwrap().world.revision,
                committed_revision
            );
        }
        assert!(
            !runtime
                .snapshot()
                .unwrap()
                .interactables
                .iter()
                .find(|item| item.entity_id == CORE_CONSOLE_ID)
                .unwrap()
                .active
        );
    }

    #[test]
    fn direct_full_health_owner_kill_does_not_require_a_phase_or_emit_core() {
        let runtime = core_runtime();
        owner_attack(&runtime, 1, 24);
        let state = runtime.state.lock().unwrap();
        assert_eq!(state.world.generic_actors[0].hp, 0);
        assert!(state.world.generic_actors[0].validate());
        assert!(state.world_persistent_v1.clockworks.regulator_defeated);
        assert!(!state.world_persistent_v1.clockworks.regulator_phase2_active);
        assert!(!completed_world_events(&state, "clockworks").contains(CORE_EVENT_ID));
        drop(state);
        assert_no_core_progress(&runtime);
    }

    #[test]
    fn direct_high_damage_death_is_accepted_without_phase_prerequisites() {
        let runtime = core_runtime();
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let mut state = runtime.state.lock().unwrap();
        let proof = capture_regulator_combat(&state, scene_guard.as_ref());
        state.world.generic_actors[0].take_damage(10_000);
        state.world.advance_clock(1.0 / 60.0).unwrap();
        assert!(record_regulator_combat_defeat(
            &mut state,
            scene_guard.as_ref(),
            proof,
            &[CombatEvent::AttackHit {
                request_id: 1,
                target_id: CLOCKWORKS_REGULATOR_SPAWN_ID.into(),
                damage: 10_000,
                modern: false,
                contact: None,
            }]
        ));
        assert!(state.world_persistent_v1.clockworks.regulator_defeated);
        assert!(!completed_world_events(&state, "clockworks").contains(CORE_EVENT_ID));
    }

    #[test]
    fn regulator_defeat_rejects_unregistered_stale_empty_and_noncombat_death() {
        for invalid in [
            "wrong-id",
            "wrong-type",
            "wrong-home",
            "empty",
            "extra",
            "wrong-scene",
            "wrong-world",
            "wrong-route",
            "stale-scene",
            "already-dead",
        ] {
            let runtime = core_runtime();
            let mut scene_guard = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            match invalid {
                "wrong-id" => state.world.generic_actors[0].entity_id = "forged-regulator".into(),
                "wrong-type" => {
                    state.world.generic_actors[0].entity_type = CLOCKWORKS_ELITE_ENTITY_TYPE.into()
                }
                "wrong-home" => state.world.generic_actors[0].home_m.x_m += 1.0,
                "empty" => state.world.generic_actors.clear(),
                "extra" => {
                    let actor = state.world.generic_actors[0].clone();
                    state.world.generic_actors.push(actor);
                }
                "wrong-scene" => state.world.scene_id = CLOCKWORKS_ELITE_SCENE_ID.into(),
                "wrong-world" => state.world.world_id = "grey_hive".into(),
                "wrong-route" => state.route.current_world_id = "grey_hive".into(),
                "stale-scene" => scene_guard.as_mut().unwrap().world_epoch += 1,
                "already-dead" => state.world.generic_actors[0].take_damage(u32::MAX),
                _ => unreachable!(),
            }
            assert!(
                capture_regulator_combat(&state, scene_guard.as_ref()).is_none(),
                "{invalid}"
            );
            assert!(!state.world_persistent_v1.clockworks.regulator_defeated);
        }
        for invalid in [
            "no-hit",
            "zero-damage",
            "wrong-target",
            "wrong-epoch",
            "wrong-tick",
            "still-alive",
        ] {
            let runtime = core_runtime();
            let scene_guard = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            let proof = capture_regulator_combat(&state, scene_guard.as_ref());
            if invalid != "still-alive" {
                state.world.generic_actors[0].take_damage(u32::MAX);
            }
            state.world.advance_clock(1.0 / 60.0).unwrap();
            if invalid == "wrong-epoch" {
                state.world.revision.world_epoch += 1;
            }
            if invalid == "wrong-tick" {
                state.world.advance_clock(1.0 / 60.0).unwrap();
            }
            let combat = if invalid == "no-hit" {
                vec![]
            } else {
                vec![CombatEvent::AttackHit {
                    request_id: 1,
                    target_id: if invalid == "wrong-target" {
                        "forged-regulator"
                    } else {
                        CLOCKWORKS_REGULATOR_SPAWN_ID
                    }
                    .into(),
                    damage: if invalid == "zero-damage" { 0 } else { 600 },
                    modern: false,
                    contact: None,
                }]
            };
            assert!(
                !record_regulator_combat_defeat(&mut state, scene_guard.as_ref(), proof, &combat),
                "{invalid}"
            );
            assert!(!state.world_persistent_v1.clockworks.regulator_defeated);
        }
    }

    #[test]
    fn core_progress_rejects_forged_trigger_generic_empty_and_unactivated_events() {
        let runtime = core_runtime();
        owner_attack(&runtime, 1, 24);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let scene = scene_guard.as_ref().unwrap();
        let mut state = runtime.state.lock().unwrap();
        let genuine = SceneEvent::Interaction {
            id: CORE_CONSOLE_ID.into(),
            event_id: Some(CORE_EVENT_ID.into()),
        };
        for events in [
            vec![genuine.clone()],
            vec![SceneEvent::Interaction {
                id: "generic-console".into(),
                event_id: Some(CORE_EVENT_ID.into()),
            }],
            vec![SceneEvent::Trigger {
                id: "forged-core-trigger".into(),
                event_id: CORE_EVENT_ID.into(),
            }],
            vec![SceneEvent::Interaction {
                id: CORE_CONSOLE_ID.into(),
                event_id: None,
            }],
            vec![
                SceneEvent::Interaction {
                    id: "benign-first".into(),
                    event_id: None,
                },
                genuine,
            ],
        ] {
            let before = state.world.revision;
            assert!(apply_scene_progress(&mut state, scene, &events, "core-forged").is_err());
            assert_eq!(state.world.revision, before);
            assert!(!completed_world_events(&state, "clockworks").contains(CORE_EVENT_ID));
        }
        apply_scene_progress(&mut state, scene, &[], "core-empty").unwrap();
        assert!(!completed_world_events(&state, "clockworks").contains(CORE_EVENT_ID));
    }

    #[test]
    fn regulator_defeat_survives_save_restore_revisit_and_console_completion() {
        let runtime = core_runtime();
        owner_attack(&runtime, 1, 24);
        {
            let scene_guard = runtime.scene_runtime.lock().unwrap();
            let state = runtime.state.lock().unwrap();
            let save = crate::save_v6::SaveV6::capture(&state, scene_guard.as_ref()).unwrap();
            let encoded = serde_json::to_vec(&save).unwrap();
            let decoded: crate::save_v6::SaveV6 = serde_json::from_slice(&encoded).unwrap();
            let mut restored = decoded.restore_state().unwrap();
            assert!(restored.world_persistent_v1.clockworks.regulator_defeated);
            assert!(!completed_world_events(&restored, "clockworks").contains(CORE_EVENT_ID));
            let definition = scene_guard.as_ref().unwrap().current_scene();
            // Includes legacy Continue fallback that reconstructs actors.
            restored.world.generic_actors = spawn_generic_actors(definition).unwrap();
            restore_regulator_progression(
                &mut restored.world,
                definition,
                &restored.world_persistent_v1.clockworks,
            )
            .unwrap();
            assert_eq!(restored.world.generic_actors[0].hp, 0);
        }
        {
            let mut state = runtime.state.lock().unwrap();
            state
                .world_persistent_v1
                .clockworks
                .forged_guard_elite_first_kill = true;
            state.world.player.position_m = Vec3::new(1.0, 0.0, 8.0).unwrap();
        }
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime
            .transition_scene(
                "cw_regulator_core_return_to_arena",
                "core-dead-return",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(23.0, 0.0, 8.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let revisit = runtime
            .transition_scene("cw_arena_to_regulator_core", "core-dead-revisit", epoch).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert_eq!(revisit.actors.len(), 1);
        assert!(!revisit.actors[0].active);
        assert_no_core_progress(&runtime);
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(12.0, 0.0, 7.0).unwrap();
        let complete = runtime
            .activate_scene_interaction(CORE_CONSOLE_ID, "core-after-revisit", revisit.world_epoch)
            .unwrap();
        assert!(complete.applied);
        let scene_guard = runtime.scene_runtime.lock().unwrap();
        let state = runtime.state.lock().unwrap();
        let save = crate::save_v6::SaveV6::capture(&state, scene_guard.as_ref()).unwrap();
        let restored = save.restore_state().unwrap();
        assert!(restored.world_persistent_v1.clockworks.regulator_defeated);
        assert!(completed_world_events(&restored, "clockworks").contains(CORE_EVENT_ID));
        drop(state);
        drop(scene_guard);
        // A completed-core revisit must preserve both dead Boss and event idempotency.
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(1.0, 0.0, 8.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime
            .transition_scene(
                "cw_regulator_core_return_to_arena",
                "core-completed-return",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(23.0, 0.0, 8.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let revisit = runtime
            .transition_scene(
                "cw_arena_to_regulator_core",
                "core-completed-revisit",
                epoch,
            ).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
            .unwrap();
        assert!(!revisit.actors[0].active);
        assert!(
            completed_world_events(&runtime.state.lock().unwrap(), "clockworks")
                .contains(CORE_EVENT_ID)
        );
        assert!(
            !revisit
                .interactables
                .iter()
                .find(|item| item.entity_id == CORE_CONSOLE_ID)
                .unwrap()
                .active
        );
        runtime.reset_new().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(
            !runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .clockworks
                .regulator_defeated
        );
    }

    #[test]
    fn public_continue_and_slot_preserve_phase3_defeat_and_completed_core() {
        let runtime = core_runtime();
        owner_attack(&runtime, 1, 17);
        {
            let scene_guard = runtime.scene_runtime.lock().unwrap();
            let mut state = runtime.state.lock().unwrap();
            assert!(state.world_persistent_v1.clockworks.regulator_phase3_active);
            state.world.server_time_ms += 900;
            state.world.player.position_m = Vec3::new(9.0, 0.0, 12.0).unwrap();
            advance_regulator_environment(&mut state, scene_guard.as_ref().unwrap()).unwrap();
        }
        let clocks = runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .clockworks
            .clone();
        assert!(!clocks.regulator_hazard_next_damage_at_ms.is_empty());
        let before = runtime.snapshot().unwrap();
        runtime.save().unwrap();
        let continued = runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(continued.world_epoch > before.world_epoch);
        assert_eq!(
            runtime.state.lock().unwrap().world_persistent_v1.clockworks,
            clocks
        );
        assert_eq!(
            serde_json::to_value(&continued.hazards).unwrap(),
            serde_json::to_value(&before.hazards).unwrap()
        );
        runtime
            .save_slot("core-phase3", "Core Phase 3", true)
            .unwrap();
        let continued = runtime.continue_slot("core-phase3").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert_eq!(
            runtime.state.lock().unwrap().world_persistent_v1.clockworks,
            clocks
        );
        assert!(continued
            .hazards
            .iter()
            .any(|h| h.kind == "moving_machinery" && h.active));
        runtime.state.lock().unwrap().paused = false;
        owner_attack(&runtime, 2, 7);
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .clockworks
                .regulator_defeated
        );
        runtime.save().unwrap();
        let dead = runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(!dead.actors[0].active);
        assert!(dead
            .hazards
            .iter()
            .filter(|h| h.phase_active.is_some())
            .all(|h| !h.active));
        assert!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .clockworks
                .regulator_defeated
        );
        runtime.state.lock().unwrap().paused = false;
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(12.0, 0.0, 7.0).unwrap();
        assert!(
            runtime
                .activate_scene_interaction(
                    CORE_CONSOLE_ID,
                    "core-public-continue-complete",
                    dead.world_epoch
                )
                .unwrap()
                .applied
        );
        runtime
            .save_slot("core-complete", "Core Complete", true)
            .unwrap();
        let completed = runtime.continue_slot("core-complete").map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
        assert!(!completed.actors[0].active);
        assert!(
            !completed
                .interactables
                .iter()
                .find(|i| i.entity_id == CORE_CONSOLE_ID)
                .unwrap()
                .active
        );
        assert!(
            completed_world_events(&runtime.state.lock().unwrap(), "clockworks")
                .contains(CORE_EVENT_ID)
        );
    }

    #[test]
    fn generic_runtime_progress_command_cannot_bypass_core_console() {
        let runtime = core_runtime();
        owner_attack(&runtime, 1, 24);
        assert_eq!(
            runtime
                .apply_route_command(RouteCommandRequest::Progress {
                    event_id: CORE_EVENT_ID.into(),
                    request_id: "generic-core-progress".into(),
                })
                .unwrap_err(),
            "E_CLOCKWORKS_CORE_SOURCE_INVALID"
        );
        assert_no_core_progress(&runtime);
    }

    #[test]
    fn old_core_event_requires_physical_confirmation_and_preserves_existing_ledgers() {
        let runtime = core_runtime();
        owner_attack(&runtime, 1, 24);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        assert!(runtime.activate_scene_interaction(CORE_CONSOLE_ID, "original-core", epoch).unwrap().applied);
        assert!(runtime.state.lock().unwrap().world_persistent_v1.clockworks.core_console_confirmed);
        runtime.state.lock().unwrap().world_persistent_v1.clockworks.core_console_confirmed = false;
        let route = runtime.state.lock().unwrap().route.clone();
        let grants = runtime.state.lock().unwrap().capabilities.grants.clone();
        runtime.save().unwrap();
        let continued = crate::formal_runtime::entry_test_support::acknowledge_ready(
            &runtime, runtime.continue_saved().unwrap());
        assert!(!runtime.state.lock().unwrap().world_persistent_v1.clockworks.core_console_confirmed);
        assert!(runtime.scene_runtime.lock().unwrap().as_ref().unwrap().object_activated(CORE_CONSOLE_ID));
        assert!(runtime.snapshot().unwrap().interactables.iter().find(|i| i.entity_id == CORE_CONSOLE_ID).unwrap().active);
        runtime.state.lock().unwrap().paused = true;
        assert!(!runtime.snapshot().unwrap().interactables.iter().find(|i| i.entity_id == CORE_CONSOLE_ID).unwrap().active);
        runtime.state.lock().unwrap().paused = false;
        let before = runtime.state.lock().unwrap().world.revision;
        let stale = runtime.activate_scene_interaction(CORE_CONSOLE_ID, "core-reconfirm", continued.world_epoch + 1).unwrap();
        assert!(!stale.applied);
        assert_eq!(runtime.state.lock().unwrap().world.revision, before);
        let response = runtime.activate_scene_interaction(CORE_CONSOLE_ID, "core-reconfirm", continued.world_epoch).unwrap();
        assert!(response.applied, "{:?}", response.error_code);
        assert!(runtime.state.lock().unwrap().world_persistent_v1.clockworks.core_console_confirmed);
        assert_eq!(runtime.state.lock().unwrap().route, route);
        assert_eq!(runtime.state.lock().unwrap().capabilities.grants, grants);
        assert!(!runtime.snapshot().unwrap().interactables.iter().find(|i| i.entity_id == CORE_CONSOLE_ID).unwrap().active);
        let revision = runtime.state.lock().unwrap().world.revision;
        assert!(!runtime.activate_scene_interaction(CORE_CONSOLE_ID, "core-reconfirm-duplicate", continued.world_epoch).unwrap().applied);
        assert_eq!(runtime.state.lock().unwrap().world.revision, revision);
    }

    #[test]
    fn old_clockworks_persistence_does_not_invent_defeat() {
        for value in [
            serde_json::json!({}),
            serde_json::json!({"regulatorPhase2Active": true}),
            serde_json::json!({"forgedGuardEliteFirstKill": true}),
        ] {
            let old: ClockworksPersistentState = serde_json::from_value(value).unwrap();
            assert!(!old.regulator_defeated);
        }
    }
}

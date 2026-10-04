//! Input-scheduled deterministic replay for trusted Rust tooling.
//!
//! This module is absent from default builds. It creates no IPC command and
//! reads no environment/save switch. Wall-clock gameplay continues to use the
//! ordinary constructor and its sole 60 Hz simulation thread.

use super::*;

impl FormalRuntime {
    /// Create a deterministic, input-only replay driver in an isolated save root.
    ///
    /// Each accepted `submit_input` advances the same authoritative owner step
    /// exactly once at the fixed engine timestep. Reads, interactions and disk
    /// I/O do not advance simulation time. The input is fresh at its scheduled
    /// tick; host delays cannot age it out. No snapshot field is normalized.
    ///
    /// This is not a real-time or native Tauri acceptance driver. Action-command
    /// scheduling is deliberately unsupported; this driver only replays input
    /// samples (including their existing combat-intent envelope).
    pub fn new_input_replay_with_save_dir(save_root: PathBuf) -> Result<Self, String> {
        Ok(Self {
            state: Arc::new(Mutex::new(Self::initial_state(1)?)),
            scene_runtime: Arc::new(Mutex::new(None)),
            scene_registry: Mutex::new(None),
            save_root,
            active_slot: Mutex::new(None),
            stop_owner: Arc::new(AtomicBool::new(false)),
            owner_handle: Mutex::new(None),
            input_replay: true,
        })
    }

    pub(super) fn advance_input_replay(
        &self,
        state: &mut RuntimeState,
    ) -> Result<WorldView, String> {
        if let Some(error) = &state.last_owner_error {
            return Err(format!("E_SIMULATION_OWNER: {error}"));
        }
        let backup = state.clone();
        let scene = self.scene_runtime.lock().map_err(|_| {
            state.last_owner_error = Some("E_SCENE_RUNTIME_LOCK_POISONED".into());
            "E_SIMULATION_OWNER: E_SCENE_RUNTIME_LOCK_POISONED"
        })?;
        if let Err(error) =
            advance_owner_step_with_input_age(state, scene.as_ref(), Some(Duration::ZERO))
        {
            *state = backup;
            state.last_owner_error = Some(error.clone());
            return Err(format!("E_SIMULATION_OWNER: {error}"));
        }
        Ok(project_scene_view(state, scene.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_dispatch_uses_the_exact_authoritative_owner_step() {
        let runtime = FormalRuntime::new_input_replay_with_save_dir(PathBuf::from(
            "unused-replay-parity-save-root",
        ))
        .unwrap();
        let sample = InputSample::new(1, 1, 50, 1.0, 0.0).unwrap();
        let mut expected = FormalRuntime::initial_state(1).unwrap();
        expected.latest_sample = sample.clone();
        expected.latest_input_received_at = Instant::now();
        advance_owner_step_with_scene(&mut expected, None).unwrap();
        let actual = runtime.submit_input(sample, vec![]).unwrap();
        assert_eq!(actual, project_view(&expected));
        let state = runtime.state.lock().unwrap();
        assert_eq!(
            serde_json::to_value(crate::save_v6::SaveV6::capture(&state, None).unwrap()).unwrap(),
            serde_json::to_value(crate::save_v6::SaveV6::capture(&expected, None).unwrap())
                .unwrap(),
        );
        assert_eq!(state.presentation_events, expected.presentation_events);
        assert_eq!(state.sound_cues, expected.sound_cues);
    }

    #[test]
    fn live_input_expiry_is_still_applied_by_the_shared_step() {
        let mut state = FormalRuntime::initial_state(1).unwrap();
        state.latest_sample = InputSample::new(1, 1, 50, 1.0, 0.0).unwrap();
        advance_owner_step_with_input_age(&mut state, None, Some(Duration::from_millis(251)))
            .unwrap();
        assert_eq!(state.world.player.position_m.x_m, 0.0);
        assert_eq!(state.world.revision.server_tick, 1);
        assert_eq!(state.last_received_seq, 1);
    }

    #[test]
    fn owner_step_failure_rolls_back_and_remains_sticky_after_admission() {
        let runtime = FormalRuntime::new_input_replay_with_save_dir(PathBuf::from(
            "unused-replay-rollback-save-root",
        ))
        .unwrap();
        let before = {
            let mut state = runtime.state.lock().unwrap();
            state.world.revision.authority_revision = u64::MAX;
            state.clone()
        };
        let before_save =
            serde_json::to_value(crate::save_v6::SaveV6::capture(&before, None).unwrap()).unwrap();
        let error = runtime
            .submit_input(
                InputSample::new(1, 1, 50, 1.0, 0.0).unwrap(),
                vec![CombatIntentRequest::Attack { request_id: 1 }],
            )
            .unwrap_err();
        assert!(error.starts_with("E_SIMULATION_OWNER:"));
        {
            let state = runtime.state.lock().unwrap();
            assert_eq!(project_view(&state), project_view(&before));
            assert_eq!(
                serde_json::to_value(crate::save_v6::SaveV6::capture(&state, None).unwrap())
                    .unwrap(),
                before_save
            );
            assert_eq!(state.presentation_events, before.presentation_events);
            assert_eq!(state.sound_cues, before.sound_cues);
            assert_eq!(state.last_received_seq, 0);
            assert_eq!(state.latest_sample.seq, 1); // accepted mailbox, not acknowledged
            assert_eq!(state.pending_combat.len(), 1); // restored with the admitted sample
            assert!(state.last_owner_error.is_some());
        }
        assert_eq!(runtime.snapshot().unwrap_err(), error);
        assert_eq!(
            runtime
                .submit_input(InputSample::new(1, 2, 100, 0.0, 0.0).unwrap(), vec![])
                .unwrap_err(),
            error
        );
        let after = runtime.state.lock().unwrap();
        assert_eq!(project_view(&after), project_view(&before));
        assert_eq!(after.last_received_seq, 0);
    }
}

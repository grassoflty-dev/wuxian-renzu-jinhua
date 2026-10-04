//! These tests run in the CLI crate, whose dependency enables deterministic replay.
//! They are therefore included by the existing Scenario Runner CI step.
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use wuxian_horror_ch1::formal_runtime::FormalRuntime;

#[path = "../../../server-rs/tests/formal_replay_driver.rs"]
mod live_constructor;

mod replay {
    use super::*;
    use wuxian_horror_ch1::{continuous_input::InputSample, formal_runtime::ActionCommandRequest};

    fn runtime() -> FormalRuntime {
        FormalRuntime::new_input_replay_with_save_dir(PathBuf::from(
            "unused-input-replay-save-root",
        ))
        .unwrap()
    }

    #[test]
    fn host_delays_do_not_advance_or_expire_scheduled_input() {
        let quick = runtime();
        let delayed = runtime();
        let initial = delayed.snapshot().unwrap();
        std::thread::sleep(Duration::from_millis(275));
        assert_eq!(initial, delayed.snapshot().unwrap());
        for seq in 1..=3 {
            let sample = InputSample::new(1, seq, seq * 50, 1.0, 0.0).unwrap();
            let a = quick.submit_input(sample.clone(), vec![]).unwrap();
            std::thread::sleep(Duration::from_millis(30));
            let b = delayed.submit_input(sample, vec![]).unwrap();
            assert_eq!(
                serde_json::to_vec(&a).unwrap(),
                serde_json::to_vec(&b).unwrap()
            );
            assert_eq!(b.server_tick, seq);
            assert_eq!(b.authority_revision, seq);
            assert_eq!(b.ack_seq, seq);
            assert_eq!(b.server_time_ms, seq * 17);
        }
        let before = delayed.snapshot().unwrap();
        std::thread::sleep(Duration::from_millis(275));
        assert_eq!(before, delayed.snapshot().unwrap());
    }

    #[test]
    fn invalid_and_paused_input_cannot_advance_replay() {
        let runtime = runtime();
        let sample = InputSample::new(1, 1, 50, 1.0, 0.0).unwrap();
        let accepted = runtime.submit_input(sample.clone(), vec![]).unwrap();
        assert_eq!(
            runtime.submit_input(sample, vec![]).unwrap_err(),
            "E_INPUT_STALE_SEQUENCE"
        );
        assert_eq!(runtime.snapshot().unwrap(), accepted);
        let stale = InputSample::new(9, 2, 100, 1.0, 0.0).unwrap();
        assert_eq!(
            runtime.submit_input(stale, vec![]).unwrap_err(),
            "E_INPUT_STALE_EPOCH"
        );
        assert_eq!(runtime.snapshot().unwrap(), accepted);
        let paused = runtime.pause().unwrap();
        assert_eq!(
            runtime
                .submit_input(InputSample::new(1, 2, 100, 1.0, 0.0).unwrap(), vec![])
                .unwrap_err(),
            "E_RUNTIME_PAUSED"
        );
        assert_eq!(runtime.snapshot().unwrap(), paused);
        let resumed = runtime.resume().unwrap();
        let next = runtime
            .submit_input(InputSample::new(1, 2, 100, 1.0, 0.0).unwrap(), vec![])
            .unwrap();
        assert_eq!(next.server_tick, resumed.server_tick + 1);
        assert_eq!(next.authority_revision, resumed.authority_revision + 1);
        assert_eq!(next.ack_seq, 2);
    }

    #[test]
    fn unsupported_action_command_fails_without_changing_replay_state() {
        let runtime = runtime();
        let before = runtime.snapshot().unwrap();
        let command: ActionCommandRequest = serde_json::from_value(serde_json::json!({
            "protocolVersion": 2, "worldEpoch": 1, "requestId": 1,
            "clientTimeMs": 0, "kind": "primaryAttack"
        }))
        .unwrap();
        assert_eq!(
            runtime.submit_action(command).unwrap_err(),
            "E_INPUT_REPLAY_ACTION_UNSUPPORTED"
        );
        assert_eq!(runtime.snapshot().unwrap(), before);
    }
}

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn child_process_scenario_replays_are_deterministic_and_leave_auditable_output() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_parent =
        std::env::temp_dir().join(format!("pivot-scenario-runner-acceptance-{nonce}"));
    fs::create_dir(&output_parent).unwrap();
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/grey-hive-power-gate-a-v1.json");
    let result = Command::new(env!("CARGO_BIN_EXE_scenario-runner"))
        .arg("verify")
        .arg("--fixture")
        .arg(fixture)
        .arg("--output-parent")
        .arg(&output_parent)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "acceptance replay failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["passed"], true);
    assert_eq!(report["runtimeDriver"], "deterministic-input-fixed-step");
    assert_eq!(report["deterministicReplaysEqual"], true);
    assert_eq!(report["processExitCodes"].as_array().unwrap().len(), 8);
    assert!(report["assertions"].as_array().unwrap().len() >= 15);
    assert_eq!(report["unsupported"].as_array().unwrap().len(), 1);
    let output = PathBuf::from(report["outputPath"].as_str().unwrap());
    let bytes = fs::read(output).unwrap();
    assert_eq!(report["evidenceRecords"], 14);
    assert!(report["processExitCodes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|code| code == 0));
    assert!(report["processIds"]
        .as_array()
        .unwrap()
        .iter()
        .all(|pid| pid.as_u64().unwrap() > 0));
    assert_eq!(report["outputSha256"], hex::encode(Sha256::digest(&bytes)));
    let run_directory = PathBuf::from(report["runDirectory"].as_str().unwrap());
    for replay in ["replay-1", "replay-2"] {
        let directory = run_directory.join(replay);
        assert_eq!(fs::read(directory.join("scenario.jsonl")).unwrap(), bytes);
        for phase in ["seed", "continue-a", "continue-b", "corrupt"] {
            let raw: Value = serde_json::from_slice(
                &fs::read(directory.join(format!("{phase}.stdout.json"))).unwrap(),
            )
            .unwrap();
            assert!(raw["errors"].as_array().unwrap().is_empty());
            assert_eq!(
                fs::read_to_string(directory.join(format!("{phase}.exit-code.txt"))).unwrap(),
                "0"
            );
            assert_eq!(
                fs::read(directory.join(format!("{phase}.stderr.txt"))).unwrap(),
                b""
            );
        }
    }
    assert!(bytes
        .windows(b"power-event-first".len())
        .any(|window| window == b"power-event-first"));
    assert!(bytes
        .windows(b"continue-slot".len())
        .any(|window| window == b"continue-slot"));
    // Intentionally leave the temporary acceptance output in place for review.
}

#[test]
fn failed_phase_keeps_raw_evidence_before_assertion_rejection() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = std::env::temp_dir().join(format!("scenario-failed-evidence-{nonce}"));
    fs::create_dir(&parent).unwrap();
    let original =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/grey-hive-power-gate-a-v1.json");
    let mut fixture: Value = serde_json::from_slice(&fs::read(original).unwrap()).unwrap();
    fixture["expected"]["closedGateMaxZ"] = serde_json::json!(0);
    let fixture_path = parent.join("deliberately-invalid-expectation.json");
    fs::write(&fixture_path, serde_json::to_vec(&fixture).unwrap()).unwrap();
    let output_parent = parent.join("runs");
    let result = Command::new(env!("CARGO_BIN_EXE_scenario-runner"))
        .args(["verify", "--fixture"])
        .arg(fixture_path)
        .arg("--output-parent")
        .arg(&output_parent)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("closed-gate-collision-blocks-passage")
    );
    let run = fs::read_dir(output_parent)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let raw: Value =
        serde_json::from_slice(&fs::read(run.join("replay-1/seed.stdout.json")).unwrap()).unwrap();
    assert!(!raw["errors"].as_array().unwrap().is_empty());
    assert_eq!(raw["records"].as_array().unwrap().len(), 9);
    assert!(run.join("replay-1/seed.stderr.txt").is_file());
    assert_eq!(
        fs::read_to_string(run.join("replay-1/seed.exit-code.txt")).unwrap(),
        "0"
    );
    assert!(!run.join("verification.json").exists());
}

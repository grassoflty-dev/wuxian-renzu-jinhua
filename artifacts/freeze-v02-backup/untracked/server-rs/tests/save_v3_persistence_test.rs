use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use wuxian_horror_ch1::save_v3_migration::migrate_v2_value;
use wuxian_horror_ch1::save_v3_persistence::{
    commit_v2_to_v3, commit_v2_to_v3_with_fault, inspect_transaction, PersistenceFault,
    SaveV3PersistenceError,
};

fn unique_root(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("tp03b-save-{label}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

fn fixture(id: &str) -> Value {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "fixtures",
        "continuous-v2",
        &format!("{id}.json"),
    ]
    .iter()
    .collect();
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn no_choice(v2: &mut Value) {
    v2["optional_choice"] = Value::Null;
    v2["decisions"] = json!({});
    v2["flags"] = json!(["hive_power", "hive_lockdown"]);
    for objective in v2["objectives"].as_array_mut().unwrap() {
        if objective["id"] == "hive_choice" {
            objective["done"] = json!(false);
        }
    }
}

fn result_no_choice() -> Value {
    let mut v2 = fixture("completed_grey_hive");
    no_choice(&mut v2);
    v2["flags"] = json!(["hive_power", "hive_lockdown", "hive_rescue"]);
    v2
}

struct Sandbox {
    root: PathBuf,
    source: PathBuf,
    target: PathBuf,
}

impl Sandbox {
    fn new(label: &str, v2: &Value) -> Self {
        let root = unique_root(label);
        let source = root.join("campaign-v2.json");
        let target = root.join("campaign-v3.json");
        fs::write(&source, serde_json::to_vec(v2).unwrap()).unwrap();
        Self {
            root,
            source,
            target,
        }
    }

    fn cleanup(self) {
        fs::remove_dir_all(self.root).unwrap();
    }
}

fn code(error: SaveV3PersistenceError) -> String {
    error.code()
}

#[test]
fn six_fixtures_commit_with_hashes_and_idempotent_target() {
    let expected = [
        ("grey_hive_initial", "locked"),
        ("post_power", "locked"),
        ("post_gate_a", "locked"),
        ("post_baizhi", "open"),
        ("pre_sentinel", "open"),
        ("completed_grey_hive", "open"),
    ];
    for (id, gate_b) in expected {
        let sandbox = Sandbox::new(id, &fixture(id));
        let report = commit_v2_to_v3(&sandbox.source, &sandbox.target, &sandbox.root).unwrap();
        assert_eq!(report.status, "committed", "{id}");
        assert_eq!(report.gate_b, gate_b, "{id}");
        assert!(report.source_unchanged, "{id}");
        assert!(report.transform_none, "{id}");
        assert_eq!(report.source_sha256, report.backup_sha256, "{id}");
        assert!(report.target_exists && !report.pending_exists, "{id}");
        let evidence = inspect_transaction(&sandbox.source, &sandbox.target);
        assert_eq!(evidence.source.sha256, evidence.backup.sha256, "{id}");
        assert!(evidence.target.exists && !evidence.pending.exists, "{id}");

        let target_hash = evidence.target.sha256.clone().unwrap();
        let second = commit_v2_to_v3(&sandbox.source, &sandbox.target, &sandbox.root).unwrap();
        assert_eq!(second.status, "already_present", "{id}");
        assert_eq!(second.target_sha256, target_hash, "{id}");
        assert_eq!(
            inspect_transaction(&sandbox.source, &sandbox.target)
                .target
                .sha256,
            Some(target_hash),
            "{id}"
        );
        sandbox.cleanup();
    }
}

#[test]
fn optional_choice_boundaries_match_ag20_without_real_data() {
    let mut high = fixture("post_baizhi");
    no_choice(&mut high);
    let high_sandbox = Sandbox::new("high-unresolved", &high);
    let high_report = commit_v2_to_v3(
        &high_sandbox.source,
        &high_sandbox.target,
        &high_sandbox.root,
    )
    .unwrap();
    assert_eq!(high_report.gate_b, "open");
    assert_eq!(high_report.choice_state, "unresolved");
    let high_target: Value =
        serde_json::from_slice(&fs::read(&high_sandbox.target).unwrap()).unwrap();
    assert_eq!(high_target["worldResume"]["checkpointId"], "gh_cp_post_bio");
    high_sandbox.cleanup();

    let mut deep = fixture("pre_sentinel");
    no_choice(&mut deep);
    let deep_sandbox = Sandbox::new("deep-unresolved", &deep);
    let deep_report = commit_v2_to_v3(
        &deep_sandbox.source,
        &deep_sandbox.target,
        &deep_sandbox.root,
    )
    .unwrap();
    assert_eq!(deep_report.gate_b, "open");
    let deep_target: Value =
        serde_json::from_slice(&fs::read(&deep_sandbox.target).unwrap()).unwrap();
    assert_eq!(
        deep_target["worldResume"]["checkpointId"],
        "gh_cp_pre_sentinel"
    );
    deep_sandbox.cleanup();

    let result_sandbox = Sandbox::new("result-unresolved", &result_no_choice());
    let result_report = commit_v2_to_v3(
        &result_sandbox.source,
        &result_sandbox.target,
        &result_sandbox.root,
    )
    .unwrap();
    assert_eq!(result_report.choice_state, "unresolved");
    let result_target: Value =
        serde_json::from_slice(&fs::read(&result_sandbox.target).unwrap()).unwrap();
    assert_eq!(result_target["worldResume"]["checkpointId"], "gh_cp_exit");
    assert_eq!(
        result_target["worldResume"]["resumePolicy"],
        "result_then_hub"
    );
    result_sandbox.cleanup();

    let locked_sandbox = Sandbox::new("locked", &fixture("post_gate_a"));
    let locked_report = commit_v2_to_v3(
        &locked_sandbox.source,
        &locked_sandbox.target,
        &locked_sandbox.root,
    )
    .unwrap();
    assert_eq!(locked_report.gate_b, "locked");
    let locked_target: Value =
        serde_json::from_slice(&fs::read(&locked_sandbox.target).unwrap()).unwrap();
    assert_ne!(
        locked_target["worldResume"]["checkpointId"],
        "gh_cp_post_bio"
    );
    locked_sandbox.cleanup();
}

#[test]
fn source_parse_and_read_fail_before_any_write() {
    let root = unique_root("source-failures");
    let invalid_source = root.join("invalid-v2.json");
    let invalid_target = root.join("invalid-v3.json");
    fs::write(&invalid_source, b"{bad").unwrap();
    assert_eq!(
        code(commit_v2_to_v3(&invalid_source, &invalid_target, &root).unwrap_err()),
        "E_JSON_INVALID"
    );
    let invalid_evidence = inspect_transaction(&invalid_source, &invalid_target);
    assert!(invalid_evidence.source.exists);
    assert!(
        !invalid_evidence.backup.exists
            && !invalid_evidence.pending.exists
            && !invalid_evidence.target.exists
    );

    let source_directory = root.join("source-directory");
    fs::create_dir(&source_directory).unwrap();
    let directory_target = root.join("directory-v3.json");
    assert_eq!(
        code(commit_v2_to_v3(&source_directory, &directory_target, &root).unwrap_err()),
        "E_SOURCE_NOT_REGULAR"
    );
    assert!(
        !inspect_transaction(&source_directory, &directory_target)
            .target
            .exists
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn backup_pending_reload_and_commit_faults_preserve_recovery_evidence() {
    let faults = [
        (
            PersistenceFault::SourceRead,
            "E_SOURCE_READ",
            false,
            false,
            false,
        ),
        (
            PersistenceFault::BackupWrite,
            "E_BACKUP_WRITE",
            false,
            false,
            false,
        ),
        (
            PersistenceFault::PendingWrite,
            "E_PENDING_WRITE",
            true,
            false,
            false,
        ),
        (
            PersistenceFault::PendingReload,
            "E_PENDING_RELOAD",
            true,
            true,
            false,
        ),
        (
            PersistenceFault::BeforeRename,
            "E_COMMIT_INTERRUPTED",
            true,
            true,
            false,
        ),
    ];
    for (fault, expected, backup, pending, target) in faults {
        let sandbox = Sandbox::new(&format!("fault-{expected}"), &fixture("post_baizhi"));
        assert_eq!(
            code(
                commit_v2_to_v3_with_fault(&sandbox.source, &sandbox.target, &sandbox.root, fault)
                    .unwrap_err()
            ),
            expected
        );
        let evidence = inspect_transaction(&sandbox.source, &sandbox.target);
        assert_eq!(evidence.source.exists, true);
        assert_eq!(evidence.backup.exists, backup, "{expected}");
        assert_eq!(evidence.pending.exists, pending, "{expected}");
        assert_eq!(evidence.target.exists, target, "{expected}");
        if backup {
            assert_eq!(evidence.source.sha256, evidence.backup.sha256, "{expected}");
        }
        sandbox.cleanup();
    }
}

#[test]
fn damaged_or_unknown_existing_states_are_never_overwritten() {
    let bad_backup = Sandbox::new("bad-backup", &fixture("post_baizhi"));
    fs::write(
        bad_backup.source.with_file_name("campaign-v2.json.bak"),
        b"wrong",
    )
    .unwrap();
    assert_eq!(
        code(
            commit_v2_to_v3(&bad_backup.source, &bad_backup.target, &bad_backup.root).unwrap_err()
        ),
        "E_BACKUP_SOURCE_MISMATCH"
    );
    assert!(
        !inspect_transaction(&bad_backup.source, &bad_backup.target)
            .target
            .exists
    );
    bad_backup.cleanup();

    let pending = Sandbox::new("pending-existing", &fixture("post_baizhi"));
    fs::write(
        pending.target.with_file_name("campaign-v3.json.pending"),
        b"stale",
    )
    .unwrap();
    assert_eq!(
        code(commit_v2_to_v3(&pending.source, &pending.target, &pending.root).unwrap_err()),
        "E_PENDING_EXISTS"
    );
    assert_eq!(
        fs::read(pending.target.with_file_name("campaign-v3.json.pending")).unwrap(),
        b"stale"
    );
    pending.cleanup();

    let bad_target = Sandbox::new("bad-target", &fixture("post_baizhi"));
    fs::write(&bad_target.target, b"corrupt").unwrap();
    assert_eq!(
        code(
            commit_v2_to_v3(&bad_target.source, &bad_target.target, &bad_target.root).unwrap_err()
        ),
        "E_TARGET_INVALID"
    );
    assert_eq!(fs::read(&bad_target.target).unwrap(), b"corrupt");
    assert!(
        !inspect_transaction(&bad_target.source, &bad_target.target)
            .backup
            .exists
    );
    bad_target.cleanup();
}

#[test]
fn target_source_mismatch_and_path_escape_are_rejected() {
    let source_mismatch = Sandbox::new("target-source-mismatch", &fixture("post_baizhi"));
    let raw = fs::read(&source_mismatch.source).unwrap();
    let mut target_envelope =
        migrate_v2_value(&fixture("post_baizhi"), "campaign-v3", Some("other-sha")).unwrap();
    target_envelope.source.source_sha256 = Some("other-sha".into());
    fs::write(
        &source_mismatch.target,
        serde_json::to_vec(&target_envelope).unwrap(),
    )
    .unwrap();
    fs::write(
        source_mismatch
            .source
            .with_file_name("campaign-v2.json.bak"),
        &raw,
    )
    .unwrap();
    assert_eq!(
        code(
            commit_v2_to_v3(
                &source_mismatch.source,
                &source_mismatch.target,
                &source_mismatch.root
            )
            .unwrap_err()
        ),
        "E_TARGET_SOURCE_MISMATCH"
    );
    source_mismatch.cleanup();

    let nested = unique_root("different-directory");
    let source = nested.join("campaign-v2.json");
    let target_dir = nested.join("nested");
    fs::create_dir(&target_dir).unwrap();
    fs::write(
        &source,
        serde_json::to_vec(&fixture("post_baizhi")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        code(commit_v2_to_v3(&source, &target_dir.join("campaign-v3.json"), &nested).unwrap_err()),
        "E_PATH_DIFFERENT_DIRECTORY"
    );
    fs::remove_dir_all(nested).unwrap();

    let escape_root = unique_root("path-escape");
    let escape_source = escape_root.join("campaign-v2.json");
    let outside = escape_root
        .parent()
        .unwrap()
        .join(format!("tp03b-outside-{}", std::process::id()));
    fs::write(
        &escape_source,
        serde_json::to_vec(&fixture("post_baizhi")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        code(
            commit_v2_to_v3(
                &escape_source,
                &outside.join("campaign-v3.json"),
                &escape_root
            )
            .unwrap_err()
        ),
        "E_PATH_OUTSIDE_ROOT"
    );
    fs::remove_dir_all(escape_root).unwrap();
}

#[test]
fn same_path_and_serde_target_are_deterministic() {
    let sandbox = Sandbox::new("same-path", &fixture("post_baizhi"));
    assert_eq!(
        code(commit_v2_to_v3(&sandbox.source, &sandbox.source, &sandbox.root).unwrap_err()),
        "E_PATH_SAME"
    );
    sandbox.cleanup();
}

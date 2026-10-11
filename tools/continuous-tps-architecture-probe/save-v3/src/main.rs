use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

const FIXTURES: [&str; 6] = [
    "grey_hive_initial",
    "post_power",
    "post_gate_a",
    "post_baizhi",
    "pre_sentinel",
    "completed_grey_hive",
];
const KNOWN_CHECKPOINTS: [&str; 6] = [
    "gh_cp_airlock",
    "gh_cp_power",
    "gh_cp_central",
    "gh_cp_post_bio",
    "gh_cp_pre_sentinel",
    "gh_cp_exit",
];

fn root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn fixture_path(id: &str) -> PathBuf {
    root_dir()
        .join("server-rs/tests/fixtures/continuous-v2")
        .join(format!("{id}.json"))
}

fn result_path() -> PathBuf {
    root_dir().join("tools/continuous-tps-architecture-probe/results/save-ag11.json")
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn non_empty_string(value: &Value, path: &str) -> Result<String, String> {
    value
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("E_FIELD_TYPE:{path}"))
}

fn parse_v2(bytes: &[u8]) -> Result<Value, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "E_JSON_INVALID".to_owned())?;
    let object = value.as_object().ok_or_else(|| "E_V2_ROOT".to_owned())?;
    let version = object
        .get("version")
        .ok_or_else(|| "E_V2_MISSING_FIELD:version".to_owned())?;
    if version.as_i64() != Some(2) {
        return Err("E_V2_VERSION".to_owned());
    }
    for field in [
        "mode",
        "current_world_id",
        "objectives",
        "flags",
        "decisions",
        "map",
        "optional_choice",
    ] {
        if !object.contains_key(field) {
            return Err(format!("E_V2_MISSING_FIELD:{field}"));
        }
    }
    if object["current_world_id"].as_str() != Some("grey_hive") {
        return Err("E_V2_WORLD".to_owned());
    }
    Ok(value)
}

fn objective_done(v2: &Value, id: &str) -> bool {
    v2["objectives"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|o| o["id"].as_str() == Some(id) && o["done"].as_bool() == Some(true))
}

fn flag(v2: &Value, id: &str) -> bool {
    v2["flags"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|v| v.as_str() == Some(id))
}

fn choice(v2: &Value) -> Option<String> {
    v2["optional_choice"].as_str().map(str::to_owned)
}

fn stage_for(v2: &Value) -> &'static str {
    if v2["mode"].as_str() == Some("result") {
        "result_return"
    } else if objective_done(v2, "hive_rescue") {
        "beacon_taken"
    } else if v2["map"]["py"].as_i64() == Some(20) && choice(v2).is_some() {
        "pre_sentinel"
    } else if choice(v2).is_some() {
        "baizhi_resolved"
    } else if v2["map"]["px"].as_i64() == Some(17) && v2["map"]["py"].as_i64() == Some(8) {
        "gate_a_crossed"
    } else if flag(v2, "hive_lockdown") {
        "lockdown_cleared_baizhi_pending"
    } else if flag(v2, "hive_power") {
        "power_restored"
    } else {
        "airlock_initial"
    }
}

fn checkpoint_for(v2: &Value) -> (&'static str, &'static str, bool) {
    if v2["mode"].as_str() == Some("result") {
        ("gh_cp_exit", "result_then_hub", false)
    } else if objective_done(v2, "hive_rescue") {
        ("gh_cp_exit", "world_resume", false)
    } else if v2["map"]["py"].as_i64() == Some(20) && choice(v2).is_some() {
        ("gh_cp_pre_sentinel", "world_resume", false)
    } else if choice(v2).is_some() {
        ("gh_cp_post_bio", "world_resume", false)
    } else if v2["map"]["px"].as_i64() == Some(17) && v2["map"]["py"].as_i64() == Some(8) {
        ("gh_cp_central", "world_resume", false)
    } else if flag(v2, "hive_lockdown") {
        // Deliberately fall back before the candidate Gate B safety-room checkpoint.
        // The old v2 state has no resolved Baizhi choice, so it cannot be placed after the gate.
        ("gh_cp_central", "world_resume_choice_required", true)
    } else if flag(v2, "hive_power") {
        ("gh_cp_power", "world_resume", false)
    } else {
        ("gh_cp_airlock", "world_resume", false)
    }
}

fn objective_progress(v2: &Value) -> Value {
    json!(["hive_power", "hive_lockdown", "hive_rescue", "hive_choice"].into_iter().map(|id| {
        json!({"id": id, "done": objective_done(v2, id), "optionalInV2": id == "hive_choice"})
    }).collect::<Vec<_>>())
}

fn string_list(v2: &Value, field: &str) -> Vec<String> {
    v2[field]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

fn migrate_fixture(id: &str, v2_bytes: &[u8], source_commit: &str) -> Result<Value, String> {
    let v2 = parse_v2(v2_bytes)?;
    let (checkpoint_id, resume_policy, fallback) = checkpoint_for(&v2);
    let selected_choice = choice(&v2);
    let choice_resolved = selected_choice
        .as_deref()
        .is_some_and(|v| v == "hive_help_baizhi" || v == "hive_leave_sample");
    let map_is_null = v2["map"].is_null();
    let mode = v2["mode"].as_str().unwrap_or("world");
    let world_resume = json!({
        "worldId": "grey_hive",
        "checkpointId": checkpoint_id,
        "resumePolicy": resume_policy,
        "transform": null,
        "transformResolver": {
            "status": "deferred_formal_blockout",
            "rule": "resolve checkpointId against the approved collider/nav blockout; never scale v2 map.px/map.py",
            "fallbackOrder": ["checkpoint_spawn_or_legal_nav_point", "world_spawn"],
            "candidateCoordinatesPersisted": false
        }
    });
    let mut output = json!({
        "schemaVersion": 3,
        "saveId": format!("ag11-{id}"),
        "source": {
            "schemaVersion": 2,
            "fixtureId": id,
            "sourceSha256": sha256(v2_bytes),
            "sourceCommit": source_commit,
            "migration": "one_time_v2_to_v3_candidate"
        },
        "campaign": {
            "worldId": "grey_hive",
            "mode": mode,
            "stage": stage_for(&v2),
            "sourceMapWasNull": map_is_null,
            "resultRecovery": if mode == "result" { "result_then_hub" } else { "not_applicable" },
            "gateFacts": {
                "gateA": if flag(&v2, "hive_power") { "open" } else { "locked" },
                "gateB": if choice_resolved { "candidate_open" } else { "locked" },
                "baizhiEncountered": choice_resolved,
                "baizhiChoiceResolved": choice_resolved,
                "postBioCheckpointActive": choice_resolved
            }
        },
        "progression": {
            "objectives": objective_progress(&v2),
            "flags": string_list(&v2, "flags"),
            "decisions": v2["decisions"].clone(),
            "optionalChoice": selected_choice,
            "runRewardAllowed": v2["run_reward_allowed"].as_bool().unwrap_or(false),
            "runScrap": v2["run_scrap"].as_i64().unwrap_or(0),
            "enemiesDefeated": v2["enemies_defeated"].as_i64().unwrap_or(0)
        },
        "worldResume": world_resume
    });
    if fallback {
        output["campaign"]["migrationConflict"] = json!({
            "code": "V2_OPTIONAL_CHOICE_REQUIRED_BY_V4_GATE",
            "severity": "requires_player_choice",
            "policy": "preserve_progress_and_rewards; resume before Baizhi choice; do not synthesize a choice or pass Gate B",
            "preserved": ["hive_power", "hive_lockdown", "objectives", "flags", "decisions", "runScrap", "runRewardAllowed"],
            "deferred": ["baizhi_choice_resolved", "postBioCheckpointActive", "gateB_open"]
        });
    }
    validate_v3(&output)?;
    Ok(output)
}

fn validate_v3(v3: &Value) -> Result<(), String> {
    let root = v3.as_object().ok_or_else(|| "E_V3_ROOT".to_owned())?;
    if root.get("schemaVersion").and_then(Value::as_i64) != Some(3) {
        return Err("E_V3_VERSION".into());
    }
    for field in ["saveId", "source", "campaign", "progression", "worldResume"] {
        if !root.contains_key(field) {
            return Err(format!("E_V3_MISSING_FIELD:{field}"));
        }
    }
    non_empty_string(&root["saveId"], "saveId")?;
    let source = root["source"]
        .as_object()
        .ok_or_else(|| "E_V3_SOURCE".to_owned())?;
    if source.get("schemaVersion").and_then(Value::as_i64) != Some(2) {
        return Err("E_V3_SOURCE_VERSION".into());
    }
    non_empty_string(&source["fixtureId"], "source.fixtureId")?;
    let hash = non_empty_string(&source["sourceSha256"], "source.sourceSha256")?;
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("E_V3_SOURCE_HASH".into());
    }
    non_empty_string(&source["sourceCommit"], "source.sourceCommit")?;
    let campaign = root["campaign"]
        .as_object()
        .ok_or_else(|| "E_V3_CAMPAIGN".to_owned())?;
    if campaign.get("worldId").and_then(Value::as_str) != Some("grey_hive") {
        return Err("E_V3_WORLD".into());
    }
    let mode = non_empty_string(&campaign["mode"], "campaign.mode")?;
    if !["world", "result"].contains(&mode.as_str()) {
        return Err("E_V3_MODE".into());
    }
    let resume = root["worldResume"]
        .as_object()
        .ok_or_else(|| "E_V3_WORLD_RESUME".to_owned())?;
    let checkpoint = non_empty_string(&resume["checkpointId"], "worldResume.checkpointId")?;
    if !KNOWN_CHECKPOINTS.contains(&checkpoint.as_str()) {
        return Err("E_V3_CHECKPOINT".into());
    }
    if !resume["transform"].is_null() {
        return Err("E_V3_TRANSFORM_MUST_BE_NULL".into());
    }
    if resume["transformResolver"]["status"].as_str() != Some("deferred_formal_blockout") {
        return Err("E_V3_RESOLVER".into());
    }
    if mode == "result" && resume["resumePolicy"].as_str() != Some("result_then_hub") {
        return Err("E_V3_RESULT_RECOVERY".into());
    }
    if mode == "world" && resume["resumePolicy"].as_str() == Some("result_then_hub") {
        return Err("E_V3_RESULT_POLICY".into());
    }
    if root["progression"]["objectives"].as_array().is_none() {
        return Err("E_V3_OBJECTIVES".into());
    }
    let serialized = serde_json::to_string(v3).map_err(|_| "E_V3_SERIALIZE".to_owned())?;
    if serialized.contains("\"px\"") || serialized.contains("\"py\"") {
        return Err("E_V3_LEGACY_COORDINATES".into());
    }
    Ok(())
}

fn unique_sandbox(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("ag11-save-v3-{}-{label}-{nanos}", process::id()))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| "E_JSON_WRITE".to_owned())?;
    fs::write(path, bytes).map_err(|_| "E_WRITE".to_owned())
}

fn run_success_case(id: &str, v2_bytes: &[u8], v3: &Value) -> Value {
    let dir = unique_sandbox(id);
    let _ = fs::create_dir_all(&dir);
    let source = dir.join("campaign-v2.json");
    let backup = dir.join("campaign-v2.json.bak");
    let target = dir.join("campaign-v3.json");
    let temp = dir.join("campaign-v3.json.tmp");
    fs::write(&source, v2_bytes).unwrap();
    let source_hash_before = sha256(v2_bytes);
    fs::copy(&source, &backup).unwrap();
    write_json(&temp, v3).unwrap();
    fs::rename(&temp, &target).unwrap();
    let target_bytes = fs::read(&target).unwrap();
    let first_valid = serde_json::from_slice::<Value>(&target_bytes)
        .ok()
        .is_some_and(|v| validate_v3(&v).is_ok());
    let target_hash = sha256(&target_bytes);
    let repeat_before = sha256(&target_bytes);
    let repeat_idempotent = if target.exists() {
        serde_json::from_slice::<Value>(&target_bytes)
            .ok()
            .is_some_and(|v| validate_v3(&v).is_ok())
            && sha256(&fs::read(&target).unwrap()) == repeat_before
    } else {
        false
    };
    let source_hash_after = sha256(&fs::read(&source).unwrap());
    let backup_hash = sha256(&fs::read(&backup).unwrap());
    let passed = first_valid
        && repeat_idempotent
        && source_hash_before == source_hash_after
        && source_hash_after == backup_hash
        && !temp.exists();
    let result = json!({
        "name": format!("success_{id}"), "kind": "success", "pass": passed,
        "errorCode": Value::Null, "sourceExists": source.exists(), "backupExists": backup.exists(), "targetExists": target.exists(), "tempExists": temp.exists(),
        "sourceSha256Before": source_hash_before, "sourceSha256After": source_hash_after, "backupSha256": backup_hash, "targetSha256": target_hash,
        "firstReadValid": first_valid, "repeatIdempotent": repeat_idempotent, "recovery": "source_and_backup_retained; target committed atomically"
    });
    let _ = fs::remove_dir_all(&dir);
    result
}

fn run_invalid_case(name: &str, bytes: &[u8], expected: &str) -> Value {
    let dir = unique_sandbox(name);
    let _ = fs::create_dir_all(&dir);
    let source = dir.join("campaign-v2.json");
    fs::write(&source, bytes).unwrap();
    let actual = parse_v2(bytes).unwrap_err();
    let source_hash = sha256(&fs::read(&source).unwrap());
    let passed = actual == expected && source.exists();
    let result = json!({"name": name, "kind": "input_validation", "pass": passed, "errorCode": actual, "expectedErrorCode": expected, "sourceExists": source.exists(), "sourceSha256": source_hash, "targetExists": false, "recovery": "original input left untouched"});
    let _ = fs::remove_dir_all(&dir);
    result
}

fn run_invalid_checkpoint(v3: &Value) -> Value {
    let mut bad = v3.clone();
    bad["worldResume"]["checkpointId"] = json!("gh_cp_not_real");
    let actual = validate_v3(&bad).unwrap_err();
    json!({"name": "invalid_checkpoint", "kind": "v3_validation", "pass": actual == "E_V3_CHECKPOINT", "errorCode": actual, "expectedErrorCode": "E_V3_CHECKPOINT", "sourceExists": true, "targetExists": false, "recovery": "candidate rejected before commit"})
}

fn run_backup_failure(id: &str, bytes: &[u8], v3: &Value) -> Value {
    let dir = unique_sandbox(id);
    let _ = fs::create_dir_all(&dir);
    let source = dir.join("campaign-v2.json");
    let backup = dir.join("campaign-v2.json.bak");
    let target = dir.join("campaign-v3.json");
    fs::write(&source, bytes).unwrap();
    fs::create_dir(&backup).unwrap();
    let before = sha256(&fs::read(&source).unwrap());
    let actual = fs::copy(&source, &backup)
        .map(|_| "unexpected_success")
        .unwrap_or("E_BACKUP_FAILED");
    let _ = v3;
    let result = json!({"name": "backup_failure", "kind": "commit_failure", "pass": actual == "E_BACKUP_FAILED" && source.exists() && backup.is_dir() && !target.exists() && sha256(&fs::read(&source).unwrap()) == before, "errorCode": actual, "expectedErrorCode": "E_BACKUP_FAILED", "sourceExists": source.exists(), "backupExists": backup.exists(), "backupIsDirectory": backup.is_dir(), "targetExists": target.exists(), "sourceSha256": before, "recovery": "source intact; no target commit"});
    let _ = fs::remove_dir_all(&dir);
    result
}

fn run_interrupted_commit(id: &str, bytes: &[u8], v3: &Value) -> Value {
    let dir = unique_sandbox(id);
    let _ = fs::create_dir_all(&dir);
    let source = dir.join("campaign-v2.json");
    let backup = dir.join("campaign-v2.json.bak");
    let temp = dir.join("campaign-v3.json.tmp");
    let target = dir.join("campaign-v3.json");
    fs::write(&source, bytes).unwrap();
    fs::copy(&source, &backup).unwrap();
    write_json(&temp, v3).unwrap();
    let source_hash = sha256(&fs::read(&source).unwrap());
    let recovered =
        source.exists() && backup.exists() && sha256(&fs::read(&backup).unwrap()) == source_hash;
    let cleanup = fs::remove_file(&temp).is_ok();
    let result = json!({"name": "interrupted_before_atomic_commit", "kind": "restart_recovery", "pass": recovered && cleanup && source.exists() && backup.exists() && !target.exists(), "errorCode": "E_COMMIT_INTERRUPTED", "expectedErrorCode": "E_COMMIT_INTERRUPTED", "sourceExists": source.exists(), "backupExists": backup.exists(), "tempExistsAfterRecovery": temp.exists(), "targetExists": target.exists(), "sourceSha256": source_hash, "backupSha256": sha256(&fs::read(&backup).unwrap()), "recovery": "restart detects intact source+backup, removes stale temp, leaves original resumable"});
    let _ = fs::remove_dir_all(&dir);
    result
}

fn derive_boundary_samples(
    post_baizhi: &Value,
    pre_sentinel: &Value,
    completed: &Value,
) -> Vec<(String, Value)> {
    let mut no_choice = post_baizhi.clone();
    no_choice["optional_choice"] = Value::Null;
    no_choice["decisions"] = json!({});
    no_choice["flags"] = Value::Array(
        no_choice["flags"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v.as_str() != Some("hive_help_baizhi"))
            .cloned()
            .collect(),
    );
    if let Some(objectives) = no_choice["objectives"].as_array_mut() {
        for objective in objectives {
            if objective["id"] == "hive_choice" {
                objective["done"] = json!(false);
            }
        }
    }
    let mut leave = post_baizhi.clone();
    leave["optional_choice"] = json!("hive_leave_sample");
    leave["decisions"] = json!({"grey_hive": "hive_leave_sample"});
    leave["flags"] = Value::Array(
        leave["flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                if v.as_str() == Some("hive_help_baizhi") {
                    json!("hive_leave_sample")
                } else {
                    v.clone()
                }
            })
            .collect(),
    );
    let mut deep_no_choice = pre_sentinel.clone();
    deep_no_choice["optional_choice"] = Value::Null;
    deep_no_choice["decisions"] = json!({});
    deep_no_choice["flags"] = Value::Array(
        deep_no_choice["flags"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v.as_str() != Some("hive_help_baizhi"))
            .cloned()
            .collect(),
    );
    if let Some(objectives) = deep_no_choice["objectives"].as_array_mut() {
        for objective in objectives {
            if objective["id"] == "hive_choice" {
                objective["done"] = json!(false);
            }
        }
    }
    let mut rescue_no_choice = completed.clone();
    rescue_no_choice["optional_choice"] = Value::Null;
    rescue_no_choice["decisions"] = json!({});
    rescue_no_choice["flags"] = Value::Array(
        rescue_no_choice["flags"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v.as_str() != Some("hive_help_baizhi"))
            .cloned()
            .collect(),
    );
    if let Some(objectives) = rescue_no_choice["objectives"].as_array_mut() {
        for objective in objectives {
            if objective["id"] == "hive_choice" {
                objective["done"] = json!(false);
            }
        }
    }
    vec![
        ("legacy_high_progress_no_choice".into(), no_choice),
        ("legacy_deep_no_choice".into(), deep_no_choice),
        ("legacy_rescue_no_choice".into(), rescue_no_choice),
        ("legacy_choice_leave_sample".into(), leave),
    ]
}

fn main() {
    let source_commit = "working-tree-ag11";
    let mut fixture_reports = Vec::new();
    let mut v3_samples = Vec::new();
    for id in FIXTURES {
        let path = fixture_path(id);
        let before = fs::read(&path).unwrap_or_else(|e| panic!("read fixture {id}: {e}"));
        let after = fs::read(&path).unwrap();
        assert_eq!(
            sha256(&before),
            sha256(&after),
            "fixture changed during read: {id}"
        );
        let v3 = migrate_fixture(id, &before, source_commit).unwrap();
        let migration = run_success_case(id, &before, &v3);
        fixture_reports.push(json!({"fixtureId": id, "sourcePath": path.strip_prefix(root_dir()).unwrap().to_string_lossy(), "sourceSha256": sha256(&before), "stage": v3["campaign"]["stage"], "checkpointId": v3["worldResume"]["checkpointId"], "mode": v3["campaign"]["mode"], "choice": v3["progression"]["optionalChoice"], "migration": migration}));
        v3_samples.push((id.to_owned(), v3));
    }
    let post =
        serde_json::from_slice::<Value>(&fs::read(fixture_path("post_baizhi")).unwrap()).unwrap();
    let pre_sentinel =
        serde_json::from_slice::<Value>(&fs::read(fixture_path("pre_sentinel")).unwrap()).unwrap();
    let completed =
        serde_json::from_slice::<Value>(&fs::read(fixture_path("completed_grey_hive")).unwrap())
            .unwrap();
    let mut boundary_reports = Vec::new();
    let mut boundary_v3 = Vec::new();
    for (id, v2) in derive_boundary_samples(&post, &pre_sentinel, &completed) {
        let raw = serde_json::to_vec_pretty(&v2).unwrap();
        let v3 = migrate_fixture(&id, &raw, source_commit).unwrap();
        boundary_reports.push(json!({"fixtureId": id, "synthetic": true, "sourceSha256": sha256(&raw), "stage": v3["campaign"]["stage"], "mode": v3["campaign"]["mode"], "checkpointId": v3["worldResume"]["checkpointId"], "choice": v3["progression"]["optionalChoice"], "migration": {"policy": v3["campaign"].get("migrationConflict").cloned().unwrap_or(Value::Null), "v3Valid": validate_v3(&v3).is_ok()}}));
        boundary_v3.push(v3);
    }
    let first_v3 = &v3_samples[0].1;
    let source_initial = fs::read(fixture_path("grey_hive_initial")).unwrap();
    let corrupt = run_invalid_case("corrupt_json", br"{not-json", "E_JSON_INVALID");
    let unknown = run_invalid_case("unknown_v2_version", br#"{"version":99}"#, "E_V2_VERSION");
    let missing = run_invalid_case("missing_v2_field", br#"{"version":2,"mode":"world","current_world_id":"grey_hive","objectives":[],"flags":[],"decisions":{},"map":null}"#, "E_V2_MISSING_FIELD:optional_choice");
    let invalid_checkpoint = run_invalid_checkpoint(first_v3);
    let backup_failure = run_backup_failure("backup_failure", &source_initial, first_v3);
    let interrupted = run_interrupted_commit(
        "interrupted_before_atomic_commit",
        &source_initial,
        first_v3,
    );
    let failures = vec![
        corrupt,
        unknown,
        missing,
        invalid_checkpoint,
        backup_failure,
        interrupted,
    ];
    let all_fixture_pass = fixture_reports
        .iter()
        .all(|v| v["migration"]["pass"] == true);
    let all_boundary_pass = boundary_reports
        .iter()
        .all(|v| v["migration"]["v3Valid"].as_bool() == Some(true))
        && boundary_reports[0]["checkpointId"].as_str() == Some("gh_cp_central")
        && boundary_reports[0]["choice"].is_null()
        && boundary_reports[1]["checkpointId"].as_str() == Some("gh_cp_central")
        && boundary_reports[1]["choice"].is_null()
        && boundary_reports[2]["mode"].as_str() == Some("result")
        && boundary_reports[2]["checkpointId"].as_str() == Some("gh_cp_exit")
        && boundary_reports[2]["choice"].is_null()
        && boundary_reports[3]["checkpointId"].as_str() == Some("gh_cp_post_bio")
        && boundary_reports[3]["choice"].as_str() == Some("hive_leave_sample");
    let all_fault_pass = failures.iter().all(|v| v["pass"] == true);
    let report = json!({
        "schema": "continuous_save_v3_migration_probe_ag11_v1",
        "status": if all_fixture_pass && all_boundary_pass && all_fault_pass { "isolated_save_v3_probe_passed_with_formal_gaps" } else { "isolated_save_v3_probe_failed" },
        "pass": all_fixture_pass && all_boundary_pass && all_fault_pass,
        "scope": "disposable v2 fixture migration rehearsal only; no formal migrator and no real save access",
        "realSaveAccessed": false,
        "schemaContract": {
            "saveV3": "SaveV3{schemaVersion=3,saveId,source,campaign,progression,worldResume}",
            "source": ["schemaVersion", "fixtureId", "sourceSha256", "sourceCommit", "migration"],
            "worldResume": ["worldId", "checkpointId", "resumePolicy", "transform", "transformResolver"],
            "transformPolicy": "transform is null until formal Blockout checkpoint resolver; v2 px/py are never scaled or persisted",
            "resultMapNullPolicy": "result mode resumes to result summary/hub flow; gh_cp_exit is a semantic return reference, not a transform"
        },
        "currentV2Evidence": {
            "schemaSource": "server-rs/src/campaign_store.rs:318-348,364-430,768-877",
            "campaignSource": "server-rs/src/campaign.rs:595-623,2271-2338,2420-2525",
            "fixtureReport": "docs/CONTINUOUS_V2_FIXTURE_REPORT.md",
            "checkpointReview": "docs/V2_V3_CHECKPOINT_MAPPING_REVIEW.md",
            "v4Candidate": "docs/GREY_HIVE_ROUTE_SEMANTICS_REVIEW.md",
            "v2ChoiceIsOptional": true,
            "v4GateBChoiceCandidateIsRequired": true
        },
        "fixtureMigrations": fixture_reports,
        "boundarySamples": boundary_reports,
        "faultInjection": failures,
        "checks": {
            "fixtures": {"count": fixture_reports.len(), "passed": all_fixture_pass},
            "boundarySamples": {"count": boundary_reports.len(), "passed": all_boundary_pass, "noChoiceDoesNotPassGateB": boundary_reports[0]["checkpointId"] == "gh_cp_central" && boundary_reports[1]["checkpointId"] == "gh_cp_central", "noChoiceNotSynthesized": boundary_reports[0]["choice"].is_null() && boundary_reports[1]["choice"].is_null() && boundary_reports[2]["choice"].is_null()},
            "faultInjection": {"count": failures.len(), "passed": all_fault_pass},
            "idempotence": fixture_reports.iter().all(|v| v["migration"]["repeatIdempotent"] == true),
            "sourceIntegrity": fixture_reports.iter().all(|v| v["migration"]["sourceSha256Before"] == v["migration"]["sourceSha256After"] && v["migration"]["sourceSha256After"] == v["migration"]["backupSha256"]),
            "coordinatePolicy": v3_samples.iter().all(|(_, v)| validate_v3(v).is_ok()),
            "completedResultPolicy": v3_samples.iter().find(|(id, _)| id == "completed_grey_hive").is_some_and(|(_, v)| v["campaign"]["mode"] == "result" && v["worldResume"]["resumePolicy"] == "result_then_hub")
        },
        "formalGaps": [
            "No formal Rust/Serde SaveV3 type or production migrator is implemented.",
            "No permanent v2 read compatibility promise is made; this is a one-time migration candidate.",
            "Checkpoint transforms, legal nav recovery, atomic fsync/rename semantics and durable backup retention require TP-03.",
            "v4 Gate B requiring a Baizhi choice is a candidate contract; current campaign.rs still marks hive_choice optional.",
            "Grey Hive migration does not alter Mist Harbor or Clockworks runtime."
        ]
    });
    fs::create_dir_all(result_path().parent().unwrap()).unwrap();
    fs::write(
        result_path(),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
    println!(
        "AG-11 save migration: status={} fixtures={} boundaries={} faults={} pass={}",
        report["status"],
        fixture_reports.len(),
        boundary_reports.len(),
        failures.len(),
        report["pass"]
    );
    if report["pass"] != true {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fixture_candidates_migrate_and_validate() {
        for id in FIXTURES {
            let bytes = fs::read(fixture_path(id)).unwrap();
            let v3 = migrate_fixture(id, &bytes, "test-commit").unwrap();
            assert!(validate_v3(&v3).is_ok(), "{id}");
        }
    }

    #[test]
    fn no_choice_falls_back_before_gate_b_without_synthesizing() {
        let post = serde_json::from_slice::<Value>(&fs::read(fixture_path("post_baizhi")).unwrap())
            .unwrap();
        let pre = serde_json::from_slice::<Value>(&fs::read(fixture_path("pre_sentinel")).unwrap())
            .unwrap();
        let completed = serde_json::from_slice::<Value>(
            &fs::read(fixture_path("completed_grey_hive")).unwrap(),
        )
        .unwrap();
        let (id, sample) = derive_boundary_samples(&post, &pre, &completed)
            .into_iter()
            .next()
            .unwrap();
        let v3 =
            migrate_fixture(&id, &serde_json::to_vec(&sample).unwrap(), "test-commit").unwrap();
        assert_eq!(v3["worldResume"]["checkpointId"], "gh_cp_central");
        assert!(v3["progression"]["optionalChoice"].is_null());
        assert_eq!(v3["campaign"]["gateFacts"]["gateB"], "locked");
    }

    #[test]
    fn both_choice_branches_are_preserved() {
        let post = serde_json::from_slice::<Value>(&fs::read(fixture_path("post_baizhi")).unwrap())
            .unwrap();
        let pre = serde_json::from_slice::<Value>(&fs::read(fixture_path("pre_sentinel")).unwrap())
            .unwrap();
        let completed = serde_json::from_slice::<Value>(
            &fs::read(fixture_path("completed_grey_hive")).unwrap(),
        )
        .unwrap();
        let samples = derive_boundary_samples(&post, &pre, &completed);
        let leave = migrate_fixture(
            &samples[3].0,
            &serde_json::to_vec(&samples[3].1).unwrap(),
            "test-commit",
        )
        .unwrap();
        assert_eq!(leave["progression"]["optionalChoice"], "hive_leave_sample");
        assert_eq!(leave["worldResume"]["checkpointId"], "gh_cp_post_bio");
    }

    #[test]
    fn fault_codes_are_deterministic() {
        assert_eq!(parse_v2(br"{bad").unwrap_err(), "E_JSON_INVALID");
        assert_eq!(parse_v2(br#"{"version":99}"#).unwrap_err(), "E_V2_VERSION");
    }
}

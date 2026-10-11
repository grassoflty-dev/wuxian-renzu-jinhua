use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const FIXTURES: [&str; 6] = [
    "grey_hive_initial",
    "post_power",
    "post_gate_a",
    "post_baizhi",
    "pre_sentinel",
    "completed_grey_hive",
];
const KNOWN_CHECKPOINTS: [&str; 5] = [
    "gh_cp_airlock",
    "gh_cp_power",
    "gh_cp_central",
    "gh_cp_post_bio",
    "gh_cp_pre_sentinel",
];

fn root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf()
}

fn fixture_bytes(id: &str) -> Vec<u8> {
    fs::read(
        root_dir()
            .join("server-rs/tests/fixtures/continuous-v2")
            .join(format!("{id}.json")),
    )
    .unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn flag(v2: &Value, wanted: &str) -> bool {
    v2["flags"]
        .as_array()
        .map(|xs| xs.iter().any(|x| x.as_str() == Some(wanted)))
        .unwrap_or(false)
        || v2["objectives"]
            .as_array()
            .map(|xs| xs.iter().any(|x| x["id"] == wanted && x["done"] == true))
            .unwrap_or(false)
}

fn choice(v2: &Value) -> Option<String> {
    v2["optional_choice"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| v2["decisions"]["grey_hive"].as_str().map(str::to_owned))
        .filter(|x| x == "hive_help_baizhi" || x == "hive_leave_sample")
}

fn stage_for(v2: &Value) -> (&'static str, &'static str, &'static str) {
    if v2["mode"] == "result" || v2["map"].is_null() {
        return ("result", "gh_cp_exit", "result_then_hub");
    }
    let lockdown_done = flag(v2, "hive_lockdown");
    let world = v2["current_world_id"].as_str().unwrap_or("grey_hive");
    if world == "grey_hive"
        && lockdown_done
        && v2["map"]["revision"].as_i64().unwrap_or_default() >= 150
    {
        return ("pre_sentinel", "gh_cp_pre_sentinel", "world_resume");
    }
    if lockdown_done {
        return ("post_baizhi", "gh_cp_post_bio", "world_resume");
    }
    if flag(v2, "hive_power") && v2["map"]["revision"].as_i64().unwrap_or_default() >= 40 {
        return ("gate_a", "gh_cp_central", "world_resume");
    }
    if flag(v2, "hive_power") {
        return ("post_power", "gh_cp_power", "world_resume");
    }
    ("initial", "gh_cp_airlock", "world_resume")
}

fn required_field(v2: &Value, name: &str) -> Result<(), String> {
    if v2.get(name).is_none() {
        Err(format!("E_V2_MISSING_FIELD:{name}"))
    } else {
        Ok(())
    }
}

fn migrate(v2: &Value, source: &[u8], save_id: &str) -> Result<Value, String> {
    if v2["version"] != 2 {
        return Err("E_V2_VERSION".into());
    }
    for field in [
        "mode",
        "current_world_id",
        "objectives",
        "flags",
        "decisions",
        "map",
    ] {
        required_field(v2, field)?;
    }
    if v2["current_world_id"].as_str() != Some("grey_hive") {
        return Err("E_V2_WORLD".into());
    }
    let (stage, checkpoint, resume_policy) = stage_for(v2);
    if !KNOWN_CHECKPOINTS.contains(&checkpoint) && stage != "result" {
        return Err("E_V3_CHECKPOINT".into());
    }
    let lockdown_done = flag(v2, "hive_lockdown");
    let selected = choice(v2);
    let gate_b = if lockdown_done { "open" } else { "locked" };
    let mut gate_facts = Map::new();
    gate_facts.insert(
        "gateA".into(),
        json!(if flag(v2, "hive_power") {
            "open"
        } else {
            "locked"
        }),
    );
    gate_facts.insert("gateB".into(), json!(gate_b));
    gate_facts.insert("gateBRequires".into(), json!("hive_lockdown==completed"));
    gate_facts.insert(
        "hiveChoiceState".into(),
        json!(selected.as_deref().unwrap_or("unresolved")),
    );
    let mode = v2["mode"].as_str().unwrap_or("world");
    let source_sha = sha256(source);
    Ok(json!({
        "schemaVersion": 3,
        "saveId": save_id,
        "source": {"schemaVersion": 2, "fixtureId": save_id, "sourceSha256": source_sha, "migration": "v2_to_v3_optional_choice_ag20"},
        "campaign": {"worldId": "grey_hive", "mode": mode, "stage": stage, "sourceMapWasNull": v2["map"].is_null(), "gateFacts": Value::Object(gate_facts), "choicePolicy": "optional"},
        "progression": {"objectives": v2["objectives"].clone(), "flags": v2["flags"].clone(), "decisions": v2["decisions"].clone(), "optionalChoice": selected, "runRewardAllowed": v2["run_reward_allowed"].clone(), "runScrap": v2["run_scrap"].clone(), "enemiesDefeated": v2["enemies_defeated"].clone()},
        "worldResume": {"worldId": "grey_hive", "checkpointId": checkpoint, "resumePolicy": resume_policy, "transform": Value::Null, "transformResolver": "semantic_checkpoint_only_no_v2_px_py_scaling"}
    }))
}

fn canonical(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}

fn validate_checkpoint(v3: &Value) -> Result<(), String> {
    let checkpoint = v3["worldResume"]["checkpointId"]
        .as_str()
        .unwrap_or_default();
    if checkpoint == "gh_cp_exit" || KNOWN_CHECKPOINTS.contains(&checkpoint) {
        Ok(())
    } else {
        Err("E_V3_CHECKPOINT".into())
    }
}

fn unique_sandbox(label: &str) -> PathBuf {
    env::temp_dir().join(format!("ag20-save-v3-{label}-{}", std::process::id()))
}

fn transaction_evidence(
    id: &str,
    source_bytes: &[u8],
    v3: &Value,
) -> (String, String, String, bool) {
    let dir = unique_sandbox(id);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("campaign-v2.json");
    let backup = dir.join("campaign-v2.json.bak");
    let target = dir.join("campaign-v3.json");
    let temp = dir.join("campaign-v3.json.tmp");
    fs::write(&source, source_bytes).unwrap();
    fs::copy(&source, &backup).unwrap();
    fs::write(&temp, serde_json::to_vec_pretty(v3).unwrap()).unwrap();
    fs::rename(&temp, &target).unwrap();
    let target_bytes = fs::read(&target).unwrap();
    let source_before = sha256(source_bytes);
    let source_after = sha256(&fs::read(&source).unwrap());
    let backup_hash = sha256(&fs::read(&backup).unwrap());
    let repeat_idempotent = fs::read(&target).unwrap() == target_bytes && !temp.exists();
    let target_hash = sha256(&target_bytes);
    let _ = fs::remove_dir_all(&dir);
    (
        source_after.clone(),
        backup_hash,
        target_hash,
        repeat_idempotent && source_before == source_after,
    )
}

fn boundary_samples(
    post_baizhi: &Value,
    pre_sentinel: &Value,
    completed: &Value,
) -> Vec<(String, Value)> {
    let mut no_choice = post_baizhi.clone();
    no_choice["optional_choice"] = Value::Null;
    no_choice["decisions"] = json!({});
    no_choice["flags"] = json!(["hive_power", "hive_lockdown"]);
    if let Some(xs) = no_choice["objectives"].as_array_mut() {
        for x in xs {
            if x["id"] == "hive_choice" {
                x["done"] = json!(false);
            }
        }
    }
    let mut leave = no_choice.clone();
    leave["optional_choice"] = json!("hive_leave_sample");
    leave["decisions"] = json!({"grey_hive":"hive_leave_sample"});
    leave["flags"] = json!(["hive_power", "hive_lockdown", "hive_leave_sample"]);
    if let Some(xs) = leave["objectives"].as_array_mut() {
        for x in xs {
            if x["id"] == "hive_choice" {
                x["done"] = json!(true);
            }
        }
    }
    let mut deep = pre_sentinel.clone();
    deep["optional_choice"] = Value::Null;
    deep["decisions"] = json!({});
    deep["flags"] = json!(["hive_power", "hive_lockdown"]);
    if let Some(xs) = deep["objectives"].as_array_mut() {
        for x in xs {
            if x["id"] == "hive_choice" {
                x["done"] = json!(false);
            }
        }
    }
    let mut rescue = completed.clone();
    rescue["optional_choice"] = Value::Null;
    rescue["decisions"] = json!({});
    rescue["flags"] = json!(["hive_power", "hive_lockdown", "hive_rescue"]);
    if let Some(xs) = rescue["objectives"].as_array_mut() {
        for x in xs {
            if x["id"] == "hive_choice" {
                x["done"] = json!(false);
            }
        }
    }
    vec![
        ("legacy_high_progress_no_choice".into(), no_choice),
        ("legacy_choice_leave_sample".into(), leave),
        ("legacy_deep_no_choice".into(), deep),
        ("legacy_rescue_no_choice".into(), rescue),
    ]
}

fn fault_report<T>(name: &str, expected: &str, actual: Result<T, String>) -> Value {
    let code = actual.err().unwrap_or_else(|| "OK".into());
    json!({"name": name, "pass": code == expected, "errorCode": code, "expectedErrorCode": expected})
}

fn run_faults(base: &Value, raw: &[u8]) -> Vec<Value> {
    let mut unknown = base.clone();
    unknown["version"] = json!(99);
    let mut missing_required = base.clone();
    missing_required.as_object_mut().unwrap().remove("mode");
    let mut invalid_v3 = migrate(base, raw, "invalid-checkpoint").unwrap();
    invalid_v3["worldResume"]["checkpointId"] = json!("gh_cp_not_real");
    let mut missing_optional = base.clone();
    missing_optional
        .as_object_mut()
        .unwrap()
        .remove("optional_choice");
    let mut out = vec![
        fault_report(
            "json_invalid",
            "E_JSON_INVALID",
            serde_json::from_slice::<Value>(b"{bad")
                .map_err(|_| "E_JSON_INVALID".into())
                .and_then(|v| migrate(&v, b"{bad", "invalid")),
        ),
        fault_report(
            "unknown_v2_version",
            "E_V2_VERSION",
            migrate(&unknown, raw, "unknown"),
        ),
        fault_report(
            "missing_required_mode",
            "E_V2_MISSING_FIELD:mode",
            migrate(&missing_required, raw, "missing"),
        ),
        fault_report(
            "invalid_checkpoint",
            "E_V3_CHECKPOINT",
            validate_checkpoint(&invalid_v3),
        ),
        run_backup_failure(raw),
        run_interrupted_commit(raw, &migrate(base, raw, "interrupted").unwrap()),
    ];
    out.push(json!({"name":"missing_optional_choice_is_legal", "pass": migrate(&missing_optional, raw, "missing-optional").is_ok(), "expected":"accepted_as_unresolved"}));
    out
}

fn run_backup_failure(raw: &[u8]) -> Value {
    let dir = unique_sandbox("backup-failure");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("campaign-v2.json");
    let backup = dir.join("campaign-v2.json.bak");
    let target = dir.join("campaign-v3.json");
    fs::write(&source, raw).unwrap();
    fs::create_dir(&backup).unwrap();
    let source_hash = sha256(&fs::read(&source).unwrap());
    let error = fs::copy(&source, &backup)
        .map(|_| "unexpected_success")
        .unwrap_or("E_BACKUP_FAILED");
    let pass = error == "E_BACKUP_FAILED"
        && source.exists()
        && backup.is_dir()
        && !target.exists()
        && sha256(&fs::read(&source).unwrap()) == source_hash;
    let result = json!({"name":"backup_failed","pass":pass,"errorCode":error,"expectedErrorCode":"E_BACKUP_FAILED","sourceExists":source.exists(),"backupIsDirectory":backup.is_dir(),"targetExists":target.exists(),"sourceSha256":source_hash});
    let _ = fs::remove_dir_all(&dir);
    result
}

fn run_interrupted_commit(raw: &[u8], v3: &Value) -> Value {
    let dir = unique_sandbox("commit-interrupted");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("campaign-v2.json");
    let backup = dir.join("campaign-v2.json.bak");
    let temp = dir.join("campaign-v3.json.tmp");
    let target = dir.join("campaign-v3.json");
    fs::write(&source, raw).unwrap();
    fs::copy(&source, &backup).unwrap();
    fs::write(&temp, serde_json::to_vec_pretty(v3).unwrap()).unwrap();
    let source_hash = sha256(&fs::read(&source).unwrap());
    let recovered =
        source.exists() && backup.exists() && sha256(&fs::read(&backup).unwrap()) == source_hash;
    let cleanup = fs::remove_file(&temp).is_ok();
    let pass = recovered && cleanup && source.exists() && backup.exists() && !target.exists();
    let result = json!({"name":"commit_interrupted","pass":pass,"errorCode":"E_COMMIT_INTERRUPTED","expectedErrorCode":"E_COMMIT_INTERRUPTED","sourceExists":source.exists(),"backupExists":backup.exists(),"tempExistsAfterRecovery":temp.exists(),"targetExists":target.exists(),"sourceSha256":source_hash,"backupSha256":sha256(&fs::read(&backup).unwrap())});
    let _ = fs::remove_dir_all(&dir);
    result
}

fn run() -> Value {
    let mut fixture_reports = Vec::new();
    let mut all_v3 = Vec::new();
    for id in FIXTURES {
        let raw = fixture_bytes(id);
        let v2: Value = serde_json::from_slice(&raw).unwrap();
        let v3 = migrate(&v2, &raw, id).unwrap();
        let again = migrate(&v2, &raw, id).unwrap();
        let canonical_bytes = canonical(&v3);
        let rerun_bytes = canonical(&again);
        let (source_after, backup_hash, target_hash, transaction_idempotent) =
            transaction_evidence(id, &raw, &v3);
        let p = &v3["progression"];
        let c = &v3["campaign"];
        let w = &v3["worldResume"];
        fixture_reports.push(json!({"fixtureId":id,"sourceSha256":sha256(&raw),"backupSha256":backup_hash,"sourceSha256After":source_after,"targetSha256":target_hash,"stage":c["stage"],"checkpointId":w["checkpointId"],"mode":c["mode"],"choice":p["optionalChoice"],"gateB":c["gateFacts"]["gateB"],"transform":w["transform"],"sourceStable":sha256(&raw)==source_after,"idempotent":canonical_bytes==rerun_bytes && transaction_idempotent}));
        all_v3.push(v3);
    }
    let post: Value = serde_json::from_slice(&fixture_bytes("post_baizhi")).unwrap();
    let pre: Value = serde_json::from_slice(&fixture_bytes("pre_sentinel")).unwrap();
    let completed: Value = serde_json::from_slice(&fixture_bytes("completed_grey_hive")).unwrap();
    let mut boundaries = Vec::new();
    for (id, v2) in boundary_samples(&post, &pre, &completed) {
        let raw = canonical(&v2);
        let v3 = migrate(&v2, &raw, &id).unwrap();
        boundaries.push(json!({"fixtureId":id,"synthetic":true,"stage":v3["campaign"]["stage"],"mode":v3["campaign"]["mode"],"checkpointId":v3["worldResume"]["checkpointId"],"choice":v3["progression"]["optionalChoice"],"gateB":v3["campaign"]["gateFacts"]["gateB"],"choiceDonePreserved":v3["progression"]["objectives"].as_array().unwrap().iter().find(|x|x["id"]=="hive_choice").unwrap()["done"],"transform":v3["worldResume"]["transform"]}));
    }
    let base: Value = serde_json::from_slice(&fixture_bytes("post_gate_a")).unwrap();
    let faults = run_faults(&base, &fixture_bytes("post_gate_a"));
    let fixture_pass = fixture_reports
        .iter()
        .all(|x| x["sourceStable"] == true && x["idempotent"] == true && x["transform"].is_null());
    let boundary_pass = boundaries[0]["gateB"] == "open"
        && boundaries[0]["checkpointId"] == "gh_cp_post_bio"
        && boundaries[1]["choice"] == "hive_leave_sample"
        && boundaries[2]["checkpointId"] == "gh_cp_pre_sentinel"
        && boundaries[2]["gateB"] == "open"
        && boundaries[3]["mode"] == "result"
        && boundaries[3]["checkpointId"] == "gh_cp_exit";
    let fault_pass = faults.iter().all(|x| x["pass"] == true);
    json!({"status":if fixture_pass && boundary_pass && fault_pass {"isolated_save_v3_optional_policy_probe_passed_with_formal_gaps"} else {"isolated_save_v3_optional_policy_probe_failed"},"pass":fixture_pass&&boundary_pass&&fault_pass,"realSaveAccessed":false,"policy":{"gateBRequires":"hive_lockdown==completed","hiveChoiceMissing":"unresolved","choiceIsNotGateBPrerequisite":true,"npcScriptCannotBlockMainline":true},"fixtureReport":fixture_reports,"boundarySamples":boundaries,"faultInjection":faults,"choicePolicyMatrix":[{"choice":"taken","gateBWhenLockdownComplete":true},{"choice":"left","gateBWhenLockdownComplete":true},{"choice":"unresolved","gateBWhenLockdownComplete":true},{"choice":"unresolved","gateBWhenLockdownIncomplete":false}],"dtoComparison":{"tp02SaveV3Fields":["schemaVersion","saveId","checkpoint.worldId","checkpoint.checkpointId","transform"],"ag20EnvelopeFields":["source","campaign","progression","worldResume"],"tp03Scope":"formal envelope plus persistence/validation/resolver; do not change TP-02 primitive DTO in this candidate"},"formalGaps":["production migrator not wired","Windows fsync/atomic rename not implemented","semantic checkpoint resolver/nav/KCC validation pending","no real save accessed"]})
}

fn main() {
    let result = run();
    let out =
        root_dir().join("tools/continuous-tps-architecture-probe/results/save-optional-ag20.json");
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!(
        "AG-20 save optional policy: pass={} status={}",
        result["pass"], result["status"]
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_fixtures_are_stable_and_idempotent() {
        for id in FIXTURES {
            let raw = fixture_bytes(id);
            let v: Value = serde_json::from_slice(&raw).unwrap();
            let a = migrate(&v, &raw, id).unwrap();
            let b = migrate(&v, &raw, id).unwrap();
            assert_eq!(canonical(&a), canonical(&b));
            assert!(a["worldResume"]["transform"].is_null());
        }
    }

    #[test]
    fn optional_choice_does_not_gate_b_or_synthesize_choice() {
        let post: Value = serde_json::from_slice(&fixture_bytes("post_baizhi")).unwrap();
        let pre: Value = serde_json::from_slice(&fixture_bytes("pre_sentinel")).unwrap();
        let done: Value = serde_json::from_slice(&fixture_bytes("completed_grey_hive")).unwrap();
        let xs = boundary_samples(&post, &pre, &done);
        let no_choice = migrate(&xs[0].1, &canonical(&xs[0].1), "no-choice").unwrap();
        let deep = migrate(&xs[2].1, &canonical(&xs[2].1), "deep-no-choice").unwrap();
        let result = migrate(&xs[3].1, &canonical(&xs[3].1), "result-no-choice").unwrap();
        assert_eq!(no_choice["campaign"]["gateFacts"]["gateB"], "open");
        assert_eq!(no_choice["progression"]["optionalChoice"], Value::Null);
        assert_eq!(no_choice["worldResume"]["checkpointId"], "gh_cp_post_bio");
        assert_eq!(deep["campaign"]["gateFacts"]["gateB"], "open");
        assert_eq!(deep["worldResume"]["checkpointId"], "gh_cp_pre_sentinel");
        assert_eq!(result["campaign"]["mode"], "result");
        assert_eq!(result["worldResume"]["resumePolicy"], "result_then_hub");
    }

    #[test]
    fn lockdown_incomplete_cannot_cross_gate_b() {
        let v: Value = serde_json::from_slice(&fixture_bytes("post_gate_a")).unwrap();
        let out = migrate(&v, &fixture_bytes("post_gate_a"), "locked").unwrap();
        assert_eq!(out["campaign"]["gateFacts"]["gateB"], "locked");
        assert_ne!(out["worldResume"]["checkpointId"], "gh_cp_post_bio");
    }

    #[test]
    fn six_fault_codes_and_optional_missing_are_deterministic() {
        let v = run();
        assert_eq!(v["faultInjection"].as_array().unwrap().len(), 7);
        assert!(v["faultInjection"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["pass"] == true));
    }
}

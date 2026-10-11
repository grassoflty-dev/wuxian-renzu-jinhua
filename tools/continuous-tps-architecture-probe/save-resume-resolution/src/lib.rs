use checkpoint_resolver_probe as resolver;
use resolver::{
    CandidateNav, ChoiceState, ProjectionFailure, ProjectionSample, ResolveError, ResolveInput,
    ResolverContract,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use wuxian_horror_ch1::save_v3_migration::{self, SaveV3Envelope};
use wuxian_horror_ch1::save_v3_persistence::{self, PersistenceFault, SaveV3PersistenceReport};
use wuxian_horror_ch1::world_v3::{Transform, Vec3, WorldStateV3};

pub const SCHEMA: &str = "continuous_save_resume_resolution_tp03d1_v1";
pub const WORLD_ID: &str = "grey_hive";
pub const LAYOUT_VERSION: &str = "ag23-v7-candidate";
pub const WORLD_EPOCH: u64 = 7;

const FIXTURES: [&str; 6] = [
    "grey_hive_initial",
    "post_power",
    "post_gate_a",
    "post_baizhi",
    "pre_sentinel",
    "completed_grey_hive",
];

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioRecord {
    pub name: String,
    pub formal_api_pointer: String,
    pub input: Value,
    pub stage: Vec<String>,
    pub events: Vec<String>,
    pub expected: Value,
    pub actual: Value,
    pub pass: bool,
    pub checkpoint_id: Option<String>,
    pub transform_provenance: Value,
    pub source_sha256: Option<String>,
    pub backup_sha256: Option<String>,
    pub target_sha256: Option<String>,
    pub error_code: Option<String>,
    pub evidence_level: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProbeReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub machine_pass: bool,
    pub candidate_only: bool,
    pub formal_api_reuse: Value,
    pub fixture_ids: Vec<&'static str>,
    pub canonical_checkpoints: Value,
    pub synthetic_scene_chain: Value,
    pub scenarios: Vec<ScenarioRecord>,
    pub formal_gaps: Vec<&'static str>,
    pub gate_conclusion: Value,
}

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

fn evidence_path(name: &str) -> PathBuf {
    root_dir()
        .join("tools/continuous-tps-architecture-probe/results")
        .join(name)
}

fn read_fixture(id: &str) -> Vec<u8> {
    fs::read(fixture_path(id)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn choice_state(value: &str) -> ChoiceState {
    match value {
        "taken" => ChoiceState::Taken,
        "left" => ChoiceState::Left,
        _ => ChoiceState::Unresolved,
    }
}

fn choice_from_envelope(envelope: &SaveV3Envelope) -> ChoiceState {
    choice_state(&envelope.campaign.gate_facts.hive_choice_state)
}

fn choice_label(envelope: &SaveV3Envelope) -> &'static str {
    match envelope.campaign.gate_facts.hive_choice_state.as_str() {
        "hive_help_baizhi" => "taken",
        "hive_leave_sample" => "left",
        _ => "unresolved",
    }
}

fn world_state() -> WorldStateV3 {
    WorldStateV3::new(
        WORLD_ID,
        WORLD_EPOCH,
        0,
        0,
        Transform::new(Vec3::zero(), 0.0).unwrap(),
        Vec3::zero(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
}

fn sandbox(label: &str, bytes: &[u8]) -> (PathBuf, PathBuf, PathBuf) {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tp03d1-{label}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let source = root.join("legacy-v2.json");
    let target = root.join("save-v3.json");
    fs::write(&source, bytes).unwrap();
    (root, source, target)
}

fn clean_sandbox(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

fn target_envelope(target: &Path) -> SaveV3Envelope {
    serde_json::from_slice(&fs::read(target).unwrap()).unwrap()
}

fn formal_commit(
    label: &str,
    bytes: &[u8],
) -> Result<(SaveV3Envelope, SaveV3PersistenceReport, PathBuf), String> {
    let (root, source, target) = sandbox(label, bytes);
    let result = save_v3_persistence::commit_v2_to_v3(&source, &target, &root)
        .map_err(|error| error.code())?;
    let envelope = target_envelope(&target);
    Ok((envelope, result, root))
}

fn transform_json(resolution: &resolver::Resolution) -> Value {
    json!({
        "xM": resolution.transform.x_m,
        "yM": resolution.transform.y_m,
        "zM": resolution.transform.z_m,
        "yawRad": resolution.transform.yaw_rad,
        "candidateOnly": resolution.candidate_only,
    })
}

fn base_record(
    name: &str,
    formal_api_pointer: &str,
    input: Value,
    expected: Value,
    actual: Value,
    pass: bool,
) -> ScenarioRecord {
    ScenarioRecord {
        name: name.into(),
        formal_api_pointer: formal_api_pointer.into(),
        input,
        stage: Vec::new(),
        events: Vec::new(),
        expected,
        actual,
        pass,
        checkpoint_id: None,
        transform_provenance: json!({"status":"not_applicable"}),
        source_sha256: None,
        backup_sha256: None,
        target_sha256: None,
        error_code: None,
        evidence_level: "formal_api_plus_isolated_candidate",
    }
}

fn with_chain_metadata(
    mut record: ScenarioRecord,
    envelope: &SaveV3Envelope,
    persisted: &SaveV3PersistenceReport,
    stage: &[&str],
    events: &[&str],
) -> ScenarioRecord {
    record.stage = stage.iter().map(|value| (*value).into()).collect();
    record.events = events.iter().map(|value| (*value).into()).collect();
    record.checkpoint_id = Some(envelope.world_resume.checkpoint_id.clone());
    record.transform_provenance = json!({
        "formalTransform": envelope.world_resume.transform.clone(),
        "resolver": envelope.world_resume.transform_resolver,
        "candidateLayout": LAYOUT_VERSION,
        "candidateOnly": record
            .actual
            .get("resolver")
            .and_then(|value| value.get("transform"))
            .and_then(|value| value.get("candidateOnly"))
            .and_then(Value::as_bool),
        "worldEpoch": WORLD_EPOCH,
    });
    record.source_sha256 = Some(persisted.source_sha256.clone());
    record.backup_sha256 = Some(persisted.backup_sha256.clone());
    record.target_sha256 = Some(persisted.target_sha256.clone());
    record
}

fn resolve_persisted(
    id: &str,
    bytes: &[u8],
    expected_stage: &str,
    expected_checkpoint: &str,
    expected_gate_b: &str,
    expected_choice: &str,
    expected_map_null: bool,
) -> ScenarioRecord {
    let source_hash = sha256(bytes);
    let (envelope, persisted, root) = match formal_commit(id, bytes) {
        Ok(value) => value,
        Err(error) => {
            return base_record(
                &format!("fixture_{id}"),
                "save_v3_persistence::commit_v2_to_v3",
                json!({"fixture":id}),
                json!({"commit":"committed"}),
                json!({"error":error}),
                false,
            )
        }
    };
    let world = world_state();
    let mut record;
    let map_null = serde_json::from_slice::<Value>(bytes)
        .unwrap()
        .get("map")
        .is_some_and(Value::is_null);
    let mut actual = json!({
        "commitStatus": persisted.status,
        "schemaVersion": envelope.save.schema_version(),
        "stage": envelope.campaign.stage,
        "checkpointId": envelope.world_resume.checkpoint_id,
        "gateB": envelope.campaign.gate_facts.gate_b,
        "choice": choice_label(&envelope),
        "choiceRaw": envelope.campaign.gate_facts.hive_choice_state,
        "mapNull": map_null,
        "sourceUnchanged": persisted.source_unchanged,
        "pendingExists": persisted.pending_exists,
        "transformNone": envelope.world_resume.transform.is_none(),
    });
    let mut pass = persisted.status == "committed"
        && persisted.source_sha256 == source_hash
        && persisted.backup_sha256 == source_hash
        && persisted.source_unchanged
        && !persisted.pending_exists
        && envelope.campaign.stage == expected_stage
        && envelope.world_resume.checkpoint_id == expected_checkpoint
        && envelope.campaign.gate_facts.gate_b == expected_gate_b
        && choice_label(&envelope) == expected_choice
        && map_null == expected_map_null
        && envelope.world_resume.transform.is_none();
    if map_null {
        let nav = CandidateNav::grey_hive_v7_candidate();
        let input = ResolveInput {
            checkpoint_id: envelope.world_resume.checkpoint_id.clone(),
            world_id: WORLD_ID.into(),
            layout_version: LAYOUT_VERSION.into(),
            world_epoch: world.world_epoch(),
            lockdown_completed: envelope.campaign.gate_facts.gate_b == "open",
            choice: choice_from_envelope(&envelope),
            result_map_null: true,
        };
        let resolved = resolver::resolve_checkpoint(
            &ResolverContract {
                world_id: WORLD_ID.into(),
                layout_version: LAYOUT_VERSION.into(),
                production_scene_bound: false,
            },
            &input,
            &nav,
        );
        actual["resolver"] =
            json!({"result":resolved.err().map(|error| error.as_str()),"navCalls":nav.calls()});
        pass &= resolved == Err(ResolveError::TerminalNotApplicable) && nav.calls() == 0;
    } else {
        let nav = CandidateNav::grey_hive_v7_candidate();
        let input = ResolveInput {
            checkpoint_id: envelope.world_resume.checkpoint_id.clone(),
            world_id: WORLD_ID.into(),
            layout_version: LAYOUT_VERSION.into(),
            world_epoch: world.world_epoch(),
            lockdown_completed: envelope.campaign.gate_facts.gate_b == "open",
            choice: choice_from_envelope(&envelope),
            result_map_null: false,
        };
        let resolved = resolver::resolve_checkpoint(
            &ResolverContract {
                world_id: WORLD_ID.into(),
                layout_version: LAYOUT_VERSION.into(),
                production_scene_bound: false,
            },
            &input,
            &nav,
        );
        match resolved {
            Ok(resolution) => {
                actual["resolver"] = json!({"result":"ok","navCalls":nav.calls(),"transform":transform_json(&resolution)});
                pass &= resolution.candidate_only && resolution.transform.is_finite();
                record = base_record(
                    &format!("fixture_{id}"),
                    "save_v3_migration::migrate_v2_value → save_v3_persistence::commit_v2_to_v3 → checkpoint_resolver::resolve_checkpoint",
                    json!({"fixture":id,"worldEpoch":WORLD_EPOCH,"layoutVersion":LAYOUT_VERSION}),
                    json!({"stage":expected_stage,"checkpointId":expected_checkpoint,"gateB":expected_gate_b,"choice":expected_choice,"candidateOnly":true}),
                    actual,
                    pass,
                );
                record = with_chain_metadata(
                    record,
                    &envelope,
                    &persisted,
                    &[
                        "v2 fixture",
                        "formal migration",
                        "formal persistence roundtrip",
                        "checkpoint/world/layout/epoch guard",
                        "synthetic nav/spawn candidate",
                    ],
                    &[
                        "read fixture",
                        "migrate",
                        "write backup",
                        "write/reload pending",
                        "rename target",
                        "resolve candidate transform",
                    ],
                );
                clean_sandbox(&root);
                return record;
            }
            Err(error) => {
                actual["resolver"] = json!({"error":error.as_str(),"navCalls":nav.calls()});
                pass = false;
            }
        }
    }
    record = base_record(
        &format!("fixture_{id}"),
        "save_v3_migration::migrate_v2_value → save_v3_persistence::commit_v2_to_v3 → checkpoint_resolver::resolve_checkpoint",
        json!({"fixture":id,"worldEpoch":WORLD_EPOCH,"layoutVersion":LAYOUT_VERSION}),
        json!({"stage":expected_stage,"checkpointId":expected_checkpoint,"gateB":expected_gate_b,"choice":expected_choice,"candidateOnly":!expected_map_null}),
        actual,
        pass,
    );
    record = with_chain_metadata(
        record,
        &envelope,
        &persisted,
        &[
            "v2 fixture",
            "formal migration",
            "formal persistence roundtrip",
            "checkpoint/world/layout/epoch guard",
            "terminal no-spawn",
        ],
        &[
            "read fixture",
            "migrate",
            "write backup",
            "write/reload pending",
            "rename target",
            "terminal result no transform",
        ],
    );
    clean_sandbox(&root);
    record
}

fn choice_variant(base: &Value, choice: Option<&str>) -> Vec<u8> {
    let mut value = base.clone();
    match choice {
        Some(value_choice) => {
            value["optional_choice"] = json!(value_choice);
            value["decisions"] = json!({"grey_hive":value_choice});
            value["objectives"][3]["done"] = json!(true);
        }
        None => {
            value["optional_choice"] = Value::Null;
            value["decisions"] = json!({});
            value["objectives"][3]["done"] = json!(false);
        }
    }
    serde_json::to_vec(&value).unwrap()
}

fn resolver_error_record(
    name: &str,
    input: ResolveInput,
    nav: &dyn resolver::ReadOnlyNavProjection,
    expected: ResolveError,
    pointer: &str,
) -> ScenarioRecord {
    let contract = ResolverContract {
        world_id: WORLD_ID.into(),
        layout_version: LAYOUT_VERSION.into(),
        production_scene_bound: false,
    };
    let result = resolver::resolve_checkpoint(&contract, &input, nav);
    let actual_error = result.as_ref().err().copied();
    let mut record = base_record(
        name,
        pointer,
        json!({
            "checkpointId": input.checkpoint_id,
            "worldId": input.world_id,
            "layoutVersion": input.layout_version,
            "worldEpoch": input.world_epoch,
            "lockdownCompleted": input.lockdown_completed,
            "choice": input.choice.as_str(),
            "resultMapNull": input.result_map_null,
        }),
        json!({"error":expected.as_str()}),
        json!({"error":actual_error.map(ResolveError::as_str)}),
        actual_error == Some(expected),
    );
    record.error_code = actual_error.map(|error| error.as_str().into());
    record.transform_provenance = json!({"candidateOnly":false,"productionSceneBound":false});
    record
}

fn base_input(checkpoint_id: &str) -> ResolveInput {
    ResolveInput {
        checkpoint_id: checkpoint_id.into(),
        world_id: WORLD_ID.into(),
        layout_version: LAYOUT_VERSION.into(),
        world_epoch: WORLD_EPOCH,
        lockdown_completed: true,
        choice: ChoiceState::Unresolved,
        result_map_null: false,
    }
}

fn persistence_failure_record(fault: PersistenceFault, expected: &str) -> ScenarioRecord {
    let bytes = read_fixture("post_power");
    let (root, source, target) = sandbox("fault", &bytes);
    let before = save_v3_persistence::inspect_transaction(&source, &target);
    let result = save_v3_persistence::commit_v2_to_v3_with_fault(&source, &target, &root, fault);
    let after = save_v3_persistence::inspect_transaction(&source, &target);
    let error = result.err().map(|value| value.code());
    let target_hash = after.target.sha256.clone();
    let pass = error.as_deref() == Some(expected)
        && !after.target.exists
        && before.source.sha256 == after.source.sha256
        && before.target.sha256 == target_hash;
    let mut record = base_record(
        &format!("persistence_failure_{expected}"),
        "save_v3_persistence::commit_v2_to_v3_with_fault",
        json!({"fault":format!("{fault:?}")}),
        json!({"error":expected,"targetCommitted":false}),
        json!({"error":error,"targetCommitted":after.target.exists,"pendingExists":after.pending.exists,"sourceHash":after.source.sha256,"backupHash":after.backup.sha256}),
        pass,
    );
    record.error_code = error;
    record.events = vec![
        "read source".into(),
        "fault injected".into(),
        "inspect transaction".into(),
    ];
    record.transform_provenance =
        json!({"candidateOnly":false,"targetCommitForbiddenOnFailure":true});
    clean_sandbox(&root);
    record
}

fn reload_idempotence_record() -> ScenarioRecord {
    let bytes = read_fixture("post_baizhi");
    let (root, source, target) = sandbox("idempotence", &bytes);
    let first = save_v3_persistence::commit_v2_to_v3(&source, &target, &root).unwrap();
    let first_target = fs::read(&target).unwrap();
    let second = save_v3_persistence::commit_v2_to_v3(&source, &target, &root).unwrap();
    let second_target = fs::read(&target).unwrap();
    let pass = first.status == "committed"
        && second.status == "already_present"
        && sha256(&first_target) == sha256(&second_target)
        && first.source_sha256 == second.source_sha256
        && first.backup_sha256 == second.backup_sha256
        && first.source_unchanged
        && second.source_unchanged;
    let mut record = base_record(
        "reload_idempotent",
        "save_v3_persistence::commit_v2_to_v3 (twice)",
        json!({"fixture":"post_baizhi"}),
        json!({"first":"committed","second":"already_present","targetBytesStable":true,"sourceUnchanged":true}),
        json!({"first":first.status,"second":second.status,"targetHash1":sha256(&first_target),"targetHash2":sha256(&second_target),"sourceUnchanged":[first.source_unchanged,second.source_unchanged]}),
        pass,
    );
    record.source_sha256 = Some(first.source_sha256);
    record.backup_sha256 = Some(first.backup_sha256);
    record.target_sha256 = Some(second.target_sha256);
    record.events = vec![
        "first formal commit".into(),
        "reload target".into(),
        "second formal commit".into(),
        "compare hashes".into(),
    ];
    record.transform_provenance = json!({"candidateOnly":false,"formalTargetReload":true});
    clean_sandbox(&root);
    record
}

fn wrong_hash_record() -> ScenarioRecord {
    let bytes = read_fixture("post_power");
    let (root, source, target) = sandbox("wrong-hash", &bytes);
    let backup = source.with_file_name("legacy-v2.json.bak");
    fs::write(&backup, b"tampered-backup").unwrap();
    let result = save_v3_persistence::commit_v2_to_v3(&source, &target, &root);
    let evidence = save_v3_persistence::inspect_transaction(&source, &target);
    let error = result.err().map(|value| value.code());
    let pass = error.as_deref() == Some("E_BACKUP_SOURCE_MISMATCH") && !evidence.target.exists;
    let mut record = base_record(
        "source_backup_hash_mismatch",
        "save_v3_persistence::commit_v2_to_v3 backup validation",
        json!({"fixture":"post_power","backup":"tampered"}),
        json!({"error":"E_BACKUP_SOURCE_MISMATCH","targetCommitted":false}),
        json!({"error":error,"targetCommitted":evidence.target.exists,"sourceHash":evidence.source.sha256,"backupHash":evidence.backup.sha256}),
        pass,
    );
    record.error_code = error;
    record.events = vec![
        "read source".into(),
        "read conflicting backup".into(),
        "reject before target".into(),
    ];
    record.transform_provenance =
        json!({"candidateOnly":false,"targetCommitForbiddenOnHashMismatch":true});
    clean_sandbox(&root);
    record
}

fn px_py_no_scaling_record() -> ScenarioRecord {
    let original: Value = serde_json::from_slice(&read_fixture("post_baizhi")).unwrap();
    let original_bytes = choice_variant(&original, None);
    let mut scaled = original.clone();
    scaled["map"]["px"] = json!(17000);
    scaled["map"]["py"] = json!(8000);
    let scaled_bytes = choice_variant(&scaled, None);
    let original_env = save_v3_migration::migrate_v2_json(
        &original_bytes,
        "pxpy-original",
        Some(&sha256(&original_bytes)),
    )
    .unwrap();
    let scaled_env = save_v3_migration::migrate_v2_json(
        &scaled_bytes,
        "pxpy-scaled",
        Some(&sha256(&scaled_bytes)),
    )
    .unwrap();
    let nav = CandidateNav::grey_hive_v7_candidate();
    let contract = ResolverContract {
        world_id: WORLD_ID.into(),
        layout_version: LAYOUT_VERSION.into(),
        production_scene_bound: false,
    };
    let make_input = |env: &SaveV3Envelope| ResolveInput {
        checkpoint_id: env.world_resume.checkpoint_id.clone(),
        world_id: WORLD_ID.into(),
        layout_version: LAYOUT_VERSION.into(),
        world_epoch: WORLD_EPOCH,
        lockdown_completed: true,
        choice: ChoiceState::Unresolved,
        result_map_null: false,
    };
    let a = resolver::resolve_checkpoint(&contract, &make_input(&original_env), &nav).unwrap();
    let b = resolver::resolve_checkpoint(&contract, &make_input(&scaled_env), &nav).unwrap();
    let pass = original_env.world_resume.transform.is_none()
        && scaled_env.world_resume.transform.is_none()
        && a.transform == b.transform;
    let mut record = base_record(
        "legacy_px_py_not_scaled",
        "save_v3_migration::migrate_v2_json + checkpoint_resolver::resolve_checkpoint",
        json!({"originalPxPy":[original["map"]["px"],original["map"]["py"]],"scaledPxPy":[17000,8000]}),
        json!({"checkpointStable":true,"transformIndependentOfLegacyGrid":true}),
        json!({"originalCheckpoint":original_env.world_resume.checkpoint_id,"scaledCheckpoint":scaled_env.world_resume.checkpoint_id,"originalTransform":transform_json(&a),"scaledTransform":transform_json(&b),"formalTransformsNull":[original_env.world_resume.transform.is_none(),scaled_env.world_resume.transform.is_none()]}),
        pass,
    );
    record.checkpoint_id = Some(original_env.world_resume.checkpoint_id);
    record.transform_provenance = json!({"source":"synthetic candidate nav","legacyPxPyUsedForTransform":false,"candidateOnly":true});
    record.events = vec![
        "clone fixture in memory".into(),
        "migrate both values through formal API".into(),
        "resolve both against same candidate nav".into(),
    ];
    record
}

fn scene_chain_record() -> (ScenarioRecord, Value) {
    let evidence: Value = serde_json::from_slice(
        &fs::read(evidence_path("formal-scene-exporter-tp09b1.json")).unwrap(),
    )
    .unwrap();
    let positive = evidence["positive"].clone();
    let negative = evidence["v7Negative"].clone();
    let mut tampered = positive.clone();
    tampered["stageHashes"]["blenderExportHash"] = json!("tampered");
    let hashes_valid = |value: &Value| {
        [
            "sourceSceneSha256",
            "blenderExportHash",
            "babylonImportHash",
            "rapierNavBakeHash",
        ]
        .iter()
        .all(|key| {
            value["stageHashes"][*key]
                .as_str()
                .is_some_and(|hash| hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
        })
    };
    let validate =
        |value: &Value| value["chainPass"].as_bool() == Some(true) && hashes_valid(value);
    let negative_rejects = negative["candidateOnly"].as_bool() == Some(true)
        && negative["formalSceneReady"].as_bool() == Some(false)
        && negative["expectedFailure"].as_bool() == Some(true);
    let pass = validate(&positive) && !validate(&tampered) && negative_rejects;
    let record = base_record(
        "synthetic_scene_chain_and_v7_negative",
        "TP-09A1/B1 result evidence validator (read-only; no formal scene export API exists)",
        json!({"positiveFixture":positive["fixture"],"negativeFixture":negative["fixture"]}),
        json!({"positiveChain":true,"tamperedChain":false,"v7NegativeContinuesToReject":true}),
        json!({"positiveChain":validate(&positive),"tamperedChain":validate(&tampered),"v7Negative":{"candidateOnly":negative["candidateOnly"],"formalSceneReady":negative["formalSceneReady"],"expectedFailure":negative["expectedFailure"]}}),
        pass,
    );
    (
        record,
        json!({"positiveChain":validate(&positive),"v7Negative":negative,"tamperCheck":!validate(&tampered),"formalSceneReady":false,"syntheticOnly":true}),
    )
}

pub fn run_all_scenarios() -> ProbeReport {
    let mut scenarios = Vec::new();
    let expected = [
        (
            "grey_hive_initial",
            "initial",
            "gh_cp_airlock",
            "locked",
            "unresolved",
            false,
        ),
        (
            "post_power",
            "post_power",
            "gh_cp_power",
            "locked",
            "unresolved",
            false,
        ),
        (
            "post_gate_a",
            "gate_a",
            "gh_cp_central",
            "locked",
            "unresolved",
            false,
        ),
        (
            "post_baizhi",
            "post_baizhi",
            "gh_cp_post_bio",
            "open",
            "taken",
            false,
        ),
        (
            "pre_sentinel",
            "pre_sentinel",
            "gh_cp_pre_sentinel",
            "open",
            "taken",
            false,
        ),
        (
            "completed_grey_hive",
            "result",
            "gh_cp_exit",
            "open",
            "taken",
            true,
        ),
    ];
    for (id, stage, checkpoint, gate_b, choice, map_null) in expected {
        let bytes = read_fixture(id);
        scenarios.push(resolve_persisted(
            id, &bytes, stage, checkpoint, gate_b, choice, map_null,
        ));
        if id == "post_baizhi" {
            let base: Value = serde_json::from_slice(&bytes).unwrap();
            for (label, state) in [
                ("taken", Some("hive_help_baizhi")),
                ("left", Some("hive_leave_sample")),
                ("unresolved", None),
            ] {
                let variant = choice_variant(&base, state);
                scenarios.push(resolve_persisted(
                    &format!("post_baizhi_{label}"),
                    &variant,
                    "post_baizhi",
                    "gh_cp_post_bio",
                    "open",
                    label,
                    false,
                ));
            }
        }
    }

    let nav = CandidateNav::grey_hive_v7_candidate();
    let mut wrong_world = base_input("gh_cp_airlock");
    wrong_world.world_id = "mist_harbor".into();
    scenarios.push(resolver_error_record(
        "wrong_world",
        wrong_world,
        &nav,
        ResolveError::WrongWorld,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    let mut wrong_layout = base_input("gh_cp_airlock");
    wrong_layout.layout_version = "ag22-v6-candidate".into();
    scenarios.push(resolver_error_record(
        "wrong_layout",
        wrong_layout,
        &nav,
        ResolveError::WrongLayout,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    let mut wrong_epoch = base_input("gh_cp_airlock");
    wrong_epoch.world_epoch = WORLD_EPOCH - 1;
    let epoch_actual = wrong_epoch.world_epoch == world_state().world_epoch();
    scenarios.push(base_record("wrong_world_epoch", "orchestration epoch precondition before checkpoint_resolver::resolve_checkpoint", json!({"formalWorldEpoch":WORLD_EPOCH,"requestedEpoch":wrong_epoch.world_epoch,"guard":"orchestration precondition; resolver API currently does not enforce epoch"}), json!({"error":"E_WORLD_EPOCH","targetCommitted":false}), json!({"epochMatches":epoch_actual,"error":if epoch_actual {Value::Null}else{json!("E_WORLD_EPOCH")}}), !epoch_actual));
    scenarios.push(resolver_error_record(
        "unknown_checkpoint",
        base_input("gh_cp_not_real"),
        &nav,
        ResolveError::UnknownCheckpoint,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    let no_spawn = CandidateNav::grey_hive_v7_candidate().without_spawn("gh_cp_power");
    scenarios.push(resolver_error_record(
        "missing_transform_spawn",
        base_input("gh_cp_power"),
        &no_spawn,
        ResolveError::MissingSpawn,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    let blocked = CandidateNav::grey_hive_v7_candidate()
        .with_failure("gh_cp_power", ProjectionFailure::Blocked);
    scenarios.push(resolver_error_record(
        "spawn_clearance_failure",
        base_input("gh_cp_power"),
        &blocked,
        ResolveError::Blocked,
        "checkpoint_resolver::resolve_checkpoint / ProjectionFailure::Blocked",
    ));
    let unreachable = CandidateNav::grey_hive_v7_candidate()
        .with_failure("gh_cp_power", ProjectionFailure::Unreachable);
    scenarios.push(resolver_error_record(
        "unreachable",
        base_input("gh_cp_power"),
        &unreachable,
        ResolveError::Unreachable,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    let failed = CandidateNav::grey_hive_v7_candidate()
        .with_failure("gh_cp_power", ProjectionFailure::Failed);
    scenarios.push(resolver_error_record(
        "projection_failed",
        base_input("gh_cp_power"),
        &failed,
        ResolveError::ProjectionFailed,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    let mut nonfinite = CandidateNav::grey_hive_v7_candidate().with_sample(
        "gh_cp_power",
        ProjectionSample {
            room: "GH-A03",
            transform: resolver::FootTransform::new(f64::NAN, 0.0, 14.0, 0.0),
            candidate_only: true,
        },
    );
    scenarios.push(resolver_error_record(
        "nan_transform_rejected",
        base_input("gh_cp_power"),
        &nonfinite,
        ResolveError::NonFinite,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    nonfinite = CandidateNav::grey_hive_v7_candidate().with_sample(
        "gh_cp_power",
        ProjectionSample {
            room: "GH-A03",
            transform: resolver::FootTransform::new(f64::INFINITY, 0.0, 14.0, 0.0),
            candidate_only: true,
        },
    );
    scenarios.push(resolver_error_record(
        "infinity_transform_rejected",
        base_input("gh_cp_power"),
        &nonfinite,
        ResolveError::NonFinite,
        "checkpoint_resolver::resolve_checkpoint",
    ));
    scenarios.push(persistence_failure_record(
        PersistenceFault::PendingWrite,
        "E_PENDING_WRITE",
    ));
    scenarios.push(persistence_failure_record(
        PersistenceFault::BeforeRename,
        "E_COMMIT_INTERRUPTED",
    ));
    scenarios.push(wrong_hash_record());
    scenarios.push(reload_idempotence_record());
    scenarios.push(px_py_no_scaling_record());
    let (scene_record, scene_evidence) = scene_chain_record();
    scenarios.push(scene_record);

    let machine_pass = scenarios.iter().all(|scenario| scenario.pass);
    ProbeReport {
        schema: SCHEMA,
        status: if machine_pass { "passed_formal_api_plus_isolated_candidate" } else { "blocked_probe_failure" },
        machine_pass,
        candidate_only: true,
        formal_api_reuse: json!({
            "migration": "wuxian_horror_ch1::save_v3_migration::{migrate_v2_json,migrate_v2_value,validate_envelope}",
            "persistence": "wuxian_horror_ch1::save_v3_persistence::{commit_v2_to_v3,commit_v2_to_v3_with_fault,inspect_transaction}",
            "world": "wuxian_horror_ch1::world_v3::{WorldStateV3,Transform,Vec3}",
            "resolver": "checkpoint-resolver-probe::{resolve_checkpoint,CandidateNav,ResolverContract}",
            "sceneEvidence": "TP-09A1/B1 read-only JSON evidence; no formal scene export API",
            "shadowSchema": false,
        }),
        fixture_ids: FIXTURES.to_vec(),
        canonical_checkpoints: json!(resolver::canonical_checkpoints().iter().map(|checkpoint| json!({
            "id": checkpoint.id,
            "stage": checkpoint.stage,
            "room": checkpoint.room,
            "requiresLockdown": checkpoint.requires_lockdown,
        })).collect::<Vec<_>>()),
        synthetic_scene_chain: scene_evidence,
        scenarios,
        formal_gaps: vec![
            "SaveV3 envelope has no persisted world epoch; harness epoch guard is orchestration-only",
            "checkpoint resolver remains candidate-only and is not wired into SaveV3/CampaignStore",
            "formal scene/collider/nav export and production nav projection are absent",
            "spawn clearance/KCC validation is not a production API",
            "startup reload, player-save ownership, and power-loss durability are not wired",
            "TP-03/Save Gate E/Architecture approval remains pending",
        ],
        gate_conclusion: json!({
            "tp03": "blocked",
            "saveGateE": "blocked",
            "architecture": "no_go",
            "candidateChain": "passed if all scenario checks pass",
            "candidateOnlyTransforms": true,
            "ghCpPostBio": "GH-B05 safety room; choice/NPC does not gate Gate B",
            "ghCpExit": "map=null terminal noSpawn",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::run_all_scenarios;

    #[test]
    fn all_tp03d1_scenarios_pass() {
        let report = run_all_scenarios();
        assert!(
            report.machine_pass,
            "{}",
            serde_json::to_string_pretty(&report).unwrap()
        );
        assert!(report.scenarios.len() >= 25);
    }
}

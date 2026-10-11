use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use wuxian_horror_ch1::save_v3_migration::{
    migrate_v2_json, migrate_v2_value, validate_checkpoint_id, validate_envelope,
};

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

fn migrate(v2: &Value, id: &str) -> wuxian_horror_ch1::save_v3_migration::SaveV3Envelope {
    migrate_v2_value(v2, id, Some("fixture-sha256")).unwrap()
}

fn set_no_choice(v2: &mut Value) {
    v2["optional_choice"] = Value::Null;
    v2["decisions"] = json!({});
    v2["flags"] = json!(["hive_power", "hive_lockdown"]);
    for objective in v2["objectives"].as_array_mut().unwrap() {
        if objective["id"] == "hive_choice" {
            objective["done"] = json!(false);
        }
    }
}

#[test]
fn six_fixtures_match_ag20_semantic_mapping() {
    let expected = [
        ("grey_hive_initial", "initial", "gh_cp_airlock", "locked"),
        ("post_power", "post_power", "gh_cp_power", "locked"),
        ("post_gate_a", "gate_a", "gh_cp_central", "locked"),
        ("post_baizhi", "post_baizhi", "gh_cp_post_bio", "open"),
        ("pre_sentinel", "pre_sentinel", "gh_cp_pre_sentinel", "open"),
        ("completed_grey_hive", "result", "gh_cp_exit", "open"),
    ];
    for (id, stage, checkpoint, gate_b) in expected {
        let envelope = migrate(&fixture(id), id);
        assert_eq!(envelope.campaign.stage, stage, "{id}");
        assert_eq!(envelope.world_resume.checkpoint_id, checkpoint, "{id}");
        assert_eq!(envelope.campaign.gate_facts.gate_b, gate_b, "{id}");
        assert!(envelope.world_resume.transform.is_none(), "{id}");
        assert_eq!(
            envelope.source.source_sha256.as_deref(),
            Some("fixture-sha256")
        );
        assert!(validate_envelope(&envelope).is_ok(), "{id}");
    }
}

#[test]
fn four_boundaries_preserve_optional_choice_policy() {
    let mut high = fixture("post_baizhi");
    set_no_choice(&mut high);
    let high_out = migrate(&high, "legacy_high_progress_no_choice");
    assert_eq!(high_out.campaign.gate_facts.gate_b, "open");
    assert_eq!(high_out.world_resume.checkpoint_id, "gh_cp_post_bio");
    assert_eq!(high_out.progression.optional_choice, None);
    assert_eq!(high_out.progression.objectives[3]["done"], false);

    let mut leave = high.clone();
    leave["optional_choice"] = json!("hive_leave_sample");
    leave["decisions"] = json!({"grey_hive":"hive_leave_sample"});
    leave["flags"] = json!(["hive_power", "hive_lockdown", "hive_leave_sample"]);
    leave["objectives"][3]["done"] = json!(true);
    let leave_out = migrate(&leave, "legacy_choice_leave_sample");
    assert_eq!(
        leave_out.progression.optional_choice.as_deref(),
        Some("hive_leave_sample")
    );
    assert_eq!(leave_out.world_resume.checkpoint_id, "gh_cp_post_bio");

    let mut deep = fixture("pre_sentinel");
    set_no_choice(&mut deep);
    let deep_out = migrate(&deep, "legacy_deep_no_choice");
    assert_eq!(deep_out.world_resume.checkpoint_id, "gh_cp_pre_sentinel");
    assert_eq!(deep_out.campaign.gate_facts.gate_b, "open");

    let mut rescue = fixture("completed_grey_hive");
    set_no_choice(&mut rescue);
    rescue["flags"] = json!(["hive_power", "hive_lockdown", "hive_rescue"]);
    rescue["objectives"][3]["done"] = json!(false);
    let rescue_out = migrate(&rescue, "legacy_rescue_no_choice");
    assert_eq!(rescue_out.campaign.mode, "result");
    assert_eq!(rescue_out.world_resume.checkpoint_id, "gh_cp_exit");
    assert_eq!(rescue_out.world_resume.resume_policy, "result_then_hub");
    assert_eq!(rescue_out.progression.optional_choice, None);
}

#[test]
fn three_choice_states_and_gate_b_truth_table_are_explicit() {
    let mut taken = fixture("post_baizhi");
    taken["optional_choice"] = json!("hive_help_baizhi");
    let taken_out = migrate(&taken, "taken");
    assert_eq!(taken_out.campaign.gate_facts.gate_b, "open");

    let mut left = fixture("post_baizhi");
    left["optional_choice"] = json!("hive_leave_sample");
    let left_out = migrate(&left, "left");
    assert_eq!(left_out.campaign.gate_facts.gate_b, "open");

    let mut unresolved = fixture("post_baizhi");
    unresolved["optional_choice"] = Value::Null;
    unresolved["decisions"] = json!({});
    let unresolved_out = migrate(&unresolved, "unresolved");
    assert_eq!(
        unresolved_out.campaign.gate_facts.hive_choice_state,
        "unresolved"
    );
    assert_eq!(unresolved_out.campaign.gate_facts.gate_b, "open");

    let locked_out = migrate(&fixture("post_gate_a"), "locked");
    assert_eq!(locked_out.campaign.gate_facts.gate_b, "locked");
    assert_ne!(locked_out.world_resume.checkpoint_id, "gh_cp_post_bio");
}

#[test]
fn missing_optional_choice_is_legal_and_not_inferred_from_decisions() {
    let mut v2 = fixture("post_baizhi");
    v2.as_object_mut().unwrap().remove("optional_choice");
    v2["decisions"] = json!({"grey_hive":"hive_help_baizhi"});
    let envelope = migrate(&v2, "missing-choice");
    assert_eq!(envelope.progression.optional_choice, None);
    assert_eq!(envelope.campaign.gate_facts.hive_choice_state, "unresolved");
    assert_eq!(envelope.campaign.gate_facts.gate_b, "open");
}

#[test]
fn result_and_transform_policy_are_preserved_without_coordinate_scaling() {
    let mut v2 = fixture("completed_grey_hive");
    v2["optional_choice"] = Value::Null;
    let envelope = migrate(&v2, "result-no-choice");
    assert_eq!(envelope.campaign.mode, "result");
    assert!(envelope.campaign.source_map_was_null);
    assert_eq!(envelope.world_resume.checkpoint_id, "gh_cp_exit");
    assert_eq!(envelope.world_resume.resume_policy, "result_then_hub");
    assert!(envelope.world_resume.transform.is_none());
    assert_eq!(envelope.progression.run_scrap, json!(40));
    assert_eq!(envelope.progression.enemies_defeated, json!(1));
}

#[test]
fn invalid_inputs_and_checkpoint_have_stable_codes() {
    assert_eq!(
        migrate_v2_json(b"{bad", "bad", None).unwrap_err().code(),
        "E_JSON_INVALID"
    );
    let mut version = fixture("post_gate_a");
    version["version"] = json!(99);
    assert_eq!(
        migrate_v2_value(&version, "version", None)
            .unwrap_err()
            .code(),
        "E_V2_VERSION"
    );
    let mut world = fixture("post_gate_a");
    world["current_world_id"] = json!("not_grey_hive");
    assert_eq!(
        migrate_v2_value(&world, "world", None).unwrap_err().code(),
        "E_V2_WORLD"
    );
    let mut contradiction = fixture("post_baizhi");
    contradiction["flags"] = json!(["hive_power"]);
    assert_eq!(
        migrate_v2_value(&contradiction, "contradiction", None)
            .unwrap_err()
            .code(),
        "E_V2_MAINLINE_CONTRADICTION:hive_lockdown"
    );
    let mut choice = fixture("post_baizhi");
    choice["optional_choice"] = json!("invented");
    assert_eq!(
        migrate_v2_value(&choice, "choice", None)
            .unwrap_err()
            .code(),
        "E_V2_CHOICE"
    );
    assert_eq!(
        validate_checkpoint_id("gh_cp_not_real").unwrap_err().code(),
        "E_V3_CHECKPOINT"
    );
}

#[test]
fn serde_roundtrip_is_deterministic_and_idempotent() {
    let envelope = migrate(&fixture("pre_sentinel"), "roundtrip");
    let encoded = serde_json::to_vec(&envelope).unwrap();
    let decoded: wuxian_horror_ch1::save_v3_migration::SaveV3Envelope =
        serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, envelope);
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), encoded);
    assert_eq!(decoded.save.schema_version(), 3);
}

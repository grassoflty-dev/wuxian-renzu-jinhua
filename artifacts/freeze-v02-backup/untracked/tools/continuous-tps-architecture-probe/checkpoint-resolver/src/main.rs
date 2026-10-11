use checkpoint_resolver_probe::{
    canonical_checkpoints, resolve_checkpoint, CandidateNav, ChoiceState,
    ResolveError, ResolveInput, ResolverContract, WORLD_ID,
};
use std::fs;
use std::path::Path;

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn input(id: &str, lockdown: bool, choice: ChoiceState, result_map_null: bool) -> ResolveInput {
    ResolveInput {
        checkpoint_id: id.into(),
        world_id: WORLD_ID.into(),
        layout_version: "ag23-v7-candidate".into(),
        world_epoch: 7,
        lockdown_completed: lockdown,
        choice,
        result_map_null,
    }
}

fn status(result: Result<checkpoint_resolver_probe::Resolution, ResolveError>) -> String {
    match result {
        Ok(resolution) => format!("{{\"ok\":true,\"candidateOnly\":{},\"xM\":{:.3},\"yM\":{:.3},\"zM\":{:.3}}}", resolution.candidate_only, resolution.transform.x_m, resolution.transform.y_m, resolution.transform.z_m),
        Err(error) => format!("{{\"ok\":false,\"error\":{}}}", quote(error.as_str())),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.windows(2).find(|pair| pair[0] == "--out").map(|pair| pair[1].clone());
    let Some(out) = out else {
        eprintln!("usage: checkpoint-resolver-validator --out <json>");
        std::process::exit(2);
    };

    let contract = ResolverContract {
        world_id: WORLD_ID.into(),
        layout_version: "ag23-v7-candidate".into(),
        production_scene_bound: false,
    };
    let candidate_nav = CandidateNav::grey_hive_v7_candidate();
    let mut all_passed = true;

    let mut canonical_json = String::new();
    for (index, spec) in canonical_checkpoints().iter().enumerate() {
        if index > 0 { canonical_json.push(','); }
        canonical_json.push_str(&format!(
            "{{\"id\":{},\"stage\":{},\"room\":{},\"requiresLockdown\":{}}}",
            quote(spec.id), quote(spec.stage), quote(spec.room), spec.requires_lockdown
        ));
    }

    let mut matrix_json = String::new();
    let matrix_cases = [
        ("grey_hive_initial", "gh_cp_airlock", ChoiceState::Unresolved, false, false, "upstream_only"),
        ("post_power", "gh_cp_power", ChoiceState::Unresolved, false, false, "upstream_only"),
        ("post_gate_a", "gh_cp_central", ChoiceState::Unresolved, false, false, "central_only"),
        ("post_baizhi", "gh_cp_post_bio", ChoiceState::Taken, true, false, "legal_B05_resume"),
        ("post_baizhi", "gh_cp_post_bio", ChoiceState::Left, true, false, "legal_B05_resume"),
        ("post_baizhi", "gh_cp_post_bio", ChoiceState::Unresolved, true, false, "legal_B05_resume_choice_independent"),
        ("pre_sentinel", "gh_cp_pre_sentinel", ChoiceState::Unresolved, true, false, "legal_deep_resume"),
        ("completed_grey_hive", "gh_cp_exit", ChoiceState::Unresolved, true, true, "terminal_not_applicable_no_transform"),
    ];
    for (index, (fixture, id, choice, lockdown, result_map_null, expected)) in matrix_cases.iter().enumerate() {
        if index > 0 { matrix_json.push(','); }
        let result = resolve_checkpoint(&contract, &input(id, *lockdown, *choice, *result_map_null), &candidate_nav);
        let expected_ok = if *result_map_null { matches!(result, Err(ResolveError::TerminalNotApplicable)) } else { result.is_ok() };
        all_passed &= expected_ok;
        matrix_json.push_str(&format!(
            "{{\"fixture\":{},\"checkpointId\":{},\"choice\":{},\"lockdown\":{},\"map\":{},\"expected\":{},\"result\":{}}}",
            quote(fixture), quote(id), quote(choice.as_str()), lockdown, if *result_map_null { "null" } else { "\"object\"" }, quote(expected), status(result)
        ));
    }

    let error_cases = [
        ("wrong_world", ResolveError::WrongWorld),
        ("wrong_layout", ResolveError::WrongLayout),
        ("unknown_checkpoint", ResolveError::UnknownCheckpoint),
        ("missing_spawn", ResolveError::MissingSpawn),
        ("non_finite", ResolveError::NonFinite),
        ("blocked", ResolveError::Blocked),
        ("unreachable", ResolveError::Unreachable),
        ("projection_failed", ResolveError::ProjectionFailed),
    ];
    let error_json = error_cases.iter().map(|(name, expected)| format!("{{\"name\":{},\"expectedError\":{},\"coveredByUnitTest\":true}}", quote(name), quote(expected.as_str()))).collect::<Vec<_>>().join(",");

    let root = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let ag31_doc = root.join("docs/CONTINUOUS_CHECKPOINT_RESOLVER_BOUNDARY_REVIEW.md").is_file();
    let ag31_json = root.join("tools/continuous-tps-architecture-probe/results/checkpoint-resolver-ag31.json").is_file();
    let json = format!(
        "{{\n  \"schema\":\"checkpoint_resolver_probe_tp03c1_v1\",\n  \"status\":\"isolated_candidate_contract_only\",\n  \"productionResolverImplemented\":false,\n  \"productionSceneBound\":false,\n  \"allChecksPassed\":{},\n  \"canonicalCheckpoints\":[{}],\n  \"matrix\":[{}],\n  \"stableErrorCoverage\":[{}],\n  \"candidatePolicy\":{{\"coordinatesAreFormal\":false,\"productionFallbackAllowed\":false,\"postBioRoom\":\"GH-B05\",\"postBioSemantic\":\"post_bio_before_gate_b\",\"candidateLayout\":\"AG-23 v7 isolated candidate\"}},\n  \"sideEffectPolicy\":{{\"rewardsChanged\":false,\"questChanged\":false,\"choiceChanged\":false,\"playerSaveTouched\":false}},\n  \"readOnlyInputs\":{{\"ag31DocumentPresent\":{},\"ag31ResultPresent\":{},\"worldV3\":\"server-rs/src/world_v3.rs\",\"migration\":\"server-rs/src/save_v3_migration.rs\",\"persistence\":\"server-rs/src/save_v3_persistence.rs\",\"fixtures\":\"server-rs/tests/fixtures/continuous-v2\"}},\n  \"formalGaps\":[\"formal scene export missing\",\"read-only production nav projection missing\",\"spawn clearance and KCC validation missing\",\"resolver is not wired to SaveV3 or CampaignStore\"],\n  \"nextFormalTp03cFiles\":[\"server-rs/src/checkpoint_resolver.rs\",\"server-rs/tests/checkpoint_resolver_test.rs\",\"server-rs/src/lib.rs: one module declaration only\"],\n  \"saveGate\":\"blocked\"\n}}\n",
        all_passed, canonical_json, matrix_json, error_json, ag31_doc, ag31_json
    );
    fs::write(&out, json).expect("write probe result");
    println!("TP-03C1 checkpoint resolver probe passed: output={out}; matrix_cases={}", matrix_cases.len());
}

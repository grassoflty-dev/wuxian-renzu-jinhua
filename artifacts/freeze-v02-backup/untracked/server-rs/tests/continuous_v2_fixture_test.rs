use serde_json::{json, Value};
use std::{
    collections::VecDeque,
    env, fs,
    path::{Path, PathBuf},
};
use wuxian_horror_ch1::campaign::CampaignState;
use wuxian_horror_ch1::{campaign, campaign_store::CampaignStore};

const FIXTURE_ENV: &str = "GENERATE_CONTINUOUS_V2_FIXTURES";
const AG02_LAYOUT_V2: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tools/continuous-tps-architecture-probe/layout/layout-v2.json"
));
const SCENARIOS: [&str; 6] = [
    "grey_hive_initial",
    "post_power",
    "post_gate_a",
    "post_baizhi",
    "pre_sentinel",
    "completed_grey_hive",
];

fn act(state: &mut CampaignState, action: &str, args: Value) -> Value {
    campaign::act(state, action, &args)
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/continuous-v2")
}

fn map(v: &Value) -> &Value {
    &v["map"]
}

fn position_of(v: &Value, collection: &str, id: &str) -> (i32, i32) {
    v["map"][collection]
        .as_array()
        .unwrap_or_else(|| panic!("{collection} is not an array"))
        .iter()
        .find(|item| item["id"].as_str() == Some(id))
        .map(|item| {
            (
                item["x"].as_i64().unwrap() as i32,
                item["y"].as_i64().unwrap() as i32,
            )
        })
        .unwrap_or_else(|| panic!("missing {collection}:{id}"))
}

fn point_position(v: &Value, id: &str) -> (i32, i32) {
    position_of(v, "points", id)
}

fn enemy_position(v: &Value, id: &str) -> (i32, i32, bool) {
    v["map"]["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"].as_str() == Some(id))
        .map(|item| {
            (
                item["x"].as_i64().unwrap() as i32,
                item["y"].as_i64().unwrap() as i32,
                item["alive"].as_bool().unwrap(),
            )
        })
        .unwrap()
}

fn walkable(v: &Value, x: i32, y: i32) -> bool {
    let m = map(v);
    let w = m["w"].as_i64().unwrap() as i32;
    let h = m["h"].as_i64().unwrap() as i32;
    if x < 0 || y < 0 || x >= w || y >= h {
        return false;
    }
    m["tiles"].as_array().unwrap()[y as usize]
        .as_str()
        .unwrap()
        .as_bytes()[x as usize]
        == b'.'
}

fn bfs(v: &Value, target: (i32, i32)) -> Vec<(i32, i32)> {
    let m = map(v);
    let start = (
        m["px"].as_i64().unwrap() as i32,
        m["py"].as_i64().unwrap() as i32,
    );
    if start == target {
        return Vec::new();
    }
    assert!(
        walkable(v, target.0, target.1),
        "target {target:?} is not floor"
    );
    let w = m["w"].as_i64().unwrap() as i32;
    let h = m["h"].as_i64().unwrap() as i32;
    let mut previous = vec![vec![None::<(i32, i32)>; w as usize]; h as usize];
    let mut queue = VecDeque::new();
    previous[start.1 as usize][start.0 as usize] = Some(start);
    queue.push_back(start);
    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == target {
            break;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (x + dx, y + dy);
            if next.0 < 0
                || next.1 < 0
                || next.0 >= w
                || next.1 >= h
                || !walkable(v, next.0, next.1)
            {
                continue;
            }
            if previous[next.1 as usize][next.0 as usize].is_none() {
                previous[next.1 as usize][next.0 as usize] = Some((x, y));
                queue.push_back(next);
            }
        }
    }
    assert!(
        previous[target.1 as usize][target.0 as usize].is_some(),
        "no route from {start:?} to {target:?}"
    );
    let mut result = Vec::new();
    let mut cursor = target;
    while cursor != start {
        result.push(cursor);
        cursor = previous[cursor.1 as usize][cursor.0 as usize].unwrap();
    }
    result.reverse();
    result
}

fn walk_to(state: &mut CampaignState, target: (i32, i32)) {
    for _ in 0..400 {
        let current = campaign::view(state);
        let pos = (
            current["map"]["px"].as_i64().unwrap() as i32,
            current["map"]["py"].as_i64().unwrap() as i32,
        );
        if pos == target {
            return;
        }
        let next = bfs(&current, target)[0];
        let reply = act(
            state,
            "move",
            json!({"dx": next.0 - pos.0, "dy": next.1 - pos.1}),
        );
        assert_eq!(
            reply["mode"], "world",
            "movement failed: {}",
            reply["last_event"]
        );
        let after = (
            reply["map"]["px"].as_i64().unwrap() as i32,
            reply["map"]["py"].as_i64().unwrap() as i32,
        );
        assert_eq!(
            after, next,
            "legal BFS move was not applied: {}",
            reply["last_event"]
        );
    }
    panic!("walk_to {target:?} exceeded step budget");
}

fn adjacent_floor(v: &Value, origin: (i32, i32)) -> (i32, i32) {
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let candidate = (origin.0 + dx, origin.1 + dy);
        if walkable(v, candidate.0, candidate.1) {
            return candidate;
        }
    }
    panic!("no floor beside {origin:?}");
}

fn enter_grey(state: &mut CampaignState) {
    let entered = act(state, "enter", json!({"world_id": "grey_hive"}));
    assert_eq!(
        entered["mode"], "world",
        "enter grey_hive: {}",
        entered["last_event"]
    );
}

fn complete_objective(state: &mut CampaignState, id: &str) {
    let target = point_position(&campaign::view(state), id);
    walk_to(state, target);
    let reply = act(state, "interact", json!({"id": id}));
    assert_eq!(
        reply["mode"], "world",
        "interact {id}: {}",
        reply["last_event"]
    );
    assert!(
        reply["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == id && o["done"] == true),
        "objective {id} did not complete"
    );
}

fn kill_final_guard(state: &mut CampaignState) {
    let initial = campaign::view(state);
    let sentinel_id = initial["map"]["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"].as_str().unwrap().contains("sentinel"))
        .and_then(|e| e["id"].as_str())
        .unwrap()
        .to_owned();
    let guard = enemy_position(&initial, &sentinel_id);
    walk_to(state, adjacent_floor(&initial, (guard.0, guard.1)));
    let shield = act(state, "skill", json!({"slot": 1}));
    assert_eq!(shield["mode"], "world");
    for _ in 0..12 {
        let snapshot = campaign::view(state);
        if !enemy_position(&snapshot, &sentinel_id).2 {
            return;
        }
        let px = snapshot["map"]["px"].as_i64().unwrap() as i32;
        let py = snapshot["map"]["py"].as_i64().unwrap() as i32;
        let (ex, ey, _) = enemy_position(&snapshot, &sentinel_id);
        let dx = (ex - px).signum();
        let dy = (ey - py).signum();
        assert!(dx != 0 || dy != 0, "test stood on the final guard");
        let attack = act(state, "attack", json!({"dx": dx, "dy": dy}));
        assert_eq!(
            attack["mode"], "world",
            "final guard fight died early: {}",
            attack["last_event"]
        );
        act(state, "tick", json!({}));
        act(state, "tick", json!({}));
    }
    panic!("final sentinel survived the combat loop");
}

fn complete_baizhi_choice(state: &mut CampaignState) {
    let npc = position_of(&campaign::view(state), "npcs", "hive_baizhi");
    walk_to(state, npc);
    let dialog = act(state, "interact", json!({"id": "hive_baizhi"}));
    assert_eq!(dialog["mode"], "dialog");
    let chosen = act(state, "choice", json!({"option_id": "hive_help_baizhi"}));
    assert_eq!(chosen["mode"], "world");
    assert_eq!(chosen["decisions"]["grey_hive"], "hive_help_baizhi");
}

fn build_scenario(name: &str) -> CampaignState {
    let mut state = CampaignState::default();
    match name {
        "grey_hive_initial" => enter_grey(&mut state),
        "post_power" => {
            enter_grey(&mut state);
            complete_objective(&mut state, "hive_power");
        }
        "post_gate_a" => {
            enter_grey(&mut state);
            complete_objective(&mut state, "hive_power");
            // The old v2 map has a single walkable opening at (17, 7). Walk
            // one cell beyond it so this fixture proves passage, not proximity.
            walk_to(&mut state, (17, 8));
        }
        "post_baizhi" => {
            enter_grey(&mut state);
            complete_objective(&mut state, "hive_power");
            complete_objective(&mut state, "hive_lockdown");
            complete_baizhi_choice(&mut state);
        }
        "pre_sentinel" => {
            enter_grey(&mut state);
            complete_objective(&mut state, "hive_power");
            complete_objective(&mut state, "hive_lockdown");
            complete_baizhi_choice(&mut state);
            let snapshot = campaign::view(&state);
            let sentinel = enemy_position(&snapshot, "hive_sentinel");
            walk_to(
                &mut state,
                adjacent_floor(&snapshot, (sentinel.0, sentinel.1)),
            );
        }
        "completed_grey_hive" => {
            enter_grey(&mut state);
            complete_objective(&mut state, "hive_power");
            complete_objective(&mut state, "hive_lockdown");
            kill_final_guard(&mut state);
            complete_objective(&mut state, "hive_rescue");
            complete_baizhi_choice(&mut state);
            let exit = position_of(&campaign::view(&state), "portals", "exit");
            walk_to(&mut state, exit);
            let result = act(&mut state, "extract", json!({}));
            assert_eq!(
                result["mode"], "result",
                "extract: {}",
                result["last_event"]
            );
        }
        other => panic!("unknown fixture scenario {other}"),
    }
    state
}

fn write_fixtures() {
    fs::create_dir_all(fixture_dir()).unwrap();
    for name in SCENARIOS {
        let state = build_scenario(name);
        let bytes = serde_json::to_vec_pretty(&state).unwrap();
        fs::write(fixture_dir().join(format!("{name}.json")), bytes).unwrap();
    }
}

fn objective_done(root: &Value, id: &str) -> bool {
    root["objectives"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == id && item["done"] == true)
}

fn flag_present(root: &Value, id: &str) -> bool {
    root["flags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|flag| flag.as_str() == Some(id))
}

fn gate_locked(root: &Value, id: &str) -> bool {
    root["map"]["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|gate| gate["id"] == id)
        .unwrap()["locked"]
        .as_bool()
        .unwrap()
}

fn assert_semantics(name: &str, root: &Value) {
    assert_eq!(root["version"], 2);
    assert_eq!(root["title"], "无限人族进化");
    assert_eq!(root["current_world_id"], "grey_hive");
    let hp = root["player"]["hp"].as_i64().unwrap();
    let max_hp = root["player"]["max_hp"].as_i64().unwrap();
    assert!(hp > 0 && hp <= max_hp);
    match name {
        "grey_hive_initial" => {
            assert_eq!(root["mode"], "world");
            assert!(!objective_done(root, "hive_power"));
            assert!(!flag_present(root, "hive_power"));
            assert!(gate_locked(root, "hive_gate_a"));
        }
        "post_power" => {
            assert_eq!(root["mode"], "world");
            assert!(objective_done(root, "hive_power"));
            assert!(flag_present(root, "hive_power"));
            assert!(!gate_locked(root, "hive_gate_a"));
            assert!(gate_locked(root, "hive_gate_b"));
        }
        "post_gate_a" => {
            assert_eq!(root["mode"], "world");
            assert!(objective_done(root, "hive_power"));
            assert!(!gate_locked(root, "hive_gate_a"));
            assert_eq!(root["map"]["px"], 17);
            assert_eq!(root["map"]["py"], 8);
            assert_eq!(
                root["map"]["tiles"][7]
                    .as_str()
                    .unwrap()
                    .matches('.')
                    .count(),
                1
            );
        }
        "post_baizhi" => {
            assert_eq!(root["mode"], "world");
            assert!(objective_done(root, "hive_power"));
            assert!(objective_done(root, "hive_lockdown"));
            assert!(objective_done(root, "hive_choice"));
            assert!(flag_present(root, "hive_help_baizhi"));
            assert_eq!(root["decisions"]["grey_hive"], "hive_help_baizhi");
        }
        "pre_sentinel" => {
            assert_eq!(root["mode"], "world");
            assert!(objective_done(root, "hive_power"));
            assert!(objective_done(root, "hive_lockdown"));
            assert!(objective_done(root, "hive_choice"));
            assert_eq!(
                root["map"]["enemies"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|enemy| enemy["id"] == "hive_sentinel")
                    .unwrap()["alive"],
                true
            );
            assert!(!objective_done(root, "hive_rescue"));
        }
        "completed_grey_hive" => {
            assert_eq!(root["mode"], "result");
            assert!(root["map"].is_null());
            assert!(root["result"].is_object());
            assert!(root["result"]["reward"].as_i64().unwrap() > 0);
            assert_eq!(root["result"]["world_name"], "灰巢设施");
        }
        _ => unreachable!(),
    }
}

fn assert_canonical_checkpoint_mapping(name: &str) {
    let expected = match name {
        "grey_hive_initial" => Some("gh_cp_airlock"),
        "post_power" => Some("gh_cp_power"),
        "post_gate_a" => Some("gh_cp_central"),
        "post_baizhi" => Some("gh_cp_post_bio"),
        "pre_sentinel" => Some("gh_cp_pre_sentinel"),
        "completed_grey_hive" => None,
        _ => unreachable!(),
    };
    let Some(id) = expected else { return };
    let layout: Value = serde_json::from_str(AG02_LAYOUT_V2).unwrap();
    assert!(
        layout["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["type"] == "checkpoint" && entity["id"] == id),
        "canonical checkpoint {id} is absent from AG-02 layout"
    );
}

#[test]
fn continuous_v2_fixtures_load_and_roundtrip() {
    if env::var_os(FIXTURE_ENV).is_some() {
        write_fixtures();
    }
    let temp_root = env::temp_dir().join(format!(
        "yujie-continuous-v2-fixtures-{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_root).unwrap();
    for name in SCENARIOS {
        let fixture_path = fixture_dir().join(format!("{name}.json"));
        assert!(
            fixture_path.is_file(),
            "missing {fixture_path:?}; set {FIXTURE_ENV}=1 once to generate fixtures"
        );
        let bytes = fs::read(&fixture_path).unwrap();
        let root: Value = serde_json::from_slice(&bytes).unwrap();
        assert_canonical_checkpoint_mapping(name);
        assert_semantics(name, &root);
        let save_path = temp_root.join(format!("{name}.json"));
        fs::write(&save_path, &bytes).unwrap();
        let mut store = CampaignStore::load(save_path.clone());
        let loaded = store.view();
        assert!(
            loaded["save_warning"].is_null(),
            "{name}: {:?}",
            loaded["save_warning"]
        );
        assert_eq!(loaded["read_only_recovery_required"], false);
        assert_eq!(loaded["save_path"], save_path.to_string_lossy().to_string());
        store.flush().unwrap();
        let reloaded = CampaignStore::load(save_path);
        let roundtrip = reloaded.view();
        assert!(
            roundtrip["save_warning"].is_null(),
            "{name} roundtrip: {:?}",
            roundtrip["save_warning"]
        );
        assert_semantics(name, &serde_json::to_value(&reloaded.state).unwrap());
    }
    assert!(
        !temp_root.join("campaign-v2.json").exists(),
        "fixture test must not touch the real campaign save"
    );
}

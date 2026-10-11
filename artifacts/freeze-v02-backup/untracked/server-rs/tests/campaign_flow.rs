use serde_json::{json, Value};
use std::collections::VecDeque;
use wuxian_horror_ch1::campaign::{self, CampaignState};

fn act(state: &mut CampaignState, action: &str, args: Value) -> Value {
    campaign::act(state, action, &args)
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
    let adjacent = adjacent_floor(&initial, (guard.0, guard.1));
    walk_to(state, adjacent);
    let shield = act(state, "skill", json!({"slot": 1}));
    assert_eq!(shield["mode"], "world");
    for _ in 0..12 {
        let snapshot = campaign::view(state);
        let (_, _, still_alive) = enemy_position(&snapshot, &sentinel_id);
        if !still_alive {
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

fn complete_world(state: &mut CampaignState, id: &str, choice: Option<&str>) -> Value {
    let entered = act(state, "enter", json!({"world_id": id}));
    assert_eq!(
        entered["mode"], "world",
        "enter {id}: {}",
        entered["last_event"]
    );
    let main_ids: Vec<String> = entered["objectives"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|objective| !objective["optional"].as_bool().unwrap())
        .map(|objective| objective["id"].as_str().unwrap().to_owned())
        .collect();
    for (index, objective_id) in main_ids.iter().enumerate() {
        if index == 2 {
            kill_final_guard(state);
        }
        let target = point_position(&campaign::view(state), objective_id);
        walk_to(state, target);
        let reply = act(state, "interact", json!({"id": objective_id}));
        assert_eq!(
            reply["mode"], "world",
            "interact {objective_id}: {}",
            reply["last_event"]
        );
        let done = reply["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"].as_str() == Some(objective_id))
            .unwrap()["done"]
            .as_bool()
            .unwrap();
        assert!(done, "objective {objective_id} did not complete");
    }
    if let Some(option) = choice {
        let npc_id = match id {
            "grey_hive" => "hive_baizhi",
            "mist_harbor" => "harbor_lucen",
            "clockworks" => "clock_mengque",
            _ => unreachable!(),
        };
        let npc = position_of(&campaign::view(state), "npcs", npc_id);
        walk_to(state, npc);
        let dialog = act(state, "interact", json!({"id": npc_id}));
        assert_eq!(dialog["mode"], "dialog");
        let chosen = act(state, "choice", json!({"option_id": option}));
        assert_eq!(chosen["mode"], "world");
        assert_eq!(chosen["decisions"][id], option);
    }
    let exit = position_of(&campaign::view(state), "portals", "exit");
    walk_to(state, exit);
    let extracted = act(state, "extract", json!({}));
    assert_eq!(
        extracted["mode"], "result",
        "extract {id}: {}",
        extracted["last_event"]
    );
    extracted
}

#[test]
fn three_worlds_have_distinct_reachable_layouts_and_mechanics() {
    let mut state = CampaignState::default();
    let mut map_signatures = Vec::new();
    for id in ["grey_hive", "mist_harbor", "clockworks"] {
        let v = act(&mut state, "enter", json!({"world_id": id}));
        let m = map(&v);
        assert_eq!(m["w"], 34);
        assert_eq!(m["h"], 24);
        assert_eq!(m["tiles"].as_array().unwrap().len(), 24);
        assert!(m["tiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row.as_str().unwrap().len() == 34));
        assert!((6..=10).contains(&m["props"].as_array().unwrap().len()));
        assert_eq!(
            m["points"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p["kind"] == "supply")
                .count(),
            2
        );
        assert!(m["enemies"].as_array().unwrap().iter().all(|e| {
            let dx = e["x"].as_i64().unwrap() as i32 - m["px"].as_i64().unwrap() as i32;
            let dy = e["y"].as_i64().unwrap() as i32 - m["py"].as_i64().unwrap() as i32;
            dx * dx + dy * dy > 9
        }));
        let first = m["points"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["kind"] == "objective")
            .unwrap();
        let first_distance = (first["x"].as_i64().unwrap() as i32 - 3).abs()
            + (first["y"].as_i64().unwrap() as i32 - 3).abs();
        assert!(
            (3..=6).contains(&first_distance),
            "{id} first terminal is {first_distance} steps away"
        );
        map_signatures.push(m["tiles"].clone());
        match id {
            "mist_harbor" => {
                assert_eq!(m["tiles"][19].as_str().unwrap().as_bytes()[23], b'~');
                assert_eq!(m["tiles"][1].as_str().unwrap().as_bytes()[1], b'~');
                assert_eq!(position_of(&v, "gates", "harbor_gate_a"), (17, 7));
                assert_eq!(position_of(&v, "gates", "harbor_gate_b"), (23, 16));
                assert_eq!(point_position(&v, "harbor_beacon_b"), (25, 11));
                assert_eq!(position_of(&v, "npcs", "harbor_lucen"), (24, 13));
                assert_eq!(position_of(&v, "portals", "exit"), (29, 21));
            }
            "clockworks" => {
                assert_eq!(position_of(&v, "gates", "clock_gate_a"), (8, 7));
                assert_eq!(position_of(&v, "gates", "clock_gate_b"), (26, 16));
                assert_eq!(m["tiles"][13].as_str().unwrap().as_bytes()[18], b'.');
                assert_eq!(m["tiles"][12].as_str().unwrap().as_bytes()[18], b'#');
                assert_eq!(m["tiles"][4].as_str().unwrap().as_bytes()[6], b'.');
                assert_eq!(m["tiles"][11].as_str().unwrap().as_bytes()[24], b'.');
                assert_eq!(
                    m["zones"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|z| z["active"] == true)
                        .count(),
                    2
                );
            }
            _ => {}
        }
        act(&mut state, "retreat", json!({}));
    }
    assert_ne!(map_signatures[0], map_signatures[1]);
    assert_ne!(map_signatures[1], map_signatures[2]);
}

#[test]
fn three_world_real_paths_extract_and_persistent_choices_change_finale() {
    let mut state = CampaignState::default();
    let grey = complete_world(&mut state, "grey_hive", Some("hive_help_baizhi"));
    assert!(grey["result"]["reward"].as_i64().unwrap() > 0);
    assert_eq!(grey["result"]["side_story_tokens"], 2);
    assert_eq!(grey["player"]["side_story_tokens"], 2);
    act(&mut state, "continue", json!({}));
    assert_eq!(
        campaign::view(&state)["decisions"]["grey_hive"],
        "hive_help_baizhi"
    );

    let harbor = complete_world(&mut state, "mist_harbor", Some("harbor_save_fuel"));
    assert!(harbor["result"]["reward"].as_i64().unwrap() > 0);
    act(&mut state, "continue", json!({}));
    let mid = campaign::view(&state);
    assert_eq!(mid["decisions"]["grey_hive"], "hive_help_baizhi");
    assert_eq!(mid["decisions"]["mist_harbor"], "harbor_save_fuel");

    let final_result = complete_world(&mut state, "clockworks", None);
    assert_eq!(final_result["result"]["all_complete"], true);
    let summary = final_result["result"]["summary"].as_str().unwrap();
    assert!(
        summary.contains("白芷"),
        "finale omitted the grey-hive choice: {summary}"
    );
    assert!(
        summary.contains("陆岑"),
        "finale omitted the mist-harbor choice: {summary}"
    );
    assert!(summary.contains("救援"));
}

#[test]
fn supply_is_one_shot_and_revisit_has_no_meta_reward() {
    let mut state = CampaignState::default();
    let entered = act(&mut state, "enter", json!({"world_id": "grey_hive"}));
    let supply = entered["map"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kind"] == "supply")
        .unwrap();
    let supply_id = supply["id"].as_str().unwrap().to_owned();
    let supply_pos = (
        supply["x"].as_i64().unwrap() as i32,
        supply["y"].as_i64().unwrap() as i32,
    );
    walk_to(&mut state, supply_pos);
    let collected = act(&mut state, "interact", json!({"id": supply_id}));
    assert_eq!(collected["effect"]["kind"], "supply");
    let scrap_after = collected["player"]["reward_points"].as_i64().unwrap();
    let medkits_after = collected["player"]["medkits"].as_i64().unwrap();
    let repeated = act(&mut state, "interact", json!({"id": supply_id}));
    assert_eq!(repeated["player"]["reward_points"], scrap_after);
    assert_eq!(repeated["player"]["medkits"], medkits_after);

    // 放弃本局后再次进入同一世界：补给点会随新地图重置可见状态，但
    // run_reward_allowed 已被持久 meta 封住，不能用回访重复刷资源。
    act(&mut state, "retreat", json!({}));
    let replay_entry = act(&mut state, "enter", json!({"world_id": "grey_hive"}));
    let replay_supply = replay_entry["map"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_str() == Some(supply_id.as_str()))
        .unwrap();
    let replay_pos = (
        replay_supply["x"].as_i64().unwrap() as i32,
        replay_supply["y"].as_i64().unwrap() as i32,
    );
    walk_to(&mut state, replay_pos);
    let before_replay_collect = campaign::view(&state);
    let replay_collected = act(&mut state, "interact", json!({"id": supply_id}));
    assert_eq!(
        replay_collected["player"]["reward_points"],
        before_replay_collect["player"]["reward_points"]
    );
    assert_eq!(
        replay_collected["player"]["medkits"],
        before_replay_collect["player"]["medkits"]
    );
}

#[test]
fn exchange_uses_points_and_vouchers_without_repeat_purchase() {
    let mut state = CampaignState::default();
    let initial = campaign::view(&state);
    assert!(initial["player"]["level"].is_null());
    assert!(initial["exchanges"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["category"].is_string()
            && item["grade"].is_string()
            && item["owned"].is_boolean()));

    let purchased = act(&mut state, "exchange", json!({"id": "reinforced_suit"}));
    assert_eq!(purchased["player"]["reward_points"], 6);
    assert_eq!(
        purchased["exchanges"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "reinforced_suit")
            .unwrap()["owned"],
        true
    );
    let duplicate = act(&mut state, "upgrade", json!({"id": "reinforced_suit"}));
    assert!(duplicate["last_event"]
        .as_str()
        .unwrap()
        .contains("重复兑换"));
    let insufficient = act(&mut state, "exchange", json!({"id": "signal_lens"}));
    assert!(insufficient["last_event"]
        .as_str()
        .unwrap()
        .contains("不足"));
}

#[test]
fn replay_clear_does_not_award_first_clear_voucher_again() {
    let mut state = CampaignState::default();
    let first = complete_world(&mut state, "grey_hive", Some("hive_help_baizhi"));
    assert_eq!(first["result"]["side_story_tokens"], 2);
    let first_total = first["player"]["side_story_tokens"].as_i64().unwrap();
    act(&mut state, "continue", json!({}));
    let replay = complete_world(&mut state, "grey_hive", None);
    assert_eq!(replay["result"]["side_story_tokens"], 0);
    assert_eq!(replay["player"]["side_story_tokens"], first_total);
}

#[test]
fn invalid_commands_are_atomic_and_save_round_trip_preserves_view() {
    let mut state = CampaignState::default();
    let entered = act(&mut state, "enter", json!({"world_id": "grey_hive"}));
    let before = campaign::view(&state);
    let bad_enter = act(&mut state, "enter", json!({"world_id": "not_a_world"}));
    assert_eq!(bad_enter["mode"], "world");
    assert_eq!(bad_enter["map"]["world_id"], before["map"]["world_id"]);
    assert_eq!(bad_enter["map"]["px"], before["map"]["px"]);
    assert_eq!(bad_enter["player"], before["player"]);
    assert_eq!(bad_enter["objectives"], before["objectives"]);
    let bad_continue = act(&mut state, "continue", json!({}));
    assert_eq!(bad_continue["mode"], "world");
    assert_eq!(bad_continue["map"], bad_enter["map"]);
    let bad_attack = act(&mut state, "attack", json!({"dx": 0, "dy": 0}));
    assert_eq!(bad_attack["map"]["px"], before["map"]["px"]);
    assert_eq!(bad_attack["map"]["py"], before["map"]["py"]);
    assert_eq!(bad_attack["player"]["energy"], before["player"]["energy"]);
    let encoded = serde_json::to_string(&state).unwrap();
    let restored: CampaignState = serde_json::from_str(&encoded).unwrap();
    let mut expected = campaign::view(&state);
    let restored_view = campaign::view(&restored);
    // last_event is part of the persisted snapshot too; equality verifies private
    // run fields (cooldowns, flags, map state) were not silently dropped.
    assert_eq!(restored_view, expected);
    expected["skills"] = restored_view["skills"].clone();
    assert_eq!(expected["skills"][0]["cooldown"], 0);
    assert_eq!(expected["skills"][0]["cooldown_max"], 3);
    assert!(expected["skills"][0]["ready"].as_bool().unwrap());
    assert_eq!(entered["player"]["hp"], 120);
    assert_eq!(entered["player"]["max_hp"], 120);
}

#[test]
fn enemy_warning_can_be_dodged_and_death_can_restart_without_leaking_to_hub() {
    let mut dodge = CampaignState::default();
    let entered = act(&mut dodge, "enter", json!({"world_id": "grey_hive"}));
    let pursuer = enemy_position(&entered, "hive_pursuer");
    let safe_adjacent = adjacent_floor(&entered, (pursuer.0, pursuer.1));
    walk_to(&mut dodge, safe_adjacent);
    let warning = act(&mut dodge, "tick", json!({}));
    let telegraph = warning["map"]["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|e| {
            e["telegraph"].as_object().map(|t| {
                (
                    e["id"].as_str().unwrap().to_owned(),
                    t["remaining"].as_i64().unwrap(),
                )
            })
        });
    assert!(
        telegraph.is_some(),
        "no visible enemy warning near a chaser"
    );
    assert!((1..=2).contains(&telegraph.unwrap().1));
    let current = campaign::view(&dodge);
    let px = current["map"]["px"].as_i64().unwrap() as i32;
    let py = current["map"]["py"].as_i64().unwrap() as i32;
    let mut moved = false;
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        if walkable(&current, px + dx, py + dy) && (px + dx, py + dy) != (pursuer.0, pursuer.1) {
            act(&mut dodge, "move", json!({"dx": dx, "dy": dy}));
            moved = true;
            break;
        }
    }
    assert!(moved, "could not step away from telegraph");
    let hp = campaign::view(&dodge)["player"]["hp"].as_i64().unwrap();
    act(&mut dodge, "tick", json!({}));
    act(&mut dodge, "tick", json!({}));
    assert_eq!(
        campaign::view(&dodge)["player"]["hp"].as_i64().unwrap(),
        hp,
        "telegraphed chaser hit after dodge"
    );

    let mut death = CampaignState::default();
    let started = act(&mut death, "enter", json!({"world_id": "grey_hive"}));
    let pursuer = enemy_position(&started, "hive_pursuer");
    let adjacent = adjacent_floor(&started, (pursuer.0, pursuer.1));
    walk_to(&mut death, adjacent);
    for _ in 0..80 {
        if campaign::view(&death)["mode"] == "dead" {
            break;
        }
        act(&mut death, "tick", json!({}));
    }
    assert_eq!(campaign::view(&death)["mode"], "dead");
    let restarted = act(&mut death, "restart", json!({}));
    assert_eq!(restarted["mode"], "world");
    assert_eq!(restarted["map"]["world_id"], "grey_hive");
    assert_eq!(restarted["player"]["hp"], restarted["player"]["max_hp"]);
    let trapped_continue = act(&mut death, "continue", json!({}));
    assert_eq!(
        trapped_continue["mode"], "world",
        "continue must not escape a live task"
    );
}

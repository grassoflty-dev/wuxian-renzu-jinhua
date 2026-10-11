//! 回归测试（§8.5/§8.6）：通关旗标映射 + 任务全量覆盖 + 心性六维
//!
//! 覆盖三块最近新增/修改、此前缺测试护栏的功能：
//!   A. worlds::clear_flag —— 世界 id → 真实通关旗标（7 个显式映射 + 回退约定）
//!   B. tasks::TASKS      —— 全部 57 个可进世界（主神空间除外）至少 1 条 Main 主线，且无重复 (world_id, main)
//!   C. state::MindState  —— 六维初值/夹取 + GameState::mind_shift/record_deliberation/
//!                           record_death_persist/promise/fulfill/break_promise
//!
//! 本文件只读 src 的公开 API，不改动 src/**。

use wuxian_horror_ch1::state::GameState;
use wuxian_horror_ch1::tasks::{self, TaskKind, TASKS};
use wuxian_horror_ch1::worlds::{self, clear_flag};

// ---------------------------------------------------------------- A. clear_flag

/// A1：7 个显式映射世界的通关旗标必须与结算写入的真实旗标一致。
#[test]
fn clear_flag_maps_explicit_worlds() {
    let cases: [(&str, &str); 7] = [
        (worlds::WORLD_BIOHAZARD, "bh_cleared"),
        (worlds::WORLD_XINGHE, "xh_brain_down"),
        (worlds::WORLD_TIANTING, "tt_cleared"),
        (worlds::WORLD_YINSE, "ys_cleared"),
        (worlds::WORLD_MORUIYA, "mo_cleared"),
        (worlds::WORLD_POXIAO, "px_cleared"),
        (worlds::WORLD_TIANSHE, "dungeon_cleared"),
    ];
    for (world_id, expect) in cases {
        assert_eq!(
            clear_flag(world_id),
            expect.to_string(),
            "世界 {world_id} 的通关旗标应为 {expect}"
        );
    }
}

/// A2：未显式映射的世界回退 {world_id}_cleared（咒怨 zhuyuan 为典型）。
#[test]
fn clear_flag_falls_back_to_world_id_convention() {
    assert_eq!(clear_flag("zhuyuan"), "zhuyuan_cleared");
    // 再抽两个未映射世界，确认回退不是特例
    assert_eq!(clear_flag("moshi_shoucheng"), "moshi_shoucheng_cleared");
    assert_eq!(clear_flag("juluoji"), "juluoji_cleared");
    // 空/未知 id 也不 panic，仍是约定回退
    assert_eq!(clear_flag(""), "_cleared");
}

/// A3：显式映射的世界不得等于 {world_id}_cleared（防止映射被误删成回退）。
#[test]
fn clear_flag_explicit_worlds_differ_from_fallback() {
    for world_id in [
        worlds::WORLD_BIOHAZARD,
        worlds::WORLD_XINGHE,
        worlds::WORLD_TIANTING,
        worlds::WORLD_YINSE,
        worlds::WORLD_MORUIYA,
        worlds::WORLD_POXIAO,
        worlds::WORLD_TIANSHE,
    ] {
        let fallback = format!("{world_id}_cleared");
        assert_ne!(
            clear_flag(world_id),
            fallback,
            "世界 {world_id} 应使用自定义通关旗标，而非回退 {fallback}"
        );
    }
}

// ---------------------------------------------------------------- B. 任务覆盖

/// B1：all_worlds() 里除主神空间外，每个世界都至少有 1 条 world_id 匹配的 Main 主线任务。
#[test]
fn every_enterable_world_has_a_main_task() {
    let mut missing: Vec<&str> = Vec::new();
    for w in worlds::all_worlds() {
        if w.id == worlds::WORLD_ZHUTIAN {
            continue; // 主神空间 = 枢纽，按设计无任务
        }
        let has_main = TASKS
            .iter()
            .any(|t| t.world_id == w.id && t.kind == TaskKind::Main);
        if !has_main {
            missing.push(w.id);
        }
    }
    assert!(
        missing.is_empty(),
        "以下可进世界缺 Main 主线任务：{:?}",
        missing
    );
}

/// B2：世界总数与有主线任务的世界数对账（58 世界 − 主神空间 = 57）。
#[test]
fn main_task_covers_exactly_the_enterable_worlds() {
    let total = worlds::all_worlds().len();
    let enterable = worlds::all_worlds()
        .iter()
        .filter(|w| w.id != worlds::WORLD_ZHUTIAN)
        .count();
    assert_eq!(total, enterable + 1, "注册表应恰好含 1 个主神空间枢纽世界");

    let mut covered: Vec<&str> = TASKS
        .iter()
        .filter(|t| t.kind == TaskKind::Main)
        .map(|t| t.world_id)
        .collect();
    covered.sort_unstable();
    covered.dedup();
    assert_eq!(
        covered.len(),
        enterable,
        "有 Main 任务的世界数应与可进世界数一致（实际 {} vs 期望 {}）",
        covered.len(),
        enterable
    );

    // 主神空间不得出现在任何任务里
    assert!(
        !TASKS.iter().any(|t| t.world_id == worlds::WORLD_ZHUTIAN),
        "主神空间不应有任务条目"
    );
}

/// B3：不得出现重复的 (world_id, Main) 组合；任务 id 全局唯一。
#[test]
fn no_duplicate_world_main_pairs_and_unique_task_ids() {
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    let mut dup_pairs: Vec<(&str, &str)> = Vec::new();
    for t in TASKS.iter().filter(|t| t.kind == TaskKind::Main) {
        let key = (t.world_id, t.kind.as_str());
        if pairs.contains(&key) {
            dup_pairs.push(key);
        } else {
            pairs.push(key);
        }
    }
    assert!(
        dup_pairs.is_empty(),
        "存在重复的 (world_id, main) 组合：{:?}",
        dup_pairs
    );

    let mut ids: Vec<&str> = TASKS.iter().map(|t| t.id).collect();
    let all_ids = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all_ids, "任务 id 必须全局唯一");
}

/// B4：每条主线的 done_when 必须由该世界 clear_flag 产出的旗标点亮
/// （即 tasks.rs 的通关判定与 worlds::clear_flag 映射不脱节）。
#[test]
fn main_task_done_when_uses_clear_flag() {
    let mut broken: Vec<(&str, &str, String)> = Vec::new();
    for w in worlds::all_worlds() {
        if w.id == worlds::WORLD_ZHUTIAN {
            continue;
        }
        let Some(main) = TASKS
            .iter()
            .find(|t| t.world_id == w.id && t.kind == TaskKind::Main)
        else {
            continue; // B1 已单独断言缺失
        };
        let mut st = GameState::new();
        st.world_id = w.id.to_string();
        assert!(!(main.done_when)(&st), "新局主线 {} 不应已完成", main.id);
        st.set_flag(&clear_flag(w.id));
        if !(main.done_when)(&st) {
            broken.push((w.id, main.id, clear_flag(w.id)));
        }
    }
    assert!(
        broken.is_empty(),
        "以下世界写入 clear_flag 后主线仍未完成（判定与旗标脱节）：{:?}",
        broken
    );
}

/// B5：task_view/current_task 按世界过滤的行为与全量覆盖一致（补护栏）。
#[test]
fn task_view_and_current_task_for_every_world() {
    for w in worlds::all_worlds() {
        let mut st = GameState::new();
        st.world_id = w.id.to_string();
        let view = tasks::task_view(&st);
        if w.id == worlds::WORLD_ZHUTIAN {
            assert!(view.is_empty(), "主神空间任务视图应为空");
            assert_eq!(tasks::current_task(&st), None);
            continue;
        }
        assert!(!view.is_empty(), "世界 {} 任务视图不应为空", w.id);
        assert!(
            view.iter().any(|t| t.kind == "main"),
            "世界 {} 任务视图应含主线",
            w.id
        );
        assert!(
            tasks::current_task(&st).is_some(),
            "世界 {} 新局应有当前主线",
            w.id
        );
    }
}

// ---------------------------------------------------------------- C. 心性六维

/// C1：新局六维全部 = 1（加权 1.0），境界 = 凡心·流转(0)。
#[test]
fn new_state_mind_starts_at_one_and_realm_zero() {
    let st = GameState::new();
    assert_eq!(st.mind.guanzhao, 1, "观照初值");
    assert_eq!(st.mind.sheli, 1, "舍离初值");
    assert_eq!(st.mind.cibei, 1, "慈悲初值");
    assert_eq!(st.mind.linzai, 1, "临在初值");
    assert_eq!(st.mind.jianyi, 1, "坚毅初值");
    assert_eq!(st.mind.guanyi, 1, "贯一初值");
    assert_eq!(st.mind.realm(), 0, "六维全 1 应为凡心·流转");
    assert_eq!(st.mind.realm_name(), "凡心·流转");

    // dims() 展示顺序与中文名固定
    let dims = st.mind.dims();
    assert_eq!(
        dims.map(|(n, _)| n),
        ["观照", "舍离", "慈悲", "临在", "坚毅", "贯一"]
    );
    assert!(dims.iter().all(|(_, v)| *v == 1));
}

/// C2：mind_shift 上下都夹取到 [1, 10]。
#[test]
fn mind_shift_clamps_to_one_through_ten() {
    let mut st = GameState::new();
    st.mind_shift("cibei", 100);
    assert_eq!(st.mind.cibei, 10, "慈悲上夹取到 10");
    st.mind_shift("cibei", 100); // 幂等，不再溢出/回绕
    assert_eq!(st.mind.cibei, 10, "已在上限应保持 10");

    st.mind_shift("cibei", -100);
    assert_eq!(st.mind.cibei, 1, "慈悲下夹取到 1");

    // 逐步累积不越界
    let mut st2 = GameState::new();
    for _ in 0..30 {
        st2.mind_shift("guanzhao", 1);
    }
    assert_eq!(st2.mind.guanzhao, 10);
    for _ in 0..30 {
        st2.mind_shift("guanzhao", -1);
    }
    assert_eq!(st2.mind.guanzhao, 1);

    // 未知维度是 no-op，不得 panic 也不得改动任何维度
    let mut st3 = GameState::new();
    st3.mind_shift("nope", 5);
    assert_eq!(st3.mind.dims().map(|(_, v)| v), [1, 1, 1, 1, 1, 1]);
}

/// C3：高心性点亮境界（加权 ≥3 → 本心，≥5 → 道心初醒）。
#[test]
fn mind_realm_tracks_weighted_score() {
    let mut st = GameState::new();
    // 加权 = 1.0；每维 +1 = +1.0
    st.mind_shift("cibei", 2); // +0.40 → 1.40
    st.mind_shift("jianyi", 2); // +0.40 → 1.80
    st.mind_shift("guanyi", 2); // +0.40 → 2.20
    assert_eq!(st.mind.realm(), 0, "2.2 仍未过 3.0");

    st.mind_shift("guanzhao", 5); // 观照 1→6：+0.75 → 2.95
    assert_eq!(st.mind.realm(), 0, "2.95 仍未过 3.0");
    st.mind_shift("sheli", 1); // +0.15 → 3.10
    assert_eq!(st.mind.realm(), 1, "加权 ≥3.0 应为本心·蒙尘");
    assert_eq!(st.mind.realm_name(), "本心·蒙尘");
}

/// C4：record_deliberation —— 思考时长驱动临在（≥8s +2，≥3s +1，<3s 不变）。
#[test]
fn record_deliberation_drives_linzai() {
    let mut st = GameState::new();
    st.record_deliberation(9000);
    assert_eq!(st.mind.linzai, 3, "9000ms 应使临在 1 → 3（+2）");

    let mut st2 = GameState::new();
    st2.record_deliberation(3000);
    assert_eq!(st2.mind.linzai, 2, "3000ms 应使临在 +1");

    let mut st3 = GameState::new();
    st3.record_deliberation(2999);
    assert_eq!(st3.mind.linzai, 1, "<3s 不加临在");

    // 只影响临在，不污染其它维度
    assert_eq!(
        [
            st.mind.guanzhao,
            st.mind.sheli,
            st.mind.cibei,
            st.mind.jianyi,
            st.mind.guanyi
        ],
        [1, 1, 1, 1, 1]
    );
}

/// C5：record_death_persist —— 死亡后继续（轮回重开）坚毅 +1。
#[test]
fn record_death_persist_drives_jianyi() {
    let mut st = GameState::new();
    st.record_death_persist();
    assert_eq!(st.mind.jianyi, 2);
    for _ in 0..20 {
        st.record_death_persist();
    }
    assert_eq!(st.mind.jianyi, 10, "坚毅同样受 1~10 夹取");
}

/// C6：promise → fulfill 使贯一 +1 且清除 promise_k 旗标。
#[test]
fn fulfill_promise_raises_guanyi_and_clears_flag() {
    let mut st = GameState::new();
    assert!(!st.flag("promise_k"), "新局不应有待兑现承诺");

    st.promise("k");
    assert!(st.flag("promise_k"), "promise(k) 应写入 promise_k 旗标");

    st.fulfill("k");
    assert_eq!(st.mind.guanyi, 2, "兑现承诺贯一 1 → 2");
    assert!(!st.flag("promise_k"), "兑现后 promise_k 旗标应被清除");

    // 重复兑现幂等：旗标已清除，不再加贯一
    st.fulfill("k");
    assert_eq!(st.mind.guanyi, 2, "重复兑现不得重复加贯一");
}

/// C7：未许愿直接兑现 / 违背承诺的边界行为。
#[test]
fn fulfill_and_break_promise_without_promise_are_noops() {
    let mut st = GameState::new();
    st.fulfill("ghost");
    assert_eq!(st.mind.guanyi, 1, "无承诺时兑现不得加贯一");
    st.break_promise("ghost");
    assert_eq!(st.mind.guanyi, 1, "无承诺时违背不得减贯一");

    st.promise("v");
    st.break_promise("v");
    assert_eq!(st.mind.guanyi, 1, "违背承诺 -1 后仍受下夹取保护，不低于 1");
    assert!(!st.flag("promise_v"), "违背后 promise_v 旗标应被清除");
}

/// C8：违背承诺确实扣贯一（心性维度可双向变化，非只增不减）。
#[test]
fn break_promise_lowers_guanyi() {
    let mut st = GameState::new();
    for _ in 0..3 {
        st.promise("a");
        st.fulfill("a");
    }
    assert_eq!(st.mind.guanyi, 4, "三次兑现：1 → 4");

    st.promise("b");
    st.break_promise("b");
    assert_eq!(st.mind.guanyi, 3, "违背一次：4 → 3");
}

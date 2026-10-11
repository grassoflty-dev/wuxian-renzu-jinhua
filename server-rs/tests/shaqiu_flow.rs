//! 《大宇宙时代 · 沙丘魔海 · 坠毁之星》副本 · 集成测试。
//! 世界展示向：以展示沙丘魔海／绿潮异星世界为主，剧情开放、无真相线阴谋指向。
//! 钩子：「绿潮吞没显示器之前，你先看清了它有多美。」
//! 测试内容：
//!   ① shaqiu_maps_reachable —— 4 层地图每行 40 字符、出生点 P=(4,14)，
//!      全部调查点/敌人/角色/战圈/传送门/门禁坐标可行走
//!   ② shaqiu_main_line_boss_finisher —— 主线链：残骸→绿潮战场→母巢→沙丘洞穴，
//!      用「诱水剂」脱水重创终结合缐渴水兽王（BOSS 选择驱动链）→ 取遗泽种子 → 结算
//!   ③ shaqiu_oxy_timer_flag_chain —— 氧气倒计时 flag 链（sq_oxy_1→2→3）降级 + 低氧扣 HP
//! 依赖主线在合并阶段接线（见 tools/design/shaqiu_impl_log.md ★外部依赖）。
use wuxian_horror_ch1::{engine, state::GameState, scenes};
use wuxian_horror_ch1::worlds;

fn world() -> &'static worlds::WorldData {
    worlds::find_world(wuxian_horror_ch1::worlds::WORLD_SHAQIU).expect("沙丘魔海世界已注册（合并阶段）")
}

fn pick(st: &GameState, keyword: &str) -> i32 {
    let scene = scenes::scene(&st.scene_id).expect("scene");
    let visible: Vec<_> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
    for (i, c) in visible.iter().enumerate() {
        if c.label.contains(keyword) {
            return i as i32;
        }
    }
    panic!("scene {} 未找到含「{}」的选项；可见: {:?}",
        st.scene_id, keyword, visible.iter().map(|c| c.label).collect::<Vec<_>>());
}

fn step(st: &mut GameState, keyword: &str) {
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    let idx = pick(st, keyword);
    engine::choose(st, idx, &mut deaths);
    println!("STEP [{keyword}] → {} (hp={} san={} pts={})",
        st.scene_id, st.hp, st.san, st.points);
    assert!(st.san >= 0, "san 越界 scene={}", st.scene_id);
}

fn choose_idx(st: &mut GameState, idx: i32) {
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    engine::choose(st, idx, &mut deaths);
}

fn fresh_combat(st: &mut GameState) { st.hp = 800; st.san = 100; }

/// ① 4 层地图可达性 + 对象坐标可行走 + 出生点
#[test]
fn shaqiu_maps_reachable() {
    let w = world();
    assert_eq!(w.floors.len(), 4, "应为 4 层");
    assert_eq!(w.initial_scene, "sq_00_intro", "落点应为主神空间外的开场场景");
    for (fi, map) in w.floors.iter().enumerate() {
        for (r, row) in map.iter().enumerate() {
            assert_eq!(row.len(), 40, "floor{fi} row{r} len != 40: {row}");
            assert!(!row.contains(' '), "floor{fi} row{r} 含空格");
        }
    }
    let (sx, sy) = w.spawn();
    assert_eq!((sx, sy), (4, 14), "出生点应为 F1 (4,14)");
    for p in w.points {
        assert!(worlds::walkable(w, p.floor, p.x, p.y), "调查点 {}@L{}({},{}) 不可走动", p.id, p.floor + 1, p.x, p.y);
    }
    for e in w.enemies {
        assert!(worlds::walkable(w, e.floor, e.x, e.y), "敌人 {}@L{}({},{}) 不可走动", e.id, e.floor + 1, e.x, e.y);
    }
    for n in w.npcs {
        assert!(worlds::walkable(w, n.floor, n.x, n.y), "角色 {}@L{} 不可走动", n.id, n.floor + 1);
    }
    for z in w.zones {
        assert!(worlds::walkable(w, z.floor, z.x, z.y), "战圈 {}@L{} 不可走动", z.id, z.floor + 1);
    }
    for pt in w.portals {
        assert!(worlds::walkable(w, pt.floor, pt.x, pt.y), "传送门 {}@L{} 不可走动", pt.id, pt.floor + 1);
        assert!(worlds::walkable(w, pt.to_floor, pt.tx, pt.ty), "传送门 {} 落点@L{} 不可走动", pt.id, pt.to_floor + 1);
    }
    for g in w.gates {
        assert!(worlds::walkable(w, g.floor, g.x, g.y), "门禁 {}@L{} 不可走动", g.id, g.floor + 1);
    }
    println!("MAP REACHABLE OK · floors={} points={} enemies={} npcs={} zones={} portals={} gates={}",
        w.floors.len(), w.points.len(), w.enemies.len(), w.npcs.len(), w.zones.len(), w.portals.len(), w.gates.len());
}

/// ② 主线链 + 诱水剂脱水终结合缐渴水兽王 + 遗泽种子结算
#[test]
fn shaqiu_main_line_boss_finisher() {
    let mut st = GameState::new();
    fresh_combat(&mut st);
    engine::goto(&mut st, "sq_00_intro", &mut Vec::new());
    assert_eq!(st.scene_id, "sq_00_intro");

    // 幕1 开场 → F1 hub
    step(&mut st, "先侦查信号方向");

    // F1：黑匣子解密 + 储物柜撬具 + 解剖台线索
    step(&mut st, "调查黑匣子");
    step(&mut st, "解析灰盒");          // sq_decrypt_ok
    assert!(st.flag("sq_decrypt_ok"), "应置灰盒解密 flag");
    step(&mut st, "翻找储物柜");
    step(&mut st, "撬出磁力撬具");       // it_sq_pry
    assert!(st.inventory.iter().any(|i| i == "it_sq_pry"), "应得磁力撬具");
    step(&mut st, "查看解剖台");
    step(&mut st, "记下解剖台");         // sq_autopsy_hint
    assert!(st.flag("sq_autopsy_hint"), "应置解剖台线索");

    // 前往 F2 绿潮战场
    step(&mut st, "翻越北侧残骸");
    assert_eq!(st.scene_id, "sq_10_f2", "应抵达 F2 营地B");

    // F2：滤毒面罩 + 救人 + 蓄电池（信标供电，结局加成）
    step(&mut st, "翻找医疗箱");
    step(&mut st, "戴上滤毒面罩");
    assert!(st.inventory.iter().any(|i| i == "it_sq_mask"), "应得滤毒面罩");
    step(&mut st, "靠近被困幸存者");
    step(&mut st, "冲进去救人");         // sq_side_survivor
    assert!(st.flag("sq_side_survivor"), "应救出格列弗");
    step(&mut st, "检查蓄电池库");
    step(&mut st, "给信标供电");         // sq_battery_saved
    assert!(st.flag("sq_battery_saved"), "应置信标供电");

    // 前往 F3 母巢：卵堆解剖合成孢子血清
    step(&mut st, "深入绿潮母巢");
    assert_eq!(st.scene_id, "sq_20_f3", "应抵达 F3 母巢");
    step(&mut st, "解剖孵化腔卵堆");
    step(&mut st, "解剖卵堆（合成孢子血清）"); // 需 autopsy_hint
    assert!(st.inventory.iter().any(|i| i == "it_sq_serum"), "应得孢子血清");
    assert!(st.flag("sq_spore_serum"), "应置孢子血清 flag");

    // 前往 F4 深渊回响：壁画弱点 + 拆穿骗局 + 制作诱水剂
    step(&mut st, "深入子宫口");
    assert_eq!(st.scene_id, "sq_30_f4", "应抵达 F4 沙丘洞穴");
    step(&mut st, "查看壁画");
    step(&mut st, "记下弱水之秘");       // sq_boss_secret
    assert!(st.flag("sq_boss_secret"), "应记下弱水弱点");
    step(&mut st, "走向深渊回廊");
    step(&mut st, "拆穿骗局");           // 需 decrypt_ok
    assert!(st.flag("sq_side_trap"), "应拆穿伪文明骗局");
    step(&mut st, "制作诱水剂");         // it_sq_lure
    assert!(st.inventory.iter().any(|i| i == "it_sq_lure"), "应得诱水剂");

    // BOSS 渴水兽王（选择驱动链）：火焰削弱 → HP<50% 倾合诱水剂脱水 → 击毙
    step(&mut st, "【面向渴水兽王】");
    assert_eq!(st.scene_id, "sq_41_round", "应进入 BOSS 回合");
    let mut rounds = 0;
    while st.scene_id == "sq_41_round" {
        rounds += 1;
        assert!(rounds < 80, "BOSS 战超限未结束");
        let s = scenes::scene(&st.scene_id).expect("boss round scene");
        let visible: Vec<(usize, &_)> = s.choices.iter().enumerate()
            .filter(|(_, c)| c.cond.map_or(true, |f| f(&st))).collect();
        let fin = visible.iter().find(|(_, c)| c.label.contains("脱水重创")).map(|(i, _)| *i as i32);
        let idx = fin.unwrap_or_else(|| visible.iter().find(|(_, c)| c.label.contains("火焰"))
            .map(|(i, _)| *i as i32).expect("火焰选项"));
        choose_idx(&mut st, idx);
    }
    assert_eq!(st.scene_id, "sq_42_boss_down", "兽王应倒下");
    assert!(st.flag("sq_boss_dead"), "应置兽王殁 flag");
    assert!(st.flag("sq_dehydrated"), "诱水剂脱水应永久停止再生");

    // 取遗泽种子 → 升空 → 结算卡片
    step(&mut st, "走近圣物室");
    step(&mut st, "取走遗泽种子");       // sq_relic_seed
    assert!(st.inventory.iter().any(|i| i == "it_sq_relic_seed"), "应得遗泽种子");
    step(&mut st, "按照落着想");
    assert_eq!(st.scene_id, "sq_45_card", "应达结算卡片");
    assert!(st.sp_grade.is_some(), "结算应带评级");
    println!("MAIN LINE (lure finisher) OK · pts={} grade={:?}", st.points, st.sp_grade);
}

/// ③ 氧气倒计时 flag 链降级 + 低氧扣 HP
#[test]
fn shaqiu_oxy_timer_flag_chain() {
    let mut st = GameState::new();
    assert_eq!(st.hp, 100, "初始 HP 100");
    engine::goto(&mut st, "sq_00_intro", &mut Vec::new());
    step(&mut st, "先侦查信号方向");    // → sq_01_hub（不推进氧耗）
    // 3 次「调查黑匣子→解析灰盒」（非休整路线，返回推进氧耗）→ oxy_1 → oxy_2 → oxy_3（低氧警戒 HP-2）
    for _ in 0..3 {
        step(&mut st, "调查黑匣子");
        step(&mut st, "解析灰盒");       // Route::Dyn(sq_route_hub1) → oxy_tick
    }
    assert!(st.flag("sq_oxy_1"), "应推进到氧 flag 第 1 档");
    assert!(st.flag("sq_oxy_2"), "应推进到氧 flag 第 2 档");
    assert!(st.flag("sq_oxy_3"), "应推进到低氧第 3 档");
    assert!(st.hp <= 98, "低氧警戒应扣血，实际 hp={}", st.hp);
    println!("OXY TIMER FLAG CHAIN OK · hp={} (低氧警戒已扣血)", st.hp);

    // 战斗表完整性：渴水兽王 BOSS HP240 / rage_at 96 / reward 600
    let fights = wuxian_horror_ch1::scenes_shaqiu::shaqiu_figths();
    let ids: std::collections::HashSet<&str> = fights.iter().map(|(k, _)| *k).collect();
    for want in ["sq_f1_sandflea", "sq_f1_carrion", "sq_f1_mut", "sq_f2_sprout", "sq_f2_vine",
                 "sq_f2_spore", "sq_f2_wrangler", "sq_f3_larva", "sq_f3_sguard", "sq_f3_piercer",
                 "sq_f3_lpack", "sq_f3_soldier", "sq_f4_echo", "sq_f4_knight", "sq_boss_king"] {
        assert!(ids.contains(want), "战斗表缺少 {want}");
    }
    let g = fights.iter().find(|(k, _)| *k == "sq_boss_king").map(|(_, c)| c).unwrap();
    assert_eq!(g.hp, 240, "渴水兽王 HP 240");
    assert_eq!(g.rage_at, Some(96), "渴水兽王狂暴阈 96");
    assert_eq!(g.reward, 600, "渴水兽王奖励 600");
    println!("FIGHT TABLE OK · fights={}", fights.len());
}
//! 《无限曙光 · 铁血·地底金字塔》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/wuxian_shuguang/tiexue_jinzita.md。
//! 本文件是全新新增文件，只导出静态数据（TIEXUE_SCENES / tiexue_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/tiexue_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `tx_` 前缀；fight id 全部 `tx_` 前缀。
//! 主线 BOSS 异形皇后与支线铁血战士均采用"选择驱动遭遇链"（参考 scenes_cangjingge.rs 的 cj_shouge /
//! scenes_zhouyuan.rs 的 route_boss_attack）：因需要「破卵增员（狂暴增伤）」「铁血肩炮/祭坛酸液 双终结二选一」
//! 「铁血可战可和（归还腕刃结盟 / 伏击猎杀）」等自定义每回合同调与分支路由，引擎原生 FightCfg
//! 无此钩子，故用 Normal 场景 + Route::Dyn 落地；同时导出 `tx_alien_queen`/`tx_iron_predator`/以及
//! 各普通杂兵 FightCfg 供 EnemyDef / ZoneDef 与揭示用。
//!
//! 支线 flag 结算加成（见 impl 日志）：frozen_predator / altar_key / eggs_smashed /
//! predator_alliance / predator_hunted 每达成 +200（结算场景统一补发）。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 冰层营地    → bg tx_bg_ice        （现用 img_zhuyuan_book.png 占位）
//!   L2 金字塔墓道  → bg tx_bg_maze       （现用 img_corridor.png 占位）
//!   L3 祭坛圣殿    → bg tx_bg_altar      （现用 img_laser.png 占位）
//! 敌人立绘复用：licker→异形、hunter→铁血战士；异形同模可出多色；新美术由主 agent 统一生图替换。

use crate::defs::*;
use crate::state::GameState;
use rand::Rng;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   通用小工具
   ===================================================================== */
fn rng(a: i32, b: i32) -> i32 { rand::thread_rng().gen_range(a..=b) }

/// 物品栏是否持有（供 cond fn 使用；闭包不能捕获，故用独立具名函数）
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/// 已捣毁异形卵巢数量（L1×4 + L2×2 计 6 处；map_objs 计数）
fn eggs_smashed_count(st: &GameState) -> i32 {
    ["tx_p_egg1", "tx_p_egg2", "tx_p_egg3", "tx_p_egg4", "tx_p_egg5", "tx_p_egg6"]
        .iter().filter(|id| st.map_objs.get(**id).copied().unwrap_or(false)).count() as i32
}

/* =====================================================================
   具名条件谓词（cond：fn 指针）
   ===================================================================== */
fn cond_has_iron_blade(st: &GameState) -> bool { inv(st, "iron_blade") }
fn cond_g2_open(st: &GameState) -> bool { st.flag("tx_g2_open") }
fn cond_has_altar_key(st: &GameState) -> bool { st.flag("altar_key") }
fn cond_allied(st: &GameState) -> bool { st.flag("predator_alliance") }
fn cond_hunted(st: &GameState) -> bool { st.flag("predator_hunted") }
fn cond_acid_primed(st: &GameState) -> bool { st.flag("tx_acid_primed") }
fn cond_queen_raged(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.raged).unwrap_or(false)
}
fn cond_can_ally(st: &GameState) -> bool { inv(st, "iron_blade") && !cond_allied(st) && !cond_hunted(st) }
fn cond_can_hunt(st: &GameState) -> bool { !cond_allied(st) && !cond_hunted(st) }
/* ---- 条件补（供内联 cond 闭包转具名 fn） ---- */
fn cond_no_blade(st: &GameState) -> bool { !inv(st, "iron_blade") }
fn cond_no_g2(st: &GameState) -> bool { !st.flag("tx_g2_open") }
fn cond_no_altar(st: &GameState) -> bool { !st.flag("altar_key") }
fn cond_no_queen(st: &GameState) -> bool { !st.flag("queen_defeated") }
fn cond_queen_defeated(st: &GameState) -> bool { st.flag("queen_defeated") }
fn cond_egg_more(st: &GameState) -> bool {
    eggs_smashed_count(st) < 4 && !st.map_objs.get("tx_p_egg2").copied().unwrap_or(false)
}
fn cond_egg5_not(st: &GameState) -> bool { !st.map_objs.get("tx_p_egg5").copied().unwrap_or(false) }
fn cond_egg6_not(st: &GameState) -> bool { !st.map_objs.get("tx_p_egg6").copied().unwrap_or(false) }
fn cond_acid_not_step1(st: &GameState) -> bool { !st.flag("tx_acid_primed") && !st.flag("tx_acid_step1") }
fn cond_acid_step1(st: &GameState) -> bool { st.flag("tx_acid_step1") && !st.flag("tx_acid_primed") }

/* =====================================================================
   钉血战士（支线 BOSS，可战可和）：选择驱动遭遇
   ===================================================================== */
fn start_predator(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("tx_iron_predator") {
            st.fight = Some(crate::power::scaled_fight("tx_iron_predator", cfg, st, vec![]));
        }
    }
    "tx_predator_round".to_string()
}

/// 归还腕刃结盟：置 predator_alliance，促铁血颔首消失（皇后战触发肩炮终结技）
fn return_blade(st: &mut GameState) -> String {
    st.set_flag("predator_alliance");
    "tx_41_alliance".to_string()
}

/// 猎杀铁血战士胜利：+300、掉腕刃·猎场之礼（predator_wristblade_elite）、置 predator_hunted
fn predator_win(st: &mut GameState) -> String {
    st.points += 300;
    crate::world::add_item(st, "predator_wristblade_elite");
    st.set_flag("predator_hunted");
    "tx_42_hunted".to_string()
}

fn predator_dead() -> String { "tx_98_death_predator".to_string() }

/// 一合：玩家对铁血战士出手。guard=防守。
fn predator_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return predator_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 60).unwrap_or(false);
    if raged { if let Some(f) = st.fight.as_mut() { f.raged = true; } }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(19, 27) } else { rng(16, 24) };
    let dodge = if guard { 0.45 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 { return predator_dead(); }
    "tx_predator_round".to_string()
}

/// 腕刃连斩终结：狂暴后触发，直接制胜
fn predator_finisher(st: &mut GameState) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = 0; }
    predator_win(st)
}

/* =====================================================================
   主线 BOSS 异形皇后（HP200，双终结二选一）：选择驱动遭遇
   ===================================================================== */
fn start_queen(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("tx_alien_queen") {
            st.fight = Some(crate::power::scaled_fight("tx_alien_queen", cfg, st, vec![]));
        }
    }
    "tx_queen_round".to_string()
}

/// 皇后战一合：扣血 → 胜利检测 → 狂暴（破卵增员：已清扫卵巢则增员压力减半）→ 敌攻
fn queen_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return queen_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 100).unwrap_or(false);
    if raged { if let Some(f) = st.fight.as_mut() { f.raged = true; } }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    // 狂暴伤害 +6；若已清扫 ≥3 卵巢（eggs_smashed），破卵增员压力减半（额外侵蚀减量）
    let mut raw = if raged_now { rng(26, 36) } else { rng(20, 30) };
    if raged_now {
        if eggs_smashed_count(st) >= 3 {
            raw -= 3; // 清扫卵巢 → 增员压力减半
        }
    }
    // 首次受击：若加固营地（camp_prepared）则首击 -8
    if !st.flag("tx_first_hit_done") {
        if st.flag("camp_prepared") { raw -= 8; }
        st.set_flag("tx_first_hit_done");
    }
    let dodge = if guard { 0.40 } else { 0.12 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - (raw.max(1))).max(0);
    }
    if st.hp <= 0 { return queen_dead(); }
    "tx_queen_round".to_string()
}

/// 铁血肩炮助战终结（结盟线）：大额固定伤害 → 胜
fn finisher_shoulder(st: &mut GameState) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - 45).max(0); }
    st.set_flag("tx_queen_shoulder");
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { queen_win(st) } else { "tx_queen_round".to_string() }
}

/// 祭坛酸液喷口终结（无结盟线）：软化甲壳 → 胜
fn finisher_acid(st: &mut GameState) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - 55).max(0); }
    st.set_flag("tx_queen_acid");
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { queen_win(st) } else { "tx_queen_round".to_string() }
}

/// 皇后胜利：+500、掉死亡神性颗粒（death_divinity_shard）、置 queen_defeated（开 G4 回归门）
fn queen_win(st: &mut GameState) -> String {
    st.points += 500;
    crate::world::add_item(st, "death_divinity_shard");
    st.set_flag("queen_defeated");
    "tx_60_queen_win".to_string()
}

fn queen_dead() -> String { "tx_98_death_queen".to_string() }

/* =====================================================================
   普通杂兵 win / death 回调（FightCfg.win 用；EnemyDef 自动战斗）
   ===================================================================== */
/// 依当前楼层回对应 hub（L1/L2/L3）
fn win_floor(st: &GameState) -> String {
    match st.floor {
        0 => "tx_10_l1_hub".to_string(),
        1 => "tx_20_l2_hub".to_string(),
        _ => "tx_30_l3_hub".to_string(),
    }
}
fn win_l1(_st: &GameState) -> String { "tx_10_l1_hub".to_string() }
fn win_l2(_st: &GameState) -> String { "tx_20_l2_hub".to_string() }
fn win_l3(_st: &GameState) -> String { "tx_30_l3_hub".to_string() }
fn tx_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   蛋卵机构路由：捣毁卵后重新统计；对齐完成后触发对应楼层 facehugger 战
   ===================================================================== */
/// 返回所在楼层的「异形伏击」战斗场景（L1→tx_20_egg_fight，L2→tx_21_egg_fight）
fn egg_route(st: &mut GameState) -> String {
    if eggs_smashed_count(st) >= 3 {
        st.set_flag("eggs_smashed");
    }
    if st.floor == 1 { "tx_21_egg_fight".to_string() } else { "tx_20_egg_fight".to_string() }
}

/* =====================================================================
   战斗配置表（id 全部 tx_ 前缀）
   ===================================================================== */
pub fn tiexue_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("tx_facehugger", FightCfg {
            name: "抱脸虫（卵袭）", hp: 34, dmg: (5, 9), reward: 10, reward_why: "击碎卵巢 · 蟎袭",
            intro: "卵膜「啵」地裂开，一头抱脸虫八足弹射而起——它是异形诸族的源头，也是猎场最常见的陷阱。",
            rage_at: None, rage_text: "", on_rage: tx_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_floor, death: "tx_98_death",
        }),
        ("tx_alien_scout", FightCfg {
            name: "异形侦察兵（雄蜂·幼体）", hp: 42, dmg: (10, 15), reward: 25, reward_why: "肃清冰原侦察兵",
            intro: "一道炭黑的弧形剪影从冰隙间跃出——无眼的长颅、双颚间的黏液丝，它在风雪里几乎隐于无形。",
            rage_at: Some(20), rage_text: "它发出高频嘶叫，背管震颤——攻势陡然凌厉！", on_rage: tx_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_l1, death: "tx_98_death",
        }),
        ("tx_alien_drone", FightCfg {
            name: "异形雄蜂", hp: 60, dmg: (11, 17), reward: 80, reward_why: "制造墓道雄蜂",
            intro: "墓道深处一具半模块的躯体弹射而出，尾刺拖行在石板上刮出刺耳的尖响——它守着通往核心的墓道。",
            rage_at: Some(30), rage_text: "尾刺连击！它在当回合多抽了你一记低伤判定！", on_rage: tx_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_floor, death: "tx_98_death",
        }),
        ("tx_alien_warrior", FightCfg {
            name: "异形战士（核心守卫）", hp: 92, dmg: (14, 21), reward: 120, reward_why: "斩杀核心墓室守卫",
            intro: "盘踞在祭坛石板前的守卫缓缓起立，甲壳带肋骨纹、背管比雄蜂粗大一倍——它把整座核心墓室据为己有。",
            rage_at: Some(40), rage_text: "酸血喷射！它肋甲爆开，飞溅的体液蚀得石板滋滋作响（追加 Hurt 8~12）！", on_rage: tx_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_l2, death: "tx_98_death",
        }),
        ("tx_alien_guard", FightCfg {
            name: "异形禁卫（皇后亲卫）", hp: 112, dmg: (16, 24), reward: 200, reward_why: "肃清皇后亲卫",
            intro: "两具如铁塔般抵在巢口，甲壳泛着血锈红边——它们是皇后的亲卫，寸步不让。",
            rage_at: Some(50), rage_text: "亲卫狂暴！它下颌大开，攻势如潮，伤害暴涨！", on_rage: tx_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_l3, death: "tx_98_death",
        }),
        ("tx_iron_predator", FightCfg {
            name: "铁血·成年礼战士", hp: 150, dmg: (16, 24), reward: 300, reward_why: "猎杀成年礼铁血战士",
            intro: "网纹装甲、束发辫、面具下颚的两道青光——铁血战士从高处无声落地，双腕刃缓缓出鞘。它没有吼叫，只发出识别身份的深长「啧」声。",
            rage_at: Some(60), rage_text: "它按下肩炮开关在即——这是隐形突袭的征兆！", on_rage: tx_rage_none,
            finisher_if: |st, _| cond_hunted(st) == false && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "【腕刃连斩 · 终结】".to_string(),
            finisher_desc: |_| "你将铁血逼至祭坛石柱的角落，腕刃三连斩自它肩甲一路剥开——成年礼的猎手，终成猎场里倒下的猎物。".to_string(),
            win: |_st| "tx_42_hunted".to_string(),
            death: "tx_98_death_predator",
        }),
        ("tx_alien_queen", FightCfg {
            name: "异形皇后", hp: 200, dmg: (20, 30), reward: 500, reward_why: "斩杀猎场之主 · 异形皇后",
            intro: "卵膜自祭坛石缝撕开——那个比巢穴阴影更庞大的轮廓，拖着产卵管的尾腹，缓缓立起。异形皇后，这座猎场真正的主人，把这座金字塔当成了育婴房。",
            rage_at: Some(100), rage_text: "破卵增员！皇后仰起长颅嘶鸣，巢壁的卵膜次第破裂，一头头异形雄蜂加入战场（若已清扫卵巢，增员压力减半）！", on_rage: tx_rage_none,
            finisher_if: |st, _| st.fight.as_ref().map(|f| f.raged).unwrap_or(false) && (cond_allied(st) || cond_acid_primed(st)),
            finisher_name: finisher_name_queen,
            finisher_desc: finisher_desc_queen,
            win: |_st| "tx_30_l3_hub".to_string(), // 皇后战斗由选择驱动链路（start_queen→queen_act→queen_win）处理，此 win 不改写
            death: "tx_98_death_queen",
        }),
    ]
}

/// 皇后终结技名称：结盟线 肩炮助战 / 无结盟 祭坛酸液
fn finisher_name_queen(st: &GameState) -> String {
    if cond_allied(st) { "【铁血·肩炮助战】".to_string() } else { "【祭坛酸液喷口】".to_string() }
}
fn finisher_desc_queen(st: &GameState) -> String {
    if cond_allied(st) {
        "铁血战士自高处按下肩炮扳机，「吭—嗡—轰」连轰三记，皇后的甲壳在轰鸣中寸寸碎裂——猎手的复仇与盟友的援手合于一处。".to_string()
    } else {
        "你踩下祭坛两侧的石板，红雾般的酸液喷口朝皇后倾泻，将它的甲壳一路腐蚀——皇后嘶鸣失衡，露出致命的软肋。".to_string()
    }
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn tx_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    tiexue_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 tx_ 前缀）
   ===================================================================== */
pub static TIEXUE_SCENES: &[SceneDef] = &[

/* ================= 第一幕 · 开场：雪与猎场（L1） ================= */
SceneDef {
    id: "tx_00_open", bg: Some("tiexue_bg.png"), loc: Some("L1 · 铁血金字塔 · 裂谷入口"),
    mood: "cold", speaker: Some("楚浩"), voice: Some("vo_tx_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>深入南极冰盖下三千米的铁血祭坛金字塔，击杀异形皇后，夺取死亡神性颗粒。",
        "楚浩（冷静推算）：「南极，冰盖之下三千米。铁血战士把这里经营成了一座猎场金字塔——他们在里面豢养异形，用于成年礼的试炼。（停顿）而主神告诉我，祭坛深处，埋着郑吒当年打碎的七十万余神性颗粒里，仍可拾取的一枚。」",
        "张恒（吐槽）：「翻译一下——我们要在全是口水的怪物窝里爬三层楼，还得顺路捡个『神性』回家。先说好，我只负责活着。」",
        "<em>猎人的猎场，闯入者——你就是下一个猎物，也是打破祭典的人。</em>",
    ]),
    choices: &[
        ChoiceDef { label: "检查冻尸", sub: "取铁血腕刃 · 铺 G2/铁血线", cond: None, effects: &NO_EFF, route: Route::To("tx_11_frozen") },
        ChoiceDef { label: "加固营地", sub: "-10 点 · 皇后首击减伤 8", cond: None,
            effects: &[Eff::Points(-10), Eff::SetFlag("camp_prepared")], route: Route::To("tx_10_l1_hub") },
        ChoiceDef { label: "直接出发", sub: "快进 · 无额外收益", cond: None, effects: &NO_EFF, route: Route::To("tx_10_l1_hub") },
    ],
    fight_id: None, video: Some("vid_tx_open.mp4"), cine_label: Some("过场 · 雪与猎场"), overlay: None,
},

SceneDef {
    id: "tx_10_l1_hub", bg: Some("tiexue_bg.png"), loc: Some("L1 · 冰层营地 · 裂谷入口"),
    mood: "cold", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        let eggs = eggs_smashed_count(st);
        let egg_line = if eggs >= 3 { format!("已清扫卵巢 {} 处（≥3）——皇后战的破卵增员压力将减半。", eggs) }
            else { format!("异形卵巢零零散散（已清 {} 处；≥3 处会使皇后增员压力减半）。", eggs) };
        format!(
            "冰原风雪在裂谷入口盘旋，营地旧房的铁门吱呀作响。\n{}",
            egg_line
        )
    }),
    choices: &[
        ChoiceDef { label: "冰原补给箱", sub: "+HP +San", cond: None, effects: &NO_EFF, route: Route::To("tx_12_supply") },
        ChoiceDef { label: "冻尸 · 铁血战士", sub: "取腕刃 iron_blade", cond: None, effects: &NO_EFF, route: Route::To("tx_11_frozen") },
        ChoiceDef { label: "捣毁异形卵", sub: "清扫卵巢（抱脸虫伏击）", cond: None, effects: &NO_EFF, route: Route::To("tx_20_egg") },
        ChoiceDef { label: "营房左耳室 · 下冰阶", sub: "T1 单向 → L2", cond: None, effects: &NO_EFF, route: Route::To("tx_15_l1_to_l2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_12_supply", bg: Some("tiexue_bg.png"), loc: Some("L1 · 冰原补给箱"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["你在冻僵的补给箱里翻出一份冰原干粮（ice_ration），就着雪啃了几口，体温回升了一些，神志也清明了些。"]),
    choices: &[ChoiceDef { label: "（收下补给）", sub: "Item ice_ration · +20 点 · San+5", cond: None,
        effects: &[Eff::AddItem("ice_ration"), Eff::San(5), Eff::Points(20), Eff::MarkPoint("tx_p_supply")],
        route: Route::To("tx_10_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_11_frozen", bg: Some("tiexue_bg.png"), loc: Some("L1 · 冻尸 · 铁血战士"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "iron_blade") {
            "那具铁血冻尸腕上的刃已被你取走。你再度审视时，只从它深陷的眼窝里读出两个字：<em>猎场。</em>".to_string()
        } else {
            "冰层里冻着一具竖立的铁血战士遗体，腕刃仍紧紧握着，指骨泛着死前的青灰。「死去的猎手，腕刃仍紧握。」你费了些力，将那双刃摘下。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "卸下铁血腕刃", sub: "Item iron_blade · 置 frozen_predator · San-5", cond: Some(cond_no_blade),
        effects: &[Eff::AddItem("iron_blade"), Eff::SetFlag("frozen_predator"), Eff::SetFlag("tx_g2_open"),
            Eff::San(-5), Eff::MarkPoint("tx_p_frozen")], route: Route::To("tx_10_l1_hub") },
        ChoiceDef { label: "抱拳致意 · 离开", sub: "尊重猎手 · 无收益", cond: Some(cond_has_iron_blade),
            effects: &[], route: Route::To("tx_10_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_15_l1_to_l2", bg: Some("tiexue_bg.png"), loc: Some("L1 → L2 · 单向下冰阶"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你踏入左耳室的下冰阶，脚下的石阶向左下方无限伸展。身后传来闷雷般的坍塌——裂谷入口塌方，退路封死。你只能前行。"]),
    choices: &[ChoiceDef { label: "（沿冰阶下行 · T1 单向）", sub: "→ L2 金字塔墓道", cond: None, effects: &NO_EFF, route: Route::To("tx_20_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 异形卵 · 清扫（4 处独立场景，捣毁后对齐统计） ---- */
SceneDef { id: "tx_20_egg", bg: Some("tiexue_bg.png"), loc: Some("L1 · 异形卵膜簇"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["一簇簇卵膜在冰层下起伏呼吸，隐隐可见里面蜷缩的黑影。破坏它们，能削弱皇后巢穴的增员力量。"]) ,
    choices: &[ChoiceDef { label: "捣毁一枚卵", sub: "异形扑袭 · 计卵巢进度", cond: None,
        effects: &[Eff::MarkPoint("tx_p_egg1")], route: Route::Dyn(egg_route) },
        ChoiceDef { label: "再捣毁一枚（任选一枚未破的卵）", sub: "异形扑袭", cond: Some(cond_egg_more),
            effects: &[Eff::MarkPoint("tx_p_egg2")], route: Route::Dyn(egg_route) },
        ChoiceDef { label: "绕行不理", sub: "线强作不出声", cond: None, effects: &NO_EFF, route: Route::To("tx_10_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_20_egg_fight", bg: Some("tiexue_bg.png"), loc: Some("L1 · 卵膜 · 洞穴"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["被你弄裂的卵膜里窜出一头抱脸虫——湿淋淋的黏液裹着八爪，朝你脸面弹射而来。（战斗）"]) ,
    choices: &NO_CH, fight_id: Some("tx_facehugger"), video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_21_egg_fight", bg: Some("tiexue_bg.png"), loc: Some("L2 · 巢室卵膜"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["墓道巢室里，几枚更大的卵膜似乎感应到你的逼近，微微颤动。你捣裂其中一枚，诱出一头抱脸虫。（战斗）"]) ,
    choices: &NO_CH, fight_id: Some("tx_facehugger"), video: None, cine_label: None, overlay: None,
},

/* ================= 第二幕 · 转折：迷宫之心（L2） ================= */
SceneDef {
    id: "tx_20_l2_hub", bg: Some("tiexue_bg.png"), loc: Some("L2 · 金字塔墓道迷宫"),
    mood: "danger", speaker: Some("楚浩"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("altar_key") {
            "墓道深处鳞次栉比的卵膜起伏呼吸。祭坛石板已解读——死亡神性就藏在 L3 祭坛中央的碎裂圣像里。圣门 g3 已为你敞开。".to_string()
        } else if st.flag("tx_g2_open") {
            "墓道深处鳞次栉比的卵膜起伏呼吸。地图显示：中央的核心墓室被一道需腕刃/酸液的假板门（G2）锁着——但你已经通过了。进去解读祭坛石板吧。".to_string()
        } else {
            "墓道深处鳞次栉比的卵膜起伏呼吸。北三墓室各藏机关，回廊中段有酸液枪管陷阱，中央核心墓室被 G2 石板门锁死——<em>需铁血腕刃（L1 冻尸），或引酸液蚀开门锁。</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "入口浮雕壁画", sub: "铁血成年礼 · 剧情", cond: None, effects: &NO_EFF, route: Route::To("tx_30_wall") },
        ChoiceDef { label: "陪葬室 · 武器架", sub: "+20 点 · 情报", cond: None, effects: &NO_EFF, route: Route::To("tx_30_warrack") },
        ChoiceDef { label: "清扫巢室卵膜", sub: "巢室卵 ×2", cond: None, effects: &NO_EFF, route: Route::To("tx_21_egg") },
        ChoiceDef { label: "酸液枪管陷阱", sub: "引酸蚀 G2 门锁 · HP-15 代价", cond: None, effects: &NO_EFF, route: Route::To("tx_32_acid") },
        ChoiceDef { label: "核心墓室 · 祭坛石板", sub: "需 G2 已开", cond: Some(cond_g2_open), effects: &NO_EFF, route: Route::To("tx_33_core") },
        ChoiceDef { label: "祭坛圣门", sub: "需 altar_key → L3", cond: Some(cond_has_altar_key), effects: &NO_EFF, route: Route::To("tx_35_l2_to_l3") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_30_wall", bg: Some("tiexue_bg.png"), loc: Some("L2 · 入口浮雕壁画"), mood: "mystery",
    speaker: Some("楚浩"), voice: None,
    text: TextSpec::Static(&["壁画刻着铁血战士的成年礼：幼年战士独身深入卵巢，猎杀第一只异形。「……他们在用异形养育战士。」楚浩的声音压得很低，「我们闯进了一场正进行到一半的祭典。」张恒：「所以那些卵——」（卵膜裂开的声音）"]) ,
    choices: &[ChoiceDef { label: "（回墓道）", sub: "剧情情报", cond: None, effects: &[Eff::MarkPoint("tx_p_wall_l2")], route: Route::To("tx_20_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_30_warrack", bg: Some("tiexue_bg.png"), loc: Some("L2 · 陪葬室 · 武器架"), mood: "calm",
    speaker: None, voice: None,
    text: TextSpec::Static(&["陪葬室的武器架上摆着几件锈铁血武器与一匣弹丸。你挑了两枚合手的补给，顺带记下了墙上的献祭铭文（提示：祭坛需要『踩对两片石板』，或可与铁血结盟以得肩炮助战）。"]) ,
    choices: &[ChoiceDef { label: "取走补给", sub: "+20 点 · 线索", cond: None,
        effects: &[Eff::Points(20), Eff::MarkPoint("tx_p_warrack")], route: Route::To("tx_20_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_32_acid", bg: Some("tiexue_bg.png"), loc: Some("L2 · 酸液枪管陷阱"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["回廊中段的石壁嵌着一杆锈蚀的铁血枪管，管口淤着经年累月的酸液。它不是武器——是用来蚀开门锁的机关。"]),
    choices: &[
        ChoiceDef { label: "引酸蚀门锁", sub: "HP-15 · 开 G2 替代路径", cond: Some(cond_no_g2),
            effects: &[Eff::Hurt(15, "tx_98_death"), Eff::SetFlag("tx_acid_burned"), Eff::SetFlag("tx_g2_open"), Eff::MarkPoint("tx_p_acid")],
            route: Route::To("tx_20_l2_hub") },
        ChoiceDef { label: "绕行", sub: "无代价 · 无 flag", cond: None, effects: &NO_EFF, route: Route::To("tx_20_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_33_core", bg: Some("tiexue_bg.png"), loc: Some("L2 · 核心墓室 · 祭坛石板"), mood: "mystery",
    speaker: Some("系统"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("altar_key") {
            "祭坛石板上的死亡法则铭文已被你记住——死亡神性颗粒就在 L3 圣殿中央的碎裂圣像里。".to_string()
        } else {
            "核心墓室中央，一座方形祭坛石板泛着幽蓝的微光，铭文是一段死亡法则：「万物终将归于尘，唯贪婪者以魂为食。」解读它，能揭开碎裂圣像的位置。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "解读祭坛石板", sub: "置 altar_key · +50 点 · 开 G3", cond: Some(cond_no_altar),
        effects: &[Eff::SetFlag("altar_key"), Eff::Points(50), Eff::MarkPoint("tx_p_altar_stela")], route: Route::To("tx_20_l2_hub") },
        ChoiceDef { label: "转身离去", sub: "回墓道", cond: None, effects: &NO_EFF, route: Route::To("tx_20_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "tx_35_l2_to_l3", bg: Some("tiexue_bg.png"), loc: Some("L2 → L3 · 圣门单向"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["你念出石板上的死亡法则，祭坛圣门 G3 的符文亮起金光。脚下石板突然塌陷，三人一路向下坠入了圣殿——这是单向的坠落，只进不出。"]),
    choices: &[ChoiceDef { label: "（坠入圣殿 · T3 单向）", sub: "→ L3 祭坛圣殿", cond: None, effects: &NO_EFF, route: Route::To("tx_30_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 巢室卵 · 清扫（2 处） ---- */
SceneDef { id: "tx_21_egg", bg: Some("tiexue_bg.png"), loc: Some("L2 · 异形巢室 · 卵膜"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["北侧巢室里卵膜挤作一团，隐约还有未孵化的幼体蜷动。清除它们同样计入卵巢进度（eggs_smashed）。"]),
    choices: &[ChoiceDef { label: "捣碎巢室卵膜", sub: "异形扑袭 · 计进度", cond: Some(cond_egg5_not),
        effects: &[Eff::MarkPoint("tx_p_egg5")], route: Route::Dyn(egg_route) },
        ChoiceDef { label: "另一侧巢卵", sub: "异形扑袭 · 计进度", cond: Some(cond_egg6_not),
            effects: &[Eff::MarkPoint("tx_p_egg6")], route: Route::Dyn(egg_route) },
        ChoiceDef { label: "退向墓道", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tx_20_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 第三幕 · 结局：祭坛上的猎与猎物（L3） ================= */
SceneDef {
    id: "tx_30_l3_hub", bg: Some("tiexue_bg.png"), loc: Some("L3 · 祭坛圣殿 · 皇后巢"),
    mood: "danger", speaker: Some("楚浩"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("queen_defeated") {
            "祭坛中央的碎裂圣像泛着幽光，死亡神性颗粒已被你取走。皇后已伏诛——归途圣门 G4 向主神空间敞开。".to_string()
        } else if st.flag("predator_alliance") {
            "圣殿深处，那名铁血战士立在高柱上，朝你颔首。你已与他结盟——皇后战时可召它肩炮助战。祭坛中央的碎裂圣像泛着幽光，更大的阴影正在卵膜深处立起。".to_string()
        } else if st.flag("predator_hunted") {
            "你猎杀了那名成年礼战士，腕刃上的猎场之礼仍有余温。祭坛中央的碎裂圣像泛着幽光——那个拖着产卵管的皇后轮廓，正在卵膜深处立起。".to_string()
        } else {
            "祭坛中央的碎裂圣像泛着幽光——死亡神性颗粒就在其中。而更大的阴影从卵膜深处立起：异形皇后，这座猎场真正的主人，正把产卵管插进祭坛的石缝。\n「它把这里当成了育婴房。」楚浩向前一步，「那就让它看看，谁才是这场祭典里先死的一方。」".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "铁血·成年礼战士", sub: "支线 · 可战可和", cond: None, effects: &NO_EFF, route: Route::To("tx_40_predator") },
        ChoiceDef { label: "祭坛酸液喷口机关", sub: "puzzle · 按序踩石板", cond: Some(cond_no_queen), effects: &NO_EFF, route: Route::To("tx_50_acid_puzzle") },
        ChoiceDef { label: "碎裂圣像 · 迎战皇后", sub: "BOSS 决战", cond: Some(cond_no_queen), effects: &NO_EFF, route: Route::To("tx_55_queen_start") },
        ChoiceDef { label: "回归圣门", sub: "胜利后 → 主神结算", cond: Some(cond_queen_defeated), effects: &NO_EFF, route: Route::To("tx_90_exit") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 支线 · 铁血战士（可战可和） ---- */
SceneDef {
    id: "tx_40_predator", bg: Some("tiexue_bg.png"), loc: Some("L3 · 圣殿 · 铁血战士"),
    mood: "mystery", speaker: Some("铁血战士"), voice: Some("vo_tx_predator"),
    text: TextSpec::Static(&[
        "一名铁血战士立在石柱高处，网纹装甲沾着经年累月的血渍，面具下冒出两缕识别的龇气流。它没有吼叫，只发出深长的「啧——」声——那是猎人打量猎物，也是见了同类的认可与警告。",
        "它手腕上那双刃，与你在 L1 冻尸上取下的一模一样。",
    ]),
    choices: &[
        ChoiceDef { label: "归还腕刃（结盟）", sub: "需 iron_blade · 置 predator_alliance", cond: Some(cond_can_ally), effects: &NO_EFF, route: Route::Dyn(return_blade) },
        ChoiceDef { label: "伏击猎杀", sub: "boss 战 · 猎杀线", cond: Some(cond_can_hunt), effects: &NO_EFF, route: Route::Dyn(start_predator) },
        ChoiceDef { label: "就此后退（不战）", sub: "回圣殿", cond: None, effects: &NO_EFF, route: Route::To("tx_30_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_41_alliance", bg: Some("tiexue_bg.png"), loc: Some("L3 · 圣殿 · 铁血战士"),
    mood: "calm", speaker: Some("铁血战士"), voice: None,
    text: TextSpec::Static(&[
        "你将 L1 冻尸上的腕刃递出的瞬间，铁血战士的动作顿住了。它低头看了看那枚腕刃，又看向你，面具下的龇气流化为一声低沉却有分量的「啧——」。（它接过腕刃，向你行了一个猎人的礼。）",
        "<em>【结盟达成】铁血颔首隐入高处的阴影——皇后战中，它将以肩炮助你斩首。</em>",
    ]),
    choices: &[ChoiceDef { label: "（受领猎手之约 · 回圣殿）", sub: "predator_alliance", cond: None, effects: &NO_EFF, route: Route::To("tx_30_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_predator_round", bg: Some("tiexue_bg.png"), loc: Some("L3 · 圣殿 · 猎杀之战"),
    mood: "danger", speaker: Some("铁血战士"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("铁血战士 HP {} / {}", f.hp.max(0), 150)).unwrap_or_else(|| "铁血战士 HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { "——隐形突袭的征兆；狂暴后可腕刃连斩终结——" } else { "" };
        format!("{f}。{}", mode)
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| predator_act(st, rng(30, 42), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-28", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| predator_act(st, rng(20, 28), false)) },
        ChoiceDef { label: "【腕刃连斩 · 终结】", sub: "狂暴后制胜", cond: Some(cond_queen_raged), effects: &NO_EFF, route: Route::Dyn(predator_finisher) },
        ChoiceDef { label: "防守蓄势", sub: "提升闪避", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| predator_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_42_hunted", bg: Some("tiexue_bg.png"), loc: Some("L3 · 圣殿 · 猎杀胜利"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["铁血战士轰然倒地，腕刃「铮」地脱落。狩猎它的是一双远比它狠的猎手——你取下它腕上的猎场之礼（predator_wristblade_elite）。祭坛中央的碎裂圣像泛着幽光。"]),
    choices: &[ChoiceDef { label: "（回圣殿 · 迎战皇后）", sub: "predator_hunted", cond: None, effects: &NO_EFF, route: Route::To("tx_30_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 祭坛酸液喷口机关（2 步 puzzle → tx_acid_primed） ---- */
SceneDef {
    id: "tx_50_acid_puzzle", bg: Some("tiexue_bg.png"), loc: Some("L3 · 祭坛酸液喷口"),
    mood: "mystery", speaker: Some("系统"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("tx_acid_primed") {
            "酸液喷口已被激活，喷道里蓄满了猩红的酸液——随时可朝皇后的甲壳倾泻，为她开出致命软肋（酸液终结技已可用）。".to_string()
        } else {
            "祭坛底座两侧各有一块可踩踏的石板，铭文提示「按序」：先左后右，可诱发酸液喷口；踩错顺序会被反噬一道液体。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "踩左侧石板", sub: "第一步（需再踩右侧完成）", cond: Some(cond_acid_not_step1),
            effects: &[Eff::SetFlag("tx_acid_step1")], route: Route::To("tx_50_acid_puzzle") },
        ChoiceDef { label: "踩右侧石板（先左后右）", sub: "完成解锁", cond: Some(cond_acid_step1),
            effects: &[Eff::SetFlag("tx_acid_primed"), Eff::MarkPoint("tx_z_acid")], route: Route::To("tx_50_acid_puzzle") },
        ChoiceDef { label: "不踩了 · 回圣殿", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tx_30_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 主线 BOSS · 异形皇后（选择驱动，双终结二选一） ---- */
SceneDef {
    id: "tx_55_queen_start", bg: Some("tiexue_bg.png"), loc: Some("L3 · 中央祭坛 · 皇后现身"),
    mood: "danger", speaker: Some("楚浩"), voice: Some("vo_tx_queen"),
    text: TextSpec::Static(&[
        "卵膜撕裂声裹着祭坛的幽光。异形皇后自石缝中立起，产卵管的尾腹拖行在石板上，猩红的肉壁随之鼓动——它把这座金字塔当成了育婴房。",
        "楚浩向前一步：「那就让它看看，谁才是这场祭典里先死的一方。」",
    ]),
    choices: &[ChoiceDef { label: "【迎战皇后】", sub: "BOSS 决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_queen) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_queen_round", bg: Some("tiexue_bg.png"), loc: Some("L3 · 皇后巢 · 激战"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("异形皇后 HP {} / {}", f.hp.max(0), 200)).unwrap_or_else(|| "异形皇后 HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——【已破卵增员】狂暴：伤害 +6，若清扫 ≥3 卵巢则增员压力减半——"
        } else { "" };
        let fin = if cond_allied(st) {
            "【铁血·肩炮助战 已可用】铁血战士伏在高处，肩炮充能「吭—嗡」。"
        } else if st.flag("tx_acid_primed") {
            "【祭坛酸液喷口 已可用】酸液蓄势待发，只待一击命中皇后甲壳。"
        } else {
            "（可先去圣殿结盟，或激活祭坛酸液机关。）"
        };
        format!("{f}{mode}\n{fin}")
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, rng(30, 42), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-28", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, rng(20, 28), false)) },
        ChoiceDef { label: "【铁血·肩炮助战】", sub: "结盟线终结", cond: Some(cond_allied), effects: &NO_EFF, route: Route::Dyn(finisher_shoulder) },
        ChoiceDef { label: "【祭坛酸液喷口】", sub: "机关线终结", cond: Some(cond_acid_primed), effects: &NO_EFF, route: Route::Dyn(finisher_acid) },
        ChoiceDef { label: "防守蓄势", sub: "提升闪避", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_60_queen_win", bg: Some("tiexue_bg.png"), loc: Some("L3 · 碎裂圣像 · 胜利"),
    mood: "calm", speaker: Some("系统"), voice: Some("vo_tx_queen_win"),
    text: TextSpec::Dyn(|st| {
        let fin = if st.flag("tx_queen_shoulder") {
            "铁血的肩炮连轰数记，皇后的甲壳在轰鸣中碎裂——猎手与盟友并肩，终结了这场千年的祭典。"
        } else if st.flag("tx_queen_acid") {
            "酸液喷口倾泻而下，皇后的甲壳被腐蚀洞开——嘶鸣失衡，它倒在了自己豢养了千年的猎场中央。"
        } else {
            "皇后的长颅重重砸落在祭坛石台上，猎场终于在今夜换了主人。"
        };
        format!(
            "{}\n\n祭坛中央的碎裂圣像泛着幽光——死亡神性颗粒就在其中。「这是郑吒当年打碎的神性之一。」",
            fin
        )
    }),
    choices: &[
        ChoiceDef { label: "带回去解析", sub: "任务绑定 · 主神占位", cond: None,
            effects: &[], route: Route::To("tx_61_shard") },
        ChoiceDef { label: "握紧感受", sub: "+10 San · 追加演出", cond: None,
            effects: &[Eff::San(10)], route: Route::To("tx_61_shard") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_61_shard", bg: Some("tiexue_bg.png"), loc: Some("L3 · 碎裂圣像 · 拾取"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["你伸出手，从碎裂圣像的裂缝间拈起那粒泛着法则微光的颗粒——死亡的概念在指尖流过。（AddItem death_divinity_shard 任务绑定，免掉落，已收于背上。）归途圣门 G4 已开——该回去了。"]),
    choices: &[ChoiceDef { label: "（走向回归圣门）", sub: "G4 · 结算", cond: None, effects: &NO_EFF, route: Route::To("tx_90_exit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结局 / 结算 ================= */
SceneDef {
    id: "tx_90_exit", bg: Some("tiexue_bg.png"), loc: Some("L3 · 回归圣门"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("queen_defeated") {
            "黄金封蜡剥落，圣门向主神空间敞开。死亡神性颗粒在你怀中泛着幽微的光——你把这座猎场金字塔的诅咒，带回了它该在的地方。".to_string()
        } else {
            "你尚不足以离开——猎场的阴影仍盘踞在圣殿深处。只有击杀异形皇后，归途才会显形。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "（踏入圣门 · 结算）", sub: "结算是分支付", cond: Some(cond_queen_defeated),
        effects: &NO_EFF, route: Route::Dyn(route_exit_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tx_95_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_tx_settle"), death: None,
        card: |st| crate::state::Card {
            title: "铁 血 金 字 塔 · 破 猎 之 归".into(), good: true,
            body_html: format!(
                "<p>你自祭坛圣殿踏出，南极的风雪与地底的闷热在圣门两侧交错。死亡神性颗粒安静地蜷在你怀中——它曾是郑吒打碎的神性之一。</p>\
                 <p style='color:#9a958a'>猎人，猎物，在这里，是同一个词。</p>\
                 <table class='statTable'>\
                 <tr><td>累计奖励点数</td><td>{}</td></tr>\
                 <tr><td>存活队友加成 × {} 人</td><td>+{}</td></tr>\
                 <tr><td>支线 flag 达成 × {} 条（各 +200）</td><td>+{}</td></tr>\
                 <tr><td>剩余理智</td><td>{}</td></tr>\
                 <tr><td style='color:#8fd0a8'><b>轮回总计</b></td><td style='color:#8fd0a8;font-size:18px'><b>{}</b></td></tr>\
                 <tr><td style='color:#ffd76a'><b>综合评价</b></td><td style='color:#ffd76a;font-size:18px'><b>{}</b></td></tr>\
                 </table><p>{}</p>",
                st.points, st.alive_count(), st.alive_count() * 100,
                tx_side_count(st), tx_side_count(st) * 200,
                st.san.max(0), st.settle_total, st.settle_rank,
                if st.flag("predator_alliance") { "你与铁血猎手并肩，终结了猎场千年的祭典。" }
                else if st.flag("predator_hunted") { "你猎杀了猎手——猎场的祭典，在你的腕刃下换了主人。" }
                else { "你独力斩杀皇后，带着神性颗粒走出猎场。" }
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案（复活扣 300 / 回主神） ================= */
SceneDef {
    id: "tx_98_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("猎场之迹", "在异形群中倒下于铁血金字塔")), card: |_st| crate::state::Card {
            title: "猎 场 之 迹".into(), good: false,
            body_html: r#"<p>冰层在头顶嘎吱作响，一头异形在你蜷缩的阴影里落下最后的爪击。猎人的猎场，从不因猎物倒下而停下。</p>
<p style='color:#ff8a8a'>【死亡档案 · 猎场之迹】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。关键道具免掉落。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "tx_98_death_queen", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("祭坛陨落", "成了这场祭典的最终供品")), card: |_st| crate::state::Card {
            title: "祭 坛 陨 落".into(), good: false,
            body_html: r#"<p>皇后的长颅低垂到你的面前，产卵管的尾腹碾过碎裂圣像。「你成了祭典的最终供品。」猎场用你的骸骨，为下一场成年礼积攒养分。</p>
<p style='color:#ff8a8a'>【死亡档案 · 祭坛陨落】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点。死亡神性颗粒免掉落。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "tx_98_death_predator", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("猎手之刃", "铁血战士的腕刃在无形中穿胸")), card: |_st| crate::state::Card {
            title: "猎 手 之 刃".into(), good: false,
            body_html: r#"<p>隐形突袭，一击穿胸。你低估了一个猎手把猎物追到角落时的决心。铁血战士收起腕刃，重新隐入圣殿高处的阴影里。</p>
<p style='color:#ff8a8a'>【死亡档案 · 猎手之刃】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* =====================================================================
   Route::Dyn 路由函数（供 static 数组使用，fn 指针）
   ===================================================================== */
/// 逃离结算：按支线 flag 统一补发 +200×N，写入结算字段并进入结算卡片。
fn route_exit_settle(st: &mut GameState) -> String {
    let n = tx_side_count(st);
    st.points += n * 200; // 支线 flag 统一补发 +200×N
    let alive_total = st.alive_count() * 100;
    let total = st.points + alive_total;
    st.settle_total = total;
    st.settle_rank = if total >= 1600 { 'S' } else if total >= 1300 { 'A' } else if total >= 1000 { 'B' } else if total >= 700 { 'C' } else { 'D' };
    if st.sp_grade.is_none() { st.sp_grade = Some(if n >= 2 { 'B' } else { 'D' }); }
    "tx_95_card".to_string()
}

/// 支线 flag 计数（结算卡片与补发共用）
fn tx_side_count(st: &GameState) -> i32 {
    ["frozen_predator", "altar_key", "eggs_smashed", "predator_alliance", "predator_hunted"]
        .iter().filter(|k| st.flag(k)).count() as i32
}
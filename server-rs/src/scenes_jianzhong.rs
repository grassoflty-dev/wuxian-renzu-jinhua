//! 《侠行天下 · 剑冢禁地》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/xiaxing_tianxia/jianzhong.md §4/§5/§6/§7/§8。
//! 本文件是全新新增文件，只导出静态数据（JIANZHONG_SCENES / jianzhong_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/jianzhong_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `jz_` 前缀；fight id 全部 `jz_`（与场景分属 scene()/fight_cfg() 两张独立查找表，无冲突）。
//! 核心差异玩法落地（§5/§10，零新增引擎能力）：
//!   ① BOSS 剑冢之灵 →「选择驱动遭遇链」（参考 scenes_zhouyuan zy_boss_round / scenes_jiguancheng
//!      jg_colossus_round）：战斗内每回合读取 GameState.san，实现 心魔加持(san≤40) / 剑心不稳(san≥60)
//!      / 心境共鸣(san≥80) / 万剑归冢狂暴(@70，每回 4-6 剑气) / 问心一剑终结(第5回合后，jz_heart_clean 或 san≥50)；
//!      同时导出 `jz_sword_spirit` FightCfg 供 ZoneDef 声明式引用。
//!   ② 心魔镜像战 克制当前武器 ：幻影战前 Dyn 文本读 GameState.weapon 给描述；数值按 §4 表基线
//!      （克制加成已折入 HP/伤害基线），三场幻影(初级/高手/剑主)击败合置 jz_mirror_all。
//!   ③ 拔剑/不拔剑双结局：boss 胜利→ jz_50_choice 二选一（互斥 flag jz_took_sword / jz_spare_sword +
//!      AddItem 无名剑/i jz_spare_sword San+15），分幕文案后进结算卡片（sp_grade=D）。
//!
//! ★背景素材已落地（新图 jianzhong_bg_*，素材子代理生成接线）：
//!   L1 山门古道 → jianzhong_bg_open.png   （入口/开场）
//!   L2 埋剑长廊 / L3 剑冢深谷 → jianzhong_bg_invest.png（调查/探索）
//!   L4 无名剑碑 / BOSS → jianzhong_bg_battle.png（战斗/核心）
//! 敌人立绘复用 §4：guard→剑仆、hunter→入魔客、zombie→灰袍剑仆、horde→巡山群像；
//! 镜像/BOSS/怨灵等新美术由主 agent 统一排期生图替换。

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

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_oldman_trust(st: &GameState) -> bool { st.flag("jz_oldman_trust") }
fn cond_has_rust_key(st: &GameState) -> bool { inv(st, "it_rust_key") }
fn cond_has_heart_clean(st: &GameState) -> bool { st.flag("jz_heart_clean") }
fn cond_low_san(st: &GameState) -> bool { st.san < 30 }
/// BOSS 前对话：为故人旧案需已拿旧案线索
fn cond_has_old_case(st: &GameState) -> bool { st.flag("jz_old_case") }

/* =====================================================================
   结算支线 side 统计（§7：6 支线，各 +200）
   ===================================================================== */
const SIDE_FLAGS: [&str; 6] = [
    "jz_old_case", "jz_heart_clean", "jz_mirror_all",
    "jz_spare_sword", "jz_san_keep", "jz_stele_secret",
];
fn side_count(st: &GameState) -> i32 { SIDE_FLAGS.iter().filter(|k| st.flag(k)).count() as i32 }

/* =====================================================================
   BOSS · 剑冢之灵（选择驱动遭遇）
   血量存 st.fight（jz_40_boss 的 Route::Dyn 初始化，引用 jz_sword_spirit 的 FightCfg）。
   每"回"是 Normal 场景 jz_boss_round；Route::Dyn 统一处理：扣血、心境 san 判定、狂暴、胜负路由。
   心境对决（§5）：每回合读 st.san ——
     san≤40 心魔加持(dmg 上限+4，每3回心魔剑 20-26 + San-10)；san≥60 剑心不稳(BOSS dmg-2)；
     san≥80 第3回合后"心境共鸣"回 hp+15（一次性）；狂暴 @HP≤70 万剑归冢每回 4-6 剑气。
   问心一剑终结：第5回合后若 (jz_heart_clean || san≥50) 可选，直接终结；san<30 被"心魔蔽目"替换。
   ===================================================================== */
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jz_sword_spirit") {
            st.fight = Some(crate::power::scaled_fight("jz_sword_spirit", cfg, st, vec![]));
        }
    }
    st.set_flag("jz_brd_1"); // 首回合计数
    "jz_boss_round".to_string()
}

/// 已进行的回合数（累计 flag jz_brd_1..jz_brd_8）
fn boss_rounds(st: &GameState) -> i32 {
    (1..=8).filter(|i| st.flag(&format!("jz_brd_{i}"))).count() as i32
}

/// 递增回合计数（jz_brd_n 顺序点亮）
fn boss_bump_round(st: &mut GameState) {
    for i in (1..8).rev() {
        if st.flag(&format!("jz_brd_{i}")) {
            st.set_flag(&format!("jz_brd_{}", i + 1));
            return;
        }
    }
    st.set_flag("jz_brd_1");
}

/// BOSS 胜利：+600、置 jz_boss_down、sp_grade=D → 结局抉择
fn boss_settle_win(st: &mut GameState) -> String {
    st.points += 600;
    st.set_flag("jz_boss_down");
    st.sp_grade = Some('D');
    "jz_50_choice".to_string()
}

fn boss_dead() -> String { "jz_99_death_boss".to_string() }

/// "心魔蔽目"（san<30 时替换问心一剑的不可用项）
fn boss_mind_shadow(st: &mut GameState) -> String {
    st.san = (st.san - 8).max(0);
    if st.san <= 0 { return "jz_99_sancollapse".to_string(); }
    "jz_boss_round".to_string()
}

/// "问心一剑"：真实伤害 60 + 终结战斗（§5 终结技条件命中即终结）
fn boss_finisher(st: &mut GameState) -> String {
    boss_settle_win(st)
}

/// 一个"回"：玩家攻击。心魔剑三回合判定用 jz_b1/b2/b3 循环。心境共鸣一次性。
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    // BOSS 受击
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return boss_settle_win(st);
    }
    // 狂暴：HP≤70
    if st.fight.as_ref().map(|f| f.hp <= 70).unwrap_or(false) {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    // 万剑归冢：狂暴每回追加 4-6 剑气
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    // san 心境决定 BOSS 出手
    let (mut lo, mut hi) = st.fight.as_ref().map(|f| f.dmg).unwrap_or((18, 26));
    let san = st.san;
    let mut sword_vision = false;
    if san <= 40 {
        // 心魔加持：伤害上限+4
        hi += 4;
    } else if san >= 60 {
        // 剑心不稳：伤害-2
        lo = (lo - 2).max(1);
        hi = (hi - 2).max(lo);
    }
    // 心境共鸣：san≥80 且第3回合后，一次性回 hp 15
    if san >= 80 && boss_rounds(st) >= 3 && !st.flag("jz_boss_resonance") {
        st.hp = (st.hp + 15).min(100);
        st.set_flag("jz_boss_resonance");
    }
    // 心魔剑：san≤40 每3回合（jz_b1/b2/b3 循环）
    if san <= 40 && st.flag("jz_b3") {
        let d = rng(20, 26);
        st.hp = (st.hp - d).max(0);
        st.san = (st.san - 10).clamp(0, 100);
        sword_vision = true;
        // 重置心魔剑周期
        st.flags.insert("jz_b1".to_string(), true);
        st.flags.insert("jz_b2".to_string(), false);
        st.flags.insert("jz_b3".to_string(), false);
    } else if san <= 40 {
        if st.flag("jz_b2") { st.set_flag("jz_b3"); }
        else if st.flag("jz_b1") { st.set_flag("jz_b2"); }
        else { st.set_flag("jz_b1"); }
    }
    // 狂暴剑气追加
    if raged {
        st.hp = (st.hp - rng(4, 6)).max(0);
    }
    // BOSS 反击（guard 大幅闪避）
    let raw = rng(lo, hi);
    let dodge = if guard { 0.55 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return boss_dead();
    }
    let _ = sword_vision;
    boss_bump_round(st);
    "jz_boss_round".to_string()
}

/// 问心一剑终结技可用：第5回合后 + (jz_heart_clean || san≥50)
fn cond_finisher_ready(st: &GameState) -> bool {
    boss_rounds(st) >= 5 && (st.flag("jz_heart_clean") || st.san >= 50)
}
/// 心魔蔽目替换：san<30 时问心一剑不可用
fn cond_finisher_blinded(st: &GameState) -> bool {
    boss_rounds(st) >= 5 && st.san < 30
}

/* =====================================================================
   梦境化镜像战（克制当前武器·Dyn 文本）
   ===================================================================== */
fn mirror_route_all(st: &mut GameState) -> String {
    if st.flag("jz_mirror_1") && st.flag("jz_mirror_2") && st.flag("jz_mirror_apex") {
        st.set_flag("jz_mirror_all");
    }
    "jz_10_arrive_l2".to_string()
}
fn mirror_win_1(_st: &GameState) -> String { "jz_13_mirror1_win".to_string() }
fn mirror_win_2(_st: &GameState) -> String { "jz_23_mirror2_win".to_string() }
fn mirror_win_apex(_st: &GameState) -> String { "jz_33_mirror_apex_win".to_string() }

/* =====================================================================
   无法克制的敌人 win 回调（FightCfg.win）
   ===================================================================== */
fn jz_win_l1(_st: &GameState) -> String { "jz_01".to_string() }
fn jz_win_patrol(st: &GameState) -> String {
    // 强闯获胜亦获守陵人信任
    let _ = st;
    "jz_03_oldman_rush_win".to_string()
}
fn jz_win_l2(_st: &GameState) -> String { "jz_10_arrive_l2".to_string() }
fn jz_win_l3(_st: &GameState) -> String { "jz_20_arrive_l3".to_string() }
fn jz_win_l4(_st: &GameState) -> String { "jz_30_arrive_l4".to_string() }
fn jz_win_rust(_st: &GameState) -> String { "jz_14_rust_win".to_string() }
fn jz_win_echo(_st: &GameState) -> String { "jz_15_echo_win".to_string() }
fn jz_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 战斗配置表（id 全部 jz_ 前缀；BOSS/镜像为选择驱动，其余走引擎原生 FightCfg）。
pub fn jianzhong_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("jz_servant", FightCfg {
            name: "守墓剑仆·灰袍", hp: 36, dmg: (7, 13), reward: 12, reward_why: "肃清守墓剑仆·灰袍",
            intro: "灰袍佝偻的老仆怀抱锈剑，垂着目缓缓抬头——雾中他的眼神是一片荒芜。",
            rage_at: Some(15), rage_text: "垂死挥剑乱舞，锈刃如雨！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_l1, death: "jz_99_death",
        }),
        ("jz_sentry", FightCfg {
            name: "守墓剑仆·执戈", hp: 38, dmg: (8, 14), reward: 12, reward_why: "守墓剑仆·执戈",
            intro: "执长戈的灰袍剑仆横在道口，戈刃上锈迹凝成暗红。",
            rage_at: Some(15), rage_text: "戈鸣示警，满山剑鸣回应——你成了众矢之的！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_l1, death: "jz_99_death",
        }),
        ("jz_patrol", FightCfg {
            name: "巡山剑仆·三人", hp: 42, dmg: (9, 15), reward: 15, reward_why: "击退巡山剑仆",
            intro: "三名灰袍剑仆并肩巡来，剑尖齐指，呈一个不散的三角阵。",
            rage_at: Some(18), rage_text: "结阵合击，剑势如潮！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_patrol, death: "jz_99_death",
        }),
        ("jz_echo", FightCfg {
            name: "剑鸣回响", hp: 40, dmg: (8, 14), reward: 15, reward_why: "平息剑鸣回响",
            intro: "断剑龛里一声锐响，满廊枯剑同时震鸣——剑鸣仿佛有了性命，朝你扑来。",
            rage_at: None, rage_text: "", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_echo, death: "jz_99_death",
        }),
        ("jz_rust", FightCfg {
            name: "锈剑傀儡", hp: 46, dmg: (10, 16), reward: 20, reward_why: "击碎锈剑傀儡",
            intro: "由断剑与铁片拼成的傀儡站起身，关节锈红，眼窝里嵌着碎裂的剑刃。",
            rage_at: Some(20), rage_text: "锈剑碎裂·伪剑意迸散！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_rust, death: "jz_99_death",
        }),
        ("jz_wraith_faint", FightCfg {
            name: "剑冢怨灵·游魂", hp: 52, dmg: (11, 17), reward: 30, reward_why: "平息剑冢怨灵·游魂",
            intro: "半透明的人形拖着一条剑形的虚影，无面，凌空朝你飘来。",
            rage_at: Some(25), rage_text: "怨灵附体·夺剑——你握兵器的虎口一阵发麻！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_l2, death: "jz_99_death",
        }),
        ("jz_wraith", FightCfg {
            name: "剑冢怨灵·深谷", hp: 60, dmg: (13, 19), reward: 45, reward_why: "平息剑冢怨灵·深谷",
            intro: "深谷幽暗里，一具剑形怨灵自石冢渗出，无面人形拖曳着剑气。",
            rage_at: Some(28), rage_text: "万剑恸哭，谷壁都在低鸣！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_l3, death: "jz_99_death",
        }),
        ("jz_sword_mad", FightCfg {
            name: "入魔剑客·残影", hp: 78, dmg: (15, 21), reward: 80, reward_why: "镇杀入魔剑客·残影",
            intro: "血瞳的黑红剑袍自雾中凝实，剑身上缠着黑气——他残影般扑来，双线斩落。",
            rage_at: Some(35), rage_text: "魔剑噬主，剑气暴涨！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_l3, death: "jz_99_death",
        }),
        ("jz_stele_guard", FightCfg {
            name: "剑碑守卫·双剑", hp: 86, dmg: (16, 22), reward: 100, reward_why: "剑碑守卫·双剑",
            intro: "铁面重甲的双剑守卫横在剑碑前，两柄剑交叉于背，披着一身夕照。",
            rage_at: Some(40), rage_text: "双剑归碑，剑气凛冽！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jz_win_l4, death: "jz_99_death",
        }),
        // 镜像三战（克制当前武器：数值基线含加成，Dyn 文本读武器）
        ("jz_phantom_1", FightCfg {
            name: "剑心幻影·初级", hp: 50, dmg: (12, 18), reward: 35, reward_why: "破剑心幻影·初级（镜像）",
            intro: "试剑龛里凝出一道持剑剪影，周身金线剑意环绕——脸，是一面反光的剑面。",
            rage_at: Some(22), rage_text: "镜像侵蚀——它看穿了你的剑路！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mirror_win_1, death: "jz_99_death",
        }),
        ("jz_phantom_2", FightCfg {
            name: "剑心幻影·高手剑客", hp: 68, dmg: (14, 20), reward: 60, reward_why: "破剑心幻影·高手（镜像）",
            intro: "你面前站着一个持剑的自己——他笑你握剑的模样，与当年那人一般可笑。",
            rage_at: Some(30), rage_text: "镜像侵蚀·进阶——剑意翻倍！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mirror_win_2, death: "jz_99_death",
        }),
        ("jz_phantom_apex", FightCfg {
            name: "剑心幻影·剑主", hp: 92, dmg: (17, 23), reward: 110, reward_why: "破剑心幻影·剑主（镜像）",
            intro: "万剑合拢成一道白发人影的剪影——剑主残影，万千剑意归一。",
            rage_at: Some(45), rage_text: "剑主残影·万剑归一！", on_rage: jz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mirror_win_apex, death: "jz_99_death",
        }),
        // BOSS 剑冢之灵（选择驱动；此 FightCfg 供 ZoneDef / 揭示引用）
        ("jz_sword_spirit", FightCfg {
            name: "剑冢之灵", hp: 150, dmg: (18, 26), reward: 600, reward_why: "剑冢之灵 · 心境对决",
            intro: "万剑齐鸣，碑中凝出一道白发人影，金瞳如落日。「这一剑，为谁而挥？」",
            rage_at: Some(70), rage_text: "千年剑意，尽归我身——场上剑气风暴，每回合追加剑气伤害！",
            on_rage: |_st, _log| {},
            finisher_if: |st, _| cond_finisher_ready(st),
            finisher_name: |_| "问心一剑".to_string(),
            finisher_desc: |st| {
                if st.flag("jz_heart_clean") {
                    "你闭目，任剑鸣穿过身体，亿万浮光在剑尖凝成一线——一剑问心，剑气直落剑冢之灵眉心。".to_string()
                } else {
                    "你深吸一口气，把百年剑冢的执念尽数沉入心底，一剑问心而出。".to_string()
                }
            },
            win: |_st| "jz_50_choice".to_string(),
            death: "jz_99_death_boss",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn jzh_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    jianzhong_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 jz_ 前缀）
   ===================================================================== */
pub static JIANZHONG_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 开场 山门古道（L1）================= */
SceneDef {
    id: "jz_00", bg: Some("bg_jz_l1_shanmen.png"), loc: Some("剑冢 · 山门古道"),
    mood: "mystery", speaker: Some("守陵人"), voice: Some("vo_jz_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>潜入千年禁地剑冢，承受剑意压迫、破心魔镜像，直面剑冢之灵，带着答案全身而退。失败代价：被扣 200 点复活。",
        "雾从石坊后涌出来，像一匹浸了水的灰布。守陵人抱着一柄没鞘的锈剑，抬眼看了看你，又看了看你腰间的兵器：「此间非江湖，剑冢不留无名客。你，为谁而来？」",
    ]),
    choices: &[
        ChoiceDef { label: "亮出主神手环", sub: "San+5 · +10 点 · 获信任", cond: None,
            effects: &[Eff::SetFlag("jz_oldman_trust"), Eff::San(5), Eff::Points(10)], route: Route::To("jz_00_trust") },
        ChoiceDef { label: "行礼求入：为凭吊", sub: "flag jz_intent_sword · 得山门令", cond: None,
            effects: &[Eff::SetFlag("jz_intent_sword")], route: Route::To("jz_00_bow") },
        ChoiceDef { label: "行礼求入：为旧案", sub: "flag jz_intent_case · 得山门令", cond: None,
            effects: &[Eff::SetFlag("jz_intent_case")], route: Route::To("jz_00_bow") },
        ChoiceDef { label: "强闯山门", sub: "战巡山剑仆 · San-5", cond: None,
            effects: &[Eff::San(-5)], route: Route::To("jz_00_rush") },
    ],
    fight_id: None, video: Some("vid_jz_opening.mp4"), cine_label: Some("过场 · 山门古道"), overlay: None,
},
SceneDef {
    id: "jz_00_trust", bg: Some("jianzhong_bg_open.png"), loc: Some("剑冢 · 山门古道"),
    mood: "calm", speaker: Some("守陵人"), voice: None,
    text: TextSpec::Static(&["你撩起袖口露出主神手环。守陵人神色一凝：「……你身上有外面的味道。」他侧身让出一步：「进去吧，此间剑，辨得出谁是同道。」"]),
    choices: &[ChoiceDef { label: "（入内）", sub: "→ L1 山门古道", cond: None, effects: &NO_EFF, route: Route::To("jz_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_00_bow", bg: Some("jianzhong_bg_open.png"), loc: Some("剑冢 · 山门古道"),
    mood: "calm", speaker: Some("守陵人"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("jz_intent_sword") {
            "你躬身一礼：「此去，为凭吊一柄千年前的名剑。」守陵人眸子一缩，从怀里取出一枚黄铜令牌递给你：「带着它过石坊，剑不拦真心人。」".to_string()
        } else {
            "你躬身一礼：「此去，为求一桩千年前的旧案真相。」守陵人凝视你良久，取出一枚黄铜令牌：「……难得还有人记得。带着它，去听剑说话。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "收下山门令", sub: "Item it_shanmen_ling · 获信任", cond: None,
        effects: &[Eff::AddItem("it_shanmen_ling"), Eff::SetFlag("jz_oldman_trust")], route: Route::To("jz_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_00_rush", bg: Some("jianzhong_bg_open.png"), loc: Some("剑冢 · 山门古道 · 强闯"),
    mood: "danger", speaker: Some("巡山剑仆"), voice: Some("vo_jz_rush"),
    text: TextSpec::Static(&["守陵人一声冷笑，雾里三柄剑同时出鞘——巡山剑仆·三人结阵拦下你的去路！（战斗）"]),
    choices: &[], fight_id: Some("jz_patrol"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_03_oldman_rush_win", bg: Some("jianzhong_bg_open.png"), loc: Some("剑冢 · 山门古道"),
    mood: "cold", speaker: Some("守陵人"), voice: None,
    text: TextSpec::Static(&["三柄剑仆伏于雾中。守陵人拊掌而叹：「剑意虽强，情亦有道。」他不拦你：「强闯者已过石坊，还需我放行么？」（已获守陵人信任）"]),
    choices: &[ChoiceDef { label: "登石阶入内", sub: "→ L1 山门古道", cond: None, effects: &NO_EFF, route: Route::To("jz_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 山门古道 hub ================= */
SceneDef {
    id: "jz_01", bg: Some("bg_jz_l1_shanmen.png"), loc: Some("L1 · 山门古道"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "冷雾锁着残破的石坊。守陵人的小屋在西南，碑林在北，一条荒径隐入西侧浓雾。主道甬道直通石阶，石阶之上便是埋剑长廊。",
        "（同行的小棠攥着剑穗一路无言——她是你此战的同伴，活到最后算存活队友 +100。）",
    ]),
    choices: &[
        ChoiceDef { label: "访守陵人", sub: "NPC · 赠气血丹", cond: None, effects: &NO_EFF, route: Route::To("jz_03_oldman") },
        ChoiceDef { label: "荒径断碑", sub: "旧案线索 1/2", cond: None, effects: &NO_EFF, route: Route::To("jz_02_huangjing") },
        ChoiceDef { label: "碑林残刻", sub: "调查 · 气血丹", cond: None, effects: &NO_EFF, route: Route::To("jz_02_beilin") },
        ChoiceDef { label: "登石阶而上", sub: "需守陵人信任/山门令 → L2 埋剑长廊", cond: Some(cond_oldman_trust), effects: &NO_EFF, route: Route::To("jz_06_arrive_l2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_02_huangjing", bg: Some("jianzhong_bg_open.png"), loc: Some("L1 · 荒径断碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["西侧荒径的断碑上，几行剑刻潦草得近乎剥落，却隐约拼出一个名字与一个年份——千年前剑主的旧案，似乎在这条无人走的小径上留有痕迹。"]),
    choices: &[ChoiceDef { label: "拓下断碑残文", sub: "旧案线索 1/2 · +15 点", cond: None,
        effects: &[Eff::SetFlag("jz_old_case_1"), Eff::MarkPoint("jz_p_huangjing"), Eff::Points(15)],
        route: Route::Dyn(route_old_case_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_02_beilin", bg: Some("jianzhong_bg_open.png"), loc: Some("L1 · 碑林"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["碑林里立着千百块无名的残碑，尽数指向山谷深处。一方祭台角落压着一枚丹丸。"]),
    choices: &[ChoiceDef { label: "取金创药·气血丹", sub: "Item it_qixue_dan · HP+30", cond: None,
        effects: &[Eff::AddItem("it_qixue_dan"), Eff::MarkPoint("jz_p_beilin")], route: Route::To("jz_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_03_oldman", bg: Some("jianzhong_bg_open.png"), loc: Some("L1 · 守陵人小屋"),
    mood: "cold", speaker: Some("守陵人"), voice: None,
    text: TextSpec::Static(&["守陵人小屋的木门半掩，他坐在灯下磨一柄锈剑。见你来，他抬眼：「……里面越深，剑意越压人。你身侧那位小棠姑娘，怕是要撑不住心神。——这枚药，带她也好。」"]),
    choices: &[ChoiceDef { label: "收下气血丹", sub: "Item it_qixue_dan", cond: None,
        effects: &[Eff::AddItem("it_qixue_dan")], route: Route::To("jz_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_06_arrive_l2", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 埋剑长廊"),
    mood: "cold", speaker: Some("小棠"), voice: None,
    text: TextSpec::Static(&["你踏上石阶，雾在脚下散开，露出一条极长的长廊。长廊两侧插着千百柄剑，锈得只剩轮廓，却每一柄都在低鸣。小棠攥住你的袖口：「师兄……这些剑，在哭。」"]),
    choices: &[
        ChoiceDef { label: "剑意压迫·运功抵御", sub: "San-15 · 气血翻涌", cond: None,
            effects: &[Eff::San(-15)], route: Route::To("jz_10_arrive_l2") },
        ChoiceDef { label: "剑意压迫·强行突破", sub: "HP-10", cond: None,
            effects: &[Eff::Hurt(10, "jz_99_death")], route: Route::To("jz_10_arrive_l2") },
        ChoiceDef { label: "剑意压迫·静心打坐", sub: "San+10 · 剑心清明 · 解深谷石门", cond: None,
            effects: &[Eff::San(10), Eff::SetFlag("jz_heart_clean")], route: Route::To("jz_10_arrive_l2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 埋剑长廊 hub ================= */
SceneDef {
    id: "jz_10_arrive_l2", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 埋剑长廊"),
    mood: "cold", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        let heart = if st.flag("jz_heart_clean") { "你的剑心澄明，剑鸣自你身侧让开一条路。" } else { "剑意尚未驯服——长廊中段那扇锈锁铁门之后，还有更深的长廊。" };
        format!("{heart}（藏剑龛在左一、试剑龛在左二；断剑龛与铭剑龛在右侧。末碑厅在长廊尽头。）")
    }),
    choices: &[
        ChoiceDef { label: "藏剑龛 · 剑穗", sub: "纪念/兑换凭证", cond: None, effects: &NO_EFF, route: Route::To("jz_11_sword_tassel") },
        ChoiceDef { label: "试剑龛 · 剑心幻影", sub: "镜像战 · 克制当前武器", cond: None, effects: &NO_EFF, route: Route::To("jz_12_mirror1") },
        ChoiceDef { label: "铭剑龛 · 剑铭", sub: "旧案真相 · 得锈锁钥匙", cond: None, effects: &NO_EFF, route: Route::To("jz_13_jianming") },
        ChoiceDef { label: "剑鸣回响（断剑龛）", sub: "遭遇战", cond: None, effects: &NO_EFF, route: Route::To("jz_15_echo") },
        ChoiceDef { label: "开锈锁铁门", sub: "需锈锁钥匙 → 长廊中段", cond: Some(cond_has_rust_key), effects: &NO_EFF, route: Route::To("jz_16_open_gate_l2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_11_sword_tassel", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 藏剑龛"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["藏剑龛的暗格里压着一枚褪色的剑穗，红缨早已沉成暗褐——那是千年前剑主佩剑的遗物。"]),
    choices: &[ChoiceDef { label: "收起剑穗", sub: "Item it_sword_tassel · 解锁藏剑龛暗格", cond: None,
        effects: &[Eff::AddItem("it_sword_tassel"), Eff::SetFlag("jz_sword_tassel")],
        route: Route::Dyn(route_shortcut) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_12_mirror1", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 试剑龛 · 剑心幻影"),
    mood: "danger", speaker: Some("剑心幻影"), voice: Some("vo_jz_mirror1"),
    text: TextSpec::Dyn(|st| {
        let w = match st.weapon {
            Some(crate::state::Weapon::Axe) => "一柄斧",
            Some(crate::state::Weapon::Gun) => "一柄火铳",
            Some(crate::state::Weapon::Sword) => "一柄短刃",
            None => "那只空拳",
        };
        format!("试剑石前凝出一道持剑剪影，金线剑意环绕周身，脸是反光的剑面。它盯着你手里的{w}，低笑：「你握剑的模样，与当年那人，一般可笑。」（镜像克制当前武器，剑路已被看穿！战斗）")
    }),
    choices: &[], fight_id: Some("jz_phantom_1"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_13_mirror1_win", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 试剑龛"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["剑心幻影碎成点点金光，剑面上的那双眼随之一同湮灭。你破了第一道心魔——但更深处的黑影，仿佛在长廊尽头张望。"]),
    choices: &[ChoiceDef { label: "（继续前行）", sub: "jz_mirror_1 ✔", cond: None,
        effects: &[Eff::SetFlag("jz_mirror_1")], route: Route::Dyn(mirror_route_all) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_13_jianming", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 铭剑龛 · 剑铭"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "铭剑龛的剑身上刻着千年前的旧案：剑主「背剑离山」被师门疑为叛徒，自戮于剑碑之下；此案疑点重重——剑铭末尾一行小字：「他从未背叛，他只是……不肯说出那晚在剑冢见到的人。」",
        "（锈锁铁门的钥匙，正压在这方剑铭之下。）",
    ]),
    choices: &[ChoiceDef { label: "取锈锁钥匙并拓剑铭", sub: "旧案线索 2/2 · Item it_rust_key", cond: None,
        effects: &[Eff::AddItem("it_rust_key"), Eff::SetFlag("jz_old_case_2"), Eff::MarkPoint("jz_p_jianming"), Eff::Points(30)],
        route: Route::Dyn(route_old_case) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_14_rust_win", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 长廊"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["锈剑傀儡瘫作一地碎铁，眼窝里的剑刃失了光。长廊归于剑鸣的呜咽。"]),
    choices: &[ChoiceDef { label: "（继续）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("jz_10_arrive_l2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_15_echo", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 断剑龛 · 剑鸣回响"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jz_echo"),
    text: TextSpec::Static(&["断剑龛里一声锐响，满廊枯剑同时震鸣——剑鸣化成一道人形镌影，向你涌来！（战斗）"]),
    choices: &[], fight_id: Some("jz_echo"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_15_echo_win", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 断剑龛"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["剑鸣回响散去，满廊枯剑复归沉寂。你平复着翻涌的气血，望向长廊尽头那扇锈锁铁门。"]),
    choices: &[ChoiceDef { label: "（折返）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("jz_10_arrive_l2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_16_open_gate_l2", bg: Some("jianzhong_bg_invest.png"), loc: Some("L2 · 长廊中段"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["锈锁铁门在你身后无声合拢，你已置身长廊的中后段——剑柱更密，剑鸣更沉。尽头是通往深谷的石阶。"]),
    choices: &[ChoiceDef { label: "登石阶下赴深谷", sub: "→ L3 剑冢深谷", cond: Some(cond_has_heart_clean), effects: &NO_EFF, route: Route::To("jz_20_arrive_l3") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 剑冢深谷 ================= */
SceneDef {
    id: "jz_20_arrive_l3", bg: Some("jianzhong_bg_invest.png"), loc: Some("L3 · 剑冢深谷"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        let heart = if st.flag("jz_heart_clean") {
            "你的剑心澄明，深谷的剑意虽沉，却没能压住你的呼吸。谷顶平台的石冢列在夕照之外，幽暗中泛着冷光。"
        } else {
            "深谷的剑意是沉甸甸的实铁，压在你肩头。你隐约听见石门之后有人声在问——「你怕了吗？」（需静心打坐/静心石 安定剑心，方能开深谷石门。）"
        };
        format!("{heart}（石冢@(10,14)藏静心石；谷壁剑痕@(25,18)是旧案另一处刻痕。）")
    }),
    choices: &[
        ChoiceDef { label: "深谷石冢 · 静心石", sub: "San+15 · 解深谷石门", cond: None, effects: &NO_EFF, route: Route::To("jz_21_shizhong") },
        ChoiceDef { label: "谷壁剑痕", sub: "San-10 · 旧案刻痕", cond: None, effects: &NO_EFF, route: Route::To("jz_22_sword_marks") },
        ChoiceDef { label: "【心魔显影】", sub: "san<30 强制镜像战 · 高手", cond: Some(cond_low_san), effects: &NO_EFF, route: Route::To("jz_23_mirror2") },
        ChoiceDef { label: "开深谷石门", sub: "需剑心清明 / 静心石 → 深谷下半+天梯", cond: Some(cond_has_heart_clean), effects: &NO_EFF, route: Route::To("jz_24_open_gate_l3") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_21_shizhong", bg: Some("jianzhong_bg_invest.png"), loc: Some("L3 · 深谷石冢"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["一方石冢供着一枚温润的静心石，石纹如水。你握住它，体内翻涌的剑意霎时平息。"]),
    choices: &[ChoiceDef { label: "取静心石", sub: "Item it_jingxin_stone · 剑心清明", cond: None,
        effects: &[Eff::AddItem("it_jingxin_stone"), Eff::SetFlag("jz_heart_clean"), Eff::MarkPoint("jz_p_shizhong")],
        route: Route::To("jz_20_arrive_l3") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_22_sword_marks", bg: Some("jianzhong_bg_invest.png"), loc: Some("L3 · 谷壁剑痕"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["谷壁上一道深可没指的剑痕，正是当日剑主自戮前最后一剑的落点。剑气扑面，你的心神一阵刺痛。"]),
    choices: &[ChoiceDef { label: "辨认剑痕", sub: "San-10 · 旧案线索", cond: None,
        effects: &[Eff::San(-10), Eff::SetFlag("jz_sword_marks"), Eff::MarkPoint("jz_p_sword_marks")],
        route: Route::To("jz_20_arrive_l3") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_23_mirror2", bg: Some("jianzhong_bg_invest.png"), loc: Some("L3 · 深谷 · 心魔显影"),
    mood: "danger", speaker: Some("剑心幻影·高手"), voice: Some("vo_jz_mirror2"),
    text: TextSpec::Dyn(|st| {
        let w = match st.weapon {
            Some(crate::state::Weapon::Axe) => "那柄斧",
            Some(crate::state::Weapon::Gun) => "那柄火铳",
            Some(crate::state::Weapon::Sword) => "那柄短刃",
            None => "那只空拳",
        };
        format!("深谷寒雾猛地一凝，一个和你同样呼吸、同样持着{w}的身影自雾中走出：「你握剑的模样，与当年那人，一般可笑。」——你听见了自己心跳错乱的声音。（镜像克制当前武器！战斗）")
    }),
    choices: &[], fight_id: Some("jz_phantom_2"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_23_mirror2_win", bg: Some("jianzhong_bg_invest.png"), loc: Some("L3 · 深谷"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["心魔影像在你剑下轰然溃散，化作一声不甘的叹息没入石壁。你抹去额角的汗——心已澄，剑始鸣。"]),
    choices: &[ChoiceDef { label: "（定心）", sub: "jz_mirror_2 ✔ · +20 点", cond: None,
        effects: &[Eff::SetFlag("jz_mirror_2"), Eff::Points(20)], route: Route::Dyn(mirror_route_all) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_24_open_gate_l3", bg: Some("jianzhong_bg_invest.png"), loc: Some("L3 · 深谷石门已成"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["深谷石门的字迹随一声长叹隐去，门缝里的寒气涌出。你穿过石门，沿绝壁栈道折返而下，尽头是天梯——通往无名剑碑之巅。"]),
    choices: &[ChoiceDef { label: "登天梯而上", sub: "→ L4 无名剑碑之巅", cond: None, effects: &NO_EFF, route: Route::To("jz_30_arrive_l4") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L4 无名剑碑之巅 ================= */
SceneDef {
    id: "jz_30_arrive_l4", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑之巅"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "夕照像熔金一样泼在整面剑碑上，万剑低鸣如潮。碑前的广场上，两尊铁面守卫横剑而立。远处的断崖，一脚踏空便是坠回深谷的路。",
        "（东/西剑痕碑与北/南残碑暗藏剑碑隐文，三处剑痕 + 北残碑可破译。）",
    ]),
    choices: &[
        ChoiceDef { label: "北残碑", sub: "破译剑碑隐文 线索", cond: None, effects: &NO_EFF, route: Route::To("jz_31_stele_north") },
        ChoiceDef { label: "东剑痕碑", sub: "隐文 1/3", cond: None, effects: &NO_EFF, route: Route::To("jz_31_stele_east") },
        ChoiceDef { label: "西剑痕碑", sub: "隐文 2/3", cond: None, effects: &NO_EFF, route: Route::To("jz_31_stele_west") },
        ChoiceDef { label: "南残碑", sub: "隐文 3/3", cond: None, effects: &NO_EFF, route: Route::To("jz_31_stele_south") },
        ChoiceDef { label: "迎战剑碑守卫", sub: "双剑守卫", cond: None, effects: &NO_EFF, route: Route::To("jz_32_guard") },
        ChoiceDef { label: "断崖捷径", sub: "坠崖回 L3 · HP-5 San-5", cond: None, effects: &NO_EFF, route: Route::To("jz_33_cliff") },
        ChoiceDef { label: "无名剑碑 · 对峙", sub: "BOSS 前对话 → 决战", cond: None, effects: &NO_EFF, route: Route::To("jz_41_prequel") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_32_guard", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 剑碑广场"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jz_guard"),
    text: TextSpec::Static(&["两尊铁面双剑守卫同时出剑，交叉的剑气在你身周扫出一个圆（战斗）。"]),
    choices: &[], fight_id: Some("jz_stele_guard"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_33_cliff", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 断崖"),
    mood: "danger", speaker: None, voice: Some("vo_jz_cliff"),
    text: TextSpec::Static(&["你一脚踩滑，断崖边缘的石砾簌簌而下——你坠入谷底！（坠崖：HP-5、San-5，回深谷底部洞厅。）"]),
    choices: &[ChoiceDef { label: "（坠入深谷）", sub: "HP-5 · San-5", cond: None, effects: &NO_EFF, route: Route::Dyn(route_cliff) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_31_stele_north", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 北残碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["北面的残碑铭着一行古字，被风化得只剩半句：「……无名，故名无名。」你记下其残文。"]),
    choices: &[ChoiceDef { label: "拓下北残碑", sub: "破译线索 1 · +10 点", cond: None,
        effects: &[Eff::SetFlag("jz_stele_seen"), Eff::MarkPoint("jz_p_stele_north"), Eff::Points(10)],
        route: Route::To("jz_30_arrive_l4") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_31_stele_east", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 东剑痕碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["东侧剑痕碑上，一道剑痕恰好劈开一行铭文——你辨认出「背剑」二字。"]),
    choices: &[ChoiceDef { label: "拓东剑痕碑", sub: "隐文 1/3", cond: None,
        effects: &[Eff::SetFlag("jz_swordmark_east"), Eff::MarkPoint("jz_p_stele_east")],
        route: Route::Dyn(route_stele) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_31_stele_west", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 西剑痕碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["西侧剑痕碑上，剑痕劈开「离山」二字。残碑的风化处，还有一个小指印。"]),
    choices: &[ChoiceDef { label: "拓西剑痕碑", sub: "隐文 2/3", cond: None,
        effects: &[Eff::SetFlag("jz_swordmark_west"), Eff::MarkPoint("jz_p_stele_west")],
        route: Route::Dyn(route_stele) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_31_stele_south", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 南残碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["南侧残碑的刻痕与其它三处剑痕似有呼应——「那晚的剑冢，有第二个脚印」。"]),
    choices: &[ChoiceDef { label: "拓南剑痕碑", sub: "隐文 3/3", cond: None,
        effects: &[Eff::SetFlag("jz_swordmark_south"), Eff::MarkPoint("jz_p_stele_south")],
        route: Route::Dyn(route_stele) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_34_stele_secret", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 剑碑隐文破译"),
    mood: "calm", speaker: Some("旁白"), voice: Some("vo_jz_stele_secret"),
    text: TextSpec::Static(&["三处剑痕与北残碑的残文拼合，你读出了剑碑之下被抹去的另一行字：「那晚他并非叛出，而是替师门拦下了一个不该出现在剑冢的人。以死证清白——却无人知晓。」剑碑隐文，一朝破译。"]),
    choices: &[ChoiceDef { label: "（隐文入心）", sub: "jz_stele_secret ✔", cond: None, effects: &NO_EFF, route: Route::To("jz_30_arrive_l4") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 剑冢之灵 · BOSS 前对话 ---- */
SceneDef {
    id: "jz_41_prequel", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑 · 对峙"),
    mood: "mystery", speaker: Some("剑冢之灵"), voice: Some("vo_jz_spirit"),
    text: TextSpec::Static(&[
        "夕照泼在剑碑上，碑中凝出一道白发人影，金瞳如落日。他目光落在你身上，极轻地笑了一声：「每柄剑都是一段未了之愿。我等你，等了一千年。」",
        "他抬手，万剑齐鸣——「这一剑，为谁而挥？」小棠在你身后屏住呼吸。",
    ]),
    choices: &[
        ChoiceDef { label: "「为天下苍生。」", sub: "San-5", cond: None, effects: &[Eff::San(-5), Eff::SetFlag("jz_answer_world")], route: Route::To("jz_42_boss") },
        ChoiceDef { label: "「为故人旧案。」", sub: "San+5 · 需旧案真相", cond: Some(cond_has_old_case),
            effects: &[Eff::San(5), Eff::SetFlag("jz_answer_case")], route: Route::To("jz_42_boss") },
        ChoiceDef { label: "「为己心证道。」", sub: "San+5", cond: None, effects: &[Eff::San(5), Eff::SetFlag("jz_answer_self")], route: Route::To("jz_42_boss") },
        ChoiceDef { label: "闭口不答，拔剑而战", sub: "San-10 · BOSS 起手 dmg+2", cond: None,
            effects: &[Eff::San(-10), Eff::SetFlag("jz_answer_silent")], route: Route::To("jz_42_boss") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_42_boss", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑 · 决战"),
    mood: "danger", speaker: Some("剑冢之灵"), voice: Some("vo_jz_spirit_fight"),
    text: TextSpec::Dyn(|st| {
        let silent_penalty = if st.flag("jz_answer_silent") { "它因你的沉默动了怒，剑气凝滞沉重（起手 dmg+2）。" } else { "你答出心中所愿，剑灵的剑意波澜一滞，凝成了一柄无形之剑。." };
        format!("「好。那一剑，我便接下了。」万剑出鞘，天地间只剩剑鸣——{silent_penalty}（心境对决：按你的 san 定此战难易——san 低则它越打越凶，san 高则它反显不稳。）")
    }),
    choices: &[ChoiceDef { label: "【逼近剑碑】", sub: "进入心境对决", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_boss_round", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑 · 激战"),
    mood: "danger", speaker: Some("剑冢之灵"), voice: None,
    text: TextSpec::Dyn(|st| {
        let hp = st.fight.as_ref().map(|f| f.hp.max(0)).unwrap_or(150);
        let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
        let san = st.san;
        let mind = if raged {
            format!("<b style='color:#ff6a6a'>万剑归冢</b>——场上剑气风暴，每回合追加剑气伤害！")
        } else if san <= 40 {
            "<b>心魔加持</b>——剑意压心，你的剑在发抖；每三回合它会以「心魔剑」贯穿你（San-10）——".to_string()
        } else if san >= 60 {
            "<b style='color:#9acdff'>剑心不稳</b>——剑灵颔首「好澄明的心境」，剑势弱了几分——".to_string()
        } else { String::new() };
        let fin = if cond_finisher_ready(st) { "　「问心一剑」已在剑尖蓄势，只待一问！" } else { "" };
        format!("<b>剑冢之灵</b>　HP {hp}/150　·　你的心境 {san}/100\n\n{mind}{fin}")
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, rng(30, 42), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-28", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, rng(20, 28), false)) },
        ChoiceDef { label: "【问心一剑】", sub: "第5回合后 · 需剑心清明或 san≥50 · 终结", cond: Some(cond_finisher_ready),
            effects: &NO_EFF, route: Route::Dyn(boss_finisher) },
        ChoiceDef { label: "【心魔蔽目】", sub: "san<30 无法出问心一剑（San-8）", cond: Some(cond_finisher_blinded),
            effects: &NO_EFF, route: Route::Dyn(boss_mind_shadow) },
        ChoiceDef { label: "后撤凝神", sub: "提升闪避", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ==== 结局抉择 · 拔剑 / 不拔剑（双结局，互斥 flag）==== */
SceneDef {
    id: "jz_50_choice", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑"),
    mood: "calm", speaker: Some("剑冢之灵"), voice: Some("vo_jz_choice"),
    text: TextSpec::Static(&[
        "剑冢之灵在你面前一寸寸淡去，万剑低鸣渐止。那柄千年无名剑，静静插在碑前，剑身在夕照里流转着未熄的光。",
        "他望着你：「这一剑，你已答过。剩下的——剑在你手，走或不走，由你。」",
    ]),
    choices: &[
        ChoiceDef { label: "拔起无名剑", sub: "AddItem 无名剑 · San-10 · jz_took_sword", cond: None,
            effects: &[Eff::AddItem("it_wuming_sword"), Eff::San(-10), Eff::SetFlag("jz_took_sword")],
            route: Route::To("jz_51_ending_took") },
        ChoiceDef { label: "不拔剑", sub: "San+15 · jz_spare_sword · 剑灵释然", cond: None,
            effects: &[Eff::San(15), Eff::SetFlag("jz_spare_sword")], route: Route::To("jz_51_ending_spare") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_51_ending_took", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑"),
    mood: "calm", speaker: Some("小棠"), voice: None,
    text: TextSpec::Static(&["你握住剑柄，那柄剑在我掌心沉沉一坠，像一段迟来了千年的许诺。你带着剑走了——剑冢的呜咽，跟了你很远。小棠在夕照里低声说：「……也罢，剑在你手，便是答案。」"]),
    choices: &[ChoiceDef { label: "（下山 · 结算）", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_end_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_51_ending_spare", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 无名剑碑"),
    mood: "calm", speaker: Some("小棠"), voice: Some("vo_jz_spare"),
    text: TextSpec::Static(&["你没有拔剑。剑冢之灵在你面前笑了笑，化作一缕夕照，渐渐消散。「……去吧，带着你的答案。」万剑入土，剑气归山。小棠立在夕照里，望着你良久：「师兄，剑冢答应你的事……做到了吗？」"]),
    choices: &[ChoiceDef { label: "（下山 · 结算）", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_end_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_52_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_jz_settle"), death: None,
        card: |st| crate::state::Card {
            title: "剑 冢 · 一 剑 问 心".into(), good: true,
            body_html: format!(
                "<p>无名剑碑安静下来，万剑入土，剑气归山。你带着答案，踏出剑冢禁地。</p>\
                 <p style='color:#9a958a'>心境支线 {}/6 · {} · 无名剑碑隐文：{}</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 <tr><td>剑冢之语</td><td>「这一剑，为谁而挥？」——你已作答。</td></tr>\
                 </table>",
                side_count(st),
                if st.flag("jz_spare_sword") { "不拔剑 · 剑灵释然" } else { "拔剑而去 · 剑随君行" },
                if st.flag("jz_stele_secret") { "已破译" } else { "未尽" },
                st.points,
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 心魔镜像·剑主 apex（L4 断崖旁，可触发）================= */
SceneDef {
    id: "jz_33_mirror_apex", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 剑主幻影"),
    mood: "danger", speaker: Some("剑心幻影·剑主"), voice: Some("vo_jz_mirror_apex"),
    text: TextSpec::Dyn(|st| {
        let w = match st.weapon {
            Some(crate::state::Weapon::Axe) => "那柄斧",
            Some(crate::state::Weapon::Gun) => "那柄火铳",
            Some(crate::state::Weapon::Sword) => "那柄短刃",
            None => "那只空拳",
        };
        format!("朝你走来的，是千年前剑主的一缕残影——他持着同样的一柄{w}，万千剑意归一：「记住这一剑。」（镜像克制当前武器！战斗）")
    }),
    choices: &[], fight_id: Some("jz_phantom_apex"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jz_33_mirror_apex_win", bg: Some("jianzhong_bg_battle.png"), loc: Some("L4 · 剑主幻影"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["剑主残影在你剑下寸寸崩解，那一点万剑归一之意，落进你心底。你破尽了全部三道心魔。」"]),
    choices: &[ChoiceDef { label: "（万剑归一）", sub: "jz_mirror_apex ✔", cond: None,
        effects: &[Eff::SetFlag("jz_mirror_apex")], route: Route::Dyn(mirror_route_all) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 死亡档案（复活扣 200/回主神）================= */
SceneDef {
    id: "jz_99_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("万剑之下", "在剑冢的剑意与执念下倒下")), card: |_st| crate::state::Card {
            title: "万 剑 之 下".into(), good: false,
            body_html: r#"<p>无数柄枯剑般的剑意涌来，你的最后一声被剑鸣吞没。剑冢依旧低鸣，像从未有过你。</p>
<p style='color:#ff8a8a'>【死亡档案 · 万剑之下】</p>
<p style='color:#666'>（复活：回主神空间扣 200 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "jz_99_death_boss", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("剑冢之巅 · 万剑穿心", "剑冢之灵的一剑贯穿了你的心口")), card: |_st| crate::state::Card {
            title: "一 剑 穿 心".into(), good: false,
            body_html: r#"<p>万剑穿过你的躯壳，剑冢之巅的夕照被你的血色浸透。「……你的答案，还不够真。」这是你听见的最后一句话。</p>
<p style='color:#ff8a8a'>【死亡档案 · 剑冢之巅·万剑穿心】</p>
<p style='color:#666'>（复活：回主神空间扣 200 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "jz_99_sancollapse", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("深谷石门前 · 心魔噬心", "剑心沦丧，你被自己的执念吞没")), card: |_st| crate::state::Card {
            title: "心 魔 噬 心".into(), good: false,
            body_html: r#"<p>剑意与执念如沸水灌入识海，你看见那柄剑主残影在雾里朝你招手。你身前那座深谷石门，终究没能迈过。</p>
<p style='color:#ff8a8a'>【死亡档案 · 深谷石门前·心魔噬心】</p>
<p style='color:#666'>（复活：回主神空间扣 200 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* =====================================================================
   Route::Dyn 路由函数（供 static 数组使用，fn 指针）
   ===================================================================== */
/// 旧案线索集齐：荒径断碑(1) + 铭剑龛(2) → jz_old_case + 旧案信物
fn route_old_case(st: &mut GameState) -> String {
    if st.flag("jz_old_case_1") && st.flag("jz_old_case_2") && !st.flag("jz_old_case") {
        st.set_flag("jz_old_case");
        crate::world::add_item(st, "it_old_case_relic");
    }
    "jz_10_arrive_l2".to_string()
}

/// 荒径断碑（L1）拓文后返回 L1 hub（旧案线索计数同 route_old_case）
fn route_old_case_l1(st: &mut GameState) -> String {
    if st.flag("jz_old_case_1") && st.flag("jz_old_case_2") && !st.flag("jz_old_case") {
        st.set_flag("jz_old_case");
        crate::world::add_item(st, "it_old_case_relic");
    }
    "jz_01".to_string()
}

/// 藏剑龛剑穗：若已获守陵人信任，则同时开暗格（单向捷径 g4/jz_shortcut_open）
fn route_shortcut(st: &mut GameState) -> String {
    if st.flag("jz_sword_tassel") && st.flag("jz_oldman_trust") {
        st.set_flag("jz_shortcut_open");
    }
    "jz_10_arrive_l2".to_string()
}

/// 断崖坠谷：HP-5 San-5 → 回深谷底部洞厅（L3）
fn route_cliff(st: &mut GameState) -> String {
    st.hp = (st.hp - 5).max(0);
    st.san = (st.san - 5).clamp(0, 100);
    if st.hp <= 0 { return "jz_99_death".to_string(); }
    "jz_20_arrive_l3".to_string()
}

/// 剑碑隐文计数：3 处剑痕(东/西/南) + 北残碑 → jz_stele_secret
fn route_stele(st: &mut GameState) -> String {
    let got = st.flag("jz_swordmark_east") && st.flag("jz_swordmark_west")
        && st.flag("jz_swordmark_south") && st.flag("jz_stele_seen");
    if got && !st.flag("jz_stele_secret") {
        st.set_flag("jz_stele_secret");
        return "jz_34_stele_secret".to_string();
    }
    "jz_30_arrive_l4".to_string()
}

/// 结局结算：通关 san≥30 → jz_san_keep；sp_grade 确保 D → 卡片
fn route_end_settle(st: &mut GameState) -> String {
    if st.san >= 30 { st.set_flag("jz_san_keep"); }
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "jz_52_card".to_string()
}
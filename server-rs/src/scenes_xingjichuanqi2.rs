//! 《寂静岭·灰雾之心》全部剧情场景与「三角头·深红审判」选择驱动 BOSS 配置。
//! α 待替换占位：bg 用 img_zhuyuan_book.png / img_laser.png / img_corridor.png（主线替换为本副本专属背景）。
//! 本文件为全新新增文件，只导出静态数据（XJ2_SCENES / xingjichuanqi2_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/xingjichuanqi2_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `xj2_` 前缀；fight id 全部 `xj2_` 前缀（与既有无重名）。
//! 核心「罪与罚」主题：玩家在迷雾矿洞→废墟教堂→灰雾医院逐层收集「罪证」碎片；
//!   集齐罪证后在深红手术室对三角头·深红审判做「最后的审判」，选择「宽恕/复仇/背负」——
//!   三种选择各导向一个开放结局分支。BOSS 为选择驱动（不碰随机战斗）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_pick(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "xj2_item_pick") }
fn cond_truth_church(st: &GameState) -> bool { st.flag("xj2_truth_church") }
fn cond_all_evidence(st: &GameState) -> bool {
    st.flag("xj2_ev_mine") && st.flag("xj2_ev_church") && st.flag("xj2_ev_hospital")
}

/* =====================================================================
   罪证链工具
   ===================================================================== */
/// 收集一个罪证碎片；三碎片齐备首置 xj2_evidence_full 并加分
fn collect_ev(st: &mut GameState, which: &str, hub: &str) -> String {
    if !st.flag(which) {
        st.set_flag(which);
        if cond_all_evidence(st) && !st.flag("xj2_evidence_full") {
            st.set_flag("xj2_evidence_full");
            st.points += 200;
        }
    }
    hub.to_string()
}
fn mark_ev_mine(st: &mut GameState) -> String { collect_ev(st, "xj2_ev_mine", "xj2_01_l1_hub") }
fn mark_ev_church(st: &mut GameState) -> String {
    st.set_flag("xj2_truth_church"); // 读透教堂旧罪，解锁医院秘密入口 G2
    collect_ev(st, "xj2_ev_church", "xj2_03_l2_hub")
}
fn mark_ev_hospital(st: &mut GameState) -> String { collect_ev(st, "xj2_ev_hospital", "xj2_04_l3_hub") }

/* ---- 环境机关（罪念压身 / 深红梦魇）---- */
/// L1 塌方竖井口：未破罪证则 San 侵蚀 + 可能扣血；破证则无事
fn zone_cavein(st: &mut GameState) -> String {
    if st.flag("xj2_ev_mine") {
        st.points += 30;
        st.set_flag("xj2_sign_mine_cleared");
        "xj2_01_l1_hub".to_string()
    } else {
        "xj2_40_death_cavein".to_string()
    }
}
/// L2 墓穴回声：未破教堂罪证则 San 大侵蚀；破证则加点
fn zone_church_depth(st: &mut GameState) -> String {
    if st.flag("xj2_ev_church") {
        st.points += 40;
        st.set_flag("xj2_sign_church_cleared");
        "xj2_03_l2_hub".to_string()
    } else {
        "xj2_41_death_church".to_string()
    }
}

/* ---- BOSS：三角头·深红审判（选择驱动）的大局起手/斩击/审判/取胜 ---- */
fn xj2_boss_start(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("xj2_boss") {
            st.fight = Some(crate::state::Fight {
                id: "xj2_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "xj2_09_boss_round".to_string()
}
/// 选择驱动 BOSS 对局：dmg 为本次伤害；guard 时本回合免伤
fn xj2_boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return xj2_boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 24 } else { 17 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "xj2_50_death".to_string(); }
    "xj2_09_boss_round".to_string()
}
fn xj2_boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("xj2_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "xj2_item_judgement");
    "xj2_30_judgement".to_string()
}
/// 最后的审判：宽恕 / 复仇 / 背负 三分支 → 三种开放结局
fn judge_forgive(st: &mut GameState) -> String {
    st.set_flag("xj2_end_forgive");
    "xj2_31_forgive".to_string()
}
fn judge_revenge(st: &mut GameState) -> String {
    st.set_flag("xj2_end_revenge");
    "xj2_32_revenge".to_string()
}
fn judge_carry(st: &mut GameState) -> String {
    st.set_flag("xj2_end_carry");
    "xj2_33_carry".to_string()
}

/* ---- 结算 ---- */
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "xj2_42_card".to_string()
}

/* ---- 普通战斗（罪念守卫）胜利桩 ---- */
fn guard_win(_st: &GameState) -> String { "xj2_gwin".to_string() }

/* =====================================================================
   战斗配置表（id 全部 xj2_ 前缀）
   ===================================================================== */
pub fn xingjichuanqi2_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("xj2_fight_miner", FightCfg {
            name: "雾中矿工·缚者", hp: 90, dmg: (12, 18), reward: 150, reward_why: "镇住矿洞第一桩罪",
            intro: "在一圈锈蚀矿灯里，一个戴旧头灯的矿工背对着你——他的脊背上钉着一名「失踪矿工」的名牌。",
            rage_at: Some(30), rage_text: "矿灯惨白地亮起来，缚者挣扎着要你替他「认罪」。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "xj2_50_death",
        }),
        ("xj2_fight_priest", FightCfg {
            name: "持烛神父·堕影", hp: 110, dmg: (13, 20), reward: 180, reward_why: "折返教堂主持忏悔",
            intro: "圣坛的蜡在黑与红之间噼啪烧。神父举起烛台，蜡泪滴成一行字：「你也有罪。」",
            rage_at: Some(40), rage_text: "烛火卷成深红，神父的影子爬满整面墙。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "xj2_50_death",
        }),
        ("xj2_fight_nurse", FightCfg {
            name: "灰雾护士·缝针者", hp: 130, dmg: (14, 22), reward: 220, reward_why: "截停手术台上的缝合",
            intro: "护士推着手术车，针线在她手里自己穿动——她在缝合一封封没有署名的病历。",
            rage_at: Some(50), rage_text: "针线纠缠成一张深红的网，朝你收拢。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "xj2_50_death",
        }),
        ("xj2_boss", FightCfg {
            name: "三角头·深红审判", hp: 200, dmg: (18, 28), reward: 500, reward_why: "在灰雾心脏完成深红审判",
            intro: "深红手术室的铁门在你身后合拢。三角头慢条斯理地拖着巨剑跨过一地病历——它要你，最后一个，替灰雾中的每一桩「罪」，给出审判。",
            rage_at: Some(80), rage_text: "巨剑扬起，深红锈水从刃上滴落——它把审判的铡刀架到了你的颈后，逼你在「宽恕/复仇/背负」里选一个。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "xj2_50_death",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn xj2_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    xingjichuanqi2_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* ---- finisher 空桩（无终结技）---- */
fn fins_name(_st: &GameState) -> String { String::new() }
fn fins_desc(_st: &GameState) -> String { String::new() }
fn fins_false(_st: &GameState, _hp: i32) -> bool { false }
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   剧情场景（id 全部 xj2_ 前缀）
   ===================================================================== */
pub static XJ2_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "xj2_00", bg: Some("xingjichuanqi2_bg.png"), loc: Some("灰雾之心 ≪寂静岭·任务发布≫"),
    mood: "mystery", speaker: Some("主神·任务发布"), voice: None,
    text: TextSpec::Static(&[
        "<b>【场景·寂静岭·灰雾之心】</b>你清醒时已站在一条灰雾深处的矿道口。耳边是别人的忏悔、别人的罪、别人的审判。",
        "<i>「寂静岭没有真相，只问你的心。」</i>雾深处有什么拖着巨剑走过，金属在地面划出深红的痕——那是一把等人还赎的铡刀。",
    ]),
    choices: &[
        ChoiceDef { label: "踏进迷雾矿洞", sub: "+5 点 · 记住这里的「罪念」", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("xj2_01_l1_hub") },
        ChoiceDef { label: "细听雾里的忏悔", sub: "San-2 · 认出这是谁的罪", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("xj2_01_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 迷雾矿洞 hub ================= */
SceneDef {
    id: "xj2_01_l1_hub", bg: Some("xingjichuanqi2_bg_mine.png"), loc: Some("L1 · 迷雾矿洞"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("xj2_ev_mine") {
            "矿洞里那桩「失踪矿工」的罪念，你已经收进了口袋。守灯老人看见你，颤巍巍地朝你举起一盏谢灯。（矿车轨道 / 竖井 / 积水坑 / 铁笼）".to_string()
        } else {
            "顶灯忽明忽暗。矿车轨道锈迹斑斑，竖井口透着冷气，积水坑里漂着半截名牌，铁笼笼口却没合拢。（矿车轨道 / 竖井 / 积水坑 / 铁笼笼口）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "废弃矿车轨道", sub: "罪证·矿洞", cond: None, effects: &NO_EFF, route: Route::To("xj2_02_rail") },
        ChoiceDef { label: "塌方竖井", sub: "罪证·矿洞", cond: None, effects: &NO_EFF, route: Route::To("xj2_02_shaft") },
        ChoiceDef { label: "积水坑", sub: "罪证·矿洞", cond: None, effects: &NO_EFF, route: Route::To("xj2_02_well") },
        ChoiceDef { label: "锈蚀铁笼", sub: "罪证·矿洞", cond: None, effects: &NO_EFF, route: Route::To("xj2_02_cage") },
        ChoiceDef { label: "与守灯老人交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("xj2_05_lamp") },
        ChoiceDef { label: "逼近塌方竖井口", sub: "罪念压身", cond: None, effects: &NO_EFF, route: Route::To("xj2_10_cavein") },
        ChoiceDef { label: "从矿道斜井上楼", sub: "需铁撬 → 教堂", cond: None, effects: &NO_EFF, route: Route::To("xj2_02_gate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 罪证观测点 ---- */
SceneDef {
    id: "xj2_02_rail", bg: Some("xingjichuanqi2_bg_mine.png"), loc: Some("L1 · 废弃矿车轨道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["矿车在轨道尽头侧翻，车斗里压着一件矿工服——名牌上写着「失踪名单」第二行。你认出同一种被埋没的罪。"]),
    choices: &[ChoiceDef { label: "【收下矿洞罪证】", sub: "xj2_ev_mine · 罪证+1", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l1_rail")], route: Route::Dyn(mark_ev_mine) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_02_shaft", bg: Some("xingjichuanqi2_bg_mine.png"), loc: Some("L1 · 塌方竖井"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["竖井口被塌下的碎石封住，井壁上用旧凿子刻满同一个名字。别人的罪，压在谁的肩上。"]),
    choices: &[ChoiceDef { label: "【记下竖井铭文】", sub: "罪证链条线索", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l1_shaft")], route: Route::To("xj2_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_02_well", bg: Some("xingjichuanqi2_bg_mine.png"), loc: Some("L1 · 积水坑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["水面漂着一截烧焦的蜡烛——教堂里点的那种。蜡烛芯上钉着半枚名牌，你把它起出来，收好。"]),
    choices: &[ChoiceDef { label: "【收下积水线索】", sub: "罪证链条线索", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l1_well")], route: Route::To("xj2_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_02_cage", bg: Some("xingjichuanqi2_bg_mine.png"), loc: Some("L1 · 锈蚀铁笼"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["铁笼笼口敞着，里面躺着半截矿工铁撬。它压住了那桩罪的名字——用它撬开教堂侧门，罪证就在门后。"]),
    choices: &[ChoiceDef { label: "取走矿工铁撬", sub: "Item xj2_item_pick · 开 G1", cond: None,
        effects: &[Eff::AddItem("xj2_item_pick"), Eff::MarkPoint("xj2_p_l1_cage")], route: Route::To("xj2_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_10_cavein", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L1 · 塌方竖井口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["灰雾从竖井口倒灌上来，裹着矿工服上那股陈旧的血锈味。有什么在暗处替你回味那桩罪——你该先拿起铁撬。"]),
    choices: &[ChoiceDef { label: "【直面竖井口】", sub: "破矿洞罪证则免死，否则 San-10 · 罪念压身", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_cavein) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_05_lamp", bg: Some("xingjichuanqi2_bg_mine.png"), loc: Some("L1 · 守灯老人的灯下"),
    mood: "cold", speaker: Some("守灯老人"), voice: None,
    text: TextSpec::Static(&["老人盯着你腰间的铁撬：「矿洞那桩，你是要替他枕着，还是替他挖出来？教堂的烛……点得比这儿还近。」"]),
    choices: &[ChoiceDef { label: "「把罪证挖出来。」", sub: "San+2 · → 教堂", cond: None,
        effects: &[Eff::San(2)], route: Route::To("xj2_02_gate") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_02_gate", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L1 · 矿道斜井出口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("xj2_ev_mine") {
            "你持着铁撬，踏出斜井。身后矿洞的风把那名矿工的名牌吹矮进雾里。教堂的钟，正朝你这边敲。".to_string()
        } else {
            "斜井口还锁着，铁撬在你腰间……可矿洞那桩罪，你还没有替它落下个「挖出来」的判决。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "持铁撬踏上斜井（去教堂）", sub: "需 xj2_item_pick · →L2", cond: Some(cond_has_pick),
            effects: &NO_EFF, route: Route::To("xj2_03_l2_hub") },
        ChoiceDef { label: "回到矿洞", sub: "", cond: None, effects: &NO_EFF, route: Route::To("xj2_01_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 废墟教堂 hub ================= */
SceneDef {
    id: "xj2_03_l2_hub", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 废墟教堂"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("xj2_ev_church") {
            "教堂那桩旧罪的姓名，你已在忏悔室读透。圣坛的蜡烧成一线细焰，像在等一个判词。（圣坛 / 忏悔室 / 长椅 / 墓穴入口）".to_string()
        } else {
            "彩窗漏下的光把长椅染成血与灰两界。圣坛前跪着谁，忏悔室的木门虚掩着，墓穴方向飘来旧纸烧焦的味。（圣坛 / 忏悔室 / 长椅 / 墓穴入口）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "圣坛", sub: "罪证·教堂", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_altar") },
        ChoiceDef { label: "忏悔室", sub: "罪证·教堂", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_conf") },
        ChoiceDef { label: "长椅刻字", sub: "罪证链条", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_pew") },
        ChoiceDef { label: "地下墓穴入口", sub: "罪念回声", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| if st.flag("xj2_ev_church") { "xj2_03_l2_hub".to_string() } else { "xj2_03_catacomb".to_string() }) },
        ChoiceDef { label: "与敲钟人交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("xj2_06_bell") },
        ChoiceDef { label: "走教堂侧门下楼", sub: "需读透旧罪 → 医院", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_gate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_03_altar", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 圣坛"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["圣坛执照前压着一卷烧焦的旧卷宗，封皮上的人名被人一遍遍划掉。你把它抽出来——那桩旧罪的名字，在灰烬边现了形。"]),
    choices: &[ChoiceDef { label: "【收下教堂罪证】", sub: "xj2_ev_church · 罪证+1", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l2_altar")], route: Route::Dyn(mark_ev_church) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_03_conf", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 忏悔室"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["木帘后静得可疑。你能听见自己心跳的回响——那是谁把「认罪」两个字压在舌底，没敢说出来。铁撬能撬开帘后的旧档案。"]),
    choices: &[ChoiceDef { label: "【撬开忏悔木帘】", sub: "需铁撬 · 罪证线索", cond: Some(cond_has_pick),
        effects: &[Eff::MarkPoint("xj2_p_l2_conf")], route: Route::To("xj2_03_l2_hub") },
        ChoiceDef { label: "（铁撬不在）回去", sub: "", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_03_pew", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 长椅刻字"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["长椅靠背被人用指甲刻下：「凶手是他们，替罪的是我。」你认得那种被牺牲的笔迹——它在朝教堂的罪证名字靠拢。"]),
    choices: &[ChoiceDef { label: "记下长椅刻字", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l2_pew")], route: Route::To("xj2_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_03_catacomb", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 地下墓穴入口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["墓穴深处传来旧纸燃烧的噼啪，还有一声含糊的「认罪」。你还没有读透教堂那桩罪的名字——下去只会被它吞掉。"]),
    choices: &[ChoiceDef { label: "【踏入墓穴回声】", sub: "破教堂罪证则免死，否则 San-10 · 罪念回声", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_church_depth) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_06_bell", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 钟楼底"),
    mood: "cold", speaker: Some("敲钟人"), voice: None,
    text: TextSpec::Static(&["敲钟人把半截蜡烛塞给你：「教堂这桩，最要命的是谁替谁背黑锅。读透圣坛那份卷宗，你就知道该不该替他认。」"]),
    choices: &[ChoiceDef { label: "「我去读圣坛卷宗。」", sub: "San+2 · → 圣坛", cond: None,
        effects: &[Eff::San(2)], route: Route::To("xj2_03_altar") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_03_gate", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L2 · 教堂侧门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("xj2_ev_church") {
            "你已读透教堂那桩旧罪的姓名。推开侧门，灰雾瞬间更浓——医院的白灯，在雾里一格格亮起来。".to_string()
        } else {
            "侧门外锁着。圣坛那卷卷宗，你还没能读出「谁该认罪」。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "推开侧门（去医院）", sub: "需读透旧罪 · →L3", cond: Some(cond_truth_church),
            effects: &NO_EFF, route: Route::To("xj2_04_l3_hub") },
        ChoiceDef { label: "回到教堂", sub: "", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 灰雾医院 hub ================= */
SceneDef {
    id: "xj2_04_l3_hub", bg: Some("xingjichuanqi2_bg_hospital.png"), loc: Some("L3 · 灰雾医院"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("xj2_ev_hospital") {
            "医院那桩「改写病历」的罪证在案。停尸房的灯排在你脚下，深红手术室的金属门尽头发着血色的光。（挂号台 / 病房 / 停尸房 / 天台）".to_string()
        } else {
            "挂号台的叫号屏闪着一行没有名字的号码，303 病房的门虚掩，停尸房深处有针线穿动的声音。（挂号台 / 303 病房 / 停尸房 / 天台）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "挂号台", sub: "罪证·医院", cond: None, effects: &NO_EFF, route: Route::To("xj2_04_reg") },
        ChoiceDef { label: "303 病房", sub: "罪证链条", cond: None, effects: &NO_EFF, route: Route::To("xj2_04_ward") },
        ChoiceDef { label: "停尸房", sub: "罪证·医院", cond: None, effects: &NO_EFF, route: Route::To("xj2_04_morgue") },
        ChoiceDef { label: "雾中天台", sub: "罪念瞭望", cond: None, effects: &NO_EFF, route: Route::To("xj2_04_roof") },
        ChoiceDef { label: "与守夜人交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("xj2_07_night") },
        ChoiceDef { label: "走向深红手术室", sub: "选择驱动 BOSS", cond: None, effects: &NO_EFF, route: Route::Dyn(xj2_boss_start) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_04_reg", bg: Some("xingjichuanqi2_bg_hospital.png"), loc: Some("L3 · 挂号台"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["挂号台后散落一摞病历，每张的名字都被血改写过。最后一页的名字，正好压在你「判罪」的名单上。"]),
    choices: &[ChoiceDef { label: "【收下医院罪证】", sub: "xj2_ev_hospital · 罪证+1", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l3_reg")], route: Route::Dyn(mark_ev_hospital) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_04_ward", bg: Some("xingjichuanqi2_bg_hospital.png"), loc: Some("L3 · 303 病房"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["床褥凹陷成一个人形，床头贴着一张没署名的监护单。你在枕下摸到半张写满「该谁认罪」的便签。"]),
    choices: &[ChoiceDef { label: "收好监护单", sub: "罪证链条线索", cond: None,
        effects: &[Eff::MarkPoint("xj2_p_l3_ward")], route: Route::To("xj2_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_04_morgue", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L3 · 停尸房"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["不锈钢抽屉缝着一根针线，线头连向手术室。你拉开一格——里面躺着的，是那桩连锁旧罪的最后一个名字。"]),
    choices: &[ChoiceDef { label: "记下停尸名录", sub: "罪证链条线索 · San-2", cond: None,
        effects: &[Eff::San(-2), Eff::MarkPoint("xj2_p_l3_morgue")], route: Route::To("xj2_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_04_roof", bg: Some("xingjichuanqi2_bg_hospital.png"), loc: Some("L3 · 雾中天台"),
    mood: "awe", speaker: None, voice: None,
    text: TextSpec::Static(&["天台被灰雾围成一口井。雾的中央，那座「深红手术室」黑魆魆地站着，像一个借了教堂与矿洞全部罪念的铡刀。"]),
    choices: &[ChoiceDef { label: "俯瞰深红手术室", sub: "+5 点 · San+2 · 看清审判之地", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("xj2_p_l3_roof")], route: Route::To("xj2_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_07_night", bg: Some("xingjichuanqi2_bg_hospital.png"), loc: Some("L3 · 医院守夜岗"),
    mood: "cold", speaker: Some("医院守夜人"), voice: None,
    text: TextSpec::Static(&["守夜人攥着半瓶水，声音压得极低：「三角头选定你做那个『判罪的』。它不杀你——它要你说出，矿洞、教堂、医院这三桩罪，到底该谁扛。」"]),
    choices: &[ChoiceDef { label: "「我去见它。」", sub: "→ 深红手术室", cond: None,
        effects: &[Eff::San(2)], route: Route::Dyn(xj2_boss_start) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= BOSS：三角头·深红审判（选择驱动） ================= */
SceneDef {
    id: "xj2_09_boss_round", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L3 · 深红手术室"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Dyn(|st| format!("深红手术室里，巨剑的锈水在两侧淌成血沟。三角头在雾中低垂着头——它还在等。罪证 {} 罪念攻你。BOSS 剩余 {} 血，你 HP {}", if st.flag("xj2_evidence_full") { "已集齐，它的审判架在你颈后" } else { "未集齐，它把铡刀压得更近" }, st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "重斩", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| xj2_boss_act(st, 32, false)) },
        ChoiceDef { label: "格挡", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| xj2_boss_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 胜利 · 最后的审判（选择驱动开放结局） ---- */
SceneDef {
    id: "xj2_30_judgement", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L3 · 深红手术室 · 审判台"), mood: "choice",
    speaker: Some("三角头·深红审判"), voice: None,
    text: TextSpec::Static(&[
        "巨剑插进手术台，深红锈水慢慢凝固。三桩罪——矿洞、教堂、医院——现在全由你一个人来定夺。",
        "三角头的刀刃横在你面前：「宽恕，还是复仇？还是把这桩错，背到自己身上了结？」",
    ]),
    choices: &[
        ChoiceDef { label: "【宽恕】", sub: "松开铡刀 · 让罪各自安葬", cond: None, effects: &NO_EFF, route: Route::Dyn(judge_forgive) },
        ChoiceDef { label: "【复仇】", sub: "以血偿血 · 让罪付出代价", cond: None, effects: &NO_EFF, route: Route::Dyn(judge_revenge) },
        ChoiceDef { label: "【背负】", sub: "替所有人认罪 · 让过去翻页", cond: None, effects: &NO_EFF, route: Route::Dyn(judge_carry) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_31_forgive", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L3 · 审判台（宽恕之径）"), mood: "awe",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你松开了铡刀。灰雾缓缓落定，矿洞的灯、教堂的烛、医院的白灯，一格格地熄灭成安稳的灰。",
        "三角头像一位完成赎罪的守卫，慢慢化作一片深红剪影，沉进雾里。寂静岭没有真相——可它放过了愿意被放过的你。",
    ]),
    choices: &[ChoiceDef { label: "向灰雾告别", sub: "San+20 · 宽恕结局 · 结算", cond: None,
        effects: &[Eff::San(20), Eff::Points(150)], route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_32_revenge", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L3 · 审判台（复仇之径）"), mood: "danger",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你举起了铡刀。深红锈水被你的手溅起，泼进雾里——那三桩罪的仇人，一个个在静默里倒伏下去。",
        "灰雾在你脚下退成一片焦土。你没有得到答案，只得到了「清算」。寂静岭记住了一张没有回头路的脸。",
    ]),
    choices: &[ChoiceDef { label: "以血开路", sub: "+200 点 · 复仇结局 · 结算", cond: None,
        effects: &[Eff::Points(200)], route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xj2_33_carry", bg: Some("xingjichuanqi2_bg.png"), loc: Some("L3 · 审判台（背负之径）"), mood: "mystery",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你没有让任何人认罪。你替三桩罪背起了它们的重量，一步一步，走进雾最浓的深处。",
        "三桩旧罪的名字在你身后钉进墓碑。灰雾第一次没有吞噬你——它只是安静地，把你送出这个副本，像送走一位古早的守墓人。",
    ]),
    choices: &[ChoiceDef { label: "背负着离开", sub: "San-10 · +250 点 · 背负结局 · 结算", cond: None,
        effects: &[Eff::San(-10), Eff::Points(250)], route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 普通战斗胜利中转：给你选择回落对应楼层 hub（保持确定性） */
SceneDef {
    id: "xj2_gwin", bg: Some("xingjichuanqi2_bg.png"), loc: Some("罪念守卫 · 消散"), mood: "calm",
    speaker: None, voice: None,
    text: TextSpec::Dyn(|_st| format!("罪念守卫在你足下塌成一堆灰烬，被雾卷走。战利点入账 {}{}。", 30, "")),
    choices: &[
        ChoiceDef { label: "回到迷雾矿洞", sub: "L1", cond: None, effects: &NO_EFF, route: Route::To("xj2_01_l1_hub") },
        ChoiceDef { label: "回到废墟教堂", sub: "L2", cond: None, effects: &NO_EFF, route: Route::To("xj2_03_l2_hub") },
        ChoiceDef { label: "回到灰雾医院", sub: "L3", cond: None, effects: &NO_EFF, route: Route::To("xj2_04_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 结算卡（胜利） ---- */
SceneDef {
    id: "xj2_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "结 算".into(), good: true,
            body_html: format!("<p>你在灰雾之心的审判完成了。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr><tr><td>结局</td><td>{}</td></tr></table>", st.points, if st.flag("xj2_end_forgive") {"宽恕静好"} else if st.flag("xj2_end_revenge") {"血仇清算"} else {"背负离场"}),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- 死亡档案（意外 / 罪念压身 / 审判失败） ---- */
SceneDef {
    id: "xj2_40_death_cavein", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("寂静岭·罪念压身", "未先取矿洞罪证，被塌方竖井的旧罪碾进雾底")),
        card: |_st| crate::state::Card {
            title: "死 亡".into(), good: false,
            body_html: r#"<p>你死在了矿洞竖井口的灰雾里。</p><p style='color:#ff8a8a'>【死亡档案】被那桩没人敢认的罪压垮。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "xj2_41_death_church", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("寂静岭·罪念回声", "未读透教堂旧罪，被墓穴的回声定罪吞没")),
        card: |_st| crate::state::Card {
            title: "死 亡".into(), good: false,
            body_html: r#"<p>你死在了教堂墓穴的回声里。</p><p style='color:#ff8a8a'>【死亡档案】在替谁认罪的漩涡里，你先一步没了声息。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "xj2_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("寂静岭·审判未竟", "在深红手术室的铡刀下没能完成审判")),
        card: |_st| crate::state::Card {
            title: "死 亡".into(), good: false,
            body_html: r#"<p>你死在了灰雾之心的审判台下。</p><p style='color:#ff8a8a'>【死亡档案】三角头的审判，没有等你把话说完。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
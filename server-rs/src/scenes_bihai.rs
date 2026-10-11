//! 《无限恐怖 · 深海阴影》全部剧情场景与「存在主义恐怖」规则流配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §「深海阴影」骨架。
//! 本文件为全新新增文件，只导出静态数据（BIHAI_SCENES / bihai_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/bihai_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `bh_` 前缀；fight id 全部 `bh_` 前缀（与既有无重名）。
//! 核心主题「世界展示向·克苏鲁式存在主义恐怖」，无真相线，开放结局：
//!   · L1 深海潜水器舱 → L2 沉船残骸 → L3 海沟深渊（克苏鲁感邪物栖息地）。
//!   · 每层 1 个「深海异形」象征战（原生死斗，取胜即印证「深渊在逼近」）。
//!   · L3 中央祭坛为「选择驱动 BOSS」深渊邪物（黄金模板 C 段：start_boss/boss_act/boss_win，HP 230）。
//!   · 击败后开放结局 3 分支：逃离海沟 / 献祭自身换取宁静 / 与邪物同化，均可结算。
//!   · 环境机关：深水压区 / 缺氧区（激励对抗，未完成即跳死亡档案「深海的拥吻」）。
//! bg 全部占位：img_zhuyuan_book.png / img_laser.png / img_corridor.png（注待主线替换）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_deep_key(st: &GameState) -> bool {
    st.inventory.iter().any(|i| i == "it_deep_key")
}
fn cond_faced_altar(st: &GameState) -> bool { st.flag("bh_faced_altar") }
fn cond_sacrifice_possible(st: &GameState) -> bool { st.flag("bh_whisper_heard") }

/* =====================================================================
   路由 / 工具 fn（返回 String）
   ===================================================================== */
/// 关闭一个环境机关（安慰/加回报），幂等保证不重复加点
fn close_env(st: &mut GameState, flag: &str, pts: i32, back: &str) -> String {
    if !st.flag(flag) {
        st.set_flag(flag);
        st.points += pts;
    }
    back.to_string()
}
fn route_pressure_escape(st: &mut GameState) -> String { close_env(st, "bh_signed_pressure", 40, "bh_l3_hub") }
fn route_anoxia_escape(st: &mut GameState) -> String { close_env(st, "bh_signed_anoxia", 40, "bh_l3_hub") }

/* =====================================================================
   环境机关 ZoneDef 触发：对抗成功 → 记录 + 加点回 hub；否则死亡档案「深海的拥吻」
   ===================================================================== */
fn zone_pressure(st: &mut GameState) -> String {
    if st.flag("bh_signed_pressure") {
        "bh_l3_hub".to_string()
    } else {
        "bh_50_death".to_string()
    }
}
fn zone_anoxia(st: &mut GameState) -> String {
    if st.flag("bh_signed_anoxia") {
        "bh_l3_hub".to_string()
    } else {
        "bh_50_death".to_string()
    }
}

/* =====================================================================
   开放结局路由
   ===================================================================== */
fn route_end_escape(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('C'); }
    st.set_flag("bh_end_escape");
    "bh_win_escape".to_string()
}
fn route_end_sacrifice(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('B'); }
    st.set_flag("bh_end_sacrifice");
    "bh_win_sacrifice".to_string()
}
fn route_end_assimilate(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('A'); }
    st.set_flag("bh_end_assimilate");
    "bh_win_assimilate".to_string()
}
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "bh_42_card".to_string()
}

/* =====================================================================
   普通战斗 win / rage / finisher 桩
   ===================================================================== */
fn on_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}
fn fins_if(_st: &GameState, _hp: i32) -> bool { false }
fn fins_name(_st: &GameState) -> String { String::new() }
fn fins_desc(_st: &GameState) -> String { String::new() }
fn win_l1(_st: &GameState) -> String { "bh_l1_hub".to_string() }
fn win_l2(_st: &GameState) -> String { "bh_l2_hub".to_string() }
fn win_l3(_st: &GameState) -> String { "bh_l3_hub".to_string() }

/* =====================================================================
   选择驱动 BOSS「深渊邪物」（黄金模板 C 段 start_boss / boss_act / boss_win）
   ===================================================================== */
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("bh_boss") {
            st.fight = Some(crate::state::Fight {
                id: "bh_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "bh_boss_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "bh_50_death".to_string(); }
    "bh_boss_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("bh_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "it_deep_key"); // 象征意义：深渊「归还」一把钥匙（同化/逃离皆可持有）
    "bh_end_choice".to_string()
}

/* =====================================================================
   战斗配置表（id 全部 bh_ 前缀）——3 种深海异形 + 1 场选择驱动 BOSS
   ===================================================================== */
pub fn bihai_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("bh_fight_l1", FightCfg {
            name: "深海异形·潜航员变异体", hp: 90, dmg: (10, 16), reward: 150, reward_why: "清除潜水器舱内的异形",
            intro: "湿漉漉的影子在你身后收拢——那曾是你的一个船员。它张开一张不该属于人的嘴，朝你爬来。",
            rage_at: Some(45), rage_text: "异形的皮肤融化了，露出底下一张张蠕动的脸——它在「替你」长大自己。",
            on_rage: on_rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_l1, death: "bh_50_death",
        }),
        ("bh_fight_l2", FightCfg {
            name: "沉船寄生怪", hp: 120, dmg: (12, 20), reward: 220, reward_why: "清除沉船货舱的寄生怪",
            intro: "甲板缝里长出一个半人半触须的东西，拽着断裂的缆绳站起来——它把整艘沉船当作宿主。",
            rage_at: Some(55), rage_text: "舱板整片掀起，寄生怪融合了整段船舷，朝你倾压下来。",
            on_rage: on_rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_l2, death: "bh_50_death",
        }),
        ("bh_fight_l3", FightCfg {
            name: "深渊异形·近侍", hp: 160, dmg: (14, 22), reward: 320, reward_why: "清除海沟深渊的异形近侍",
            intro: "祭坛影子里浮出一尊细长的近侍。它没有脸，只在它「看」你时，你耳边响起身后低语的共鸣。",
            rage_at: Some(70), rage_text: "近侍的身形重叠成无数层，把整处深渊的光都吸进它那张空壳里。",
            on_rage: on_rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_l3, death: "bh_50_death",
        }),
        ("bh_boss", FightCfg {
            name: "深渊邪物", hp: 230, dmg: (16, 26), reward: 500, reward_why: "直面·&选择深渊邪物的收场",
            intro: "整座海沟祭坛塌进它脚下。邪物睁开无数只眼，每只眼里都映着一个「你」。它没有说话——因为语言从未抵达过它的层面。",
            rage_at: Some(80), rage_text: "邪物的触须缠成一片海，深渊的呼吸碾过你：它不再邀请你「看懂」，只要求你「选择」。",
            on_rage: on_rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: |_st| "bh_end_choice".to_string(), death: "bh_50_death",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn bh_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    bihai_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 bh_ 前缀）
   ===================================================================== */
pub static BIHAI_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "bh_00", bg: Some("bihai_bg.png"), loc: Some("深海舱 · ≪阴影从海底亮起≫"),
    mood: "mystery", speaker: Some("主神·任务发布"), voice: Some("vo_bihai_1"),
    text: TextSpec::Static(&[
        "<b>【主线任务 · 深海阴影】</b>下潜至海沟深渊，看清「深渊邪物」——然后，用你的方式为它写下结局。",
        "<i>「深渊从不问你看到什么。它只在你睁眼时，承认你在。」</i>深海潜水器的灯，在第一千米的地方，一盏接一盏地灭下去。",
    ]),
    choices: &[
        ChoiceDef { label: "下潜 · 进入潜水器舱", sub: "+5 点 · 开始深海之旅", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("bh_l1_hub") },
        ChoiceDef { label: "静听舱外的水声", sub: "San-2 · 确认「这里有东西」", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("bh_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 深海潜水器舱 hub ================= */
SceneDef {
    id: "bh_l1_hub", bg: Some("bihai_bg.png"), loc: Some("L1 · 深海潜水器舱"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bh_p_l1_sonar_seen") {
            "声呐屏一直亮着同一个光点——就在潜水器正下方，一动不动等你。舱壁的裂缝又裂开一点，水珠正顺着锈痕往下爬。（声呐 / 舷窗 / 裂缝 / 氧气 / 储物舱）".to_string()
        } else {
            "灯只亮在几个角。声呐屏在一格一格转，储物舱的门半开着，氧气管路里有规律的滴答声——像谁的脉搏。你闻到了一股比海更老的腥味。（声呐 / 舷窗 / 舱壁裂缝 / 氧气管路 / 储物舱）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "声呐屏", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bh_01_sonar") },
        ChoiceDef { label: "观察舷窗", sub: "看向深渊", cond: None, effects: &NO_EFF, route: Route::To("bh_01_view") },
        ChoiceDef { label: "舱壁裂缝", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bh_01_hull") },
        ChoiceDef { label: "氧气管路", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bh_01_air") },
        ChoiceDef { label: "储物舱", sub: "翻找", cond: None, effects: &NO_EFF, route: Route::To("bh_01_store") },
        ChoiceDef { label: "幸存潜水员在舱角", sub: "交谈", cond: None, effects: &NO_EFF, route: Route::To("bh_n1_diver") },
        ChoiceDef { label: "走向水密舱门（去沉船）", sub: "需海渊钥匙 → L2", cond: None, effects: &NO_EFF, route: Route::To("bh_01_gate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点 ---- */
SceneDef {
    id: "bh_01_sonar", bg: Some("bihai_bg.png"), loc: Some("L1 · 声呐室"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bh_p_l1_sonar_seen") {
            "声呐屏仍锁着正下方那一个光点。它在最深的地方，用一个不变的频率，替你「保持」着深渊的呼吸。".to_string()
        } else {
            "一个光点固执地停在声呐屏正中：就在潜水器正下方的海床上。周围空无一片回声——仿佛那片海，是特意为它留出的位置。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "记下这个光点", sub: "伏笔 · 线索", cond: None,
        effects: &[Eff::SetFlag("bh_p_l1_sonar_seen"), Eff::MarkPoint("bh_p_l1_sonar")], route: Route::To("bh_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_01_view", bg: Some("bihai_bg.png"), loc: Some("L1 · 观察舷窗"),
    mood: "awe", speaker: None, voice: None,
    text: TextSpec::Static(&["舷窗外只有一盏探照灯的光柱，被黑暗吞进去又吐出来。在那光柱的尽头，有什么巨大的东西，正在极慢地翻了个身——它的体量，比整艘潜水器都宽。"]),
    choices: &[ChoiceDef { label: "移开视线", sub: "San-4 · 你明白了「大小」", cond: None,
        effects: &[Eff::San(-4)], route: Route::To("bh_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_01_hull", bg: Some("bihai_bg.png"), loc: Some("L1 · 舱壁裂缝"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["缝隙里不是海水——是一截湿滑、泛着荧光的触手末端，正有节奏地缩进缩出。它不急着进来，仿佛只是在试探：这里，值不值得进去。"]),
    choices: &[ChoiceDef { label: "撬开裂缝", sub: "看看里面", cond: None,
        effects: &[Eff::San(-2)], route: Route::To("bh_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_01_air", bg: Some("bihai_bg.png"), loc: Some("L1 · 氧气管路"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["氧气管有规律的滴答声来自管壁上一枚开凿的钉子——不是意外。有人在氧气里「标」过什么，像在它的供氧里先钉下一枚眼。"]),
    choices: &[ChoiceDef { label: "拔出那枚钉子", sub: "San+2 · 但滴答声没停", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bh_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_01_store", bg: Some("bihai_bg.png"), loc: Some("L1 · 储物舱"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["储物柜最深处压着一张手写的海图。最下方标注着红线：「那里没有该存在的东西。若你读到这句——它已经知道你在读了。」"]),
    choices: &[ChoiceDef { label: "收好这张海图", sub: "线索 · MarkPoint", cond: None,
        effects: &[Eff::MarkPoint("bh_p_l1_store")], route: Route::To("bh_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_n1_diver", bg: Some("bihai_bg.png"), loc: Some("L1 · 潜水员舱"),
    mood: "cold", speaker: Some("幸存潜水员"), voice: None,
    text: TextSpec::Static(&["潜水员蜷在舱角，喘着气：「下面……下面那些不该在这儿的东西，它不吃我们，它只是在『认』我们。你要是想活，就别再看它了。」"]),
    choices: &[ChoiceDef { label: "「可我已经看见了。」", sub: "San+2 · 潜水员沉默", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bh_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_01_gate", bg: Some("bihai_bg.png"), loc: Some("L1 · 中央水密舱门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if cond_has_deep_key(st) {
            "你插进海渊钥匙，舱门呻吟着让开。水压带着一股深海的腥甜涌进来——你踩上通往海沟的下潜阶梯。".to_string()
        } else {
            "水密舱门死死咬合，锈住的门闩上盘着海藻。你还没有海渊钥匙——那应该躺在某艘沉船的船长室里。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "打开舱门（去 L2 沉船）", sub: "需 it_deep_key · →L2", cond: Some(cond_has_deep_key),
            effects: &NO_EFF, route: Route::To("bh_l2_hub") },
        ChoiceDef { label: "回到潜水器舱", sub: "", cond: None, effects: &NO_EFF, route: Route::To("bh_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 沉船残骸 hub ================= */
SceneDef {
    id: "bh_l2_hub", bg: Some("bihai_bg.png"), loc: Some("L2 · 沉船残骸"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if cond_has_deep_key(st) {
            "海渊钥匙已经在手。这艘沉船倾斜得厉害，鬼火般的磷光在断裂甲板间游荡；船长室的窗缝里，透出一盏还没熄灭的灯。（船长室 / 货舱 / 机房 / 石碑）".to_string()
        } else {
            "沉船像一头跪着的巨兽，甲板朝左倾了三十度。磷光在断裂的甲板下爬行，货舱深处有动静，船长室的窗里却亮着一盏不该还亮着的灯。（船长室 / 货舱 / 底舱 / 石碑）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "船长室", sub: "海渊钥匙", cond: None, effects: &NO_EFF, route: Route::To("bh_02_captain") },
        ChoiceDef { label: "货舱", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bh_02_cargo") },
        ChoiceDef { label: "底舱机房", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bh_02_machine") },
        ChoiceDef { label: "邪神石碑残片", sub: "线索", cond: None, effects: &NO_EFF, route: Route::To("bh_02_artifact") },
        ChoiceDef { label: "幸存船员在底舱", sub: "交谈", cond: None, effects: &NO_EFF, route: Route::To("bh_n2_survivor") },
        ChoiceDef { label: "沿舱底裂口下潜 → L3", sub: "（需石碑线索后可）", cond: None, effects: &NO_EFF, route: Route::To("bh_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 调查点 ---- */
SceneDef {
    id: "bh_02_captain", bg: Some("bihai_bg.png"), loc: Some("L2 · 船长室"),
    mood: "cold", speaker: Some("船长日记"), voice: None,
    text: TextSpec::Static(&["桌上摊着船长的航海日志，最后一页只写一句：「它不是敌人。它是这片海照出我们的样子。钥匙……我把它放进抽屉，等敢直视它的人来取。」抽屉里，静静躺着一枚海渊钥匙。"]),
    choices: &[ChoiceDef { label: "取走海渊钥匙", sub: "Item it_deep_key · 开G1", cond: None,
        effects: &[Eff::AddItem("it_deep_key"), Eff::MarkPoint("bh_p_l2_captain")], route: Route::To("bh_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_02_cargo", bg: Some("bihai_bg.png"), loc: Some("L2 · 货舱"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["货舱的板条箱全被某种液体浸透，像被一只巨手从里面捂过。箱面凹进一个又一个手指印——不是人的，是指节太多、太密的那种。"]),
    choices: &[ChoiceDef { label: "避开货舱深处", sub: "San-2 · 那里面有东西", cond: None,
        effects: &[Eff::San(-2)], route: Route::To("bh_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_02_machine", bg: Some("bihai_bg.png"), loc: Some("L2 · 底舱机房"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["底舱的机器早已锈死，可一台老式发报机还在自己嗒嗒地发着莫尔斯电码。你数了好一会儿，它翻来覆去只发同一串：'不要带着答案上来。'"]),
    choices: &[ChoiceDef { label: "拍下发报声", sub: "线索 · MarkPoint", cond: None,
        effects: &[Eff::MarkPoint("bh_p_l2_machine")], route: Route::To("bh_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_02_artifact", bg: Some("bihai_bg.png"), loc: Some("L2 · 邪神石碑残片"),
    mood: "awe", speaker: None, voice: None,
    text: TextSpec::Static(&["石碑残片上刻着一只同心圆大眼，圆周密布着蜷缩的人形。你读懂一句古文字：「祂在海沟的最深处，等一个愿意与祂『相认』的人。」——你终于知道，海图上的光点是什么了。（线索：bh_know_truth）"]),
    choices: &[ChoiceDef { label: "直视石碑上的巨眼", sub: "San-6 · 你记住了它的形状", cond: None,
        effects: &[Eff::San(-6), Eff::SetFlag("bh_know_truth")], route: Route::To("bh_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_n2_survivor", bg: Some("bihai_bg.png"), loc: Some("L2 · 底舱角落"),
    mood: "cold", speaker: Some("幸存船员"), voice: None,
    text: TextSpec::Static(&["船员抱着膝盖发抖：「我们看到了那颗光点……船长说那是『祂』。祂不吃我们，祂只是把我们『照』进祂的眼睛里。别下去，一旦下去，就再也上不来了。」"]),
    choices: &[ChoiceDef { label: "「可我已经『照』进去了。」", sub: "San+2", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bh_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 海沟深渊·邪物栖息地 hub ================= */
SceneDef {
    id: "bh_l3_hub", bg: Some("bihai_bg.png"), loc: Some("L3 · 海沟深渊"),
    mood: "awe", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if cond_faced_altar(st) {
            "你已在祭坛前与那双巨眼对视过。深渊的触须朝你让开一条路——它邀请你，走到最深处，写下结局。（祭坛 / 巨眼 / 裂隙 / 邪教遗民）".to_string()
        } else {
            "海沟在最深处张开。中央祭坛浮在摇曳的磷光里，一枚巨眼在祭坛上空缓缓转动；低语裂隙渗出刺痛耳膜的低鸣。这里是——它栖息的地方。（祭坛 / 巨眼 / 裂隙 / 遗民）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "中央祭坛", sub: "直面深渊邪物（BOSS）", cond: Some(cond_faced_altar), effects: &NO_EFF, route: Route::To("bh_03_altar") },
        ChoiceDef { label: "深渊巨眼", sub: "正视它", cond: None, effects: &NO_EFF, route: Route::To("bh_03_eye") },
        ChoiceDef { label: "低语裂隙", sub: "听它说什么", cond: None, effects: &NO_EFF, route: Route::To("bh_03_whisper") },
        ChoiceDef { label: "邪教遗民在渊边", sub: "交谈", cond: None, effects: &NO_EFF, route: Route::To("bh_n3_cult") },
        ChoiceDef { label: "直面深渊祭坛 → BOSS", sub: "需 bh_faced_altar", cond: None, effects: &NO_EFF, route: Route::To("bh_boss") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 调查点 ---- */
SceneDef {
    id: "bh_03_altar", bg: Some("bihai_bg.png"), loc: Some("L3 · 中央祭坛"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["祭坛中央的下陷里，盛着漆黑而黏稠的「海水」。你把手指探进去，立即明白：那不是水，是它的一部分。低头的一瞬，深渊在你倒影里睁开了一只眼。"]),
    choices: &[
        ChoiceDef { label: "【与深渊对视】", sub: "bh_faced_altar · San+0 · 直面真相", cond: None,
            effects: &[Eff::SetFlag("bh_faced_altar")], route: Route::To("bh_l3_hub") },
        ChoiceDef { label: "转身逃开", sub: "San-8 · 你还没准备好", cond: None,
            effects: &[Eff::San(-8)], route: Route::To("bh_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_03_eye", bg: Some("bihai_bg.png"), loc: Some("L3 · 深渊巨眼"),
    mood: "awe", speaker: None, voice: None,
    text: TextSpec::Static(&["那枚巨眼缓缓转向你。没有恶意，没有食欲——只有一股「相认」的、近乎虔诚的注视。你在它虹膜的最深处，看见自己一行行被它「读到」成一句祷词。"]),
    choices: &[ChoiceDef { label: "回报以注视", sub: "San-6 · 你与它相认了一半", cond: None,
        effects: &[Eff::San(-6), Eff::SetFlag("bh_faced_altar")], route: Route::To("bh_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_03_whisper", bg: Some("bihai_bg.png"), loc: Some("L3 · 低语裂隙"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["裂隙里的低语终于拼成一句人话：「把你自己还给我，我让这片海归于宁静。」你不确定那是威胁，还是它唯一会的祈求。"]),
    choices: &[ChoiceDef { label: "听清这句低语", sub: "线索 · 开启「献祭」结局", cond: None,
        effects: &[Eff::SetFlag("bh_whisper_heard")], route: Route::To("bh_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_n3_cult", bg: Some("bihai_bg.png"), loc: Some("L3 · 渊边巨石"),
    mood: "cold", speaker: Some("邪教遗民"), voice: None,
    text: TextSpec::Static(&["遗民匍匐在渊边，念着：「祂不是为了伤害而来的。祂只是……永远地醒着，等着有人能与祂『一同存在』。献祭自己，或是借走祂的一双眼——都是与祂相认的办法。」"]),
    choices: &[ChoiceDef { label: "「相认……该怎么选？」", sub: "San+2 · 指向 3 条结局", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bh_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 深浅异形原生 fight（每层 1 场） ---- */
SceneDef {
    id: "bh_06_e1", bg: Some("bihai_bg.png"), loc: Some("L1 · 潜水器舱 · 遭遇战"),
    mood: "danger", speaker: Some("深海异形"), voice: None,
    text: TextSpec::Static(&["湿漉漉的影子在你身后收拢——那曾是你的一个船员。战斗没有预兆，就像深海从来不必预告它要你「看见」什么。（战斗）"]),
    choices: &[], fight_id: Some("bh_fight_l1"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_06_e2", bg: Some("bihai_bg.png"), loc: Some("L2 · 货舱 · 遭遇战"),
    mood: "danger", speaker: Some("沉船寄生怪"), voice: None,
    text: TextSpec::Static(&["甲板缝里长出的寄生怪拖着一整段船舷站起来。它挡住你通往石碑的路。（战斗）"]),
    choices: &[], fight_id: Some("bh_fight_l2"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_06_e3", bg: Some("bihai_bg.png"), loc: Some("L3 · 祭坛影 · 遭遇战"),
    mood: "danger", speaker: Some("深渊异形·近侍"), voice: None,
    text: TextSpec::Static(&["祭坛影子里浮出细长的近侍，挡在邪物与你的凝视之间。它「守卫」得近乎虔诚。（战斗）"]),
    choices: &[], fight_id: Some("bh_fight_l3"), video: None, cine_label: None, overlay: None,
},

/* ---- 选择驱动 BOSS「深渊邪物」 ---- */
SceneDef {
    id: "bh_boss", bg: Some("bihai_bg.png"), loc: Some("L3 · 深渊祭坛 · BOSS"),
    mood: "danger", speaker: Some("深渊邪物"), voice: None,
    text: TextSpec::Dyn(|st| {
        if cond_faced_altar(st) {
            format!("你已在祭坛前与它对过眼神。邪物睁开所有眼，触须垂到地面——它邀请你，用「你的方式」为它写下结局。当前记号：HP {} / {}。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), 230)
        } else {
            "祭坛的力量把你吸向深渊。你必须先与它对视，才能在这里站立。「深渊」——它这样称呼自己，而你还没有资格直视它。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【与深渊相认】", sub: "直面邪物 · 开始战斗", cond: Some(cond_faced_altar),
            effects: &NO_EFF, route: Route::Dyn(start_boss) },
        ChoiceDef { label: "退后一步", sub: "", cond: None, effects: &NO_EFF, route: Route::To("bh_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_boss_round", bg: Some("bihai_bg.png"), loc: Some("L3 · 深渊祭坛 · 决战"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| format!("邪物剩余 {} 血，你 HP {}。它不再攻击——它在『读』你在深渊前的姿态。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "重击", sub: "高伤（象征你在正面相认）", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
        ChoiceDef { label: "防御", sub: "本回合免伤（象征你在承受）", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ChoiceDef { label: "回敬深渊（重击+）", sub: "需已听懂低语 · 高伤但自伤", cond: Some(cond_sacrifice_possible), effects: &[Eff::Hurt(4, "bh_50_death")], route: Route::Dyn(|st| boss_act(st, 45, false)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_end_choice", bg: Some("bihai_bg.png"), loc: Some("L3 · 深渊 · 开放结局"),
    mood: "choice", speaker: Some("深渊邪物"), voice: None,
    text: TextSpec::Static(&["邪物在你的重击下坍缩成一片深海。它没有死——它终于「认出了」你。深渊在你脚下裂开三条路：逃离海沟，把自己还给祂，或是……与祂一同存在。结局由你写下。"]),
    choices: &[
        ChoiceDef { label: "逃离海沟", sub: "C 级 · 带着目睹活着上岸", cond: None, effects: &NO_EFF, route: Route::Dyn(route_end_escape) },
        ChoiceDef { label: "献祭自己，换取宁静", sub: "B 级 · 需听懂低语", cond: Some(cond_sacrifice_possible), effects: &NO_EFF, route: Route::Dyn(route_end_sacrifice) },
        ChoiceDef { label: "与邪物同化", sub: "A 级 · 借走祂的一双眼", cond: None, effects: &NO_EFF, route: Route::Dyn(route_end_assimilate) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 开放结局场景 ---- */
SceneDef {
    id: "bh_win_escape", bg: Some("bihai_bg.png"), loc: Some("结局 · 逃离海沟"),
    mood: "calm", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["你带着那道注视，拼死游上浮标。海面在脚下重新亮起日光。你没能带答案上岸——但你把「看过祂」这件事，永远地带上了岸。这已足够。"]),
    choices: &[ChoiceDef { label: "确认撤离", sub: "结算", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_win_sacrifice", bg: Some("bihai_bg.png"), loc: Some("结局 · 献祭"),
    mood: "calm", speaker: Some("深渊邪物"), voice: None,
    text: TextSpec::Static(&["你把曾记下的答案一点点还给它。裂隙的低语终于安静下来——海归于宁静。你不确定自己是否还「在那里」，但你知道，那片海，从今往后不再有阴影。"]),
    choices: &[ChoiceDef { label: "确认牺牲", sub: "结算", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_win_assimilate", bg: Some("bihai_bg.png"), loc: Some("结局 · 同化"),
    mood: "awe", speaker: Some("深渊邪物"), voice: None,
    text: TextSpec::Static(&["你借走了祂的一双眼。睁开——整片海在你眼底清晰得没有阴影，你终于与祂一同存在：不是胜利，是相认。你不再是「岸上的人」。"]),
    choices: &[ChoiceDef { label: "确认与祂同化", sub: "结算", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 环境机关对抗场景 ---- */
SceneDef {
    id: "bh_20_pressure", bg: Some("bihai_bg.png"), loc: Some("L3 · 深水压区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["水压骤增，舱壁像被一只巨掌合拢。你若不能稳住，就会在这里被碾成深海的一缕磷光。（对抗深水压）"]),
    choices: &[ChoiceDef { label: "稳住身形 · 顶住水压", sub: "对抗 · 成功回 hub", cond: None,
        effects: &NO_EFF, route: Route::Dyn(route_pressure_escape) },
        ChoiceDef { label: "【向水压屈服】", sub: "未对抗成功 · 死亡档案", cond: None,
            effects: &[Eff::San(-10)], route: Route::Dyn(zone_pressure) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bh_21_anoxia", bg: Some("bihai_bg.png"), loc: Some("L3 · 缺氧区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["氧气瞬间见底。你若屏不住这口气，海水会顺着肺涌进来——成为深渊最新的一层。（对抗缺氧）"]),
    choices: &[ChoiceDef { label: "屏住呼吸 · 求生", sub: "对抗 · 成功回 hub", cond: None,
        effects: &NO_EFF, route: Route::Dyn(route_anoxia_escape) },
        ChoiceDef { label: "【放弃屏息】", sub: "未对抗成功 · 死亡档案", cond: None,
            effects: &[Eff::San(-10)], route: Route::Dyn(zone_anoxia) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结算卡（胜利/结束） ================= */
SceneDef {
    id: "bh_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "深 海 的 结 局".into(), good: true,
            body_html: format!(
                "<p>你从海沟深处回来，给「深渊邪物」的注视写下了一个结局。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>结局</td><td style='color:#9a958a'>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>{} 级</td></tr>\
                 </table>",
                st.points,
                if st.flag("bh_end_escape") { "逃离海沟 —— 带着目睹，活着上岸" }
                else if st.flag("bh_end_sacrifice") { "献祭自身 —— 换取一片宁静的海" }
                else if st.flag("bh_end_assimilate") { "与邪物同化 —— 借走了祂的一双眼" }
                else { "直面深渊（未作出抉择）" },
                st.sp_grade.unwrap_or('D'),
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案 · 深海的拥吻（复活扣 300 / 回主神） ================= */
SceneDef {
    id: "bh_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("深海的拥吻", "你死于深渊的注视与水压——被深海收进了它最新的一层")), card: |_st| crate::state::Card {
            title: "深 海 的 拥 吻".into(), good: false,
            body_html: r#"<p>海水在你肺里合拢，剖开它自己，好把你吞进去。深渊没有疼痛，也没有嘲笑——它只是，安详地，把你收进了它最新的一层。</p>
<p style='color:#ff8a8a'>【死亡档案 · 深海的拥吻】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
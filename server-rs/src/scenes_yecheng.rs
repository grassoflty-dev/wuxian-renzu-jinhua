//! 《无限曙光 · 我们是传奇》位面 · 夜城·孤独之城 全部剧情场景与战斗配置（world slug: yecheng，前缀 yc_）。
//! 本块为「原创致敬」拓展：00_wuxian_shuguang_research.md §5 备选五「孤独之城（我是传奇位面/我们是传奇卷）」——
//!   该位面仅列入研究候选清迈，未排入 00_INDEX_EXPANSION 骨架、无 10 节文档；本实现首次为其落地世界观。
//! 剧情开放、世界展示为主：白日搜寻/见证，入夜夜魔潮，决战夜魔·进化体之王；无绝对真相线/阴谋论。
//! 昼夜以叙事钟 flag 链推进（yc_nightfall → yc_deep_night），不引入真实回合计时（复用破晓/死雾默认方案）。
//! 本文件为全新文件，只导出静态数据（YECHENG_SCENES / yecheng_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/yecheng_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `yc_` 前缀；fight id 全部 `yc_` 前缀（与既有无重名）。
//! 动态难度：全部 FightCfg 写难度 1.0 基准值（HP 55/80/150），不手动适配玩家强度（缩放由引擎负责）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_deep_night(st: &GameState) -> bool { st.flag("yc_deep_night") }
fn cond_research(st: &GameState) -> bool { st.flag("yc_research") }
fn cond_boss_down(st: &GameState) -> bool { st.flag("yc_boss_down") }

/* =====================================================================
   分支/昼夜 flag 链工具
   ===================================================================== */
/// 入夜：置 yc_nightfall（L1 白日结束，进入 L2 街区）
fn go_night(st: &mut GameState) -> String {
    st.set_flag("yc_nightfall");
    "yc_l2_hub".to_string()
}
/// 深入夜色：在 L2 调查街区线索后置 yc_deep_night（解锁夜魔巢门禁）
fn go_deep_night(st: &mut GameState) -> String {
    if !st.flag("yc_deep_night") {
        st.set_flag("yc_deep_night");
        st.points += 50;
    }
    "yc_l3_hub".to_string()
}

/* ---- L1 白日调查点的改动路由（标点后回 L1 hub） ---- */
fn mark_radio(st: &mut GameState) -> String {
    st.set_flag("yc_radio_fixed");
    st.points += 30;
    "yc_l1_hub".to_string()
}
fn mark_super(st: &mut GameState) -> String { st.set_flag("yc_super_seen"); "yc_l1_hub".to_string() }
fn mark_mural(st: &mut GameState) -> String { st.set_flag("yc_mural_seen"); st.points += 20; "yc_l1_hub".to_string() }
fn mark_roof(st: &mut GameState) -> String { st.set_flag("yc_roof_seen"); "yc_l1_hub".to_string() }

/* ---- L2 入夜调查点的改动路由（标点后回 L2 hub） ---- */
fn mark_patrol(st: &mut GameState) -> String { st.set_flag("yc_patrol_seen"); st.points += 25; "yc_l2_hub".to_string() }
fn mark_bell(st: &mut GameState) -> String { st.set_flag("yc_bell_seen"); st.points += 20; "yc_l2_hub".to_string() }
fn mark_lamp(st: &mut GameState) -> String { st.set_flag("yc_lamp_seen"); "yc_l2_hub".to_string() }
fn mark_lab(st: &mut GameState) -> String {
    st.set_flag("yc_research");
    crate::world::add_item(st, "it_yc_blood"); // 研究员残留的夜魔血液样本
    st.points += 60;
    "yc_l2_hub".to_string()
}

/* ---- L3 巢中调查点 ---- */
fn mark_nest(st: &mut GameState) -> String { st.set_flag("yc_nest_seen"); st.points += 30; "yc_l3_hub".to_string() }
fn mark_altar(st: &mut GameState) -> String { st.set_flag("yc_altar_seen"); st.points += 30; "yc_l3_hub".to_string() }

/* ---- 结算 ---- */
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "yc_42_card".to_string()
}

/* ---- 夜魔之王 rage 桩 ---- */
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 原生杂兵战 win 目标路由（回当前 hub）
fn win_l2(_st: &GameState) -> String { "yc_l2_hub".to_string() }
fn win_boss(_st: &GameState) -> String { "yc_boss_win".to_string() }

/* =====================================================================
   选择驱动 BOSS：夜魔·进化体之王（HP150）
   黄金模板 C：start_boss / boss_act / boss_win
   ===================================================================== */
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("yc_boss") {
            st.fight = Some(crate::state::Fight {
                id: "yc_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "yc_boss_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 34 } else { 24 };
    let hit = !guard; // guard 时免伤
    if hit { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "yc_50_death".to_string(); }
    "yc_boss_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 400;
    st.set_flag("yc_boss_down");
    st.sp_grade = Some('D');
    crate::world::add_item(st, "it_yc_fang"); // 战利：进化体之牙
    "yc_boss_win".to_string()
}

/* =====================================================================
   战斗配置表（id 全部 yc_ 前缀）—— 3 场：普通夜魔游荡者 / 夜魔前锋 / BOSS 进化体之王
   数值为难度 1.0 基准（不缩放适配玩家强度）。
   ===================================================================== */
pub fn yecheng_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("yc_e_slave", FightCfg {
            name: "夜魔·游荡者", hp: 55, dmg: (10, 15), reward: 20, reward_why: "击杀夜魔·游荡者",
            intro: "路灯的蓝白色电压杂音里，一道灰白的影子贴着墙角站起——它的皮肤被阳光灼成碳色，只在夜里活着，喉咙里滚着破碎的嘶叫。",
            rage_at: Some(25), rage_text: "它听见巢里的呼应，灰白的四肢骤然发力——夜魔的狂潮要来了！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| String::new(),
            finisher_desc: |_st| String::new(),
            win: win_l2, death: "yc_50_death",
        }),
        ("yc_e_hunter", FightCfg {
            name: "夜魔·前锋", hp: 80, dmg: (13, 19), reward: 80, reward_why: "击杀夜魔·前锋",
            intro: "它比游荡者更高、更敏捷，胸口烙着一道旧伤，像是被某种武器逼退过。它挡在通往巢穴的方向，一步不让。",
            rage_at: Some(35), rage_text: "它怒吼着扯下一段铁管作武器——进化体的凶性被彻底点燃！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| String::new(),
            finisher_desc: |_st| String::new(),
            win: win_l2, death: "yc_50_death",
        }),
        ("yc_boss", FightCfg {
            name: "夜魔·进化体之王", hp: 150, dmg: (18, 26), reward: 400, reward_why: "击杀夜魔·进化体之王",
            intro: "巢心处，一头体型远超同族的夜魔之王蜷绕在血祭台上。它的眼睛映着最后一缕火光——那是曾经的「人」留下的最后一寸清明，如今只剩狩猎的本能。",
            rage_at: Some(60), rage_text: "它仰首长啸，巢穴深处传来回应的嘶鸣——夜王要催动整座巢的夜魔潮！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| String::new(),
            finisher_desc: |_st| String::new(),
            win: win_boss, death: "yc_50_death",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn yc_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    yecheng_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 yc_ 前缀）
   ===================================================================== */
pub static YECHENG_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "yc_00", bg: Some("yc_bg_city.png"), loc: Some("夜城 · 孤独之城 ≪我们是传奇·原创致敬≫"),
    mood: "somber", speaker: Some("主神·任务发布"), voice: Some("vo_yecheng_1"),
    text: TextSpec::Static(&[
        "<b>【主线任务·孤独之城】</b>这是人类文明的废墟——病毒把城市里九成的人变成了昼伏夜出的夜魔。白天，你是阳光下最后一个幸存者；夜晚，你会在街角听见它们的心跳。",
        "任务：<i>活着等第一个日出</i>。找到可以修好的无线电、一位愿意把故事讲给你的幸存者，或者干脆——掀翻夜魔的巢。结算评级由你留下的「世界记忆」决定。",
    ]),
    choices: &[
        ChoiceDef { label: "环顾安全屋", sub: "+10 点 · 接管这栋日间废弃超市", cond: None,
            effects: &[Eff::Points(10)], route: Route::To("yc_l1_hub") },
        ChoiceDef { label: "望向窗外的落日", sub: "San-2 · 你还剩多少白昼？", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("yc_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 白昼 · 日间超市/安全屋 hub ================= */
SceneDef {
    id: "yc_l1_hub", bg: Some("yc_bg_super.png"), loc: Some("L1 · 白昼 · 日间超市/安全屋"),
    mood: "calm", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yc_nightfall") {
            "你已深入夜色。正午的阳光让这栋废墟重归寂静——夜魔都缩回了地下。也许是时候再搜一次，然后考虑那扇通往更暗处的门。".to_string()
        } else {
            "日光从破窗倾泻，货架被洗劫一空，只有灰尘。这里白天属于你，夜晚则属于那些会在墙根底下爬的影子。（坏掉的无线电 / 空荡的超市货架 / 墙上的幸存者涂鸦 / 天台望远镜 / 幸存者老何 / 巡逻广播员）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "坏掉的无线电", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("yc_01_radio") },
        ChoiceDef { label: "空荡的超市货架", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("yc_02_super") },
        ChoiceDef { label: "墙上的幸存者涂鸦", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("yc_03_mural") },
        ChoiceDef { label: "天台望远镜", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("yc_04_roof") },
        ChoiceDef { label: "与幸存者老何交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("yc_n1_survivor") },
        ChoiceDef { label: "与巡逻广播员交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("yc_n2_broadcast") },
        ChoiceDef { label: "走出安全屋 · 入夜", sub: "置 yc_nightfall · 进 L2 街区", cond: None, effects: &NO_EFF, route: Route::To("yc_20_dusk") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 白昼调查点 ---- */
SceneDef {
    id: "yc_01_radio", bg: Some("yc_bg_super.png"), loc: Some("L1 · 安全屋 · 无线电台"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["一台老旧收音机，旋钮拧下来只剩沙沙的电流声。你把断裂的天线重新接上——如果修好它，也许能在日落后听到别处还有活人的回响。"]),
    choices: &[ChoiceDef { label: "【修好无线电】", sub: "yc_radio_fixed · +30 点留痕", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l1_radio")], route: Route::Dyn(mark_radio) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_02_super", bg: Some("yc_bg_super.png"), loc: Some("L1 · 空荡的超市货架"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["冷冻柜早已断电，墙面被潮气泡出霉斑。架子上零星散着几罐过期的罐头——这是这栋楼里「人」的最后痕迹，如今只剩你来替它们计数。"]),
    choices: &[ChoiceDef { label: "点货架上的罐头", sub: "伏笔 · San+1", cond: None,
        effects: &[Eff::San(1), Eff::MarkPoint("yc_p_l1_super")], route: Route::Dyn(mark_super) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_03_mural", bg: Some("yc_bg_super.png"), loc: Some("L1 · 墙上的幸存者涂鸦"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["墙上用煤块画着歪歪扭扭的字：「别信广播里的，这里还有十七个人。」下面又补了一行，「十七——现在十六。」你数了数那些刻痕，像是有人在漫长地、一个名字一个名字地悼念。"]),
    choices: &[ChoiceDef { label: "记住墙上的话", sub: "yc_mural_seen · +20 点", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l1_mural")], route: Route::Dyn(mark_mural) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_04_roof", bg: Some("yc_bg_city.png"), loc: Some("L1 · 天台望远镜"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["天台上立着一架军用望远镜，镜片被拖把布裹着。透过它，你能看见远处街区教堂的尖顶，和一条直通市中心的电缆线——那是夜魔巢的方向。"]),
    choices: &[ChoiceDef { label: "遥望夜魔巢方向", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l1_roof")], route: Route::Dyn(mark_roof) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 NPC ---- */
SceneDef {
    id: "yc_n1_survivor", bg: Some("yc_bg_super.png"), loc: Some("L1 · 安全屋 · 里间"),
    mood: "cold", speaker: Some("幸存者·老何"), voice: None,
    text: TextSpec::Static(&["老何递给你半块压缩饼干，声音干涩：「别出去太晚。夜魔……它们在第三盏路灯之后就成了群。你要找活路，就去教堂钟楼上——黎明时谁能敲响那口钟，谁就能把这整条街的夜魔困在屋里。」"]),
    choices: &[ChoiceDef { label: "「那口钟上，真的有钟舌吗？」", sub: "老何摇摇头 · San+2", cond: None,
        effects: &[Eff::San(2)], route: Route::To("yc_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_n2_broadcast", bg: Some("yc_bg_city.png"), loc: Some("L1 · 街角广播车"),
    mood: "calm", speaker: Some("巡逻广播员"), voice: None,
    text: TextSpec::Static(&["一辆改装的广播车停在街角，扬声器循环播放着沙哑的人声：「……幸存者请注意，幸存者请注意，日出后请前往旧码头集合。夜魔不可怕，可怕的是你一个人在夜里……停，这里有活人吗，喂，喂。」"]),
    choices: &[ChoiceDef { label: "「修好无线电，我们把频道对上。」", sub: "him 眼睛一亮 · yc_guard_signal 支线线索", cond: None,
        effects: &[Eff::SetFlag("yc_guard_signal_hint")], route: Route::To("yc_l1_hub") },
        ChoiceDef { label: "记下车里那张城区地图", sub: "伏笔 · San+1", cond: None,
            effects: &[Eff::San(1)], route: Route::To("yc_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 → L2 入夜（叙事钟） ================= */
SceneDef {
    id: "yc_20_dusk", bg: Some("yc_bg_dusk.png"), loc: Some("L1 → L2 · 落日斜巷"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["你把门从身后带上。日头沉下去，城市像被谁拧灭了所有灯。第三盏路灯「啪」地亮起，随后是第二盏、第一盏——你听见远处传来第一声不属于任何活物的嘶叫。"]),
    choices: &[ChoiceDef { label: "攥紧手电走进夜色", sub: "置 yc_nightfall · 进 L2 街区", cond: None,
        effects: &[Eff::San(-3)], route: Route::Dyn(go_night) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 入夜 · 街区 hub ================= */
SceneDef {
    id: "yc_l2_hub", bg: Some("yc_bg_street.png"), loc: Some("L2 · 入夜 · 街区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yc_deep_night") {
            "街道彻底回响着夜魔的低鸣。你避过一道探照灯似的月光，决定遁入那条通往更深处的铁栅门——夜魔巢的入口就在地底。".to_string()
        } else {
            "路灯的光把它们逼在阴影里，但你能听见墙根下数不清的爬行声。找线索，或直接和挡路的夜魔过招。（废弃警车 / 教堂钟楼 / 最后一盏路灯 / 研究员地下实验室 / 独行的德牧 / 夜色杀机 / 夜魔巢铁栅门）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "废弃警车", sub: "调查 · 警用电台线索", cond: None, effects: &NO_EFF, route: Route::To("yc_05_patrol") },
        ChoiceDef { label: "教堂钟楼", sub: "调查 · 老何的提示", cond: None, effects: &NO_EFF, route: Route::To("yc_06_bell") },
        ChoiceDef { label: "最后一盏路灯", sub: "调查 · 夜魔习性", cond: None, effects: &NO_EFF, route: Route::To("yc_07_lamp") },
        ChoiceDef { label: "研究员地下实验室", sub: "调查 · 样本", cond: None, effects: &NO_EFF, route: Route::To("yc_08_lab") },
        ChoiceDef { label: "与独行的德牧交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("yc_n3_dog") },
        ChoiceDef { label: "逼近暗处响动", sub: "遭遇战 yc_e_slave", cond: None, effects: &NO_EFF, route: Route::To("yc_f_slave") },
        ChoiceDef { label: "撞上夜魔前锋", sub: "遭遇战 yc_e_hunter", cond: None, effects: &NO_EFF, route: Route::To("yc_f_hunter") },
        ChoiceDef { label: "深入夜色 · 下夜魔巢", sub: "置 yc_deep_night · 进 L3", cond: None, effects: &NO_EFF, route: Route::Dyn(go_deep_night) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 入夜调查点 ---- */
SceneDef {
    id: "yc_05_patrol", bg: Some("yc_bg_street.png"), loc: Some("L2 · 废弃警车"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["警车停在路口，车门开着，驾驶座上只有一枚警徽和一台还能亮的警用电台。电台里循环着一段录音：「……夜魔的巢，就在地铁一号线塌方处，白天它们是死人，晚上它们是猎手，别让它们在黄昏堵住你……重复，别让它们在黄昏堵住你。」"]),
    choices: &[ChoiceDef { label: "记下车内电台", sub: "yc_patrol_seen · +25 点", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l2_patrol")], route: Route::Dyn(mark_patrol) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_06_bell", bg: Some("yc_bg_church.png"), loc: Some("L2 · 教堂钟楼"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["钟楼的铜钟还在，钟舌被布条缠住——老何说过，黎明敲响它能把整个街区的夜魔困住。但此刻夜色正浓，你不敢敲。你在钟柱底找到一截被血浸透的绳，像是有人试过，又放弃了。"]),
    choices: &[ChoiceDef { label: "记下钟声的位置", sub: "yc_bell_seen · +20 点", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l2_bell")], route: Route::Dyn(mark_bell) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_07_lamp", bg: Some("yc_bg_street.png"), loc: Some("L2 · 最后一盏路灯"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["街角那盏路灯被一件旧大衣罩着，只漏出一条光缝。你读懂了这个街区的生存法则：<i>光，是白昼的碎片。</i>只要穿上「白天」，夜魔就会在阴影里爬过。"]),
    choices: &[ChoiceDef { label: "披上旧大衣贴近照明区", sub: "伏笔 · San+2", cond: None,
        effects: &[Eff::San(2), Eff::MarkPoint("yc_p_l2_lamp")], route: Route::Dyn(mark_lamp) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_08_lab", bg: Some("yc_bg_lab.png"), loc: Some("L2 · 研究员地下实验室"),
    mood: "cold", speaker: Some("研究员·临终笔记（录音）"), voice: None,
    text: TextSpec::Static(&["地下实验室的灯亮着，像一座孤岛。搁在工作台上的笔记写着：「病毒让『人』退回了『猎物』——它们怕光，聚在黑暗，听巢的声音。但我提取的这批血液样本里，有一样东西被人忽略了：<i>它们也会做梦。</i>那梦里，又有一个太阳。」"]),
    choices: &[ChoiceDef { label: "【取走夜魔血液样本】", sub: "yc_research · it_yc_blood · +60 点", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l2_lab")], route: Route::Dyn(mark_lab) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 NPC ---- */
SceneDef {
    id: "yc_n3_dog", bg: Some("yc_bg_street.png"), loc: Some("L2 · 街角 · 独行的德牧"),
    mood: "cold", speaker: Some("独行的德牧"), voice: None,
    text: TextSpec::Static(&["一只瘦骨嶙峋的德牧蹲在路灯下，叼着半枚名牌，项圈上摩挲着一枚褪色的哨子。它一定曾是某支搜寻队的同伴。它朝地下实验室的方向低吠了两声，又朝你摇摇尾巴——像是替你守望这个街区。"]),
    choices: &[ChoiceDef { label: "喂它半块压缩饼干", sub: "San+2 · 它认得了你的气味", cond: None,
        effects: &[Eff::San(2)], route: Route::To("yc_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 遭遇战（原生 fight）---- */
SceneDef {
    id: "yc_f_slave", bg: Some("yc_bg_street.png"), loc: Some("L2 · 夜色杀机 · 游荡者"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["路灯电压骤闪，一道灰白影子从阴影里直扑过来——它比你见过的任何夜魔都要快。"]),
    choices: &NO_CH, fight_id: Some("yc_e_slave"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_f_hunter", bg: Some("yc_bg_street.png"), loc: Some("L2 · 夜色杀机 · 前锋"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["一道更高的身影挡住去路，胸口旧伤发亮——这是夜魔里的前锋，专门负责驱赶走散的人回巢。"]),
    choices: &NO_CH, fight_id: Some("yc_e_hunter"), video: None, cine_label: None, overlay: None,
},

/* ================= L2 → L3 夜魔巢（门禁 yc_g2 提示场景） ================= */
SceneDef {
    id: "yc_21_nest_gate", bg: Some("yc_bg_tunnel.png"), loc: Some("L2 · 夜魔巢铁栅门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yc_deep_night") {
            "你已深入夜色，铁栅门缝里漏出的风裹着铁锈与血腥。你握住门，走向那段通往地底夜魔巢的楼梯。".to_string()
        } else {
            "锈死的铁栅门上指甲抓痕密密麻麻地向内延伸。你还没有深入夜色——此刻硬闯，只会被夜魔潮吞掉。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "推开铁栅门下巢", sub: "需 yc_deep_night · 进 L3", cond: Some(cond_deep_night),
            effects: &NO_EFF, route: Route::To("yc_l3_hub") },
        ChoiceDef { label: "退回街区", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yc_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 深处 · 夜魔巢 hub ================= */
SceneDef {
    id: "yc_l3_hub", bg: Some("yc_bg_nest.png"), loc: Some("L3 · 深处 · 夜魔巢"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yc_boss_down") {
            "血祭台死寂，夜魔之王倒下后的巢，像退潮后的滩涂——那些嘶鸣声正渐渐散进更深处的地铁隧道。黎明，应该快到了。".to_string()
        } else {
            "地铁隧道的穹顶浸着水，回响着绵长的趴行声。你是这些年来第一个活着走到这里的「猎物」。（夜魔巢残迹 / 血祭台 / 撤离信标 / 决战夜魔之王）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "夜魔巢残迹", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("yc_09_nest") },
        ChoiceDef { label: "血祭台", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("yc_10_altar") },
        ChoiceDef { label: "撤离信标", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("yc_11_exit") },
        ChoiceDef { label: "决战夜魔之王", sub: "BOSS 战 yc_boss（HP150）", cond: None, effects: &NO_EFF, route: Route::To("yc_boss_intro") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 巢中调查点 ---- */
SceneDef {
    id: "yc_09_nest", bg: Some("yc_bg_nest.png"), loc: Some("L3 · 夜魔巢残迹"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["这座巢的「建材」很古怪——纸箱、旧轮胎、烧过的铁皮，还有一截歪斜的巴士站牌。夜魔并非野兽，它们拾捡、垒叠，像在模仿某种早已遗忘的建筑。你对这个物种的地平线又多了一分警惕。"]),
    choices: &[ChoiceDef { label: "记下巢的结构", sub: "yc_nest_seen · +30 点", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l3_nest")], route: Route::Dyn(mark_nest) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_10_altar", bg: Some("yc_bg_nest.png"), loc: Some("L3 · 血祭台"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["血祭台中央嵌着一枚残破的路牌「希望街」。你忽然明白，这些「猎手」也许正试图守住它们最后的、属于「人」的遗迹——哪怕手段早已与当初的守望背道而驰。"]),
    choices: &[ChoiceDef { label: "记下这枚路牌", sub: "yc_altar_seen · +30 点", cond: None,
        effects: &[Eff::MarkPoint("yc_p_l3_altar")], route: Route::Dyn(mark_altar) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 BOSS 战（选择驱动） ================= */
SceneDef {
    id: "yc_boss_intro", bg: Some("yc_bg_nest.png"), loc: Some("L3 · 巢心 · 血祭台"),
    mood: "danger", speaker: Some("夜魔·进化体之王"), voice: None,
    text: TextSpec::Static(&["巢心的空气停止流动。夜魔之王从血祭台上缓缓站起，比想象中更像「人」——只是那身灰白的皮肤和发红的眼，已在漫长的夜晚里褪去了所有温驯。它张开嘴，发出一声极低、极沉的叹息。"]),
    choices: &[
        ChoiceDef { label: "出手破夜王", sub: "BOSS 战 yc_boss（HP150）", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
        ChoiceDef { label: "转身冲向撤离信标", sub: "不战 · 撤离结算", cond: None, effects: &NO_EFF, route: Route::To("yc_settle") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_boss_round", bg: Some("yc_bg_nest.png"), loc: Some("L3 · 巢心 · 决战"), mood: "danger",
    speaker: Some("夜魔·进化体之王"), voice: None,
    text: TextSpec::Dyn(|st| format!("夜王剩余 {} 血，你 HP {}。它的嚎叫在隧道里层层回荡，招来整座巢的回声。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "重击夜王", sub: "高伤害", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 34, false)) },
        ChoiceDef { label: "借壁躲过扑咬", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_boss_win", bg: Some("yc_bg_nest.png"), loc: Some("L3 · 巢心"),
    mood: "calm", speaker: Some("夜魔·进化体之王"), voice: None,
    text: TextSpec::Static(&["夜王轰然倒下，那声叹息终于不再。一缕晨光从塌方的缝隙漏进来，落在它的眼窝上——它竟没有像其它夜魔那样灼烧，而是静静地、像一枚终于熄掉的太阳。你在它的牙床下取走那枚象征终局的利齿。"]),
    choices: &[ChoiceDef { label: "对照废墟记录", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("yc_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 撤离信标 / 结算 ================= */
SceneDef {
    id: "yc_11_exit", bg: Some("yc_bg_dawn.png"), loc: Some("L3 · 撤离信标"),
    mood: "calm", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["信标是残破的交通灯改装的，绿灯撑着一整夜。你伸手按下它，刺耳的联络音穿过废墟——像有人在世界的另一端，第一次为你开了一盏灯。"]),
    choices: &[ChoiceDef { label: "对照废墟记录", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("yc_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_settle", bg: Some("yc_bg_dawn.png"), loc: Some("孤独之城 · 黎明结算"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yc_boss_down") && st.flag("yc_research") {
            "你掀了夜魔巢，又带回了研究员的血液样本与进化体的利齿。这座孤独之城记住了你——不是作为猎物，而是作为那束在天亮前把它们从黑暗里看清楚的「光」。".to_string()
        } else if st.flag("yc_boss_down") {
            "你掀了夜魔巢，夜魔之王倒在你脚下。这座城终于没有了那声整夜的叹息。你带着战利，等到了第一个日出。".to_string()
        } else {
            "你活着等到了日出。有些事你留下了，有些没有——但这座城市的一角，因为你昨晚的所作所为，比昨天多亮了一盏灯。主线判定你的支线结算评级。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【掀巢之路】", sub: "破了夜王 · 结算", cond: Some(cond_boss_down),
            effects: &NO_EFF, route: Route::Dyn(route_settle) },
        ChoiceDef { label: "【样本之路】", sub: "带回样本 · 结算", cond: Some(cond_research),
            effects: &NO_EFF, route: Route::Dyn(route_settle) },
        ChoiceDef { label: "确认走向撤离光柱", sub: "结算 · 评级", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yc_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "天 亮 了".into(), good: true,
            body_html: format!(
                "<p>你推开废墟的门，第一缕晨光落在你肩上。夜魔退回地下，街道第一次安静得像一座普通城市。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>城中记忆</td><td style='color:#9a958a'>无线电{} 样本{} 掀巢{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级{}</td></tr>\
                 </table>",
                st.points,
                if st.flag("yc_radio_fixed") { "✓" } else { "✗" },
                if st.flag("yc_research") { "✓" } else { "✗" },
                if st.flag("yc_boss_down") { "✓" } else { "✗" },
                if st.flag("yc_boss_down") { " · 夜王已破" } else { "" },
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案 · 夜魔围猎（复活回主神） ================= */
SceneDef {
    id: "yc_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("夜魔之手", "在夜色中被围猎吞噬")), card: |_st| crate::state::Card {
            title: "没 有 等 到 天 亮".into(), good: false,
            body_html: r#"<p>你倒下的那一刻，路灯光斑里站满了灰白的影子。它们没有嚎叫，只是静静地看着你——像在看一截又一次熄灭的太阳。</p>
<p style='color:#ff8a8a'>【死亡档案 · 夜魔围猎】</p>
<p style='color:#666'>（复活：回主神空间扣除点数，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

//! 《无限恐怖 · 黑珍珠》全部剧情场景与航海冒险「展示向」世界配置。
//! 设计依据 game_design · 海盗冒险展示向骨架（无真相线，开放结局）。
//! 本文件为全新新增文件，只导出静态数据（JIALEBI_SCENES / jialebi_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/jialebi_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `jb_` 前缀；fight id 全部 `jb_` 前缀（与既有无重名）。
//! 场景数 30 个 SceneDef，开场钩子 → L1 hub → 调查点 → L2 hub → 调查点 → L3 hub → BOSS 铺垫
//!   → BOSS round → 开放结局 → 结算卡 + 死亡卡。
//! 核心定位：三层逐层推进的开放冒险——
//!   L1 黑珍珠海盗船（舵轮 / 瞭望台 / 火药桶 / 船长室 / 醉水手战）
//!   → L2 沉船湾（宝箱 / 沉船 / 渡板 / 锈锚 / 巨爪蟹战，收集藏宝图开门）
//!   → L3 财宝洞（月光水池 / 祭坛 / 白骨堆 / 拟态宝箱怪战）
//!   终点：选择驱动 BOSS「亡灵船长·巴博萨」（HP 220）→ 击败进入开放结局 → 结算卡；
//!   途中任意失败 → 死亡档案（复活扣 300 回主神）。
//! bg 占位待主线替换：img_zhuyuan_book.png / img_laser.png / img_corridor.png。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_compass(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "jb_compass") }
fn cond_boss_primed(st: &GameState) -> bool { st.flag("jb_boss_primed") }

/* =====================================================================
   路由 fn（返回 String）与杂项工具
   ===================================================================== */
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "jb_42_card".to_string()
}
fn route_l1_hub(_st: &GameState) -> String { "jb_l1_hub".to_string() }
fn route_l2_hub(_st: &GameState) -> String { "jb_l2_hub".to_string() }
fn route_l3_hub(_st: &GameState) -> String { "jb_l3_hub".to_string() }
/// 选择驱动 BOSS 的 FightCfg.win 占位（真实结算走 boss_win(&mut) 链，此处不被调用）
fn win_boss_stub(_st: &GameState) -> String { "jb_boss_win".to_string() }

/* ---- L3 坍方区：有罗盘则绕过，否则活埋（环境危区） ---- */
fn zone_cavein(st: &mut GameState) -> String {
    if st.inventory.iter().any(|i| i == "jb_compass") {
        st.points += 40;
        st.set_flag("jb_dodge_cavein");
        "jb_l3_hub".to_string()
    } else {
        "jb_51_death_fight".to_string()
    }
}

/* ---- 亡灵船长·巴博萨（选择驱动 BOSS）黄金模板 C 段 ---- */
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}
fn fins_if(_st: &GameState, _ehp: i32) -> bool { false }
fn fins_name(_st: &GameState) -> String { String::new() }
fn fins_desc(_st: &GameState) -> String { String::new() }

fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jb_boss") {
            st.fight = Some(crate::state::Fight {
                id: "jb_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "jb_boss_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 24 } else { 18 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "jb_50_death".to_string(); }
    "jb_boss_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500;
    st.set_flag("jb_boss_down");
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    crate::world::add_item(st, "jb_black_pearl");
    "jb_boss_win".to_string()
}

/* =====================================================================
   战斗配置表（id 全部 jb_ 前缀）
   1 场选择驱动 BOSS（jb_boss）+ 3 场原生小怪战（每层一场）
   ===================================================================== */
pub fn jialebi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("jb_boss", FightCfg {
            name: "亡灵船长·巴博萨", hp: 220, dmg: (18, 28), reward: 500, reward_why: "亡灵船长·巴博萨的最后一击",
            intro: "亡灵船长的骨手从月光里探出，甲板上晾起的黑帆嘶嘶燃烧成灰。他咧嘴一笑，露出一排被诅咒的牙齿：「宝藏？先拿命来换。」",
            rage_at: Some(120), rage_text: "巴博萨的船身从海水里立起，亡灵船员齐声嚎叫——他进入狂暴，刀锋更狠。",
            on_rage: rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_boss_stub, death: "jb_50_death",
        }),
        ("jb_fight_l1", FightCfg {
            name: "醉水手", hp: 60, dmg: (8, 12), reward: 80, reward_why: "制伏甲板上的醉水手",
            intro: "醉水手拎着酒瓶摔到你面前，咧嘴一笑，招呼了一嗓子。他晃晃悠悠朝你扑来，甲板上的弟兄把他当笑料。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: route_l1_hub, death: "jb_51_death_fight",
        }),
        ("jb_fight_l2", FightCfg {
            name: "巨爪蟹", hp: 90, dmg: (10, 16), reward: 120, reward_why: "击退礁湾的巨爪蟹",
            intro: "沙丘下的巨爪蟹猛地掀翻沙土，钳子像两扇城门一样朝你合拢。它的壳上嵌着一枚海盗银币。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: route_l2_hub, death: "jb_51_death_fight",
        }),
        ("jb_fight_l3", FightCfg {
            name: "拟态匣·宝箱怪", hp: 110, dmg: (12, 18), reward: 160, reward_why: "刺穿扮成宝箱的巨口",
            intro: "你伸手翻开宝箱盖——盖子却张开成一张满是倒刺的巨口，朝你兜头咬下。这是一只「拟态匣」，专吃贪财的冒险者。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: route_l3_hub, death: "jb_51_death_fight",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn jb_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    jialebi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 jb_ 前缀）
   ===================================================================== */
pub static JIALEBI_SCENES: &[SceneDef] = &[

/* ================= 开场钩子 ================= */
SceneDef {
    id: "jb_00", bg: Some("jialebi_bg.png"), loc: Some("主神空间 · 任务发布 ≪黑珍珠≫"),
    mood: "mystery", speaker: Some("主神·任务发布"), voice: None,
    text: TextSpec::Static(&[
        "<b>【支线任务 · 黑珍珠宝藏】</b>登上黑珍珠号，穿过沉船湾，潜入财宝洞，取回传说中的呼吸之银。<i>「这是一方等着被写进传说的海。」</i>",
        "海风裹着咸腥扑面而来。你被抛在被诅咒的黑珍珠甲板上，远处舵轮在月光下闪着冷光。没有真相可追，只有一场值得被传唱的冒险。",
    ]),
    choices: &[
        ChoiceDef { label: "环顾甲板", sub: "记住这艘船的轮廓 · +5 点", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("jb_l1_hub") },
        ChoiceDef { label: "深吸一口海风", sub: "San+2 · 定神", cond: None,
            effects: &[Eff::San(2)], route: Route::To("jb_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 黑珍珠海盗船 hub ================= */
SceneDef {
    id: "jb_l1_hub", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 黑珍珠甲板"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "船身在浪里轻轻起伏。舵轮、主桅瞭望台、堆着火药桶的舱口、船长室依次排开。戴上眼的船员见了陌生人，咯咯笑着给你指路——指哪里都是一场赌局。",
        "（舵轮 / 瞭望台 / 火药桶 / 船长室 / 交谈 / 战醉水手 / 下舷梯去沉船湾）",
    ]),
    choices: &[
        ChoiceDef { label: "舵轮", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("jb_01_wheel") },
        ChoiceDef { label: "主桅瞭望台", sub: "俯瞰海图", cond: None, effects: &NO_EFF, route: Route::To("jb_01_mast") },
        ChoiceDef { label: "火药桶堆", sub: "翻找补给 · 得朗姆酒", cond: None, effects: &NO_EFF, route: Route::To("jb_01_barrel") },
        ChoiceDef { label: "船长室门", sub: "巴博萨的厢房", cond: None, effects: &NO_EFF, route: Route::To("jb_01_cabin") },
        ChoiceDef { label: "与船厨阿朵交谈", sub: "线索与朗姆", cond: None, effects: &NO_EFF, route: Route::To("jb_npc_cook") },
        ChoiceDef { label: "战醉水手", sub: "战斗 jb_fight_l1", cond: None, effects: &NO_EFF, route: Route::To("jb_l1_fight") },
        ChoiceDef { label: "沿舷梯下到沉船湾", sub: "→ L2", cond: None, effects: &NO_EFF, route: Route::To("jb_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点 ---- */
SceneDef {
    id: "jb_01_wheel", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 舵轮"),
    mood: "cold", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["铜铸的舵轮被海盐锈得发绿，罗盘指针疯转不歇。一半刻着航线，一半刻着「亡灵」的注脚——这艘船不靠风行驶，它靠「还不了的债」。"]),
    choices: &[ChoiceDef { label: "转舵一圈", sub: "+10 点", cond: None,
        effects: &[Eff::Points(10), Eff::MarkPoint("jb_p_l1_wheel")], route: Route::To("jb_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_01_mast", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 主桅瞭望台"),
    mood: "awe", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["爬上湿滑的绳梯，从瞭望台往下望——黑色船帆层层叠叠，海面尽头躺着一座搁浅的船坞。你默默记下了沉船湾的方向。"]),
    choices: &[ChoiceDef { label: "记下航道", sub: "San+2 · 指向 L2", cond: None,
        effects: &[Eff::San(2), Eff::MarkPoint("jb_p_l1_mast")], route: Route::To("jb_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_01_barrel", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 火药桶堆"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["火药桶码得整整齐齐，桶缝里塞着一只漏着酒味的皮囊。这尸是放哨弟兄偷藏的朗姆酒——船厨阿朵正找它呢。"]),
    choices: &[ChoiceDef { label: "顺走朗姆酒", sub: "Item jb_rum · 开船员舱闩门", cond: None,
        effects: &[Eff::AddItem("jb_rum"), Eff::MarkPoint("jb_p_l1_barrel")], route: Route::To("jb_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_01_cabin", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 船长室"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["船长室里空无一人，桌上摊着一卷羊皮海图，红叉指向一座海岛洞穴。海图边一行小字：<i>「财宝洞 · 还清了债的，才能活着走出来。」</i>"]),
    choices: &[ChoiceDef { label: "收起羊皮海图", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("jb_p_l1_cabin")], route: Route::To("jb_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_npc_cook", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 厨房"),
    mood: "calm", speaker: Some("船厨·阿朵"), voice: None,
    text: TextSpec::Static(&["阿朵一边剁鱼一边抬眼：「这船欠了海神一船的债，船长把自己典给了月亮。你要去财宝洞？先弄点朗姆酒，让看舱门的醉鬼放你过去——他那点儿酒瘾，拿一桶就够了。」"]),
    choices: &[ChoiceDef { label: "「一桶朗姆，谢啦。」", sub: "San+3", cond: None,
        effects: &[Eff::San(3)], route: Route::To("jb_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_l1_fight", bg: Some("jialebi_bg_deck.png"), loc: Some("L1 · 甲板 · 战斗"),
    mood: "danger", speaker: Some("醉水手"), voice: None,
    text: TextSpec::Static(&["醉水手几步跨到你面前，酒瓶往地上一掼：「新来的？先教教你，谁的船。」（战斗）"]),
    choices: &[], fight_id: Some("jb_fight_l1"), video: None, cine_label: None, overlay: None,
},

/* ================= L2 沉船湾 hub ================= */
SceneDef {
    id: "jb_l2_hub", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 沉船湾 · 礁石湾"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "舷梯尽头的浅滩横着一艘半沉的古船，桅杆劈裂成獠牙。锈蚀巨锚砸进沙里，潮水在宝箱与渡板间来回舔。独眼海盗坐在断桅头，眯眼打量你。",
        "（宝箱 / 沉船残骸 / 湿滑渡板 / 锈蚀巨锚 / 交谈 / 战巨爪蟹 / 进入财宝洞）",
    ]),
    choices: &[
        ChoiceDef { label: "海蚀洞·宝箱", sub: "得藏宝图", cond: None, effects: &NO_EFF, route: Route::To("jb_02_chest") },
        ChoiceDef { label: "沉船残骸", sub: "调查船舱", cond: None, effects: &NO_EFF, route: Route::To("jb_02_wreck") },
        ChoiceDef { label: "湿滑渡板", sub: "渡礁 · 得罗盘", cond: None, effects: &NO_EFF, route: Route::To("jb_02_plank") },
        ChoiceDef { label: "锈蚀巨锚", sub: "割发币", cond: None, effects: &NO_EFF, route: Route::To("jb_02_anchor") },
        ChoiceDef { label: "与独眼海盗交谈", sub: "领路暗礁水道", cond: None, effects: &NO_EFF, route: Route::To("jb_npc_pirate") },
        ChoiceDef { label: "战巨爪蟹", sub: "战斗 jb_fight_l2", cond: None, effects: &NO_EFF, route: Route::To("jb_l2_fight") },
        ChoiceDef { label: "潜入财宝洞", sub: "→ L3", cond: None, effects: &NO_EFF, route: Route::To("jb_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 调查点 ---- */
SceneDef {
    id: "jb_02_chest", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 海蚀洞·宝箱"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["暗礁洞里藏着一口蒙尘的宝箱，箱盖压着一张泛黄的藏宝图，红圈圈住的正是「财宝洞」的入口航线。没有它，暗礁水道十船九翻。"]),
    choices: &[ChoiceDef { label: "取走藏宝图", sub: "Item jb_treasure_map · 开暗礁水道闸", cond: None,
        effects: &[Eff::AddItem("jb_treasure_map"), Eff::MarkPoint("jb_p_l2_chest")], route: Route::To("jb_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_02_wreck", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 沉船残骸"),
    mood: "cold", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["船骸的舱壁上钉着七具白骨，骨手握着一枚枚烂成渣的银币。他们到死都攥着财宝不放——那场风暴，是海神来讨债的。"]),
    choices: &[ChoiceDef { label: "记下这处警示", sub: "+10 点", cond: None,
        effects: &[Eff::Points(10), Eff::MarkPoint("jb_p_l2_wreck")], route: Route::To("jb_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_02_plank", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 湿滑渡板"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["一根淌着海水的破木板从礁石伸向对岸，板下是旋涡。你踩稳节奏踏过去，板缝里别着一只锈而准的旧罗盘——落洞前它保准能用上。"]),
    choices: &[ChoiceDef { label: "捡起旧罗盘", sub: "Item jb_compass · 防坍方", cond: None,
        effects: &[Eff::AddItem("jb_compass"), Eff::MarkPoint("jb_p_l2_plank")], route: Route::To("jb_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_02_anchor", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 锈蚀巨锚"),
    mood: "cold", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["巨锚锈得和礁石长在一起，锚环上缠着一枚被海盐磨亮的银币，像是海神落下的零钱。你把它掂了掂，塞进怀里。"]),
    choices: &[ChoiceDef { label: "取走银币", sub: "+15 点", cond: None,
        effects: &[Eff::Points(15), Eff::MarkPoint("jb_p_l2_anchor")], route: Route::To("jb_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_npc_pirate", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 断桅头"),
    mood: "calm", speaker: Some("独眼海盗"), voice: None,
    text: TextSpec::Static(&["独眼海盗嚼着干鱼，用船钩往财宝洞方向一指：「那洞收人很凶。要有张像样的藏宝图认得暗礁水道；进了洞，看紧头顶——洞顶的石头比人还馋。」"]),
    choices: &[ChoiceDef { label: "「我有罗盘备着。」", sub: "San+3 · 关键提醒", cond: Some(cond_has_compass),
        effects: &[Eff::San(3)], route: Route::To("jb_l2_hub") },
        ChoiceDef { label: "「记下你说的。」", sub: "提示坍方", cond: None, effects: &NO_EFF, route: Route::To("jb_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_l2_fight", bg: Some("jialebi_bg_cove.png"), loc: Some("L2 · 礁湾 · 战斗"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["沙土一松，巨爪蟹破土而出。它认得你怀里的银币气味，钳子朝你兜头合拢。（战斗）"]),
    choices: &[], fight_id: Some("jb_fight_l2"), video: None, cine_label: None, overlay: None,
},

/* ================= L3 财宝洞 hub ================= */
SceneDef {
    id: "jb_l3_hub", bg: Some("jialebi_bg.png"), loc: Some("L3 · 财宝洞 · 洞穴宝库"),
    mood: "awe", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "穿过暗礁水道，洞顶漏下一束月光，把整个宝库照亮。金币堆成山，月光水池泛着银波，中央祭坛立着一团褪色的黑帆——巴博萨的气息贴着地面爬过来。",
        "（月光水池 / 献祭祭坛 / 白骨堆 / 交谈鬼魂 / 战拟态宝箱怪 / 巴博萨战圈 / 坍方落石区）",
    ]),
    choices: &[
        ChoiceDef { label: "月光水池", sub: "照见自身的债", cond: None, effects: &NO_EFF, route: Route::To("jb_03_pool") },
        ChoiceDef { label: "献祭祭坛", sub: "BOSS 铺垫", cond: None, effects: &NO_EFF, route: Route::To("jb_03_altar") },
        ChoiceDef { label: "白骨堆", sub: "先人的警告", cond: None, effects: &NO_EFF, route: Route::To("jb_03_bones") },
        ChoiceDef { label: "与老海盗鬼魂交谈", sub: "BOSS 铺垫", cond: None, effects: &NO_EFF, route: Route::To("jb_npc_ghost") },
        ChoiceDef { label: "战拟态宝箱怪", sub: "战斗 jb_fight_l3", cond: None, effects: &NO_EFF, route: Route::To("jb_l3_fight") },
        ChoiceDef { label: "走进巴博萨战圈", sub: "选择驱动 BOSS", cond: None, effects: &NO_EFF, route: Route::To("jb_boss_area") },
        ChoiceDef { label: "洞顶坍方落石带", sub: "需罗盘绕行", cond: None, effects: &NO_EFF, route: Route::To("jb_03_env_cavein") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 调查 / 铺垫 ---- */
SceneDef {
    id: "jb_03_pool", bg: Some("jialebi_bg.png"), loc: Some("L3 · 月光水池"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["水池映出一轮假月亮。你低头看，水面的倒影全是金币，唯一不往下沉的那枚，是你要带走的。巴博萨的债，就写在这不沉的一枚里。"]),
    choices: &[ChoiceDef { label: "捞起不沉的银币", sub: "+20 点", cond: None,
        effects: &[Eff::Points(20), Eff::MarkPoint("jb_p_l3_pool")], route: Route::To("jb_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_03_altar", bg: Some("jialebi_bg.png"), loc: Some("L3 · 献祭祭坛"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["祭坛正中立着一截烧剩的黑帆，上面用血字写巴博萨的真名。你把帆角压进怀里——传说，叫得出亡者真名的人，能在最后一击中胜他半招。"]),
    choices: &[ChoiceDef { label: "记住真名·【BOSS 铺垫】", sub: "jb_boss_primed", cond: None,
        effects: &[Eff::SetFlag("jb_boss_primed"), Eff::MarkPoint("jb_p_l3_altar")], route: Route::To("jb_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_03_bones", bg: Some("jialebi_bg.png"), loc: Some("L3 · 白骨堆"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["堆成小山的白骨里，有一副还摆着「抱金而亡」的姿势。先驱把话刻在了洞壁：<i>「洞顶的石头贪得无厌，没带罗盘，谁都别硬闯。」</i>"]),
    choices: &[ChoiceDef { label: "记下洞壁警告", sub: "San+2", cond: None,
        effects: &[Eff::San(2), Eff::MarkPoint("jb_p_l3_bones")], route: Route::To("jb_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_npc_ghost", bg: Some("jialebi_bg.png"), loc: Some("L3 · 洞窟阴处"),
    mood: "mystery", speaker: Some("老海盗鬼魂"), voice: None,
    text: TextSpec::Static(&["半透明的老海盗摊开手，掌心里浮着一柄船长的诅咒刀：「巴博萨把灵魂典给了海，谁赢他，谁就能替他'赎'这一船人。孩子，叫得出他真名，你手里才有那把刀。」"]),
    choices: &[ChoiceDef { label: "「巴博萨…我记住了。」", sub: "jb_boss_primed · San+2", cond: None,
        effects: &[Eff::San(2), Eff::SetFlag("jb_boss_primed")], route: Route::To("jb_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_l3_fight", bg: Some("jialebi_bg.png"), loc: Some("L3 · 宝库 · 战斗"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["一口宝箱在金币堆里微微震动，盖子裂开一道缝，里面不是金——是一排牙齿。拟态匣饿了。（战斗）"]),
    choices: &[], fight_id: Some("jb_fight_l3"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_03_env_cavein", bg: Some("jialebi_bg.png"), loc: Some("L3 · 坍方落石带"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["洞顶传来碎裂声，落石正伺机兜头砸下。你有罗盘的活路，没罗盘就葬在这堆宝石下面。（有罗盘 → 退避；否则活埋）"]),
    choices: &[ChoiceDef { label: "【顶石而行】", sub: "有罗盘则绕过坍方，否则活埋", cond: None,
        effects: &[Eff::San(-8)], route: Route::Dyn(zone_cavein) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 巴博萨（选择驱动 BOSS）================= */
SceneDef {
    id: "jb_boss_area", bg: Some("jialebi_bg.png"), loc: Some("L3 · 巴博萨战圈"),
    mood: "danger", speaker: Some("亡灵船长·巴博萨"), voice: None,
    text: TextSpec::Static(&["你踏进月光正中。黑帆倏地收拢成一道人形，巴博萨从影子里走脱，枯手往你肩头一拍，冰凉刺骨：「你欠海神的债，我来收。」"]),
    choices: &[ChoiceDef { label: "拔剑迎战", sub: "发起 BOSS 战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_boss_round", bg: Some("jialebi_bg.png"), loc: Some("L3 · 与巴博萨决战"),
    mood: "danger", speaker: Some("亡灵船长·巴博萨"), voice: None,
    text: TextSpec::Dyn(|st| format!(
        "{}\n巴博萨剩余 {} 血，你 HP {}。",
        if st.flag("jb_boss_primed") { "你默念那三个血写的字，刀上的圣光一晃——巴博萨第一次显得迟疑。" } else { "巴博萨的骨刀带着海水的腥气，在你面前抡圆。" },
        st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp
    )),
    choices: &[
        ChoiceDef { label: "重击", sub: "高伤害", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
        ChoiceDef { label: "防御", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ChoiceDef { label: "呼唤真名·绝杀", sub: "需已铺垫 · 大伤害", cond: Some(cond_boss_primed), effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 55, false)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_boss_win", bg: Some("jialebi_bg.png"), loc: Some("L3 · 巴博萨受死"),
    mood: "calm", speaker: Some("亡灵船长·巴博萨"), voice: None,
    text: TextSpec::Static(&["巴博萨的受诅的刀寸寸碎裂，船身的黑帆褪回海面，亡灵船员的骨影一个接一个沉进沙里。他把最后一枚血封的银币推到你面前，咧开嘴：「债，清了。」海神的长叹从洞顶吹过。"]),
    choices: &[ChoiceDef { label: "收下黑珍珠徽章", sub: "Item jb_black_pearl", cond: None,
        effects: &NO_EFF, route: Route::To("jb_ending") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 开放结局（无真相线，全导向结算卡）================= */
SceneDef {
    id: "jb_ending", bg: Some("jialebi_bg.png"), loc: Some("财宝洞 · 洞门 · 日出"),
    mood: "awe", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["你抱着战利品走出洞口，海平面正染成金红色。黑珍珠号在远处渐沉，没有真相要追，只有这一船的传说等着你亲口去讲。你想怎么续写它？"]),
    choices: &[
        ChoiceDef { label: "扬帆远去（自由）", sub: "把自由写进风里", cond: None, effects: &NO_EFF, route: Route::To("jb_settle") },
        ChoiceDef { label: "满载归来（传说）", sub: "把传说带回岸上", cond: None, effects: &NO_EFF, route: Route::To("jb_settle") },
        ChoiceDef { label: "与鬼魂同饮（羁绊）", sub: "为沉船湾的老朋友们干一杯", cond: None, effects: &NO_EFF, route: Route::To("jb_settle") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jb_settle", bg: Some("jialebi_bg.png"), loc: Some("主神·结算"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["黑珍珠的传说在你身后合拢。这一趟，要么带着财宝与自由回去，要么把故事留在那片被诅咒的海上。主神为你结算本次冒险评级。"]),
    choices: &[ChoiceDef { label: "确认走向撤离光柱", sub: "结算 · 评级", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结算卡（模板 A 段） ================= */
SceneDef {
    id: "jb_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "黑 珍 珠 · 结 算".into(), good: true,
            body_html: format!(
                "<p>你从财宝洞里走出来，把一枚不沉的银币抛向晨光——海神把债一笔勾销，传说由你定调。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 <tr><td>带走的传说</td><td style='color:#9a958a'>{}</td>\
                 </table>",
                st.points,
                if st.inventory.iter().any(|i| i == "jb_black_pearl") { "黑珍珠徽章" } else { "一船的故事" },
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案（模板 B 段） ================= */
SceneDef {
    id: "jb_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("巴博萨的债", "败给亡灵船长·巴博萨")), card: |_st| crate::state::Card {
            title: "还 不 完 的 债".into(), good: false,
            body_html: r#"<p>巴博萨的枯手按住你的心口，把最后一口气收进那枚血封银币里。黑帆重新升起——海神那桩债，换你来还。</p>
<p style='color:#ff8a8a'>【死亡档案 · 亡者之债】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "jb_51_death_fight", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("葬身黑珍珠的航程", "死在黑珍珠的冒险途中（战斗 / 坍方）")), card: |_st| crate::state::Card {
            title: "沉 在 传 说 下 面".into(), good: false,
            body_html: r#"<p>风浪、巨爪蟹或洞顶的落石，给这场冒险画上了句号。你沉进那片等着传说的海——黑珍珠驶过时，会替你多记一笔。</p>
<p style='color:#ff8a8a'>【死亡档案 · 沉没】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
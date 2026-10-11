//! 《无限恐怖 · 弗莱迪归来》全部剧情场景与战斗表。
//! 设计依据 design/zhttty_universe 梦境副本骨架 + 研究文档 wuxian_kongbu。
//! 本文件为全新新增文件，只导出静态数据（JISHUJING_SCENES / jishujing_figths / 查询辅助），
//! 不写入 scenes.rs 的静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg() 同时检索本文件。
//! 场景 id 全部 `jj2_` 前缀；fight id 全部 `jj2_` 前缀（与既有无重名）。
//!
//! 核心定位：**梦境惊悚 · 世界展示向 · 开放结局 · 无真相线**。
//! 梦魇弗莱迪（BOSS HP 210）为**选择驱动战**（黄金模板 C 段 start_boss/boss_act/boss_win），
//! 开放结局三分支：挣脱梦境 / 与弗莱迪共眠 / 把梦交给同伴，全部落向结算卡 jj2_42_card。
//! 每层 1 场前哨象征战（影子/怪物/蒸汽鬼）。失败 → 死亡档案 jj2_50_death。
//! 若梦境中照破「记忆碎片」（jj2_awake 清醒锚），决战时多一份「燃烧清醒意志」的高伤手可用。
//! bg/loc 占位待主线替换（见 jishujing_impl_log.md ★外部依赖）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
/// 已照破记忆碎片、握住「清醒的锚」
fn cond_awake(st: &GameState) -> bool { st.flag("jj2_awake") }

/* ---- 路由 fn ---- */
/// 神庙/L1 hub 通用返回
fn route_l1(_st: &mut GameState) -> String { "jj2_l1_hub".to_string() }
fn route_l2(_st: &mut GameState) -> String { "jj2_l2_hub".to_string() }
fn route_l3(_st: &mut GameState) -> String { "jj2_l3_hub".to_string() }

/// 记录玩家已瞥见弗莱迪名字（提示战后「挣脱」更稳）
fn mark_seen_name(st: &mut GameState) -> String { st.set_flag("jj2_seen_name"); "jj2_l2_hub".to_string() }
/// 照破记忆碎片 → 握住清醒之锚
fn awake_anchor(st: &mut GameState) -> String {
    if !st.flag("jj2_awake") {
        st.set_flag("jj2_awake");
        st.points += 40;
    }
    "jj2_l3_hub".to_string()
}
/// env 机关：记忆回潮（预兆 jj2_seen_name 则免被拖走，否则 San-10 · 死亡档案）
fn zone_flash(st: &mut GameState) -> String {
    if st.flag("jj2_seen_name") {
        st.san += 10;
        st.set_flag("jj2_anchor_held");
        "jj2_l3_hub".to_string()
    } else {
        "jj2_50_death".to_string()
    }
}

/* ---- 前哨象征战的 rage / finisher / win 桩 ---- */
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}
fn fins_if(_st: &GameState, _hp: i32) -> bool { false }
fn fins_name(_st: &GameState) -> String { String::new() }
fn fins_desc(_st: &GameState) -> String { String::new() }
fn win_l1(_st: &GameState) -> String { "jj2_l1_hub".to_string() }
fn win_l2(_st: &GameState) -> String { "jj2_l2_hub".to_string() }
fn win_l3(_st: &GameState) -> String { "jj2_l3_hub".to_string() }

/* =====================================================================
   战斗配置表（id 全部 jj2_ 前缀）：3 场前哨象征战 + 1 场选择驱动 BOSS 梦魇弗莱迪
   ===================================================================== */
pub fn jishujing_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("jj2_fight_l1", FightCfg {
            name: "榆树街的影子", hp: 80, dmg: (12, 18), reward: 120, reward_why: "击碎榆树街的影子",
            intro: "那影子没有形状，只是贴着你的影子一同站起，五指慢慢张开成四根钝爪。",
            rage_at: Some(40), rage_text: "影子撕开衣领，露出喉咙里一枚锈齿——它学你的声音嘀咕「别睡」。",
            on_rage: rage_none, finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_l1, death: "jj2_50_death",
        }),
        ("jj2_fight_l2", FightCfg {
            name: "困在教室的怪物", hp: 90, dmg: (14, 20), reward: 140, reward_why: "放空困局里的黑板",
            intro: "黑板上的字自己一笔一划地长出来：『你认识我的脸。』随后一个喉咙比教室还高的东西从第一排座位站起。",
            rage_at: Some(45), rage_text: "怪物把所有课桌叠成一架，朝你压来——你记得它是谁推倒的那个人。",
            on_rage: rage_none, finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_l2, death: "jj2_50_death",
        }),
        ("jj2_fight_l3", FightCfg {
            name: "锅炉蒸汽鬼影", hp: 100, dmg: (15, 22), reward: 160, reward_why: "从蒸汽里拧出的残响",
            intro: "蒸汽在锅炉前凝成一个半透明的东西，胸腔里滚着同一段睡前的哼唱。它朝你伸出手，像在讨回什么。",
            rage_at: Some(50), rage_text: "鬼影化作一片烫雾包向你的咽喉——那是弗莱迪留给锅炉房的一位旧「住客」。",
            on_rage: rage_none, finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: win_l3, death: "jj2_50_death",
        }),
        ("jj2_boss", FightCfg {
            name: "梦魇弗莱迪", hp: 210, dmg: (16, 22), reward: 500, reward_why: "击败梦魇弗莱迪 · 挣脱梦境",
            intro: "灰帽、绿色条纹、利爪手套，男人从锅炉的阴影里一步步走到你面前。他歪头一笑：『乖，把眼睛闭上——梦，给你看个好看的。』",
            rage_at: Some(80), rage_text: "弗莱迪的爪子在钢板上刮出一声长啸，整个锅炉房在震颤中亮成白昼——他要让你『醒来』在一场更深的梦。",
            on_rage: rage_none, finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: |_st| "jj2_end_flee".to_string(), death: "jj2_50_death",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn jj2_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    jishujing_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   选择驱动 BOSS（黄金模板 C 段）：start_boss / boss_act / boss_win
   ===================================================================== */
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jj2_boss") {
            st.fight = Some(crate::state::Fight {
                id: "jj2_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "jj2_boss_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 26 } else { 20 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "jj2_50_death".to_string(); }
    "jj2_boss_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500;
    st.set_flag("jj2_freddy_down");
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "jj2_end_flee".to_string()
}
/// 开放结局 2：把梦交给弗莱迪 · 与弗莱迪共眠
fn route_sleep(st: &mut GameState) -> String {
    st.set_flag("jj2_slept");
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "jj2_end_sleep".to_string()
}
/// 开放结局 3：唤醒同伴 · 把梦交给他们
fn route_share(st: &mut GameState) -> String {
    st.set_flag("jj2_shared");
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "jj2_end_share".to_string()
}
/// 结算
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "jj2_42_card".to_string()
}

/* =====================================================================
   剧情场景（id 全部 jj2_ 前缀）
   ===================================================================== */
pub static JISHUJING_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "jj2_00", bg: Some("jishujing_bg.png"), loc: Some("梦境边缘 ≪任务发布≫"),
    mood: "mystery", speaker: Some("主神·梦境入口"), voice: Some("vo_jj2_1"),
    text: TextSpec::Static(&[
        "<b>【主线任务·寻回梦境】</b>你被拖进榆树街的午后。午睡本不该是杀人的地方——可梦里有人睡得「富可敌国」，有人却再也没能醒。",
        "<i>「弗莱迪在烧锅炉。而在熟睡的人群脚下，正响起四根铁爪的刮擦声。」</i>不许睡。睡了，就轮到你的名字上墙了。",
    ]),
    choices: &[
        ChoiceDef { label: "环顾榆树街", sub: "+5 点 · 记下梦里的「清醒标志」", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("jj2_l1_hub") },
        ChoiceDef { label: "掐一下自己追证是梦", sub: "San-2 · 你确实没醒", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("jj2_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 梦境小镇 · 榆树街 hub ================= */
SceneDef {
    id: "jj2_l1_hub", bg: Some("jishujing_bg.png"), loc: Some("L1 · 榆树街"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("jj2_slept") {
            "你把自己的一部分留在了梦里，但脚还踩着榆树街的砖。街尽头的门半阖着，提醒你：梦没完。".to_string()
        } else {
            "午后的光黄得像旧胶片。街尽头那扇门锈成铁锈色，门缝里漏出锅炉的热气；孩子的口哨声在每家窗台下回响。（梦中小屋 / 街心榆树 / 街道雕像 / 梦境之门 / 孩子）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "梦中小屋", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("jj2_l1_house") },
        ChoiceDef { label: "街心榆树", sub: "爪痕钥匙线索", cond: None, effects: &NO_EFF, route: Route::To("jj2_l1_tree") },
        ChoiceDef { label: "街道雕像", sub: "梦境碑文", cond: None, effects: &NO_EFF, route: Route::To("jj2_l1_statue") },
        ChoiceDef { label: "梦境之门", sub: "需爪痕钥匙 → 学校", cond: None, effects: &NO_EFF, route: Route::To("jj2_l1_door") },
        ChoiceDef { label: "与街心的孩子交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("jj2_l1_kids") },
        ChoiceDef { label: "靠近榆树街的影子", sub: "象征战", cond: None, effects: &NO_EFF, route: Route::To("jj2_fight_l1") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点 ---- */
SceneDef {
    id: "jj2_l1_house", bg: Some("jishujing_bg.png"), loc: Some("L1 · 梦中小屋"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["屋里所有钟都停在 2:04。桌上摆着一碗没动过的麦片，椅背上搭着一件条纹睡衣——衣角的爪子印痕还新鲜。"]),
    choices: &[ChoiceDef { label: "记下小屋", sub: "伏笔 · 地图标记", cond: None,
        effects: &[Eff::MarkPoint("jj2_p_l1_house"), Eff::Points(5)], route: Route::Dyn(route_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l1_tree", bg: Some("jishujing_bg.png"), loc: Some("L1 · 街心榆树"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["街心那棵大榆树的树影里，埋着一枚四指爪印的铜钥匙。孩子说过：『那是他忘在梦里的钥匙，别让腕表型的门咬住你。』"]),
    choices: &[ChoiceDef { label: "抠出爪痕钥匙", sub: "Item jj2_key · 开 L1 梦境门", cond: None,
        effects: &[Eff::AddItem("jj2_key"), Eff::MarkPoint("jj2_p_l1_tree")], route: Route::To("jj2_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l1_statue", bg: Some("jishujing_bg.png"), loc: Some("L1 · 街道雕像"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["雕像是熟睡的孩子，脸上被刻出一排名字。最底下是你的名字——笔迹刚干，像刚有人用指尖蘸着灰写的。"]),
    choices: &[ChoiceDef { label: "读自己的名字", sub: "San-2 · 你被盯上了", cond: None,
        effects: &[Eff::San(-2), Eff::MarkPoint("jj2_p_l1_statue")], route: Route::Dyn(route_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l1_door", bg: Some("jishujing_bg.png"), loc: Some("L1 · 梦境之门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.inventory.iter().any(|i| i == "jj2_key") {
            "你拧开那扇腕表型的门，榆树街整条朝下弯去——你正跌进更深一层的梦，学校走廊的荧光灯正一排排亮起。".to_string()
        } else {
            "门锁是四指爪痕，缝隙里漏出台灯与锅炉的杂音。还缺一把钥匙——孩子说它埋在街心榆树的影子里。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "开门走进学校（L2）", sub: "需 jj2_key · → L2", cond: Some(|st| st.inventory.iter().any(|i| i == "jj2_key")),
            effects: &NO_EFF, route: Route::To("jj2_l2_hub") },
        ChoiceDef { label: "回到榆树街", sub: "", cond: None, effects: &NO_EFF, route: Route::To("jj2_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l1_kids", bg: Some("jishujing_bg.png"), loc: Some("L1 · 榆树街"),
    mood: "cold", speaker: Some("榆树街的孩子"), voice: None,
    text: TextSpec::Static(&["孩子拽着你的衣角，声音压得很低：「他睡在你隔壁那间教室。别让他知道你记得他的脸——他怕『记得』的人。」"]),
    choices: &[ChoiceDef { label: "「我记住了。」", sub: "San+2 · 伏笔", cond: None,
        effects: &[Eff::San(2), Eff::SetFlag("jj2_seen_name")], route: Route::To("jj2_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 象征战 ---- */
SceneDef {
    id: "jj2_fight_l1", bg: Some("jishujing_bg.png"), loc: Some("L1 · 榆树街暗影带"),
    mood: "danger", speaker: Some("榆树街的影子"), voice: None,
    text: TextSpec::Static(&["影子与你隔着半条街对峙，它的轮廓边缘在融化。它朝你手上爬——「别睡，别睡，别睡。」"]),
    choices: &[], fight_id: Some("jj2_fight_l1"), video: None, cine_label: None, overlay: None,
},

/* ================= L2 梦境学校 hub ================= */
SceneDef {
    id: "jj2_l2_hub", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 梦境学校"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("jj2_seen_name") {
            "你在走廊看清了墙上的告示牌——校长的名字被你随手擦出一个缺口。弗莱迪的名字依旧撬不开，但你知道他能被你『记得』。".to_string()
        } else {
            "走廊的荧光灯一半亮一半熄。守夜的老师把一教室的孩子哄睡着后，自己也靠在墙根打起了鼾。钟盘上缺一枚齿轮。（课桌 / 时钟 / 窗外 / 锅炉房铁门 / 老师）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "教室课桌", sub: "线索", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_desk") },
        ChoiceDef { label: "失控时钟", sub: "余烬线索", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_clock") },
        ChoiceDef { label: "窗外注视", sub: "弗莱迪的名字", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_window") },
        ChoiceDef { label: "锅炉房铁门", sub: "需余烬 → 锅炉房", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_furnace_door") },
        ChoiceDef { label: "与被困的老师交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_teacher") },
        ChoiceDef { label: "靠近教室的怪物", sub: "象征战", cond: None, effects: &NO_EFF, route: Route::To("jj2_fight_l2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 调查点 ---- */
SceneDef {
    id: "jj2_l2_desk", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 教室课桌"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["课桌上刻满了同一个名字，笔迹从第一行稚嫩到这几年尖利。最上面一行还新：那正是你同桌的学号。"]),
    choices: &[ChoiceDef { label: "擦去同桌的名字", sub: "伏笔 · 地图标记", cond: None,
        effects: &[Eff::MarkPoint("jj2_p_l2_desk"), Eff::Points(5)], route: Route::Dyn(route_l2) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l2_clock", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 失控时钟"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["钟盘停在 2:04，时针被什么掰断，缺的那枚齿轮躺在钟盘下的炉灰里，还亮着一粒余烬的火星。"]),
    choices: &[ChoiceDef { label: "捻起那枚余烬", sub: "Item jj2_ember · 开 L2 手印门", cond: None,
        effects: &[Eff::AddItem("jj2_ember"), Eff::MarkPoint("jj2_p_l2_clock")], route: Route::To("jj2_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l2_window", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 窗外"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["窗玻璃上印着一排外翻的手印，像有人从外面的夜晚往里扒过。玻璃最上方，一个名字被热气描出来——那不是你同桌的。"]),
    choices: &[ChoiceDef { label: "看清那个名字", sub: "SetFlag jj2_seen_name · 记忆回潮前置", cond: None,
        effects: &[Eff::MarkPoint("jj2_p_l2_window"), Eff::San(-2)], route: Route::Dyn(mark_seen_name) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l2_furnace_door", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 锅炉房铁门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.inventory.iter().any(|i| i == "jj2_ember") {
            "你把余烬按进锁孔，手印门上的指针一转，整座学校像电梯一样下沉。你还未落地，锅炉的热浪已扑面而来。".to_string()
        } else {
            "门上是一只校准到 2:04 的手印。钟盘缺的那枚齿轮就是钥匙——它躺在某个没关的炉灰里。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "用余烬开锁（L3）", sub: "需 jj2_ember · → L3", cond: Some(|st| st.inventory.iter().any(|i| i == "jj2_ember")),
            effects: &NO_EFF, route: Route::To("jj2_l3_hub") },
        ChoiceDef { label: "回到走廊", sub: "", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l2_teacher", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 教师办公室"),
    mood: "cold", speaker: Some("被困的老师"), voice: None,
    text: TextSpec::Static(&["老师抱着一摞名册，声音发抖：「他烧锅炉那年，就把名字一件件收进了炉里。可我记得他——他被『记得』的时候，会害怕。」"]),
    choices: &[ChoiceDef { label: "「我也记得。」", sub: "San+2 · 弗莱迪的破绽", cond: None,
        effects: &[Eff::San(2), Eff::SetFlag("jj2_seen_name")], route: Route::To("jj2_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 象征战 ---- */
SceneDef {
    id: "jj2_fight_l2", bg: Some("jishujing_bg_highschool.png"), loc: Some("L2 · 教室"),
    mood: "danger", speaker: Some("困在教室的怪物"), voice: None,
    text: TextSpec::Static(&["那些课桌一起朝前滑了半步，露出一张熟悉的、属于某个同学的脸。它在等你把它「记得」起来，然后放它睡。"]),
    choices: &[], fight_id: Some("jj2_fight_l2"), video: None, cine_label: None, overlay: None,
},

/* ================= L3 意识深处 · 弗莱迪的锅炉房 hub ================= */
SceneDef {
    id: "jj2_l3_hub", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 弗莱迪的锅炉房"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("jj2_awake") {
            "锅炉的膛火被你眼底的清醒压得晃了晃。你身上那杆「清醒的锚」，让每一张睡梦里的脸都朝你偏了偏。".to_string()
        } else {
            "锅炉的膛火烧得正旺，蒸汽里隐约有人哼歌。熔炉本体 / 记忆碎片 / 裂镜 / 锈墙刻痕 / 梦中女孩残影，都在等你把「醒来」的路认全。（面对锅炉核心 → 决战）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "熔炉本体", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("jj2_l3_boiler") },
        ChoiceDef { label: "记忆碎片", sub: "清醒之锚前置", cond: None, effects: &NO_EFF, route: Route::To("jj2_l3_memory") },
        ChoiceDef { label: "裂镜", sub: "镜内的自己", cond: None, effects: &NO_EFF, route: Route::To("jj2_l3_mirror") },
        ChoiceDef { label: "锈墙刻痕", sub: "刻痕里的名字", cond: None, effects: &NO_EFF, route: Route::To("jj2_l3_wall") },
        ChoiceDef { label: "与梦中女孩残影交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("jj2_l3_ghost") },
        ChoiceDef { label: "靠近锅炉核心", sub: "决战 · 梦魇弗莱迪", cond: None, effects: &NO_EFF, route: Route::To("jj2_boss") },
        ChoiceDef { label: "回望学校走廊", sub: "返回 L2", cond: None, effects: &NO_EFF, route: Route::To("jj2_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 调查点 ---- */
SceneDef {
    id: "jj2_l3_boiler", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 熔炉本体"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["熔炉的舌头舔过一摞试卷，把名字一张张化成灰。你听见炉膛深处，一阵压抑到极点的、属于成群熟睡者的呼吸。"]),
    choices: &[ChoiceDef { label: "伸手探炉", sub: "San-3 · 你摸到了灰里的一个名字", cond: None,
        effects: &[Eff::San(-3), Eff::MarkPoint("jj2_p_l3_boiler")], route: Route::Dyn(route_l3) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l3_memory", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 记忆碎片"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["蒸汽的涡里飘着一片玻璃，里面是你醒着时的卧室。握着它，你有一瞬确信：外面还有人等你，你不是光躺在这炉边的名字。"]),
    choices: &[ChoiceDef { label: "握住记忆碎片", sub: "SetFlag jj2_awake · 清醒之锚 +40点", cond: None,
        effects: &NO_EFF, route: Route::Dyn(awake_anchor) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l3_mirror", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 裂镜"),
    mood: "danger", speaker: Some("裂镜里的自己"), voice: None,
    text: TextSpec::Static(&["裂镜里你的脸有一瞬变成弗莱迪的脸，又拼回原样。镜中人开口：「别怕他——他怕的是『记得』他的人。」"]),
    choices: &[ChoiceDef { label: "与裂镜里的自己击掌", sub: "SetFlag jj2_seen_name", cond: None,
        effects: &[Eff::MarkPoint("jj2_p_l3_mirror")], route: Route::Dyn(mark_seen_name) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l3_wall", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 锈墙刻痕"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["锈墙上刻着一排排划正的『正』字，最顶上一行被划到一半。下面是弗莱迪的名字，笔画被他自己的爪痕涂蓝了三分。"]),
    choices: &[ChoiceDef { label: "把弗莱迪的名字刻完", sub: "伏笔 · 地图标记", cond: None,
        effects: &[Eff::MarkPoint("jj2_p_l3_wall"), Eff::Points(5)], route: Route::Dyn(route_l3) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_l3_ghost", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 炉边"),
    mood: "cold", speaker: Some("梦中女孩残影"), voice: None,
    text: TextSpec::Static(&["残影贴得很近，声音像从很远的水底传来：「别再往炉里添我的名字。你记得我，我就走得掉。你也能——只要你在锅炉烧起来以前醒来。」"]),
    choices: &[ChoiceDef { label: "「我会记得你。」", sub: "San+2 · 弗莱迪的破绽 +", cond: None,
        effects: &[Eff::San(2), Eff::SetFlag("jj2_seen_name")], route: Route::To("jj2_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 env 机关：记忆回潮 ---- */
SceneDef {
    id: "jj2_l3_flash", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 记忆回潮区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["蒸汽忽然拧成你刚进副本那段回忆，往咽喉里灌。你若没认出「弗莱迪的名字」，回忆会把你原来的睡意拖回炉边。"]),
    choices: &[ChoiceDef { label: "【沉入回潮】", sub: "已见名字则+San免被拖走，否则San-10 · 死亡档案", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_flash) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 象征战 ---- */
SceneDef {
    id: "jj2_fight_l3", bg: Some("jishujing_bg_boiler.png"), loc: Some("L3 · 蒸汽锅炉侧"),
    mood: "danger", speaker: Some("锅炉蒸汽鬼影"), voice: None,
    text: TextSpec::Static(&["蒸汽凝成半透明的东西，朝你伸出一只手。它的胸腔里滚着一段睡前的哼唱——它是被弗莱迪收进炉里的一位「住客」。"]),
    choices: &[], fight_id: Some("jj2_fight_l3"), video: None, cine_label: None, overlay: None,
},

/* ================= BOSS · 梦魇弗莱迪 ================= */
SceneDef {
    id: "jj2_boss", bg: Some("jishujing_bg.png"), loc: Some("L3 · 锅炉核心 · 决战处"),
    mood: "danger", speaker: Some("梦魇弗莱迪"), voice: None,
    text: TextSpec::Static(&["灰帽绿纹的男人从锅炉阴影里踱出，四根铁爪蹭着钢板。他朝你勾了勾手：「梦做一半最香，你偏要吵醒它——那就把你也收进炉里。」"]),
    choices: &[
        ChoiceDef { label: "【紧握清醒 · 迎战】", sub: "开始选择驱动战 · HP210", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
        ChoiceDef { label: "【垂下眼睛】", sub: "把梦交给弗莱迪 · 与他共眠（开放结局2）", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_sleep) },
        ChoiceDef { label: "【回头唤醒同伴】", sub: "把梦交给同伴 · 由他们做梦（开放结局3）", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_share) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_boss_round", bg: Some("jishujing_bg.png"), loc: Some("L3 · 锅炉核心 · 决战"),
    mood: "danger", speaker: Some("梦魇弗莱迪"), voice: None,
    text: TextSpec::Dyn(|st| format!("弗莱迪剩余 {} 血，你 HP {}。铁爪在钢板上拉出四道火星。<i>（你瞥见炉膛里熟睡着的人群——有谁能替你记得弗莱迪的脸？）</i>", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "重击", sub: "高伤 扣30", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
        ChoiceDef { label: "防御", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ChoiceDef { label: "燃烧清醒意志", sub: "若握有清醒锚 · 高伤扣45", cond: Some(cond_awake), effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 45, false)) },
        ChoiceDef { label: "放下刀 · 把梦交给弗莱迪", sub: "与他共眠（开放结局2）", cond: None, effects: &NO_EFF, route: Route::Dyn(route_sleep) },
        ChoiceDef { label: "大声唤醒同伴", sub: "把梦交给同伴（开放结局3）", cond: None, effects: &NO_EFF, route: Route::Dyn(route_share) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 开放结局 · 挣脱梦境（击败弗莱迪）================= */
SceneDef {
    id: "jj2_end_flee", bg: Some("jishujing_bg.png"), loc: Some("契约边界 · 醒来"),
    mood: "calm", speaker: Some("主神·结算"), voice: None,
    text: TextSpec::Static(&["锅炉在一记重击里熄了火。弗莱迪的灰帽滚进炉膛，人是醒了——你攥紧记忆碎片，把这句话说给炉膛里所有人听：「你不记得他，他就忘了你。」你睁开眼，枕头边还留着那枚爪痕钥匙的凉意。"]),
    choices: &[ChoiceDef { label: "确认结算", sub: "开放结局1 · 挣脱梦境", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_end_sleep", bg: Some("jishujing_bg.png"), loc: Some("梦境最深 · 与他共眠"),
    mood: "choice", speaker: Some("梦魇弗莱迪"), voice: None,
    text: TextSpec::Static(&["你放下刀，把眼睛阖上。弗莱迪的爪尖轻轻合拢你的手，哼起那首没唱完的摇篮曲：「乖，梦外面不是真……这儿才永远有人记得你。」——不知道这一觉，你还会不会醒。"]),
    choices: &[ChoiceDef { label: "把这一梦收进结算", sub: "开放结局2 · 与弗莱迪共眠", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jj2_end_share", bg: Some("jishujing_bg.png"), loc: Some("现实边缘 · 把梦交给同伴"),
    mood: "choice", speaker: Some("同伴·梦境接管"), voice: None,
    text: TextSpec::Static(&["你回头把沉睡的同伴一个个拍醒：「替我记住弗莱迪的脸——这张脸，得由你们来记得。」梦里的锅炉一条条熄下去。你卸下了「记得」的重担，他们却接过了它。你醒来时，肩头轻得发空。"]),
    choices: &[ChoiceDef { label: "把梦交给同伴 · 结算", sub: "开放结局3 · 由同伴接手", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结算卡（开放结局共用）================= */
SceneDef {
    id: "jj2_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "梦 醒 之 后".into(), good: true,
            body_html: format!(
                "<p>{}</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>清醒之锚</td><td style='color:#9a958a'>{}</td></tr>\
                 <tr><td>睁开结局</td><td style='color:#ffd76a'>{}</td></tr>\
                 </table>",
                ending_text(st),
                st.points,
                if st.flag("jj2_awake") { "✓ 握住记忆碎片，你证明了自己仍醒着" } else { "✗ 你始终没分清真梦" },
                ending_label(st),
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案 ================= */
SceneDef {
    id: "jj2_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("弗莱迪 · 炉边长眠", "你没能识别梦境中最凶的一觉，被弗莱迪连同名字一起收进了锅炉")), card: |_st| crate::state::Card {
            title: "炉 边 长 眠".into(), good: false,
            body_html: r#"<p>铁爪合拢的一瞬，你听见所有人同时睡去的呼吸。你的名字被划进锈墙最底下——「记得」你的人，从此只剩下你醒来以前的那一段。</p>
<p style='color:#ff8a8a'>【死亡档案 · 梦魇回收】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* ---- 结算卡辅助 ---- */
fn ending_text(st: &GameState) -> String {
    if st.flag("jj2_slept") { "你与弗莱迪共眠在一场不必醒的午后，把梦的钥匙交给了他".to_string() }
    else if st.flag("jj2_shared") { "你把心头最沉的那个梦分给同伴，让他们替你去记得弗莱迪的脸".to_string() }
    else if st.flag("jj2_freddy_down") { "锅炉在你手中熄火，你挣脱了弗莱迪的梦境，带着一身冷汗与爪痕钥匙醒来".to_string() }
    else { "你在梦境边缘睁开了眼，梦魇的爪痕淡成枕边一道凉意".to_string() }
}
fn ending_label(st: &GameState) -> String {
    if st.flag("jj2_slept") { "与弗莱迪共眠".to_string() }
    else if st.flag("jj2_shared") { "把梦交给同伴".to_string() }
    else { "挣脱梦境".to_string() }
}
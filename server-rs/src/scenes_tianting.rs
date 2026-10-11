//! 《洪荒天庭》高难任务世界 · 全部剧情场景与战斗配置。
//! 世界观依据 design/zhttty_universe/honghuang_li/00_honghuang_li_research.md（洪荒天庭 / 封神 / 东天二皇 / 伏羲·四象五行八卦阵）。
//! 本文件是全新新增文件，只导出静态数据（TIANTING_SCENES / tianting_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/tianting_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `tt_` 前缀，与既有 SCENES 无重名。
//! 战斗 id 全部 `tf_` 前缀（BOSS 天庭神将·封神投影两段式 tf_shenjiang_r1/r2）。
//!
//! 多段 BOSS（天庭神将·封神投影）：一段战胜利 → 转场演出（东天二皇投影）→ 放二段战斗场景，用"场景链"衔接，
//! 无 next_fight 引擎字段（设计取舍，参考 scenes_yinse.rs 的 BOSS 场景链写法）。
//! **圣位演出红线（铁律）**：东天二皇（帝俊天皇 / 太一东皇）投影只做 OverlayDef/video 文本演出，绝不进 fight 数值；
//! 本文件东天二皇不建任何可战 Fight（design §10.4：圣位及皇级不可进 FIGHTS 数值）。
//! sp_grade 用 Route::Dyn 写 `st.sp_grade = Some('A')`（高难副本，结局统一 A 级）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes_yinse.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   条件谓词（cond，全部具名：CondFn 为 fn 指针不能捕获闭包）
   ===================================================================== */
fn cond_has_p1(st: &GameState) -> bool { st.flag("tt_fengshen_p1") }
fn cond_has_p3(st: &GameState) -> bool { st.flag("tt_fengshen_p3") }
fn cond_has_truth(st: &GameState) -> bool { st.flag("tt_fengshen_truth") }
fn cond_lingxiao_open(st: &GameState) -> bool { st.flag("tt_lingxiao_open") }

/* =====================================================================
   动态文本辅助
   ===================================================================== */
/// 天条断碑：倒悬的王座 —— 主题钩子 + 封神榜残页·一
fn txt_stele(_st: &GameState) -> String {
    "断碑上，天条倒着刻。被倒悬的王座镇压的人，读不出正确的句子。\n\n<b>「这里的天，是倒悬的王座。凡升者，必坠。」</b>（San -4）".to_string()
}

/// 神将一形态（HP 260）
fn txt_boss_r1(_st: &GameState) -> String {
    "封神台上的投影自封神榜的狂化余烬中凝聚——神将金甲，面庞被圣光蚀刻成一片空白，腰间封神榜残页翻卷。\n\n<b>「镇压——凡升者，必坠。」他复读着注入他躯壳的圣裁。</b>".to_string()
}
/// 神将二形态（封神投影·真灵）
fn txt_boss_r2(_st: &GameState) -> String {
    "投影的甲胄从内部炸开，卷入封神榜残页的笔墨。狂化的万族虚影在他身后凝成一件人形叛军——只剩那枚封神章印，替他睁着一只空洞的『天眼』。\n\n<b>「……摘不下来的章，就是我的脸。」</b>".to_string()
}
fn txt_round_r1(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(260);
    let interrupted = st.flag("tt_r1_interrupted");
    let head = if interrupted {
        "人皇封条嵌进封神榜残页，投影的蓄力被掐灭——它每回合的『圣裁回响』不再触发。"
    } else {
        "神将投影一次次把封神章印按进大地——蓄力越深，下一次『圣裁回响』越重。<em>若你洞悉封神真相，可在蓄力轮注入人皇封条打断。</em>"
    };
    format!("<b>天庭神将 · 封神投影（一形态）</b>　HP {hp}/260\n\n{head}")
}
fn txt_round_r2(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(150);
    let head = "二形态的狂化虚影卷着残页在倒悬骑士四周翻腾——每轮都对全队发起『封神狂化』（San -3/击）。";
    format!("<b>天庭神将 · 封神投影（二形态·真灵）</b>　HP {hp}/150\n\n{head}")
}

/* =====================================================================
   战斗配置表（TF 专属；导出供主线把查询扩展进来）
   ===================================================================== */
fn tt_rage_common(_st: &mut GameState, log: &mut Vec<String>) {
    log.push("<span class='crit'>圣裁的虚影在他周身炸开——封神榜的狂化余烬再度抬升。</span>".into());
}

/* ---- 胜利路由（两段式 BOSS 场景链衔接） ---- */
/// 一段 BOSS 胜利路由：进入东天二皇投影演出（放二段）
fn tt_win_r1(st: &GameState) -> String {
    let _ = st;
    "tt_huang_cast".into()
}
/// 二段 BOSS 胜利路由：进入结局抉择
fn tt_win_r2(st: &GameState) -> String {
    let _ = st;
    "tt_17_choice".into()
}
/// 杂兵战斗胜利（回上一层调查点）
fn tt_win_common(_st: &GameState) -> String { "tt_02_gate".into() }

fn rnd_tt(a: i32, b: i32) -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(a..=b)
}

/// 战斗配置表（id 全部 tf_ 前缀；BOSS 两段式，参照 scenes_yinse.rs 的 ws_waro_r1/r2 写法）。
pub fn tianting_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("tf_tianbing", FightCfg {
            name: "天兵残魂", hp: 60, dmg: (10, 16), reward: 20, reward_why: "天兵残魂 · 清剿",
            intro: "一道披着残旧天甲的身影自云端坠下——面甲下没有脸，只有封神榜的印记在眼底反复烧写：「镇压」。它拔剑，剑身锈了一半。",
            rage_at: Some(25), rage_text: "<b>狂暴@25</b>：天条缚锁，下一击命中带 San -2。", on_rage: tt_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        ("tf_wandu", FightCfg {
            name: "狂化万族残灵", hp: 48, dmg: (9, 15), reward: 16, reward_why: "狂化万族残灵 · 清理",
            intro: "封神榜的余烬在一具万族枯骨上重燃——它双目赤红，狂化嘶吼着朝你扑来。它生前也许是个普通的求存者。",
            rage_at: Some(20), rage_text: "<b>狂暴@20</b>：狂化自噬（San -2）。", on_rage: |st, log| {
                st.san = (st.san - 2).clamp(0, 100);
                log.push("<span class='crit'>狂化残灵自噬，一丝理智被烧成余烬（San -2）。</span>".into());
            },
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        ("tf_shenjiang_mini", FightCfg {
            name: "神将禁军投影", hp: 120, dmg: (14, 22), reward: 90, reward_why: "神将禁军投影 · 破阵",
            intro: "一尊低阶神将投影拄着断戟立在兵冢上——他是封神战里被抹去名字的那一类。看见你，他咧嘴：「又一个要被钉进榜里的名字。」",
            rage_at: Some(55), rage_text: "<b>狂暴@55</b>：封神一章，命中携带 San -4。", on_rage: tt_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        ("tf_lingshe", FightCfg {
            name: "苍天灵蛇残魂", hp: 66, dmg: (12, 18), reward: 40, reward_why: "苍天灵蛇残魂 · 斩除",
            intro: "倒悬的云海间垂落一条苍色灵蛇的残魂——它本是万族的『升维希望』，如今被东天二皇的圣裁钉死在神桥之上，喉间还卡着一句没说完的话。",
            rage_at: Some(30), rage_text: "<b>狂暴@30</b>：残躯回光（San -3）。", on_rage: tt_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        ("tf_kuang", FightCfg {
            name: "封神榜·狂化禁军", hp: 110, dmg: (14, 22), reward: 80, reward_why: "封神榜·狂化禁军 · 破阵",
            intro: "封神台四周，被狂化的禁军团团围上来——他们曾替你这类『升者』开路，如今反噬为刀。封神榜把万族的气数，喂成了杀机。",
            rage_at: Some(50), rage_text: "<b>狂暴@50</b>：狂化增员，每轮 San -2。", on_rage: tt_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        ("tf_sanqing", FightCfg {
            name: "三清归位·残像", hp: 150, dmg: (16, 26), reward: 160, reward_why: "三清归位·残像 · 镇压",
            intro: "封神台深处，三清归位的残像自弥罗天网阵的碎片中走出——李二太清、李四上清的影子合一又散开。它们身后，是三十三天被撕开的边缘。",
            rage_at: Some(70), rage_text: "<b>狂暴@70</b>：弥罗天网倒卷，San -6。", on_rage: |st, log| {
                st.san = (st.san - 6).clamp(0, 100);
                log.push("<span class='crit'>弥罗天网阵的碎片倒卷——你的理智被天条刮去一线（San -6）。</span>".into());
            },
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        ("tf_weiwang", FightCfg {
            name: "凌霄禁军残魂", hp: 130, dmg: (15, 24), reward: 120, reward_why: "凌霄禁军残魂 · 清剿",
            intro: "凌霄殿断柱之间，禁军的残魂列阵而立——它们直到最后一刻，都在执行『镇压』的圣谕。看见你，它们整齐地拔刀：「受命者，镇。」",
            rage_at: Some(60), rage_text: "<b>狂暴@60</b>：帝威压顶，San -4。", on_rage: tt_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tt_win_common, death: "tt_lose_common",
        }),
        /* ---- BOSS 两段式（天庭神将·封神投影） ---- */
        ("tf_shenjiang_r1", FightCfg {
            name: "天庭神将 · 封神投影（一形态）", hp: 260, dmg: (20, 30), reward: 600, reward_why: "封神投影 · 一段肃清",
            intro: "封神台的狂化余烬聚拢成一尊金甲神将——它是被东天二皇凌驾之名注入的『封神投影』，腰间封神榜残页翻卷，圣裁的虚影在周身炸响。",
            rage_at: Some(120), rage_text: "<b>圣裁回响</b>：封神章印按入大地，每轮追加一次「圣裁回响」（全队 dmg 6~10 无视护甲）。",
            on_rage: |st, log| {
                st.hp = (st.hp - rnd_tt(6, 10)).max(0);
                log.push("<span class='crit'>「凡升者，必坠。」封神章印炸开——圣裁回响穿透护甲（dmg 6~10）。</span>".into());
            },
            finisher_if: |_st, ehp| ehp <= 50,
            finisher_name: |st| { if st.flag("tt_r1_interrupted") { "钉入人皇封条 · 打断".into() } else { "封神一章 · 降临".into() } },
            finisher_desc: |st| {
                if st.flag("tt_r1_interrupted") {
                    "你在投影蓄力至最高点前，把三张封神榜残页化作的人皇封条一把按下。圣裁仿佛被掐灭的引线，「嗡」地一滞——那声『凡升者，必坠』被生生咽回。".into()
                } else {
                    "他全身的圣裁虚影同时亮到最刺目的一刻——『封神一章 · 降临』！一线金光将你与整片封神台钉在原地（dmg 26×2 + San -10）。".into()
                }
            },
            win: tt_win_r1, death: "tt_lose_r1",
        }),
        ("tf_shenjiang_r2", FightCfg {
            name: "天庭神将 · 封神投影（二形态·真灵）", hp: 150, dmg: (24, 36), reward: 800, reward_why: "封神投影 · 两段平定",
            intro: "投影的甲胄被东天二皇的投影演出彻底击碎之后，封神榜残页裹缠而上，卷成一尊狂化的『叛军』。那枚封神章印替他睁着一只空洞的天眼——真正的敌人，是被注进兵甲的『封神之刑』本身。",
            rage_at: Some(70), rage_text: "<b>封神狂化</b>：狂化万族虚影卷入，每轮 San -3/击；用它为你自己量刑。",
            on_rage: |st, log| {
                st.san = (st.san - 3).clamp(0, 100);
                log.push("<span class='crit'>狂化的万族虚影卷入战斗——封神章印替你量着『该不该升』的罪（San -3）。</span>".into());
            },
            finisher_if: |_, _| true,
            finisher_name: |st| {
                if st.flag("tt_fengshen_truth") { "以真相断章 · 宽宥".into() }
                else if st.flag("tt_r1_interrupted") { "人皇封条余威 · 镇压".into() }
                else { "强杀 · 断章".into() }
            },
            finisher_desc: |st| {
                if st.flag("tt_fengshen_truth") {
                    "你把洞悉的封神真相砸进章印——「镇压来自东天二皇，不是天道」。狂化虚影一滞，章印剧烈熔解。".to_string()
                } else if st.flag("tt_r1_interrupted") {
                    "人皇封条的余威在你掌心烫了一下，投影最后的蓄力被你顶了回去。".to_string()
                } else {
                    "你咬牙将最后一击贯入那枚空洞的天眼。封神章印整片爆开，落了满地发烫的金屑。".to_string()
                }
            },
            win: tt_win_r2, death: "tt_lose_r2",
        }),
    ]
}

/* =====================================================================
   剧情场景（id 全部 tt_ 前缀）
   ===================================================================== */
pub static TIANTING_SCENES: &[SceneDef] = &[

/* ---- 幕 0 ・ 坠落：南天门残垣（L1） ---- */
SceneDef {
    id: "tt_01_drop", bg: Some("tianting_bg.png"), loc: Some("主神广场 · 解锁·洪荒天庭"),
    mood: "danger", speaker: Some("李铭（记录员）"), voice: Some("vo_tianting_liming_start"),
    text: TextSpec::Static(&[
        "<b>【高难副本·主线任务】</b>深入被封印的「洪荒天庭残境」，调查封神战的真相，在镇压与被镇压之间求生。",
        "「这里的天，是倒悬的王座。」李铭的声音冷得像天条。「东天二皇堵死了所有『升者』的路——可被堵死的那条路，自己还在流血。」",
        "「档案编号零零三——洪荒天庭 · 封神战场。三十三重天碎了，可王座没有。」",
    ]),
    choices: &[
        ChoiceDef { label: "【接受任务】", sub: "接受 → 坠落南天门", cond: None,
            effects: &[Eff::SetFlag("tt_mission"), Eff::Points(0)], route: Route::To("tt_01_drop_land") },
        ChoiceDef { label: "【问：封神榜是什么？】", sub: "旁白给设定 · San-2", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("tt_01_fengshen_ask") },
        ChoiceDef { label: "【拒绝接受】", sub: "李铭：高难任务不可拒绝 · San-4", cond: None,
            effects: &[Eff::San(-4)], route: Route::To("tt_01_refuse") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_01_drop_land", bg: Some("tianting_bg.png"), loc: Some("L1 南天门残垣 · 坠落点"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "脚下是碎裂的天庭砖瓦，倒悬在云海之上——你踩着的，是三十三天里被撕下来的某一片。",
        "远处，一扇半塌的残门横亘天边，门楣上「南天门」三字被从中劈成两半。门里的天空，是倒着的。",
        "（风里有天条的铁锈味，和封神榜煮沸的纸墨气。）",
    ]),
    choices: &[
        ChoiceDef { label: "【上前查看天条断碑】", sub: "主题钩子 · 封神榜残页一情报", cond: None, effects: &NO_EFF, route: Route::To("tt_03_stele") },
        ChoiceDef { label: "【走向南天门残门】", sub: "G1", cond: Some(cond_has_p1), effects: &NO_EFF, route: Route::To("tt_02_gate") },
        ChoiceDef { label: "【查看天庭兵冢】", sub: "遭遇神将禁军 · 情报", cond: None, effects: &NO_EFF, route: Route::To("tt_04_ruins") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_01_fengshen_ask", bg: Some("tianting_bg.png"), loc: Some("主神广场 · 任务门"),
    mood: "cold", speaker: Some("李铭（记录员）"), voice: None,
    text: TextSpec::Static(&[
        "「封神榜——蛇以三界胎膜布局、万族签押的刑名之书。」李铭顿了顿，「被它钉住的族，气数会被抽去喂『天』。」",
        "「伏羲曾以四象五行八卦阵反封天庭……可在你们之前，没人走到过那一步。」",
    ]),
    choices: &[ChoiceDef { label: "（回到任务门）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_01_drop") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_01_refuse", bg: Some("tianting_bg.png"), loc: Some("主神广场 · 任务门"),
    mood: "danger", speaker: Some("李铭（记录员）"), voice: Some("vo_tianting_liming_refuse"),
    text: TextSpec::Static(&[
        "「高难修正不可拒绝。」李铭的声音不带温度。「倒悬的王座压了几千年——这一次，轮到你们去把天空翻回来。」",
    ]),
    choices: &[ChoiceDef { label: "（被强制弹入坠落）", sub: "强制接受", cond: None, effects: &[Eff::Points(0)], route: Route::To("tt_01_drop_land") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 天条断碑（封神榜残页·一） */
SceneDef {
    id: "tt_03_stele", bg: Some("tianting_bg.png"), loc: Some("L1 · 天条断碑"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(txt_stele),
    choices: &[
        ChoiceDef { label: "【揭下封神榜残页·一】", sub: "残页一 · 天条线索 · San-4", cond: None,
            effects: &[Eff::SetFlag("tt_fengshen_p1"), Eff::San(-4), Eff::PointsIfFlag("tt_fengshen_p1", 100)], route: Route::To("tt_02_gate") },
        ChoiceDef { label: "（后退，先看兵冢）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_04_ruins") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 南天门残门（G1） */
SceneDef {
    id: "tt_02_gate", bg: Some("tianting_bg.png"), loc: Some("L1 南天门残门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残门被一道黯淡的天条封着。门缝里漏出神桥的风——冷，且带着天条煮沸纸墨的腥气。",
        "你手中的封神榜残页·一微微发烫，与封印泛起同源的光。",
    ]),
    choices: &[
        ChoiceDef { label: "【穿越南天门（G1）】", sub: "需残页一 → L2 神桥", cond: Some(cond_has_p1),
            effects: &NO_EFF, route: Route::To("tt_05_bridge") },
        ChoiceDef { label: "【回兵冢搜刮】", sub: "遭遇 · 神将禁军", cond: None, effects: &NO_EFF, route: Route::To("tt_04_ruins") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 天庭兵冢（遭遇 + 神将禁军情报） */
SceneDef {
    id: "tt_04_ruins", bg: Some("tianting_bg.png"), loc: Some("L1 天庭兵冢"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "兵冢上插满断裂的天戟，戟尖朝下——仿佛每一把戟，都在钉住某个『升上去的人』。",
        "一尊低阶神将投影拄着断戟立在正中，看见你，咧嘴：「又一个要被钉进榜里的名字。」",
        "（兵冢情报 + 神将禁军遭遇。）",
    ]),
    choices: &[
        ChoiceDef { label: "【迎战神将禁军】", sub: "tf_shenjiang_mini", cond: None,
            effects: &[Eff::Points(20)], route: Route::To("tt_04_win") },
        ChoiceDef { label: "（绕行，先回残门）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_02_gate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_04_win", bg: Some("tianting_bg.png"), loc: Some("L1 天庭兵冢 · 清理"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["断戟坠地。你在戟坑里翻出一角快烂掉的天条残片，上面用血写着半句话：『……被镇压的，从来不是最弱的。』"]),
    choices: &[ChoiceDef { label: "（返回南天门残门）", sub: "G1", cond: None,
        effects: &[Eff::SetFlag("tt_ruins_cleared"), Eff::PointsIfFlag("tt_fengshen_p1", 50)], route: Route::To("tt_02_gate") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 1 ・ 神桥（L2）：录事官残魂 / 星宿残阵 ---- */
SceneDef {
    id: "tt_05_bridge", bg: Some("tianting_bg.png"), loc: Some("L2 天庭神桥 · 断裂处"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "神桥从南天门断崖探进云海，走到一半轰然断裂。桥下的倒悬云海深处，有星星点点的『光』在反向坠落。",
        "一处断柱后，蜷着一道披着旧文吏袍的残魂——他抬头，空洞的眼眶里竟有一颗还亮着的泪珠。",
    ]),
    choices: &[
        ChoiceDef { label: "【与录事官残魂对话】", sub: "NPC · 封神真相", cond: None, effects: &NO_EFF, route: Route::To("tt_05_bridge_lushi") },
        ChoiceDef { label: "【去云海断碑】", sub: "封神榜残页二", cond: None, effects: &NO_EFF, route: Route::To("tt_06_cloud_stele") },
        ChoiceDef { label: "【去星宿残阵】", sub: "puzzle · 点亮阵眼", cond: None, effects: &NO_EFF, route: Route::To("tt_07_stars") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_05_bridge_lushi", bg: Some("tianting_bg.png"), loc: Some("L2 神桥 · 录事官残魂"),
    mood: "cold", speaker: Some("录事官残魂 · 敬"), voice: Some("vo_tianting_lushi"),
    text: TextSpec::Static(&[
        "「别怕，我……」残魂的声音飘忽。「我是录事官敬。几千年前，我负责替东天二皇誊抄『镇压』的名单。」",
        "「我誊了一辈子别人的名。直到有一年，我看见自己的名字也被写进封神榜——我才明白，我们这些『天兵天将』，也不过是随时能被典掉的气数。」",
        "「云海断碑下，藏着第二张封神榜残页。它记的，不是罪，是……伏羲布阵前，那四象五行的一角。」",
    ]),
    choices: &[
        ChoiceDef { label: "【谢过，前往云海断碑】", sub: "残页二线索 · 支线敬谊 +200", cond: None,
            effects: &[Eff::SetFlag("tt_lushi_friend"), Eff::PointsIfFlag("tt_lushi_friend", 200)], route: Route::To("tt_06_cloud_stele") },
        ChoiceDef { label: "（先去星宿残阵）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_07_stars") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 云海断碑（残页二） */
SceneDef {
    id: "tt_06_cloud_stele", bg: Some("tianting_bg.png"), loc: Some("L2 云海断碑"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "云海断碑上，刻着一副残缺的四象阵图。青龙、白虎、朱雀的纹路被圣裁刮花，只剩玄武一角还泛着幽蓝的光。",
        "碑脚下压着第二张封神榜残页——墨迹正写到『东天二皇』四字之前，被一道天雷劈断。",
    ]),
    choices: &[
        ChoiceDef { label: "【取走封神榜残页·二】", sub: "残页二 · 四象阵图 · San-3", cond: None,
            effects: &[Eff::SetFlag("tt_fengshen_p2"), Eff::San(-3), Eff::PointsIfFlag("tt_fengshen_p2", 100)], route: Route::To("tt_05_bridge") },
        ChoiceDef { label: "（赴星宿残阵）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_07_stars") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 星宿残阵（puzzle） */
SceneDef {
    id: "tt_07_stars", bg: Some("tianting_bg.png"), loc: Some("L2 星宿残阵"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "二十八宿残碑散落在石阵里，多数倒伏，只有几颗还倔强地亮着。碑上用最古老的天文笔迹刻着一句：『星宿归位，方见天路。』",
        "（机关：按序点亮失去光泽的星宿，可开启残阵深处的捷径与回程断梯。）",
    ]),
    choices: &[
        ChoiceDef { label: "【重排二十八宿残碑】", sub: "puzzle · 点亮 tt_stars_lit", cond: None,
            effects: &[Eff::SetFlag("tt_stars_lit"), Eff::Points(50)], route: Route::To("tt_07_stars_done") },
        ChoiceDef { label: "（回神桥对话录事官）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_05_bridge") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_07_stars_done", bg: Some("tianting_bg.png"), loc: Some("L2 星宿残阵 · 归位"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["你按四象阵图的残页依次扶正残碑。二十八宿次第亮起，在倒悬的云海间拼出一座短暂完整的星空——像旧天庭还活着的那一刻。"]),
    choices: &[ChoiceDef { label: "（步向封神台方向 L3）", sub: "pt_down2", cond: None, effects: &NO_EFF, route: Route::To("tt_08_fengshen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 2 ・ 封神台（L3）：真相 / BOSS 战场 ---- */
SceneDef {
    id: "tt_08_fengshen", bg: Some("tianting_bg.png"), loc: Some("L3 封神台 · 入口"),
    mood: "danger", speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "跨进封神台，天变得越发倒悬。残破的天庭殿宇漂浮在狂化万族的哀嚎里——每一扇窗后，都是一双被封神榜典掉的眼睛。",
        "封神台核心静静立着第三张封神榜残页；深处的诛仙剑痕残壁与万族囚笼，藏着封神真相的最后一块拼图。",
    ]),
    choices: &[
        ChoiceDef { label: "【查看封神台核心】", sub: "残页三 · 封神真相核心", cond: None, effects: &NO_EFF, route: Route::To("tt_08_fengshen_core") },
        ChoiceDef { label: "【去诛仙剑痕残壁】", sub: "真相关键 · 三清归位", cond: None, effects: &NO_EFF, route: Route::To("tt_09_truth_wall") },
        ChoiceDef { label: "【进万族囚笼】", sub: "San-8 · 真相线索", cond: None, effects: &NO_EFF, route: Route::To("tt_10_cages") },
        ChoiceDef { label: "（L3 已开）主升降井下行", sub: "fL4", cond: Some(cond_lingxiao_open), effects: &NO_EFF, route: Route::To("tt_12_lingxiao") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_08_fengshen_core", bg: Some("tianting_bg.png"), loc: Some("L3 封神台核心"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "封神台核心的正中，第三张封神榜残页被一圈狂化的万族虚影护住。残页上的墨痕，恰好接续了云海断碑被劈断的那句——",
        "『……镇压的从来不是天道，是东天二皇。』",
    ]),
    choices: &[
        ChoiceDef { label: "【抢下封神榜残页·三】", sub: "残页三 · San-5", cond: None,
            effects: &[Eff::SetFlag("tt_fengshen_p3"), Eff::San(-5), Eff::PointsIfFlag("tt_fengshen_p3", 120)], route: Route::To("tt_09_truth_wall") },
        ChoiceDef { label: "（先去万族囚笼看真相）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tt_10_cages") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_09_truth_wall", bg: Some("tianting_bg.png"), loc: Some("L3 诛仙剑痕残壁"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "封神台深处的残壁上，横着一道诛仙四剑劈出的剑痕。剑痕之下的石皮剥落，露出一行被隐去了几个世纪的刻字：",
        "『封神榜者，非天道，乃蛇之布局、二皇之刑。伏羲四象五行八卦阵，逆此而封。』",
        "（封神真相 · 关键拼图。）",
    ]),
    choices: &[
        ChoiceDef { label: "【拼接三张残页 → 洞悉真相】", sub: "需残页一二三 → tt_fengshen_truth", cond: Some(cond_has_p3),
            effects: &[Eff::SetFlag("tt_fengshen_truth"), Eff::SetFlag("tt_fengshen_truth_seed"), Eff::PointsIfFlag("tt_fengshen_truth", 200)], route: Route::To("tt_11_fengshen_truth") },
        ChoiceDef { label: "【记录剑痕后退出】", sub: "暂不拼接 · 仍可强打封神台", cond: None,
            effects: &NO_EFF, route: Route::To("tt_08_fengshen") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_11_fengshen_truth", bg: Some("tianting_bg.png"), loc: Some("L3 诛仙剑痕 · 真相揭晓"),
    mood: "danger", speaker: Some("录事官残魂 · 敬（残响）"), voice: None,
    text: TextSpec::Static(&[
        "三张封神榜残页在你手中相合，你读懂了整件事——封神榜吞噬万族气数供养『天』，而天上坐着的，是东天二皇。",
        "伏羲的四象五行八卦阵，不是为了灭天庭，而是为了把倒悬的王座，重新翻回人该站的天空之下。",
        "（你洞悉了真相。封神台结界对你这般『洞悉者』，裂开了一条路。）",
    ]),
    choices: &[
        ChoiceDef { label: "【赴封神台结界（G3）开战】", sub: "BOSS · tf_shenjiang_r1", cond: None,
            effects: &[Eff::SetFlag("tt_can_interrupt")], route: Route::To("tt_13_fight_gate") },
        ChoiceDef { label: "（先看万族囚笼）", sub: "San-8 · 补全真相", cond: None, effects: &NO_EFF, route: Route::To("tt_10_cages") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_10_cages", bg: Some("tianting_bg.png"), loc: Some("L3 万族囚笼"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("tt_fengshen_truth") {
            "囚笼里空了大半——那些『升者』的残魂，在你洞悉真相的那一刻，不再是罪人，而是一张张被撕毁的封神榜名册。你听见他们说：谢谢你，愿我们的牺牲，换来你头顶的天不再倒悬。（San -2）".to_string()
        } else {
            "万族囚笼里，一排排被狂化的残魂隔着牢门朝你抓挠。它们眼底都烧着同一行封神榜的印记：『升者·待镇』。（San -6）".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "（转身，走向封神台结界）", sub: "G3 · 视真相而定", cond: None,
        effects: &[Eff::San(-6)], route: Route::Dyn(route_gate3) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 凌霄殿残殿 · 倒悬王座（L4 入口；条件路由：BOSS 已破 → 结局抉择，未破 → 迎战封神投影） */
SceneDef {
    id: "tt_12_lingxiao", bg: Some("tianting_bg.png"), loc: Some("L4 凌霄殿残殿 · 倒悬王座"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("tt_lingxiao_open") {
            "主升降井将你送上凌霄殿残殿。黄金王座仍倒悬在天顶，可封神投影已在你手中溃散——王座下，人皇伏羲那未竟的四象五行八卦阵边缘，泛着一圈等你落子的微光。".to_string()
        } else {
            "你从主升降井登上凌霄殿残殿，黄金王座倒悬在天顶。可通往王座的必经之路上，封神台的狂化余烬又凝聚成一尊金甲神将——封神投影横在她与王座之间。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【（前往倒悬王座）】", sub: "视战况而定", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_lingxiao) },
        ChoiceDef { label: "（回头检查诛仙剑痕残壁）", sub: "若洞悉真相可强化打断", cond: None,
            effects: &NO_EFF, route: Route::To("tt_09_truth_wall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_13_fight_gate", bg: Some("tianting_bg.png"), loc: Some("L3 封神台结界 · 开战前"),
    mood: "danger", speaker: Some("天庭神将 · 封神投影"), voice: Some("vo_tianting_shenjiang"),
    text: TextSpec::Static(&[
        "结界在你面前裂开。封神台的狂化余烬冲天而起，凝聚成一尊金甲神将——封神投影。",
        "「受东天二皇之名，镇，凡升者。」它把封神章印按进大地，圣裁虚影轰然炸响。",
    ]),
    choices: &[
        ChoiceDef { label: "【迎战（一形态）】", sub: "tf_shenjiang_r1", cond: None,
            effects: &NO_EFF, route: Route::Dyn(start_shenjiang_r1) },
        ChoiceDef { label: "（检查诛仙剑痕残壁）", sub: "若洞悉真相可推动打断", cond: None, effects: &NO_EFF, route: Route::To("tt_09_truth_wall") },
    ],
    fight_id: None, video: None, cine_label: Some("决战 · 天庭神将 · 封神投影"), overlay: None,
},

/* ---- 幕 3 ・ 决战：封神投影 两段式（场景链） ---- */
SceneDef {
    id: "tt_14_r1", bg: Some("tianting_bg.png"), loc: Some("L3 封神台 · 决战 · 一形态"),
    mood: "danger", speaker: Some("天庭神将 · 封神投影"), voice: None,
    text: TextSpec::Dyn(txt_boss_r1),
    choices: &[
        ChoiceDef { label: "【开始战斗（一形态）】", sub: "tf_shenjiang_r1 · HP260", cond: None,
            effects: &NO_EFF, route: Route::To("tt_14_round_r1") },
        ChoiceDef { label: "（检查残页能否打断）", sub: "需两真知 · 蓄力轮生效", cond: None,
            effects: &NO_EFF, route: Route::To("tt_14_interrupt_check") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_14_interrupt_check", bg: Some("tianting_bg.png"), loc: Some("L3 封神台 · 打断判定"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("tt_fengshen_truth") {
            "你认出封神章印与三张封神榜残页同源——集齐残页、洞悉真相者，可用「人皇封条」打断它的蓄力！".to_string()
        } else {
            "封神章印古老而陌生，你无从下手。要打断圣裁，也许得先拼全封神榜残页。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【强大：以人皇封条打断蓄力】", sub: "需 tt_fengshen_truth · 生效", cond: Some(cond_has_truth),
            effects: &[Eff::SetFlag("tt_r1_interrupted")], route: Route::To("tt_14_round_r1") },
        ChoiceDef { label: "（无法打断，直接开战）", sub: "无打断机制", cond: None,
            effects: &NO_EFF, route: Route::To("tt_14_round_r1") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 一段战 · 选择驱动回合 */
SceneDef {
    id: "tt_14_round_r1", bg: Some("tianting_bg.png"), loc: Some("L3 · 决战 · 一形态"),
    mood: "danger", speaker: Some("⚔ 决战"), voice: None,
    text: TextSpec::Dyn(txt_round_r1),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-46", cond: None, effects: &NO_EFF,
            route: Route::Dyn(|st| route_r1_attack(st, rnd_tt(30, 46))) },
        ChoiceDef { label: "【以人皇封条打断蓄力】", sub: "需真相 · 大额伤害", cond: Some(cond_has_truth),
            effects: &NO_EFF, route: Route::Dyn(|st| { st.set_flag("tt_r1_interrupted"); route_r1_attack(st, rnd_tt(25, 45)) }) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 转场演出：东天二皇投影（圣位红线 · 演出级 · 不可战） */
SceneDef {
    id: "tt_huang_cast", bg: Some("tianting_bg.png"), loc: Some("L3 封神台上空 · 转场演出"),
    mood: "danger", speaker: Some("东皇太一投影"), voice: Some("vo_tianting_taiyi"),
    text: TextSpec::Static(&[
        "一形态溃灭的刹那，封神台的天幕轰然碎裂。一线双重日冕剪影自倒悬的天顶降临——",
        "帝俊执河图洛书，太一执东皇钟。太一一字一顿：「镇压——凡升者，必坠。」帝俊沉默，河图洛书的光芒一暗。",
        "天条如瀑布垂落，把两皇投影连同一整个时代钉在原地。你感到脚下的大地在无数个『升者』的挣扎中，剧烈地翻了个面——<b>王座在倒。</b>",
    ]),
    choices: &[
        ChoiceDef { label: "（凝望倒悬王座被撼动）", sub: "演出 · 转入二形态", cond: None,
            effects: &[Eff::San(-6)], route: Route::Dyn(start_shenjiang_r2) },
    ],
    fight_id: None, video: Some("cine_huang_yaji_tianting"), cine_label: Some("东天二皇 · 镇压"), overlay: None,
},
/* 二段战 · 入口 */
SceneDef {
    id: "tt_15_r2", bg: Some("tianting_bg.png"), loc: Some("L3 封神台 · 决战 · 二形态"),
    mood: "danger", speaker: Some("天庭神将 · 封神投影（真灵）"), voice: None,
    text: TextSpec::Dyn(txt_boss_r2),
    choices: &[
        ChoiceDef { label: "【开始决战（二形态）】", sub: "tf_shenjiang_r2 · HP150", cond: None,
            effects: &NO_EFF, route: Route::To("tt_15_round_r2") },
        ChoiceDef { label: "（狂化提示）", sub: "封神狂化 San-3/击", cond: None,
            effects: &NO_EFF, route: Route::To("tt_15_round_r2") },
    ],
    fight_id: None, video: None, cine_label: Some("决战 · 封神投影·真灵"), overlay: None,
},
/* 二段战 · 选择驱动回合 */
SceneDef {
    id: "tt_15_round_r2", bg: Some("tianting_bg.png"), loc: Some("L3 · 决战 · 二形态"),
    mood: "danger", speaker: Some("⚔ 决战"), voice: None,
    text: TextSpec::Dyn(txt_round_r2),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 34-50", cond: None, effects: &NO_EFF,
            route: Route::Dyn(|st| route_r2_attack(st, rnd_tt(34, 50))) },
        ChoiceDef { label: "【以封神真相断章（抢先结算）】", sub: "需真相 · 提前结束", cond: Some(cond_has_truth),
            effects: &NO_EFF, route: Route::Dyn(route_r2_surrender_truth) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 4 ・ 抉择结局（倒悬王座前） ---- */
SceneDef {
    id: "tt_17_choice", bg: Some("tianting_bg.png"), loc: Some("L4 凌霄殿残殿 · 倒悬王座前"),
    mood: "danger", speaker: Some("帝俊（投影残响）"), voice: None,
    text: TextSpec::Static(&[
        "封神投影彻底溃散。你站在凌霄殿残殿的废墟中央——头顶，是一座倒悬的黄金王座；王座下，压着人皇伏羲那未竟的四象五行八卦阵的边缘。",
        "东天二皇的投影早已消散，可那句「凡升者，必坠」还在天顶回荡。轮到你了：倒悬的王座，是塌，是翻，还是……为自己留下点什么。",
    ]),
    choices: &[
        ChoiceDef { label: "【揭开封神榜真相，传扬天下】", sub: "真相结局 · 支线评 A", cond: None,
            effects: &[Eff::SetFlag("tt_ending_unmask")], route: Route::To("tt_16_ending_unmask") },
        ChoiceDef { label: "【助人皇伏羲之阵印，重镇天庭】", sub: "封印结局 · 支线评 A", cond: None,
            effects: &[Eff::SetFlag("tt_ending_seal")], route: Route::To("tt_16_ending_seal") },
        ChoiceDef { label: "【自取神性碎片（留待自身）】", sub: "神性结局 · 支线评 A", cond: None,
            effects: &[Eff::SetFlag("tt_ending_self")], route: Route::To("tt_16_ending_self") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* ---- 结局（按 flag 分支） ---- */
SceneDef {
    id: "tt_16_ending_unmask", bg: Some("tianting_bg.png"), loc: Some("L4 凌霄殿 · 真相传世"),
    mood: "calm", speaker: Some("李铭"), voice: Some("vo_tianting_liming_unmask"),
    text: TextSpec::Static(&[
        "你把三张封神榜残页并作一卷，掷向那些焚毁的万族名册。封神真相随天风传遍残境——「镇压的从来不是天道，是东天二皇。」",
        "天幕裂开数道缝，无数『升者』的残魂在真相里抬眼。李铭：「高难修正完成——封神榜的伪装，被你们扒了下来。」",
        "（结局·揭封神榜真相：倒悬的王座，第一次在无数眼睛里现出它伪造天道的嘴脸。）",
    ]),
    choices: &[
        ChoiceDef { label: "（踏入撤离传送门 → 主神空间）", sub: "pt_exit · 结算 · sp_grade A", cond: None,
            effects: &[Eff::SetFlag("tt_cleared")], route: Route::Dyn(settle_a) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_16_ending_seal", bg: Some("tianting_bg.png"), loc: Some("L4 凌霄殿 · 助伏羲镇天"),
    mood: "calm", speaker: Some("伏羲（残响）"), voice: Some("vo_tianting_fuxi"),
    text: TextSpec::Static(&[
        "你循着残页上的四象五行阵角，补完人皇伏羲那未竟的封印。青龙白虎朱雀玄武次第归位，倒悬的王座被一缕人道的微光，缓缓翻回它该在的地方。",
        "伏羲的残响自阵中传来：「你们不是我，却也替我走完了这一步。愿人族的天空，从此不再倒悬。」",
        "（结局·助人皇阵印：四象五行八卦阵重现，天庭被以人道重新定格。）",
    ]),
    choices: &[
        ChoiceDef { label: "（踏入撤离传送门 → 主神空间）", sub: "pt_exit · 结算 · sp_grade A", cond: None,
            effects: &[Eff::SetFlag("tt_cleared")], route: Route::Dyn(settle_a) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tt_16_ending_self", bg: Some("tianting_bg.png"), loc: Some("L4 凌霄殿 · 自取神性"),
    mood: "cold", speaker: Some("李铭（旁白）"), voice: None,
    text: TextSpec::Static(&[
        "你没有选择把真相传世，也没有替伏羲补阵。你伸手，从封神投影溃散之处，攫走了一枚仍泛着圣光的「神性碎片」。",
        "李铭望着你，语气平淡：「倒悬的王座塌了一角，会有新的名字坐上去。你带着块碎片回去——它烫手，但你不能否认……它很有用。」",
        "（结局·自取神性碎片：个人获益，神性碎片兑换券 + 神性碎片入手。）",
    ]),
    choices: &[
        ChoiceDef { label: "（携带碎片撤离 → 主神空间）", sub: "pt_exit · 结算 · sp_grade A", cond: None,
            effects: &[Eff::SetFlag("tt_cleared"), Eff::AddItem("item_shenxing_fragment")], route: Route::Dyn(settle_a) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 结算卡（sp_grade A） ---- */
SceneDef {
    id: "tt_18_settle", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "倒 悬 的 王 座 ——".into(), good: true,
            body_html: format!(
                "<p>你从被封印的天庭残境全身而退。脚下的云海，这一次是正的。</p>\
                 <p style='color:#9a958a'>《洪荒天庭 · 封神战场》高难副本 · 已完成</p>\
                 <table class='statTable'><tr><td>奖励点数</td><td>{}</td></tr><tr><td>理智</td><td>{}</td></tr></table>\
                 <p style='color:#ffd76a'>支线评级：{}　（高难副本统一 A 级；结算见主神．）</p>\
                 <p style='color:#b0c4de'>东天二皇的镇压穿透了几千年，可在你们手里，它的谎话第一次被凿穿。</p>",
                st.points, st.san.max(0),
                st.sp_grade.map(|g| format!("{g} 级")).unwrap_or_else(|| "暂无".into())
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: Some("vo_tianting_liming_done"),
        },
    }),
},
/* ---- 失败 / 死亡档案 ---- */
SceneDef {
    id: "tt_lose_r1", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被封神章印钉名", "封神榜在倒悬王座下，替你写了名")), card: |_st| crate::state::Card {
            title: "被 钉 进 封 神 榜".into(), good: false,
            body_html: r#"<p>封神章印在你眉心落笔。倒悬的天替你写了名——东天二皇的圣裁，在最后一刻钉住了你。</p>
<p style='color:#ff8a8a'>【死亡档案 · 被钉进封神榜】＃0003</p>
<p style='color:#666'>(复活：回主神空间扣除相应点数，复活系统接线。本条死亡历史已由记录员修正。)</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "tt_lose_r2", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被狂化的万族吞没", "封神残页卷过的万族虚影，把你当成了新的名")), card: |st| {
            let sealed = st.flag("tt_fengshen_truth");
            crate::state::Card {
                title: if sealed { "被 拒 出 倒 悬 天（幸存）".into() } else { "封 神 狂 化 吞 没".into() },
                good: false,
                body_html: if sealed {
                    r#"<p>封神投影看穿了你洞悉真相的眼睛。它没有杀你——用最后一丝被撕裂的圣裁，把你们整队推出倒悬天。</p>
<p style='color:#ff8a8a'>【特殊失败 · 已洞悉真相】只扣 200 点，全部 flag 保留。</p>"#.to_string()
                } else {
                    r#"<p>狂化的万族虚影卷着残页扑上来，把你拖进封神台的墨色深处。耳边只剩那句不属于任何种族的：「凡升者，必坠。」</p>
<p style='color:#ff8a8a'>【死亡档案 · 被封神狂化吞没】＃0003</p>
<p style='color:#666'>(复活：回主神空间扣相应点数。本条死亡历史已由记录员修正。)</p>"#.to_string()
                },
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            }
        },
    }),
},
SceneDef {
    id: "tt_lose_common", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("战死天庭残境", "倒悬天压下来的前一刻")), card: |_st| crate::state::Card {
            title: "战 死 天 庭 残 境".into(), good: false,
            body_html: r#"<p>你在倒悬的天庭砖瓦间倒下。云海在你头顶，王座在你脚下——这一次，是你先坠。</p>
<p style='color:#ff8a8a'>【死亡档案 · 战死天庭残境】</p>
<p style='color:#666'>(复活：回主神空间扣除相应点数。本条死亡历史已由记录员修正。)</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* =====================================================================
   路由辅助（Route::Dyn 引用；含 BOSS 选择驱动轮次、结局 sp_grade 结算）
   ===================================================================== */
fn route_gate3(st: &mut GameState) -> String {
    st.set_flag("tt_can_interrupt");
    "tt_13_fight_gate".to_string()
}

/// L4 凌霄殿残殿前置路由：BOSS（封神投影）已击破 → 结局抉择；未破 → 迎战封神投影
fn route_lingxiao(st: &mut GameState) -> String {
    if st.flag("tt_lingxiao_open") {
        "tt_17_choice".to_string()
    } else {
        "tt_13_fight_gate".to_string()
    }
}

/// 初始化一段 BOSS 会话（从 tf_shenjiang_r1 的 FightCfg 建 Fight）
fn start_shenjiang_r1(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("tf_shenjiang_r1") {
            st.fight = Some(crate::power::scaled_fight("tf_shenjiang_r1", cfg, st, vec![]));
        }
    }
    "tt_14_round_r1".to_string()
}
fn route_r1_attack(st: &mut GameState, dmg: i32) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    // 圣裁回响：若未被打断，蓄力越深反击越重
    let p_dmg = if st.flag("tt_r1_interrupted") { rnd_tt(14, 22) } else { rnd_tt(20, 30) };
    st.hp = (st.hp - p_dmg).max(0);
    if st.hp <= 0 { return "tt_lose_r1".to_string(); }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        st.points += 600;
        return "tt_huang_cast".to_string();
    }
    "tt_14_round_r1".to_string()
}
/// 初始化二阶段 BOSS（承接一段胜利奖励，重开法阵）
fn start_shenjiang_r2(st: &mut GameState) -> String {
    if let Some(cfg) = crate::scenes::fight_cfg("tf_shenjiang_r2") {
        st.fight = Some(crate::power::scaled_fight("tf_shenjiang_r2", cfg, st, vec![]));
    }
    "tt_15_r2".to_string()
}
fn route_r2_attack(st: &mut GameState, dmg: i32) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    let p_dmg = rnd_tt(24, 36);
    st.hp = (st.hp - p_dmg).max(0);
    if st.hp <= 0 { return "tt_lose_r2".to_string(); }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        st.points += 800;
        st.set_flag("tt_lingxiao_open");
        refine_r2_reward(st);
        return "tt_17_choice".to_string();
    }
    "tt_15_round_r2".to_string()
}
fn route_r2_surrender_truth(st: &mut GameState) -> String {
    st.set_flag("tt_lingxiao_open");
    st.set_flag("tt_surrendered_truth");
    st.points += 800;
    refine_r2_reward(st);
    "tt_17_choice".to_string()
}
/// 二段战胜利结算（神性碎片兑换券入手；sp_grade 在结局 settle 另设 A）
fn refine_r2_reward(st: &mut GameState) {
    crate::world::add_item(st, "item_shenxing_voucher");
}
/// 结局结算路由：统一置 sp_grade = 'A'，进入结算卡
fn settle_a(st: &mut GameState) -> String {
    if st.flag("tt_ending_self") {
        // 自取神性碎片：掉落兑换券（结局已加碎片本身）
        crate::world::add_item(st, "item_shenxing_voucher");
    }
    st.sp_grade = Some('A');
    "tt_18_settle".to_string()
}

/// 本文件场景查询辅助（主线合并查询扩展时可直接使用）
pub fn tt_scene(id: &str) -> Option<&'static SceneDef> {
    TIANTING_SCENES.iter().find(|s| s.id == id)
}

/// 查询辅助（主线合并 fight_cfg 扩展时可直接调用）
pub fn tt_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    tianting_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}
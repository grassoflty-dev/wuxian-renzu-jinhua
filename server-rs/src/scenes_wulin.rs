//! 《侠行天下 · 武林大会》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.7 武林大会行 + xiaxing_tianxia research 候选3。
//! 本文件为全新新增文件，只导出静态数据（WULIN_SCENES / wulin_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/wulin_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `wl_` 前缀，与既有 SCENES 无重名；fight id 全部 `wc_` 前缀。
//! 擂台轮战=FIGHTS 连号战链（wc_fight_1..3 + wc_hei_ma），由 wl_arena hub 顺序调度。
//! 卧底=flag 网审/身份反转：密令(→wl_black_hint)、旧案宗卷(→wl_plot_exposed)、卧底密室(→wl_wo_di_found) 拼合。
//! BOSS 黑化盟主采用「选择驱动遭遇链」（参考 scenes_zhouyuan.rs 的 zy_boss_round / scenes_jiguancheng.rs 的 colossus）：
//! 需「反戈相助（卧底反转）/ 出示信物 / 力战」等自定义每回合同调，引擎原生 FightCfg 无此钩子，
//! 故用 Normal 场景 + Route::Dyn 落地；同时导出 `wc_menzhu` FightCfg 供 ZoneDef 与揭示用。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 坊市 井 bg wl_bg_gate  （现用 img_zhuyuan_book.png 占位）
//!   L2 擂台 井 bg wl_bg_arena （现用 img_nexus.png 占位）
//!   L3 后台 井 bg wl_bg_rear  （现用 img_zhuyuan_book.png 占位）
//!   L4 密道 井 bg wl_bg_palace（现用 img_corridor.png 占位）
//! 敌人立绘复用 §3：guard→护院、hunter→黑马高手面纱客、zombie→魔教教众暴徒；新美术由主 agent 统一生图替换。

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
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_signed(st: &GameState) -> bool { st.flag("wl_signin_done") }
fn cond_plot_exposed(st: &GameState) -> bool { st.flag("wl_plot_exposed") }
/// 卧底已明 + 盟主黑化已揭 → 可反戈相助/出示信物免战线
fn cond_revolt_possible(st: &GameState) -> bool {
    st.flag("wl_wo_di_found") && st.flag("wl_plot_exposed")
}
/// 卧底仍有信物 + BOSS 狂暴 → 出示信物揭露免战
fn cond_show_token(st: &GameState) -> bool {
    inv(st, "it_wl_token") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false)
}
fn cond_after_all_fights(st: &GameState) -> bool {
    st.flag("wl_beat_1") && st.flag("wl_beat_2") && st.flag("wl_beat_3") && st.flag("wl_beat_hm")
}
/// 结局已解锁（胜利或反转后出现撤离阵选项）
fn cond_end_unlock(st: &GameState) -> bool {
    st.flag("wl_end") || st.flag("wl_revolt")
}
/// 擂台登台路由：进战前记下本场已挑（供连胜链判定）
fn r_beat_1(st: &mut GameState) -> String { st.set_flag("wl_beat_1"); "wl_fight_1".to_string() }
fn r_beat_2(st: &mut GameState) -> String { st.set_flag("wl_beat_2"); "wl_fight_2".to_string() }
fn r_beat_3(st: &mut GameState) -> String { st.set_flag("wl_beat_3"); "wl_fight_3".to_string() }
fn r_beat_hm(st: &mut GameState) -> String { st.set_flag("wl_beat_hm"); "wl_fight_hei_ma".to_string() }

/* =====================================================================
   BOSS · 黑化盟主（选择驱动遭遇链）
   血量存 st.fight（由 wl_menzhu_intro 的 Route::Dyn 初始化，引用 wc_menzhu 的 FightCfg）。
   ===================================================================== */
/// 初始化盟主会话（从 wc_menzhu 的 FightCfg 建 Fight）。需主线合并后 fight_cfg 能解析 wc_menzhu。
fn start_menzhu(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("wc_menzhu") {
            st.fight = Some(crate::power::scaled_fight("wc_menzhu", cfg, st, vec![]));
        }
    }
    "wl_menzhu_round".to_string()
}

/// 盟主胜利结算：+500 全奖、落 flag wl_menzhu_down、写 sp_grade=D，返回胜利幕。
fn menzhu_win(st: &mut GameState) -> String {
    st.points += 500;
    st.set_flag("wl_menzhu_down");
    st.set_flag("wl_end");
    st.sp_grade = Some('D');
    "wl_32_card".to_string()
}

/// 反戈相助（卧底反转）胜利：+680 全奖、信物、落 wl_revolt 结局 flag、sp_grade=D。
fn menzhu_revolt(st: &mut GameState) -> String {
    st.points += 680;
    crate::world::add_item(st, "it_wl_menzhu_tally");
    st.set_flag("wl_menzhu_down");
    st.set_flag("wl_revolt");
    st.set_flag("wl_end");
    st.sp_grade = Some('D');
    "wl_33_revolt".to_string()
}

fn menzhu_dead() -> String { "wl_40_death_menzhu".to_string() }

/// 每回合：入力进攻 / 卸力防守；若出示信物/反戈则走独立终结行。
/// dmg 由场景选项给出（重击/轻功/卸力）。
fn menzhu_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return menzhu_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 88).unwrap_or(false); // 220*40%
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(24, 34) } else { rng(16, 24) };
    let dodge = if guard { 0.55 } else { 0.18 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return menzhu_dead();
    }
    "wl_menzhu_round".to_string()
}

/* =====================================================================
   普通敌人 win 回调（native FightCfg 由引擎在胜负后调用 win）
   ===================================================================== */
fn wl_win_arena(_st: &GameState) -> String { "wl_arena".to_string() }
fn wl_win_hu1(_st: &GameState) -> String { "wl_00".to_string() }
fn wl_win_tie(_st: &GameState) -> String { "wl_back".to_string() }
fn wl_win_jiao(_st: &GameState) -> String { "wl_secret".to_string() }
fn wl_win_hei_ma(_st: &GameState) -> String { "wl_after_hm".to_string() }
fn wl_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 战斗配置表（id 全部 wc_ 前缀）。
pub fn wulin_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("wc_hu_yuan", FightCfg {
            name: "山门护院", hp: 40, dmg: (7, 13), reward: 15, reward_why: "肃清山门护院",
            intro: "横臂短棍的护院拦在门首，沉声喝令外人不得擅闯武林大会会场。",
            rage_at: None, rage_text: "", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_hu1, death: "wl_40_death",
        }),
        ("wc_fight_1", FightCfg {
            name: "峨嵋净空掌", hp: 60, dmg: (10, 16), reward: 60, reward_why: "擂台轮战第一场胜",
            intro: "执拂尘的峨嵋女修纵身跃上擂台，双掌一合，清越喝一声：「请！」",
            rage_at: None, rage_text: "", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_arena, death: "wl_40_death",
        }),
        ("wc_fight_2", FightCfg {
            name: "北吼门金狮拳", hp: 70, dmg: (11, 17), reward: 80, reward_why: "擂台轮战第二场胜",
            intro: "北吼门健儿抱拳一拱，狮吼般的气息随拳势涌来，擂鼓声一时盖过满场喝彩。",
            rage_at: Some(32), rage_text: "金狮法相怒目圆睁，拳风带虎狼势，攻速骤增！", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_arena, death: "wl_40_death",
        }),
        ("wc_fight_3", FightCfg {
            name: "崂山鬼门刀", hp: 80, dmg: (12, 18), reward: 100, reward_why: "擂台轮战第三场胜",
            intro: "灰袍刀客横刀身前，刀身泛寒铁青光，是崂山鬼门刀传人——败者留名，胜者登顶。",
            rage_at: Some(38), rage_text: "刀光接连成网，招招索魂！", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_arena, death: "wl_40_death",
        }),
        ("wc_hei_ma", FightCfg {
            name: "黑马高手 · 面纱客", hp: 100, dmg: (14, 20), reward: 130, reward_why: "击退黑马高手 · 面纱客",
            intro: "面纱客一言不发跃上台，出手却招招阴诡——你不禁多想：此人武路，像极了传闻中的魔教暗手。",
            rage_at: Some(48), rage_text: "面纱被掌风掀开一角——露出的冷峻半张脸竟与盟主府金吾卫相似！", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_hei_ma, death: "wl_40_death",
        }),
        ("wc_tie_wei", FightCfg {
            name: "盟主府铁卫", hp: 85, dmg: (12, 18), reward: 60, reward_why: "闯过盟主府铁卫",
            intro: "披甲持戟的铁卫横在后台通廊，甲叶铿锵：「后台重地，闲人莫入！」",
            rage_at: Some(40), rage_text: "铁卫裂甲拔戟横斩，势大力沉！", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_tie, death: "wl_40_death",
        }),
        ("wc_jiao_zhong", FightCfg {
            name: "魔教教众暴徒", hp: 70, dmg: (11, 17), reward: 50, reward_why: "清剿密道魔教教众",
            intro: "阴影里窜出几个玄衣暴徒，腰悬铁牌，低语着魔教的切口。",
            rage_at: None, rage_text: "", on_rage: wl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: wl_win_jiao, death: "wl_40_death",
        }),
        ("wc_menzhu", FightCfg {
            name: "黑化盟主", hp: 220, dmg: (16, 24), reward: 500, reward_why: "击败黑化盟主 · 夺其盟主令",
            intro: "暗厅烛影里，端坐的盟主缓缓抬眼——那本该是豪迈的脸，此刻爬满魔纹。你闯入的，是武林大会最不该被撞破的秘密。",
            rage_at: Some(88), rage_text: "盟主瞳中黑雾炸裂，拔刀之际满厅兵器嗡鸣——<b>黑化狂暴</b>！伤害暴增，且每回合反震入体 5 点气血。",
            on_rage: |st, _log| { st.hp = (st.hp - 5).max(0); },
            finisher_if: |st, _| inv(st, "it_wl_token") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "出示信物 · 揭其卧底".to_string(),
            finisher_desc: |_| "你出示那枚密令信物，揭穿盟主身后的魔教勾连——满座哗然，盟主身形一顿，被你趁机窥破破绽。".to_string(),
            win: |_st| "wl_32_card".to_string(),
            death: "wl_40_death_menzhu",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn wl_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    wulin_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 wl_ 前缀）
   ===================================================================== */
pub static WULIN_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 山门坊市（开场 hub） ================= */
SceneDef {
    id: "wl_00", bg: Some("wulin_bg.png"), loc: Some("L1 · 山门会场坊市"),
    mood: "excitement", speaker: Some("大会执事 · 擂鼓鸣锣"), voice: Some("vo_wl_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>签到场会 → 登台夺擂 → 揭穿大会背后阴谋 → 讨伐黑化盟主。失败代价：被扣 400 点复活。",
        "三年一度的武林大会在山门坊市拉开帷幕。旌旗猎猎，兵器摊、药摊、留言壁一字排开；远处锣声一响，执事高喝「各派亮帖签到」。

        西隅暗门缝里，你瞥见一枚似曾相识的魔教密令——这届大会，怕是不止比武那么简单。

        ⊙ 主线钩子：「赢了擂主，输了人心。」",
    ]),
    choices: &[
        ChoiceDef { label: "到签到处亮帖签到", sub: "+15 点 · 领引帖", cond: None, effects: &NO_EFF, route: Route::To("wl_signin") },
        ChoiceDef { label: "看大会榜文", sub: "摸清赛制", cond: None, effects: &NO_EFF, route: Route::To("wl_notice") },
        ChoiceDef { label: "逛兵器摊", sub: "买把趁手的兵刃", cond: None, effects: &NO_EFF, route: Route::To("wl_stall") },
        ChoiceDef { label: "药摊买伤药", sub: "备一帖金创药", cond: None, effects: &NO_EFF, route: Route::To("wl_herb") },
        ChoiceDef { label: "观者留言壁", sub: "探口风", cond: None, effects: &NO_EFF, route: Route::To("wl_watch_post") },
        ChoiceDef { label: "西隅暗门", sub: "查魔教密令", cond: None, effects: &NO_EFF, route: Route::To("wl_dark_door") },
        ChoiceDef { label: "进入隆重场关", sub: "需已签到 · 登台", cond: Some(cond_signed), effects: &NO_EFF, route: Route::To("wl_arena") },
    ],
    fight_id: None, video: None, cine_label: Some("过场 · 武林大会开幕"), overlay: None,
},

/* ---- 坊市支线 ---- */
SceneDef {
    id: "wl_notice", bg: Some("wulin_bg.png"), loc: Some("L1 · 大会榜文"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["榜上墨迹：今科武林大会，登台连胜者夺魁，魁首与盟主比武夺号。落款处一点朱砂刺眼得像血。"]),
    choices: &[ChoiceDef { label: "默记赛制", sub: "+5 点 · 情报", cond: None, effects: &[Eff::Points(5)], route: Route::To("wl_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_signin", bg: Some("wulin_bg.png"), loc: Some("L1 · 大会签到处"),
    mood: "calm", speaker: Some("大会执事"), voice: None,
    text: TextSpec::Static(&["执事提笔勾名：「侠士何人？壮哉，引帖一枚，凭此登台。」他打量你两眼，压低声音：「台上水深，莫贪魁首。」"]),
    choices: &[ChoiceDef { label: "领引帖 · 签到", sub: "Item it_wl_post · 记名入册", cond: None,
        effects: &[Eff::SetFlag("wl_signin_done"), Eff::AddItem("it_wl_post"), Eff::Points(15)], route: Route::To("wl_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_stall", bg: Some("wulin_bg.png"), loc: Some("L1 · 兵器摊"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["兵器摊主耍弄一柄青锋：『好剑斩的未必是歹人。要不要来一柄壮壮胆？』——你总留意到摊角压着半页魔教的符纸。"]),
    choices: &[ChoiceDef { label: "买一柄青锋", sub: "Weapon + 线索 · 5 点", cond: None,
        effects: &[Eff::Points(5), Eff::Weapon(crate::state::Weapon::Sword)], route: Route::To("wl_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_herb", bg: Some("wulin_bg.png"), loc: Some("L1 · 药摊 · 伤药"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["药摊老妪递来一帖金创药，摩挲着药纸低声道：可巧，这几日有『贵人』收了好几副生骨断筋的药。"]),
    choices: &[ChoiceDef { label: "买下伤药", sub: "San 恢复 · 5 点", cond: None,
        effects: &[Eff::San(15), Eff::Points(5)], route: Route::To("wl_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_watch_post", bg: Some("wulin_bg.png"), loc: Some("L1 · 观者留言壁"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["留言壁贴满各派事先放话的帖子。你翻到一张被撕了半边的——落款竟是『某人』，透出一句：今夜诸派，莫登盟主台。"]),
    choices: &[ChoiceDef { label: "记住这句警告", sub: "+5 点 · 阴谋前兆", cond: None, effects: &[Eff::Points(5)], route: Route::To("wl_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_dark_door", bg: Some("wulin_bg.png"), loc: Some("L1 · 西隅暗门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["西隅暗门半掩，门缝里滑出一枚玄铁令牌——上刻魔教『血月』印记。你恍然：这届武林大会，早有魔教的手脚。"]),
    choices: &[ChoiceDef { label: "拾起密令信物", sub: "Item it_wl_token · 为卧底免战线埋钥", cond: None,
        effects: &[Eff::AddItem("it_wl_token"), Eff::SetFlag("wl_black_hint")], route: Route::To("wl_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 幕二 · 擂台广场（擂台轮战） ================= */
SceneDef {
    id: "wl_arena", bg: Some("wulin_bg.png"), loc: Some("L2 · 擂台广场"),
    mood: "excitement", speaker: Some("锣鼓声 · 满场喝彩"), voice: None,
    text: TextSpec::Dyn(|st| {
        if cond_after_all_fights(st) {
            "擂台轮战你连下四城，满场肃然，无人再敢轻视你这匹黑马。执事凑近低语：「真正的水深……在盟主的后台里。」".to_string()
        } else {
            "英雄擂上尘土飞扬，东西两侧各派高手的兵刃交击作响。擂主台前立着三面战旗——登台连胜，方能夺魁。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "登台 · 战峨嵋净空掌", sub: "第一场", cond: None, effects: &NO_EFF, route: Route::Dyn(r_beat_1) },
        ChoiceDef { label: "登台 · 战北吼金狮拳", sub: "第二场", cond: Some(cond_signed), effects: &NO_EFF, route: Route::Dyn(r_beat_2) },
        ChoiceDef { label: "登台 · 战崂山鬼门刀", sub: "第三场", cond: Some(cond_signed), effects: &NO_EFF, route: Route::Dyn(r_beat_3) },
        ChoiceDef { label: "迎战黑马高手 · 面纱客", sub: "疑点重重", cond: Some(cond_signed), effects: &NO_EFF, route: Route::Dyn(r_beat_hm) },
        ChoiceDef { label: "循执事所指 · 上后台", sub: "需连胜四场", cond: Some(cond_after_all_fights), effects: &NO_EFF, route: Route::To("wl_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* 擂台战 fight 入口场景（native FightCfg 由引擎处理胜负） */
SceneDef { id: "wl_fight_1", bg: Some("wulin_bg.png"), loc: Some("L2 · 英雄擂"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["锣响三声，峨嵋净空掌已立于台上。你纵身登台——擂台轮战开始。"]),
    choices: &NO_CH, fight_id: Some("wc_fight_1"), video: None, cine_label: None, overlay: None },
SceneDef { id: "wl_fight_2", bg: Some("wulin_bg.png"), loc: Some("L2 · 英雄擂"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["北吼门狮吼震场，你稳稳接住拳势。第二轮。"]),
    choices: &NO_CH, fight_id: Some("wc_fight_2"), video: None, cine_label: None, overlay: None },
SceneDef { id: "wl_fight_3", bg: Some("wulin_bg.png"), loc: Some("L2 · 英雄擂"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["鬼门刀寒光扑面，你侧身避过致命一刀，反手逼进。第三轮。"]),
    choices: &NO_CH, fight_id: Some("wc_fight_3"), video: None, cine_label: None, overlay: None },
SceneDef { id: "wl_fight_hei_ma", bg: Some("wulin_bg.png"), loc: Some("L2 · 英雄擂"), mood: "danger", speaker: Some("面纱客"), voice: None,
    text: TextSpec::Static(&["面纱客不答话，先动了手。剑走偏锋、招招取要害——这不是名门正派的武式。"]),
    choices: &NO_CH, fight_id: Some("wc_hei_ma"), video: None, cine_label: None, overlay: None },
SceneDef {
    id: "wl_after_hm", bg: Some("wulin_bg.png"), loc: Some("L2 · 擂台后台"),
    mood: "mystery", speaker: Some("面纱客"), voice: None,
    text: TextSpec::Static(&["败北的面纱客低声撂下一句：「道上的事，你一个外人莫要强出头。」他转身没入后台，你看见他腰间令牌——正是你拾到的那枚血月印记的同类！"]),
    choices: &[ChoiceDef { label: "尾随入后台", sub: "入 L3 · 阴谋线", cond: None, effects: &NO_EFF, route: Route::To("wl_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 幕三 · 后台 · 盟主府前堂（阴谋揭露） ================= */
SceneDef {
    id: "wl_back", bg: Some("wulin_bg.png"), loc: Some("L3 · 后台 · 盟主府前堂"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["后台帐幔重重，前堂却空了一半——本应挤满的护院，此刻只剩下零星几处。你嗅到一丝极淡的血腥气。"]),
    choices: &[
        ChoiceDef { label: "翻检密令夹层", sub: "查魔教密令", cond: None, effects: &NO_EFF, route: Route::To("wl_mi_ling") },
        ChoiceDef { label: "翻阅盟主旧案宗卷", sub: "查黑化铁证", cond: None, effects: &NO_EFF, route: Route::To("wl_old_case") },
        ChoiceDef { label: "查看后台议事壁", sub: "补全谋局", cond: Some(cond_plot_exposed), effects: &NO_EFF, route: Route::To("wl_back_hall") },
        ChoiceDef { label: "盘问杂役耳房", sub: "敲边鼓", cond: None, effects: &NO_EFF, route: Route::To("wl_servant") },
        ChoiceDef { label: "掀密道口 · 下密道", sub: "入 L4 · 需已知底细", cond: Some(cond_plot_exposed), effects: &NO_EFF, route: Route::To("wl_secret") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_mi_ling", bg: Some("wulin_bg.png"), loc: Some("L3 · 密令夹层"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["夹层里压着一沓以血月印封缄的密令——内容是：今夜大会散去前，『送』盟主一程。底下另有半句接不上。"]),
    choices: &[ChoiceDef { label: "记下密令", sub: "wl_black_hint · 确认魔教插手", cond: None,
        effects: &[Eff::SetFlag("wl_black_hint"), Eff::Points(10)], route: Route::To("wl_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_old_case", bg: Some("wulin_bg.png"), loc: Some("L3 · 盟主旧案宗卷"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["宗卷里夹着多封盟主以真迹盖印的密信——往来的对象，正是魔教。你翻到最后一页：这位『盟主』早在三年前就换了人，真盟主只怕已……reacting你攥紧信纸，真相已然大白。"]),
    choices: &[ChoiceDef { label: "确认黑化 · 铁证在手", sub: "wl_plot_exposed · 阴谋揭露", cond: None,
        effects: &[Eff::SetFlag("wl_plot_exposed"), Eff::San(-5), Eff::Points(15)], route: Route::To("wl_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_back_hall", bg: Some("wulin_bg.png"), loc: Some("L3 · 后台议事壁"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["议事壁的地面溅着未干涸的血点，一路延伸到密道口。你确认：这场武林大会，从一开始就是魔教设的局。"]),
    choices: &[ChoiceDef { label: "循血路下密道", sub: "入 L4 · 决战在即", cond: None, effects: &NO_EFF, route: Route::To("wl_secret") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_servant", bg: Some("wulin_bg.png"), loc: Some("L3 · 杂役耳房"),
    mood: "cold", speaker: Some("杂役老赵"), voice: None,
    text: TextSpec::Static(&["老赵筛着糠，压着嗓子：「侠士莫管闲事……这三年，盟主金吾卫里的人换了一茬又一茬，净是些生面孔。」"]),
    choices: &[ChoiceDef { label: "谢过老赵", sub: "+5 点 · 卧底早有端倪", cond: None, effects: &[Eff::Points(5)], route: Route::To("wl_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 幕四 · 盟主府密道（卧底反转 / 决战抉择） ================= */
SceneDef {
    id: "wl_secret", bg: Some("wulin_bg.png"), loc: Some("L4 · 盟主府密道"),
    mood: "danger", speaker: Some("烛影 · 低语"), voice: None,
    text: TextSpec::Static(&["密道烛火摇曳，两侧石壁刻满魔教的祭纹。前方暗厅里坐着『盟主』，身后立着一个通体玄衣的舵主——正是你要找的卧底。"]),
    choices: &[
        ChoiceDef { label: "摸进卧底密室", sub: "揪出魔教舵主", cond: None, effects: &NO_EFF, route: Route::To("wl_wo_di") },
        ChoiceDef { label: "暗查密道藏信", sub: "补全勾结细节", cond: None, effects: &NO_EFF, route: Route::To("wl_hidden_doc") },
        ChoiceDef { label: "看逃脱指示碑", sub: "预备退路", cond: None, effects: &NO_EFF, route: Route::To("wl_escape_sign") },
        ChoiceDef { label: "闯暗厅 · 决战黑化盟主", sub: "需真相大白", cond: Some(cond_plot_exposed), effects: &NO_EFF, route: Route::To("wl_menzhu_intro") },
        ChoiceDef { label: "循火把撤离阵", sub: "先斩后奏 · 撤离", cond: Some(cond_end_unlock), effects: &NO_EFF, route: Route::Dyn(route_exit_settle) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_wo_di", bg: Some("wulin_bg.png"), loc: Some("L4 · 卧底密室"),
    mood: "danger", speaker: Some("摩云舵主"), voice: None,
    text: TextSpec::Static(&["舵主见你举着那枚血月信物，瞳孔骤缩——他本欲出手，却在看清你油然而生的江湖气后，压低了刀：「原来是你……」他反手一指暗厅：『那里面坐的，根本不是我的人。』"]),
    choices: &[ChoiceDef { label: "窥破卧底 · 身份反转", sub: "wl_wo_di_found · 反戈线", cond: None,
        effects: &[Eff::SetFlag("wl_wo_di_found"), Eff::Points(20)], route: Route::To("wl_secret") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_hidden_doc", bg: Some("wulin_bg.png"), loc: Some("L4 · 密道藏信"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["砖缝里藏着一封告密信——是魔教某长老抱怨『盟主位不稳，当速速以血祭坛』。你明白了整盘棋。"]),
    choices: &[ChoiceDef { label: "收好藏信", sub: "+5 点 · 拔细", cond: None, effects: &[Eff::Points(5)], route: Route::To("wl_secret") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_escape_sign", bg: Some("wulin_bg.png"), loc: Some("L4 · 逃脱指示碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["石壁刻着「事败由此走」的记号，直通外场的撤离阵。你记下退路，以备万一。"]),
    choices: &[ChoiceDef { label: "记下退路", sub: "＋撤离阵坐标", cond: None, effects: &[Eff::Points(5)], route: Route::To("wl_secret") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 决战 · 黑化盟主（选择驱动遭遇） ================= */
SceneDef {
    id: "wl_menzhu_intro", bg: Some("wulin_bg.png"), loc: Some("L4 · 盟主议事暗厅"),
    mood: "danger", speaker: Some("黑化盟主"), voice: Some("vo_wl_menzhu"),
    text: TextSpec::Static(&["『真盟主三年前就死在我手上。』他撕开伪装，满厅兵刃齐鸣——武林大会真正的魁首，从来不是我。『所以，你也去死吧。』"]),
    choices: &[ChoiceDef { label: "拔剑 · 与黑化盟主一决", sub: "决战开始", cond: None, effects: &NO_EFF, route: Route::Dyn(start_menzhu) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "wl_menzhu_round", bg: Some("wulin_bg.png"), loc: Some("L4 · 盟主议事暗厅"),
    mood: "danger", speaker: Some("黑化盟主"), voice: None,
    text: TextSpec::Dyn(|st| {
        let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(0);
        format!("烛火摇曳，黑化盟主魔纹暴起。你与他缠斗已深，其残躯还剩 <b>{hp}</b> 气力——他刀势越沉，鹰隼般的目光越冷。")
    }),
    choices: &[
        ChoiceDef { label: "重击", sub: "稳定追击", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| { let d = rng(18, 24); menzhu_act(st, d, false) }) },
        ChoiceDef { label: "轻功闪进", sub: "高杀伤 · 有风险", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| { let d = rng(26, 32); menzhu_act(st, d, false) }) },
        ChoiceDef { label: "卸力防守", sub: "格挡蓄势", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| menzhu_act(st, rng(8, 12), true)) },
        ChoiceDef { label: "【出示信物 · 反戈相助】", sub: "卧底反转 · 免战", cond: Some(cond_revolt_possible), effects: &NO_EFF, route: Route::Dyn(menzhu_revolt) },
        ChoiceDef { label: "【出示信物 · 揭其卧底】", sub: "狂暴后可用 · 免战受创", cond: Some(cond_show_token), effects: &NO_EFF, route: Route::Dyn(menzhu_revolt) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结局幕 ================= */
SceneDef {
    id: "wl_32_card", bg: Some("wulin_bg.png"), loc: Some("盟主府 · 暗厅余烬"),
    mood: "calm", speaker: Some("武林各派"), voice: Some("vo_wl_win"),
    text: TextSpec::Static(&["黑化盟主颓然倒地，武林大会的闹剧就此收场。各派掌门这才看清，这三年来供上盟主位的是个冒名顶替的魔头。

        ⊙ 你赢了擂主，却也揭穿了人心的鬼。理想上你挽救了满堂英雄，可从此再无人敢轻易相信『江湖』二字。"]),
    choices: &[ChoiceDef { label: "（领英雄帖 · 离场）", sub: "sp_grade D · 结算", cond: None, effects: &NO_EFF, route: Route::Dyn(route_finalize) }],
    fight_id: None, video: None, cine_label: None, overlay: Some(OverlayDef {
        voice: Some("vo_wl_win"), death: None, card: |_st| crate::state::Card {
            title: "英 雄 擂 下".into(), good: true,
            body_html: r#"<p>满场刀兵散尽，你握着那枚血月信物站在晨光里。赢了擂主，也看清了人心。</p>
<p style='color:#7CCD7C'>【武林大会 · 夺魁 + 止损】</p>
<p style='color:#666'>（结算：sp_grade D · 武林大会副本通关）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "wl_33_revolt", bg: Some("wulin_bg.png"), loc: Some("盟主府 · 暗厅余烬"),
    mood: "calm", speaker: Some("摩云舵主"), voice: Some("vo_wl_revolt"),
    text: TextSpec::Static(&["你选择反戈相助——与真正的卧底摩云舵主联手，里应外合压下了黑化盟主最后的反扑。盟主令被你握在手中，一场朝廷与武林勾结的乱局就此止于暗处。"]),
    choices: &[ChoiceDef { label: "（持盟主令 · 离场）", sub: "sp_grade D · 反转结局", cond: None, effects: &NO_EFF, route: Route::Dyn(route_finalize) }],
    fight_id: None, video: None, cine_label: None, overlay: Some(OverlayDef {
        voice: Some("vo_wl_revolt"), death: None, card: |_st| crate::state::Card {
            title: "盟 主 令 下".into(), good: true,
            body_html: r#"<p>你赢了人心，也稳住了武林最后的体面。卧底低语散尽，盟主令重归正途。</p>
<p style='color:#7CCD7C'>【武林大会 · 反戈 + 荣誉】</p>
<p style='color:#666'>（结算：sp_grade D · 反转结局）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案（复活扣 400/回主神） ================= */
SceneDef {
    id: "wl_40_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("大会之下的无名者", "在武林大会的刀兵与暗巷里倒下")), card: |_st| crate::state::Card {
            title: "刀 兵 之 下".into(), good: false,
            body_html: r#"<p>热闹的武林大会，最后只记得闪过你眼前的最后一道刀光。</p>
<p style='color:#ff8a8a'>【死亡档案 · 刀兵之下】</p>
<p style='color:#666'>（复活：回主神空间扣 400 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "wl_40_death_menzhu", bg: None, loc: None, mood: "danger", speaker: None, voice: Some("vo_wl_death"),
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("黑化盟主之刀", "你在盟主议事暗厅里被黑化盟主斩杀")), card: |_st| crate::state::Card {
            title: "盟 主 之 刀".into(), good: false,
            body_html: r#"<p>暗厅烛火一灭，你的视线随黑化盟主那记横斩一同归于黑暗。武林大会的真相，终究没能传出去。</p>
<p style='color:#ff8a8a'>【死亡档案 · 盟主之刀】</p>
<p style='color:#666'>（复活：回主神空间扣 400 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* =====================================================================
   Route::Dyn 路由函数 / 结算辅助（供 static 数组使用）
   ===================================================================== */
fn route_exit_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    st.set_flag("wl_end");
    "wl_32_card".to_string()
}

fn route_finalize(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    if st.flag("wl_revolt") {
        "wl_33_revolt".to_string()
    } else {
        "wl_32_card".to_string()
    }
}
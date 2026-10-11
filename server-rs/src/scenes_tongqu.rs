//! 《侠行天下 · 通衢古镇 · 夜雨镖局》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.7 `tonggu_guzhen` 骨架
//! 与 xiaxing_tianxia/00_xiaxing_tianxia_research.md 第七节江湖群像/镖局侠义（推断·自创）。
//! 本文件是全新新增文件，只导出静态数据（TONGQU_SCENES / tongqu_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/tongqu_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `tq_` 前缀，与既有 SCENES 无重名；fight id 全部 `tqf_` 前缀。
//! BOSS 雪夜劫镖·蒙面头领采用"选择驱动遭遇链"（参考 scenes_zhouyuan.rs / scenes_jiguancheng.rs）：
//! 因需「夜雨回合场地判定」与「识破黑店密谋/以信物免战」等自定义每回合同调，引擎原生 FightCfg
//! 无此钩子，故用 Normal 场景 + Route::Dyn 落地；同时导出 `tqf_boss` FightCfg 供 ZoneDef 与揭示用。
//!
//! 三线汇流：追凶（江湖恩怨）/ 护镖（镖局侠义）/ 黑店（黑店阴谋）——各自完成择一段，即揭雪夜伏击真相、
//! 获 sp_grade=D 并汇入镇尾古宅决战。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 镇门  井 bg tq_bg_gate   （现用 img_zhuyuan_book.png 占位）
//!   L2 市井  井 bg tq_bg_street （现用 img_corridor.png 占位）
//!   L3 古宅  井 bg tq_bg_mansion（现用 img_train.png 占位）
//! 敌人立绘复用：guard→护院、hunter→黑店打手；新美术由主 agent 统一生图替换。

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

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包；逐个具名定义供 static 数组使用）
   ===================================================================== */
fn cond_has_biaoju(st: &GameState) -> bool { st.flag("tq_biaoju_trust") }
fn cond_has_trace(st: &GameState) -> bool { st.flag("tq_trace_2") }
fn cond_has_dark(st: &GameState) -> bool { st.flag("tq_heidian_truth") }
/// 已掌握追凶或黑店任一佐证，方可取信沈镖头接护镖
fn cond_can_accept(st: &GameState) -> bool { cond_has_trace(st) || cond_has_dark(st) }

/* =====================================================================
   BOSS · 雪夜劫镖·蒙面头领（选择驱动遭遇）
   血量存 st.fight（tq_boss_enter 的 Route::Dyn 初始化，引用 tqf_boss 的 FightCfg）。
   每"回"是一个 Normal 场景 tq_boss_round；Route::Dyn 统一处理：扣血、夜雨反震、胜负路由。
   ===================================================================== */
/// 初始化头领会话（从 tqf_boss 的 FightCfg 建 Fight）。需主线合并后 fight_cfg 能解析 tqf_boss
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("tqf_boss") {
            st.fight = Some(crate::power::scaled_fight("tqf_boss", cfg, st, vec![]));
        }
    }
    "tq_boss_round".to_string()
}

/// 头领败落：+520、得密信（护送终局），置 sp_grade=D
fn boss_win(st: &mut GameState) -> String {
    st.points += 520;
    crate::world::add_item(st, "it_tq_secret_letter");
    st.set_flag("tq_boss_down");
    st.sp_grade = Some('D');
    "tq_boss_win".to_string()
}

fn boss_dead() -> String { "tq_40_death_boss".to_string() }

/// 一"回"：玩家进攻头领。guard=后撤观察（提升闪避，夜雨反震）；dodge 判定夜雨闪避。
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return boss_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 70).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    // 蒙面头领雪夜攻袭：狂暴后加力；夜雨滴落 HUD（每回合微量气血流逝）
    let raw = if raged_now { rng(20, 28) } else { rng(14, 22) };
    st.hp = (st.hp - 2).max(0); // 夜雨寒气反震 Hurt(-2)
    let dodge = if guard { 0.52 } else { 0.15 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return boss_dead();
    }
    "tq_boss_round".to_string()
}

/* =====================================================================
   胜利 / 失败共通 win 回调（普通敌人；FightCfg.win 用）
   ===================================================================== */
fn tq_win_l1(_st: &GameState) -> String { "tq_01".to_string() }
fn tq_win_huyuan(_st: &GameState) -> String { "tq_20_huyuan_win".to_string() }
fn tq_win_thug(_st: &GameState) -> String { "tq_21_dark_win".to_string() }
fn tq_win_tuishun(_st: &GameState) -> String { "tq_22_inn_win".to_string() }
fn tq_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 战斗配置表（id 全部 tqf_ 前缀）。guard→护院、hunter→黑店打手复用。
pub fn tongqu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("tqf_lad", FightCfg {
            name: "街头闲汉", hp: 32, dmg: (6, 11), reward: 8, reward_why: "喝退挡路的闲汉",
            intro: "雨巷里趔趄出个醉醺醺的汉子，白刃反着冷光——通衢古镇的夜，连闲汉都不好惹。",
            rage_at: None, rage_text: "", on_rage: tq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tq_win_l1, death: "tq_40_death",
        }),
        ("tqf_huyuan", FightCfg {
            name: "镖局护院", hp: 42, dmg: (9, 15), reward: 20, reward_why: "与镖局护院过招",
            intro: "一杆红缨枪横在院门——护院误你为夜行贼，提枪便刺：「镖局重地，岂容宵小！」",
            rage_at: Some(20), rage_text: "护院拧腰换了一路枪势，攻势陡急！", on_rage: tq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tq_win_huyuan, death: "tq_40_death",
        }),
        ("tqf_thug", FightCfg {
            name: "黑店打手", hp: 44, dmg: (10, 16), reward: 25, reward_why: "放倒黑店打手",
            intro: "黑店的灶火一暗，两条壮汉从隔板后扑出，手攥杀猪刀——店里的气味不对，早该警觉。",
            rage_at: Some(24), rage_text: "打手掀翻案板，取来双斧，攻势如骤雨！", on_rage: tq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tq_win_thug, death: "tq_40_death",
        }),
        ("tqf_tuishun", FightCfg {
            name: "蒙面探子", hp: 50, dmg: (11, 17), reward: 30, reward_why: "截住黑店的蒙面探子",
            intro: "客栈后檐上一道人影正要翻墙——蒙面布下一双眼掠过恶怒，见你追近，拔刀便斩。",
            rage_at: Some(26), rage_text: "探子窥破你身法破绽，连斩三刀，刀刀追魂！", on_rage: tq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: tq_win_tuishun, death: "tq_40_death",
        }),
        ("tqf_boss", FightCfg {
            name: "雪夜劫镖·蒙面头领", hp: 180, dmg: (14, 22), reward: 520, reward_why: "镇尾古宅雪夜伏击 · 破其蒙面",
            intro: "雪夜里，蒙面头领自古宅中堂走出，斗笠压眉、黑巾遮面，手中雁翎刀缓缓出鞘——他开口，声线里藏着一点你认得的熟悉轰鸣：「镖，终究运到我这来了。」",
            rage_at: Some(70), rage_text: "他揭开半张蒙面——正是黑店幕后。刀势骤变，雪夜滴水成刃，攻势如寒潮席卷！",
            on_rage: |_st, _log| {},
            finisher_if: |st, _| st.flag("tq_heidian_truth") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "揭面指认".to_string(),
            finisher_desc: |_| "你当众揭破他是黑店幕后与护镖内鬼。刀在半空一滞——那一瞬的破绽，是雪夜唯一的机会。".to_string(),
            win: |_st| "tq_boss_win".to_string(),
            death: "tq_40_death_boss",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn tongqu_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    tongqu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 tq_ 前缀）
   ===================================================================== */
pub static TONGQU_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 夜雨投镇（tq_00）================= */
SceneDef {
    id: "tq_00", bg: Some("tongqu_bg.png"), loc: Some("通衢古镇 · 镇口"),
    mood: "mystery", speaker: Some("更夫梆声"), voice: Some("vo_tq_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>夜投通衢古镇，就一身镖局侠义——追凶、护卫、识破黑店，直指雪夜劫镖。失败代价：被扣 300 点复活。",
        "秋雨漫过镇口石碣，「通衢」两个大字被水汽洇得发胀。更夫的梆声隔雨传来，镖旗在风里猎猎——这座江湖边缘的古镇，正为一个不该交到镖局手里的东西，准备一场雪夜。",
    ]),
    choices: &[
        ChoiceDef { label: "细听更鼓", sub: "+5 点 · 藏雨打更的暗拍", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("tq_01") },
        ChoiceDef { label: "闻一闻官道的泥泞", sub: "雨后车辙线索", cond: None,
            effects: &NO_EFF, route: Route::To("tq_10_track") },
        ChoiceDef { label: "直入长街", sub: "往市井去", cond: None,
            effects: &NO_EFF, route: Route::To("tq_01") },
    ],
    fight_id: None, video: Some("vid_tq_open.mp4"), cine_label: Some("过场 · 通衢古镇夜雨"), overlay: None,
},

/* ---- L1 镇门 hub ---- */
SceneDef {
    id: "tq_01", bg: Some("tongqu_bg.png"), loc: Some("L1 · 通衢镇门"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "镇门望楼的水珠一滴接一滴坠下青石，留下深浅不一的车辙。通衢是南来北往的中继，什么人都往这儿过——镖师、商贩、还有那些不该在落雨天赶路的夜行人。",
        "（中轴长街通向市井；望楼下镇石碣、雨后车辙、半幅摔落的镖旗、当铺招牌可查。）",
    ]),
    choices: &[
        ChoiceDef { label: "镇门石碣", sub: "古镇沿革", cond: None, effects: &NO_EFF, route: Route::To("tq_05_stele") },
        ChoiceDef { label: "雨后车辙", sub: "追凶线索", cond: None, effects: &NO_EFF, route: Route::To("tq_10_track") },
        ChoiceDef { label: "镖旗残片", sub: "半幅摔落的镖旗", cond: None, effects: &NO_EFF, route: Route::To("tq_05_flag") },
        ChoiceDef { label: "当铺招牌", sub: "一个昔日的当票", cond: None, effects: &NO_EFF, route: Route::To("tq_04_dangpu") },
        ChoiceDef { label: "进市井长街", sub: "入 L2 市井街巷", cond: None, effects: &NO_EFF, route: Route::To("tq_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点场景 ---- */
SceneDef {
    id: "tq_05_stele", bg: Some("tongqu_bg.png"), loc: Some("L1 · 镇门石碣"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["石碣背面刻着古镇三百年：商道咽喉、镖局林立，近年却因一桩「押纲失踪案」逐渐败落——「镇以路兴，亦以路衰」。"]),
    choices: &[ChoiceDef { label: "指腹摩挲石碣刻痕", sub: "+5 点 · 记住三家镖局旧名", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("tq_p_l1_1")], route: Route::To("tq_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_10_track", bg: Some("tongqu_bg.png"), loc: Some("L1 · 雨后车辙 · 追凶线索"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["官道泥泞里两行车辙宽窄不一——后面那辆是重载镖车，压痕却浅得不像装货；却有一行极新的夜行脚印，直往黑店方向追去，足尖朝向镇尾。"]),
    choices: &[ChoiceDef { label: "记下车辙与脚印的朝向", sub: "追凶线索 · 指黑店", cond: None,
        effects: &[Eff::SetFlag("tq_trace_1"), Eff::MarkPoint("tq_p_l1_2")], route: Route::To("tq_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_05_flag", bg: Some("tongqu_bg.png"), loc: Some("L1 · 镖旗残片"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["半幅摔落的镖旗浸在泥水里，一角绣着「振远镖局」。旗面被人横着斩断——镖旗落地，是江湖最重的凶讯。"]),
    choices: &[ChoiceDef { label: "收起镖旗残片", sub: "伏笔 · 护镖始末", cond: None,
        effects: &[Eff::AddItem("it_tq_flag_shred"), Eff::MarkPoint("tq_p_l1_3")], route: Route::To("tq_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_04_dangpu", bg: Some("tongqu_bg.png"), loc: Some("L1 · 当铺招牌"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["当铺檐下挂着一张泛黄的当票，押物写着「一箱二十年陈·染血绸帕」。落款是当今镖局内外院的一个旧暗号。"]),
    choices: &[ChoiceDef { label: "记下当票暗号", sub: "护镖线伏笔", cond: None,
        effects: &[Eff::SetFlag("tq_dangpiao"), Eff::MarkPoint("tq_p_l1_4")], route: Route::To("tq_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 市井街巷 ================= */
SceneDef {
    id: "tq_02", bg: Some("tongqu_bg.png"), loc: Some("L2 · 市井长街"),
    mood: "cold", speaker: Some("镖旗猎猎"), voice: None,
    text: TextSpec::Static(&[
        "市井长街一分为二——西头是「振远镖局」的朱漆大门，东头是一家悬着旧酒幌的黑店。夜雨里，镖局灯火通明，黑店却早早落了板，招牌在风里吱呀作响。",
        "（镖局柜房的沈镖头端坐；黑店暗门、客栈追凶、当铺柜台散布街巷。）",
    ]),
    choices: &[
        ChoiceDef { label: "镖局柜房 · 沈镖头", sub: "接护镖委托", cond: None, effects: &NO_EFF, route: Route::To("tq_20_biaoju") },
        ChoiceDef { label: "黑店暗门", sub: "探黑店阴谋", cond: None, effects: &NO_EFF, route: Route::To("tq_21_dark_shop") },
        ChoiceDef { label: "客栈追凶", sub: "江湖恩怨 · 截探子", cond: None, effects: &NO_EFF, route: Route::To("tq_22_inn") },
        ChoiceDef { label: "当铺柜台", sub: "护镖物证", cond: None, effects: &NO_EFF, route: Route::To("tq_23_dangpu2") },
        ChoiceDef { label: "循线索出镇尾", sub: "入 L3 镇尾古宅", cond: Some(cond_has_biaoju), effects: &NO_EFF, route: Route::To("tq_03") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tq_20_biaoju", bg: Some("tongqu_bg.png"), loc: Some("L2 · 振远镖局 · 柜房"),
    mood: "cold", speaker: Some("老镖师 · 沈镖头"), voice: Some("vo_tq_biaoju"),
    text: TextSpec::Dyn(|st| {
        if st.flag("tq_biaoju_trust") {
            "沈镖头按了按你的肩头：「走吧。雪夜一到，镖局大门我守着。」内院密闸已为你敞开。".to_string()
        } else {
            "一名续着灰白胡须的老者抬头，把一盏粗茶推到你面前：「外地人？听口音不似本镇。昨夜押纲失踪，镖旗断在半路——你若要插手，先把这条命押上。」".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "接护镖委托", sub: "护卫镖局 · 得沈镖头信物", cond: Some(cond_can_accept),
            effects: &NO_EFF, route: Route::Dyn(tq_accept_biaoju) },
        ChoiceDef { label: "问押纲下落", sub: "追凶线索", cond: None, effects: &NO_EFF, route: Route::To("tq_22_inn") },
        ChoiceDef { label: "说黑店之疑", sub: "黑店阴谋", cond: None, effects: &NO_EFF, route: Route::To("tq_21_dark_shop") },
        ChoiceDef { label: "退回市井", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tq_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 护镖支线：接委托后 · 先与挡路护院过招 ---- */
SceneDef {
    id: "tq_20_huyuan_fight", bg: Some("tongqu_bg.png"), loc: Some("L2 · 镖局内院口"),
    mood: "danger", speaker: Some("护院"), voice: None,
    text: TextSpec::Static(&["沈镖头刚一点头，院门那杆红缨枪便横了过来——新任护院不解内情，误你为趁乱而入的夜行贼。"]),
    choices: &[ChoiceDef { label: "【过招】", sub: "tqf_huyuan 护院", cond: None, effects: &NO_EFF, route: Route::To("tq_20_huyuan_fight2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_20_huyuan_fight2", bg: Some("tongqu_bg.png"), loc: Some("L2 · 镖局内院口"),
    mood: "danger", speaker: None, voice: Some("vo_tq_huyuan"),
    text: TextSpec::Static(&["「睡多了，眼拙。」——护院一枪刺来。（战斗）"]),
    choices: &[], fight_id: Some("tqf_huyuan"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_20_huyuan_win", bg: Some("tongqu_bg.png"), loc: Some("L2 · 镖局内院口"),
    mood: "calm", speaker: Some("护院"), voice: None,
    text: TextSpec::Static(&["护院收枪抱拳：「失礼。既得沈镖头亲许，请进。」红缨枪撤下，内院密闸的门缝里透出沈镖头浑浊却笃定的目光。"]),
    choices: &[ChoiceDef { label: "踏进内院 · 获镖局信物", sub: "tq_biaoju_trust · 开 G1", cond: None,
        effects: &[Eff::SetFlag("tq_biaoju_trust"), Eff::AddItem("it_tq_biaoju_token")], route: Route::To("tq_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 黑店阴谋支线 ---- */
SceneDef {
    id: "tq_21_dark_shop", bg: Some("tongqu_bg.png"), loc: Some("L2 · 黑店暗门"),
    mood: "danger", speaker: Some("黑店掌柜"), voice: Some("vo_tq_dark"),
    text: TextSpec::Dyn(|st| {
        if st.flag("tq_heidian_truth") {
            "板缝里卷着几枚染血的铜钱——黑店的底，你已经看在眼里。".to_string()
        } else {
            "掌柜隔板探出半个头，眼珠滴溜：「客官夜深，小店打烊。」——可你分明听见密室深处传来搬箱的闷响与压低的号子声。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "推开后厨的隔板", sub: "识破黑店 · 打手现身", cond: None, effects: &NO_EFF, route: Route::To("tq_21_dark_fight") },
        ChoiceDef { label: "用追凶脚印质问", sub: "需雨后车辙线索", cond: Some(cond_has_trace), effects: &NO_EFF, route: Route::To("tq_21_aced_shock") },
        ChoiceDef { label: "退出黑店", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tq_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_21_aced_shock", bg: Some("tongqu_bg.png"), loc: Some("L2 · 黑店后厨"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你把雨后车辙的脚印指给他看——掌柜脸色一僵，后厨的杀猪刀「哐当」落地。黑店的底，被一句话凿穿。"]),
    choices: &[ChoiceDef { label: "逼问幕后", sub: "识破黑店 · 打手现身", cond: None,
        effects: &NO_EFF, route: Route::To("tq_21_dark_fight") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_21_dark_fight", bg: Some("tongqu_bg.png"), loc: Some("L2 · 黑店后厨"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["两条打手掀翻案板扑来！（战斗）"]),
    choices: &[], fight_id: Some("tqf_thug"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_21_dark_win", bg: Some("tongqu_bg.png"), loc: Some("L2 · 黑店后厨"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["打手瘫倒，掌柜被按在灶台前。他招了：昨夜押纲旁落、镖旗断旗，皆是「镇尾那头」主使——黑店只是洗货收脏的下脚。你已握黑店阴谋的核心：幕后是那位本该护镖的头领。"]),
    choices: &[ChoiceDef { label: "记下黑店供词", sub: "tq_heidian_truth · 指认幕后", cond: None,
        effects: &[Eff::SetFlag("tq_heidian_truth"), Eff::Points(100)], route: Route::Dyn(tq_after_dark) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 追凶（江湖恩怨）支线 ---- */
SceneDef {
    id: "tq_22_inn", bg: Some("tongqu_bg.png"), loc: Some("L2 · 客栈 · 追凶"),
    mood: "danger", speaker: Some("店小二"), voice: None,
    text: TextSpec::Static(&["客栈二楼也蹊跷——后檐那道人影脚程急，显然是在追着某件镖物离开。夜色里，他腰间甩出一角黑巾。"]),
    choices: &[
        ChoiceDef { label: "追到后檐", sub: "截蒙面探子", cond: None, effects: &NO_EFF, route: Route::To("tq_22_inn_fight") },
        ChoiceDef { label: "问店小二昨夜押纲", sub: "情报 · 供词", cond: Some(cond_has_dark), effects: &NO_EFF, route: Route::To("tq_22_inn_tip") },
        ChoiceDef { label: "退回市井", sub: "", cond: None, effects: &NO_EFF, route: Route::To("tq_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_22_inn_tip", bg: Some("tongqu_bg.png"), loc: Some("L2 · 客栈二楼"),
    mood: "cold", speaker: Some("店小二"), voice: None,
    text: TextSpec::Static(&["小二压低声音：「昨夜押纲确实过境——可苏记镖局只留守了一辆车，空车。另一车货，天亮前被人从黑店后门接走了。」追凶之路，直指镇尾。"]),
    choices: &[ChoiceDef { label: "合上黑店供词", sub: "追凶与黑店汇流", cond: None,
        effects: &[Eff::SetFlag("tq_trace_2"), Eff::Points(40)], route: Route::To("tq_22_self") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_22_inn_fight", bg: Some("tongqu_bg.png"), loc: Some("L2 · 客栈后檐"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["那道人影见你追近，拔刀便斩——正是黑店的蒙面探子！（战斗）"]),
    choices: &[], fight_id: Some("tqf_tuishun"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_22_inn_win", bg: Some("tongqu_bg.png"), loc: Some("L2 · 客栈后檐"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["探子被制，怀里的押纲清单散落一地——上面签着黑店的暗记，也牵出一句惊雷：「镇尾古宅，货在灯火尽处等人。」江湖恩怨终有归处：追凶直入古宅。"]),
    choices: &[ChoiceDef { label: "收起押纲清单", sub: "tq_trace_2 · 追凶入古宅", cond: None,
        effects: &[Eff::SetFlag("tq_trace_2"), Eff::Points(60)], route: Route::To("tq_22_self") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_22_self", bg: Some("tongqu_bg.png"), loc: Some("L2 · 市井 · 三条线索在攥手"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["押纲下落、黑店供词、沈镖头的信物在胸前凑齐。镖局护镖、追凶恩怨、黑店阴谋——三线同指一处：镇尾古宅，雪夜伏击，灯火尽处的蒙面头领。"]),
    choices: &[ChoiceDef { label: "循线索出镇尾", sub: "入 L3 镇尾古宅 · 决战", cond: None, effects: &NO_EFF, route: Route::To("tq_03") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 当铺柜台（护镖物证支线）---- */
SceneDef {
    id: "tq_23_dangpu2", bg: Some("tongqu_bg.png"), loc: Some("L2 · 当铺柜台"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["当铺柜台后方压着一张羊皮契——「押纲兑银，银由黑店后门结。」字迹与镖局旧暗号同出一手：内鬼不是外人，是押纲的见证人。"]),
    choices: &[ChoiceDef { label: "说要买回那匹染血绸帕", sub: "护镖物证 · +40 点", cond: None,
        effects: &[Eff::Points(40), Eff::SetFlag("tq_hubiao_proof")], route: Route::To("tq_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 镇尾古宅 ================= */
SceneDef {
    id: "tq_03", bg: Some("tongqu_bg.png"), loc: Some("L3 · 镇尾古宅"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "古宅的中堂里灯火尽处坐着一个人——雪夜落在他肩头，蒙面的黑巾抚平。押纲的箱子就搁在供桌下，箱盖半开，露出一点不该在镖局的寒光。",
        "（雪夜伏击的中心：供桌/密信/古宅残简可查；中堂是那蒙面人的擂台。）",
    ]),
    choices: &[
        ChoiceDef { label: "直面雪夜伏击", sub: "BOSS 决战 · 蒙面头领", cond: None, effects: &NO_EFF, route: Route::To("tq_boss_enter") },
        ChoiceDef { label: "古宅供桌", sub: "押纲的箱", cond: None, effects: &NO_EFF, route: Route::To("tq_30_altar") },
        ChoiceDef { label: "密信残页", sub: "内鬼书信", cond: None, effects: &NO_EFF, route: Route::To("tq_31_letter") },
        ChoiceDef { label: "古宅残简", sub: "旧镖局名册", cond: None, effects: &NO_EFF, route: Route::To("tq_32_scroll") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "tq_30_altar", bg: Some("tongqu_bg.png"), loc: Some("L3 · 古宅供桌"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["供桌上搁着押纲的箱。箱盖半开，露出一锭官银和一封火漆封印的书信——这不是寻常货，是让人动了杀心的「镖」。"]),
    choices: &[ChoiceDef { label: "读封在金漆下的密信", sub: "+5 点", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("tq_p_l3_1")], route: Route::To("tq_03") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_31_letter", bg: Some("tongqu_bg.png"), loc: Some("L3 · 密信残页"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["半张残页被雪水润开：「……镖既入我手，何须再守江湖道义？三日后雪夜，以蒙面收纲。」字迹与当铺羊皮契同源——幕后，就在镖局最信赖的人影里。"]),
    choices: &[ChoiceDef { label: "收密信残页作证", sub: "Item it_tq_letter_shred", cond: None,
        effects: &[Eff::AddItem("it_tq_letter_shred"), Eff::MarkPoint("tq_p_l3_2")], route: Route::To("tq_03") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_32_scroll", bg: Some("tongqu_bg.png"), loc: Some("L3 · 古宅残简"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["墙角压着一册烧得只剩半边的旧镖局名册——黑店掌柜，早年竟是振远镖局外押的「总把式」。江湖恩怨，原来早在十几年前就系在一根绳上。"]),
    choices: &[ChoiceDef { label: "收起旧名册", sub: "+5 点", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("tq_p_l3_3")], route: Route::To("tq_03") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- BOSS · 雪夜劫镖·蒙面头领（选择驱动）---- */
SceneDef {
    id: "tq_boss_enter", bg: Some("tongqu_bg.png"), loc: Some("L3 · 古宅中堂 · 决战"),
    mood: "danger", speaker: Some("蒙面头领"), voice: Some("vo_tq_boss"),
    text: TextSpec::Static(&[
        "「你瞧见了镖。」蒙面头领缓缓起身，雁翎刀出鞘一寸，「雪夜里，镖，运送的不只是货——是人心。」",
        "（雪夜滴水成刃，每回合你的气血会被寒气削去些许；识破黑店者，可在狂暴后「揭面指认」。）",
    ]),
    choices: &[ChoiceDef { label: "【逼近头领】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_boss_round", bg: Some("tongqu_bg.png"), loc: Some("L3 · 古宅中堂 · 激战"),
    mood: "danger", speaker: Some("蒙面头领"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("头领 HP {} / {}", f.hp.max(0), 180)).unwrap_or_else(|| "HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——雪夜滴水成刃，他狂暴了！攻势翻倍，每回合寒气削你 2 点气血；识破黑店者可「揭面指认」——"
        } else { "" };
        format!("{f}。{mode}（雪夜在场，每回合被寒气削蚀 2 点气血。）")
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 32-46", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, rng(32, 46), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 22-30", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, rng(22, 30), false)) },
        ChoiceDef { label: "【揭面指认】", sub: "识破黑店 + 狂暴 · 40 固伤并破其刀势", cond: Some(cond_has_dark),
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 40, false)) },
        ChoiceDef { label: "后撤观察", sub: "提升闪避", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_boss_win", bg: Some("tongqu_bg.png"), loc: Some("L3 · 古宅中堂"),
    mood: "calm", speaker: Some("蒙面头领"), voice: Some("vo_tq_boss_win"),
    text: TextSpec::Dyn(|st| {
        let mode = if st.flag("tq_heidian_truth") {
            "你揭下他的蒙面——正是守门护镖的旧总把式。他怔住，雁翎刀「哐当」坠地：「原是……你早看穿了。」"
        } else {
            "雁翎刀脱手飞出，蒙面头领跪伏在中堂雪里。压在他膝下的密信写明：这箱镖，本是他给自己攒的最后一趟「侠义与交易」的帐。"
        };
        format!("雪夜斗罢，蒙面头领的身形在灯火下寸寸矮下去。他嘴角牵着一点自嘲——「镖，终究运到了绝处。」\n\n{mode}\n\n供桌上那封印着漆的密信，正是你此行最沉的答案。")
    }),
    choices: &[ChoiceDef { label: "（走向供桌·收取密信）", sub: "护镖终局", cond: None, effects: &NO_EFF, route: Route::To("tq_33_exit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结局 / 结算 ================= */
SceneDef {
    id: "tq_33_exit", bg: Some("tongqu_bg.png"), loc: Some("L3 · 古宅 · 撤离阵"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "密信在手中沉甸甸。你踏出古宅，雪夜不知何时停了——主神的光柱自头顶降下，将这通衢古镇的一夜收进那片永恒的白光里。",
        "（结算：护镖侠义兑现，sp_grade=D。）",
    ]),
    choices: &[ChoiceDef { label: "（踏入撤离阵 · 结算）", sub: "sp_grade 结算 · 回主神空间", cond: None,
        effects: &NO_EFF, route: Route::Dyn(tq_exit_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "tq_34_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_tq_settle"), death: None,
        card: |st| crate::state::Card {
            title: "通 衢 古 镇 · 夜 雨 护 镖".into(), good: true,
            body_html: format!(
                "<p>雪夜伏击落幕，蒙面头领的密信在你掌中化为齑粉。你守住了一趟「镖」最重的东西——人心。</p>\
                 <p style='color:#9a958a'>护镖三线：追凶 {a} / 护镖 {b} / 识黑店 {c}。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{p}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 <tr><td>主神光柱</td><td>「镖路通衢，人心不渡。」</td></tr>\
                 </table>",
                a = if st.flag("tq_trace_2") { "已结" } else { "未结" },
                b = if st.flag("tq_biaoju_trust") { "已结" } else { "未结" },
                c = if st.flag("tq_heidian_truth") { "已结" } else { "未结" },
                p = st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案（复活扣 300/回主神）================= */
SceneDef {
    id: "tq_40_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("通衢夜雨下的无名者", "在古镇的雨巷与刀光里倒下")), card: |_st| crate::state::Card {
            title: "雨 巷 之 下".into(), good: false,
            body_html: r#"<p>通衢的夜雨把你淋透了，刀光过处，你倒在当铺屋檐漏下的水滴里。镖旗在风里猎猎，不知哪一处是终点。</p>
<p style='color:#ff8a8a'>【死亡档案 · 通衢夜雨下的无名者】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "tq_40_death_boss", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("雪夜之下", "蒙面头领的雁翎刀在雪光里贯穿了你")), card: |_st| crate::state::Card {
            title: "雪 夜 之 下".into(), good: false,
            body_html: r#"<p>雁翎刀在雪光里贯穿你的胸腹——蒙面头领收刀，洒了一地血迹与碎雪。「镖路到头，你也到头。」</p>
<p style='color:#ff8a8a'>【死亡档案 · 雪夜之下】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* =====================================================================
   Route::Dyn 路由函数（供 static 数组使用，fn 指针）
   ===================================================================== */
/// 护镖委托接受（需已掌握追凶或黑店任一佐证，才谈得上信服）：置 tq_biaoju_trust + 信物 → 过招护院。
fn tq_accept_biaoju(st: &mut GameState) -> String {
    st.set_flag("tq_biaoju_trust");
    crate::world::add_item(st, "it_tq_biaoju_token");
    "tq_20_huyuan_fight".to_string()
}

/// 黑店清理后回市井（已置 tq_heidian_truth，供后续「揭面指认」/追凶汇流）
fn tq_after_dark(_st: &mut GameState) -> String {
    "tq_02".to_string()
}

/// 撤离结算：确保 sp_grade=D 并由卡片收尾
fn tq_exit_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "tq_34_card".to_string()
}
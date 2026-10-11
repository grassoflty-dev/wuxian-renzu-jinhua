//! 《无限恐怖 · 命运清单 · 第二端》全部剧情场景与「规则流机关」配置（world slug: baisun，前缀 bs_）。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.1「死神来了」系列 + 第二端研究骨架。
//! 本文件为全新新增文件，只导出静态数据（BAISUN_SCENES / baisun_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/baisun_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `bs_` 前缀；fight id 全部 `bs_` 前缀（与既有无重名）。
//! 核心「规则流机关」——死神无实体，以「连环意外」收命，剧作靠预判/识破机关来改命：
//!   · 三条环境机关死亡线（停车场坠物/商场电梯夹/电影院火灾逃生），每线对应一个「死亡征兆」flag（bs_foresee_*）；
//!   · 识破征兆 → flag 改写命运，进入环境机关 zone 时免死并加点；
//!   · 未识破 → San 惩罚 + 命中即跳死亡档案「意外身故」（复活扣 300 回主神）；
//!   · 三征兆齐备 → bs_fate_rewritten（命运清单全革新，+150 结束加成 + sp_grade=D）；
//!   · 仅 L3 楼梯间 1 个「死神·使者」选择驱动象征战（HP150，可完全绕过，规则流主线零正面武力）。
//! 敌人立绘复用：guard→逃生梯巡查员（使者投影）；bg 占位待主线替换（见 §外部依赖/素材）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_all_foreseen(st: &GameState) -> bool {
    st.flag("bs_foresee_drop") && st.flag("bs_foresee_elev") && st.flag("bs_foresee_fire")
}
fn cond_fate_rewritten(st: &GameState) -> bool { st.flag("bs_fate_rewritten") }
fn cond_elev(st: &GameState) -> bool { st.flag("bs_foresee_elev") }
fn cond_fire(st: &GameState) -> bool { st.flag("bs_foresee_fire") }
fn cond_boss_down(st: &GameState) -> bool { st.flag("bs_boss_down") }

/* =====================================================================
   规则流 flag 链工具
   ===================================================================== */
/// 置一个征兆 flag；三条齐备则首次置 bs_fate_rewritten 并 +150 结算加成
fn foresaw(st: &mut GameState, which: &str) {
    if !st.flag(which) {
        st.set_flag(which);
        if cond_all_foreseen(st) && !st.flag("bs_fate_rewritten") {
            st.set_flag("bs_fate_rewritten");
            st.points += 150;
        }
    }
}

/* ---- 各征兆观测点的改动路由（返回所在楼 hub） ---- */
fn mark_drop_l1(st: &mut GameState) -> String { foresaw(st, "bs_foresee_drop"); "bs_l1_hub".to_string() }
fn mark_elev(st: &mut GameState) -> String { foresaw(st, "bs_foresee_elev"); "bs_l2_hub".to_string() }
fn mark_fire(st: &mut GameState) -> String { foresaw(st, "bs_foresee_fire"); "bs_l3_hub".to_string() }

/* ---- 环境机关 ZoneDef 触发：识破征兆 → 改命免死；否则死亡档案「意外身故」 ---- */
fn zone_drop(st: &mut GameState) -> String {
    if st.flag("bs_foresee_drop") {
        st.points += 40;
        st.set_flag("bs_sign_drop_dodged");
        foresaw(st, "bs_foresee_drop"); // 幂等：确保 fate 计数
        "bs_l1_hub".to_string()
    } else {
        "bs_50_death".to_string()
    }
}
fn zone_elev(st: &mut GameState) -> String {
    if st.flag("bs_foresee_elev") {
        st.points += 50;
        st.set_flag("bs_sign_elev_dodged");
        "bs_l2_hub".to_string()
    } else {
        "bs_50_death".to_string()
    }
}
fn zone_fire(st: &mut GameState) -> String {
    if st.flag("bs_foresee_fire") {
        st.points += 60;
        st.set_flag("bs_sign_fire_dodged");
        "bs_l3_hub".to_string()
    } else {
        "bs_50_death".to_string()
    }
}

/* ---- 结算 ---- */
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "bs_42_card".to_string()
}

/* ---- 使者象征战 rage 桩 ---- */
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   选择驱动 BOSS：死神·使者（HP150）
   黄金模板 C：start_boss / boss_act / boss_win
   ===================================================================== */
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("bs_boss") {
            st.fight = Some(crate::state::Fight {
                id: "bs_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "bs_boss_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    let hit = !guard; // guard 时免伤
    if hit { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "bs_50_death".to_string(); }
    "bs_boss_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("bs_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "bs_medallion");
    "bs_boss_win".to_string()
}

/* =====================================================================
   战斗配置表（id 全部 bs_ 前缀）——仅有 1 场：死神·使者选择驱动象征战
   ===================================================================== */
pub fn baisun_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("bs_boss", FightCfg {
            name: "死神·使者", hp: 150, dmg: (16, 24), reward: 400, reward_why: "破除死神使者的象征战",
            intro: "逃生梯的风彻骨，把每一级台阶吹得发亮。一个无面目的人形站在最下一阶，指尖悬着三枚「意外」——坠物、电梯、火灾——像在把玩三枚骰子。它把你的名字在名单上划了一遍，却不急着收：它有的是时间，而你只有这一次。",
            rage_at: Some(60), rage_text: "使者的身影碎裂成一片倒计时的数字，贴上你的眼，数字一格一格归零——它要替你「复演」停车场那一秒坠物，让你亲眼看着自己的名字被收走！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| String::new(),
            finisher_desc: |_st| String::new(),
            win: |_st| "bs_boss_win".to_string(),
            death: "bs_50_death",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn bs_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    baisun_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 bs_ 前缀）
   ===================================================================== */
pub static BAISUN_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "bs_00", bg: Some("bs_bg_airport.png"), loc: Some("医院停车场 ≪第二端 ≫"),
    mood: "mystery", speaker: Some("主神·任务发布"), voice: Some("vo_baisun_1"),
    text: TextSpec::Static(&[
        "<b>【主线任务·命运清单·第二端】</b>识破连环意外，改写死神给三桩「机关」排下的死亡名单。失败代价：被扣 300 点复活。",
        "<i>「死神从不现身，只在你放下警惕的下一格台阶。」</i>车灯在你抬手的一瞬灭了整排，地面忽然静得能听见自己的心跳。吊机的摇臂停在半空，像在等你的错觉真正发生——这座停车场，正按一份没人见过的名单，一笔一笔地收着人命。",
    ]),
    choices: &[
        ChoiceDef { label: "环顾停车场", sub: "+5 点 · 记住这里的「异常」", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("bs_l1_hub") },
        ChoiceDef { label: "细听头顶吊机", sub: "San-2 · 确认「死神在你身边」", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("bs_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 医院停车场 hub ================= */
SceneDef {
    id: "bs_l1_hub", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 医院停车场"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bs_foresee_drop") {
            "吊机摇臂那截松动，你已记下了。头顶的铁箱不再往下压——死亡名单上「坠物」那一行，被你划掉了。".to_string()
        } else {
            "车灯忽明忽暗，吊机悬在半空的铁箱锈得发脆，锈屑簌簌往下落。地上拖着一行湿滑警戒带，像一条刚被谁匆忙划出的边界。空气里那股烧焦的胶皮味挥之不去——总有什么「不该在这」，而它正在等你靠近。（吊机摇臂 / 警戒带 / 焦黑轿车 / 悬空铁箱）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "吊机摇臂", sub: "坠物征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_01_crane") },
        ChoiceDef { label: "湿滑警戒带", sub: "坠物征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_01_slip") },
        ChoiceDef { label: "焦黑轿车", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bs_01_car") },
        ChoiceDef { label: "悬空铁箱", sub: "坠物征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_01_box") },
        ChoiceDef { label: "与停车场保安交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("bs_02_guard") },
        ChoiceDef { label: "逼近吊机下方", sub: "坠物机关区", cond: None, effects: &NO_EFF, route: Route::To("bs_10_drop") },
        ChoiceDef { label: "走向出口升降梯", sub: "→ 地图传送 L2", cond: None, effects: &NO_EFF, route: Route::To("bs_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 征兆观测点 ---- */
SceneDef {
    id: "bs_01_crane", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 吊机摇臂"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["吊机的摇臂基座缺了半颗螺栓，风一过就往下沉一格，锈红的断面在灯光下一明一灭。你在铁锈的缝隙里读出一句无声的警告：这台机器不是在老化，而是在「等」——等一个正好站到正下方的人。这里的「坠物」正要发生。"]),
    choices: &[ChoiceDef { label: "【识破坠物征兆】", sub: "bs_foresee_drop · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l1_crane")], route: Route::Dyn(mark_drop_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_01_slip", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 湿滑警戒带"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["警戒带被谁撕开一角，露出的地面汪着一片水，水面上浮着一层薄薄的油光。你低头，水面正好倒映着吊机铁箱的影子，像一只悬在头顶的眼睛——摇臂一动，这儿就是落点。它已经替你算好了距离。"]),
    choices: &[ChoiceDef { label: "【识破坠物征兆】", sub: "bs_foresee_drop · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l1_slip")], route: Route::Dyn(mark_drop_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_01_car", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 焦黑轿车"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["一辆烧黑的轿车横在车位线外，引擎盖上砸出一个凹坑，凹坑中央还嵌着半截断裂的钢缆。钢梁像纸一样皱，车门从里面被顶开过一次——这辆车，是上一次「意外」留下的。而这次的坠物，正悬在你们头顶。"]),
    choices: &[ChoiceDef { label: "记下焦黑轿车", sub: "伏笔 · San+1", cond: None,
        effects: &[Eff::San(1), Eff::MarkPoint("bs_p_l1_car")], route: Route::To("bs_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_01_box", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 悬空铁箱"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["头顶的铁箱只有一根钢缆吊着，绳股断了快一半，断口处泛着新磨的金属白。你盯着箱底的划痕，认出同一种「坠物」的预兆——它正攒着劲，随着每一阵风微微晃动，等你站到正下方，好让这一场意外显得「合情合理」。"]),
    choices: &[ChoiceDef { label: "【识破坠物征兆】", sub: "bs_foresee_drop · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l1_box")], route: Route::Dyn(mark_drop_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_02_guard", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 值班室"),
    mood: "cold", speaker: Some("停车场保安"), voice: None,
    text: TextSpec::Static(&["保安攥着对讲机，声音发干，手背上青筋绷着：「那台吊机……我今早就觉着不对，报了两回检修，可没人听。别往铁箱底下站，听见没？上回也是这么——没人碰它，没人动它，它自己就下来了。」他咽了口唾沫，压低声音：「这地方，好像有东西在按名单办事。」"]),
    choices: &[ChoiceDef { label: "「那你替我盯着它。」", sub: "保安点头 · San+2", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bs_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 环境机关 · 坠物 ================= */
SceneDef {
    id: "bs_10_drop", bg: Some("bs_bg_highway.png"), loc: Some("L1 · 吊机坠物区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["你走到吊机正下方。头顶的钢缆发出一声脆响，锈屑迸溅，铁箱猛地一沉，开始往下坠——这零点几秒里，你只有一次识破它、或逃开它的机会。名单正在你头顶落下。"]),
    choices: &[ChoiceDef { label: "【逼近落点扫描征兆】", sub: "识破坠物征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_drop) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 室内商场合订结构 hub ================= */
SceneDef {
    id: "bs_l2_hub", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 室内商场合订结构"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bs_foresee_elev") {
            "停用轿厢那道会咬人的门缝，你已记下了。扶梯还在往上送人，可你知道——名单上「电梯夹」那一行，被你划掉了。".to_string()
        } else {
            "商场中庭的扶梯一层叠一层，像凝固的银色瀑布，一侧停着一部锈住的货梯，铁门半掩，门缝里漏出一线冷光。空气里带着一股久未通电的焦味，混着说不清的金属腥气——这里没有人，却处处像刚有人来过。（扶梯扶手 / 停用电梯轿厢 / 超载报警灯 / 商场风道）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "扶梯扶手", sub: "电梯征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_04_handrail") },
        ChoiceDef { label: "停用电梯轿厢", sub: "电梯征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_04_cabinet") },
        ChoiceDef { label: "超载报警灯", sub: "电梯征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_04_sign") },
        ChoiceDef { label: "商场风道", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bs_04_duct") },
        ChoiceDef { label: "与商场客服交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("bs_05_clerk") },
        ChoiceDef { label: "探进货梯轿厢", sub: "电梯机关区", cond: None, effects: &NO_EFF, route: Route::To("bs_11_elev") },
        ChoiceDef { label: "走向货运电梯", sub: "需识破电梯征兆 → L3", cond: None, effects: &NO_EFF, route: Route::To("bs_06_gate2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L2 征兆观测点 ---- */
SceneDef {
    id: "bs_04_handrail", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 扶梯扶手"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["扶梯的橡胶扶手被一件东西来回绞过，表面留下两条平行的压痕，深得能嵌进指甲。扶梯还在缓缓上行，仿佛什么都没发生过——你盯着那压痕，读懂了一句：这里的「电梯夹」正要发生，而它已经替下一位乘客预演过一次。"]),
    choices: &[ChoiceDef { label: "【识破电梯征兆】", sub: "bs_foresee_elev · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l2_handrail")], route: Route::Dyn(mark_elev) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_04_cabinet", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 停用电梯轿厢"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["货轿的铁门开着一道缝，门缝里露出一截被切断的防坠链，断口齐整得不像锈断的。你凑近，听见轿厢深处有极轻的金属摩擦声，像牙关在慢慢磨——它不是在坏，是在等一个身体探进来，好把门合上。"]),
    choices: &[ChoiceDef { label: "【识破电梯征兆】", sub: "bs_foresee_elev · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l2_cabinet")], route: Route::Dyn(mark_elev) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_04_sign", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 超载报警灯"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["货梯的超载报警灯半灭半明，灯罩上积着灰，底下贴着一张「停用」褪色告示，边角被撕掉一块。你盯着那盏灯，读出它的意思：每次快夹到人时，它都会先闪三下——像在倒数，也像在提醒。现在它正一下一下地闪。"]),
    choices: &[ChoiceDef { label: "【识破电梯征兆】", sub: "bs_foresee_elev · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l2_sign")], route: Route::Dyn(mark_elev) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_04_duct", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 商场风道"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["风道出风口积着厚灰，被某次巨响震开一道缝，露出里面锈死的检修门，门把上缠着一截消防绳。你把它记下了——也许这是另一条出去的路，也许，是别人逃命时留下的记号。"]),
    choices: &[ChoiceDef { label: "记下风道检修门", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l2_duct")], route: Route::To("bs_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_05_clerk", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 客服台"),
    mood: "cold", speaker: Some("商场客服"), voice: None,
    text: TextSpec::Static(&["客服递给你一杯热水，手指在杯沿上直打颤，声音压低：「那部货梯早该报废了，报修单堆了一摞没人来。昨儿夜里有人听见它自己「咔哒」一声，像合上牙似的——就一声，再没动静。别往门缝里凑，听见没？」她顿了顿，又补了一句：「三楼还压着个清洁工，别去招惹。」"]),
    choices: &[ChoiceDef { label: "「它合上牙那次，夹住谁了？」", sub: "客服一抖 · San+2", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bs_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_06_gate2", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 货运电梯轿厢门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bs_foresee_elev") {
            "你已识破电梯夹缝的征兆，避开那道会咬人的门缝。货梯「咣」地停稳，震荡在整面墙上传开，铁丝网门朝你敞开一条直上通道——你踏进轿厢，身后那两扇铁门缓缓合拢，像失落了一次收网。你走向那里的逃生梯。".to_string()
        } else {
            "锈住的货梯门缝里漏出被绞断的防坠链，链头的断口泛着新光。你还没有识破「电梯夹」的征兆——这两扇门正在门缝里耐心地等，硬闯，只会被它收走。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "通过轿厢（上L3）", sub: "需 bs_foresee_elev · →L3", cond: Some(cond_elev),
            effects: &NO_EFF, route: Route::To("bs_l3_hub") },
        ChoiceDef { label: "回到商场中庭", sub: "", cond: None, effects: &NO_EFF, route: Route::To("bs_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 环境机关 · 电梯夹 ================= */
SceneDef {
    id: "bs_11_elev", bg: Some("bs_bg_mall.png"), loc: Some("L2 · 电梯轿厢夹缝"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["你探进货轿的一瞬，铁门「咔哒」一声往前滑，门缝里的冷风贴着你的手背收拢。那道门缝正在合拢，像一张慢慢咬合的嘴——你只有一次识破它、或让它咬住你的机会。"]),
    choices: &[ChoiceDef { label: "【探入夹缝扫描征兆】", sub: "识破电梯征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_elev) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 电影院逃生梯 hub ================= */
SceneDef {
    id: "bs_l3_hub", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 电影院逃生梯"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bs_fate_rewritten") {
            "三桩意外——坠物、电梯夹、火灾——都被你逐行划掉，死亡名单上再无你的名字。逃生梯里的风突然静止，像一双眼睛在辨认，又猛地灌回来：它认得你，命运已被改写。这座电影院的火，烧不到一个不在名单上的人。".to_string()
        } else {
            "这是最后一个连环，也收得最紧。放映室的胶卷在冒热味，灭火器柜是空的，洒水喷头的阀门被人拧死，逃生指示牌忽明忽暗——整条逃生路线，被一件件「刚好」提前拆掉。暗处有风，风里有烧焦的纸味。（放映室 / 灭火器柜 / 洒水喷头 / 指示牌 / 清洁工 / 逃生梯）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "放映室胶卷", sub: "火灾征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_07_projector") },
        ChoiceDef { label: "灭火器柜", sub: "火灾征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_07_extinguisher") },
        ChoiceDef { label: "洒水喷头", sub: "火灾征兆", cond: None, effects: &NO_EFF, route: Route::To("bs_07_sprinkler") },
        ChoiceDef { label: "逃生指示牌", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("bs_07_sign") },
        ChoiceDef { label: "与电影院清洁工交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("bs_08_janitor") },
        ChoiceDef { label: "逼近失火段", sub: "火灾机关区", cond: None, effects: &NO_EFF, route: Route::To("bs_12_fire") },
        ChoiceDef { label: "逃生梯楼梯间 · 使者", sub: "象征战（可绕）", cond: None, effects: &NO_EFF, route: Route::To("bs_garage") },
        ChoiceDef { label: "对照命运清单", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("bs_settle") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L3 征兆观测点 ---- */
SceneDef {
    id: "bs_07_projector", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 放映室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["放映室没人，座椅空空荡荡，胶片盘却越转越快，滚轴渗出刺鼻的热味，银幕上投着一场没人看的雪。你认出那股预热——「火灾」正在这里积蓄，它不需要点火的人，只需要一个观众到齐的时刻。"]),
    choices: &[ChoiceDef { label: "【识破火灾征兆】", sub: "bs_foresee_fire · 改命", cond: None,
        effects: &[Eff::San(-3), Eff::MarkPoint("bs_p_l3_projector")], route: Route::Dyn(mark_fire) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_07_extinguisher", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 灭火器柜"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["灭火器柜是空的，柜门大敞，墙上残留一道新擦的划痕，像有人为「逃跑」提前搬空了消火器。柜门内侧贴着手写的四字：火，要来了。字迹墨很新，像是写完没多久。"]),
    choices: &[ChoiceDef { label: "【识破火灾征兆】", sub: "bs_foresee_fire · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l3_extinguisher")], route: Route::Dyn(mark_fire) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_07_sprinkler", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 洒水喷头"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["天花板的洒水喷头，玻璃泡被人用手指碾碎，阀门被人拧死，拧痕上还留着指腹的余温。这条无论如何都浇不下来的走廊，正是起火后唯一的出口——火一起，这里就是一条烧红的死巷。"]),
    choices: &[ChoiceDef { label: "【识破火灾征兆】", sub: "bs_foresee_fire · 改命", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l3_sprinkler")], route: Route::Dyn(mark_fire) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_07_sign", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 逃生指示牌"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["逃生指示牌的绿光忽明忽暗，像一盏快要没电的灯，把走廊照得明明灭灭。灯罩后塞着一张烧掉一半的影票，票根上的场次正是今天——你总觉得，这才是这一整层唯一的「真出口」，而它已经被提前标好了。"]),
    choices: &[ChoiceDef { label: "记下指示牌", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("bs_p_l3_sign")], route: Route::To("bs_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_08_janitor", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 走廊拐角"),
    mood: "cold", speaker: Some("电影院清洁工"), voice: None,
    text: TextSpec::Static(&["清洁工手里攥着一卷被火燎黑的胶片，指缝里全是灰，声音发抖：「上边……放映室没人了，可胶卷还在转，转了整整一夜。这层的灭火器昨晚全被人搬空了，你知道那是谁干的吗？」他顿了顿，望着楼梯口：「我总觉着，有人在替这栋楼，安排一场没人想看的电影。」"]),
    choices: &[ChoiceDef { label: "「你在躲什么？」", sub: "清洁工回头看了一眼楼梯 · San+2", cond: None,
        effects: &[Eff::San(2)], route: Route::To("bs_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 环境机关 · 火灾逃生 ================= */
SceneDef {
    id: "bs_12_fire", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 逃生梯失火段"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["走廊尽头的逃生梯口，一道火舌翻卷着堵死退路，浓烟贴着天花板压下来，把指示牌的绿光熏成暗红。防烟门在高温里发红，门把烫得变形——你只有一次识破它、或冲进火里的机会，而火，已经替你把退路数了一遍。"]),
    choices: &[ChoiceDef { label: "【逼近火段扫描征兆】", sub: "识破火灾征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_fire) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 · 死神·使者（选择驱动象征战入口） ================= */
SceneDef {
    id: "bs_garage", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 逃生梯楼梯间"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["楼梯间最下一阶，无面目的人形悬在正中，周身的光线像被它吸走一寸。它指尖悬着三枚「意外」——坠物、电梯夹、火灾——绕成一个小小的漩涡，每一枚都映着你曾走过的路。它没动，但整条楼梯的风都朝它倾斜，像在替它让路。"]),
    choices: &[
        ChoiceDef { label: "出手破使者", sub: "象征战 bs_boss（HP150）", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
        ChoiceDef { label: "转身逃生（绕过）", sub: "规则流 · 不战", cond: None, effects: &NO_EFF, route: Route::To("bs_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_boss_round", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 逃生梯楼梯间 · 决战"), mood: "danger",
    speaker: Some("死神·使者"), voice: None,
    text: TextSpec::Dyn(|st| format!("使者剩余 {} 血，你 HP {}。它把名单在风里抖开，等你亲手划掉最后一行。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "重击三枚意外", sub: "高伤害", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
        ChoiceDef { label: "侧身躲过化身", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_boss_win", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 逃生梯楼梯间"),
    mood: "calm", speaker: Some("死神·使者"), voice: None,
    text: TextSpec::Static(&["使者化作一片倒计时数字，散进风里，数字一粒粒熄灭，像被谁亲手按停。三枚「意外」依次落回你掌心，被你逐个划掉——它把「名单」还给你，你亲手写下自己的名字，又亲手划去。从此，没有哪一桩机关能再选上你。"]),
    choices: &[ChoiceDef { label: "对照命运清单", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("bs_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 · 防火门（G3 软锁提示场景） ================= */
SceneDef {
    id: "bs_09_gate3", bg: Some("bs_bg_cinema.png"), loc: Some("L3 · 逃生梯防火门"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bs_foresee_fire") {
            "你已识破影院火灾的征兆，绕开冒烟的走廊。防火门背后，消防通道的冷风让你活了下来——你走向那扇门后的世界。".to_string()
        } else {
            "防火门被高温烤得发红，门把冒着白烟。你还没有识破「火灾」的征兆——硬推，只会把自己送进去。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "推开防火门", sub: "需 bs_foresee_fire", cond: Some(cond_fire),
            effects: &NO_EFF, route: Route::To("bs_settle") },
        ChoiceDef { label: "回到逃生梯", sub: "", cond: None, effects: &NO_EFF, route: Route::To("bs_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结算 ================= */
SceneDef {
    id: "bs_settle", bg: Some("bs_bg_cinema.png"), loc: Some("命运清单 · 第二端 · 结算"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("bs_fate_rewritten") {
            "你把三桩机关的征兆一一划掉，连死神·使者的象征战都被你踩在名单之上。命运清单·第二端，无可再收——你活着走到这里，本身就已是名单上最刺眼的一笔。".to_string()
        } else if st.flag("bs_boss_down") {
            "你破了使者，却还有一两桩征兆没来得及读。但你仍带着那些「改过的名字」走到这里。主线判定你的支线结算评级。".to_string()
        } else {
            "还有些征兆你没来得及读，使者则被你远远绕开。你仍活着，走到这扇防火门前。主线判定你的支线结算评级。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【名单之路（全征兆）】", sub: "命运已改写 · 结算加成", cond: Some(cond_fate_rewritten),
            effects: &NO_EFF, route: Route::Dyn(route_settle) },
        ChoiceDef { label: "【破使者之路】", sub: "破了象征战 · 结算", cond: Some(cond_boss_down),
            effects: &NO_EFF, route: Route::Dyn(route_settle) },
        ChoiceDef { label: "确认走向撤离光柱", sub: "结算 · 评级", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "bs_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "谁 也 收 不 了 你".into(), good: true,
            body_html: format!(
                "<p>你走出逃生梯，身后三桩「机关」都化作烟尘，没有一桩带走你。名单上的名字被风卷走，而你的名字，还在自己手里。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>命运清单</td><td style='color:#9a958a'>坠物{} 电梯{} 火灾{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级{}</td></tr>\
                 </table>",
                st.points,
                if st.flag("bs_foresee_drop") { "✓" } else { "✗" },
                if st.flag("bs_foresee_elev") { "✓" } else { "✗" },
                if st.flag("bs_foresee_fire") { "✓" } else { "✗" },
                if st.flag("bs_boss_down") { " · 使者已破" } else { "" },
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案 · 意外身故（复活扣 300 / 回主神） ================= */
SceneDef {
    id: "bs_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("三桩机关之外的收笔", "未识破连环意外征兆，被一桩「机关」意外收走")), card: |_st| crate::state::Card {
            title: "机 关 之 外".into(), good: false,
            body_html: r#"<p>你漏过的那桩「意外」终于追上你——无论是坠落、电梯夹还是火灾，它都来得那么「合理」：恰好松脱的螺栓，恰好关上的门，恰好燃起的火。死神从不现身，它只是替你递上了预定的那份请柬。</p>
<p style='color:#ff8a8a'>【死亡档案 · 意外身故】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
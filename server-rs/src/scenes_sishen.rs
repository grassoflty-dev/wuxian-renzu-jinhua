//! 《无限恐怖 · 死神来了》全部剧情场景与「命运清单」规则流配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.1「死神来了」骨架。
//! 本文件为全新新增文件，只导出静态数据（SISHEN_SCENES / sishen_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/sishen_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `ss_` 前缀；fight id 全部 `ss_` 前缀（与既有无重名）。
//! 核心「规则流」——死神无实体，以「意外死亡」连环收命：
//!   · 三条环境机关死亡线（坠落/爆炸/触电），每线对应一个「死亡征兆」 flag（ss_foresee_*）；
//!   · 预判征兆 → flag 改写命运，进入环境机关 zone 时免死并加点；
//!   · 未预判 → San 惩罚 + 命中所及即跳死亡档案「意外身故」（复活扣 300 回主神）；
//!   · 三条征兆齐备 → ss_fate_rewritten（命运清单全革新，+150 结束加成 + sp_grade=D）。
//!   · 仅 L3 车库 1 个「死神·使者」象征战（可完全绕过，规则流主线零战斗）。
//! 敌人立绘复用：guard→搬家的执法者（使者投影）；bg 占位待主线替换（见 §3）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_pass(_st: &GameState) -> bool { false } // 占位占位，未用（G1 门禁走 GateDef need_item）
fn cond_all_foreseen(st: &GameState) -> bool {
    st.flag("ss_foresee_fall") && st.flag("ss_foresee_explosion") && st.flag("ss_foresee_shock")
}
fn cond_explosion(st: &GameState) -> bool { st.flag("ss_foresee_explosion") }

/* =====================================================================
   命运清单 flag 链工具
   ===================================================================== */
/// 置一个征兆 flag；三条齐备则首次置 ss_fate_rewritten 并 +150 结算加成
fn foresaw(st: &mut GameState, which: &str) {
    if !st.flag(which) {
        st.set_flag(which);
        if cond_all_foreseen(st) && !st.flag("ss_fate_rewritten") {
            st.set_flag("ss_fate_rewritten");
            st.points += 150;
        }
    }
}

/* ---- 各征兆观测点的改动路由（返回所在楼 hub） ---- */
fn mark_fall_l1(st: &mut GameState) -> String { foresaw(st, "ss_foresee_fall"); "ss_01_l1_hub".to_string() }
fn mark_fall_l3(st: &mut GameState) -> String { foresaw(st, "ss_foresee_fall"); "ss_04_l3_hub".to_string() }
fn mark_explosion(st: &mut GameState) -> String { foresaw(st, "ss_foresee_explosion"); "ss_03_l2_hub".to_string() }
fn mark_shock(st: &mut GameState) -> String { foresaw(st, "ss_foresee_shock"); "ss_04_l3_hub".to_string() }

/* ---- 环境机关 ZoneDef 触发：预判征兆 → 改命免死；否则死亡档案「意外身故」 ---- */
fn zone_fall(st: &mut GameState) -> String {
    if st.flag("ss_foresee_fall") {
        st.points += 40;
        st.set_flag("ss_sign_fall_dodged");
        foresaw(st, "ss_foresee_fall"); // 幂等：确保 fate 计数
        "ss_01_l1_hub".to_string()
    } else {
        "ss_40_death_fall".to_string()
    }
}
fn zone_boom(st: &mut GameState) -> String {
    if st.flag("ss_foresee_explosion") {
        st.points += 50;
        st.set_flag("ss_sign_boom_dodged");
        "ss_03_l2_hub".to_string()
    } else {
        "ss_41_death_boom".to_string()
    }
}
fn zone_shock(st: &mut GameState) -> String {
    if st.flag("ss_foresee_shock") {
        st.points += 60;
        st.set_flag("ss_sign_shock_dodged");
        "ss_04_l3_hub".to_string()
    } else {
        "ss_42_death_shock".to_string()
    }
}
fn zone_stair(st: &mut GameState) -> String {
    if st.flag("ss_foresee_fall") {
        st.points += 30;
        st.set_flag("ss_sign_stair_dodged");
        "ss_04_l3_hub".to_string()
    } else {
        "ss_43_death_stair".to_string()
    }
}

/* ---- 结算 ---- */
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "ss_21_card".to_string()
}

/* ---- 使者象征战 win / rage 桩 ---- */
fn emissary_rage(_st: &mut GameState, _log: &mut Vec<String>) {}
fn emissary_win(_st: &GameState) -> String { "ss_09_emissary_win".to_string() }
fn fins_name(_st: &GameState) -> String { String::new() }
fn fins_desc(_st: &GameState) -> String { String::new() }
fn fins_if(_st: &GameState, _hp: i32) -> bool { false }

/* =====================================================================
   战斗配置表（id 全部 ss_ 前缀）——仅有 1 场：死神·使者象征战
   ===================================================================== */
pub fn sishen_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("ss_emissary", FightCfg {
            name: "死神·使者", hp: 150, dmg: (16, 24), reward: 400, reward_why: "破除死神使者的象征战",
            intro: "一个无面目的人形悬在车库阴影里——它不攻击，只在你每次侧身时更近一分。它替你「临演」了所有你没避开的意外。",
            rage_at: Some(60), rage_text: "使者身形碎裂成一片倒计时的数字，贴上你的眼——它要「替你」重演第一件意外！",
            on_rage: emissary_rage,
            finisher_if: fins_if, finisher_name: fins_name, finisher_desc: fins_desc,
            win: emissary_win, death: "ss_44_death_enforcer",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn ss_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    sishen_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 ss_ 前缀）
   ===================================================================== */
pub static SISHEN_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "ss_00", bg: Some("sishen_bg.png"), loc: Some("机场候机大厅 ≪警报到计时≫"),
    mood: "mystery", speaker: Some("主神·任务发布"), voice: Some("vo_sishen_1"),
    text: TextSpec::Static(&[
        "<b>【主线任务·命运清单】</b>预判「死亡征兆」，改写死神给你排下的死亡名单。失败代价：被扣 300 点复活。",
        "<i>「死神从不露面，只在你放松的下一秒。」</i>广播在你落座那一刻彻底静默了。候机屏上，你「未来三小时」的航班号，正一个接一个地亮成红色。",
    ]),
    choices: &[
        ChoiceDef { label: "环顾候机大厅", sub: "+5 点 · 记住这里的「异常」", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("ss_01_l1_hub") },
        ChoiceDef { label: "细听广播静默", sub: "San-2 · 确认「死神在你身边」", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("ss_01_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 机场候机大厅 hub ================= */
SceneDef {
    id: "ss_01_l1_hub", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 机场候机大厅"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ss_foresee_fall") {
            "扶梯口金属护栏那截锈痕，你已记下了。广播仍静默着，但你知道——死亡名单上「坠落」那一行，被你划掉了。".to_string()
        } else {
            "值机屏的航班在闪，候机座椅排空无一人，扶梯口那截金属护栏在震动里泛着锈光。总有什么「不该在这」。（值机台 / 座椅 / 扶梯口 / 落地玻璃 / 金属通道）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "值机屏 · 航班牌", sub: "预判线索", cond: None, effects: &NO_EFF, route: Route::To("ss_02_board") },
        ChoiceDef { label: "候机座椅排", sub: "检查登机牌", cond: None, effects: &NO_EFF, route: Route::To("ss_02_sit") },
        ChoiceDef { label: "自动扶梯口", sub: "坠落征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_02_escalator") },
        ChoiceDef { label: "落地玻璃幕", sub: "坠落征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_02_glass") },
        ChoiceDef { label: "金属通道缝隙", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("ss_02_metal") },
        ChoiceDef { label: "与值班广播员交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("ss_05_announcer") },
        ChoiceDef { label: "走向登机安检口", sub: "需登机牌 → 下高速", cond: None, effects: &NO_EFF, route: Route::To("ss_02_gate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 征兆观测点 ---- */
SceneDef {
    id: "ss_02_board", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 值机屏 · 航班牌"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["值机屏上，你的航班排在最后一行，后面是一段「无航班」的红字。广播想说点什么，在第一个音节处断了。"]),
    choices: &[ChoiceDef { label: "拍下航班牌", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l1_board")], route: Route::To("ss_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_02_sit", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 候机座椅排"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["一张座位空得反常——椅垫下压着一枚登机牌，名字正是你这个航班。牌角缺了一块，像被死神咬过。"]),
    choices: &[ChoiceDef { label: "取走登机牌", sub: "Item it_boarding_pass · 开 G1", cond: None,
        effects: &[Eff::AddItem("it_boarding_pass"), Eff::MarkPoint("ss_p_l1_sit")], route: Route::To("ss_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_02_escalator", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 自动扶梯口"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["扶梯口的金属护栏松了一截，被什么来回摇过。你在齿痕里读懂了一句无声的警告：这里的「坠落」正要发生。"]),
    choices: &[ChoiceDef { label: "【识破坠落征兆】", sub: "ss_foresee_fall · 改命", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l1_nexus")], route: Route::Dyn(mark_fall_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_02_glass", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 落地玻璃幕"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["整面落地玻璃的里层裂着一道蛛网纹。透过纹路，外面的车道亮得像下一秒就要发生什么。你认出同一种「坠落」的预兆。"]),
    choices: &[ChoiceDef { label: "【识破坠落征兆】", sub: "ss_foresee_fall · 改命", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l1_glass")], route: Route::Dyn(mark_fall_l1) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_02_metal", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 金属通道缝隙"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["登机通道的金属地板撬起一角，露出下面湿漉漉的配电井。水珠顺着电线往上爬。你把它记下了。"]),
    choices: &[ChoiceDef { label: "记下金属缝隙", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l1_metal")], route: Route::To("ss_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_02_gate", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 登机安检口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ss_foresee_fall") {
            "你站到安检口，登机牌在手。广播终于挤出半个字音——然后一片死寂。护栏的锈响在身后追了你一路。你走出闸口，踏上通往高速的匝道。".to_string()
        } else {
            "安检门红灯闪烁。你还没有登机牌，也还没能读出喻示「坠落」的征兆。要走出去，就得闯过那扇红门。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "出示登机牌通过（去高速）", sub: "需 it_boarding_pass · →L2", cond: Some(cond_has_pass),
            effects: &NO_EFF, route: Route::To("ss_03_l2_hub") },
        ChoiceDef { label: "回到候机大厅", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ss_01_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_05_announcer", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 广播值班室"),
    mood: "cold", speaker: Some("值班广播员"), voice: None,
    text: TextSpec::Static(&["广播员盯着只剩红字的屏幕，声音发紧：「航班……一个又一个的航班，都停了。你看那个扶梯口——有人正朝护栏那头去。」"]),
    choices: &[ChoiceDef { label: "「是谁要去？帮我盯住他们。」", sub: "广播员点头 · San+2", cond: None,
        effects: &[Eff::San(2)], route: Route::To("ss_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 环境机关 · 坠落 ================= */
SceneDef {
    id: "ss_10_death_fall", bg: Some("sishen_bg_open.png"), loc: Some("L1 · 扶梯金属护栏"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["你不该靠近这截松动的护栏——可它偏要你靠近。金属在脚下发出令人牙酸的撕扯声，自动扶梯开始朝下加速。"]),
    choices: &[ChoiceDef { label: "【逼近护栏】", sub: "预判坠落征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_fall) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 高速公路 hub ================= */
SceneDef {
    id: "ss_03_l2_hub", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 明州高速公路"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ss_foresee_explosion") {
            "油罐车侧翻的残骸还在冒烟，但你绕开了它的「爆点」。巡警朝你摆手——他在匝道护栏缺口等你。".to_string()
        } else {
            "车流在浓雾里断开。跨线天桥下，一辆油罐车侧翻，空气里全是汽油味；路面中央的轮胎印刹车痕绕成了一个圈。（天桥 / 油罐车 / 轮胎印 / 撞车现场）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "跨线天桥", sub: "征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_03_overpass") },
        ChoiceDef { label: "油罐车残骸", sub: "爆炸征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_03_truck") },
        ChoiceDef { label: "路中央轮胎印", sub: "征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_03_tire") },
        ChoiceDef { label: "连环撞车现场", sub: "征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_03_car") },
        ChoiceDef { label: "与公路巡警交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("ss_06_trooper") },
        ChoiceDef { label: "走匝道护栏缺口", sub: "需识破爆炸征兆 → 住宅", cond: None, effects: &NO_EFF, route: Route::To("ss_03_gap") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_03_overpass", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 跨线天桥"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["天桥在风里晃，护栏上一片片青苔被刮出新茬。底下的车流安静得反常——这一切都在等着谁走上去。"]),
    choices: &[ChoiceDef { label: "记下天桥晃动", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l2_over")], route: Route::To("ss_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_03_truck", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 油罐车侧翻点"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["油罐车翻在应急车道，焊缝处渗出熟悉的汽油味，滴到路面上汇成一行向下的水迹。你读出那行字：'爆炸，是给你的。'"]),
    choices: &[ChoiceDef { label: "【识破爆炸征兆】", sub: "ss_foresee_explosion · 改命 · 开G2", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l2_truck")], route: Route::Dyn(mark_explosion) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_03_tire", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 轮胎印"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["路面中央的轮胎印绕成一个完整的圈，然后是一条笔直的、发着焦味的刹车线。这圈路，好像有人提前替你跑过了。"]),
    choices: &[ChoiceDef { label: "记下轮胎印", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l2_tire")], route: Route::To("ss_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_03_car", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 连环撞车现场"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["三辆车撞成一串，钢梁像纸一样皱。副驾位空着——但座椅上有一个人形凹痕，仿佛刚有人被「抽」走。"]),
    choices: &[ChoiceDef { label: "去看空座", sub: "伏笔 · San-2", cond: None,
        effects: &[Eff::San(-2), Eff::MarkPoint("ss_p_l2_car")], route: Route::To("ss_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_06_trooper", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 巡警岗"),
    mood: "cold", speaker: Some("公路巡警"), voice: None,
    text: TextSpec::Static(&["巡警攥着对讲机，声音发颤：「那边……油罐车要炸了，我知道，可我拦不住它。你要是能让它别在咱这段烧起来，就快去看那焊缝。」"]),
    choices: &[ChoiceDef { label: "「就看一眼那焊缝。」", sub: "→ 油罐车残骸", cond: None,
        effects: &[Eff::San(2)], route: Route::To("ss_03_truck") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_03_gap", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 匝道护栏缺口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ss_foresee_explosion") {
            "你已识破爆炸征兆，绕开泄漏点踏出高速肩道。身后传来一声闷响——油罐车炸在辅导空地上，无人伤亡。你走向前方那栋住宅。".to_string()
        } else {
            "护栏豁口外是燃气泄漏的油罐车现场。你还没有识破「爆炸」的征兆——硬闯，只会被它收走。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "通过缺口（去住宅）", sub: "需 ss_foresee_explosion · →L3", cond: Some(cond_explosion),
            effects: &NO_EFF, route: Route::To("ss_04_l3_hub") },
        ChoiceDef { label: "回到加油站收费站", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ss_03_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 环境机关 · 爆炸 ================= */
SceneDef {
    id: "ss_11_death_boom", bg: Some("sishen_bg_battle.png"), loc: Some("L2 · 油罐车侧翻点 · 泄漏核心"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["汽油味的浓度猛地翻倍。焊缝那头，一点火花正顺着地表水迹朝燃烧球爬去。空气热得像要烧起来。"]),
    choices: &[ChoiceDef { label: "【逼近泄漏核心】", sub: "预判爆炸征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_boom) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 住宅 hub ================= */
SceneDef {
    id: "ss_04_l3_hub", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 郊外住宅"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ss_fate_rewritten") {
            "三桩意外——坠落、爆炸、触电——都被你从名单上划掉。住宅里的电灯忽明忽暗，最后亮成一排：它们认得你。".to_string()
        } else {
            "这是最后一个连环。一台忘了关的煤气灶在冒甜味，电箱跳了闸，地面积着水，楼梯扶手在轻轻发抖。（电箱 / 煤气灶 / 楼梯 / 积水 / 邻居 / 车库）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "跳闸电箱", sub: "触电征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_04_fuse") },
        ChoiceDef { label: "煤气灶", sub: "爆炸余响", cond: None, effects: &NO_EFF, route: Route::To("ss_04_oven") },
        ChoiceDef { label: "楼梯口扶手", sub: "坠落征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_04_stair") },
        ChoiceDef { label: "地面积水", sub: "触电征兆", cond: None, effects: &NO_EFF, route: Route::To("ss_04_water") },
        ChoiceDef { label: "与隔壁邻居交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("ss_07_neighbor") },
        ChoiceDef { label: "车库 · 死神·使者", sub: "象征战（可绕）", cond: None, effects: &NO_EFF, route: Route::To("ss_04_garage") },
        ChoiceDef { label: "对照命运清单", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("ss_20_settle") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_04_fuse", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 跳闸电箱"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["电箱盖半开，跳闸的把手悬在半空，底下缠着一截裸露的铜线。你想碰它，心底却响起一个名字：触电。"]),
    choices: &[ChoiceDef { label: "【识破触电征兆】", sub: "ss_foresee_shock · 改命", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l3_fuse")], route: Route::Dyn(mark_shock) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_04_oven", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 煤气灶"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["煤气灶还开着，火苗被风吹得乱晃，把窗台上积的那层蜡熏出一股甜味。你嗅到机场油罐车一样的——火的气息。"]),
    choices: &[ChoiceDef { label: "关上煤气阀", sub: "GoodEnd 前置 · San+2", cond: None,
        effects: &[Eff::San(2), Eff::MarkPoint("ss_p_l3_oven")], route: Route::To("ss_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_04_stair", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 楼梯口"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["楼梯的扶手松得能整根掰下来。楼梯尽头漆黑，像是早就等在原处要接住一个「坠落」的人。你认出了这个征兆。"]),
    choices: &[ChoiceDef { label: "【识破坠落征兆·住宅】", sub: "ss_foresee_fall · 改命", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l3_stair")], route: Route::Dyn(mark_fall_l3) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_04_water", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 地面积水"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["厨房和走廊的地砖上，水正沿着裂缝汇成一小片。水里映着一盏忽明忽暗的灯——那盏灯下，正是没关的电箱。你也识破了「触电」。"]),
    choices: &[ChoiceDef { label: "【识破触电征兆】", sub: "ss_foresee_shock · 改命", cond: None,
        effects: &[Eff::MarkPoint("ss_p_l3_water")], route: Route::Dyn(mark_shock) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_07_neighbor", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 隔壁邻居"),
    mood: "cold", speaker: Some("隔壁邻居"), voice: None,
    text: TextSpec::Static(&["邻居扒着门缝，声音小得像怕打扰什么：「那家人……昨儿全搬走了。灯还亮着，水还开着，可一个人都不在了。像……死神刚来吃过饭。」"]),
    choices: &[ChoiceDef { label: "「这屋子我自己进。」", sub: "San+2 · 提示", cond: None,
        effects: &[Eff::San(2)], route: Route::To("ss_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_04_garage", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 车库"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["车库门自动卷起。无面目的人形悬在正中——它没动，但整个车库的地面都朝它倾斜。它替你「临演」了每一件逃过的意外。"]),
    choices: &[
        ChoiceDef { label: "出手破使者", sub: "象征战 ss_emissary", cond: None, effects: &NO_EFF, route: Route::To("ss_09_emissary") },
        ChoiceDef { label: "转身离开（绕过）", sub: "规则流 · 不战", cond: None, effects: &NO_EFF, route: Route::To("ss_04_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 使者象征战 ---- */
SceneDef {
    id: "ss_09_emissary", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 车库 · 决战"),
    mood: "danger", speaker: Some("死神·使者"), voice: None,
    text: TextSpec::Static(&["使者抬起手，指尖悬着一枚「意外」。它轻笑：「你改得了名字，改不了名单。」（战斗）"]),
    choices: &[], fight_id: Some("ss_emissary"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_09_emissary_win", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 车库"),
    mood: "calm", speaker: Some("死神·使者"), voice: None,
    text: TextSpec::Static(&["使者化作一片倒计时数字，散进灯影里。地上的水退去，电箱「啪」地合上，煤气阀自动拧紧——它把名单还给了你。你再没被「意外」选上。"]),
    choices: &[ChoiceDef { label: "对照命运清单", sub: "结算", cond: None, effects: &NO_EFF, route: Route::To("ss_20_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 环境机关 · 触电 / 坠落 ================= */
SceneDef {
    id: "ss_12_death_shock", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 短路积水区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["水漫过你的鞋沿，一瞬的电弧从电箱那头炸亮，整片积水亮成一条通电的河。死神没出手——是这条「意外」自己动了。"]),
    choices: &[ChoiceDef { label: "【触碰带电的积水】", sub: "预判触电征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_shock) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_13_death_stair", bg: Some("sishen_bg_invest.png"), loc: Some("L3 · 楼梯坠落点"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["楼梯扶手在你抓空的瞬间整根崩下，脚下木板塌陷。这具「坠落」早为你备好了角度。"]),
    choices: &[ChoiceDef { label: "【踏向塌陷的楼梯】", sub: "预判坠落征兆则改写命运，否则San-10 · 意外身故", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_stair) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结算 ================= */
SceneDef {
    id: "ss_20_settle", bg: Some("sishen_bg_invest.png"), loc: Some("命运清单 · 结算"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ss_fate_rewritten") {
            "你把三桩意外的征兆一一划掉。死神收回了名单——它明白，这趟已无可收。".to_string()
        } else {
            "还有些征兆你没来得及读。但你仍带着那些「改过的名字」走到这里。主线判定你的支线结算评级。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "确认走向撤离光柱", sub: "结算 · 评级", cond: None, effects: &NO_EFF, route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ss_21_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "死 神 从 不 露 面".into(), good: true,
            body_html: format!(
                "<p>你站在住宅门口，回头——那三桩「意外」都化作烟尘，没有一桩带走你。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>命运清单</td><td style='color:#9a958a'>坠落{} 爆炸{} 触电{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 </table>",
                st.points,
                if st.flag("ss_foresee_fall") { "✓" } else { "✗" },
                if st.flag("ss_foresee_explosion") { "✓" } else { "✗" },
                if st.flag("ss_foresee_shock") { "✓" } else { "✗" },
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案 · 意外身故（复活扣 300 / 回主神）================= */
SceneDef {
    id: "ss_40_death_fall", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("扶梯之外的坠落", "未预判坠落征兆，坠下候机大厅楼层")), card: |_st| crate::state::Card {
            title: "扶 梯 之 外".into(), good: false,
            body_html: r#"<p>金属护栏在你体重落下那一刻松开，你坠向两层楼下的地面。广播只来得及用半个音节，为你的「意外身故」画上句点。</p>
<p style='color:#ff8a8a'>【死亡档案 · 意外身故】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "ss_41_death_boom", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("油罐车的爆点", "未识破爆炸征兆，撞进爆心")), card: |_st| crate::state::Card {
            title: "油 罐 车 之 心".into(), good: false,
            body_html: r#"<p>你踏进泄漏核心的一瞬，火舌舔上油面。整段高速炸成一个闷响——「意外身故」四个字，是巡警用血写的。</p>
<p style='color:#ff8a8a'>【死亡档案 · 意外身故】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "ss_42_death_shock", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("众水之下的电流", "未识破触电征兆，踏入带电积水")), card: |_st| crate::state::Card {
            title: "通 电 的 河".into(), good: false,
            body_html: r#"<p>电弧顺着积水爬上你，你连惨叫都来不及发出。死神没露面——这件「意外」，它替你办妥了。</p>
<p style='color:#ff8a8a'>【死亡档案 · 意外身故】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "ss_43_death_stair", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("十二级台阶", "未预判坠落征兆，摔下自家楼梯")), card: |_st| crate::state::Card {
            title: "十 二 级 台 阶".into(), good: false,
            body_html: r#"<p>扶手在你抓空的瞬间崩断，你顺着塌陷的木板滚下去。最下面一阶，正好是你「意外身故」的位置。</p>
<p style='color:#ff8a8a'>【死亡档案 · 意外身故】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "ss_44_death_enforcer", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("名单上的余笔", "被使者种下的一桩「意外」追索而亡")), card: |_st| crate::state::Card {
            title: "名 单 上 的 余 笔".into(), good: false,
            body_html: r#"<p>使者在散去的倒数里回头看你一眼。你漏掉的那桩「意外」，正沿着你还没改写的名字追上来。</p>
<p style='color:#ff8a8a'>【死亡档案 · 意外身故】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
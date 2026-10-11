//! 《生化危机·伊芙琳·浣熊市地下》全部剧情场景与「追踪者·复仇女神」选择驱动 BOSS 配置。
//! α 待替换占位：bg 用 img_zhuyuan_book.png / img_laser.png / img_corridor.png（主线替换为本副本专属背景）。
//! 本文件为全新新增文件，只导出静态数据（SH3_SCENES / shenghua3_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/shenghua3_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `sh3_` 前缀；fight id 全部 `sh3_` 前缀（与既有无重名）。
//! 核心「地下幸存」主题：下水道→警察局→实验室逐层收集「样本/信条/孵化日志」三份证据；
//!   在孵化室门前的终端对追踪者·复仇女神发动神经干扰的「第二选择」——宽赦幸存者 / 把样本喂给
//!   复仇女神 / 引爆孵化室同归于尽，三分支开放结局。BOSS 为选择驱动（不碰随机战斗）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_drug(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "sh3_antibiotic") }
fn cond_all_evidence(st: &GameState) -> bool {
    st.flag("sh3_ev_sample") && st.flag("sh3_ev_creed") && st.flag("sh3_ev_log")
}

/* =====================================================================
   证据链工具（收集三份证据，齐备置 sh3_evidence_full 并加分）
   ===================================================================== */
fn collect_ev(st: &mut GameState, which: &str, hub: &str) -> String {
    if !st.flag(which) {
        st.set_flag(which);
        if cond_all_evidence(st) && !st.flag("sh3_evidence_full") {
            st.set_flag("sh3_evidence_full");
            st.points += 200;
        }
    }
    hub.to_string()
}
fn mark_ev_sample(st: &mut GameState) -> String { collect_ev(st, "sh3_ev_sample", "sh3_01_l1_hub") }
fn mark_ev_creed(st: &mut GameState) -> String {
    st.set_flag("sh3_creed_read"); // 读通伞公司信条，解锁井道 G1
    collect_ev(st, "sh3_ev_creed", "sh3_03_l2_hub")
}
fn mark_ev_log(st: &mut GameState) -> String {
    st.set_flag("sh3_log_read"); // 读孵化日志，解锁孵化室终端门 G2
    collect_ev(st, "sh3_ev_log", "sh3_04_l3_hub")
}

/* ---- 环境机关：污水渠漫水区（未查水质则感染，查证后安然） ---- */
fn zone_sewage(st: &mut GameState) -> String {
    if st.flag("sh3_water_checked") {
        st.points += 30;
        st.set_flag("sh3_sign_sew_ok");
        "sh3_01_l1_hub".to_string()
    } else {
        "sh3_40_death_sewage".to_string()
    }
}

/* ---- BOSS：追踪者·复仇女神（选择驱动）大局起手/推进/取胜 ---- */
fn sh3_boss_start(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sh3_boss") {
            st.fight = Some(crate::state::Fight {
                id: "sh3_boss".into(), name: cfg.name.to_string(), hp: cfg.hp, max_hp: cfg.hp,
                dmg: cfg.dmg, reward: cfg.reward, reward_why: cfg.reward_why.to_string(),
                raged: false, rage_at: cfg.rage_at, guard_turn: false,
                pending_log: vec![cfg.intro.to_string()],
            });
        }
    }
    "sh3_09_boss_round".to_string()
}
/// 选择驱动 BOSS 对局：dmg 为本次伤害；guard 时本回合免伤
fn sh3_boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return sh3_boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 26 } else { 19 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "sh3_50_death".to_string(); }
    "sh3_09_boss_round".to_string()
}
fn sh3_boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("sh3_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "sh3_item_prototype");
    "sh3_30_final_choice".to_string()
}

/* ---- 第二选择 · 开放结局三分支 ---- */
fn end_spare(st: &mut GameState) -> String {
    st.set_flag("sh3_end_spare");
    "sh3_31_spare".to_string()
}
fn end_feed(st: &mut GameState) -> String {
    st.set_flag("sh3_end_feed");
    "sh3_32_feed".to_string()
}
fn end_blowup(st: &mut GameState) -> String {
    st.set_flag("sh3_end_blowup");
    "sh3_33_blowup".to_string()
}

/* ---- 结算 ---- */
fn route_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() { st.sp_grade = Some('D'); }
    "sh3_42_card".to_string()
}

/* ---- 普通战斗胜利桩 / 终结空桩 ---- */
fn guard_win(_st: &GameState) -> String { "sh3_05_win".to_string() }
fn fins_false(_st: &GameState, _hp: i32) -> bool { false }
fn fins_name(_st: &GameState) -> String { String::new() }
fn fins_desc(_st: &GameState) -> String { String::new() }
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   战斗配置表（id 全部 sh3_ 前缀）
   ===================================================================== */
pub fn shenghua3_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sh3_fight_l1", FightCfg {
            name: "下水道舔食者", hp: 100, dmg: (12, 18), reward: 140, reward_why: "肃清水道威胁",
            intro: "污水里爬出半只通体发白的舔食者，皮肤湿漉漉地反着光，没有眼睑的孔洞朝你张开——它没有眼睛，却凭气味咬定你，喉间滚出一串低沉的咕噜。",
            rage_at: Some(35), rage_text: "舔食者伸长舌头缠住钢梁，加速朝你扑来！",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "sh3_50_death",
        }),
        ("sh3_fight_l2", FightCfg {
            name: "暴君投影·地下防御体", hp: 130, dmg: (14, 20), reward: 180, reward_why: "突破警察局地下防线",
            intro: "审讯室的血化投影撕开墙皮站起，混凝土碎屑簌簌往下掉——暴君的钢爪在档案室深处咔咔作响，每一次开合都像在测试。它是这座城市地下的最后一道闸，而你的档案，正摊在它脚下。",
            rage_at: Some(45), rage_text: "暴影撕开天花板，混凝土碎块兜头压下。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "sh3_50_death",
        }),
        ("sh3_fight_l3", FightCfg {
            name: "追踪者亲卫", hp: 150, dmg: (15, 22), reward: 220, reward_why: "撕开孵化室的最后屏障",
            intro: "一个套着防化服、拎着重型榴弹的「亲卫」立在孵化室门前，防化服下的身体匀称得不正常，护着的正是复仇女神的容器。他开口，声音却像是从喉咙深处挤出来的：「再往前，就是她的家了。」",
            rage_at: Some(55), rage_text: "亲卫卸下防化面罩，露出一张与你曾救过的幸存者一模一样的脸。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "sh3_50_death",
        }),
        ("sh3_boss", FightCfg {
            name: "追踪者·复仇女神", hp: 260, dmg: (19, 28), reward: 500, reward_why: "在孵化室完成第二选择",
            intro: "孵化室的门在你身后焊死，电弧的火花还挂在门缝上。追踪者·复仇女神提着火箭筒与霰弹枪，从培养舱的硝烟里走出来，每一步都踩得很稳——它不是来杀你的，是来问你：这满城的罪，要还给谁。你身后再无退路，面前只有一个答案的位置。",
            rage_at: Some(90), rage_text: "复仇女神的眼亮成两盏红灯，火箭筒抵住你的胸口——它把「第二选择」的扳机，塞进了你手里。",
            on_rage: rage_none, finisher_if: fins_false, finisher_name: fins_name, finisher_desc: fins_desc,
            win: guard_win, death: "sh3_50_death",
        }),
    ]
}
/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn sh3_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    shenghua3_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 sh3_ 前缀）
   ===================================================================== */
pub static SH3_SCENES: &[SceneDef] = &[

/* ================= 开场 ================= */
SceneDef {
    id: "sh3_00", bg: Some("shenghua3_bg.png"), loc: Some("浣熊市地下 ≪生化·任务发布≫"),
    mood: "mystery", speaker: Some("主神·任务发布"), voice: None,
    text: TextSpec::Static(&[
        "<b>【场景·浣熊市地下】</b>你坠进一条腥臭的下水道，水没过脚踝，头顶是丧尸抓挠铁栅的嘶声，指甲刮过铁条的声音在管道里来回撞。空气里浮着一股淡淡的药味与尸体味，混成这座城市地底独有的味道。",
        "<i>「这座城市已经死了，可总有人要替它活。」</i>远处，某扇巨型闸门后传来一声沉闷的机械轰鸣，像一颗巨大的心脏在地下跳动——那是孵化室的呼吸。",
    ]),
    choices: &[
        ChoiceDef { label: "沿下水道前进", sub: "+5 点 · 记住这里的「生化残留」", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("sh3_01_l1_hub") },
        ChoiceDef { label: "细听孵化室的呼吸", sub: "San-2 · 确认复仇女神仍在", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("sh3_01_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 地下水质 hub ================= */
SceneDef {
    id: "sh3_01_l1_hub", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 浣熊市地下水道"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("sh3_ev_sample") {
            "你把水样溶液收进口袋，下水道的腥风似乎淡了些。蕾吉在闸门边等你，手里攥着半支抗生素。（闸门 / 控制阀 / 抽水泵 / 道边尸体）".to_string()
        } else {
            "铁栅外丧尸的嘶声忽远忽近，忽而又静止，像在等什么动静。生锈的控制阀下积着荧光污水，泛着不该有的绿；抽水泵早停摆，泵体上挂着半干的青苔；道边一句没写完的字碑被水泡开，字迹洇成一团墨。（闸门 / 控制阀 / 抽水泵 / 道边尸体）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "地下水闸门", sub: "证据·水样", cond: None, effects: &NO_EFF, route: Route::To("sh3_02_gate") },
        ChoiceDef { label: "生锈控制阀", sub: "证据·水样", cond: None, effects: &NO_EFF, route: Route::To("sh3_02_valve") },
        ChoiceDef { label: "废弃抽水泵", sub: "证据·水样", cond: None, effects: &NO_EFF, route: Route::To("sh3_02_machine") },
        ChoiceDef { label: "道边尸体", sub: "证据·水样", cond: None, effects: &NO_EFF, route: Route::To("sh3_02_corpse") },
        ChoiceDef { label: "逼近污水渠", sub: "感染环境危区", cond: None, effects: &NO_EFF, route: Route::To("sh3_10_sewage") },
        ChoiceDef { label: "与地下幸存者蕾吉交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("sh3_05_survive") },
        ChoiceDef { label: "走下闸门", sub: "需抗生素 → 警察局地下", cond: None, effects: &NO_EFF, route: Route::To("sh3_02_gateway") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_02_gate", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 地下水闸门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["闸门边的污水泛着奇异的荧光，绿光在水面一漾一漾，像某种东西在底下呼吸。水里漂着一管冷却剂密封的水样瓶，瓶身贴着编号。你认得那种绿——那是伞公司用来培育亲卫的培养基，而这管水样，就是这座地下城的第一份罪证。"]),
    choices: &[ChoiceDef { label: "【取走水样·样本】", sub: "sh3_ev_sample · 证据+1", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l1_gate")], route: Route::Dyn(mark_ev_sample) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_02_valve", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 生锈控制阀"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["生锈的控制阀封着一个压力表，表针停在一个不该停的刻度，像被什么东西死死摁住。你拧不动它，却在表盘背面看见一行刻字：「样本在疫苗前，生命在数据后。」刻痕很深，像是有人用指甲一刀一刀划出来的。"]),
    choices: &[ChoiceDef { label: "记下控制阀刻字", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l1_valve")], route: Route::To("sh3_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_02_machine", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 废弃抽水泵"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["抽水泵卡着一只防化手套，袖口绣着半枚伞公司 logo，指缝里还沾着干涸的培养液。机器转不动，但你闻到那股熟悉的药味——它就是从孵化室排下来的，顺着这条管道，喂给了整座地下城。"]),
    choices: &[ChoiceDef { label: "记下排水路径", sub: "罪证/样本线索", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l1_machine")], route: Route::To("sh3_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_02_corpse", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 道边尸体"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["那句没写完的字碑被水泡开，露出一半：「……他们喂的不是疫苗，是……」。你把它扶正，水珠顺着刻痕往下淌，你读到了那个没说完的名字——而它本该是这座城市里最该被记住的人。"]),
    choices: &[ChoiceDef { label: "收好字碑碎片", sub: "样本线索 · San-2", cond: None,
        effects: &[Eff::San(-2), Eff::MarkPoint("sh3_p_l1_corpse")], route: Route::To("sh3_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_10_sewage", bg: Some("shenghua3_bg.png"), loc: Some("L1 · 污水渠漫水区"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["污水没过脚踝，荧光随水波漾开，覆盖在每一寸水下皮肤上，泛起细密的刺痒。你还没有查过水质——那些发绿的培养液，正在顺着毛孔往你身体里钻，像一群看不见的活物。"]),
    choices: &[ChoiceDef { label: "【污染防治】", sub: "查水质则免感染，否则 San-10 · 生化感染", cond: None,
        effects: &[Eff::San(-10)], route: Route::Dyn(zone_sewage) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 查水质后安然返回 */
SceneDef {
    id: "sh3_10_sewage_ok", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 净水取样点"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["你已在闸门取过水样，水质你心里有数。你踩过漫水区，荧光在水面散开又合拢，污染在你脚下退了半步——样本在手，生化威胁翻不了天。你第一次觉得，这个地下城没那么容易吞掉你。"]),
    choices: &[ChoiceDef { label: "回到闸门水道", sub: "→ L1 hub", cond: None,
        effects: &[Eff::Points(30)], route: Route::To("sh3_01_l1_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_05_survive", bg: Some("shenghua3_bg_underground.png"), loc: Some("L1 · 幸存者据点"),
    mood: "cold", speaker: Some("地下幸存者·蕾吉"), voice: None,
    text: TextSpec::Static(&["蕾吉把半支抗生素塞给你，指尖冰凉：「地下毒水熬不过我这种幸存者，可它会熬透你。闸门下去就是井道——酒席底下那点票子，都在孵化室。」她压低声音补了一句：「孵出来的东西不叫怪物，叫『复仇女神』。别把它当敌人，也别把它当朋友。」"]),
    choices: &[ChoiceDef { label: "「带我去井道。」", sub: "拿抗生素 sh3_antibiotic · San+2", cond: None,
        effects: &[Eff::AddItem("sh3_antibiotic"), Eff::San(2)], route: Route::To("sh3_02_gateway") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_02_gateway", bg: Some("shenghua3_bg.png"), loc: Some("L1 · 地下水闸口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("sh3_ev_sample") {
            "你攥着水样与抗生素，踏进井道，水靴踩在铁梯上发出空洞的回响。铁栅外丧尸的嘶声在身后远成潮水，渐渐听不见了——下一站，是这座城市权力的地下。".to_string()
        } else {
            "井道闸还封着，抗生素在你口袋，可水样还沉在水里，闸门不会为两手空空的人打开——你得先证明自己扛得住这个地下的毒。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "持抗生素走下井道（去警察局地下）", sub: "需 sh3_antibiotic · →L2", cond: Some(cond_has_drug),
            effects: &NO_EFF, route: Route::To("sh3_03_l2_hub") },
        ChoiceDef { label: "回到水道", sub: "", cond: None, effects: &NO_EFF, route: Route::To("sh3_01_l1_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 警察局地下 hub ================= */
SceneDef {
    id: "sh3_03_l2_hub", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 警察局地下"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("sh3_ev_creed") {
            "伞公司的信条你已读通，井道不再设防，连通风口的风都顺畅了几分。档案室的灯在为谁留着？灯光昏黄，像有人刚离开不久。（审讯室 / 档案桌 / 证据柜 / 逃生井盖）".to_string()
        } else {
            "巡逻灯的蓝光一下一下扫过潮湿的瓷砖墙，把墙上的水痕照成一张张面孔。审讯室的门虚掩着，门缝里透着冷气；档案桌抽屉没合上，露出一角纸页；证据柜里锁着一行字，隔着玻璃看不太清。（审讯室 / 档案桌 / 证据柜 / 逃生井盖）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "审讯室门", sub: "证据·信条", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_cell") },
        ChoiceDef { label: "档案桌", sub: "证据·信条", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_desk") },
        ChoiceDef { label: "证据柜", sub: "证据·信条", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_evidence") },
        ChoiceDef { label: "逃生井盖", sub: "伏笔", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_safe") },
        ChoiceDef { label: "与受困警员交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("sh3_06_officer") },
        ChoiceDef { label: "沿井道下楼", sub: "需读通信条 → 实验室", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_gateway") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_03_cell", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 审讯室门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["审讯室木门后贴着一张纸条，边缘被水汽浸得卷起：「革命需要代价，代价总是最穷的人先付。」下面压着一枚伞公司信封，封口还封着蜡——那是「信条」的钥匙，也是这座警察局地下最后一份体面的文书。"]),
    choices: &[ChoiceDef { label: "【取走伞公司信条】", sub: "sh3_ev_creed · 证据+1", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l2_cell")], route: Route::Dyn(mark_ev_creed) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_03_desk", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 档案桌"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["档案桌抽屉弹出半张化验单，患者名录上的人全被粗黑的线划去——只留最底下那个还亮着：S-伊芙琳·RE。划痕力道很重，像是划去的时候手在发抖。你把这个名字收好。"]),
    choices: &[ChoiceDef { label: "记下化验单名单", sub: "信条线索", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l2_desk")], route: Route::To("sh3_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_03_evidence", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 证据柜"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["证据柜锁着一只沙漏，玻璃罩上蒙着灰，沙漏底压着一份伞公司「信条」的抄件。你把沙漏翻过来，细沙簌簌落下——沙子落下的方向，指向井道更深处的孵化室，像一份静默的告密。"]),
    choices: &[ChoiceDef { label: "收下信条抄件", sub: "信条线索", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l2_evidence")], route: Route::To("sh3_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_03_safe", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 逃生井盖"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["逃生井盖锈死了，缝里塞着一截防化服的袖口——袖口的绣线告诉你，和下水道那只是同一双手。你撬开一角，锈屑落了一地，露出一条通往实验室的通风道，风从里面灌出来，带着消毒水的凉意。"]),
    choices: &[ChoiceDef { label: "记下逃生井盖", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l2_safe")], route: Route::To("sh3_03_l2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_06_officer", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 警员避难岗"),
    mood: "cold", speaker: Some("受困警员"), voice: None,
    text: TextSpec::Static(&["警员握着只剩两格的手电，光柱在地板上抖，声音发哑：「地下室的人全替它跑了，就剩我一个。孵化室那怪物……它不杀我，只让我活着看。走——别回头看。」他指了指档案桌，又补了一句：「要看就去看那封信条，看完赶紧走。」"]),
    choices: &[ChoiceDef { label: "「我去读那份信条。」", sub: "San+2 · → 档案桌", cond: None,
        effects: &[Eff::San(2)], route: Route::To("sh3_03_desk") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_03_gateway", bg: Some("shenghua3_bg.png"), loc: Some("L2 · 井道底层"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("sh3_ev_creed") {
            "你读通了伞公司的信条，井道的隐秘门顺滑地滑动开，像一扇早就认识你的门。消毒喷雾喷了你一脸，冰凉的雾在灯光下散开——实验室的气闸，正等你按下最后一个按钮，把这场罪证推向尾声。".to_string()
        } else {
            "井道底的通气窗还焊着，焊点锃亮，是最近才补上的。你该先回去读透那把「信条」的钥匙——它不会平白无故锁着这么深的地方。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "通过气闸进实验室（→L3）", sub: "需 sh3_creed_read · →L3", cond: None,
            effects: &NO_EFF, route: Route::To("sh3_04_l3_hub") },
        ChoiceDef { label: "回到警察局地下", sub: "", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_l2_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 实验室 hub ================= */
SceneDef {
    id: "sh3_04_l3_hub", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 实验室·孵化室"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("sh3_ev_log") {
            "孵化日志在案，神经干扰代码已在手，最后一块拼图就位。孵化室的门缝里漏出红灯，一明一灭，像一只眼睛在门后眨动——追踪者·复仇女神，正等着你的「第二个选择」。（停尸间 / 主控台 / 菌株容器 / 数据终端）".to_string()
        } else {
            "无菌灯管嗡嗡作响，光白得晃眼。停尸间的拉门没关严，露出一线黑缝；主控台屏幕滚着「孵化指令」，光标自己一行行往下跳；菌株容器里游着一条蠕动的绿影，像一条没有眼睛的鱼。（停尸间 / 主控台 / 菌株容器 / 数据终端）".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "停尸间", sub: "证据·孵化日志", cond: None, effects: &NO_EFF, route: Route::To("sh3_04_morgue") },
        ChoiceDef { label: "主控台", sub: "证据·孵化日志", cond: None, effects: &NO_EFF, route: Route::To("sh3_04_console") },
        ChoiceDef { label: "菌株容器", sub: "伏笔", cond: None, effects: &NO_EFF, route: Route::To("sh3_04_vat") },
        ChoiceDef { label: "数据终端", sub: "证据·孵化日志", cond: None, effects: &NO_EFF, route: Route::To("sh3_04_data") },
        ChoiceDef { label: "与卧底医生韩交谈", sub: "提示", cond: None, effects: &NO_EFF, route: Route::To("sh3_07_doctor") },
        ChoiceDef { label: "走向孵化室", sub: "选择驱动 BOSS", cond: None, effects: &NO_EFF, route: Route::Dyn(sh3_boss_start) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_04_morgue", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 停尸间"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["停尸间最里一格躺着个没名字的人，白布掀了一半，关节处都焊着伞公司的「S」标签，像出厂时的编号。你在他胸口压着的那页纸上，读到孵化日志的开头——第一行写着：「第 001 号，自愿。」"]),
    choices: &[ChoiceDef { label: "【取走孵化日志】", sub: "sh3_ev_log · 证据+1", cond: None,
        effects: &[Eff::San(-2), Eff::MarkPoint("sh3_p_l3_morgue")], route: Route::Dyn(mark_ev_log) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_04_console", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 主控台"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["主控台弹出一段孵化日志，屏幕的冷光映在你脸上：「复仇女神会服从携带『第二个选择』的操控者——一旦启动，它只认你一个人。」最后一行停顿了很久才刷新出来：「她曾是人。」你把代码抄下来，把那三个字也记在心里。"]),
    choices: &[ChoiceDef { label: "抄下神经干扰代码", sub: "孵化日志线索", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l3_console")], route: Route::To("sh3_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_04_vat", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 菌株容器"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["菌株容器里游着一条发光的绿线，标签写着「EVE-RE 样本」，编号旁还有一行小字：伊芙琳。你一靠近它，容器上的红灯就闪成心跳的节拍，快了几分——它认得你，或者说，它在等一个名字。"]),
    choices: &[ChoiceDef { label: "记录菌株心跳", sub: "伏笔", cond: None,
        effects: &[Eff::MarkPoint("sh3_p_l3_vat")], route: Route::To("sh3_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_04_data", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 数据终端"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["数据终端存着孵化室的地形图，一格格亮在屏幕上——复仇女神的出生槽位、通风口、逃生路线，还有那个标着「第二选择」的红色按钮，孤零零亮在最深处。你全记下了，连同按钮下那行注释：「仅限一人。」"]),
    choices: &[ChoiceDef { label: "拉取孵花室地形", sub: "+5 点 · 孵化日志链", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("sh3_p_l3_data")], route: Route::To("sh3_04_l3_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_07_doctor", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 医生值守台"),
    mood: "cold", speaker: Some("卧底医生·韩"), voice: None,
    text: TextSpec::Static(&["医生摘下手套，露出一枚伞公司的士官戒，戒面被磨得发亮：「它不叫追踪者，它叫『复仇女神』。你要么给它『第二个选择』，要么，别让它替你选。」他顿了顿，声音低下去：「我见过它选——那之后，没人活着出来过。」"]),
    choices: &[ChoiceDef { label: "「我去读孵化日志。」", sub: "San+2 · →主控台", cond: None,
        effects: &[Eff::San(2)], route: Route::To("sh3_04_console") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= BOSS：追踪者·复仇女神（选择驱动） ================= */
SceneDef {
    id: "sh3_09_boss_round", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 孵化室"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Dyn(|st| format!("孵化室的灭菌灯在复仇女神背后亮成一行红影。它在陪你打这场「第二选择」的仗——证据 {}，神经干扰披甲。BOSS 剩余 {} 血，你 HP {}", if st.flag("sh3_evidence_full") { "已集齐，干扰代码在手" } else { "未集齐，它只压你七分" }, st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| sh3_boss_act(st, 34, false)) },
        ChoiceDef { label: "格挡", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| sh3_boss_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 胜利 · 第二个选择（开放结局三分支） ---- */
SceneDef {
    id: "sh3_30_final_choice", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 孵化室 · 神经干扰终端"), mood: "choice",
    speaker: Some("追踪者·复仇女神"), voice: None,
    text: TextSpec::Static(&[
        "复仇女神的火箭筒缓缓垂下，枪口还冒着烟。它把枪口对准自己胸口那个闪烁的孵化器，红灯一明一灭地映在它脸上，它看着你：「第二个选择，只有你能下。」",
        "屏幕亮出三个字：宽赦 / 喂给 / 引爆。满城活人与这具复仇的女神，都在等你按下去。空气里只剩下红灯跳动的节拍。",
    ]),
    choices: &[
        ChoiceDef { label: "【宽赦幸存者】", sub: "关闭孵化器 · 让地下的人活下去", cond: None, effects: &NO_EFF, route: Route::Dyn(end_spare) },
        ChoiceDef { label: "【把样本喂给复仇女神】", sub: "让它记住这场浩劫 · 维持秩序", cond: None, effects: &NO_EFF, route: Route::Dyn(end_feed) },
        ChoiceDef { label: "【引爆孵化室】", sub: "同归于尽 · 让这座城市彻底结束", cond: None, effects: &NO_EFF, route: Route::Dyn(end_blowup) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_31_spare", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 孵化室（宽赦之径）"), mood: "awe",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你按下「宽赦」。孵化器的培养液缓缓排空，哗啦的水声在安静的实验室里格外清晰，复仇女神那只闪烁的红灯，第一次倒转为平和的蓝。",
        "井道深处，幸存者的灯一格格重新点亮，像有人把这座城的灯一盏盏擦亮。你没有救回整座城，却给了这座城地下残余的人一个天亮。",
    ]),
    choices: &[ChoiceDef { label: "见证重启", sub: "San+20 · 宽赦结局 · 结算", cond: None,
        effects: &[Eff::San(20), Eff::Points(150)], route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_32_feed", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 孵化室（喂给之径）"), mood: "danger",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你把 EVE-RE 样本倒进复仇女神胸口的容器，绿色的光丝顺着导管游进去。它吸收后，红灯取代蓝灯，却只是站在原地，没再对你举起武器。",
        "它站在你身侧，像一尊沉默的哨兵，替这座城市守着那道没人敢跨的界线。这样也好——总得有人，让浩劫不再重演，哪怕那人是具复仇的兵器。",
    ]),
    choices: &[ChoiceDef { label: "与复仇女神并肩", sub: "+200 点 · 喂给结局 · 结算", cond: None,
        effects: &[Eff::Points(200)], route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "sh3_33_blowup", bg: Some("shenghua3_bg_lab.png"), loc: Some("L3 · 孵化室（引爆之径）"), mood: "mystery",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你按下了引爆。孵化室的玻璃在掌心里炸成白光，冲击波推着碎片擦过耳侧，复仇女神没有后退，它用身体替你挡掉了第一波冲击，裂痕爬满它的镜面。",
        "你抱着最后的样本冲出气闸时，身后的整座地下城塌进了更深的黑暗，轰鸣一浪接一浪——那是这座被病毒摧毁的城市，最后一次扬声，也是最后一次有人替它发声。",
    ]),
    choices: &[ChoiceDef { label: "带着样本离开", sub: "San-10 · +250 点 · 引爆结局 · 结算", cond: None,
        effects: &[Eff::San(-10), Eff::Points(250)], route: Route::Dyn(route_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 普通战斗胜利中转 */
SceneDef {
    id: "sh3_05_win", bg: Some("shenghua3_bg_lab.png"), loc: Some("浣熊市地下 · 威胁清除"), mood: "calm",
    speaker: None, voice: None,
    text: TextSpec::Static(&["生化威胁在你脚下散成一滩腥水，被下水道的风卷走。你获得一份幸存者的谢礼。" ]),
    choices: &[
        ChoiceDef { label: "回下水道", sub: "L1", cond: None, effects: &NO_EFF, route: Route::To("sh3_01_l1_hub") },
        ChoiceDef { label: "回警察局地下", sub: "L2", cond: None, effects: &NO_EFF, route: Route::To("sh3_03_l2_hub") },
        ChoiceDef { label: "回实验室", sub: "L3", cond: None, effects: &NO_EFF, route: Route::To("sh3_04_l3_hub") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 结算卡（胜利） ---- */
SceneDef {
    id: "sh3_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "结 算".into(), good: true,
            body_html: format!("<p>你在浣熊市地下的生化浩劫里做了第二个选择——那座城市没有等来救世主，却等来了一个替它按下按钮的人。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr><tr><td>结局</td><td>{}</td></tr></table>", st.points, if st.flag("sh3_end_spare") {"宽赦重启"} else if st.flag("sh3_end_feed") {"并肩守界"} else {"引爆离场"}),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- 死亡档案（生化感染 / 孵化失败） ---- */
SceneDef {
    id: "sh3_40_death_sewage", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("浣熊市·生化感染", "未先查水质，污水渠的培养液渗透进血液")),
        card: |_st| crate::state::Card {
            title: "死 亡".into(), good: false,
            body_html: r#"<p>你倒在了污水渠的绿光里，荧光在水面漾开，盖住你的影子。这座城市吞掉一个人的方式，向来是这样悄无声息。</p><p style='color:#ff8a8a'>【死亡档案】那支没早一步到手的抗生素，成了最后的差一步。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "sh3_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("浣熊市·第二选择未竟", "在孵化室的神经干扰前没能给出答案")),
        card: |_st| crate::state::Card {
            title: "死 亡".into(), good: false,
            body_html: r#"<p>你死在了孵化室的红光里，红灯在你闭上眼前最后闪了一下，像一声叹息。</p><p style='color:#ff8a8a'>【死亡档案】复仇女神没等到你的第二个选择，先替你选了。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
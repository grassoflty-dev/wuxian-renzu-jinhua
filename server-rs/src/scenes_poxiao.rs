//! 《无限曙光 · 破晓封锁区》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/wuxian_shuguang/shixue_poxiao.md（§3/§5/§6/§10）。
//! 方向：嗜血破晓世界观下的黎明之城·人类与血族对峙；弱化阴谋线，强调世界与三方博弈。
//! 钩子：「太阳快出来了——这对某些人，是末日。」
//! 日光倒计时用剧情 flag 降级（px_phase_1/2 + px_daylight）；三方势力互斥 flag；BOSS 日光终结用镜阵 flag 链。
//! 本文件为全新新增文件，只导出静态数据（POXIAO_SCENES / poxiao_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/poxiao_impl_log.md ★外部依赖）。
//! 场景 id 全部 `px_` 前缀；fight id 全部 `pc_` 前缀；BOSS 格里高尔用选择驱动遭遇链。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 换成新图；当前用现有图占位）：
//!   L1 街道 bg px_bg_street   （现用 img_zhuyuan_book.png 占位）
//!   L2 叛军据点 bg px_bg_sewer（现用 img_corridor.png 占位）
//!   L3 尖塔 bg px_bg_spire   （现用 img_redqueen.png 占位）
//! 敌人立绘复用：guard→守卫/血族、hunter→沉沦者；BOSS 新立绘由主 agent 统一生图替换。

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
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
/// 是否已选择某一阵营（互斥：三者只置其一）
fn cond_no_faction(st: &GameState) -> bool {
    !st.flag("px_faction_human") && !st.flag("px_faction_moderate") && !st.flag("px_faction_neutral")
}
/// 镜阵列已校准（左→右→主控 全完成）
fn cond_dawn_mirror(st: &GameState) -> bool { st.flag("px_dawn_mirror") }
/// 主控镜阵可点亮：左右镜已对准且尚未校准
fn cond_mirrors_ready(st: &GameState) -> bool {
    st.flag("px_mirror_l") && st.flag("px_mirror_r") && !st.flag("px_dawn_mirror")
}
/// 军械库对人类路线开放
fn cond_armory_open(st: &GameState) -> bool { st.flag("px_armory_open") }
/// 档案已解密（顶层闸门 / 决战前置）
fn cond_archive(st: &GameState) -> bool { st.flag("px_archive") }
/// 决战需先有档案 + 镜阵，方可选「校准主控台」
fn cond_calibrate_ok(st: &GameState) -> bool { cond_archive(st) && cond_dawn_mirror(st) }
/// BOSS 低血 + 日光射线就绪 → 终结技可选
fn cond_sunray_strike(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.hp <= 40).unwrap_or(false) && st.flag("px_sunray_ready")
}
/// 泵房三处（阀门A/阀门B/控制台）齐备 → 排空水闸
fn cond_valve_done(st: &GameState) -> bool {
    st.flag("px_valve_a") && st.flag("px_valve_b") && st.flag("px_valve_console")
}
/// 货运电梯已通电（可上尖塔）
fn cond_generator(st: &GameState) -> bool { st.flag("px_generator") }

/* =====================================================================
   普通敌人 win 回调（native FightCfg 由引擎在胜负后调用 win）
   ===================================================================== */
fn px_win_l1(_st: &GameState) -> String { "px_l1_hub".to_string() }
fn px_win_deg_after(_st: &GameState) -> String { "px_deg_after".to_string() }
fn px_win_l2(_st: &GameState) -> String { "px_l2_hub".to_string() }
fn px_win_l3(_st: &GameState) -> String { "px_l3_hub".to_string() }
fn px_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   战斗配置表（id 全部 pc_ 前缀）
   ===================================================================== */
pub fn poxiao_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("pc_vamp_civil", FightCfg {
            name: "平民吸血鬼", hp: 36, dmg: (7, 12), reward: 12, reward_why: "肃清夜行的平民吸血鬼",
            intro: "一个穿着晚礼服残骸的苍白人型拦住去路，红瞳一缩张开獠牙——这座城市的日常，是把人类当成流动的血浆站。",
            rage_at: None, rage_text: "", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l1, death: "px_death_vamp",
        }),
        ("pc_guard", FightCfg {
            name: "血站守卫·吸血鬼", hp: 40, dmg: (8, 13), reward: 15, reward_why: "击退血站守卫吸血鬼",
            intro: "黑制服的守卫吸血鬼端着血袋，像看一桶待宰的肉一样打量你：『宵禁之后，还出来送死的，只有你们人类。』",
            rage_at: None, rage_text: "", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l1, death: "px_death_vamp",
        }),
        ("pc_degenerate", FightCfg {
            name: "沉沦者", hp: 55, dmg: (11, 17), reward: 25, reward_why: "击退失控的沉沦者",
            intro: "暗巷里的东西已算不上吸血鬼——肢体反关节、指甲长成利爪、嘴角撕裂到耳根。它循着人味扑来，嘴里含着不成调的低语：血……血……",
            rage_at: Some(20), rage_text: "沉沦者发出疯吼挣脱束缚，下一击的快意失控翻涌——狂暴！", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_deg_after, death: "px_death_deg",
        }),
        ("pc_degenerate_horde", FightCfg {
            name: "沉沦者群", hp: 60, dmg: (12, 18), reward: 30, reward_why: "冲出十字路口的沉沦者群",
            intro: "十字路口被喘着白气的沉沦者围满，惨白的皮肤在霓虹下泛着血光。带头那只回身一吼，又招来一只平民吸血鬼。",
            rage_at: Some(25), rage_text: "更多的沉沦者从暗巷涌来——血腥味把整条街都点着了！", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l1, death: "px_death_deg",
        }),
        ("pc_rebel_guard", FightCfg {
            name: "武装人类叛军", hp: 42, dmg: (9, 15), reward: 20, reward_why: "击败武装人类叛军",
            intro: "土黄战术装的人类举起枪，眼底是破釜沉舟的决绝：『这个黎明，要么把太阳还给城市，要么一起死。』",
            rage_at: None, rage_text: "", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l2, death: "px_death_vamp",
        }),
        ("pc_vamp_patrol", FightCfg {
            name: "血族巡逻队", hp: 50, dmg: (10, 16), reward: 25, reward_why: "击退地下排水道的血族巡逻队",
            intro: "地下排水道主干的阴影里，一队持着电棍的血族巡逻队巡过，靴声在管道里回荡——这里是他们的辖区，人类从不该出现。",
            rage_at: None, rage_text: "", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l2, death: "px_death_vamp",
        }),
        ("pc_elite", FightCfg {
            name: "嗜血沉沦者·精英", hp: 65, dmg: (13, 20), reward: 45, reward_why: "击杀发电机房的嗜血沉沦者精英",
            intro: "发电机房里盘踞着一只明显更强的沉沦者，背棘外露、双瞳赤红，连同类都要绕着它走——这是沉沦者中的『精英』。",
            rage_at: Some(25), rage_text: "精英沉沦者狂化，四肢反折迸发出非人的爆发力——ATK 骤升！", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l2, death: "px_death_deg",
        }),
        ("pc_spire_guard", FightCfg {
            name: "尖塔卫队·血族士兵", hp: 50, dmg: (11, 17), reward: 30, reward_why: "击败黎明尖塔的血族卫队",
            intro: "黑金制服的尖塔卫队横在门前，冷眼打量你这个混入黎明之塔的人类：『底层的人类，不该知道塔顶藏着什么。』",
            rage_at: None, rage_text: "", on_rage: px_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: px_win_l3, death: "px_death_vamp",
        }),
        ("pc_boss_gregor", FightCfg {
            name: "高级沉沦者·格里高尔", hp: 120, dmg: (16, 24), reward: 380, reward_why: "击败高级沉沦者·格里高尔 · 夺回黎明尖塔",
            intro: "格里高尔从镜阵下站起，两米六的畸变躯干像活物一样蠕动。血族实验的『完美日间战士』彻底失控——它吞掉了一整个实验层，如今只剩嗜血的空壳。『血……还给我……』",
            rage_at: Some(60), rage_text: "格里高尔双目赤红炸裂——<b>嗜血狂化</b>！血……还给我——！！",
            on_rage: px_rage_none,
            finisher_if: |st, _| cond_sunray_strike(st),
            finisher_name: |_| "引动日光射线".to_string(),
            finisher_desc: |_| "你扳下主控台，穹顶玻璃折射入塔——一道日光射线贯穿格里高尔，其失控的再生在光照下寸寸崩溃！".to_string(),
            win: |_st| "px_gregor_round".to_string(),
            death: "px_death_gregor",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn px_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    poxiao_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   BOSS · 高级沉沦者·格里高尔（选择驱动遭遇链）
   血量存 st.fight（px_duel 的 Route::Dyn 初始化，引用 pc_boss_gregor 的 FightCfg）。
   ===================================================================== */
/// 初始化格里高尔会话（从 pc_boss_gregor 的 FightCfg 建 Fight）。需主线合并后 fight_cfg 能解析 pc_boss_gregor。
fn start_gregor(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("pc_boss_gregor") {
            st.fight = Some(crate::power::scaled_fight("pc_boss_gregor", cfg, st, vec![format!("<span class='miss'>{}</span>", cfg.intro)]));
        }
    }
    "px_gregor_round".to_string()
}

/// 胜利结算：+380（日光终结 +20）全奖、落 flag、按三方阵营路由结局、写 sp_grade。
/// 三方势力（互斥）→ 不同结局文本：人类=救世者(B)、血族=交易者(C)、中立=独行者(C)。
fn gregor_win(st: &mut GameState) -> String {
    st.points += 380;
    crate::world::add_item(st, "it_px_sun_crystal");
    crate::world::add_item(st, "it_px_edgar_file");
    st.set_flag("px_gregor_down");
    st.set_flag("px_sunrise");
    st.set_flag("px_end");
    st.set_flag("px_cleared");
    st.set_flag("px_daylight");
    if st.flag("px_faction_human") {
        st.sp_grade = Some('B');
        "px_end_human".to_string()
    } else if st.flag("px_faction_moderate") {
        st.sp_grade = Some('C');
        "px_end_moderate".to_string()
    } else {
        st.sp_grade = Some('C');
        "px_end_neutral".to_string()
    }
}

fn gregor_dead() -> String { "px_death_gregor".to_string() }

/// 每回合：进攻 / 卸力防守；狂暴@60 后半血嗜血，ATK 增至 21~29。
fn gregor_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return gregor_win(st);
    }
    let should_rage = st.fight.as_ref().map(|f| f.hp <= 60 && !f.raged).unwrap_or(false);
    if should_rage {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged { rng(21, 29) } else { rng(16, 24) };
    let dodge = if guard { 0.5 } else { 0.18 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return gregor_dead();
    }
    "px_gregor_round".to_string()
}

/// 日光射线终结：BOSS 半血狂暴 + 镜阵校准后触发，60 点固定折射伤害；处决窗口若斩杀则直接胜利。
fn gregor_sunray(st: &mut GameState) -> String {
    if let Some(f) = st.fight.as_mut() {
        f.hp = (f.hp - 60).max(0);
        f.pending_log.push("穹顶玻璃折射——日光射线贯穿格里高尔，其失控的再生在光照下寸寸崩溃！".to_string());
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        st.points += 20; // 日光终结奖励上调（380→400）
        return gregor_win(st);
    }
    gregor_act(st, 0, false)
}

/* =====================================================================
   剧情场景（id 全部 px_ 前缀）
   ===================================================================== */
pub static POXIAO_SCENES: &[SceneDef] = &[

    /* ================= 幕 0 · 开场「进入！嗜血破晓！」 ================= */
    SceneDef {
        id: "px_00_open", bg: Some("poxiao_bg.png"), loc: Some("L1 · 封锁区午夜街道"),
        mood: "cold", speaker: Some("主神系统 · 白光"), voice: Some("vo_px_open"),
        text: TextSpec::Static(&[
            "任务发布——Ｄ级绝境任务：破晓封锁区。目标：找到研究员爱德华·道尔顿，在黎明前将他护送至黎明尖塔顶层，校准日光镜阵。",
            "警告：找到道尔顿后，不得离开其半径范围——违者，抹杀。任务成败无支线剧情奖励，一切以结算为准。",
            "远处传来钟楼报时：凌晨三点零四分。<em>「太阳快出来了——这对某些人，是末日。」</em>",
        ]),
        choices: &[
            ChoiceDef { label: "冷静观察四周", sub: "+5 点 · 记住这片黎明前的城市", cond: None, effects: &[Eff::Points(5)], route: Route::To("px_00_check") },
            ChoiceDef { label: "沿大街前进", sub: "街上似乎有巡夜的吸血鬼", cond: None, effects: &NO_EFF, route: Route::To("px_00_check") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_00_check", bg: Some("poxiao_bg.png"), loc: Some("L1 · 教堂广场"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "空荡的封锁城区压在永夜的紫蓝天空下。霓虹残灯、废弃车辆、教堂剪影……这座曾属于人类的城市，如今灯火通明的是几处血站。",
            "教堂的彩色玻璃后漏出一线昏黄。门缝里，一个穿着研究员白大褂、脸色苍白得近乎透明的男人正在写什么——他就是爱德华·道尔顿。",
            "（地图上可调查各调查点：圣坛笔记/血站钟楼/空屋信件/店主遗物/坍塌废墟）",
        ]),
        choices: &[ChoiceDef { label: "推门进教堂", sub: "面见道尔顿", cond: None, effects: &NO_EFF, route: Route::To("px_dalton") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= L1 hub * 街道 ================= */
    SceneDef {
        id: "px_l1_hub", bg: Some("poxiao_bg.png"), loc: Some("L1 · 封锁城区街道"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["永夜的城市压在头顶，钟楼的时针悬在凌晨。道尔顿的白大褂在昏黄里一闪——他还在教堂等你。你需要在黎明前走完这条街，也走完这座城市的夜。"]),
        choices: &[
            ChoiceDef { label: "教堂 · 见道尔顿", sub: "L1 主线", cond: None, effects: &NO_EFF, route: Route::To("px_dalton") },
            ChoiceDef { label: "圣坛笔记", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("px_altar") },
            ChoiceDef { label: "钟楼调查", sub: "得知尖塔方位与时间", cond: None, effects: &NO_EFF, route: Route::To("px_belltower") },
            ChoiceDef { label: "废弃血站", sub: "血清情报", cond: None, effects: &NO_EFF, route: Route::To("px_bloodbank") },
            ChoiceDef { label: "空屋信件", sub: "背景补完", cond: None, effects: &NO_EFF, route: Route::To("px_apt_letter") },
            ChoiceDef { label: "店面废墟", sub: "血族银币线索", cond: None, effects: &NO_EFF, route: Route::To("px_store_relic") },
            ChoiceDef { label: "坍塌废墟", sub: "环境叙事", cond: None, effects: &NO_EFF, route: Route::To("px_ruin") },
            ChoiceDef { label: "地铁口 · 前往地下", sub: "L1→L2 单向向下", cond: None, effects: &NO_EFF, route: Route::To("px_metro") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_altar", bg: Some("poxiao_bg.png"), loc: Some("L1 · 圣坛笔记"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["教堂圣坛上压着一页手写的笔记，墨水干涸发褐：『苔丝尔德最后一天的日记——太阳落下第三年了。血族说这是新的秩序。可我记得，人类曾靠光活着。』"]),
        choices: &[ChoiceDef { label: "合上笔记", sub: "+5 点 · 人类记忆", cond: None, effects: &[Eff::Points(5)], route: Route::To("px_l1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_belltower", bg: Some("poxiao_bg.png"), loc: Some("L1 · 钟楼调查"),
        mood: "cold", speaker: Some("钟楼"), voice: None,
        text: TextSpec::Static(&["钟楼的指针悬在凌晨三时。锁着的钟面下一块黄铜铭牌刻着尖塔的方位——黎明尖塔，是把整座城市照亮的那个塔顶，也是你此行的终点。"]) ,
        choices: &[ChoiceDef { label: "记下方位与时间", sub: "+5 点 · 掐着黎明赶路", cond: None, effects: &[Eff::Points(5), Eff::SetFlag("px_phase_1")], route: Route::To("px_l1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_bloodbank", bg: Some("poxiao_bg.png"), loc: Some("L1 · 废弃血站终端"),
        mood: "cold", speaker: Some("道尔顿"), voice: None,
        text: TextSpec::Static(&["道尔顿在血站终端前调出泛蓝的档案：「血清原型……当年能让人重返阳光的那个配方。配方碎片被拆成三份——一份在这里，一份在叛军手里，一份锁在尖塔档案室。」"]),
        choices: &[ChoiceDef { label: "记下血清情报", sub: "Item it_px_plasma · +30 点 · px_blood_bank", cond: None,
            effects: &[Eff::SetFlag("px_blood_bank"), Eff::AddItem("it_px_plasma"), Eff::Points(30)], route: Route::To("px_l1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_apt_letter", bg: Some("poxiao_bg.png"), loc: Some("L1 · 空屋信件"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["公寓空屋的桌上压着一封信，落款是没来得及寄出的家书：『若你还能看到太阳，替我告诉爸，我没有怕。』窗外的血族无声地巡过。"]),
        choices: &[ChoiceDef { label: "把信揣好", sub: "+2 点 · 记下一个名字", cond: None, effects: &[Eff::Points(2), Eff::San(5)], route: Route::To("px_l1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_store_relic", bg: Some("poxiao_bg.png"), loc: Some("L1 · 店面废墟"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["店面废墟的收银台抽屉里散着几枚血族银币——这座城市仍在使用血族的货币，就像它仍在呼吸一样。你掂了掂，没有带走。"]),
        choices: &[ChoiceDef { label: "记下血族银币线索", sub: "+5 点 · 世界的一角", cond: None, effects: &[Eff::Points(5)], route: Route::To("px_l1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_ruin", bg: Some("poxiao_bg.png"), loc: Some("L1 · 坍塌废墟"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["一处坍塌的废墟压在路口，钢筋从残骸里刺出。太阳落下前，这里是街角的咖啡店。你绕过去，脚下是碎玻璃与落叶。"]),
        choices: &[ChoiceDef { label: "绕过废墟", sub: "不必停留", cond: None, effects: &NO_EFF, route: Route::To("px_l1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_metro", bg: Some("poxiao_bg.png"), loc: Some("L1 · 地铁口"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["地铁口吐着潮湿的凉气，阶梯一路向下通入黑暗。你在台阶口停顿——地下的世界，藏着这座城市的另一群人。"]),
        choices: &[ChoiceDef { label: "下阶梯", sub: "px_metro_portal · 单向进地下", cond: None, effects: &NO_EFF, route: Route::To("px_l2_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕 1 · 关键转折「沉沦者！」 ================= */
    SceneDef {
        id: "px_dalton", bg: Some("poxiao_bg.png"), loc: Some("L1 · 教会收容所"),
        mood: "danger", speaker: Some("爱德华·道尔顿"), voice: Some("vo_px_dalton"),
        text: TextSpec::Static(&[
            "道尔顿抬眼，白大褂下的脸色苍白得近乎透明，却带着一种学者式的疲惫：「你们是……主神空间的人？呵，我早该知道。听着——血清原型在市中心的血站里，日光镜阵在尖塔顶层，而塔里的东西……已经不是吸血鬼了。它叫沉沦者。」",
            "话音未落，教堂外传来指甲刮擦玻璃的声响——暗巷里的沉沦者循着人味围了上来。",
        ]),
        choices: &[
            ChoiceDef { label: "『我护送你。』", sub: "接受任务限制 · 教学战", cond: None, effects: &[Eff::SetFlag("px_anchored"), Eff::Points(10)], route: Route::To("px_deg_fight") },
            ChoiceDef { label: "『先说说你跟血族贵族的关系。』", sub: "盘问 → 埃德加线铺垫", cond: None, effects: &[Eff::SetFlag("px_int_edgar"), Eff::Points(10)], route: Route::To("px_deg_fight") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_deg_fight", bg: Some("poxiao_bg.png"), loc: Some("L1 · 教堂外暗巷"),
        mood: "danger", speaker: Some("沉沦者"), voice: None,
        text: TextSpec::Static(&["暗巷里的沉沦者撞开教堂后门，白惨惨的手指扣进木梁，双瞳血红地锁住你们。道尔顿往后一退：「别走远——半径之内，才是活路。」（遭遇战）"]),
        choices: &NO_CH, fight_id: Some("pc_degenerate"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_deg_after", bg: Some("poxiao_bg.png"), loc: Some("L1 · 教堂"),
        mood: "confidence", speaker: Some("道尔顿"), voice: None,
        text: TextSpec::Static(&["沉沦者抽搐着倒下。道尔顿蹲下捏起一撮灰：「看到了吗？它原本也是血族……它们失去了理智，成了行走的饥饿。我们得在太阳升起前，把那束光还给这座城。」"]),
        choices: &[ChoiceDef { label: "跟随道尔顿前往血站", sub: "血清原型情报", cond: None, effects: &NO_EFF, route: Route::To("px_bloodbank") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕 2 · 「地下！」· 三方势力抉择 ================= */
    SceneDef {
        id: "px_l2_arrive", bg: Some("poxiao_bg.png"), loc: Some("L2 · 地下排水道 · 到达点"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["阶梯尽头的黑暗里亮着几点油灯。下水道的潮气混着机油味，管道深处的滴水声单调如钟。远处隐隐传来人声——这里居然住了人。"]),
        choices: &[ChoiceDef { label: "循着人声前进", sub: "进入叛军据点", cond: None, effects: &NO_EFF, route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_l2_hub", bg: Some("poxiao_bg.png"), loc: Some("L2 · 叛军据点"),
        mood: "cold", speaker: Some("奥黛丽·班尼特"), voice: None,
        text: TextSpec::Static(&["油炸火光照亮一张张警惕又疲惫的人脸。奥黛丽·班尼特抱着步枪打量你：「人类要在天亮前把镜阵对准城里每一扇窗——这是我们的城市。」地下，是人类的最后一道防线。"]),
        choices: &[
            ChoiceDef { label: "通讯台 · 三方抉择", sub: "人类 / 温和血族 / 中立", cond: None, effects: &NO_EFF, route: Route::To("px_rebels") },
            ChoiceDef { label: "军械库", sub: "需人类路线许可", cond: None, effects: &NO_EFF, route: Route::To("px_armory") },
            ChoiceDef { label: "泵房控制台", sub: "阀门谜题 · 排空水闸", cond: None, effects: &NO_EFF, route: Route::To("px_pump_console") },
            ChoiceDef { label: "深水渠巢穴", sub: "沉沦者 · 血浆样本", cond: None, effects: &NO_EFF, route: Route::To("px_nest") },
            ChoiceDef { label: "发电机房", sub: "给货运电梯通电", cond: None, effects: &NO_EFF, route: Route::To("px_generator") },
            ChoiceDef { label: "主排水道巡逻", sub: "与叛军擦肩", cond: None, effects: &NO_EFF, route: Route::To("px_rebel_patrol") },
            ChoiceDef { label: "货运电梯", sub: "需发电机通电 · 上尖塔", cond: Some(cond_generator), effects: &NO_EFF, route: Route::To("px_l3_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_rebels", bg: Some("poxiao_bg.png"), loc: Some("L2 · 通讯台"),
        mood: "drama", speaker: Some("奥黛丽 + 无线电嘶响"), voice: None,
        text: TextSpec::Static(&["通讯台的数据灯明灭交替。奥黛丽按着枪：「人类要向天下黎明抢下一座城。」通讯器忽然嘶响，一个低沉柔滑的贵族腔插了进来：『血族贵族埃德加·冯·豪森请求通话。』",
            "你们面前摊着一道选择题——三方都想要这座城的黎明，但想要的黎明并不相同。没有对错，只有你选择站哪边。"]),
        choices: &[
            ChoiceDef { label: "帮人类叛军", sub: "SetFlag px_faction_human · +50 · 开军械库", cond: Some(cond_no_faction), effects: &[Eff::SetFlag("px_faction_human"), Eff::SetFlag("px_armory_open"), Eff::Points(50)], route: Route::To("px_rebel_oath") },
            ChoiceDef { label: "帮温和血族", sub: "SetFlag px_faction_moderate · +50 · 埃德加援助", cond: Some(cond_no_faction), effects: &[Eff::SetFlag("px_faction_moderate"), Eff::AddItem("it_px_sentry_charm"), Eff::Points(50)], route: Route::To("px_edgar_deal") },
            ChoiceDef { label: "中立独行", sub: "SetFlag px_faction_neutral · +80 · 两派都不欠", cond: Some(cond_no_faction), effects: &[Eff::SetFlag("px_faction_neutral"), Eff::Points(80)], route: Route::To("px_neutral") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_rebel_oath", bg: Some("poxiao_bg.png"), loc: Some("L2 · 叛军据点 · 誓约"),
        mood: "drama", speaker: Some("奥黛丽·班尼特"), voice: None,
        text: TextSpec::Static(&["奥黛丽把一枚灰白的手印按在你肩头，仿佛烙下一个盟约：「记住，是你的选择，把太阳还给这座城市的人类的。塔上镜阵的接入密钥，叛军永远站在你背后。」"]),
        choices: &[ChoiceDef { label: "接过盟约", sub: "+20 点 · 人类线", cond: None, effects: &[Eff::Points(20)], route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_edgar_deal", bg: Some("poxiao_bg.png"), loc: Some("L2 · 通讯台 · 埃德加的通话"),
        mood: "drama", speaker: Some("埃德加·冯·豪森"), voice: None,
        text: TextSpec::Static(&["低沉柔滑的贵族嗓音含着一丝笑意：「聪明的选择，离太阳太近的物种都会燃尽。你我做个交易——阳光会来，但要在血族的秩序之后。道尔顿的血清量产，归我。」",
            "通讯中断前，他留了个口信：「若你在塔上遇险，这支家室的血钟会认得你自己人。」你收好那枚暗红勋章。"]),
        choices: &[ChoiceDef { label: "收下血族援助", sub: "Item it_px_sentry_charm · +20 点", cond: None, effects: &[Eff::Points(20)], route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_neutral", bg: Some("poxiao_bg.png"), loc: Some("L2 · 通讯台 · 中立"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["你扯断了通讯线，谁的人情都没接。「这场战斗，我不替任何一边打。」奥黛丽沉默，埃德加的嗓音也淡了。你握紧武器——独行在这座黎明之城，自己认自己的路。"]),
        choices: &[ChoiceDef { label: "转身，独自前行", sub: "+20 点 · 独行者", cond: None, effects: &[Eff::Points(20)], route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_elvis", bg: Some("poxiao_bg.png"), loc: Some("L2 · 叛军爆破手埃尔维斯"),
        mood: "calm", speaker: Some("埃尔维斯"), voice: None,
        text: TextSpec::Static(&["埃尔维斯摆弄着一支无线电，咕哝道：「地下住久了，差点忘了太阳长啥样。兄弟，要是你真能送它回来，回头上我这儿来顿热的——我请你喝汤。」"]),
        choices: &[ChoiceDef { label: "记下这份人情", sub: "+10 点 · 世界的一角", cond: None, effects: &[Eff::Points(10)], route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_rebel_patrol", bg: Some("poxiao_bg.png"), loc: Some("L2 · 主排水道 · 叛军巡逻"),
        mood: "danger", speaker: Some("武装人类叛军"), voice: None,
        text: TextSpec::Static(&["主排水道阴影里，奥黛丽的人抬枪：「站住。叛军？血族？还是主神的狗？」枪口对着你，月光从格栅缝漏下来，一瞬间连空气都像绷紧了。"]),
        choices: &[
            ChoiceDef { label: "说明来意 / 出示血族勋章", sub: "两义势力 · 放行", cond: None, effects: &[Eff::Points(20)], route: Route::To("px_l2_hub") },
            ChoiceDef { label: "直接动手", sub: "战斗 pc_rebel_guard", cond: None, effects: &NO_EFF, route: Route::Dyn(px_rebel_fight) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_armory", bg: Some("poxiao_bg.png"), loc: Some("L2 · 叛军军械库"),
        mood: "cold", speaker: Some("埃尔维斯"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("px_armory_open") {
                "军械库的铁门对你敞开，弹药箱堆到齐肩。埃尔维斯拍拍一把圣水炸弹：「兄弟，这是给那些魍魉的见面礼——收好。」".to_string()
            } else {
                "军械库的钢板门焊死着，缝隙里透出弹药箱的气息。埃尔维斯耸耸肩：「这里只给咱们自己人开——想进来，下一道人类的许可，或带够血浆。」".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "领取圣水炸弹与弹药", sub: "Item it_px_holy_bomb · +50 · 需人类路线", cond: Some(cond_armory_open), effects: &[Eff::AddItem("it_px_holy_bomb"), Eff::Points(50)], route: Route::To("px_l2_hub") },
            ChoiceDef { label: "暂时离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("px_l2_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_pump_console", bg: Some("poxiao_bg.png"), loc: Some("L2 · 泵房控制台"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("px_valves") {
                "泵房的水已排干，水闸电机的轰鸣渐止。通往深水渠巢穴的闸口洞开，冷风从黑暗里灌进来。".to_string()
            } else {
                "泵房横着两根锈蚀的大阀门（A/B）与一台老式控制台。排水渠图上标着：三处齐备，方能排干水闸。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "扳动控制台排空积水", sub: "需 阀门A+阀门B+控制台 · 开水闸", cond: Some(cond_valve_done), effects: &[Eff::SetFlag("px_valves"), Eff::Points(30)], route: Route::To("px_l2_hub") },
            ChoiceDef { label: "查看阀门 A", sub: "管道阀门", cond: None, effects: &NO_EFF, route: Route::To("px_valve_a") },
            ChoiceDef { label: "查看阀门 B", sub: "管道阀门", cond: None, effects: &NO_EFF, route: Route::To("px_valve_b") },
            ChoiceDef { label: "离开泵房", sub: "", cond: None, effects: &NO_EFF, route: Route::To("px_l2_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_valve_a", bg: Some("poxiao_bg.png"), loc: Some("L2 · 泵房 · 阀门A"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["锈蚀的阀门 A 卡得死紧，你使上全身力气才扳动半圈，排水管里传来低沉的汩汩声。管线图上 A 格被划了一道。"]),
        choices: &[ChoiceDef { label: "扳开阀门A", sub: "px_valve_a · 排水进度", cond: None, effects: &[Eff::SetFlag("px_valve_a")], route: Route::To("px_pump_console") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_valve_b", bg: Some("poxiao_bg.png"), loc: Some("L2 · 泵房 · 阀门B"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["阀门 B 在暗角，扳手一拧，水声渐大。排水渠图上的最后一格光亮起来，只差把控制台合上电闸。"]),
        choices: &[ChoiceDef { label: "扳开阀门B", sub: "px_valve_b · 排水进度", cond: None, effects: &[Eff::SetFlag("px_valve_b"), Eff::SetFlag("px_valve_console")], route: Route::To("px_pump_console") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_nest", bg: Some("poxiao_bg.png"), loc: Some("L2 · 深水渠巢穴"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["排干水后的深水渠露出一片沉沦者的巢穴，骸骨与血浆结晶铺了一地。你在角落找到一个没被舔净的样本箱——里面的血浆还带着微温。"]),
        choices: &[ChoiceDef { label: "取走血浆样本", sub: "Item it_px_plasma · +25 · px_nest", cond: None, effects: &[Eff::AddItem("it_px_plasma"), Eff::SetFlag("px_nest"), Eff::Points(25)], route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_generator", bg: Some("poxiao_bg.png"), loc: Some("L2 · 发电机房"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["发电机房里引擎低沉，配电盘上「货运电梯」一格是暗的。你找到总闸，拧着散热阀把最后一路电送上电梯——指示灯哔地亮起。"]),
        choices: &[ChoiceDef { label: "给货运电梯通电", sub: "px_generator · 上尖塔门禁开通", cond: None, effects: &[Eff::SetFlag("px_generator"), Eff::Points(30)], route: Route::To("px_l2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "px_fight_rebel", bg: Some("poxiao_bg.png"), loc: Some("L2 · 主排水道 · 交火"),
        mood: "danger", speaker: Some("武装人类叛军"), voice: None,
        text: TextSpec::Static(&["一声叹息之后是利落的枪响——你没能说破来意，拔了刀。同样被永夜压着的一群人，此刻在枪口下对峙。（遭遇战 · px_rebel_blood）"]),
        choices: &NO_CH, fight_id: Some("pc_rebel_guard"), video: None, cine_label: None, overlay: None,
    },
    /* ================= 幕 3 · 「等待阳光」· L3 解密与镜阵 ================= */
    SceneDef {
        id: "px_l3_arrive", bg: Some("poxiao_bg.png"), loc: Some("L3 · 黎明尖塔 · 大堂"),
        mood: "danger", speaker: Some("道尔顿"), voice: None,
        text: TextSpec::Static(&["货运电梯在尖塔大堂停稳。玻璃穹顶透进来一线极淡的血橙，那是黎明的边缘色。道尔顿抬起被玻璃映亮的脸：「镜阵在顶层，钥匙在档案室——先弄清楚它的弱点，再上去送死。」"]),
        choices: &[ChoiceDef { label: "进入尖塔中层", sub: "实验区 / 档案室", cond: None, effects: &NO_EFF, route: Route::To("px_l3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_l3_hub", bg: Some("poxiao_bg.png"), loc: Some("L3 · 黎明尖塔 · 中层"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("px_dawn_mirror") {
                "镜阵列已校准，表面流转着晨曦般的光。决战平台的门缝里透出逼人的冷气——格里高尔已经醒了。" .to_string()
            } else if st.flag("px_archive") {
                "档案室的口令已解开，顶层闸门亮起绿光。镜阵的三面镜子还停在原位——校准它们，才能让日光真正灌进来。".to_string()
            } else {
                "尖塔中层冷光与管道错落。一面是实验区如笼，一面是档案室堆满纸页。你要在决战前，先替这束光找到钥匙。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "实验记录", sub: "温和血族线情报", cond: None, effects: &NO_EFF, route: Route::To("px_lab_log") },
            ChoiceDef { label: "机密档案", sub: "格里高尔弱点 · 开顶层闸", cond: None, effects: &NO_EFF, route: Route::To("px_archive") },
            ChoiceDef { label: "左镜", sub: "镜阵校准 1/3", cond: None, effects: &NO_EFF, route: Route::To("px_mirror_l") },
            ChoiceDef { label: "右镜", sub: "镜阵校准 2/3", cond: None, effects: &NO_EFF, route: Route::To("px_mirror_r") },
            ChoiceDef { label: "主控镜阵", sub: "镜阵校准 3/3", cond: None, effects: &NO_EFF, route: Route::To("px_mirror_c") },
            ChoiceDef { label: "接待台", sub: "环境叙事", cond: None, effects: &NO_EFF, route: Route::To("px_reception") },
            ChoiceDef { label: "登上决战平台", sub: "需档案弱点 · 决战前置", cond: Some(cond_archive), effects: &NO_EFF, route: Route::To("px_duel") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_reception", bg: Some("poxiao_bg.png"), loc: Some("L3 · 接待台"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["尖塔大堂的接待台落满细尘，登记簿停在太阳落下前最后一页。一名血族前台接待员的工牌还挂在椅背上，名牌旁边贴着一句打印的标语：「黎明之塔，服务永恒之夜。」"]),
        choices: &[ChoiceDef { label: "记下这句标语", sub: "+5 点 · 世界的一角", cond: None, effects: &[Eff::Points(5)], route: Route::To("px_l3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_lab_log", bg: Some("poxiao_bg.png"), loc: Some("L3 · 实验记录"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&["实验记录潦草的字迹翻到最后一页：「格里高尔，代号日间战士。血清原型与贵族血脉混合的完美实验体——但嗜血狂热无法抑制。档案室封存了它的全部弱点。」"]),
        choices: &[ChoiceDef { label: "记下血清日志", sub: "px_serum_log · +15 点", cond: None, effects: &[Eff::SetFlag("px_serum_log"), Eff::Points(15)], route: Route::To("px_l3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_archive", bg: Some("poxiao_bg.png"), loc: Some("L3 · 机密档案"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["档案最后一页用红字写着：『格里高尔，代号「日间战士」。实验失败。嗜血狂热无法抑制；唯一记录的弱点——直接日照下其再生崩溃。』",
            "远处钟楼遥遥敲响：凌晨四点。你了然于心——要打赢它，得先把阳光拆进这座塔。"]),
        choices: &[ChoiceDef { label: "解密档案 · 解开弱点", sub: "px_archive · 开顶层闸门 · +30", cond: None, effects: &[Eff::SetFlag("px_archive"), Eff::Points(30)], route: Route::To("px_l3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_mirror_l", bg: Some("poxiao_bg.png"), loc: Some("L3 · 决战平台 · 左镜"),
        mood: "calm", speaker: Some("道尔顿"), voice: None,
        text: TextSpec::Static(&["左镜的镜面蒙着岁月，你对准折射角缓缓拧动，一抹天光从穹顶漏进，在镜阵里来回跳了一格。道尔顿：「左边成了——下一个，右边。」"]),
        choices: &[ChoiceDef { label: "校准左镜", sub: "镜阵 1/3", cond: None, effects: &[Eff::SetFlag("px_mirror_l")], route: Route::To("px_l3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_mirror_r", bg: Some("poxiao_bg.png"), loc: Some("L3 · 决战平台 · 右镜"),
        mood: "calm", speaker: Some("道尔顿"), voice: None,
        text: TextSpec::Static(&["右镜的镜面反着你与道尔顿的倒影，一道光束又接上。道尔顿扶着镜框：「石阵的光已经连成一条线——只差把主控台的枢纽点亮了。」"]),
        choices: &[ChoiceDef { label: "校准右镜", sub: "镜阵 2/3", cond: None, effects: &[Eff::SetFlag("px_mirror_r")], route: Route::To("px_l3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_mirror_c", bg: Some("poxiao_bg.png"), loc: Some("L3 · 决战平台 · 主控镜阵"),
        mood: "calm", speaker: Some("道尔顿"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("px_mirror_l") && st.flag("px_mirror_r") {
                "你合上主控台的枢纽，三面镜阵在同一道晨光里同时亮起，折射的路像一条通往太阳的甬道。道尔顿攥紧拳：「齐了——格里高尔怕光，这束光，就是它的刑具。」".to_string()
            } else {
                "主控镜阵的枢纽还暗着。道尔顿皱眉：「得先把左右两面镜子对准——镜阵的光连不上，主控点亮也没用。」".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "点亮主控枢纽", sub: "需左右镜已校准 · px_dawn_mirror · +40", cond: Some(cond_mirrors_ready), effects: &[Eff::SetFlag("px_dawn_mirror"), Eff::Points(40)], route: Route::To("px_l3_hub") },
            ChoiceDef { label: "先行离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("px_l3_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕 4 · 决战「决战！……回归！」 ================= */
    SceneDef {
        id: "px_duel", bg: Some("poxiao_bg.png"), loc: Some("L3 · 决战平台 · 玻璃穹顶下"),
        mood: "danger", speaker: Some("格里高尔 + 道尔顿"), voice: Some("vo_px_duel"),
        text: TextSpec::Static(&[
            "顶层决战的玻璃穹顶漏进第一缕血橙的天光。格里高尔从镜阵下站起身，畸变的肌肉像活物一样蠕动：「血……还给我……」",
            "道尔顿举起手：「现在！校准主控——！」你环顾四周：三面镜阵，一道黎明，一头睡醒的噩梦。",
        ]),
        choices: &[
            ChoiceDef { label: "校准主控台，引日光入塔", sub: "需镜阵已校准 · 引日光 · px_sunray_ready", cond: Some(cond_calibrate_ok), effects: &[Eff::SetFlag("px_sunray_ready"), Eff::SetFlag("px_daylight")], route: Route::Dyn(start_gregor) },
            ChoiceDef { label: "直接开战", sub: "无镜阵前置 · 硬碰硬", cond: None, effects: &[Eff::SetFlag("px_daylight")], route: Route::Dyn(start_gregor) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "px_gregor_round", bg: Some("poxiao_bg.png"), loc: Some("L3 · 决战平台 · 与格里高尔缠斗"),
        mood: "danger", speaker: Some("格里高尔"), voice: None,
        text: TextSpec::Dyn(|st| {
            let hp = st.fight.as_ref().map(|f| f.hp.max(0)).unwrap_or(0);
            let state = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
                "　<b>——嗜血狂化！</b>血……还给我——！！".to_string()
            } else if st.flag("px_sunray_ready") {
                "　穹顶日光已被引入，格里高尔的再生在光照下微微迟滞。".to_string()
            } else {
                String::new()
            };
            format!("巨物在镜阵与晨光间野兽般腾跃，其残躯还剩 <b>{hp}</b> 气力。{state}")
        }),
        choices: &[
            ChoiceDef { label: "全力进攻", sub: "高杀伤 · 有风险", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| gregor_act(st, rng(28, 42), false)) },
            ChoiceDef { label: "稳定攻击", sub: "稳扎稳打", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| gregor_act(st, rng(16, 26), false)) },
            ChoiceDef { label: "卸力防守", sub: "格挡蓄势", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| gregor_act(st, rng(6, 10), true)) },
            ChoiceDef { label: "【引动日光射线】", sub: "需半血狂暴+镜阵校准 · 60 固伤", cond: Some(cond_sunray_strike), effects: &NO_EFF, route: Route::Dyn(gregor_sunray) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 结局三变体（按三方 flag 路由；均强制 px_sunrise） ================= */
    SceneDef {
        id: "px_end_human", bg: None, loc: None, mood: "calm", speaker: None, voice: Some("vo_px_end_human"),
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_px_end_human"), death: None,
            card: |_st| crate::state::Card {
                title: "人 类 晨 曦 之 约".into(), good: true,
                body_html: r#"<p>第一缕阳光穿过穹顶，镜阵把日光泼向整座城市。街道上，皮肤苍白的吸血鬼在晨光里颤抖、燃烧、然后——呼吸。</p>
<p>道尔顿站在窗前，声音发颤：「谢谢你们，把太阳还给了这座城市。」人类的地下，第一次有人看见自己的影子。</p>
<p style='color:#7CCD7C'>【破晓封锁区 · 人类线 · 通关】</p>
<p style='color:#ffd76a'>评定 S级 · 「救世者」</p>"#.to_string(),
                buttons: vec![("查 看 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "px_end_moderate", bg: None, loc: None, mood: "calm", speaker: None, voice: Some("vo_px_end_moderate"),
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_px_end_moderate"), death: None,
            card: |_st| crate::state::Card {
                title: "血 族 黄 昏 协 议".into(), good: true,
                body_html: r#"<p>埃德加的人接管了塔顶，日光在血族的秩序之后缓缓降临。「阳光会来，但要在血族的秩序之后。」他把一把钥匙推进你手心，静默良久。</p>
<p>道尔顿静静看着你：「你选了交易，还是换了个主人？」第一缕光照在每个人的脸上，却没有照亮答案。</p>
<p style='color:#7CCD7C'>【破晓封锁区 · 温和血族线 · 通关】</p>
<p style='color:#ffd76a'>评定 B级 · 「交易者」</p>"#.to_string(),
                buttons: vec![("查 看 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "px_end_neutral", bg: None, loc: None, mood: "calm", speaker: None, voice: Some("vo_px_end_neutral"),
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_px_end_neutral"), death: None,
            card: |_st| crate::state::Card {
                title: "独 行 者".into(), good: true,
                body_html: r#"<p>你走下塔时，城市正在变成两种颜色——燃烧的，和重生的。你没有选边，所以两边都不会记住你的名字。</p>
<p>只有钟楼记得：这个黎明，有人独自把它敲响了。</p>
<p style='color:#7CCD7C'>【破晓封锁区 · 独行线 · 通关】</p>
<p style='color:#ffd76a'>评定 C级 · 「独行者」徽记</p>"#.to_string(),
                buttons: vec![("查 看 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 死亡档案（复活扣 300/回主神） ================= */
    SceneDef {
        id: "px_death_deg", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("被沉沦者分食", "在破晓封锁区的暗巷与地下被失控的沉沦者撕碎")), card: |_st| crate::state::Card {
                title: "沉 沦 之 下".into(), good: false,
                body_html: r#"<p>混乱的爪牙掠过你的视野，永夜的城市的最后一眼是远处尚未亮起的尖塔。道尔顿的喊声被越拉越远。</p>
<p style='color:#ff8a8a'>【死亡档案 · 被沉沦者分食】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "px_death_gregor", bg: None, loc: None, mood: "danger", speaker: None, voice: Some("vo_px_death_gregor"),
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("被高级沉沦者撕碎", "在黎明尖塔顶层被高级沉沦者·格里高尔斩杀")), card: |_st| crate::state::Card {
                title: "格 里 高 尔 之 爪".into(), good: false,
                body_html: r#"<p>畸变的巨影遮住穹顶最后一缕天光，你没能把阳光引进来。尖塔的镜阵沉默地立在晨昏交界。</p>
<p style='color:#ff8a8a'>【死亡档案 · 被高级沉沦者撕碎】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "px_death_vamp", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("陨于黎明之城", "在破晓封锁区的城市与地下被血族势力击倒")), card: |_st| crate::state::Card {
                title: "永 夜 之 下".into(), good: false,
                body_html: r#"<p>这座城市不会为倒下的人类停下分秒。灯红酒绿的血站依旧，你的坐标又空了一个。</p>
<p style='color:#ff8a8a'>【死亡档案 · 陨于黎明之城】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "px_death_sunrise", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("暴晒尸潮", "太阳升起而镜阵未校准，整座城的沉沦者在晨光中疯狂相噬")), card: |_st| crate::state::Card {
                title: "暴 晒 尸 潮".into(), good: false,
                body_html: r#"<p>太阳升起，镜阵未校准。没有日光的城市在晨曦里失控，沉沦者互相撕咬的血潮把你卷入其中。</p>
<p style='color:#ff8a8a'>【死亡档案 · 暴晒尸潮（日光倒计时未校准）】</p>
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
fn px_rebel_fight(st: &mut GameState) -> String {
    st.set_flag("px_rebel_blood");
    "px_fight_rebel".to_string()
}
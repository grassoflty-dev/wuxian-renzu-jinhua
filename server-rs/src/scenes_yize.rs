//! 《大宇宙时代 · 远古遗迹·遗泽》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/dayuzhou_shidai/yiji_yize.md §5 BOSS / §6 剧情 / §9 素材。
//! 本文件是全新新增文件，只导出静态数据（YIZE_SCENES / yize_figths / 辅助查询），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/yize_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `yz_` 前缀，fight id 全部 `yz_` 前缀。
//! BOSS「遗迹仲裁者」采用「选择驱动遭遇链」落地（参考 scenes_jiguancheng.rs jc_colossus / scenes_mojiao.rs 教主模式）：
//! 因需要每回合并护盾系数 / 顺序交互 / 仲裁裁定等自定义每回合同调，引擎原生 FightCfg 无此钩子，
//! 故用 Normal 场景 + Route::Dyn 落地；同时导出 `yz_arbiter` FightCfg 供 ZoneDef 与揭示用。
//! 护盾四维护罩：flag 伪实现（§10 风险 1）。碎片座 S1-S4 按 SHIELD_ORDER=[3,1,4,2] 顺序交互；
//! 每关一片伤害系数 0.4→0.6→0.8→1.0，顺序错则复位全局碎片并触发相位冲击(Hurt 6)。
//! 抉择结局三分支（带走/留下/强夺）在仲裁者战后分叉，sp_grade 统一写 Some('D')。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   F1 遗迹外层 bg yz_bg_outer   （现用 img_laser.png 占位）
//!   F2 能量矩阵 bg yz_bg_matrix_hall （现用 img_redqueen.png 占位）
//!   F3 守卫引擎库 bg yz_bg_engine_bay （现用 img_zhuyuan_book.png 占位）
//!   F4 遗泽圣所 / 全息立壁 bg yz_bg_sanctum / yz_bg_legacy_holo （现用 img_redqueen.png / img_laser.png 占位）
//! 敌人立绘复用：guard→风化守护者/圣所哨兵、hunter→撕裂者/仲裁者相关精英；新美术由主 agent 统一生图替换。

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
fn cond_has_key1(st: &GameState) -> bool { inv(st, "relic_key1") }
fn cond_matrix_open(st: &GameState) -> bool { st.flag("yz_matrix_open") }
fn cond_engine_open(st: &GameState) -> bool { st.flag("yz_engine_open") }
fn cond_history(st: &GameState) -> bool { st.flag("yz_legacy_history") }

/* =====================================================================
   护盾四维护罩 · flag 伪实现（SHIELD_ORDER = S3→S1→S4→S2）
   ===================================================================== */
const SHIELD_ORDER: [i32; 4] = [3, 1, 4, 2];

fn shield_closed(st: &GameState) -> i32 {
    ["yz_s1_closed", "yz_s2_closed", "yz_s3_closed", "yz_s4_closed"]
        .iter().filter(|k| st.flag(k)).count() as i32
}

/// 伤害系数：0.4 + 0.2 × 已关碎片数（0→4 片对应 0.4→1.0）
fn shield_coeff(st: &GameState) -> f64 { 0.4 + 0.2 * shield_closed(st) as f64 }

fn ord_count(st: &GameState) -> i32 {
    ["yz_ord1", "yz_ord2", "yz_ord3", "yz_ord4"].iter().filter(|k| st.flag(k)).count() as i32
}

fn shield_reset(st: &mut GameState) {
    for k in ["yz_s1_closed", "yz_s2_closed", "yz_s3_closed", "yz_s4_closed",
              "yz_ord1", "yz_ord2", "yz_ord3", "yz_ord4"] {
        st.flags.insert(k.to_string(), false);
    }
}

/// 尝试关闭碎片座 idx（1..=4）。返回路由场景 id。
fn shield_act(st: &mut GameState, idx: i32) -> String {
    let ord = ord_count(st) as usize;
    if SHIELD_ORDER.get(ord) == Some(&idx) {
        let closed = match idx { 1 => "yz_s1_closed", 2 => "yz_s2_closed", 3 => "yz_s3_closed", _ => "yz_s4_closed" };
        st.set_flag(closed);
        st.set_flag(["yz_ord1", "yz_ord2", "yz_ord3", "yz_ord4"][ord]);
        if ord == 3 {
            st.set_flag("yz_unlock_order"); // 结算支线 +200
        }
        "yz_04_hall".to_string()
    } else {
        // 顺序错：复位全碎片 + 相位冲击(Hurt 6) + San-8
        shield_reset(st);
        st.san = (st.san - 8).clamp(0, 100);
        st.hp = (st.hp - 6).max(0);
        if st.hp <= 0 { "yz_lose_arb".to_string() } else { "yz_sh_wrong".to_string() }
    }
}
fn sh1_act(st: &mut GameState) -> String { shield_act(st, 1) }
fn sh2_act(st: &mut GameState) -> String { shield_act(st, 2) }
fn sh3_act(st: &mut GameState) -> String { shield_act(st, 3) }
fn sh4_act(st: &mut GameState) -> String { shield_act(st, 4) }

/* =====================================================================
   战斗配置表（id 全部 yz_ 前缀）。各敌 FightCfg.win 只用于 fight_id 直战场面；
   BOSS 仲裁者由选择驱动回合逻辑接管胜负，FightCfg.win 仅作兜底揭示。
   ===================================================================== */
fn yz_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn yz_win_passage(_st: &GameState) -> String { "yz_02_passage".to_string() }
fn yz_win_city(_st: &GameState) -> String { "yz_03_city".to_string() }
fn yz_win_engine(_st: &GameState) -> String { "yz_04_engine_hub".to_string() }
fn yz_win_sanctum(_st: &GameState) -> String { "yz_05_sanctum".to_string() }
fn yz_win_graverob(_st: &GameState) -> String { "yz_ending_graverob".to_string() }

pub fn yize_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("yz_sentinel", FightCfg {
            name: "风化守护者", hp: 38, dmg: (8, 14), reward: 12, reward_why: "击损风化门廊的自律兵器",
            intro: "一尊风化斑驳的石锈机械卫兵缓步拦路，单眼蓝灯扫过你——「检测：生命体。评估：掠夺者。」",
            rage_at: None, rage_text: "", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_passage, death: "yz_lose_arb",
        }),
        ("yz_drone", FightCfg {
            name: "维护哨无人机", hp: 42, dmg: (9, 15), reward: 15, reward_why: "击落扫描哨无人机",
            intro: "一枚球形冷蓝核心悬停半空，四轴悬臂低鸣，扫描灯由蓝转红——「警戒协议启动。」",
            rage_at: Some(18), rage_text: "扫描灯转红，无人机过载尖啸，能量流激射！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_passage, death: "yz_lose_arb",
        }),
        ("yz_phantom", FightCfg {
            name: "镜像幻影", hp: 46, dmg: (9, 15), reward: 18, reward_why: "驱散镜像幻影",
            intro: "半透明白色能量人形从矩阵光幕里走出，面无五官，行动如残影——它仿佛在对镜子里的你模仿。",
            rage_at: Some(22), rage_text: "镜像崩散又重组，下一击双影叠判！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_city, death: "yz_lose_arb",
        }),
        ("yz_ghost", FightCfg {
            name: "能量幽灵 · 守卫灵", hp: 78, dmg: (12, 19), reward: 90, reward_why: "镇灭能量之井的守卫灵（精英）",
            intro: "无面幽灵披风悬浮于能量井上，内部星点漩涡缓缓旋转，下摆消散为能量粒——「继承者？还是……猎食者？」",
            rage_at: Some(35), rage_text: "守卫灵相位闪烁，身影忽明忽暗，闪避骤升！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_city, death: "yz_lose_arb",
        }),
        ("yz_heavy", FightCfg {
            name: "重装卫兵", hp: 55, dmg: (11, 17), reward: 22, reward_why: "压制兵舍重装卫兵",
            intro: "双肩炮管的金属重甲自律兵器轰然站起，履带底盘碾过石地，装甲接缝透出橙红火光。",
            rage_at: Some(25), rage_text: "装甲碎裂，露出过载的能量芯——伤害骤增！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_engine, death: "yz_lose_arb",
        }),
        ("yz_ripper", FightCfg {
            name: "兵舍近卫 · 撕裂者", hp: 95, dmg: (14, 21), reward: 120, reward_why: "击毁近卫 · 撕裂者（精英）",
            intro: "双链锯臂的尖兵微倾前冲，周身冷蓝电弧噼啪作响——它等待唤醒的时刻，已经等得太久。",
            rage_at: Some(45), rage_text: "双链锯臂完全展开，嘶鸣连击概率激增！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_engine, death: "yz_lose_arb",
        }),
        ("yz_sentry", FightCfg {
            name: "圣所哨兵", hp: 50, dmg: (10, 16), reward: 20, reward_why: "清除圣所哨兵",
            intro: "守卫遗泽入口的哨兵冷光乍亮，机械面甲居高临下——它以自身为最后一道岗。",
            rage_at: Some(24), rage_text: "冷光过载，哨兵攻击带着灼热的能量尾迹！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_sanctum, death: "yz_lose_arb",
        }),
        ("yz_ripper_phase", FightCfg {
            name: "相位撕裂者 · 强化", hp: 100, dmg: (15, 22), reward: 130, reward_why: "击杀相位撕裂者（精英）",
            intro: "圣所入口的相位撕裂者周身冷紫电弧缠绕，链锯臂在半空都有重影——它比兵舍的同类更凶悍。",
            rage_at: Some(48), rage_text: "相位突进，先手概率骤升，重影三叠！", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_sanctum, death: "yz_lose_arb",
        }),
        ("yz_arbiter", FightCfg {
            name: "遗迹仲裁者", hp: 180, dmg: (16, 25), reward: 300, reward_why: "通过九级文明自律执刑者的最后考试",
            intro: "无面苍白机械像自祭坛缓缓升起，周身悬浮四块四维护罩菱形碎片，胸口冷蓝核心幽然发亮。它不是生物——是「最后一课」的考官。",
            rage_at: Some(60), rage_text: "仲裁者石锈橙过载光暴涨，胸口核心转红，嗓音化为警告——「警告。熵值超限。考卷作废——」",
            on_rage: yz_rage_none,
            finisher_if: |st, hp| shield_closed(st) == 4 && hp < 45, // 四碎片全关 && HP<45 → 仲裁裁定
            finisher_name: |_| "仲裁裁定".to_string(),
            finisher_desc: |_| "胸口冷蓝核心骤然爆裂，四块护盾碎片同时亮起又熄灭。仲裁者空洞的嗓音回荡：「裁定——继承权确认。」".to_string(),
            win: |_st| "yz_05_ending_choice".to_string(),
            death: "yz_lose_arb",
        }),
        ("yz_arb_remnant", FightCfg {
            name: "仲裁者残骸", hp: 60, dmg: (16, 25), reward: 150, reward_why: "镇压仲裁者残骸的二次再起",
            intro: "你挥向祭坛光球的一击，竟唤醒仲裁者崩裂的残骸——它机械地举起臂刃，那是它最后的一课。",
            rage_at: None, rage_text: "", on_rage: yz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yz_win_graverob, death: "yz_lose_arb",
        }),
    ]
}

/* =====================================================================
   BOSS · 遗迹仲裁者（选择驱动遭遇）
   血量存 st.fight（yz_d_altar 的 Route::Dyn 初始化，引用 yz_arbiter 的 FightCfg）。
   每"回"是 Normal 场景 yz_arb_round；Route::Dyn 统一处理：护盾系数、扣血、狂暴、终结算裁定、胜负。
   ===================================================================== */
fn start_arbiter(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("yz_arbiter") {
            st.fight = Some(crate::power::scaled_fight("yz_arbiter", cfg, st, vec![format!("<span class='miss'>{}</span>", cfg.intro)]));
        }
    }
    "yz_arb_round".to_string()
}

/// 仲裁者胜场结算：+300、遗泽核心、sp_grade=D、置 killed；若 unlock_order 再 +200。
fn arb_win(st: &mut GameState) -> String {
    st.points += 300;
    crate::world::add_item(st, "legacy_core");
    if st.flag("yz_unlock_order") {
        st.points += 200;
    }
    st.sp_grade = Some('D');
    st.set_flag("yz_arbiter_defeated");
    "yz_5_arb_win".to_string()
}

fn arb_dead() -> String { "yz_lose_arb".to_string() }

/// 一个"回"：玩家攻击仲裁者。护盾开启时伤害按系数折减（0.4→1.0）。
fn arb_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    let coeff = shield_coeff(st);
    if !guard {
        let eff = (dmg as f64 * coeff) as i32;
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - eff.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return arb_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 60).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let rg = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if rg { rng(20, 30) } else { rng(16, 25) };
    let dodge = if guard { 0.5 } else { 0.15 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 { return arb_dead(); }
    // 终结：四碎片全关 && HP<45 → 仲裁裁定
    let closed = shield_closed(st) == 4;
    let low = st.fight.as_ref().map(|f| f.hp < 45).unwrap_or(false);
    if closed && low { return "yz_arb_finisher".to_string(); }
    "yz_arb_round".to_string()
}

/* =====================================================================
   强夺 · 仲裁者残骸（选择驱动）HP60，无护盾系数
   ===================================================================== */
fn start_remnant(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("yz_arb_remnant") {
            st.fight = Some(crate::power::scaled_fight("yz_arb_remnant", cfg, st, vec![]));
        }
    }
    "yz_ga_round".to_string()
}

fn graverob_win(st: &mut GameState) -> String {
    st.points += 150;
    crate::world::add_item(st, "legacy_shard");
    st.san = (st.san - 20).clamp(0, 100);
    st.set_flag("yz_legacy_graverob");
    // 灰色结局：不计结算支线 flag 中的结局项（§6.3）
    "yz_ending_graverob".to_string()
}

fn ga_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return graverob_win(st);
    }
    let raw = rng(16, 25);
    let dodge = if guard { 0.5 } else { 0.15 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 { return arb_dead(); }
    "yz_ga_round".to_string()
}

/* =====================================================================
   剧情场景（id 全部 yz_ 前缀）
   ===================================================================== */
pub static YIZE_SCENES: &[SceneDef] = &[

    /* ------------------- 幕一 · 踏入神迹（F1） ------------------- */
    SceneDef {
        id: "yz_01_arrive", bg: Some("yize_bg.png"), loc: Some("F1 遗迹外层 · 尘封巨门前厅"),
        mood: "awe", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "<b>【主线任务】</b>追踪高等文明信号进入远古遗迹，破解 4 层机关迷城，击败自律兵器「遗迹仲裁者」，在圣所作出最后抉择。",
            "希望的星图在金属天花板上流转，风化柱廊与尘封巨门在冷蓝光里静静矗立。这不是殿堂——是前人留下的火。",
            "「停。前面那扇门……它在看我们。」张恒的预知刺痛，低低地警示你。",
            "念夕空轻声呢喃：「这里……很久没有'人'了，全是冷的。」",
        ]),
        choices: &[
            ChoiceDef { label: "让鹰前出侦察", sub: "San(+5) · 侦查入口", cond: None,
                effects: &[Eff::SetFlag("yz_f1_scout"), Eff::San(5)], route: Route::To("yz_f1_drone_fight") },
            ChoiceDef { label: "先读观测终端", sub: "+20点 · 观测室线索", cond: None,
                effects: &NO_EFF, route: Route::To("yz_d_obs") },
            ChoiceDef { label: "直接推门而入", sub: "San(-5) · 陷阱区强行闯入", cond: None,
                effects: &[Eff::San(-5)], route: Route::To("yz_f1_trapfight") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_obs", bg: Some("yize_bg.png"), loc: Some("F1 柱廊北 · 观测室"),
        mood: "mystery", speaker: Some("尊主（全息残影）"), voice: None,
        text: TextSpec::Static(&[
            "观测终端的屏幕骤然亮起，一段自称「尊主」的文明预告影像缓缓浮现——面孔模糊，语带倦意。",
            "「若你们读到这里，我们大约是死了。别怕，这里没有诅咒——只有一道考试。」",
            "「遗泽有两样：一颗种子，和一块石碑。后来者，你们要准备好回答：你们配拿走哪一样？」",
        ]),
        choices: &[
            ChoiceDef { label: "记下这段预告", sub: "+20 点 · 线索", cond: None,
                effects: &[Eff::SetFlag("yz_relic_prologue"), Eff::Points(20)], route: Route::To("yz_d_trap") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_supply", bg: Some("yize_bg.png"), loc: Some("F1 补给舱"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "补给舱的柜门被风化锁锈住，撬开后，一枚刻着「文明遗迹·Lv1」的权限卡静静躺在里面。",
        ]),
        choices: &[
            ChoiceDef { label: "取走权限卡", sub: "得到 遗迹权限卡·Lv1", cond: None,
                effects: &[Eff::AddItem("relic_key1")], route: Route::To("yz_02_passage") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_n_zhangheng", bg: Some("yize_bg.png"), loc: Some("F1 观测室旁"),
        mood: "tension", speaker: Some("张恒 · 预知者"), voice: None,
        text: TextSpec::Static(&["张恒闭眼片刻，眉心紧锁：「它们没骗人——骗人的是我们自己，如果我们贪的话。」"]),
        choices: &[ChoiceDef { label: "（回到走廊）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_02_passage") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_trap", bg: Some("yize_bg.png"), loc: Some("F1 扫描陷阱回廊"),
        mood: "fear", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "冷蓝的扫描灯在回廊两侧一明一暗，墙面上布满风化的能量槽——踩错一步，就会触发全场警报。",
            "扫描灯一明一暗，你注意到散射路径之间，有一条可以逐个绕行的缝。",
        ]),
        choices: &[
            ChoiceDef { label: "观察 → 逐个绕行", sub: "避开陷阱 · +200结算支线", cond: None,
                effects: &[Eff::SetFlag("yz_f1_stealth"), Eff::Points(10)], route: Route::To("yz_02_passage") },
            ChoiceDef { label: "强行闯阵", sub: "触发陷阱守卫群战", cond: None,
                effects: &NO_EFF, route: Route::To("yz_f1_trapfight") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_f1_drone_fight", bg: Some("yize_bg.png"), loc: Some("F1 巨门门厅"),
        mood: "fear", speaker: Some("维护哨无人机"), voice: None,
        text: TextSpec::Static(&["鹰前出侦察，视野开阔处，一枚维护哨无人机骤然亮起红灯扑来——伏击！"]),
        choices: &NO_CH, fight_id: Some("yz_drone"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_f1_trapfight", bg: Some("yize_bg.png"), loc: Some("F1 扫描陷阱回廊"),
        mood: "danger", speaker: Some("风化守护者"), voice: None,
        text: TextSpec::Static(&["警报炸响，两尊风化守护者自墙面升起，封锁了你的退路。"]),
        choices: &NO_CH, fight_id: Some("yz_sentinel"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_02_passage", bg: Some("yize_bg.png"), loc: Some("F1 主厅 · 巨门前"),
        mood: "awe", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["巨门高耸，中央的权限凹槽无声地等待。柱廊尽头的两个裂缝，可作偷渡捷径直落 F2。"]),
        choices: &[
            ChoiceDef { label: "前往补给舱", sub: "取 权限卡Lv1", cond: None, effects: &NO_EFF, route: Route::To("yz_d_supply") },
            ChoiceDef { label: "与张恒交谈", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_n_zhangheng") },
            ChoiceDef { label: "走向尘封巨门", sub: "需 权限卡Lv1", cond: Some(cond_has_key1), effects: &NO_EFF, route: Route::To("yz_02_gate1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_02_gate1", bg: Some("yize_bg.png"), loc: Some("F1 尘封巨门 · G1"),
        mood: "awe", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["你以权限卡扫过凹槽，巨门内部传来齿轮的低吼，尘封千百年的通道缓缓敞开，向下通往 F2 中庭的冷蓝光。"]),
        choices: &[ChoiceDef { label: "沿阶而下 · F2", sub: "进入能量矩阵大厅", cond: None, effects: &NO_EFF, route: Route::To("yz_03_city") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 幕二 · 矩阵与齿轮（F2/F3） ------------------- */
    SceneDef {
        id: "yz_03_city", bg: Some("yize_bg.png"), loc: Some("F2 中庭 · 能量矩阵大厅"),
        mood: "tension", speaker: Some("念夕空"), voice: None,
        text: TextSpec::Static(&["冷蓝网格铺满大厅，四角矩阵核心微微发光。「它们在等一个顺序。」念夕空低语。"]),
        choices: &[
            ChoiceDef { label: "破解能量矩阵", sub: "顺序谜题 · +40点", cond: None, effects: &NO_EFF, route: Route::To("yz_d_matrix") },
            ChoiceDef { label: "取能量电池", sub: "供电闸门捷径", cond: None, effects: &NO_EFF, route: Route::To("yz_d_battery") },
            ChoiceDef { label: "读全息档案台", sub: "序列镜像线索", cond: None, effects: &NO_EFF, route: Route::To("yz_d_archive") },
            ChoiceDef { label: "镇能量之井", sub: "精英守卫灵", cond: None, effects: &NO_EFF, route: Route::To("yz_f2_ghost_fight") },
            ChoiceDef { label: "走向北闸口", sub: "需 矩阵完成 或 电池", cond: Some(cond_matrix_open), effects: &NO_EFF, route: Route::To("yz_03_gate2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_battery", bg: Some("yize_bg.png"), loc: Some("F2 西翼 · 能量电池库"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["电池库的货架上，一枚充能饱满的能量电池发出柔和的白光。" ]),
        choices: &[ChoiceDef { label: "取走能量电池", sub: "解锁北闸口（OR）", cond: None,
            effects: &[Eff::AddItem("energy_cell"), Eff::SetFlag("yz_matrix_open")], route: Route::To("yz_03_city") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_archive", bg: Some("yize_bg.png"), loc: Some("F2 东翼 · 记录档案馆"),
        mood: "mystery", speaker: Some("全息档案台"), voice: None,
        text: TextSpec::Static(&["档案台投影出一串映像序列，画面上四枚核心依次亮起——那是「镜像」的答案：S3 → S1 → S4 → S2。"]),
        choices: &[ChoiceDef { label: "记下序列", sub: "矩阵线索", cond: None, effects: &NO_EFF, route: Route::To("yz_03_city") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_matrix", bg: Some("yize_bg.png"), loc: Some("F2 中央矩阵大厅"),
        mood: "tension", speaker: Some("矩阵核心"), voice: None,
        text: TextSpec::Static(&["四枚矩阵核心亮起，冷蓝符文在你脚下列成阵列。念夕空：「它们在等一个顺序。」"]),
        choices: &[
            ChoiceDef { label: "按档案序列点亮", sub: "顺序正确 · +40点", cond: None,
                effects: &[Eff::SetFlag("yz_matrix_order"), Eff::Points(40), Eff::SetFlag("yz_matrix_open")],
                route: Route::To("yz_03_city") },
            ChoiceDef { label: "乱序强行点灯", sub: "San(-8) · 镜像幻影增援", cond: None,
                effects: &[Eff::San(-8), Eff::SetFlag("yz_matrix_open")], route: Route::To("yz_f2_phantom_fight") },
            ChoiceDef { label: "求助任涛解析", sub: "得正确序 · 守卫灵警觉", cond: None,
                effects: &[Eff::SetFlag("yz_matrix_hint"), Eff::SetFlag("yz_matrix_order"), Eff::SetFlag("yz_matrix_open")],
                route: Route::To("yz_03_city") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_f2_phantom_fight", bg: Some("yize_bg.png"), loc: Some("F2 中央矩阵大厅"),
        mood: "fear", speaker: Some("镜像幻影"), voice: None,
        text: TextSpec::Static(&["乱序点灯激起能量乱流，两具镜像幻影自光幕中析出，对你发起增援战。"]),
        choices: &NO_CH, fight_id: Some("yz_phantom"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_f2_ghost_fight", bg: Some("yize_bg.png"), loc: Some("F2 能量之井"),
        mood: "fear", speaker: Some("能量幽灵 · 守卫灵"), voice: None,
        text: TextSpec::Static(&["能量之井幽光暴涨，守卫灵铺开幽灵披风拦在井前——这是中庭最强的守卫。"]),
        choices: &NO_CH, fight_id: Some("yz_ghost"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_03_gate2", bg: Some("yize_bg.png"), loc: Some("F2 北闸口 · G2"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["闸口控制台获得供给，指示灯次第亮起，升降闸门轰然升开，深井内透出更冷的气息。"]),
        choices: &[ChoiceDef { label: "沿升降闸上行 · F3", sub: "进入守卫引擎库", cond: None, effects: &NO_EFF, route: Route::To("yz_04_engine_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_04_engine_hub", bg: Some("yize_bg.png"), loc: Some("F3 深廓 · 守卫引擎库"),
        mood: "tension", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["引擎库的嗡鸣像一颗缓缓跳动的心脏。兵舍舱一排排待机——等待唤醒指令。三枚扳手分立三处，是断能的关键。"]),
        choices: &[
            ChoiceDef { label: "引擎操控台", sub: "解读断能流程", cond: None, effects: &NO_EFF, route: Route::To("yz_d_engine") },
            ChoiceDef { label: "扳手 ⅰ", sub: "第一步断电", cond: None, effects: &NO_EFF, route: Route::To("yz_d_wrench1") },
            ChoiceDef { label: "扳手 ⅱ", sub: "第二步断电", cond: None, effects: &NO_EFF, route: Route::To("yz_d_wrench2") },
            ChoiceDef { label: "扳手 ⅲ", sub: "第三步断电", cond: None, effects: &NO_EFF, route: Route::To("yz_d_wrench3") },
            ChoiceDef { label: "排热阀", sub: "泄压", cond: None, effects: &NO_EFF, route: Route::To("yz_d_valve") },
            ChoiceDef { label: "走向轨道闸", sub: "需 三步断电完成", cond: Some(cond_engine_open), effects: &NO_EFF, route: Route::To("yz_04_gate3") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_engine", bg: Some("yize_bg.png"), loc: Some("F3 中央引擎主舱"),
        mood: "tension", speaker: Some("全息标牌"), voice: None,
        text: TextSpec::Static(&["引擎操控台全息标牌闪烁：「三处扳手缺失，主供电未断，兵舍同步待唤醒。」每断一步，东面兵舍便惊醒一只重装卫兵。"]),
        choices: &[ChoiceDef { label: "记下流程", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_04_engine_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_wrench1", bg: Some("yize_bg.png"), loc: Some("F3 中央引擎主舱"),
        mood: "tension", speaker: Some("扳手 ⅰ"), voice: None,
        text: TextSpec::Static(&["你扳下第一枚扳手，引擎声一颤，东兵舍一座舱门轰然弹开——重装卫兵苏醒了。"]),
        choices: &[ChoiceDef { label: "继续断电", sub: "", cond: None,
            effects: &[Eff::SetFlag("yz_w1")], route: Route::Dyn(engine_step) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_wrench2", bg: Some("yize_bg.png"), loc: Some("F3 中央引擎主舱"),
        mood: "tension", speaker: Some("扳手 ⅱ"), voice: None,
        text: TextSpec::Static(&["第二枚扳手被扳下，第二座兵舍舱苏醒，灯光一点点暗下去。"]),
        choices: &[ChoiceDef { label: "继续断电", sub: "", cond: None,
            effects: &[Eff::SetFlag("yz_w2")], route: Route::Dyn(engine_step) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_wrench3", bg: Some("yize_bg.png"), loc: Some("F3 中央引擎主舱"),
        mood: "tension", speaker: Some("扳手 ⅲ"), voice: None,
        text: TextSpec::Static(&["最后一枚扳手被扳下，主引擎彻底断电，整座兵舍由震动转为死寂——沉睡。"]),
        choices: &[ChoiceDef { label: "完成断电", sub: "", cond: None,
            effects: &[Eff::SetFlag("yz_w3")], route: Route::Dyn(engine_step) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_valve", bg: Some("yize_bg.png"), loc: Some("F3 排热阀"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["阀门被拧松，一股灼热蒸汽泄出，廊道里弥漫起金属尘的味道。「遗泽不是礼物，是检疫。」一声人类语言残响混在嗡鸣里。"]),
        choices: &[ChoiceDef { label: "记下这句残响", sub: "伏笔", cond: None, effects: &NO_EFF, route: Route::To("yz_04_engine_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_04_gate3", bg: Some("yize_bg.png"), loc: Some("F3 北升降轨道闸 · G3"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["三路断电指令校验通过，轨道闸嗡鸣退开，升降平台在深井尽头亮起一星冷蓝的光。"]),
        choices: &[ChoiceDef { label: "乘平台上行 · F4", sub: "进入遗泽圣所", cond: None, effects: &NO_EFF, route: Route::To("yz_05_sanctum") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 幕三 · 遗泽真相（F4 全息立壁） ------------------- */
    SceneDef {
        id: "yz_05_sanctum", bg: Some("yize_bg.png"), loc: Some("F4 入口大厅 · 遗泽圣所"),
        mood: "revelation", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["圣所的冷蓝光照亮十字大殿，全息立壁立在殿东北端，祭坛在中央沉默地等待。北端隐约可见通关传送门的幽光。"]),
        choices: &[
            ChoiceDef { label: "读取全息立壁", sub: "遗泽真相", cond: None, effects: &NO_EFF, route: Route::To("yz_d_holo") },
            ChoiceDef { label: "与念夕空交谈", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_n_nianxikong") },
            ChoiceDef { label: "进入十字殿", sub: "需 读过遗泽纪录", cond: Some(cond_history), effects: &NO_EFF, route: Route::To("yz_04_hall") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_n_nianxikong", bg: Some("yize_bg.png"), loc: Some("F4 圣所入口"),
        mood: "revelation", speaker: Some("念夕空 · 传递者"), voice: None,
        text: TextSpec::Static(&["念夕空望着祭坛方向，轻声：「能把火留到今天的文明，自己却没能等到明天。我们走对了路，可路也到尽头了。」"]),
        choices: &[ChoiceDef { label: "（回到圣所前厅）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_05_sanctum") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_holo", bg: Some("yize_bg.png"), loc: Some("F4 全息立壁 · 遗泽纪录"),
        mood: "revelation", speaker: Some("尊主（全息残影）"), voice: None,
        text: TextSpec::Static(&[
            "全息立壁亮起，尊主的残影眉眼模糊、语带疲惫：「我们是八级往上的文明。我们以为能走到第九级。然后我们听到了那首歌——银色之物唱的歌。」",
            "「遗泽有两样：一颗种子，和一块石碑。试卷只有一题：后来者，你们配拿走哪一样？」",
            "（九级「毁灭之歌」与银色战争的伏笔，随画面淡去。）",
        ]),
        choices: &[
            ChoiceDef { label: "完整听完", sub: "San(-5) · +30点 · 解锁殿门", cond: None,
                effects: &[Eff::SetFlag("yz_legacy_history"), Eff::San(-5), Eff::Points(30)], route: Route::To("yz_05_sanctum") },
            ChoiceDef { label: "质疑真实性，二次扫描", sub: "San(-10) · 同样解锁殿门", cond: None,
                effects: &[Eff::SetFlag("yz_legacy_history"), Eff::San(-10)], route: Route::To("yz_05_sanctum") },
            ChoiceDef { label: "转身就走，不听", sub: "殿门不解锁 · 惩罚线", cond: None,
                effects: &NO_EFF, route: Route::To("yz_05_sanctum") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 十字殿 · 护盾碎片顺序 + 祭坛 BOSS ------------------- */
    SceneDef {
        id: "yz_04_hall", bg: Some("yize_bg.png"), loc: Some("F4 十字殿 · 四维护盾碎片阵"),
        mood: "tension", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["护盾碎片座在殿中浮起，四座菱形碎片安静地等待着「顺序」。依照遗泽纪录，它们必须是：S3 → S1 → S4 → S2。"]),
        choices: &[
            ChoiceDef { label: "关闭碎片座 S3", sub: "第一片", cond: None, effects: &NO_EFF, route: Route::To("yz_d_sh3") },
            ChoiceDef { label: "关闭碎片座 S1", sub: "第二片", cond: None, effects: &NO_EFF, route: Route::To("yz_d_sh1") },
            ChoiceDef { label: "关闭碎片座 S4", sub: "第三片", cond: None, effects: &NO_EFF, route: Route::To("yz_d_sh4") },
            ChoiceDef { label: "关闭碎片座 S2", sub: "第四片", cond: None, effects: &NO_EFF, route: Route::To("yz_d_sh2") },
            ChoiceDef { label: "走向祭坛", sub: "迎战 遗迹仲裁者", cond: None, effects: &NO_EFF, route: Route::To("yz_d_altar") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_sh1", bg: Some("yize_bg.png"), loc: Some("F4 护盾碎片座 S1"),
        mood: "tension", speaker: Some("碎片座 S1"), voice: None,
        text: TextSpec::Static(&["你伸手触碰 S1 座。若顺序正确，菱形碎片将熄灭；若错误，全场相位冲击。"]),
        choices: &[ChoiceDef { label: "尝试关闭", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(sh1_act) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_sh2", bg: Some("yize_bg.png"), loc: Some("F4 护盾碎片座 S2"),
        mood: "tension", speaker: Some("碎片座 S2"), voice: None,
        text: TextSpec::Static(&["S2 座上的菱形碎片正在缓缓自转。"]),
        choices: &[ChoiceDef { label: "尝试关闭", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(sh2_act) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_sh3", bg: Some("yize_bg.png"), loc: Some("F4 护盾碎片座 S3"),
        mood: "tension", speaker: Some("碎片座 S3"), voice: None,
        text: TextSpec::Static(&["S3 座是一切顺序的起点——先辈在第一片碎片里埋下了教训。"]),
        choices: &[ChoiceDef { label: "尝试关闭", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(sh3_act) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_sh4", bg: Some("yize_bg.png"), loc: Some("F4 护盾碎片座 S4"),
        mood: "tension", speaker: Some("碎片座 S4"), voice: None,
        text: TextSpec::Static(&["S4 座离祭坛最近，碎片里映着仲裁者的冷蓝核心。"]),
        choices: &[ChoiceDef { label: "尝试关闭", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(sh4_act) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_sh_wrong", bg: Some("yize_bg.png"), loc: Some("F4 十字殿"),
        mood: "danger", speaker: Some("仲裁者（广播）"), voice: None,
        text: TextSpec::Static(&["「顺序判定——错误。」四块碎片同时迸亮复现，全场相位冲击轰然扫过（6 点伤害），已关碎片尽数复位。" ]),
        choices: &[ChoiceDef { label: "（重整碎片阵）", sub: "重新尝试顺序", cond: None, effects: &NO_EFF, route: Route::To("yz_04_hall") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_d_altar", bg: Some("yize_bg.png"), loc: Some("F4 祭坛 · 仲裁者"),
        mood: "danger", speaker: Some("遗迹仲裁者"), voice: None,
        text: TextSpec::Static(&["你走近祭坛，无面苍白机械像自中央缓缓升起，四块四维护罩碎片在周身悬浮。它空洞的嗓音响起：\n「检测：生命体。评估：掠夺者。仲裁开始。」"]),
        choices: &[ChoiceDef { label: "【迎战仲裁者】", sub: "进入四维护罩战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_arbiter) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_arb_round", bg: Some("yize_bg.png"), loc: Some("F4 圣所 · 仲裁者"),
        mood: "danger", speaker: Some("遗迹仲裁者"), voice: None,
        text: TextSpec::Dyn(|st| {
            let coeff = shield_coeff(st);
            let closed = shield_closed(st);
            let rg = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
            let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(0);
            format!("护盾系数 ×{:.1}（已关 {} 片）。仲裁者核心 {}。{}{}",
                coeff, closed,
                if rg { "转为红橙过载光……「警告。熵值超限——」".to_string() } else { "冷蓝幽然".to_string() },
                if closed == 4 { "四片碎片全灭，伤害系数已抵 1.0。".to_string() } else { "护盾未全关，攻击被四维偏折。".to_string() },
                format!("<br/>当前仲裁者 HP {}。", hp))
        }),
        choices: &[
            ChoiceDef { label: "重击", sub: "全力一击（受护盾系数折减）", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| arb_act(st, rng(28, 34), false)) },
            ChoiceDef { label: "蓄力强攻", sub: "更高但更慢", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| arb_act(st, rng(36, 42), false)) },
            ChoiceDef { label: "防御", sub: "减伤 · 蓄守势", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| arb_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_arb_finisher", bg: Some("yize_bg.png"), loc: Some("F4 圣所 · 仲裁裁定"),
        mood: "choice", speaker: Some("遗迹仲裁者"), voice: None,
        text: TextSpec::Static(&[
            "四块护盾碎片同时亮起又熄灭。仲裁者胸口冷蓝核心骤然爆裂，化作一片光雨。它空洞的声音终于带上一丝温度：",
            "「裁定：继承权确认。你们……配得上那团火。」过载的机械像缓缓跪落、静默宕机。",
        ]),
        choices: &[ChoiceDef { label: "接受裁定", sub: "获取 遗泽核心", cond: None,
            effects: &NO_EFF, route: Route::Dyn(arb_win) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_5_arb_win", bg: Some("yize_bg.png"), loc: Some("F4 圣所 · 仲裁者已沉默"),
        mood: "choice", speaker: Some("姚源（广播）"), voice: None,
        text: TextSpec::Static(&["祭坛随之开启，遗泽本体显形——一团「种子」拼成的光球，与一方「石碑」的剪影并立。姚源的声音在你脑中响起：\n「我们拿走能拿走的，把警告留给后来者。」"]),
        choices: &[ChoiceDef { label: "（走向祭坛 · 作出抉择）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_05_ending_choice") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 抉择结局三分支（带走 / 留下 / 强夺） ------------------- */
    SceneDef {
        id: "yz_05_ending_choice", bg: Some("yize_bg.png"), loc: Some("F4 遗泽圣所 · 祭坛"),
        mood: "choice", speaker: Some("姚源（广播）"), voice: None,
        text: TextSpec::Static(&["种子与石碑静静悬浮。你已经看清真相：这团火，既可以是馈赠，也可以是诅咒。三选无对错——只在于你打算把它交给谁。"]),
        choices: &[
            ChoiceDef { label: "带走遗泽 · 种子", sub: "+400点 · 三枚碎片 · 能量武器线", cond: None,
                effects: &[Eff::SetFlag("yz_legacy_take"), Eff::Points(400), Eff::AddItem("legacy_shard"),
                    Eff::AddItem("legacy_shard"), Eff::AddItem("legacy_shard")],
                route: Route::To("yz_ending_take") },
            ChoiceDef { label: "留下警示 · 石碑", sub: "+400点 · 封印录 · 空间技术线", cond: None,
                effects: &[Eff::SetFlag("yz_legacy_warn"), Eff::Points(400), Eff::AddItem("relic_seal_tome")],
                route: Route::To("yz_ending_warn") },
            ChoiceDef { label: "贪婪强夺", sub: "攻击祭坛 · 强制夺取", cond: None,
                effects: &NO_EFF, route: Route::Dyn(start_remnant) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_ga_round", bg: Some("yize_bg.png"), loc: Some("F4 祭坛 · 仲裁者残骸"),
        mood: "danger", speaker: Some("仲裁者残骸"), voice: None,
        text: TextSpec::Static(&["那团光在你掌下沸腾，本已静默的残骸骤然再起，链锯臂高悬——它要阻止这一记「饕餮」。「悬浮的绝不都该由你独呑。」"]),
        choices: &[
            ChoiceDef { label: "重击", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| ga_act(st, rng(28, 34), false)) },
            ChoiceDef { label: "防御", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| ga_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_ending_take", bg: Some("yize_bg.png"), loc: Some("F4 圣所 · 结局 · 带走"),
        mood: "choice", speaker: Some("张恒（低声）"), voice: None,
        text: TextSpec::Static(&["你捧起那团种子光球。身后石碑和传送门一起,在圣所巨门的长吟中渐渐闭合。张恒你生涩地低语：「你带走的……是一整个战争。」希望号在星海中远去，黑暗中，某物缓缓睁开了眼。（遗泽 · 带走）"]),
        choices: &[ChoiceDef { label: "（驶向主神空间 · 结算）", sub: "能量武器线解锁", cond: None,
            effects: &[
                Eff::PointsIfFlag("yz_f1_stealth", 200), Eff::PointsIfFlag("yz_matrix_order", 200),
                Eff::PointsIfFlag("yz_engine_room_core", 200), Eff::PointsIfFlag("yz_unlock_order", 200),
                Eff::PointsIfFlag("yz_legacy_take", 200),
            ], route: Route::To("yz_settle_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_ending_warn", bg: Some("yize_bg.png"), loc: Some("F4 圣所 · 结局 · 留下警示"),
        mood: "choice", speaker: Some("姚源（广播）"), voice: None,
        text: TextSpec::Static(&["你伸手将那方石碑牢牢嵌入祭坛，种子光球缓缓沉入圣所深处，等待下一个后来者。姚源：「我们该让后来者自己决定。」圣所巨门长吟闭合，黑暗中，某物缓缓睁开了眼。（遗泽 · 留下警示）"]),
        choices: &[ChoiceDef { label: "（驶向主神空间 · 结算）", sub: "空间技术线解锁", cond: None,
            effects: &[
                Eff::PointsIfFlag("yz_f1_stealth", 200), Eff::PointsIfFlag("yz_matrix_order", 200),
                Eff::PointsIfFlag("yz_engine_room_core", 200), Eff::PointsIfFlag("yz_unlock_order", 200),
                Eff::PointsIfFlag("yz_legacy_warn", 200),
            ], route: Route::To("yz_settle_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_ending_graverob", bg: Some("yize_bg.png"), loc: Some("F4 圣所 · 结局 · 强夺"),
        mood: "fear", speaker: Some("张恒"), voice: None,
        text: TextSpec::Static(&["你强行撕下种子与碎片，越过倒下的残骸直扑传送门。身后传来姚源沉重的低语：「盗墓者带走了种子，也带走了警告。」（遗泽 · 强夺 · 灰色结局）"]),
        choices: &[ChoiceDef { label: "（仓皇驶向主神空间 · 结算）", sub: "不计结算结局支线", cond: None,
            effects: &[
                Eff::PointsIfFlag("yz_f1_stealth", 200), Eff::PointsIfFlag("yz_matrix_order", 200),
                Eff::PointsIfFlag("yz_engine_room_core", 200), Eff::PointsIfFlag("yz_unlock_order", 200),
            ], route: Route::To("yz_settle_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 结算卡片 / 通关点 ------------------- */
    SceneDef {
        id: "yz_settle_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "遗 泽 · 抉 择 已 立".into(), good: true,
                body_html: format!(
                    "<p>圣所的冷蓝光随巨门闭合而熄灭。你在星海中带回一团火，或一块碑，或一身尘埃。</p>\
                     <p style='color:#9a958a'>四维护罩 / 仲裁裁定 / 抉择结局 已走完。</p>\
                     <table class='statTable'>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                     <tr><td>主神光柱</td><td>「这不是殿堂，是前人留下的火。」</td></tr>\
                     </table>",
                    st.points
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "yz_lose_arb", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("葬于神迹", "神迹的冷光照着你逐渐冷却的身体")), card: |_st| crate::state::Card {
                title: "葬 于 神 迹".into(), good: false,
                body_html: r#"<p>神迹的静默吞没了你的最后一息。文明留下的火，最终照见的是一片倒下的猎场。</p>
<p style='color:#ff8a8a'>【死亡档案 · 葬于神迹】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线；本次已得 points 保留，圣所封门需再入重打。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "yz_end_gate", bg: Some("yize_bg.png"), loc: Some("F4 北端 · 通关传送门"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["通关传送门的幽光在你面前展开——你知道，只要迈过去，便是主神空间。"]),
        choices: &[ChoiceDef { label: "（穿过传送门 · 结算）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yz_settle_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

/* =====================================================================
   Route::Dyn 辅助（fn 指针，供 static 数组使用）
   ===================================================================== */
/// F3 三步断电：完成 yz_w1/w2/w3 → engine_room_core + engine_open
fn engine_step(st: &mut GameState) -> String {
    if st.flag("yz_w1") && st.flag("yz_w2") && st.flag("yz_w3") {
        st.set_flag("yz_engine_room_core");
        st.set_flag("yz_engine_open");
    }
    "yz_04_engine_hub".to_string()
}

// ---------- 查询辅助 ----------
pub fn yz_scene(id: &str) -> Option<&'static SceneDef> { YIZE_SCENES.iter().find(|s| s.id == id) }
pub fn yz_fight_cfg(id: &str) -> Option<&'static FightCfg> { yize_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v) }
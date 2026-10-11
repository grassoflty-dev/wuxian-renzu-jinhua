//! 《侠行天下 · 魔教总坛·血月坛》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.7 魔教总坛 + xiaxing_tianxia 研究文档 §五候选4。
//! 本文件是全新新增文件，只导出静态数据（MOJIAO_SCENES / mojiao_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/mojiao_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `mj_` 前缀，与既有 SCENES 无重名；fight id 全部 `mj_` 前缀。
//! BOSS 血池傀儡主 / 魔教教主采用「选择驱动遭遇链」落地（参考 scenes_jiguancheng.rs 的 jc_colossus 模式）：
//! 因需要每回扣血 / 狂暴 / 血池侵蚀等自定义每回合同调，引擎原生 FightCfg 无此钩子，
//! 故用 Normal 场景 + Route::Dyn 落地；同时导出 `mj_pool_boss`/`mj_jiaozhu` FightCfg 供 ZoneDef 与揭示用。
//! 抉择结局三分支（夺位/焚坛/揭真相）在教主战后分叉，sp_grade 统一写 Some('D')。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 血月山道 bg mj_bg_path  （现用 img_zhuyuan_book.png 占位）
//!   L2 总坛前殿 bg mj_bg_hall  （现用 img_redqueen.png 占位）
//!   L3 血池殿   bg mj_bg_pool  （现用 img_redqueen.png 占位）
//!   L4 密室     bg mj_bg_lord  （现用 img_corridor.png 占位）
//! 敌人立绘复用：guard→魔教教众/影卫、hunter→红衣护法、horde→血池傀儡；新美术由主 agent 统一生图替换。

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

/// 物品栏是否持有（供 cond fn 使用；闭包不能捕获，故用独立具名函数）
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_moon_pass(st: &GameState) -> bool { inv(st, "it_mj_moon_pass") }
fn cond_has_hufa_token(st: &GameState) -> bool { inv(st, "it_mj_hufa_token") }
fn cond_has_pool_key(st: &GameState) -> bool { inv(st, "it_mj_pool_key") }
fn cond_has_tome(st: &GameState) -> bool { inv(st, "it_mj_tome") }
fn cond_tome_read(st: &GameState) -> bool { st.flag("mj_tome_read") }
fn cond_pool_clear(st: &GameState) -> bool { st.flag("mj_pool_clear") }
fn cond_lord_down(st: &GameState) -> bool { st.flag("mj_lord_down") }

/* =====================================================================
   战斗配置表（id 全部 mj_ 前缀）。每个 FightCfg 的 win 回调只用于 fight_id 直战场面；
   BOSS（血池傀儡主/教主）由选择驱动回合逻辑接管胜负，FightCfg.win 仅作兜底揭示。
   ===================================================================== */
fn mj_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn mj_win_l1(_st: &GameState) -> String { "mj_01".to_string() }
fn mj_win_qian(_st: &GameState) -> String { "mj_05_arrive_qian".to_string() }
fn mj_win_hufa(_st: &GameState) -> String { "mj_06_hufa_win".to_string() }
fn mj_win_pool(_st: &GameState) -> String { "mj_10_arrive_pool".to_string() }
fn mj_win_lord_g(_st: &GameState) -> String { "mj_22_arrive_lord".to_string() }

pub fn mojiao_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("mj_jiaozhong", FightCfg {
            name: "魔教教众", hp: 34, dmg: (7, 13), reward: 12, reward_why: "清剿血月山道巡山教众",
            intro: "一名穿着血月纹黑袍的教众自山道阴影里拦路，袖中一柄弯刀寒光一闪——「血月之敌，来一个，杀一个。」",
            rage_at: None, rage_text: "", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_l1, death: "mj_50_death",
        }),
        ("mj_sentry", FightCfg {
            name: "血月哨卫", hp: 38, dmg: (8, 14), reward: 15, reward_why: "拔除登坛路口的血月哨卫",
            intro: "高台上的哨卫敲响铜锣，一声悠长的示警回荡在血月山道间，他从台上一跃而下。",
            rage_at: None, rage_text: "", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_l1, death: "mj_50_death",
        }),
        ("mj_yingwei", FightCfg {
            name: "影卫教众", hp: 42, dmg: (10, 16), reward: 25, reward_why: "肃清前方影卫",
            intro: "两名影卫贴着柱子无声落地，面具下的目光冷得像血。手中短刺交错成一道杀阵。",
            rage_at: Some(20), rage_text: "影卫陷入疯狂，短刺连刺——攻势骤增！", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_qian, death: "mj_50_death",
        }),
        ("mj_hufa", FightCfg {
            name: "红衣护法 · 赤", hp: 60, dmg: (11, 18), reward: 40, reward_why: "击败红衣护法 · 赤，得红衣令牌",
            intro: "红衣护法·赤立在殿门之前，披风如血泼洒。他缓缓开口：「能走到这里，你手里已经染了教众的血。」",
            rage_at: Some(28), rage_text: "护法周身涌起血雾，掌法带着血月邪气——伤害暴增！", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_hufa, death: "mj_50_death",
        }),
        ("mj_kulei", FightCfg {
            name: "血池傀儡", hp: 50, dmg: (10, 17), reward: 30, reward_why: "击碎血池傀儡",
            intro: "血肉与陶土捏成的傀儡从血池边站起，关节咯咯作响，喉间发出空洞的哭嚎。",
            rage_at: Some(24), rage_text: "傀儡的血口大张，粘稠血丝抽向你——攻击附带血蚀！", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_pool, death: "mj_50_death",
        }),
        ("mj_hufa2", FightCfg {
            name: "红衣护法 · 黑", hp: 70, dmg: (12, 19), reward: 60, reward_why: "击败红衣护法 · 黑",
            intro: "镇守血池殿的黑衣护法袍袖膨胀，掌中结出一朵血莲：「禁地再深一寸，命就再短一寸。」",
            rage_at: Some(34), rage_text: "血莲绽放，护法身法如鬼魅——闪避与伤害齐涨！", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_pool, death: "mj_50_death",
        }),
        ("mj_jiaozhu_guard", FightCfg {
            name: "教主心腹 · 狂徒", hp: 55, dmg: (11, 17), reward: 40, reward_why: "击倒教主心腹",
            intro: "两名狂徒一左一右盯着你，毒刀出鞘，眼白里全是渴血的狂热。",
            rage_at: Some(26), rage_text: "狂徒吞下一枚血丹，脚步凌乱地向你扑来！", on_rage: mj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mj_win_lord_g, death: "mj_50_death",
        }),
        ("mj_pool_boss", FightCfg {
            name: "血池傀儡主", hp: 120, dmg: (14, 22), reward: 150, reward_why: "通过血池试炼，得赤血钥",
            intro: "血池骤然沸腾，一头由万千傀儡血肉熔成的巨影自池心升起——「血池试炼……要么活，要么成池中料。」",
            rage_at: Some(60), rage_text: "血池傀儡主进入狂暴，血浪拍岸，每三回血蚀全场！",
            on_rage: mj_rage_none,
            finisher_if: |st, _| st.flag("mj_tome_read") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "以残卷镇血".to_string(),
            finisher_desc: |_| "你展开从密室暗牢得来的残卷，诵出上古血符——血池傀儡主狰狞的血口一滞，被镇在原地！".to_string(),
            win: |_st| "mj_21_trial_done".to_string(),
            death: "mj_50_death",
        }),
        ("mj_jiaozhu", FightCfg {
            name: "魔教教主 · 血月尊者", hp: 200, dmg: (16, 24), reward: 200, reward_why: "战胜魔教教主 · 血月尊者",
            intro: "宝座上的人缓缓睁眼，一身暗红血袍在无风下猎猎作响。「血月升起，总坛才开门——你来的时候，血正好红了。」",
            rage_at: Some(100), rage_text: "教主血袍绽放，血月邪功全开，化作漫天血掌——伤害暴增，且每三回血蚀全场！",
            on_rage: mj_rage_none,
            finisher_if: |st, _| inv(st, "it_mj_tome") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "祭出残卷真意".to_string(),
            finisher_desc: |_| "你在血月邪功最盛时祭出残卷，血光骤暗——教主眼底第一次闪过慌乱，却仍咬牙硬撑。".to_string(),
            win: |_st| "mj_31_lord_down".to_string(),
            death: "mj_50_death_lord",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn mj_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    mojiao_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   BOSS · 血池傀儡主（选择驱动遭遇）
   血量存 st.fight（mj_20_trial 的 Route::Dyn 初始化，引用 mj_pool_boss 的 FightCfg）。
   每"回"是一个 Normal 场景 mj_pool_round；Route::Dyn 统一处理：扣血、狂暴、血蚀、胜负。
   ===================================================================== */
fn start_pool(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("mj_pool_boss") {
            st.fight = Some(crate::power::scaled_fight("mj_pool_boss", cfg, st, vec![]));
        }
    }
    "mj_pool_round".to_string()
}

/// 血池试炼胜利结算：+150、赤血钥（G3）、置 mj_pool_clear
fn pool_win(st: &mut GameState) -> String {
    st.points += 150;
    crate::world::add_item(st, "it_mj_pool_key");
    st.set_flag("mj_pool_clear");
    "mj_21_trial_done".to_string()
}

/// 血蚀：每 3 回全场判定（每名队员 Hurt；不需要，此处对玩家 Hurt 8，持残卷减半）
fn pool_erode(st: &mut GameState) -> String {
    let erode = if inv(st, "it_mj_tome") { 4 } else { 8 };
    st.hp = (st.hp - erode).max(0);
    if st.hp <= 0 {
        return "mj_50_death".to_string();
    }
    "mj_pool_round".to_string()
}

fn pool_dead() -> String { "mj_50_death".to_string() }

/// 一个"回"：玩家攻击池中傀儡主。
fn pool_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return pool_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 60).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(22, 30) } else { rng(14, 22) };
    let dodge = if guard { 0.5 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return pool_dead();
    }
    // 每 3 回血蚀
    let count: usize = {

        let c1 = st.flag("mj_pc1"); let c2 = st.flag("mj_pc2");
        if c1 && c2 { 3 } else if c1 { 2 } else if c2 { 2 } else { 0 }
    };
    if count >= 2 {
        st.flags.insert("mj_pc1".to_string(), false);
        st.flags.insert("mj_pc2".to_string(), false);
        return pool_erode(st);
    }
    // 推进计数器：pc1 未置则置，已置则置 pc2
    if st.flag("mj_pc1") { st.set_flag("mj_pc2"); } else { st.set_flag("mj_pc1"); }
    "mj_pool_round".to_string()
}

/* =====================================================================
   BOSS · 魔教教主 · 血月尊者（选择驱动遭遇）
   ===================================================================== */
fn start_lord(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("mj_jiaozhu") {
            st.fight = Some(crate::power::scaled_fight("mj_jiaozhu", cfg, st, vec![]));
        }
    }
    "mj_lord_round".to_string()
}

/// 教主败北结算：+200、sp_grade=D、置 mj_lord_down
fn lord_win(st: &mut GameState) -> String {
    st.points += 200;
    st.sp_grade = Some('D');
    st.set_flag("mj_lord_down");
    "mj_31_lord_down".to_string()
}

fn lord_dead() -> String { "mj_50_death_lord".to_string() }

/// 一个"回"：玩家攻击教主。
fn lord_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return lord_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 100).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(24, 32) } else { rng(16, 24) };
    let dodge = if guard { 0.5 } else { 0.15 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return lord_dead();
    }
    "mj_lord_round".to_string()
}

/* =====================================================================
   剧情场景（id 全部 mj_ 前缀）
   ===================================================================== */
pub static MOJIAO_SCENES: &[SceneDef] = &[

    SceneDef {
        id: "mj_00", bg: Some("mojiao_bg.png"), loc: Some("血月山道 · 山脚"),
        mood: "mystery", speaker: Some("旁白"), voice: Some("vo_mj_open"),
        text: TextSpec::Static(&[
            "<b>【主线任务】</b>追踪红衣使者深入魔教总坛，闯血池殿，闯过血池试炼，直面魔教教主。失败代价：被扣 300 点复活。",
            "暮色里一轮血月缓缓升起。山道的石阶被血浸成暗红，尽头那扇总坛石门的匾额在血光下明灭——你追寻已久的红衣人，最后的身影消失在山门之后。",
            "「血月升起时，总坛才开门。」",
        ]),
        choices: &[
            ChoiceDef { label: "拾级而上", sub: "进入血月山道", cond: None, effects: &NO_EFF, route: Route::To("mj_01") },
        ],
        fight_id: None, video: Some("vid_mj_opening.mp4"), cine_label: Some("过场 · 血月总坛"), overlay: None,
    },

    /* ---- L1 血月山道 hub ---- */
    SceneDef {
        id: "mj_01", bg: Some("mojiao_bg.png"), loc: Some("L1 · 血月山道"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_has_moon_pass(st) {
                "血月令牌已在手，登坛牌坊随同兽首「咔」地应你而开。山道尽头的总坛石门洞开，血月的光一直铺到门里。".to_string()
            } else {
                "血月下，山道盘旋而上。两侧巡山教众与哨卫往来逡巡，一块刻满了暗红符文的血月刻石立在道旁。登坛牌坊锁着总坛入口——需一枚血月令牌。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "查看血月刻石", sub: "暗号·血月令牌线索", cond: None, effects: &NO_EFF, route: Route::To("mj_02_moon_mark") },
            ChoiceDef { label: "拾起断裂剑鞘", sub: "剧情伏笔", cond: None, effects: &NO_EFF, route: Route::To("mj_02_sheath") },
            ChoiceDef { label: "探查巡山暗道", sub: "+15 点 · 情报", cond: None, effects: &NO_EFF, route: Route::To("mj_03_alt_path") },
            ChoiceDef { label: "登坛牌坊（需血月令牌）", sub: "G1 → 总坛前殿", cond: Some(cond_has_moon_pass), effects: &NO_EFF, route: Route::To("mj_04_gate") },
            ChoiceDef { label: "清扫巡山教众", sub: "练手 · 得径上讯息", cond: None, effects: &NO_EFF, route: Route::To("mj_01_fight_jiaozhong") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_02_moon_mark", bg: Some("mojiao_bg.png"), loc: Some("L1 · 血月刻石"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["血月刻石上镂着古老的符文，缝隙里嵌着一枚油亮的黑色令牌——血月令牌。碑侧一行小字：「血月升起时，总坛才开门。」"]),
        choices: &[ChoiceDef { label: "取下血月令牌", sub: "Item it_mj_moon_pass · 开 G1", cond: None,
            effects: &[Eff::AddItem("it_mj_moon_pass"), Eff::MarkPoint("mj_p_l1_1")], route: Route::To("mj_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_02_sheath", bg: Some("mojiao_bg.png"), loc: Some("L1 · 断裂剑鞘"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["道边一柄断剑，鞘上刻着两个字：「除魔」。剑身却染着发黑的血——拔剑的是一位曾立志除魔的人，最后却死在了血月之下。"]),
        choices: &[ChoiceDef { label: "收下断剑作戒", sub: "剧情伏笔 · 得断剑", cond: None,
            effects: &[Eff::AddItem("it_mj_broken_sword"), Eff::MarkPoint("mj_p_l1_2")], route: Route::To("mj_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_03_alt_path", bg: Some("mojiao_bg.png"), loc: Some("L1 · 巡山暗道"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["楚道的岩缝里藏着一条供暗哨换班的窄径，尽头一块断碑记着：「血月三年，坛主欲以活人喂养其血月邪功——凡妄入者，皆成池中料。」"]),
        choices: &[ChoiceDef { label: "记下血月真相", sub: "+15 点 · 情报", cond: None,
            effects: &[Eff::Points(15), Eff::MarkPoint("mj_p_l1_3")], route: Route::To("mj_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_01_fight_jiaozhong", bg: Some("mojiao_bg.png"), loc: Some("L1 · 血月山道 · 遭遇"),
        mood: "danger", speaker: None, voice: Some("vo_mj_jiaozhong"),
        text: TextSpec::Static(&["你想强闯，巡山教众立刻围了上来，弯刀出鞘。（战斗）"]),
        choices: &[], fight_id: Some("mj_jiaozhong"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_04_gate", bg: Some("mojiao_bg.png"), loc: Some("L1 · 登坛牌坊（G1 已开）"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&["血月令牌嵌入兽首，「咔」地一转，石门泛起一片血晕，在血月下缓缓两开。你踏过牌坊，身后山门轰然合拢——没有回头路了。"]),
        choices: &[ChoiceDef { label: "（踏入总坛前殿）", sub: "p_mj_1 单向 · 进 L2", cond: None, effects: &NO_EFF, route: Route::To("mj_05_arrive_qian") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= L2 总坛前殿 ================= */
    SceneDef {
        id: "mj_05_arrive_qian", bg: Some("mojiao_bg.png"), loc: Some("L2 · 总坛前殿"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_has_hufa_token(st) {
                "红衣令牌在手，正殿朱门上的血印应它而散。你把目光投向殿后那扇通往更深处血味的门——血池殿。".to_string()
            } else {
                "前殿宽阔阴森，两廊影卫垂手而立，正中那袭红衣护法站在殿门前。你需要他的红衣令牌，才能推开那道锁着血池殿的朱门。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "翻看影卫名录", sub: "情报 · 弱点", cond: None, effects: &NO_EFF, route: Route::To("mj_05_rost") },
            ChoiceDef { label: "查察铜鼎机关", sub: "殿中机括 · 动机关", cond: None, effects: &NO_EFF, route: Route::To("mj_05_ding") },
            ChoiceDef { label: "会一会红衣护法 · 赤", sub: "击败得红衣令牌", cond: None, effects: &NO_EFF, route: Route::To("mj_06_hufa_fight") },
            ChoiceDef { label: "前殿正门（需红衣令牌）", sub: "G2 → 血池殿", cond: Some(cond_has_hufa_token), effects: &NO_EFF, route: Route::To("mj_06_gate") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_05_rost", bg: Some("mojiao_bg.png"), loc: Some("L2 · 影卫名录"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["一份鬼魅般的影卫名册摊在案上，记着一个个编号与「归处：血池」。最后一行墨色新干：「红衣护法 · 赤，掌前殿，令一环。」"]),
        choices: &[ChoiceDef { label: "记下护法令牌所在", sub: "剧情", cond: None,
            effects: &[Eff::MarkPoint("mj_p_l2_1")], route: Route::To("mj_05_arrive_qian") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_05_ding", bg: Some("mojiao_bg.png"), loc: Some("L2 · 铜鼎机关"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["殿中一口青铜巨鼎，鼎腹刻着一头血月纹。你试着转动鼎耳，机簧低鸣，整座前殿的气窗齐齐开合——这是操控殿中傀儡的枢纽。"]),
        choices: &[ChoiceDef { label: "转动鼎耳", sub: "+10 点 · 殿内响动", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("mj_p_l2_2")], route: Route::To("mj_05_arrive_qian") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_06_hufa_fight", bg: Some("mojiao_bg.png"), loc: Some("L2 · 前殿 · 会护法"),
        mood: "danger", speaker: Some("红衣护法 · 赤"), voice: Some("vo_mj_hufa"),
        text: TextSpec::Static(&["红衣护法·赤的目光掠过你，「刷」地抽出那柄血纹长刀——「能走到这里，你手里已经染了教众的血。」（战斗）"]),
        choices: &[], fight_id: Some("mj_hufa"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_06_hufa_win", bg: Some("mojiao_bg.png"), loc: Some("L2 · 前殿"),
        mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["护法倒下的衣袂下滚出一枚暗红令牌。血月印记温热如活物——红衣令牌，锁着通往血池殿的朱门。"]),
        choices: &[ChoiceDef { label: "拾取红衣令牌", sub: "Item it_mj_hufa_token · 开 G2", cond: None,
            effects: &[Eff::AddItem("it_mj_hufa_token")], route: Route::To("mj_05_arrive_qian") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_06_gate", bg: Some("mojiao_bg.png"), loc: Some("L2 · 前殿正门（G2 已开）"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&["红衣令牌贴上护封，血印寸寸消融。朱门无声滑开，一股浓得化不开的血腥气扑面——血池殿，就在门后。"]),
        choices: &[ChoiceDef { label: "（入血池殿密梯）", sub: "p_mj_2 单向 · 进 L3", cond: None, effects: &NO_EFF, route: Route::To("mj_10_arrive_pool") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= L3 血池殿 ================= */
    SceneDef {
        id: "mj_10_arrive_pool", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池殿"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_pool_clear(st) {
                "血池的浪头已平，池心浮着赤血钥的微光。殿门通往更深处——教主的密室。".to_string()
            } else {
                "脚下的地面在微微颤动。正中一方足有数丈的开阔血池，池心一道雪白的影子正缓缓立起——血池试炼之主。红衣使者站在池畔，似乎在等你。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "与红衣使者相谈", sub: "剧情 · 身世", cond: None, effects: &NO_EFF, route: Route::To("mj_10_hongshang") },
            ChoiceDef { label: "读试炼碑", sub: "血池试炼规则", cond: None, effects: &NO_EFF, route: Route::To("mj_12_trial_stele") },
            ChoiceDef { label: "探四方傀儡关节", sub: "洞中机密", cond: None, effects: &NO_EFF, route: Route::To("mj_13_puppet") },
            ChoiceDef { label: "踏血池试炼", sub: "miniboss 池中傀儡主", cond: None, effects: &NO_EFF, route: Route::To("mj_20_trial") },
            ChoiceDef { label: "血池殿门（需赤血钥）", sub: "G3 → 教主密室", cond: Some(cond_has_pool_key), effects: &NO_EFF, route: Route::To("mj_22_gate_pool") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_10_hongshang", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池殿 · 红衣使者"),
        mood: "cold", speaker: Some("红衣使者"), voice: None,
        text: TextSpec::Static(&["红衣使者背对着你，轻声道：「数十年前，也是这样的血月。一位立志除魔的剑客奉命潜入总坛……他没能活着出来。他留下了一柄断剑。」他抬眼看了你一眼，「你和他，很像。」"]),
        choices: &[ChoiceDef { label: "听完这段往事", sub: "若持断剑 · 触真相线", cond: None, effects: &NO_EFF, route: Route::To("mj_05_arrive_qian_hint") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_05_arrive_qian_hint", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池殿"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            if inv(st, "it_mj_broken_sword") {
                "红衣使者望向你腰间的断剑，微微颔首。「你果然带着它。教主密室有一间暗牢，锁着当年那位剑客留下的东西——若你解得开墙上的残卷，兴许能寻到镇血月之法。」".to_string()
            } else {
                "红衣使者笑笑，不再多言。".to_string()
            }
        }),
        choices: &[ChoiceDef { label: "回到血池殿", sub: "", cond: None, effects: &NO_EFF, route: Route::To("mj_10_arrive_pool") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_12_trial_stele", bg: Some("mojiao_bg.png"), loc: Some("L3 · 试炼碑"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["试炼碑上刻着血池试炼的规矩：「池中傀儡主以万千傀儡血肉为体，狂暴后血浪每三回侵蚀一次。若有镇血之物，可暂缓其势。」"]),
        choices: &[ChoiceDef { label: "记下试炼规则", sub: "情报", cond: None,
            effects: &[Eff::MarkPoint("mj_p_l3_2")], route: Route::To("mj_10_arrive_pool") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_13_puppet", bg: Some("mojiao_bg.png"), loc: Some("L3 · 四方傀儡窟"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&["血池四面开有四个石窟，一身身陶土傀儡整齐伏跪。你隐约听见窟里有暗道通往回层的声响——那是一条万一血池殿失守的退路。"]),
        choices: &[ChoiceDef { label: "记下傀儡窟秘道", sub: "+10 点 · 回跳线索", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("mj_p_l3_3")], route: Route::To("mj_10_arrive_pool") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_20_trial", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池 · 试炼起"),
        mood: "danger", speaker: Some("血池傀儡主"), voice: Some("vo_mj_pool"),
        text: TextSpec::Static(&[
            "血池骤然沸腾，万千傀儡的血肉从池底汇聚，一股狰狞的巨影自池心缓缓立起——血池傀儡主。",
            "「血池试炼……要么活，要么，成池中料。」",
        ]),
        choices: &[ChoiceDef { label: "【迎战傀儡主】", sub: "进入试炼", cond: None, effects: &NO_EFF, route: Route::Dyn(start_pool) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_pool_round", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池 · 激战"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            let f = st.fight.as_ref().map(|f| format!("池中傀儡主 HP {} / {}", f.hp.max(0), 120)).unwrap_or_else(|| "HP --".to_string());
            let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
                "——狂暴！血浪每三回侵蚀（Hurt 8，持残卷减半）——"
            } else { "" };
            format!("{f}。{}", mode)
        }),
        choices: &[
            ChoiceDef { label: "重击（强攻）", sub: "伤害 26-36", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| pool_act(st, rng(26, 36), false)) },
            ChoiceDef { label: "连击（迅疾）", sub: "伤害 18-26", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| pool_act(st, rng(18, 26), false)) },
            ChoiceDef { label: "【祭出残卷镇血】", sub: "需残卷 + 狂暴 · 40 固伤并压制", cond: Some(cond_tome_read),
                effects: &NO_EFF, route: Route::Dyn(|st| pool_act(st, 40, false)) },
            ChoiceDef { label: "后撤喘息", sub: "提升闪避", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| pool_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_21_trial_done", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池 · 试炼毕"),
        mood: "calm", speaker: Some("红衣使者"), voice: None,
        text: TextSpec::Static(&["血池傀儡主轰然塌回池底，血浪方平。池心浮起一枚暗红的赤血钥——它通向更深处，教主的密室。红衣使者望着你：「去吧，那是最后一扇门了。」"]),
        choices: &[ChoiceDef { label: "潜身拾起赤血钥", sub: "Item it_mj_pool_key · 开 G3", cond: None,
            effects: &[Eff::AddItem("it_mj_pool_key")], route: Route::To("mj_10_arrive_pool") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_22_gate_pool", bg: Some("mojiao_bg.png"), loc: Some("L3 · 血池殿门（G3 已开）"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&["赤血钥旋入锁孔，「嗡——」铁链纷纷断裂。通往教主密室的门在你面前缓缓打开，血月的冷光倾泻而下。"]),
        choices: &[ChoiceDef { label: "（入教主密梯）", sub: "p_mj_3 单向 · 进 L4", cond: None, effects: &NO_EFF, route: Route::To("mj_22_arrive_lord") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= L4 教主密室 ================= */
    SceneDef {
        id: "mj_22_arrive_lord", bg: Some("mojiao_bg.png"), loc: Some("L4 · 教主密室"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_lord_down(st) {
                "宝座空悬，血月的残光仍在殿顶流转。你方才所经的三关——闯坛、血池试炼、教主战，都已尽数落幕。".to_string()
            } else {
                "墓室般的密室里，一尊暗红宝座高踞大殿之巅。四壁悬着枯骨与残卷。两名教主心腹持刀而立，宝座上那道身影正缓缓转过来。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "察看教主宝座", sub: "剧情", cond: None, effects: &NO_EFF, route: Route::To("mj_21_throne") },
            ChoiceDef { label: "取秘术书 · 残卷", sub: "隐藏 · 解锁暗牢", cond: None, effects: &NO_EFF, route: Route::To("mj_21_tome") },
            ChoiceDef { label: "与前朝残魂交谈", sub: "真相线", cond: None, effects: &NO_EFF, route: Route::To("mj_23_ghost") },
            ChoiceDef { label: "迎战魔教教主", sub: "BOSS 决战", cond: None, effects: &NO_EFF, route: Route::To("mj_30_lord") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_21_throne", bg: Some("mojiao_bg.png"), loc: Some("L4 · 教主宝座"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["宝座扶手两侧各刻着一行字。左：「生于此，亦葬于此。」右：「血月之下，无人能逃。」座上留下一枚早该腐朽的铜令——旧主的信物。"]),
        choices: &[ChoiceDef { label: "记下宝座铭文", sub: "剧情", cond: None,
            effects: &[Eff::MarkPoint("mj_p_l4_1")], route: Route::To("mj_22_arrive_lord") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_21_tome", bg: Some("mojiao_bg.png"), loc: Some("L4 · 秘术书 · 残卷"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["一本泛黄的残卷静静躺在石案上，封面以血朱砂写着一个「镇」字。你翻开，上面记着镇血月邪功的上古符诀——与血池试炼碑上说的「镇血之物」同源。"]),
        choices: &[ChoiceDef { label: "研读残卷", sub: "解锁密室暗牢 · 可镇血", cond: None,
            effects: &[Eff::AddItem("it_mj_tome"), Eff::SetFlag("mj_tome_read"), Eff::MarkPoint("mj_p_l4_2")], route: Route::To("mj_22_arrive_lord") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_23_ghost", bg: Some("mojiao_bg.png"), loc: Some("L4 · 前朝残魂"),
        mood: "cold", speaker: Some("前朝残魂"), voice: None,
        text: TextSpec::Static(&["一道近乎透明的残影守在宝座之侧，开口时声音像从血里飘出来：「我……便是当年那个立志除魔的剑客。我把一柄断剑留在山道上，把一身技艺锁进这间暗牢——意图有朝一日，有人能替我斩断血月。」"]),
        choices: &[ChoiceDef { label: "追寻当年真相", sub: "truth 线", cond: None,
            effects: &[Eff::AddItem("it_mj_truth_scroll"), Eff::SetFlag("mj_truth")], route: Route::To("mj_22_arrive_lord") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_30_lord", bg: Some("mojiao_bg.png"), loc: Some("L4 · 教主密室 · 决战"),
        mood: "danger", speaker: Some("魔教教主 · 血月尊者"), voice: Some("vo_mj_lord"),
        text: TextSpec::Static(&[
            "宝座上的血月尊者缓缓起身，血袍无风自动。「血月升起，总坛才开门——你来的时候，血正好红了。能闯过血池试炼，你是个可造之材……可惜，找错了对手。」",
            "他伸掌，整座密室的血光都朝他掌心涌去。",
        ]),
        choices: &[ChoiceDef { label: "【迎战血月尊者】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_lord) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_lord_round", bg: Some("mojiao_bg.png"), loc: Some("L4 · 教主密室 · 激战"),
        mood: "danger", speaker: Some("魔教教主"), voice: None,
        text: TextSpec::Dyn(|st| {
            let f = st.fight.as_ref().map(|f| format!("血月尊者 HP {} / {}", f.hp.max(0), 200)).unwrap_or_else(|| "HP --".to_string());
            let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
                "——血月邪功全开！伤害暴增——"
            } else { "" };
            format!("{f}。{}", mode)
        }),
        choices: &[
            ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| lord_act(st, rng(30, 42), false)) },
            ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-30", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| lord_act(st, rng(20, 30), false)) },
            ChoiceDef { label: "【祭出残卷真意】", sub: "需残卷 + 狂暴 · 40 固伤", cond: Some(cond_has_tome),
                effects: &NO_EFF, route: Route::Dyn(|st| lord_act(st, 40, false)) },
            ChoiceDef { label: "后撤喘息", sub: "提升闪避", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| lord_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_31_lord_down", bg: Some("mojiao_bg.png"), loc: Some("L4 · 教主密室"),
        mood: "calm", speaker: Some("血月尊者（临终）"), voice: Some("vo_mj_lord_down"),
        text: TextSpec::Static(&["血月尊者踉跄跪地，血袍褪成惨白。「生于此……亦葬于此……」他望向殿顶那轮血月，忽然流下一滴清泪，「原来……血月底下……也有想逃的人。」他缓缓阖上眼，血月的光芒随之黯淡。"]),
        choices: &[ChoiceDef { label: "（走向血月尊者的遗愿）", sub: "抉择结局", cond: None, effects: &NO_EFF, route: Route::To("mj_40_ending") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 抉择结局三分支 ================= */
    SceneDef {
        id: "mj_40_ending", bg: Some("mojiao_bg.png"), loc: Some("L4 · 教主密室"),
        mood: "mystery", speaker: Some("红衣使者"), voice: Some("vo_mj_ending"),
        text: TextSpec::Static(&[
            "红衣使者不知何时出现在你身后，声音低沉：「教主已死，血月将落。而这地方……要如何收场，全在你一念之间。」",
            "他退后一步，看着你，「你若要这血月下的座，我劝你再想想。」",
        ]),
        choices: &[
            ChoiceDef { label: "登临教主之位", sub: "+200 点 · 执掌总坛", cond: None,
                effects: &[Eff::SetFlag("mj_end_rule"), Eff::AddItem("it_mj_throne_token"), Eff::Points(200),
                    Eff::SetFlag("mj_ending_done")], route: Route::To("mj_41_exit") },
            ChoiceDef { label: "焚毁魔教总坛", sub: "+250 点 · 永绝邪根", cond: None,
                effects: &[Eff::SetFlag("mj_end_destroy"), Eff::Points(250), Eff::SetFlag("mj_ending_done")],
                route: Route::To("mj_41_exit") },
            ChoiceDef { label: "揭穿真相 · 斩断血月", sub: "+250 点 · 真相结局", cond: None,
                effects: &[Eff::SetFlag("mj_end_truth"), Eff::AddItem("it_mj_truth_scroll"), Eff::Points(250),
                    Eff::SetFlag("mj_ending_done")], route: Route::To("mj_41_exit") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_41_exit", bg: Some("mojiao_bg.png"), loc: Some("L4 · 撤离阵"),
        mood: "calm", speaker: Some("主神"), voice: None,
        text: TextSpec::Dyn(|st| {
            match (st.flag("mj_end_rule"), st.flag("mj_end_destroy"), st.flag("mj_end_truth")) {
                (true, _, _) => "你坐上那尊暗红宝座，血月的苍白映在你脸上。你忽然想起山道上那柄断剑，和那句「除魔」——血月易主，可血终究还是红的。你踏入撤离阵。".to_string(),
                (_, true, _) => "你点燃总坛，血月在烈火里烧成灰烬。傀儡、教众、诅咒般的传承，一并葬进长夜。你回头望了一眼跳动的火，踏入撤离阵。".to_string(),
                (_, _, true) => "你把当年那位剑客的断剑插在教主宝座上，血月纹寸寸崩碎。红衣使者长揖而退：「这一刀，替他斩了血月。」你踏入撤离阵，身后终于亮起天光。".to_string(),
                _ => "血月将落，你转身踏入撤离阵。".to_string(),
            }
        }),
        choices: &[ChoiceDef { label: "（踏入撤离阵 · 结算）", sub: "sp_grade 结算 · 回主神空间", cond: None,
            effects: &NO_EFF, route: Route::Dyn(mj_route_exit_settle) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mj_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_mj_settle"), death: None,
            card: |st| crate::state::Card {
                title: "魔 教 总 坛 · 血 月 落".into(), good: true,
                body_html: format!(
                    "<p>血月退去，总坛的廊檐重归死寂。你带着一身血迹踏出撤离阵。</p>\
                     <p style='color:#9a958a'>血月散场：闯坛已过 / 血池试炼已过 / 抉择已作。</p>\
                     <table class='statTable'>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                     <tr><td>主神光柱</td><td>「血月之下，也有想逃的人。」</td></tr>\
                     </table>",
                    st.points
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 死亡档案（复活扣 300/回主神）================= */
    SceneDef {
        id: "mj_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("血月山道下的无名者", "葬在魔教总坛的某一条血影之下")), card: |_st| crate::state::Card {
                title: "血 月 之 下".into(), good: false,
                body_html: r#"<p>你的躯体冷却在总坛的某条回廊里，浓重的血腥裹住你逐渐模糊的意识。</p>
<p style='color:#ff8a8a'>【死亡档案 · 血月山道下的无名者】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "mj_50_death_lord", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("血月尊者掌下", "血袍的血掌击碎了你的最后一段神识")), card: |_st| crate::state::Card {
                title: "血 月 尊 者".into(), good: false,
                body_html: r#"<p>漫天血掌轰然压下，你在教主宝座前跪伏下去。血月的光冷冷地照在你愈合不了的伤口上。</p>
<p style='color:#ff8a8a'>【死亡档案 · 血月尊者掌下】</p>
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
/// 撤离结算：落日终局 flag + 评级（D 级已由教主战写;此处确保）→ 卡片
fn mj_route_exit_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "mj_42_card".to_string()
}
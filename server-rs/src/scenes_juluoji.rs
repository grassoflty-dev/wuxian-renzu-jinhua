//! 《无限恐怖 · 侏罗纪公园（失序乐园）》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md 「侏罗纪公园 · 失序乐园」骨架。
//! 本文件是全新新增文件，只导出静态数据（JULUOJI_SCENES / juluoji_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/juluoji_impl_log.md ★依赖）。
//!
//! 场景 id 全部 `jl_` 前缀；fight id 全部 `jl_` 前缀。
//! 恐龙追击 = EnemyDef.radius 拉近 + 剧情用 Dyn 演出（进层即被追→战斗）。
//! BOSS 霸王龙（HP260）采用"选择驱动遭遇"（参考 scenes_jiguancheng.rs 的 colossus_round）：
//! 撕咬反击 / 冲撞换位（改变战距，冲撞后下回合撕咬更凶）引擎原生 FightCfg 无每回合同调钩子，
//! 故用 Normal 场景 + Route::Dyn 落地；同时导出 `jl_trex` FightCfg 供 ZoneDef 与揭示用。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 园区   井 bg jl_bg_park    （现用 img_zhuyuan_book.png 占位）
//!   L2 丛林   井 bg jl_bg_jungle  （现用 img_horde.png 占位）
//!   L3 围栏区 井 bg jl_bg_fence   （现用 img_laser.png 占位）
//! 敌人立绘复用：licker→迅猛龙、hunter→霸王龙、horde→剑龙群。

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

/// 物品栏是否持有
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/* =====================================================================
   具名条件谓词
   ===================================================================== */
fn cond_has_wrench(st: &GameState) -> bool { inv(st, "it_wrench") }
fn cond_has_bait(st: &GameState) -> bool { inv(st, "it_bait_meat") }

/* =====================================================================
   BOSS · 霸王龙（选择驱动遭遇）
   血量存 st.fight（jl_trex_slam indent 的 Route::Dyn 初始化，引用 jl_trex 的 FightCfg）。
   找他人可随时"尝试冲撞"(slam)，成功后置 jl_slammed 使下回合撕咬更凶（换位压力）。
   ===================================================================== */
/// 初始化霸王龙会话（从 jl_trex 的 FightCfg 建 Fight）
fn start_trex(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jl_trex") {
            st.fight = Some(crate::power::scaled_fight("jl_trex", cfg, st, vec![]));
        }
    }
    "jl_trex_round".to_string()
}

/// 霸王龙击杀结算：+650、掉恐龙牙饰、置 jl_trex_down、写 sp_grade=B
fn trex_win(st: &mut GameState) -> String {
    st.points += 650;
    crate::world::add_item(st, "it_trex_tooth");
    st.set_flag("jl_trex_down");
    st.sp_grade = Some('B');
    "jl_13_trex_down".to_string()
}

fn trex_dead() -> String { "jl_40_death_trex".to_string() }

/// 一个"回"：玩家攻击霸王龙。slam=true 尝试冲撞换位（成功后下回合撕咬翻倍）。
fn trex_act(st: &mut GameState, dmg: i32, slam: bool, guard: bool) -> String {
    if guard {
        // 后撤观察：本回不出手，只提升闪避
        if let Some(f) = st.fight.as_mut() { f.hp = f.hp.max(0); }
    } else if !slam {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    } else {
        // 冲撞换位：尝试把霸王龙撞离贴身距离
        let r: f64 = rand::thread_rng().gen();
        if r < 0.5 {
            st.set_flag("jl_slammed"); // 换位成功：下回合撕咬更凶（本回不扣血）
        } else {
            // 撞空：霸王龙趁势反扑，玩家先承一记
            let raw = rng(20, 30);
            st.hp = (st.hp - raw).max(0);
            if st.hp <= 0 { return trex_dead(); }
        }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return trex_win(st);
    }
    // 狂暴触发（HP ≤ 100）
    let raged = st.fight.as_ref().map(|f| f.hp <= 100).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    // 霸王龙反击
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let slammed = st.flag("jl_slammed");
    // 换位成功后冲撞本回无伤，但撕咬判定更凶（tap 加成）
    let raw = if raged_now { rng(26, 38) } else { rng(18, 28) }
        + if slammed { 8 } else { 0 };
    let dodge = if guard { 0.55 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return trex_dead();
    }
    // 换位后状态在本回结算后清空（避免连锁）
    if slammed { st.flags.insert("jl_slammed".to_string(), false); }
    "jl_trex_round".to_string()
}

/* =====================================================================
   胜利 / 失败共通 win 回调（普通敌人；FightCfg.win 用）
   ===================================================================== */
fn jl_win_l1(_st: &GameState) -> String { "jl_01".to_string() }
fn jl_win_l2(_st: &GameState) -> String { "jl_02_arrive_jungle".to_string() }
fn jl_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 战斗配置表（id 全部 jl_ 前缀）。
pub fn juluoji_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("jl_raptor", FightCfg {
            name: "迅猛龙", hp: 46, dmg: (10, 16), reward: 35, reward_why: "甩掉追击的迅猛龙",
            intro: "一头迅猛龙从雕像后窜出，低伏着身，爪尖在砾石地上划出三线火星。",
            rage_at: None, rage_text: "", on_rage: jl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jl_win_l1, death: "jl_40_death",
        }),
        ("jl_stego", FightCfg {
            name: "剑龙群", hp: 70, dmg: (13, 20), reward: 55, reward_why: "剑龙群横扫让开前路",
            intro: "一群剑龙甩着骨板尾巴横在路上，背甲上的撞痕历历在目——它们被失去控制的荷尔蒙刺激得躁动不安。",
            rage_at: Some(30), rage_text: "剑龙群甩尾横扫，骨板像一排盾刃碾过来——伤害提升！", on_rage: jl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jl_win_l1, death: "jl_40_death",
        }),
        ("jl_raptor_pack", FightCfg {
            name: "迅猛龙群", hp: 88, dmg: (14, 22), reward: 70, reward_why: "击退丛林迅猛龙群",
            intro: "三头迅猛龙成三角队形把你去路堵死，灰绿色的鳞甲油光发亮，喉咙里的低鸣连成一片。",
            rage_at: Some(30), rage_text: "它们散开包抄，从多个方向同时发起扑击！", on_rage: jl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jl_win_l2, death: "jl_40_death",
        }),
        ("jl_stego2", FightCfg {
            name: "剑龙群 · 受惊", hp: 92, dmg: (15, 23), reward: 75, reward_why: "惊走的剑龙群",
            intro: "沼泽洼地边缘，一大群受惊的剑龙在泥泞里踩出轰响，呆呆地望着你——随后一起转身冲撞而来。",
            rage_at: Some(35), rage_text: "剑龙群彻底狂暴，骨板尾连扫带踩，整片洼地都在抖！", on_rage: jl_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jl_win_l2, death: "jl_40_death",
        }),
        ("jl_trex", FightCfg {
            name: "霸王龙", hp: 260, dmg: (18, 28), reward: 650, reward_why: "猎杀最后失控的霸王龙",
            intro: "围栏电网熄灭的巨响过后，一道山峦般的黑影正朝你碾来——霸王龙。它粗壮的颈低俯，发出能震碎玻璃的低吼，布满口水的巨齿在昏暗中发着冷光。",
            rage_at: Some(100), rage_text: "它彻底狂暴——要冲破最后一圈电网扑向你！伤害暴增，且撕咬能咬碎你的围栏战位！",
            on_rage: |_st, _log| {},
            finisher_if: |st, _| st.flag("fence_power"),
            finisher_name: |_| "合拢电网".to_string(),
            finisher_desc: |_| "你拼尽最后一搏冲回电闸，将它合拢——蓝光瞬间亮起，狂暴的霸王龙撞上电栅，全身痉挛着轰然倒地！".to_string(),
            win: |_st| "jl_13_trex_down".to_string(),
            death: "jl_40_death_trex",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn jl_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    juluoji_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 jl_ 前缀）
   ===================================================================== */
pub static JULUOJI_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 开场（园区断电）================= */
SceneDef {
    id: "jl_00", bg: Some("juluoji_bg.png"), loc: Some("侏罗纪公园 · 游客中心"),
    mood: "mystery", speaker: Some("旁白"), voice: Some("vo_jl_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>穿越失序乐园，抵达围栏区撤离台。失败代价：被扣 300 点复活。",
        "暴风雨抽打着游客中心的穹顶。墙上的全息导览图忽明忽灭，一只恐龙剪影正从断裂的围栏间踱出。广播里的男声仍在循环：<em>「园区发生大规模停电……请游客留在原地……我们正在……」</em>——然后被一声低吼盖过。",
    ]),
    choices: &[
        ChoiceDef { label: "查看断电控制台", sub: "+5 点 · 停电日志", cond: None,
            effects: &[Eff::SetFlag("jl_powerlog"), Eff::Points(5)], route: Route::To("jl_02_powerlog") },
        ChoiceDef { label: "直冲园区大门", sub: "San-3 · 立刻拉开逃生的序幕", cond: None,
            effects: &[Eff::San(-3)], route: Route::To("jl_01") },
        ChoiceDef { label: "凝神听那声低吼", sub: "获提示 · 追着你的是食肉者", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("jl_01") },
    ],
    fight_id: None, video: Some("vid_jl_opening.mp4"), cine_label: Some("过场 · 失序乐园"), overlay: None,
},

/* ---- L1 园区 hub ---- */
SceneDef {
    id: "jl_01", bg: Some("juluoji_bg_open.png"), loc: Some("L1 · 园区"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("fence_power") {
            "售货亭的冰柜门歪斜地敞着，满地摔碎的纪念品、翻倒的桌椅。远处围栏区传来不祥的「嗡——」电流声。".to_string()
        } else {
            "断电让整座园区陷入昏黄背光。售货亭冷藏门锁着；断电控制台还在跳着故障码；监控视频墙只剩雪花。迅猛龙的低吼在建筑间回荡——它离你很近。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "断电控制台", sub: "停电日志", cond: None, effects: &NO_EFF, route: Route::To("jl_02_powerlog") },
        ChoiceDef { label: "售货亭冰柜", sub: "取午餐肉", cond: None, effects: &NO_EFF, route: Route::To("jl_03_bait") },
        ChoiceDef { label: "监控视频墙", sub: "知晓恐龙动向", cond: None, effects: &NO_EFF, route: Route::To("jl_04_monitor") },
        ChoiceDef { label: "园区导览牌", sub: "园区地图", cond: None, effects: &NO_EFF, route: Route::To("jl_05_guide") },
        ChoiceDef { label: "找兽医格兰杰", sub: "NPC · 情报", cond: None, effects: &NO_EFF, route: Route::To("jl_12_granger") },
        ChoiceDef { label: "穿过电击围栏侧门", sub: "→ L2 丛林 · 迅猛龙早已在此伏击", cond: None, effects: &NO_EFF, route: Route::To("jl_02_arrive_jungle") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点 ---- */
SceneDef {
    id: "jl_02_powerlog", bg: Some("juluoji_bg_open.png"), loc: Some("L1 · 断电控制台"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["控制台屏幕跳着最后的日志：「15:42 园区主电网负载异常 → 15:43 备用电源切换失败 → 15:44 全园断电」。末行是手写的红字：<em>『有人放走了所有恐龙。』</em>"]),
    choices: &[ChoiceDef { label: "记下跌电点", sub: "主线布景", cond: None,
        effects: &[Eff::MarkPoint("jl_p_l1_console")], route: Route::To("jl_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_03_bait", bg: Some("juluoji_bg_open.png"), loc: Some("L1 · 售货亭冰柜"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_bait_meat") {
            "你已有一块午餐肉。冰柜里挤着盒装肉块、碎冰，和一柄掉漆的扳手。".to_string()
        } else {
            "冰柜门锁梁锈死，但缝隙够伸进手。冷藏肉块诱人地看着你——那是给食肉恐龙留的饵，也能在危险时引开注意。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "撬开冷藏门", sub: "需扳手", cond: Some(cond_has_wrench),
            effects: &[Eff::AddItem("it_bait_meat"), Eff::MarkPoint("jl_p_l1_souvenir")], route: Route::To("jl_01") },
        ChoiceDef { label: "强行拽开", sub: "San-4 · 噪音引来恐龙", cond: None,
            effects: &[Eff::San(-4), Eff::AddItem("it_bait_meat"), Eff::SetFlag("jl_noise")], route: Route::To("jl_01") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_04_monitor", bg: Some("juluoji_bg_open.png"), loc: Some("L1 · 监控视频墙"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["仅存的两路信号还在跳：北区密林，一头迅猛龙领着三只幼体压低身位潜行；围栏区，一道山峦般的剪影正焦躁地来回踱步——那不是迅猛龙。"]),
    choices: &[ChoiceDef { label: "看清剪影轮廓", sub: "霸王龙预告", cond: None,
        effects: &[Eff::Points(5)], route: Route::To("jl_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_05_guide", bg: Some("juluoji_bg_open.png"), loc: Some("L1 · 园区导览牌"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["导览牌上画着金字塔形地图：园门→<b>丛林</b>（沼泽洼地/密林）→<b>围栏区</b>（主电闸/围场/撤离台）。每层都标着一枚红色惊叹号。"]),
    choices: &[ChoiceDef { label: "记下逃生路线", sub: "园区→丛林→围栏区", cond: None,
        effects: &[Eff::Points(5)], route: Route::To("jl_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 丛林（逃生伏击） ================= */
SceneDef {
    id: "jl_02_arrive_jungle", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 丛林入口"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jl_jungle"),
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_bait_meat") {
            "自电击围栏侧门钻入丛林，湿热的绿气扑面。你摸出午餐肉——迅猛龙果然被香味牵动，低伏着身围了上来，但还隔着一段距离打量。".to_string()
        } else {
            "自电击围栏侧门钻入丛林。藤蔓像网一样垂着，你刚踏上泥径，三头迅猛龙便从两侧的低灌木中窜出，成三角队形把你堵死！".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "扔出午餐肉引开", sub: "需午餐肉 · 免战引导", cond: Some(cond_has_bait),
            effects: &[Eff::SetFlag("jl_used_bait")], route: Route::To("jl_02_jungle_hub") },
        ChoiceDef { label: "拔腿就跑", sub: "San-3 · 被追入密林", cond: None,
            effects: &[Eff::San(-3)], route: Route::To("jl_02_jungle_hub") },
        ChoiceDef { label: "反击迅猛龙群", sub: "战斗 jl_raptor_pack", cond: None,
            effects: &NO_EFF, route: Route::To("jl_rap_pack_fight") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_rap_pack_fight", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 丛林 · 追击战"),
    mood: "danger", speaker: Some("迅猛龙群"), voice: Some("vo_jl_raptor"),
    text: TextSpec::Static(&["三头迅猛龙已然扑上来——（战斗）"]),
    choices: &[], fight_id: Some("jl_raptor_pack"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_02_jungle_hub", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 丛林"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["密林深处的闷热让你浑身发汗。泥泞足印、撕抓树痕、岗哨补给箱——每一处都可能是迅猛龙设下的记号。而穿过这片湿绿的尽头，是通往围栏区的窄道。"]),
    choices: &[
        ChoiceDef { label: "泥泞足印", sub: "辨识恐龙动向", cond: None, effects: &NO_EFF, route: Route::To("jl_06_track") },
        ChoiceDef { label: "撕抓树痕", sub: "迅猛龙领地", cond: None, effects: &NO_EFF, route: Route::To("jl_07_marks") },
        ChoiceDef { label: "岗哨补给箱", sub: "取工具", cond: None, effects: &NO_EFF, route: Route::To("jl_08_outpost") },
        ChoiceDef { label: "找幸存游客", sub: "NPC · 情报", cond: None, effects: &NO_EFF, route: Route::To("jl_12_survivor") },
        ChoiceDef { label: "灌木丛有动静", sub: "避开被伏击的迅猛龙（绕路）", cond: None, effects: &NO_EFF, route: Route::To("jl_02_jungle_hub") },
        ChoiceDef { label: "穿越密林窄道", sub: "→ L3 围栏区 · 迎面撞上受惊剑龙", cond: None, effects: &NO_EFF, route: Route::To("jl_03_arrive_fence") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_06_track", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 泥泞足印"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["泥地上的足印又大又深，三趾，前爪几乎垂直于地面——是迅猛龙的。足迹一段段乱糟糟地交错，说明它刚才在这里盘桓，等你路过。"]),
    choices: &[ChoiceDef { label: "沿足迹反向绕开", sub: "避开伏击点", cond: None,
        effects: &[Eff::MarkPoint("jl_p_l2_track"), Eff::Points(5)], route: Route::To("jl_02_jungle_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_07_marks", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 撕抓树痕"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["一棵粗壮的蕨树树干上，三道平行的深痕直切到接近根部的高度——迅猛龙用后爪划的，是它标记领地的信号。树皮缝里卡着一截锈铁丝。"]),
    choices: &[ChoiceDef { label: "抽走铁丝", sub: "起获铁丝钳 · 开 GJ2", cond: None,
        effects: &[Eff::AddItem("it_wire_cutters"), Eff::MarkPoint("jl_p_l2_marks")], route: Route::To("jl_02_jungle_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_08_outpost", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 岗哨补给箱"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["丛林深处的补给箱弹开箱门，里面垫着碎布：一把卷刃扳手、军用水壶和半包压缩饼干。扳手是园方维修工的遗物。"]),
    choices: &[ChoiceDef { label: "取走扳手", sub: "Item it_wrench · 开冷藏门", cond: None,
        effects: &[Eff::AddItem("it_wrench"), Eff::MarkPoint("jl_p_l2_outpost")], route: Route::To("jl_02_jungle_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 围栏区 · 决战 ================= */
SceneDef {
    id: "jl_03_arrive_fence", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围栏区"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jl_fence"),
    text: TextSpec::Static(&[
        "你冲出了密林，眼前是钢筋水泥的围栏区。远处那座最高的围场里，一头霸王龙正用粗壮的尾一次次砸向电栅，焦黑铁杆上火花飞溅——它要出来了。",
        "（主电闸在观察台一侧；围栏电网 GJ3 需先拉闸恢复供电；撤离台在朝北的停机坪。）",
    ]),
    choices: &[
        ChoiceDef { label: "围栏主电闸", sub: "拉起 · 恢复电网", cond: None, effects: &NO_EFF, route: Route::To("jl_09_fuse") },
        ChoiceDef { label: "围场观察台", sub: "看清霸王龙弱点", cond: None, effects: &NO_EFF, route: Route::To("jl_10_observatory") },
        ChoiceDef { label: "仰望霸王龙", sub: "San-6 · 直面巨兽", cond: None, effects: &NO_EFF, route: Route::To("jl_13_trex") },
        ChoiceDef { label: "找濒死门卫", sub: "NPC · 遗言", cond: None, effects: &NO_EFF, route: Route::To("jl_12_guard") },
        ChoiceDef { label: "检视撤离台", sub: "→ 结算路线", cond: None, effects: &NO_EFF, route: Route::To("jl_11_exit") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_09_fuse", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围栏主电闸"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("fence_power") {
            "主电闸已经被你合拢，电网重新嗡嗡作响。霸王龙围场的电流像个劈柴的雷云，把它暂时困住了。".to_string()
        } else {
            "主电闸的把手被烧得焦黑，闸箱里爆裂的保险丝散落一地。拉下它，围栏电网才能重新供电——但这也意味着你要在大闸的瞬间直面那头巨兽。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "合拢主电闸", sub: "fence_power · 恢复电网起身的巨响惊醒霸王龙", cond: None,
        effects: &[Eff::SetFlag("fence_power"), Eff::MarkPoint("jl_p_l3_fuse")], route: Route::Dyn(route_fuse) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_10_observatory", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围场观察台"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["观察台的双筒望远镜被撞歪。调焦后你能看清：霸王龙的左腿跛得厉害，每次起身都会把重心压到另一侧——那是它唯一的弱点。"]),
    choices: &[ChoiceDef { label: "记下跛脚", sub: "BOSS 提示", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("jl_p_l3_corral")], route: Route::To("jl_03_arrive_fence") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- BOSS · 霸王龙（选择驱动）---- */
SceneDef {
    id: "jl_13_trex", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围场 · 决战"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jl_trex"),
    text: TextSpec::Static(&[
        "你踏进围场的那一刻，电栅「嗡」地彻底熄灭——不是换电成功，是被那样的重量碾断了电线。",
        "霸王龙朝你侧过头，一只浑浊的眼孔转动锁定你。它跛着脚，但试探性的、沉重的步伐已经向你逼近。",
    ]),
    choices: &[ChoiceDef { label: "【直面霸王龙】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_trex) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_trex_round", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围场 · 激战"),
    mood: "danger", speaker: Some("霸王龙"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("霸王龙 HP {} / {}", f.hp.max(0), 260)).unwrap_or_else(|| "霸王龙 HP --".to_string());
        let slammed = if st.flag("jl_slammed") { "——你刚被冲撞换位移位，下一记撕咬更凶——" } else { "" };
        let rage = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——它彻底狂暴，冲撞换位如影随形——"
        } else { "" };
        format!("{f}。{}{}", rage, slammed)
    }),
    choices: &[
        ChoiceDef { label: "瞄准跛脚重击", sub: "伤害 32-44", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| trex_act(st, rng(32, 44), false, false)) },
        ChoiceDef { label: "快刀连击", sub: "伤害 22-30", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| trex_act(st, rng(22, 30), false, false)) },
        ChoiceDef { label: "【冲撞换位】", sub: "赌一把把霸主撞离战位 · 失败反遭扑袭", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| trex_act(st, 0, true, false)) },
        ChoiceDef { label: "侧身滚避", sub: "提升闪避", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| trex_act(st, 0, false, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_13_trex_down", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围场"),
    mood: "calm", speaker: Some("旁白"), voice: Some("vo_jl_trex_down"),
    text: TextSpec::Static(&["霸王龙的巨躯掀起的尘暴里，最后一击落在它残缺的腿侧。它轰然跪地，粗重的呼吸逐渐断线，最后停在围场正中——庞大的、不再动弹的剪影。"]),
    choices: &[ChoiceDef { label: "（走向撤离台）", sub: "B 级 · 猎杀成功", cond: None,
        effects: &NO_EFF, route: Route::To("jl_11_exit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 撤离结算 ================= */
SceneDef {
    id: "jl_11_exit", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 直升机撤离台"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        let gauge = if st.flag("jl_trex_down") {
            "你把霸王龙的屠戮终结在撤离台前。直升机桨叶的呼啸里，主神光柱在停机坪中央亮起。"
        } else {
            "你绕开那头仍在咆哮的巨兽，抢在它撞破围栏前登上撤离台。主神光柱虚弱地笼住了你。"
        };
        format!("{gauge}")
    }),
    choices: &[ChoiceDef { label: "（踏入撤离光柱 · 结算）", sub: "sp_grade 结算 · 回主神空间", cond: None,
        effects: &NO_EFF, route: Route::Dyn(route_exit_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_12_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_jl_settle"), death: None,
        card: |st| crate::state::Card {
            title: "失 序 乐 园 · 逃 生".into(), good: true,
            body_html: format!(
                "<p>你在直升机桨叶掀起的尘暴里回望这座失序乐园——断电网、咆哮的兽影、烧焦的午餐肉。</p>\
                 <p style='color:#9a958a'>失序足迹：围栏电力 {} / 午餐肉 {} / 迅猛龙追迹 {}。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>B 级</td></tr>\
                 <tr><td>主神光柱</td><td>「停电只是开始。」</td></tr>\
                 </table>",
                if st.flag("fence_power") { "已恢复" } else { "未拉起" },
                if st.flag("jl_used_bait") { "已诱饵" } else { "未用" },
                if st.flag("jl_noise") { "惊动" } else { "安静" },
                st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= NPC ================= */
SceneDef {
    id: "jl_12_granger", bg: Some("juluoji_bg_open.png"), loc: Some("L1 · 游客中心"),
    mood: "cold", speaker: Some("兽医 格兰杰"), voice: None,
    text: TextSpec::Static(&["一个浑身泥泞的中年男人在门口喘着气，手里握着一柄兽医钳：「孩子，我有话要说。那头放出来的霸王龙左腿有旧伤——它追不上'换位'的人。快跑，别回头。」"]),
    choices: &[ChoiceDef { label: "收下兽医的忠告", sub: "BOSS 弱点提示", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("jl_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_12_survivor", bg: Some("juluoji_bg_invest.png"), loc: Some("L2 · 丛林"),
    mood: "cold", speaker: Some("幸存游客"), voice: None,
    text: TextSpec::Static(&["一个背包散落、脸上带着抓痕的年轻人蹲在蕨丛后：「它们不吃人，它们'猎'人……迅猛龙在丛林到处划记号。你手上可有引开它们的东西？」"]),
    choices: &[ChoiceDef { label: "问清丛林记号", sub: "情报", cond: None,
        effects: &[Eff::Points(5)], route: Route::To("jl_02_jungle_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jl_12_guard", bg: Some("juluoji_bg_battle.png"), loc: Some("L3 · 围栏区"),
    mood: "danger", speaker: Some("濒死门卫"), voice: None,
    text: TextSpec::Static(&["一个门卫半靠在观察台栏杆上，制服染红：「……跑……朝北的撤离台……电……电网，先……合闸！」他握住了你的手，另一只手指着主电闸的方向，没了气息。"]),
    choices: &[ChoiceDef { label: "合上他的眼", sub: "遗言 · 提醒先合主闸", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("jl_03_arrive_fence") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 死亡档案（复活扣 300/回主神）================= */
SceneDef {
    id: "jl_40_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("恐龙之下", "在迅猛龙的尖爪与追击中倒下")), card: |_st| crate::state::Card {
            title: "恐 龙 之 下".into(), good: false,
            body_html: r#"<p>低伏的黑影在你背后停下——你在一株被撕断的蕨树下失血过多，体温被湿热的丛林一点点抽走。</p>
<p style='color:#ff8a8a'>【死亡档案 · 恐龙之下】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "jl_40_death_trex", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("巨兽之口", "被霸王龙的撕咬与冲撞碾碎在围场")), card: |_st| crate::state::Card {
            title: "巨 兽 之 口".into(), good: false,
            body_html: r#"<p>霸王龙那坨阴影彻底盖过了你——一次冲撞把你换到它口下，撕咬咬碎了最后一道挽留。这是失序乐园最深的黑暗。</p>
<p style='color:#ff8a8a'>【死亡档案 · 巨兽之口】</p>
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
/// 拉电闸后返回围栏区 hub
fn route_fuse(_st: &mut GameState) -> String {
    "jl_03_arrive_fence".to_string()
}

/// 撤离结算：评级（B 级已由霸王龙胜利写;此处兜底）→ 卡片
fn route_exit_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('B');
    }
    "jl_12_card".to_string()
}
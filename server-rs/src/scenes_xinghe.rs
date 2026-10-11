//! 《星河战队 · 脑虫巢穴》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.1（星河战队行）+ 00_wuxian_kongbu_research.md §4.6。
//! 本文件是全新新增文件，只导出静态数据（XINGHE_SCENES / xinghe_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/xinghe_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `xh_` 前缀，与既有 SCENES 无重名；fight id 一律 `xh_` 前缀。
//! 大规模杂兵多波次增员：horde on_rage 模板（虫群 rage 后增员）+ ZoneDef 波次链（战斗场景链 win→下一波），
//! 参照 scenes_moshi.rs 多波战链。
//! BOSS 脑虫（HP220）采用"选择驱动遭遇链"（参考 scenes_zhouyuan.rs 的 zy_boss_round / scenes_jiguancheng.rs 的 colossus_round）：
//! 因需「控制虫群增员 + 精神尖啸 SAN 蚀」每回合同调，引擎原生 FightCfg 无此钩子，
//! 故用 Normal 场景 + Route::Dyn 落地；同时导出 `xh_brain` FightCfg 供 ZoneDef 与揭示用。
//! sp_grade = Some('B')（B 级支线，对齐 moruiya）。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 登陆场  井 bg xh_bg_landing（现用 img_horde.png 占位）
//!   L2 地洞    井 bg xh_bg_tunnel （现用 img_corridor.png 占位）
//!   L3 脑虫巢  井 bg xh_bg_nest    （现用 img_laser.png 占位）
//! 敌人立绘复用：horde→虫群、zombie→虫兵、hunter→巨型虫；BOSS 脑虫新立绘 enemy_brain_bug.png 待生成。

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
fn cond_has_membrane(st: &GameState) -> bool { inv(st, "it_xh_membrane") }
fn cond_rune_read(st: &GameState) -> bool { st.flag("xh_rune_read") }

/// 狂暴 + 已收集脑虫痕迹 → 「集中火力」终结（撼动控制中枢，解狂并追加伤害）
fn cond_brain_finish(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.raged).unwrap_or(false) && st.flag("xh_brain_trace")
}

/* =====================================================================
   BOSS · 脑虫（选择驱动遭遇）
   血量存 st.fight（xh_30_brain 的 Route::Dyn 初始化，引用 xh_brain 的 FightCfg）。
   每"回"是 Normal 场景 xh_brain_round；Route::Dyn 统一处理：扣脑虫血、狂暴、控制虫群增员、
   精神尖啸 SAN 蚀、胜负路由。
   ===================================================================== */
/// 初始化脑虫会话（从 xh_brain 的 FightCfg 建 Fight）
fn start_brain(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("xh_brain") {
            st.fight = Some(crate::power::scaled_fight("xh_brain", cfg, st, vec![]));
        }
    }
    "xh_brain_round".to_string()
}

/// 脑虫击杀结算：+950、掉脑虫晶核 + 巢膜残片、置 xh_brain_down、写 sp_grade=B
fn brain_win(st: &mut GameState) -> String {
    st.points += 950;
    crate::world::add_item(st, "it_xh_brain_core");
    crate::world::add_item(st, "it_xh_membrane");
    st.set_flag("xh_brain_down");
    st.sp_grade = Some('B');
    "xh_41_settle".to_string()
}

fn brain_dead() -> String { "xh_51_death_brain".to_string() }

/// 一个"回"：玩家攻击脑虫。每回固有「精神尖啸」SAN 蚀（狂暴后加重）；
/// 狂暴后每 3 回触发一次「控制虫群增员」（附加全场增员虫群啃咬 Hurt，小概率跳过）。
/// finish=true 表示狂暴后「集中火力」（固伤 + 解狂一次）。
fn brain_act(st: &mut GameState, dmg: i32, guard: bool, finish: bool) -> String {
    if finish {
        if let Some(f) = st.fight.as_mut() {
            f.hp = (f.hp - 60).max(0);
            f.raged = false;
            f.dmg = (20, 30);
        }
    } else if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return brain_win(st);
    }
    // 疯狂判定（HP ≤ 100 → 狂暴）
    let raged_now = st.fight.as_ref().map(|f| f.hp <= 100).unwrap_or(false);
    if raged_now {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    // 精神尖啸 SAN 蚀
    let shriek = if raged { 6 } else { 3 };
    st.san = (st.san - shriek).max(0);
    // 控制虫群增员：狂暴后每 3 回触发一次增援虫群啃咬（Hurt 附加）
    if raged {
        let c = if st.flag("xh_bf3") { 0 } else if st.flag("xh_bf2") { 2 } else if st.flag("xh_bf1") { 1 } else { 0 };
        if c == 0 {
            st.set_flag("xh_bf1");
        } else if c == 1 {
            st.set_flag("xh_bf2");
        } else {
            st.flags.insert("xh_bf1".to_string(), false);
            st.flags.insert("xh_bf2".to_string(), false);
            st.flags.insert("xh_bf3".to_string(), true);
            st.hp = (st.hp - 10).max(0); // 增员虫群啃咬
        }
    }
    // 脑虫反击（触须冲击 + 精神尖啸残响）
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged { rng(18, 26) } else { rng(13, 21) };
    let dodge = if guard { 0.5 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return brain_dead();
    }
    "xh_brain_round".to_string()
}

/* =====================================================================
   胜利 / 失败 / 波次增员 on_rage 回调（普通敌人；FightCfg.win / on_rage 用）
   ===================================================================== */
fn xh_win_l1(_st: &GameState) -> String { "xh_01".to_string() }
fn xh_win_tunnel(_st: &GameState) -> String { "xh_10_arrive_tunnel".to_string() }

/// 波次增员 chain win：逐波胜利路由到「休息回复节点」，再进下一场战斗
fn win_wave_a(_st: &GameState) -> String { "xh_rest_wave1".to_string() }
fn win_wave_b(_st: &GameState) -> String { "xh_rest_wave2".to_string() }
fn rage_swarm(_st: &mut GameState, log: &mut Vec<String>) {
    let _ = _st;
    log.push("<span class='crit'>虫群开始增员——四面八方爬出更多的虫兵，嘶鸣声压过了枪声。</span>".into());
}
fn rage_shriek(_st: &mut GameState, log: &mut Vec<String>) {
    let _ = _st;
    log.push("<span class='crit'>巨型虫甲壳爆出裂响，口器高抬，发出刺耳的脑波尖叫——它要啃击重创。</span>".into());
}

/// 战斗配置表（id 全部 xh_ 前缀）。数值对齐 00_ENGINE_CONTEXT §2.3 蜂巢基线。
pub fn xinghe_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("xh_warrior", FightCfg {
            name: "虫兵", hp: 46, dmg: (9, 15), reward: 15, reward_why: "虫兵 · 击毙",
            intro: "锯齿螯足的虫兵破土而出，口器滴着酸液，向你扑来——这是 P 星虫族军团的最普通一兵。",
            rage_at: Some(22), rage_text: "", on_rage: rage_shriek,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: xh_win_l1, death: "xh_50_death",
        }),
        ("xh_swarm", FightCfg {
            name: "虫群", hp: 68, dmg: (13, 21), reward: 30, reward_why: "虫群 · 击退",
            intro: "黑影般涌来的虫群如潮水，节肢相互碰撞发出密集的咔嚓声——一支小队，对上一千只虫的预演。",
            rage_at: Some(32), rage_text: "", on_rage: rage_swarm,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: xh_win_l1, death: "xh_50_death",
        }),
        ("xh_giant", FightCfg {
            name: "巨型虫", hp: 130, dmg: (18, 27), reward: 120, reward_why: "巨型虫 · 猎杀",
            intro: "两足多高的巨型虫横在通道中央，金属般油亮的甲壳在菌毯上反光，螯足扫出一圈尘土——虫族的精英阶。",
            rage_at: Some(55), rage_text: "<b>爆壳狂化</b>——甲壳裂开，口器高抬，进入高机动暴走，伤害暴涨！", on_rage: rage_shriek,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: xh_win_l1, death: "xh_50_death",
        }),
        // ---- 登陆场多波增员链（win 逐波衔接，波次间休息回满血）----
        ("xh_wave_a", FightCfg {
            name: "虫群 · 第一波", hp: 68, dmg: (13, 21), reward: 30, reward_why: "登陆场第一波虫群 · 击退",
            intro: "登陆场四周的菌毯鼓起，第一批虫群从地表爬出，嘶咬着压向你撑出的防线。",
            rage_at: Some(32), rage_text: "", on_rage: rage_swarm,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_wave_a, death: "xh_50_death",
        }),
        ("xh_wave_b", FightCfg {
            name: "虫兵群 · 第二波", hp: 78, dmg: (12, 20), reward: 35, reward_why: "登陆场第二波虫兵群 · 击退",
            intro: "第一波被钉死在缺口，第二波虫兵从崩裂的岩缝里涌出——它们在增员。",
            rage_at: Some(36), rage_text: "", on_rage: rage_swarm,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_wave_b, death: "xh_50_death",
        }),
        ("xh_wave_c", FightCfg {
            name: "虫群 · 第三波", hp: 88, dmg: (15, 24), reward: 45, reward_why: "登陆场第三波虫群 · 清场",
            intro: "最后一波黑压压的虫群漫上来，把登陆场围成一座嘶鸣的孤岛。",
            rage_at: Some(40), rage_text: "", on_rage: rage_swarm,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: xh_win_l1, death: "xh_50_death",
        }),
        // ---- 地洞隧道增员遭遇（巨型虫 + 虫群）----
        ("xh_tunnel_swarm", FightCfg {
            name: "隧道虫群 · 增员", hp: 78, dmg: (14, 22), reward: 40, reward_why: "地洞隧道虫群增员 · 击退",
            intro: "地道深处传来密集的节肢声响——脑虫的脑波传来，更多虫群沿着菌毯向你集合。",
            rage_at: Some(36), rage_text: "", on_rage: rage_swarm,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: xh_win_tunnel, death: "xh_50_death",
        }),
        // ---- BOSS · 脑虫（导出 FightCfg 供 start_brain / ZoneDef 揭示）----
        ("xh_brain", FightCfg {
            name: "脑虫", hp: 220, dmg: (13, 21), reward: 950, reward_why: "终结脑虫 · 摧毁虫族指挥中枢",
            intro: "高台中央的菌毯上盘踞着一头人高的脑虫，淡青的巨颅半透明，脑膜下无数神经束如血管般搏动——虫群千螯之上，唯一能思考的那只。",
            rage_at: Some(100), rage_text: "<b>疯狂</b>——脑虫巨颅猛地一胀，精神尖啸如针扎进识海，全场虫群开始失控增员！",
            on_rage: rage_swarm,
            finisher_if: |st, _| st.flag("xh_brain_trace") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "集中火力".to_string(),
            finisher_desc: |_| "你辨识出脑虫精神尖啸的『频段』，小队齐射装甲喷流——脑波的指挥频段被你打断，狂化中枢短暂失效，追加 60 点固伤！".to_string(),
            win: |_st| "xh_41_settle".to_string(),
            death: "xh_51_death_brain",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn xinghe_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    xinghe_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 xh_ 前缀）
   ===================================================================== */
pub static XINGHE_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 开场 & 登陆场（xh_00） ================= */
SceneDef {
    id: "xh_00", bg: Some("xinghe_bg.png"), loc: Some("L1 · 登陆场"),
    mood: "danger", speaker: Some("特战队口令"), voice: Some("vo_xinghe_1"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>虫族军团入侵 P 星。特种小队偷入脑虫巢，摧毁虫族指挥中枢。失败代价：被扣 500 点复活。",
        "天空被登陆舰的尾焰撕成一道橙红，尘暴过后，旷野上全是刨开的菌毯与虫尸。通讯耳麦里挤出一句沙沙的军令：「一支小队，对上一千只虫——别去送死，去找那只会思考的虫子。」",
        "（登陆舱残骸、通讯信标、尸兵野战队记可调查；虫群在四面八方逼近。）",
    ]),
    choices: &[
        ChoiceDef { label: "查看登陆舱残骸", sub: "+10 · 调查", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l1_1"), Eff::Points(10)], route: Route::To("xh_02_craft") },
        ChoiceDef { label: "调试通讯信标", sub: "剧情·主线提示", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l1_2"), Eff::Points(10)], route: Route::To("xh_03_beacon") },
        ChoiceDef { label: "翻看尸兵野战队记", sub: "伏笔 · 装甲动力格", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l1_3"), Eff::AddItem("it_xh_armor")], route: Route::To("xh_04_hunter_note") },
        ChoiceDef { label: "【迎战登陆场虫群】", sub: "登陆场波次链 · 第一波", cond: None,
            effects: &NO_EFF, route: Route::To("xh_combat_wave1") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点场景 ---- */
SceneDef {
    id: "xh_02_craft", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 登陆舱残骸"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["登陆舱半埋在尘土里，舱门被蚀成锯齿。舱壁的武器架上还挂着一副动力装甲——你剥下一块『装甲动力格』。"]),
    choices: &[ChoiceDef { label: "取下装甲动力格", sub: "Item it_xh_armor · 开 G1 装甲闸", cond: None,
        effects: &[Eff::AddItem("it_xh_armor")], route: Route::To("xh_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_03_beacon", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 通讯信标"),
    mood: "danger", speaker: Some("通讯广播"), voice: None,
    text: TextSpec::Static(&["你用装甲脉冲把信标点亮，全频广播里滚过一段断断续续的军令：「……地洞……脑虫……它用脑波调兵……斩首，斩首……」"]),
    choices: &[ChoiceDef { label: "记下军令", sub: "剧情提示", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("xh_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_04_hunter_note", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 尸兵野战队记"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["一具被螯足劈穿的尸兵压在战术板上，指头还按在一行字上：「虫群是手，脑虫是脑。手可以砍一万次，脑子只要砍一次。」"]),
    choices: &[ChoiceDef { label: "收起战术板", sub: "剧情伏笔", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("xh_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 登陆场 · 多波次增员（战斗场景链：波1→回血→波2→回血→波3→过） ---- */
SceneDef {
    id: "xh_combat_wave1", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 登陆场"),
    mood: "danger", speaker: Some("⚔ 第一波"), voice: None,
    text: TextSpec::Static(&["第一批虫群从地表拱出，嘶咬着压向你撑出的防线。你抬枪迎上——"]),
    choices: &[ChoiceDef { label: "迎战", sub: "虫群 · 第一波", cond: None, effects: &NO_EFF, route: Route::To("xh_combat_wave2") }],
    fight_id: Some("xh_wave_a"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_rest_wave1", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 岩台掩体后"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["第一波被你钉死在缺口。你退回岩台掩体后，上紧弹药、喘匀一口气——增员的低鸣没有停。"]),
    choices: &[ChoiceDef { label: "（休整回满血）", sub: "HP 回满 → 第二波", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "xh_combat_wave2".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_combat_wave2", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 登陆场"),
    mood: "danger", speaker: Some("⚔ 第二波"), voice: None,
    text: TextSpec::Static(&["第二波虫兵从崩裂的岩缝里涌出——它们在增员，越杀越多。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "虫兵群 · 第二波", cond: None, effects: &NO_EFF, route: Route::To("xh_combat_wave3") }],
    fight_id: Some("xh_wave_b"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_rest_wave2", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 岩台掩体后"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["第二波虫兵被击散。你咬着止血带把渗血的护甲带勒紧，把最后的体力留给最后一波。"]),
    choices: &[ChoiceDef { label: "（休整回满血）", sub: "HP 回满 → 第三波", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "xh_combat_wave3".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_combat_wave3", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 登陆场"),
    mood: "danger", speaker: Some("⚔ 第三波"), voice: None,
    text: TextSpec::Static(&["最后一波黑压压的虫群漫上来，把登陆场围成一座嘶鸣的孤岛。清掉它们，找出地洞的入口。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "虫群 · 第三波", cond: None, effects: &NO_EFF, route: Route::To("xh_01") }],
    fight_id: Some("xh_wave_c"), video: None, cine_label: None, overlay: None,
},

/* ---- L1 过关 hub：地洞入口 ---- */
SceneDef {
    id: "xh_01", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 登陆场 · 地洞入口"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "虫群退潮般缩回地底，登陆场归于一片死寂。不远处一处坍裂的岩缝露出一道向下的黑口，泥土里全是新鲜刨开的菌毯——脑虫把它们都叫回家了。",
        "（装甲闸 G1 在东南角，若无动力格只能绕过虫群密集区；地洞入口在东北角岩缝下。）",
    ]),
    choices: &[
        ChoiceDef { label: "跃下地洞（坠洞）", sub: "p_xh_1 单向坠入 → L2", cond: None,
            effects: &NO_EFF, route: Route::To("xh_10_arrive_tunnel") },
        ChoiceDef { label: "再清剿残余虫兵", sub: "战斗 xh_warrior", cond: None,
            effects: &NO_EFF, route: Route::To("xh_01_skirmish") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_01_skirmish", bg: Some("xinghe_bg_open.png"), loc: Some("L1 · 登陆场"),
    mood: "danger", speaker: None, voice: Some("vo_xh_warrior"),
    text: TextSpec::Static(&["一只漏网的虫兵从尸堆里破土而出，螯足几乎擦着你的喉——（战斗）"]),
    choices: &[],
    fight_id: Some("xh_warrior"), video: None, cine_label: None, overlay: None,
},

/* ================= 幕二 · 地洞隧道（xh_10 / xh_11） ================= */
SceneDef {
    id: "xh_10_arrive_tunnel", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 地洞入口"),
    mood: "cold", speaker: Some("老兵 · 里科"), voice: None,
    text: TextSpec::Static(&[
        "坠入地洞那一刻，头顶的菌毯在身后合拢。黑暗里响起一个压低的嗓音——老兵里科举着战术电筒：「走地洞，能避开地表虫群的大部队。但这里……是它们的窝。」",
        "（菌毯卵堆、地道壁画·虫纹、溶洞巢蜥残骸可调查；深坑闸 G2 需解读地道壁画虫纹方能垂降。）",
    ]),
    choices: &[
        ChoiceDef { label: "调查菌毯卵堆", sub: "调查", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l2_1"), Eff::Points(10)], route: Route::To("xh_11_egg") },
        ChoiceDef { label: "解读地道壁画 · 虫纹", sub: "置 xh_rune_read · 开 G2", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l2_2"), Eff::SetFlag("xh_rune_read")], route: Route::To("xh_12_rune") },
        ChoiceDef { label: "查看溶洞巢蜥残骸", sub: "调查 · 巨型虫足迹", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l2_3"), Eff::Points(10)], route: Route::To("xh_13_gainttrack") },
        ChoiceDef { label: "【迎战地洞虫群增员】", sub: "隧道波次链 xh_tunnel_swarm", cond: None,
            effects: &NO_EFF, route: Route::To("xh_combat_tunnel") },
        ChoiceDef { label: "深坑垂降（需解读虫纹）", sub: "G2 → L3 脑虫巢 · 单向", cond: Some(cond_rune_read), effects: &NO_EFF, route: Route::To("xh_14_drop") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_11_egg", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 菌毯卵堆"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["菌毯下压着一窝半透明虫卵，卵壁上能看见蜷缩的幼虫。里科捻灭电筒：「别碰。脑虫就靠这些续它的军团。」"]),
    choices: &[ChoiceDef { label: "记下卵堆分布", sub: "剧情", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("xh_10_arrive_tunnel") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_12_rune", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 地道壁画 · 虫纹"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["隧洞壁上刻满一圈圈虫纹，像某只的脑波轨迹。你随纹路推演半晌，读懂了那句兽语般的指挥令——「垂降的节奏，是三长两短」。（xh_rune_read）"]),
    choices: &[ChoiceDef { label: "记下虫纹节律", sub: "xh_rune_read · 开 G2", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("xh_10_arrive_tunnel") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_13_gainttrack", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 溶洞巢蜥残骸"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["一具巨型虫被啃得只剩甲壳，钜裂的创伤边缘敷着黏腻的菌毯——脑虫拿它喂了新孵的虫群。你识得了巨型虫的猎杀特征。"]),
    choices: &[ChoiceDef { label: "记下巨型虫特征", sub: "战斗对策", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("xh_10_arrive_tunnel") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 地洞隧道虫群增员（多波次环节：战斗场景链 + 休息） */
SceneDef {
    id: "xh_combat_tunnel", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 隧道岔口"),
    mood: "danger", speaker: Some("⚔ 隧道增员"), voice: None,
    text: TextSpec::Static(&["地道深处传来密集的节肢声响——脑虫的脑波传来，更多虫群沿着菌毯向你集合。一只巨型虫当先堵住岔口。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "隧道虫群 · 增员", cond: None, effects: &NO_EFF, route: Route::To("xh_11_tunnel_rest") }],
    fight_id: Some("xh_tunnel_swarm"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_11_tunnel_rest", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 隧道岩龛后"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["隧道虫群被你扫空一歇，你退进一处岩龛，把护甲带勒紧，等脑虫再派下一批之前冲过去。"]),
    choices: &[ChoiceDef { label: "（休整回满血）", sub: "HP 回满", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "xh_10_arrive_tunnel".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_14_drop", bg: Some("xinghe_bg_invest.png"), loc: Some("L2 · 深坑垂降"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你默念虫纹的三长两短，菌毯分解开一架钢索吊筐。你踏上去，黑暗里一路下沉——头顶的菌毯在身后合拢，再无回头。"]),
    choices: &[ChoiceDef { label: "（深坑垂降至脑虫巢）", sub: "p_xh_2 单向 · 进 L3", cond: None,
        effects: &NO_EFF, route: Route::To("xh_20_arrive_nest") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 地沟潜行（险路，p_xh_3）入口场景——从 hub 也可选（此处不加选项，仅闭环存在） */

/* ================= 幕三 · 脑虫巢决战（xh_20 / xh_30） ================= */
SceneDef {
    id: "xh_20_arrive_nest", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 脑虫巢 · 巢穴外周"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "深坑尽头豁然开朗——一座由菌毯与骸骨垒成的中枢巢穴。半透明的巢膜盖着高台，膜面下隐约可见一团蠕动的淡青色巨影。",
        "巢膜观察窗可窥探；脑虫高台入口被一层核心巢膜封住（G3），需『脑虫巢膜残片』方可破膜直入。",
    ]),
    choices: &[
        ChoiceDef { label: "巢膜观察窗窥视", sub: "剧情 · 脑虫状态", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l3_1"), Eff::Points(10)], route: Route::To("xh_21_observe") },
        ChoiceDef { label: "剐取脑虫巢膜残片", sub: "Item it_xh_membrane · 开 G3", cond: None,
            effects: &[Eff::MarkPoint("xh_p_l3_2"), Eff::AddItem("it_xh_membrane")], route: Route::To("xh_22_membrane") },
        ChoiceDef { label: "【迎战巢卫虫群】", sub: "战斗 xh_swarm", cond: None,
            effects: &NO_EFF, route: Route::To("xh_23_nest_guard") },
        ChoiceDef { label: "破膜逼近脑虫高台", sub: "需巢膜残片 → BOSS 决战", cond: Some(cond_has_membrane), effects: &NO_EFF, route: Route::To("xh_30_brain") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_21_observe", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 巢膜观察窗"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["透过淡青的巢膜，你看见那只脑虫——巨颅半透明，脑膜下神经束搏动如血脉，一道无形的脑波正向外辐射指挥虫群。你记下了它尖啸的『频段』。（xh_brain_trace）"]),
    choices: &[ChoiceDef { label: "记下脑波频段", sub: "xh_brain_trace · 终结前置", cond: None,
        effects: &[Eff::SetFlag("xh_brain_trace"), Eff::Points(10)], route: Route::To("xh_20_arrive_nest") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_22_membrane", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 巢膜边缘"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你从巢膜边沿剐下一片带着黏液的残片——这是脑虫巢的东西，也是唯一能破开高台那层核心巢膜的『钥匙』。" ]),
    choices: &[ChoiceDef { label: "收下巢膜残片", sub: "Item it_xh_membrane · 开 G3", cond: None,
        effects: &[Eff::AddItem("it_xh_membrane")], route: Route::To("xh_20_arrive_nest") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_23_nest_guard", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 巢穴外周"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["巢卫虫群从骸骨缝隙里爬出，朝你收拢——（战斗）"]),
    choices: &[],
    fight_id: Some("xh_swarm"), video: None, cine_label: None, overlay: None,
},

/* ---- BOSS · 脑虫（选择驱动遭遇） ---- */
SceneDef {
    id: "xh_30_brain", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 脑虫高台 · 决战"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你将巢膜残片贴上膜面，气流般向两侧裂开。高台上的脑虫缓缓转过半透明的巨颅，无数神经束同时转向你——整个巢穴的虫群在同一瞬抬起头。",
        "「……一千只虫，压不过一颗脑子。」你端起装甲喷流。脑虫发出第一声精神尖啸。",
    ]),
    choices: &[ChoiceDef { label: "【逼近脑虫】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_brain) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_brain_round", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 脑虫高台 · 激战"),
    mood: "danger", speaker: Some("脑虫"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("脑虫 HP {} / {}", f.hp.max(0), 220)).unwrap_or_else(|| "脑虫 HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——疯狂！精神尖啸每回合蚀 SAN 6，且虫群失控增员啃咬——"
        } else { "——精神尖啸每回合蚀 SAN 3——" };
        format!("{f}。{}", mode)
    }),
    choices: &[
        ChoiceDef { label: "装甲喷流（强攻）", sub: "伤害 34-48", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| brain_act(st, rng(34, 48), false, false)) },
        ChoiceDef { label: "突击步枪连射（速攻）", sub: "伤害 24-32", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| brain_act(st, rng(24, 32), false, false)) },
        ChoiceDef { label: "【集中火力】", sub: "需狂暴+脑波频段 · 60 固伤并解狂一次", cond: Some(cond_brain_finish),
            effects: &NO_EFF, route: Route::Dyn(|st| brain_act(st, 60, false, true)) },
        ChoiceDef { label: "龟缩掩体", sub: "提升闪避 · 抑尖啸", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| brain_act(st, 0, true, false)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_41_settle", bg: Some("xinghe_bg_battle.png"), loc: Some("L3 · 脑虫高台"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "你辨出脑波频段，装甲喷流与突击火力齐射贯穿脑虫的巨颅——它轰然倒伏，整座巢穴的虫群在九十万分之一秒里失去指挥，如退潮般四散溃逃。",
        "斩首成功。P 星的地洞，为你洞开一条通往主神空间的撤离阵。",
    ]),
    choices: &[
        ChoiceDef { label: "（走向撤离阵 · 结算）", sub: "sp_grade B · 回主神空间", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st: &mut GameState| { if st.sp_grade.is_none() { st.sp_grade = Some('B'); } "xh_42_card".into() }) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "xh_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "斩 首 · 脑 虫".into(), good: true,
            body_html: format!(
                "<p>脑虫的淡青巨颅在你脚下熄灭。失去指挥中枢的虫族军团，在 P 星地表四散溃逃。</p>\
                 <p style='color:#ffd76a'>【主神】：特种小队完成斩首。全歼虫群指挥中枢——一支小队，确实杀穿了那一千只虫。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>B 级（大规模杂兵战）</td></tr>\
                 <tr><td>掉落</td><td>脑虫晶核 it_xh_brain_core · 巢膜残片</td></tr>\
                 </table>",
                st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案（复活扣 500/回主神） ================= */
SceneDef {
    id: "xh_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("菌毯上的无名者", "在 P 星的虫群里倒下")), card: |_st| crate::state::Card {
            title: "虫 海 之 下".into(), good: false,
            body_html: r#"<p>螯足与嘶鸣把你淹没在菌毯黑潮里。那群虫甚至没停步，从你尚温的骨头边爬过头也不回地赶往下一片血。</p>
<p style='color:#ff8a8a'>【死亡档案 · 虫海之下】</p>
<p style='color:#666'>（复活：回主神空间扣 500 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "xh_51_death_brain", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("尖啸中枢", "精神尖啸蚀尽识海，你在脑虫巢穴倒下未竟的斩首")), card: |_st| crate::state::Card {
            title: "尖 啸 中 枢".into(), good: false,
            body_html: r#"<p>脑虫的精神尖啸像一枚楔子钉进你的识海——你抱着头倒下，被失控的虫群嚼碎在它高台与撤离阵之间。</p>
<p style='color:#ff8a8a'>【死亡档案 · 尖啸中枢】</p>
<p style='color:#666'>（复活：回主神空间扣 500 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
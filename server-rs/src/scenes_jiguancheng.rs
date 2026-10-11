//! 《侠行天下 · 机关城核心》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/xiaxing_tianxia/jiguancheng.md §4/§5/§6/§7/§8。
//! 本文件是全新新增文件，只导出静态数据（JIGUAN_SCENES / jiguancheng_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/jiguancheng_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `jg_` 前缀，与既有 SCENES 无重名；fight id 全部 `jc_` 前缀。
//! BOSS 巨像 / 隐藏守城人采用"选择驱动遭遇链"（参考 scenes_zhouyuan.rs 的 zy_boss_round/route_boss_attack）：
//! 因需要「三令夺枢」终结 / 齿轮碾阵每 3 回场地判定 / 出示墨令免战等自定义每回合同调，引擎原生 FightCfg
//! 无此钩子，故用 Normal 场景 + Route::Dyn 落地；同时导出 `jc_colossus`/`jc_keeper` FightCfg 供 ZoneDef 与揭示用。
//! 三启 flag 拼合：三个齿轮机关各置 gear_sw_a/b/c（设计 §3 命名），三处齐发 → 置 gear_puzzle_clear（结算+G2 升降梯）
//! 与 jg_pivot_gate（接线概念闭环标志，任务约定）。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L1 城门  井 bg jg_bg_gate    （现用 img_zhuyuan_book.png 占位）
//!   L2 工坊  井 bg jg_bg_workshop（现用 img_laser.png 占位）
//!   L3 回廊  井 bg jg_bg_corridor（现用 img_corridor.png 占位）
//!   L4 核心  井 bg jg_bg_core    （现用 img_corridor.png 占位）
//!   L4 静室  井 bg jg_bg_keeper_room（现用 img_zhuyuan_book.png 占位）
//! 敌人立绘复用 §9.2：guard→墨卫、zombie→齿轮兽、hunter→弩手；新美术由主 agent 统一生图替换。

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
   具名条件谓词（cond：fn 指针，不能捕获闭包；逐个具名定义供 static 数组使用）
   ===================================================================== */
fn cond_has_pivot_key(st: &GameState) -> bool { inv(st, "it_pivot_key") }
fn cond_has_mo_ling_a(st: &GameState) -> bool { inv(st, "it_mo_ling_a") }
fn cond_has_mo_ling_full(st: &GameState) -> bool { inv(st, "it_mo_ling_full") }
fn cond_gear_clear(st: &GameState) -> bool { st.flag("gear_puzzle_clear") }
fn cond_mo_ling_broken(st: &GameState) -> bool { st.flag("mo_ling_broken") }
/// 三枚枢机令齐备 + 巨像已狂暴 → 「以令夺枢」可选
fn cond_three_tokens_raged(st: &GameState) -> bool {
    inv(st, "it_gear_token_a")
        && inv(st, "it_gear_token_b")
        && inv(st, "it_gear_token_c")
        && st.fight.as_ref().map(|f| f.raged).unwrap_or(false)
}
/// 守城人狂暴后仍可出示完整墨令免战
fn cond_keeper_finish(st: &GameState) -> bool {
    inv(st, "it_mo_ling_full") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false)
}



/* =====================================================================
   BOSS · 枢机巨像（选择驱动遭遇）
   血量存 st.fight（jg_22_colossus 的 Route::Dyn 初始化，引用 jc_colossus 的 FightCfg）。
   每"回"是 Normal 场景 jg_colossus_round；Route::Dyn 统一处理：扣巨像血、狂暴、齿轮碾阵、胜负路由。
   ===================================================================== */
/// 初始化巨像会话（从 jc_colossus 的 FightCfg 建 Fight）。需主线合并后 fight_cfg 能解析 jc_colossus
fn start_colossus(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jc_colossus") {
            st.fight = Some(crate::power::scaled_fight("jc_colossus", cfg, st, vec![]));
        }
    }
    "jg_colossus_round".to_string()
}

/// 巨像击杀结算：+550、掉图谱残页+枢机核、置 jc_colossus_down、写 sp_grade=D
fn colossus_win(st: &mut GameState) -> String {
    st.points += 550;
    crate::world::add_item(st, "it_wuxue_map");
    crate::world::add_item(st, "it_colossus_core");
    st.set_flag("jc_colossus_down");
    st.sp_grade = Some('D');
    "jg_23_colossus_down".to_string()
}

/// 齿轮碾阵：每 3 回一次全场判定（每名队员 Hurt；jg_qinggong 轻功 flag 可减半）。用 jg_c1/c2/c3 循环计数。
fn gear_crush(st: &mut GameState) {
    let crush = if st.flag("jg_qinggong") { 4 } else { 8 };
    if st.flag("jg_c3") {
        st.hp = (st.hp - crush).max(0);
        st.flags.insert("jg_c1".to_string(), true);
        st.flags.insert("jg_c2".to_string(), false);
        st.flags.insert("jg_c3".to_string(), false);
    } else if st.flag("jg_c2") {
        st.set_flag("jg_c3");
    } else if st.flag("jg_c1") {
        st.set_flag("jg_c2");
    } else {
        st.set_flag("jg_c1");
    }
}

/// 玩家死亡路由
fn colossus_dead() -> String { "jg_40_death_colossus".to_string() }

/// 一个"回"：玩家攻击巨像。yi_ling=true 表示三令夺枢（40 固伤 + 解除狂暴）。
fn colossus_act(st: &mut GameState, dmg: i32, yi_ling: bool, guard: bool) -> String {
    // 三令夺枢：解除狂暴
    if yi_ling {
        if let Some(f) = st.fight.as_mut() {
            f.raged = false;
            f.dmg = (16, 24);
        }
        st.flags.insert("jg_c1".to_string(), false);
        st.flags.insert("jg_c2".to_string(), false);
        st.flags.insert("jg_c3".to_string(), false);
    }
    // 玩家进攻（guard = 后撤观察，不出手）
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return colossus_win(st);
    }
    // 狂暴触发（HP ≤ 70）
    let raged = st.fight.as_ref().map(|f| f.hp <= 70).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    // 齿轮碾阵：每 3 回全场判定
    gear_crush(st);
    // 巨像反击
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(22, 30) } else { rng(16, 24) };
    let dodge = if guard { 0.55 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return colossus_dead();
    }
    "jg_colossus_round".to_string()
}

/* =====================================================================
   隐藏 BOSS · 入魔的守城人（选择驱动遭遇）
   ===================================================================== */
fn start_keeper(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jc_keeper") {
            st.fight = Some(crate::power::scaled_fight("jc_keeper", cfg, st, vec![]));
        }
    }
    "jg_keeper_round".to_string()
}

/// 守城人解脱：+400 全奖、信物、keeper_freed
fn keeper_freed(st: &mut GameState) -> String {
    st.points += 400;
    crate::world::add_item(st, "it_shoucheng_token");
    st.set_flag("keeper_freed");
    "jg_26_keeper_freed".to_string()
}

fn keeper_dead() -> String { "jg_41_death_keeper".to_string() }

fn keeper_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return keeper_freed(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 60).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
        st.hp = (st.hp - 3).max(0); // 执着反噬自身 Hurt(-3) 之外的玩家侧反震
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(19, 27) } else { rng(14, 22) };
    let dodge = if guard { 0.5 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return keeper_dead();
    }
    "jg_keeper_round".to_string()
}

/* =====================================================================
   胜利 / 失败共通 win 回调（普通敌人；FightCfg.win 用）
   ===================================================================== */
fn jg_win_l1(_st: &GameState) -> String { "jg_01".to_string() }
fn jg_win_l2(_st: &GameState) -> String { "jg_02_arrive_workshop".to_string() }
fn jg_win_l3(_st: &GameState) -> String { "jg_03_arrive_corridor".to_string() }
fn jg_win_l4(_st: &GameState) -> String { "jg_21_arrive_core".to_string() }
fn jg_win_gearbest2(_st: &GameState) -> String { "jg_13_gearbest_win".to_string() }
fn jg_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 战斗配置表（id 全部 jc_ 前缀）。
pub fn jiguancheng_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("jc_sentry", FightCfg {
            name: "巡城铁哨", hp: 34, dmg: (7, 13), reward: 10, reward_why: "肃清深巷哨兵",
            intro: "齿轮声在一尊昂首的铁铸哨兵体内嗡嗡作响，锈蚀的眼窝亮起冷光。",
            rage_at: None, rage_text: "", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l1, death: "jg_40_death",
        }),
        ("jc_guard1", FightCfg {
            name: "铜护楼傀儡", hp: 36, dmg: (8, 14), reward: 12, reward_why: "击碎东裂隙铜护楼傀儡",
            intro: "青铜木傀横在裂隙口，肩上的齿轮缓缓咬合，朝你摆开守关架势。",
            rage_at: None, rage_text: "", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l1, death: "jg_40_death",
        }),
        ("jc_guard2", FightCfg {
            name: "机关傀儡 · 墨卫", hp: 42, dmg: (11, 18), reward: 25, reward_why: "机关傀儡 · 墨卫",
            intro: "墨黑油面的木躯立于齿轮阵间，青铜护臂张开，眼窝橙光跃动。",
            rage_at: Some(20), rage_text: "墨甲炸裂而开，露出体内绞动的齿轮——它攻来的段数增加了！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l2, death: "jg_40_death",
        }),
        ("jc_bowman", FightCfg {
            name: "毒箭弩手", hp: 50, dmg: (10, 16), reward: 30, reward_why: "拔除天桥毒箭弩手",
            intro: "半跪的墨衣射手扬手挺弩，森绿的毒矢袋在腰间晃动。",
            rage_at: Some(24), rage_text: "它换装成毒矢连射，箭雨附加中毒草——每回合你被蚕食 3 点气血！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l2, death: "jg_40_death",
        }),
        ("jc_gearbest", FightCfg {
            name: "齿轮兽", hp: 60, dmg: (11, 17), reward: 40, reward_why: "西爬梯齿轮兽",
            intro: "青铜齿轮拼合的鼍形机械兽伏在爬梯口，腹部主齿轮缓转，锈渣簌簌落下。",
            rage_at: Some(28), rage_text: "齿轮变向急转，机械兽的闪避陡然提升！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l2, death: "jg_40_death",
        }),
        ("jc_guard3", FightCfg {
            name: "机关傀儡 · 铁卫", hp: 50, dmg: (11, 17), reward: 35, reward_why: "枢机回廊 · 铁卫",
            intro: "双持机臂的铁卫在桥门楼窖住廊道，机簧连响，随时会发起连击。",
            rage_at: Some(25), rage_text: "双持机臂连打，攻势如雨！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l3, death: "jg_40_death",
        }),
        ("jc_gearbest2", FightCfg {
            name: "巨齿轮兽", hp: 65, dmg: (12, 18), reward: 50, reward_why: "击破中央桥下巨齿轮兽 · 掉落断裂墨令·乙",
            intro: "桥下阴影里，一头足有两三人高的巨齿轮兽低吼，周身齿轮绞动如磨盘。",
            rage_at: Some(30), rage_text: "它发出机械咆哮，整个桥院的地面齿轮都跟着绞动！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_gearbest2, death: "jg_40_death",
        }),
        ("jc_bowwall", FightCfg {
            name: "弩阵 · 毒矢机括", hp: 72, dmg: (12, 19), reward: 70, reward_why: "拆毁弩阵 · 毒矢机括",
            intro: "整面墙的青铜臂弩同时抬指，机括每一个都在高速上弦。",
            rage_at: Some(35), rage_text: "机括齐射，全屏矢雨倾泻而下！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l3, death: "jg_40_death",
        }),
        ("jc_guard4", FightCfg {
            name: "枢机护卫 · 玄甲", hp: 80, dmg: (13, 20), reward: 70, reward_why: "击破枢机护卫 · 玄甲",
            intro: "全青铜重甲机关武士横在祭台前，胸前墨玉核幽幽发亮。",
            rage_at: Some(35), rage_text: "护心甲轰然弹开，它转为纯攻，再无防守！", on_rage: jg_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: jg_win_l4, death: "jg_40_death",
        }),
        ("jc_colossus", FightCfg {
            name: "枢机巨像", hp: 160, dmg: (16, 24), reward: 550, reward_why: "击碎枢机巨像，夺其核心",
            intro: "五米青铜机关巨人从虚空踏出，三层齿轮颅缓缓展开，胸口墨玉核心曝出暗金光芒。",
            rage_at: Some(70), rage_text: "周身齿轮外翻，墨玉核心直射——<b>狂暴齿轮碾阵</b>开启！伤害暴增，且每三回合碾压全场！",
            on_rage: |_st, _log| {},
            finisher_if: |st, _| inv(st, "it_gear_token_a") && inv(st, "it_gear_token_b")
                && inv(st, "it_gear_token_c") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "以令夺枢".to_string(),
            finisher_desc: |_| "三枚枢机令嵌入巨像背脊的机关巢，机械巨身轰然跪伏——枢机被夺，狂化解除！".to_string(),
            win: |_st| "jg_23_colossus_down".to_string(),
            death: "jg_40_death_colossus",
        }),
        ("jc_keeper", FightCfg {
            name: "入魔的守城人", hp: 140, dmg: (14, 22), reward: 400, reward_why: "解脱入魔的守城人",
            intro: "枯坐的老人缓缓抬头，双目赤红，铜链自朽木中生出缠上双臂——他开口，九个字沙哑若机械：「你……来晚了三百年。」",
            rage_at: Some(60), rage_text: "入魔执念显化，铜链化为鞭刃——伤害提升，执着反噬自身！",
            on_rage: |_st, _log| {},
            finisher_if: |st, _| inv(st, "it_mo_ling_full") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "出示墨令".to_string(),
            finisher_desc: |_| "你递出那枚完整的墨令。铜链在半空凝滞——守城人怔住，赤红如潮水般退去。「……三百年前以命封匣，今日……你来了。」".to_string(),
            win: |_st| "jg_26_keeper_freed".to_string(),
            death: "jg_41_death_keeper",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn jg_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    jiguancheng_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 jg_ 前缀）
   ===================================================================== */
pub static JIGUAN_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 开场（s_jc_open）================= */
SceneDef {
    id: "jg_00", bg: Some("jiguancheng_bg.png"), loc: Some("机关城 · 外城广场"),
    mood: "mystery", speaker: Some("墨门断碑遗刻"), voice: Some("vo_jg_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>潜入机关城，破解齿轮阵，直取核心密室。失败代价：被扣 300 点复活。",
        "山口浓雾漫过青苔断碑，碑上一个淋漓的「墨」字。齿轮呜咽声自地底涌来，城门洞黑如兽口——这座消失百年的机关城，正在为一个不该被启动的枢机缓缓苏醒。",
    ]),
    choices: &[
        ChoiceDef { label: "查看断碑", sub: "+5 点 · 铭文残句", cond: None,
            effects: &[Eff::SetFlag("jc_stele_scan"), Eff::Points(5)], route: Route::To("jg_01") },
        ChoiceDef { label: "细听地底声响", sub: "San-2 · 获提示", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("jg_01") },
        ChoiceDef { label: "直闯城门", sub: "可绕隙而行", cond: None,
            effects: &NO_EFF, route: Route::To("jg_01") },
    ],
    fight_id: None, video: Some("vid_jg_opening.mp4"), cine_label: Some("过场 · 机关城苏醒"), overlay: None,
},

/* ---- L1 城门 hub ---- */
SceneDef {
    id: "jg_01", bg: Some("jiguancheng_bg.png"), loc: Some("L1 · 城门与外围"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "外城广场的青石缝里冒出芜荽般的锈绿。你已站在那扇洞开的城门前。",
        "（西侧深巷有藏龛；中轴内城门楼看台高悬；东侧箭楼残件可拓铭文。城门闸 G1 在东北角——无钥只能走东裂隙。）",
    ]),
    choices: &[
        ChoiceDef { label: "深巷藏龛", sub: "取铜钥·墨工", cond: None, effects: &NO_EFF, route: Route::To("jg_02_gear_key") },
        ChoiceDef { label: "墨令残片 · 甲", sub: "L1 调查点", cond: None, effects: &NO_EFF, route: Route::To("jg_02_ling_a") },
        ChoiceDef { label: "门楼铜钟", sub: "调查", cond: None, effects: &NO_EFF, route: Route::To("jg_05_rune_bell") },
        ChoiceDef { label: "先贤绘像", sub: "墨门守城誓言", cond: None, effects: &NO_EFF, route: Route::To("jg_05_ancestor") },
        ChoiceDef { label: "箭楼残件（铭文）", sub: "拓铭文 1/4", cond: None, effects: &NO_EFF, route: Route::To("jg_05_rune_arrow") },
        ChoiceDef { label: "进城门闸", sub: "需铜钥或绕东裂隙 → 滑道坠入工坊", cond: None, effects: &NO_EFF, route: Route::To("jg_02_arrive_workshop") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L1 调查点场景 ---- */
SceneDef {
    id: "jg_02_gear_key", bg: Some("jiguancheng_bg.png"), loc: Some("L1 · 深巷藏龛"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["深巷尽头的一方石龛里，供着一枚黄铜齿轮钥匙——钥匙柄刻着「墨工」。"]),
    choices: &[ChoiceDef { label: "取走铜钥·墨工", sub: "Item it_gear_key · 开 G1", cond: None,
        effects: &[Eff::AddItem("it_gear_key"), Eff::MarkPoint("jg_p_l1_4")], route: Route::To("jg_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_02_ling_a", bg: Some("jiguancheng_bg.png"), loc: Some("L1 · 墨令残片 · 甲"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["城门口的石缝里卡着一枚断裂的墨令——断口新得反常，像才被人掰开。"]),
    choices: &[ChoiceDef { label: "拾取断裂墨令·甲", sub: "Item it_mo_ling_a", cond: None,
        effects: &[Eff::AddItem("it_mo_ling_a"), Eff::MarkPoint("jg_p_l1_1")], route: Route::To("jg_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_05_rune_bell", bg: Some("jiguancheng_bg.png"), loc: Some("L1 · 门楼铜钟"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["门楼看台上悬着一口锈铜钟。钟身受击时会发出极规律的低鸣——像在给什么计数。"]),
    choices: &[ChoiceDef { label: "记下铜钟的节拍", sub: "剧情", cond: None,
        effects: &[Eff::Points(5)], route: Route::To("jg_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_05_ancestor", bg: Some("jiguancheng_bg.png"), loc: Some("L1 · 墨门先贤绘像"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["门楼下绘着一位银发老者的像，题字：「墨守……不活于正……枢机不当启」。"]),
    choices: &[ChoiceDef { label: "默记守城誓言", sub: "剧情", cond: None,
        effects: &[Eff::Points(5)], route: Route::To("jg_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_05_rune_arrow", bg: Some("jiguancheng_bg.png"), loc: Some("L1 · 箭楼残件（铭文 1/4）"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["东侧箭楼残破，弩臂上刻着铭文：「三启以定枢……闲人莫问」。你把铭文拓印下来。"]),
    choices: &[ChoiceDef { label: "拓印铭文", sub: "rune 1/4", cond: None,
        effects: &[Eff::SetFlag("jg_rune_1"), Eff::MarkPoint("jg_p_l1_5")], route: Route::Dyn(jg_rune_arrow_route) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 齿轮工坊 ================= */
SceneDef {
    id: "jg_02_arrive_workshop", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮工坊"),
    mood: "cold", speaker: Some("齿轮阵警报音"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("gear_puzzle_clear") {
            "主齿轮已全部点亮。齿轮阵三启已成，墙上的铭文「枢机不当启」在整座工坊里震颤。".to_string()
        } else {
            "（自城门滑道坠入工坊）天桥与主齿轮之间，三处齿轮机关的机簧低鸣：甲 (6,12)、乙 (22,5)、丙 (23,9)。要把升降梯唤起来，得三处齐发。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "齿轮机关 · 甲", sub: "枢机令甲 · 三启 1/3", cond: None, effects: &NO_EFF, route: Route::To("jg_11_gear_a") },
        ChoiceDef { label: "齿轮机关 · 乙", sub: "枢机令乙 · 三启 2/3", cond: None, effects: &NO_EFF, route: Route::To("jg_11_gear_b") },
        ChoiceDef { label: "齿轮机关 · 丙", sub: "枢机令丙 · 三启 3/3", cond: None, effects: &NO_EFF, route: Route::To("jg_11_gear_c") },
        ChoiceDef { label: "天桥铭牌（铭文 2/4）", sub: "拓铭文 · 提示升降梯需三启", cond: None, effects: &NO_EFF, route: Route::To("jg_05_rune_workshop") },
        ChoiceDef { label: "地沟零件池", sub: "可得气血丹", cond: None, effects: &NO_EFF, route: Route::To("jg_04_parts") },
        ChoiceDef { label: "解读断裂墨令（工坊枢纽）", sub: "需断裂墨令两片", cond: Some(cond_has_mo_ling_a), effects: &NO_EFF, route: Route::To("jg_06_turn") },
        ChoiceDef { label: "上升降梯（需三启）", sub: "G2 → L3 回廊", cond: Some(cond_gear_clear), effects: &NO_EFF, route: Route::To("jg_02_lift") },
        ChoiceDef { label: "走西爬梯（险路）", sub: "无门禁但敌众 · 单向进 L3", cond: None, effects: &NO_EFF, route: Route::To("jg_02_ladder") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_11_gear_a", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮机关 · 甲"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你一扳那枚齿轮机关，主齿轮组轰然点亮一角，机簧里滚落一枚枢机令（甲）。"]),
    choices: &[ChoiceDef { label: "拔动机关·甲", sub: "枢机令·甲 · 三启 1/3", cond: None,
        effects: &[Eff::AddItem("it_gear_token_a"), Eff::SetFlag("gear_sw_a")], route: Route::Dyn(route_gear_pivot) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_11_gear_b", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮机关 · 乙"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["天桥高处的机关乙逆时针转满一圈，滚出一枚枢机令（乙）。"]),
    choices: &[ChoiceDef { label: "拧动机关·乙", sub: "枢机令·乙 · 三启 2/3", cond: None,
        effects: &[Eff::AddItem("it_gear_token_b"), Eff::SetFlag("gear_sw_b")], route: Route::Dyn(route_gear_pivot) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_11_gear_c", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮机关 · 丙"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["主齿轮组下方的机关丙被压下，机簧深处滚出最后一枚枢机令（丙）。"]),
    choices: &[ChoiceDef { label: "压动机关·丙", sub: "枢机令·丙 · 三启 3/3", cond: None,
        effects: &[Eff::AddItem("it_gear_token_c"), Eff::SetFlag("gear_sw_c")], route: Route::Dyn(route_gear_pivot) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_05_rune_workshop", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 天桥铭牌（铭文 2/4）"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["天桥铁牌上刻着铭文：「枢机升降，唯三启是从」。拓印后，你记下升降梯的规矩。"]),
    choices: &[ChoiceDef { label: "拓印铭文", sub: "rune 2/4", cond: None,
        effects: &[Eff::SetFlag("jg_rune_2"), Eff::MarkPoint("jg_p_l2_4")], route: Route::Dyn(jg_rune_workshop_route) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_04_parts", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 地沟零件池"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["工坊底沟的零件坑里泡着锈齿轮与断簧，其间滚着一枚丹药。"]),
    choices: &[ChoiceDef { label: "捞起气血丹", sub: "Item it_qixue_dan", cond: None,
        effects: &[Eff::AddItem("it_qixue_dan")], route: Route::To("jg_02_arrive_workshop") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_02_rune_done", bg: Some("jiguancheng_bg.png"), loc: Some("机关城 · 铭文集拓"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("rune_full") {
            "四枚铭文拼合于一处：<b>「枢机不当启——若启，以令封之。」</b>你获得了对齿轮碾阵的深一层理解。".to_string()
        } else {
            "你收好这枚铭文拓本。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "继续探索", sub: "", cond: None, effects: &NO_EFF, route: Route::To("jg_02_arrive_workshop") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_06_turn", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 工坊枢纽 · 关键转折"),
    mood: "danger", speaker: Some("齿轮阵警报音（铭文）"), voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_mo_ling_b") {
            "两片断裂墨令在黑暗中闪光。墙皮剥落，显影「枢机不当启」——如今你两片都在手，可拼合成完整墨令。".to_string()
        } else {
            "你只有断裂墨令·甲。一枚残令还不足以拼合——另一片据说在 L3 的巨齿轮兽身上。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "解读断裂墨令（合成）", sub: "需两片 → it_mo_ling_full · 解锁 L4 静室", cond: Some(cond_has_mo_ling_a),
            effects: &NO_EFF, route: Route::Dyn(route_mo_ling_synth) },
        ChoiceDef { label: "直接拉动总闸", sub: "San-8 · 触发铁卫增援", cond: None,
            effects: &[Eff::San(-8), Eff::SetFlag("jc_rash")], route: Route::To("jg_06_rash") },
        ChoiceDef { label: "折返调查箭楼残件", sub: "+10 点", cond: None,
            effects: &[Eff::Points(10)], route: Route::To("jg_05_rune_arrow") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_06_rash", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮阵 · 铁卫增援"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["总闸被猛地拉下，整座工坊的机簧纹路乱颤——一群铁卫从墙后的暗格鱼贯而出！"]),
    choices: &[ChoiceDef { label: "迎战铁卫增援", sub: "jc_guard3 增援", cond: None, effects: &NO_EFF, route: Route::To("jg_13_guard3_reinf") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_13_guard3_reinf", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮阵"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["一尊铁卫已经扑到你面前——（战斗）"]),
    choices: &[ChoiceDef { label: "【迎战】", sub: "jc_guard3", cond: None, effects: &NO_EFF, route: Route::To("jg_13_guard3_reinf_fight") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_13_guard3_reinf_fight", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 齿轮阵"),
    mood: "danger", speaker: None, voice: Some("vo_jg_guard3"),
    text: TextSpec::Static(&["那尊铁卫朝你过来了。" ]),
    choices: &[], fight_id: Some("jc_guard3"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_02_lift", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 升降梯（G2 已开）"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["三启已成，升降梯的缆索轰然咬合。你踏上去，铁笼朝上爬去——身后缆索齐齐断裂，铁梯轰然坠入黑暗。"]),
    choices: &[ChoiceDef { label: "（升梯至枢机回廊）", sub: "p_jc_2 单向 · 缆断", cond: None, effects: &NO_EFF, route: Route::To("jg_03_arrive_corridor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_02_ladder", bg: Some("jiguancheng_bg.png"), loc: Some("L2 · 西爬梯（险路）"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["西侧爬梯窄得只容一人侧身。吊锤在头顶隆隆摆动，齿轮兽伏在梯口——这是比升降梯险得多的一条路，而且……进去了就回不来。"]),
    choices: &[ChoiceDef { label: "攀梯而上（单向进 L3）", sub: "p_jc_3 · 敌众+吊锤", cond: None, effects: &NO_EFF, route: Route::To("jg_03_arrive_corridor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 枢机回廊 ================= */
SceneDef {
    id: "jg_03_arrive_corridor", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 枢机回廊"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_pivot_key") {
            "高穹顶的冷光绿锈垂落，中央桥院横跨深渊。枢机桥的铁闸在尽头——如今你握着枢机钥，可以把它放下了。".to_string()
        } else {
            "高穹顶的冷光绿锈垂落，中央桥院横跨深渊。枢机桥闸 G3 在尽头，锁孔是一枚三棱枢机钥——你还没有它。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "调查暗格 · 零件箱（枢机钥）", sub: "取枢机钥", cond: None, effects: &NO_EFF, route: Route::To("jg_13_pivot_key") },
        ChoiceDef { label: "守城人笔记", sub: "剧情伏笔", cond: None, effects: &NO_EFF, route: Route::To("jg_13_keeper_note") },
        ChoiceDef { label: "壁雕 · 墨门机关总图（铭文 3/4）", sub: "拓铭文", cond: None, effects: &NO_EFF, route: Route::To("jg_05_rune_corridor") },
        ChoiceDef { label: "迎战巨齿轮兽", sub: "得断裂墨令·乙", cond: None, effects: &NO_EFF, route: Route::To("jg_13_gearbest") },
        ChoiceDef { label: "上枢机桥（需枢机钥）", sub: "G3 → L4 核心 · 单向", cond: Some(cond_has_pivot_key), effects: &NO_EFF, route: Route::To("jg_02_pivot_bridge") },
        ChoiceDef { label: "回跳秘道滑轮", sub: "p_jc_4 单向回跳 → L2（唯一后悔药）", cond: None, effects: &NO_EFF, route: Route::To("jg_02_arrive_workshop") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_13_pivot_key", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 暗格 · 零件箱"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["桥门楼墙体内的暗格，藏着一枚三棱枢机钥。钥柄温润，是活人久握过的。"]),
    choices: &[ChoiceDef { label: "取走枢机钥", sub: "Item it_pivot_key · 开 G3", cond: None,
        effects: &[Eff::AddItem("it_pivot_key"), Eff::MarkPoint("jg_p_l3_1")], route: Route::To("jg_03_arrive_corridor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_13_keeper_note", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 守城人笔记"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["遗骸旁压着一卷朽烂的帛书，字迹用力到纸背凸起：「封匣之人，即是我等。墨者不死，城何以灭……三百年前我等以命封匣，今日以血继之。」"]),
    choices: &[ChoiceDef { label: "读完遗书", sub: "伏笔 · 守城人自述", cond: None,
        effects: &[Eff::MarkPoint("jg_p_l3_3")], route: Route::To("jg_03_arrive_corridor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_05_rune_corridor", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 壁雕 · 墨门机关总图（铭文 3/4）"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["整面墙的机关总图里，枢机大殿的位置被一枚三齿轮记号框出。你在角落拓到第 3 枚铭文。"]),
    choices: &[ChoiceDef { label: "拓印铭文", sub: "rune 3/4", cond: None,
        effects: &[Eff::SetFlag("jg_rune_3"), Eff::MarkPoint("jg_p_l3_2")], route: Route::Dyn(jg_rune_corridor_route) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_13_gearbest", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 中央桥下"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jg_gearbest2"),
    text: TextSpec::Static(&["你纵身跃下中央桥——巨齿轮兽伏在桥下阴影里。（战斗）"]),
    choices: &[], fight_id: Some("jc_gearbest2"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_13_gearbest_win", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 中央桥下"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["巨齿轮兽瘫伏不动，腹部的断裂墨令从齿缝里滑落——这是断裂墨令·乙。"]),
    choices: &[ChoiceDef { label: "拾取断裂墨令·乙", sub: "Item it_mo_ling_b", cond: None,
        effects: &[Eff::AddItem("it_mo_ling_b")], route: Route::To("jg_03_arrive_corridor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_02_pivot_bridge", bg: Some("jiguancheng_bg.png"), loc: Some("L3 · 枢机桥（G3 已开）"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["枢机钥旋入锁孔，「嗡——」铁闸升起，枢机桥在你面前展开。你踏上最后一道单向门——身后轰然合拢，再无回头的路。"]),
    choices: &[ChoiceDef { label: "（过枢机桥 → 核心密室）", sub: "p_jc_5 单向 · 决战前最后单向门", cond: None, effects: &NO_EFF, route: Route::To("jg_21_arrive_core") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L4 核心密室 ================= */
SceneDef {
    id: "jg_21_arrive_core", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 核心密室"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "墨玉核心幽蓝的光在整座大殿里漫流，暗金脉纹沿壁走向穹顶。远处，五米高的青铜剪影正一缕缕凝实——枢机巨像。",
        "（中轴枢机大殿是巨像的擂台；东侧静室暗门需解读墨令可进；密匣石台高踞中殿之侧。）",
    ]),
    choices: &[
        ChoiceDef { label: "迎战枢机巨像", sub: "BOSS 决战", cond: None, effects: &NO_EFF, route: Route::To("jg_22_colossus") },
        ChoiceDef { label: "枢机核心壁雕（铭文 4/4）", sub: "拓铭文 · 剧情", cond: None, effects: &NO_EFF, route: Route::To("jg_05_rune_core") },
        ChoiceDef { label: "探查静室暗门（隐藏）", sub: "需解读墨令", cond: Some(cond_mo_ling_broken), effects: &NO_EFF, route: Route::To("jg_25_keeper") },
        ChoiceDef { label: "静室遗物", sub: "L4 调查点", cond: None, effects: &NO_EFF, route: Route::To("jg_24_keeper_rel_ac") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_05_rune_core", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 枢机核心壁雕（铭文 4/4）"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["枢机大殿的壁雕刻着一枚巨大的墨玉核心，其下铭文：「齿轮为经、机关为纬——枢机若启，勿让匣物见天日」。你拓下最后一枚铭文。"]),
    choices: &[ChoiceDef { label: "拓印铭文", sub: "rune 4/4", cond: None,
        effects: &[Eff::SetFlag("jg_rune_4"), Eff::MarkPoint("jg_p_l4_2")], route: Route::Dyn(jg_rune_core_route) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_24_keeper_rel_ac", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 静室遗物"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["静室门槛外散落着一叠守城人的遗物——一封未寄出的信：「封匣之人，即是我等。勿让后人再启。」你拾起封缄的遗书。"]),
    choices: &[ChoiceDef { label: "收下守城人遗书", sub: "Item it_shoucheng_letter", cond: None,
        effects: &[Eff::AddItem("it_shoucheng_letter"), Eff::MarkPoint("jg_p_l4_3")], route: Route::To("jg_21_arrive_core") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- BOSS · 枢机巨像（选择驱动）---- */
SceneDef {
    id: "jg_22_colossus", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 枢机大殿 · 决战"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_jg_colossus"),
    text: TextSpec::Static(&[
        "你踏进枢机大殿那一刻，五米的青铜巨像轰然睁目。三层齿轮颅缓缓展开，胸口墨玉核心直射暗金光芒。",
        "「齿轮为经、机关为纬——你敢启这枢机？」",
    ]),
    choices: &[ChoiceDef { label: "【逼近巨像】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_colossus) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_colossus_round", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 枢机大殿 · 激战"),
    mood: "danger", speaker: Some("枢机巨像"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("巨像 HP {} / {}", f.hp.max(0), 160)).unwrap_or_else(|| "巨像 HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——狂暴齿轮碾阵！每三回合它喘着碾压全场（Hurt 8，轻功可减半）——"
        } else { "" };
        format!("{f}。{}", mode)
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| colossus_act(st, rng(30, 42), false, false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-28", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| colossus_act(st, rng(20, 28), false, false)) },
        ChoiceDef { label: "【以令夺枢】", sub: "需三枚枢机令 + 狂暴 · 40 固伤并解狂暴", cond: Some(cond_three_tokens_raged),
            effects: &NO_EFF, route: Route::Dyn(|st| colossus_act(st, 40, true, false)) },
        ChoiceDef { label: "后撤观察", sub: "提升闪避", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| colossus_act(st, 0, false, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_23_colossus_down", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 枢机大殿"),
    mood: "calm", speaker: Some("守城人残响"), voice: Some("vo_jg_colossus_down"),
    text: TextSpec::Dyn(|st| {
        let mode = match (st.flag("jc_box_open"), st.flag("jc_box_seal"), st.flag("jc_box_destroy")) {
            (true, _, _) => "你开启了密匣，那星图刻在匣底。",
            (_, true, _) => "你以墨令封匣——「以令封之，愿此惑百年不再启。」",
            (_, _, true) => "你毁去密匣，齿轮风暴短暂发作后归于寂静。",
            _ => "巨像跪伏，枢机石台自大殿升起，那口沉睡的密匣在嗡鸣中缓缓浮出。上古武学图谱残页的墨意在半空舒展。",
        };
        format!("巨像轰然跪伏，机械核心熄成暗灰……\n\n石台升起，密匣近在咫尺。\n\n{mode}")
    }),
    choices: &[ChoiceDef { label: "（走向密匣石台）", sub: "密匣之择", cond: None, effects: &NO_EFF, route: Route::To("jg_30_box") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结局三分支 · 密匣之择 ================= */
SceneDef {
    id: "jg_30_box", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 密匣石台"),
    mood: "mystery", speaker: Some("守城人残响"), voice: Some("vo_jg_box"),
    text: TextSpec::Static(&[
        "石台上的跨界密匣贴满墨门封条，匣底的星图隔着暗香微微透光。守城人枯哑的声音自深处传来，藏着一丝机关音：",
        "「你若为图谱而来……匣中所藏，是你担不起的东西。」",
    ]),
    choices: &[
        ChoiceDef { label: "开启密匣", sub: "+150 点 · Item 跨界密匣(开) · 星图彩蛋", cond: None,
            effects: &[Eff::SetFlag("jc_box_open"), Eff::AddItem("it_cross_box_open"), Eff::Points(150),
                Eff::SetFlag("jc_box_choice")], route: Route::To("jg_31_exit") },
        ChoiceDef { label: "以墨令封匣", sub: "+100 点 · Item 跨界密匣(封) · 典雅结局", cond: None,
            effects: &[Eff::SetFlag("jc_box_seal"), Eff::AddItem("it_cross_box_sealed"), Eff::Points(100),
                Eff::SetFlag("jc_box_choice")], route: Route::To("jg_31_exit") },
        ChoiceDef { label: "毁匣", sub: "+150 点 · 点数 · 防祸再启", cond: None,
            effects: &[Eff::SetFlag("jc_box_destroy"), Eff::Points(150), Eff::SetFlag("jc_box_choice")], route: Route::To("jg_31_exit") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_31_exit", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 撤离阵"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        match (st.flag("jc_box_open"), st.flag("jc_box_seal"), st.flag("jc_box_destroy")) {
            (true, _, _) => "匣内星图与主神光柱同源——「汝等所见之城，只是门缝」。你顺势踏入撤离阵，光柱吞没了你。"
                .to_string(),
            (_, true, _) => "墨令封匣，空间纹路缓缓闭合——「愿此惑岁月不再启」。你踏入撤离阵，光柱温和地笼住你。"
                .to_string(),
            _ => "齿轮风暴刮过又归于寂静，守护者英灵的字幕在你眼前浮现。你踏入撤离阵，光柱吞没了你。".to_string(),
        }
    }),
    choices: &[ChoiceDef { label: "（踏入撤离阵 · 结算）", sub: "sp_grade 结算 · 回主神空间", cond: None,
        effects: &NO_EFF, route: Route::Dyn(route_exit_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_32_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_jg_settle"), death: None,
        card: |st| crate::state::Card {
            title: "机 关 城 · 破 枢".into(), good: true,
            body_html: format!(
                "<p>巨像跪伏，枢机石台沉入那束光里。你带着破枢的余烬踏出撤离阵。</p>\
                 <p style='color:#9a958a'>破枢线索：三启已全 / 铭文 {} / 密匣之择已作。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 <tr><td>主神光柱</td><td>「星图之外，另有星图。」</td></tr>\
                 </table>",
                if st.flag("rune_full") { "4/4" } else { "未完" }, st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 隐藏 BOSS · 入魔的守城人 ================= */
SceneDef {
    id: "jg_25_keeper", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 静室"),
    mood: "danger", speaker: Some("入魔的守城人"), voice: Some("vo_jg_keeper"),
    text: TextSpec::Static(&[
        "石门无声滑开，一股朽木与烛火的潮气扑来。枯坐的老人缓缓抬头，双目赤红，铜链自朽木中生出——「你……来晚了三百年。」",
    ]),
    choices: &[
        ChoiceDef { label: "出示完整墨令", sub: "免战 · +400 · 解脱", cond: Some(cond_has_mo_ling_full),
            effects: &NO_EFF, route: Route::Dyn(keeper_freed) },
        ChoiceDef { label: "拔剑相向", sub: "隐藏战 jc_keeper", cond: None, effects: &NO_EFF, route: Route::Dyn(start_keeper) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_keeper_round", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 静室 · 激战"),
    mood: "danger", speaker: Some("入魔的守城人"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("守城人 HP {} / {}", f.hp.max(0), 140)).unwrap_or_else(|| "HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——执念显化，铜链化鞭；狂暴后可出示完整墨令免战——" } else { "" };
        format!("{f}。{}", mode)
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| keeper_act(st, rng(30, 42), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-28", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| keeper_act(st, rng(20, 28), false)) },
        ChoiceDef { label: "【出示墨令】", sub: "狂暴后可免战 · 解脱", cond: Some(cond_keeper_finish),
            effects: &NO_EFF, route: Route::Dyn(keeper_freed) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "jg_26_keeper_freed", bg: Some("jiguancheng_bg.png"), loc: Some("L4 · 静室"),
    mood: "calm", speaker: Some("守城人"), voice: Some("vo_jg_keeper_free"),
    text: TextSpec::Static(&["铜链自他腕上寸寸滑落。守城人怔怔望着你手里的墨令，赤红如潮水褪去：「封匣之人，即是我等……你今日以墨令，封了那人犯下的罪。」他化作一片光尘，石室归于寂静。"]),
    choices: &[ChoiceDef { label: "（转身离开静室）", sub: "keeper_freed · 回核心大殿", cond: None, effects: &NO_EFF, route: Route::To("jg_21_arrive_core") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 死亡档案（复活扣 300/回主神）================= */
SceneDef {
    id: "jg_40_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("机关下的无名者", "在机关城的齿轮与傀儡下倒下")), card: |_st| crate::state::Card {
            title: "机 关 之 下".into(), good: false,
            body_html: r#"<p>机械的血肉在你耳边屈辱地倒下，机关城的低鸣裹住你冷却的躯壳。</p>
<p style='color:#ff8a8a'>【死亡档案 · 机关下的无名者】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "jg_40_death_colossus", bg: None, loc: None, mood: "danger", speaker: None, voice: Some("vo_jg_death_colossus"),
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("枢机之下", "巨像的机关足踏碎了你的残影")), card: |_st| crate::state::Card {
            title: "枢 机 之 下".into(), good: false,
            body_html: r#"<p>五米巨像的机关足踏下——你的残影被碾碎在最后一道齿轮阵里。机械轰鸣盖过所有的呼喊。</p>
<p style='color:#ff8a8a'>【死亡档案 · 枢机之下】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "jg_41_death_keeper", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("静室遗言", "入魔的守城人铜链缠颈，你未解其执念")), card: |_st| crate::state::Card {
            title: "静 室 遗 言".into(), good: false,
            body_html: r#"<p>铜链从朽木里射出，锁住你的咽喉。「你……还是来晚了。」守城人的最后一声，混着齿轮停转的嘶哑。</p>
<p style='color:#ff8a8a'>【死亡档案 · 静室遗言】</p>
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
fn route_gear_pivot(st: &mut GameState) -> String {
    if st.flag("gear_sw_a") && st.flag("gear_sw_b") && st.flag("gear_sw_c") {
        st.set_flag("gear_puzzle_clear");
        st.set_flag("jg_pivot_gate");
    }
    "jg_02_arrive_workshop".to_string()
}

/// 断裂墨令合成：持 it_mo_ling_a + it_mo_ling_b → it_mo_ling_full + mo_ling_broken
fn route_mo_ling_synth(st: &mut GameState) -> String {
    if inv(st, "it_mo_ling_a") && inv(st, "it_mo_ling_b") {
        crate::world::add_item(st, "it_mo_ling_full");
        st.set_flag("mo_ling_broken");
    }
    "jg_02_arrive_workshop".to_string()
}

/// 撤离结算：落日终局 flag + 评级（D 级已由巨像写;此处确保）→ 卡片
fn route_exit_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "jg_32_card".to_string()
}

/// 铭文点调查后调用：which 为该点编号，全部 4 点拓印 → rune_full
fn route_rune(st: &mut GameState, which: &str) -> String {
    st.set_flag(which);
    if st.flag("jg_rune_1") && st.flag("jg_rune_2") && st.flag("jg_rune_3") && st.flag("jg_rune_4") {
        st.set_flag("rune_full");
    }
    "jg_02_rune_done".to_string()
}

// 铭文落定路由（Rust 只允许一次定义,用单独 fn 避免重复）
fn jg_rune_arrow_route(st: &mut GameState) -> String { route_rune(st, "jg_rune_1") }
fn jg_rune_workshop_route(st: &mut GameState) -> String { route_rune(st, "jg_rune_2") }
fn jg_rune_corridor_route(st: &mut GameState) -> String { route_rune(st, "jg_rune_3") }
fn jg_rune_core_route(st: &mut GameState) -> String { route_rune(st, "jg_rune_4") }
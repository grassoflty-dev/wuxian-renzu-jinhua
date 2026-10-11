//! 《无限恐怖 · 木乃伊（哈姆纳塔地宫）》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.1 第 40 行 + wuxian_kongbu/00_..._research.md §4.5。
//! 本文件是全新新增文件，只导出静态数据（MUMIYI_SCENES / mumiyi_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/mumiyi_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `mm_` 前缀，与既有 SCENES 无重名；fight id 也全部 `mm_` 前缀。
//! 剧情线：考古营地 →(对照碑文/开棺)→ 诅咒苏醒 → 地宫(机关门→圣甲虫厅→祭司墓室)
//!        → 祭司战(伊莫顿二阶段) → 封棺/逃离结局。
//! BOSS 大祭司·伊莫顿 = HP210 复生二阶段：「选择驱动的遭遇链」（参考 scenes_zhouyuan.rs zy_boss_round）：
//! 因需要 二段复活转场 / 弱水终结技前置 / 圣甲虫潮增员 等自定义每回合同调，用 Normal 场景 + Route::Dyn
//! 落地；同时导出 `mm_imhotep`/`mm_imhotep2` FightCfg 供 ZoneDef 与揭示用。
//! 弱水终结技前置：mm_12_well 取「尼罗弱水」(it_mumi_water)，则二段可选「以弱水终结」一化合杀。
//! 圣甲虫潮增员：BOSS 一段狂暴(on_rage) 额外召唤虫潮叙事 + 二段每回结束 San 胁迫。
//! sp_grade = C（玩家击杀伊莫顿后写 Some('C')）。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   F0 入口   → mm_bg_entrance（现用 img_zhuyuan_book.png 占位）
//!   F1 圣甲虫厅→ mm_bg_scarab  （现用 img_laser.png 占位）
//!   F2 祭司墓室→ mm_bg_tomb    （现用 img_redqueen.png 占位）
//! 敌人立绘复用：horde→圣甲虫潮、zombie→木乃伊战士；BOSS 新美术由主 agent 统一生图替换。

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

/// 当前 BOSS 血量（无则视作已结束）
fn boss_hp(st: &GameState) -> i32 {
    st.fight.as_ref().map(|f| f.hp).unwrap_or(0)
}

/// 玩家受击；归零跳败北场景
fn hurt_player(st: &mut GameState, lo: i32, hi: i32) -> String {
    let d = rng(lo, hi);
    st.hp = (st.hp - d).max(0);
    if st.hp <= 0 { return "mm_29_lose".to_string(); }
    String::new()
}

/// 依据 id 从本文件战斗表取 FightCfg（供选择驱动相遇用之，自包含，不依赖主神合并）
fn find_fight(id: &str) -> Option<&'static FightCfg> {
    mumiyi_figths().iter().find(|(k, _)| *k == id).map(|(_, c)| c)
}

/// 从 FightCfg 初始化一场 BOSS Fight
fn start_fight(st: &mut GameState, id: &str) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = find_fight(id) {
            st.fight = Some(crate::power::scaled_fight(id, cfg, st, vec![]));
        }
    }
    String::new()
}

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_key(st: &GameState) -> bool { inv(st, "it_mumi_key") }
fn cond_has_scarab_sac(st: &GameState) -> bool { inv(st, "it_mumi_scarab_sac") }
fn cond_has_water(st: &GameState) -> bool { inv(st, "it_mumi_water") }

/* =====================================================================
   动态文本辅助
   ===================================================================== */
fn txt_boss1(st: &GameState) -> String {
    let hp = boss_hp(st);
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let head = if raged {
        "伊莫顿周身黄沙狂卷，木乃伊绷带寸寸爆裂——他扬手，万千圣甲虫自壁画涌出，在场者尽为虫雨所困。\n\n<b style='color:#ff6a6a'>圣甲虫潮增员：每回合结束 San -5，虫啮使你应接不暇。</b>"
    } else {
        "石棺上刻满诅咒经文。伊莫顿睁开空洞的眼眶，一握黄沙凝成人形——大祭司复活了。"
    };
    format!("{head}\n\n<b>大祭司·伊莫顿</b> · 本体　HP {hp}/210\n\n（你听见拉丁咒语在低语，地板下的虫群蠢蠢欲动……）")
}

fn txt_boss2(st: &GameState) -> String {
    let hp = boss_hp(st);
    let water = if inv(st, "it_mumi_water") {
        "\n\n你腰间的水囊沉甸甸——那是取自弱水井的<b>尼罗弱水</b>。伊莫顿的弱点是水。".to_string()
    } else { String::new() };
    format!("复生的伊莫顿形销骨立，黄沙也难以续命。\n\n<b>大祭司·伊莫顿 · 复生</b>　HP {hp}/120{water}")
}

/* =====================================================================
   伊莫顿 · 选择驱动遭遇
   ===================================================================== */

/// 一段开始：初始化 BOSS(HP210) → mm_23_imhotep1
fn route_start_imhotep1(st: &mut GameState) -> String {
    start_fight(st, "mm_imhotep");
    st.set_flag("mm_imhotep_fight");
    "mm_23_imhotep1".to_string()
}

/// 一段攻击回合
fn route_imhotep1_atk(st: &mut GameState) -> String {
    let dmg = rng(14, 24);
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    // 狂暴：圣甲虫潮增员（San 胁迫叙事，引擎无助调，用场景数据落地）
    let max = st.fight.as_ref().map(|f| f.max_hp).unwrap_or(210);
    let hp = boss_hp(st);
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    if !raged && (max - hp) >= (max * 40 / 100 + max * 4 / 100) {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
        st.san = (st.san - 5).clamp(0, 100);
    } else if raged {
        st.san = (st.san - 5).clamp(0, 100); // 狂暴期间每回 San-5
    }
    // 玩家反击
    let dr = hurt_player(st, 10, 18);
    if !dr.is_empty() { return dr; }
    // 一段击倒 → 复生二段
    if boss_hp(st) <= 0 {
        st.set_flag("mm_imhotep1_down");
        return "mm_24_reborn".to_string();
    }
    "mm_23_imhotep1".to_string()
}

/// 一段·弱水祭炼（前置：取过弱水；此回强但一段必复生，故不终结）
fn route_imhotep1_water(st: &mut GameState) -> String {
    if inv(st, "it_mumi_water") {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - 60).max(0); }
        st.set_flag("mm_water_prep");
    } else {
        // 无水解手：以水囊空浇，聊胜于无
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - 10).max(0); }
    }
    let dr = hurt_player(st, 8, 14);
    if !dr.is_empty() { return dr; }
    if boss_hp(st) <= 0 {
        st.set_flag("mm_imhotep1_down");
        return "mm_24_reborn".to_string();
    }
    "mm_23_imhotep1".to_string()
}

/// 考古队合力：阿尔德与乔伊举火把助阵，额外一轮输出但耗费体力更大
fn route_imhotep1_help(st: &mut GameState) -> String {
    let dmg = rng(10, 18);
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    let dr = hurt_player(st, 14, 24);
    if !dr.is_empty() { return dr; }
    if boss_hp(st) <= 0 {
        st.set_flag("mm_imhotep1_down");
        return "mm_24_reborn".to_string();
    }
    "mm_23_imhotep1".to_string()
}

/// 二段开始：重置复生体(HP120) → mm_25_imhotep2
fn route_start_imhotep2(st: &mut GameState) -> String {
    if let Some(cfg) = crate::scenes::fight_cfg("mm_imhotep2") {
        st.fight = Some(crate::power::scaled_fight("mm_imhotep2", cfg, st, vec![]));
    }
    "mm_25_imhotep2".to_string()
}

/// 二段攻击回合
fn route_imhotep2_atk(st: &mut GameState) -> String {
    let dmg = rng(12, 20);
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    st.san = (st.san - 4).clamp(0, 100); // 复生体诅咒胁迫
    let dr = hurt_player(st, 12, 22);
    if !dr.is_empty() { return dr; }
    if boss_hp(st) <= 0 {
        st.sp_grade = Some('C');
        st.set_flag("mm_end_seal");
        st.points += 800;
        return "mm_27_win".to_string();
    }
    "mm_25_imhotep2".to_string()
}

/// 二段·弱水终结（前置：尼罗弱水；一合将生命固定归零）
fn route_imhotep2_water(st: &mut GameState) -> String {
    if inv(st, "it_mumi_water") {
        if let Some(f) = st.fight.as_mut() { f.hp = 0; }
        st.set_flag("mm_water_finish");
        st.set_flag("mm_end_seal");
        st.points += 800;
        st.sp_grade = Some('C');
        return "mm_27_win".to_string();
    }
    // 无法师水：退回普通攻击
    route_imhotep2_atk(st)
}

/* =====================================================================
   圣甲虫潮 / 木乃伊 · 狂暴回调
   ===================================================================== */
fn mm_rage_scarab(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>虫潮自墙缝涌出，铺天盖地——圣甲虫爬过的地方，都是潮。</span>".into());
}
fn mm_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   胜利/败北回调（导出 FightCfg 与场景共用）
   ===================================================================== */
fn mm_win_reborn(_st: &GameState) -> String { "mm_24_reborn".into() }
fn mm_win_seal(_st: &GameState) -> String { "mm_27_win".into() }
/// 弱水终结前置判定：持有尼罗弱水
fn mm_finisher_water(st: &GameState, _enemy_hp: i32) -> bool { inv(st, "it_mumi_water") }
fn mm_finisher_none(_st: &GameState, _e: i32) -> bool { false }
fn mm_finisher_none_name(_st: &GameState) -> String { String::new() }
fn mm_finisher_none_desc(_st: &GameState) -> String { String::new() }
fn mm_finisher_water_name(_st: &GameState) -> String { "以尼罗弱水终结".to_string() }
fn mm_finisher_water_desc(st: &GameState) -> String {
    if inv(st, "it_mumi_water") {
        "一囊尼罗弱水浇上复生的绷带之躯，咒文寸寸溃散。".to_string()
    } else { String::new() }
}

/* =====================================================================
   战斗配置表（id 全部 mm_ 前缀）
   ===================================================================== */
pub fn mumiyi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("mm_scarab_swarm_light", FightCfg {
            name: "圣甲虫潮·散兵", hp: 26, dmg: (6, 12), reward: 40, reward_why: "圣甲虫潮 · 清剿",
            intro: "壁龛里的圣甲虫蠢蠢爬出，黑压压一片向你的脚踝涌来。",
            rage_at: None, rage_text: "", on_rage: mm_rage_none,
            finisher_if: mm_finisher_none, finisher_name: mm_finisher_none_name, finisher_desc: mm_finisher_none_desc,
            win: mm_win_seal, death: "mm_29_lose",
        }),
        ("mm_scarab_swarm", FightCfg {
            name: "圣甲虫潮", hp: 40, dmg: (8, 16), reward: 70, reward_why: "圣甲虫潮 · 镇灭",
            intro: "整面虫穴都活着——圣甲虫如潮水般涌出，爬过的地方都是潮。",
            rage_at: Some(16), rage_text: "虫潮愈演愈烈，开始啃噬墙壁与烛台。",
            on_rage: mm_rage_scarab,
            finisher_if: mm_finisher_none, finisher_name: mm_finisher_none_name, finisher_desc: mm_finisher_none_desc,
            win: mm_win_seal, death: "mm_29_lose",
        }),
        ("mm_mummy_guard", FightCfg {
            name: "木乃伊战士", hp: 48, dmg: (8, 15), reward: 60, reward_why: "木乃伊战士 · 击碎",
            intro: "缠着脏绷带的守墓木乃伊提刀立起，眼眶深处有两点幽火。",
            rage_at: None, rage_text: "", on_rage: mm_rage_none,
            finisher_if: mm_finisher_none, finisher_name: mm_finisher_none_name, finisher_desc: mm_finisher_none_desc,
            win: mm_win_seal, death: "mm_29_lose",
        }),
        ("mm_mummy_sentinel", FightCfg {
            name: "虫涌木乃伊", hp: 60, dmg: (11, 19), reward: 90, reward_why: "虫涌木乃伊 · 焚毁",
            intro: "被虫群占据的哨卫木乃伊，关节喀喀作响，朝你掷出裹火的短矛。",
            rage_at: None, rage_text: "", on_rage: mm_rage_none,
            finisher_if: mm_finisher_none, finisher_name: mm_finisher_none_name, finisher_desc: mm_finisher_none_desc,
            win: mm_win_seal, death: "mm_29_lose",
        }),
        ("mm_imhotep", FightCfg {
            name: "大祭司·伊莫顿", hp: 210, dmg: (14, 26), reward: 700, reward_why: "祭司战 · 一段",
            intro: "石棺崩裂，黄沙凝成人形——大祭司伊莫顿自诅咒中复活。",
            rage_at: Some(105), rage_text: "伊莫顿狂怒，圣甲虫潮增员（每回 San-5）。",
            on_rage: mm_rage_scarab,
            finisher_if: mm_finisher_water, finisher_name: mm_finisher_water_name, finisher_desc: mm_finisher_water_desc,
            win: mm_win_reborn, death: "mm_29_lose",
        }),
        ("mm_imhotep2", FightCfg {
            name: "大祭司·伊莫顿·复生", hp: 120, dmg: (18, 30), reward: 600, reward_why: "祭司战 · 二段封印",
            intro: "他一把握住散去的黄沙，复生为形体——没有水的克制，他将不死不灭。",
            rage_at: Some(60), rage_text: "复生之体疯狂汲取沙土，虫群蔽日。",
            on_rage: mm_rage_scarab,
            finisher_if: mm_finisher_water, finisher_name: mm_finisher_water_name, finisher_desc: mm_finisher_water_desc,
            win: mm_win_seal, death: "mm_29_lose",
        }),
    ]
}

/// 取本文件战斗表
pub fn mumiyi_fight(id: &str) -> Option<&'static FightCfg> {
    mumiyi_figths().iter().find(|(k, _)| *k == id).map(|(_, c)| c)
}

/* =====================================================================
   剧情场景表
   ===================================================================== */
pub static MUMIYI_SCENES: &[SceneDef] = &[
    // ================= 考古营地 · 主神交付 =================
    SceneDef {
        id: "mm_00_camp", bg: Some("mumiyi_bg.png"),
        loc: Some("哈姆纳塔 · 考古营地"), mood: "黄沙夜色，篝火明灭",
        speaker: Some("主神提示"), voice: None,
        text: TextSpec::Static(&[
            "主神冰冷的声音在你脑中响起：",
            "「任务世界：木乃伊 · 哈姆纳塔地宫。目标——深挖地宫，取走宝藏，并在祭司完全苏醒前了结它。」",
            "你环顾营地，考古队长阿尔德领着一支埃及考古队，刚刚凿穿了一道刻满鸟形文字的石门。",
        ]),
        choices: &[
            ChoiceDef { label: "走向被凿穿的石门", sub: "由考古队先行，你跟在其后", cond: None, effects: &NO_EFF, route: Route::To("mm_01_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ================= F0 地宫入口 =================
    SceneDef {
        id: "mm_01_arrive", bg: Some("mumiyi_bg_battle.png"),
        loc: Some("地宫入口"), mood: "昏暗地道，风沙呜咽",
        speaker: Some("外景"), voice: None,
        text: TextSpec::Static(&[
            "石门之后是一条下行的地底回廊，火把把你的影子拉得很长。",
            "左侧立着一块无名祭碑，右侧墙上是满壁圣甲虫壁画；更深处，一座贴满金箔的墓门若隐若现。",
        ]),
        choices: &[
            ChoiceDef { label: "检视无名祭碑", sub: "解读上面的鸟形铭文", cond: None, effects: &NO_EFF, route: Route::To("mm_03_stele") },
            ChoiceDef { label: "查看壁画虫纹", sub: "圣甲虫图腾与咒语", cond: None, effects: &NO_EFF, route: Route::To("mm_04_scarab") },
            ChoiceDef { label: "与考古队长谈", sub: "眼下这支队伍的来历", cond: None, effects: &NO_EFF, route: Route::To("mm_02_npc") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_02_npc", bg: Some("mumiyi_bg_open.png"),
        loc: Some("地宫入口 · 火把下"), mood: "考古队的低语",
        speaker: Some("考古队长·阿尔德"), voice: None,
        text: TextSpec::Static(&[
            "阿尔德用手电扫过你：「你是雇佣兵？这场考古可不是盗墓——我们是被诅咒缠上了。」",
            "他压低声音：「那个开棺的人被自己的护符杀死，死前喊——『它醒了』。」",
            "「这支队伍只管进，不管出。你要的宝藏，都在那扇贴金箔的墓门后面。」",
        ]),
        choices: &[
            ChoiceDef { label: "转身走向祭碑", sub: "先去读那块无名碑", cond: None, effects: &NO_EFF, route: Route::To("mm_03_stele") },
            ChoiceDef { label: "走向金色墓门", sub: "准备突破机关", cond: Some(cond_has_key), effects: &NO_EFF, route: Route::To("mm_07_gate_scarab") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_02_npc_joy", bg: Some("mumiyi_bg_open.png"),
        loc: Some("地宫入口 · 南侧偏室"), mood: "学徒的虔诚",
        speaker: Some("见习考古员·乔伊"), voice: None,
        text: TextSpec::Static(&[
            "乔伊抱着一只陶罐，怯生生地凑过来：「队长说这里埋着活人的亡魂，我不信……但它真的在呼吸。」",
            "「你要找宝藏，就去南边那只看守石匣——据说藏着前一批考古队留下的黄金。」",
        ]),
        choices: &[
            ChoiceDef { label: "打开石匣·陶罐", sub: "看看前人留下的宝藏", cond: None, effects: &NO_EFF, route: Route::To("mm_06_box_a") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_03_stele", bg: Some("mumiyi_bg_open.png"),
        loc: Some("地宫入口 · 无名祭碑"), mood: "晦涩经文",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "祭碑上是古代祭司文：",
            "「圣甲虫司命物天、物地、物水火。欲入哈姆纳塔深处，先以钥启甲，再以匣封门。」",
            "碑座下露出一枚刻着甲虫图样的青铜钥匙——正是圣甲虫机关门的钥。",
        ]),
        choices: &[
            ChoiceDef { label: "取下青铜甲虫钥", sub: "钥匙入手", cond: None, effects: &[Eff::AddItem("it_mumi_key")], route: Route::To("mm_01_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_04_scarab", bg: Some("mumiyi_bg_open.png"),
        loc: Some("地宫入口 · 壁画虫纹"), mood: "墙会呼吸",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "满墙圣甲虫壁画在你注视时竟齐齐转头。",
            "拉丁文咒语在耳畔低语：你若在此久留，虫就会从墙里爬出来。",
        ]),
        choices: &[
            ChoiceDef { label: "退后，不去惊动虫", sub: "壁画图腾不可轻犯", cond: None, effects: &NO_EFF, route: Route::To("mm_01_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_05_altar", bg: Some("mumiyi_bg_open.png"),
        loc: Some("地宫入口 · 前室祭坛"), mood: "干涸的血",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "前室祭坛上摆着一只被劈裂的木乃伊护符盒——这正是那个被护符杀死的研究员打开的东西。",
            "盒底压着一张羊皮纸，画着圣甲虫厅与祭司墓室的示意图。",
        ]),
        choices: &[
            ChoiceDef { label: "记下地宫结构", sub: "圣甲虫厅→祭司墓室的路线了然于胸", cond: None, effects: &[Eff::SetFlag("mm_map_known")], route: Route::To("mm_01_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_06_box_a", bg: Some("mumiyi_bg_open.png"),
        loc: Some("地宫入口 · 石匣·陶罐"), mood: "前人遗金",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "石匣覆着陶土，撬开后滚出金灿灿的东西——前一批考古队私藏的黄金器皿。",
            "你在夹层里还摸到一枚圣甲虫护符与一把火器弹药。",
        ]),
        choices: &[
            ChoiceDef { label: "取走宝藏", sub: "黄金器皿 + 圣甲虫护符 + 弹药", cond: None,
                effects: &[Eff::AddItem("it_mumi_loot_gold"), Eff::AddItem("it_mumi_amulet"), Eff::Points(120)], route: Route::To("mm_01_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_07_gate_scarab", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("圣甲虫机关门"), mood: "机关嗡鸣",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "石门中央的甲虫凹槽正等着钥匙。你手头的青铜甲虫钥恰好吻合。",
        ]),
        choices: &[
            ChoiceDef { label: "嵌入钥匙，开机关门", sub: "石门后是圣甲虫厅的石梯", cond: Some(cond_has_key),
                effects: &[Eff::SetFlag("mm_gate_scarab_open")], route: Route::To("mm_10_arrive_f1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ================= F1 圣甲虫厅 =================
    SceneDef {
        id: "mm_10_arrive_f1", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("圣甲虫厅"), mood: "虫影重重",
        speaker: Some("外景"), voice: None,
        text: TextSpec::Static(&[
            "石梯尽头豁然开朗——圣甲虫厅！四壁刻满爬行的甲虫，地面铺着一层会动的砂。",
            "厅中央是一座被虫包裹的巨大石棺；北侧一口泛着幽蓝光的弱水井；南侧一道同样刻满虫玺的封印墓门。",
        ]),
        choices: &[
            ChoiceDef { label: "走向弱水井", sub: "那口泛幽蓝光的井", cond: None, effects: &NO_EFF, route: Route::To("mm_12_well") },
            ChoiceDef { label: "查探虫巢神像", sub: "厅中央被虫包住的石棺神像", cond: None, effects: &NO_EFF, route: Route::To("mm_13_hollow") },
            ChoiceDef { label: "走向南侧封印墓门", sub: "通往祭司墓室", cond: Some(cond_has_scarab_sac), effects: &NO_EFF, route: Route::To("mm_15_gate_tomb") },
            ChoiceDef { label: "与悔罪祭司谈", sub: "那个石柱后躲着的祭司", cond: None, effects: &NO_EFF, route: Route::To("mm_11_npc") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_11_npc", bg: Some("mumiyi_bg_open.png"),
        loc: Some("圣甲虫厅 · 石柱后"), mood: "悔罪的低语",
        speaker: Some("悔罪祭司·安卡图"), voice: None,
        text: TextSpec::Static(&[
            "一名白发祭司从柱后探出半个身子，声音颤抖：",
            "「我……我是千年前侍奉哈姆纳塔的末裔祭司。我们族里只有一条祖训——」",
            "「莫开棺。开了棺，伊莫顿醒来，弱水是唯一克星。圣甲虫之匣可封死墓门。」",
        ]),
        choices: &[
            ChoiceDef { label: "谢过这位祭司", sub: "得到关于弱水与虫匣的提示", cond: None,
                effects: &[Eff::SetFlag("mm_priest_hint")], route: Route::To("mm_10_arrive_f1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_12_well", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("圣甲虫厅 · 弱水井"), mood: "幽蓝微光",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "井水泛着诡异的幽蓝，倒映着无数爬动的甲虫阴影。",
            "安卡图说那是「尼罗弱水」——天造地设克制伊莫顿之物。你装满了一只水囊。",
        ]),
        choices: &[
            ChoiceDef { label: "装满一囊弱水", sub: "弱水终结技的前置道具", cond: None,
                effects: &[Eff::AddItem("it_mumi_water")], route: Route::To("mm_10_arrive_f1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_13_hollow", bg: Some("mumiyi_bg_open.png"),
        loc: Some("圣甲虫厅 · 虫巢神像"), mood: "虫在神像里",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "巨大神像的胸腔被凿空，塞满了早已风化的尸骸——那是历代献祭者的骨。",
            "神像腹中吐着一枚黑漆漆的圣甲虫之匣，半阖的匣口爬出几只活虫。",
        ]),
        choices: &[
            ChoiceDef { label: "取出圣甲虫之匣", sub: "用它封死祭司墓门", cond: None,
                effects: &[Eff::AddItem("it_mumi_scarab_sac")], route: Route::To("mm_10_arrive_f1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_14_vault", bg: Some("mumiyi_bg_open.png"),
        loc: Some("圣甲虫厅 · 宝库供案"), mood: "尘封的宝库",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "疏落的南侧偏室摆着一张石供案，案上是一只被虫脐环抱的翡翠圣匣。",
            "你掀开圣匣——里面是历代祭司殉葬的宝石与金饰，还有几枚圣甲虫护符。",
        ]),
        choices: &[
            ChoiceDef { label: "席卷宝库珍宝", sub: "翡翠+宝石+金饰 多掉落", cond: None,
                effects: &[Eff::AddItem("it_mumi_loot_gem"), Eff::AddItem("it_mumi_trinket"), Eff::Points(150)], route: Route::To("mm_10_arrive_f1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_15_gate_tomb", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("封印墓门"), mood: "啮齿咬合",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "墓门两侧各有一个甲虫加玺凹槽，正好容纳你手里的圣甲虫之匣。",
        ]),
        choices: &[
            ChoiceDef { label: "嵌入圣甲虫之匣", sub: "封印墓门向两侧退开", cond: Some(cond_has_scarab_sac),
                effects: &[Eff::SetFlag("mm_gate_tomb_open")], route: Route::To("mm_20_sarc_room") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ================= F2 祭司墓室 =================
    SceneDef {
        id: "mm_20_sarc_room", bg: Some("mumiyi_bg_battle.png"),
        loc: Some("祭司墓室"), mood: "正中一具石棺",
        speaker: Some("外景"), voice: None,
        text: TextSpec::Static(&[
            "封印墓门后便是祭司墓室。室内正中静静躺着一具巨大的石碑石棺——其上刻满诅咒。",
            "考古队已经抢先一步，正围着石棺撬凿；阿尔德冲你喊：「别让他们开棺！」",
        ]),
        choices: &[
            ChoiceDef { label: "上前查看伊莫顿石棺", sub: "石棺上的铭文隐约许诺『未亡者』", cond: None, effects: &NO_EFF, route: Route::To("mm_21_open_sarc") },
            ChoiceDef { label: "走向北侧弱水池演出", sub: "一座通向地底的露台水潭", cond: None, effects: &NO_EFF, route: Route::To("mm_26_pool") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_21_open_sarc", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("伊莫顿石棺"), mood: "棺材在颤",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你伸手按住棺盖，触手冰凉。石棺内部传来极轻的、规律的吐息。",
            "考古队队长阿尔德抢先一步，一把撬开棺盖——棺内缠着绷带的身影猛地睁眼。",
        ]),
        choices: &[
            ChoiceDef { label: "诅咒苏醒了！", sub: "伊莫顿自棺中直直坐起", cond: None,
                effects: &[Eff::SetFlag("mm_open_sarc"), Eff::SetFlag("mm_curse"), Eff::San(-10)],
                route: Route::To("mm_22_curse") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    SceneDef {
        id: "mm_22_curse", bg: Some("mumiyi_bg_battle.png"),
        loc: Some("祭司墓室 · 中央"), mood: "黄沙漫卷",
        speaker: Some("考古队长·阿尔德"), voice: Some("vo_mumiyi_1"),
        text: TextSpec::Static(&[
            "「不——」阿尔德踉跄退步，那个开棺的人当场被自己的护符击穿，倒地气绝。",
            "石棺内涌出黄沙，在中央凝成一尊披绷带的人形。满室烛火尽灭，圣甲虫自壁画倾巢而出。",
            "大祭司伊莫顿，自诅咒中——复活。",
        ]),
        choices: &[
            ChoiceDef { label: "拔出武器，迎战伊莫顿", sub: "正面应战大祭司", cond: None,
                effects: &NO_EFF, route: Route::Dyn(route_start_imhotep1) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ---- 伊莫顿一段 · 选择驱动 ----
    SceneDef {
        id: "mm_23_imhotep1", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("祭司墓室 · 诅咒战场"), mood: "沙暴压顶",
        speaker: Some("大祭司·伊莫顿"), voice: None,
        text: TextSpec::Dyn(txt_boss1),
        choices: &[
            ChoiceDef { label: "挥剑强攻", sub: "直取伊莫顿本体", cond: None, effects: &NO_EFF, route: Route::Dyn(route_imhotep1_atk) },
            ChoiceDef { label: "以弱水祭炼", sub: "用尼罗弱水浇向绷带之躯（需先取水）", cond: Some(cond_has_water), effects: &NO_EFF, route: Route::Dyn(route_imhotep1_water) },
            ChoiceDef { label: "伺机抽身", sub: "先退一步重整旗鼓", cond: None, effects: &NO_EFF, route: Route::To("mm_20_sarc_room") },
            ChoiceDef { label: "呼唤考古队合力", sub: "阿尔德与乔伊举火把支援", cond: None, effects: &NO_EFF, route: Route::Dyn(route_imhotep1_help) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ---- 复生二段 ----
    SceneDef {
        id: "mm_24_reborn", bg: Some("mumiyi_bg_battle.png"),
        loc: Some("祭司墓室 · 裂棺"), mood: "他再度凝形",
        speaker: Some("大祭司·伊莫顿"), voice: Some("vo_mumiyi_2"),
        text: TextSpec::Static(&[
            "你一刀斩断了他的绷带之躯，黄沙四散飞溅。",
            "然而转瞬之间——四散的黄沙逆向聚拢，伊莫顿自沙尘中复生，骨节喀喀作响，面容狰狞。",
            "「凡人之刃，杀不死的……唯有圣水，方能真正令我归于沉寂。」",
        ]),
        choices: &[
            ChoiceDef { label: "再次迎向复生之躯", sub: "圣甲虫潮增员，你握紧腰间水囊", cond: None,
                effects: &NO_EFF, route: Route::Dyn(route_start_imhotep2) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ---- 伊莫顿二段 · 选择驱动 ----
    SceneDef {
        id: "mm_25_imhotep2", bg: Some("mumiyi_bg_invest.png"),
        loc: Some("祭司墓室 · 夺命离场"), mood: "穷途末路",
        speaker: Some("大祭司·伊莫顿"), voice: None,
        text: TextSpec::Dyn(txt_boss2),
        choices: &[
            ChoiceDef { label: "挥剑强攻", sub: "硬撼复生之体", cond: None, effects: &NO_EFF, route: Route::Dyn(route_imhotep2_atk) },
            ChoiceDef { label: "以尼罗弱水终结", sub: "一囊弱水浇落，咒文溃散", cond: Some(cond_has_water), effects: &NO_EFF, route: Route::Dyn(route_imhotep2_water) },
            ChoiceDef { label: "后退重整", sub: "避开虫雨再找破绽", cond: None, effects: &NO_EFF, route: Route::To("mm_20_sarc_room") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ---- 弱水池演出（支线）----
    SceneDef {
        id: "mm_26_pool", bg: Some("mumiyi_bg_battle.png"),
        loc: Some("祭司墓室 · 弱水池演出"), mood: "倒流的水",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "墓室北侧有座露台水潭，潭水幽蓝，竟在逆流。",
            "你隐约听见水底传来被囚禁者的哀号——那是历代试图逃脱哈姆纳塔之人的亡魂。",
        ]),
        choices: &[
            ChoiceDef { label: "回到墓室中央", sub: "剑拔弩张的战场", cond: None, effects: &NO_EFF, route: Route::To("mm_20_sarc_room") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // ---- 结局 ----
    SceneDef {
        id: "mm_27_win", bg: Some("mumiyi_bg_open.png"),
        loc: Some("祭司墓室 · 封棺"), mood: "尘埃落定",
        speaker: Some("主神提示"), voice: None,
        text: TextSpec::Static(&[
            "弱水浇透伊莫顿的绷带之躯，他发出一声不像人声的哀鸣，崩解成一地白砂。",
            "你们以圣甲虫之连通牢不可破地封上石棺，诅咒随伊莫顿一同归寂。",
            "主神的声音响起：「任务完成。回归。」",
        ]),
        choices: &[
            ChoiceDef { label: "领取奖励，回归主神空间", sub: "伊莫顿已封印", cond: None,
                effects: &[Eff::SetFlag("mm_end_seal"), Eff::Points(800)], route: Route::To("mm_28_escape") },
        ],
        fight_id: None, video: None, cine_label: Some("mumiyi_win"), overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |_st| crate::state::Card {
                title: "木乃伊 · 哈姆纳塔地宫 · 通关".into(),
                good: true,
                body_html: "<p>你封住了大祭司伊莫顿，诅咒褪去，黄金与宝石尽入囊中。</p><p><b>任务评级：C（★★★★ 改编）</b></p>".into(),
                buttons: vec![("回归主神空间".into(), "mm_28_escape".into())],
                voice: None,
            },
        }),
    },

    SceneDef {
        id: "mm_28_escape", bg: Some("mumiyi_bg_open.png"),
        loc: Some("哈姆纳塔 · 撤离"), mood: "破晓",
        speaker: Some("外景"), voice: None,
        text: TextSpec::Static(&[
            "你们携着打出的黄金器皿、翡翠圣匣与圣甲虫护符，在破晓前撤出地宫。",
            "身后的沙丘缓缓吞噬石门，哈姆纳塔再次归于尘沙。某些诅咒，永远也不该被掘出。",
        ]),
        choices: &NO_CH,
        fight_id: None, video: None, cine_label: Some("ending_escape"), overlay: None,
    },

    SceneDef {
        id: "mm_29_lose", bg: Some("mumiyi_bg_battle.png"),
        loc: Some("祭司墓室 · 沙海"), mood: "黄沙没顶",
        speaker: Some("大祭司·伊莫顿"), voice: None,
        text: TextSpec::Static(&[
            "你脱力倒在虫雨之中。伊莫顿嘲讽地俯视你——",
            "「凡人的血肉，喂饱我的圣甲虫。」",
            "最后一刻，黄沙漫过你的视野。任务失败。",
        ]),
        choices: &NO_CH,
        fight_id: None, video: None, cine_label: Some("ending_lose"), overlay: Some(OverlayDef {
            voice: None, death: Some(("死亡档案 · 木乃伊", "被圣甲虫吞噬")),
            card: |_st| crate::state::Card {
                title: "回归主神空间".into(), good: false,
                body_html: "<p>你葬身于哈姆纳塔遍地的虫潮之中。</p><p>主神扣除复活费 400 点。</p>".into(),
                buttons: vec![("返回主神空间".into(), "mm_29_lose".into())],
                voice: None,
            },
        }),
    },
];
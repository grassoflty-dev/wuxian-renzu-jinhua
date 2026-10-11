//! 《侠行天下 · 藏经阁》全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/00_INDEX_EXPANSION.md §1.7 `cangjingge`（藏经阁 · 绝学之争）
//! 与 xiaxing_tianxia/00_xiaxing_tianxia_research.md §5 候选5（藏经阁深处）。
//! 本文件是全新新增文件，只导出静态数据（CANGJING_SCENES / cangjingge_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/cangjingge_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `cj_` 前缀，与既有 SCENES 无重名；fight id 全部 `cj_` 前缀。
//! BOSS 入魔守阁僧采用"选择驱动遭遇链"（参考 scenes_jiguancheng.rs 的 jc_keeper / scenes_zhouyuan.rs 的 route_boss_attack）：
//! 因需要「出示檀木信物免战解脱」「破碎心法许老僧入定」等自定义每回合同调，引擎原生 FightCfg
//! 无此钩子，故用 Normal 场景 + Route::Dyn 落地；同时导出 `cj_shouge`/`cj_xinmo`/`cj_shuling` FightCfg
//! 供 ZoneDef 与揭示用。
//! 秘籍收集链：三卷武学残卷（it_miju_a/b/c）→ 守经人手札 → 守经人游魂指引（keeper_clue_1/2）
//! → 取檀木信物（it_tan_token）供 BOSS 免战。sp_grade 用 Route::Dyn 写 Some('D')。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   L0 经堂   井 bg cj_bg_gate    （现用 img_zhuyuan_book.png 占位）
//!   L1 一楼书房 井 bg cj_bg_hall   （现用 img_laser.png 占位）
//!   L2 禁书库  井 bg cj_bg_scripture（现用 img_corridor.png 占位）
//!   L3 秘籍塔顶 井 bg cj_bg_scripture_tower（现用 img_zhuyuan_book.png 占位）
//!   L3 心魔洞  井 bg cj_bg_xinmo   （现用 img_laser.png 占位）
//! 敌人立绘复用：guard→护阁武僧、hunter→叛经者、zombie→书页纸傀；新美术由主 agent 统一生图替换。

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
   具名条件谓词（cond：fn 指针）
   ===================================================================== */
fn cond_has_tongling(st: &GameState) -> bool { inv(st, "it_tongling") }
fn cond_has_tan_key(st: &GameState) -> bool { inv(st, "it_tan_key") }
fn cond_keeper_clue2(st: &GameState) -> bool { st.flag("keeper_clue_2") }
fn cond_xinmo_unlock(st: &GameState) -> bool { st.flag("xinmo_unlock") }
fn cond_shouge_raged(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.raged).unwrap_or(false)
}

/* =====================================================================
   BOSS · 入魔守阁僧（选择驱动遭遇，参照 jc_keeper）
   血量存 st.fight（cj_22_shouge 的 Route::Dyn 初始化，引用 cj_shouge 的 FightCfg）。
   每"回"是 Normal 场景 cj_shouge_round；Route::Dyn 统一处理：扣血、狂暴、免战解脱、胜负路由。
   ===================================================================== */
fn start_shouge(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("cj_shouge") {
            st.fight = Some(crate::power::scaled_fight("cj_shouge", cfg, st, vec![]));
        }
    }
    "cj_shouge_round".to_string()
}

/// 守阁僧解脱（出示檀木信物）：+300、檀木信物消耗掉、置 cj_shouge_freed + sp_grade=D
fn shouge_freed(st: &mut GameState) -> String {
    st.points += 300;
    crate::world::add_item(st, "it_juexue_xin");
    st.set_flag("cj_shouge_freed");
    st.sp_grade = Some('D');
    "cj_24_shouge_freed".to_string()
}

/// 击杀入魔守阁僧：+300、掉绝学心法钞、置 cj_shouge_down + sp_grade=D
fn shouge_win(st: &mut GameState) -> String {
    st.points += 300;
    crate::world::add_item(st, "it_juexue_xin");
    st.set_flag("cj_shouge_down");
    st.sp_grade = Some('D');
    "cj_23_shouge_down".to_string()
}

fn shouge_dead() -> String { "cj_40_death_shouge".to_string() }

/// 一个"回"：玩家攻击守阁僧。guard = 闭心守势（提升闪避）。
fn shouge_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return shouge_win(st);
    }
    // 狂暴（HP ≤ 60）
    let raged = st.fight.as_ref().map(|f| f.hp <= 60).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
        st.hp = (st.hp - 2).max(0); // 执念外溢反噬(-2)
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(20, 28) } else { rng(14, 22) };
    let dodge = if guard { 0.52 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return shouge_dead();
    }
    "cj_shouge_round".to_string()
}

/* =====================================================================
   隐藏 BOSS · 心魔（选择驱动遭遇）
   ===================================================================== */
fn start_xinmo(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("cj_xinmo") {
            st.fight = Some(crate::power::scaled_fight("cj_xinmo", cfg, st, vec![]));
        }
    }
    "cj_xinmo_round".to_string()
}

/// 心魔压制：+120、掉破障心法、置 xinmo_down
fn xinmo_win(st: &mut GameState) -> String {
    st.points += 120;
    crate::world::add_item(st, "it_xinmo_perscroll");
    st.set_flag("xinmo_down");
    "cj_27_xinmo_win".to_string()
}

fn xinmo_dead() -> String { "cj_40_death_xinmo".to_string() }

fn xinmo_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return xinmo_win(st);
    }
    let raged = st.fight.as_ref().map(|f| f.hp <= 40).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged_now = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged_now { rng(17, 24) } else { rng(12, 19) };
    let dodge = if guard { 0.5 } else { 0.16 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return xinmo_dead();
    }
    "cj_xinmo_round".to_string()
}

/* =====================================================================
   胜利 / 失败共通 win 回调（普通敌人；FightCfg.win 用）
   ===================================================================== */
fn cj_win_l1(_st: &GameState) -> String { "cj_10_arrive_floor1".to_string() }
fn cj_win_l2(_st: &GameState) -> String { "cj_14_arrive_floor2".to_string() }
fn cj_win_shuling(_st: &GameState) -> String { "cj_20_shuling_win".to_string() }
fn cj_win_guard2(_st: &GameState) -> String { "cj_10_guard2_win".to_string() }
fn cj_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/// 战斗配置表（id 全部 cj_ 前缀）。
pub fn cangjingge_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("cj_guard1", FightCfg {
            name: "护阁武僧", hp: 36, dmg: (7, 13), reward: 12, reward_why: "肃清一楼护阁武僧",
            intro: "一尊披灰褂的武僧立在书架夹道，像一座会呼吸的灰塔，铜戒在指节上铛铛作响，每响一下，书架就静一分。他摆出守经架势，目光却先落在你腰间的书卷上。",
            rage_at: None, rage_text: "", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_l1, death: "cj_40_death",
        }),
        ("cj_zhikui", FightCfg {
            name: "书页纸傀", hp: 32, dmg: (6, 11), reward: 10, reward_why: "书页纸傀 · 散作纸屑",
            intro: "无数发黄的书页凭空卷成一个纸人，刷刷作响，纸面绷得笔直，像一把把没开刃的刀，朝你扑来，带起的风里全是墨味。",
            rage_at: None, rage_text: "", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_l1, death: "cj_40_death",
        }),
        ("cj_guard2", FightCfg {
            name: "护阁武僧 · 持棍", hp: 46, dmg: (9, 15), reward: 22, reward_why: "一楼护阁武僧 · 持棍",
            intro: "一名壮实的武僧横一根熟铜棍在入口，铜棍上缠着旧布，棍头磨得发亮，袍角无风自动，神情戒备——他守的不是门，是门后那排书架。",
            rage_at: Some(22), rage_text: "熟铜棍旋成一片棍影，攻势陡然加密！", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_guard2, death: "cj_40_death",
        }),
        ("cj_hunter", FightCfg {
            name: "叛经者", hp: 54, dmg: (10, 16), reward: 30, reward_why: "制伏叛经者",
            intro: "一个黑袍人贴着禁书库的暗影闪出，脚步比猫还轻，袖中藏着撕下的残页，纸边还带着新鲜的血迹——他是来偷经的，而且偷的不是第一卷。",
            rage_at: Some(24), rage_text: "他暴起反扑，出招阴狠且快！", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_l2, death: "cj_40_death",
        }),
        ("cj_guard3", FightCfg {
            name: "护阁武僧 · 统领", hp: 62, dmg: (11, 17), reward: 40, reward_why: "制伏二楼武僧统领",
            intro: "二楼禁书库廊上，一名赤膊武僧统领闭目而立，背上的肌肉绷得像铁铸的，双手合十，缓缓睁眼——眼里的光，比廊上的灯还亮一分。",
            rage_at: Some(28), rage_text: "他口诵怒目金刚咒，拳风斗然大盛！", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_l2, death: "cj_40_death",
        }),
        ("cj_guard4", FightCfg {
            name: "绝学门人", hp: 66, dmg: (12, 18), reward: 50, reward_why: "秘境塔顶层 · 绝学门人",
            intro: "密室门后的习武之人拔剑而立，剑锋映着塔顶天光，剑尖垂在离地三寸处——他守着一门绝学，也守着自己练了半生的那口气。",
            rage_at: Some(30), rage_text: "他身体里浮现一门武功的虚影，出剑更快更凶！", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_l2, death: "cj_40_death",
        }),
        ("cj_shuling", FightCfg {
            name: "经阁书灵", hp: 80, dmg: (11, 17), reward: 60, reward_why: "平定经阁书灵 · 得残卷真解",
            intro: "一册无字的经书在半空自行翻页，纸页哗哗响成一片，字迹潮水般涌出凝成一个人形，眉眼模糊得像隔着一层墨——经阁之灵，它睁开眼时，整座一楼的书都跟着翻了一页。",
            rage_at: Some(30), rage_text: "书页如刃卷作风暴，把整座书架都掀动起来！", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: cj_win_shuling, death: "cj_40_death",
        }),
        ("cj_shouge", FightCfg {
            name: "入魔守阁僧", hp: 150, dmg: (16, 24), reward: 300, reward_why: "解脱入魔守阁僧",
            intro: "枯坐于塔顶的老僧缓缓抬头，双目赤红如炭火，眼白爬满血丝，袈裟下四肢骨瘦如柴，却稳得像生了根。「你……也想取那部禁书？」他的声音像翻破的书页，一字一字，都带着纸屑的脆响。",
            rage_at: Some(60), rage_text: "入魔执念化为纸链缠上全身——他口中念的，变成了禁书里的字。",
            on_rage: |_st, _log| {},
            finisher_if: |st, _| inv(st, "it_tan_token") && st.fight.as_ref().map(|f| f.raged).unwrap_or(false),
            finisher_name: |_| "出示檀木信物".to_string(),
            finisher_desc: |_| "你将那枚熏过佛前檀香的信物递到他掌心。老僧怔住，纸链自指间寸寸断裂，赤红的火色如潮水般褪去。".to_string(),
            win: |_st| "cj_23_shouge_down".to_string(),
            death: "cj_40_death_shouge",
        }),
        ("cj_xinmo", FightCfg {
            name: "心魔", hp: 90, dmg: (12, 19), reward: 120, reward_why: "镇压心魔 · 得破障心法",
            intro: "洞中一团黑影幻化你的形状，连站姿都学得一模一样，脚边的经页上浮着毁容的「你」字，字迹是你自己的笔迹。它开口，用的是你自己的声音：「你连自己都不认得，怎么认得我？」",
            rage_at: Some(40), rage_text: "它暴长出与你一模一样的脸，森森笑着朝你逼近！", on_rage: cj_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |_st| "cj_27_xinmo_win".to_string(),
            death: "cj_40_death_xinmo",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn cj_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    cangjingge_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 cj_ 前缀）
   ===================================================================== */
pub static CANGJING_SCENES: &[SceneDef] = &[

/* ================= 幕一 · 开场（山门）================= */
SceneDef {
    id: "cj_00", bg: Some("cangjingge_bg.png"), loc: Some("藏经阁 · 山门"),
    mood: "mystery", speaker: Some("旁侍老僧"), voice: Some("vo_cj_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>追查一部失传绝学的下落，摸清守阁老僧的立场。失败代价：被扣 300 点复活。",
        "山门石阶浸在雨后的青苔里，一脚踩下去就留下湿印，两侧石经幢默立，像两个不看路的守夜人。一位旁侍老僧在檐下扫落叶，扫帚一顿，落叶停在半空，他抬眼看你：「施主此来，可是为那一卷——禁书？」",
    ]),
    choices: &[
        ChoiceDef { label: "细看山门石经幢", sub: "+5 点 · 铭文", cond: None,
            effects: &[Eff::SetFlag("cj_stele_scan"), Eff::Points(5)], route: Route::To("cj_05_stele") },
        ChoiceDef { label: "向老僧问路", sub: "免战信物线索", cond: None,
            effects: &[], route: Route::To("cj_03_laoseng") },
        ChoiceDef { label: "直入经堂", sub: "登廊入殿", cond: None,
            effects: &NO_EFF, route: Route::To("cj_01") },
    ],
    fight_id: None, video: Some("vid_cj_opening.mp4"), cine_label: Some("过场 · 山门钟声"), overlay: None,
},

/* ---- L0 经堂 hub ---- */
SceneDef {
    id: "cj_01", bg: Some("cangjingge_bg.png"), loc: Some("L0 · 山门与经堂"),
    mood: "mystery", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_tongling") {
            "铜铃已在腰畔。经堂内门 G1 在正北——有铃可直入一楼正道。".to_string()
        } else {
            "经堂大殿悬着长长的经幡，从梁上垂下来，风过时像一排静默的诵经人。东侧配殿有解签炉，香灰已冷；正北经堂内门挂着铜闩，闩上结了薄锈——<em>无铜铃者，只能走屋脊滑道坠楼（p_cj_1）。</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "解签炉", sub: "求一签 · 守经人伏笔", cond: None, effects: &NO_EFF, route: Route::To("cj_04_sign") },
        ChoiceDef { label: "铜铃法台", sub: "取铜铃 · 开 G1", cond: None, effects: &NO_EFF, route: Route::To("cj_06_bell") },
        ChoiceDef { label: "藏经阁匾额", sub: "读题字", cond: None, effects: &NO_EFF, route: Route::To("cj_05_plaque") },
        ChoiceDef { label: "进经堂内门", sub: "需铜铃 → 一楼正道", cond: Some(cond_has_tongling), effects: &NO_EFF, route: Route::To("cj_10_arrive_floor1") },
        ChoiceDef { label: "屋脊滑道", sub: "p_cj_1 单向坠楼 · 无门禁但险", cond: None, effects: &NO_EFF, route: Route::To("cj_10_slip") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- L0 调查点场景 ---- */
SceneDef {
    id: "cj_05_stele", bg: Some("cangjingge_bg.png"), loc: Some("L0 · 山门石经幢"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["石经幢上刻着经偈，字迹被风雨磨得发亮：「藏经如藏火，翻书如纵己。一字掀起万卷浪，卷尽江湖是非心。」你念了两遍，觉得那经偈像一句警告，又像一句收殓词。"]),
    choices: &[ChoiceDef { label: "默记经偈", sub: "+5 点", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("cj_p_l0_1")], route: Route::To("cj_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_03_laoseng", bg: Some("cangjingge_bg.png"), loc: Some("L0 · 檐下 · 旁侍老僧"),
    mood: "cold", speaker: Some("旁侍老僧"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("keeper_clue_1") {
            "老僧深深看你一眼，扫帚拄在地上：「守经人告诉你说——信物在二楼锁钥架的檀木匣里。老衲只一句：经是死的，人是活的。别让一部书，替你把一辈子过完。」".to_string()
        } else {
            "老僧缓缓道，声音像落叶扫过石阶：「阁中有一部禁书，还有一位守了它一辈子的老人，从青丝守到白头。你若想让他放下——先找到那位守着残卷的游魂，他认得回去的路。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "谢过老僧", sub: "线索 · 守经人游魂", cond: None,
        effects: &[Eff::SetFlag("cj_lao_clue"), Eff::MarkPoint("cj_p_l0_4")], route: Route::To("cj_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_04_sign", bg: Some("cangjingge_bg.png"), loc: Some("L0 · 解签炉"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你从解签炉里抽出一根签，签身还带着炉灰的温热，签文：『字多则魔生，阅尽乃得安』。炉底压着一页泛黄的武学残卷，页角卷得像被人翻过千百遍——它自己跳进了你的行囊。"]),
    choices: &[ChoiceDef { label: "取走残卷·乙", sub: "Item it_miju_b", cond: None,
        effects: &[Eff::AddItem("it_miju_b"), Eff::SetFlag("cj_miju_l0"), Eff::MarkPoint("cj_p_l0_2")], route: Route::To("cj_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_06_bell", bg: Some("cangjingge_bg.png"), loc: Some("L0 · 铜铃法台"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["铜铃法台上一枚巴掌大的铜铃泛着温润的铜光，像被无数只手摩挲过。铃身铸着小字：「鸣一声，藏经开。」你握住它，铃舌在掌心里微微一颤，像它也等了很久。"]),
    choices: &[ChoiceDef { label: "取下铜铃", sub: "Item it_tongling · 开 G1", cond: None,
        effects: &[Eff::AddItem("it_tongling"), Eff::MarkPoint("cj_p_l0_3")], route: Route::To("cj_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_05_plaque", bg: Some("cangjingge_bg.png"), loc: Some("L0 · 藏经阁匾额"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["匾上四个大字「藏经在望」，金漆剥落了大半，木边刻着一行小字：『经在楼台，人在楼台之下』。你仰头看了许久，觉得那行小字，比四个大字更像这阁楼的题眼。"]),
    choices: &[ChoiceDef { label: "微忖匾意", sub: "+5 点", cond: None,
        effects: &[Eff::Points(5), Eff::MarkPoint("cj_p_l0_4")], route: Route::To("cj_01") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L1 一楼 · 书架丛林 ================= */
SceneDef {
    id: "cj_10_arrive_floor1", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 藏经阁一楼"),
    mood: "cold", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("keeper_clue_2") {
            "一楼真卷 / 残卷已窥全貌，东侧书灵封印台可按心法化去。".to_string()
        } else {
            "（自经堂而入一楼）满楼是书架密林，一排排望不到头，纸页潮气与墨香混作一处，呛得人喉头发紧。你循谶言寻那三卷武学残卷，以及——一位守着残卷的游魂。书架深处有翻页声，却不见人影。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "书架 · 武学残卷·甲", sub: "残卷 1/3", cond: None, effects: &NO_EFF, route: Route::To("cj_11_scroll_a") },
        ChoiceDef { label: "书架 · 武学残卷·乙", sub: "残卷 2/3", cond: None, effects: &NO_EFF, route: Route::To("cj_11_scroll_b") },
        ChoiceDef { label: "书架 · 武学残卷·丙", sub: "残卷 3/3", cond: None, effects: &NO_EFF, route: Route::To("cj_11_scroll_c") },
        ChoiceDef { label: "守经人手札", sub: "剧情伏笔", cond: None, effects: &NO_EFF, route: Route::To("cj_13_keeper_note") },
        ChoiceDef { label: "经阁书灵封印台", sub: "书灵之战", cond: None, effects: &NO_EFF, route: Route::To("cj_20_shuling") },
        ChoiceDef { label: "中央书梯（需守经人指引）", sub: "G2 → 二楼禁书库", cond: Some(cond_keeper_clue2), effects: &NO_EFF, route: Route::To("cj_14_lift") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_11_scroll_a", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 书架 · 武学残卷·甲"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["书架深处的暗格里躺着一卷发黄的武学残卷，扉页缺了一角，像是被谁撕走的，内文是一种说不出名字的掌法——起手式画得极简，简到近乎无声，却让人不敢久看。"]),
    choices: &[ChoiceDef { label: "取走残卷·甲", sub: "Item it_miju_a", cond: None,
        effects: &[Eff::AddItem("it_miju_a"), Eff::MarkPoint("cj_p_l1_1")], route: Route::Dyn(route_miju_collect) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_11_scroll_b", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 书架 · 武学残卷·乙"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["书页夹层里抖落出一页残卷，墨迹新得反常，浓得发亮——像才被人从某部经书里撕下，还没来得及藏稳。撕口还留着半截线头，牵着某本没合拢的书。"]),
    choices: &[ChoiceDef { label: "取走残卷·乙", sub: "Item it_miju_b", cond: None,
        effects: &[Eff::AddItem("it_miju_b"), Eff::MarkPoint("cj_p_l1_2")], route: Route::Dyn(route_miju_collect) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_11_scroll_c", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 书架 · 武学残卷·丙"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["最底层的书箱压着一卷残页，油布裹了三层，拆开时手先凉了半截。扉页血字一行，颜色已沉成暗褐：「此卷不假之人，因果自背。」字是写的人自己蘸着写下的，你数了数，正好十个字。"]),
    choices: &[ChoiceDef { label: "取走残卷·丙", sub: "Item it_miju_c", cond: None,
        effects: &[Eff::AddItem("it_miju_c"), Eff::MarkPoint("cj_p_l1_3")], route: Route::Dyn(route_miju_collect) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_13_keeper_note", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 守经人手札"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_miju_full") {
            "守经人手札最后几行被血浸透，血字漫过纸背：『所托已全……若有一日有人集齐三卷残页来此，便是我等守者毕生所求。』你合上手札的刹那，一个游魂在你身边渐渐凝实，他望着你，像望着一封迟到的信。".to_string()
        } else {
            "书架夹层压着一卷守经人的手札，纸页边角被磨得发毛：「藏经楼以书为界，以人铸关。三卷残页集齐之日，守经人之魂方肯指路。」落款处没有名字，只有一枚按了又按的指印。你要集齐三卷。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "合上手札", sub: "触发守经人指引（集结三卷后）", cond: None,
        effects: &[Eff::SetFlag("cj_note_read"), Eff::MarkPoint("cj_p_l1_4")], route: Route::Dyn(route_keeper_clue) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_20_shuling", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 经阁书灵封印台"),
    mood: "danger", speaker: Some("经阁书灵"), voice: Some("vo_cj_shuling"),
    text: TextSpec::Static(&["封印台上那册无字经书自空中盘旋翻页，书页哗哗地响，字迹潮涌之上凝成一团人烟——经阁书灵睁目，无瞳的眼扫过你，满楼纸页应声狂舞，墨香陡然浓成腥味。"]),
    choices: &[ChoiceDef { label: "【迎战书灵】", sub: "cj_shuling", cond: None, effects: &NO_EFF, route: Route::To("cj_20_shuling_fight") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_20_shuling_fight", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 经阁书灵封印台"),
    mood: "danger", speaker: None, voice: Some("vo_cj_shuling"),
    text: TextSpec::Static(&["书页如刃卷成风暴，纸屑贴着地面旋起来，朝你当面削来，每一片纸边都绷得笔直。（战斗）"]),
    choices: &[], fight_id: Some("cj_shuling"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_20_shuling_win", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 经阁书灵封印台"),
    mood: "calm", speaker: Some("经阁书灵残响"), voice: None,
    text: TextSpec::Static(&["书页散落一地，像一场下完了的雪，那册无字经书缓缓合上，落入你手中成为一部「内容真解」。你揭开了残卷扉页的封印，字迹在纸面上一行行亮起来，像终于等到人读。"]),
    choices: &[ChoiceDef { label: "拾起真解批注", sub: "Item it_miju_full · 触发守经人指引", cond: None,
        effects: &[Eff::AddItem("it_miju_full"), Eff::MarkPoint("cj_p_l1_5")], route: Route::Dyn(route_keeper_clue) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_10_guard2_win", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 书梯口"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["持棍武僧拄棍倒下，铜棍在石板上滚了两圈，一楼归于沉寂，只剩纸页落灰的声音。书梯口的铜闸仍横着戒律，锁着通往二楼的路。"]),
    choices: &[ChoiceDef { label: "（回一楼继续）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("cj_10_arrive_floor1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_10_slip", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 屋脊滑道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["无铃也行。你顺着屋脊滑道一坠，瓦片在脚下碎裂，风灌满衣袖，最后重重落进一楼的一堆草蒲团里，扬起一片灰——险路无门禁，但一路纸傀惊醒，书页在暗处窸窸窣窣地立起来。"]),
    choices: &[ChoiceDef { label: "（落地 · 一楼）", sub: "p_cj_1 单向 · 纸傀", cond: None, effects: &NO_EFF, route: Route::To("cj_10_arrive_floor1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_14_lift", bg: Some("cangjingge_bg.png"), loc: Some("L1 · 中央书梯（G2 已开）"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["念出戒律暗语，铜闸缓缓沉入地板，齿轮声在脚底滚过。你踏入书梯的木笼——绳缆吱呀上行，越升越陡，身后一声尖叫，缆绳断裂，木笼坠回黑暗，而你被稳稳送上了二楼。"]),
    choices: &[ChoiceDef { label: "（升梯至二楼禁书库）", sub: "p_cj_2 单向 · 缆断", cond: None, effects: &NO_EFF, route: Route::To("cj_14_arrive_floor2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L2 二楼 · 禁书库 ================= */
SceneDef {
    id: "cj_14_arrive_floor2", bg: Some("cangjingge_bg.png"), loc: Some("L2 · 禁书库"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Dyn(|st| {
        if inv(st, "it_tan_key") {
            "二楼禁书库里，经柜错落、锁钥串响。西侧守阁僧禅房闭户，东侧锁钥架——你已握着檀木秘钥。".to_string()
        } else {
            "二楼禁书库里，经柜错落，锁钥串响，灯光昏黄得只够照见三步。西侧守阁僧禅房闭户，门缝里漏出一线烛火；东侧藏稿阁的锁钥架上悬着一枚檀木秘钥，在光里微微泛红——那是开 L3 绝学密室的钥匙。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "禁书卷题", sub: "读禁书 · 剧情", cond: None, effects: &NO_EFF, route: Route::To("cj_15_forbidden") },
        ChoiceDef { label: "守阁僧禅房 · 闭户", sub: "剧情 / 守阁僧立场", cond: None, effects: &NO_EFF, route: Route::To("cj_16_chanfang") },
        ChoiceDef { label: "锁钥架 · 檀木匣", sub: "取檀木秘钥（开 G3）", cond: None, effects: &NO_EFF, route: Route::To("cj_17_tan_key") },
        ChoiceDef { label: "经阁密道", sub: "p_cj_back 单向回跳 → 一楼（唯一后悔药）", cond: None, effects: &NO_EFF, route: Route::To("cj_10_arrive_floor1") },
        ChoiceDef { label: "禁书库走廊西口", sub: "p_cj_4 单向 → 秘籍塔", cond: None, effects: &NO_EFF, route: Route::To("cj_19_arrive_tower") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_15_forbidden", bg: Some("cangjingge_bg.png"), loc: Some("L2 · 禁书卷题"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["禁书以铁页封皮锁着，锁扣早已锈死，卷题晦涩：「绝学一念，可遁入势；勘破者，天地之巅亦只是一阁。」守经人游魂立在你身侧，也盯着那行字，像在等你问他一个字。"]),
    choices: &[ChoiceDef { label: "听守经人解说", sub: "剧情", cond: None,
        effects: &[Eff::MarkPoint("cj_p_l2_1")], route: Route::To("cj_14_arrive_floor2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_16_chanfang", bg: Some("cangjingge_bg.png"), loc: Some("L2 · 守阁僧禅房 · 闭户"),
    mood: "cold", speaker: Some("守经人游魂"), voice: None,
    text: TextSpec::Static(&["禅房紧闭，门缝里漏出昏黄烛火，火苗一颤一颤，像里面的灯油快尽了。守经人游魂低声道：「守了它一辈子的老人，如今把自己也锁了进去，锁得比那部书还牢——你若能让他放下那部禁书，他或肯解脱。」"]),
    choices: &[ChoiceDef { label: "记下守经人嘱托", sub: "守经人线索 · 2/2", cond: None,
        effects: &[Eff::SetFlag("keeper_clue_1"), Eff::MarkPoint("cj_p_l2_2")], route: Route::To("cj_14_arrive_floor2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_17_tan_key", bg: Some("cangjingge_bg.png"), loc: Some("L2 · 锁钥架 · 檀木匣"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["藏稿阁的锁钥架上悬着众多钥匙，铜的、铁的、锈成一片的，其中一枚檀木匙单独放置在匣中，傍着一枚贯孔的木牌——那便是檀木信物，熏过佛前香，隔了这么远仍闻得到淡淡檀味。"]),
    choices: &[ChoiceDef { label: "取走檀木秘钥", sub: "Item it_tan_key · 开 G3", cond: None,
        effects: &[Eff::AddItem("it_tan_key"), Eff::MarkPoint("cj_p_l2_3")], route: Route::To("cj_14_arrive_floor2") },
        ChoiceDef { label: "一并取走檀木信物", sub: "Item it_tan_token · BOSS 免战", cond: Some(cond_keeper_clue2),
            effects: &[Eff::AddItem("it_tan_token")], route: Route::To("cj_14_arrive_floor2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= L3 顶层秘籍塔 ================= */
SceneDef {
    id: "cj_19_arrive_tower", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 秘籍塔顶"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "塔顶四壁画着天地山川与一门门失传的武功，壁画褪色，笔意却依旧凌厉。中央塔台之上，一人枯坐，袈裟沾灰，像一尊落了尘的佛——入魔守阁僧。",
        "（西侧绝学密室需檀木秘钥；东侧心魔洞裂隙只在确知禁书真谛后现形。）",
    ]),
    choices: &[
        ChoiceDef { label: "迎战入魔守阁僧", sub: "BOSS 决战", cond: None, effects: &NO_EFF, route: Route::To("cj_22_shouge") },
        ChoiceDef { label: "绝学密室 · 心法刻壁", sub: "需檀木秘钥", cond: Some(cond_has_tan_key), effects: &NO_EFF, route: Route::To("cj_18_juexue") },
        ChoiceDef { label: "心魔洞裂隙", sub: "隐藏战 · 需破障", cond: Some(cond_xinmo_unlock), effects: &NO_EFF, route: Route::To("cj_26_xinmo_door") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_18_juexue", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 绝学密室 · 心法刻壁"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["檀木钥开青铜门，门轴碾出一声闷响，密室四壁刻着一门古奥心法，字字如剑。你盘膝默悟，一道明悟透体而至，四肢百骸都跟着一轻——你把绝学记在了心里，也把它的代价记在了心里。"]),
    choices: &[ChoiceDef { label: "默记绝学", sub: "Item it_juexue · +30 点", cond: None,
        effects: &[Eff::AddItem("it_juexue"), Eff::Points(30), Eff::MarkPoint("cj_p_l3_1")], route: Route::Dyn(route_juexue_gain) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* ---- BOSS · 入魔守阁僧（选择驱动）---- */
SceneDef {
    id: "cj_22_shouge", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 秘籍塔 · 决战"),
    mood: "danger", speaker: Some("入魔守阁僧"), voice: Some("vo_cj_shouge"),
    text: TextSpec::Static(&[
        "你踏上塔台，枯坐的老僧缓缓抬头，双目赤红如炭，眼白早已被血丝浸满。「你……也想取那部禁书？」他的声音像翻破的书页，一开口，塔顶的灰尘便簌簌落了一层。",
        "「真正的绝学，藏在你翻书之前。」他念完这一句，像念完了一生的功课，才慢慢站起来。",
    ]),
    choices: &[ChoiceDef { label: "【逼近塔台】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_shouge) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_shouge_round", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 秘籍塔 · 激战"),
    mood: "danger", speaker: Some("入魔守阁僧"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("守阁僧 HP {} / {}", f.hp.max(0), 150)).unwrap_or_else(|| "守阁僧 HP --".to_string());
        let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
            "——执念化纸链缠身；狂暴后可出示檀木信物免战——"
        } else { "" };
        format!("{f}。{}", mode)
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-42", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| shouge_act(st, rng(30, 42), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 20-28", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| shouge_act(st, rng(20, 28), false)) },
        ChoiceDef { label: "【出示檀木信物】", sub: "狂暴后可免战 · 解脱", cond: Some(cond_shouge_raged),
            effects: &NO_EFF, route: Route::Dyn(shouge_freed) },
        ChoiceDef { label: "闭心守势", sub: "提升闪避", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| shouge_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_23_shouge_down", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 秘籍塔"),
    mood: "calm", speaker: Some("守阁僧残响"), voice: Some("vo_cj_shouge_down"),
    text: TextSpec::Static(&["那缕执念与你的拳风一同溃散，纸链在空气中寸寸化作灰。老僧的袈裟垂落，静坐如涅槃，眉间那点赤红终于褪尽，檀香气味漫开来——一部绝学心法钞落在他膝前，像他替自己抄了半辈子的遗书。"]),
    choices: &[ChoiceDef { label: "（走向经匣石台）", sub: "经匣之择", cond: None, effects: &NO_EFF, route: Route::To("cj_30_box") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_24_shouge_freed", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 秘籍塔 · 解脱"),
    mood: "calm", speaker: Some("守阁僧"), voice: Some("vo_cj_shouge_freed"),
    text: TextSpec::Static(&["檀木信物贴上他的掌心，纸链寸寸断裂，落在地上化成一地碎屑。「……守了一辈子，原来只因一句放不下。」老僧阖目敛心，袈裟落座，如一枚熄灭的灯，灯芯却在你身上重新亮起——一缕清净之气渡你全身，暖得近乎滚烫。"]),
    choices: &[ChoiceDef { label: "（走向经匣石台）", sub: "经匣之择", cond: None, effects: &NO_EFF, route: Route::To("cj_30_box") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 隐藏 · 心魔（选择驱动）---- */
SceneDef {
    id: "cj_26_xinmo_door", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 心魔洞口"),
    mood: "danger", speaker: Some("内心之声"), voice: None,
    text: TextSpec::Static(&["说出禁书字缝里的真谛，石壁幕帘般裂开，寒气从裂隙里涌出来，冻得声音发颤。洞中涌出彻骨寒意，一个与你一模一样的人影在水镜里立起，它先开口，用的却是你的声音：「等你好久了。」"]),
    choices: &[
        ChoiceDef { label: "压下自己的倒影", sub: "进心魔战 · xinmo_unlock", cond: None, effects: &NO_EFF, route: Route::Dyn(start_xinmo) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_xinmo_round", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 心魔洞 · 激战"),
    mood: "danger", speaker: Some("心魔"), voice: None,
    text: TextSpec::Dyn(|st| {
        let f = st.fight.as_ref().map(|f| format!("心魔 HP {} / {}", f.hp.max(0), 90)).unwrap_or_else(|| "心魔 HP --".to_string());
        format!("{f}。它用你的脸笑着，出招却比你还狠。")
    }),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 26-38", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| xinmo_act(st, rng(26, 38), false)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 18-25", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| xinmo_act(st, rng(18, 25), false)) },
        ChoiceDef { label: "闭心守势", sub: "提升闪避", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| xinmo_act(st, 0, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_27_xinmo_win", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 心魔洞"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["倒影碎裂成水沫，溅了满壁，洞中只余一册破障心法，纸页还是温的。你将它收入怀中，那道裂隙悄然合拢，像一面镜子缓缓合上眼。"]),
    choices: &[ChoiceDef { label: "（回秘籍塔顶）", sub: "xinmo_down", cond: None,
        effects: &[Eff::MarkPoint("cj_p_l3_3")], route: Route::To("cj_19_arrive_tower") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结局分支 · 经匣之择（禁书取舍） ================= */
SceneDef {
    id: "cj_30_box", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 经匣石台"),
    mood: "mystery", speaker: Some("守阁僧残响"), voice: Some("vo_cj_box"),
    text: TextSpec::Static(&[
        "塔顶石台中央，那部被铁页封着的禁书静静躺着，封皮爬满经年守护的墨印，一层叠一层，像一圈圈年轮。风从塔顶灌进来，纸页在铁页下轻轻动了一下。",
        "守阁僧枯哑的声音自风中传来，像从很远的地方：「绝学一念，可遁入势——你当真，要翻开它？」",
    ]),
    choices: &[
        ChoiceDef { label: "研读禁书", sub: "+150 点 · Item 绝学真解 · 求索之路", cond: None,
            effects: &[Eff::SetFlag("cj_book_read"), Eff::AddItem("it_juexue_zhen"), Eff::Points(150),
                Eff::SetFlag("cj_book_choice")], route: Route::To("cj_31_exit") },
        ChoiceDef { label: "以铁页重封", sub: "+100 点 · Item 绝学守经录 · 守经之路", cond: None,
            effects: &[Eff::SetFlag("cj_book_seal"), Eff::AddItem("it_juexue_shou"), Eff::Points(100),
                Eff::SetFlag("cj_book_choice")], route: Route::To("cj_31_exit") },
        ChoiceDef { label: "焚毁禁书", sub: "+150 点 · 防火患再启 · 破妄之路", cond: None,
            effects: &[Eff::SetFlag("cj_book_burn"), Eff::Points(150), Eff::SetFlag("cj_book_choice")], route: Route::To("cj_31_exit") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_31_exit", bg: Some("cangjingge_bg.png"), loc: Some("L3 · 撤离阵"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        match (st.flag("cj_book_read"), st.flag("cj_book_seal"), st.flag("cj_book_burn")) {
            (true, _, _) => "绝学真解化于你掌心，塔顶诸经卷如朝圣般垂落，纸页哗啦一声全数展开，像在向你行礼。「真正的绝学，藏在你翻书之前。」你踏入撤离阵，光柱吞没了你，也吞没了你翻开的这一页。".to_string(),
            (_, true, _) => "铁页重封，墨印再合——守经人最后的执念亲眼看着自己守护之物归于沉静，嘴角动了一下，像笑。你踏入撤离阵，光柱温和地笼住你，像一只闭拢的手掌。".to_string(),
            _ => "火苗吞尽那一册铁页经书，铁页在火里蜷曲、发红，塔顶纸灰如雪。守阁僧的残响自言自语：「好……好……」像终于卸下了一生的担子。你踏入撤离阵，光柱吞没了你。".to_string(),
        }
    }),
    choices: &[ChoiceDef { label: "（踏入撤离阵 · 结算）", sub: "sp_grade 结算 · 回主神空间", cond: None,
        effects: &NO_EFF, route: Route::Dyn(route_exit_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "cj_32_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_cj_settle"), death: None,
        card: |st| crate::state::Card {
            title: "藏 经 阁 · 破 卷".into(), good: true,
            body_html: format!(
                "<p>塔顶经匣已决，主神光柱笼住你的身形，你带着经阁决议踏出撤离阵——那一页翻没翻，都算翻过了。</p>\
                 <p style='color:#9a958a'>绝学之争：残卷 {} / 书灵已平 / 禁书之择已作。</p>\
                 <table class='statTable'>\
                 <tr><td>存活点数</td><td>{}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 <tr><td>主神光柱</td><td>「刚翻过的那一页，是修行也是因果。」</td></tr>\
                 </table>",
                if st.flag("cj_miju_done") { "3/3" } else { "未完" }, st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ================= 死亡档案（复活扣 300/回主神）================= */
SceneDef {
    id: "cj_40_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("书页之下", "在藏经阁的纸傀与武僧合击下倒下")), card: |_st| crate::state::Card {
            title: "书 页 之 下".into(), good: false,
            body_html: r#"<p>千万书页此刻都成了你的墓碑，墨香裹住你冷却的躯壳，纸灰一层层落在你身上，像有人替你盖了一床书的被子。</p>
<p style='color:#ff8a8a'>【死亡档案 · 书页之下】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "cj_40_death_shouge", bg: None, loc: None, mood: "danger", speaker: None, voice: Some("vo_cj_death_shouge"),
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("塔顶残页", "入魔守阁僧以纸链锁颈，你未解得其执念")), card: |_st| crate::state::Card {
            title: "塔 顶 残 页".into(), good: false,
            body_html: r#"<p>纸链从塔顶缠下来，锁住你的咽喉，越收越紧。「你……终究也是想翻开它的人。」守阁僧的最后一声，混着经页哗然的寂静，那部禁书在石台上，始终没有被人翻开。</p>
<p style='color:#ff8a8a'>【死亡档案 · 塔顶残页】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "cj_40_death_xinmo", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("洞中倒影", "心魔以你的形状取你性命")), card: |_st| crate::state::Card {
            title: "洞 中 倒 影".into(), good: false,
            body_html: r#"<p>水镜里与你有同样容颜的影子，在你最熟悉的一招里，给了你最深刻的一拳——它比你先出招，因为你心里那点迟疑，它全看见了。</p>
<p style='color:#ff8a8a'>【死亡档案 · 洞中倒影】</p>
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
/// 残卷收集：三卷（a/b/c）齐 → it_miju_full 真解批注（守经人线索推进）。返回一楼 hub。
fn route_miju_collect(st: &mut GameState) -> String {
    if inv(st, "it_miju_a") && inv(st, "it_miju_b") && inv(st, "it_miju_c") {
        crate::world::add_item(st, "it_miju_full");
        st.set_flag("cj_miju_done");
    }
    "cj_10_arrive_floor1".to_string()
}

/// 守经人指引：集结三卷（it_miju_full）→ keeper_clue_2（开 G2 登二楼）
fn route_keeper_clue(st: &mut GameState) -> String {
    if inv(st, "it_miju_full") {
        st.set_flag("keeper_clue_1");
        st.set_flag("keeper_clue_2");
    }
    "cj_10_arrive_floor1".to_string()
}

/// 绝学领悟：记下密室内心法后，可破心魔洞（xinmo_unlock）
fn route_juexue_gain(st: &mut GameState) -> String {
    st.set_flag("xinmo_unlock");
    "cj_19_arrive_tower".to_string()
}

/// 撤离结算：确保 sp_grade=D（BOSS 已写；此处兜底）→ 卡片
fn route_exit_settle(st: &mut GameState) -> String {
    if st.sp_grade.is_none() {
        st.sp_grade = Some('D');
    }
    "cj_32_card".to_string()
}
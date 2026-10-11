//! 《异位面 · 倒影界》开放探索副本——全部剧情场景与战斗配置。
//! 设计取向：「世界展示向」。没有幕后黑手、没有真相要揭、没有阴谋要拆。
//! 玩家是探索者，自由漫游一个物理法则异常、景色奇诡的平行世界；调查点=奇观（景色+可选小互动，不推进任何阴谋），
//! 敌人=原生异兽（只是这个世界的居民），BOSS=界域守护兽（可选战或友好交流，掉落异界标本）。
//! 结局开放（继续漫游/带走标本/就地安歇看星），无对错无真相；sp_grade 按探索度给（D~/C）。
//! 钩子：「这里没有故事，只有风景。你来过了，就是了。」
//!
//! 本文件是全新新增文件，只导出静态数据（HEZI_SCENES / hezi_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/HEZI_impl_log.md ★外部依赖）。
//! 场景 id 全 `hz_` 前缀；fight id 全 `hz_` 前缀。
//!
//! 界域守护兽 · 镜潮兽采用「选择驱动遭遇链」落地（参考 scenes_mojiao 血池模式）：
//!   温和态 →（若被激怒）→ 怒态（镜潮翻涌）。也可在开场即选「友好交流」跳过战斗拿到标本。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   F1 倒映平原 bg hz_bg_plain    （现用 img_laser.png 占位）
//!   F2 荧光石林 bg hz_bg_forest   （现用 img_redqueen.png 占位）
//!   F3 倒悬星海 bg hz_bg_stars    （现用 img_zhuyuan_book.png 占位）
//!   界域守护兽 bg hz_bg_guardian  （现用 img_laser.png 占位）
//! 敌人立绘复用：hunter/mech→界域守护兽·镜潮兽、zombie→荧光兽；新美术由主 agent 统一生图替换。

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

/// 已探索的奇观数量（用于按探索度结算 sp_grade）
fn wonder_count(st: &GameState) -> i32 {
    ["hz_w_river", "hz_w_grass", "hz_w_rain", "hz_w_pebble",
     "hz_w_forest", "hz_w_echo", "hz_w_flowers", "hz_w_lake",
     "hz_w_stars", "hz_w_gate", "hz_w_giant"]
        .iter().filter(|k| st.flag(k)).count() as i32
}

/* =====================================================================
   具名条件谓词
   ===================================================================== */
fn cond_has_prism(st: &GameState) -> bool { inv(st, "it_yijie_crystal") }
fn cond_has_specimen(st: &GameState) -> bool { inv(st, "it_yijie_specimen") }

/* =====================================================================
   战斗配置表（id 全部 hz_ 前缀）。
   原生异兽多为温顺居民；fights 仅「遭遇」用，风味偏观测，非阴谋产物。
   ===================================================================== */
fn hz_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn hz_win_l1(_st: &GameState) -> String { "hz_01".to_string() }
fn hz_win_f2(_st: &GameState) -> String { "hz_10_arrive".to_string() }
fn hz_win_f3(_st: &GameState) -> String { "hz_20_arrive".to_string() }
fn hz_win_guardian(_st: &GameState) -> String { "hz_33_guardian_down".to_string() }

pub fn hezi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("hz_upstream_fish", FightCfg {
            name: "逆流鱼群", hp: 30, dmg: (5, 9), reward: 10, reward_why: "与逆流鱼群擦肩而过，得些许闪光鳞",
            intro: "一群发光的鱼正沿着河向上逆流游去，鳞片在水里拖出一道道细长的光痕，掠过你身边时，带起一阵细碎的水花。它们没有恶意，只是这片世界的居民，赶着一场没人知道的水路。",
            rage_at: None, rage_text: "", on_rage: hz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: hz_win_l1, death: "hz_50_death",
        }),
        ("hz_glow_deer", FightCfg {
            name: "荧光水鹿", hp: 34, dmg: (6, 11), reward: 12, reward_why: "目送荧光水鹿远去",
            intro: "一头角上开着花的水鹿立在河边，鹿角间的花在风里轻轻点头，静静看着你。你原以为它会逃跑，它却只是低了低头，继续饮水，水面上漾开一圈圈细纹，像在说：不必紧张。",
            rage_at: None, rage_text: "", on_rage: hz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: hz_win_l1, death: "hz_50_death",
        }),
        ("hz_gentle_herd", FightCfg {
            name: "温顺兽群", hp: 28, dmg: (4, 8), reward: 10, reward_why: "穿过温顺兽群",
            intro: "一群皮毛如草叶般的兽排成一列，托着低垂的云缓慢走过原野，蹄声像雨点落在草上。它们围着你转了一圈，又各自散去，像完成一次无声的问候礼。",
            rage_at: None, rage_text: "", on_rage: hz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: hz_win_l1, death: "hz_50_death",
        }),
        ("hz_crystal_owl", FightCfg {
            name: "石林枭", hp: 56, dmg: (10, 16), reward: 22, reward_why: "与石林枭共鸣一次",
            intro: "一只通体晶亮的枭从石林间无声掠过，翅膀切开两道光影，在月光与荧光交汇处停了停，歪头冲你眨眨眼——随即振翅，卷起一阵发光的雪，碎屑在空气里慢慢往下落，像一场很小的流星雨。",
            rage_at: Some(26), rage_text: "石林枭被惊扰，发出清越的啸鸣，荧光随之炸裂！", on_rage: hz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: hz_win_f2, death: "hz_50_death",
        }),
        ("hz_glow_stag", FightCfg {
            name: "荧光巨鹿", hp: 62, dmg: (11, 17), reward: 25, reward_why: "与荧光巨鹿碰了碰角",
            intro: "一头披着星光的巨鹿从石林深处踱出，每一步都让脚下的荧光散开又聚拢，鹿角间悬着一串小月亮，随着步伐轻轻晃。它低头，与你碰了碰额角，角上的光凉凉地触了你一下，仿佛打个招呼。",
            rage_at: Some(30), rage_text: "巨鹿受惊，蹄下涌起荧光浪，步法陡然凌厉！", on_rage: hz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: hz_win_f2, death: "hz_50_death",
        }),
        ("hz_star_whale", FightCfg {
            name: "游弋星鲸", hp: 70, dmg: (12, 19), reward: 30, reward_why: "伴随星鲸漫游一会",
            intro: "一头以星为鳞的巨鲸无声地游过倒悬的星海，身体过处，星辰像被犁开一道银色的浪，尾鳍扫过带起一阵银河碎屑。它看见了独行的你，缓缓放慢了速度，像认出了什么。",
            rage_at: Some(34), rage_text: "星鲸感知到你的紧张，发出一声低鸣，声浪在星海间激荡！", on_rage: hz_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: hz_win_f3, death: "hz_50_death",
        }),
        ("hz_guardian", FightCfg {
            name: "界域守护兽 · 镜潮兽", hp: 200, dmg: (15, 24), reward: 200, reward_why: "与界域守护兽 · 镜潮兽和解",
            intro: "倒悬星海的最深处，一头通体如镜的巨兽缓缓浮出，镜面皮肤上流转着星云与你们的全部倒影——它不是守卫，更像这个世界的「看门人」，好奇地端详着这位不请自来的漫游者，镜面里映出的目光温和而认真。",
            rage_at: Some(100), rage_text: "镜潮兽翻涌成怒潮，镜面闪过冷光——你若无意相安，它便认真起来！",
            on_rage: hz_rage_none,
            finisher_if: |st, _| st.fight.as_ref().map(|f| f.raged).unwrap_or(false) && inv(st, "it_yijie_crystal"),
            finisher_name: |_| "以棱光石净化镜潮".to_string(),
            finisher_desc: |_| "你举起棱光石，镜潮兽映出的怒意在棱光里一点点平息，镜面重新澄澈如初。".to_string(),
            win: hz_win_guardian, death: "hz_50_death",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn hz_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    hezi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   界域守护兽 · 镜潮兽（选择驱动遭遇；可友好交流or短暂交手）
   血量存 st.fight（hz_30_guardian 的 Route::Dyn 初始化，引用 hz_guardian 的 FightCfg）。
   温和态（不还手，只反射）→（仅当选择交战时）→ 怒态（镜潮翻涌）。
   ===================================================================== */
fn start_guardian(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("hz_guardian") {
            st.fight = Some(crate::power::scaled_fight("hz_guardian", cfg, st, vec![]));
            st.set_flag("hz_guardian_start");
        }
    }
    "hz_31_round".to_string()
}

/// 友好交流成功：不战斗，直接得标本（界域守护兽赠予）
fn guardian_friendly(st: &mut GameState) -> String {
    crate::world::add_item(st, "it_yijie_specimen");
    st.points += 150;
    st.set_flag("hz_guardian_peace");
    "hz_32_guardian_peace".to_string()
}

/// 交手胜利：得标本 + 点数
fn guardian_win(st: &mut GameState) -> String {
    crate::world::add_item(st, "it_yijie_specimen");
    st.points += 200;
    st.set_flag("hz_guardian_win");
    "hz_33_guardian_down".to_string()
}

fn guardian_dead() -> String { "hz_50_death".to_string() }

/// 一个"回"（仅当选择交战时进入）：攻击镜潮兽。
fn guardian_act(st: &mut GameState, dmg: i32, guard: bool, peaceful: bool) -> String {
    if peaceful {
        return guardian_friendly(st);
    }
    if st.fight.as_ref().map(|f| f.hp <= 100 && !f.raged).unwrap_or(false) {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return guardian_win(st);
    }
    // 温和态镜潮兽不主动伤人，只反射；怒态才反击
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged { rng(20, 28) } else { rng(8, 14) };
    if raged {
        let roll: f64 = rand::thread_rng().gen();
        if roll >= 0.3 {
            st.hp = (st.hp - raw).max(0);
        }
    }
    if st.hp <= 0 {
        return guardian_dead();
    }
    "hz_31_round".to_string()
}

/* =====================================================================
   剧情场景（id 全部 hz_ 前缀）
   ===================================================================== */
pub static HEZI_SCENES: &[SceneDef] = &[

    /* ================= 序 · 世界展示 ================= */
    SceneDef {
        id: "hz_00", bg: Some("hezi_bg.png"), loc: Some("倒影界 · 入境口"),
        mood: "wonder", speaker: Some("旁白"), voice: Some("vo_hz_open"),
        text: TextSpec::Static(&[
            "一阵失重感过后，你踏入一个不属于任何已知经纬的地方。天空倒映在脚下，河流向天上倒流，远处的岛悬在半空，缓缓转动着，像一颗睡着的星球。",
            "这里没有熟悉的光，却也没有恶意——风是暖的，草是软的，脚下的「天空」里偶尔有发光的鱼游过。你只是到了一个风景奇诡、却安宁的平行世界。",
            "<em>「这里没有故事，只有风景。你来过了，就是了。」</em>",
        ]),
        choices: &[
            ChoiceDef { label: "慢些走入", sub: "开始漫游倒影界", cond: None, effects: &NO_EFF, route: Route::To("hz_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ---- F1 hub · 倒映平原 ---- */
    SceneDef {
        id: "hz_01", bg: Some("hezi_bg.png"), loc: Some("F1 · 倒映平原 · 逆流之河"),
        mood: "wonder", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_has_prism(st) {
                "逆流之河在脚下静静向天上流去，浮岛的草叶垂向地面。你已拾到棱光石，被滤暖的光让视野清亮——石林的强光不再是阻碍。".to_string()
            } else {
                "原野开阔，草从倒悬的浮岛往下长，一直垂到能蹭过你的发梢。一条河沿着你的脚倒流向天空，水声像远处有人在轻轻哼歌，唱着听不懂的调子。你看到几处值得停下来看的地方。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "逆流之河", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_01_river") },
            ChoiceDef { label: "倒悬草地", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_02_grass") },
            ChoiceDef { label: "静止的雨", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_03_rain") },
            ChoiceDef { label: "棱光石滩", sub: "奇观 · 得棱光石", cond: None, effects: &NO_EFF, route: Route::To("hz_04_pebble") },
            ChoiceDef { label: "与河畔兽群擦肩", sub: "原生异兽 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("hz_01_fight") },
            ChoiceDef { label: "走向石林（需棱光石）", sub: "微光幕 G1 → F2", cond: Some(cond_has_prism), effects: &NO_EFF, route: Route::To("hz_04_gate") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_01_river", bg: Some("hezi_bg.png"), loc: Some("F1 · 逆流之河"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["你蹲在河岸，把手探进水流——水没有向下流，而是一点点往天上淌去，带走你指尖的温度，像被风轻轻提走了。水面映着你的脸，又映着更上面的天空，两层倒影叠在一起。你静静看着，很久很久，没有想离开。"]),
        choices: &[ChoiceDef { label: "捞起一点闪亮的水光", sub: "奇观记于册 · 无后续", cond: None,
            effects: &[Eff::SetFlag("hz_w_river"), Eff::Points(10), Eff::MarkPoint("hz_pl1_1")], route: Route::To("hz_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_02_grass", bg: Some("hezi_bg.png"), loc: Some("F1 · 倒悬草地"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["头顶的浮岛上，草叶朝下生长，垂到你伸手可及的地方，像一片倒挂的草原。你碰了碰一株，它轻轻卷了一下你的指尖，像回应又像自在的摆动。夜里它们会发出细细的荧光，你没等到夜，却也没觉得遗憾。"]),
        choices: &[ChoiceDef { label: "摘一片叶留作凭记", sub: "奇观记于册", cond: None,
            effects: &[Eff::SetFlag("hz_w_grass"), Eff::Points(10), Eff::MarkPoint("hz_pl1_2")], route: Route::To("hz_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_03_rain", bg: Some("hezi_bg.png"), loc: Some("F1 · 静止的雨"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["雨点悬在半空，像一根根透明的线，把天空和地面轻轻缝在一起，缝得整整齐齐。你伸出手，握住一粒停住的雨，凉凉的，像握住一粒小小的星；松开，它仍悬在那里，不急落。这个世界连雨都不赶时间。"]),
        choices: &[ChoiceDef { label: "穿过静止的雨幕", sub: "奇观记于册", cond: None,
            effects: &[Eff::SetFlag("hz_w_rain"), Eff::Points(10), Eff::MarkPoint("hz_pl1_3")], route: Route::To("hz_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_04_pebble", bg: Some("hezi_bg.png"), loc: Some("F1 · 棱光石滩"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["河滩上铺着一片圆润的石子，被水光浸得温润，像撒了一地的月亮。其中一颗尤其特别——握在手里，能滤去刺目的光，把强光柔成暖色。你想，带它也许在此后用得着，于是把它收进怀里。"]),
        choices: &[ChoiceDef { label: "拾起棱光石", sub: "Item it_yijie_crystal · 可滤光 · 开微光幕", cond: None,
            effects: &[Eff::AddItem("it_yijie_crystal"), Eff::SetFlag("hz_w_pebble"), Eff::MarkPoint("hz_pl1_4")], route: Route::To("hz_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_01_fight", bg: Some("hezi_bg.png"), loc: Some("F1 · 平原 · 擦肩"),
        mood: "danger", speaker: None, voice: Some("vo_hz_herd"),
        text: TextSpec::Static(&["你穿过兽群，几只发光的兽抬起头看了看你，又低下头，继续啃食它们够得到的草叶。也许是热情的问候，也许只是好奇——你回应地低了低头，继续你的漫游，身后的蹄声渐渐远了。（擦肩而过）"]),
        choices: &[], fight_id: Some("hz_gentle_herd"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_04_gate", bg: Some("hezi_bg.png"), loc: Some("F1 · 微光幕（G1 已开）"),
        mood: "wonder", speaker: None, voice: None,
        text: TextSpec::Static(&["棱光石滤开强光，石林在你眼前露出一角——一片发光的森林，树与树之间流动着光，像一条会呼吸的水晶河。你踏过光幕，光在身侧合拢，身后的平原被染成回忆的颜色。"]),
        choices: &[ChoiceDef { label: "（踏入荧光石林）", sub: "pt_hz_1 单向 · 进 F2", cond: None, effects: &NO_EFF, route: Route::To("hz_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F2 · 荧光石林 ================= */
    SceneDef {
        id: "hz_10_arrive", bg: Some("hezi_bg.png"), loc: Some("F2 · 荧光石林 · 水晶回廊"),
        mood: "wonder", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["发光的石头在林间轻轻脉动，像大片沉睡的心跳，一下一下，很有耐心。水晶在风里碰出细碎的音，叮叮当当，你不懂那是什么语言，却莫名觉得安心。这里只是深一点的风景，仍没有人要你做什么，也没有人催你。"]),
        choices: &[
            ChoiceDef { label: "荧光石林", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_11_forest") },
            ChoiceDef { label: "回音谷", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_12_echo") },
            ChoiceDef { label: "结晶花丛", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_13_flowers") },
            ChoiceDef { label: "会说话的湖", sub: "奇观 · 微互动", cond: None, effects: &NO_EFF, route: Route::To("hz_14_lake") },
            ChoiceDef { label: "与石林兽共鸣", sub: "原生异兽 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("hz_10_fight") },
            ChoiceDef { label: "走向倒悬星海", sub: "pt_hz_2 单向 · 进 F3", cond: None, effects: &NO_EFF, route: Route::To("hz_20_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_11_forest", bg: Some("hezi_bg.png"), loc: Some("F2 · 荧光石林"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["每一块石头都像一团凝固极光的影子，呼吸般明灭，绿、蓝、淡紫的光轮流漫过你的手背。你靠着一块石坐下，它的光把你的轮廓晕成暖色，连影子都变得温柔。树的脉络是发光的纹路，像血管一样安静地通向更深处。"]),
        choices: &[ChoiceDef { label: "跟着石林的光走一圈", sub: "奇观记于册", cond: None,
            effects: &[Eff::SetFlag("hz_w_forest"), Eff::Points(10), Eff::MarkPoint("hz_pl2_1")], route: Route::To("hz_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_12_echo", bg: Some("hezi_bg.png"), loc: Some("F2 · 回音谷"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["你站在谷口喊了一声，回声没有学你的话——它换了个更柔软的声音回应你，像另一个人也在同一条山谷里走着，走得很慢。你多喊了几声，回声也陪你玩了一会儿，最后那一声尾音拖得很长，像是山谷舍不得收。"]),
        choices: &[ChoiceDef { label: "与回声聊几句", sub: "奇观记于册 · 无意义的小快乐", cond: None,
            effects: &[Eff::SetFlag("hz_w_echo"), Eff::Points(10), Eff::MarkPoint("hz_pl2_2")], route: Route::To("hz_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_13_flowers", bg: Some("hezi_bg.png"), loc: Some("F2 · 结晶花丛"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["水晶凝成的花开了一整坡，透明的花瓣里封着一小团光，风吹过时，花瓣相碰发出清脆的颤音，像一整坡在轻轻合唱。你没有踩进去，只是站在花丛边，听了一整段风的曲子，听到天光又换了一种颜色。"]),
        choices: &[ChoiceDef { label: "在花旁站到风停", sub: "奇观记于册", cond: None,
            effects: &[Eff::SetFlag("hz_w_flowers"), Eff::Points(10), Eff::MarkPoint("hz_pl2_3")], route: Route::To("hz_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_14_lake", bg: Some("hezi_bg.png"), loc: Some("F2 · 会说话的湖"),
        mood: "cold", speaker: Some("湖面"), voice: None,
        text: TextSpec::Static(&["湖面静止得如一整块银镜，把天空和你的影子都收得整整齐齐。你俯身，湖水泛起涟漪，慢慢拼出两个字：『你好』。没有威胁，也没有恳求——就好像这湖只是太久没见过人，想打个招呼，又怕吓着你，说完又恢复了平静。"]),
        choices: &[ChoiceDef { label: "对湖也说一声你好", sub: "奇观 · 友好回应", cond: None,
            effects: &[Eff::SetFlag("hz_w_lake"), Eff::Points(15), Eff::MarkPoint("hz_pl2_4")], route: Route::To("hz_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_10_fight", bg: Some("hezi_bg.png"), loc: Some("F2 · 石林 · 共鸣"),
        mood: "danger", speaker: None, voice: Some("vo_hz_owl"),
        text: TextSpec::Static(&["一只晶亮的枭落到你面前枝头，收拢翅膀时抖落几点光尘，冲你眨眨眼。你与它静静对视了一瞬，它才振翅飞掠而去，留下一串发光的碎屑，像有人替你撒了一把星星。（擦肩）"]),
        choices: &[], fight_id: Some("hz_crystal_owl"), video: None, cine_label: None, overlay: None,
    },

    /* ================= F3 · 倒悬星海 ================= */
    SceneDef {
        id: "hz_20_arrive", bg: Some("hezi_bg.png"), loc: Some("F3 · 倒悬星海 · 界域阙"),
        mood: "wonder", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_has_specimen(st) {
                "倒悬的星海铺在脚下，像踩在一整片天空上。你与界域守护兽镜潮兽有过一次友好的相遇了，此刻的星海显得格外温柔。".to_string()
            } else {
                "脚下的地面消失了，取而代之的是一整片倒悬的星空——你走在星的上面，每颗星都隔着一步的距离。远处一道没有门的巨拱「界域阙」矗立着，像世界某个尽头的地标，安静地等路过的人，也安静地送走过路的人。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "倒悬星海", sub: "奇观", cond: None, effects: &NO_EFF, route: Route::To("hz_21_stars") },
            ChoiceDef { label: "界域阙", sub: "奇观 · 世界尽头", cond: None, effects: &NO_EFF, route: Route::To("hz_22_gate") },
            ChoiceDef { label: "沉睡的巨人", sub: "奇观 · 微互动", cond: None, effects: &NO_EFF, route: Route::To("hz_23_giant") },
            ChoiceDef { label: "听巨鲸低鸣", sub: "角色 · 漫游伙伴", cond: None, effects: &NO_EFF, route: Route::To("hz_npc_whale") },
            ChoiceDef { label: "与星鲸伴游", sub: "原生异兽 · 擦肩", cond: None, effects: &NO_EFF, route: Route::To("hz_20_fight") },
            ChoiceDef { label: "面向界域守护兽", sub: "界域守护兽 · 交流或交手", cond: None, effects: &NO_EFF, route: Route::To("hz_30_guardian") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_21_stars", bg: Some("hezi_bg.png"), loc: Some("F3 · 倒悬星海"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["你在星上走，每踩一步，脚下的星云都像被轻轻踏开又合拢，溅起细碎的光点。低头看，是另一片更深的星海；抬头看，也是。你分不清上下，也不再想分——反正无论朝哪个方向躺下，都会有星光托住你。"]),
        choices: &[ChoiceDef { label: "在星海里躺一会儿", sub: "奇观记于册 · 安歇", cond: None,
            effects: &[Eff::SetFlag("hz_w_stars"), Eff::Points(10), Eff::MarkPoint("hz_pl3_1")], route: Route::To("hz_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_22_gate", bg: Some("hezi_bg.png"), loc: Some("F3 · 界域阙"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["一座没有门的巨拱立在星海尽头，拱内只是一片会流动的光，像一条静静站立的河。没有门也没有锁——它不拦人，也没有人真正走出过。你站在拱下，风从两侧掠过，带起衣角，世界在你身后依旧安稳，仿佛这一站，就是它想让你看见的全部。"]),
        choices: &[ChoiceDef { label: "在阙下站一会", sub: "奇观记于册 · 无答案", cond: None,
            effects: &[Eff::SetFlag("hz_w_gate"), Eff::Points(10), Eff::MarkPoint("hz_pl3_2")], route: Route::To("hz_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_23_giant", bg: Some("hezi_bg.png"), loc: Some("F3 · 沉睡的巨人"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["远处连绵的「山脊」其实是一头熟睡巨兽的脊背，随呼吸缓缓起伏，每一次起伏都带起一整片草原的轻颤。它庞大到你不确定它有没有「醒来」这个概念——也许对它来说，睡与醒只是两种呼吸。你放轻脚步，没有惊动它。"]),
        choices: &[ChoiceDef { label: "贴着它的呼吸声看了看夜空", sub: "奇观 · 轻触也温柔", cond: None,
            effects: &[Eff::SetFlag("hz_w_giant"), Eff::Points(10), Eff::MarkPoint("hz_pl3_3")], route: Route::To("hz_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_npc_whale", bg: Some("hezi_bg.png"), loc: Some("F3 · 巨鲸低鸣"),
        mood: "cold", speaker: Some("星鲸"), voice: None,
        text: TextSpec::Static(&["一头以星为鳞的巨鲸靠了过来，用巨大的眼瞳望着你，眼瞳里倒映着整片星海和你。你听到一声极低的鸣叫，低得像是从很深很远的地方传来——那是它对你说的唯一一句话。而后它缓缓游开，尾鳍一摆，游进漫天的星里。你连它叫什么都不知道，却觉得同行过一段路。"]),
        choices: &[ChoiceDef { label: "目送巨鲸游远", sub: "漫游伙伴 · 故人未知", cond: None,
            effects: &[Eff::Points(15), Eff::SetFlag("hz_whale_friend")], route: Route::To("hz_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_20_fight", bg: Some("hezi_bg.png"), loc: Some("F3 · 星海 · 伴游"),
        mood: "danger", speaker: None, voice: Some("vo_hz_whale"),
        text: TextSpec::Static(&["你与一头幼星鲸并排游了一小段，它的鳞片擦过你的手背，凉丝丝地发着光。它绕着你转圈，像在确认你也是个会发光的生物——然后放缓了速度，等你先走，像一位彬彬有礼的小旅伴。（擦肩）"]),
        choices: &[], fight_id: Some("hz_star_whale"), video: None, cine_label: None, overlay: None,
    },

    /* ================= BOSS · 界域守护兽 · 镜潮兽（可交流或交手） ================= */
    SceneDef {
        id: "hz_30_guardian", bg: Some("hezi_bg.png"), loc: Some("F3 · 界域守护兽 · 镜潮兽"),
        mood: "wonder", speaker: Some("界域守护兽 · 镜潮兽"), voice: Some("vo_hz_guardian"),
        text: TextSpec::Static(&[
            "倒悬星海的最深处，镜潮兽缓缓从星云中浮起，通体如一面巨大的活镜，镜面缓缓流转，映出你这个小小漫游者的倒影，也映出你身后漫天的星。它没有发出声音，只是静静地看你，像在端详一件漂到门口的旧物。",
            "你可以选择与它和平地打个招呼，或，以一场认真的对视开始。",
        ]),
        choices: &[
            ChoiceDef { label: "朝它挥手问好", sub: "友好交流 · 和平得标本", cond: None, effects: &NO_EFF, route: Route::To("hz_31_friendly") },
            ChoiceDef { label: "与它认真对视", sub: "短暂交手 · 以棱光石和解", cond: None, effects: &NO_EFF, route: Route::To("hz_31_battle") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_31_friendly", bg: Some("hezi_bg.png"), loc: Some("F3 · 镜潮兽 · 握手"),
        mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["镜潮兽映出一个同样挥手的倒影，镜面一荡，误会一触即散。它从镜面深处托出一枚泛着星光的标本，轻轻推到你的面前，像推出一朵水花——像一份慷慨的地主送客礼，又像一份临别的赠言。"]),
        choices: &[ChoiceDef { label: "收下异界标本", sub: "Item it_yijie_specimen · 守护兽礼赠", cond: None,
            effects: &NO_EFF, route: Route::Dyn(guardian_friendly) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_31_battle", bg: Some("hezi_bg.png"), loc: Some("F3 · 直面镜潮兽"),
        mood: "danger", speaker: Some("镜潮兽"), voice: None,
        text: TextSpec::Static(&["你一凝神，镜潮兽便认真起来。镜面泛起涟漪，温和的注视化作一场认真的相搏，涟漪一圈圈荡开，整片星海都在为这场切磋让路——它不会伤你性命，但想看看你这个外来者有几分成色。（交手）"]),
        choices: &[ChoiceDef { label: "【切磋一场】", sub: "进入与镜潮兽的短暂交手", cond: None, effects: &NO_EFF, route: Route::Dyn(start_guardian) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_31_round", bg: Some("hezi_bg.png"), loc: Some("F3 · 镜潮兽 · 切磋"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            let f = st.fight.as_ref().map(|f| format!("镜潮兽 HP {} / {}", f.hp.max(0), 200)).unwrap_or_else(|| "HP --".to_string());
            let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
                "——镜潮翻涌！认真起来了——"
            } else {
                "——温和态 · 镜面反射你的一招一式——"
            };
            format!("{f}。{mode}")
        }),
        choices: &[
            ChoiceDef { label: "重击（认真出招）", sub: "伤害 32-44", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| guardian_act(st, rng(32, 44), false, false)) },
            ChoiceDef { label: "轻触镜面（收手）", sub: "立刻转友好 · 得标本", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| guardian_act(st, 0, false, true)) },
            ChoiceDef { label: "【以棱光石净化镜潮】", sub: "需怒态 + 棱光石 · 40 固伤和解", cond: Some(cond_has_prism),
                effects: &NO_EFF, route: Route::Dyn(|st| guardian_act(st, 40, false, false)) },
            ChoiceDef { label: "侧身回避", sub: "提升闪避", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| guardian_act(st, 0, true, false)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_32_guardian_peace", bg: Some("hezi_bg.png"), loc: Some("F3 · 与镜潮兽和解"),
        mood: "calm", speaker: None, voice: Some("vo_hz_guardian_peace"),
        text: TextSpec::Static(&["镜潮兽映出你放松的眉眼，镜面一层层恢复了澄澈，像雨后的水面。它把一枚星光标本轻轻放进你的掌心，触感微凉——没有阴谋，没有条件，就当作一位老朋友送你的一枚旅途纪念，纪念你来过这片星海。"]),
        choices: &[ChoiceDef { label: "（转身，继续你的漫游或收尾）", sub: "开放结局抉择", cond: None, effects: &NO_EFF, route: Route::To("hz_40_ending") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_33_guardian_down", bg: Some("hezi_bg.png"), loc: Some("F3 · 切磋毕"),
        mood: "calm", speaker: None, voice: Some("vo_hz_guardian_down"),
        text: TextSpec::Static(&["镜潮平息，涟漪一圈圈收拢，镜面上的倒影与你相视而笑——这一场认真的相搏，反而让你们像真的认识了。镜潮兽把一枚星光标本推向你的脚边，像完成一场仪式，然后缓缓沉回星海，沉入时还在镜面上留下一圈淡淡的、像是回礼的涟漪。"]),
        choices: &[ChoiceDef { label: "拾起异界标本", sub: "Item it_yijie_specimen · 切磋结谊", cond: None,
            effects: &[Eff::AddItem("it_yijie_specimen")], route: Route::To("hz_40_ending") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 开放结局（无对错无真相） ================= */
    SceneDef {
        id: "hz_40_ending", bg: Some("hezi_bg.png"), loc: Some("F3 · 倒悬星海 · 抉择"),
        mood: "mystery", speaker: Some("旁白"), voice: Some("vo_hz_ending"),
        text: TextSpec::Static(&[
            "星海在脚下漫延，风从四面八方来，又往四面八方去，你站在这一程的尽头。这里没有要揭的真相，没有要拆的阴谋——只是到该为自己这段漫游收个尾的时候了。",
            "你想怎么结束这段与倒影界的相遇？",
        ]),
        choices: &[
            ChoiceDef { label: "继续漫游", sub: "留在这片星海里，再走一段", cond: None,
                effects: &[Eff::SetFlag("hz_end_roam"), Eff::Points(20), Eff::SetFlag("hz_ending_done")],
                route: Route::To("hz_41_exit") },
            ChoiceDef { label: "带走标本", sub: "把这段异界装进口袋，起身离去", cond: None,
                effects: &[Eff::SetFlag("hz_end_take"), Eff::Points(30), Eff::SetFlag("hz_ending_done")],
                route: Route::To("hz_41_exit") },
            ChoiceDef { label: "就地安歇看星", sub: "躺下，什么都不想，看完这片星", cond: None,
                effects: &[Eff::SetFlag("hz_end_rest"), Eff::Points(25), Eff::SetFlag("hz_ending_done")],
                route: Route::To("hz_41_exit") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_41_exit", bg: Some("hezi_bg.png"), loc: Some("F3 · 星海 · 尾声"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            match (st.flag("hz_end_roam"), st.flag("hz_end_take"), st.flag("hz_end_rest")) {
                (true, _, _) => "你又往前走了很远，把这片倒影界又看了一遍，看逆流的河，看静止的雨，看每一块呼吸的石头。有些风景来过就好，不必抓住。直到风转暖，你才想起该回头了。".to_string(),
                (_, true, _) => "你把异界标本妥帖收好，像把一段异乡的晚风带回了家。你最后望了一眼倒悬的星海，星光在你眼底闪了很久，然后你转身离开。".to_string(),
                (_, _, true) => "你躺在那片星上，星光从脚下往上漫，漫过你，把你轻轻托住，像躺进一整片温柔里。你什么都没想，就那样看完了整片星。这一程，没有输赢，只有来过的痕迹。".to_string(),
                _ => "星海在你身后缓缓合拢，倒影界安静如初，好像从没有人来过。".to_string(),
            }
        }),
        choices: &[ChoiceDef { label: "（这一程，也到说再见的时候了）", sub: "按探索度结算", cond: None,
            effects: &NO_EFF, route: Route::Dyn(hz_route_exit_settle) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hz_42_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_hz_settle"), death: None,
            card: |st| crate::state::Card {
                title: "倒 影 界 · 一 程 风 景".into(), good: true,
                body_html: format!(
                    "<p>你离开了那个物理法则异常、却安宁得不像话的平行世界。没有真相被揭开，没有阴谋被拆穿——你只是去看过，也认真看过，把十一处风景都装进了记忆里。</p>\
                     <p style='color:#9a958a'>「这里没有故事，只有风景。你来过了，就是了。」</p>\
                     <table class='statTable'>\
                     <tr><td>勘探奇观</td><td>{} / 11 处</td></tr>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>探界评级</td><td style='color:#ffd76a'>{}</td></tr>\
                     </table>",
                    wonder_count(st),
                    st.points,
                    if wonder_count(st) >= 8 { "C 级（涉猎颇广）" } else { "D 级（惊鸿一瞥）" }
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 死亡档案（坠入星海/失足） ================= */
    SceneDef {
        id: "hz_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("倒影界 · 失足的漫游者", "在一片倒悬星海里落了空")), card: |_st| crate::state::Card {
                title: "星 海 落 空".into(), good: false,
                body_html: r#"<p>你在星海与山脊之间一脚踏空，身体向下／向上地飘远，星光从身边一颗颗掠过，最终落进一片深蓝的安宁里。倒影界依旧如初，风继续吹着草叶，雨继续悬在半空——只是少了一个来看风景的人。</p>
<p style='color:#ff8a8a'>【死亡档案 · 倒影界失足的漫游者】</p>
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
/// 尾声结算：按探索度写 sp_grade（D~/C）→ 卡片
fn hz_route_exit_settle(st: &mut GameState) -> String {
    let w = wonder_count(st);
    st.sp_grade = Some(if w >= 8 { 'C' } else { 'D' });
    "hz_42_card".to_string()
}


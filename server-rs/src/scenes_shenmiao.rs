//! 《死亡开端 · 旧神遗迹 · 沉没神殿》场景 + 战斗配置。
//!
//! 主题：一座沉在「倒流之海」底部的旧神遗迹。物理颠倒——水向天上去，地板长在天顶，
//! 阶梯往下通到更高处。旧神已死千万年，「眷属」仍在殿中行走，侍奉一座空神龛。
//! 钩子：「这里的水是倒着流的。」
//!
//! BOSS 旧神眷属 HP=200，采用「选择驱动 + 相位闪现」机制：既非原生随机战斗，也非纯剧情，
//! 而是每一回合由玩家选择如何应对它在实相/虚相之间的闪现——选对了手法，伤害落在实相，
//! 选错或防守，它继续漂浮。击败它，抵达神龛的真相结局。
//!
//! 本文件为全新新增文件，只导出静态数据（SHENMIAO_SCENES / shenmiao_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/shenmiao_impl_log.md ★外部依赖）。
//! 场景 id 全 `sm_` 前缀；fight id 全 `sm_` 前缀。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   通用小工具
   ===================================================================== */
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/// 已揭开「旧神真相」的里程碑计数（用于按探索度结算 sp_grade）
fn relic_count(st: &GameState) -> i32 {
    ["sm_true_vortex", "sm_true_pool", "sm_true_ceiling", "sm_true_eye", "sm_true_bones"]
        .iter().filter(|k| st.flag(k)).count() as i32
}

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   具名条件谓词
   ===================================================================== */
fn cond_has_reliquary(st: &GameState) -> bool { inv(st, "it_shenmiao_reliquary") }

/* =====================================================================
   战斗配置表（id 全部 sm_ 前缀）。原生敌为「殿中居民」，非阴谋哨兵。
   ===================================================================== */
fn sm_win_l1(_st: &GameState) -> String { "sm_10_f1".to_string() }
fn sm_win_l2(_st: &GameState) -> String { "sm_20_f2".to_string() }
fn sm_win_l3(_st: &GameState) -> String { "sm_30_f3".to_string() }
fn sm_win_spawn(_st: &GameState) -> String { "sm_36_win".to_string() }

pub fn shenmiao_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sm_current_shade", FightCfg {
            name: "逆流之影", hp: 34, dmg: (6, 10), reward: 14, reward_why: "驱散逆流之影",
            intro: "一道垂在涡心深处的影子，颜色比水更暗，逆着水流朝你爬来。它不叫嚷，只是贴着倒流的水面，缓缓逼近。那是许多年前溺亡在这片倒流里的「人」留下的一层壳。",
            rage_at: Some(16), rage_text: "逆流之影猛地回头，空洞的眼窝里涌出一整卷水流，逆撞向你！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sm_win_l1, death: "sm_50_death",
        }),
        ("sm_drowned_priest", FightCfg {
            name: "溺行祭司", hp: 48, dmg: (9, 15), reward: 20, reward_why: "让溺行祭司停下脚步",
            intro: "一个神色木然的祭司顶着圣坛的倒影从水中走来，手里的灯倒着举着，光却朝下照进水里。它张开口，吐出的不是祷词，而是气泡与泥沙。它早已不是活人，只是仍尽职地看守那道门。",
            rage_at: Some(22), rage_text: "祭司手里的灯猛地灌进一束向上的火，倒悬的经幡哗然翻卷！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sm_win_l1, death: "sm_50_death",
        }),
        ("sm_inverted_servant", FightCfg {
            name: "倒影侍从", hp: 56, dmg: (10, 16), reward: 24, reward_why: "看清倒影侍从的来路",
            intro: "一群半透明的侍从跪在颠倒回廊的天顶上，朝下俯身，像是要侍奉脚下的你。它们每动一下，地面的倒影就错开半拍——那不是侍从的影子，而是它们的世界。",
            rage_at: Some(26), rage_text: "倒影侍从全部直起身，天顶的圣坛轰然下压了一寸！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sm_win_l2, death: "sm_50_death",
        }),
        ("sm_faceless_statue", FightCfg {
            name: "无面石像", hp: 70, dmg: (12, 19), reward: 30, reward_why: "越过无面石像",
            intro: "一尊没有脸的石像立在回廊正中，面孔的位置只有一片光滑的凹坑。当你靠近，凹坑里缓缓浮起一张脸——你的脸。它挡住了通往沉眠神龛的路。",
            rage_at: Some(34), rage_text: "石像的面孔嘎嘎转动，凹坑里同时映出几十张脸，尖啸着涌来！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sm_win_l2, death: "sm_50_death",
        }),
        ("sm_god_sludge", FightCfg {
            name: "旧神唾沫", hp: 66, dmg: (11, 18), reward: 28, reward_why: "挣脱旧神唾沫",
            intro: "神龛前的黑水里渗出几团黏腻的浊液，表面浮着细小的眼睛——那是旧神死亡时最后一口气凝结成的唾沫，仍以为自己是神明，想要吞掉路过的活物来「复活」。",
            rage_at: Some(32), rage_text: "旧神唾沫炸开成漫天黑雨，每一滴都长着眨动的眼！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sm_win_l3, death: "sm_50_death",
        }),
        ("sm_oldgod_spawn", FightCfg {
            name: "旧神眷属", hp: 200, dmg: (14, 22), reward: 300, reward_why: "让旧神眷属彻底安息",
            intro: "穹顶那一泓深暗里，旧神眷属「走」出水来——它没有脚，却能在水面站定；它没有脸，眦裂之处却张着一圈口器，齐齐朝你。千万年过去，它是这座神殿唯一还「活」的东西，也是旧神最后一点残念。",
            rage_at: Some(90), rage_text: "眷属发出一声不属于人类喉咙的尖啸，周身的相位疯狂闪灭——实相与虚相再无间隙！", on_rage: rage_none,
            finisher_if: |st, _| st.fight.as_ref().map(|f| f.raged).unwrap_or(false) && inv(st, "it_shenmiao_reliquary"),
            finisher_name: |_| "以旧神祭器刺入虚相".to_string(),
            finisher_desc: |_| "你举起旧神祭器，对准眷属闪灭不定的一点——它恰好现身实相的刹那，被你一击钉在水上。残念四散，归于那泓倒流的涡心。".to_string(),
            win: sm_win_spawn, death: "sm_50_death",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn shenmiao_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    shenmiao_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   BOSS 旧神眷属 · 相位闪现（选择驱动，非随机，测试确定性）
   血量存 st.fight（sm_35_boss 的 Route::Dyn 初始化，引用 sm_oldgod_spawn 的 FightCfg）。
   机制：眷属在「实相 / 虚相」之间闪灭。玩家每一回合选择：
     - 「重击」：在眷属显形实相时落刀，必定命中，固定伤害。
     - 「虚相追斩」：趁它闪入虚相的瞬间再斩，伤害更高，但若它已狂暴则落空（转成随机伤害到玩家）。
     - 「破相（需祭器）」：以祭器钉入虚相，固定重创 + 易伤。
     - 「防御」：本回合免伤，站稳阵脚。
   以固定数值实现，便于确定性测试；狂热（hp<=90）后绝大多数选择都会招来反击。
   ===================================================================== */
fn start_spawn(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sm_oldgod_spawn") {
            st.fight = Some(crate::power::scaled_fight("sm_oldgod_spawn", cfg, st, vec![]));
            st.set_flag("sm_spawn_start");
        }
    }
    "sm_35_boss_round".to_string()
}

fn spawn_win(st: &mut GameState) -> String {
    crate::world::add_item(st, "it_shenmiao_ash");
    st.points += 300;
    st.set_flag("sm_spawn_dead");
    st.sp_grade = Some('S');
    "sm_36_win".to_string()
}

/// 一套行动。dmg=本次对眷属的伤害（0 或被防御化），guard=防御（免伤），
/// 返回下一场景 id。固定数值，无随机，保证测试确定性。
fn spawn_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    // 推进狂热：hp<=90 时进入狂暴（相位失序，更凶）
    if st.fight.as_ref().map(|f| f.hp <= 90 && !f.raged).unwrap_or(false) {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    // 玩家造成伤害（防御回合打不出有效刀）
    if !guard {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return spawn_win(st);
    }
    // 眷属反击：狂暴后几乎必伤；未狂暴时防守完好则免伤，重击/追斩也可能招来流动反击
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let took_hit = if guard {
        false // 防御本回合免伤
    } else if raged {
        true // 狂暴后相位失序，任何主动出招都被反噬
    } else {
        !(dmg >= 45) // 未狂暴时若一击重创（破相/追斩），可短暂压住它，免遭反击
    };
    if took_hit {
        let raw = if raged { 26 } else { 16 };
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return "sm_50_death".to_string();
    }
    "sm_35_boss_round".to_string()
}

/* =====================================================================
   剧情场景（id 全部 sm_ 前缀）
   ===================================================================== */
pub static SHENMIAO_SCENES: &[SceneDef] = &[

    /* ================= 序 · 沉入倒流之海 ================= */
    SceneDef {
        id: "sm_00", bg: Some("shenmiao_bg.png"), loc: Some("倒流之海 · 深处"),
        mood: "mystery", speaker: None, voice: Some("vo_sm_open"),
        text: TextSpec::Static(&[
            "你睁开眼时，身体正「向上」下沉——不，是被一股看不见的力托着，「向下」浮去。四周是比夜还深的水，唯一的光来自一座悬在头顶、却像踩在你脚下的巨大遗迹。",
            "这里的水是倒着流的。",
            "你落进那道横亘的廊檐，水从你脚边向天上退去，露出湿滑的石阶。一座沉没千年的旧神遗迹，在你面前缓缓睁开了檐角。",
        ]),
        choices: &[
            ChoiceDef { label: "在石阶上站稳，向里走", sub: "进入逆流之涡", cond: None,
                effects: &NO_EFF, route: Route::To("sm_10_f1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F1 逆流之涡（hub） ================= */
    SceneDef {
        id: "sm_10_f1", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 逆流之涡 · 倒垂神柱"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            if cond_has_reliquary(st) {
                "涡心倒流着墨绿的水，巨柱自下而上倒垂回穹顶。你已握到旧神祭器，那颗系着旧神遗音的青铜在水光里嗡鸣——远处的倒置门环，像是正等你把它转正。".to_string()
            } else {
                "这里的水绕着一根根倒垂的神柱往天上流，柱身刻满逆写的祷文，谁也看不懂——或者读懂的人，都已沉在这片涡底。几处最深的水涡里，你隐约听见旧神咽气时的一声长鸣。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "倒流涡心", sub: "调查点 · 揭真相", cond: None, effects: &NO_EFF, route: Route::To("sm_11_vortex") },
            ChoiceDef { label: "倒垂神柱", sub: "调查点 · 逆写祷文", cond: None, effects: &NO_EFF, route: Route::To("sm_12_pillar") },
            ChoiceDef { label: "上层低语", sub: "调查点", cond: None, effects: &NO_EFF, route: Route::To("sm_13_whisper") },
            ChoiceDef { label: "逆泳之池", sub: "调查点 · 得祭器", cond: None, effects: &NO_EFF, route: Route::To("sm_14_pool") },
            ChoiceDef { label: "从溺行祭司身侧挤过", sub: "原住民 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("sm_15_fight") },
            ChoiceDef { label: "向倒置门扉游去", sub: "倒置门环 G1 → F2", cond: Some(cond_has_reliquary), effects: &NO_EFF, route: Route::To("sm_16_gate") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_11_vortex", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 倒流涡心"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["涡心不是往下陷，而是往天上鼓——像一片倒扣的海。你把手探进涡流，水是温的，温得像谁刚咽的气。你隐约拼出一个念头：这座神殿，其实是旧神沉下去的咽喉。"]),
        choices: &[ChoiceDef { label: "记下涡心的逆流朝向", sub: "真相碎片 · 判实相", cond: None,
            effects: &[Eff::SetFlag("sm_true_vortex"), Eff::Points(10), Eff::MarkPoint("sm_pl1_1")], route: Route::To("sm_10_f1") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_12_pillar", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 倒垂神柱"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["你把脸凑到柱文前，那些逆写的祷文在水光里竟慢慢转正，拼成一行你竟读得懂的话：『水向上流的时候，死者会从天上回来。』你数了一下，柱上刻满的，全是名字。"]),
        choices: &[ChoiceDef { label: "记下一个名字", sub: "逆写祷文 · 点亮一柱", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("sm_pl1_2")], route: Route::To("sm_10_f1") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_13_whisper", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 上层低语"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["你抬头，看见倒悬的「上一层」有很多双眼睛贴在头上方的地板上往下看你——那是整个倒流之海浅层溺亡者残存的注视。它们一动不动，只是看，看得你脊背发凉，却也没拦你。"]),
        choices: &[ChoiceDef { label: "快步低头走过低语", sub: "不再看它们", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("sm_pl1_3")], route: Route::To("sm_10_f1") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_14_pool", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 逆泳之池"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["池水逆着浮力往上漫，池底沉着一样东西——一枚系着倒流波纹的青铜祭器。你伸手够，水却把祭器往「上」送，像要故意放你够着一般。拿到手里，掌心立刻传来一声极轻的、倒放的钟鸣。"]),
        choices: &[ChoiceDef { label: "捞出旧神祭器", sub: "Item it_shenmiao_reliquary · 可转正倒置门环", cond: None,
            effects: &[Eff::AddItem("it_shenmiao_reliquary"), Eff::SetFlag("sm_true_pool"), Eff::MarkPoint("sm_pl1_4")], route: Route::To("sm_10_f1") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_15_fight", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 涡口 · 遭遇"),
        mood: "danger", speaker: None, voice: Some("vo_sm_fight"),
        text: TextSpec::Static(&["水幕被什么撕开——一头浑身水壳的残影撞过来。更远处，木然的溺行祭司正一步步跟来。这不是欢迎。你把祭器攥紧，迎上去。"]),
        choices: &[], fight_id: Some("sm_current_shade"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_16_gate", bg: Some("shenmiao_bg.png"), loc: Some("F1 · 倒置门环（G1 已开）"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["祭器一转，倒置门环咔地转正，倒悬的门扉垂成一条通往「上一层」的路。你顺着倒流的水向上走——其实是你往更深的地方去。颠倒回廊在前方伸展开。"]),
        choices: &[ChoiceDef { label: "（踏入颠倒回廊）", sub: "pt_sm_1 单向 · 进 F2", cond: None,
            effects: &NO_EFF, route: Route::To("sm_20_f2") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F2 颠倒回廊（hub） ================= */
    SceneDef {
        id: "sm_20_f2", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 颠倒回廊 · 天顶圣坛"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["你踩的是一片高高在上的「天花板」，地板长得只在你的头顶。向前是倒悬的柱阵与回廊，尽头立着一尊雕花的石像——它挡住了通往更深处的路。这里的一切都在倒着生活，只有你，一个倒着走的人。"]),
        choices: &[
            ChoiceDef { label: "天顶圣坛", sub: "调查点 · 真相", cond: None, effects: &NO_EFF, route: Route::To("sm_21_ceiling") },
            ChoiceDef { label: "颠倒回廊", sub: "调查点", cond: None, effects: &NO_EFF, route: Route::To("sm_22_echo") },
            ChoiceDef { label: "倒悬龙柱", sub: "调查点", cond: None, effects: &NO_EFF, route: Route::To("sm_23_fall") },
            ChoiceDef { label: "与倒悬祭司交谈", sub: "角色 · 唯一醒着的人", cond: None, effects: &NO_EFF, route: Route::To("sm_25_npc") },
            ChoiceDef { label: "被倒影侍从围拢", sub: "原住民 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("sm_24_fight") },
            ChoiceDef { label: "绕过无面石像", sub: "pt_sm_2 单向 · 进 F3", cond: None, effects: &NO_EFF, route: Route::To("sm_26_enter_f3") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_21_ceiling", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 天顶圣坛"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["圣坛倒扣在头顶，坛心里的神像却面朝下，像要凝视路过的你。你看见圣坛底座刻着旧神的生卒年——它不是被谁杀死，而是「寿终正寝」，老死在这座下沉殿里，沉了千万年。"]),
        choices: &[ChoiceDef { label: "记下旧神的生卒年", sub: "真相碎片 · 旧神已死", cond: None,
            effects: &[Eff::SetFlag("sm_true_ceiling"), Eff::Points(10), Eff::MarkPoint("sm_pl2_1")], route: Route::To("sm_20_f2") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_22_echo", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 颠倒回廊"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["回廊空旷，你喊了一声，回声却从「上方」落下来，叠成两个人声。你回头，没有人。旧的你没有跟上，新的你走在了前面——这条回廊，藏着比回音更早的东西。"]),
        choices: &[ChoiceDef { label: "与自己的回声错身", sub: "回廊之谜", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("sm_pl2_2")], route: Route::To("sm_20_f2") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_23_fall", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 倒悬龙柱"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["雕成龙形的石柱从脚底向天上倒悬，龙首垂在你面前。它只剩一个空腔，腔里是干涸的旧血印——这曾经是旧神的「圣物」，如今只是根柱。你伸手，指腹划过粗糙的鳞纹。"]),
        choices: &[ChoiceDef { label: "拂过倒悬龙柱的旧痕", sub: "圣物朽尽", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("sm_pl2_3")], route: Route::To("sm_20_f2") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_24_fight", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 回廊 · 围拢"),
        mood: "danger", speaker: None, voice: Some("vo_sm_fight"),
        text: TextSpec::Static(&["天顶的倒影侍从猛地齐刷刷站起，几十双眼睛同时望向你——它们不让你再往前了。倒悬的金铃哗啦啦响成一片。"]),
        choices: &[], fight_id: Some("sm_inverted_servant"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_25_npc", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 倒悬祭司"),
        mood: "awe", speaker: Some("倒悬祭司"), voice: None,
        text: TextSpec::Static(&["一个极老的活人，用一根绳子把自己吊在天顶上，头朝下看着你——他是这座殿里唯一「醒着」的存在。「别往前了，」他说，「神龛里还留着一口气。你去，它会醒。」他顿了很久，补了一句，「可那口气，是旧神咽不下的。」"]),
        choices: &[ChoiceDef { label: "问他神龛里留着什么", sub: "角色留笔 · 铺向真相", cond: None,
            effects: &[Eff::Points(15), Eff::SetFlag("sm_npc_told")], route: Route::To("sm_20_f2") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_26_enter_f3", bg: Some("shenmiao_bg.png"), loc: Some("F2 · 无面石像前"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&["你正要绕过无面石像，它却移开了——凹坑面庞里浮起你的脸，轻轻让开了一条路。也许它终于认出，你并不是来亵渎的，只是来接那一口咽不下的气。身后的回廊合拢，只剩向前的深暗。"]),
        choices: &[ChoiceDef { label: "（踏进沉眠神龛）", sub: "pt_sm_2 单向 · 进 F3", cond: None,
            effects: &NO_EFF, route: Route::To("sm_30_f3") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F3 沉眠神龛（hub + BOSS） ================= */
    SceneDef {
        id: "sm_30_f3", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 沉眠神龛 · 旧神祭窟"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("sm_spawn_dead") {
                "黑水已归于平静。旧神眷属躺平平地散了，像溪流尽头化开的墨。你站在空神龛前，指尖还能感到那抹余温——这是你能给一个「寿终正寝」的神，最后的安魂。".to_string()
            } else {
                "这里像一具合拢的棺椁：穹顶一泓深暗垂注，水面上浮着薄薄一层白霜。空神龛立在正中，龛里没有神像，只有一缕不肯散的、温的暗。你听见水深处有什么正向水面「沉」上来——倒着。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "空神龛", sub: "调查点 · 旧神的真正归宿", cond: None, effects: &NO_EFF, route: Route::To("sm_31_basilica") },
            ChoiceDef { label: "穹顶之眼", sub: "调查点 · 真相", cond: None, effects: &NO_EFF, route: Route::To("sm_32_eye") },
            ChoiceDef { label: "旧神残骨", sub: "调查点 · 真相", cond: None, effects: &NO_EFF, route: Route::To("sm_33_bones") },
            ChoiceDef { label: "被旧神唾沫缠住", sub: "原住民 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("sm_34_fight") },
            ChoiceDef { label: "【直面旧神眷属】", sub: "BOSS · 相位闪现", cond: None, effects: &NO_EFF, route: Route::To("sm_35_boss") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_31_basilica", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 空神龛"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["你靠近空神龛，龛内竟有温度——是活着的那种温。你终于明白：眷属不是守护者，它本来就是旧神「残念」的一部分；神龛里的空，是旧神留给自己最后那口气的位置。你放轻手，没有去填它。"]),
        choices: &[ChoiceDef { label: "把手按在空龛上", sub: "真相碎片 · 空即余温", cond: None,
            effects: &[Eff::Points(10), Eff::MarkPoint("sm_pl3_1")], route: Route::To("sm_30_f3") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_32_eye", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 穹顶之眼"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["你抬头，穹顶那泓深暗原来是一只眼——垂死的神，最后睁着的那只眼。它已经不再看，只是没有闭上。你与它对视的几秒里，听见它想说却再也说不出的一句话，消散在倒流的水声里。"]),
        choices: &[ChoiceDef { label: "替那只眼闭上视野", sub: "真相碎片 · 垂死仍未瞑目", cond: None,
            effects: &[Eff::SetFlag("sm_true_eye"), Eff::Points(10), Eff::MarkPoint("sm_pl3_2")], route: Route::To("sm_30_f3") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_33_bones", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 旧神残骨"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["水底的残骨巨大而完整，像一座沉没的山脉。骨上覆着细密的、倒着生长的脉络——它一生都在「供养」这片倒流之海，直到把自己也长成海的一部分。你不再觉得它可怕，只觉它老透了。"]),
        choices: &[ChoiceDef { label: "拾起一枚骨屑作凭记", sub: "真相碎片 · 旧神已老", cond: None,
            effects: &[Eff::SetFlag("sm_true_bones"), Eff::Points(10), Eff::MarkPoint("sm_pl3_3")], route: Route::To("sm_30_f3") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_34_fight", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 神龛前 · 缠斗"),
        mood: "danger", speaker: None, voice: Some("vo_sm_fight"),
        text: TextSpec::Static(&["黑水猛然沸腾，几团旧神唾沫扑了上来，翻着细小的眼。它们不是来拦你去路的——它们只是不想让你碰那只空神龛。"]),
        choices: &[], fight_id: Some("sm_god_sludge"), video: None, cine_label: None, overlay: None,
    },

    /* ================= BOSS · 旧神眷属（相位闪现） ================= */
    SceneDef {
        id: "sm_35_boss", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 行走的水面"),
        mood: "danger", speaker: Some("旧神眷属"), voice: Some("vo_sm_boss"),
        text: TextSpec::Static(&[
            "黑水中央，旧神眷属「站」上了水面。它没有脚，却在水的倒影里踩出脚印；它没有脸，眦裂处却张着一圈口器，齐齐朝你。",
            "它不扑来，只是站在你们之间——像一扇还没决定要不要开给活人的门。你只要站了这片水，就已是它视线里唯一的「活物」。",
        ]),
        choices: &[
            ChoiceDef { label: "【直面开战】", sub: "相位闪现决战 · 旧神残念", cond: None,
                effects: &NO_EFF, route: Route::Dyn(start_spawn) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_35_boss_round", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 旧神眷属 · 相位闪现"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            let f = st.fight.as_ref().map(|f| format!("旧神眷属 实相 HP {} / {}", f.hp.max(0), 200)).unwrap_or_else(|| "…".to_string());
            let mode = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) {
                "——相位失序！实相与虚相疯狂闪灭，每一招都可能惹来反噬——"
            } else {
                "——它在实相与虚相间游移。握稳祭器，等它落回实相的刹那——"
            };
            format!("{f}。{mode}")
        }),
        choices: &[
            ChoiceDef { label: "重击", sub: "实相落刀 · 固定 34 伤", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| spawn_act(st, 34, false)) },
            ChoiceDef { label: "虚相追斩", sub: "虚闪现斩 · 固定 46 伤（狂暴后反噬）", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| spawn_act(st, 46, false)) },
            ChoiceDef { label: "【以祭器破相】", sub: "需旧神祭器 · 固定 60 伤 · 必中压境", cond: Some(cond_has_reliquary),
                effects: &NO_EFF, route: Route::Dyn(|st| spawn_act(st, 60, false)) },
            ChoiceDef { label: "防御", sub: "站稳身形 · 本回合免伤", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| spawn_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_36_win", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 空神龛 · 安魂"),
        mood: "calm", speaker: None, voice: Some("vo_sm_boss_down"),
        text: TextSpec::Static(&["你一击落在它显形的实相上。旧神眷属没有溃散成血雾——它只是裂开，像一面碎了很久的镜子，慢慢归拢进那泓倒流的涡心。水面恢复平静，白霜化开，那只穹顶之眼，终于闭上了。"]),
        choices: &[ChoiceDef { label: "把祭器放进空神龛", sub: "安魂 · 得旧神残灰", cond: None,
            effects: &[Eff::AddItem("it_shenmiao_ash"), Eff::Points(30)], route: Route::To("sm_37_ending") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_37_ending", bg: Some("shenmiao_bg.png"), loc: Some("F3 · 旧神祭窟 · 尾声"),
        mood: "mystery", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "水不再向上流。它们终于学会往下落了——千百年来第一次。旧神那口咽不下的气，在你手里被妥帖地放回了空龛。",
            "你低头，看见倒流之海的倒影正在一寸寸「正」过来。原来这一切颠倒，都只是它不肯咽气时，一个垂死之神的最后执念。而今执念尽了，海面平了。",
            "你该带着这桩安魂，离开这座沉了千年的殿了。",
        ]),
        choices: &[
            ChoiceDef { label: "顺流而上，离开倒流之海", sub: "按揭示真相的碎片数结算", cond: None,
                effects: &NO_EFF, route: Route::Dyn(sm_route_settle) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sm_40_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_sm_settle"), death: None,
            card: |st| crate::state::Card {
                title: "沉 没 神 殿 · 安 魂 之 章".into(), good: true,
                body_html: format!(
                    "<p>你让一座沉了千年的旧神遗迹，第一次「正」了过来。旧神眷属安息，倒流之海归平。</p>\
                     <p style='color:#9a958a'>「这里的水是倒着流的。」——而今，它终于愿意往下流。</p>\
                     <p style='color:#7fb3ff'>★ S 级真相：旧神并非被弑，而是寿终正寝，至死不肯咽气。</p>\
                     <table class='statTable'>\
                     <tr><td>揭示真相碎片</td><td>{} / 5 处</td></tr>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>探界评级</td><td style='color:#ffd76a'>{}</td></tr>\
                     </table>",
                    relic_count(st),
                    st.points,
                    if relic_count(st) >= 4 { "S 级（识破旧神真相）" } else { "A 级（安魂成功）" }
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 死亡档案（溺于倒流之海） ================= */
    SceneDef {
        id: "sm_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("沉没神殿 · 溺于倒流之海的探寻者", "在旧神遗迹里，水倒着流过你的肺")), card: |_st| crate::state::Card {
                title: "倒 流 之 海".into(), good: false,
                body_html: r#"<p>你最后是被「向上」卷走的——那些倒流的水终于找到你，把你卷住，沉进更深也更浅的地方。旧神遗迹依旧沉静，倒流的海面上，多了一具不再挣扎的倒影。</p>
<p style='color:#ff8a8a'>【死亡档案 · 沉没神殿溺亡者】</p>
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
fn sm_route_settle(st: &mut GameState) -> String {
    let r = relic_count(st);
    st.sp_grade = Some(if r >= 4 { 'S' } else { 'A' });
    "sm_40_card".to_string()
}
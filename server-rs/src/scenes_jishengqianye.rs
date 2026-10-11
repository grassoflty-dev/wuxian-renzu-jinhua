//! 《无限恐怖·寄生前夜》全部剧情场景与战斗配置。
//! 线粒体异变（幕后科研实验「M-进程」）· 纽约剧院爆炸 · 线粒体怪物。
//! 设计依据：任务钩子「你的每个细胞，都可能在背叛你」。
//! 对应 worlds 文件提供：jishengqianye.rs 之 JISHENGQIANYE_F1_MAP..F3_MAP + JISHENGQIANYE_FLOOR_NAMES
//!                    + POINTS/ENEMIES/NPCS/ZONES/PORTALS/GATES。
//! 本文件为全新新增文件，只导出静态数据（JISHENGQIANYE_SCENES / jishengqianye_figths），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/jishengqianye_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `js_` 前缀；fight id 全部 `wc_jq_` 前缀。
//! 内核：线粒体感染 flag 链（jq_infected_1 → jq_infected_2 → jq_fused），门控 3 结局。
//! BOSS 线粒体聚合体（HP 200 选择驱动，霸者阶段增回=狂暴后攻增且每回合自愈 8 点）。
//! 结局开放：清除（直战）/ 共存（融合后缔约）/ 自毁（融合后同归于尽），sp_grade=Some('D')（任务指令）。
//!
//! ★待素材替换：bg 均用现有图占位（img_nexus / img_corridor / img_zhuyuan_book），新美术由主线统一生图替换。

use crate::defs::*;
use crate::state::GameState;
use rand::Rng;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

/* =====================================================================
   通用小工具
   ===================================================================== */
fn rng(a: i32, b: i32) -> i32 { rand::thread_rng().gen_range(a..=b) }
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_inf1(st: &GameState) -> bool { st.flag("jq_infected_1") }
fn cond_inf2(st: &GameState) -> bool { st.flag("jq_infected_2") }
fn cond_fused(st: &GameState) -> bool { st.flag("jq_fused") }
fn cond_source(st: &GameState) -> bool { st.flag("jq_source_found") }
fn cond_gun(st: &GameState) -> bool { inv(st, "it_jq_gun") }

/* =====================================================================
   Route::Dyn 分支辅助
   ===================================================================== */
fn r_to_f3(_st: &mut GameState) -> String { "js_f3_lobby".to_string() }
fn r_alley_fight(st: &mut GameState) -> String { st.set_flag("jq_alley_fight"); "js_f2_alley_fight".to_string() }

/* =====================================================================
   BOSS · 线粒体聚合体（选择驱动遭遇链，HP 200，霸者阶段增回）
   ===================================================================== */
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("wc_jq_boss") {
            st.fight = Some(crate::power::scaled_fight("wc_jq_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "js_boss_round".to_string()
}

/// 单回合结算：dmg 玩家造成伤害，guard 是否防守，inf 是否本次选择深化自身感染。
/// 狂暴阶段（霸者阶段）聚合体每回合自愈 8 点（增回），且攻增。
fn boss_act(st: &mut GameState, dmg: i32, guard: bool, inf: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() {
            f.hp = (f.hp - dmg).max(0);
        }
    }
    if inf {
        st.set_flag("jq_fused_act");
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return boss_win(st);
    }
    // 霸者阶段（增回）：HP ≤ 80 → 狂暴，且每回合自愈 8
    let raged = st.fight.as_ref().map(|f| f.hp <= f.rage_at.unwrap_or(80)).unwrap_or(false);
    if raged {
        if let Some(f) = st.fight.as_mut() {
            if !f.raged {
                f.raged = true;
                f.hp = (f.hp + 8).min(f.max_hp).max(1);
                st.points += 10; // 了解狂暴·自愈的偿还
            }
        }
    }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { rng(26, 38) } else { rng(16, 26) };
    let dodge = if guard { 0.55 } else { 0.18 };
    let roll: f64 = rand::thread_rng().gen();
    if roll >= dodge {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 {
        return "js_death".to_string();
    }
    "js_boss_round".to_string()
}

/// 聚合体胜利结算：按「抉择 flag」分流到三结局场景，再统一进 js_card。
fn boss_win(st: &mut GameState) -> String {
    st.points += cfg_reward("wc_jq_boss");
    st.set_flag("jq_aggregate_down");
    st.sp_grade = Some('D');
    if st.flag("jq_selfdestruct") {
        crate::world::add_item(st, "it_jq_ember");
        return "js_win_selfdestruct".to_string();
    }
    if st.flag("jq_coexist") {
        crate::world::add_item(st, "it_jq_pact");
        return "js_win_coexist".to_string();
    }
    return "js_win_purge".to_string();
}

/// 读取 fight 表奖励（供 boss_win 使用；无法解析时回退 500）
fn cfg_reward(id: &str) -> i32 {
    crate::scenes::fight_cfg(id).map(|c| c.reward).unwrap_or(500)
}

/* =====================================================================
   普通敌人 win 回调（native FightCfg 由引擎在胜负后调用 win）
   ===================================================================== */
fn win_back(_st: &GameState) -> String { "js_boom".to_string() }
fn win_alley(_st: &GameState) -> String { "js_f2_alley".to_string() }

/// 战斗配置表（id 全部 wc_jq_ 前缀）。
pub fn jishengqianye_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("wc_jq_intro_prop", FightCfg {
            name: "剧场线粒体傀儡", hp: 30, dmg: (6, 10), reward: 10, reward_why: "肃清剧场傀儡",
            intro: "爆炸烟尘里，一个披着戏服的『观众』僵硬地站起来——眼眶里挤满蠕动、苍白的线粒体触须。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_back, death: "js_death",
        }),
        ("wc_jq_drone", FightCfg {
            name: "线粒体傀儡", hp: 45, dmg: (8, 14), reward: 15, reward_why: "肃清线粒体傀儡",
            intro: "零散的『人民』在灯光里抽搐、重组，眨眼间化作扑向你的人形怪物。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_back, death: "js_death",
        }),
        ("wc_jq_evolved", FightCfg {
            name: "线粒体异形体", hp: 70, dmg: (11, 17), reward: 25, reward_why: "击倒线粒体异形体",
            intro: "完全挣脱人形的异形体高耸而起，半透明肉质里翻涌着无数分叉的嵴——它周身缠着淡蓝的能量丝。",
            rage_at: Some(28), rage_text: "异形体膜质炸裂，触须如鞭劈落！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_back, death: "js_death",
        }),
        ("wc_jq_hound", FightCfg {
            name: "线粒体猎犬", hp: 60, dmg: (10, 16), reward: 20, reward_why: "击退线粒体猎犬",
            intro: "一只皮开肉绽的猎犬状怪物，脊柱处顶出开合的气管，喉间挤出人类的低语。",
            rage_at: Some(26), rage_text: "猎犬裂腔吸气，声浪震得你耳膜发涨！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_alley, death: "js_death",
        }),
        ("wc_jq_boss", FightCfg {
            name: "线粒体聚合体", hp: 200, dmg: (16, 26), reward: 500, reward_why: "终结线粒体聚合体",
            intro: "深室中央，千百枚线粒体触须编成的聚合体缓缓展开——如同一颗搏动的细胞体，无数裂缝里燃起幽蓝的光。千百道重叠的声音几乎震碎临场：<b>「你的细胞，已在为我点灯。」</b>",
            rage_at: Some(80), rage_text: "霸者阶段：聚合体膜层炸裂，狂暴攻增，且每回合自愈 8 点——<b>增回！</b>", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| "聚合体共鸣".to_string(),
            finisher_desc: |_| "千道声线同时响起，聚合体体内凝聚出一枚跳动的心脏，与你的胸脯同频搏动。".to_string(),
            win: |_st| "js_win_purge".to_string(),
            death: "js_death",
        }),
    ]
}

/* =====================================================================
   剧情场景（id 全部 js_ 前缀）
   ===================================================================== */
pub static JISHENGQIANYE_SCENES: &[SceneDef] = &[

/* ================= 幕一 · F1 歌剧院 · 开幕之夜 ================= */
SceneDef {
    id: "js_00", bg: Some("jishengqianye_bg.png"), loc: Some("F1 歌剧院 · 前厅"),
    mood: "calm", speaker: Some("主神广播"), voice: Some("vo_jq_open"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>剧院开幕夜爆发线粒体异变 → 循着苏醒的线粒体样本追查研究所核心 → 终结『线粒体聚合体』。代价：失败被扣 400 点复活。",
        "纽约，卡内基歌剧院。今晚《迷雾歌剧》首演，满座衣香鬓影。这里的空气静得出奇——可你的指尖，却在骨子里嗡鸣。",
        "⊙ 主线钩子：「你的每个细胞，都可能在背叛你。」",
    ]),
    choices: &[
        ChoiceDef { label: "听主神任务提示", sub: "记下行动方针", cond: None, effects: &NO_EFF, route: Route::To("js_00_note") },
        ChoiceDef { label: "走向剧场前厅门廊", sub: "调查入口", cond: None, effects: &NO_EFF, route: Route::To("js_f1_foyer") },
        ChoiceDef { label: "走进后台化妆间", sub: "先查后台", cond: None, effects: &NO_EFF, route: Route::To("js_f1_dress") },
        ChoiceDef { label: "翻后台储物间", sub: "找一件趁手武器", cond: None, effects: &NO_EFF, route: Route::To("js_f1_storage") },
    ],
    fight_id: None, video: None, cine_label: Some("过场 · 开幕之夜"), overlay: None,
},
SceneDef {
    id: "js_00_note", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 前台任务栏"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["前台的铜牌上压着一张皱巴巴的纸：『凌晨零点，前排第 13 座』。你总觉得这串数字，与恐怖谷里的秘密脱不开干系。"]),
    choices: &[ChoiceDef { label: "记下情报", sub: "+5 点", cond: None, effects: &[Eff::Points(5)], route: Route::To("js_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f1_foyer", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 剧场前厅门廊"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["门廊镀金的栏杆上映着无数宾客的倒影。你瞥见其中一个『宾客』的倒影没有脸——一团蠕动的蓝，随即又恢复如常。你的视网膜在告诉你：是错觉，还是你已经开始看见线粒体？"]),
    choices: &[
        ChoiceDef { label: "转身回中场", sub: "继续探查", cond: None, effects: &NO_EFF, route: Route::To("js_00") },
        ChoiceDef { label: "追向前台服务员", sub: "藏一手线索", cond: None, effects: &NO_EFF, route: Route::To("js_f1_scientist") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f1_dress", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 后台化妆间"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["后台化妆间飘着檀香，妆镜前坐着一名女演员，正对镜精心补妆。你走近才看清——她膨胀发青的面颊静脉里，有颗粒粗大的『核』正随光线忽明忽暗。她的血液，正被某种更古老的东西替换。","女演员缓缓转头，笑颜之下，眼神苍白而空洞。"]),
    choices: &[
        ChoiceDef { label: "后退", sub: "避免惊动", cond: None, effects: &NO_EFF, route: Route::To("js_00") },
        ChoiceDef { label: "轻唤她", sub: "试探", cond: None, effects: &NO_EFF, route: Route::To("js_f1_stage") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f1_stage", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 主舞台"),
    mood: "mystery", speaker: Some("克丽丝 · 基因研究员"), voice: None,
    text: TextSpec::Static(&["你追到主舞台。深红帷幕下，一位挂着研究员证的蓝袍女子蹲在布景后，用镊子夹取一团渗血的肉块观察。见你靠近，她猛地压低声音：","「别靠近！这是……奇迹，也是灾厄。细胞里的线粒体，正抛开人的意愿，按它自己的图纸重建宿主。——我叫克丽丝，只是来观摩歌剧。现在我看的，是末日的开场彩排。」"]),
    choices: &[
        ChoiceDef { label: "向克丽丝询问线粒体", sub: "感染链 · 阶段一", cond: None,
            effects: &[Eff::SetFlag("jq_infected_1"), Eff::Points(10)], route: Route::To("js_f1_scientist") },
        ChoiceDef { label: "用手机记录样本", sub: "+10 点", cond: None, effects: &[Eff::Points(10)], route: Route::To("js_00") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f1_scientist", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 后台 · 克丽丝"),
    mood: "mystery", speaker: Some("克丽丝 · 基因研究员"), voice: None,
    text: TextSpec::Static(&["克丽丝把那团肉块放进封存瓶，神情凝重：「这些线粒体在『繁殖』。它们在优秀的细胞里潜伏，在环境中失控，直至把整座剧场变成巢穴。我有个大胆的猜测——真正的主谋，不在剧场，而在那栋亮着蓝光的研究大楼里。」","她顿了顿：「……你自己，可千万小心。一旦你的细胞也开始供能给它们，你就再回不了头。」"]),
    choices: &[
        ChoiceDef { label: "拿取实验室配发的枪", sub: "Weapon Gun + Item it_jq_gun", cond: None,
            effects: &[Eff::Weapon(crate::state::Weapon::Gun), Eff::AddItem("it_jq_gun")], route: Route::To("js_00") },
        ChoiceDef { label: "询问大楼位置", sub: "锁定 F3 线索", cond: None, effects: &NO_EFF, route: Route::To("js_f2_tower") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f1_storage", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 后台储物间"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["储物间堆满戏服与道具。你在回收箱里翻出一把口径偏旧的手枪，枪身刻着一行小字：『季/·/07』。顺手，还压着半张泛蓝光的课题组名片。"]),
    choices: &[
        ChoiceDef { label: "取枪 · 压弹", sub: "Weapon Gun + Item it_jq_gun", cond: None,
            effects: &[Eff::Weapon(crate::state::Weapon::Gun), Eff::AddItem("it_jq_gun"), Eff::Points(5)], route: Route::To("js_00") },
        ChoiceDef { label: "只拿走那张名片", sub: "+5 点", cond: None, effects: &[Eff::Points(5)], route: Route::To("js_00") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef { id: "js_boom", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 剧场 · 零点来临"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["零点整。舞台中央的巨型吊灯轰然炸裂，千百吨玻璃与碎屑泼洒向观众席——紧接着，满场『观众』腾地站起，抽搐着，从发际、耳窝、眼眶里挤出蠕动的线粒体触须。整座歌剧院，在一瞬之间变成了紫红与蓝的巢穴。"]),
    choices: &NO_CH, fight_id: Some("wc_jq_intro_prop"), video: None, cine_label: None, overlay: None },
SceneDef { id: "js_f1_exit", bg: Some("jishengqianye_bg.png"), loc: Some("F1 · 剧场出口"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["浓烟从剧场各个出口涌出。你一脚踹开消防门，撞进十月冷冽的纽约夜色——街道上，路灯忽明忽暗，警戒线刚拉起，远处晃动的人影，正以一种不属于常人的僵直步态，向你围拢。"]),
    choices: &[ ChoiceDef { label: "冲进封锁街道", sub: "入 F2", cond: None, effects: &NO_EFF, route: Route::Dyn(r_to_f3) } ],
    fight_id: None, video: None, cine_label: None, overlay: None },

/* ================= 幕二 · F2 街区 · 感染蔓延 ================= */
SceneDef {
    id: "js_f2_tower", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 封锁街道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["警戒塔顶的红灯扫过街道。法医封条、翻倒的救护车、沿路拖行的血痕——这座城市，正被看不见的『客人』一格格吞入。你循着蓝光，锁定了那座矗立在街区尽头的中央研究所大楼。"]),
    choices: &[
        ChoiceDef { label: "去废弃救护站", sub: "查看临时收容", cond: None, effects: &NO_EFF, route: Route::To("js_f2_clinic") },
        ChoiceDef { label: "转进黑巷", sub: "避开巡逻傀儡", cond: None, effects: &NO_EFF, route: Route::To("js_f2_alley") },
        ChoiceDef { label: "向警戒塔守警打听", sub: "寻警官布伦", cond: None, effects: &NO_EFF, route: Route::To("js_f2_officer") },
        ChoiceDef { label: "进中央研究所大楼", sub: "需先摸清感染真相", cond: Some(cond_inf2), effects: &NO_EFF, route: Route::Dyn(r_to_f3) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f2_clinic", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 废弃救护站"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["救护站的担架上横着几具戛然而止的『病人』，他们体内都残留着同一种泛蓝的胶状物。一名裹着毯子的幸存女孩（梅）抬起眼：「他们说这里安全……但他们是把所有人送进来，去喂那个东西……救救我们，别让那栋楼再亮下去。」"]),
    choices: &[
        ChoiceDef { label: "安抚梅 · 记下证词", sub: "+10 点", cond: None, effects: &[Eff::Points(10)], route: Route::To("js_f2_tower") },
        ChoiceDef { label: "调查活性样本", sub: "感染链 · 阶段二", cond: None,
            effects: &[Eff::SetFlag("jq_infected_2"), Eff::San(-5)], route: Route::To("js_f2_clinic_sample") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef { id: "js_f2_clinic_sample", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 救护站样本室"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你把指尖探入那团胶状物取样——只一瞬，你感到骨髓里一阵微麻。你再抬头，梅的倒影在你视网膜里，多了千万重颤动的细影。你明白：你也被写进了那张图纸。"]),
    choices: &[ ChoiceDef { label: "压抑心悸", sub: "感染链 · 深化", cond: None, effects: &[Eff::SetFlag("jq_fused"), Eff::San(-8)], route: Route::To("js_f2_tower") } ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f2_morgue", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 法医处"),
    mood: "mystery", speaker: Some("法医 · 罗伊"), voice: None,
    text: TextSpec::Static(&["法医罗伊掀开白布，露出解剖台上的『尸体』——那具躯体的五脏六腑，已被整整齐齐替换成一层层纺织的嵴膜。「这不是尸体，」罗伊摘下手套，「这是一张结网的图纸。真正的线粒体源头，就在那栋中央研究所里。」"]),
    choices: &[
        ChoiceDef { label: "请罗伊出具尸检报告", sub: "请布伦放行", cond: Some(cond_inf1),
            effects: &[Eff::SetFlag("jq_officer_ok"), Eff::Points(10)], route: Route::To("js_f2_tower") },
        ChoiceDef { label: "翻查法医档案", sub: "找研究所底细", cond: None, effects: &NO_EFF, route: Route::To("js_f2_doc") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef { id: "js_f2_doc", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 法医档案"), mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["档案里夹着一页盖章不久的《中央研究所 · 生物学实验 · 折A》，铅印处只留半枚残印。你认出身形——和储物间名片上那枚『季/·/07』，同出一源。"]),
    choices: &[ ChoiceDef { label: "拼合线索", sub: "感染链 · 阶段一显明", cond: None, effects: &[Eff::SetFlag("jq_infected_1"), Eff::Points(10)], route: Route::To("js_f2_tower") } ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f2_officer", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 警戒塔 · 布伦"),
    mood: "cold", speaker: Some("警官 · 布伦"), voice: None,
    text: TextSpec::Static(&["布伦警官按着腰间的枪横在你面前：「市民，这街区已封锁。退回去。上头有令——谁进那栋楼，就按『疑似病原体』处理。」他顿了顿，压低声音：「……不过，你若能证明自己已见过『图纸』的真面目，我或许能给你放行。」"]),
    choices: &[
        ChoiceDef { label: "出示感染链证据", sub: "获布伦放行", cond: Some(cond_inf1),
            effects: &[Eff::SetFlag("jq_officer_ok"), Eff::Points(10)], route: Route::To("js_f2_tower") },
        ChoiceDef { label: "请求绕道黑巷", sub: "不惊动布伦", cond: None, effects: &NO_EFF, route: Route::To("js_f2_alley") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f2_alley", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 黑巷"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["黑巷墙角的垃圾堆上，伏着一只背脊裂开的线粒体猎犬，正用人类嗓音低低抽泣——它看见你，喉间的缝隙一张一合：『……放……放我……回……回……』"]),
    choices: &[
        ChoiceDef { label: "拔枪迎战", sub: "战猎犬", cond: Some(cond_gun), effects: &NO_EFF, route: Route::Dyn(r_alley_fight) },
        ChoiceDef { label: "绕开它", sub: "退向警戒塔", cond: None, effects: &NO_EFF, route: Route::To("js_f2_tower") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef { id: "js_f2_alley_fight", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 黑巷 · 遭遇线粒体猎犬"), mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["猎犬作势扑来，黑巷里刮起腥风。"]),
    choices: &NO_CH, fight_id: Some("wc_jq_hound"), video: None, cine_label: None, overlay: None },
SceneDef { id: "js_f2_survivor", bg: Some("jishengqianye_bg.png"), loc: Some("F2 · 救护站前 · 幸存者梅"), mood: "calm", speaker: Some("幸存者 · 梅"), voice: None,
    text: TextSpec::Static(&["梅把一块暖手的芯片塞进你掌心：「拿着。这是我趁乱从大楼通风口拍的……上面全是培育细胞的编号。他们要的从来不是救人。」"]),
    choices: &[ ChoiceDef { label: "收下芯片", sub: "Item it_jq_chip · 锁定 F3", cond: None,
        effects: &[Eff::AddItem("it_jq_chip"), Eff::Points(10)], route: Route::To("js_f2_tower") } ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 幕三 · F3 中央研究所 ================= */
SceneDef {
    id: "js_f3_lobby", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 研究所中庭大厅"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["你推门而入，中庭大厅的中央立着一尊巨大的『细胞切片』树脂雕塑。蓝光自上方倾泻——这里没有一丝活人气息，只有无数培养皿的嗡鸣与一浪高过一浪的低频搏动。"]),
    choices: &[
        ChoiceDef { label: "前往机密档案室", sub: "破解『图纸』来源", cond: None, effects: &NO_EFF, route: Route::To("js_f3_archive") },
        ChoiceDef { label: "探样本室", sub: "采集聚合体残基", cond: None, effects: &NO_EFF, route: Route::To("js_f3_sample") },
        ChoiceDef { label: "浏览病历中心", sub: "追查实验记录", cond: None, effects: &NO_EFF, route: Route::To("js_f3_med") },
        ChoiceDef { label: "深入中庭机组", sub: "关闭维持系统", cond: None, effects: &NO_EFF, route: Route::To("js_f3_atrium") },
        ChoiceDef { label: "直下培育舱", sub: "需已破解档案机密", cond: Some(cond_source), effects: &NO_EFF, route: Route::To("js_f3_cultivate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f3_archive", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 机密档案室"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["档案柜里压着一份未盖章的最高机密：『M-进程』。首页只有一行铅字：『线粒体聚合体 = 唤醒被囚于每个人细胞里的元意识』。你翻到最后，发现了那枚『季/·/07』的签名——正是出品剧场样本的研究员，季学。"]),
    choices: &[
        ChoiceDef { label: "确认聚合体蓝图", sub: "jq_source_found · 破门培育舱", cond: None,
            effects: &[Eff::SetFlag("jq_source_found"), Eff::Points(20)], route: Route::To("js_f3_lobby") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f3_sample", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 样本室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["样本室内，数千只封存的『线粒体细胞』在灯光下苏醒、震颤，像一片等待合唱的温床。你采集一枚极活跃的，触到了聚合体的门脉。"]),
    choices: &[
        ChoiceDef { label: "核对样本活性", sub: "+15 点 · 聚合体情报", cond: None, effects: &[Eff::Points(15)], route: Route::To("js_f3_lobby") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f3_med", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 病历中心"),
    mood: "mystery", speaker: None, voice: None,
    text: TextSpec::Static(&["病历屏上滚动着『受试者α』的体征曲线。你在第 12 页看到一行批注：『宿主细胞同步率达峰时，宿主将失去『我』。那便是聚合体的王座——也是唯一的出路。』"]),
    choices: &[ ChoiceDef { label: "辨读批注", sub: "提示感染链终点", cond: None, effects: &[Eff::Points(10)], route: Route::To("js_f3_lobby") } ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f3_atrium", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 中庭机组"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你扳断中庭机组的冷却管，啸鸣的蓝光骤降——整栋大楼的搏动霎时紊乱，聚合体的『心跳』漏了一拍。你在管道间瞥见一具外壳，与你身上的——一模一样。","那是『你』的复制体，已被培育到线粒体同步的临界。"]),
    choices: &[
        ChoiceDef { label: "拆解复制体外壳", sub: "感染链 · 阶段二深化", cond: None,
            effects: &[Eff::SetFlag("jq_infected_2"), Eff::San(-8)], route: Route::To("js_f3_lobby") },
        ChoiceDef { label: "只记下坐标", sub: "+10 点", cond: None, effects: &[Eff::Points(10)], route: Route::To("js_f3_lobby") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f3_cultivate", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 培育舱"),
    mood: "mystery", speaker: Some("聚合体低语"), voice: None,
    text: TextSpec::Static(&["培育舱灯火通明，中央一池乳白的『培养液』里，浸泡着一枚搏动的细胞核——聚合体的孩子。它认出你，缓缓开口：","「你已经被写入图纸。你的细胞，我的灯……放下抵抗，与我融为一体，你将成为新意识的子民；或倾颓这场旧纪元。」"]),
    choices: &[
        ChoiceDef { label: "拒绝 · 保持自我", sub: "走向清除线", cond: Some(cond_inf2), effects: &[Eff::Points(20)], route: Route::To("js_f3_decision") },
        ChoiceDef { label: "沉溺 · 融入聚合体", sub: "开启共存/自毁线", cond: Some(cond_fused), effects: &[Eff::Points(30), Eff::San(-10)], route: Route::To("js_f3_decision") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---------------- 终局抉择 + BOSS ---------------- */
SceneDef {
    id: "js_f3_decision", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 聚合体深室闸前"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("jq_fused") {
            "聚合体借你的细胞向你低语：你已与它血脉相连——你既可在深室奋力把它从血肉图纸里连根清除；也可饮下这座王座，与它共存；甚至引爆自己的细胞，与它同归于尽。".to_string()
        } else {
            "聚合体在深室深处步步逼近。你知道：此刻唯有把它的『核心』从细胞图纸中剥离、彻底清除，才能救回这座城市。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "踏入深室", sub: "准备终局抉择", cond: None,
            effects: &[Eff::SetFlag("jq_boss_ready"), Eff::San(-5)], route: Route::To("js_f3_bossgate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "js_f3_bossgate", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 聚合体深室"),
    mood: "danger", speaker: Some("线粒体聚合体"), voice: None,
    text: TextSpec::Static(&["深室闸门液压撑开——线粒体聚合体在幽蓝光雾中舒张成一座搏动的恢弘心脏。千百道视野同时望向门前的你，声音重叠得几乎撕碎耳膜：","「进。你是我最后的容器。」"]),
    choices: &[
        ChoiceDef { label: "拔枪开战 · 清除", sub: "直面聚合体", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
        ChoiceDef { label: "先定下共存之约", sub: "需已融合(jq_fused)", cond: Some(cond_fused),
            effects: &[Eff::SetFlag("jq_coexist"), Eff::Points(30)], route: Route::Dyn(start_boss) },
        ChoiceDef { label: "引爆自身细胞 · 自毁", sub: "需已融合(jq_fused)", cond: Some(cond_fused),
            effects: &[Eff::SetFlag("jq_selfdestruct"), Eff::Points(30), Eff::San(-20)], route: Route::Dyn(start_boss) },
        ChoiceDef { label: "再校准药箱", sub: "镇定心神 · 疗伤", cond: None, effects: &[Eff::San(15)], route: Route::To("js_f3_bossgate") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* BOSS 选择驱动回合（HP 存 st.fight，由 start_boss 建立） */
SceneDef {
    id: "js_boss_round", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 聚合体深室"),
    mood: "danger", speaker: Some("线粒体聚合体"), voice: None,
    text: TextSpec::Dyn(|st| format!("聚合体剩余 {} 点（原始 200）——你 HP {}。它的膜层正像呼吸般扩张，那枚养殖到极致的核心，随每一下搏动向你胸膛逼近。",
        st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
    choices: &[
        ChoiceDef { label: "密集火力 · 净化核心", sub: "高伤害 · 可能被反噬", cond: None, effects: &NO_EFF,
            route: Route::Dyn(|st| boss_act(st, rng(30, 46), false, false)) },
        ChoiceDef { label: "冷静卸力 · 诱导分裂", sub: "稳伤害 · 低风险", cond: None, effects: &NO_EFF,
            route: Route::Dyn(|st| boss_act(st, rng(14, 26), true, false)) },
        ChoiceDef { label: "借自身线粒体共鸣", sub: "增幅伤害 · 深化感染", cond: Some(cond_fused), effects: &NO_EFF,
            route: Route::Dyn(|st| boss_act(st, rng(36, 52), false, true)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ================= 结局三线（都收敛到 js_card 结算） ================= */
SceneDef {
    id: "js_win_purge", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 聚合体残骸"),
    mood: "calm", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["聚合体的心脏被你的火力贯穿，凝结成一个静止的蓝核，坠落在地，裂成碎屑。窗外，纽约的夜空第一次露出了干净的星河。","清除线达成。线粒体巢穴连根拔除——你从研究所断壁间走出，把那段蓝光留在了身后。"]),
    choices: &[ ChoiceDef { label: "撤离研究所", sub: "完成副本", cond: None, effects: &NO_EFF, route: Route::Dyn(|_st| "js_card".to_string()) } ],
    fight_id: None, video: None, cine_label: Some("结局 · 清除"), overlay: None,
},
SceneDef {
    id: "js_win_coexist", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 聚合体王座"),
    mood: "mystery", speaker: Some("线粒体聚合体"), voice: None,
    text: TextSpec::Static(&["你没有拔枪。你伸手，触碰了那枚搏动的核心。一瞬的灼痛后，蓝光沿你的血管改道，你的细胞与新意识订下契约——不再吞噬，转为共生。餐馆、街道、整座城市里苏醒的『它们』静了下来，重新睡回每个细胞的深处。","共存线达成。人类不再被吞噬，聚合体真正成为细胞的远方兄弟。你带着一枚温和跳动的『第二心脏』离开大楼。"]),
    choices: &[ ChoiceDef { label: "携契约离场", sub: "完成副本", cond: None, effects: &NO_EFF, route: Route::Dyn(|_st| "js_card".to_string()) } ],
    fight_id: None, video: None, cine_label: Some("结局 · 共存"), overlay: None,
},
SceneDef {
    id: "js_win_selfdestruct", bg: Some("jishengqianye_bg.png"), loc: Some("F3 · 聚合体王座"),
    mood: "danger", speaker: Some("线粒体聚合体"), voice: None,
    text: TextSpec::Static(&["你把指尖探入自己的胸腔，触悸到那枚已被线粒体同步的心脏。聚合体怔住：「你……愿为这座城？」你按下了引爆的引信。","细胞图谱自你体内轰然燃烧，蓝与红在大楼中庭炸开成一道贯穿天顶的光柱。聚合体在崩解前最后发出的，是数不清的『它』所喊出的、本属于人类的恐惧。","自毁线达成。你以自己为代价，把线粒体异变整个从纽约的夜空里点燃殆尽。"]),
    choices: &[ ChoiceDef { label: "……请记得我", sub: "完成副本 · 自毁结局", cond: None, effects: &NO_EFF, route: Route::Dyn(|_st| "js_card".to_string()) } ],
    fight_id: None, video: None, cine_label: Some("结局 · 自毁"), overlay: None,
},

/* ================= 结算 / 死亡（offcanvas 卡片，回主神） ================= */
SceneDef {
    id: "js_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "结 算".into(), good: true,
            body_html: format!(
                "<p>你终结了<em>寄生前夜</em>的线粒体异变。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr><tr><td>支线评级</td><td>D</td></tr></table><p>异变总在细胞深处蛰伏——你已学会聆听，也学会抵抗。</p>",
                st.points),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "js_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("无限恐怖 · 寄生前夜 · 殒命", "细胞终为线粒体点灯")),
        card: |_st| crate::state::Card {
            title: "死 亡".into(), good: false,
            body_html: r#"<p>你的细胞，最终成了这台机器点亮的最后一盏灯。</p><p style='color:#ff8a8a'>【死亡档案】殒命于无限恐怖 · 寄生前夜</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];
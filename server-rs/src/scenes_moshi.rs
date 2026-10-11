//! 《末世死城·人类防线》任务世界 · 全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/wuxian_weilai/moshi_shoucheng.md §4/§5/§6/§7/§8。
//! 本文件是全新新增文件，只导出静态数据（MOSHI_SCENES / moshi_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/moshi_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `ms_` 前缀，与既有 SCENES / zy_* 无重名；战斗 id 全部 `fight_*` 前缀（照抄 §4/§5）。
//! BOSS 战（狂化攻城巨兽 fight_r_siege_beast）用 fight_id 交给引擎回合制，轨道支援终结技用 FightCfg 的
//! finisher_if（cond: orbital_auth 且已调查信标塔）表达——使用时引擎 fight_win → win 路由到结局；120 固伤
//! 以终结技实现，胜利分支绝地清场（evac_done + sp_grade C）或惨胜（evac_hard）。
//! 多波次战场用「战斗场景链」（win → 下一下一场 fight 场景）+ ZoneDef(kind=fight) 引用各波场景实现，无新引擎字段。
//! sp_grade 用 Route::Dyn 内联 st.sp_grade = Some('C')（引擎已有字段，Eff 不新增）。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   条件谓词（cond，具名 fn，CondFn 为 fn 指针不能捕获闭包）
   ===================================================================== */
/// 已通关蜂巢（引擎实际完成 flag 为 bh_cleared；设计文档写 hive_cleared → 实现用引擎权威名）
fn cond_hive_cleared(st: &GameState) -> bool { st.flag("bh_cleared") }
/// 是否持有撬棍（F1 巴士 / F2 杂物间）
fn cond_has_crowbar(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "crowbar") }
/// F2 病房调查点取得上尉办公室钥匙
fn cond_has_office_key(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "office_key") }
/// 是否持有军火库钥匙卡（上尉办公室或 F3 指挥官发放）
fn cond_has_keycard(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "keycard_armory") }
/// 是否已取得轨道授权（F3 指挥官）
fn cond_orbital_auth(st: &GameState) -> bool { st.flag("orbital_auth") }
/// 电力已恢复 → F3 货梯/通讯阵列可用
fn cond_power_restored(st: &GameState) -> bool { st.flag("power_restored") }
/// 幸存者计数达到 3（三段计数 sur_n1/n2/n3，结算支线 survivors_3）
fn cond_survivors_3(st: &GameState) -> bool {
    st.flag("ms_sur_n1") && st.flag("ms_sur_n2") && st.flag("ms_sur_n3")
}

/* =====================================================================
   动态文本辅助
   ===================================================================== */
fn txt_comander(st: &GameState) -> String {
    if st.flag("orbital_auth") {
        "老指挥官把授权码收回来：「轨道支援只有一发。你已经拿到了想拿的东西，登炮台吧——把这一发，留给最大的那个家伙。」".into()
    } else if st.flag("power_restored") {
        "「电力回来了，反应堆只是暂时被你稳住。」老指挥官点亮观测屏，「授权码在通讯阵列的加密文件里——先重启它，我才能把轨道支援放给你。」".into()
    } else {
        "地下指挥所断电，只有应急灯在闪。老指挥官把一枚冰冷的钥匙卡放到桌上：「先把反应堆弄亮，把通讯阵列拉起来，我再考虑要不要把轨道支援的授权码给你。」".into()
    }
}

fn txt_beacon(st: &GameState) -> String {
    if st.flag("ms_beacon_ready") {
        "轨道信标塔的指示灯已经攀升至全绿，塔尖的激光笔直刺进黄昏天幕。你上一次呼叫轨道支援的条件，已经就绪。".into()
    } else {
        "信标塔前的发射控制器还亮着「未授权」的红灯。你用力按下测试键——塔身嗡鸣一声，射出一束标定激光指向东方。你记住了射界参数（ms_beacon_ready）。".into()
    }
}

/* =====================================================================
   战斗配置表（moshi 专属；导出供主线把 query 扩展进来）
   数值全部照抄 §4 敌人表 / §5 BOSS 表（含建议区间，取表中建议值）。
   ===================================================================== */
fn rage_reinforce(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>兽潮开始增员——远处响起更多粗重的喘息。</span>".into());
}
fn rage_speed(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>它低吼一声，灰褐毛皮绷紧，速度陡增。</span>".into());
}
fn rage_leap(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>跳扑兽四肢一曲，向你跃扑而来——命中率大增。</span>".into());
}
fn rage_burrow(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>掘地兽一头钻进地下——下一回合它的攻击必中。</span>".into());
}
fn rage_fortify(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>兽兵披甲硬顶，鳞甲合拢，减伤 20%。</span>".into());
}
fn rage_shriek(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>医疗变异体发出刺耳嘶吼——远处传来增援的脚步声。</span>".into());
}

/// 通用胜利路由：多以 ms_02 起衔接后续（各 fight 可在 win 上自定义进入下一波场景）
fn win_next(st: &GameState) -> String {
    let _ = st;
    "ms_02".into()
}

/// F1 城门口广场多波战链、F2 医森线、F4 决死线专用 win 函数：
/// 逐波胜利路由到「休息回复节点」（全回满血），再进下一场战斗，规避连续战无回复导致中途阵亡。
fn win_wave_a(st: &GameState) -> String { let _ = st; "ms_rest_wave1".into() }
fn win_wave_b(st: &GameState) -> String { let _ = st; "ms_rest_wave2".into() }
fn win_wave_c(st: &GameState) -> String { let _ = st; "ms_02".into() }
fn win_f2prep_a(st: &GameState) -> String { let _ = st; "ms_rest_f2".into() }
fn win_f2prep_b(st: &GameState) -> String { let _ = st; "ms_medic_win".into() }
fn win_f4prep_a(st: &GameState) -> String { let _ = st; "ms_rest_f4".into() }
fn win_f4prep_b(st: &GameState) -> String { let _ = st; "ms_f4_boss_intro".into() }

/// 战斗配置表（id 全部 fight_ 前缀）。
pub fn moshi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("fight_f1_beast", FightCfg {
            name: "兽兵", hp: 44, dmg: (9, 17), reward: 13, reward_why: "兽兵 · 击退",
            intro: "灰褐毛皮、橙红竖瞳的兽兵挥爪扑来——城头的第一道生物浪。",
            rage_at: Some(20), rage_text: "", on_rage: rage_speed,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f1_leaper", FightCfg {
            name: "跳扑兽", hp: 47, dmg: (10, 18), reward: 15, reward_why: "跳扑兽 · 击退",
            intro: "深蓝甲片的跳扑兽从废墟后窜出，一个低姿弹跳直扑你面门。",
            rage_at: Some(24), rage_text: "", on_rage: rage_leap,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f1_pack", FightCfg {
            name: "兽潮小队", hp: 72, dmg: (14, 22), reward: 26, reward_why: "兽潮小队 · 击退",
            intro: "一队灰褐异兽挤过城门口的缺口，低吼着渐次压上。",
            rage_at: Some(33), rage_text: "", on_rage: rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f2_mutant", FightCfg {
            name: "医疗变异体", hp: 78, dmg: (13, 21), reward: 60, reward_why: "医疗变异体 · 消灭",
            intro: "白大褂被缝合的肢体撑开，多关节畸形的手臂拖在地上，发出「哒哒」的脚步声。",
            rage_at: Some(39), rage_text: "", on_rage: rage_shriek,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |st| { let _ = st.flag("medic_trusted"); "ms_medic_win".into() },
            death: "ms_lose",
        }),
        ("fight_f2_stalker", FightCfg {
            name: "潜行猎兽", hp: 120, dmg: (18, 27), reward: 100, reward_why: "潜行猎兽 · 猎杀",
            intro: "黑灰色潜行猎兽贴着墙影潜行，下一秒从你肋下窜出——",
            rage_at: Some(52), rage_text: "", on_rage: rage_speed,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f3_burrower", FightCfg {
            name: "掘地兽", hp: 60, dmg: (12, 19), reward: 50, reward_why: "掘地兽 · 镇压",
            intro: "土黄褐鳞甲的掘地兽从走廊地砖下拱出，钳足卡进混凝土。",
            rage_at: Some(30), rage_text: "", on_rage: rage_burrow,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f3_pack", FightCfg {
            name: "兽潮·深部", hp: 84, dmg: (16, 25), reward: 30, reward_why: "兽潮·深部 · 击退",
            intro: "更暗色的深部兽群从纵深走廊涌来，数量还在增。",
            rage_at: Some(40), rage_text: "", on_rage: rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f3_vanguard", FightCfg {
            name: "高阶兽兵", hp: 100, dmg: (17, 26), reward: 70, reward_why: "高阶兽兵 · 击破",
            intro: "深黑红涂装的高阶兽兵披着甲胄硬顶上前，是兽潮的精锐。",
            rage_at: Some(45), rage_text: "", on_rage: rage_fortify,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        ("fight_f4_pack", FightCfg {
            name: "兽潮·决死", hp: 88, dmg: (17, 26), reward: 35, reward_why: "兽潮·决死 · 挡下",
            intro: "焦黑火点的决死兽群在观测甲板上反扑——这是它们最后的突围。",
            rage_at: Some(42), rage_text: "", on_rage: rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_next, death: "ms_lose",
        }),
        // ---- F1 城门口广场 · 多波战专用链（win 逐波衔接）----
        ("fight_f1_wave_a", FightCfg {
            name: "兽潮小队", hp: 72, dmg: (14, 22), reward: 26, reward_why: "第一波兽潮 · 击退",
            intro: "第一波兽潮小队冲过城门缺口，低吼着压上。",
            rage_at: Some(33), rage_text: "", on_rage: rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_wave_a, death: "ms_lose",
        }),
        ("fight_f1_wave_b", FightCfg {
            name: "跳扑兽", hp: 47, dmg: (10, 18), reward: 15, reward_why: "第二波跳扑兽 · 击退",
            intro: "兽潮小队被击散，一头跳扑兽从血雾里跃出，直扑面门。",
            rage_at: Some(24), rage_text: "", on_rage: rage_leap,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_wave_b, death: "ms_lose",
        }),
        ("fight_f1_wave_c", FightCfg {
            name: "兽兵×2", hp: 88, dmg: (9, 17), reward: 26, reward_why: "第三波兽兵 ×2 · 击退",
            intro: "两头发狂暴的兽兵从瓦砾后扑出，把最后一波填进缺口。",
            rage_at: Some(20), rage_text: "", on_rage: rage_speed,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_wave_c, death: "ms_lose",
        }),
        // ---- F2 救护士 · 医森线链 ----
        ("fight_f2_mutant_wave", FightCfg {
            name: "医疗变异体", hp: 78, dmg: (13, 21), reward: 60, reward_why: "医疗变异体 · 消灭（救护士线）",
            intro: "缝合肢体的医疗变异体拖着畸形手臂，正朝你扑来。",
            rage_at: Some(39), rage_text: "", on_rage: rage_shriek,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f2prep_a, death: "ms_lose",
        }),
        ("fight_f2_mutant_wave2", FightCfg {
            name: "医疗变异体·2", hp: 78, dmg: (13, 21), reward: 60, reward_why: "医疗变异体 · 消灭（救护士线）",
            intro: "又一只变异体从病房深处撕开帘布扑出。",
            rage_at: Some(39), rage_text: "", on_rage: rage_shriek,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f2prep_b, death: "ms_lose",
        }),
        // ---- F4 决死清场 · BOSS 前链 ----
        ("fight_f4_prep_a", FightCfg {
            name: "兽潮·决死", hp: 88, dmg: (17, 26), reward: 35, reward_why: "决死兽潮 · 挡下",
            intro: "观测甲板四周的决死兽群收拢，朝信标塔逼近。",
            rage_at: Some(42), rage_text: "", on_rage: rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f4prep_a, death: "ms_lose",
        }),
        ("fight_f4_prep_b", FightCfg {
            name: "兽潮·决死·之二", hp: 88, dmg: (17, 26), reward: 35, reward_why: "决死兽潮 · 挡下",
            intro: "又一批决死兽群不要命地扑向信标塔。",
            rage_at: Some(42), rage_text: "", on_rage: rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f4prep_b, death: "ms_lose",
        }),
        ("fight_r_siege_beast", FightCfg {
            name: "狂化攻城巨兽", hp: 150, dmg: (20, 30), reward: 700, reward_why: "绝地清场 · 击破狂化攻城巨兽",
            intro: "F4 东墙轰然崩裂——三米高的装甲鳞甲巨兽破墙登台，断裂的犄角喷着火星，口器滴落的岩浆在甲板烫出焦痕。",
            rage_at: Some(100), rage_text: "<b>破城狂暴</b>——巨兽震天怒吼，城墙塌陷段掩体失效，熔火甲片剥落，伤害暴涨！",
            on_rage: rage_reinforce,
            finisher_if: |st, _ehp| st.flag("orbital_auth") && st.flag("ms_beacon_ready"),
            finisher_name: |st| {
                if st.flag("orbital_auth") && st.flag("ms_beacon_ready") { "轨道支援 · 绝地清场".into() } else { "全力一击".into() }
            },
            finisher_desc: |_st| {
                "你按亮信标塔的授权终端。「轨道支援，坐标标定完毕。」\
                 \n天幕裂开一道细缝——轨道武器(T: 轨道炮)落下一束贯穿天地的白虹，正中巨兽。\
                 \n<b>120 点固定伤害</b>贯穿它的熔火甲片，场上所有增援兽潮在同一瞬被白光吞没。".into()
            },
            win: |st| {
                // 胜利路由：先进入评级写入场景（Route::Dyn 内联 st.sp_grade），再展示结算卡
                if st.flag("orbital_auth") && st.flag("ms_beacon_ready") {
                    "ms_settle_orbital_setup".into()
                } else {
                    "ms_settle_hard_setup".into()
                }
            },
            death: "ms_lose",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn moshi_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    moshi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/// 剧情场景（id 全部 ms_ 前缀，bg 用已部署背景文件，BOSS 立绘 enemy_siege_beast.png）
pub static MOSHI_SCENES: &[SceneDef] = &[

/* ---- 幕一 · 开场「兽潮将至」（F1 城墙黄昏） ---- */
SceneDef {
    id: "ms_00", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城墙平台"),
    mood: "danger", speaker: Some("主神"), voice: Some("vo_moshi_commander_tide"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>兽潮将至，人类的时代只剩下最后四面墙。受死城守军委托，撑过『第七次兽潮』直到轨道支援清场。",
        "防空警报在城墙上方拉响——三短一长。指挥所广播里挤出一句话：「第七次兽潮通报：三十七分钟——我们只有三十七分钟。」",
        "民兵在城墙上整列，远地平线烟尘翻滚，像一道灰黑的潮在向这座城滚动——这是人类自己的主场。",
    ]),
    choices: &[
        ChoiceDef { label: "【即刻上城墙协防】", sub: "点数+15 · 领武器 · 触发首波战斗", cond: None,
            effects: &[Eff::Points(15), Eff::SetFlag("f1_first_combat"), Eff::Weapon(crate::state::Weapon::Axe)], route: Route::To("ms_combat_a") },
        ChoiceDef { label: "【先奔军需配给站领弹药】", sub: "加护: ammo_ready（首战火力）", cond: None,
            effects: &[Eff::Points(10), Eff::AddItem("ammo_crate"), Eff::SetFlag("ammo_ready"), Eff::Weapon(crate::state::Weapon::Axe)], route: Route::To("ms_supply") },
        ChoiceDef { label: "【我是轮回者，先摸清守备再做决断】", sub: "需已通关蜂巢 · 点数+20", cond: Some(cond_hive_cleared),
            effects: &[Eff::Points(20), Eff::SetFlag("recon_done"), Eff::Weapon(crate::state::Weapon::Axe)], route: Route::To("ms_recon") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 军需配给站领弹药后返回 */
SceneDef {
    id: "ms_supply", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 军需配给站"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "军需配给站的民兵把一箱弹药塞进你怀里：「压在城墙脚下，能多撑一发是一发。去吧，城墙那边顶不住了。」",
        "（配给站调查已完成；首战的弹药火力已备——ammo_ready）",
    ]),
    choices: &[ChoiceDef { label: "回城墙协防", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ms_combat_a") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 侦察路线 */
SceneDef {
    id: "ms_recon", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口 · 侦察"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你翻上城头一段哨位，望远镜里看清了兽潮的来向：它们正沿着城门广场的缺口，两翼都是沼气味的兽群。",
        "（recon_done：你掌握了兽潮路线，已知 gate_city_gate 的焊死分支选项）",
    ]),
    choices: &[ChoiceDef { label: "回城墙协防", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ms_combat_a") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- F1 城门口广场 · 多波次战斗（战斗场景链：波1→波2→波3→过关） ---- */
SceneDef {
    id: "ms_combat_a", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口广场"),
    mood: "danger", speaker: Some("⚔ 第一波"), voice: None,
    text: TextSpec::Static(&["第一波兽潮小队冲过城门缺口——守军的战线压力骤增。你迎上去补上缺口。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "兽潮小队", cond: None, effects: &NO_EFF, route: Route::To("ms_fight_a") }],
    fight_id: Some("fight_f1_wave_a"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_fight_a", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口广场"),
    mood: "danger", speaker: Some("⚔ 决胜波之二"), voice: None,
    text: TextSpec::Static(&["兽潮小队被击散，但紧接着一头跳扑兽从血雾里跃出——"]),
    choices: &[ChoiceDef { label: "迎战", sub: "跳扑兽", cond: None, effects: &NO_EFF, route: Route::To("ms_fight_b") }],
    fight_id: Some("fight_f1_wave_b"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_fight_b", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口广场"),
    mood: "danger", speaker: Some("⚔ 决胜波之三"), voice: None,
    text: TextSpec::Static(&["跳扑兽倒下的同一瞬，两头发狂暴的兽兵从瓦砾后扑出，把最后一波填进缺口。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "兽兵+兽兵", cond: None, effects: &NO_EFF, route: Route::To("ms_02") }],
    fight_id: Some("fight_f1_wave_c"), video: None, cine_label: None, overlay: None,
},
/* ---- F1 波战间休息回复节点（全回满血，避免连续战阵亡） ---- */
SceneDef {
    id: "ms_rest_wave1", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口广场 · 防火门后"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["兽潮小队被你打退的片刻，你退进一扇没有堵死的防火门后，抓紧喘息、把止血带缠紧伤口。" ]),
    choices: &[ChoiceDef { label: "（休整回满血，再战）", sub: "HP 回满", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "ms_fight_a".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_rest_wave2", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口广场 · 瓦砾掩体后"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["跳扑兽刚被击退，你借着最后一层瓦砾掩体狠吸一口气，把体力拉到极限再冲出去。" ]),
    choices: &[ChoiceDef { label: "（休整回满血，再战）", sub: "HP 回满", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "ms_fight_b".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 第一波战后：焊死城门闸 */
SceneDef {
    id: "ms_02", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城门口广场 · 战后"),
    mood: "cold", speaker: Some("民兵队长"), voice: None,
    text: TextSpec::Static(&[
        "第一波兽潮被钉死在城门缺口。民兵队长满身是土：「把闸门焊死——不能让它们疏通城墙东西两段的直线火线。」",
        "（city_gate_welded：城门闸焊死，城墙平台东西段连成防线；退路由内街主街兜底）",
    ]),
    choices: &[ChoiceDef { label: "（焊死闸门）", sub: "city_gate_welded · 点数+30 · 休整回血", cond: None,
        effects: &[Eff::SetFlag("city_gate_welded"), Eff::Points(30)],
        route: Route::Dyn(|st: &mut GameState| { st.hp = (st.hp + 60).min(100); "ms_03_downtown".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_03_downtown", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 内街"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "城墙平台焊成一道铁壁。内街此刻被瓦砾和废墟切出两条路——",
        "主线要下沉去城内医院军火库（F2）；东南角还有通向城内的下城阶梯。",
    ]),
    choices: &[
        ChoiceDef { label: "调查防空警报塔", sub: "支线 siren_recorded", cond: None,
            effects: &NO_EFF, route: Route::To("ms_siren") },
        ChoiceDef { label: "调查废弃巴士", sub: "撬棍来源①", cond: None,
            effects: &NO_EFF, route: Route::To("ms_bus") },
        ChoiceDef { label: "从城墙升降梯下沉 F2", sub: "Portal F1→F2", cond: None,
            effects: &NO_EFF, route: Route::To("ms_enter_f2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* F1 调查点 */
SceneDef {
    id: "ms_siren", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 防空警报塔"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "警报塔的喇叭还挂着型号牌，塔基的金属铭牌上刻着历次兽潮的间隔规律。",
        "你把规律抄进记忆——后续波次的预警都有了依据（siren_recorded）。",
    ]),
    choices: &[ChoiceDef { label: "（记录兽潮规律）", sub: "支线 siren_recorded · 点数+200（结算）", cond: None,
        effects: &[Eff::SetFlag("siren_recorded"), Eff::Points(15)], route: Route::To("ms_03_downtown") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_bus", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 废弃巴士"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["废弃巴士的座椅倒了一地。驾驶座下的工具箱里插着一把撬棍——（撬棍来源①）。"]),
    choices: &[ChoiceDef { label: "收下撬棍", sub: "Item crowbar", cond: None,
        effects: &[Eff::AddItem("crowbar"), Eff::Points(10)], route: Route::To("ms_03_downtown") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_enter_f2", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城墙升降梯(下)"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["城墙升降梯的吊索早就锈死，此刻挂着一只铁筐缓缓放入城内。你跨进去，向 F2 医院落下。"]),
    choices: &[ChoiceDef { label: "下沉 F2", sub: "Portal F1→F2", cond: None, effects: &NO_EFF, route: Route::To("ms_f2_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕二 · 关键转折「弹尽粮绝」（F2 医院） ---- */
SceneDef {
    id: "ms_f2_arrive", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 急诊大厅"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你落在 F2 医院的急诊大厅。落日透过破碎的玻璃倾泻进来，走廊深处传来低吼与拖动声。",
        "军医的声音从药房方向传来：「药房和病房都进了东西——你们是城外来的吗？」",
    ]),
    choices: &[
        ChoiceDef { label: "【随军医救护士】", sub: "战斗 fight_f2_mutant · 达成 medic_trusted", cond: None,
            effects: &NO_EFF, route: Route::To("ms_medic_fight") },
        ChoiceDef { label: "【先夺军火库要紧】", sub: "跳过救护士 → medic_failed · 进 F2 中枢", cond: None,
            effects: &[Eff::SetFlag("medic_failed")], route: Route::To("ms_f2_hub") },
        ChoiceDef { label: "调查病房（钥匙①）", sub: "office_key 来源①", cond: None,
            effects: &NO_EFF, route: Route::To("ms_ward") },
        ChoiceDef { label: "调查药房", sub: "急救包", cond: None,
            effects: &NO_EFF, route: Route::To("ms_pharmacy") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_medic_fight", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 病房 · 救护士"),
    mood: "danger", speaker: Some("⚔ 救护士"), voice: None,
    text: TextSpec::Static(&["你随军医冲进病房。一只缝合肢体的医疗变异体正围着一个缩在床下的护士——救下她！"]),
    choices: &[ChoiceDef { label: "迎战", sub: "医疗变异体", cond: None, effects: &NO_EFF, route: Route::To("ms_f2_prep_b") }],
    fight_id: Some("fight_f2_mutant_wave"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_rest_f2", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 病房 · 墙根"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["第一只变异体被你放倒。你借护士床单撕成的绷带把伤口兜住，喘匀一口气再扑向异动传来的方向。" ]),
    choices: &[ChoiceDef { label: "（休整回满血，再战）", sub: "HP 回满", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "ms_f2_prep_b".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_f2_prep_b", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 病房 · 救护士"),
    mood: "danger", speaker: Some("⚔ 救护士·之二"), voice: None,
    text: TextSpec::Static(&["第一只变异体倒下，病房深处又撕开帘布钻出一只，直追缩在床下的护士扑去。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "医疗变异体·2", cond: None, effects: &NO_EFF, route: Route::To("ms_medic_win") }],
    fight_id: Some("fight_f2_mutant_wave2"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_medic_win", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 病房"),
    mood: "cold", speaker: Some("军医"), voice: Some("vo_moshi_doctor"),
    text: TextSpec::Dyn(|st| {
        if st.flag("medic_trusted") {
            "护士已经给你包扎过伤口。军医点点头：「医院东门我开了，去军火库搬家伙吧——城楼上，还有血要流。」".into()
        } else {
            "护士被你从变异体身下拉了出来，吓得直发抖，却还是朝你挤出笑。军医感激地拍拍你肩：「这份情我记下了——医院东门、还有军火库，我替你通路。」".into()
        }
    }),
    choices: &[
        ChoiceDef { label: "（信任军医）", sub: "medic_trusted · 幸存者+1 · 回血", cond: None,
            effects: &[Eff::SetFlag("medic_trusted"), Eff::SetFlag("ms_sur_n1"), Eff::Points(60)],
            route: Route::Dyn(|st: &mut GameState| { st.hp = (st.hp + 60).min(100); "ms_f2_hub".into() }) },
        ChoiceDef { label: "调查上尉办公室", sub: "需 office_key", cond: Some(cond_has_office_key),
            effects: &NO_EFF, route: Route::To("ms_captain") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* F2 调查点 */
SceneDef {
    id: "ms_ward", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 病房"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["病床底压着半截钥匙——那是上尉办公室的门钥匙（office_key 来源①）。"]),
    choices: &[ChoiceDef { label: "收下 office_key", sub: "Item office_key", cond: None,
        effects: &[Eff::AddItem("office_key"), Eff::SetFlag("ms_sur_n2"), Eff::Points(10)], route: Route::To("ms_f2_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_pharmacy", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 药房"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["药房急救架上还有几盒完好的急救包。"]),
    choices: &[ChoiceDef { label: "取走急救包", sub: "Item gj_medkit · 回复 30", cond: None,
        effects: &[Eff::AddItem("gj_medkit"), Eff::Points(10)], route: Route::To("ms_f2_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_captain", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 上尉办公室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你用 office_key 打开上尉办公室。抽屉里除了几张撤离名单，还有一张军火库钥匙卡。",
        "（office_secret：撤离名单——影响幸存者目标；keycard_armory：解锁军火库）",
    ]),
    choices: &[
        ChoiceDef { label: "取走钥匙卡", sub: "Item keycard_armory", cond: None,
            effects: &[Eff::AddItem("keycard_armory")], route: Route::To("ms_captain2") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_captain2", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 上尉办公室"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你在撤离名单上划掉了几道名字，又补了几笔新的。这份档案被收进贴身的衣袋（office_secret）。"]),
    choices: &[ChoiceDef { label: "（归档）", sub: "支线 office_secret", cond: None,
        effects: &[Eff::SetFlag("office_secret"), Eff::Points(20)], route: Route::To("ms_f2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* F2 地图中枢 */
SceneDef {
    id: "ms_f2_hub", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 中央广场"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["F2 中央广场的血泊里躺着刚刚击破的兽潮。军火库在军火库区(S 高墙)，控制柜在广场东侧；下 F3 的机要楼梯在货站东南角。"]),
    choices: &[
        ChoiceDef { label: "打开军火库（需 keycard_armory）", sub: "军火库重火力", cond: Some(cond_has_keycard),
            effects: &[Eff::SetFlag("armory_opened"), Eff::Points(30), Eff::AddItem("gj_pistol"), Eff::AddItem("gj_grenade")],
            route: Route::To("ms_armory") },
        ChoiceDef { label: "调查电梯控制柜", sub: "cell_restored · 回程电梯", cond: None,
            effects: &[Eff::SetFlag("cell_restored")], route: Route::To("ms_cell") },
        ChoiceDef { label: "下沉 F3 指挥所", sub: "Portal F2→F3", cond: None,
            effects: &NO_EFF, route: Route::To("ms_f3_arrive") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_armory", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 军火库"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["军火库铁架上是成排的重火力：高斯手枪、燃烧手雷，还有一箱没开封的弹药。这一仗的底气回来了。"]),
    choices: &[ChoiceDef { label: "（武装）", sub: "高斯手枪+燃烧手雷 ×2", cond: None,
        effects: &[Eff::Weapon(crate::state::Weapon::Gun), Eff::Points(20)], route: Route::To("ms_f2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_cell", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 电梯控制柜"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你扳下电梯控制柜的熔丝——一声清响，全楼电梯恢复了供电（cell_restored）。回城墙上补给的路通了。"]),
    choices: &[ChoiceDef { label: "（恢复供电）", sub: "cell_restored · 回程电梯可用", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("ms_f2_hub") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_f3_arrive", bg: Some("scene_moshi_command.png"), loc: Some("F3 · 机要楼梯间"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["机要楼梯一路下沉，你踏进地下指挥所。应急灯忽明忽暗，观测屏的蓝光照亮老指挥官佝偻的背影。"]),
    choices: &[
        ChoiceDef { label: "【重启反应堆，换取授权】", sub: "需撬棍 → power_restored + orbital_auth", cond: Some(cond_has_crowbar),
            effects: &NO_EFF, route: Route::To("ms_reactor") },
        ChoiceDef { label: "与老指挥官对话", sub: "授权码 / 轨道支援剧情", cond: None,
            effects: &NO_EFF, route: Route::To("ms_comander") },
        ChoiceDef { label: "调查通讯阵列", sub: "需 power_restored → beacon_log", cond: Some(cond_power_restored),
            effects: &NO_EFF, route: Route::To("ms_comms") },
        ChoiceDef { label: "上 F4 炮台（硬扛）", sub: "需 orbital_auth 才放行防爆电梯", cond: Some(cond_orbital_auth),
            effects: &NO_EFF, route: Route::To("ms_enter_f4") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 反应堆 */
SceneDef {
    id: "ms_reactor", bg: Some("scene_moshi_command.png"), loc: Some("F3 · 反应堆配电室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你用撬棍别开防辐射门，钻进配电室。高压柜的制动卡死在一个开裂的位置——你手动合闸。",
        "嗡——整座地下指挥所的灯光由红转白，观测屏重新亮起全息地图（power_restored）。",
    ]),
    choices: &[ChoiceDef { label: "（重启反应堆）", sub: "power_restored · 点数+40", cond: None,
        effects: &[Eff::SetFlag("power_restored"), Eff::Points(40)], route: Route::To("ms_reactor_done") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_reactor_done", bg: Some("scene_moshi_command.png"), loc: Some("F3 · 指挥所"),
    mood: "cold", speaker: Some("老指挥官"), voice: Some("vo_moshi_commander_auth"),
    text: TextSpec::Static(&[
        "老指挥官看着重新点亮的观测屏，沉默半晌，把一张薄薄的授权卡推到你面前：「轨道支援只有一次。要么现在清场，要么留着等那个破墙的巨兽爬上来。」",
        "（orbital_auth：你拿到了轨道支援授权）",
    ]),
    choices: &[ChoiceDef { label: "（收下授权码）", sub: "orbital_auth · 加入称号 · 回血", cond: None,
        effects: &[Eff::SetFlag("orbital_auth"), Eff::Points(20)],
        route: Route::Dyn(|st: &mut GameState| { st.hp = (st.hp + 60).min(100); "ms_f3_arrive".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 指挥官对话 */
SceneDef {
    id: "ms_comander", bg: Some("scene_moshi_command.png"), loc: Some("F3 · 指挥中枢"),
    mood: "danger", speaker: Some("老指挥官"), voice: None,
    text: TextSpec::Dyn(txt_comander),
    choices: &[
        ChoiceDef { label: "「先上炮台硬扛」", sub: "无授权 → 直接登 F4（高难路线）", cond: None,
            effects: &[Eff::SetFlag("ms_no_orbital"), Eff::Points(10)], route: Route::To("ms_enter_f4") },
        ChoiceDef { label: "（有授权后）登记授权", sub: "已授权", cond: Some(cond_orbital_auth),
            effects: &NO_EFF, route: Route::To("ms_f3_arrive") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 通讯阵列 */
SceneDef {
    id: "ms_comms", bg: Some("scene_moshi_command.png"), loc: Some("F3 · 通讯阵列"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "通讯阵列的频谱上，一段加密信号反复闪烁——那是一段『盒外信号』。你把参数记下来：观测箱体的参数异常，读数与这个世界常模存在偏差。",
        "（beacon_log：Z 宇宙真相线长线伏笔「盒子观测参数异常」）",
    ]),
    choices: &[ChoiceDef { label: "（记录盒外信号）", sub: "支线 beacon_log · 点数+200（结算）", cond: None,
        effects: &[Eff::SetFlag("beacon_log"), Eff::Points(20)], route: Route::To("ms_f3_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 上 F4 */
SceneDef {
    id: "ms_enter_f4", bg: Some("scene_moshi_command.png"), loc: Some("F3 · 防爆电梯门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["防爆电梯在授权绿灯开启后向两侧滑开。轨道一路向上，通往 F4 炮台观测台。"]),
    choices: &[ChoiceDef { label: "登炮台", sub: "Portal F3→F4", cond: None, effects: &NO_EFF, route: Route::To("ms_f4_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕三 · F4 炮台观测台 · 决战 ---- */
SceneDef {
    id: "ms_f4_arrive", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 顶层观测甲板"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你登上 F4 炮台观测台。夕照把整片死城染成橙灰，观测镜里，兽潮如蚁群般正沿着城墙攀爬。",
        "东墙方向传来轰鸣——烟尘里，一个三米高的身影正用犄角撞开城墙的最后一段。",
    ]),
    choices: &[
        ChoiceDef { label: "调查观测镜", sub: "兽潮总数 / 局势文本", cond: None,
            effects: &NO_EFF, route: Route::To("ms_scope") },
        ChoiceDef { label: "调查轨道信标塔", sub: "终结技前置", cond: None,
            effects: &[Eff::SetFlag("ms_beacon_ready")], route: Route::To("ms_beacon") },
        ChoiceDef { label: "在观测甲板迎敌", sub: "BOSS 战前清场兽潮 · 休整回血", cond: None,
            effects: &NO_EFF,
            route: Route::Dyn(|st: &mut GameState| { st.hp = (st.hp + 60).min(100); "ms_f4_prep".into() }) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_scope", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测镜"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你透过观测镜看去：四面墙里，最后一面还完整。兽潮的洪流正朝这一面集结——人类时代剩下的，只有这一面墙了。"]),
    choices: &[ChoiceDef { label: "返回甲板", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ms_f4_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_beacon", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 轨道信标塔"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(txt_beacon),
    choices: &[ChoiceDef { label: "返回甲板", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ms_f4_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* BOSS 前清场兽潮（战斗场景链：决死活兽 ×2） */
SceneDef {
    id: "ms_f4_prep", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板"),
    mood: "danger", speaker: Some("⚔ 决死反扑"), voice: None,
    text: TextSpec::Static(&["观测甲板四周的决死兽群在做最后的靠近——巨兽破墙之前，先把这些清掉。"]),
    choices: &[ChoiceDef { label: "迎战", sub: "兽潮·决死", cond: None, effects: &NO_EFF, route: Route::To("ms_f4_prep2") }],
    fight_id: Some("fight_f4_prep_a"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_rest_f4", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板 · 掩体后"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["第一波决死兽群被清空。你靠在主炮残件后面喝一口水，把体力拉满，等最后一批扑上来。" ]),
    choices: &[ChoiceDef { label: "（休整回满血，再战）", sub: "HP 回满", cond: None,
        effects: &NO_EFF,
        route: Route::Dyn(|st: &mut GameState| { st.hp = 100; "ms_f4_prep2".into() }) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_f4_prep2", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板"),
    mood: "danger", speaker: Some("⚔ 决死反扑·之二"), voice: None,
    text: TextSpec::Static(&["又一批决死兽群扑上甲板，它们不要命地冲向信标塔——挡下！"]),
    choices: &[ChoiceDef { label: "迎战", sub: "兽潮·决死", cond: None, effects: &NO_EFF, route: Route::To("ms_f4_boss_intro") }],
    fight_id: Some("fight_f4_prep_b"), video: None, cine_label: None, overlay: None,
},
/* BOSS 战引导 */
SceneDef {
    id: "ms_f4_boss_intro", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 东墙"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "东墙轰然崩裂。狂化攻城巨兽破墙登台，熔火甲片在夕照下发红——人类的时代只剩下最后四面墙，现在是第三面。",
        "整座城墙的掩体在它爪下塌陷，而你的身后，信标塔的红外射界已经对准了它。",
    ]),
    choices: &[ChoiceDef { label: "迎战狂化攻城巨兽", sub: "fight_r_siege_beast · BOSS", cond: None,
        effects: &NO_EFF, route: Route::To("ms_f4_boss") }],
    fight_id: None, video: Some("cin_f4_boss.mp4"), cine_label: Some("破城"), overlay: None,
},
SceneDef {
    id: "ms_f4_boss", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板 · 决战"),
    mood: "danger", speaker: Some("⚔ 绝地清场"), voice: Some("vo_moshi_beast_growl"),
    text: TextSpec::Static(&["巨兽低吼，口器滴落的岩浆在甲板烫出焦痕。你若已取得轨道授权并标定信标，便能呼叫轨道支援一笔清场。"]),
    choices: &[ChoiceDef { label: "决战", sub: "fight_r_siege_beast", cond: None, effects: &NO_EFF, route: Route::To("ms_boss_done") }],
    fight_id: Some("fight_r_siege_beast"), video: None, cine_label: None, overlay: None,
},
/* BOSS 战后结算导引（绝地清场 / 惨胜） */
SceneDef {
    id: "ms_boss_done", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板"),
    mood: "calm", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["巨兽轰然倒下，硝烟散尽。城下的兽潮在余晖里慢慢退去，守军此起彼伏地欢呼起来。"]),
    choices: &[ChoiceDef { label: "（看结算）", sub: "", cond: None,
        effects: &[Eff::AddItem("beast_core"), Eff::SetFlag("evac_hard")], route: Route::To("ms_32") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* ---- 绝地清场 / 惨胜 · 结算前置（Route::Dyn 内联写 sp_grade） ---- */
SceneDef {
    id: "ms_settle_orbital_setup", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "白虹冲天，狂化攻城巨兽灰飞烟灭。城下的兽潮在余晖里如退潮般散去，幸存者沿着索道开始撤离。",
        "你以轨道支援实现绝地清场——这一战，主神评定授予科技侧 C 级支线门票。",
    ]),
    choices: &[ChoiceDef {
        label: "（绝地清场 · 领取 C 级评级）", sub: "sp_grade C · evac_done · 巨兽晶核",
        cond: None,
        effects: &[Eff::AddItem("beast_core"), Eff::SetFlag("evac_done")],
        route: Route::Dyn(|st: &mut GameState| { st.sp_grade = Some('C'); "ms_30_orbital".into() }),
    }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_settle_hard_setup", bg: Some("scene_moshi_observatory.png"), loc: Some("F4 · 观测甲板"),
    mood: "danger", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "巨兽在甲板上倒伏——你硬生生把它磨死，没有轨道支援兜底。城墙半毁，部分幸存者损失，士气在血与火里燃到最后一刻。",
        "没有轨道清场的绝地，换来的是『战斗自豪』。",
    ]),
    choices: &[ChoiceDef {
        label: "（惨胜 · 记录）", sub: "evac_hard · last_stand_honor",
        cond: None,
        effects: &[Eff::AddItem("beast_core"), Eff::SetFlag("evac_hard"), Eff::SetFlag("last_stand_honor")],
        route: Route::To("ms_30_hard"),
    }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* 绝地清场（orbital 胜利结算） */
SceneDef {
    id: "ms_30_orbital", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "绝 地 清 场".into(), good: true,
            body_html: format!(
                "<p>白光贯穿甲板，巨兽在轨道打击下灰飞烟灭。被吞没的不只是它——你调用轨道清场的那一刻，场上所有增员兽潮都化为飞灰。</p>\
                 <p style='color:#ffd76a'>【主神】：绝地清场达成。幸存者沿着索道撤离，黎明剪影投在崩坏的城墙上——这是人类赢下的一夜。</p>\
                 <table class='statTable'>\
                 <tr><td>奖励点数（BOSS·绝地清场）</td><td>+700</td></tr>\
                 <tr><td>科技侧 C 级支线评级</td><td style='color:#ffd76a'>C 级门票</td></tr>\
                 <tr><td>掉落道具</td><td>巨兽晶核 beast_core（科技侧兑换抵扣物）</td></tr>\
                 </table>\
                 <p style='color:#8fd0a8'>剩余点数：{}　达成支线：绝地清场 evac_done</p>",
                st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
/* 惨胜（无 orbital）结算 */
SceneDef {
    id: "ms_30_hard", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "惨 胜 · 最后的墙".into(), good: true,
            body_html: format!(
                "<p>你硬生生把狂化攻城巨兽磨死在甲板上，没有轨道支援。城墙半毁，部分幸存者损失——没有一发轨道弹来兜底，这一夜格外漫长。</p>\
                 <p style='color:#ffd76a'>战斗自豪：last_stand_honor</p>\
                 <table class='statTable'><tr><td>奖励点数（BOSS·惨胜）</td><td>+700</td></tr></table>\
                 <p style='color:#8fd0a8'>剩余点数：{}　达成支线：惨胜 evac_hard（幸存者计数减一）</p>",
                st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- BOSS 战失败 → 死亡档案「战死城墙」→ 主神空间扣 500 复活 ---- */
SceneDef {
    id: "ms_lose", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("战死城墙", "在守城战中阵亡，回主神空间扣 500 点复活")),
        card: |_st| crate::state::Card {
            title: "战 死 城 墙".into(), good: false,
            body_html: r#"<p>城墙还在，兽潮还在爬。你倒下的地方，民兵把你的枪捡了起来——「城在，人在。」</p>
<p style='color:#ff8a8a'>【死亡档案 · 战死城墙】</p>
<p style='color:#666'>（复活：回主神空间扣 500 点复活；点数不足按 0 处理并挂『死亡债务』，下一副本结算优先偿债。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: Some("vo_moshi_cityman"),
        },
    }),
},

/* ---- F2 医院/杂项与幸存者线 ---- */
SceneDef {
    id: "ms_linen", bg: Some("scene_moshi_hospital.png"), loc: Some("F2 · 医院杂物间"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["杂物间折倒的急救架下塞着一根撬棍（撬棍来源②）。"]),
    choices: &[ChoiceDef { label: "收下撬棍", sub: "Item crowbar", cond: None,
        effects: &[Eff::AddItem("crowbar"), Eff::Points(10)], route: Route::To("ms_f2_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ms_landmark", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 废墟近道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["一处倒塌的废墟，需要撬棍才能撬开近道（gate_rubble）。"]),
    choices: &[ChoiceDef { label: "（撬开近道）", sub: "需 crowbar", cond: Some(cond_has_crowbar),
        effects: &[Eff::Points(10)], route: Route::To("ms_03_downtown") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* F1 民兵团对话（NPC） */
SceneDef {
    id: "ms_00_minuteman", bg: Some("scene_moshi_citywall_dusk.png"), loc: Some("F1 · 城墙平台"),
    mood: "danger", speaker: Some("民兵队长"), voice: Some("vo_moshi_minuteman_last"),
    text: TextSpec::Static(&["民兵队长把最后一箱弹药抬上阵地，哑着嗓子喊：「把这一发留给最大的那个家伙！」城墙上所有人抬头看向东方烟尘。"]),
    choices: &[ChoiceDef { label: "（继续协防）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ms_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 幸存者计数支线（图腾：救 3 名 → survivors_3） */
SceneDef {
    id: "ms_survivor3", bg: None, loc: None, mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你已累计救下 3 名守军幸存者——图腾点亮，撤离名单上的名字都划上了复活标记（survivors_3）。"]),
    choices: &[ChoiceDef { label: "（记录）", sub: "支线 survivors_3", cond: Some(cond_survivors_3),
        effects: &[Eff::SetFlag("survivors_3"), Eff::Points(200)], route: Route::To("ms_03_downtown") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
];

/// 本文件场景查询辅助（主线合并查询扩展时可直接使用）
pub fn moshi_scene(id: &str) -> Option<&'static SceneDef> {
    MOSHI_SCENES.iter().find(|s| s.id == id)
}
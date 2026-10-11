//! 《无限恐怖 · 生化危机 · 浣熊市》地面城市战副本 —— 全部剧情场景与战斗配置。
//! 钩子：「蜂巢在地下，而地狱在地上。」——与地下蜂巢主线区分：这里是丧尸围城、
//! 核弹倒计时、逃生求生的地面战场。零新引擎。
//! 本文件为全新新增文件，只导出静态数据（HUANXIONGSHI_SCENES / huanxiongshi_figths / 辅助查询），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/huanxiongshi_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `hx_` 前缀，fight id 全部 `hx_` 前缀。
//! BOSS「暴君 Tyrant」采用【选择驱动遭遇链】落地（参考 scenes_yize.rs 仲裁者模式）：血量存 st.fight，
//! 用 Normal 场景 + Route::Dyn 落地，每回合同调伤害/受击/狂暴/终结。城郊尸潮为强敌原生战斗。
//! 核弹倒计时：flag 伪实现（hx_n1/hx_n2/hx_n3 计数器，时间到 → hx_nuke_death 死亡档案）。
//! 抉择结局三分支（逃离 / 乘机 / 留下），持 tab 各写 sp_grade Some('D')。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   街道/警局 bg img_corridor.png（占位）；城郊/尸潮 bg img_horde.png（占位）。

use crate::defs::*;
use crate::state::GameState;
use rand::Rng;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   具名条件谓词（cond：fn 指针，不能捕获闭包）
   ===================================================================== */
fn cond_has_gatekey(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "it_hx_gatekey") }
fn cond_radio_done(st: &GameState) -> bool { st.flag("hx_radio_done") }

/* =====================================================================
   核弹倒计时 · flag 伪实现（hx_n1/n2/n3 → 时间到 = 死亡）
   ===================================================================== */
fn nuke_t(st: &GameState) -> i32 {
    ["hx_n1", "hx_n2", "hx_n3"].iter().filter(|k| st.flag(k)).count() as i32
}

/// 稍作停留：倒计时推进；第 3 次停留 → 核爆（死亡档案）。
fn nuke_pause(st: &mut GameState) -> String {
    let t = nuke_t(st);
    match t {
        0 => { st.set_flag("hx_n1"); "hx_nuke_room".to_string() }
        1 => { st.set_flag("hx_n2"); "hx_nuke_room".to_string() }
        _ => { st.set_flag("hx_n3"); "hx_nuke_death".to_string() }
    }
}

/* =====================================================================
   BOSS · 暴君 Tyrant（选择驱动 HP200）
   ===================================================================== */
fn start_tyrant(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("hx_tyrant") {
            st.fight = Some(crate::power::scaled_fight("hx_tyrant", cfg, st, vec![format!("<span class='miss'>{}</span>", cfg.intro)]));
        }
    }
    "hx_tyrant_round".to_string()
}

/// 暴君胜场结算：+400、主神沙鹰、sp_grade=D、置 killed。
fn tyrant_win(st: &mut GameState) -> String {
    st.points += 400;
    crate::world::add_item(st, "hx_tyrant_trophy");
    st.sp_grade = Some('D');
    st.set_flag("hx_tyrant_down");
    "hx_f1_boss_defeated".to_string()
}

/// 暴君一个"回"：玩家攻击（dmg）。狂暴后伤害提升；guard 免伤。
fn tyrant_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return tyrant_win(st);
    }
    // 狂暴判定
    let raged = st.fight.as_ref().map(|f| f.hp <= 80).unwrap_or(false);
    if raged { if let Some(f) = st.fight.as_mut() { f.raged = true; } }
    let rg = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if rg { 26 } else { 18 };
    if !guard {
        let roll: f64 = rand::thread_rng().gen();
        if roll >= 0.15 { st.hp = (st.hp - raw).max(0); }
    }
    if st.hp <= 0 { return "hx_lose_boss".to_string(); }
    // 暴君终结：狂暴后期 HP<30 的弱点打击
    let low = st.fight.as_ref().map(|f| f.hp < 30).unwrap_or(false);
    if rg && low { return "hx_tyrant_finisher".to_string(); }
    "hx_tyrant_round".to_string()
}
fn tyrant_strike(st: &mut GameState) -> String { tyrant_act(st, 30, false) }
fn tyrant_heavy(st: &mut GameState) -> String { tyrant_act(st, 38, false) }
fn tyrant_guard(st: &mut GameState) -> String { tyrant_act(st, 0, true) }

/// 终结：以近战致命一击结果暴君。
fn tyrant_finish(st: &mut GameState) -> String {
    st.points += 100;
    tyrant_win(st)
}

/* =====================================================================
   结局结算路由
   ===================================================================== */
fn end_escape(st: &mut GameState) -> String { st.sp_grade = Some('D'); "hx_settle_card".to_string() }
fn end_heli(st: &mut GameState) -> String { st.sp_grade = Some('D'); "hx_settle_card".to_string() }
fn end_stay(st: &mut GameState) -> String { st.sp_grade = Some('D'); "hx_settle_card".to_string() }

/* =====================================================================
   剧情场景（id 全部 hx_ 前缀）
   ===================================================================== */
pub static HUANXIONGSHI_SCENES: &[SceneDef] = &[

    /* ------------------- 幕一 · 降临警局（F1） ------------------- */
    SceneDef {
        id: "hx_00", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 RPD 警局 · 主厅"),
        mood: "danger", speaker: Some("主神空间（回响）"), voice: None,
        text: TextSpec::Static(&[
            "<b>【任务】</b>浣熊市爆发 T 病毒，NeST 已启动「灭菌」核弹，倒计时不到一小时。活过核爆，或死在这里。",
            "你从光柱中跌入 RPD 警局残破的主厅。窗外火光冲天，玻璃在风中呜咽——蜂巢在地下，而地狱，在地上。",
            "「蜂巢的账，得在这座城还……」郑咤站在尸体堆里，枪口还没放下。他瞥你一眼，「想活？就跟着跑。」",
        ]),
        choices: &[
            ChoiceDef { label: "（走上前，查看警局大厅）", sub: "进入警局主厅", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_hub", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 RPD 警局 · 主厅"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "大厅被翻得底朝天，墙上的警徽还在。丧尸的低吼从四楼隐约传来。你要在核弹落下前找到出路。",
            "（探索各处收集钥匙/补给，或直接深入下层迎战破墙而出的暴君。）",
        ]),
        choices: &[
            ChoiceDef { label: "撬开枪械保险柜", sub: "取得 哨所钥匙·手枪", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_lockup") },
            ChoiceDef { label: "翻找医务室", sub: "取得 急救包", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_med") },
            ChoiceDef { label: "接通对讲电台", sub: "情报", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_radio") },
            ChoiceDef { label: "读警长日志", sub: "+20 点", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_log") },
            ChoiceDef { label: "推开停尸间", sub: "线索", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_morgue") },
            ChoiceDef { label: "与沈哲交谈", sub: "幸存警员", cond: None,
                effects: &NO_EFF, route: Route::To("hx_n_shen") },
            ChoiceDef { label: "爬上市政屋顶", sub: "俯瞰火海", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_roof") },
            ChoiceDef { label: "深入停尸间下层 · 暴君区", sub: "迎战 暴君 Tyrant", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_boss_zone") },
            ChoiceDef { label: "从警局侧门突围 · F2", sub: "进入燃烧街道", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f2_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_lockup", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 枪械保险柜"),
        mood: "tension", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["警员的尸体歪在柜前，钥匙还插在锁孔里。柜内躺着一把警用手枪，和一串通往城郊哨所的铁质钥匙。"]),
        choices: &[ChoiceDef { label: "取走手枪与哨所钥匙", sub: "得 手枪/哨所钥匙(+20)", cond: None,
            effects: &[Eff::AddItem("it_hx_pistol"), Eff::AddItem("it_hx_gatekey"), Eff::Points(20)],
            route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_med", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 医务室"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["医务室的柜门被撬开一半，几卷绷带和一支肾上腺针露出来。沈哲靠在门口喘气：「保命要紧。」"]),
        choices: &[ChoiceDef { label: "收下急救包", sub: "得 急救包 · San(+5)", cond: None,
            effects: &[Eff::AddItem("it_hx_medkit"), Eff::San(5)], route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_radio", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 对讲电台"),
        mood: "mystery", speaker: Some("电台（嘶哑人声）"), voice: None,
        text: TextSpec::Static(&[
            "电台嘶哑地响着：「……城郊……铁丝网外，军方有最后一批直升机……他们说要把能带走的都带走，其余的……留给核弹。」",
            "「如果你手里有哨所的钥匙——往西，冲过路障，还有一架直升机等你。」信号戛然而止。",
        ]),
        choices: &[ChoiceDef { label: "记下直升机情报", sub: "情报 · San(-5)(+20)", cond: None,
            effects: &[Eff::SetFlag("hx_radio_done"), Eff::San(-5), Eff::Points(20)], route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_log", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 警长日志"),
        mood: "mystery", speaker: Some("警官·艾恩斯（日记）"), voice: None,
        text: TextSpec::Static(&["日志最后几页写满了潦草的笔迹：警局沦陷、街道沦陷、……「他们说是蜂巢泄的毒。可蜂巢在地下。那这地上的地狱，是谁造的？」"]),
        choices: &[ChoiceDef { label: "合上日志", sub: "+20 点", cond: None,
            effects: &[Eff::Points(20)], route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_morgue", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 停尸间"),
        mood: "fear", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["冷柜的门被猛地顶开，数具腐尸瘫在地上——他们是第一批倒在病毒下的警察。角落里一张城市地图标出了一条通往城郊的捷径。"]),
        choices: &[ChoiceDef { label: "记下捷径", sub: "情报 · San(-5)", cond: None,
            effects: &[Eff::SetFlag("hx_morgue_done"), Eff::San(-5)], route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_roof", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 警局屋顶"),
        mood: "awe", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["夜风卷着灰烬。整个浣熊市都在燃烧，街道上蠕动着一片片身影。极远处，一架直升机拖着火光坠落——那是活人毫无希望地想要逃离。"]),
        choices: &[ChoiceDef { label: "收回目光", sub: "San(-5)", cond: None,
            effects: &[Eff::San(-5)], route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_n_shen", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 大厅 · 沈哲旁"),
        mood: "tension", speaker: Some("沈哲 · 幸存警员"), voice: None,
        text: TextSpec::Static(&["沈哲握着一把弹匣见底的手枪，声音沙哑：「我能活到现在，凭的是两条：不走回头路，不回头看死人。」他顿了顿，「城郊那架直升机，是唯一的活路——但你得先能冲过街上的那群东西。」"]),
        choices: &[ChoiceDef { label: "（回到大厅）", sub: "", cond: None,
            effects: &NO_EFF, route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    /* ---- 暴君 Tyrant（选择驱动 BOSS，血存 st.fight） ---- */
    SceneDef {
        id: "hx_f1_boss_zone", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 地下机库 · 暴君破墙"),
        mood: "danger", speaker: Some("暴君 Tyrant"), voice: None,
        text: TextSpec::Static(&[
            "你刚踏入地下机库的阴影，身后的水泥墙轰然炸开。一个逾两米的身影撕开碎屑缓步走出——灰色风衣、绷带勒紧的肌肉、猩红的独眼。",
            "「It's Tyrant.」郑咤低吼，「别硬接，找它的罩门——它狂了以后再打要害！」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战暴君】", sub: "进入选择驱动战", cond: None,
                effects: &NO_EFF, route: Route::Dyn(start_tyrant) },
            ChoiceDef { label: "避开暴君，退回大厅", sub: "规避", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f1_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_tyrant_round", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 地下机库 · vs 暴君"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            let rg = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
            format!("暴君 HP {} / 你 HP {}。{}",
                st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp,
                if rg { "它已狂暴——风衣撕裂，肌肉暴突，要寻找致命罩门！" } else { "它缓步逼近，猩红独眼锁定你。" })
        }),
        choices: &[
            ChoiceDef { label: "重击", sub: "命中 30", cond: None, effects: &NO_EFF, route: Route::Dyn(tyrant_strike) },
            ChoiceDef { label: "蓄力强攻", sub: "命中 38（更慢）", cond: None, effects: &NO_EFF, route: Route::Dyn(tyrant_heavy) },
            ChoiceDef { label: "防御", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(tyrant_guard) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_tyrant_finisher", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 地下机库 · 致命一击"),
        mood: "danger", speaker: Some("暴君 Tyrant"), voice: None,
        text: TextSpec::Static(&["暴君单膝跪地，绷带下露出苍白的核心。它仰头嘶吼，猩红独眼浑浊地扫过你。你看见了破绽——一记对准核心的致命穿刺。"]),
        choices: &[ChoiceDef { label: "【致命一击】", sub: "终结暴君", cond: None,
            effects: &NO_EFF, route: Route::Dyn(tyrant_finish) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f1_boss_defeated", bg: Some("huanxiongshi_bg.png"), loc: Some("F1 地下机库 · 暴君已倒下"),
        mood: "choice", speaker: Some("郑咤"), voice: None,
        text: TextSpec::Static(&["暴君轰然倒下，机库在余震中扬起灰尘。郑咤擦了把血：「不错的爆发。不过核弹可不等我们——走。」"]),
        choices: &[ChoiceDef { label: "（返回大厅 · 突围）", sub: "得 暴君战斗勋章", cond: None,
            effects: &NO_EFF, route: Route::To("hx_f1_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 幕二 · 燃烧街道（F2） ------------------- */
    SceneDef {
        id: "hx_f2_arrive", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 街道 · 岔路口"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["你从警局侧门钻入燃烧的街道。翻覆的警车横在路心，尸群在店铺之间游荡。极西的路障后，是通往城郊的最后一段路。"]),
        choices: &[
            ChoiceDef { label: "（进入街道枢纽）", sub: "遭遇丧尸犬", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f2_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f2_dog_fight", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 街道 · 丧尸犬群"),
        mood: "fear", speaker: Some("丧尸犬"), voice: None,
        text: TextSpec::Static(&["脱缰的丧尸犬从车底扑向你！它们的獠牙浸着黑色的毒涎。"]),
        choices: &NO_CH, fight_id: Some("hx_dog"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f2_hub", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 街道枢纽"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["燃烧的街巷呈网格铺开。艾彬躲在翻覆的警车后面打手势，郑咤则守着角落清扫尸群。城郊路障在极西——需要哨所钥匙才升得起。"]),
        choices: &[
            ChoiceDef { label: "搜翻覆警车", sub: "补给", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_car") },
            ChoiceDef { label: "翻便利店货架", sub: "另一把钥匙", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_store") },
            ChoiceDef { label: "进废弃咖啡馆", sub: "San", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_cafe") },
            ChoiceDef { label: "清扫街上的丧尸犬", sub: "战斗", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_dog_fight") },
            ChoiceDef { label: "查看路障与广播", sub: "情报", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_barricade") },
            ChoiceDef { label: "与艾彬交谈", sub: "轮回者", cond: None, effects: &NO_EFF, route: Route::To("hx_n_aibin") },
            ChoiceDef { label: "与郑咤交谈", sub: "队长", cond: None, effects: &NO_EFF, route: Route::To("hx_n_zhou") },
            ChoiceDef { label: "冲向城郊路障", sub: "需 哨所钥匙", cond: Some(cond_has_gatekey), effects: &NO_EFF, route: Route::To("hx_f3_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f2_car", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 翻覆警车"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["警车后备箱被撞开，散落着几卷绷带与一支完好无损的肾上腺素针。"]),
        choices: &[ChoiceDef { label: "取走急救包", sub: "得 急救包 · San(+5)", cond: None,
            effects: &[Eff::AddItem("it_hx_medkit"), Eff::San(5)], route: Route::To("hx_f2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f2_store", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 便利店货架"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["货架被洗劫一空，却在收银台下的暗格里躺着一枚城郊哨所的钥匙——大概是哪个警员私藏的。"]),
        choices: &[ChoiceDef { label: "取走哨所钥匙", sub: "得 哨所钥匙", cond: None,
            effects: &[Eff::AddItem("it_hx_gatekey")], route: Route::To("hx_f2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f2_cafe", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 废弃咖啡馆"),
        mood: "mystery", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["咖啡香气早已被血腥取代。墙上涂满求救的血字，角落里半杯凉透的咖啡旁，坐着一具握拳的干尸——它到死都护着桌上的全家福。"]),
        choices: &[ChoiceDef { label: "轻掩双眼", sub: "San(-5) · +20 点", cond: None,
            effects: &[Eff::San(-5), Eff::Points(20)], route: Route::To("hx_f2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f2_barricade", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 路障 · 广播柱"),
        mood: "mystery", speaker: Some("广播"), voice: None,
        text: TextSpec::Static(&["路障边的广播柱上夹着一张皱巴巴的军令：「最后一批撤离定于 55 分钟后，地点城郊直升机坪。未持有哨所授权者，不得通过路障。」"]),
        choices: &[ChoiceDef { label: "撕下军令", sub: "情报 · +10 点", cond: None,
            effects: &[Eff::SetFlag("hx_barricade_done"), Eff::Points(10)], route: Route::To("hx_f2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_n_aibin", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 警车后 · 艾彬"),
        mood: "tension", speaker: Some("艾彬（主神轮回者）"), voice: None,
        text: TextSpec::Static(&["艾彬压低声音：「看那片黑烟——核弹快来了。我们要么抢直升机仓皇出逃，要么……塞给所谓'军方'一群丧尸陪葬。」他盯着你的眼睛，「你选哪个？是活着逃，还是烧一场大火？」"]),
        choices: &[ChoiceDef { label: "（回街道）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_n_zhou", bg: Some("huanxiongshi_bg.png"), loc: Some("F2 街角 · 郑咤"),
        mood: "tension", speaker: Some("郑咤（轮回者）"), voice: None,
        text: TextSpec::Static(&["郑咤掸净弹匣上的血，声音硬：「主神给了活路，但没给'活'。想活着离开浣熊市，要么在核弹落地前挤上那最后几架直升机，要么扛到天亮看烟花——你挑一个。」"]),
        choices: &[ChoiceDef { label: "（回街道）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("hx_f2_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 幕三 · 城郊 / 核弹倒计时 / 终结（F3） ------------------- */
    SceneDef {
        id: "hx_f3_arrive", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 城郊 · 铁丝网"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["哨所钥匙升起路障，你冲入城郊。铁丝网外是军方最后的撤离点，破败的避难屋、肃立的直升机坪，都暴露在核爆半径的边缘。"]),
        choices: &[ChoiceDef { label: "（探查城郊）", sub: "遭遇尸潮", cond: None,
            effects: &NO_EFF, route: Route::To("hx_f3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f3_horde_fight", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 城郊 · 尸潮"),
        mood: "fear", speaker: Some("丧尸群"), voice: None,
        text: TextSpec::Static(&["铁丝网下蠕动着大片尸潮——它们从掩体后爬起，像一片跌撞的潮水向你涌来！"]),
        choices: &NO_CH, fight_id: Some("hx_horde"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f3_hub", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 城郊枢纽"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["远处传来沉闷的倒计时电子音。直升机坪的探照灯扫过浓烟，扫过你的脸。核弹，不等人。"]),
        choices: &[
            ChoiceDef { label: "搜郊区避难屋", sub: "补给/压惊", cond: None, effects: &NO_EFF, route: Route::To("hx_f3_shelter") },
            ChoiceDef { label: "逼近直升机坪", sub: "联系飞行员", cond: None, effects: &NO_EFF, route: Route::To("hx_n_pilot") },
            ChoiceDef { label: "前往城郊哨所", sub: "情报", cond: None, effects: &NO_EFF, route: Route::To("hx_f3_gate") },
            ChoiceDef { label: "走向核弹倒计时室", sub: "倒计时", cond: None, effects: &NO_EFF, route: Route::To("hx_nuke_room") },
            ChoiceDef { label: "清剿城郊尸潮", sub: "战斗", cond: None, effects: &NO_EFF, route: Route::To("hx_f3_horde_fight") },
            ChoiceDef { label: "【作出逃生抉择】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("hx_ending_choice") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f3_shelter", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 避难屋"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["避难屋的应急灯昏亮，货架上堆着军粮与急救包。艾彬靠在墙上数着呼吸：「留在这儿，能睡一觉……也可能，就此醒不来。」"]),
        choices: &[
            ChoiceDef { label: "取走急救包", sub: "得 急救包", cond: None,
                effects: &[Eff::AddItem("it_hx_medkit")], route: Route::To("hx_f3_hub") },
            ChoiceDef { label: "稍作停留", sub: "倒计时推进", cond: None,
                effects: &NO_EFF, route: Route::Dyn(nuke_pause) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_f3_gate", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 城郊哨所"),
        mood: "mystery", speaker: Some("哨所广播"), voice: None,
        text: TextSpec::Static(&["哨所电台一遍遍重复：「撤离窗口……剩余 40 分钟……军方不保证返航……重复，军方不保证返航。」最后一行小字疯狂闪烁：<b>核弹代号「灭菌」</b>。"]),
        choices: &[ChoiceDef { label: "记下核弹代号", sub: "+10 点", cond: None,
            effects: &[Eff::Points(10)], route: Route::To("hx_f3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hx_n_pilot", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 直升机坪 · 飞行员"),
        mood: "tension", speaker: Some("直升机飞行员"), voice: None,
        text: TextSpec::Static(&["飞行员掀开护目镜，烟嗓沙哑：「最后一批。要上就现在上，油不够绕第二圈。」他盯着你：「我只载一具活人。你是那个活人吗？」"]),
        choices: &[ChoiceDef { label: "（回到城郊枢纽）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("hx_f3_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    /* ---- 核弹倒计时 ---- */
    SceneDef {
        id: "hx_nuke_room", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 · 核弹倒计时"),
        mood: "fear", speaker: Some("系统广播"), voice: None,
        text: TextSpec::Dyn(|st| {
            let left = 3 - nuke_t(st);
            format!("远处电子音震得耳膜发麻：<b>「灭菌」当前剩余 {}{}</b>。你若决定留下，每停留一次倒计时都更近一步。",
                if left > 0 { left.to_string() } else { "0".to_string() }, "00（约）")
        }),
        choices: &[
            ChoiceDef { label: "立即撤离", sub: "前往逃生抉择", cond: None,
                effects: &NO_EFF, route: Route::To("hx_ending_choice") },
            ChoiceDef { label: "稍作停留", sub: "倒计时-1", cond: None,
                effects: &NO_EFF, route: Route::Dyn(nuke_pause) },
            ChoiceDef { label: "退回城郊枢纽", sub: "", cond: None,
                effects: &NO_EFF, route: Route::To("hx_f3_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 抉择结局三分支（逃离 / 乘机 / 留下） ------------------- */
    SceneDef {
        id: "hx_ending_choice", bg: Some("huanxiongshi_bg.png"), loc: Some("F3 · 城郊岔路"),
        mood: "choice", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["核爆半径的终点近在眼前。三条路，三条命。没有对错——只在于，你愿意把自己交给哪一种结局。"]),
        choices: &[
            ChoiceDef { label: "徒步冲出封锁线 · 逃离", sub: "+400 点 · 硬闯尸海", cond: None,
                effects: &[Eff::SetFlag("hx_end_escape"), Eff::Points(400)], route: Route::Dyn(end_escape) },
            ChoiceDef { label: "挤上最后直升机 · 乘机", sub: "+400 点 · 生还", cond: Some(cond_radio_done),
                effects: &[Eff::SetFlag("hx_end_heli"), Eff::Points(400)], route: Route::Dyn(end_heli) },
            ChoiceDef { label: "留在浣熊市 · 留下", sub: "+400 点 · 灰烬的抉择", cond: None,
                effects: &[Eff::SetFlag("hx_end_stay"), Eff::Points(400)], route: Route::Dyn(end_stay) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ------------------- 结算卡片 / 死亡档案 ------------------- */
    SceneDef {
        id: "hx_settle_card", bg: None, loc: None, mood: "choice", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "浣 熊 市 · 逃 生 已 定".into(), good: true,
                body_html: format!(
                    "<p>核弹的倒计时在火海尽头嘀嗒作响。你终于从这座地上的地狱里，拣回了一条命——或一段灰烬。</p>\
                     <p style='color:#9a958a'>暴君 · 尸潮 · 核弹倒计时 已走完。</p>\
                     <table class='statTable'>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>支线评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                     <tr><td>主神光柱</td><td>「蜂巢在地下，而地狱在地上。」</td></tr>\
                     </table>",
                    st.points
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "hx_nuke_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("葬于核爆", "你还在为要不要留下而犹豫。倒计时归零——白昼被整片掀起的蘑菇云取代，你与围城一起归于尘。")),
            card: |_st| crate::state::Card {
                title: "葬 于 核 爆".into(), good: false,
                body_html: r#"<p>你没有迈出那一步。核弹「灭菌」如约而至，浣熊市与你的犹豫一起，被光与尘抹去。</p>
<p style='color:#ff8a8a'>【死亡档案 · 葬于核爆】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线；本次已得 points 保留，需重打逃生。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "hx_lose_boss", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("死于暴君爪下", "暴君的利爪贯穿你的胸膛。你在主神光柱熄灭前的最后一刻看见的，是它猩红独眼里的冷漠。")),
            card: |_st| crate::state::Card {
                title: "死 于 暴 君".into(), good: false,
                body_html: r#"<p>你未能击穿的暴君，也未能击穿的绝望，一同把你钉在浣熊市的地下。游戏结束。</p>
<p style='color:#ff8a8a'>【死亡档案 · 死于暴君爪下】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

/* =====================================================================
   战斗配置表（id 全部 hx_ 前缀）
   ===================================================================== */
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn win_f1_hub(_st: &GameState) -> String { "hx_f1_hub".to_string() }
fn win_f2_hub(_st: &GameState) -> String { "hx_f2_hub".to_string() }
fn win_f3_hub(_st: &GameState) -> String { "hx_f3_hub".to_string() }

pub fn huanxiongshi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("hx_zombie", FightCfg {
            name: "游荡丧尸", hp: 30, dmg: (6, 10), reward: 20, reward_why: "击倒围城的游荡丧尸",
            intro: "一具扭曲的身影拖着踉跄脚步靠近，涎水从开裂的下颌滴下——它想要你的温度。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f1_hub, death: "hx_lose_boss",
        }),
        ("hx_dog", FightCfg {
            name: "丧尸犬", hp: 24, dmg: (5, 9), reward: 15, reward_why: "击退扑咬的丧尸犬群",
            intro: "脱缰的丧尸犬从车底蹿出，獠牙带着黑色毒涎，扑向你的咽喉。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f2_hub, death: "hx_lose_boss",
        }),
        ("hx_licker", FightCfg {
            name: "舔食者", hp: 45, dmg: (10, 16), reward: 60, reward_why: "镇灭警局楼道的舔食者",
            intro: "天花板上传来窸窣，一只无皮赤红的巨影趴伏而下，长舌扫过你刚才站的地方。",
            rage_at: Some(18), rage_text: "舔食者嘶鸣着加速，利爪破空的疾响撕裂空气！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f1_hub, death: "hx_lose_boss",
        }),
        ("hx_horde", FightCfg {
            name: "城郊尸潮", hp: 120, dmg: (12, 18), reward: 120, reward_why: "清剿城郊铁丝网下的尸潮",
            intro: "尸体像潮水一样从掩体后翻涌而来，嘶哑的低吼汇聚成整片死亡的涛声。",
            rage_at: Some(50), rage_text: "尸潮彻底失控，无数只手同时向你抓来！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f3_hub, death: "hx_lose_boss",
        }),
        ("hx_tyrant", FightCfg {
            name: "暴君 Tyrant", hp: 200, dmg: (18, 28), reward: 400, reward_why: "击破浣熊市 RPD 的暴君 Tyrant",
            intro: "灰色风衣在气流中猎猎作响，暴君破墙而出，猩红独眼锁定了你——它是这座死城的绞肉机。",
            rage_at: Some(80), rage_text: "暴君风衣尽裂，肌肉暴突，速度骤然拔升——它开始毫无章法地挥砸！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |_st| "hx_f1_boss_defeated".to_string(),
            death: "hx_lose_boss",
        }),
    ]
}

// ---------- 查询辅助 ----------
pub fn hx_scene(id: &str) -> Option<&'static SceneDef> { HUANXIONGSHI_SCENES.iter().find(|s| s.id == id) }
pub fn hx_fight_cfg(id: &str) -> Option<&'static FightCfg> { huanxiongshi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v) }
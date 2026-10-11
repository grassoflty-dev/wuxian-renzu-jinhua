//! 洪荒历 · 低纬度领地 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、世界居民 NPC、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「低纬度的影子，会追着活人。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("dw_boss") {
            st.fight = Some(crate::power::scaled_fight("dw_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "dw_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "dw_death".to_string(); }
    "dw_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("dw_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "dw_reward");
    "dw_end_choice".to_string()
}

pub static DIWEIDU_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "dw_00", bg: Some("diweidu_bg.png"), loc: Some("低维边陲 · 入口"), mood: "tension",
        speaker: Some("旁白"), voice: Some("vo_diweidu_1"),
        text: TextSpec::Static(&[
            "你踏入了「洪荒历 · 低纬度领地」。",
            "这里的光是斜的、影子是活的。低纬度的世界像一面裂了缝的镜子，把现实照得支离破碎。",
            "一个裹着灰袍的老者拦下你，声音像砂纸磨过铁：「别回头看你的影子。在这里，影子会先你一步，找到回家的路。」",
        ]),
        choices: &[
            ChoiceDef { label: "问清影子之忌", sub: "San+5 · 听清禁忌", cond: None, effects: &[Eff::San(5)], route: Route::To("dw_hub") },
            ChoiceDef { label: "径自踏入领地", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "dw_hub", bg: Some("diweidu_bg.png"), loc: Some("低维领地 · 裂隙市"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "裂隙市建在无数现实倒影的夹缝里。街道的尽头是另一条街道，所有人的影子都朝反方向走。",
            "传闻统治这片领地的大领主，正藏在无数个『自己』的倒影深处。要想穿过低纬度，你绕不开他。",
        ]),
        choices: &[
            ChoiceDef { label: "看裂隙里的旧屋", sub: "现实倒影 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("dw_pt_mirror") },
            ChoiceDef { label: "察市中央高塔", sub: "领主之塔 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("dw_pt_tower") },
            ChoiceDef { label: "端详地上的影子", sub: "活影 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("dw_pt_shadow") },
            ChoiceDef { label: "看破碎喷泉", sub: "最初之像 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("dw_pt_fountain") },
            ChoiceDef { label: "逛裂隙集市", sub: "倒影之货 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("dw_pt_market") },
            ChoiceDef { label: "听拾荒者之语", sub: "居民 · 桠子", cond: None, effects: &NO_EFF, route: Route::To("dw_np_hoarder") },
            ChoiceDef { label: "访倒影画师", sub: "居民 · 画师", cond: None, effects: &NO_EFF, route: Route::To("dw_np_painter") },
            ChoiceDef { label: "走向大领主宫邸", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("dw_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "dw_pt_mirror", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 倒影旧屋"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一扇半掩的木门推开，屋内摆设与正常世界别无二致，却在镜面里全部颠倒——桌上的茶是满的，镜里的却是空的。",
            "墙上挂着一幅泛黄的全家福。镜里面容清晰可辨的那个人，正朝镜外的你看过来，嘴角缓缓勾起。",
            "传说低纬度的一切倒影，都是『人』不愿回头的那些日子。你关上门，把那张脸关在了镜子里。",
        ]),
        choices: &[
            ChoiceDef { label: "细看倒影全家福", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("dw_pt_1"), Eff::Points(20), Eff::San(-3)], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_pt_tower", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 中央高塔"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "市中央的领主高塔以肉眼可见的『错位』矗立——塔身每一层都朝不同方向歪斜，却又始终不倒。",
            "有人告诉你，大领主把自己切成了无数个倒影，分别锁在每一层，好让任何活人都无法同时击败他全部的自己。",
            "塔底刻着一行字：「要穿过低纬，先照见自己。」但没人看懂它到底在提醒你什么。",
        ]),
        choices: &[
            ChoiceDef { label: "绕塔一周记下结构", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("dw_pt_2"), Eff::Points(20)], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_pt_shadow", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 空旷广场"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "广场正午无日，可地上每个人的影子都在缓慢地、执着地朝同一个方向爬——朝市外。",
            "老人说过，低纬度的影子会追活人。可你分明看见，那些影子追的根本不是人。它们追的是……光。是能带它们脱离低纬的『光』。",
            "你站在原地，第一次分不清自己究竟是光，还是被困在影子里的那个人。",
        ]),
        choices: &[
            ChoiceDef { label: "背对广场 · 不再看影", sub: "调查完成 · +15点 · San+3", cond: None,
                effects: &[Eff::MarkPoint("dw_pt_3"), Eff::Points(15), Eff::San(3)], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_pt_fountain", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 破碎喷泉"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一口被踩得稀碎的喷泉，泉心雕着一位张开双臂的『人』。它怀里本该抱着的圣物，只有一圈空空的凹槽。",
            "路过的拾荒者低声说：「这是低纬最初的领主。他也想带大家走出去，可最后……他把所有人都困在了影子里。」",
            "水已干涸，但凹槽深处还残留着一小块，能照见人影的碎镜。",
        ]),
        choices: &[
            ChoiceDef { label: "拾起碎镜", sub: "调查完成 · +20点 · 得倒影碎镜", cond: None,
                effects: &[Eff::MarkPoint("dw_pt_4"), Eff::Points(20), Eff::AddItem("dw_shard")], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_pt_market", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 倒影集市"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "倒影集市卖的不是货，是一段段被重新拼接的『人生』。有人兜售『别人的童年』，有人叫卖『未发生的黄昏』。",
            "一个摊位前,摆着满满一面墙的镜子，每面镜里都有一扇从没打开过的门。摊主笑吟吟地说：「挑一扇吧，能看见你想逃去的地方。」",
            "你知道，那门后多半是低纬在诱骗你留下的把戏。可即便如此，还是有那么一扇门，让你站在原地看了很久。",
        ]),
        choices: &[
            ChoiceDef { label: "买下那扇门的倒影", sub: "调查完成 · +20点 · 得门影纪念", cond: None,
                effects: &[Eff::MarkPoint("dw_pt_5"), Eff::Points(20), Eff::AddItem("dw_doormem")], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "dw_np_hoarder", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 拾荒者窝棚"), mood: "calm",
        speaker: Some("拾荒者 · 桠子"), voice: None,
        text: TextSpec::Static(&[
            "拾荒者桠子背着满满一袋『倒影』兜售，嘴里絮叨：「低纬的东西换不来真钱，可在这里，倒影比命值钱。」",
            "他神秘兮兮地压低声音：「你知道人为什么怕影子追着自己吗？因为那影子，其实是你在高维里舍不得丢下的一部分自己。」",
            "「要是影子和人合不上，这个人就永远走不出低纬了。」他拍拍你的肩，「你可别让它追上了。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下影子之说", sub: "NPC 对话 · +15点", cond: None,
                effects: &[Eff::Points(15), Eff::SetFlag("dw_know_shadow")], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_np_painter", bg: Some("diweidu_bg.png"), loc: Some("裂隙市 · 画师铺"), mood: "sad",
        speaker: Some("倒影画师 · 挽"), voice: None,
        text: TextSpec::Static(&[
            "画师铺里挂满了画——可每一张都是空的，只有底色的白。画师挽握着笔，呆望着某处出神。",
            "「我画了半辈子，只想把一个人画回来。可低纬只给影子留位置，不给活人留。」",
            "他望向窗外领主的高塔，声音很轻：「大领主也是这么想的吧。可他把人困进影子，靠的却是把他们的『光』吞进自己。那不是拯救，是封存。」",
        ]),
        choices: &[
            ChoiceDef { label: "安慰挽", sub: "NPC 对话 · +15点 · San-3", cond: None,
                effects: &[Eff::Points(15), Eff::San(-3)], route: Route::To("dw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "dw_01b_prep", bg: Some("diweidu_bg.png"), loc: Some("大领主宫邸 · 镜厅门前"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "大领主宫邸的镜厅门前，无数面镜子围成一圈，每一面都映着同一个坐姿的身影——它们都在等你。",
            "面前高踞王座的『大领主』缓缓抬头，你分不清这是本人还是倒影，因为他浑身由无数重叠的影子构成。",
            "他开口，声音像四面八方同时传来：「影子不追活人。是我把你……拉进影子里来了。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 大领主 · 灾厄聚合体】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "再看一眼喷泉碎镜", sub: "San-3 · 聚神", cond: None, effects: &[Eff::San(-3)], route: Route::To("dw_pt_fountain") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景
    SceneDef {
        id: "dw_01", bg: Some("diweidu_bg.png"), loc: Some("大领主宫邸"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["灾厄聚合体 挡在出口。低纬度的影子，会追着活人。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("dw_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_round", bg: Some("diweidu_bg.png"), loc: Some("大领主宫邸 · 决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 dw_card） =====
    SceneDef {
        id: "dw_end_choice", bg: Some("diweidu_bg.png"), loc: Some("决战之后 · 裂隙尽头"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当最后一个倒影崩解，大领主终于露出下方那个枯槁而年老的『人』。他望着裂隙尽头透进来的光，浑浊的眼里落下泪。",
            "缠在低纬天空的网破了个洞，光正从那个洞倾泻而下。你站在光的边缘，影子第一次安静地贴在你的脚下。",
            "沿着裂隙往前走，你自己决定这一程要怎么结束。",
        ]),
        choices: &[
            ChoiceDef { label: "凝望天隙的光", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("dw_sight")], route: Route::To("dw_card") },
            ChoiceDef { label: "带走那面碎镜", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("dw_memento")], route: Route::To("dw_card") },
            ChoiceDef { label: "留在裂隙市", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("dw_stay")], route: Route::To("dw_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dw_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>影子终于不再追着你。你穿过低纬度，也照见了被自己丢在身后的一部分。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "dw_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("洪荒历 · 低纬度领地 · 殒命", "殒命于洪荒历 · 低纬度领地")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn diweidu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("dw_boss", FightCfg {
            name: "灾厄聚合体", hp: 230, dmg: (18, 28), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "dw_card".to_string(), death: "dw_death",
        }),
    ]
}
//! 洪荒历 · 三联盟会盟 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、世界居民、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「举杯的下一秒，脚下是祭坛。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sl_boss") {
            st.fight = Some(crate::power::scaled_fight("sl_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "sl_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "sl_death".to_string(); }
    "sl_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("sl_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "sl_reward");
    "sl_end_choice".to_string()
}

pub static SANLIAN_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "sl_00", bg: Some("sanlian_bg.png"), loc: Some("三联盟 · 会盟高原"), mood: "tension",
        speaker: Some("旁白"), voice: Some("vo_sanlian_1"),
        text: TextSpec::Static(&[
            "你踏入了「洪荒历 · 三联盟会盟」。",
            "三面颜色各异的盟旗在高原上并列飘扬，三族的使节齐聚一堂，正准备歃血为盟。高原上摆着一条望不到头的长宴。",
            "一个捧着酒坛的司礼官凑到你耳边，笑得意味深长：「盟约要成了……可在洪荒，举杯的下一秒，往往是祭坛。」",
        ]),
        choices: &[
            ChoiceDef { label: "问清祭坛之说", sub: "San+5 · 生疑", cond: None, effects: &[Eff::San(5)], route: Route::To("sl_hub") },
            ChoiceDef { label: "入场等待结盟", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "sl_hub", bg: Some("sanlian_bg.png"), loc: Some("会盟场 · 长宴台侧"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "长宴台两侧坐满了三族衣冠各异的宾客，觥筹交错间，你隐约看见台心那口被绸布盖着的『盟碑』下，似乎有暗红的纹路。",
            "三联盟本该在此结为铁盟，可你总觉得，这宴席图的不只是结盟那么简单。",
        ]),
        choices: &[
            ChoiceDef { label: "察看盟碑之下", sub: "暗红纹路 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("sl_pt_stele") },
            ChoiceDef { label: "看长宴的青铜鼎", sub: "献祭之鼎 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("sl_pt_cauldron") },
            ChoiceDef { label: "读三族誓约卷", sub: "盟约细文 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("sl_pt_treaty") },
            ChoiceDef { label: "看旧三盟信物", sub: "带伤的信物 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("sl_pt_tokens") },
            ChoiceDef { label: "听司礼官的低语", sub: "居民 · 司礼官", cond: None, effects: &NO_EFF, route: Route::To("sl_np_master") },
            ChoiceDef { label: "寻三族的女医", sub: "居民 · 女医", cond: None, effects: &NO_EFF, route: Route::To("sl_np_medic") },
            ChoiceDef { label: "走向高台主祭位", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("sl_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "sl_pt_stele", bg: Some("sanlian_bg.png"), loc: Some("会盟场 · 盟碑底"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你趁人不备掀起绸布一角——盟碑不是石，是一整块深青色的骨。碑心刻着三族先祖的名字，正中却空了三个勒痕。",
            "那勒痕的形状，正好是『活物献祭』的规格。这碑立在这里，仿佛等的从来不是三族的血，而是某一场祭祀。",
            "你放下绸布，心跳得厉害。司礼官的话在耳边回响：「举杯的下一秒，是祭坛。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下盟碑勒痕", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("sl_pt_1"), Eff::Points(20), Eff::San(-3)], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_pt_cauldron", bg: Some("sanlian_bg.png"), loc: Some("长宴台 · 青铜鼎"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "长宴台中央那口三人高的青铜鼎里，盛着望不到底的浓汤。鼎足刻着繁复的兽纹，每一道纹路都在往中心聚拢。",
            "旧礼说，三族结盟要向天献一鼎牲美。可你低头细看，鼎底那些兽纹分明是——一张张皱着、却在笑的人脸。",
            "你几乎想把自己伸进去的手缩回来：这鼎烧的，恐怕不是三牲。",
        ]),
        choices: &[
            ChoiceDef { label: "记下鼎底人脸纹", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("sl_pt_2"), Eff::Points(15), Eff::San(-5)], route: Route::To("sl_hub") },
            ChoiceDef { label: "搅动汤羹细看", sub: "San-8 · 惊动暗影", cond: None,
                effects: &[Eff::San(-8), Eff::Hurt(6, "sl_death")], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_pt_treaty", bg: Some("sanlian_bg.png"), loc: Some("会盟桌 · 三族条约"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "三族共签的盟约卷轴摊在长桌上，墨迹尚新。你读到最后一行小字，一愣——「凡盟誓之日，三族各出最勇者一名，歃血于鼎，奉于天。」",
            "原来『结盟』的末尾是一条献祭条款。三族都把最勇的人送上祭坛，用他们的血和勇气，换三天太平。",
            "这不是结盟。这是一场循环的、吞噬勇者的丛林法则。",
        ]),
        choices: &[
            ChoiceDef { label: "记下献祭条款", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("sl_pt_3"), Eff::Points(20)], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_pt_map", bg: Some("sanlian_bg.png"), loc: Some("会盟场 · 沙盘舆图"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "高台一侧铺着整片平原的沙盘，三族疆域以祭坛为轴心环列，像三道护墙环着一口井。",
            "沙盘边缘散落着三枚残缺的兵棋，分别属于早已覆灭的旧盟。老人说过，旧盟也走过同样的路——先是结盟，再是献祭，最后是分崩。",
            "你忽然明白:这片洪荒从不缺盟约，缺的是一次『不举向祭坛的酒』。",
        ]),
        choices: &[
            ChoiceDef { label: "记下旧盟兵棋", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("sl_pt_4"), Eff::Points(20)], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_pt_tokens", bg: Some("sanlian_bg.png"), loc: Some("长宴台 · 旧盟信物匣"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "长宴台脚下压着一只落了灰的旧木匣，锁已锈死。你撬开匣子，里面是一打三族各自的信物——有的刃口崩了，有的玉碎了，有的被血染得发黑。",
            "法器边压着一张发黄的纸，只写了一行字:「三盟之约，以勇者血换三日太平。旧盟如此，新盟亦然。」",
            "你沉默良久。原来每一代踏入这里的『最勇者』，都以为自己能打破循环——可他们最终都被写进了同一张纸上。",
        ]),
        choices: &[
            ChoiceDef { label: "记下旧盟信物", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("sl_pt_5"), Eff::Points(20)], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "sl_np_master", bg: Some("sanlian_bg.png"), loc: Some("长宴台 · 司礼官"), mood: "calm",
        speaker: Some("司礼官 · 玄成"), voice: None,
        text: TextSpec::Static(&[
            "司礼官玄成一手扶着酒坛，一手按着心口，笑眯眯地望着满座宾客：「别紧张，这是洪荒最体面的宴席。」",
            "他凑近你，压低声音：「可越是体面的宴席，越容易出最不体面的事。我伺候了三代盟会……每一次，都有人没从宴席上走回去。」",
            "他给你斟满一杯：「你且安心饮。至于饮完之后脚下是什么，那不是你能选的。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下司礼官的戒语", sub: "NPC 对话 · +15点", cond: None,
                effects: &[Eff::Points(15), Eff::SetFlag("sl_know_rite")], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_np_medic", bg: Some("sanlian_bg.png"), loc: Some("会盟场 · 营地药棚"), mood: "sad",
        speaker: Some("外族女医 · 涟"), voice: None,
        text: TextSpec::Static(&[
            "营帐外的女医涟正替一位伤兵换药，动作既快又轻。她头也不抬地说:「宴上的人，都在等着看哪位勇士被选中。」",
            "她压低了声音:「我见过太多『最勇者』走进那座祭坛，再没走出来。他们以为那是荣耀，却不知那是耗材。」",
            "她抬眼望你，目光沉静:「你若真能活下去，就替他们问一句——为什么勇者的荣耀，最后总要变成别人的柴火。」",
        ]),
        choices: &[
            ChoiceDef { label: "郑重应下这句话", sub: "NPC 对话 · +15点 · San-3", cond: None,
                effects: &[Eff::Points(15), Eff::San(-3)], route: Route::To("sl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "sl_01b_prep", bg: Some("sanlian_bg.png"), loc: Some("高台主祭位 · 祭坛显现"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "宴至高潮，高台的绸布被风掀起——脚下的石地赫然浮现出整圈暗红的祭纹。举杯的下一秒，你的脚下是祭坛。",
            "主祭位的『狂誓者』缓缓起身，原来他从来不是来结盟的，而是来主持这场吞噬勇者的大逃杀的。",
            "他望着满堂惊惧的宾客，声音像锈蚀的钟:「既然盟约已成，那便用它来喂饱这洪荒——献上你们的『最勇者』。」他的目光，落在了你身上。",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 狂誓者】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "再看一眼盟约条款", sub: "San-3 · 了然", cond: None, effects: &[Eff::San(-3)], route: Route::To("sl_pt_treaty") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景
    SceneDef {
        id: "sl_01", bg: Some("sanlian_bg.png"), loc: Some("高台主祭位"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["狂誓者 挡在出口。举杯的下一秒，脚下是祭坛。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("sl_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_round", bg: Some("sanlian_bg.png"), loc: Some("高台主祭位 · 决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 sl_card） =====
    SceneDef {
        id: "sl_end_choice", bg: Some("sanlian_bg.png"), loc: Some("决战之后 · 宴席散场"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当狂誓者轰然倒下，高台祭纹骤然熄灭。满场鸦雀无声，所有人都在等——这一次，勇者能不能活，能不能不变成祭品。",
            "三族的旗重新在风里猎猎扬起。你站在祭坛中央，脚下那些暗红的纹路渐渐淡去。走了这么多年的洪荒，原来是可以有别的活的。",
            "你看着满场惊惧而后又隐约期盼的目光，自己来决定，这一场你如何作结。",
        ]),
        choices: &[
            ChoiceDef { label: "眺望散去的宾客", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("sl_sight")], route: Route::To("sl_card") },
            ChoiceDef { label: "拾起一块盟碑残片", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("sl_memento")], route: Route::To("sl_card") },
            ChoiceDef { label: "留下整顿这场盟约", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("sl_stay")], route: Route::To("sl_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sl_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>这一夜，你让一只本要被献上的酒杯，变成了真正的结盟之酒。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "sl_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("洪荒历 · 三联盟会盟 · 殒命", "殒命于洪荒历 · 三联盟会盟")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn sanlian_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sl_boss", FightCfg {
            name: "狂誓者", hp: 180, dmg: (16, 24), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "sl_card".to_string(), death: "sl_death",
        }),
    ]
}
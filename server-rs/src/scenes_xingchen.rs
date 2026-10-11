//! 大宇宙时代 · 星辰吞噬者 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、巨兽体内幸存者 NPC、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「它的胃，是一整个星团。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("xc_boss") {
            st.fight = Some(crate::power::scaled_fight("xc_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "xc_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "xc_death".to_string(); }
    "xc_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("xc_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "xc_reward");
    "xc_end_choice".to_string()
}

pub static XINGCHEN_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "xc_00", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 入口腔道"), mood: "tension",
        speaker: Some("旁白"), voice: Some("vo_xingchen_1"),
        text: TextSpec::Static(&[
            "你踏入了「大宇宙时代 · 星辰吞噬者」。",
            "你的穿梭舰被某股无法抗拒的引力拖进一片幽暗。等你睁眼，脚下踩着的已不是舰舱，而是一片由星骸、气尘与扭曲光构成的「地面」。",
            "一个断了半臂、靠着残骸甲板的男人抬起头，涩声道:「欢迎……进了它的胃。它把一整片星团都吞下了，你走不出去的。」",
        ]),
        choices: &[
            ChoiceDef { label: "问清这是谁的胃", sub: "San+5 · 了然处境", cond: None, effects: &[Eff::San(5)], route: Route::To("xc_hub") },
            ChoiceDef { label: "径自深入腔道", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "xc_hub", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 腔内浮岛"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你立于一片由破碎行星与报废舰体拼成的浮岛上，四周是被引力扭曲得怪异的星骸。这里是「星辰吞噬者」的腹腔深处。",
            "传闻镇压这片星团的星核守卫，正守在通往出口的腔道。要离开这头巨兽，你绕不开它。",
        ]),
        choices: &[
            ChoiceDef { label: "察星骸压缩区", sub: "坍缩奇观 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("xc_pt_heap") },
            ChoiceDef { label: "探引力之井", sub: "扭曲漩涡 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("xc_pt_well") },
            ChoiceDef { label: "看光尘回廊", sub: "星尘余晖 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("xc_pt_hall") },
            ChoiceDef { label: "端详胃壁晶簇", sub: "结晶生长 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("xc_pt_crystal") },
            ChoiceDef { label: "走巨兽脊龙骨", sub: "脊骨奇观 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("xc_pt_keel") },
            ChoiceDef { label: "听残骸男人讲旧事", sub: "被困者 · 弗", cond: None, effects: &NO_EFF, route: Route::To("xc_np_cast") },
            ChoiceDef { label: "询星核衍生物", sub: "衍生物 · 萤", cond: None, effects: &NO_EFF, route: Route::To("xc_np_firefly") },
            ChoiceDef { label: "走向出口腔道", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("xc_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "xc_pt_heap", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 星骸压缩区"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "附近整片空间被引力压得发亮，无数行星残骸、舰体与气尘被揉成一个缓慢旋转的「星骸盘」，越靠近中心越小、越亮。",
            "你回望了一眼:那些碎成残骸的行星与被吞的舰体，也被卷进这一格引力盘里，像被它收拢的、再也吐不出的『粮』。",
            "你忽然明白，这头巨兽吞噬的从不是某颗星……而是几乎一整片被它当作粮仓的星团。它的胃，装得下星空。",
        ]),
        choices: &[
            ChoiceDef { label: "记下星骸盘结构", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("xc_pt_1"), Eff::Points(20), Eff::San(-3)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_pt_well", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 引力之井"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "脚下深处是一口深不见底的引力井，井口的光与气尘正被一格一格地扯进旋涡里。那是融化的星骸在朝更深的腹腔沉去的通道。",
            "你往井中凝望，竟在旋涡深处看见无数闪动的『萤火』——那是被吞下的、尚未完全消化的星尘在哀鸣般的明灭。",
            "那对同样被困的对象来说，既是通往灭亡的坠落，也是这冰冷腹腔里唯一仍在发光的景。",
        ]),
        choices: &[
            ChoiceDef { label: "静望引力之井", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("xc_pt_2"), Eff::Points(15), Eff::San(-5)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_pt_hall", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 光尘回廊"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一段由漂浮星尘构成的长廊，在幽暗中泛着细碎的银光，像一条被引力拉直了的银河。光尘随你的脚步轻轻漾开。",
            "这些光尘，是这片星团里无数星辰临终前的余光，被巨兽连同它们生前的一切，一并吞进了腹腔。",
            "你伸手捞起一把，它们从指间漏下，安静地回到缓慢漂流的回廊里。此处不辨生死，只有光还在迟疑地亮着。",
        ]),
        choices: &[
            ChoiceDef { label: "掬一捧光尘", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("xc_pt_3"), Eff::Points(20)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_pt_crystal", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 胃壁晶簇"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "腹腔壁上长出大片半透明的晶簇，像倒生的钟乳一般密密悬挂。晶簇内部封着凝固的星尘，在黑暗中幽幽发亮。",
            "你贴近一根晶簇细看，竟隐约看见里面蜷着一团被冻结的、还维持着探手姿态的『剪影』——那是某艘被吞下的飞行器与船员的最后姿态。",
            "你后退一步。这头巨兽的『胃壁』，竟在缓慢地把吞下去的星骸与生命，一点点结晶、封存、凝固成它自己的血肉。",
        ]),
        choices: &[
            ChoiceDef { label: "抚过一根晶簇", sub: "调查完成 · +20点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("xc_pt_4"), Eff::Points(20), Eff::San(-5)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_pt_keel", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 脊龙骨"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你攀上一道横贯腹腔的庞大龙骨——那是这头吞噬星团巨兽的脊骨，由凝固的星骸与坍缩的引力纹路铸成，一路延伸向幽暗的远端。",
            "龙骨的每一节都嵌着一段幽蓝的『年轮』，像一本被凝成化石的星历，记录着它吞下的一颗又一颗星。",
            "你抚过一处最浅的凹痕——那里刻着一行古老坐标，是一颗曾经存在、如今却只剩回响的恒星的名字。",
        ]),
        choices: &[
            ChoiceDef { label: "记下龙骨坐标", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("xc_pt_5"), Eff::Points(20)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "xc_np_cast", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 残骸营地"), mood: "sad",
        speaker: Some("残骸男人 · 弗"), voice: None,
        text: TextSpec::Static(&[
            "断臂的残骸男人弗靠着甲板，一遍遍擦着一块看不出花纹的怀表:「它是我们舰的领航。吞进这胃里那天，他拦着我去关引擎，自己却跟着舰体一起沉下去了。」",
            "他涩声笑了笑:「你以为能在星海里找到出口。可这头畜生吞了几乎一整片星团——它肚子里根本没有『外』，只有更深的腹。」",
            "他把怀表递给你:「若你真能走出去，带着它吧。至少替我问问外头的人，恒星……还有没有剩下的。」",
        ]),
        choices: &[
            ChoiceDef { label: "收下那块怀表", sub: "NPC 对话 · +15点 · 得怀表", cond: None,
                effects: &[Eff::Points(15), Eff::AddItem("xc_pocket")], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_np_firefly", bg: Some("xingchen_bg.png"), loc: Some("巨兽体内 · 星尘浮游"), mood: "calm",
        speaker: Some("星核衍生物 · 萤"), voice: None,
        text: TextSpec::Static(&[
            "一团巴掌大、由星尘聚成的柔和光点，缓缓在你面前凝成一个无害的小影。它不像异种，更像是这腹腔里『少数没被消化』的光。",
            "你伸手，那光点依恋地绕着你转了一圈，发出一串哑光的颤动，仿佛在对你说:「别害怕……我还记得自己是颗星。」",
            "它停在你肩头，替你引亮一小片黑暗。星核衍生物不全是这头巨兽的血肉——至少这一团，还留着它作为星辰的记忆。",
        ]),
        choices: &[
            ChoiceDef { label: "任它停在你肩头", sub: "NPC 对话 · +15点 · San+5", cond: None,
                effects: &[Eff::Points(15), Eff::San(5)], route: Route::To("xc_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "xc_01b_prep", bg: Some("xingchen_bg.png"), loc: Some("出口腔道 · 星核守卫"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你顺着浮岛漂向出口腔道，迎面撞上一头由坍缩星核与结晶凝成的庞然身影——星核守卫。它挡在那道唯一能让光透进来的裂缝前。",
            "它通体由旋转的星骸与幽蓝的引力气场构成，胸口那颗核心缓慢搏动，仿佛正替这头巨兽跳着某种亘古的『心跳』。",
            "它的声音穿过整段腔道，不像威胁，更像陈述:「没有哪颗星，能从我守着的地方离开。除非——你让它心甘情愿放行。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 星核守卫】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "再看一眼光尘回廊", sub: "San-3 · 了然", cond: None, effects: &[Eff::San(-3)], route: Route::To("xc_pt_hall") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景
    SceneDef {
        id: "xc_01", bg: Some("xingchen_bg.png"), loc: Some("出口腔道"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["星核守卫 挡在出口。它的胃，是一整个星团。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("xc_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_round", bg: Some("xingchen_bg.png"), loc: Some("出口腔道 · 决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 xc_card） =====
    SceneDef {
        id: "xc_end_choice", bg: Some("xingchen_bg.png"), loc: Some("决战之后 · 星团裂缝"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当星核守卫的胸口核心熄灭，出口腔道那道裂缝被撕得更开，久违的浩瀚星光倾泻而入，照亮了整片漂浮的星骸。",
            "你回望了一眼:那些被吞进肚子的星尘、光尘与残骸飞船，正被裂缝外涌入的光染成一片温和的银白色。巨兽的心跳，渐渐慢了下来。",
            "你站在这头吞噬星辰者的腹地上，头顶是通往星海的裂口。这一程要怎么收尾，由你决定。",
        ]),
        choices: &[
            ChoiceDef { label: "仰望裂口的星河", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("xc_sight")], route: Route::To("xc_card") },
            ChoiceDef { label: "带走那块怀表", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("xc_memento")], route: Route::To("xc_card") },
            ChoiceDef { label: "留在星骸腹地", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("xc_stay")], route: Route::To("xc_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xc_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>你从一头吞噬星团的巨兽腹中，带着属于星辰的光走了出来。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "xc_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("大宇宙时代 · 星辰吞噬者 · 殒命", "殒命于大宇宙时代 · 星辰吞噬者")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn xingchen_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("xc_boss", FightCfg {
            name: "星核守卫", hp: 220, dmg: (18, 28), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "xc_card".to_string(), death: "xc_death",
        }),
    ]
}
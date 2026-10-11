//! 侠行天下 · 武极境破虚 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「武的尽头是另一个世界的开始。武极于此，天地便开一缝给你看。」
//! 结构：开场(武极高地) → pv_hub(调查中转) → 调查点/武的极境 + NPC → 境界松动 → 异界来者迎战 → 开放结局三选 → pv_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("pv_boss") {
            st.fight = Some(crate::power::scaled_fight("pv_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "pv_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "pv_death".to_string(); }
    "pv_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("pv_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "pv_reward");
    "pv_card".to_string()
}

pub static POXU_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "pv_00", bg: Some("poxu_bg.png"), loc: Some("武极之巅 · 破虚台"),
        mood: "calm", speaker: Some("界碑铭文"), voice: Some("vo_wujie_poxu_1"),
        text: TextSpec::Static(&[
            "你踏上这片悬在云海之上的武极高地。脚下是终年不化的云，头顶却裂开一道青白色的长缝——那不是云层，是「天」本身被某道力劈开的一线。",
            "界碑上刻着两行风蚀的旧字：<em>「武之极也，非止于拳脚。尽头处有门，门后是另一个世界的开始。」</em>",
            "你运足一口气，感觉丹田里多年苦修的那股力，在脚下这块地上格外清亮——仿佛这片高地,是为「把武练到尽头」的人备好的。远处，那道天缝正微微发亮。",
        ]),
        choices: &[
            ChoiceDef { label: "看向那道天缝", sub: "+5 点 · 记住武的尽头", cond: None, effects: &[Eff::Points(5)], route: Route::To("pv_ascent") },
            ChoiceDef { label: "环视这方破虚台", sub: "看清各条登顶的路", cond: None, effects: &NO_EFF, route: Route::To("pv_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "pv_hub", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 登峰古道"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "云海在脚下翻涌，几块巨岩依着山势立在博大的空地上——那是历代武极者留下的足迹。你要向上登，也要朝那道天缝走。",
            "这片高地不缺「武」的气息：断剑、铭文、废墟，都等着被有心人读懂。<em>武的尽头是另一个世界的开始</em>——你决定，先把这座武之极境看个明白。",
        ]),
        choices: &[
            ChoiceDef { label: "崖壁武刻", sub: "上古武者留下的铭文", cond: None, effects: &NO_EFF, route: Route::To("pv_petroglyph") },
            ChoiceDef { label: "剑冢", sub: "插满断剑的高原", cond: None, effects: &NO_EFF, route: Route::To("pv_swordmound") },
            ChoiceDef { label: "云海武馆废墟", sub: "一座倒在雾里的练武场", cond: None, effects: &NO_EFF, route: Route::To("pv_dojo") },
            ChoiceDef { label: "破虚台之顶", sub: "那道名为「破虚」的天缝", cond: None, effects: &NO_EFF, route: Route::To("pv_ascent") },
            ChoiceDef { label: "远处有人影", sub: "与守在山巅的武者交谈", cond: None, effects: &NO_EFF, route: Route::To("pv_last_swordsman") },
            ChoiceDef { label: "放空心境，向武之极限靠拢", sub: "触摸武的尽头", cond: None, effects: &NO_EFF, route: Route::To("pv_gather") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：崖壁武刻 =================
    SceneDef {
        id: "pv_petroglyph", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 崖壁武刻"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一面近乎垂直的崖壁上，刻满招式与掌法的人形，从最基础的马步一直到最后一套凌空碎云的拳法。每一道刻痕都透着力道，像是古人用掌风直接拍进石里。",
            "最后一个人形图的头顶，裂着一道与天上天缝相呼应的短发状裂纹——它在「破」与「立」之间定格。<em>武的极处，古人早已刻下同一个答案：打通「我」与「天地」之间那最后一层隔膜。</em>",
        ]),
        choices: &[ChoiceDef { label: "临摹那套刻影图", sub: "MarkPoint · AddItem 武刻拓片 · +15 点", cond: None,
            effects: &[Eff::MarkPoint("pv_pt_petroglyph"), Eff::AddItem("pv_rubbing"), Eff::Points(15)], route: Route::To("pv_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：剑冢 =================
    SceneDef {
        id: "pv_swordmound", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 剑冢"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一片平整的高原上，密密麻麻插着几百柄断剑，剑刃统统朝天，像收敛锋芒的兵刃都在回应那道天缝。风从剑丛间穿过，发出低沉的嗡鸣。",
            "你在剑冢中央摸到一柄尚温的剑——不是锈，而是仍留着搏杀的余温。这里大概是历代追求「破虚」的武者们，最终把佩剑留下、赤手走向尽头的地方。",
        ]),
        choices: &[ChoiceDef { label: "从剑冢拔出一柄断剑", sub: "MarkPoint · AddItem 破虚断剑 · +15 点", cond: None,
            effects: &[Eff::MarkPoint("pv_pt_swordmound"), Eff::AddItem("pv_broken_sword"), Eff::Points(15)], route: Route::To("pv_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：云海武馆废墟 =================
    SceneDef {
        id: "pv_dojo", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 云海武馆"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一座只剩一半屋顶的武馆倒在云海边缘，演武场的木人桩东倒西歪，墙上还挂着褪色的「武」字匾额。地板被千万次踏脚磨出浅浅的凹坑。",
            "收卷在角落的一本手抄拳谱，扉页写着：「吾辈在此练了一辈子，只为离那道缝更近一步。」你把它卷起收好——这不是武功秘籍，是一个武者站在尽头前的念想。",
        ]),
        choices: &[ChoiceDef { label: "收起那本手抄拳谱", sub: "MarkPoint · AddItem 旧拳谱 · +15 点", cond: None,
            effects: &[Eff::MarkPoint("pv_pt_dojo"), Eff::AddItem("pv_manual"), Eff::Points(15)], route: Route::To("pv_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：破虚台之顶 =================
    SceneDef {
        id: "pv_ascent", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 破虚台之顶"),
        mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "你登上破虚台的最高点，离那道天缝只剩一步之遥。缝隙恰好一人宽，透进来的光却不像天光——它冷而透亮，像在「看」你。",
            "你把一只手探进缝里，指腹触到另一侧极轻的一缕风，还有一片若有若无的、翻涌的更广阔的「响动」。那一瞬间你明白：这道缝的另一头，确实站着另一个世界。",
            "你收回手，心口却怦怦直跳。<em>武的尽头，天真的开了一缝。</em>",
        ]),
        choices: &[ChoiceDef { label: "记住缝另一侧的风", sub: "MarkPoint · +20 点 · 破虚之顶", cond: None,
            effects: &[Eff::MarkPoint("pv_pt_ascent"), Eff::Points(20), Eff::San(5)], route: Route::To("pv_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：武极山巅的最后一位武者 =================
    SceneDef {
        id: "pv_last_swordsman", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 山巅孤屏"),
        mood: "calm", speaker: Some("山巅武者 · 须发尽白"), voice: None,
        text: TextSpec::Static(&[
            "一块孤岩上，坐着一个须发尽白的老者，怀抱一柄无鞘长剑，望着那道天缝出神。他头也不回：「来讨武道的？上来这一年，我早把武练到没处练了。」",
            "他轻抚剑刃：「年轻时候以为，练到尽头是为了赢。站到这儿才明白——武到了顶，不是要照见别人，是要照见自己那身力气，究竟该往哪里放。」",
            "「那道缝另一头，也许是更大的搏杀，也许是更静的山水。」他转头，眼里澄澈，「走过去，武的尽头就是开始；停下来，也无人能说你错。」",
        ]),
        choices: &[ChoiceDef { label: "听老者说完这句「尽头」", sub: "+10 点 · 武者的道", cond: None,
            effects: &[Eff::Points(10), Eff::San(10)], route: Route::To("pv_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：云海边缘的行脚武者 =================
    SceneDef {
        id: "pv_vagabond", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 云海栈道"),
        mood: "cold", speaker: Some("行脚武者"), voice: None,
        text: TextSpec::Static(&[
            "一个背着破旧行囊的行脚武者正就着云海盘坐，吃着半块干饼。见你上来，他咧嘴一笑：「咦？也是个不要命的，敢登这破虚台。」",
            "「我叫石头，四处讨武饭吃的。」他拍拍身边的地，「这片高地怪得很——越往上，拳脚越轻，天却越近。我猜它是想教人一件事：把功夫练到极致，是奔着「放下」去的。」",
            "他掰了半块饼递给你：「歇会儿吧。武的路长，别只顾着抬头那阵缝，忘了脚下的饼也是真的。」",
        ]),
        choices: &[ChoiceDef { label: "接过那半块干饼", sub: "AddItem 行脚干饼 · +10 点 · San+5", cond: None,
            effects: &[Eff::AddItem("pv_biscuit"), Eff::Points(10), Eff::San(5)], route: Route::To("pv_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫：境界松动 =================
    SceneDef {
        id: "pv_gather", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 放空心境"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你盘膝坐下，把多年的招式一件件从心里卸下——描至极致那套拳谱、剑冢温存的断剑、武馆旧拳谱里的那句话，都在这一刻翻涌上来。你察觉到，自己离武的尽头，其实已经很近很近了。",
            "就在你放空到极点的一瞬，那道天缝猛地震了一下，像被什么从另一侧「撞」了。整个高地微微发颤，云海翻涌着朝裂缝倒灌。",
            "你豁然睁开眼——不是你要过那道缝，是<em>缝的另一头，有东西正想过来。</em>",
        ]),
        choices: &[
            ChoiceDef { label: "向破虚台顶迎去", sub: "直面从缝那头来的人", cond: None, effects: &NO_EFF, route: Route::To("pv_prelude") },
            ChoiceDef { label: "先稳住心神", sub: "与山巅老者共守一口真元", cond: None, effects: &[Eff::San(5)], route: Route::To("pv_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pv_prelude", bg: Some("poxu_bg.png"), loc: Some("武极高地 · 破虚台前"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "天缝裂到最大，一道身影从光里缓步走出——它通体覆盖着像另一种武极练就的甲壳，脚不沾尘，周身气息沉沉压着整片高地。",
            "那是个「异界来者」，一个把武练到另一个世界尽头的人。它停在破虚台中央，那气息宣告着：它不只是来「看」这条天缝的——它是来印证，要跨过这道缝，得先过了它这一关。",
            "<em>「武的尽头是另一个世界的开始。」</em>——眼前这一战，便是两界武者的开局。",
        ]),
        choices: &[ChoiceDef { label: "举拳，正面相对", sub: "与异界来者印证武功", cond: None, effects: &NO_EFF, route: Route::To("pv_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "pv_01", bg: Some("poxu_bg.png"), loc: Some("破虚台 · 两界之界"), mood: "danger",
        speaker: Some("异界来者"), voice: None,
        text: TextSpec::Static(&["异界来者 挡在出口，气息锁着你周身八门。武的尽头，是另一个世界的开始——它正是那个世界的来者。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "目光锁定它的身形", sub: "+5 · 记下异界武的起手", cond: None, effects: &[Eff::Points(5)], route: Route::To("pv_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pv_round", bg: Some("poxu_bg.png"), loc: Some("决战 · 两界之界"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("异界来者 尚余 {} 气力，你 HP {}。每破它一招一式，两界的门就松动一分。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 pv_card） =================
    SceneDef {
        id: "pv_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["异界来者退回天缝另一侧，金光缓缓合拢。你在离开前，把脚下的破虚台最后看了一遍——武的尽头，原来就在你站过的地方。"]),
        choices: &[ChoiceDef { label: "记住这道天缝合拢的样子", sub: "+30 点 · 两界之界", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("pv_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pv_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走了那柄剑冢断剑。它始终没有重铸，但你明白——武者留它在这里，本就是等着有人替它兑现那句「武有尽头」。"]),
        choices: &[ChoiceDef { label: "携断剑下山", sub: "+25 点 · AddItem 破虚断剑 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("pv_broken_sword")], route: Route::To("pv_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pv_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你在破虚台旁又练了一遍最朴素的起手式。没有跨过缝，也没有回头——就像山巅老者说的，把武练到尽头，练的就是「往哪里放」这几十年。"]),
        choices: &[ChoiceDef { label: "再打一趟最本色的拳", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("pv_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "pv_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "武极境破虚 · 结算".into(), good: true,
                body_html: format!("<p>你在武的尽头与另一界的来者印证了一场。</p><p>天缝合拢，破虚台归于云海。武到了尽头，也可以只是开始。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "pv_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("侠行天下 · 武极境破虚 · 殒命两界", "在破虚台被异界来者击倒")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你倒在武的尽头。异界来者最后看了你一眼，转身回缝——它替你把「未尽之战」留在了破虚台上。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命两界】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn poxu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("pv_boss", FightCfg {
            name: "异界来者", hp: 320, dmg: (22, 34), reward: 500, reward_why: "击败 BOSS", intro: "异界来者跨出天缝——两界之界一战开启！",
            rage_at: Some(60), rage_text: "异界来者气息暴涨，破虚台为之震颤！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "pv_card".to_string(), death: "pv_death",
        }),
    ]
}
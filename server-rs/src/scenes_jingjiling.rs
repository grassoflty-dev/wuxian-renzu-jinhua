//! 寂静岭 · 表里世界 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「雾里有东西在敲。敲的不是门，是你心里那扇不敢开的门。」
//! 结构：开场(灰雾小镇) → jj_hub(调查中转) → 调查点/表里切换 + NPC → 声音逼近 → 三角头迎战 → 开放结局三选 → jj_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("jj_boss") {
            st.fight = Some(crate::power::scaled_fight("jj_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "jj_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "jj_death".to_string(); }
    "jj_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("jj_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "jj_reward");
    "jj_card".to_string()
}

pub static JINGJILING_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "jj_00", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 小镇入口"),
        mood: "danger", speaker: Some("旁白"), voice: Some("vo_jj_open"),
        text: TextSpec::Static(&[
            "灰雾涌进这座小镇，像一层会呼吸的墙。废弃的店铺、空荡的街道、无声的钟楼——一切都在雾里泡得发灰，唯有空气里那股「刚有人来过」的气息挥之不去。",
            "你往前走了一步，脚下的路面却忽然剥落成锈蚀的铁锈色；再一步，它又恢复成寻常的柏油。这个小镇在两种样子之间轻轻晃动——像有人在梦里反复换着滤镜。",
            "雾的那一头，传来均匀的、不紧不慢的敲击声。<em>「笃、笃、笃」</em>——像在敲一面墙，又像在敲你的脑壳。",
        ]),
        choices: &[
            ChoiceDef { label: "循着敲击声走", sub: "向声音的来处", cond: None, effects: &NO_EFF, route: Route::To("jj_close") },
            ChoiceDef { label: "先稳住，看表里切换", sub: "观察这个镇子的两种样子", cond: None, effects: &NO_EFF, route: Route::To("jj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "jj_hub", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 主街"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你站在主街上，灰雾在你四周流动。这个镇子像一张摊开的地图，每个转角都可能通向「表」或「里」的一种样子——锈蚀的、或是干净的；喧闹的旧影，或是无人问津的空屋。",
            "敲击声仍在远处回响。你还有时间把这座既熟悉又陌生的镇子看个清楚。",
        ]),
        choices: &[
            ChoiceDef { label: "雾巷里的收音机", sub: "调频，听一听信号里的声音", cond: None, effects: &NO_EFF, route: Route::To("jj_radio") },
            ChoiceDef { label: "废弃医院", sub: "锈蚀的走廊与病床", cond: None, effects: &NO_EFF, route: Route::To("jj_hospital") },
            ChoiceDef { label: "旧校舍", sub: "表里的教室", cond: None, effects: &NO_EFF, route: Route::To("jj_school") },
            ChoiceDef { label: "歪斜的漫画书屋", sub: "一间叠着两层的房子", cond: None, effects: &NO_EFF, route: Route::To("jj_comic") },
            ChoiceDef { label: "雾里有人影在躲闪", sub: "追上那点活的动静", cond: None, effects: &NO_EFF, route: Route::To("jj_survivor") },
            ChoiceDef { label: "听任敲击声引路", sub: "走向声音的主人", cond: None, effects: &NO_EFF, route: Route::To("jj_close") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：雾巷收音机 =================
    SceneDef {
        id: "jj_radio", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 雾巷口的收音机"),
        mood: "mystery", speaker: Some("收音机·沙沙作响"), voice: None,
        text: TextSpec::Static(&[
            "雾巷口摆着一台老式收音机，旋钮被人拧到一半，正发出嘈杂的雪花声。你俯身细听，沙沙声里隐隐夹着一段几乎听不清的人声，反复重复着同一句话。",
            "「……它把这条路……留给我们……回去，别再让它……敲开。」信号断断续续，忽强忽弱。你旋着旋钮，那声音在某一格上变得清晰，又从下一格消失——像这个小镇的「表里」，永远隔着一格。",
        ]),
        choices: &[ChoiceDef { label: "记住那段沙沙的劝告", sub: "MarkPoint · +15 点 · 静岭的信号", cond: None,
            effects: &[Eff::MarkPoint("jj_pt_radio"), Eff::Points(15), Eff::San(-5)], route: Route::To("jj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：废弃医院 =================
    SceneDef {
        id: "jj_hospital", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 废弃医院"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "医院的大厅空着，电梯门却开开合合，像有什么在不大的楼层间来回徘徊。墙上的病历被撕得只剩一角，挂号处积着厚厚的灰。",
            "你推门进了一间病房——「表」的它是一间整洁的病室，「里」的它爬满铁锈与暗痕。两种画面在你的视野里互相覆盖、拉扯，直到你说不清哪一帧才是真的。",
        ]),
        choices: &[ChoiceDef { label: "拾起一张旧病历", sub: "MarkPoint · AddItem 医院病历 · +15 点", cond: None,
            effects: &[Eff::MarkPoint("jj_pt_hospital"), Eff::AddItem("jj_file"), Eff::Points(15)], route: Route::To("jj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：旧校舍 =================
    SceneDef {
        id: "jj_school", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 旧校舍"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "校舍的黑板擦了一半，粉笔字只剩半句话：「记住你从哪来。」课桌椅整整齐齐，像是孩子们刚课间离开，铃一响就会跑回来。",
            "你抬头，教学楼的走廊尽头钉着一张逃生路线图，图上的楼梯却和现实里的方向正好相反。你把图记进心里——在这个镇子里，连「路线」都可能骗你。",
        ]),
        choices: &[ChoiceDef { label: "记下那张反着的逃生图", sub: "MarkPoint · +15 点 · 表里的教室", cond: None,
            effects: &[Eff::MarkPoint("jj_pt_school"), Eff::Points(15)], route: Route::To("jj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：歪斜的漫画书屋 =================
    SceneDef {
        id: "jj_comic", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 漫画书屋"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一间两层楼的书屋歪斜地立在转角，一楼堆满连环画，二楼却只摆着一面巨大的、蒙尘的镜子。你不经意一瞥——镜子里照出的走廊，和现实里隔着一间屋子。",
            "你伸手想去碰那面镜子，镜面却在指腹触及前泛起一圈涟漪。这本书屋解释不了一件事：当「里」的世界盖住「表」的世界时，到底是哪一边在照镜子。",
        ]),
        choices: &[ChoiceDef { label: "把镜中走廊记下", sub: "MarkPoint · San-5 · 交错的里表", cond: None,
            effects: &[Eff::MarkPoint("jj_pt_comic"), Eff::San(-5)], route: Route::To("jj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：雾里的避难者 =================
    SceneDef {
        id: "jj_survivor", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 关紧门窗的屋子"),
        mood: "cold", speaker: Some("避难者 · 惊惶"), voice: None,
        text: TextSpec::Static(&[
            "一扇只开一条缝的门后，露出半张苍白紧张的脸：「嘘——别敲，别敲门。你怎么还走在雾里？」她压低声音喘着气，把你拉进门，塞给你一盏手电。",
            "「它就在镇里游荡，拖着个大铁锤，专门砸响那些没关严的东西。」她眼睛发红，「可奇怪的是……它敲的不是门，是我们心里那些早就该忘了的事。」",
            "「我不骗你，也说不出它是什么。」她抱紧自己，「你记住一点就够——别让它敲开你。」",
        ]),
        choices: &[ChoiceDef { label: "收下她递来的手电", sub: "AddItem 旧手电 · +10 点 · San-5", cond: None,
            effects: &[Eff::AddItem("jj_flashlight"), Eff::Points(10), Eff::San(-5)], route: Route::To("jj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：往事的传道者 =================
    SceneDef {
        id: "jj_pastor", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 半塌的礼拜堂"),
        mood: "mystery", speaker: Some("传道者 · 低声"), voice: None,
        text: TextSpec::Static(&[
            "一尊半塌的礼拜堂里，一个披着褪色披肩的人跪在圣坛前，正自语般地说：「表里……不过是同一件事的两种说法。」",
            "他转头看你，眼神清透得反常：「这个镇子从不制造恐惧，它只把你心里已有的、掖着的一点事，放大了给你们看。镜子里有另一条走廊——那不是别的，是你自己岔开的心。」",
            "「它手里那把锤子，是替每一个不敢面对的人敲的。」他低声笑了笑，「你有勇气听吗？敲门的，从来都是你自己。」",
        ]),
        choices: &[ChoiceDef { label: "听完这位传道者的话", sub: "+10 点 · 表与里", cond: None,
            effects: &[Eff::Points(10), Eff::San(10)], route: Route::To("jj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫：声音逼近 =================
    SceneDef {
        id: "jj_close", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 敲击声的源头"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你顺着敲击声走进镇子最深处。灰雾在这里变得又稠又冷，「表里」切换得越来越快——整洁的街道、锈蚀的街道，像两张幻灯片在你眼前疯狂轮换。",
            "敲击声越来越近，终于，在一堵只剩半边的墙前停下。你屏住呼吸——那声音不是从墙外传来，而是从墙里、从你脑内传来的。<em>它在等你走近。</em>",
        ]),
        choices: &[
            ChoiceDef { label: "走向那堵墙", sub: "看看墙后面是什么", cond: None, effects: &NO_EFF, route: Route::To("jj_prelude") },
            ChoiceDef { label: "退后一步，重新系紧鞋带", sub: "稳一稳心里那扇门", cond: None, effects: &[Eff::San(5)], route: Route::To("jj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "jj_prelude", bg: Some("jingjiling_bg.png"), loc: Some("寂静岭 · 锈蚀的十字街心"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "那半堵墙塌了下来——准确的说是「里」的世界伸了过来。灰雾尽头，一个巨大的身影缓缓挪步，裹着深色风衣，手里拖着一只硕大的、像工地旁的三角铁块。",
            "它没有脸，只有那枚遮住整个头部的钢筋「三角头」。它停在你面前，锤子落地，发出沉重的闷响。<em>雾里有东西在敲——它，就是你心里不敢应的那个声音。</em>",
        ]),
        choices: &[ChoiceDef { label: "抬步，正视三角头", sub: "直面雾里敲门的「它」", cond: None, effects: &NO_EFF, route: Route::To("jj_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "jj_01", bg: Some("jingjiling_bg.png"), loc: Some("锈蚀十字街心"), mood: "danger",
        speaker: Some("三角头"), voice: None,
        text: TextSpec::Static(&["三角头 挡在出口，拖着铁锤一步一步靠近。雾里有东西在敲——它替你敲开那扇门。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "握紧手电，分辨虚实", sub: "+5 · 看穿里表", cond: None, effects: &[Eff::Points(5)], route: Route::To("jj_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "jj_round", bg: Some("jingjiling_bg.png"), loc: Some("决战 · 十字街心"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("三角头 尚余 {} 气力，你 HP {}。每打碎一段锈蚀，心里的门就锁紧一分。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 jj_card） =================
    SceneDef {
        id: "jj_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["三角头在雾里缓缓退去，锈蚀褪成寻常小镇的安宁。你在离开前，最后看了一次这座终于停止「表里切换」的镇子。"]),
        choices: &[ChoiceDef { label: "记住雾散后的寂静岭", sub: "+30 点 · 里表的尽头", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("jj_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "jj_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走了一张旧病历。回到雾外，那上面的字迹却空荡荡——像这个镇子只肯把它的某一面，留给你带走。"]),
        choices: &[ChoiceDef { label: "把病历收进背包", sub: "+25 点 · AddItem 旧病历 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("jj_file")], route: Route::To("jj_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "jj_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你在镇口坐了很久，直到敲击声完全停下。灰雾最后一次漫过你脚边，然后缓缓退去，露出这座镇子原本的、干干净净的样子。"]),
        choices: &[ChoiceDef { label: "等到雾彻底散尽", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("jj_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "jj_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "寂静岭 · 结算".into(), good: true,
                body_html: format!("<p>你穿过了寂静岭的表与里，也关上了心里那扇门。</p><p>雾散之后，小镇重归沉默。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "jj_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("寂静岭 · 表里世界 · 殒命于雾", "在绣蚀十字街心被三角头击倒")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你没能关上心里那扇门。寂静岭的雾把你连同那把锤子的回声，一起收进了「里」的那一面。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命于雾】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn jingjiling_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("jj_boss", FightCfg {
            name: "三角头", hp: 180, dmg: (16, 24), reward: 500, reward_why: "击败 BOSS", intro: "铁锤重重落地——三角头停下了它的「敲」！",
            rage_at: Some(60), rage_text: "三角头举锤猛砸，锈蚀的地面迸出火星！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "jj_card".to_string(), death: "jj_death",
        }),
    ]
}
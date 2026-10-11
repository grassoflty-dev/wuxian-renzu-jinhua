//! 猛鬼街 · 弗莱迪梦境 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「别睡着。睡着了，这里就成了它的猎场。」
//! 结构：开场(梦境锅炉房) → mg_hub(调查中转) → 调查点/梦境锅炉房奇观 + NPC → 困意铺垫 → 弗莱迪迎战 → 开放结局三选 → mg_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("mg_boss") {
            st.fight = Some(crate::power::scaled_fight("mg_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "mg_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "mg_death".to_string(); }
    "mg_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("mg_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "mg_reward");
    "mg_card".to_string()
}

pub static MENGGUIJIE_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "mg_00", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 锅炉房"),
        mood: "danger", speaker: Some("衔梦者 · 低语"), voice: Some("vo_mg_open"),
        text: TextSpec::Static(&[
            "你睁开眼时，正站在一间巨大的锅炉房里。铜绿的管道交缠着拱过黑乎乎的天顶，锅炉的红光一明一灭，把地面烤得发烫。",
            "你说不清自己是什么时候走进这里的——或许你已经睡着很久了。空气里有股铁锈的、带着焦糖的怪味，一个声音很轻地在你耳边说：<em>「别睡。睡着了，它就来接你。」</em>",
            "锅炉深处传来指甲轻轻刮过金属的声响。你在梦里，可这个梦，认床。",
        ]),
        choices: &[
            ChoiceDef { label: "掐自己一把，确认在梦里", sub: "San-5 · 记住别睡", cond: None, effects: &[Eff::San(-5)], route: Route::To("mg_hub") },
            ChoiceDef { label: "环顾锅炉房", sub: "摸清这片梦境的边缘", cond: None, effects: &NO_EFF, route: Route::To("mg_basement") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "mg_hub", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 锅炉房边缘"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "锅炉房的尽头裂开几道门，每一道都通向这个梦的不同角落。红光在你身后鼓动着，像是催促，又像是挽留。",
            "清醒的代价是困意不断涌上来。你知道——在这座用睡眠搭建的迷宫里，任何一丝松懈，都会让某扇门被从里面敲开。",
        ]),
        choices: &[
            ChoiceDef { label: "锅炉迷宫", sub: "摸清蒸汽与管道的走向", cond: None, effects: &NO_EFF, route: Route::To("mg_basement") },
            ChoiceDef { label: "梦里的大钟楼", sub: "老井与列车轨的梦境", cond: None, effects: &NO_EFF, route: Route::To("mg_dream") },
            ChoiceDef { label: "泉水街", sub: "梦与现实交界的街道", cond: None, effects: &NO_EFF, route: Route::To("mg_spring") },
            ChoiceDef { label: "墙上的涂鸦", sub: "读懂「它」留下的字", cond: None, effects: &NO_EFF, route: Route::To("mg_graffiti") },
            ChoiceDef { label: "梦境里迷路的孩子们", sub: "与梦中的居民交谈", cond: None, effects: &NO_EFF, route: Route::To("mg_kids") },
            ChoiceDef { label: "任困意漫上来", sub: "接近睡意的边缘", cond: None, effects: &NO_EFF, route: Route::To("mg_drowsy") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：锅炉迷宫 =================
    SceneDef {
        id: "mg_basement", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 锅炉迷宫"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "你沿着锅炉间唯一的窄梯爬进管道的夹层，这里像是一座倒扣的、被蒸汽包裹的宫殿。阀门上的指针疯狂旋转，却没有一处是真的。",
            "蒸汽在冷光里凝成一张张模糊的人脸，又散开。这是梦的「骨骼」——你把它们组成线条的逻辑牢牢记下，仿佛记住了这座梦境里唯一真实的几何。",
        ]),
        choices: &[ChoiceDef { label: "记下管道走向", sub: "MarkPoint · +15 点 · 梦的地图", cond: None,
            effects: &[Eff::MarkPoint("mg_pt_basement"), Eff::Points(15)], route: Route::To("mg_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：梦里的大钟楼/老井 =================
    SceneDef {
        id: "mg_dream", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 大钟楼"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一栋钟楼的轮廓立在梦的深处，钟面却是空的。钟楼脚下横着一口填了一半的老井，井沿上放着几颗磨得发亮的弹珠。",
            "你听见钟摆声，却找不到钟摆；井里明明干涸，却泛起水汽的凉意。这是梦的另一个切面——它把白天清醒时见过的一切，悄悄揉碎、重组，懒得解释。",
        ]),
        choices: &[ChoiceDef { label: "拾起一颗弹珠", sub: "MarkPoint · AddItem 梦中弹珠 · San+5", cond: None,
            effects: &[Eff::MarkPoint("mg_pt_dream"), Eff::AddItem("mg_marble"), Eff::San(5)], route: Route::To("mg_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：泉水街 =================
    SceneDef {
        id: "mg_spring", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 泉水街"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一条潮湿的街道铺在你脚下。路标写着「水源街」，路灯一盏盏亮起，又诡异地同时熄灭。梦里的街道没有车，只有远处一个提着塑料袋的背影，慢悠悠地消失在雾里。",
            "你隐约觉得这条街在原野上是真实存在的——那条日夜叮当的火车线，也曾哐当作响地穿过这里。你分不清自己站的是真实的地砖，还是被梦拼接的冒牌货。",
        ]),
        choices: &[ChoiceDef { label: "记下这条街的名字", sub: "MarkPoint · +15 点 · 梦的原乡", cond: None,
            effects: &[Eff::MarkPoint("mg_pt_spring"), Eff::Points(15)], route: Route::To("mg_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：墙上的涂鸦 =================
    SceneDef {
        id: "mg_graffiti", bg: Some("mengguijie_bg.png"), loc: Some("锅炉房 · 墙上的字"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "锅炉房粗粝的砖墙上，密密麻麻刻着同一句话，旧的被新的覆盖：「弗莱迪在梦乡等你。」「别睡，别睡，别睡。」",
            "字迹尺幅不一，像是不同的人在几乎崩溃的清醒里用指甲划下的。最底下那行潦草的字旁，画着一顶宽檐帽的侧影——它偶尔出现在你视野边缘，又立刻隐去。",
        ]),
        choices: &[ChoiceDef { label: "用指尖描一遍那行字", sub: "MarkPoint · San-5 · 记下警告", cond: None,
            effects: &[Eff::MarkPoint("mg_pt_graffiti"), Eff::San(-5)], route: Route::To("mg_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：梦里迷路的孩子们 =================
    SceneDef {
        id: "mg_kids", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 地下游乐场"),
        mood: "cold", speaker: Some("孩子们 · 回声"), voice: None,
        text: TextSpec::Static(&[
            "几个穿着睡衣的孩子挤在一间没有门的房间里，画纸散了一地。最小的那个抬头看你，平静得近乎故扮：「你也睡不着吗？」",
            "「我们早就不睡了。一睡着，就会被带走。」一个孩子顿了顿，「大人们说，别怕，弗莱迪只是梦里的小丑。可谁都没能醒着数完他手指上的刀。」",
            "他们递给你半截蜡笔：「留着吧——这是我们从那个梦里唯一带得回来的东西。」",
        ]),
        choices: &[ChoiceDef { label: "收下那半截蜡笔", sub: "AddItem 蜡笔 · +10 点 · San+5", cond: None,
            effects: &[Eff::AddItem("mg_crayon"), Eff::Points(10), Eff::San(5)], route: Route::To("mg_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：不敢闭眼的守夜人 =================
    SceneDef {
        id: "mg_parent", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 老屋门廊"),
        mood: "cold", speaker: Some("守夜人 · 一位父亲"), voice: None,
        text: TextSpec::Static(&[
            "门廊的摇椅上坐着一个男人，眼皮底下满满都是血丝。他剥着一只早已剥完的橙子，声音沙哑：「我在这个梦里找了三年。他们说，只要我不睡，它就没法来带孩子走。」",
            "「可人不睡会死的呀。」他抬头，眼眶通红，露出一个疲惫至极的笑，「所以我换着法子睡——睁着一只眼睡，站着睡……只要梦里有人醒着，它就不敢来敲门。」",
            "他望着门廊尽头某个黑洞洞的走廊：「你要是困了，就到这儿来。我守着，你歇会儿。」",
        ]),
        choices: &[ChoiceDef { label: "向他道谢并站一会儿岗", sub: "+10 点 · 守夜的人", cond: None,
            effects: &[Eff::Points(10), Eff::San(10)], route: Route::To("mg_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫：困意 =================
    SceneDef {
        id: "mg_drowsy", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 困意边缘"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "困意像一床沉重的湿棉被，一点点压上你的眼皮。锅炉的红光开始扭曲，砖墙的缝隙里渗出寒意——睡着，就对这个世界敞开了大门。",
            "你猛地睁眼，用指甲掐住掌心，把那股睡意钉在原地。你意识到：在这个梦里，没睡着的恐惧感，恰恰是你唯一的「清醒」——而它正在一点点耗尽。",
        ]),
        choices: &[
            ChoiceDef { label: "撑住不睡，循着重喘声前进", sub: "直面梦境的主人", cond: None, effects: &NO_EFF, route: Route::To("mg_prelude") },
            ChoiceDef { label: "先退到守夜人那里缓一缓", sub: "守住清醒", cond: None, effects: &[Eff::San(5)], route: Route::To("mg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mg_prelude", bg: Some("mengguijie_bg.png"), loc: Some("锅炉房 · 最深一扇门"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "最深一扇门被蒸汽顶开一条缝，一截露出金属指甲的手搭在了门框上。那顶宽檐帽的剪影，把整间锅炉房的红光都吸了过去。",
            "一个拖长的、带着笑意的声音在你脑中响起：「来了啊……真乖。我都等你困得不行了。」门里，是弗莱迪的猎场。<em>——别睡着。这一次，它很清醒。</em>",
        ]),
        choices: &[ChoiceDef { label: "踏进门去", sub: "正面迎战弗莱迪", cond: None, effects: &NO_EFF, route: Route::To("mg_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "mg_01", bg: Some("mengguijie_bg.png"), loc: Some("梦境 · 弗莱迪的锅炉房"), mood: "danger",
        speaker: Some("弗莱迪·克鲁格"), voice: None,
        text: TextSpec::Static(&["弗莱迪 挡在出口，咧开嘴指着自己手套上的刀刃。别睡着。睡着了，就是它的。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "紧握清醒，先观察", sub: "+5 · 记下它的动作", cond: None, effects: &[Eff::Points(5)], route: Route::To("mg_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mg_round", bg: Some("mengguijie_bg.png"), loc: Some("决战 · 梦境锅炉房"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("弗莱迪 尚余 {} 气力，你 HP {}。清醒一寸，它就弱一分。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 mg_card） =================
    SceneDef {
        id: "mg_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["梦境在弗莱迪倒下后一寸寸褪色，锅炉房的红光熄灭。你在离开前，最后看了一次这座困住无数人的迷宫。"]),
        choices: &[ChoiceDef { label: "记住这座梦乡", sub: "+30 点 · 梦的边界", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("mg_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mg_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走了那半截蜡笔。回到清醒世界后，它的笔尖却始终画不出一滴水——你知道那是梦里带回的、只有你自己看得见的颜色。"]),
        choices: &[ChoiceDef { label: "把蜡笔收好", sub: "+25 点 · AddItem 蜡笔 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("mg_crayon")], route: Route::To("mg_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mg_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你在锅炉房边缘坐了很久，直到梦完全退去。再睁开眼时，天光正透过真正的窗——你没有睡，也终于能睡了。"]),
        choices: &[ChoiceDef { label: "安心地让梦退场", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("mg_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "mg_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "猛鬼街 · 结算".into(), good: true,
                body_html: format!("<p>你从弗莱迪的梦乡里夺回了清醒。</p><p>锅炉房的红光熄了，它不再接走梦中人。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "mg_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("猛鬼街 · 弗莱迪梦境 · 殒命于梦", "在弗莱迪的梦乡中闭上眼")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你终究没能撑住清醒。梦乡收走了你，锅炉房的红光里，多了一个没有名字的剪影。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命于梦】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn mengguijie_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("mg_boss", FightCfg {
            name: "弗莱迪·克鲁格", hp: 190, dmg: (16, 26), reward: 500, reward_why: "击败 BOSS", intro: "弗莱迪从锅炉的阴影里走出——它的猎场亮了！",
            rage_at: Some(60), rage_text: "弗莱迪狂笑着逼近，爪子刮过金属！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "mg_card".to_string(), death: "mg_death",
        }),
    ]
}
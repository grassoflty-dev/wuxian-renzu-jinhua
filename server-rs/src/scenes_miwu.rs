//! 无限恐怖 · 迷雾 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、被困幸存者 NPC、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「雾里最可怕的，是雾里回来的人。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("mw_boss") {
            st.fight = Some(crate::power::scaled_fight("mw_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "mw_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "mw_death".to_string(); }
    "mw_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("mw_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "mw_reward");
    "mw_end_choice".to_string()
}

pub static MIWU_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "mw_00", bg: Some("miwu_bg.png"), loc: Some("迷雾 · 超市门口"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你踏入了「无限恐怖 · 迷雾」。",
            "无边的灰白色浓雾吞没了整座小镇，你慌忙躲进路边的一家超市。门一锁上，你才听见外面雾里传来低沉的、不属于活物的拖曳声。",
            "货架后，一个脸色苍白的店员攥着钢管，压着声音说:「记住一件事——雾里回来的，都不是原来那个人了。」",
        ]),
        choices: &[
            ChoiceDef { label: "问清雾中邪物", sub: "San+5 · 警惕", cond: None, effects: &[Eff::San(5)], route: Route::To("mw_hub") },
            ChoiceDef { label: "先进货架深处", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "mw_hub", bg: Some("miwu_bg.png"), loc: Some("超市 · 中庭货架区"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "超市被强行改造成了一座临时的避难堡垒。货架被推成工事，冷柜成了掩体，幸存者把能找到的吃喝都堆在了收银台后面。",
            "窗外浓雾始终不散，偶尔会有几道模糊的轮廓贴着玻璃慢慢滑过。你分不清那是雾，还是雾里回来的『人』。",
        ]),
        choices: &[
            ChoiceDef { label: "察看雾封的橱窗", sub: "窗外轮廓 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("mw_pt_window") },
            ChoiceDef { label: "读货架后的留言", sub: "前人遗言 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("mw_pt_note") },
            ChoiceDef { label: "端详冷藏柜后面", sub: "深处的门 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("mw_pt_coldroom") },
            ChoiceDef { label: "查雾侵的药品架", sub: "灰斑 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("mw_pt_pharmacy") },
            ChoiceDef { label: "听店员讲旧事", sub: "幸存者 · 店员", cond: None, effects: &NO_EFF, route: Route::To("mw_np_clerk") },
            ChoiceDef { label: "聊被困的女人", sub: "幸存者 · 芮", cond: None, effects: &NO_EFF, route: Route::To("mw_np_depot") },
            ChoiceDef { label: "走向冷库深处的巨物", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("mw_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "mw_pt_window", bg: Some("miwu_bg.png"), loc: Some("超市 · 落地橱窗"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你贴着落地橱窗朝外望，能见度不足十米。雾里偶有模糊的轮廓贴着玻璃滑过，有的像人，有的根本不像。",
            "货架边残缺登记的店员说过，雾里的东西不敢进光。可你分明看见，其中一条轮廓在玻璃外停了几秒，正直勾勾地‘看’着你。",
            "你后退一步，它才缓缓滑远。那轮廓的落寞，让你几乎忘记它曾是活物。",
        ]),
        choices: &[
            ChoiceDef { label: "隔玻璃目送那轮廓", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("mw_pt_1"), Eff::Points(15), Eff::San(-5)], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_pt_note", bg: Some("miwu_bg.png"), loc: Some("超市 · 货架检修间"), mood: "sad",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "货架后面的检修间墙上钉满了便签，是不同时间被困在这里的人留下的。最老的一张已经卷边，墨水淡得几乎看不清。",
            "你照着念:「第三天，我们把马克忘在雾里了。第六天，他回来了。可他不是马克了。」",
            "下面叠着无数张相似的纸，字迹越来越急促，最后一张没写完，只深深划了一道。",
        ]),
        choices: &[
            ChoiceDef { label: "读尽留言墙", sub: "调查完成 · +20点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("mw_pt_2"), Eff::Points(20), Eff::San(-5)], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_pt_coldroom", bg: Some("miwu_bg.png"), loc: Some("超市 · 冷库门前"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "超市最深处的冷库门被人从里面用链子锁死了。锁上挂着十几把齿形各异的钥匙——它们不是来开锁的，是来拦着门不让人进。",
            "门缝里不断渗出白雾，冷库明明早已断电，可里面却像有什么东西在让空气自动凝结成冰。",
            "你摸到门缝边一处划痕——那似乎是被人从‘里面’一寸寸刻下的三个字:「别进来。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下门上的警告", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("mw_pt_3"), Eff::Points(20), Eff::San(-3)], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_pt_aisle", bg: Some("miwu_bg.png"), loc: Some("超市 · 罐头货道"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "罐头货道被扫得半空，幸存者把能吃的都集中到了这里。货架标价牌还立着，可日期早已翻过了这场灾难开始的点。",
            "在货架最底层，你翻到一本被垫在罐头底下的《小镇图册》。雾来临前的街道照片，整洁明朗，和此刻窗外判若两界。",
            "你把图册合上。这片迷雾，多半是某个不该存在的东西，把整座小镇连人带梦一起卷了进来。",
        ]),
        choices: &[
            ChoiceDef { label: "带走那本旧图册", sub: "调查完成 · +20点 · 得图册纪念", cond: None,
                effects: &[Eff::MarkPoint("mw_pt_4"), Eff::Points(20), Eff::AddItem("mw_atlas")], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_pt_pharmacy", bg: Some("miwu_bg.png"), loc: Some("超市 · 药品架"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "药品架被翻得七零八落，幸存者把能用的药都抢走了。可架子的死角里，还躺着几瓶未开封的褪热药，瓶身上覆着一层细细的灰斑。",
            "你捻起那瓶药——灰斑不是灰尘，像是从那雾里析出的、凝固的东西。你想起芮的话：雾里回来的‘人’，身体会渐渐蒙上一层同样的灰。",
            "你把药瓶放回原处。也许这场雾不是从外面来的。也许它一开始，就藏在某个人吃过的、这片被施了什么东西的药里。",
        ]),
        choices: &[
            ChoiceDef { label: "记下药瓶灰斑", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("mw_pt_5"), Eff::Points(15), Eff::San(-5)], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "mw_np_clerk", bg: Some("miwu_bg.png"), loc: Some("超市 · 收银台工事"), mood: "calm",
        speaker: Some("超市店员 · 邹望"), voice: None,
        text: TextSpec::Static(&[
            "店员邹望抱着一杆扳手守在收银台后，眼圈乌黑:「雾来那天，我们在冷库杀了几个冲进来的。可后来出去找食物的人，一个都没回来。」",
            "他压低声音:「我们一直以为雾里的东西不敢进光。可后来我发现，它们不是怕光——它们只是‘学会了等我们关灯的那一晚’。」",
            "他把一段钢钎递给你:「你看着像能打的人。别一个人去冷库深处，那雾里最大的东西，就蹲在超市下面。」",
        ]),
        choices: &[
            ChoiceDef { label: "收下钢钎", sub: "NPC 对话 · +15点 · 得钢钎", cond: None,
                effects: &[Eff::Points(15), Eff::AddItem("mw_bar")], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_np_depot", bg: Some("miwu_bg.png"), loc: Some("超市 · 员工区"), mood: "sad",
        speaker: Some("被困女人 · 芮"), voice: None,
        text: TextSpec::Static(&[
            "被困员工区的女人芮抱着丈夫的外套，呆望着窗外:「我丈夫是雾来的第一天冲出去找药的。两天后他回来了……」",
            "她声音发抖:「他没受伤，没瘦，甚至还在门口朝我笑。可我知道那不是他——因为真正的他，从来不会叫我女儿的名字。」",
            "芮抬起头，那双眼睛里有泪也有恐惧:「你们都说雾里最可怕的是邪物。可雾里回来的‘人’，才是最可怕的。」",
        ]),
        choices: &[
            ChoiceDef { label: "安静听芮讲完", sub: "NPC 对话 · +15点 · San-5", cond: None,
                effects: &[Eff::Points(15), Eff::San(-5)], route: Route::To("mw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "mw_01b_prep", bg: Some("miwu_bg.png"), loc: Some("超市地下 · 雾渊"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "超市地下的通风井被冲开，一股浓得发黑的白雾顺着楼梯翻涌上来。那一刻，你听见冷库深处传来一声低沉的、像在吞咽什么的震动。",
            "雾的最深处，蜷着一头几乎与整间超市等宽的‘雾中巨物’。它通体由灰白的雾凝成，五官稀薄得随时会被吹散——它似乎是这片迷雾的‘心脏’。",
            "浓雾猛地翻卷鼓胀，它睁开了眼。那不是一头野兽，是这片雾想把一切活物都拖回它这里来的具象。",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 雾中巨物】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "再看一眼留言墙", sub: "San-3 · 了然", cond: None, effects: &[Eff::San(-3)], route: Route::To("mw_pt_note") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景
    SceneDef {
        id: "mw_01", bg: Some("miwu_bg.png"), loc: Some("超市地下 · 雾渊"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["雾中巨物 挡在出口。雾里最可怕的，是雾里回来的人。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("mw_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_round", bg: Some("miwu_bg.png"), loc: Some("超市地下 · 决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 mw_card） =====
    SceneDef {
        id: "mw_end_choice", bg: Some("miwu_bg.png"), loc: Some("决战之后 · 超市门口"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当雾中巨物在最后一击里崩散成一团触手可及的灰雾，整座超市上空的浓雾也跟着撕开一道裂缝，漏进久违的日光。",
            "幸存者们纷纷冲出来,在光里又哭又笑。你回头看了一眼，那些被雾卷进来的‘人’，在阳光里一个接一个地淡去、消失。",
            "你站在雾与光的交界，呼出一口长气。这场把整座小镇拖进灰里的噩梦，你自己决定要怎么和它告别。",
        ]),
        choices: &[
            ChoiceDef { label: "迎光走出雾界", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("mw_sight")], route: Route::To("mw_card") },
            ChoiceDef { label: "带走那本旧图册", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("mw_memento")], route: Route::To("mw_card") },
            ChoiceDef { label: "留下等人聚拢", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("mw_stay")], route: Route::To("mw_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "mw_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>你活着走出了那场雾，也从雾里把更多人一起带回了光里。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "mw_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("无限恐怖 · 迷雾 · 殒命", "殒命于无限恐怖 · 迷雾")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn miwu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("mw_boss", FightCfg {
            name: "雾中巨物", hp: 220, dmg: (18, 28), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "mw_card".to_string(), death: "mw_death",
        }),
    ]
}
//! 洪荒历 · 函谷关攻防 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、居民 NPC、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「人族的城墙，是最后一道。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("hg_boss") {
            st.fight = Some(crate::power::scaled_fight("hg_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "hg_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "hg_death".to_string(); }
    "hg_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("hg_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "hg_reward");
    "hg_end_choice".to_string()
}

pub static HANGU_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "hg_00", bg: Some("hangu_bg.png"), loc: Some("函谷关口 · 城墙下"), mood: "tension",
        speaker: Some("旁白"), voice: Some("vo_hangu_1"),
        text: TextSpec::Static(&[
            "你踏入了「洪荒历 · 函谷关攻防」。",
            "朔风卷着兽潮的腥气扑面而来。关外的火把铺满地平线，那是数量多到让人腿软的围城军。",
            "老卒拄着断枪，望着那些火把低声道：「人族的城墙，是最后一道。守它的人，早就把命许进去了。」",
        ]),
        choices: &[
            ChoiceDef { label: "登城远眺", sub: "San+5 · 看清局势", cond: None, effects: &[Eff::San(5)], route: Route::To("hg_hub") },
            ChoiceDef { label: "先巡视关内", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "hg_hub", bg: Some("hangu_bg.png"), loc: Some("函谷关 · 主街"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "关内灯火稀疏却坚定。民夫抬着箭箱穿梭，铁匠铺炉火不熄，墙头老卒往复巡哨。",
            "这是一个被围困却不肯熄火的世界。你知道，这一夜的城墙承载着某样比命更重的东西。",
        ]),
        choices: &[
            ChoiceDef { label: "登北城墙", sub: "远眺兽潮 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("hg_pt_wall") },
            ChoiceDef { label: "访铁匠铺", sub: "关内炉火 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("hg_pt_forge") },
            ChoiceDef { label: "观关内祭坛", sub: "旧日图腾 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("hg_pt_altar") },
            ChoiceDef { label: "查枯井封痕", sub: "夜半阴声 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("hg_pt_well") },
            ChoiceDef { label: "看关内箭楼", sub: "满城箭簇 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("hg_pt_arsenal") },
            ChoiceDef { label: "与送粮信使交谈", sub: "NPC · 阿宁", cond: None, effects: &NO_EFF, route: Route::To("hg_np_people") },
            ChoiceDef { label: "与守关晁伯交谈", sub: "NPC · 老将", cond: None, effects: &NO_EFF, route: Route::To("hg_np_chaobo") },
            ChoiceDef { label: "走向决战关隘", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("hg_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "hg_pt_wall", bg: Some("hangu_bg.png"), loc: Some("北城墙 · 女墙"), mood: "awe",
        speaker: Some("老卒"), voice: None,
        text: TextSpec::Static(&[
            "北城墙上风沙如刀。关外黑压压的兽潮绵延到天际，中央一面残破军旗下，立着那个几乎无人敢直视的身影。",
            "老卒望着那影，声音发涩：「三年前，他是守城人。那晚兽潮破门，他一个人挡了六天，最后被人从尸堆里抬回来……」",
            "「可他回来后，就再没变回『人』了。守到最后的人，往往先死的是心。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下这段旧事", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("hg_pt_1"), Eff::Points(20)], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_pt_forge", bg: Some("hangu_bg.png"), loc: Some("关内 · 铁匠铺"), mood: "calm",
        speaker: Some("铁匠阿岩"), voice: None,
        text: TextSpec::Static(&[
            "铁匠阿岩把一块烧红的铁敲得火花四溅：「这口炉子啊，从先人建关那天就没歇过。墙上的每一片甲，城下的每一杆枪，都从这儿出去。」",
            "他从灰里翻出一枚发黑的护符塞给你：「旧朝铸的，说是护城墙的。你带着吧，说不定能少挨一刀。」",
        ]),
        choices: &[
            ChoiceDef { label: "收下护符", sub: "调查完成 · +20点 · 得护身符", cond: None,
                effects: &[Eff::MarkPoint("hg_pt_2"), Eff::Points(20), Eff::AddItem("hg_talisman")], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_pt_altar", bg: Some("hangu_bg.png"), loc: Some("关内 · 旧祭坛"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "祭坛上供着三牲与香火，却不见神像——只有一块磨得发亮的石砖，刻着四行歪斜的字。",
            "「若要问墙为何还在——因为有人在墙里。若是墙倒了——那人心早已空了。」",
            "据说这是若干年前第一次兽潮时，一个无名守城人咬破指尖刻下的。",
        ]),
        choices: &[
            ChoiceDef { label: "读尽碑文", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("hg_pt_3"), Eff::Points(20)], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_pt_well", bg: Some("hangu_bg.png"), loc: Some("关内 · 枯井"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一口被石板压死的枯井，井沿钉着七根铁钉，封条上画着看不懂的符。半夜有人听见井下传来低低的歌。",
            "你凑近一闻——不是水汽，是一股极淡的、像是『家』的味道。枯井封的不是水。封的是某个再也等不到的人。",
        ]),
        choices: &[
            ChoiceDef { label: "不惊动 · 记下这口井", sub: "调查完成 · +15点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("hg_pt_4"), Eff::Points(15), Eff::San(-3)], route: Route::To("hg_hub") },
            ChoiceDef { label: "撬开一角窥看", sub: "San-8 · 触怒井底", cond: None,
                effects: &[Eff::San(-8), Eff::Hurt(8, "hg_death")], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_pt_arsenal", bg: Some("hangu_bg.png"), loc: Some("关内 · 箭楼"), mood: "calm",
        speaker: Some("箭楼义军 · 阿秧"), voice: None,
        text: TextSpec::Static(&[
            "箭楼二层堆满了绑着白布条的箭簇，一个名叫阿秧的义军正一枚枚地给箭头裹上火油。",
            "「人手不够啦，只能让箭先『替』城里的兵守住。每枚箭上块白布，是给家里人报的平安。」",
            "他抬起头，眼眶红红的：「等这场仗打完，我就能回家，把这布条烧给小桃看了。」",
        ]),
        choices: &[
            ChoiceDef { label: "帮阿秧缠一枚火油箭", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("hg_pt_5"), Eff::Points(20), Eff::San(-3)], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "hg_np_people", bg: Some("hangu_bg.png"), loc: Some("关内 · 民夫营"), mood: "calm",
        speaker: Some("送粮信使 · 阿宁"), voice: None,
        text: TextSpec::Static(&[
            "一个背着粮袋的瘦弱少年阿宁抬眼望你，咧嘴笑了：「你是外面来的吧？别怕，墙虽然旧了，可还没倒呢。」",
            "他从怀里掏出一块干粮塞给你：「吃吧。守城这行当啊，饿着肚子是砍不动刀、也守不住心的。」",
            "「我爹说，只要墙里还有喘气的『人』，墙就还活着。」他朝墙头老卒的方向努了努嘴。",
        ]),
        choices: &[
            ChoiceDef { label: "收下干粮并道谢", sub: "NPC 对话 · +15点", cond: None,
                effects: &[Eff::Points(15), Eff::San(5)], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_np_chaobo", bg: Some("hangu_bg.png"), loc: Some("主墙楼 · 晁伯"), mood: "sad",
        speaker: Some("守关老将 · 晁伯"), voice: None,
        text: TextSpec::Static(&[
            "守关老将晁伯皱纹里嵌着风沙，他望向关外那个身影，沉默良久：「他叫箜邪。年轻时血气方刚，说要守到人族最后一个字倒下。」",
            "「那年他把整座城护在身后，也把自己护进了黑里。如今他只认『战』字，不认『人』字了。」",
            "晁伯拍拍你的肩：「你若真要过去……记着，他不是怪物，是个没能走出那一晚的守城人。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下箜邪之名", sub: "NPC 对话 · +15点", cond: None,
                effects: &[Eff::Points(15), Eff::SetFlag("hg_know_kongxie")], route: Route::To("hg_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "hg_01b_prep", bg: Some("hangu_bg.png"), loc: Some("决战关隘 · 城门口"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "决战在即。城门口横着残破的拒马，关外的号角一声比一声急。狂化军团长箜邪立于关前，铠甲锈红，战旗猎猎。",
            "你想起晁伯的话——他不是怪物，是个没能走出那一晚的人。可此刻他挥下的刀，是真的想取你性命。",
            "风里传来他沙哑的旧语：「守城的人……早就死在那一年了。来吧，让这道墙，痛快地塌一次。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 狂化军团长箜邪】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "请晁伯再讲一遍旧事", sub: "San-3 · 了然于心", cond: None, effects: &[Eff::San(-3)], route: Route::To("hg_np_chaobo") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景（供地图/其他 route 引用）
    SceneDef {
        id: "hg_01", bg: Some("hangu_bg.png"), loc: Some("决战关隘"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["狂化军团长箜邪 挡在出口。人族的城墙，是最后一道。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("hg_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_round", bg: Some("hangu_bg.png"), loc: Some("决战关隘"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 hg_card） =====
    SceneDef {
        id: "hg_end_choice", bg: Some("hangu_bg.png"), loc: Some("决战之后 · 城墙之上"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当箜邪的刀终于落地，狂化消散，他在你面前缓缓跪倒。锈红的甲缝里，露出那个早已干涸的守城人最后的轮廓。",
            "天边残阳如血，兽潮的号角声渐渐退去。你站在『最后一道』城墙上，风终于不再是哀嚎，而是送行的长歌。",
            "这一战之后，你是想再看一眼这堵墙，还是带着什么离开，又或者——留下来，当下一道墙？",
        ]),
        choices: &[
            ChoiceDef { label: "再看一眼关外", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("hg_sight")], route: Route::To("hg_card") },
            ChoiceDef { label: "收起护符作纪念", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("hg_memento")], route: Route::To("hg_card") },
            ChoiceDef { label: "留下来当城墙", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("hg_stay")], route: Route::To("hg_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "hg_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>人族的城墙，是最后一道——你守住了它，也守住了自己曾是『人』的一夜。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "hg_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("洪荒历 · 函谷关攻防 · 殒命", "殒命于洪荒历 · 函谷关攻防")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn hangu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("hg_boss", FightCfg {
            name: "狂化军团长箜邪", hp: 240, dmg: (18, 30), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "hg_card".to_string(), death: "hg_death",
        }),
    ]
}
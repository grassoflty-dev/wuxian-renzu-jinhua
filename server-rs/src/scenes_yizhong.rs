//! 无限恐怖 · 异种 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、滞留科研员 NPC、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「它不是入侵——是进化错误。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("yz_boss") {
            st.fight = Some(crate::power::scaled_fight("yz_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "yz_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "yz_death".to_string(); }
    "yz_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("yz_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "yz_reward");
    "yz_end_choice".to_string()
}

pub static YIZHONG_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "yz_00", bg: Some("yizhong_bg.png"), loc: Some("外星基因实验室 · 入口闸"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你踏入了「无限恐怖 · 异种」。",
            "冷白的走廊铺满培养槽的蓝光，空气中飘着消毒水与某种腥甜的气息。这不是人类造的实验室，它不属于任何已知文明。",
            "一个角落的扩音器忽然响起，声音沙哑又急促:「别——别碰那些茧。它们不是入侵的怪物，这是一场进化……出了错的进化!」",
        ]),
        choices: &[
            ChoiceDef { label: "回应那句警告", sub: "San+5 · 警觉", cond: None, effects: &[Eff::San(5)], route: Route::To("yz_hub") },
            ChoiceDef { label: "径直深入实验室", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "yz_hub", bg: Some("yizhong_bg.png"), loc: Some("基因实验室 · 主过道"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "主过道两侧是一排排自动培养舱，舱内液体翻腾，隐约可见正在成形的『异种』。它们不是从外面来的——它们在这里被『长』出来。",
            "你意识到：这场席卷而来的『异种』，并非外星入侵，而是这间实验室里某段失控基因工程亲手放出来的产物。",
        ]),
        choices: &[
            ChoiceDef { label: "察看培养舱序列", sub: "基因图谱 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("yz_pt_vat") },
            ChoiceDef { label: "读主控台日志", sub: "实验记录 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("yz_pt_console") },
            ChoiceDef { label: "观异形茧群", sub: "孵化之茧 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("yz_pt_cocoon") },
            ChoiceDef { label: "看脱下的旧皮囊", sub: "蜕皮标本 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("yz_pt_shed") },
            ChoiceDef { label: "查灭菌走廊壁痕", sub: "爪痕 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("yz_pt_scars") },
            ChoiceDef { label: "寻幸存科研员", sub: "居民 · 博士", cond: None, effects: &NO_EFF, route: Route::To("yz_np_doctor") },
            ChoiceDef { label: "访最早进去的实验员", sub: "居民 · 实验员", cond: None, effects: &NO_EFF, route: Route::To("yz_np_tech") },
            ChoiceDef { label: "走向茧室核心", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("yz_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "yz_pt_vat", bg: Some("yizhong_bg.png"), loc: Some("基因实验室 · 培养舱区"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "培养舱顶部投影着一幅基因链的立体图谱。你读不懂它全部，却看清了最关键的一环：这个物种的基因序列，被刻意截断并接入了一段不属于它的『进化片段』。",
            "图谱角落用褪色的标注写着:「失败品，编号Φ-7。它不该被这样接上。」",
            "你终于明白那位博士的意思——异种不穷凶极恶地‘入侵’，它们只是那个突变基因错误长成的模样。",
        ]),
        choices: &[
            ChoiceDef { label: "记下Φ-7的图谱", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("yz_pt_1"), Eff::Points(20), Eff::San(-3)], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_pt_console", bg: Some("yizhong_bg.png"), loc: Some("主控台 · 实验日志"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "主控台屏上跳着一行行加密日志。你逐条翻下去，看懂了大意:这项实验本想把异种改造成『更好的猎手』来服务于战争。",
            "可某次传导故障，那段进化片段自己『跑偏』了——它不再朝着猎人进化，而是朝着『活下去』无限增殖。",
            "日志最后一页只有一行字:「我们以为在造工具，其实在造一个不会停的错。它裂开了。」",
        ]),
        choices: &[
            ChoiceDef { label: "读全实验日志", sub: "调查完成 · +20点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("yz_pt_2"), Eff::Points(20), Eff::San(-5)], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_pt_cocoon", bg: Some("yizhong_bg.png"), loc: Some("实验室 · 异形茧室"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "茧室四壁糊满半透明的异形茧，茧壁透着里面蜷缩的、还在轻轻蠕动的影子。每一个茧，都是那场进化错误的又一份证明。",
            "你凑近一只颤动得最厉害的茧，透过半透明的壁，看见里面蜷缩的……竟然隐约有几分『人』的影子。",
            "你想，也许它确实曾经是一只猎犬，一架实验体，甚至……一个曾经想变强的人。",
        ]),
        choices: &[
            ChoiceDef { label: "隔着茧壁静观", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("yz_pt_3"), Eff::Points(15), Eff::San(-5)], route: Route::To("yz_hub") },
            ChoiceDef { label: "用刀剖开一角", sub: "San-8 · 惊动茧心", cond: None,
                effects: &[Eff::San(-8), Eff::Hurt(6, "yz_death")], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_pt_shed", bg: Some("yizhong_bg.png"), loc: Some("实验室 · 蜕皮标本"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一间玻璃柜里整齐陈列着一整排蜕下的旧皮囊。它们一具比一具成熟，最后一具的皮囊早已不是任何『猎物』该有的形态。",
            "柜内标签写着物种的完整演化阶段。你看清楚：这只异种每蜕一层皮，就更接近人类几分——也远离了它该有的模样几分。",
            "原来它不是越进化越不像『敌人』，它是越靠近我们，越显得陌生。",
        ]),
        choices: &[
            ChoiceDef { label: "细致比对蜕皮", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("yz_pt_4"), Eff::Points(20)], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_pt_scars", bg: Some("yizhong_bg.png"), loc: Some("实验室 · 灭菌走廊"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "灭菌走廊的合金墙面上缀满了深浅不一的爪痕，每一道都带着被高温麻醉的焦痕——那是异种在试图离开培养区时留下的。",
            "你数了数，墙一直抓到天花板，密度大得惊人。它们拼尽力气想跳出这一整片被设计好的命运，却只在那一道道上留给我们它曾经挣扎过的证词。",
            "痕沟最深的一处，隐约还嵌着一小片划破的旧皮。那不是野兽发狂，是囚徒想逃。",
        ]),
        choices: &[
            ChoiceDef { label: "抚摸那处最深爪痕", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("yz_pt_5"), Eff::Points(15), Eff::San(-5)], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "yz_np_doctor", bg: Some("yizhong_bg.png"), loc: Some("实验室 · 隔离舱"), mood: "sad",
        speaker: Some("滞留博士 · 纪"), voice: None,
        text: TextSpec::Static(&[
            "隔离舱里，滞留此地的博士纪隔着玻璃与你相望，语气疲惫却又固执:「别杀它们，也别把它们当怪物。它们只是一场实验的错。」",
            "「可那个错一旦裂开，就不可能再乖乖装回基因链里。要么它们学会像‘人’一样活，要么……」他顿了顿，声音更低:『要么我们只好替这个错赎罪。』",
            "他递给你一枚绿色应急针剂:「带着。若被茧液伤了，它能帮你撑一阵。」",
        ]),
        choices: &[
            ChoiceDef { label: "收下应急针", sub: "NPC 对话 · +15点 · 得针剂", cond: None,
                effects: &[Eff::Points(15), Eff::AddItem("yz_antidote")], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_np_tech", bg: Some("yizhong_bg.png"), loc: Some("实验室 · 副控室"), mood: "calm",
        speaker: Some("幸存实验员 · 荷"), voice: None,
        text: TextSpec::Static(&[
            "副控室里，戴着手环的实验员荷正用颤抖的手敲着没电的键盘发呆。他听见脚步声，猛地抬头:「你……你也是来清剿异种的?」",
            "他惨笑一声:「他们都说异种是入侵。可我跟它们一起长大的，我太清楚了——它们只是想活下去。只是想别再被‘制成更聪明’而已。」",
            "他手一抖，把一段数据塞给你:「这是我偷偷记下的培养序列。别让上头知道……拿去，或许能救它们，也救你们。」",
        ]),
        choices: &[
            ChoiceDef { label: "收下培养序列数据", sub: "NPC 对话 · +15点", cond: None,
                effects: &[Eff::Points(15), Eff::AddItem("yz_gene_data")], route: Route::To("yz_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "yz_01b_prep", bg: Some("yizhong_bg.png"), loc: Some("茧室核心 · 成体显现"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你推开茧室核心的闸门，冷白灯光下，一头几乎完全成形的异种成体正缓缓从最大的茧中挣出身来。它不是破壁而入的入侵者——它就在这里，等到了自己破茧的时刻。",
            "它那双混浊的眼睛望着你，没有恶意，只有从一个物种的茧里被迫长成另一副样子的茫然与饥渴。",
            "它低低地咆哮了一声，那不是宣战，更像是在问:你们，为什么要把我们造成这样?",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 异种成体】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "再看一眼蜕皮标本", sub: "San-3 · 了然", cond: None, effects: &[Eff::San(-3)], route: Route::To("yz_pt_shed") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景
    SceneDef {
        id: "yz_01", bg: Some("yizhong_bg.png"), loc: Some("茧室核心"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["异种成体 挡在出口。它不是入侵——是进化错误。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("yz_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_round", bg: Some("yizhong_bg.png"), loc: Some("茧室核心 · 决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 yz_card） =====
    SceneDef {
        id: "yz_end_choice", bg: Some("yizhong_bg.png"), loc: Some("决战之后 · 实验室闸外"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当异种成体轰然倒下，它最后那声低鸣终于带上了一丝释然——仿佛这一场被基因设计出来的错，终于有人替它挡下了终结。",
            "实验室的应急灯一盏盏亮起，那些尚未破茧的异形，在失去母体后重又陷入沉寂。也许它们会被封存，也可能在某天重新苏醒。",
            "你站在闸门前，手里攥着等待被解读的基因数据。这一场由进化的『错』引发的纷争，你自己决定怎么收尾。",
        ]),
        choices: &[
            ChoiceDef { label: "凝望夜空的逃口", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("yz_sight")], route: Route::To("yz_card") },
            ChoiceDef { label: "带走那管基因数据", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("yz_memento")], route: Route::To("yz_card") },
            ChoiceDef { label: "留下重启茧室", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("yz_stay")], route: Route::To("yz_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yz_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>你亲手终结了一场因进化而出错的战争，也给尚未破茧的生命留了一线可能。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "yz_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("无限恐怖 · 异种 · 殒命", "殒命于无限恐怖 · 异种")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn yizhong_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("yz_boss", FightCfg {
            name: "异种成体", hp: 170, dmg: (14, 24), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "yz_card".to_string(), death: "yz_death",
        }),
    ]
}
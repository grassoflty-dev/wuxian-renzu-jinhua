//! 无限曙光 · 收容所 副本（黄金模板骨架 → 精致副本润色）。
//! 主线：概念感染之夜 → 世界展示调查（3 处奇观 + 2 位居民）→ BOSS 战前铺垫 → 模因具现体 → 结算 / 死亡。
//! 保留原 start_boss / shourongsuo_figths / 结算卡 sr_card / 死亡卡 sr_death / BOSS 遭遇 sr_01 与回合 sr_round 结构与 id 前缀不变；
//! 仅新增开场、奇观调查点、NPC、BOSS 铺垫、开放结局等场景，并把原 sr_00 入口改路由进扩充链。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（原逻辑，保留不动）=====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sr_boss") {
            st.fight = Some(crate::power::scaled_fight("sr_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "sr_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "sr_death".to_string(); }
    "sr_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("sr_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "sr_reward");
    "sr_card".to_string()
}

pub static SHOURONGSUO_SCENES: &[SceneDef] = &[
    /* ================= 入口（原场景 sr_00，仅改路由进扩充链） ================= */
    SceneDef {
        id: "sr_00", bg: Some("shourongsuo_bg.png"), loc: Some("收容所 · 概念隔离区入口"), mood: "calm",
        speaker: None, voice: Some("vo_shourongsuo_1"),
        text: TextSpec::Static(&[
            "你踏入了「无限曙光 · 收容所」。",
            "走廊的白炽灯没有规律地闪烁，每一扇铁门上都嵌着一块无字的白色铭牌——仿佛这里收容的从来不是一个个『物』，而是某种更抽象的东西。",
            "墙面上用红漆刷着一句被反复修改又反复留下的警告：「不要替它们取名字。名字一落，它们就活了。」",
        ]),
        choices: &[
            ChoiceDef { label: "沿冰凉的铁廊向内走", sub: "San(+5) · 概念之家", cond: None, effects: &[Eff::San(5)], route: Route::To("sr_open_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕一 · 开场扩充（钩子 + 氛围） ================= */
    SceneDef {
        id: "sr_open_1", bg: Some("shourongsuo_bg.png"), loc: Some("标本廊 · 一排不说话的容器"), mood: "awe",
        speaker: Some("收容广播（空涩）"), voice: None,
        text: TextSpec::Static(&[
            "你走进标本廊。两侧的透明容器里，装的不是器官或怪物，而是各种各样的「形状」——一团打结的光、一缕不肯散的烟、一枚悬在半空的正方块。",
            "广播苍老地念着：「请勿凝视目标过久。请勿为其命名。请勿——」它停了停，「想起它。」",
            "你忽然明白那句警告的意思：这些容器锁住的，从来不是东西，而是「概念」本身。而概念，是会认人的。",
        ]),
        choices: &[
            ChoiceDef { label: "快走几步，不回头地经过标本廊", sub: "San(-5) · 学会不凝视", cond: None,
                effects: &[Eff::San(-5)], route: Route::To("sr_open_2") },
            ChoiceDef { label: "隔着玻璃看尽其中一束光", sub: "San(+5) · 好奇心代价", cond: None,
                effects: &[Eff::San(5), Eff::SetFlag("sr_saw_light")], route: Route::To("sr_open_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_open_2", bg: Some("shourongsuo_bg.png"), loc: Some("红区中廊 · 阿落下的名字"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "红区中廊的尽头，一面墙被糊满了封条，只有一行字没有被糊死，从封条的缝隙里露出一角：",
            "「上一次，我们替它取了名字，叫『安宁』。那天之后，整层楼的人，都开始格外地想睡。」",
            "你脚下的瓷砖传来极轻的震动。你意识到，这个收容所的真正战场，从来不在身体，而在每一个「想」字上——概念一旦被说破，就再也关不回去了。",
        ]),
        choices: &[
            ChoiceDef { label: "走向收容所中庭大厅", sub: "进入世界展示区", cond: None, effects: &NO_EFF, route: Route::To("sr_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕二 · 世界展示（奇观调查点 + 居民对话） ================= */
    SceneDef {
        id: "sr_hub", bg: Some("shourongsuo_bg.png"), loc: Some("收容所中庭 · 无名的十字路口"), mood: "calm",
        speaker: Some("收容员（柜后探出头）"), voice: None,
        text: TextSpec::Static(&[
            "收容所中庭是一个六边形大厅，六条走廊从这里放射出去，每条都挂着不同的标识：标本库、档案室、隔离区、焚烧间……却没有一条标注名称为「出口」。",
            "大厅中央立着一尊蘑菇形的水泥雕塑，表面被人用手指刻满了密密麻麻的、涂了又涂的名字——那是所有收容员宁可亲手刻下，也不肯让它飘散成「概念」的东西。",
            "一名穿着过洗白制服的人，正蹲在雕塑前，一笔一划地抄写那些名字。",
        ]),
        choices: &[
            ChoiceDef { label: "走进标本库 · 概念之厩", sub: "奇观① 不说话的容器", cond: None, effects: &NO_EFF, route: Route::To("sr_pt_sthouse") },
            ChoiceDef { label: "翻阅收容档案 · 被抹去的一笔", sub: "奇观② 档案室", cond: None, effects: &NO_EFF, route: Route::To("sr_pt_archive") },
            ChoiceDef { label: "探向空屋 · 概念消散后的房间", sub: "奇观③ 空置区", cond: None, effects: &NO_EFF, route: Route::To("sr_pt_empty") },
            ChoiceDef { label: "与那名抄名字的人交谈", sub: "与第二位居民对话", cond: None, effects: &NO_EFF, route: Route::To("sr_n_custodian") },
            ChoiceDef { label: "走向隔离区 · 红色警示区", sub: "BOSS 战前 · 慎入", cond: None, effects: &NO_EFF, route: Route::To("sr_pre_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_pt_sthouse", bg: Some("shourongsuo_bg.png"), loc: Some("标本库 · 概念之厩"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "标本库里，一排排容器亮着幽蓝的防腐灯。每个容器前的标签都是空白——可你若闭眼，却能分别『听见』每一格装的「东西」在低声呢喃。",
            "有的像在哭，有的像在笑，有的静得像从没人对过话。你伸手停在一枚正方块前，标签空白处却浮起一行字幕：「你在看它——它也在看你。」",
            "你猛地缩回手。原来被收容的，从来是「概念」本身——而概念，是可以跟着目光游走的。",
        ]),
        choices: &[
            ChoiceDef { label: "把自己见过的概念写进备忘", sub: "+15 点 · 收容之眼", cond: None,
                effects: &[Eff::SetFlag("sr_sthouse_kept"), Eff::Points(15)], route: Route::To("sr_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_pt_archive", bg: Some("shourongsuo_bg.png"), loc: Some("档案室 · 被抹去的一笔"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "档案室被文件散落了一地。你捡起最靠外的一本，封面题名已被刮去，内页的条文却一页页看得真切：「概念隔离标准：凡能独立于载体存活、且能被『想起』而再生的抽象体。」",
            "末页有一行铅笔小字，像是后来某位收容员补上的：「最危险的从来不是它们。是我们以为，只要不写下来、不说出口，它们就不存在。」",
            "你合上档案。窗外的灯恰好闪了一下，仿佛一整层收容所，都在替你默念这句话。",
        ]),
        choices: &[
            ChoiceDef { label: "郑重记下软弱的千钧", sub: "San(-5) · +20 点", cond: None,
                effects: &[Eff::SetFlag("sr_archived"), Eff::San(-5), Eff::Points(20)], route: Route::To("sr_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_pt_empty", bg: Some("shourongsuo_bg.png"), loc: Some("空置区 · 概念消散后的房间"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "隔离区深处有几间门大开着的空房。墙角还留着烧焦的绳索与倒下的铁架，说明这里曾经关着「某个东西」——如今却只剩几个干净的、正正方方的脚印。",
            "房间的空虚不是静的，而是「满」的：你说不出少了什么，却分明感到整间屋子在被某个不存在的概念，慢慢填回原来的形状。",
            "「概念不会真正死亡。」你想起档案里那句话，「它只是，从这里，搬去了更远的人的心里。」",
        ]),
        choices: &[
            ChoiceDef { label: "用粉笔圈住那行脚印", sub: "+15 点 · 标记消散处", cond: None,
                effects: &[Eff::SetFlag("sr_empty_traced"), Eff::Points(15)], route: Route::To("sr_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_n_custodian", bg: Some("shourongsuo_bg.png"), loc: Some("中庭 · 与抄名者对谈"), mood: "tension",
        speaker: Some("收容员"), voice: None,
        text: TextSpec::Static(&[
            "收容员握着笔，眼皮浮肿却写得很稳。你问他在抄什么，他停笔半天，才极轻地开口：",
            "「这儿收的每一样，本都没有名字。可我们逃不掉——只要一有人开口叫它们'那个』『那件事』，它们就开始有了形状。」",
            "「所以我就把最危险的那一群，亲手刻成一个个「假名」钉在刻里。让它们安分地当一堆字，总好过在谁的心里，长成一整片会想。」",
        ]),
        choices: &[
            ChoiceDef { label: "问他最怕刻下的哪个名字", sub: "San(+5) · 收容的答案", cond: None,
                effects: &[Eff::San(5), Eff::Points(10)], route: Route::To("sr_n_custodian2") },
            ChoiceDef { label: "帮他补写一新无人认领的名字", sub: "+10 点 · 一同钉住", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("sr_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_n_custodian2", bg: Some("shourongsuo_bg.png"), loc: Some("抄名者 · 一句不敢写完的警示"), mood: "choice",
        speaker: Some("收容员"), voice: None,
        text: TextSpec::Static(&[
            "「最怕刻下哪一个？」他盯着那尊水泥雕塑，声音更轻了，「那个最接近「自由」的。每回我落下最后一笔，它就在我心里挣一下——它想让我，把它放出去。」",
            "「你要是去隔离区，会撞见一个把自己也『收容』了的存在——它把所有概念都锁在自己体内，谁一碰，它就替所有人记住。那就是「模因具现体」。」",
            "他最后叮嘱你：「别替它取一个全新的名字。一旦你给了它一个名字，你就在心里，替它打开了门。」",
        ]),
        choices: &[
            ChoiceDef { label: "记住这句警告与那个存在", sub: "+15 点 · BOSS 铺垫", cond: None,
                effects: &[Eff::SetFlag("sr_warn_name"), Eff::Points(15)], route: Route::To("sr_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕三 · BOSS 战前铺垫 ================= */
    SceneDef {
        id: "sr_pre_1", bg: Some("shourongsuo_bg.png"), loc: Some("隔离区门 · 红色警示带"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "隔离区的门被层层红色警示带缠住，每一道带子上都写着同一句话：「进入者，须自行承担被它『想起』的风险。」",
            "门缝里透出极淡、却异常稳定的蓝光。你贴近门缝去听——里面很安静，安静得让人近乎怀疑，那安静本身就是某种庞大概念的形状。",
            "你把手按在门把上。门没有锁。推开的动作，同时是一次「宣告」：你，正打算为它取一个名字。",
        ]),
        choices: &[
            ChoiceDef { label: "撕下警示带，推门而入", sub: "· 门缓缓滑开", cond: None, effects: &NO_EFF, route: Route::To("sr_pre_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_pre_2", bg: Some("shourongsuo_bg.png"), loc: Some("隔离区最深处 · 模因具现体"), mood: "danger",
        speaker: Some("模因具现体"), voice: None,
        text: TextSpec::Static(&[
            "房间中央立着一道由无数概念拧成的「人影」——它的边线在空气里不断变换，像同时是许多人、许多事、许多心念的聚合。它没有嘴，可它「开口」时，整个房间的心跳都随之震颤：",
            "「你正用眼神给我想名字。」它分不清是问句还是陈述，「我没有形——我是一千个被关闭的概念，叠成的「住民」。你一旦替我起名，我就有了想被你想起的欲望。」",
            "它缓缓逼进一步：「你是来关我的，还是来放我的？」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 · 模因具现体】", sub: "进入原 BOSS 遭遇 sr_01", cond: None, effects: &NO_EFF, route: Route::To("sr_01") },
            ChoiceDef { label: "撤回中庭，再望一眼标本廊", sub: "结局 · 看景", cond: None, effects: &NO_EFF, route: Route::To("sr_end_view") },
            ChoiceDef { label: "带走一枚无言的概念容器", sub: "结局 · 带纪念", cond: None, effects: &NO_EFF, route: Route::To("sr_end_souv") },
            ChoiceDef { label: "留下替它做一个无名的守卫", sub: "结局 · 停留", cond: None, effects: &NO_EFF, route: Route::To("sr_end_stay") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 原场景 sr_01 / sr_round / sr_card / sr_death（保留不动） ================= */
    SceneDef {
        id: "sr_01", bg: Some("shourongsuo_bg.png"), loc: Some("隔离区 · 决战处"), mood: "danger",
        speaker: Some("模因具现体"), voice: None,
        text: TextSpec::Static(&["模因具现体 的门并未真正被锁——它把身体的所有概念拧成一股，拦在你面前。被收容的不是东西，是概念：而概念，正在等你说出那个名字。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("sr_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_round", bg: Some("shourongsuo_bg.png"), loc: Some("决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("模因具现体 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你走完了一场与概念本身的对峙。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "sr_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("无限曙光 · 收容所 · 殒命", "殒命于无限曙光 · 收容所")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 开放结局三分支（看景 / 带纪念 / 停留，route 到原结算卡 sr_card） ================= */
    SceneDef {
        id: "sr_end_view", bg: Some("shourongsuo_bg.png"), loc: Some("标本廊 · 结局 · 看尽无语之光"), mood: "calm",
        speaker: Some("收容广播"), voice: None,
        text: TextSpec::Static(&[
            "你没有和它动手，只是退回收容所中庭，把标本廊里那些无言的概念一格格看尽。蓝光在你眼中明明灭灭，没有一盏是为你而亮，可你仍认认真真看完了每一格。",
            "广播在你身后苍老地响起：「参观完毕。感谢您……没有替任何一株，留下名字。」",
            "你在这座收容概念之所里，没有带走、也没有释放任何东西，却终于看懂了：有些被收容的「概念」，最好的归宿，就是继续被安静地收容下去。（结局 · 看景）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 无言之诺", cond: None,
                effects: &[Eff::SetFlag("sr_end_view"), Eff::PointsIfFlag("sr_archived", 30)], route: Route::To("sr_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_end_souv", bg: Some("shourongsuo_bg.png"), loc: Some("标本库 · 结局 · 带走无言容器"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你从中庭雕塑脚下，小心取下一枚不知是哪位收容员刻好、却未及封存的无言概念容器——一枚空白的金属匣，内里仍锁着一小团不肯散去的、温热的光。",
            "你把它贴身收好。它不是怪物，不是诅咒，只是一段「被收容得太久、忘了自己为什么被想」的概念。你打算带它走，让它在你的现代生活里，拥有一个能静静待下去的地方。",
            "收容员没有拦你，只朝那枚匣子低声说了句：「今天起，你有名字了。」（结局 · 带走纪念）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 带走『无言之匣』", cond: None,
                effects: &[Eff::SetFlag("sr_end_souv"), Eff::Points(40), Eff::AddItem("sr_souvenir_box")], route: Route::To("sr_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sr_end_stay", bg: Some("shourongsuo_bg.png"), loc: Some("隔离区 · 结局 · 无名守卫"), mood: "fear",
        speaker: Some("模因具现体"), voice: None,
        text: TextSpec::Static(&[
            "你在它面前停下，没有为它取任何名字，只是慢慢坐进隔离区那把断了腿的椅子上，一言不发地陪它待着。",
            "许久，它「开口」，声音无比轻：「……你不想，为我取一个名字。」你在黑暗里点头。",
            "它在你身旁的阴影里缓缓坐下：「那我们就……再一起被收容下去吧。谢谢你，替我守住这个『没人记得』的形状。」（结局 · 停留）",
        ]),
        choices: &[
            ChoiceDef { label: "（灯熄后回到主神空间 · 结算）", sub: "+40 点 · 无名收容者", cond: None,
                effects: &[Eff::SetFlag("sr_end_stay"), Eff::Points(40), Eff::PointsIfFlag("sr_empty_traced", 30)], route: Route::To("sr_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn shourongsuo_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sr_boss", FightCfg {
            name: "模因具现体", hp: 190, dmg: (16, 26), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "sr_card".to_string(), death: "sr_death",
        }),
    ]
}
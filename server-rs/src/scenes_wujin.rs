//! 洪荒历 · 无尽森林 —— 剧情扩充（保留原选择驱动 BOSS / 结算卡 / 死亡卡）
//! 新增：开场气氛、世界展示调查点、部族居民 NPC、BOSS 战前铺垫、开放结局 2-3 分支。
//! 主线 hook：「森林会吃人——也吃文明。」
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（保留原逻辑不变） =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("wj_boss") {
            st.fight = Some(crate::power::scaled_fight("wj_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "wj_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "wj_death".to_string(); }
    "wj_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("wj_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "wj_reward");
    "wj_end_choice".to_string()
}

pub static WUJIN_SCENES: &[SceneDef] = &[
    // ===== 开场扩充（钩子 + 氛围） =====
    SceneDef {
        id: "wj_00", bg: Some("wujin_bg_open.png"), loc: Some("无尽森林 · 林缘"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你踏入了「洪荒历 · 无尽森林」。",
            "参天的古木遮蔽天光，藤蔓纠缠成墙，兽吼与虫鸣交织成一整座会呼吸的丛林。这片森林一眼望不到尽头。",
            "一个脸上涂着白灰的部族猎人蹲在树根上，望着你，慢悠悠地说:「森林会吃人。会吃文明，也会吃那些想驯服它的人。」",
        ]),
        choices: &[
            ChoiceDef { label: "问清森林生态", sub: "San+5 · 听其所以", cond: None, effects: &[Eff::San(5)], route: Route::To("wj_hub") },
            ChoiceDef { label: "径自深入密林", sub: "+5点", cond: None, effects: &[Eff::Points(5)], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示中枢（hub） =====
    SceneDef {
        id: "wj_hub", bg: Some("wujin_bg_invest.png"), loc: Some("无尽森林 · 部落实地"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "密林深处有一处部落实地，木桩上挂着兽骨与旗帜，营地中央立着酋长的大帐。这里有几个部族共处，却各怀心思。",
            "有人低声告诉你，正在逼近的『兽人战潮』并非外敌——它更像是森林对盘踞在这里的文明，一次迟来的反噬。",
        ]),
        choices: &[
            ChoiceDef { label: "看圣树下的祭火", sub: "部族图腾 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("wj_pt_altar") },
            ChoiceDef { label: "察藤蔓编织的图", sub: "丛林生态 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("wj_pt_totem") },
            ChoiceDef { label: "看兽骨堆的旧痕", sub: "战潮旧迹 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("wj_pt_victims") },
            ChoiceDef { label: "察看部族柱碑", sub: "祖声 · 调查点", cond: None, effects: &NO_EFF, route: Route::To("wj_pt_stele") },
            ChoiceDef { label: "听部族猎人训话", sub: "居民 · 猎人", cond: None, effects: &NO_EFF, route: Route::To("wj_np_hunter") },
            ChoiceDef { label: "访部族的割藤老者", sub: "居民 · 割藤人", cond: None, effects: &NO_EFF, route: Route::To("wj_np_elder") },
            ChoiceDef { label: "走向战潮迎击口", sub: "BOSS 前奏", cond: None, effects: &NO_EFF, route: Route::To("wj_01b_prep") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界展示调查点 4 个 =====
    SceneDef {
        id: "wj_pt_altar", bg: Some("wujin_bg_invest.png"), loc: Some("部落实地 · 圣树祭火"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "营地中央的圣树要几人合抱，树根盘成的祭台上燃着长年不熄的火。树身上刻满了各部落图腾，越往高处越凶戾。",
            "部族老祭师守在火旁，低声道:「我们向森林献火、献骨、献最早的安宁。可森林记得，这些安宁是用什么换的。」",
            "火苗一颤，仿佛在替森林回答。",
        ]),
        choices: &[
            ChoiceDef { label: "临祭火静观", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("wj_pt_1"), Eff::Points(20), Eff::San(-3)], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_pt_totem", bg: Some("wujin_bg_open.png"), loc: Some("密林 · 藤蔓图腾"), mood: "mystery",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一株巨藤被部族编成了整片图腾墙，藤蔓上依序缠着兽骨、彩羽与黑石。越靠中间的物什越古老，也越像……某种人的遗物。",
            "你读懂了一种图腾的排列：最下面抓的是猎物，中间挂的是被森林吞掉的东西，而最上端那部分，缠绕的形状像极了一张挣扎的脸。",
            "猎人的话浮上心头:森林会吃人。这图腾墙，就是它亲口列下的『菜单』。",
        ]),
        choices: &[
            ChoiceDef { label: "读清图腾的排列", sub: "调查完成 · +20点 · San-3", cond: None,
                effects: &[Eff::MarkPoint("wj_pt_2"), Eff::Points(20), Eff::San(-3)], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_pt_victims", bg: Some("wujin_bg_open.png"), loc: Some("密林 · 兽骨堆"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "一处低洼地的兽骨堆已经腐烂发黑，可骨堆最外面压着的，竟是几截折断的船桨与一部部族徽记的断矛。",
            "那不是兽的骨头。那是上一次『文明』试图在森林里定居时，留下的痕迹。森林把它连人带营一起收走了。",
            "你忽然明白，兽人战潮不是森林里唯一『吃人』的东西。森林本身，就是最大的那个。",
        ]),
        choices: &[
            ChoiceDef { label: "记下骨堆旧迹", sub: "调查完成 · +15点 · San-5", cond: None,
                effects: &[Eff::MarkPoint("wj_pt_3"), Eff::Points(15), Eff::San(-5)], route: Route::To("wj_hub") },
            ChoiceDef { label: "翻动骨堆深处", sub: "San-8 · 惊醒旧影", cond: None,
                effects: &[Eff::San(-8), Eff::Hurt(6, "wj_death")], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_pt_pool", bg: Some("wujin_bg_open.png"), loc: Some("密林 · 心湖倒影"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "林间一泓幽碧的湖，水面安安静静，可倒影里却游着一些你在水面看不到的『生物』。",
            "部族传说里，这片心湖能照见森林真正想要什么。你若凝望，会看见千万年之前——这里曾是没有森林的草原，和一群安居的部落。",
            "是部落为了自保，才把记忆埋进这一泓幽绿里，企图忘记自己才是曾经扰乱这片天地的东西。",
        ]),
        choices: &[
            ChoiceDef { label: "静望心湖倒影", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("wj_pt_4"), Eff::Points(20)], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_pt_stele", bg: Some("wujin_bg_invest.png"), loc: Some("部落实地 · 祖声柱碑"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "营地西侧立着一根三人高的柱碑，碑顶供着一串会随着风声呜呜作响的骨哨。部族管它叫『祖声』。",
            "祭师说，每当有难，他们便吹响祖声，让森林听见他们还记着先人是怎么在这片地方活下来的。",
            "可你听明白了：那哨声与其说是在唤祖先，不如说是在哀求森林——别把我们从这片它曾经允许安居的地方赶走。",
        ]),
        choices: &[
            ChoiceDef { label: "抚碑听祖声", sub: "调查完成 · +20点", cond: None,
                effects: &[Eff::MarkPoint("wj_pt_5"), Eff::Points(20)], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 世界居民 NPC 2 个 =====
    SceneDef {
        id: "wj_np_hunter", bg: Some("wujin_bg_invest.png"), loc: Some("部落实地 · 猎人围火"), mood: "calm",
        speaker: Some("部族猎人 · 褐爪"), voice: None,
        text: TextSpec::Static(&[
            "脸上涂着白灰的猎人褐爪守着火，一边削箭头一边道:「我们从不猎杀，只打森林分给我们的那一份。越贪，死得越快。」",
            "他望着逼近的战潮方向，沉默了:「可酋长不服。他说要让部族踩进森林，当这里唯一的王。他根本不懂——森林从不让人称王。」",
            "他把自己削好的三枚木箭递给你:「带着。进了林子，命比脸面值钱。」",
        ]),
        choices: &[
            ChoiceDef { label: "收下三枚木箭", sub: "NPC 对话 · +15点 · 得木箭", cond: None,
                effects: &[Eff::Points(15), Eff::AddItem("wj_arrows")], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_np_elder", bg: Some("wujin_bg_invest.png"), loc: Some("部落实地 · 割藤老者棚"), mood: "sad",
        speaker: Some("割藤老者 · 苍"), voice: None,
        text: TextSpec::Static(&[
            "割藤老者苍佝偻着背，一遍遍用麻绳勒紧藤蔓，嘴里念念有词:「老啦，割不动了。可藤还是要割，不然它会把整片营地都缠进土里。」",
            "他抬头看你，浑浊的眼里有光:「你们这些外来人总以为森林是能被征服的。可你割得断一根藤，割不断千万根。藤是会记得的。」",
            "「等战潮把它以前欠的账都讨回来，这林子，就再没有空地能给文明扎根了。」他叹口气，继续割。",
        ]),
        choices: &[
            ChoiceDef { label: "帮苍割一捆藤", sub: "NPC 对话 · +15点 · San-3", cond: None,
                effects: &[Eff::Points(15), Eff::San(-3)], route: Route::To("wj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== BOSS 战前铺垫（route 到迎战，保留原 BOSS 逻辑） =====
    SceneDef {
        id: "wj_01b_prep", bg: Some("wujin_bg_battle.png"), loc: Some("迎击口 · 战潮压顶"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "营地外，兽人战潮正像一道黑色的潮水漫过林线。为首的是『兽人战潮王』，他比周围所有兽人都高出半截，战斧拖地。",
            "部族的号角吹响了，可你听得出来，那号角里没有当年抵御外敌的底气，只有对『能不能继续住下去』的惶然。",
            "战潮王停在营前，声音如闷雷:「森林累了。它要收回它借给你们的这一片安宁。识相的，交出兵刃与文明。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 兽人战潮王】", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "再看一眼心湖倒影", sub: "San-3 · 想清来由", cond: None, effects: &[Eff::San(-3)], route: Route::To("wj_pt_pool") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // 保留原 BOSS 入口场景
    SceneDef {
        id: "wj_01", bg: Some("wujin_bg_battle.png"), loc: Some("迎击口"), mood: "danger",
        speaker: Some("BOSS"), voice: None,
        text: TextSpec::Static(&["兽人战潮王 挡在出口。森林会吃人——也吃文明。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("wj_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_round", bg: Some("wujin_bg_battle.png"), loc: Some("迎击口 · 决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("BOSS 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ===== 开放结局 2-3 分支（route 到原结算卡 wj_card） =====
    SceneDef {
        id: "wj_end_choice", bg: Some("wujin_bg_battle.png"), loc: Some("决战之后 · 林间空场"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "当战潮王的战斧脱手砸进泥里，潮水般的兽群竟缓缓散去——它们来讨的账，似乎在足够多的『人』倒下后也被抵消了几分。",
            "你喘息着站定，晨光穿过密林投下斑驳的影。森林还在，营地还在，只是那个曾想踩进林子的酋长幻想，已经被砍落在地。",
            "你望着这片会吃人、也会记得人的无尽森林，由你决定，要怎么和它作结。",
        ]),
        choices: &[
            ChoiceDef { label: "凝望透进林的光", sub: "看景 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("wj_sight")], route: Route::To("wj_card") },
            ChoiceDef { label: "带回一枚兽牙结", sub: "带纪念 · +100点", cond: None,
                effects: &[Eff::Points(100), Eff::SetFlag("wj_memento")], route: Route::To("wj_card") },
            ChoiceDef { label: "留在森林里定居", sub: "停留 · +120点", cond: None,
                effects: &[Eff::Points(120), Eff::SetFlag("wj_stay")], route: Route::To("wj_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "wj_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这个副本。</p><p style='color:#9a958a'>这一夜，森林没有收走最后一个想驯服它的人——而那个人，是你。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "wj_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("洪荒历 · 无尽森林 · 殒命", "殒命于洪荒历 · 无尽森林")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn wujin_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("wj_boss", FightCfg {
            name: "兽人战潮王", hp: 210, dmg: (16, 26), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "wj_card".to_string(), death: "wj_death",
        }),
    ]
}
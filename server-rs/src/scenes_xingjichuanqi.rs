//! 星际传奇 · CD星球 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「这个星球的美，只在白天。太阳落山后，它属于另一种东西。」
//! 结构：开场(短) → xj_hub(调查中转) → 调查点/双日荒原奇观 + NPC → 迎战铺垫 → xx_01 选择驱动 BOSS → 开放结局三选 → xj_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("xj_boss") {
            st.fight = Some(crate::power::scaled_fight("xj_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "xj_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "xj_death".to_string(); }
    "xj_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("xj_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "xj_reward");
    "xj_card".to_string()
}

pub static XINGJICHUANQI_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "xj_00", bg: Some("xingjichuanqi_bg.png"), loc: Some("CD星球 · 双日荒原 · 登陆点"),
        mood: "calm", speaker: Some("领航 AI"), voice: Some("vo_xj_open"),
        text: TextSpec::Static(&[
            "你说不准这颗星球是不是「活着」。脚下是干燥的赭红色荒原，头顶悬着两颗恒星——一大一小，一暖一冷，正把整个世界拖进漫长而炫目的白昼。",
            "登陆记录里只有一句话警告：<em>「如果要欣赏这颗星球，请赶在白天。夜里，荒原会换一个主人。」</em>",
            "远处地平线上，一座废弃的观测站像一枚插在沙里的旧钉子。双日光在你脚下投出两道交错的影，像是这片土地给陌生人的第一份赠礼。",
        ]),
        choices: &[
            ChoiceDef { label: "抬头看那两颗太阳", sub: "+5 点 · 记住这份白昼之美", cond: None, effects: &[Eff::Points(5)], route: Route::To("xj_sun") },
            ChoiceDef { label: "环顾荒原", sub: "看清脚下的世界", cond: None, effects: &NO_EFF, route: Route::To("xj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "xj_hub", bg: Some("xingjichuanqi_bg.png"), loc: Some("CD星球 · 荒原哨站"),
        mood: "calm", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "废弃观测站的阴影恰好遮住你，让双日显得不那么灼人。荒原在正午的光里泛着恰到好处的棕红——这是这颗星球一天里头最慷慨的一刻。",
            "黄昏还远，你还有时间把这方天地看仔细。远处，有一条被沙尘半埋的石径通向更高的荒原，和一片泛着微光的裂谷。",
        ]),
        choices: &[
            ChoiceDef { label: "双日叠景 · 荒原奇观", sub: "看那两颗太阳缓缓相交", cond: None, effects: &NO_EFF, route: Route::To("xj_sun") },
            ChoiceDef { label: "化石峡谷", sub: "星球古老过去的残骸", cond: None, effects: &NO_EFF, route: Route::To("xj_fossil") },
            ChoiceDef { label: "白昼磷光地", sub: "只在白天发光的植物", cond: None, effects: &NO_EFF, route: Route::To("xj_glow") },
            ChoiceDef { label: "守日人的哨站", sub: "与荒原居民交谈", cond: None, effects: &NO_EFF, route: Route::To("xj_resident") },
            ChoiceDef { label: "凝视阴面的远方", sub: "黄昏将至的预警", cond: None, effects: &NO_EFF, route: Route::To("xj_prelude") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：双日叠景 =================
    SceneDef {
        id: "xj_sun", bg: Some("xingjichuanqi_bg.png"), loc: Some("荒原 · 双日叠景观测点"),
        mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "你站上最高的石脊。半空中，那颗温暖的大日正在缓缓向冰冷的小日靠拢——CD 星球每 47 昼夜才有一次「双日相叠」。",
            "两道光在天空中交织成一条淡金色的绸带，整片荒原的轮廓在这一刻都被镀上温柔的光。你忽然明白那句警告为何说「美，只在白天」——这一刻如此短暂，却清澈得不真实。",
        ]),
        choices: &[ChoiceDef { label: "把这一刻记进脑海", sub: "MarkPoint · +15 点 · 白昼之美", cond: None,
            effects: &[Eff::MarkPoint("xj_pt_sun"), Eff::Points(15), Eff::San(5)], route: Route::To("xj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：化石峡谷 =================
    SceneDef {
        id: "xj_fossil", bg: Some("xingjichuanqi_bg.png"), loc: Some("荒原 · 化石峡谷"),
        mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "峡谷里裸露的岩层像一册被撕掉封面的古书，层层叠叠的化石嵌在赭石与暗灰之间——某种巨大的、有脊骨的生物，断裂的背骨横贯半面崖壁。",
            "这颗星球曾经有别的生命，比「夜行者」更古老、也更庞大的生命。它们死在双日的灼晒下，只把骨架留给了荒原，供后来者匆匆一瞥。",
        ]),
        choices: &[ChoiceDef { label: "拓下一段背骨纹路", sub: "MarkPoint · AddItem 骨化石 · +15 点", cond: None,
            effects: &[Eff::MarkPoint("xj_pt_fossil"), Eff::AddItem("xj_fossil_souvenir"), Eff::Points(15)], route: Route::To("xj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：白昼磷光地 =================
    SceneDef {
        id: "xj_glow", bg: Some("xingjichuanqi_bg.png"), loc: Some("荒原 · 白昼磷光地"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一片低矮的银白色植被在日光下流转着波光——它们叫「昼菀草」，只在白昼吸饱双日光，再于夜里蛰伏成灰黑色的枯枝。",
            "你蹲下来碰了碰一株，细细的绒毛在指尖微微发烫。这是整个荒原上唯一「活着」的证据：它们记得光，也记得该躲开的黑夜。",
        ]),
        choices: &[ChoiceDef { label: "摘一片发光的叶", sub: "MarkPoint · AddItem 昼菀叶 · San+5", cond: None,
            effects: &[Eff::MarkPoint("xj_pt_glow"), Eff::AddItem("xj_glow_leaf"), Eff::San(5)], route: Route::To("xj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：守日人 =================
    SceneDef {
        id: "xj_resident", bg: Some("xingjichuanqi_bg.png"), loc: Some("弃站 · 守日人的帐篷"),
        mood: "calm", speaker: Some("守日人 · 老迈"), voice: None,
        text: TextSpec::Static(&[
            "一个裹着褪色披巾的老人盘坐在帐篷口，正眯着眼看天。他连头都没回：「别数日子，孩子，数光。白天我们攒光，夜里它们来取——两清了。」",
            "他朝地平线那边努努嘴：「那边裂谷下面有东西，比夜里的还古老。想活到下一个白天，就只当它是荒原的骨头，别去挖。」",
            "「可这星球的白天啊……」他忽然笑了，皱纹里全是光，「值。」",
        ]),
        choices: &[ChoiceDef { label: "听老人说完", sub: "+10 点 · 荒原的老人", cond: None,
            effects: &[Eff::Points(10), Eff::San(5)], route: Route::To("xj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：夜行者研究旅人 =================
    SceneDef {
        id: "xj_traveler", bg: Some("xingjichuanqi_bg.png"), loc: Some("荒原石径 · 观星者"),
        mood: "mystery", speaker: Some("旅人 · 夜行者观察者"), voice: None,
        text: TextSpec::Static(&[
            "一个背着观测仪器的旅人蹲在石径边，手里转着一支没点燃的灯。她低声说：「别误会，我不是来抓它们的——我只是记录。夜里从地缝里爬出来的那一群，其实……只是饿。」",
            "「头上有两颗太阳的生命圈，地缝里却能容下一种只认夜晚的东西。我没法说它是什么，只能说——它从不在意这颗星球的白日有多美。」",
            "她抬头看了眼渐斜的光：「天快黑了。你要是还要赶路，就挑个白天多的地方走。」",
        ]),
        choices: &[ChoiceDef { label: "记下她对黑夜的一番话", sub: "+10 点 · 夜的另一面", cond: None,
            effects: &[Eff::Points(10)], route: Route::To("xj_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫 =================
    SceneDef {
        id: "xj_prelude", bg: Some("xingjichuanqi_bg.png"), loc: Some("荒原 · 望向裂谷"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你站上最后一道石脊，阴面被漫长阴影吞没。双日渐沉，冷光消隐，荒原褪去白昼的棕红，一层暗青从地缝漫上来。",
            "底下那道裂谷像一张张开的巨口，吐着灰用。顺着裂谷看去——有东西正沿着它在日光最后的余韵里，翻上荒原。",
            "你这才明白：所谓「出口」，从来不在你身后的空地上，而在那道裂谷的尽头。<em>在那里，白昼完全落幕。</em>",
        ]),
        choices: &[
            ChoiceDef { label: "顺着裂谷追过去", sub: "直面将要挡住去路的｢它｣", cond: None, effects: &NO_EFF, route: Route::To("xj_prelude2") },
            ChoiceDef { label: "先退回到哨站", sub: "再备一次，权当最后一眼", cond: None, effects: &NO_EFF, route: Route::To("xj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xj_prelude2", bg: Some("xingjichuanqi_bg.png"), loc: Some("裂谷口 · 夜幕将合"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "裂谷深处，一双双灰白的眼在黑暗中睁开——那是夜行嗜血生物群，正被最后一缕日光惊醒。白昼留给这颗星球的美，在此刻彻底关上了门。",
            "它们汇成一道缓慢的潮，挡住裂谷尽头那条可能是出口的路。<em>「这个星球的美，只在白天。」——而黑夜，已至。</em>",
        ]),
        choices: &[ChoiceDef { label: "迎战", sub: "与夜行嗜血生物群对峙", cond: None, effects: &NO_EFF, route: Route::To("xj_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "xj_01", bg: Some("xingjichuanqi_bg.png"), loc: Some("裂谷尽头"), mood: "danger",
        speaker: Some("夜行嗜血生物群"), voice: None,
        text: TextSpec::Static(&["嗜血生物群 挡在出口。夜里，它们是这颗星球的领主。这里的美，只在白天——而此刻，是夜。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "后退一步，屏息观察", sub: "+5 · 记住它们的轮廓", cond: None, effects: &[Eff::Points(5)], route: Route::To("xj_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xj_round", bg: Some("xingjichuanqi_bg.png"), loc: Some("决战处 · 裂谷尽头"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("夜行嗜血生物群 尚余 {} 气力，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 xj_card） =================
    SceneDef {
        id: "xj_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["夜潮在晨光里退去。你在离开前，最后站上那道石脊，最后一次看这颗星球苏醒。"]),
        choices: &[ChoiceDef { label: "记住这最后一眼的白昼", sub: "+30 点 · 白昼之美", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("xj_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xj_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走一片昼菀叶。它离开日光后，在掌心慢慢蜷成枯灰色——像这颗星球固执地提醒你：它的美，留在了白天。"]),
        choices: &[ChoiceDef { label: "把这片枯叶收进口袋", sub: "+25 点 · AddItem 昼菀枯叶 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("xj_glow_leaf")], route: Route::To("xj_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xj_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你没急着走，而是在哨站的阴影里多留了一会儿。灰烬般的荒原上，双日正在重新爬到天顶——你知道，下一个白昼还会这么美。"]),
        choices: &[ChoiceDef { label: "再多看一会儿，然后起身", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("xj_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "xj_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "星际传奇 · 结算".into(), good: true,
                body_html: format!("<p>你穿过了 CD 星球的白昼与黑夜。</p><p>夜潮退去后，荒原重归双日的温柔。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "xj_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("星际传奇 · CD星球 · 殒命于夜色", "在裂谷尽头被夜行嗜血生物群淹没")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你没能活过这个夜晚。荒原在这颗星球的黑暗中，填上了一个新的、安静的身影。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命于夜色】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn xingjichuanqi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("xj_boss", FightCfg {
            name: "夜行嗜血生物群", hp: 160, dmg: (14, 24), reward: 500, reward_why: "击败 BOSS", intro: "夜潮翻上荒原——它们来了！",
            rage_at: Some(60), rage_text: "嗜血生物群在黑暗中狂躁起来！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "xj_card".to_string(), death: "xj_death",
        }),
    ]
}
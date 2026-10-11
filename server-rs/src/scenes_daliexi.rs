//! 死亡开端 · 大裂隙 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「裂口下面是另一个死亡。别靠近边缘，也别以为深渊只会往上爬。」
//! 结构：开场(大地裂口) → dl_hub(调查中转) → 调查点/裂隙深处 + NPC → 深渊聚合 → 裂隙行尸聚合体迎战 → 开放结局三选 → dl_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("dl_boss") {
            st.fight = Some(crate::power::scaled_fight("dl_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "dl_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "dl_death".to_string(); }
    "dl_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("dl_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "dl_reward");
    "dl_card".to_string()
}

pub static DALIEXI_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "dl_00", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 裂口边缘"),
        mood: "cold", speaker: Some("旁白"), voice: Some("vo_dl_open"),
        text: TextSpec::Static(&[
            "你的脚下是一条横贯大地的巨型裂缝，宽到对岸只剩一条模糊的白线。裂口深处吐着灰雾，雾顺着石壁爬上来，把边缘磨得湿滑。",
            "大地像被巨物掀开的一道口子，露出层层叠叠的、燃烧过又冷却的岩层。你听见深渊里传来很闷的回声——像是某种庞然之物，正沿着裂缝缓慢翻身。",
            "石壁上有人用白灰写着潦草的字：<em>「裂口下面是另一个死亡。活着的人，别往下看太久。」</em>",
        ]),
        choices: &[
            ChoiceDef { label: "多看了一眼深渊", sub: "San-5 · 记住那股灰雾", cond: None, effects: &[Eff::San(-5)], route: Route::To("dl_hub") },
            ChoiceDef { label: "沿着裂口边缘走", sub: "看清这条缝的走向", cond: None, effects: &NO_EFF, route: Route::To("dl_edge") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "dl_hub", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 边缘营地"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你在一块凸出的岩台上落脚，灰雾在四周涌动，像整条裂缝在轻微地呼吸。远处，有半座塌陷的屋子正悬在裂口边，还有一道老旧的吊桥横跨着两片破碎的崖岸。",
            "要离开这片被撕裂的大地，你可能得想办法渡过这条缝——或者，弄清楚它究竟要往你这边吐什么。",
        ]),
        choices: &[
            ChoiceDef { label: "断层岩台", sub: "裂口边缘的地质奇观", cond: None, effects: &NO_EFF, route: Route::To("dl_edge") },
            ChoiceDef { label: "跨裂的旧吊桥", sub: "连接两座破碎崖岸的桥", cond: None, effects: &NO_EFF, route: Route::To("dl_bridge") },
            ChoiceDef { label: "沉下去的屋子", sub: "半座被吞进裂缝的房", cond: None, effects: &NO_EFF, route: Route::To("dl_subsidence") },
            ChoiceDef { label: "升腾的灰雾柱", sub: "深渊还在往上冒", cond: None, effects: &NO_EFF, route: Route::To("dl_ascent") },
            ChoiceDef { label: "营地里有人声", sub: "与守在边缘的人交谈", cond: None, effects: &NO_EFF, route: Route::To("dl_prospector") },
            ChoiceDef { label: "把目光交给灰雾最浓处", sub: "望向裂口会爬出什么", cond: None, effects: &NO_EFF, route: Route::To("dl_gather") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：断层岩台 =================
    SceneDef {
        id: "dl_edge", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 断层岩台"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "断层像被一把巨刃切开，切面平滑得近乎人为。你趴到岩台边缘，灰雾在下方来回滚，偶尔露出一截更深的、泛着暗红的岩缝。",
            "你数了数，这条裂缝至少有三层「台阶」——每往下一层，岩壁的颜色就深一分。没人知道最深处有什么，但每一个台阶底下，都摆着一圈熄灭的火把，像是前人沿着深渊一路照过的路标。",
        ]),
        choices: &[ChoiceDef { label: "记下断层那些熄灭的火把", sub: "MarkPoint · +15 点 · 深渊的台阶", cond: None,
            effects: &[Eff::MarkPoint("dl_pt_edge"), Eff::Points(15), Eff::San(-5)], route: Route::To("dl_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：旧吊桥 =================
    SceneDef {
        id: "dl_bridge", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 跨裂旧桥"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一道摇摇晃晃的吊桥横跨裂缝，木板沤得发黑，铁索上结着厚锈。你试着踩上一块板，桥身立刻发出令人牙酸的吱呀声。",
            "桥头挂着一面被风吹得歪斜的牌子，字迹被水汽泡糊了，只剩下一个向前的箭头和一行小字：「……桥的那头，还算是这条路的一部分。」桥下，灰雾正缓缓上涌。",
        ]),
        choices: &[ChoiceDef { label: "试探着走两步再退回来", sub: "MarkPoint · +10 点 · 桥的另一头", cond: None,
            effects: &[Eff::MarkPoint("dl_pt_bridge"), Eff::Points(10)], route: Route::To("dl_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：沉下去的屋子 =================
    SceneDef {
        id: "dl_subsidence", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 塌陷的屋"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "半座木屋歪斜地悬在塌陷区边缘，屋顶被裂缝撕开一道豁口。屋外的小院里，晾衣绳还挂着几件洗过的衣裳，在灰雾里轻轻摆动。",
            "你猜它原本是某个守在裂隙边讨生活的人的家——裂缝夺走了一半的宅院，却勉强留住了晾衣绳。你在门槛缝里夹着一张被叠得很小、却很重要的纸条，上面只写着一个字：「走」。",
        ]),
        choices: &[ChoiceDef { label: "把那张纸条收好", sub: "MarkPoint · AddItem 撕裂的纸条 · +15 点", cond: None,
            effects: &[Eff::MarkPoint("dl_pt_subsidence"), Eff::AddItem("dl_note"), Eff::Points(15)], route: Route::To("dl_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：升腾的灰雾柱 =================
    SceneDef {
        id: "dl_ascent", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 灰雾柱"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "一两道灰雾柱正从裂缝深处的某几处裂口直冲上来，像巨人淋过雨的大树，把整片天穹都染上一层灰。这是裂缝「醒着」的证明——它并非静止的伤，而是活的呼吸口。",
            "你抬头看那些雾柱顶端，总有个瞬间你会错觉雾里有人形；揉揉眼，又只剩灰。你把「深渊在冒气」这件事记下，心里却清楚：越涨越浓的雾，迟早要漫过崖岸。",
        ]),
        choices: &[ChoiceDef { label: "记下雾柱的异动", sub: "MarkPoint · +15 点 · 活着的伤", cond: None,
            effects: &[Eff::MarkPoint("dl_pt_ascent"), Eff::Points(15), Eff::San(-5)], route: Route::To("dl_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：探矿者 =================
    SceneDef {
        id: "dl_prospector", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 边缘营地"),
        mood: "cold", speaker: Some("探矿者"), voice: None,
        text: TextSpec::Static(&[
            "一个攥着磨损罗盘的探矿者坐在营火边，火光照得他脸上的刻痕忽明忽暗。「你别皱眉——这裂缝下头有矿，可也有比矿更糟的东西。」",
            "他压低声音：「我在这一带挖了十年，从没见过它老实过。平白给你点甜头，底下就憋着下一场塌。你看那些塌下去的屋子——全是贪它点东西的。」",
            "他摆摆手：「你要过去？走吊桥，别信别的路。也别在这过夜——夜里，裂口的雾，是会「往上来」的。」",
        ]),
        choices: &[ChoiceDef { label: "听探矿者说完", sub: "AddItem 磨损罗盘 · +10 点", cond: None,
            effects: &[Eff::AddItem("dl_compass"), Eff::Points(10), Eff::San(5)], route: Route::To("dl_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：守夜人 =================
    SceneDef {
        id: "dl_watcher", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 守夜人棚"),
        mood: "mystery", speaker: Some("守夜人"), voice: None,
        text: TextSpec::Static(&[
            "一个披着厚斗篷的人一动不动地坐在裂缝边，面前架着一支长管正对着深渊。他像一座石雕，听见你过来，才缓缓开口：「别出声。我在数它翻身。」",
            "「这条裂缝下头的东西，白天不动，晚上醒。」他声音低沉，「我守了七年，一次也没数清过。可我至少知道——它要是真爬上来，这一带就没有活人了。」",
            "他递给你一支没烧完的短烛：「留个念想。要是撑不住，就看着火，别去看下面。」",
        ]),
        choices: &[ChoiceDef { label: "接过那支短烛", sub: "AddItem 守夜短烛 · +10 点 · San+5", cond: None,
            effects: &[Eff::AddItem("dl_candle"), Eff::Points(10), Eff::San(5)], route: Route::To("dl_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫：深渊聚合 =================
    SceneDef {
        id: "dl_gather", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 雾浓处"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "灰雾忽然不再「上涌」，而是开始向同一个中心收缩、聚拢，像整个深渊在一瞬间屏住了呼吸。岩壁剧烈地颤抖，火把一根根被雾吞灭。",
            "你本能地后退，却发现涌上来的不只是雾——裂缝深处那些松动的东西，正沿着一面近乎垂直的岩壁，向地面攀爬。空气里深处的腐气越来越重。<em>裂口下面的「另一个死亡」，正在往你这条路上来。</em>",
        ]),
        choices: &[
            ChoiceDef { label: "迎向那股聚拢的雾", sub: "看清深渊究竟吐出了什么", cond: None, effects: &NO_EFF, route: Route::To("dl_prelude") },
            ChoiceDef { label: "退到守夜人棚边缓一缓", sub: "借火光稳一稳", cond: None, effects: &[Eff::San(5)], route: Route::To("dl_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dl_prelude", bg: Some("daliexi_bg.png"), loc: Some("大裂隙 · 裂口最先醒的地方"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "灰雾散开的瞬间，你看到了它——一具由无数断裂的肢体、碎骨与岩石糅合成的巨大「聚合体」，正从裂缝里一寸寸探出上半身，像被深渊缝起来的一场噩梦。",
            "裂隙行尸聚合体堵住了跨向对岸的最后一段崖岸。它抬起由无数只手拼成的一只巨掌，朝你缓缓张开——<em>裂口下面是另一个死亡，它爬上来，亲自来收这条路的债了。</em>",
        ]),
        choices: &[ChoiceDef { label: "抬步，直面聚合体", sub: "与裂隙里的东西一战", cond: None, effects: &NO_EFF, route: Route::To("dl_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "dl_01", bg: Some("daliexi_bg.png"), loc: Some("裂口崖岸"), mood: "danger",
        speaker: Some("裂隙行尸聚合体"), voice: None,
        text: TextSpec::Static(&["裂隙行尸聚合体 挡在出口，由无数碎裂之躯拼成的巨影缓缓压近。裂口下面，是另一个死亡。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "握紧守夜短烛", sub: "+5 · 借火光看清它的破绽", cond: None, effects: &[Eff::Points(5)], route: Route::To("dl_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dl_round", bg: Some("daliexi_bg.png"), loc: Some("决战 · 裂口崖岸"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("裂隙行尸聚合体 尚余 {} 气力，你 HP {}。每打散一缕，深渊就退一段。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 dl_card） =================
    SceneDef {
        id: "dl_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["聚合体在灰雾里碎裂、坠落、重新沉入深渊。你在离开前，站在崖岸上，最后看了一次这道吞过许多东西的裂缝。"]),
        choices: &[ChoiceDef { label: "记住裂缝归于平静的模样", sub: "+30 点 · 深渊的沉睡", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("dl_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dl_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走了探矿者那支磨损的罗盘。它最后的指向仍朝着那条裂缝——像这片大地，永远记得自己裂过的地方。"]),
        choices: &[ChoiceDef { label: "把罗盘收进背囊", sub: "+25 点 · AddItem 磨损罗盘 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("dl_compass")], route: Route::To("dl_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "dl_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你在崖岸边坐了很久，直到灰雾真正散去、露出裂缝的寂静轮廓。你没有急着离开——你要记住，这片裂开的大地，也曾是疗愈自己的一部分。"]),
        choices: &[ChoiceDef { label: "在崖边多留一会儿", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("dl_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "dl_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "大裂隙 · 结算".into(), good: true,
                body_html: format!("<p>你越过了大裂隙，也把深渊里爬出来的东西送了回去。</p><p>雾散后，裂缝把缄默还给大地。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "dl_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("大裂隙 · 殒命于深渊", "在裂口崖岸被裂隙行尸聚合体吞没")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你没能越过大裂隙。裂口下面那个「另一个死亡」，把你当成又一块它拼进去的碎块，缓缓带回了雾里。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命于深渊】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn daliexi_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("dl_boss", FightCfg {
            name: "裂隙行尸聚合体", hp: 220, dmg: (18, 28), reward: 500, reward_why: "击败 BOSS", intro: "聚合体探出崖岸——深渊正在往上爬！",
            rage_at: Some(60), rage_text: "聚合体狂怒地攥紧满手断肢，灰雾喷涌！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "dl_card".to_string(), death: "dl_death",
        }),
    ]
}
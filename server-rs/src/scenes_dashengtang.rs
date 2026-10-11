//! 死亡开端 · 大教堂圣所 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「圣光最盛处，腐得最深。」
//! 结构：开场(哥特教堂) → ds_hub(调查中转) → 调查点/圣光下的圣所 + NPC → 圣光生变 → 污染圣物之灵迎战 → 开放结局三选 → ds_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("ds_boss") {
            st.fight = Some(crate::power::scaled_fight("ds_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "ds_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "ds_death".to_string(); }
    "ds_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("ds_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "ds_reward");
    "ds_card".to_string()
}

pub static DASHENGTANG_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "ds_00", bg: Some("dashengtang_bg.png"), loc: Some("大圣所 · 正门"),
        mood: "cold", speaker: Some("侧翼执事之声"), voice: Some("vo_dashengtang_1"),
        text: TextSpec::Static(&[
            "这座哥特教堂比任何一座都高、都暗。尖拱如合拢的指尖直插穹顶，彩色玻璃把白昼滤成一片深重的紫红。你推开厚重的正门，一步踏入——圣所内壁，烛火通明。",
            "烛与光本该是这里最温暖的景致，可你闻到的却是一股陈朽的甜腥。圣坛上那位「圣光」最明亮处，大理石表面却爬满细密如血管的金色裂纹。",
            "你心头一凛——那句流传在这片土地上的话浮上来：<em>「圣光最盛处，腐得最深。离祭坛越近的祝福，越要当心。」</em>",
        ]),
        choices: &[
            ChoiceDef { label: "向圣坛走近几步", sub: "看看圣光最盛的地方", cond: None, effects: &NO_EFF, route: Route::To("ds_tanhai") },
            ChoiceDef { label: "先打量整座圣所", sub: "记住烛火与回廊的格局", cond: None, effects: &NO_EFF, route: Route::To("ds_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "ds_hub", bg: Some("dashengtang_bg.png"), loc: Some("大圣所 · 中殿"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你在中殿站定，抬头看向那束从穹顶泻下的光。它正中悬着一尊巨大的吊灯，圣光在此处最盛——也是最怪的地方：明明该是洁净的辉光，却透出一层不祥的金灰。",
            "这座圣所有无数条回廊通向未知：陈列室、圣坛、地下墓穴。你要穿过它，就得先弄明白，那股在圣光里悄悄发腻的甜腥，究竟藏在哪里。",
        ]),
        choices: &[
            ChoiceDef { label: "圣物陈列室", sub: "被封存的圣器", cond: None, effects: &NO_EFF, route: Route::To("ds_reliquary") },
            ChoiceDef { label: "大吊灯下", sub: "圣光最盛的地方", cond: None, effects: &NO_EFF, route: Route::To("ds_chandelier") },
            ChoiceDef { label: "圣坛前", sub: "那道金色的裂纹", cond: None, effects: &NO_EFF, route: Route::To("ds_tanhai") },
            ChoiceDef { label: "地下墓穴", sub: "圣所深处的旧骨", cond: None, effects: &NO_EFF, route: Route::To("ds_crypt") },
            ChoiceDef { label: "执灯人靠近", sub: "与前来祈祷的人交谈", cond: None, effects: &NO_EFF, route: Route::To("ds_acolyte") },
            ChoiceDef { label: "随圣光往更深的地方走", sub: "走进那股甜腥的源头", cond: None, effects: &NO_EFF, route: Route::To("ds_gather") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：圣物陈列室 =================
    SceneDef {
        id: "ds_reliquary", bg: Some("dashengtang_bg_open.png"), loc: Some("圣物陈列室"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "陈列室里一部一部圣骨匣陈列在玻璃罩后，烛光在匣盖的金片上流转。你俯身看，其中一具匣盖内侧，有人用极细的字刻着一句话，几乎被岁月磨平。",
            "「吾日以此匣承圣光，光愈盛，匣愈沉。它先是装圣髑的器皿，后来……装的是别的。」你一时分辨不出，那令匣盖沉甸甸的，究竟是祝福，还是别的什么。",
        ]),
        choices: &[ChoiceDef { label: "记下匣盖上的那句刻字", sub: "MarkPoint · +15 点 · 圣匣之重", cond: None,
            effects: &[Eff::MarkPoint("ds_pt_reliquary"), Eff::Points(15), Eff::San(-5)], route: Route::To("ds_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：大吊灯下 =================
    SceneDef {
        id: "ds_chandelier", bg: Some("dashengtang_bg_open.png"), loc: Some("中殿 · 大吊灯下"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "你站到大吊灯正下方，把脸投向头顶那圈最亮的光。圣光一贯的洁净此刻却让你后背发凉——你注意到，吊灯的铜链上，每隔几环就缠着一缕细而黑的线。",
            "它们在圣光最盛处微微蠕动，像在吸吮什么。你忽然懂了那句话的另一层意思：最亮的光，恰是最容易被黑暗借住的地方。",
        ]),
        choices: &[ChoiceDef { label: "记下吊灯上的异样", sub: "MarkPoint · San-5 · 光里的黑线", cond: None,
            effects: &[Eff::MarkPoint("ds_pt_chandelier"), Eff::San(-5)], route: Route::To("ds_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：圣坛 =================
    SceneDef {
        id: "ds_tanhai", bg: Some("dashengtang_bg_open.png"), loc: Some("圣坛前"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "走近了才发现，圣坛上那道金色裂纹不是岁月——它细密如蛛网，正沿着大理石缓缓蔓延，像活着的根。裂纹最深处，渗出一缕幽暗的、几乎看不清的甜气。",
            "你伸手去触那道裂纹，指尖擦过的瞬间，整个圣所的光线都暗了一瞬。你收回手，掌心里留着一丝不属于圣坛的寒意。<em>它确实在盛光之下，悄然腐着。</em>",
        ]),
        choices: &[ChoiceDef { label: "记下这道活着的裂纹", sub: "MarkPoint · +20 点 · 盛光的腐", cond: None,
            effects: &[Eff::MarkPoint("ds_pt_altar"), Eff::Points(20), Eff::San(-5)], route: Route::To("ds_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：地下墓穴 =================
    SceneDef {
        id: "ds_crypt", bg: Some("dashengtang_bg_invest.png"), loc: Some("大圣所 · 地下墓穴"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "沿一道旋转石阶下到墓穴，冷气扑面。这里的骨殖被码放得整整齐齐，每一具上都盖着织工细致的圣布——这是圣所有意为之的「安息」。",
            "可最靠近入口、本该最被祝福的那一具棺木，圣布却滑落一角，露出一截泛黑的指骨。你替它把圣布盖好，指节触到棺壁时，好像听到极轻的一声「谢谢」。",
        ]),
        choices: &[ChoiceDef { label: "为棺木重新盖上圣布", sub: "MarkPoint · +15 点 · 墓穴安息", cond: None,
            effects: &[Eff::MarkPoint("ds_pt_crypt"), Eff::Points(15), Eff::San(10)], route: Route::To("ds_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：执灯人 =================
    SceneDef {
        id: "ds_acolyte", bg: Some("dashengtang_bg_open.png"), loc: Some("中殿 · 执灯人"),
        mood: "cold", speaker: Some("执灯人 · 低声"), voice: None,
        text: TextSpec::Static(&[
            "一个披着深蓝法衣的执灯人提着一盏小灯，正一排排点亮烛台。他见了你并不吃惊，只缓缓道：「来祈祷的吗？这里人不多啦——都怕那光。」",
            "他指了指自己提的手灯：「这小灯，是我们私下用的。堂里的圣光太盛，盛到没人肯多待。他们都说，最亮的地方养着最深的阴影。」",
            "他把一盏未燃的烛递给你：「这一盏，拿你自己的火点亮。别借圣所的光——那光，喂过太久的东西了。」",
        ]),
        choices: &[ChoiceDef { label: "接过那盏未燃的烛", sub: "AddItem 手灯 · +10 点 · San+5", cond: None,
            effects: &[Eff::AddItem("ds_lantern"), Eff::Points(10), Eff::San(5)], route: Route::To("ds_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：守门司事 =================
    SceneDef {
        id: "ds_verger", bg: Some("dashengtang_bg_open.png"), loc: Some("圣所侧门 · 司事"),
        mood: "cold", speaker: Some("守门司事"), voice: None,
        text: TextSpec::Static(&[
            "侧门边的司事正在擦拭一杆不知名的长烛架，听见你脚步头也不抬：「里头那束光，年轻，自以为是光的也都那么想。等它自己发觉照出了什么，就晚了。」",
            "「我在这门口守了三十年，见过的人不多，可每一个都嫌里头太亮。」他顿了顿，「亮得能烫伤眼睛的祝福，往往离诅咒只有一层薄薄的距离。」",
            "他把一支拔了芯的粗烛塞给你：「要是那光不对劲了，就点这个——它烧的是蜡，不是祈祷。」",
        ]),
        choices: &[ChoiceDef { label: "收下那支粗蜡烛", sub: "AddItem 守门蜡烛 · +10 点 · 司事的话", cond: None,
            effects: &[Eff::AddItem("ds_candle"), Eff::Points(10)], route: Route::To("ds_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫：圣光生变 =================
    SceneDef {
        id: "ds_gather", bg: Some("dashengtang_bg_open.png"), loc: Some("大圣所 · 圣光渐热"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你越是往圣所深处走，那束圣光越是发烫。吊灯下的黑线一根根鼓胀起来，圣坛的金色裂纹像被浇了油般蔓延，整座圣所的地面都在极轻地颤动。",
            "你发现光不再「照」你——它在「看」你。那股甜腥的腐气浓得像能拧出水，逼得你必须扶着墙才站得稳。<em>圣光自己，成了这圣所里最污秽的东西。</em>",
        ]),
        choices: &[
            ChoiceDef { label: "循着腐气的源头走", sub: "逼近那个揣着圣光的东西", cond: None, effects: &NO_EFF, route: Route::To("ds_prelude") },
            ChoiceDef { label: "退到执灯人身边缓一缓", sub: "借一捧自己的微光", cond: None, effects: &[Eff::San(5)], route: Route::To("ds_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ds_prelude", bg: Some("dashengtang_bg_battle.png"), loc: Some("圣坛后 · 圣光之核"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "圣坛后那片最亮的光里，有什么正在成形——一具蜷曲的、由圣骨与阴影糅成的形体缓缓舒展开来。它身上披着揉碎的圣布，眉眼处却是一片空洞的、吞光的黑。",
            "这是「污染圣物之灵」——被圣所供奉了太久的圣光，终于在盛极处养出了自己的影子。它堵在通往圣所出口的甬道上。<em>圣光最盛处，腐得最深——此刻，它亲自站在你面前。</em>",
        ]),
        choices: &[ChoiceDef { label: "走向那片圣光之核", sub: "直面污染圣物之灵", cond: None, effects: &NO_EFF, route: Route::To("ds_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "ds_01", bg: Some("dashengtang_bg_invest.png"), loc: Some("圣坛后的甬道"), mood: "danger",
        speaker: Some("污染圣物之灵"), voice: None,
        text: TextSpec::Static(&["污染圣物之灵 挡在出口，圣光在它身侧淌成黑。圣光最盛处，腐得最深。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "取出守门蜡烛，照它的影子", sub: "+5 · 用蜡分清光与影", cond: None, effects: &[Eff::Points(5)], route: Route::To("ds_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ds_round", bg: Some("dashengtang_bg_battle.png"), loc: Some("决战 · 圣光之核"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("污染圣物之灵 尚余 {} 气力，你 HP {}。每斩断一缕腐光，圣所就还你一分清白。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 ds_card） =================
    SceneDef {
        id: "ds_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["污染圣物之灵在圣光里散尽，金色的裂纹一寸寸闭合。你在离开前，最后看了一次这座重新归于寂静的圣所。"]),
        choices: &[ChoiceDef { label: "记住圣光洗净后的穹顶", sub: "+30 点 · 圣所的安详", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("ds_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ds_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走了执灯人那盏手灯。走出圣所时它仍燃着——用自己的火点亮的东西，才不畏惧任何光。"]),
        choices: &[ChoiceDef { label: "提着那盏手灯离开", sub: "+25 点 · AddItem 手灯 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("ds_lantern")], route: Route::To("ds_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ds_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你在中殿的长椅上坐了很久，直到圣光褪成寻常的、温柔的日光。这座圣所终于可以只为活着的人点亮了。"]),
        choices: &[ChoiceDef { label: "在圣所再坐一会儿", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("ds_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "ds_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "大教堂圣所 · 结算".into(), good: true,
                body_html: format!("<p>你穿过大圣所的光与影，也弄懂了那句话。</p><p>盛极的圣光刺破自己的影子，把清白还给穹顶。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "ds_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("大教堂圣所 · 殒命于圣光", "在圣坛后被污染圣物之灵吞没")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你倒在大圣所最亮的那束光里。盛极的圣光终于把它养了太久的东西，喂饱了。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命于圣光】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn dashengtang_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("ds_boss", FightCfg {
            name: "污染圣物之灵", hp: 180, dmg: (16, 24), reward: 500, reward_why: "击败 BOSS", intro: "圣光之核睁开了眼——污染圣物之灵动身了！",
            rage_at: Some(60), rage_text: "圣光凝成稠黑，污染圣物之灵开始咆哮！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "ds_card".to_string(), death: "ds_death",
        }),
    ]
}
//! 死亡开端 · 死雾镇 scenes（精致副本）
//! 世界展示向·剧情开放·无真相指向。
//! 世界观钩子：「雾里没有活人。可雾，还记得他们。」
//! 结构：开场(灰雾小镇) → sw_hub(调查中转) → 调查点/雾中遗物 + NPC → 雾深铺垫 → 行尸之王迎战 → 开放结局三选 → sw_card。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sw_boss") {
            st.fight = Some(crate::power::scaled_fight("sw_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "sw_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "sw_death".to_string(); }
    "sw_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("sw_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "sw_reward");
    "sw_card".to_string()
}

pub static SIWUZHEN_SCENES: &[SceneDef] = &[
    // ================= 开场 =================
    SceneDef {
        id: "sw_00", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 镇口"),
        mood: "cold", speaker: Some("旁白"), voice: Some("vo_sw_open"),
        text: TextSpec::Static(&[
            "灰雾像一层湿透的纱，把整个小镇糊在半死不活的光里。木屋的窗全钉着木板，路灯灭在雾里，只有远处教堂的尖顶露出一角。",
            "空气里没有别的味道——只有土腥，和一种「被搬空」的安静。你路过第一扇门，门板上用粉笔写着歪歪扭扭的字：<em>「雾里没有活人。快走。」</em>",
            "可雾的那一头，传来很轻的、拖沓的脚步声——像是有人（或者什么东西）正在镇子里，一间一间地找着什么。",
        ]),
        choices: &[
            ChoiceDef { label: "念那行粉笔字", sub: "San-5 · 记下警告", cond: None, effects: &[Eff::San(-5)], route: Route::To("sw_hub") },
            ChoiceDef { label: "循着脚步的方向走", sub: "向小镇深处", cond: None, effects: &NO_EFF, route: Route::To("sw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= hub 中转站 =================
    SceneDef {
        id: "sw_hub", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 主街"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你站在死雾镇唯一一条主街上，雾把两边的铺子都泡得发白。缝纫店、杂货铺、牙医诊室——招牌字迹还清晰，只是再没有一盏灯属于活人。",
            "雾最淡的方向是一条通向教堂的石板路，最浓的则绕向镇后的那条河。你要穿过这个小镇，就得先弄明白，雾到底想留住什么。",
        ]),
        choices: &[
            ChoiceDef { label: "镇中央的水井", sub: "镇里唯一的活水", cond: None, effects: &NO_EFF, route: Route::To("sw_well") },
            ChoiceDef { label: "废弃教堂", sub: "尖顶下的回廊", cond: None, effects: &NO_EFF, route: Route::To("sw_church") },
            ChoiceDef { label: "雾中邮局", sub: "未曾寄出的信", cond: None, effects: &NO_EFF, route: Route::To("sw_post") },
            ChoiceDef { label: "镇招待所", sub: "留下的行囊", cond: None, effects: &NO_EFF, route: Route::To("sw_hotel") },
            ChoiceDef { label: "雾里有人影", sub: "跟上那片模糊的轮廓", cond: None, effects: &NO_EFF, route: Route::To("sw_hermit") },
            ChoiceDef { label: "走向雾最深的地方", sub: "把镇子交给夜色", cond: None, effects: &NO_EFF, route: Route::To("sw_gray") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：水井 =================
    SceneDef {
        id: "sw_well", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 中央水井"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "镇子正中凿着一口老井，井沿被绳索磨出深深的光滑凹槽。坠下去的桶悬在半途，井水幽深，却没有一丝倒影。",
            "你俯身看下去，贴着井壁的石缝里长着几丛青苔——在这个满是灰雾的、像被拧干了水分的镇子里，它们是难得还「绿」着的东西。井水轻轻晃了晃，像是回应你的目光。",
        ]),
        choices: &[ChoiceDef { label: "舀起一捧井水", sub: "MarkPoint · +15 点 · 活水", cond: None,
            effects: &[Eff::MarkPoint("sw_pt_well"), Eff::Points(15), Eff::San(5)], route: Route::To("sw_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：教堂 =================
    SceneDef {
        id: "sw_church", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 废弃教堂"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "教堂的门虚掩着，里面齐刷刷排满长椅，每张椅子上都放着一件干净的旧衣——像是全镇的人在某天清晨离开了，只把外衣留了下来。",
            "祭坛上的蜡烛早已燃尽，蜡油凝成一滩滩。有人用炭在墙上描了一幅画：一群人正往雾里走，回头望着身后空空的镇子。",
        ]),
        choices: &[ChoiceDef { label: "记下那幅画像", sub: "MarkPoint · +15 点 · 离去的信徒", cond: None,
            effects: &[Eff::MarkPoint("sw_pt_church"), Eff::Points(15)], route: Route::To("sw_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：邮局 =================
    SceneDef {
        id: "sw_post", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 雾中邮局"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "邮局的柜台堆着成捆的信，邮戳停在同一天。你随手拾起一封拆开，信纸被水汽泡得发皱：「亲爱的，我一定把这批货送出去。雾再大，路我认得……」",
            "信封背面用更幼的字迹补了一句：「如果这封信没能寄到，那大概是我在路上。」你数了数——这些信，全都没有贴邮票。",
        ]),
        choices: &[ChoiceDef { label: "带走那封没寄出的信", sub: "MarkPoint · AddItem 雾中信 · San+5", cond: None,
            effects: &[Eff::MarkPoint("sw_pt_post"), Eff::AddItem("sw_letter"), Eff::San(5)], route: Route::To("sw_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 调查点：招待所 =================
    SceneDef {
        id: "sw_hotel", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 镇招待所"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&[
            "招待所前台挂着一串钥匙，登记簿翻开在某一页，墨迹尚未干透，却已是多年前的日期。每个房间门口都留着一只行李箱，像是住客走得非常仓促。",
            "登记簿的最后一页，有人用铅笔写了一行小字：「如果还有人来，告诉他我去了雾那边，去找那句话。」你合上薄子，不知道「那句话」指的是什么。",
        ]),
        choices: &[ChoiceDef { label: "摘下前台一枚钥匙", sub: "MarkPoint · +10 点 · 未归的住客", cond: None,
            effects: &[Eff::MarkPoint("sw_pt_hotel"), Eff::Points(10)], route: Route::To("sw_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：雾里的隐者 =================
    SceneDef {
        id: "sw_hermit", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 废弃磨坊"),
        mood: "cold", speaker: Some("磨坊主 · 隐者"), voice: None,
        text: TextSpec::Static(&[
            "你在一座停转的磨坊里找到一个人。他裹着油布，正把一小把麦磨成粉，动作很慢、很专注。见你进来，他抬起眼：「镇上最后一个活人？……哦，我啊。我又算哪门子活人。」",
            "「雾是个念想，孩子。人走了，雾替他们把日子罩着，好让谁不迷走。」他顿了顿，「别信那句话——『雾里没有活人』。雾里全是旧人，只是没人肯醒了。」",
            "他把一小袋磨好的粉推给你：「带着。路再长，也得分得清哪边是家。」",
        ]),
        choices: &[ChoiceDef { label: "接过那袋面粉", sub: "AddItem 磨坊面粉 · +10 点 · San+5", cond: None,
            effects: &[Eff::AddItem("sw_flour"), Eff::Points(10), Eff::San(5)], route: Route::To("sw_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= NPC：雾边搭话的拾荒者 =================
    SceneDef {
        id: "sw_sister", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 雾边的空屋"),
        mood: "mystery", speaker: Some("拾荒者 · 一路人"), voice: None,
        text: TextSpec::Static(&[
            "空屋里蹲着一个披着灰头巾的拾荒者，正把几件旧衣裳仔细叠好。她没抬头：「你也逃过来的？」又自问自答，「这儿雾大，路不认人，可东西还行——都给镇里那些人留着呢。」",
            "「这镇子奇怪，人不在了，物件倒都齐整。好像它们舍不得，非要替主人守着家。」她抬头笑了笑，「你说，雾是想留住他们，还是想让他们别回来了？」",
            "她把一枚干净的手帕递给你：「擦擦吧。走远路，最忌身上沾灰。」",
        ]),
        choices: &[ChoiceDef { label: "收下手帕和那番话", sub: "+10 点 · 雾边的话", cond: None,
            effects: &[Eff::Points(10), Eff::San(5)], route: Route::To("sw_hub") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战铺垫：雾深 =================
    SceneDef {
        id: "sw_gray", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 雾最深处的街口"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你往前走，雾越来越稠，稠到像能挤出水的灰棉花。四下里那些拖沓的脚步声停了——取而代之的，是你自己的呼吸声，被雾无限放大。",
            "雾里浮起一具又一具模糊的人影，它们都面朝着同一个方向，缓缓向镇后那条河移动。你忽然意识到：这就是那句话的意思——<em>雾里没有活人，只有一群永远走不出去的旧影。</em>",
        ]),
        choices: &[
            ChoiceDef { label: "顺着人影的方向追去", sub: "逼近那个「行尸之王」", cond: None, effects: &NO_EFF, route: Route::To("sw_prelude") },
            ChoiceDef { label: "先回主街喘口气", sub: "把雾看透再走", cond: None, effects: &[Eff::San(5)], route: Route::To("sw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sw_prelude", bg: Some("siwuzhen_bg.png"), loc: Some("死雾镇 · 镇后河岸"),
        mood: "danger", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "河岸的雾几乎凝成实体。在雾最浓的那一团里，一个比其他旧影高大得多的轮廓正缓缓直起身——它裹着缠满水草的麻衣，像一具会走路的雾本身。",
            "它堵住了河上唯一那座桥。<em>雾中行尸之王——镇子里所有没走掉的人，最后都成了一缕它还留着的执念。</em>你要过河，就得穿过它。",
        ]),
        choices: &[ChoiceDef { label: "踏雾走向桥头", sub: "直面行尸之王", cond: None, effects: &NO_EFF, route: Route::To("sw_01") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 迎战 BOSS（保留原结构） =================
    SceneDef {
        id: "sw_01", bg: Some("siwuzhen_bg.png"), loc: Some("镇后河桥桥头"), mood: "danger",
        speaker: Some("雾中行尸之王"), voice: None,
        text: TextSpec::Static(&["雾中行尸之王 挡在出口。雾里没有活人——可它偏偏还记得自己曾是个人。" ]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "屏息，看清它的动作", sub: "+5 · 记住雾的味道", cond: None, effects: &[Eff::Points(5)], route: Route::To("sw_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sw_round", bg: Some("siwuzhen_bg.png"), loc: Some("决战 · 河桥之上"), mood: "danger",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| format!("雾中行尸之王 尚余 {} 气力，你 HP {}。每击散一缕雾，它的执念就薄一分。", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 开放结局（三选，均回 sw_card） =================
    SceneDef {
        id: "sw_end_look", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["行尸之王在雾里碎成一地尘土，像终于把心事还给了土地。你在离开前，站在桥头，最后看了一次这座被雾罩住的镇子。"]),
        choices: &[ChoiceDef { label: "记住雾后的剪影", sub: "+30 点 · 雾镇的黄昏", cond: None,
            effects: &[Eff::Points(30), Eff::San(10)], route: Route::To("sw_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sw_end_souvenir", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你带走了那封没寄出的信。离开雾镇后，你试着把它寄出去——哪怕收件人早已不存在。这封信，总算有了出路。"]),
        choices: &[ChoiceDef { label: "把信贴身收好", sub: "+25 点 · AddItem 雾中信 · 纪念", cond: None,
            effects: &[Eff::Points(25), Eff::AddItem("sw_letter")], route: Route::To("sw_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sw_end_stay", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["你又在磨坊外站了一会儿。雾散了些，露出镇子曾经的模样——像一具终于安眠的骨，不再攥着谁不放。"]),
        choices: &[ChoiceDef { label: "看雾散尽再上路", sub: "+20 点 · 就地停留", cond: None,
            effects: &[Eff::Points(20), Eff::San(5)], route: Route::To("sw_card") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    // ================= 结算卡 =================
    SceneDef {
        id: "sw_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "死雾镇 · 结算".into(), good: true,
                body_html: format!("<p>你穿过了死雾镇，也穿过了它的执念。</p><p>桥头的雾散开，河对岸的路重新清晰。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    // ================= 死亡卡 =================
    SceneDef {
        id: "sw_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("死亡开端 · 死雾镇 · 殒命于雾", "在死雾镇的河桥被雾中行尸之王击垮")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你倒在了死雾镇的雾里。河水把你的影子带远，旧影的队伍里，又多了一缕没走成的人。</p><p style='color:#ff8a8a'>【死亡档案 · 殒命于雾】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn siwuzhen_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sw_boss", FightCfg {
            name: "雾中行尸之王", hp: 180, dmg: (16, 24), reward: 500, reward_why: "击败 BOSS", intro: "行尸之王在雾里直起身——雾浓得化不开了！",
            rage_at: Some(60), rage_text: "行尸之王震怒，雾涛翻涌！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "sw_card".to_string(), death: "sw_death",
        }),
    ]
}
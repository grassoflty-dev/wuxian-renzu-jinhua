//! 《死亡开端·霜白村》场景
//! 第一夜灰雾降临、村中枯井、首位复苏者。
//! 钩子：「所有雾，都是从这口井里长出来的。」
use crate::defs::*;
use crate::state::{Card, GameState};

static NO_EFF: [Eff; 0] = [];

// —— 条件（具名 fn，不能捕获闭包）——
fn cond_has_greykey(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "sb_key_grey") }
fn cond_has_rope(st: &GameState) -> bool { st.flag("sb_has_rope") }

// —— 路由 fn（返回 String）——
fn route_noop(_st: &mut GameState) -> String { "sb_hub".to_string() }
fn route_goto_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sb_boss") {
            st.fight = Some(crate::power::scaled_fight("sb_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "sb_boss_round".to_string()
}
// —— on_rage 空实现（BOSS 回合由场景文本呈现狂暴）——
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// —— 选择驱动 BOSS 回合逻辑（首位复苏者 HP150）——
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    let hit = !guard;
    if hit { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "sb_death".to_string(); }
    "sb_boss_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 1500;
    st.set_flag("sb_boss_down");
    st.sp_grade = Some('A');
    crate::world::add_item(st, "sb_ash_shard");
    "sb_settle".to_string()
}
fn route_death(_st: &mut GameState) -> String { "sb_death".to_string() }

pub static SHUANGBAI_SCENES: &[SceneDef] = &[
    // —— 开场：灰雾降临 ——
    SceneDef {
        id: "sb_00",
        bg: Some("shuangbai_bg.png"),
        loc: Some("霜白村村口"),
        mood: "mystery",
        speaker: Some("守井的老温"),
        voice: Some("vo_shuangbai_1"),
        text: TextSpec::Static(&[
            "黄昏最后一缕光被灰雾吞没。霜白村笼在白纱里，静得只剩下雾气爬过屋脊的细响。",
            "你一路追着那场瘟疫的尾巴，来到地图尽头这座无名村。枯井立在村中央，井沿结着一层灰白的霜。",
            "老温站在井边，火把在雾里只剩一圈昏黄。他开口，声音像从井底传上来的：",
            "「所有雾，都是从这口井里长出来的。」",
        ]),
        choices: &[
            ChoiceDef {
                label: "问老温井的事",
                sub: "打听这场雾的来历",
                cond: None,
                effects: &NO_EFF,
                route: Route::To("sb_05"),
            },
            ChoiceDef {
                label: "检查枯井井沿",
                sub: "伸手摸那道霜",
                cond: None,
                effects: &[Eff::Points(10)],
                route: Route::To("sb_01"),
            },
            ChoiceDef {
                label: "先看看老屋",
                sub: "沿村道走向老屋",
                cond: None,
                effects: &NO_EFF,
                route: Route::To("sb_03"),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 枯井井沿 ——
    SceneDef {
        id: "sb_01",
        bg: Some("shuangbai_bg.png"),
        loc: Some("枯井井沿"),
        mood: "mystery",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "井口径不足一人，井沿那道霜泛着微光，凑近能看见雾从井口的缝隙里一缕缕渗出。",
            "「别靠太近，」老温在你身后压着嗓子，「这雾会掐脖子。我夜里听见过井底有人说话。」",
            "你想顺着井壁往下看，黑得什么也瞧不见，只有湿冷的潮气贴上来。",
        ]),
        choices: &[
            ChoiceDef {
                label: "顺着井壁往下爬",
                sub: "下井查个究竟",
                cond: Some(cond_has_rope),
                effects: &[Eff::SetFlag("sb_went_down")],
                route: Route::To("sb_02"),
            },
            ChoiceDef {
                label: "找根绳子再来",
                sub: "井口太滑，差一段绳",
                cond: Some(cond_has_rope),
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
            ChoiceDef {
                label: "推一截朽木盖井口",
                sub: "暂时压住渗出的雾",
                cond: None,
                effects: &[Eff::Points(15)],
                route: Route::Dyn(route_noop),
            },
            ChoiceDef {
                label: "回村道",
                sub: "退开枯井",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 下井 ——
    SceneDef {
        id: "sb_02",
        bg: Some("shuangbai_bg.png"),
        loc: Some("井道（下沉中）"),
        mood: "danger",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "你把绳头系在老屋的门闩上，咬住火折子顺井壁滑下。",
            "下滑到一半，井壁渗出密密麻麻的霜，像无数只眼睛。井底传来一声低沉的、像是骨头在泥里搅动的声音。",
            "你双脚落在一片湿冷地面，头顶遥遥的光只剩一个白点。这里就是雾的源头。",
        ]),
        choices: &[
            ChoiceDef {
                label: "点亮火折子张望",
                sub: "看清井底白骨坪",
                cond: None,
                effects: &[Eff::SetFlag("sb_went_down")],
                route: Route::To("sb_12"),
            },
            ChoiceDef {
                label: "顺着来路退回井口",
                sub: "先回村做准备",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 老屋门口 ——
    SceneDef {
        id: "sb_03",
        bg: Some("shuangbai_bg.png"),
        loc: Some("老屋门口"),
        mood: "mystery",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "老屋的门早已腐朽，门缝里飘出一股陈年糕饼腐坏的气味。堂屋正中供着一口小小的枯井模型，井口盖着个泥封。",
            "墙角的灰雾比其他地方浓，凝成人形的轮廓，又在你眨眼时散开。",
        ]),
        choices: &[
            ChoiceDef {
                label: "掀开泥封",
                sub: "查看那口枯井模型",
                cond: None,
                effects: &[Eff::Points(15)],
                route: Route::To("sb_04"),
            },
            ChoiceDef {
                label: "翻找老屋",
                sub: "找绳子和线索",
                cond: None,
                effects: &[Eff::SetFlag("sb_has_rope"), Eff::Points(10)],
                route: Route::To("sb_06"),
            },
            ChoiceDef {
                label: "回村道",
                sub: "离开老屋",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 谷仓（门禁）——
    SceneDef {
        id: "sb_04",
        bg: Some("shuangbai_bg.png"),
        loc: Some("谷仓门口"),
        mood: "mystery",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "老屋模型下的泥封里没有水，只有一团还在缓慢旋转的灰雾，被看得见摸不着的气旋托着。",
            "「这是井的倒影，」老温不知何时站在门口，「它锁着的东西在村西谷仓——第一个被雾薅走的人，就封在那里。」",
            "谷仓的门被枯绳与灰雾缠死，隐约能听见里面传来指甲刮擦木板的声音。",
        ]),
        choices: &[
            ChoiceDef {
                label: "用游魂掉落的枯绳开门",
                sub: "需要 sb_key_grey",
                cond: Some(cond_has_greykey),
                effects: &[Eff::Points(20)],
                route: Route::To("sb_07"),
            },
            ChoiceDef {
                label: "用力推门",
                sub: "门封得死紧，推不开",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
            ChoiceDef {
                label: "回村道",
                sub: "先处理井里的事",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 谷仓开门的遭遇（可选遭遇战）——
    SceneDef {
        id: "sb_07",
        bg: Some("shuangbai_bg.png"),
        loc: Some("谷仓内"),
        mood: "danger",
        speaker: Some("复苏者·旧温"),
        voice: None,
        text: TextSpec::Static(&[
            "枯绳在你指尖碎成灰末，谷仓门吱呀敞开。",
            "里面横着一口黑棺，棺盖已经裂开大半。一个披着村民麻衣的「人」端坐在棺中，脸朝着井的方向，眼窝里没有眼珠，只有一片流动的灰雾。",
            "它开口，和老温一模一样的声音：「回来……都回到雾里来……」",
        ]),
        choices: &[
            ChoiceDef {
                label: "战斗",
                sub: "与扭曲的村民交手",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: Some("sb_fight_warped"),
        video: None, cine_label: None, overlay: None,
    },

    // —— 老温讲述 ——
    SceneDef {
        id: "sb_05",
        bg: Some("shuangbai_bg.png"),
        loc: Some("枯井井沿"),
        mood: "mystery",
        speaker: Some("守井的老温"),
        voice: None,
        text: TextSpec::Static(&[
            "老温用火把拨了拨井沿的霜：「这场雾不是第一天。我爷爷那年，雾从井口涨到屋檐，村人走进去就没了影。」",
            "「死了的人，尸身还会回来。先是那个叫阿禾的孩子，然后是……我儿子。」他顿了顿，「它们都从这口井里爬出来，眼窝里塞着雾，还在叫村里人的名字。」",
            "「你要去井底，就记住：雾里叫你的名字，别应。」",
        ]),
        choices: &[
            ChoiceDef {
                label: "再问复苏者的事",
                sub: "它总在谷仓里坐着",
                cond: None,
                effects: &NO_EFF,
                route: Route::To("sb_04"),
            },
            ChoiceDef {
                label: "检查枯井井沿",
                sub: "顺着井壁往下爬",
                cond: None,
                effects: &NO_EFF,
                route: Route::To("sb_01"),
            },
            ChoiceDef {
                label: "告辞",
                sub: "去老屋/村道",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 老屋翻找，遭遇游魂 ——
    SceneDef {
        id: "sb_06",
        bg: Some("shuangbai_bg.png"),
        loc: Some("老屋阁楼"),
        mood: "danger",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "你撬开朽烂的阁楼踏板，摸到一段编得结实的麻绳，还压着半截熏黑的烛台。",
            "头顶传来瓦片滑动的响动，一团凝成「人」形的灰雾从梁上无声垂落，眼窝的位置对着你，张口的动作像在念你的名字。",
        ]),
        choices: &[
            ChoiceDef {
                label: "战斗",
                sub: "驱散雾中游魂",
                cond: None,
                effects: &[Eff::Points(15)],
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: Some("sb_fight_grey"),
        video: None, cine_label: None, overlay: None,
    },

    // —— 井底白骨坪（大小 BOSS 前）——
    SceneDef {
        id: "sb_12",
        bg: Some("shuangbai_bg.png"),
        loc: Some("井底白骨坪"),
        mood: "danger",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "白骨横七竖八铺成一块坪，从村人骸骨的空荡眼窝里，灰雾正一丝丝往上冒，汇向井口。",
            "坪中央，有一具「人」单膝跪地，背脊上覆着一件白衣，指尖陷进自己胸口的肋骨里。",
            "它没有回头，声音却在你耳边清晰响起——是老温的声音：「阿……禾……回来了吗——」",
            "你认出来了：这是霜白村第一位复苏者，死在井底的那个孩子，如今堵在雾的出口。",
        ]),
        choices: &[
            ChoiceDef {
                label: "拔剑上前",
                sub: "与首位复苏者一战",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_goto_boss),
            },
            ChoiceDef {
                label: "退到井壁，捡起井边的霜石",
                sub: "先冷静观察",
                cond: None,
                effects: &[Eff::Points(10)],
                route: Route::To("sb_13"),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— BOSS 战后复盘分支 ——
    SceneDef {
        id: "sb_13",
        bg: Some("shuangbai_bg.png"),
        loc: Some("井底白骨坪"),
        mood: "mystery",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "你捡起那块霜石，石面冰凉，上面刻着人形的简笔画——像是孩子画的，线条歪斜，一圈一圈缠着一口井。",
            "等你再看，那复苏者已经直起身来，眼窝两团白雾转向你。",
        ]),
        choices: &[
            ChoiceDef {
                label: "拔剑上前",
                sub: "与首位复苏者一战",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_goto_boss),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 选择驱动 BOSS（Normal + Dyn 多段回合）——
    SceneDef {
        id: "sb_boss_round",
        bg: Some("shuangbai_bg.png"),
        loc: Some("井底白骨坪"),
        mood: "danger",
        speaker: Some("首位复苏者"),
        voice: None,
        text: TextSpec::Dyn(|st| format!(
            "灰雾在它周身旋转收紧。首位复苏者还剩 {} 血，你 HP {}。",
            st.fight.as_ref().map(|f| f.hp).unwrap_or(0),
            st.hp
        )),
        choices: &[
            ChoiceDef { label: "重击灰雾核心", sub: "高伤害", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 35, false)) },
            ChoiceDef { label: "闪身/防御", sub: "本回合免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
            ChoiceDef { label: "横斩腿骨", sub: "中等伤害", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 22, false)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— BOSS 战后结算 ——
    SceneDef {
        id: "sb_settle",
        bg: Some("shuangbai_bg.png"),
        loc: Some("井口（雾散）"),
        mood: "awe",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "灰雾在它倒下的一瞬猛地抽紧，随即像被拔掉塞子的水，整片雾沿着井道朝上涌出的方向寸寸收拢。",
            "首位复苏者化成一捧会呼吸的灰烬，风一吹便散进井底。它胸口那件白衣落在地上，翻开，露出一张村人的全家福，边角还画着一口小井。",
            "你从井底爬回地面时，天已微微发白。霜白村的雾淡了，可从那口井的深处，隐约还有一声极远的、叫名字的声音，幽幽通向某个更黑的地方——",
            "死雾镇，还在等。",
        ]),
        choices: &[
            ChoiceDef {
                label: "收好白衣与灰芯",
                sub: "S 级完成本次调查",
                cond: None,
                effects: &[Eff::SetFlag("sb_epilogue")],
                route: Route::Dyn(route_noop),
            },
        ],
        fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None,
            death: None,
            card: |st| {
                let s = if st.flag("sb_epilogue") { "完成" } else { "进行中" };
                Card {
                    title: format!("《死亡开端·霜白村》·{}", s),
                    good: true,
                    body_html: String::from("<b>枯井下的雾，终于夹在井口被日光晒化。</b><br/>你带走了白衣与一捧灰芯，顺着雾最淡的方向往东走。<br/>东边那份更浓的雾下，躺着传说里的死雾镇。"),
                    buttons: vec![("trace".to_string(), "继续追踪灰雾".to_string())],
                    voice: None,
                }
            },
        }),
    },

    // —— 死亡 ——
    SceneDef {
        id: "sb_death",
        bg: Some("shuangbai_bg.png"),
        loc: Some("灰雾深处"),
        mood: "danger",
        speaker: None,
        voice: None,
        text: TextSpec::Static(&[
            "雾裹住了你。它们温驯地缠上你的手腕、脚踝，一寸一寸把你按进井底。",
            "最后一丝清醒里，你听见无数声音同时叫你，最清楚的是老温的：「回来——回到雾里来——」",
            "霜白村的夜里，又多了一个徘徊在井沿的、答话的影。",
        ]),
        choices: &[
            ChoiceDef {
                label: "陷入灰雾",
                sub: "死亡",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn(route_death),
            },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    // —— 中枢（玩家被打回村道）——
    SceneDef {
        id: "sb_hub",
        bg: Some("shuangbai_bg.png"),
        loc: Some("霜白村村道"),
        mood: "mystery",
        speaker: None,
        voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("sb_went_down") {
                "灰雾从井口拧成几缕垂下，像冻僵的手指。你已经摸清井底藏着复苏者——它守着雾的出口。".to_string()
            } else {
                "霜白村的雾又开始变浓。井沿那道霜闪了一下，仿佛在催你走近。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "枯井井沿", sub: "顺着井壁往下爬", cond: None, effects: &NO_EFF, route: Route::To("sb_01") },
            ChoiceDef { label: "老屋门口", sub: "继续翻找", cond: None, effects: &NO_EFF, route: Route::To("sb_03") },
            ChoiceDef { label: "谷仓门口", sub: "查看被封的谷仓", cond: None, effects: &NO_EFF, route: Route::To("sb_04") },
            ChoiceDef { label: "找到老温", sub: "问清楚雾的来历", cond: None, effects: &NO_EFF, route: Route::To("sb_05") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn shuangbai_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sb_fight_grey", FightCfg {
            name: "雾中游魂",
            hp: 60,
            dmg: (8, 14),
            reward: 120,
            reward_why: "驱散游魂",
            intro: "一团灰雾扑向你，喉咙里挤出你的名字。",
            rage_at: Some(30),
            rage_text: "游魂骤然拉长出十几个虚影！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| "".to_string(),
            finisher_desc: |_st| "".to_string(),
            win: |_st| "sb_hub".to_string(),
            death: "sb_death",
        }),
        ("sb_fight_warped", FightCfg {
            name: "扭曲的村民",
            hp: 90,
            dmg: (12, 18),
            reward: 200,
            reward_why: "击败扭曲的村民",
            intro: "穿麻衣的复苏者扑出黑棺，指甲刮过地面的声音混进雾里。",
            rage_at: Some(45),
            rage_text: "它裂开的喉咙里涌出大片灰雾！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| "".to_string(),
            finisher_desc: |_st| "".to_string(),
            win: |_st| "sb_hub".to_string(),
            death: "sb_death",
        }),
        ("sb_boss", FightCfg {
            name: "首位复苏者",
            hp: 150,
            dmg: (16, 22),
            reward: 1500,
            reward_why: "击败首位复苏者",
            intro: "首位复苏者直起身——眼窝两团白雾，朝你张开双臂，仿佛雾的源头在拥抱它的第一个捕获物。",
            rage_at: Some(80),
            rage_text: "它胸口那团雾猛地炸开，白骨坪的雾随风收紧成一只巨手的轮廓！",
            on_rage: rage_none,
            finisher_if: |_st, _ehp| false,
            finisher_name: |_st| "".to_string(),
            finisher_desc: |_st| "".to_string(),
            win: |_st| "sb_settle".to_string(),
            death: "sb_death",
        }),
    ]
}
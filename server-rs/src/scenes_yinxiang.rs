//! 大宇宙时代 · 银色战争 舰桥副本（黄金模板骨架 → 精致副本润色）。
//! 主线：突入舰桥 → 世界展示调查（3 处奇观 + 2 位 NPC）→ BOSS 战前铺垫 → 银色舰长 → 结算 / 死亡。
//! 保留原 start_boss / yinxiang_figths / 结算卡 yx_card / 死亡卡 yx_death / BOSS 遭遇 yx_01 与回合 yx_round 结构与 id 前缀不变；
//! 仅新增开场、奇观调查点、NPC、BOSS 铺垫、开放结局等场景，并把原 yx_00 入口改路由进扩充链。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（原逻辑，保留不动）=====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("yx_boss") {
            st.fight = Some(crate::power::scaled_fight("yx_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "yx_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "yx_death".to_string(); }
    "yx_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("yx_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "yx_reward");
    "yx_card".to_string()
}

pub static YINXIANG_SCENES: &[SceneDef] = &[
    /* ================= 入口（原场景 yx_00，仅改路由进扩充链） ================= */
    SceneDef {
        id: "yx_00", bg: Some("yinxiang_bg.png"), loc: Some("银白升降桥 · 突入点"), mood: "calm",
        speaker: None, voice: Some("vo_yinxiang_1"),
        text: TextSpec::Static(&[
            "你踏入了「大宇宙时代 · 银色战争」。",
            "破损的舰桥在你面前缓缓点亮，四百年的尘埃在真空里悬停，如同凝固的星屑。",
            "「真空里没有声音，」通讯频段里传来一句低语，「但你会听得很清——那是你自己的心跳。」",
        ]),
        choices: &[
            ChoiceDef { label: "借惯性漂过断裂闸门", sub: "San(+5) · 深空突入", cond: None, effects: &[Eff::San(5)], route: Route::To("yx_open_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕一 · 开场扩充（钩子 + 氛围） ================= */
    SceneDef {
        id: "yx_open_1", bg: Some("yinxiang_bg.png"), loc: Some("舷窗 · 悬停在银白战场之上"), mood: "awe",
        speaker: Some("锻（全智噪波）"), voice: None,
        text: TextSpec::Static(&[
            "舷窗外的星海被劈成两半——一侧是燃烧的残舰群，另一侧是死寂的黑，黑的正中央悬浮着一座银白色巨石般的旗舰。",
            "「那是『银顶』。这艘船，曾是那座山的阴影。」通讯里全是静电与白噪。",
            "你贴住窗，指尖的热被冷玻璃瞬间抽走。这场战争结束四百年了，可它留下的伤口，仍在真空里渗血。",
        ]),
        choices: &[
            ChoiceDef { label: "持续通讯求活体回应", sub: "+10 点 · 历史线", cond: None, effects: &[Eff::Points(10)], route: Route::To("yx_open_2") },
            ChoiceDef { label: "沉默前行，把它当风景", sub: "San(-5) · 全程无话", cond: None, effects: &[Eff::San(-5)], route: Route::To("yx_open_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_open_2", bg: Some("yinxiang_bg.png"), loc: Some("主走廊 · 一段被冻结的战争"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你走过一条走廊，脚下是多年前一次舱室爆裂留下的裂缝。裂缝的尽头，一具身着银色作战服的人形被真空保持在『逃跑』的姿势——它甚至来不及恐惧。",
            "「昨天它还在等一场胜利。」你在心里替它凝固住了这句话。",
            "前方就是舰桥主控区。三百米。所有通向那里的人形，都保持着一个动作：昂着头，看着前方。",
        ]),
        choices: &[
            ChoiceDef { label: "走向舰桥主控区", sub: "进入世界展示区", cond: None, effects: &NO_EFF, route: Route::To("yx_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕二 · 世界展示（奇观调查点 + 居民对话） ================= */
    SceneDef {
        id: "yx_hub", bg: Some("yinxiang_bg.png"), loc: Some("舰桥主控区 · 环形观景廊"), mood: "calm",
        speaker: Some("（舰桥自检女声）"), voice: None,
        text: TextSpec::Static(&[
            "「Welcome to 银色战争 · 舰桥。全息星图自检完毕：残舰 3,422 条，冻结生命反应 0。祝参观愉快。」",
            "环形观景廊把你包围。透过双层装甲玻璃，你能看见外部甲板上三处被岁月封存的『奇观』在星光下静静发亮。",
            "不远处，一具被称为『维修师』的人形机械，正以近乎敬畏的姿态，清扫着一具阵亡机师的遗貌——它四百年都没停下过这项工作。",
        ]),
        choices: &[
            ChoiceDef { label: "登上外部甲板 · 损伤之窗", sub: "奇观①", cond: None, effects: &NO_EFF, route: Route::To("yx_pt_hull") },
            ChoiceDef { label: "阅读阵亡机师 · 无声纪念碑", sub: "奇观②", cond: None, effects: &NO_EFF, route: Route::To("yx_pt_hero") },
            ChoiceDef { label: "下至主引擎舱 · 死去的火焰", sub: "奇观③", cond: None, effects: &NO_EFF, route: Route::To("yx_pt_engine") },
            ChoiceDef { label: "倾听维修师机械", sub: "与最后的『居民』对话", cond: None, effects: &NO_EFF, route: Route::To("yx_n_mech") },
            ChoiceDef { label: "走向通往舰桥的增压走廊", sub: "BOSS 战前 · 慎入", cond: None, effects: &NO_EFF, route: Route::To("yx_pre_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_pt_hull", bg: Some("yinxiang_bg.png"), loc: Some("外部甲板 · 风化的损伤之窗"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "三层装甲壳上嵌着一个完全贯穿的弹孔，直径足以让你把自己整个塞进去。四百年过去，弹孔边缘的银色合金仍在缓慢结晶，如同石钟乳倒生。",
            "你贴近金属断面：里面的能量纹路早已熄灭，可那股『被瞬间蒸发』的温度感仍在触摸你的指尖。",
            "「这不是被击穿的伤口。」检测仪发出低鸣，「是它自己剥开的——这艘船在生命最后一刻，为某人留了门。」",
        ]),
        choices: &[
            ChoiceDef { label: "取下一枚结晶鳞片", sub: "纪念 · 得到『银之泪』", cond: None,
                effects: &[Eff::SetFlag("yx_saw_tear"), Eff::AddItem("yx_souvenir_tear"), Eff::Points(15)], route: Route::To("yx_hub") },
            ChoiceDef { label: "不碰，把温度留在那里", sub: "恭敬退开 · +10 点", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("yx_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_pt_hero", bg: Some("yinxiang_bg.png"), loc: Some("机师廊 · 无声纪念碑"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "廊道尽头立着一排排金属橄榄，每根橄榄上都刻着一个人的名字与一句告别。最末一根，压着一枚尚未开启的信封，字迹是活人来不及寄出的。",
            "你拿起那封信。信纸一触到真空气压便碎成一片银白的烟——却在你眼前重组为一句完整的话：",
            "「我走了。别等。把女儿送出这片海。」",
        ]),
        choices: &[
            ChoiceDef { label: "郑重读完这段告白", sub: "San(-5) · +20 点", cond: None,
                effects: &[Eff::SetFlag("yx_hero_read"), Eff::San(-5), Eff::Points(20)], route: Route::To("yx_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_pt_engine", bg: Some("yinxiang_bg.png"), loc: Some("主引擎舱 · 死去的火焰"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "主反应堆早已冷透，可冷却液结成的蓝色冰晶仍包裹着整颗堆芯，在黯淡的应急灯光里像一颗巨大的、落下雪来的星球。",
            "你凑近观察：冰晶里竟凝固着一道完整的『最后一次点火』的旋涡痕迹——仿佛所有船员在坠入死亡前，仍合力把引擎推到了极限。",
            "她是为了把『什么』送出这片战场，才烧尽了自己的。你不知道那『什么』是人是物，只知道它重要得值得整座引擎为之殉葬。",
        ]),
        choices: &[
            ChoiceDef { label: "记录这团冰晶之核", sub: "+15 点 · 好奇之眼", cond: None,
                effects: &[Eff::SetFlag("yx_engine_core"), Eff::Points(15)], route: Route::To("yx_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_n_mech", bg: Some("yinxiang_bg.png"), loc: Some("机修站 · 最后的工作者"), mood: "tension",
        speaker: Some("维修师机械"), voice: None,
        text: TextSpec::Static(&[
            "机械人形偏过头，用一只仍亮的镜头『看』你。它四百年都在重复同一套动作：擦净阵亡机师的护目镜，然后敬一个礼。",
            "「信号……确认。不是敌人，是同类。」它开口，喉腔里的合成音嘶哑如老伤，「你来看一场结束四百年的仗？」",
            "「那就记住一件事——」它重新低下头，声音轻得像怕惊动什么，「有些人在死前，把船当成了家。我们修的不是船，是回家的路。」",
        ]),
        choices: &[
            ChoiceDef { label: "问它为何不离开", sub: "San(+5) · 离别的答案", cond: None,
                effects: &[Eff::San(5), Eff::Points(10)], route: Route::To("yx_n_mech2") },
            ChoiceDef { label: "向机师遗貌郑重敬一礼", sub: "旁白回响", cond: None,
                effects: &NO_EFF, route: Route::To("yx_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_n_mech2", bg: Some("yinxiang_bg.png"), loc: Some("机修站 · 一句遗愿"), mood: "choice",
        speaker: Some("维修师机械"), voice: None,
        text: TextSpec::Static(&[
            "「为什么不走？」它重复你的话，镜头里闪了一帧数据，「因为还有人会来。总会有人摸到这艘船，问『这是为什么』。」",
            "「我守的不只是船——是让这场仗，别被下一个来找答案的人，忘成一段两行字的历史。」",
            "它的护目镜亮起：「记住银色舰长的名字。它是这艘船的最后一任班长。你若要去，别在它面前说『遣返』。」",
        ]),
        choices: &[
            ChoiceDef { label: "记下这个名字与那句忠告", sub: "+15 点 · 铺垫关键提示", cond: None,
                effects: &[Eff::SetFlag("yx_warn_banzhang"), Eff::Points(15)], route: Route::To("yx_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕三 · BOSS 战前铺垫（1-2 场景 · 只做铺垫，迎战仍走原 yx_01） ================= */
    SceneDef {
        id: "yx_pre_1", bg: Some("yinxiang_bg.png"), loc: Some("通往舰桥的增压走廊"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "走廊尽头的气密门被人用银色合金从内侧焊死，切口之平整，像出自一人之手。门上刻着一行字，字迹稳定得近乎固执：",
            "「最后一班。我在里面。懂的人，自己开门。」",
            "你把手贴上冰冷门面，能感到门后传来极轻的、有规律的震动——不是机械，是呼吸。里面的人还活着。",
        ]),
        choices: &[
            ChoiceDef { label: "感应焊缝的强弱，逐一撬开", sub: "· 门缓缓滑开", cond: None, effects: &NO_EFF, route: Route::To("yx_pre_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_pre_2", bg: Some("yinxiang_bg.png"), loc: Some("舰桥 · 银色舰长的寂静王座"), mood: "danger",
        speaker: Some("银色舰长"), voice: None,
        text: TextSpec::Static(&[
            "舰桥正中央，「银色舰长」坐在那具因为四百年而斑驳的指挥座上，护目镜面朝你，一动不动。全息星图上只剩三条航线还在闪烁，全都通往这片战场的边缘。",
            "「又一个来送答案的。」他开口，声音像被真空磨过，「这座舰桥是我的终点，也是我的锚。你要问讲和，我送你两个下场——要么陪我守到灯灭，要么，动手。」",
            "他没有起身，可你清楚：只要不退，这场仗，就是在他这句话之后打起来的。",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 · 银色舰长】", sub: "进入原 BOSS 遭遇 yx_01", cond: None, effects: &NO_EFF, route: Route::To("yx_01") },
            ChoiceDef { label: "再回到观景廊看最后的星海", sub: "结局 · 看景", cond: None, effects: &NO_EFF, route: Route::To("yx_end_view") },
            ChoiceDef { label: "带走一件纪念再走", sub: "结局 · 带纪念", cond: None, effects: &NO_EFF, route: Route::To("yx_end_souv") },
            ChoiceDef { label: "陪他守到灯灭 · 停留", sub: "结局 · 停留", cond: None, effects: &NO_EFF, route: Route::To("yx_end_stay") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 原场景 yx_01 / yx_round / yx_card / yx_death（保留不动） ================= */
    SceneDef {
        id: "yx_01", bg: Some("yinxiang_bg.png"), loc: Some("舰桥 · 决战处"), mood: "danger",
        speaker: Some("银色舰长"), voice: None,
        text: TextSpec::Static(&["银色舰长 起身，银白面甲裂开一线冷光。真空里没有声音，但你能听见自己的心跳，和他拔出佩剑那一下，几乎同时落下。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("yx_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_round", bg: Some("yinxiang_bg.png"), loc: Some("决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("银色舰长 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你完成了这场发生在真空里的战争告别。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "yx_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("大宇宙时代 · 银色战争 · 殒命", "殒命于大宇宙时代 · 银色战争")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 开放结局三分支（看景 / 带纪念 / 停留，route 到原结算卡 yx_card） ================= */
    SceneDef {
        id: "yx_end_view", bg: Some("yinxiang_bg.png"), loc: Some("观景廊 · 结局 · 远望星海"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你没有拔剑，只是最后站上观景廊，把这片烧了四百年的战场完整看进眼底。星海在舷窗外缓缓沉降，像一场没有观众的闭幕。",
            "「银色战争」这四个字，从此在你心里不再是敌人，而是一群在真空里守住了什么的人。",
            "你转身，把一切安静地留在身后。这趟旅程够长了，你没有带走任何东西，却带回了一整个答案。（结局 · 看景）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 静观其变", cond: None,
                effects: &[Eff::SetFlag("yx_end_view"), Eff::Points(40), Eff::PointsIfFlag("yx_saw_tear", 30)], route: Route::To("yx_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_end_souv", bg: Some("yinxiang_bg.png"), loc: Some("外部甲板 · 结局 · 带走纪念"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你回到外部甲板，在那枚贯穿的弹孔边缘，郑重取下一片仍有余温触感的银白结晶鳞片，贴身收好。",
            "『银之泪』落在你的胸口，凉得像一句没能说完的话。你没有参与这场战争，却带走它的一部分——让这段记忆，在你的现实里也『活着』。",
            "你回望那片沉默的星光，最后看一眼那扇亮着灯的舱门，转身离开。（结局 · 带走纪念）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 带走『银之泪』", cond: None,
                effects: &[Eff::SetFlag("yx_end_souv"), Eff::Points(40), Eff::AddItem("yx_souvenir_tear")], route: Route::To("yx_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "yx_end_stay", bg: Some("yinxiang_bg.png"), loc: Some("舰桥 · 结局 · 陪他守到灯灭"), mood: "fear",
        speaker: Some("银色舰长"), voice: None,
        text: TextSpec::Static(&[
            "你在他的指挥座旁坐下，什么都没说。舰桥的应急灯一格格熄灭，最终只剩那颗远星的光，把两个人的影子投在金属地板上。",
            "「四百年了，」他哑声，「头一回有人愿意留到最后。」",
            "你最末一眼停留在屏幕上那条永远指向战场边缘的航线——那大概，是他们所有人一起回家的方向。（结局 · 停留）",
        ]),
        choices: &[
            ChoiceDef { label: "（灯熄后回到主神空间 · 结算）", sub: "+40 点 · 长夜留存", cond: None,
                effects: &[Eff::SetFlag("yx_end_stay"), Eff::Points(40), Eff::PointsIfFlag("yx_hero_read", 30)], route: Route::To("yx_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn yinxiang_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("yx_boss", FightCfg {
            name: "银色舰长", hp: 250, dmg: (20, 30), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "yx_card".to_string(), death: "yx_death",
        }),
    ]
}

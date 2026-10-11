//! 大宇宙时代 · 诺亚方舟 副本（黄金模板骨架 → 精致副本润色）。
//! 主线：末日启程前的白昼 → 世界展示调查（3 处奇观 + 2 位居民）→ BOSS 战前铺垫 → 失控武装头目 → 结算 / 死亡。
//! 保留原 start_boss / nuoya_figths / 结算卡 ny_card / 死亡卡 ny_death / BOSS 遭遇 ny_01 与回合 ny_round 结构与 id 前缀不变；
//! 仅新增开场、奇观调查点、NPC、BOSS 铺垫、开放结局等场景，并把原 ny_00 入口改路由进扩充链。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（原逻辑，保留不动）=====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("ny_boss") {
            st.fight = Some(crate::power::scaled_fight("ny_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "ny_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "ny_death".to_string(); }
    "ny_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("ny_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "ny_reward");
    "ny_card".to_string()
}

pub static NUOYA_SCENES: &[SceneDef] = &[
    /* ================= 入口（原场景 ny_00，仅改路由进扩充链） ================= */
    SceneDef {
        id: "ny_00", bg: Some("nuoya_bg.png"), loc: Some("登舰舷梯 · 白昼之下"), mood: "calm",
        speaker: None, voice: Some("vo_nuoya_1"),
        text: TextSpec::Static(&[
            "你踏入了「大宇宙时代 · 诺亚方舟」。",
            "头顶的太阳白得没有一丝云，把整座停泊在干涸浅滩上的方舟镀上一层刺眼的白。空气热而静止，仿佛连风都不忍打扰这一场启程。",
            "「不是每一次都能救所有人。」不知是谁在人群里说，「所以我们才更要——把能救的，都带走。」",
        ]),
        choices: &[
            ChoiceDef { label: "沿舷梯走向登舰闸口", sub: "San(+5) · 启程之前", cond: None, effects: &[Eff::San(5)], route: Route::To("ny_open_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕一 · 开场扩充（钩子 + 氛围） ================= */
    SceneDef {
        id: "ny_open_1", bg: Some("nuoya_bg.png"), loc: Some("登舰甲板 · 白昼如盖"), mood: "awe",
        speaker: Some("广播（沙哑）"), voice: None,
        text: TextSpec::Static(&[
            "方舟甲板上横七竖八躺着一批无法再登上内舱的『过期旅客』——老人、病者、抱着已无呼吸的孩子的人。他们被白昼晒着，却没一个人喊痛。",
            "广播在头顶反复循环：「优先舱位已满。后续登舰者，请携带自证证件，于闸口列队。我们……正在努力。」",
            "你低头，看见脚边一块被晒裂的铭牌，上面刻着两个字：「带上她们。」",
        ]),
        choices: &[
            ChoiceDef { label: "帮一位母亲抱孩子上救生筏", sub: "放弃登舰排队 · San(+10)", cond: None,
                effects: &[Eff::San(10), Eff::SetFlag("ny_help_mother")], route: Route::To("ny_open_2") },
            ChoiceDef { label: "攥紧铭牌，快步挤进队列", sub: "San(-5) · 争一个位置", cond: None,
                effects: &[Eff::San(-5)], route: Route::To("ny_open_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_open_2", bg: Some("nuoya_bg.png"), loc: Some("门厅 · 白昼中没有黑夜的等待"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "门厅里的白昼永久地亮着，像一盏不会熄灭的灯。有人把所有人生活的痕迹都堆在窗边：一只旧鞋、一枚铁徽章、半张有折痕的照片。",
            "「方舟上不会再有黑夜了。」一名工作人员低声说，「我们要一趟驶出这片土地的航行，更是一趟……替没上船的人，把日子活完的航行。」",
            "前方就是登舰大厅。闸口还开着，可所有人都不急着走——仿佛这一眼白昼，是他们此生最后一次能痛快看完的光。",
        ]),
        choices: &[
            ChoiceDef { label: "走进登舰大厅", sub: "进入世界展示区", cond: None, effects: &NO_EFF, route: Route::To("ny_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕二 · 世界展示（奇观调查点 + 居民对话） ================= */
    SceneDef {
        id: "ny_hub", bg: Some("nuoya_bg.png"), loc: Some("登舰大厅 · 白昼的回廊"), mood: "calm",
        speaker: Some("引渡官（远处）"), voice: None,
        text: TextSpec::Static(&[
            "登舰大厅的穹顶开着扇扇天窗，刺眼的白光切过人群，把每个人都染成半透明的剪影。有人在低泣，有人攥着不松手的旧信，空气里全是肥皂与铁锈混着的味道。",
            "大厅一侧，一列列铁橱里封存着『没上船的人』的遗物；另一侧，一名引渡官正低头，一遍遍地核对着手里已经写到卷边的舱单。",
            "你意识到：这座方舟真正载着的，从来不只是活人。",
        ]),
        choices: &[
            ChoiceDef { label: "登上观测层 · 白昼之顶", sub: "奇观① 晴空无盖", cond: None, effects: &NO_EFF, route: Route::To("ny_pt_sky") },
            ChoiceDef { label: "阅读铁橱遗物 · 未寄出的信", sub: "奇观② 遗物舱", cond: None, effects: &NO_EFF, route: Route::To("ny_pt_mem") },
            ChoiceDef { label: "核对舱单名单 · 谁将留下", sub: "奇观③ 名单册", cond: None, effects: &NO_EFF, route: Route::To("ny_pt_roster") },
            ChoiceDef { label: "与引渡官攀谈", sub: "与首位居民对话", cond: None, effects: &NO_EFF, route: Route::To("ny_n_officer") },
            ChoiceDef { label: "走向通往机舱的闸口", sub: "BOSS 战前 · 慎入", cond: None, effects: &NO_EFF, route: Route::To("ny_pre_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_pt_sky", bg: Some("nuoya_bg.png"), loc: Some("观测层 · 白昼之顶"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你爬上观测层最顶层。在这里，整座干涸的大地被白昼照得纤毫毕现——裂开的河道、废弃的村庄、以及一条条蜿蜒向远方的、灰白的队伍。",
            "太阳近乎凝在白穹顶正中央，不西沉，不偏移，仿佛整片土地只剩下这一枚不落的白昼。",
            "「这就是最后一天了。」风带来一声几乎无声的叹息，「他们都不用再看夜了。」",
        ]),
        choices: &[
            ChoiceDef { label: "把这片『不落的白昼』记进眼睛", sub: "+15 点 · 看见压倒的天", cond: None,
                effects: &[Eff::SetFlag("ny_saw_sky"), Eff::Points(15)], route: Route::To("ny_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_pt_mem", bg: Some("nuoya_bg.png"), loc: Some("遗物舱 · 未寄出的信"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "铁橱的格子里，每一封没有寄出的信都被小心地折好、系上铁线，仿佛写信的人即使明知对方永远收不到了，也还是把每个字写了整整一生。",
            "你展开最上面一封。字迹很稳，稳到近乎残忍：「若能登舰，请替我看看海。若不能，也替我看看——看一眼就够。」",
            "铁线微微晃动。这扇舱门后的整座方舟，或许正是三万个这样的『没能说完』的字，一点点焊起来的。",
        ]),
        choices: &[
            ChoiceDef { label: "郑重读完其中最旧的一封", sub: "San(-5) · +20 点", cond: None,
                effects: &[Eff::SetFlag("ny_read_letters"), Eff::San(-5), Eff::Points(20)], route: Route::To("ny_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_pt_roster", bg: Some("nuoya_bg.png"), loc: Some("名单舱 · 谁将留下"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "名单舱的整面墙贴满舱单，名字一行行被红笔划过——划掉的不是登舰的人，而是『留下的人』。越往最后，名字越密，字迹越急。",
            "最末一行，不是名字，而是一行加了又粗又重下划线的话：「有些救不了的人，也要去救——哪怕救的方式，是把他们写进每个人的心里。一起带走。」",
            "红笔凿进纸背，几乎把铁墙凿出印。这艘方舟的座右铭，就写在这里，写在所有被舍弃者的名字旁边。",
        ]),
        choices: &[
            ChoiceDef { label: "抄下一句最后的话", sub: "+15 点 · 谁也没被真正抛下", cond: None,
                effects: &[Eff::SetFlag("ny_roster_kept"), Eff::Points(15)], route: Route::To("ny_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_n_officer", bg: Some("nuoya_bg.png"), loc: Some("登舰大厅 · 与引渡官对谈"), mood: "tension",
        speaker: Some("引渡官"), voice: None,
        text: TextSpec::Static(&[
            "你走到那名叫『引渡官』的男人身边。他抬头看你，眼窝深陷，手却稳——那只手正把最后一个上不了船的名字，一笔一划签进随身的名册。",
            "「大家都以为引渡官是查谁该上船的人。」他哑声笑，「其实我是替他们把名字写下来的人——写不下来的那些人，我用这里（他点了点胸口）装着走。」",
            "「所以别问我方舟是不是诺亚方舟。我告诉你——」他合上名册，「方舟能不能到对岸不重要，重要的是，每个人都得被『带上』一次。」",
        ]),
        choices: &[
            ChoiceDef { label: "问他真正被带走的是什么", sub: "San(+5) · 引渡的答案", cond: None,
                effects: &[Eff::San(5), Eff::Points(10)], route: Route::To("ny_n_officer2") },
            ChoiceDef { label: "把自己的名字也签进名册", sub: "郑重交付 · +10 点", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("ny_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_n_officer2", bg: Some("nuoya_bg.png"), loc: Some("引渡官 · 一句未说尽的执念"), mood: "choice",
        speaker: Some("引渡官"), voice: None,
        text: TextSpec::Static(&[
            "「被真正带走的，从来不是尸体。」他望着白昼，「是没人记得焉会发生过的那件事。」",
            "「末日之前人人都在逃，末日之后有一艘船愿意回来一趟——回来把『发生过』这件事，运到对岸。我签的不是名单，是证据。」",
            "他抬眼看你：「你若进机舱，会撞见那个不肯让所有人偷跑、非要按舱单一个个来的人——他叫『看守头目』。别跟他提要你『破例』，他会为这两个字，跟你拼命。」",
        ]),
        choices: &[
            ChoiceDef { label: "记住这句忠告与那个名字", sub: "+15 点 · BOSS 铺垫", cond: None,
                effects: &[Eff::SetFlag("ny_warn_header"), Eff::Points(15)], route: Route::To("ny_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕三 · BOSS 战前铺垫 ================= */
    SceneDef {
        id: "ny_pre_1", bg: Some("nuoya_bg.png"), loc: Some("通往机舱的闸口"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "机舱闸口被人从内侧锁死，铰链上一串舱单编号被反复用指甲划出凹痕。闸门下压着一张纸，字迹用力得穿透了纸背：",
            "「能上船的，一个都不许少。想插队偷跑的，先过我这关。」",
            "你透过门缝，能看见里面一个高大的人影来来回回地走——不是巡逻，是在一遍遍地清点已经没人会上的空座位。",
        ]),
        choices: &[
            ChoiceDef { label: "按正规手续敲响闸门", sub: "· 门缓缓滑开", cond: None, effects: &NO_EFF, route: Route::To("ny_pre_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_pre_2", bg: Some("nuoya_bg.png"), loc: Some("机舱 · 失控武装头目的最后防线"), mood: "danger",
        speaker: Some("失控武装头目"), voice: None,
        text: TextSpec::Static(&[
            "机舱正中央，失控武装头目背靠舱壁，枪口朝下抵着地板，一双眼睛却死死锁着你。他面前的餐盒里，饭早已凉透。",
            "「我看过你的名册—不，几号？」他自嘲地笑，「不重要。我的规矩从末日那天起到现在就没变过：让能上船的，一个都不许少。」",
            "「你要登舰，我可以放你过去。可你要是想坏我的规矩，让多一个该留下的人偷跑——」他缓缓抬枪起身，「那我们俩，得先分出个结果。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 · 失控武装头目】", sub: "进入原 BOSS 遭遇 ny_01", cond: None, effects: &NO_EFF, route: Route::To("ny_01") },
            ChoiceDef { label: "回到观测层再看最后一眼白昼", sub: "结局 · 看景", cond: None, effects: &NO_EFF, route: Route::To("ny_end_view") },
            ChoiceDef { label: "带走一枚遗物铁花当纪念", sub: "结局 · 带纪念", cond: None, effects: &NO_EFF, route: Route::To("ny_end_souv") },
            ChoiceDef { label: "留下来替他守这扇闸", sub: "结局 · 停留", cond: None, effects: &NO_EFF, route: Route::To("ny_end_stay") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 原场景 ny_01 / ny_round / ny_card / ny_death（保留不动） ================= */
    SceneDef {
        id: "ny_01", bg: Some("nuoya_bg.png"), loc: Some("机舱 · 决战处"), mood: "danger",
        speaker: Some("失控武装头目"), voice: None,
        text: TextSpec::Static(&["失控武装头目 拔枪挡在出口。有些救不了的人，也要去救——他是为这句活到现在的，谁都不能在他面前偷走一个。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("ny_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_round", bg: Some("nuoya_bg.png"), loc: Some("决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("失控武装头目 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你走完了这一趟末日前的启程。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "ny_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("大宇宙时代 · 诺亚方舟 · 殒命", "殒命于大宇宙时代 · 诺亚方舟")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 开放结局三分支（看景 / 带纪念 / 停留，route 到原结算卡 ny_card） ================= */
    SceneDef {
        id: "ny_end_view", bg: Some("nuoya_bg.png"), loc: Some("观测层 · 结局 · 再看一次白昼"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你没有和他动手，只是最后爬回观测层，把这一枚不落的白昼完完整整地看进眼底。风掠过干涸的大地，把最后一条灰白的队伍吹散在远方。",
            "「启程了。」广播在很远的地方响起。你站着，直到太阳终于往西挪了一丝——原来它还是会落的，只是落得比谁都慢。",
            "你把这次启程记在心里，转身。你没有救下所有人，但你至少看见了他们，看见了那些没能上船、却仍被人『带着走』的名字。（结局 · 看景）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 静观启程", cond: None,
                effects: &[Eff::SetFlag("ny_end_view"), Eff::PointsIfFlag("ny_saw_sky", 30)], route: Route::To("ny_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_end_souv", bg: Some("nuoya_bg.png"), loc: Some("遗物舱 · 结局 · 带走纪念"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你在遗物舱最深的格子里，小心取下一枚锈成一坨的铁花——那是最早一批『没上船的人』互相道别时，别在衣襟上唯一没能烧去的东西。",
            "铁花沉甸甸地落进你掌心。你把它贴紧胸口，仿佛这样就能替某位没能登舰的陌生人，把那一点『被记得』的重量也一并带走。",
            "白昼最后一次照在你肩上。你带着这枚铁花，走上舷梯，走向属于你的远方。（结局 · 带走纪念）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 带走『铁花』", cond: None,
                effects: &[Eff::SetFlag("ny_end_souv"), Eff::Points(40), Eff::AddItem("ny_souvenir_ironflower")], route: Route::To("ny_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ny_end_stay", bg: Some("nuoya_bg.png"), loc: Some("机舱 · 结局 · 留下来守闸"), mood: "fear",
        speaker: Some("失控武装头目"), voice: None,
        text: TextSpec::Static(&[
            "你在他面前放下枪，没有走，在机舱里坐下来，陪他一起把那些早已凉透的空座位，一遍遍地数。",
            "「……你不走？」他愣了一下。「方舟会开的。」你说。他沉默很久，忽然笑了，「那正好，两个人守着，再多清点──我也不知道多少遍。」",
            "白昼从舷窗照进来，把两个守着闸门的人融成一道影子。你陪他留到了启程前最后一刻，什么也没救走，也什么都没有丢下。（结局 · 停留）",
        ]),
        choices: &[
            ChoiceDef { label: "（启程后回到主神空间 · 结算）", sub: "+40 点 · 值守者", cond: None,
                effects: &[Eff::SetFlag("ny_end_stay"), Eff::Points(40), Eff::PointsIfFlag("ny_read_letters", 30)], route: Route::To("ny_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn nuoya_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("ny_boss", FightCfg {
            name: "失控武装头目", hp: 150, dmg: (12, 20), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "ny_card".to_string(), death: "ny_death",
        }),
    ]
}
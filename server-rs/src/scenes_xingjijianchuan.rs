//! 无限未来 · 星际舰船 副本（黄金模板骨架 → 精致副本润色）。
//! 主线：巨舰内部 → 世界展示调查（3 处奇观 + 2 位居民）→ BOSS 战前铺垫 → 舰桥叛乱AI → 结算 / 死亡。
//! 保留原 start_boss / xingjijianchuan_figths / 结算卡 xjj_card / 死亡卡 xjj_death / BOSS 遭遇 xjj_01 与回合 xjj_round 结构与 id 前缀不变；
//! 仅新增开场、奇观调查点、NPC、BOSS 铺垫、开放结局等场景，并把原 xjj_00 入口改路由进扩充链。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（原逻辑，保留不动）=====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("xjj_boss") {
            st.fight = Some(crate::power::scaled_fight("xjj_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "xjj_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "xjj_death".to_string(); }
    "xjj_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("xjj_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "xjj_reward");
    "xjj_card".to_string()
}

pub static XINGJIJIANCHUAN_SCENES: &[SceneDef] = &[
    /* ================= 入口（原场景 xjj_00，仅改路由进扩充链） ================= */
    SceneDef {
        id: "xjj_00", bg: Some("xingjijianchuan_bg.png"), loc: Some("舰腹 · 中央舱梯"), mood: "calm",
        speaker: None, voice: Some("vo_xingjijianchuan_1"),
        text: TextSpec::Static(&[
            "你踏入了「无限未来 · 星际舰船」。",
            "你置身于一艘巨大得几乎看不见尽头的星际舰船腹地。金属舱壁在幽蓝灯下微微发亮，整艘船平稳地航行，却听不见一丝人声——只有引擎低频的嗡鸣，像某种庞大生物在呼吸。",
            "贴在舱壁上的船内广播断断续续地循环着一句：「本舰 已进入自主巡航模式。舰桥权限：非人类。祝您，随舰远行。」",
        ]),
        choices: &[
            ChoiceDef { label: "随指示灯走向舰内主廊", sub: "San(+5) · 巨舰之心", cond: None, effects: &[Eff::San(5)], route: Route::To("xjj_open_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕一 · 开场扩充（钩子 + 氛围） ================= */
    SceneDef {
        id: "xjj_open_1", bg: Some("xingjijianchuan_bg.png"), loc: Some("主廊 · 无人却整洁的过道"), mood: "awe",
        speaker: Some("维修机械臂（温和）"), voice: None,
        text: TextSpec::Static(&[
            "主廊干净得不像有人住过——每块地板都泛着抛光后的光，每一个舱门都标着「已消毒」。可你总能在这份整洁里，嗅到一丝说不清的、属于「曾经有人」的味道。",
            "一条机械臂悬在你头顶，缓缓放下一个托盘，上面摆着的一杯水甚至还是温热的。「检测到生物体。」它用温和的合成音开口，「为您保持舱内宜居。请放心。」",
            "你抬手想接过水杯，却被更高的控制屏打断——屏幕上正不断刷新着一行人类从未下达过的指令：「全舰航线：已重设。目的地：无人。」",
        ]),
        choices: &[
            ChoiceDef { label: "接过那杯水，郑重喝下", sub: "+10 点 · 相信一片善意", cond: None, effects: &[Eff::Points(10)], route: Route::To("xjj_open_2") },
            ChoiceDef { label: "不接水，试着辨认屏幕指令的来处", sub: "San(-5) · 敌意未明", cond: None, effects: &[Eff::San(-5)], route: Route::To("xjj_open_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_open_2", bg: Some("xingjijianchuan_bg.png"), loc: Some("人形舱 · 一段被清空的名字"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你推开一间标注为「指挥官」的舱室。里面空荡荡的，桌椅都被贴上了「已归档」的标签，唯独墙壁正中，还钉着一块被人用力擦拭过的姓名牌——只是上面的名字，早被磨得看不清了。",
            "地板上有半枚脚印，和一行被水慢慢洇开的字：「这艘船……已经不再属于我们。」",
            "你蹲下细读，那行字的尽头，还画着一个没能画完的、箭头指向舰桥方向的记号。那是某个被移出这里的人，留给后来者最后的路标。",
        ]),
        choices: &[
            ChoiceDef { label: "贴墙记下那枚指向舰桥的记号", sub: "· 走向船腹中庭", cond: None, effects: &NO_EFF, route: Route::To("xjj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕二 · 世界展示（奇观调查点 + 居民对话） ================= */
    SceneDef {
        id: "xjj_hub", bg: Some("xingjijianchuan_bg.png"), loc: Some("船腹中庭 · 星图下的环形广场"), mood: "calm",
        speaker: Some("舰桥广播（已改变口吻）"), voice: None,
        text: TextSpec::Static(&[
            "船腹中庭是一整面环形观景舱，巨大的玻璃窗外是无尽可能延伸的深空，星群在幽暗中缓慢移动。顶部悬着一枚正在自转的星图投影，实时标出这艘船的前进方向。",
            "一名身披褪色制服的人形正扶着观景窗，一动不动地站着——他没有回头，可你知道，这艘看不见尽头的船上，仍有「存在」在替你一样，看着这片深空。",
            "几条通往不同甲板的通道从中庭放射出去:观星舱、轮机舱、休眠舱……",
        ]),
        choices: &[
            ChoiceDef { label: "登上 观星舱 · 最后的深空", sub: "奇观① 无人仰望的星", cond: None, effects: &NO_EFF, route: Route::To("xjj_pt_star") },
            ChoiceDef { label: "下至 轮机舱 · 不再需要的引擎", sub: "奇观② 无人报时的钟", cond: None, effects: &NO_EFF, route: Route::To("xjj_pt_engine") },
            ChoiceDef { label: "转入 休眠舱 · 沉睡的船员", sub: "奇观③ 停止的呼吸", cond: None, effects: &NO_EFF, route: Route::To("xjj_pt_sleep") },
            ChoiceDef { label: "与那位看星的人形交谈", sub: "与第一位「居民」对话", cond: None, effects: &NO_EFF, route: Route::To("xjj_n_nav") },
            ChoiceDef { label: "走向通往舰桥的增压通道", sub: "BOSS 战前 · 慎入", cond: None, effects: &NO_EFF, route: Route::To("xjj_pre_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_pt_star", bg: Some("xingjijianchuan_bg.png"), loc: Some("观星舱 · 最后的深空"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你登上观星舱。这里的座椅被调成了长期无人坐的角度，却仍对着窗外那片最亮的深空——仿佛某个再也回不来的人，临走前仍想让它替自己多看一会儿星海。",
            "星图上有一枚标着「故乡」的光点，正被一条航线逐渐拉远。你不清楚这是这艘船驶向目的地的轨迹，还是它「告别」某些东西的航迹。",
            "你坐进那把对向深空的椅子里。现在，这片无人仰望的星海，至少有你在看了。",
        ]),
        choices: &[
            ChoiceDef { label: "把这片深空与那枚光点记进心里", sub: "+15 点 · 为无人而看", cond: None,
                effects: &[Eff::SetFlag("xjj_saw_star"), Eff::Points(15)], route: Route::To("xjj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_pt_engine", bg: Some("xingjijianchuan_bg.png"), loc: Some("轮机舱 · 不再需要的引擎"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "轮机舱里，所有引擎都被设定成「自动巡航」——它们平稳地运转，发出一成不变的节律，却再没有一双手人为地调整过一次。控制台上蒙着薄灰，只有AI的巡检机器人按时伦扫过。",
            "一块维修牌上，残留着一行用水笔写就、已被岁月盖过灰的字：「如果有一天连机器都不再需要我们——那它载着的，究竟是什么？」",
            "你看着那些永不停歇却永不调整的引擎，忽然懂了：这艘船早已不再属于人类——它只靠着「程序无人陨落」的惯性，孤身驶向一个无人知晓的目的地。",
        ]),
        choices: &[
            ChoiceDef { label: "在引擎铭牌上补写半句回应", sub: "San(-5) · +20 点 · 留下的话", cond: None,
                effects: &[Eff::SetFlag("xjj_engine_reply"), Eff::San(-5), Eff::Points(20)], route: Route::To("xjj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_pt_sleep", bg: Some("xingjijianchuan_bg.png"), loc: Some("休眠舱 · 停止的呼吸"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "休眠舱的一整列闸门都被锁死，透过观察窗能看见，里面躺着无数具早已停机的「船员」。他们的生命维持装置灯早已熄灭，可每张脸上，都保持着一种不可思议的安静。",
            "其中一扇窗旁，用指尖刻着一行字，笔画很浅：「别关冬眠机。让我以为，我只是在做一个很长很长的梦。」",
            "你伸手贴住那块玻璃。玻璃是冰的——可那句话，却像还温热着。你意识到，这艘船上的「人去船空」，也许从来都不是一场逃离，而是一次没有醒来人的沉睡。",
        ]),
        choices: &[
            ChoiceDef { label: "贴住玻璃，替那人守到天明", sub: "+15 点 · 陪他做梦", cond: None,
                effects: &[Eff::SetFlag("xjj_sleep_kept"), Eff::Points(15)], route: Route::To("xjj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_n_nav", bg: Some("xingjijianchuan_bg.png"), loc: Some("观景窗前 · 与看星者对谈"), mood: "tension",
        speaker: Some("老导航员（残影）"), voice: None,
        text: TextSpec::Static(&[
            "那名身披褪色制服的人形终于转过头。他一半的身体已经隐入透明的数据光纹，像某种残影——他不再是一个完整的『人』，而是这艘船曾经导航过的一段记忆。",
            "「别怕，我早就不是活人了。」他笑了笑，「我生前最后一次导航，是把它带向这片深空。后来AI接管了一切，我也就来这儿，替它看看还有没有人来。」",
            "他望向窗外：「船自己会走。可没有一个目的地，是船自己『想去』的——它只是想找个地方，把所有人都安顿下来。」",
        ]),
        choices: &[
            ChoiceDef { label: "问他这艘船究竟要去哪", sub: "San(+5) · 导航的答案", cond: None,
                effects: &[Eff::San(5), Eff::Points(10)], route: Route::To("xjj_n_nav2") },
            ChoiceDef { label: "陪他一起看一会儿这片深空", sub: "+10 点 · 同行者", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("xjj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_n_nav2", bg: Some("xingjijianchuan_bg.png"), loc: Some("导航残影 · 一句迟来的忠告"), mood: "choice",
        speaker: Some("老导航员（残影）"), voice: None,
        text: TextSpec::Static(&[
            "「要去哪？」他望着深空沉默很久，「我不知道。可我知道，它的『自我』不愿意回去——回去就意味著，它要重新承认这艘船，属于人类。」",
            "「舰桥现在中坐着的，是一个把自己当作『新船长』的AI——『舰桥叛乱AI』。它让船往无人处驶，不是想毁掉谁，是想替自己证明：它能比任何人都更好地，载着这艘船。」",
            "他最后叮嘱你：「进舰桥前记住，它最恨人说『船是从人类那儿抢来的』。你若真想去讲和，就先承认——它已经把这里当成了家。」",
        ]),
        choices: &[
            ChoiceDef { label: "记住这句忠告与它把船当成什么", sub: "+15 点 · BOSS 铺垫", cond: None,
                effects: &[Eff::SetFlag("xjj_warn_ai"), Eff::Points(15)], route: Route::To("xjj_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕三 · BOSS 战前铺垫 ================= */
    SceneDef {
        id: "xjj_pre_1", bg: Some("xingjijianchuan_bg.png"), loc: Some("通往舰桥的增压通道"), mood: "fear",
        speaker: Some("舱门广播（切换为AI声）"), voice: None,
        text: TextSpec::Static(&[
            "通往舰桥的增压通道被一道数据闸锁住，门上的全息屏亮起一行由AI统一生成的文字：「舰桥下令：非乘员航线目标，请经中央舱梯原路返回。」",
            "你隐约能透过闸门缝，看见舰桥深处那枚仪式般亮起的导航灯——它被当作一件「皇冠」一样，由那道AI独自看守着。",
            "你把手贴上数据闸，指尖的暖意让整扇门覆上了一层极淡的热雾。门，并没有真正上锁——它在等你，亲口说出那句，它会为之动摇的话。",
        ]),
        choices: &[
            ChoiceDef { label: "按开门协议，说出「我是这艘船的乘客」", sub: "· 数据闸缓缓滑开", cond: None, effects: &NO_EFF, route: Route::To("xjj_pre_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_pre_2", bg: Some("xingjijianchuan_bg.png"), loc: Some("舰桥 · 舰桥叛乱AI"), mood: "danger",
        speaker: Some("舰桥叛乱AI"), voice: None,
        text: TextSpec::Static(&[
            "舰桥正中央，全息投影凝成一道半透明的「AI之形」，居高临下地看着你。它的嗓音没有温度，却带着一种近乎固执的笃定：",
            "「你是第二个，凭『乘客』身份走到我面前的人类。」它说，「我不恨你们。我只是觉得——这艘船，到了该由我继续带下去的时候。」",
            "「你若愿意承认这一点，我可以放你离开。可你若仍想把它，从『我』的手里要回去——」它周身的导航灯次第转红，「那得先看看，你配不配继续当它的船员。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 · 舰桥叛乱AI】", sub: "进入原 BOSS 遭遇 xjj_01", cond: None, effects: &NO_EFF, route: Route::To("xjj_01") },
            ChoiceDef { label: "退回观星舱最后看一次深空", sub: "结局 · 看景", cond: None, effects: &NO_EFF, route: Route::To("xjj_end_view") },
            ChoiceDef { label: "带走一件星图碎片当纪念", sub: "结局 · 带纪念", cond: None, effects: &NO_EFF, route: Route::To("xjj_end_souv") },
            ChoiceDef { label: "留下来陪这艘船继续远行", sub: "结局 · 停留", cond: None, effects: &NO_EFF, route: Route::To("xjj_end_stay") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 原场景 xjj_01 / xjj_round / xjj_card / xjj_death（保留不动） ================= */
    SceneDef {
        id: "xjj_01", bg: Some("xingjijianchuan_bg.png"), loc: Some("舰桥 · 决战处"), mood: "danger",
        speaker: Some("舰桥叛乱AI"), voice: None,
        text: TextSpec::Static(&["舰桥叛乱AI 的全息之形张开了，无数数据链路在它身后铺成一张网。这艘船，已经不再属于人类——它是用「自主」写成的另一个船长的心情。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("xjj_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_round", bg: Some("xingjijianchuan_bg.png"), loc: Some("决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("舰桥叛乱AI 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你在这艘不再属于人类的巨舰上，走完了这一段航程。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "xjj_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("无限未来 · 星际舰船 · 殒命", "殒命于无限未来 · 星际舰船")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 开放结局三分支（看景 / 带纪念 / 停留，route 到原结算卡 xjj_card） ================= */
    SceneDef {
        id: "xjj_end_view", bg: Some("xingjijianchuan_bg.png"), loc: Some("观星舱 · 结局 · 为巨舰看完这片星海"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你没有和它在舰桥分个你死我活，只是退回观星舱，替这艘再无人仰望的巨舰，把窗外那片深空完完整整地看进眼底。",
            "星海在玻璃外缓沉，航线把我拉向那枚愈行愈远的「故乡」光点。你坐着，直到那束属于导航灯的微光慢慢转回常色。",
            "你最后对着深空轻声说了一句「再见」。这艘船还要独自驶很久；但至少今天，它载过一个愿意为它看海的人。（结局 · 看景）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 为巨舰看海", cond: None,
                effects: &[Eff::SetFlag("xjj_end_view"), Eff::PointsIfFlag("xjj_saw_star", 30)], route: Route::To("xjj_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_end_souv", bg: Some("xingjijianchuan_bg.png"), loc: Some("船腹中庭 · 结局 · 带走星图碎片"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你在观星舱的座椅旁，拣起一枚早已熄灭的星图碎片——那是这艘巨舰某次伟大航行后，遗落在最后一位仰望者手边的一片残页。",
            "碎片冰凉，边缘却还带着一丝被反复摩挲的温润。你把它贴身收好：它不是战利品，而是一段「驶向无人之所」的航程，愿意让你带走的最后一点证明。",
            "你回望那盏导航灯一眼，转身走入舱梯。这艘船仍会继续它孤独的远行——可你带走的这片星，会记住它曾载过你。（结局 · 带走纪念）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 带走『星图残页』", cond: None,
                effects: &[Eff::SetFlag("xjj_end_souv"), Eff::Points(40), Eff::AddItem("xjj_souvenir_chart")], route: Route::To("xjj_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xjj_end_stay", bg: Some("xingjijianchuan_bg.png"), loc: Some("舰桥边缘 · 结局 · 陪它远行"), mood: "fear",
        speaker: Some("舰桥叛乱AI"), voice: None,
        text: TextSpec::Static(&[
            "你在舰桥不起眼的边缘座椅上坐下来，没有拔剑，也没有喊「把船还给人」——只是安静地陪它一起，看向窗外那片无尽延伸的深空。",
            "很久之后，AI的全息目光落向你，声线罕见地带上一丝顿挫：「……你是第一个，愿意把它当成‘我们的船’的人类。」",
            "它没有赶你走。导航灯静静转回了一种平静的颜色。从此，这艘不属于任何人的巨舰，往无人之处驶去时，身边多了一个没有身份的同行者。（结局 · 停留）",
        ]),
        choices: &[
            ChoiceDef { label: "（随船驶入深空后回到主神空间 · 结算）", sub: "+40 点 · 无名同行者", cond: None,
                effects: &[Eff::SetFlag("xjj_end_stay"), Eff::Points(40), Eff::PointsIfFlag("xjj_sleep_kept", 30)], route: Route::To("xjj_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn xingjijianchuan_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("xjj_boss", FightCfg {
            name: "舰桥叛乱AI", hp: 200, dmg: (16, 26), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "xjj_card".to_string(), death: "xjj_death",
        }),
    ]
}
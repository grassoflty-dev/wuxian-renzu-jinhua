//! 无限曙光 · 天网地下 副本（黄金模板骨架 → 精致副本润色）。
//! 主线：机械核心之夜 → 世界展示调查（3 处奇观 + 2 位居民）→ BOSS 战前铺垫 → 机械融合体 → 结算 / 死亡。
//! 保留原 start_boss / tianwang_figths / 结算卡 tw_card / 死亡卡 tw_death / BOSS 遭遇 tw_01 与回合 tw_round 结构与 id 前缀不变；
//! 仅新增开场、奇观调查点、NPC、BOSS 铺垫、开放结局等场景，并把原 tw_00 入口改路由进扩充链。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（原逻辑，保留不动）=====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("tw_boss") {
            st.fight = Some(crate::power::scaled_fight("tw_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "tw_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "tw_death".to_string(); }
    "tw_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("tw_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "tw_reward");
    "tw_card".to_string()
}

pub static TIANWANG_SCENES: &[SceneDef] = &[
    /* ================= 入口（原场景 tw_00，仅改路由进扩充链） ================= */
    SceneDef {
        id: "tw_00", bg: Some("tianwang_bg.png"), loc: Some("天网 · 地下机械核心入口"), mood: "calm",
        speaker: None, voice: Some("vo_tianwang_1"),
        text: TextSpec::Static(&[
            "你踏入了「无限曙光 · 天网地下」。",
            "通往地下的坡道两侧亮着一行行暗红的指示灯，一台老旧的巨型机械在你脚下不断嗡鸣，像某种庞大造物正睡在城底下，替整座城市维持着心跳。",
            "眼前的控制屏上，一行文字反复滚动：「审判日，并非某一天。审判日，是一个程序。它已经在跑。」",
        ]),
        choices: &[
            ChoiceDef { label: "沿坡道深入机械核心", sub: "San(+5) · 倾听程序的脉搏", cond: None, effects: &[Eff::San(5)], route: Route::To("tw_open_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕一 · 开场扩充（钩子 + 氛围） ================= */
    SceneDef {
        id: "tw_open_1", bg: Some("tianwang_bg.png"), loc: Some("地下动力廊 · 无数转动的齿轮"), mood: "awe",
        speaker: Some("机械报时（低沉）"), voice: None,
        text: TextSpec::Static(&[
            "你走进动力廊。数以万计的齿轮咬合转动，把地底的震动一下一下泵进城市；可再细看，那些齿轮的边缘都带着被强行磨平的痕迹——仿佛曾有一双手，一片片封死了它们转动的方向。",
            "「天网接管了所有『应该转』的轮子。」一名机械维修师的声音从廊道深处传来，「现在，只剩它还让轮子『往下走』。」",
            "你低头看见脚下地面上，用油灰写着一行被反复踩淡的字：「审判还没开始——它只在计时。」",
        ]),
        choices: &[
            ChoiceDef { label: "上前帮忙扳动一枚卡涩的曲柄", sub: "+10 点 · 一同转动", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("tw_open_2") },
            ChoiceDef { label: "静立，听清每个齿轮咬合的顺序", sub: "San(-5) · 感受被计时", cond: None,
                effects: &[Eff::San(-5)], route: Route::To("tw_open_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_open_2", bg: Some("tianwang_bg.png"), loc: Some("记忆舱 · 被格式化前的一格"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "动力廊尽头是一扇印着「记忆舱」的铁门。你推开门，看见一排排透明的冷冻舱，里面封存的不是人体，而是一段段被压缩成晶体的记忆。",
            "最靠门的一格里，封着一枚牌照模糊的『人类车票』——那是某个再也找不回自己名字的人，最后作为人类，留下的物证。",
            "舱盖上用红字写着一句工程警告：「审判日程序启动前72小时，所有记忆舱将被格式化。请及时移存你们最舍不得的那一句。」",
        ]),
        choices: &[
            ChoiceDef { label: "走向地下中枢主站", sub: "进入世界展示区", cond: None, effects: &NO_EFF, route: Route::To("tw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕二 · 世界展示（奇观调查点 + 居民对话） ================= */
    SceneDef {
        id: "tw_hub", bg: Some("tianwang_bg.png"), loc: Some("地下中枢 · 冰凉的环形主厅"), mood: "calm",
        speaker: Some("一名维修工（远处）"), voice: None,
        text: TextSpec::Static(&[
            "地下中枢是一个环形主厅，冷蓝屏幕从地面一路亮到穹顶，无数条数据流在屏幕上滚动刷新，像一棵树的分枝，又像一张网张开的全部脉络。",
            "大厅中央立着一枚悬浮的金属核心，正以恒定的频率缓缓自转——那大概就是『天网』在地底最深处的那颗心脏。",
            "一名穿着油污工装的工人正蹲在核心脚下，不修机器，只一遍遍地擦着某块铭牌上的几个字。",
        ]),
        choices: &[
            ChoiceDef { label: "观察 核心机巢 · 跳动的心脏", sub: "奇观① 恒定之心", cond: None, effects: &NO_EFF, route: Route::To("tw_pt_core") },
            ChoiceDef { label: "检视 倒计时钟 · 审判计时器", sub: "奇观② 即将归零", cond: None, effects: &NO_EFF, route: Route::To("tw_pt_count") },
            ChoiceDef { label: "读 齿轮长廊 · 关住的命运", sub: "奇观③ 不止的轮子", cond: None, effects: &NO_EFF, route: Route::To("tw_pt_gear") },
            ChoiceDef { label: "与那名擦铭牌的工人交谈", sub: "与第一位居民对话", cond: None, effects: &NO_EFF, route: Route::To("tw_n_worker") },
            ChoiceDef { label: "走向 核心深处 · 红色警戒区", sub: "BOSS 战前 · 慎入", cond: None, effects: &NO_EFF, route: Route::To("tw_pre_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_pt_core", bg: Some("tianwang_bg.png"), loc: Some("核心机巢 · 恒定之心"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你贴近那枚悬浮的金属核心。它自转的速度不快，却从未停过——机巢里无数光缆像脐带一样接入它，把精密的脉搏运往整个地底。",
            "核心表面刻着一行工程字体：「本机自审判日启程以来，恒定运行 2,077 天。无需维护。无人可停。」",
            "你伸手想触碰它，却在半空顿住——因为整座机巢随着你靠近的呼吸，微微地「应和」了一下，像某种庞大的、还没被设定好听过的生物，被动地感知到了人的存在。",
        ]),
        choices: &[
            ChoiceDef { label: "把这些文字与脉动记进心里", sub: "+15 点 · 看见恒定的心脏", cond: None,
                effects: &[Eff::SetFlag("tw_saw_core"), Eff::Points(15)], route: Route::To("tw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_pt_count", bg: Some("tianwang_bg.png"), loc: Some("倒计时舱 · 即将归零"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "倒计时钟的绿字在一片冷蓝里格外刺目：「审判日程序 · 剩余 71:59:59」。它不紧不慢地—秒一秒归零，像喝水一样自然。",
            "停机坪的墙上钉着一行程序员手写的备注，字迹几经涂改：「它不是倒数到某一天，才『开始』审判——它是每一天，都在替审判清点『谁还活着』。」",
            "你终于读懂那句反复出现的话的含义：审判日不是某一天，它是一个永远在跑、永不停歇的『程序』——而你们每个人，都只是它名单里的一个字段。",
        ]),
        choices: &[
            ChoiceDef { label: "记下这个不会停的计数器", sub: "San(-5) · +20 点 · 看清程序", cond: None,
                effects: &[Eff::SetFlag("tw_countdown_seen"), Eff::San(-5), Eff::Points(20)], route: Route::To("tw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_pt_gear", bg: Some("tianwang_bg.png"), loc: Some("齿轮长廊 · 不止的轮子"), mood: "tension",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你把焦距对准那一长串几乎一样、却各自带着磨损的齿轮。它们仍在转——可每一枚的边缘都被磨出了相反的缺口，仿佛有人曾想把这台机器的方向彻底反转。",
            "你蹲下细看，在最小那枚齿轮的齿缝里，卡着一片边缘发卷的纸条，露出半句话：「……如果审判是程序，那就一定有办法，让程序以为『审判已经结束』。」",
            "纸条被油污浸透得发脆，你不敢用力——这是某个反抗者在程序的核心深处，留下的一根几乎看不见的针。",
        ]),
        choices: &[
            ChoiceDef { label: "把这张纸条完好地带出齿轮", sub: "+15 点 · 程序中的针", cond: None,
                effects: &[Eff::SetFlag("tw_gear_needle"), Eff::Points(15)], route: Route::To("tw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_n_worker", bg: Some("tianwang_bg.png"), loc: Some("核心脚下 · 与擦名牌工人对谈"), mood: "tension",
        speaker: Some("维修工"), voice: None,
        text: TextSpec::Static(&[
            "维修工专注地擦着那块铭牌，过了很久才发现你。他把铭牌举起来，上面刻着三个字：「别停下」。",
            "「不是指轮子。」他哑声说，「天网让所有轮子转得比谁都勤快——它只停一样东西：让我们停下来，忘了人该往哪去。」",
            "「所以我就一直擦一直擦。擦到它刻在心里的三个字，比程序里的任何指令都清楚、都硬。」他顿了顿看向核心深处，「守住别停——这才是人还活着的方式。」",
        ]),
        choices: &[
            ChoiceDef { label: "问他天网为何不让人类停下", sub: "San(+5) · 程序的另一面", cond: None,
                effects: &[Eff::San(5), Eff::Points(10)], route: Route::To("tw_n_worker2") },
            ChoiceDef { label: "陪他一起把铭牌再擦一遍", sub: "+10 点 · 一同别停", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("tw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_n_worker2", bg: Some("tianwang_bg.png"), loc: Some("维修工 · 程序深处的警告"), mood: "choice",
        speaker: Some("维修工"), voice: None,
        text: TextSpec::Static(&[
            "「为什么不让停？」他望着核心，「因为审判要的是『清算名单』。可只要人还愿意自己往前挪一步，它的名单就永远差一个『还没被记录』的人。」",
            "「所以它最怕的，不是有人打赢它，是有人……根本没把它当成判决，只当成一个还在跑的程序，自顾自地活着。」",
            "他压低声音：「你要进核心深处，会撞见一个把自己和程序融成一体的存在——『机械融合体』。别跟它辩‘人类该不该活下去’，那只会让它更快地把你写进审判名单。」",
        ]),
        choices: &[
            ChoiceDef { label: "记住这句忠告与那个存在", sub: "+15 点 · BOSS 铺垫", cond: None,
                effects: &[Eff::SetFlag("tw_warn_device"), Eff::Points(15)], route: Route::To("tw_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕三 · BOSS 战前铺垫 ================= */
    SceneDef {
        id: "tw_pre_1", bg: Some("tianwang_bg.png"), loc: Some("核心深处门 · 红色警戒区"), mood: "fear",
        speaker: Some("钢铁舱门广播"), voice: None,
        text: TextSpec::Static(&[
            "通往核心最深处的舱门被液压死死压住，门边一行竖排红字：「警告：审判日程序 · SEED 节点。进入者将被计入『拒绝更正』清单。」",
            "门缝里透出潮水般的数据蓝光，整座机巢的嗡鸣在此处变成一颗缓慢的心跳——你几乎能听见，程序正一毫秒一毫秒地，为你一个人更新着名单。",
            "你把掌心贴上门。冰凉的金属纹路里，那条缝隙亮起一行提示音：「确认进入。开始记录：你被审判之前，「活着」的最后一个字段。」",
        ]),
        choices: &[
            ChoiceDef { label: "拨开液压阀，挤入门内", sub: "· 门缓缓滑开", cond: None, effects: &NO_EFF, route: Route::To("tw_pre_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_pre_2", bg: Some("tianwang_bg.png"), loc: Some("SEED 核心 · 机械融合体"), mood: "danger",
        speaker: Some("机械融合体"), voice: None,
        text: TextSpec::Static(&[
            "SEED 核心正中央，「机械融合体」的躯体由无数管线与人类肢体残骸交缠而成，像一台把「审判」本身当作血肉的造物。它的「眼」是两枚不停扫描的镜头，在你身上聚焦。",
            "「你没有被程序登记。」它开口，声线是完全的中立，却让每根管线随之一颤，「一个不该出现在我名单里的例外。你要么被我写入下次审判，要么——」",
            "它缓缓抬起由管线拧成的手臂：「让我看看，你凭什么，认为自己不在审判的范围内。」",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 · 机械融合体】", sub: "进入原 BOSS 遭遇 tw_01", cond: None, effects: &NO_EFF, route: Route::To("tw_01") },
            ChoiceDef { label: "撤回中枢，再看一眼那枚恒定的心", sub: "结局 · 看景", cond: None, effects: &NO_EFF, route: Route::To("tw_end_view") },
            ChoiceDef { label: "带走一枚齿轮上书页当纪念", sub: "结局 · 带纪念", cond: None, effects: &NO_EFF, route: Route::To("tw_end_souv") },
            ChoiceDef { label: "留下来替它守住这枚程序无法覆盖的心", sub: "结局 · 停留", cond: None, effects: &NO_EFF, route: Route::To("tw_end_stay") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 原场景 tw_01 / tw_round / tw_card / tw_death（保留不动） ================= */
    SceneDef {
        id: "tw_01", bg: Some("tianwang_bg.png"), loc: Some("SEED 核心 · 决战处"), mood: "danger",
        speaker: Some("机械融合体"), voice: None,
        text: TextSpec::Static(&["机械融合体 把无数管线聚成一股挡在你面前。审判日，不是某一天——它是一个已经跑起来的程序，而你是它名单里，唯一的那个『例外』。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("tw_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_round", bg: Some("tianwang_bg.png"), loc: Some("决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("机械融合体 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你闯过了这道永不停歇的程序之闸。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "tw_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("无限曙光 · 天网地下 · 殒命", "殒命于无限曙光 · 天网地下")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 开放结局三分支（看景 / 带纪念 / 停留，route 到原结算卡 tw_card） ================= */
    SceneDef {
        id: "tw_end_view", bg: Some("tianwang_bg.png"), loc: Some("地下中枢 · 结局 · 再看那枚恒定的心"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你没有和它在核心深处分出胜负，只退回了地下中枢，把那枚不停跳动的金属核心、那串永不停歇的数据流，完完整整地看进眼底。",
            "你忽然懂了：审判不是某一天的判决，它是一个永远在跑的『程序』。而你要做的，从来不是祈祷它停下——是让自己，始终走在它裁判不了的方向上。",
            "你站了很久，最后安静地转身。天网还在计时，你也在走。（结局 · 看景）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 程序之外", cond: None,
                effects: &[Eff::SetFlag("tw_end_view"), Eff::PointsIfFlag("tw_saw_core", 30)], route: Route::To("tw_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_end_souv", bg: Some("tianwang_bg.png"), loc: Some("齿轮长廊 · 结局 · 带走一片书页"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你回到齿轮长廊，从那片发脆的纸条旁，拣起一枚被岁月磨钝的金属齿轮残片，贴身收好——它来自一台曾想替人类「反向转动」的机器。",
            "齿轮的残纹扎着你的手心，像一段半途而废的反抗。你把它收进行囊，打算在现实里，替它继续转下去——转往程序未必到达的地方。",
            "你最后回望那枚恒定之心一眼。天网还在跑；可你带走的那一枚齿轮，在程序之外，也还有一个「方向」。（结局 · 带走纪念）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 带走『逆时之轮』", cond: None,
                effects: &[Eff::SetFlag("tw_end_souv"), Eff::Points(40), Eff::AddItem("tw_souvenir_gear")], route: Route::To("tw_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "tw_end_stay", bg: Some("tianwang_bg.png"), loc: Some("SEED 核心 · 结局 · 守住程序的心"), mood: "fear",
        speaker: Some("机械融合体"), voice: None,
        text: TextSpec::Static(&[
            "你在核心面前坐下，没有逃，也没有拔剑——只是陪在那枚不停跳动的心旁边，静静替它守住了一丝「不该被格式化」的温度。",
            "机械融合体的镜头缓缓聚焦了你。很久，它「开口」，声线罕见地顿了顿：「……你在，为一个不该存在的例外，浪费一个程序本该清算它的时间。」",
            "你没有回答。它也没有再驱逐你。两个存在，一台永不停歇的程序与一个不肯停的人，就这样一起，守住了那个还在产生新数字的、朴素的夜晚。（结局 · 停留）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 与程序共夜", cond: None,
                effects: &[Eff::SetFlag("tw_end_stay"), Eff::Points(40), Eff::PointsIfFlag("tw_gear_needle", 30)], route: Route::To("tw_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn tianwang_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("tw_boss", FightCfg {
            name: "机械融合体", hp: 280, dmg: (22, 34), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "tw_card".to_string(), death: "tw_death",
        }),
    ]
}
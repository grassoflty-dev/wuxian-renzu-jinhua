//! 《咒怨》任务世界 · 全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/wuxian_kongbu/zhouyuan.md §5/§6/§7/§8。
//! 本文件是全新新增文件，只导出静态数据（ZHOUYUAN_SCENES / zhouyuan_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/zhouyuan_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `zy_` 前缀，与既有 SCENES 无重名。
//! 诅咒计数用方案 A：连号 flag `zy_curse_1/2/3`（纯 Eff::SetFlag + cond，零共享文件改动）。
//! BOSS 战黑发领域「每回合 San-5」/ 仪式镇压「每回合上限 -8%」因引擎 FightCfg 无每回合同调钩子，
//! 用「选择驱动的遭遇链」落地（see_log 取舍 2）；同时导出 `b_kayako` FightCfg 供 ZoneDef 引擎直战复用。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   条件谓词（cond）
   ===================================================================== */
/// 诅咒计数（方案 A）：当前已叠层数（仅判定用）
fn curse_count(st: &GameState) -> usize {
    ["zy_curse_1", "zy_curse_2", "zy_curse_3"].iter().filter(|k| st.flag(k)).count()
}

/// 已有佛珠 + 真相 → 可布置解除怨念仪式
fn cond_exorcism_possible(st: &GameState) -> bool {
    st.flag("zy_buddha") && st.flag("zy_diary_truth")
}

/// 已达成仪式 flag（zy_exorcism）
fn cond_has_exorcism(st: &GameState) -> bool { st.flag("zy_exorcism") }

/// 主卧密道 P3 需要地下室钥匙
fn cond_has_ghost_key(st: &GameState) -> bool { st.flag("zy_ghost_key") }
/// 具名 flag 条件（CondFn 为 fn 指针不能捕获闭包 → 逐个具名定义，供 static 数组使用）
fn cond_has_buddha(st: &GameState) -> bool { st.flag("zy_buddha") }
fn cond_has_toshio_key(st: &GameState) -> bool { st.flag("zy_toshio_key") }
fn cond_has_cat_trust(st: &GameState) -> bool { st.flag("zy_cat_trust") }
fn cond_has_diary_truth(st: &GameState) -> bool { st.flag("zy_diary_truth") }

/* =====================================================================
   动态文本辅助
   ===================================================================== */
fn txt_cat_choice(st: &GameState) -> String {
    if st.inventory.iter().any(|i| i == "item_cat_food") {
        "你摸了摸口袋里的猫粮——一个孩子的声音在你心里说「谢谢你，请我吃饭。」".to_string()
    } else {
        "你没有猫粮。空着手跟一只怨灵走捷径，你想起了资深者的话——「别让猫走前面，除非你带了口粮。」".to_string()
    }
}

/* =====================================================================
   BOSS 战 · 选择驱动遭遇（黑发领域）
   BOSS 血量存 st.fight（由 zy_15_fight 的 Route::Dyn 初始化，引用 b_kayako 的 FightCfg）。
   每"回"是一个 Normal 场景 zy_boss_round；Route::Dyn 统一处理：扣 Boss 血、狂暴回 San-5、
   诅咒叠层、仪式镇压、胜负路由。
   ===================================================================== */

/// 初始化 BOSS 会话（从 b_kayako 的 FightCfg 建 Fight）。需主线合并后 fight_cfg 能解析 b_kayako。
fn start_kayako(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("b_kayako") {
            st.fight = Some(crate::power::scaled_fight("b_kayako", cfg, st, vec![]));
        }
    }
    "zy_boss_round".to_string()
}

/// BOSS 是否已狂暴（HP ≤ 40% = 56）
fn boss_raged(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.hp <= 56).unwrap_or(false)
}

/// 击杀结算（选择驱动遭遇）：依据是否立仪式给奖励/道具/评级，返回胜利场景。
/// 设计文档 zhouyuan_impl_log「取舍 3」：仪式胜 500(基础)+200(额外)=700；强杀胜 250、无支线副产。
fn settle_kayako(st: &mut GameState) -> String {
    if st.flag("zy_exorcism") {
        st.points += 500 + 200;
        st.sp_grade = Some('D');
        crate::world::add_item(st, "item_talisman");
        st.set_flag("zy_exorcism_done");
    } else {
        st.points += 250;
        st.set_flag("zy_strongkill");
    }
    "zy_16_win".to_string()
}

/// 一个"回"：玩家攻击 BOSS。
fn route_boss_attack(st: &mut GameState, dmg: i32) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    // 黑发领域：狂暴期间每回结束 San-5 且头颈反折凝视附带诅咒标记叠加（引擎无助调，用场景数据落地）
    let raged = boss_raged(st);
    if raged {
        st.san = (st.san - 5).clamp(0, 100);
        // 狂暴期间被凝视：累计一层诅咒（上限 3）
        if !st.flag("zy_curse_3") {
            if !st.flag("zy_curse_1") { st.set_flag("zy_curse_1"); }
            else if !st.flag("zy_curse_2") { st.set_flag("zy_curse_2"); }
            else { st.set_flag("zy_curse_3"); }
        }
    }
    // 玩家被 BOSS 反击
    let p_dmg = rnd_zy(10, 18);
    st.hp = (st.hp - p_dmg).max(0);
    // 玩家倒下 → 失败
    if st.hp <= 0 { return "zy_17_lose".to_string(); }
    // 击倒 BOSS（HP≤0）→ 先结算奖励/flag 再进胜利场景（击杀当回即使诅咒满层亦胜）
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return settle_kayako(st); }
    // 诅咒叠满 3 层 → 本"回"之后即被拖入二重死
    if curse_count(st) >= 3 { return "zy_17_lose_curse".to_string(); }
    "zy_boss_round".to_string()
}

/// 仪式镇压（需 zy_exorcism 且 BOSS HP<50%）：每回上限 -8%（此处直接折算为高压制伤害）
fn route_boss_ritual(st: &mut GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(140);
    let cap = (hp as f32 * 0.08) as i32 + 6; // 每回上限 ≈ 8%（至少 6）
    route_boss_attack(st, cap.max(8))
}

fn rnd_zy(a: i32, b: i32) -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(a..=b)
}

/* ---- BOSS 战 文本 ---- */
fn txt_boss_round(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(140);
    let raged = boss_raged(st);
    let head = if raged {
        "满屋都是黑发。伽椰子的头从肩膀后反折过来，惨白的脸正对着你——「嘎——」\n\n<b style='color:#ff6a6a'>黑发领域：每回合结束，你的理智被蚕食（San -5）。</b>"
    } else {
        "地下室结界圈泛着惨白的光。伽椰子四肢着地，黑发覆脸，缓慢地向你逼近。"
    };
    let curse = curse_count(st);
    format!("{head}\n\n<b>伽椰子</b> · 本体　HP {hp}/140\n\n你身上的诅咒层数：{curse}/3。\n\n（黑发在向你脚踝攀爬……）")
}

/* =====================================================================
   战斗配置表（ZY 专属；导出供主线把 query 扩展进来）
   ===================================================================== */
fn zy_rage_common(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>怨念暴涨——她的速度、力道再度拔高。</span>".into());
}
fn zy_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn zy_win_common(st: &GameState) -> String {
    let _ = st.flag("zy_exorcism");
    "zy_16_win".into()
}

/// 咒怨战斗配置表（id 全部 b_ 前缀）。
pub fn zhouyuan_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("b_servant", FightCfg {
            name: "怨灵化佣人", hp: 38, dmg: (7, 13), reward: 20, reward_why: "怨灵化佣人 · 净化",
            intro: "灰黑围裙的佣人垂着手，双目空洞，拖着步子向你逼近。",
            rage_at: None, rage_text: "", on_rage: zy_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: zy_win_common, death: "zy_17_lose",
        }),
        ("b_shade", FightCfg {
            name: "怨灵亡影·残", hp: 32, dmg: (6, 11), reward: 15, reward_why: "怨灵亡影 · 净除",
            intro: "半透明的灰绿影子在厨房阴处凝实，月光从它体侧透过来。",
            rage_at: None, rage_text: "", on_rage: zy_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: zy_win_common, death: "zy_17_lose",
        }),
        ("b_shade2", FightCfg {
            name: "怨灵亡影", hp: 44, dmg: (8, 14), reward: 25, reward_why: "怨灵亡影 · 击破",
            intro: "走廊尽头的人形挣脱壁纸半身探出，十指指痕从墙里向外抓。",
            rage_at: Some(25), rage_text: "虚影分裂成两个——它一回合攻来两记！", on_rage: zy_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: zy_win_common, death: "zy_17_lose",
        }),
        ("b_toshio", FightCfg {
            name: "俊雄暴走", hp: 88, dmg: (10, 16), reward: 120, reward_why: "制止俊雄暴走 · 童锁钥匙",
            intro: "苍白的童影在墙角咧嘴——两个虚影同时向你有来。他不再引路，而是……扑了过来。",
            rage_at: Some(35), rage_text: "虚影分裂！场上同时出现两个俊雄——", on_rage: zy_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |st| { let _ = st.flag("zy_toshio_key"); "zy_10_toys".into() },
            death: "zy_17_lose",
        }),
        ("b_shade3", FightCfg {
            name: "怨灵亡影·强化", hp: 52, dmg: (9, 15), reward: 40, reward_why: "亡影强化 · 击破",
            intro: "更深惨绿的地影在地板上蠕动，拍球声在你耳边放大……",
            rage_at: Some(30), rage_text: "拍球声涌来——每回合蚕食理智（San-3/回）。", on_rage: zy_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: zy_win_common, death: "zy_17_lose",
        }),
        ("b_kayako_shade", FightCfg {
            name: "伽椰子残影", hp: 56, dmg: (11, 17), reward: 80, reward_why: "伽椰子残影 · 驱散",
            intro: "灰白剪影从壁橱底爬出，黑发在你脚边铺开——这是残影，却已经够你死一次。",
            rage_at: Some(30), rage_text: "她四肢并用地爬过来，速度暴涨——", on_rage: zy_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: zy_win_common, death: "zy_17_lose",
        }),
        ("b_kayako", FightCfg {
            name: "伽椰子 · 怨念之源", hp: 140, dmg: (12, 18), reward: 500, reward_why: "击败伽椰子本体",
            intro: "满屋黑发沸腾。她四肢着地从楼梯上爬下来，头从肩膀后转过来——脸上只有一个白。白后面是无尽的黑。",
            rage_at: Some(40), rage_text: "黑发吞没了整间地下室——<b>黑发领域</b>开启！每回合她都会蚕食你的理智（San -5/回）。",
            on_rage: |st, log| {
                st.san = (st.san - 5).clamp(0, 100);
                log.push("<span class='crit'>黑发攀上你的脚踝，理智仿佛被抽走一线（San -5）。</span>".into());
            },
            finisher_if: |_, ehp| ehp <= 28, finisher_name: |st| {
                if st.flag("zy_exorcism") { "仪式镇压 · 解脱".into() } else { "强杀".into() }
            },
            finisher_desc: |st| {
                if st.flag("zy_exorcism") {
                    "你念出那行字——「只要他看我一眼就好。」地下室的怨念像被攥住心脏，黑发猛地收缩、坍缩成一粒光尘。她化作一片光尘，白猫从角落走出，蹭了蹭你的裤脚。".into()
                } else {
                    "你咬牙将最后一击狠狠砸进黑发深处。她倒下了，可地板阴影里，还有一小撮黑发在朝出口爬……".into()
                }
            },
            win: |st| { if st.flag("zy_exorcism") { "zy_16_win".into() } else { "zy_16_win".into() } },
            death: "zy_17_lose",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn zy_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    zhouyuan_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 zy_ 前缀）
   ===================================================================== */
pub static ZHOUYUAN_SCENES: &[SceneDef] = &[

/* ---- 幕 1 · 开场：玄关 · 雨夜 ---- */
SceneDef {
    id: "zy_01", bg: Some("scene_zy_house_exterior.png"), loc: Some("佐伯家 · 玄关"),
    mood: "danger", speaker: Some("主神"), voice: Some("vo_zy_mission"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>在佐伯家宅邸内存活至清晨六点，并调查怨念之源。任务完成前，禁止离开宅邸。失败代价：被咒入二重死。",
        "雨水从门缝渗进来，屋内比外面更冷。玄关的木台阶上摆着一双小小的儿童雨鞋——鞋口朝里，已经干透。身后的大门传来「啪嗒」一声，门闩自己落下了。",
        "二楼传来「啪嗒、啪嗒、啪嗒」——湿漉漉的脚步声，很规律，像有人穿着拖鞋在走廊上来回走。",
    ]),
    choices: &[
        ChoiceDef { label: "【换鞋进门】", sub: "无额外效果", cond: None,
            effects: &[Eff::SetFlag("zy_entered")], route: Route::To("zy_02") },
        ChoiceDef { label: "【先检查那双儿童雨鞋】", sub: "支线 zy_shoe_checked · San-5 · 点数+10", cond: None,
            effects: &[Eff::SetFlag("zy_shoe_checked"), Eff::San(-5), Eff::Points(10)],
            route: Route::To("zy_02") },
        ChoiceDef { label: "【问队友是不是听错了】", sub: "San-3", cond: None,
            effects: &[Eff::San(-3)], route: Route::To("zy_02") },
    ],
    fight_id: None, video: Some("vid_zy_opening.mp4"), cine_label: Some("过场 · 雨夜凶宅（H3 本地生成）"), overlay: None,
},
SceneDef {
    id: "zy_02", bg: Some("scene_zy_house_exterior.png"), loc: Some("佐伯家 · 玄关"),
    mood: "danger", speaker: Some("旁白"), voice: Some("vo_zy_senior_lock"),
    text: TextSpec::Static(&[
        "你踏入玄关。木地板在你脚下发出沉闷的吱呀，像是承受着什么不该承受的重量。",
        "资深者压低声音：「别让恐惧先于理智支配你。这栋房子在‘锁’你了——从现在起，每一项调查都在和它赛跑。」",
        "（左侧佛龛间 / 右侧客厅厨房 / 前方楼梯口 等待探索。）",
    ]),
    choices: &[
        ChoiceDef { label: "走向佛龛 · 储物间", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_03_butsudan") },
        ChoiceDef { label: "走向客厅 · 厨房", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_03_tv") },
        ChoiceDef { label: "直接上二楼", sub: "走楼梯", cond: None, effects: &NO_EFF, route: Route::To("zy_04") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 玄关调查点 · 儿童雨鞋（调查点 zy_p_rainboots → zy_02_shoe） ---- */
SceneDef {
    id: "zy_02_shoe", bg: Some("scene_zy_house_exterior.png"), loc: Some("佐伯家 · 玄关 · 儿童雨鞋"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "玄关木阶上那双小小的儿童雨鞋，鞋口朝里，已经干透。你蹲下把它拎起来——鞋底还夹着一片干枯的枫叶，像从深秋的庭院里带进来的。",
        "雨鞋内衬有一道深深的抓痕，从脚踝连到鞋口，像是有什么东西硬生生拽住过它。",
    ]),
    choices: &[
        ChoiceDef { label: "记住这双鞋（支线 zy_shoe_checked · San-5 · 点数+10）", sub: "调查雨鞋", cond: None,
            effects: &[Eff::SetFlag("zy_shoe_checked"), Eff::San(-5), Eff::Points(10)], route: Route::To("zy_02") },
        ChoiceDef { label: "放回原处，返回玄关", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- F1 调查点 ---- */
SceneDef {
    id: "zy_03_butsudan", bg: Some("scene_zy_house_exterior.png"), loc: Some("F1 · 佛龛·储物间"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "佛龛前的供品碟里，摆着一盘早已干硬的饭团——那是给俊雄留的过期供品。供品旁压着一小袋猫粮。」",
        "资深者低声：「把吃的留在孩子看得见的地方……怨灵也爱收供品。」",
    ]),
    choices: &[
        ChoiceDef { label: "取走供品碟里的猫粮", sub: "Item item_cat_food", cond: None,
            effects: &[Eff::AddItem("item_cat_food"), Eff::Points(10)], route: Route::To("zy_03_back") },
        ChoiceDef { label: "查看佛龛下的壁橱（俊雄捷径）", sub: "P1 起点", cond: None,
            effects: &[Eff::MarkPoint("zy_p_butsudan")], route: Route::To("zy_04") },
        ChoiceDef { label: "返回玄关", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_03_back", bg: Some("scene_zy_house_exterior.png"), loc: Some("F1 · 玄关/客厅间"),
    mood: "cold", speaker: None, voice: Some("vo_zy_toshio_meow"),
    text: TextSpec::Static(&["你把猫粮揣进口袋，佛龛前的灯火微微晃了晃。远处传来一声极轻的「喵」。"]),
    choices: &[ChoiceDef { label: "继续探索", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_03_fridge", bg: Some("scene_zy_house_exterior.png"), loc: Some("F1 · 厨房 · 冰箱"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "冰箱门上的彩色磁贴拼出一行歪扭的字：「妈妈，你在哪」。冰箱里空了一半——罐头打开过，是猫粮的牌子。",
        "有人在厨房悄悄喂过那只神秘的白猫。",
    ]),
    choices: &[
        ChoiceDef { label: "记住这条线索", sub: "点数+5", cond: None,
            effects: &[Eff::Points(5)], route: Route::To("zy_03_back") },
        ChoiceDef { label: "检查角落的菜刀装饰", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_03_knife") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_03_knife", bg: Some("scene_zy_house_exterior.png"), loc: Some("F1 · 厨房"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["菜刀插在木菜板上，刀身泛着冷光。刀下压着一张被咬破的纸——是供品猫粮的说明书的一角。"]),
    choices: &[ChoiceDef { label: "返回", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_03_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_03_tv", bg: Some("scene_zy_house_exterior.png"), loc: Some("F1 · 客厅 · 电视"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "客厅的电视机没开，可屏幕却自己亮着满屏雪花。雪花里隐约浮出一个反着坐的小孩剪影。",
        "茶几上散着几张蜡笔画——画里是一个长头发的妈妈，被一圈黑色的线围住。",
        "资深者：「卧室应该就在楼上……俊雄的房间，也许藏着这把钥匙的来历。」",
    ]),
    choices: &[
        ChoiceDef { label: "端详蜡笔画", sub: "支线 zy_toshio_room 前置 · 点数+10", cond: None,
            effects: &[Eff::SetFlag("zy_toshio_room"), Eff::Points(10)], route: Route::To("zy_03_coffee") },
        ChoiceDef { label: "走进厨房", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_03_fridge") },
        ChoiceDef { label: "回玄关", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_03_coffee", bg: Some("scene_zy_house_exterior.png"), loc: Some("F1 · 客厅 · 茶几"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["蜡笔画的背面，画着一个开得很大的壁橱，里面有很深很深的隧道——像是通往某处的捷径。"]),
    choices: &[ChoiceDef { label: "继续探索", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 2 · 楼梯口 · 俊雄引路 ---- */
SceneDef {
    id: "zy_04", bg: Some("scene_zy_corridor.png"), loc: Some("佐伯家 · 一楼楼梯口"),
    mood: "danger", speaker: Some("俊雄"), voice: Some("vo_zy_toshio_come"),
    text: TextSpec::Dyn(|st| {
        let (head, cat) = (
            "楼梯口的阴影里站着一个穿灰白和服的小男孩。他歪着头，黑眼圈深得像两个洞。他伸手指向自己身后——那是一扇半开的壁橱门，里面没有底，只有一片更深的黑。",
            txt_cat_choice(st),
        );
        format!("{head}\n\n他指了指壁橱，又指指楼上，像是在问你：走捷径，还是走楼梯？\n\n{cat}")
    }),
    choices: &[
        ChoiceDef { label: "【跟猫走（进壁橱捷径）】", sub: "P1 直达 F3；无猫粮=围堵", cond: None,
            effects: &[Eff::SetFlag("zy_cat_trap"), Eff::San(-10)], route: Route::Dyn(route_cat_shortcut) },
        ChoiceDef { label: "【走楼梯（拒绝捷径）】", sub: "San-3 → F2 走廊", cond: None,
            effects: &[Eff::SetFlag("zy_cat_refuse"), Eff::San(-3)], route: Route::To("zy_05") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_04_trap", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼天窗侧"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "壁橱尽头的黑暗把你吐在阁楼的天窗下。你还没来得及站稳，落地点三格内，一缕黑发已无声地铺到你脚边——",
        "伽椰子的残影从梁上倒挂着下来。「——喵。」俊雄在远处轻声说。那不是招呼，是嘲笑。",
    ]),
    choices: &[
        ChoiceDef { label: "迎战残影", sub: "强制遭遇 b_kayako_shade", cond: None,
            effects: &NO_EFF, route: Route::To("zy_13_encounter") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 3 · 二楼走廊 · 壁纸里的人形 ---- */
SceneDef {
    id: "zy_05", bg: Some("scene_zy_corridor.png"), loc: Some("佐伯家 · 二楼走廊"),
    mood: "danger", speaker: Some("队友（资深者）"), voice: Some("vo_zy_senior_wall"),
    text: TextSpec::Static(&[
        "二楼走廊尽头的壁纸鼓起一个<b>人形的包</b>，像有什么东西正从墙里往外面挤。",
        "挂钟停在 3:00。就在你看清挂钟的瞬间，整座房子安静得像被按了静音——连雨声都停了。",
        "头顶传来「哒咔哒咔……哒咔哒咔」，那是四肢同时着地、在阁楼地板上爬行的声音。",
    ]),
    choices: &[
        ChoiceDef { label: "【贴墙绕行，装作没看见】", sub: "San-10", cond: None,
            effects: &[Eff::San(-10)], route: Route::To("zy_06") },
        ChoiceDef { label: "【拉开壁纸检查】", sub: "San-15 · Points+30 · 触发亡影", cond: None,
            effects: &[Eff::San(-15), Eff::Points(30), Eff::SetFlag("zy_photo")], route: Route::To("zy_05_wallpaper") },
        ChoiceDef { label: "【喊俊雄的名字】", sub: "if zy_cat_trust → 浴室线索；else 触发亡影", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_call_toshio) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_05_wallpaper", bg: Some("scene_zy_corridor.png"), loc: Some("F2 · 走廊 · 壁纸"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "壁纸后面是灰泥。灰泥上有一道一道的指痕，从里面向外抓。你退开半步——指痕的深度正好是十根手指，连着抓了三次。",
        "就在这时，灰泥猛地向内凹陷，一只苍白的手从墙里朝你抓来——",
    ]),
    choices: &[
        ChoiceDef { label: "战斗！", sub: "怨灵亡影 b_shade2", cond: None, effects: &NO_EFF, route: Route::To("zy_05_encounter") },
        ChoiceDef { label: "夺路而逃（被拉入墙内）", sub: "d2 壁纸里的人形", cond: None,
            effects: &[Eff::SetFlag("zy_dead_wallpaper")], route: Route::To("zy_17_lose_wall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_05_encounter", bg: Some("scene_zy_corridor.png"), loc: Some("F2 · 走廊"),
    mood: "danger", speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["怨灵亡影从墙里完全挤出，张牙舞爪地扑来——"]),
    choices: &[ChoiceDef { label: "战斗", sub: "b_shade2", cond: None, effects: &NO_EFF, route: Route::To("zy_05_win") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_05_win", bg: Some("scene_zy_corridor.png"), loc: Some("F2 · 走廊"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "亡影溃散成灰烬。拍球声在远处渐弱，像被打断的梦。走廊重新安静下来。",
    ]),
    choices: &[ChoiceDef { label: "继续深入二楼", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_06") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- F2 调查点 / 俊雄房间 / 主卧 ---- */
SceneDef {
    id: "zy_06", bg: Some("scene_zy_room.png"), loc: Some("F2 · 和室·次卧"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "和室的榻榻米上，被褥隆起一个诡异的人形轮廓。墙角的挂轴在无人靠近时，轻轻晃了一下。",
    ]),
    choices: &[
        ChoiceDef { label: "掀开被褥查看", sub: "San-8 · 发现头发", cond: None,
            effects: &[Eff::San(-8)], route: Route::To("zy_06_tatami") },
        ChoiceDef { label: "查看挂轴", sub: "俊雄虚影闪现", cond: None,
            effects: &NO_EFF, route: Route::To("zy_06_kakejiku") },
        ChoiceDef { label: "去走廊尽头的主卧", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_07") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_06_tatami", bg: Some("scene_zy_room.png"), loc: Some("F2 · 和室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["被褥底下压着一大团干枯的黑发，已经和榻榻米纠缠在一起，像生了根。"]),
    choices: &[ChoiceDef { label: "后退", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_06") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_06_kakejiku", bg: Some("scene_zy_room.png"), loc: Some("F2 · 和室 · 挂轴"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["挂轴是一幅婴孩图。你定睛再看时，画里的孩子扭过头，用没有黑眼珠的眼睛看着你。"]),
    choices: &[ChoiceDef { label: "后退", sub: "", cond: None, effects: &[Eff::San(-4)], route: Route::To("zy_06") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_07", bg: Some("scene_zy_room.png"), loc: Some("F2 · 主卧门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["主卧的门缝里漏出惨白的光。你伸手想推，门却纹丝不动——门上贴着一张符纸，佛珠的香气若有若无。"]),
    choices: &[
        ChoiceDef { label: "（佛珠镇门）用佛珠开门", sub: "需 zy_buddha", cond: Some(cond_has_buddha),
            effects: &NO_EFF, route: Route::To("zy_08_bedroom") },
        ChoiceDef { label: "（强开）推门硬闯", sub: "San-15 · 强制遭遇亡影", cond: None,
            effects: &[Eff::San(-15)], route: Route::To("zy_07_forcetrap") },
        ChoiceDef { label: "先探索俊雄房间", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_10_toys") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_07_forcetrap", bg: Some("scene_zy_room.png"), loc: Some("F2 · 主卧门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你用尽全力推开主卧门，门缝里涌出一股寒意——怨灵亡影从门内扑出！"]),
    choices: &[ChoiceDef { label: "迎战", sub: "b_shade2", cond: None, effects: &NO_EFF, route: Route::To("zy_05_encounter") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_07_clock", bg: Some("scene_zy_corridor.png"), loc: Some("F2 · 走廊 · 挂钟"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "走廊尽头的挂钟停在 <b>3:00</b>——不是坏了，是它从来没走过别的数字。秒针悬在 12 的正上方，微微发颤，像被什么攥住了。",
        "你看清挂钟的瞬间，整座房子安静得像被按了静音，连雨声都停了。（挂钟的停摆，似乎与怨念的苏醒同步。）",
    ]),
    choices: &[
        ChoiceDef { label: "记下这不详的时刻（Points+10）", sub: "调查挂钟", cond: None,
            effects: &[Eff::Points(10)], route: Route::To("zy_05") },
        ChoiceDef { label: "退回二楼走廊", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_05") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_08_bedroom", bg: Some("scene_zy_room.png"), loc: Some("F2 · 主卧（伽椰子卧房）"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "主卧的床头柜上立着一张<b>婚礼照</b>。照片里伽椰子笑得很幸福——可照片的门缝外，站着一个黑色的影子。",
        "照片上伽椰子的脸，已经被一层黑气糊掉了。",
    ]),
    choices: &[
        ChoiceDef { label: "凝视婚礼照", sub: "zy_photo · San-15", cond: None,
            effects: &[Eff::SetFlag("zy_photo"), Eff::San(-15), Eff::Points(10)], route: Route::To("zy_08_bed") },
        ChoiceDef { label: "搜查床边（床下）", sub: "San-5", cond: None, effects: &[Eff::San(-5)], route: Route::To("zy_08_bed") },
        ChoiceDef { label: "打开衣柜暗门（P3 密道）", sub: "需 item_ghost_key", cond: Some(cond_has_ghost_key),
            effects: &NO_EFF, route: Route::To("zy_13_basement_trap") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_08_bed", bg: Some("scene_zy_room.png"), loc: Some("F2 · 主卧 · 床边"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "床底一片漆黑。你俯身……看到一截青白的指尖，也正从床底朝床外爬——",
        "你猛地抬头，床尾没有任何东西。错觉。只能是错觉。",
    ]),
    choices: &[
        ChoiceDef { label: "后退（心悸）", sub: "San-7", cond: None, effects: &[Eff::San(-7)], route: Route::To("zy_08_bedroom") },
        ChoiceDef { label: "查看衣柜", sub: "P3 起点", cond: None, effects: &NO_EFF, route: Route::To("zy_08_wardrobe") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_08_wardrobe", bg: Some("scene_zy_room.png"), loc: Some("F2 · 主卧 · 衣柜"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "衣柜的背板鼓着一个包——像一张被压在墙里的脸。你想起了小时候看的鬼片：衣柜打开，里面什么都没有，才是最可怕的。",
        "背板最底下有一颗松动的木栓——那是一条暗道的方向（P3 主卧密道）。",
    ]),
    choices: &[
        ChoiceDef { label: "（有地下室钥匙）钻入暗道", sub: "P3 → 地下室", cond: Some(cond_has_ghost_key),
            effects: &NO_EFF, route: Route::To("zy_13_basement_trap") },
        ChoiceDef { label: "关上柜门", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_08_bedroom") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* F2 浴室 */
SceneDef {
    id: "zy_08_bathroom", bg: Some("scene_zy_room.png"), loc: Some("F2 · 浴室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["浴室镜子前的霜气正在缓缓化开。镜子里你的倒影，抬手比你自己慢了半秒。"]),
    choices: &[
        ChoiceDef { label: "试探镜中的自己", sub: "San-10 · 倒影延迟", cond: None,
            effects: &[Eff::San(-10)], route: Route::To("zy_09_mirror") },
        ChoiceDef { label: "打开排风扇（P2 逃生）", sub: "全副本限一次 · San-10", cond: None,
            effects: &[Eff::SetFlag("zy_vent_used"), Eff::San(-10)], route: Route::To("zy_09_vent") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_09_mirror", bg: Some("scene_zy_room.png"), loc: Some("F2 · 浴室 · 镜子"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["镜子里的你终于抬起了手——可是你并没有动。镜面热度骤升，你猛地一掌拍在镜子上——镜中人却把脸朝你贴了过来。"]),
    choices: &[ChoiceDef { label: "快退（逃跑）", sub: "", cond: None, effects: &[Eff::San(-6)], route: Route::To("zy_06") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_09_vent", bg: Some("scene_zy_room.png"), loc: Some("F2 · 浴室 · 排风扇"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["排风扇叶片拉开一条黑黢黢的通道（P2）。你钻了进去，呼地滑落——被吐回一楼玄关内侧。"]),
    choices: &[ChoiceDef { label: "起身", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- F2 俊雄房间：佛珠 / 地下室钥匙 / 精英战 ---- */
SceneDef {
    id: "zy_10_toys", bg: Some("scene_zy_room.png"), loc: Some("F2 · 俊雄房间"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "俊雄的房间四壁贴满蜡笔画。画面全是同一个场景：妈妈被黑发缠住，一个孩子跪在门口哭。",
        "角落的木制玩具箱上摆着一串<b>佛珠</b>。谁能想到，这个被怨念侵占的孩子，还供奉着如此慈悲的东西。",
    ]),
    choices: &[
        ChoiceDef { label: "取走玩具箱上的佛珠", sub: "zy_buddha · Points+40 · 解锁G2", cond: None,
            effects: &[Eff::SetFlag("zy_buddha"), Eff::Points(40), Eff::AddItem("item_buddha")], route: Route::To("zy_10_toys2") },
        ChoiceDef { label: "翻查玩具箱底层", sub: "地下室钥匙（需先取佛珠）", cond: Some(cond_has_buddha),
            effects: &[Eff::AddItem("item_ghost_key"), Eff::SetFlag("zy_ghost_key")], route: Route::To("zy_10_toys2") },
        ChoiceDef { label: "（童锁钥匙后）返回走廊去阁楼口", sub: "", cond: Some(cond_has_toshio_key),
            effects: &NO_EFF, route: Route::To("zy_10_atticdoor") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_10_toys2", bg: Some("scene_zy_room.png"), loc: Some("F2 · 俊雄房间"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["玩具箱微光一暗一明，像一个人蹲在箱底呼吸。箱底的一枚老钥匙在夜里泛着冷光——那是<b>地下室钥匙</b>（item_ghost_key）。"]),
    choices: &[ChoiceDef { label: "继续探索", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_10_toys") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_10_atticdoor", bg: Some("scene_zy_attic.png"), loc: Some("F2 · 阁楼楼梯口"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "通往阁楼的门锁着童锁。门缝里漏出昏黄的光。你想要童锁钥匙——",
        "走廊深处，俊雄的房间方向传来一阵低沉的抽泣。钥匙，也许就在他暴走失控之后掉落的地方。",
    ]),
    choices: &[
        ChoiceDef { label: "向俊雄房间逼近（触发精英战）", sub: "b_toshio", cond: None, effects: &NO_EFF, route: Route::To("zy_10_toshio") },
        ChoiceDef { label: "绕过（P1 壁橱捷径）", sub: "需猫粮信任", cond: Some(cond_has_cat_trust), effects: &NO_EFF, route: Route::To("zy_11_diary") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_10_toshio", bg: Some("scene_zy_room.png"), loc: Some("F2 · 俊雄房间 · 精英战"),
    mood: "danger", speaker: Some("⚔ 精英战"), voice: None,
    text: TextSpec::Static(&["本可以帮助你的俊雄，此刻双目漆黑、四肢着地，尖利地吼着朝你扑来——他暴走了。"]),
    choices: &[ChoiceDef { label: "战斗", sub: "b_toshio · 胜得童锁钥匙", cond: None, effects: &NO_EFF, route: Route::To("zy_10_toshio_win") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_10_toshio_win", bg: Some("scene_zy_room.png"), loc: Some("F2 · 俊雄房间"),
    mood: "cold", speaker: Some("俊雄"), voice: Some("vo_zy_toshio_thanks"),
    text: TextSpec::Static(&["虚影退散，俊雄恢复成那个苍白的小男孩。他怯生生地把一枚<b>童锁钥匙</b>放在地上，然后化成一缕黑烟窜向阁楼方向——他愿意为你引路了。「喵。」"]),
    choices: &[ChoiceDef { label: "拾起童锁钥匙", sub: "zy_toshio_key", cond: None,
        effects: &[Eff::SetFlag("zy_toshio_key"), Eff::AddItem("item_toshio_key")], route: Route::To("zy_10_atticdoor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_10_toshio_win2", bg: Some("scene_zy_attic.png"), loc: Some("F2 · 阁楼楼梯口"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["童锁钥匙落进门锁。阁楼的门，在夜色里「吱呀」一声打开了一条缝。"]),
    choices: &[ChoiceDef { label: "推门上阁楼", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_11_corpse") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 4 · 阁楼 · 藏尸处与日记 ---- */
SceneDef {
    id: "zy_11_corpse", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼夹层"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "天窗漏下一束惨白的光。光里，天花板夹层的木板被撬开，报纸和塑料布裹着一具蜷缩的女性尸体——<b>伽椰子</b>。她保持着死去的姿势：双手前伸，像是那一刻还在向什么人伸手。",
        "（zy_p_corpse）你触碰裹尸布一角——骸骨在报纸下微微发冷，像还留着体温。",
    ]),
    choices: &[
        ChoiceDef { label: "【取走日记，记住真相】", sub: "zy_diary_truth · Points+80 · San-12 · 解锁G3", cond: None,
            effects: &[Eff::SetFlag("zy_diary_truth"), Eff::Points(80), Eff::San(-12), Eff::AddItem("item_diary")],
            route: Route::To("zy_12") },
        ChoiceDef { label: "【合掌默哀三秒】", sub: "zy_mourned · San-5", cond: None,
            effects: &[Eff::SetFlag("zy_mourned"), Eff::San(-5)], route: Route::Dyn(|st: &mut GameState| {
                st.set_flag("zy_diary_truth"); st.points += 0; "zy_11_mourned".to_string()
            }) },
        ChoiceDef { label: "【焚毁尸体，断根】", sub: "zy_burn · San-18 · 强制遭遇残影", cond: None,
            effects: &[Eff::SetFlag("zy_burn"), Eff::San(-18)], route: Route::To("zy_11_burn") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_11_mourned", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼 · 藏尸处"),
    mood: "cold", speaker: Some("伽椰子"), voice: Some("vo_zy_kayako_thank"),
    text: TextSpec::Static(&["你双手合十。尸体上方浮起一团淡蓝的光雾，缓缓落回地板。你听见一声极轻的叹息——是释然，不是怨恨。「……谢谢你。」"]),
    choices: &[ChoiceDef { label: "（默哀后）取日记离开", sub: "", cond: None,
        effects: &[Eff::SetFlag("zy_diary_truth"), Eff::Points(80), Eff::San(-12)], route: Route::To("zy_12") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_11_burn", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["火苗顺着报纸窜起。尸体在火里<b>动了</b>——像是要爬出来！伽椰子的残影踩着火舌扑来。"]),
    choices: &[ChoiceDef { label: "迎战残影", sub: "b_kayako_shade", cond: None, effects: &NO_EFF, route: Route::To("zy_13_encounter") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_11_diary", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼 · 旧皮箱"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "旧皮箱里压着一本笔记本。扉页写着：<b>「只要他看我一眼就好——小林君。」</b>日期停在她被杀的三天前。",
        "你读完了她最后的日记。那不是怨恨的恶咒，而是一个被辜负的、从未被真正看见的女人，把最后一点执念，缠在了这栋房子上。",
    ]),
    choices: &[
        ChoiceDef { label: "记住真相", sub: "zy_diary_truth · Points+80", cond: None,
            effects: &[Eff::SetFlag("zy_diary_truth"), Eff::Points(80), Eff::San(-12), Eff::AddItem("item_diary")],
            route: Route::To("zy_12") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* F3 天窗日出（若真相未达成 → d4 迟到者） */
SceneDef {
    id: "zy_12", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼台阶/地下室交界"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "阁楼的台阶尽头，是一扇刻满抓痕的铁门（G3 地下室铁门）。台下是地下室，结界核心就在那里。",
        "资深者：「底下，就是怨念的核心。想结束这一切，必须下去。」",
    ]),
    choices: &[
        ChoiceDef { label: "【下地下室（需真相 zy_diary_truth）】", sub: "G3", cond: Some(cond_has_diary_truth),
            effects: &NO_EFF, route: Route::To("zy_13_basement") },
        ChoiceDef { label: "（无真相）强行撬门（绕行）", sub: "P3 密道 或 强行下潜", cond: None,
            effects: &NO_EFF, route: Route::To("zy_13_basement_trap") },
        ChoiceDef { label: "（天亮）回阁楼天窗", sub: "zy_p_skylight", cond: None,
            effects: &NO_EFF, route: Route::To("zy_18_dawn") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_13_basement", bg: Some("scene_zy_battle.png"), loc: Some("F3 · 地下室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "铁门在身后合拢，埃埃而下。地下室近乎漆黑，只有结界圈的白线泛着惨白微光。",
        "资深者把一根黑发从你肩头拈开：「从这里开始，我们只能靠自己了。」",
    ]),
    choices: &[ChoiceDef { label: "走向结界圈", sub: "决战", cond: None, effects: &NO_EFF, route: Route::To("zy_14_well") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_13_basement_trap", bg: Some("scene_zy_battle.png"), loc: Some("F3 · 地下室 · 绕行"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你从密道/侧路潜入漆黑的地下室。黑发井边传来低语，而你没有真相——怨念未曾被理解，结界更凶险。"]),
    choices: &[ChoiceDef { label: "走向结界圈", sub: "未解真相", cond: None, effects: &NO_EFF, route: Route::To("zy_14_well") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_14_well", bg: Some("scene_zy_battle.png"), loc: Some("F3 · 地下室 · 黑发井/结界圈"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "地下室正中，有人用白线画过一个巨大的圈，四角压着黑发。整栋宅邸七年来的怨念，都在这里熬成了一点一点的黑。",
        "伽椰子的本体，此刻正从圈的中央，缓缓朝你的方向爬来。",
    ]),
    choices: &[ChoiceDef { label: "迎向结界核心", sub: "进入决战", cond: None, effects: &NO_EFF, route: Route::To("zy_15_fight") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 5 · 决战 · 结界 ---- */
SceneDef {
    id: "zy_15_fight", bg: Some("scene_zy_battle.png"), loc: Some("佐伯家 · 地下室结界圈"),
    mood: "danger", speaker: Some("伽椰子"), voice: Some("vo_zy_kayako_leave"),
    text: TextSpec::Static(&[
        "「嘎——」从脖子反折的女声里挤出。伽椰子趴在地上，头从肩膀后转过来看着你。「……谁，也别想离开这栋房子。」",
        "地下室中央的白线结界亮着。你想起日记里的那句话——「只要他看我一眼就好。」",
    ]),
    choices: &[
        ChoiceDef { label: "【布置解除怨念仪式】", sub: "需 佛珠+真相 Config → zy_exorcism", cond: Some(cond_exorcism_possible),
            effects: &[Eff::SetFlag("zy_exorcism"), Eff::San(-15)], route: Route::Dyn(start_kayako) },
        ChoiceDef { label: "【直接决战】", sub: "无仪式加成", cond: None,
            effects: &[Eff::San(-10)], route: Route::Dyn(start_kayako) },
        ChoiceDef { label: "【掉头逃跑】", sub: "San-30 · 被堵回战场", cond: None,
            effects: &[Eff::San(-30), Eff::SetFlag("zy_cornered")], route: Route::Dyn(start_kayako) },
    ],
    fight_id: None, video: Some("vid_zy_boss.mp4"), cine_label: Some("过场 · 地下室结界 · 决战（H3 本地生成）"), overlay: None,
},
/* BOSS 战 · 选择驱动回合 */
SceneDef {
    id: "zy_boss_round", bg: Some("scene_zy_battle.png"), loc: Some("佐伯家 · 地下室 · 黑发领域"),
    mood: "danger", speaker: Some("⚔ 决战"), voice: Some("vo_zy_kayako_growl"),
    text: TextSpec::Dyn(txt_boss_round),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 34-46 · 易露破绽", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| route_boss_attack(st, rnd_zy(34, 46))) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 22-30 · 稳", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| route_boss_attack(st, rnd_zy(22, 30))) },
        ChoiceDef { label: "【仪式镇压】", sub: "每回上限 -8%（需仪式+HP<50%）", cond: Some(cond_has_exorcism),
            effects: &NO_EFF, route: Route::Dyn(route_boss_ritual) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 胜利结算（zy_16_win） */
SceneDef {
    id: "zy_16_win", bg: Some("scene_zy_battle.png"), loc: Some("佐伯家 · 地下室结界"),
    mood: "calm", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "黑发从她身上剥落，露出被束缚了七年的年轻女人的脸。她张了张嘴，没有声音，然后化作一片光尘。",
        "白猫从角落走出来，蹭了蹭你的裤脚，消失在雨里。门外，雨停了，天亮了。",
    ]),
    choices: &[ChoiceDef { label: "（等待天明 · 光柱传送）", sub: "主神光柱强制传送（由主线 glok 接线）", cond: None,
        effects: &[Eff::SetFlag("zy_dawn"), Eff::SetFlag("zy_win")], route: Route::Dyn(route_win_settle) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_16_card_exorcism", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "解 脱".into(), good: true,
            body_html: format!(
                "<p>……门外，雨停了，天亮了。你卸下这一晚的咒怨，白发的勒痕还在腕上，却已不再发烫。</p>\
                 <p style='color:#9a958a'>真凶得偿，怨念得释。</p>\
                 <table class='statTable'>\
                 <tr><td>奖励点数（击败伽椰子本体）</td><td>+500</td></tr>\
                 <tr><td>仪式达成额外奖励（zy_exorcism）</td><td>+200</td></tr>\
                 <tr><td>支线剧情评级</td><td style='color:#ffd76a'>D 级</td></tr>\
                 <tr><td>获取道具</td><td>驱邪符 item_talisman（对异形/魔物系+25%）</td></tr>\
                 </table>\
                 <p style='color:#8fd0a8'>剩余点数：{}　完成支线：解毒 zy_exorcism</p>\
                 <p style='color:#ffd76a'>【主神】：新手已祛除怨念之源。下一场……是《异形》。你们的驱邪符，留着会有用的。</p>",
                st.points
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "zy_16_card_strong", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |_st| crate::state::Card {
            title: "未 竟 的 咒".into(), good: true,
            body_html: "<p>她倒在结界里，黑发散了一地。可你看见，地板的阴影里，还有一小撮黑发在慢慢……朝出口爬。</p>\
                <p style='color:#666'>「……还会再来的。」她在你耳边说。</p>\
                <p style='color:#9a958a'>未祛除诅咒的胜利：奖励减半、无支线评级、无驱邪符。</p>\
                <p style='color:#666'>(此胜利不结算 D 级支线，通关评价为普通胜利。)</p>".to_string(),
            buttons: vec![("返 回 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
/* 失败/死亡档案（四种） */
SceneDef {
    id: "zy_17_lose", bg: None, loc: None, mood: "danger", speaker: None, voice: Some("vo_zy_kayako_defeat"),
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被咒入二重死", "伽椰子索命：被拖进黑发深处，听见第二次心跳停止")), card: |_st| crate::state::Card {
            title: "被 咒 入 二 重 死".into(), good: false,
            body_html: r#"<p>黑发缠上你的脚踝，拉进地板。你听见自己的心跳——第一次停了，第二次，又停了。伽椰子的声音贴着耳朵：「我说过，谁也别想离开。」</p>
<p style='color:#ff8a8a'>【死亡档案 · d3 被咒入二重死】</p>
<p style='color:#666'>（复活：回主神空间扣 400 点，由主线主神复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "zy_17_lose_curse", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被咒入二重死", "诅咒标记叠满三层，下回合你的心跳停止了两次")), card: |_st| crate::state::Card {
            title: "二 重 死".into(), good: false,
            body_html: r#"<p>三枚黑发结成的环在你腕上收紧。你数到第三层的那一刻，整个世界都安静下来——你听见自己的心跳，第一次停了，第二次，又停了。</p>
<p style='color:#ff8a8a'>【死亡档案 · d3 被咒入二重死（诅咒叠满）】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "zy_17_lose_wall", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("壁纸里的人形", "强开壁纸被拉入墙内")), card: |_st| crate::state::Card {
            title: "壁 纸 里 的 人 形".into(), good: false,
            body_html: r#"<p>你想逃，可灰泥里的人更快。苍白的手抓住你的手腕，把你拖进墙里。最后的余光里，是走廊尽头的挂钟——3:00。</p>
<p style='color:#ff8a8a'>【死亡档案 · d2 壁纸里的人形】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "zy_17_lose_san", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("湿冷雨夜", "理智归零，精神先于肉体被拖入黑发深处")), card: |_st| crate::state::Card {
            title: "湿 冷 雨 夜".into(), good: false,
            body_html: r#"<p>你坐在玄关的台阶上，雨声越来越大。你忽然觉得自己很冷，很困。等资深者回来时，你已经没有呼吸了——脸上却带着笑。</p>
<p style='color:#ff8a8a'>【死亡档案 · d1 湿冷雨夜】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "zy_17_lose_late", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("阁楼的迟到者", "天亮未解真相，被二重死")), card: |_st| crate::state::Card {
            title: "阁 楼 的 迟 到 者".into(), good: false,
            body_html: r#"<p>天窗外泛白。你终于找到藏尸处，可已经太迟——你没有在夜晚结束前洞悉真相，怨念在你醒来前把你拖下阁楼。</p>
<p style='color:#ff8a8a'>【死亡档案 · d4 阁楼的迟到者】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- 天亮出口 / 天窗 ---- */
SceneDef {
    id: "zy_18_dawn", bg: Some("scene_zy_attic.png"), loc: Some("F3 · 阁楼 · 天窗"),
    mood: "cold", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "天窗外泛起灰白。你来到天窗下——若真相与仪式都已达成，天亮 6:00 的这一刻，主神会降下光柱。",
    ]),
    choices: &[
        ChoiceDef { label: "（真相达成）迎接光柱传送", sub: "主线完成", cond: Some(|st: &GameState| st.flag("zy_diary_truth") && st.flag("zy_exorcism_done")),
            effects: &NO_EFF, route: Route::To("zy_do_teleport") },
        ChoiceDef { label: "（真相未达成）在天窗下等待", sub: "d4 迟到者", cond: None,
            effects: &[Eff::SetFlag("zy_dead_late")], route: Route::To("zy_17_lose_late") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_do_teleport", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["【任务完成】清晨六点。主神光柱自天窗降下，将你笼罩——强制传送回主神空间。"]),
    choices: &[ChoiceDef { label: "（光柱传送）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_19_done") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "zy_19_done", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "任 务 完 成".into(), good: true,
            body_html: format!("<p>你站在雨后的门口，天光大亮。这一夜，你从佐伯家凶宅活着走了出来，还解除了缠绕了七年的咒怨。</p>\
                <p style='color:#9a958a'>《咒怨》副本 · 已完成（出口/复活/结算由主神空间系统接线）</p>\
                <table class='statTable'><tr><td>奖励点数</td><td>{}</td></tr><tr><td>理智</td><td>{}</td></tr></table>\
                <p style='color:#ffd76a'>下一场：<b>《异形》</b>。驱邪符留着，会用上的。</p>", st.points, st.san.max(0)),
            buttons: vec![("进 入 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* —— NPC 对话（资深者）—— */
SceneDef {
    id: "zy_00_teammate", bg: Some("scene_zy_house_exterior.png"), loc: Some("佐伯家 · 玄关"),
    mood: "danger", speaker: Some("资深者"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("zy_diary_truth") {
            "资深者攥着那本日记，声音低得像怕吵醒什么：「原来……她只是想要被看见。别让怨念吞了你，也别让恐惧吞了你。」".to_string()
        } else {
            "资深者靠在玄关柱子上，压低声音：「孩子会引路，也会害人。想从这栋房子活着出去——要么带吃的，要么带真相。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "（继续探索）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("zy_02") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
];

/// 跟猫走：有猫粮走安全线（P1 契约），无猫粮走围堵（落地点遭遇残影）
fn route_cat_shortcut(st: &mut GameState) -> String {
    if st.inventory.iter().any(|i| i == "item_cat_food") {
        st.set_flag("zy_cat_trust");
        st.set_flag("zy_cat_safe");
        st.points += 20;
        st.san = (st.san - 5).clamp(0, 100);
        // 安全落地 F3 阁楼天窗侧（P1）
        "zy_11_diary".to_string()
    } else {
        // 无猫粮 → 围堵：落地即遇伽椰子残影
        "zy_04_trap".to_string()
    }
}

fn route_call_toshio(st: &mut GameState) -> String {
    if st.flag("zy_cat_trust") {
        st.set_flag("zy_bath_hint");
        st.san = (st.san - 5).clamp(0, 100);
        "zy_08_bathroom".to_string()
    } else {
        st.san = (st.san - 8).clamp(0, 100);
        "zy_05_encounter".to_string()
    }
}

// zy_16_win 结算按钮：奖励/道具/评级已在 settle_kayako（击杀回）核算完毕，此处仅路由到对应结算卡。
fn route_win_settle(st: &mut GameState) -> String {
    if st.flag("zy_exorcism") {
        // 仪式胜：5+200 已计入、item_talisman/sp_grade 已置位
        "zy_16_card_exorcism".to_string()
    } else {
        // 强杀胜：250 已由 settle_kayako 计入、zy_strongkill 已置位
        "zy_16_card_strong".to_string()
    }
}


/// 本文件场景查询辅助（主线合并查询扩展时可直接使用）
pub fn zy_scene(id: &str) -> Option<&'static SceneDef> {
    ZHOUYUAN_SCENES.iter().find(|s| s.id == id)
}

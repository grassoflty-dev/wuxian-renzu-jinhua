//! 《异形4·奥瑞迦号》任务世界 · 全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/wuxian_kongbu/yiying.md §4/§5/§6/§7/§8/§10。
//! 本文件是全新新增文件，只导出静态数据（YIYING_SCENES / yiying_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/yiying_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `yiy_` 前缀，与既有 SCENES 无重名。
//! 寄生倒计时用方案 A：连号 flag `yiy_parasite_1/2/3`（纯 Eff::SetFlag + cond，零共享文件改动）——
//! 被抱脸的普维斯被救后，每推进一幕置一层；第 3 幕仍未取样（yiy_cured）则破胸死亡（「摇篮曲」）。
//! sp_grade=C（C 级支线剧情）用 Route::Dyn 写 `Some('C')`，供兑换闭环判定。
//! 酸血残留（post_kill_acid）：击杀工兵及以上异形后，下一行动用场景 flag 一次性结算 `Hurt(4)`。

use crate::defs::*;
use crate::state::{GameState, Weapon};

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   条件谓词（cond，具名 fn 指针供 static 数组使用）
   ===================================================================== */
fn cond_father_off(st: &GameState) -> bool { st.flag("yiy_father_off") }
fn cond_has_pulse(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "yiy_pulse") }
fn cond_has_medkey(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "yiy_key_med") }
fn cond_not_queen_final(st: &GameState) -> bool { !st.flag("yiy_queen_final") }
/// Father 关停 或 持脉冲枪 —— 主控室/巢穴门的双条件放行
fn cond_gate_dyn(st: &GameState) -> bool { st.flag("yiy_father_off") || st.inventory.iter().any(|i| i == "yiy_pulse") }
/// 管道过热熔毁终结前置：已关停 Father
fn cond_queen_pipe(st: &GameState) -> bool { st.flag("yiy_father_off") }
/// 电磁束缚终结前置：持有电磁束缚装置
fn cond_queen_em(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "yiy_em_restraint") }

/* =====================================================================
   寄生倒计时（方案 A 连号 flag）＋ 酸血残留
   ===================================================================== */
fn parasite_stage(st: &GameState) -> usize {
    ["yiy_parasite_1", "yiy_parasite_2", "yiy_parasite_3"].iter().filter(|k| st.flag(k)).count()
}

/// 推进寄生倒计时一层；若已在第 3 幕仍未取样 → 返回破胸死亡场景（「摇篮曲」）。
fn parasite_tick(st: &mut GameState) -> String {
    if !st.flag("yiy_infected") || st.flag("yiy_cured") {
        return String::new();
    }
    if !st.flag("yiy_parasite_1") { st.set_flag("yiy_parasite_1"); }
    else if !st.flag("yiy_parasite_2") { st.set_flag("yiy_parasite_2"); }
    else if !st.flag("yiy_parasite_3") { st.set_flag("yiy_parasite_3"); }
    else {
        st.set_flag("yiy_parasite_dead");
        return "yiy_dead_parasite".into();
    }
    String::new()
}

/// 酸血残留落地（一次性）
fn apply_acid(st: &mut GameState) {
    if st.flag("yiy_post_kill_acid_pending") {
        st.hp = (st.hp - 4).max(0);
        st.set_flag("yiy_post_kill_acid_done");
    }
}

fn rnd(a: i32, b: i32) -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(a..=b)
}

/* =====================================================================
   战斗配置表
   §4 敌人数值（蜂巢 +30%）：普通 HP70-90 / 精英 HP100-130 / BOSS 200。
   ===================================================================== */
fn yy_rage_common(st: &mut GameState, log: &mut Vec<String>) {
    st.san = (st.san - 3).clamp(0, 100);
    log.push("<span class='crit'>异形发出刺耳嘶鸣，速度、力道再度拔高（San -3）。</span>".into());
}
fn yy_rage_reinforce(_st: &mut GameState, log: &mut Vec<String>) {
    log.push("<span class='crit'>隧道深处又翻涌出一只破胸体/工兵——增员！</span>".into());
}
fn yy_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn yy_win_common(_st: &GameState) -> String { "yiy_win_card".into() }

// FightCfg.win 是 fn(&GameState)->String（不可变），故酸血/工兵奖励/烧巢 flag 不在此落地，
// 而是 `win` 返回对应 win 场景，由该场景的 Route::Dyn 选择落地副作用（见 yiy_win_acid/worker/burn 场景）。
fn yy_win_acid(_st: &GameState) -> String { "yiy_win_acid".into() }
fn yy_win_worker(_st: &GameState) -> String { "yiy_win_worker".into() }
fn yy_win_burn(_st: &GameState) -> String { "yiy_win_burn".into() }

fn yy_win_queen(_st: &GameState) -> String { "yiy_queen_win".into() }

/// 异形战斗配置表（id 全部 f_yiy_ 前缀）。
pub fn yiying_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("f_yiy_facehugger", FightCfg {
            name: "抱脸虫", hp: 35, dmg: (2, 6), reward: 20, reward_why: "首战教学 · 击退抱脸虫",
            intro: "一只灰褐泛黄的抱脸虫从暗角扑出，八爪张开，直取你的门面——它想往你脸上寄一只卵！",
            rage_at: Some(15), rage_text: "被逼入绝境的抱脸虫猛地弹起，八只爪朝你脸上糊来——", on_rage: yy_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yy_win_common, death: "yiy_dead_nest",
        }),
        ("f_yiy_chestburst", FightCfg {
            name: "破胸体 · 群", hp: 45, dmg: (8, 14), reward: 40, reward_why: "第一具破尸 · 涌出的破胸体",
            intro: "餐厅那具尸体的肋骨折断——一只惨白的破胸体带着黏血钻出来，紧跟着是第二只、第三只。",
            rage_at: Some(20), rage_text: "破胸体翻涌聚集——又多被咬出一只！", on_rage: yy_rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yy_win_common, death: "yiy_dead_nest",
        }),
        ("f_yiy_worker1", FightCfg {
            name: "异形工兵 · 初现", hp: 70, dmg: (12, 18), reward: 120, reward_why: "层末教学精英战 · 击退工兵",
            intro: "直立的气门甲士从阴影里抬起身，黑色外骨骼在应急灯下反着冷光——它躬身警戒，尾鞭在地面划出火星。",
            rage_at: Some(25), rage_text: "工兵脊背起伏，身形倏地沉入阴影——藏匿隐遁，准备偷袭！", on_rage: yy_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |st| yy_win_acid(st), death: "yiy_dead_nest",
        }),
        ("f_yiy_worker2", FightCfg {
            name: "异形工兵 · 巡逻", hp: 75, dmg: (12, 18), reward: 160, reward_why: "主控室门 · 击退巡逻工兵",
            intro: "一只工兵在主控室门前巡弋，机械而警觉。它似乎正与某个「上级信号」保持联络。",
            rage_at: Some(25), rage_text: "工兵信号紊乱，暴起突刺——隐遁重袭！", on_rage: yy_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |st| yy_win_worker(st), death: "yiy_dead_nest",
        }),
        ("f_yiy_workerpack", FightCfg {
            name: "工兵伏击群（×2）", hp: 85, dmg: (13, 19), reward: 200, reward_why: "孵化室门口 · 伏击群击破",
            intro: "孵化室门口两团黑影同时扑出——工兵伏击群！若先用喷灯引燃走廊，可少一只。",
            rage_at: Some(25), rage_text: "潜伏的第三只工兵挣脱束缚地涌出——增员！", on_rage: yy_rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |st| yy_win_acid(st), death: "yiy_dead_nest",
        }),
        ("f_yiy_sentinel1", FightCfg {
            name: "异形哨兵（B）", hp: 100, dmg: (15, 22), reward: 250, reward_why: "镇守孵化室 · 击破哨兵",
            intro: "孵化室深处的哨兵直立而起，肩甲化的骨板在卵光下泛绿——比工兵粗壮一圈，尾端是致命的枪状尾刺。",
            rage_at: Some(30), rage_text: "哨兵尾刺化作残影连刺三记——尾刺连击！", on_rage: yy_rage_common,
            finisher_if: |_, ehp| ehp <= 30, finisher_name: |_| "尾枪贯刺".into(),
            finisher_desc: |_| "你迎着刺出的尾枪侧步闪过，反手一击贯入哨兵的颅腔——骨板崩裂，它轰然坠地。".into(),
            win: |st| yy_win_acid(st), death: "yiy_dead_nest",
        }),
        ("f_yiy_sentinel2", FightCfg {
            name: "异形哨兵 · 驻守（B）", hp: 110, dmg: (16, 23), reward: 250, reward_why: "引擎桥驻守 · 击破哨兵",
            intro: "引擎桥的哨兵把守着反应堆闸门，粗大的尾刺警惕地指向你。绕过它走通风管道更安全……但它挡住了近路。",
            rage_at: Some(30), rage_text: "哨兵尾刺连击——破绽在它收刺的瞬间！", on_rage: yy_rage_common,
            finisher_if: |_, ehp| ehp <= 30, finisher_name: |_| "尾枪贯刺".into(),
            finisher_desc: |_| "你抓住它收刺的刹那，以全速贯入哨兵咽喉。酸血喷溅，你在腥液里卧倒闪过。".into(),
            win: |st| yy_win_acid(st), death: "yiy_dead_nest",
        }),
        ("f_yiy_hunter", FightCfg {
            name: "异形猎手（精英）", hp: 125, dmg: (16, 24), reward: 280, reward_why: "中段精英 · 击退猎手",
            intro: "通风管壁的暗处，一道瘦高的无眼骷髅缓缓垂下四条超长四肢——异形猎手。它半跪潜行，瞳孔亮着暗红。",
            rage_at: Some(40), rage_text: "猎手身形消失又现，半秒内掠过三米——狂暴潜袭！", on_rage: yy_rage_common,
            finisher_if: |_, ehp| ehp <= 30, finisher_name: |_| "潜行腰斩".into(),
            finisher_desc: |_| "你预判它的潜袭轨迹，侧滚后反手一击砍进它肋骨缝。它痉挛着垂下，那双暗红瞳孔彻底熄灭。".into(),
            win: |st| yy_win_acid(st), death: "yiy_dead_hunter",
        }),
        ("f_yiy_queenhold", FightCfg {
            name: "巢穴抱脸虫群", hp: 50, dmg: (6, 10), reward: 80, reward_why: "卵房清剿（可选战）",
            intro: "卵苞成片开合，一只只抱脸虫扑跳而出——清剿它们，为烧掉蜂巢扫清障碍。",
            rage_at: Some(20), rage_text: "更多抱脸虫从卵苞里涌出——", on_rage: yy_rage_reinforce,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: yy_win_burn, death: "yiy_dead_nest",
        }),
        ("f_yiy_queen", FightCfg {
            name: "异形皇后", hp: 200, dmg: (16, 24), reward: 450, reward_why: "击败异形皇后",
            intro: "骨冠巨兽缓缓抬起——异形皇后盘踞在卵房中央，四臂腹囊蠕动，尾带毒刺。整个巢穴都在回应它的心跳。",
            rage_at: Some(35), rage_text: "皇后发出撕裂空气的尖啸，腹囊蠕动膨胀——卵从体内滚落，卵苞自行打开，抱脸虫如潮水般涌向地板！",
            on_rage: |st, log| {
                st.san = (st.san - 5).clamp(0, 100);
                log.push("<span class='crit'>产卵暴走：酸血四溅（每回合蚀伤 3），尾刺范围攻击（dmg+2），每 2 回合增员一只抱脸虫。</span>".into());
            },
            finisher_if: |st, _| st.flag("yiy_queen_pipe") || st.flag("yiy_queen_em"),
            finisher_name: |st| if st.flag("yiy_queen_pipe") { "管道过热熔毁".into() } else if st.flag("yiy_queen_em") { "电磁束缚 · 重火力".into() } else { "强杀".into() },
            finisher_desc: |st| {
                if st.flag("yiy_queen_pipe") {
                    "你把皇后引向冷却管道——Father 已失联，散热协议失效。3000°C 等离子蒸汽烧穿空气，皇后被裹进一股白热的洪流，未及落地酸血便已蒸发。".into()
                } else if st.flag("yiy_queen_em") {
                    "你扣动电磁束缚装置——电弧箍住皇后的骨冠，它瘫痪嘶吼。随后两回合，你把全部重火力倾泻进它甲壳。".into()
                } else {
                    "你咬牙将最后一击轰进皇后的腹囊。它坠地，但酸血已在金属地面嘶嘶蚀开——必须立刻退开！".into()
                }
            },
            win: yy_win_queen, death: "yiy_dead_nest",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn yiy_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    yiying_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

fn queen_cfg() -> &'static FightCfg {
    yiy_fight_cfg("f_yiy_queen").expect("f_yiy_queen 存在于 yiying_figths")
}

/* =====================================================================
   情景剧情（id 全部 yiy_ 前缀）
   ===================================================================== */
pub static YIYING_SCENES: &[SceneDef] = &[

/* ---- 幕 0 · 开场：「奥瑞迦号的任务」 ---- */
SceneDef {
    id: "yiy_s0_arrive", bg: Some("yiying_bg.png"), loc: Some("贝蒂号登陆坞 · 主神光柱"),
    mood: "cold", speaker: Some("主神"), voice: Some("vo_yiying_missiongod"),
    text: TextSpec::Static(&[
        "<b>【主线任务发布】</b>任务世界·异形4（奥瑞迦号）。主线：①关闭船载 AI「Father」→ ②引爆反应堆 → ③生还撤离。奖励点数 <b>400</b>。",
        "主神光柱（冰冷男声）：『任务世界·异形4（奥瑞迦号）。关闭 Father，引爆反应堆，生还撤离。奖励 400 点。』",
        "张杰（低沉人声）：『这回是《异形4》，伙计们。老规矩——活着回来的人才有资格抱怨。……哦对，别碰那些卵。』",
        "登陆坞里，考尔（Call）、约翰纳、克里斯蒂、弗里茨四名船员正警惕地聚拢——他们是可能存活的队友。",
    ]),
    choices: &[
        ChoiceDef { label: "【检查装备（弹药补给）】", sub: "Weapon 手枪 · 主线继续", cond: None,
            effects: &[Eff::Weapon(Weapon::Gun)], route: Route::To("yiy_s1_hall") },
        ChoiceDef { label: "【问张杰：这世界有什么要命的规矩？】", sub: "San+2 · Points+5", cond: None,
            effects: &[Eff::San(2), Eff::Points(5)], route: Route::To("yiy_s0_arrive") },
        ChoiceDef { label: "【直接出舱】", sub: "前往船员区", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s1_hall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s1_hall", bg: Some("yiying_bg_open.png"), loc: Some("L1 船员生活区 · 主走廊"),
    mood: "danger", speaker: Some("Father（船载 AI）"), voice: Some("vo_yiying_father_quarantine"),
    text: TextSpec::Static(&[
        "你踏入船员区主走廊。头顶的公共广播骤然响起——冷冽的女声，带一丝失真：",
        "Father：『生命体征异常已隔离，甲板 4 号舱等待强制检疫。请全部成员前往 4 号舱，接受系统检测。』",
        "屏幕角落滚动着一份被划掉的名单——第一批『检疫者』已全部失联。餐厅方向，传来一声闷响。",
    ]),
    choices: &[
        ChoiceDef { label: "【去餐厅调查那声闷响】", sub: "第一具破尸", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s2_corpse") },
        ChoiceDef { label: "【顶舱通风管（需 Father 关停）】", sub: "g_yiy_vents_lock", cond: Some(cond_father_off),
            effects: &NO_EFF, route: Route::To("yiy_s_vents") },
        ChoiceDef { label: "【去电梯井下楼】", sub: "先下到 L2", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_elevator") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s2_corpse", bg: Some("yiying_bg_invest.png"), loc: Some("L1 · 餐厅 · 第一具破尸"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "餐厅中央倒着一具尸体。他的胸腔破开一个大洞，肋骨朝外翻着——有什么东西从里面顶了出来。",
        "就在这时，破洞深处涌出黏血——三只破胸体拖着湿漉漉的膜爬了出来。",
    ]),
    choices: &[
        ChoiceDef { label: "【先搜尸】", sub: "得医疗钥匙卡 yiy_key_med · San-5", cond: None,
            effects: &[Eff::AddItem("yiy_key_med"), Eff::San(-5)], route: Route::To("yiy_s2_fight") },
        ChoiceDef { label: "【夺路而逃】", sub: "San-10 · 破胸体紧随", cond: None,
            effects: &[Eff::San(-10)], route: Route::To("yiy_s2_fight") },
        ChoiceDef { label: "【迎战破胸体群】", sub: "f_yiy_chestburst", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s2_fight") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s2_fight", bg: Some("yiying_bg_invest.png"), loc: Some("L1 · 餐厅"),
    mood: "danger", speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["破胸体群扑咬过来——你举枪还击！"]),
    choices: &[ChoiceDef { label: "战斗", sub: "f_yiy_chestburst", cond: None, effects: &NO_EFF, route: Route::To("yiy_s2_win") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s2_win", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 餐厅"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "破胸体被逐一碾碎，黏液溅了满地。你喘息着站稳——这座船，是一个蜂巢。",
        "Father 的广播再度响起：『异常处理完毕。请继续前往 4 号舱。』",
    ]),
    choices: &[
        ChoiceDef { label: "【回主走廊】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s1_hall") },
        ChoiceDef { label: "【开医疗区门（需钥匙卡）】", sub: "g_yiy_med", cond: Some(cond_has_medkey),
            effects: &NO_EFF, route: Route::To("yiy_s_med_room") },
        ChoiceDef { label: "【去电梯井下楼】", sub: "先下到 L2", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_elevator") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_med_room", bg: Some("yiying_bg_invest.png"), loc: Some("L1 · 医疗区"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["医疗区的冷柜与器械泛着白光。这里似乎刚被搜掠过——但角落的器械柜还能用。"]),
    choices: &[
        ChoiceDef { label: "【搜刮器械柜】", sub: "San-5 · Points+10", cond: None,
            effects: &[Eff::San(-5), Eff::Points(10)], route: Route::To("yiy_s_med_room2") },
        ChoiceDef { label: "【回主走廊】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s1_hall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_med_room2", bg: Some("yiying_bg_invest.png"), loc: Some("L1 · 医疗区 · 器械柜"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["器械柜底部压着半张烧焦的检疫名单——第一批『检疫者』全被送进了 Father 的隔离舱，再没回来。对应的，Father 的散热协议住在船首。"]),
    choices: &[ChoiceDef { label: "【离开】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s_med_room") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_vents", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 顶舱通风管口"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["Father 已失联，通风管格栅弹开。你钻入黑暗的滑道——这是单向捷径，直通 L2 实验室后舱。"]),
    choices: &[ChoiceDef { label: "【滑下管道】", sub: "单向 → L2(34,22)", cond: None,
        effects: &NO_EFF, route: Route::To("yiy_s4_incubator") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_elevator", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 电梯井"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你按下电梯。电梯井的滑轮在静默中咔哒作响——一道幽蓝的光从舱门缝里漏出来。"]),
    choices: &[ChoiceDef { label: "【下到 L2】", sub: "双向电梯 L1→L2", cond: None, effects: &NO_EFF, route: Route::To("yiy_s_l2_arrive") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_l2_arrive", bg: Some("yiying_bg_battle.png"), loc: Some("L2 · 到达厅"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "L2 到达厅警报骤响！红灯旋转。Father 的广播尖利拔高：『非登记生命体征——立即前往 4 号舱强制检疫！』",
        "主控室（Father 核心）在你左侧深处，生物实验室与医疗舱在右侧。",
    ]),
    choices: &[
        ChoiceDef { label: "【进主控室（Father）】", sub: "g_yiy_lab", cond: Some(cond_gate_dyn),
            effects: &NO_EFF, route: Route::To("yiy_s3_father") },
        ChoiceDef { label: "【强启主控室门（需安保脉冲枪）】", sub: "g_yiy_lab", cond: Some(cond_has_pulse),
            effects: &[Eff::San(-10)], route: Route::To("yiy_s3_father") },
        ChoiceDef { label: "【进生物实验室】", sub: "寄生手术点 / 取样台", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s4_incubator") },
        ChoiceDef { label: "【检查到达厅安全柜】", sub: "安保脉冲枪", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_lab_security") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_lab_security", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 到达厅 · 安全柜"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["安全柜里有<b>安保脉冲枪</b>（yiy_pulse）——可强开 Father 主控室 / 巢穴门，对工兵有轻微伤害修正。"]),
    choices: &[
        ChoiceDef { label: "【取走安保脉冲枪】", sub: "yiy_pulse", cond: None,
            effects: &[Eff::AddItem("yiy_pulse")], route: Route::To("yiy_s_l2_arrive") },
        ChoiceDef { label: "【不要】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s_l2_arrive") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_lab_chest", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 生物实验室 · 物资箱"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["物资箱里静静躺着一台<b>电磁束缚装置</b>（yiy_em_restraint）——皇后战的二段终结技。"]),
    choices: &[
        ChoiceDef { label: "【取走电磁束缚装置】", sub: "yiy_em_restraint", cond: None,
            effects: &[Eff::AddItem("yiy_em_restraint")], route: Route::To("yiy_s4_incubator") },
        ChoiceDef { label: "【不要】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s4_incubator") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 1 · 关键转折 A：「Father 的低语」 ---- */
SceneDef {
    id: "yiy_s3_father", bg: Some("yiying_bg_battle.png"), loc: Some("L2 · 主控室（Father 核心）"),
    mood: "danger", speaker: Some("Father（AI）"), voice: Some("vo_yiying_father_material"),
    text: TextSpec::Static(&[
        "主控室的幽蓝屏幕墙铺展开来，冷冽的女声从四面响起：Father：『你们不是我的检疫对象。你们是……胁制品。配合调查，可获豁免。』",
        "屏幕角落滚动着被划掉的名单——那批『检疫者』其实被 Father 用作异形的育卵宿主。考尔低声：「它把我们当耗材，分批送进隔离舱。……别听它的。」",
        "屏幕中央浮现一条脆弱的退路：Father 的散热协议可以在这台终端上被伏击式关闭——那也会让全船所有被它握持的门禁与管道锁同时失效。",
    ]),
    choices: &[
        ChoiceDef { label: "【假装顺从，诱导 Father 打开隔离舱门（实为关闭其散热协议）】", sub: "yiy_father_off · Points+100 · 智慧路线", cond: None,
            effects: &[Eff::SetFlag("yiy_father_off"), Eff::Points(100)], route: Route::Dyn(father_off_run) },
        ChoiceDef { label: "【强行物理断电（安保脉冲枪）】", sub: "需 yiy_pulse · San-10 · 粗暴路线（少 100 点）", cond: Some(cond_has_pulse),
            effects: &[Eff::SetFlag("yiy_father_off"), Eff::San(-10)], route: Route::Dyn(father_off_run) },
        ChoiceDef { label: "【转身就跑，不作判定】", sub: "yiy_miss_father · 危险", cond: None,
            effects: &[Eff::MarkPoint("yiy_miss_father"), Eff::San(-8)], route: Route::Dyn(father_flee_run) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s3_after", bg: Some("yiying_bg_battle.png"), loc: Some("L2 · 主控室"),
    mood: "cold", speaker: Some("Father（AI）"), voice: None,
    text: TextSpec::Static(&[
        "Father 的冷冽女声霎时断成一片杂音，屏幕化为雪花，散热协议的伺服器发出最后一声频率衰竭的低鸣。",
        "考尔长舒一口气：「……成了。它的控制网断线了——通风管、孵化室、引擎桥的门禁都会陆续失压。」",
        "<b>（达成支线 flag `yiy_father_off`：解锁通风管捷径 / 孵化门 / 管道过热熔毁终结前置。）</b>",
    ]),
    choices: &[ChoiceDef { label: "【前往孵化室】", sub: "g_yiy_incubator 已开", cond: None,
        effects: &NO_EFF, route: Route::To("yiy_s4_incubator") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s3_flee", bg: Some("yiying_bg_open.png"), loc: Some("L2 · 主控室外走廊"),
    mood: "danger", speaker: Some("考尔"), voice: None,
    text: TextSpec::Static(&["你撤出主控室。Father 的封锁警报回荡——门禁都需脉冲枪强启（或硬闯）。考尔皱眉：「没关掉它，我们走哪都是『待检疫』。但先把普维斯的事办了也行。」"]),
    choices: &[
        ChoiceDef { label: "【去生物实验室】", sub: "", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s4_incubator") },
        ChoiceDef { label: "【硬闯孵化室】", sub: "遇防空炮 · 失躲则死", cond: None,
            effects: &NO_EFF, route: Route::Dyn(father_flee_incubator) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 2 · 关键转折 B：「孵化室的真相」（寄生/巢穴核心） ---- */
SceneDef {
    id: "yiy_s4_incubator", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 孵化室（卵区）"),
    mood: "danger", speaker: Some("普维斯·被寄生者"), voice: Some("vo_yiying_puvisi"),
    text: TextSpec::Static(&[
        "无数卵苞后仰张开，黏液中浮现出抱脸虫干燥的手指。普维斯突然捂住胸口——肋骨折断的脆响在静默中格外清晰。",
        "普维斯（干呕与黏声）：『帮……帮我……它在我胸里动了——』考尔拔枪：「他已经是计时炸弹了。」",
    ]),
    choices: &[
        ChoiceDef { label: "【带他冲医疗舱取样（宿主生还路线）】", sub: "yiy_infected · 启动寄生倒计时", cond: None,
            effects: &[Eff::SetFlag("yiy_infected"), Eff::SetFlag("yiy_parasite_1")],
            route: Route::To("yiy_s5_med") },
        ChoiceDef { label: "【就地了结他（防破胸）】", sub: "San-15 · Points+30 · 考尔信任 -1", cond: None,
            effects: &[Eff::San(-15), Eff::Points(30), Eff::KillTeam("purvis")], route: Route::To("yiy_s5_nest") },
        ChoiceDef { label: "【丢下他撤离】", sub: "yiy_abandon · 结算降档", cond: None,
            effects: &[Eff::SetFlag("yiy_abandon"), Eff::San(-8)], route: Route::To("yiy_s5_nest") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s5_med", bg: Some("yiying_bg_open.png"), loc: Some("L2 · 通往医疗舱的走廊"),
    mood: "danger", speaker: Some("普维斯"), voice: None,
    text: TextSpec::Dyn(|st| {
        format!("你扶住蜷缩的普维斯往医疗舱赶。他额头全是冷汗，胸腔每隔几秒就顶起一次。考尔催促：「快。倒计时在走。」（寄生幕数 {}/3）", parasite_stage(st))
    }),
    choices: &[
        ChoiceDef { label: "【直奔医疗舱取样台】", sub: "急行 · 优先采样", cond: None,
            effects: &NO_EFF, route: Route::Dyn(go_medlab) },
        ChoiceDef { label: "【先安顿一下再走（多耗一幕）】", sub: "寄生倒计时 +1 幕", cond: None,
            effects: &NO_EFF, route: Route::Dyn(bide_medlab) },
        ChoiceDef { label: "【掉头去烧巢点（先清巢）】", sub: "倒计时 +1 幕 · 可选战", cond: None,
            effects: &NO_EFF, route: Route::Dyn(detour_burn) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_medlab", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 医疗舱 · 取样台"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yiy_cured") {
            "你已经完成取样。普维斯虚弱地靠在椅子上，胸腔不再起伏——他暂时安全了。".to_string()
        } else {
            format!("取样台就在眼前。寄生幕数 {}/3。（取样可救普维斯）", parasite_stage(st))
        }
    }),
    choices: &[
        ChoiceDef { label: "【取样（救普维斯）】", sub: "yiy_cured · yiy_embryo_sample · Points+120 · 支线C", cond: None,
            effects: &[Eff::SetFlag("yiy_cured"), Eff::AddItem("yiy_embryo_sample"), Eff::Points(120)],
            route: Route::Dyn(sample_run) },
        ChoiceDef { label: "【先不做，稍后】", sub: "返回孵化室", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s5_nest") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s5_nest", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 孵化室 · 卵区边缘"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你退到卵区边缘。烧巢调查点就在卵群核心——如需，可用喷灯把它们连片焚毁。",
        "约翰纳低声：「孵化室后面有个大货梯，貌似能直达引擎桥下面。」",
    ]),
    choices: &[
        ChoiceDef { label: "【去烧巢点】", sub: "可选战 f_yiy_queenhold · 烧巢+100", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_nest_fire") },
        ChoiceDef { label: "【回到达厅】", sub: "", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_l2_arrive") },
        ChoiceDef { label: "【乘货运电梯下 L3】", sub: "双向电梯 L2→L3", cond: None,
            effects: &NO_EFF, route: Route::Dyn(go_l3) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_nest_fire", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 卵区 · 烧巢点"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["喷灯的蓝焰喷向最近的卵苞。卵壳在灼热中爆开，抱脸虫群嘶叫着扑了出来——烧巢清剿战。"]),
    choices: &[ChoiceDef { label: "【战斗】", sub: "f_yiy_queenhold · 胜得烧巢 flag", cond: None,
        effects: &NO_EFF, route: Route::To("yiy_s_burn_do") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_burn_do", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 卵区"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["火焰顺着卵壳连成一片，整片卵区在灼热中坍缩。你退到卵区外——<b>干净利落</b>。"]),
    choices: &[ChoiceDef { label: "【乘货运电梯下 L3】", sub: "双向电梯 L2→L3", cond: None,
        effects: &NO_EFF, route: Route::Dyn(go_l3) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_sample", bg: Some("yiying_bg_invest.png"), loc: Some("L2 · 取样台"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["取样台：这是救下被寄生者的手术点。"]),
    choices: &[ChoiceDef { label: "（前往医疗舱）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s_medlab") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_airlock", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 气闸区"),
    mood: "danger", speaker: Some("考尔"), voice: None,
    text: TextSpec::Static(&[
        "你听到身后传来工兵的气门声。眼前就是双气闸——把异形引进去，一键真空抛射，能干净利落地清掉它。",
    ]),
    choices: &[
        ChoiceDef { label: "【引工兵进气闸 · 真空抛射】", sub: "Points+80 · 环境击杀", cond: None,
            effects: &[Eff::Points(80), Eff::SetFlag("yiy_airlock_kill")], route: Route::To("yiy_s_airlock_ok") },
        ChoiceDef { label: "【错位误触气闸】", sub: "失败 · Hurt(10) · San-10 · 误伤队友", cond: None,
            effects: &[Eff::Hurt(10, "yiy_dead_airlock"), Eff::San(-10), Eff::KillTeam("call")], route: Route::Dyn(airlock_fail) },
        ChoiceDef { label: "【不理会，绕行】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s1_hall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_airlock_ok", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 气闸区"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["气闸在真空中噗地闭合，那道黑影被抛入星海的迷茫里。干净利落。"]),
    choices: &[ChoiceDef { label: "【回主走廊】", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s1_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 3 · 皇后决战 ---- */
SceneDef {
    id: "yiy_s_l3_arrive", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 电梯到达厅（引擎层）"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "电梯门在 L3 引擎层开启，幽蓝的反物质反应堆辉光与警报红闪交织。船首深处传来耳膜发麻的巨大心跳——皇后在巢穴里。",
        "引擎桥（引爆总闸）在你左侧，反应堆管道区横亘在中线，皇后巢穴在更深处的卵房里。",
    ]),
    choices: &[
        ChoiceDef { label: "【去引擎桥（引爆总闸）】", sub: "g_yiy_reactor", cond: Some(cond_father_off),
            effects: &NO_EFF, route: Route::To("yiy_s7_evac") },
        ChoiceDef { label: "【进反应堆管道区（BOSS 终结技触发点）】", sub: "yiy_z_pipe", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_pipe") },
        ChoiceDef { label: "【直闯皇后巢穴】", sub: "g_yiy_queen", cond: Some(cond_gate_dyn),
            effects: &NO_EFF, route: Route::To("yiy_s_queen_pre") },
        ChoiceDef { label: "【强启巢穴闸门（需脉冲枪）】", sub: "g_yiy_queen 强闯", cond: Some(cond_has_pulse),
            effects: &[Eff::San(-8)], route: Route::To("yiy_s_queen_pre") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_pipe", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 反应堆管道区"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yiy_father_off") {
            "冷却管道横贯整片区域。Father 已失联，散热协议失效——这是把皇后引向过热熔毁的最佳位置。".to_string()
        } else {
            "冷却管道正常工作，散热协议尚在 Father 手中。贸然靠近管道只会让自己先被高温蒸汽吞没。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【主动挑衅皇后（引出巢穴）】", sub: "把皇后引向管道", cond: Some(cond_queen_pipe),
            effects: &[Eff::SetFlag("yiy_queen_pipe")], route: Route::To("yiy_s_queen_pre") },
        ChoiceDef { label: "【先上引擎桥引爆总闸】", sub: "g_yiy_reactor", cond: Some(cond_father_off),
            effects: &NO_EFF, route: Route::To("yiy_s7_evac") },
        ChoiceDef { label: "【放弃，先去巢穴硬战】", sub: "不做环境终结", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_queen_pre") },
        ChoiceDef { label: "【直接扑向管道（未关 Father）】", sub: "危险：高温蒸汽灭团", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|_st| "yiy_dead_pipe".to_string()) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_queen_pre", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 皇后巢穴 · 卵房"),
    mood: "danger", speaker: Some("考尔"), voice: None,
    text: TextSpec::Static(&[
        "卵房的骨白穹顶下，皇后从阴影里缓缓抬起骨冠——它已经注意到你了。腹囊蠕动，卵苞在四周依次张开。",
        "考尔：「两个方法——要么把它引进管道用 Father 失效的过热区烧穿它；要么用电磁束缚装置把它电瘫，重火力硬拆。」",
    ]),
    choices: &[
        ChoiceDef { label: "【释放电磁束缚 · 重火力】", sub: "需 yiy_em_restraint", cond: Some(cond_queen_em),
            effects: &[Eff::SetFlag("yiy_queen_em"), Eff::San(-5)], route: Route::To("yiy_s_queen_start") },
        ChoiceDef { label: "【引向管道 · 过热熔毁】", sub: "需 yiy_father_off（环境终结）", cond: Some(cond_queen_pipe),
            effects: &[Eff::SetFlag("yiy_queen_pipe")], route: Route::To("yiy_s_queen_start") },
        ChoiceDef { label: "【直接强杀（无终结加成）】", sub: "硬打 · 需防酸血", cond: None,
            effects: &NO_EFF, route: Route::To("yiy_s_queen_start") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_queen_start", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 皇后巢穴"),
    mood: "danger", speaker: Some("⚔ BOSS 战"), voice: Some("vo_yiying_queen_roar"),
    text: TextSpec::Static(&["皇后尖啸着扑来——决战开始！"]),
    choices: &[ChoiceDef { label: "【进入决战】", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_queen_fight) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_queen_round", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 皇后巢穴 · 决战"),
    mood: "danger", speaker: Some("⚔ 异形皇后"), voice: None,
    text: TextSpec::Dyn(txt_queen_round),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 34-46 · 易露破绽", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, QueenAction::Heavy)) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 22-30 · 稳", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, QueenAction::Combo)) },
        ChoiceDef { label: "【电磁束缚 · 重火力（×1.5）】", sub: "需装置 · 皇后瘫痪 1 回合", cond: Some(cond_queen_em), effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, QueenAction::Restraint)) },
        ChoiceDef { label: "【引向管道 · 过热熔毁】", sub: "需 Father 关停 · 环境终结", cond: Some(cond_queen_pipe), effects: &NO_EFF, route: Route::Dyn(|st| queen_act(st, QueenAction::Pipe)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_queen_win", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 皇后巢穴"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["皇后坠地，巢穴骤然安静。酸血在它周围嘶嘶蚀开，你小心避让。"]),
    choices: &[ChoiceDef { label: "【打扫战场】", sub: "", cond: None,
        effects: &NO_EFF, route: Route::Dyn(route_queen_reward) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 4 · 结局：「引爆与撤离」 ---- */
SceneDef {
    id: "yiy_s7_evac", bg: Some("yiying_bg_battle.png"), loc: Some("L3 · 引擎控制桥 · 引爆总闸"),
    mood: "danger", speaker: Some("主神"), voice: Some("vo_yiying_father_evac"),
    text: TextSpec::Static(&[
        "引爆总闸前，Father（残响）最后一次发声：『引爆将摧毁 7 层生命维持。确认。』",
        "主神光柱在贝蒂号方向亮起：『主线目标 3/3。撤离倒计时开始。』皇后尖啸从通风管深处追来……",
        "你必须在倒计时归零前冲上贝蒂号对接舱（34,22）。",
    ]),
    choices: &[
        ChoiceDef { label: "【直奔对接舱（满速撤离）】", sub: "yiy_evac · 寄生未处理则破胸打断", cond: None,
            effects: &[Eff::SetFlag("yiy_evac")], route: Route::Dyn(|st| { st.set_flag("yiy_rush"); "yiy_s8_evac".to_string() }) },
        ChoiceDef { label: "【反手给皇后补一刀（若未终结）】", sub: "需 !yiy_queen_final · Points+100", cond: Some(cond_not_queen_final),
            effects: &[Eff::Points(100), Eff::SetFlag("yiy_queen_final")], route: Route::Dyn(|st| { let dead = parasite_tick(st); if dead.is_empty() { "yiy_s8_evac".into() } else { dead } }) },
        ChoiceDef { label: "【返回主控室摧毁 Father 核心数据】", sub: "需 yiy_father_off · Points+80 · San+6 · 撤离 -1 幕", cond: Some(cond_father_off),
            effects: &[Eff::Points(80), Eff::San(6)], route: Route::To("yiy_s7_cleanup") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s7_cleanup", bg: Some("yiying_bg_battle.png"), loc: Some("L2 · 主控室 · Father 核心"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["你用最后的时间返回主控室，把 Father 的罪证与散热数据一并格式化。屏幕最后闪出一行无意义的乱码，然后熄灭。倒计时只剩最后一格——"]),
    choices: &[ChoiceDef { label: "【冲向对接舱】", sub: "撤离（已耗 1 幕）", cond: None, effects: &NO_EFF, route: Route::To("yiy_s8_evac") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s8_evac", bg: Some("yiying_bg_open.png"), loc: Some("L3 · 贝蒂号对接舱 (34,22)"),
    mood: "danger", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["贝蒂号对接舱的舷梯在你脚下。倒计时蜂鸣逼近临界……"]),
    choices: &[
        ChoiceDef { label: "【登上贝蒂号】", sub: "主线完成 · 撤离成功", cond: None,
            effects: &[Eff::SetFlag("yiy_final"), Eff::Points(400)], route: Route::Dyn(|_st| "yiy_settle".to_string()) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 胜利收尾场景（酸血 / 工兵奖励 / 烧巢）· win 回调不可变，副作用经 Route::Dyn 落地 ---- */
SceneDef {
    id: "yiy_win_acid", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["异形成体被你击杀。在你收枪的瞬间，酸血在地板上嘶嘶蚀开，溅上你的鞋底——"]),
    choices: &[ChoiceDef { label: "【甩开酸血】", sub: "Hurt(4) 一次性结算", cond: None,
        effects: &[Eff::Hurt(4, "yiy_dead_nest")], route: Route::To("yiy_win_card") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_win_worker", bg: None, loc: None, mood: "cold", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yiy_father_off") {
            "你已关停 Father——这只工兵『失联』般迟缓了许多（+20 点数）。".to_string()
        } else {
            "你击退了巡逻工兵。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "【离开】", sub: "若 Father 已关则 +20", cond: None,
        effects: &[Eff::PointsIfFlag("yiy_father_off", 20)], route: Route::To("yiy_win_card") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_win_burn", bg: None, loc: None, mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["清剿完成，卵区一片狼藉。你点起喷灯——", ]),
    choices: &[ChoiceDef { label: "【焚毁蜂巢】", sub: "yiy_burn_nest · +100", cond: None,
        effects: &[Eff::SetFlag("yiy_burn_nest"), Eff::Points(100)], route: Route::To("yiy_win_card") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 结算卡（sp_grade = C） / 战斗后卡片 ---- */
SceneDef {
    id: "yiy_win_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "战 斗 结 束".into(), good: true,
            body_html: format!("<p>异形被击退，你拖着一身酸血腥味继续向前。</p><p style='color:#9a958a'>当前剩余点数：{}</p>", st.points),
            buttons: vec![("继 续 ▶".into(), "__continue__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_queen_win_card2", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "皇 后 已 讫".into(), good: true,
            body_html: format!(
                "<table class='statTable'>\
                 <tr><td>奖励点数（击败异形皇后）</td><td>+{}</td></tr>\
                 <tr><td>支线剧情评级</td><td style='color:#ffd76a'>C 级</td></tr>\
                 <tr><td>科技道具</td><td>高斯图纸 yiy_gauss_blueprint</td></tr>\
                 </table><p style='color:#8fd0a8'>剩余点数：{}</p>",
                REWARD_QUEEN_BASE, st.points),
            buttons: vec![("继 续 ▶".into(), "__continue__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_settle", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "任 务 完 成".into(), good: true,
            body_html: format!(
                "<p>贝蒂号挣脱奥瑞迦号，身后爆出一团反物质辉光。你活着离开了这艘蜂巢之船。</p>\
                 <p style='color:#ffd76a'><b>支线剧情评级：C 级</b>（解锁中级吸血鬼血统 / 高斯手枪 / 抗酸涂层兑换门槛）</p>\
                 <table class='statTable'>\
                 <tr><td>奖励点数</td><td>{}</td></tr>\
                 <tr><td>存活队友</td><td>{}</td></tr>\
                 <tr><td>支线达成</td><td>{} 条</td></tr>\
                 </table>\
                 <p style='color:#ffd76a'>詹岚（远处）：下一个世界更暗——矿坑里的鼓声。</p>\
                 <p style='color:#9a958a'>张杰：B 级难度的里程碑在等着你。</p>",
                st.points, st.alive_count(), count_subqueries(st)),
            buttons: vec![("进 入 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- NPC 对话 ---- */
SceneDef {
    id: "yiy_s_father_npc", bg: Some("yiying_bg_battle.png"), loc: Some("L2 · 主控室 · Father 终端"),
    mood: "danger", speaker: Some("Father（AI）"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yiy_father_off") {
            "终端一片死寂，屏幕残留着格式化的残影。Father 已经不复存在。".to_string()
        } else {
            "Father 冷冽地重复检疫指令。你注意到屏幕角落那行小字：『散热协议占用核心进程』。".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "（回到主控室）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s3_father") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_call", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 登陆坞 · 考尔"),
    mood: "cold", speaker: Some("考尔（Call）"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("yiy_host_survive") {
            "考尔点点头：「你把那个被寄生的人救下来了。这船上的『人味』，比仪器管用。」".to_string()
        } else if st.flag("yiy_abandon") {
            "考尔别过脸去，沉默。她大概觉得你丢了普维斯。".to_string()
        } else {
            "考尔：「记住——Father 之外，还有别的东西在船里。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "（继续）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s1_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "yiy_s_johnner", bg: Some("yiying_bg_open.png"), loc: Some("L1 · 登陆坞 · 约翰纳"),
    mood: "cold", speaker: Some("约翰纳"), voice: None,
    text: TextSpec::Static(&["约翰纳拍拍枪：「贝蒂号在那边。只要能活着到那艘飞船，老子今天就能脱身。」"]),
    choices: &[ChoiceDef { label: "（继续）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("yiy_s1_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 死亡档案（6 种）：
       摇篮曲 / 巢穴养分 / 气闸歉意 / 父的裁决 / 硫磺与蒸汽 / 暗角 ---- */
SceneDef {
    id: "yiy_dead_parasite", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("摇篮曲", "寄生倒计时超时，破胸体破胸而出")),
        card: |_st| crate::state::Card {
            title: "摇 篮 曲".into(), good: false,
            body_html: "<p>第 3 幕仍未取样。你（或普维斯）在走廊上突然僵住——胸腔里，那个不该存在的<b>心跳</b>骤然加速。</p>\
                 <p style='color:#ff8a8a'>【死亡档案 · 摇篮曲】破胸体破胸而出，你在那阵撕开的脆响中结束。</p>\
                 <p style='color:#666'>（复活：回主神空间扣 600 点，由主线主神复活系统接线。）</p>".to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_dead_nest", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("巢穴的养分", "BOSS 战被皇后拖入卵房，成为育卵宿主")),
        card: |_st| crate::state::Card {
            title: "巢 穴 的 养 分".into(), good: false,
            body_html: r#"<p>皇后的尾刺横扫，你被掀翻在地。几只抱脸虫抓住你的眼眶、喉咙、胸口——卵被送进你的体内。</p>
<p style='color:#ff8a8a'>【死亡档案 · 巢穴的养分】你成了这座蜂巢最新一块育卵宿主。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_dead_airlock", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("气闸的歉意", "误触气闸，被抛入真空")),
        card: |_st| crate::state::Card {
            title: "气 闸 的 歉 意".into(), good: false,
            body_html: r#"<p>你一步踏错了节奏。气闸内层在错位瞬间关闭，外层随即开启——你（或队友）被真空抛入星海，连声音都来不及发出。</p>
<p style='color:#ff8a8a'>【死亡档案 · 气闸的歉意】飘向群星的一个微不足道的错误。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_dead_father", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("父的裁决", "顺从 AI 进入隔离舱，被防空炮射界点名")),
        card: |_st| crate::state::Card {
            title: "父 的 裁 决".into(), good: false,
            body_html: r#"<p>Father 的隔离舱门在你身后合拢。你才意识到「强制检疫」意味着什么——射线从舱壁的暗槽里转出，把你困进防空炮的射界。</p>
<p style='color:#ff8a8a'>【死亡档案 · 父的裁决】你没有顺从它，却也未曾真正反抗它。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_dead_pipe", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("硫磺与蒸汽", "反应堆熔毁波及，或管道高温蒸汽吞没")),
        card: |_st| crate::state::Card {
            title: "硫 磺 与 蒸 汽".into(), good: false,
            body_html: r#"<p>Father 的散热协议仍在运转。管道在高温下失控，足以蚀穿船体的硫磺蒸汽与等离子洪流将你裹住——你没能活着离开这艘船。</p>
<p style='color:#ff8a8a'>【死亡档案 · 硫磺与蒸汽】倒计时归零，蜂巢与你一同沉入熔毁。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "yiy_dead_hunter", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("暗角", "在通风管中被猎手腰斩伏击")),
        card: |_st| crate::state::Card {
            title: "暗 角".into(), good: false,
            body_html: r#"<p>你以为这段管道是安全的捷径。直到那四道超长的四肢从黑暗里无声垂下——一抹暗红色的瞳孔，在你后颈处亮起。</p>
<p style='color:#ff8a8a'>【死亡档案 · 暗角】猎手在通风管里等你，早已多时。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
];

/* =====================================================================
   路由函数（fn 指针，供 Route::Dyn / cond 使用）
   ===================================================================== */
fn father_off_run(st: &mut GameState) -> String {
    st.set_flag("yiy_father_off");
    "yiy_s3_after".into()
}
fn father_flee_run(st: &mut GameState) -> String {
    // 未关 Father：进入隔离舱有概率被防空炮点名
    if rnd(1, 3) == 1 {
        st.set_flag("yiy_father_no_off");
        "yiy_dead_father".into()
    } else {
        "yiy_s3_flee".into()
    }
}
fn father_flee_incubator(st: &mut GameState) -> String {
    if rnd(1, 4) == 1 { "yiy_dead_father".into() } else { st.set_flag("yiy_father_no_off"); "yiy_s4_incubator".into() }
}
fn go_medlab(st: &mut GameState) -> String {
    let dead = parasite_tick(st);
    if dead.is_empty() { "yiy_s_medlab".into() } else { dead }
}
fn bide_medlab(st: &mut GameState) -> String {
    let dead = parasite_tick(st);
    if dead.is_empty() { "yiy_s5_med".into() } else { dead }
}
fn detour_burn(st: &mut GameState) -> String {
    let dead = parasite_tick(st);
    if dead.is_empty() { "yiy_s_nest_fire".into() } else { dead }
}
fn go_l3(st: &mut GameState) -> String {
    let dead = parasite_tick(st);
    if dead.is_empty() { "yiy_s_l3_arrive".into() } else { dead }
}
fn sample_run(st: &mut GameState) -> String {
    st.set_flag("yiy_cured");
    st.set_flag("yiy_host_survive");
    "yiy_s5_nest".into()
}
fn airlock_fail(st: &mut GameState) -> String {
    if st.hp <= 0 { "yiy_dead_airlock".into() } else { "yiy_s_airlock_ok".into() }
}

/* =====================================================================
   BOSS 战（皇后）· 选择驱动
   ===================================================================== */
pub const REWARD_QUEEN_BASE: i32 = 450;
const QUEEN_MAX_HP: i32 = 200;

#[derive(Clone, Copy, PartialEq)]
enum QueenAction { Heavy, Combo, Restraint, Pipe }

fn txt_queen_round(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(QUEEN_MAX_HP);
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let danger = if raged {
        "<b style='color:#ff6a6a'>产卵暴走：酸血四溅（每回合蚀伤 3），尾刺范围攻击，每 2 回合增员一只抱脸虫。</b>"
    } else { "" };
    let plan = if st.flag("yiy_queen_pipe") {
        "  <span style='color:#8fd0a8'>终结预案：<b>管道过热熔毁</b></span>"
    } else if st.flag("yiy_queen_em") {
        "  <span style='color:#8fd0a8'>终结预案：<b>电磁束缚 · 重火力</b></span>"
    } else { "" };
    format!(
        "皇后盘踞在卵房中央，骨冠晶亮的边缘滑过暗红的光。\n\n<b>异形皇后</b>　HP {hp}/{QUEEN_MAX_HP}\n{danger}{plan}"
    )
}

fn start_queen_fight(st: &mut GameState) -> String {
    if st.fight.is_none() {
        let cfg = queen_cfg();
        st.fight = Some(crate::power::scaled_fight("f_yiy_queen", cfg, st, vec![]));
    }
    apply_acid(st);
    if st.hp <= 0 { return "yiy_dead_nest".to_string(); }
    "yiy_queen_round".to_string()
}

fn queen_raged(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.hp <= (QUEEN_MAX_HP * 35 / 100)).unwrap_or(false)
}

fn queen_act(st: &mut GameState, act: QueenAction) -> String {
    use QueenAction::*;
    apply_acid(st);

    // 管道过热熔毁（环境终结）：需 Father 关停（cond 已保证）
    if act == Pipe {
        st.set_flag("yiy_queen_final");
        st.set_flag("yiy_queen_plan");
        st.points += REWARD_QUEEN_BASE;
        give_queen_tech(st);
        st.sp_grade = Some('C');
        st.fight = None;
        return "yiy_queen_win".to_string();
    }

    // 玩家伤害
    let base = match act {
        Heavy => rnd(34, 46),
        Combo => rnd(22, 30),
        Restraint => { st.set_flag("yiy_queen_restrained"); rnd(30, 40) }
        Pipe => unreachable!(),
    };
    // 电磁束缚重火力：直接释放当回合 ×1.5；其后硬直窗口两回合玩家 ×1.5
    let boost = if act == Restraint || st.flag("yiy_queen_restrained") {
        (base as f32 * 1.5) as i32
    } else { base };

    if let Some(f) = st.fight.as_mut() {
        f.hp = (f.hp - boost).max(0);
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        return queen_reward_route(st);
    }

    // 皇后反击
    let q_dmg = if queen_raged(st) { rnd(18, 26) } else { rnd(16, 24) };
    st.hp = (st.hp - q_dmg).max(0);
    if st.hp <= 0 { return "yiy_dead_nest".to_string(); }

    // 狂暴产卵暴走：每 2 回合增员 + 酸血溅射
    if queen_raged(st) {
        // 用两个 toggle flag 模拟偶数回合增员
        let even = if !st.flag("yiy_queen_nest_a") { st.set_flag("yiy_queen_nest_a"); false }
                   else { st.set_flag("yiy_queen_nest_b"); true };
        if even {
            st.hp = (st.hp - 3).max(0);
            st.san = (st.san - 3).clamp(0, 100);
        }
        if st.hp <= 0 { return "yiy_dead_nest".to_string(); }
    }
    // 更新 raged
    if let Some(f) = st.fight.as_mut() {
        f.raged = f.hp <= (QUEEN_MAX_HP * 35 / 100);
    }
    "yiy_queen_round".to_string()
}

fn queen_reward_route(st: &mut GameState) -> String {
    st.set_flag("yiy_queen_final");
    st.set_flag("yiy_queen_down");
    st.points += REWARD_QUEEN_BASE;
    give_queen_tech(st);
    st.sp_grade = Some('C');
    st.fight = None;
    "yiy_queen_win".to_string()
}

fn route_queen_reward(st: &mut GameState) -> String {
    // 奖励已在 queen_reward_route / Pipe 分支发放，此处仅呈现结算卡片（避免重复入账）
    st.fight = None;
    "yiy_queen_win_card2".to_string()
}

fn give_queen_tech(st: &mut GameState) {
    if !st.inventory.iter().any(|i| i == "yiy_gauss_blueprint") && !st.inventory.iter().any(|i| i == "yiy_acid_sample") {
        crate::world::add_item(st, "yiy_gauss_blueprint");
    }
}

fn count_subqueries(st: &GameState) -> i32 {
    ["yiy_trust_call", "yiy_father_off", "yiy_queen_plan", "yiy_host_survive", "yiy_burn_nest"]
        .iter().filter(|k| st.flag(k)).count() as i32
}

/// 本文件场景查询辅助（主线合并查询扩展时可直接使用）
pub fn yiy_scene(id: &str) -> Option<&'static SceneDef> {
    YIYING_SCENES.iter().find(|s| s.id == id)
}
//! 《魔戒·摩瑞亚矿坑》任务世界 · 全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/wuxian_kongbu/moruiya.md §4/§5/§6/§7/§8。
//! 本文件为全新新增文件，只导出静态数据（MORUIYA_SCENES / moruiya_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/moruiya_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `mo_` 前缀，与既有 SCENES 无重名。
//! BOSS 战「持续 SAN 光环」（火焰形态每回合 San 扣减 + 全场高温 San(-3)/回）因引擎 FightCfg 无每回合同调钩子，
//! 用「选择驱动遭遇链」落地（同咒怨作法）：mo_boss_round 逐回调 Route::Dyn 扣 San；同时导出 `b_balrog`
//! FightCfg 供 ZoneDef 引擎直战复用。
//! 双结局终结技（断桥坠渊 vs 甘道夫式牺牲）用 Route::Dyn + 互斥 SetFlag（mo_side_survive / mo_side_sacrifice /
//! mo_balrog_down / mo_sacrifice_done），结算用 Eff::PointsIfFlag 计支线分；甘道夫式牺牲经 Route::Dyn
//! 写 `st.sp_grade = Some('B')`（B 级支线）。
//! 火把/光照按文档 §10 最小改动退路实现：纯 flag + 文本降级（mo_torch_lit + Eff::San），不引入网格光照。
//! 坠落陷阱用 PointDef 调查场景 + Eff::Hurt(amount, death_route) → 死亡档案（mo_death_stair）。
//!
//! 注意：TextSpec::Dyn 只负责渲染（只读），不得在其中产生副作用（会重复触发）；所有 flag/点数副作用
//! 一律落在 Route::Dyn 或 ChoiceDef.effects 里。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   条件谓词（cond）—— 具名 fn，不能捕获闭包（CondFn 为 fn 指针）
   ===================================================================== */
fn cond_has_rune(st: &GameState) -> bool { st.flag("mo_rune_decoded") }
fn cond_rune_undone(st: &GameState) -> bool { !st.flag("mo_rune_decoded") }
fn cond_book_undone(st: &GameState) -> bool { !st.flag("mo_book_read") }
fn cond_has_mithril_key(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "mithril_key") }
/// 尚未取走秘银钥匙石（供密室宝箱"收下"选项显示）
fn cond_no_mithril_key(st: &GameState) -> bool { !st.inventory.iter().any(|i| i == "mithril_key") }
/// 断桥坠渊可用条件：甘道夫存活 && 炎魔 HP<30%（<81）
fn cond_breachable(st: &GameState) -> bool {
    st.flag("mo_gandalf_alive")
        && st.fight.as_ref().map(|f| f.hp < 81).unwrap_or(false)
}
/// 甘道夫式牺牲可用（任意阶段，需已开战）
fn cond_sacrifice(st: &GameState) -> bool {
    st.flag("mo_gandalf_alive") && st.fight.is_some()
}
/// 东门可推（炎魔任一结局后）
fn cond_cleared(st: &GameState) -> bool { st.flag("mo_cleared") }
/// 炎魔是否已狂暴（火焰形态，HP≤50%=135）
fn balrog_raged(st: &GameState) -> bool {
    st.fight.as_ref().map(|f| f.hp <= 135).unwrap_or(false)
}

/* =====================================================================
   随机
   ===================================================================== */
fn rnd_mo(a: i32, b: i32) -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(a..=b)
}

/* =====================================================================
   BOSS 战 · 选择驱动遭遇（卡扎督姆桥 · 炎魔）
   每"回"是 Normal 场景 mo_boss_round；Route::Dyn 统一处理：扣炎魔血、火焰形态 San 光环、
   高温 San 光环、双结局路由、胜负路由。
   ===================================================================== */
/// 从 b_balrog 的 FightCfg 建 Fight（无论是否已有残战都重建，保证桥战为全新炎魔实例）
fn start_balrog(st: &mut GameState) -> String {
    if let Some(cfg) = crate::scenes::fight_cfg("b_balrog") {
        st.fight = Some(crate::power::scaled_fight("b_balrog", cfg, st, vec![]));
    }
    st.set_flag("mo_gandalf_alive");
    st.san = (st.san - 3).clamp(0, 100); // 进战斗即全场高温 San-3（§5）
    "mo_boss_round".to_string()
}

/// 一个"回"：玩家攻击炎魔（dmg=0 视为后撤重整只走光环），处理 San 光环 + 反击 + 双结局/胜负。
fn route_boss_attack(st: &mut GameState, dmg: i32) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(270);
    let raged = balrog_raged(st);
    // 持续 SAN 光环：全场高温 in-fight 即生效 San-3；火焰形态额外 San-6
    st.san = (st.san - 3).clamp(0, 100);
    if raged {
        st.san = (st.san - 6).clamp(0, 100);
    }
    // 炎魔反击（火焰形态烈焰长鞭横扫：上限 +3）
    let (p0, p1) = if raged { (18, 29) } else { (18, 26) };
    let p_dmg = rnd_mo(p0, p1);
    st.hp = (st.hp - p_dmg).max(0);
    // 玩家倒下 → 战败死亡档案
    if st.hp <= 0 { return "mo_death_balrog".to_string(); }
    // 炎魔被击倒（HP≤0，硬杀兜底）→ 按已置结局归入，未置则视同断桥
    let boss_dead = st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false);
    if boss_dead {
        let _ = hp;
        return route_ragnarok(st);
    }
    // SAN 归零 → 黑暗中窒息
    if st.san <= 0 { return "mo_death_dark".to_string(); }
    "mo_boss_round".to_string()
}

/// 断桥坠渊（A 结局，环境判胜）：置结局旗标 + 主神目标 + 支线，+600 炎魔奖励（由结算 PointsIfFlag 再 +200）。
fn route_bridge_break(st: &mut GameState) -> String {
    st.points += 600;
    st.set_flag("mo_side_survive");
    st.set_flag("mo_cleared");
    st.set_flag("mo_balrog_down");
    crate::world::add_item(st, "elven_cloak"); // 传说掉落二选一（存活线 → 精灵斗篷）
    st.fight = None;
    "mo_ending_survive".to_string()
}

/// 甘道夫式牺牲（B 结局，环境判胜）：+600 炎魔奖励 + B 级支线（sp_grade B），+300 由结算 PointsIfFlag。
fn route_sacrifice(st: &mut GameState) -> String {
    st.points += 600;
    st.set_flag("mo_side_sacrifice");
    st.set_flag("mo_cleared");
    st.set_flag("mo_sacrifice_done");
    st.sp_grade = Some('B');
    st.set_flag("mo_gandalf_lost"); // 甘道夫"牺牲"（剧情失联，不计入死亡档案）
    crate::world::add_item(st, "mithril_schematic"); // 传说掉落（牺牲线 → 秘银护甲图纸）
    st.fight = None;
    "mo_ending_sacrifice".to_string()
}

/// 炎魔 HP≤0 硬杀兜底（防御性；正常流程由断桥/牺牲触发）
fn route_ragnarok(st: &mut GameState) -> String {
    if st.flag("mo_sacrifice_done") {
        st.points += 600;
        st.set_flag("mo_cleared");
        st.fight = None;
        "mo_ending_sacrifice".to_string()
    } else if st.flag("mo_balrog_down") {
        st.points += 600;
        st.set_flag("mo_cleared");
        st.fight = None;
        "mo_ending_survive".to_string()
    } else {
        st.points += 600;
        st.set_flag("mo_side_survive");
        st.set_flag("mo_cleared");
        st.set_flag("mo_balrog_down");
        crate::world::add_item(st, "elven_cloak");
        st.fight = None;
        "mo_ending_survive".to_string()
    }
}

/* ---- BOSS 战 文本（只读渲染） ---- */
fn txt_boss_round(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(270);
    let raged = balrog_raged(st);
    let head = if raged {
        "桥下黑暗炸裂成橙红的火焰——炎魔双角升腾，烈焰长鞭横扫全场！鼓声骤然停止（\"The drums… have stopped.\"），灼热的气浪裹挟着所有人的理智。\n\n<b style='color:#ff6a2a'>火焰形态：每回合理智蚕食（San -6）＋全场高温（San -3/回）。</b>".to_string()
    } else {
        "石桥横跨无底深渊，宽仅容两人并肩。两团火焰在桥下黑暗里亮起——炎魔自深渊升起，烈焰在它身上翻涌成影。\n\n（全场高温：每回合 San -3）".to_string()
    };
    format!("{head}\n\n<b>炎魔 · 都灵的克星</b>　HP {hp}/270\n\n甘道夫法杖抵地，踏前一步：「You shall not pass!」")
}

/* =====================================================================
   战斗配置表（MO 专属；导出供主线把 query 扩展进来）
   注：数值 §4 参照生化蜂巢基线 +60%（普通 34-60/奖 25-60；精英 100-145/奖 250-400；BOSS 270/600）。
   引擎自动把 cfg.reward 加进 points；watcher/balrog 另加 `b_` 别名供 ZoneDef 引擎直战复用。
   ===================================================================== */
fn mo_win_world_fallback(st: &GameState) -> String { let _ = st; "mo_world_back".to_string() }
fn mo_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}
fn mo_rage_common(_st: &mut GameState, _log: &mut Vec<String>) {}

fn watcher_rage(st: &mut GameState, log: &mut Vec<String>) {
    st.san = (st.san - 4).clamp(0, 100);
    log.push("<span class='crit'>触手缠上脚踝把你往水里拖——理智被盐水腌透（San -4）。</span>".into());
}
fn watcher_finisher_desc(_st: &GameState) -> String {
    "你趁触手收束的间隙，一剑（或一斧）捅进监视者那枚巨眼——粘液爆裂，它惨叫着沉回湖底，再无声息。".to_string()
}
fn orc_captain_rage(_st: &mut GameState, _log: &mut Vec<String>) {}
fn orc_guard_rage(_st: &mut GameState, _log: &mut Vec<String>) {}
fn balrog_rage(st: &mut GameState, log: &mut Vec<String>) {
    st.san = (st.san - 6).clamp(0, 100);
    log.push("<span class='crit'>火焰形态降临——深渊之火点燃桥下虚空，理智被高温蚕食（San -6）。</span>".into());
}
/// watcher 胜利后路由：先到 after 场景领支线旗标（engine win 不可写，故经场景 effects）
fn watcher_win_after(_st: &GameState) -> String { "mo_watcher_after".to_string() }
fn balrog_win_bridge(st: &GameState) -> String {
    if st.flag("mo_balrog_down") { "mo_ending_survive".into() } else { "mo_boss_round".into() }
}

/// 摩瑞亚战斗表（id 沿用设计 §4 原名；watcher/balrog 另加 `b_` 别名供 ZoneDef 直战复用）。
pub fn moruiya_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("goblin_scout", FightCfg {
            name: "哥布林斥候", hp: 34, dmg: (8, 12), reward: 25, reward_why: "斥候半兽人 · 击退",
            intro: "一道佝偻的绿皮身影从石柱后扑出，獠牙外露，弯刀泛起冷光——哥布林斥候发现了你们。",
            rage_at: None, rage_text: "", on_rage: mo_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_flame",
        }),
        ("goblin_pack", FightCfg {
            name: "半兽人巡逻队", hp: 48, dmg: (9, 13), reward: 40, reward_why: "半兽人巡逻队 · 清剿",
            intro: "反光从石廊尽头扫过——三名半兽人巡逻队拖着长矛逼近，鼓声在它们身后沉沉敲响。",
            rage_at: Some(25), rage_text: "巡逻队嘶吼着召唤增援——一名哥布林斥候从阴影里加入战团！", on_rage: mo_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_flame",
        }),
        ("watcher", FightCfg {
            name: "水中监视者", hp: 145, dmg: (16, 22), reward: 400, reward_why: "斩杀水中监视者",
            intro: "湖面炸裂——触手如鞭破水而出，湿滑粘液的吸盘卷向岸边。监视者自幽黑湖心扬起半截身躯，巨眼在中央睁开。",
            rage_at: Some(60), rage_text: "触手狂乱！每根触手都缠向一名同伴——每回合理智被盐水腌透（San -4）。",
            on_rage: watcher_rage,
            finisher_if: |_, ehp| ehp <= 20, finisher_name: |_| "斩断触手心核".to_string(),
            finisher_desc: watcher_finisher_desc,
            win: watcher_win_after, death: "mo_death_watcher",
        }),
        ("b_watcher", FightCfg {
            name: "水中监视者", hp: 145, dmg: (16, 22), reward: 400, reward_why: "斩杀水中监视者",
            intro: "湖面炸裂——触手如鞭破水而出，湿滑粘液的吸盘卷向岸边。监视者自幽黑湖心扬起半截身躯，巨眼在中央睁开。",
            rage_at: Some(60), rage_text: "触手狂乱！每根触手都缠向一名同伴——每回合理智被盐水腌透（San -4）。",
            on_rage: watcher_rage,
            finisher_if: |_, ehp| ehp <= 20, finisher_name: |_| "斩断触手心核".to_string(),
            finisher_desc: watcher_finisher_desc,
            win: watcher_win_after, death: "mo_death_watcher",
        }),
        ("goblin_raider", FightCfg {
            name: "半兽人掠夺者", hp: 42, dmg: (9, 14), reward: 30, reward_why: "掠夺者 · 击退",
            intro: "独眼的半兽人掠夺者拖着弯刀从书排间跃出，匕首在指间翻转，目光贪婪地扫过你的行囊。",
            rage_at: Some(20), rage_text: "掠夺者吹响骨笛——又一批半兽人循声围拢！", on_rage: mo_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_flame",
        }),
        ("drum_ambush", FightCfg {
            name: "鼓声伏击·半兽人群", hp: 60, dmg: (10, 16), reward: 60, reward_why: "击退鼓声伏击",
            intro: "鼓声骤然加速逼近——黑压压的半兽人从书架阴影里涌出，把你们围成铁桶：「咚…咚…咚…」",
            rage_at: Some(30), rage_text: "鼓声越敲越近越高——增援从两翼涌入，数量再翻！", on_rage: mo_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_flame",
        }),
        ("orc_captain", FightCfg {
            name: "半兽人队长", hp: 115, dmg: (15, 21), reward: 300, reward_why: "斩杀半兽人队长",
            intro: "铁甲走动的声音闷响——半兽人队长拖着锯齿大剑堵住去路，板甲上沾满旧血，獠牙里咬碎了半截断刃。",
            rage_at: Some(50), rage_text: "怒斩！他双目充血，一剑撕开你的架势——反击必中一击。", on_rage: orc_captain_rage,
            finisher_if: |_, ehp| ehp <= 30, finisher_name: |_| "缴械 · 断其刃".to_string(),
            finisher_desc: |_| "你趁他抬剑的瞬间格开锯齿大剑，一脚蹬在他持刃的手——队长虎口震裂，武器脱手飞出，惨嚎着败退。".to_string(),
            win: |_| "mo_06_vault".into(), death: "mo_death_flame",
        }),
        ("cave_troll", FightCfg {
            name: "洞穴巨魔", hp: 100, dmg: (14, 20), reward: 250, reward_why: "击倒洞穴巨魔",
            intro: "灰石色的庞大身影横在无底阶梯口，圆耳塌鼻，一脚踏得碎石飞溅——洞穴巨魔堵住了路。可绕行，也可硬闯。",
            rage_at: Some(40), rage_text: "践踏！巨魔抬脚猛砸地面，碎石如浪——你被震得眼前发黑，眩晕一回合（闪避下降）。", on_rage: mo_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_flame",
        }),
        ("orc_guard", FightCfg {
            name: "半兽人禁卫", hp: 50, dmg: (10, 15), reward: 40, reward_why: "击倒半兽人禁卫",
            intro: "王厅门槛前一杆长枪横出——半兽人禁卫低吼着挡住去路，铁盔下的眼睛死死盯着你们。",
            rage_at: Some(25), rage_text: "死战！禁卫狂暴后力道暴涨，攻势更沉（dmg +2）。", on_rage: orc_guard_rage,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_flame",
        }),
        ("balrog", FightCfg {
            name: "炎魔 · 都灵的克星", hp: 270, dmg: (18, 26), reward: 600, reward_why: "击败炎魔 · 都灵的克星",
            intro: "鼓声在这一刻骤然停止——然后，桥下的黑暗里燃起两团火焰。一股古老、灼热的恐怖自深渊升起。甘道夫踏前一步，法杖砸地：「You shall not pass!」",
            rage_at: Some(135), rage_text: "火焰厉啸——炎魔进入火焰形态！烈焰长鞭横扫全场，深渊之火点燃桥下虚空。", on_rage: balrog_rage,
            finisher_if: |st, _ehp| cond_breachable(st), finisher_name: |_| "断桥坠渊".to_string(),
            finisher_desc: |_| "甘道夫驱动全部法力，石桥自中段轰然断裂——炎魔嚎叫着坠向无底深渊。（断桥坠渊）".to_string(),
            win: balrog_win_bridge, death: "mo_death_balrog",
        }),
        ("b_balrog", FightCfg {
            name: "炎魔 · 都灵的克星", hp: 270, dmg: (18, 26), reward: 600, reward_why: "击败炎魔 · 都灵的克星",
            intro: "鼓声在这一刻骤然停止——然后，桥下的黑暗里燃起两团火焰。一股古老、灼热的恐怖自深渊升起。甘道夫踏前一步，法杖砸地：「You shall not pass!」",
            rage_at: Some(135), rage_text: "火焰厉啸——炎魔进入火焰形态！烈焰长鞭横扫全场，深渊之火点燃桥下虚空。", on_rage: balrog_rage,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: mo_win_world_fallback, death: "mo_death_balrog",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn mo_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    moruiya_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   剧情场景（id 全部 mo_ 前缀）
   ===================================================================== */
pub static MORUIYA_SCENES: &[SceneDef] = &[

/* ---- 幕 1 · 西闸门·湖中监视者（开场） ---- */
SceneDef {
    id: "mo_01_gate", bg: Some("moruiya_bg.png"), loc: Some("摩瑞亚 · 西闸门湖景"),
    mood: "danger", speaker: Some("甘道夫"), voice: None,
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>穿过摩瑞亚——从西闸门抵达东门，击败深渊之下苏醒的炎魔·都灵的克星。任务完成前无法离开矿坑。",
        "从林谷出发的第七日，你们站在卡扎督姆的西门前。湖水幽黑如镜，倒映着黑曜石门柱。甘道夫低语：「摩瑞亚……比传说中更古老，也更黑暗。」",
        "话音未落，湖面泛起一圈涟漪——涟漪中心，有什么东西在向你们靠近。远处，水底传来一种黏腻的、多足刮擦的声响。甘道夫补上一句：「矮人们挖掘得太深、太贪婪——都灵的克星，就在底下。」",
    ]),
    choices: &[
        ChoiceDef { label: "【检查湖面】", sub: "调查 · San-4 · 唤起水中监视者", cond: None,
            effects: &[Eff::San(-4), Eff::MarkPoint("mo_p_lake")], route: Route::To("mo_lake") },
        ChoiceDef { label: "【立刻冲入闸门】", sub: "后路断绝 · San-2 · 无奖励", cond: None,
            effects: &[Eff::San(-2), Eff::SetFlag("mo_gate_sealed"), Eff::MarkPoint("mo_p_lake")],
            route: Route::To("mo_02_hall") },
        ChoiceDef { label: "【与甘道夫交谈再入】", sub: "情报 · 台词", cond: None,
            effects: &NO_EFF, route: Route::To("mo_npc_gandalf") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 湖岸（可选监视者遭遇） ---- */
SceneDef {
    id: "mo_lake", bg: Some("moruiya_bg_open.png"), loc: Some("西闸门 · 幽黑湖岸"),
    mood: "danger", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&[
        "幽黑湖水倒映着石柱。吉姆利伏低身子，盯着水中那道黑影：「湖水太死了——连一条游鱼都没有。太安静了。」",
        "就在你靠近水岸线的刹那，湖面「哗」地炸开——水之监视者的触手破水而出！弗罗多被一根触手卷向水面的嘶喊刺穿空气。",
    ]),
    choices: &[
        ChoiceDef { label: "【迎战水中监视者】", sub: "强战 · 胜利 +400 与支线 A", cond: None,
            effects: &[Eff::SetFlag("mo_watcher_engaged"), Eff::San(-4)], route: Route::To("mo_watcher_fight") },
        ChoiceDef { label: "【夺门冲入闸门】", sub: "放弃战利品 · G1 封死后路", cond: None,
            effects: &[Eff::SetFlag("mo_gate_sealed"), Eff::San(-2)], route: Route::To("mo_02_hall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_watcher_fight", bg: Some("moruiya_bg_open.png"), loc: Some("西闸门 · 湖岸"),
    mood: "danger", speaker: Some("⚔ 决战"), voice: None,
    text: TextSpec::Static(&["触手如林挥舞，监视者半截身躯拔出湖面。甘道夫挥杖断喝：「看好你们的缺口——就是现在！」"]),
    choices: &NO_CH, fight_id: Some("watcher"), video: None, cine_label: None, overlay: None,
},
/* watcher 胜利 → 领支线 A（engine reward 已 +400，此处只置旗标） */
SceneDef {
    id: "mo_watcher_after", bg: Some("moruiya_bg_open.png"), loc: Some("西闸门 · 湖岸"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["监视者的触手俱断，沉入湖底再无声息。湖水与触手把西闸门封得严严实实——后路断绝，只能向前。门内，拱柱撑起百尺高穹顶，烛火在尘埃里摇曳。"]),
    choices: &[ChoiceDef { label: "（踏进门内 · 支线 A 结算）", sub: "支线 A · 水中监视者已斩", cond: None,
        effects: &[Eff::SetFlag("mo_side_watcher"), Eff::SetFlag("mo_watcher_slain")], route: Route::To("mo_02_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 2 柱厅·刻字石板（教学/解密） ---- */
SceneDef {
    id: "mo_02_hall", bg: Some("moruiya_bg_open.png"), loc: Some("卡扎督姆 · 柱厅"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&[
        "拱柱撑起百尺高穹顶，烛火在尘埃里摇曳。吉姆利抚过石柱上的矮人文字：「这是卡扎督姆的全盛时代……你们看，这块石板刻着开门咒文。」身侧，一根火把在墙槽里微微晃动——你举起来，黑暗退开两步。",
    ]),
    choices: &[
        ChoiceDef { label: "【研读石板】", sub: "解密 C · 成功 +100 / 失败 San-5+斥候战", cond: Some(cond_rune_undone),
            effects: &[Eff::MarkPoint("mo_p_slab"), Eff::San(-5)], route: Route::Dyn(route_rune_resolve) },
        ChoiceDef { label: "【推开柱厅大门（已解密）】", sub: "需解密", cond: Some(cond_has_rune), effects: &NO_EFF, route: Route::To("mo_collapse") },
        ChoiceDef { label: "【绕行南廊】", sub: "遭遇 goblin_pack", cond: None, effects: &NO_EFF, route: Route::Dyn(route_south_goblin) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_rune", bg: Some("moruiya_bg_open.png"), loc: Some("柱厅 · 刻字石板"),
    mood: "danger", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["你凑近石板。矮人咒文在烛火下泛着微光——吉姆利念出几个音节，石门后传来沉闷的机簧转动声。但要完全解开，得按正确顺序拨动石钮。"]),
    choices: &[
        ChoiceDef { label: "【按咒文顺序拨动石钮】", sub: "解密成功 · 支线 C +100", cond: None,
            effects: &[Eff::SetFlag("mo_rune_decoded"), Eff::SetFlag("mo_side_rune"), Eff::Points(100)], route: Route::To("mo_02_hall") },
        ChoiceDef { label: "【强行撬动石钮】", sub: "失败 San-5 · 触发斥候战", cond: None,
            effects: &[Eff::MarkPoint("mo_p_slab"), Eff::San(-5), Eff::SetFlag("mo_rune_failed")], route: Route::To("mo_rune_scout") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_rune_scout", bg: Some("moruiya_bg_open.png"), loc: Some("柱厅 · 石门"),
    mood: "danger", speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["石钮被你撬得咔咔作响，机簧走火——一道石门缝炸开，一名哥布林斥候从中扑出！"]),
    choices: &NO_CH, fight_id: Some("goblin_scout"), video: None, cine_label: None, overlay: None,
},

/* ---- 幕 3 塌方与绕行 ---- */
SceneDef {
    id: "mo_collapse", bg: Some("moruiya_bg_invest.png"), loc: Some("北廊 · 塌方"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["前方轰然塌方，碎石封死了去路。北廊尽头传来低沉的石块摩擦声——这条路恐怕已经断了几百年。绕行南廊要多打一场遭遇战。"]),
    choices: &[
        ChoiceDef { label: "【清理碎石】", sub: "耗时 · Hurt(-10) · 打开捷径", cond: None,
            effects: &[Eff::SetFlag("mo_collapse_cleared"), Eff::MarkPoint("mo_p_collapse"), Eff::Hurt(10, "mo_death_crush")], route: Route::To("mo_02_hall") },
        ChoiceDef { label: "【绕行南廊】", sub: "遭遇半兽人巡逻队", cond: None, effects: &NO_EFF, route: Route::To("mo_south_ambush") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_south_ambush", bg: Some("moruiya_bg_invest.png"), loc: Some("南廊"),
    mood: "danger", speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["南廊的黑暗里，几名半兽人巡逻队拖矛逼来——没有捷径，只能杀过去。"]),
    choices: &NO_CH, fight_id: Some("goblin_pack"), video: None, cine_label: None, overlay: None,
},

/* ---- 幕 4 书库·巴林之墓（关键转折·鼓声） ---- */
SceneDef {
    id: "mo_book", bg: Some("moruiya_bg_invest.png"), loc: Some("马扎布尔书库 · 石棺"),
    mood: "danger", speaker: Some("甘道夫"), voice: None,
    text: TextSpec::Static(&[
        "书库尽头立着一具石棺，墓碑上刻着：『巴林，卡扎督姆之王。』残破的书页记着远征队的覆灭：『……他们来了。鼓声，在深井里。我们出不去……』",
        "就在你读到这里时，深井之下传来第一声——<b>咚。</b>鼓声渐密，从书架深处涌出半兽人的脚步。",
    ]),
    choices: &[
        ChoiceDef { label: "【读完《马扎布尔之书》】", sub: "+150 · San-8 · 解锁书库大门", cond: Some(cond_book_undone),
            effects: &[Eff::SetFlag("mo_book_read"), Eff::SetFlag("mo_side_book"), Eff::Points(150), Eff::MarkPoint("mo_p_book")], route: Route::To("mo_drum_ambush_scene") },
        ChoiceDef { label: "【合上书，立刻警戒】", sub: "San-4 · 同样触发伏击战", cond: None,
            effects: &[Eff::San(-4), Eff::MarkPoint("mo_p_book")], route: Route::To("mo_drum_ambush_scene") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_drum_ambush_scene", bg: Some("moruiya_bg_invest.png"), loc: Some("书库深厅"),
    mood: "danger", speaker: Some("⚔ 鼓声伏击"), voice: None,
    text: TextSpec::Static(&["咚…咚…咚…——鼓声近在咫尺。黑压压的半兽人自书排之间涌出，把去路围成铁桶。"]),
    choices: &NO_CH, fight_id: Some("drum_ambush"), video: None, cine_label: None, overlay: None,
},

/* ---- F2 密室 · 秘银钥匙石宝箱（Give mithril_key → 支线 E 前置） ---- */
SceneDef {
    id: "mo_chest", bg: Some("moruiya_bg_invest.png"), loc: Some("书库底层 · 密室"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.inventory.iter().any(|i| i == "mithril_key") {
            "你已经取走了那枚秘银钥匙石。石首宝箱已经空了——只有底座上一道浅浅的矮人刻痕：『钥匙开的是……宝库的大门。』".to_string()
        } else {
            "密室角落的石首宝箱覆着尘土。你撬开箱盖，一枚被月光照得泛银白的<b>秘银钥匙石</b>静静躺着。（mithril_key——可开 F3 王厅宝库 G5）".to_string()
        }
    }),
    choices: &[ChoiceDef {
        label: "【收下秘银钥匙石】", sub: "任务道具 mithril_key", cond: Some(cond_no_mithril_key),
        effects: &[Eff::AddItem("mithril_key"), Eff::MarkPoint("mo_p_chest")], route: Route::To("mo_world_back"),
    }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 5 无底阶梯（三处坠落陷阱） ---- */
SceneDef {
    id: "mo_stair", bg: Some("moruiya_bg_invest.png"), loc: Some("无底阶梯 · 一步踏空"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(txt_stair_round),
    choices: &[
        ChoiceDef { label: "【贴墙缓行】", sub: "耗时省心 · Hurt(-15)", cond: None,
            effects: &[Eff::Hurt(15, "mo_death_stair")], route: Route::Dyn(route_stair_pass) },
        ChoiceDef { label: "【快步通过】", sub: "冒险 · Hurt(30) · 一步踏空即坠渊", cond: None,
            effects: &[Eff::Hurt(30, "mo_death_stair")], route: Route::Dyn(route_stair_pass) },
    ],
    fight_id: None, video: Some("vid_mo_stair.mp4"), cine_label: Some("过场 · 无底阶梯（H3 本地生成）"), overlay: None,
},

/* ---- 幕 6 王厅宝库（支线·可选） ---- */
SceneDef {
    id: "mo_vault", bg: Some("moruiya_bg_invest.png"), loc: Some("王厅宝库"),
    mood: "danger", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["王厅深处，一扇秘银包边的石门静静立着。吉姆利咽了口唾沫：「矮人的宝库……钥匙在书库底层的密室里。」"]),
    choices: &[
        ChoiceDef { label: "【开启宝库】", sub: "需 mithril_key · 支线 E +300", cond: Some(cond_has_mithril_key),
            effects: &[Eff::SetFlag("mo_vault_open"), Eff::SetFlag("mo_side_vault"), Eff::Points(300), Eff::MarkPoint("mo_p_vault_chest")],
            route: Route::To("mo_vault_take") },
        ChoiceDef { label: "【检查机关再开】", sub: "跳过则 Hurt-20 · 可能塌方", cond: None,
            effects: &[Eff::San(-5)], route: Route::To("mo_vault_check") },
        ChoiceDef { label: "【离开宝库】", sub: "放弃支线", cond: None, effects: &NO_EFF, route: Route::To("mo_bridge_desc") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_vault_take", bg: Some("moruiya_bg_invest.png"), loc: Some("王厅宝库"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["你拧开秘银宝箱——堆叠的沉甸甸银块与一件轻盈的精灵斗篷静静躺在灰尘里。吉姆利抱着秘银块，眼眶发红：『矮人的荣光，还没全沉进黑暗。』"]),
    choices: &[ChoiceDef { label: "（收下秘银块与精灵斗篷）", sub: "+300 · AddItem mithril_block / elven_cloak", cond: None,
        effects: &[Eff::SetFlag("mo_vault_open"), Eff::SetFlag("mo_side_vault"), Eff::Points(300), Eff::AddItem("mithril_block"), Eff::AddItem("elven_cloak")], route: Route::To("mo_bridge_desc") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_vault_check", bg: Some("moruiya_bg_invest.png"), loc: Some("王厅宝库 · 机关"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你凑近锁孔旁的机关，看见藏于石缝里的绊索——抬手拨开，机关无声解除。你拧开宝库，取走秘银块与精灵斗篷（支线 E +300）。"]),
    choices: &[ChoiceDef { label: "（取走宝物）", sub: "检查成功 · 无伤", cond: None,
        effects: &[Eff::SetFlag("mo_vault_open"), Eff::SetFlag("mo_side_vault"), Eff::Points(300), Eff::AddItem("mithril_block"), Eff::AddItem("elven_cloak")], route: Route::To("mo_bridge_desc") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 7 卡扎督姆桥·炎魔（终局） ---- */
SceneDef {
    id: "mo_bridge_desc", bg: Some("moruiya_bg_battle.png"), loc: Some("卡扎督姆桥"),
    mood: "epic", speaker: Some("甘道夫"), voice: None,
    text: TextSpec::Static(&["石桥横跨无底深渊，宽仅容两人并肩。鼓声在这一刻骤然停止——然后，桥下的黑暗里燃起两团火焰。甘道夫踏前一步，法杖砸地：「You shall not pass!」"]),
    choices: &[ChoiceDef { label: "（迎向炎魔）", sub: "开战", cond: None, effects: &NO_EFF, route: Route::Dyn(start_balrog) }],
    fight_id: None, video: Some("vid_mo_bridge.mp4"), cine_label: Some("过场 · 卡扎督姆桥决战（H3 本地生成）"), overlay: None,
},

/* ---- BOSS 战 · 选择驱动回合（每回 SAN 光环） ---- */
SceneDef {
    id: "mo_boss_round", bg: Some("moruiya_bg_battle.png"), loc: Some("卡扎督姆桥 · 中段"),
    mood: "danger", speaker: Some("⚔ 决战 · 炎魔"), voice: None,
    text: TextSpec::Dyn(txt_boss_round),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 28-40 · 易露破绽", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| route_boss_attack(st, rnd_mo(28, 40))) },
        ChoiceDef { label: "连击（迅疾）", sub: "伤害 18-26 · 稳", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| route_boss_attack(st, rnd_mo(18, 26))) },
        ChoiceDef { label: "【引它冲锋，斩断桥索】", sub: "断桥坠渊 · 需炎魔 HP<30% 且甘道夫存活", cond: Some(cond_breachable),
            effects: &NO_EFF, route: Route::Dyn(route_bridge_break) },
        ChoiceDef { label: "【让甘道夫断桥】", sub: "甘道夫式牺牲 · +300 + B 级支线", cond: Some(cond_sacrifice),
            effects: &NO_EFF, route: Route::Dyn(route_sacrifice) },
        ChoiceDef { label: "【后撤重整】", sub: "恢复架势 · 仅走 SAN 光环", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st| route_boss_attack(st, 0)) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 双结局终结 ---- */
SceneDef {
    id: "mo_ending_survive", bg: Some("moruiya_bg_battle.png"), loc: Some("卡扎督姆桥 · 断桥"),
    mood: "epic", speaker: Some("甘道夫"), voice: None,
    text: TextSpec::Static(&[
        "甘道夫驱动全部法力，石桥自中段轰然断裂。炎魔嘶吼着坠向无底深渊，烈焰在降落的瞬间被黑暗吞没——深渊之火熄灭了。",
        "断桥上，甘道夫还站着，法杖还点着火星。他望着桥下浓黑：「深渊之火……记住了，你们对抗过它。」",
    ]),
    choices: &[ChoiceDef { label: "（向东门而行 · 支线 F 结算）", sub: "F_gandalf_survive +200", cond: None,
        effects: &[Eff::PointsIfFlag("mo_side_survive", 200)], route: Route::To("mo_exit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_ending_sacrifice", bg: Some("moruiya_bg_battle.png"), loc: Some("卡扎督姆桥 · 断桥"),
    mood: "epic", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "甘道夫回头最后看了你们一眼，转身立在断桥边缘。他将法杖砸向桥面——",
        "「Fly, you fools!」石桥轰塌，炎魔与甘道夫一同坠入深渊。火焰在黑暗里烧了很长、很长的一瞬，然后熄灭。",
        "<b>甘道夫牺牲了</b>——他拉着都灵的克星一同坠入无底深渊。这是《指环王》里最疼的那一页。",
    ]),
    choices: &[ChoiceDef { label: "（哀悼后向东门而行 · 支线 G 结算）", sub: "G_gandalf_sacrifice +300 · B 级支线（sp_grade B）", cond: None,
        effects: &[Eff::PointsIfFlag("mo_side_sacrifice", 300)], route: Route::To("mo_exit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 8 东门·黎明 + 完成结算 ---- */
SceneDef {
    id: "mo_exit", bg: Some("moruiya_bg_battle.png"), loc: Some("东门 · 迪姆瑞尔山谷"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["晨光从东门缝隙漏进来——你们做到了。身后是沉寂的摩瑞亚，前方是山风与林海。『我们穿越了卡扎督姆，』队伍里有人说，『深渊之火……记住了，你们对抗过它。』"]),
    choices: &[ChoiceDef { label: "（推开东门）", sub: "G6 已开", cond: Some(cond_cleared), effects: &NO_EFF, route: Route::To("mo_done") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_done", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "任 务 完 成 · 穿 越 摩 瑞 亚".into(), good: true,
            body_html: format!(
                "<p>晨光铺满迪姆瑞尔山谷。你们从黑暗矿坑里活着走了出来——深渊之火的名字：<b>都灵的克星</b>，被留在了身后。</p>\
                 <p style='color:#9a958a'>《魔戒·摩瑞亚矿坑》副本 · 已完成（出口/复活/结算由主神空间系统接线）</p>\
                 <table class='statTable'>\
                 <tr><td>当前奖励点数（击败炎魔 +600）</td><td>{}</td></tr>\
                 <tr><td>支线剧情评级</td><td style='color:#ffd76a'>{}</td></tr>\
                 <tr><td>传说掉落</td><td>{}</td></tr>\
                 </table>\
                 <p style='color:#8fd0a8'>剩余点数：{}　（支线 A/B/C/D/E + 结局 F 或 G + H 的 ±200 已在结算中计）</p>\
                 <p style='color:#ffd76a'>【主神】：你们对抗过的深渊之火，留下了回响。下一场大幕……是 <b>《异形4》</b>。</p>",
                st.points,
                match st.sp_grade { Some(g) => format!("{} 级", g), None => "暂无".to_string() },
                if st.inventory.iter().any(|i| i == "elven_cloak") {
                    "精灵斗篷 elven_cloak".to_string()
                } else if st.inventory.iter().any(|i| i == "mithril_schematic") {
                    "秘银护甲图纸 mithril_schematic".to_string()
                } else { "—".to_string() },
                st.points
            ),
            buttons: vec![("进 入 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- 矿山趣味小节：矿车（H_cart_ride） ---- */
SceneDef {
    id: "mo_cart", bg: Some("moruiya_bg_invest.png"), loc: Some("矿车月台"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["一辆报废但还能滑行的矮人矿车斜停在月台上。吉姆利努努嘴：『卡扎督姆的老规矩——上得去，不一定回得来。要不要试试快感？』"]),
    choices: &[
        ChoiceDef { label: "【跳上矿车直滑一层】", sub: "趣味 · 支线 H +50 · 单向坠回 F1 月台", cond: None,
            effects: &[Eff::SetFlag("mo_side_cart"), Eff::Points(50), Eff::MarkPoint("mo_p_cart")], route: Route::To("mo_cart_ride") },
        ChoiceDef { label: "（不下车，继续行军）", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(goto_continue_f2) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_cart_ride", bg: Some("moruiya_bg_invest.png"), loc: Some("矿车 · 下坠"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["矿车哐当滑下,越过翻滚的石轨,直坠 F1 月台。你们抱头滚下矿车,人仰马翻却都活着——卡扎督姆的老规矩,还真不是说说而已。"]),
    choices: &[ChoiceDef { label: "（继续探索）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("mo_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 通用 · 地图返回 ---- */
SceneDef {
    id: "mo_world_back", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["队伍调整好姿态，继续在摩瑞亚的黑暗里行军。"]),
    choices: &[ChoiceDef { label: "（继续探索）", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(continue_explore) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- NPC 对话 ---- */
SceneDef {
    id: "mo_npc_gandalf", bg: Some("moruiya_bg_invest.png"), loc: Some("西闸门 · 甘道夫"),
    mood: "calm", speaker: Some("甘道夫"), voice: None,
    text: TextSpec::Static(&["甘道夫望着幽黑的湖水，声音压得很低：『矮人们挖掘得太深、太贪婪——They delved too greedily and too deep……唤醒了\"都灵的克星\"。摩瑞亚，你们愿意的话可以叫它卡扎督姆。』"]),
    choices: &[ChoiceDef { label: "（点头，走向闸门）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("mo_01_gate") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_npc_boromir", bg: Some("moruiya_bg_invest.png"), loc: Some("柱厅 · 波罗莫"),
    mood: "calm", speaker: Some("波罗莫"), voice: None,
    text: TextSpec::Static(&["波罗莫挎着盾，警惕地环顾石柱：『一座空的矿坑，不等于一座安静的矿坑。——黑暗里有东西在听我们讲话。』他顿了顿，『它不喜欢火把。』"]),
    choices: &[ChoiceDef { label: "（举起火把，继续向前）", sub: "torch_lit 点亮提示", cond: None,
        effects: &[Eff::SetFlag("mo_torch_lit")], route: Route::To("mo_02_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_npc_gimli", bg: Some("moruiya_bg_invest.png"), loc: Some("书库 · 吉姆利"),
    mood: "calm", speaker: Some("吉姆利"), voice: None,
    text: TextSpec::Static(&["吉姆利在石棺前蹲了很久，声音有些哑：『这是巴林……我的亲戚，卡扎督姆之王。他们想夺回摩瑞亚，没能活着出去。』他抬头：『但我们会。』"]),
    choices: &[ChoiceDef { label: "（安慰他，继续向前）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("mo_book") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_npc_troll", bg: Some("moruiya_bg_invest.png"), loc: Some("无底阶梯口 · 巨魔"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["巨大的灰石头颅挡住去路，巨魔低头嗅了嗅你们——它没有主动攻击，只是占着这条必经之阶。可绕行，要多走一段黑廊（San -4）。"]),
    choices: &[
        ChoiceDef { label: "【硬闯巨魔】", sub: "越级精英战 · 胜利 +250", cond: None, effects: &NO_EFF, route: Route::To("mo_troll_fight") },
        ChoiceDef { label: "【绕行黑廊】", sub: "San-4 · 避战", cond: None, effects: &[Eff::San(-4)], route: Route::To("mo_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "mo_troll_fight", bg: Some("moruiya_bg_invest.png"), loc: Some("无底阶梯口 · 巨魔"),
    mood: "danger", speaker: Some("⚔ 巨魔"), voice: None,
    text: TextSpec::Static(&["你拔武器迎上,巨魔抡起石锤怒吼着砸下来——硬碰硬。"]),
    choices: &NO_CH, fight_id: Some("cave_troll"), video: None, cine_label: None, overlay: None,
},

/* ---- 死亡档案（overlay · 6 种：设计 §10.2） ---- */
SceneDef {
    id: "mo_death_watcher", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("坠入湖中 · 湖妖拖行", "湖中监视者把你拖入水中，盐水和着触手的摆动灌进肺里")),
        card: |_| crate::state::Card {
            title: "湖 妖 拖 行".into(), good: false,
            body_html: r#"<p>触手缠上你的脚踝，力道大得挣不脱——你被拖进幽黑的湖水。水底有个巨大的透明瞳仁，缓缓转向你。</p>
<p style='color:#ff8a8a'>【死亡档案 · 湖妖拖行】</p>
<p style='color:#666'>（复活：回主神空间扣 400 点，由主线复活系统接线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())], voice: None,
        },
    }),
},
SceneDef {
    id: "mo_death_stair", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("坠入无底阶梯", "一步踏空，坠向看不见底的深渊，失重声在耳膜里拉成尖啸")),
        card: |_| crate::state::Card {
            title: "坠 入 无 底 阶 梯".into(), good: false,
            body_html: r#"<p>石阶在你脚下化成齑粉——你一脚踏空，身体坠向那片连声音都被吞掉的黑暗。低沉的鼓声仿佛来自井底，又仿佛来自你正在坠落的尽头。</p>
<p style='color:#ff8a8a'>【死亡档案 · 坠入无底阶梯】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())], voice: None,
        },
    }),
},
SceneDef {
    id: "mo_death_balrog", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("坠入卡扎督姆深渊", "被炎魔拉下断桥，坠向无底的深渊之火")),
        card: |_| crate::state::Card {
            title: "坠 入 卡 扎 督 姆 深 渊".into(), good: false,
            body_html: r#"<p>烈焰长鞭卷住你的腰际，你被拖下断桥。深渊里，都灵的克星在翻涌的火光中与你一同下坠。</p>
<p style='color:#ff8a8a'>【死亡档案 · 坠入卡扎督姆深渊】</p>
<p style='color:#666'>（复活：回主神空间扣 600 点，按难度递增；生化 300 基线。）</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())], voice: None,
        },
    }),
},
SceneDef {
    id: "mo_death_flame", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("烈焰焚身", "被深渊之火点燃，火焰嘶鸣吞没全身")),
        card: |_| crate::state::Card {
            title: "烈 焰 焚 身".into(), good: false,
            body_html: r#"<p>半兽人的长矛、巨魔的石锤、炎魔的火鞭——摩瑞亚的黑暗把所有伤口都点成了同样的火。</p>
<p style='color:#ff8a8a'>【死亡档案 · 烈焰焚身】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())], voice: None,
        },
    }),
},
SceneDef {
    id: "mo_death_dark", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("黑暗中窒息", "理智崩断，在不见五指的黑暗中停止呼吸")),
        card: |_| crate::state::Card {
            title: "黑 暗 中 窒 息".into(), good: false,
            body_html: r#"<p>灯灭了。没有火把的黑暗里，鼓声越来越近。你分不清那是脚步声，还是自己心跳的回响——然后一切归于安静。</p>
<p style='color:#ff8a8a'>【死亡档案 · 黑暗中窒息】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())], voice: None,
        },
    }),
},
SceneDef {
    id: "mo_death_crush", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被塌方掩埋", "清理碎石时石梁砸落，把你埋进矿坑的尘埃里")),
        card: |_| crate::state::Card {
            title: "被 塌 方 掩 埋".into(), good: false,
            body_html: r#"<p>你撬到石梁的瞬间，整个塌方轰然压下来。黑暗里只听得见自己咽下的那口土腥气。</p>
<p style='color:#ff8a8a'>【死亡档案 · 被塌方掩埋】</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())], voice: None,
        },
    }),
},
];

/* =====================================================================
   路由辅助（Route::Dyn 专用；均为具名 fn）
   ===================================================================== */
/// 石板研读分支：final 走向
fn route_rune_resolve(st: &mut GameState) -> String {
    if st.flag("mo_rune_decoded") {
        "mo_02_hall".to_string()
    } else {
        "mo_rune".to_string()
    }
}
/// 绕行南廊 → 若已清塌方则回柱厅，否则进南廊遭遇
fn route_south_goblin(st: &mut GameState) -> String {
    if st.flag("mo_collapse_cleared") {
        "mo_02_hall".to_string()
    } else {
        "mo_south_ambush".to_string()
    }
}
/// 无底阶梯三处已安全通过数（连号 flag mo_stair_1/2/3）
fn stair_count(st: &GameState) -> usize {
    ["mo_stair_1", "mo_stair_2", "mo_stair_3"].iter().filter(|k| st.flag(k)).count()
}
/// 无底阶梯文本（只读渲染，显示进度）
fn txt_stair_round(st: &GameState) -> String {
    let done = stair_count(st);
    format!(
        "阶梯在黑暗里无休止地螺旋下坠，扶手早已风化。每一级都可能是最后一级——石阶边缘的碎屑正簌簌落入看不见的深渊。\n\n（无底阶梯陷阱：{}/3 处已安全通过；贴墙缓行 Hurt(-15)，快步 Hurt(30)、一步踏空即坠渊。三处全部安全通过 → 支线 D +150。）",
        done.min(3)
    )
}
/// 无底阶梯三处（route 引用同一场景，用连号 flag 累计安全通过数）：累计满 3 → 支线 D +150。
fn route_stair_pass(st: &mut GameState) -> String {
    let c = stair_count(st);
    if c >= 3 {
        // 防御：已满仍被进入（理论上不会）
        "mo_world_back".to_string()
    } else {
        // 累加下一道旗标
        match c {
            0 => st.set_flag("mo_stair_1"),
            1 => st.set_flag("mo_stair_2"),
            _ => st.set_flag("mo_stair_3"),
        }
        if c + 1 >= 3 {
            st.set_flag("mo_side_stair");
            st.points += 150;
            st.set_flag("mo_stair_finished");
        }
        "mo_world_back".to_string()
    }
}
/// 矿车不下车：继续 F2（无实际楼层切换，服务为外沿场景占位）
fn goto_continue_f2(st: &mut GameState) -> String { let _ = st; "mo_world_back".to_string() }
/// 通用地图返回（实际楼层/坐标切换由开放世界层处理；此处仅提供可交互出口）
fn continue_explore(st: &mut GameState) -> String { let _ = st; "mo_world_back".to_string() }
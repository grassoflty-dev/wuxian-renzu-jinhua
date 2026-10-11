//! 《大宇宙时代 · 沙丘魔海 · 坠毁之星》——全部剧情场景与战斗配置。
//! 设计取向：「世界展示向」。以展示沙丘魔海／绿潮异星世界为主，剧情开放、无真相线阴谋指向。
//! 钩子：「绿潮吞没显示器之前，你先看清了它有多美。」
//! 玩家作为探索者穿越 4 层（坠毁穿梭机残骸→沙漠地表·绿潮战场→共生体母巢→沙丘洞穴·深渊回响），
//! 用 flag 链管理「氧气倒计时」（零新引擎字段），在绿潮吞没前看清这个世界的究竟。
//!
//! 本文件是全新新增文件，只导出静态数据（SHAQIU_SCENES / shaqiu_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/shaqiu_impl_log.md ★外部依赖）。
//! 场景 id 全 `sq_` 前缀；fight id 全 `sq_` 前缀。
//!
//! BOSS 渴水兽王走「选择驱动遭遇链」（Route::Dyn，非引擎 FIGHTS 表）：
//!   HP240 / dmg(18,28) / rage_at Some(96)（狂暴再生 +15，最多 3 回合，除非诱水剂脱水终止）。
//!   弱火 ×1.3 / 弱电 ×1.3 在战斗回合选择内手动乘算；「诱水剂」在 HP<50% 解锁终结技「脱水重创」。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 字段换成新图；当前用现有图占位）：
//!   F1 沙海残骸 bg sq_bg_f1_wreck  （现用 img_laser.png 占位）
//!   F1 驾驶舱 bg sq_bg_cockpit    （现用 img_zhuyuan_book.png 占位）
//!   F2 绿潮战场 bg sq_bg_f2_green （现用 img_redqueen.png 占位）
//!   F3 共生体母巢 bg sq_bg_f3_nest（现用 img_zhuyuan_book.png 占位）
//!   F4 沙丘洞穴 bg sq_bg_f4_cave  （现用 img_laser.png 占位）
//!   BOSS bg sq_bg_boss            （现用 img_laser.png 占位）
//!   结局升空 bg sq_bg_rise        （现用 img_zhuyuan_book.png 占位）
//! 敌人立绘复用：zombie→绿潮共生体、hunter→渴水兽王（占位）、horde→虫群；新美术由主 agent 统一生图替换。

use crate::defs::*;
use crate::state::GameState;
use rand::Rng;

/// 空 effect / choice 惯用静态（同 scenes.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   通用小工具
   ===================================================================== */
fn rng(a: i32, b: i32) -> i32 { rand::thread_rng().gen_range(a..=b) }

/// 物品栏是否持有
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

/// 触发感应的 flag（幕6 前置）
fn has_decrypt(st: &GameState) -> bool { st.flag("sq_decrypt_ok") }

/* =====================================================================
   氧气倒计时（零新引擎字段，剧情 flag 链降级）
   sq_oxy_1 → sq_oxy_2 → sq_oxy_3（低氧警戒）→ 严重缺氧。
   低氧阶段按探索步数扣 HP，营造"压缩氧气量"的生存压力。
   ===================================================================== */
fn oxy_tick(st: &mut GameState) {
    if !st.flag("sq_oxy_1") {
        st.set_flag("sq_oxy_1");
    } else if !st.flag("sq_oxy_2") {
        st.set_flag("sq_oxy_2");
    } else if !st.flag("sq_oxy_3") {
        st.set_flag("sq_oxy_3");
        st.hp = (st.hp - 2).max(0); // 低氧警戒：每步 2 伤害
    } else {
        st.hp = (st.hp - 4).max(0); // 严重缺氧：每步 4 伤害
    }
}

// 注：在休整点（通风台/鹰/医疗箱）不推进氧耗，作为生存"喘口气"节点——
// 这些场景的返回统一走 sq_route_stay（不调 oxy_tick）。

/* =====================================================================
   BOSS 渴水兽王 · 选择驱动遭遇链
   HP 存 st.fight（sq_40_boss_intro 的 Route::Dyn 初始化，引用 sq_boss_king 的 FightCfg）。
   狂暴再生：flag 链计数（rgc1/rgc2/rgc3 表示已消耗的再生回合，最多 3）。
   ===================================================================== */
fn regen_consumed(st: &GameState) -> i32 {
    [st.flag("sq_rgc1"), st.flag("sq_rgc2"), st.flag("sq_rgc3")]
        .iter().filter(|b| **b).count() as i32
}

fn regen_active(st: &GameState) -> bool {
    st.flag("sq_raged") && !st.flag("sq_dehydrated") && regen_consumed(st) < 3
}

fn mark_regen(st: &mut GameState) {
    if st.flag("sq_rgc1") { st.set_flag("sq_rgc2"); }
    else if st.flag("sq_rgc2") { st.set_flag("sq_rgc3"); }
    else { st.set_flag("sq_rgc1"); }
}

fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("sq_boss_king") {
            st.fight = Some(crate::power::scaled_fight("sq_boss_king", cfg, st, vec![]));
        }
    }
    "sq_41_round".to_string()
}

fn boss_hp(st: &GameState) -> i32 {
    st.fight.as_ref().map(|f| f.hp.max(0)).unwrap_or(0)
}

/// 比赛胜负判定：胜利→boss_down；玩家倒下→death
fn boss_win(st: &mut GameState) -> String {
    if st.fight.as_ref().map(|f| f.hp > 0).unwrap_or(true) {
        return "sq_41_round".to_string();
    }
    crate::world::add_item(st, "it_sq_relic_hint");
    st.points += 600;
    st.set_flag("sq_boss_dead");
    st.fight = None;
    "sq_42_boss_down".to_string()
}

fn boss_dead(st: &mut GameState) -> String {
    if st.hp <= 0 { return "sq_50_death".to_string(); }
    "sq_41_round".to_string()
}

/// 每回合：先判定死/再生，再反击。
fn boss_act(st: &mut GameState, base: i32, weak: bool, guard: bool, finisher: bool) -> String {
    // 终结技：脱水重创（诱水剂）——固定 60 伤 + 永久停止再生
    if finisher {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - 60).max(0); }
        st.set_flag("sq_dehydrated");
        if boss_hp(st) <= 0 { return boss_win(st); }
        return boss_retaliate(st, guard);
    }
    // 普通攻击：弱火/弱电 ×1.3
    let dmg = if weak { ((base as f64) * 1.3) as i32 } else { base };
    if let Some(f) = st.fight.as_mut() {
        f.hp = (f.hp - dmg.max(0)).max(0);
    }
    if boss_hp(st) <= 0 { return boss_win(st); }
    // 狂暴触发 @96（40%）
    if boss_hp(st) <= 96 && !st.flag("sq_raged") {
        st.set_flag("sq_raged");
    }
    // 绿潮再生：狂暴后 +15，最多 3 回合
    if regen_active(st) {
        if let Some(f) = st.fight.as_mut() {
            f.hp = (f.hp + 15).min(f.max_hp);
        }
        mark_regen(st);
    }
    boss_retaliate(st, guard)
}

fn boss_retaliate(st: &mut GameState, guard: bool) -> String {
    if !guard {
        let raw = rng(18, 28);
        st.hp = (st.hp - raw).max(0);
    }
    boss_dead(st)
}

/// 战斗回合各攻击：火/电弱 ×1.3
fn atk_fire(st: &mut GameState) -> String { boss_act(st, rng(40, 50), true, false, false) }
fn atk_elect(st: &mut GameState) -> String { boss_act(st, rng(38, 46), true, false, false) }
fn atk_normal(st: &mut GameState) -> String { boss_act(st, rng(30, 40), false, false, false) }
fn atk_guard(st: &mut GameState) -> String { boss_act(st, 0, false, true, false) }
fn finisher_lure(st: &mut GameState) -> String { boss_act(st, 0, false, false, true) }

// 诱水剂终结技条件：持有诱水剂 且 兽王 HP<50%
fn cond_lure_finisher(st: &GameState) -> bool {
    inv(st, "it_sq_lure") && boss_hp(st) < 120
}

/* =====================================================================
   具名条件谓词（路线明示/解锁）
   ===================================================================== */
fn cond_autopsy_hint(st: &GameState) -> bool { st.flag("sq_autopsy_hint") }

/* =====================================================================
   战斗配置表（id 全 sq_ 前缀）。
   普通杂兵为「遭遇」用 fight 表；渴水兽王 BOSS 走选择驱动（此表仅提供定义/数值供结算与测试核对）。
   ===================================================================== */
fn sq_rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}
fn sq_win_f1(_st: &GameState) -> String { "sq_01_hub".to_string() }
fn sq_win_f2(_st: &GameState) -> String { "sq_10_f2".to_string() }
fn sq_win_f3(_st: &GameState) -> String { "sq_20_f3".to_string() }
fn sq_win_f4(_st: &GameState) -> String { "sq_30_f4".to_string() }
fn sq_win_boss(_st: &GameState) -> String { "sq_42_boss_down".to_string() }

pub fn shaqiu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("sq_f1_sandflea", FightCfg {
            name: "沙蚤群", hp: 34, dmg: (7, 12), reward: 10, reward_why: "击退扑咬的沙蚤群",
            intro: "一群沙色的甲虫从残骸阴影里跳出来，复眼血红，围着你的脚踝嗡嗡振翅。它们是这片废土的拾荒者，此刻把你当成了猎物。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f1, death: "sq_50_death",
        }),
        ("sq_f1_carrion", FightCfg {
            name: "尸骸爬虫", hp: 38, dmg: (8, 13), reward: 12, reward_why: "斩杀尸骸爬虫",
            intro: "一只四足爬行的东西从废墟墙根窜出，皮肤干裂成龟纹，三条刃螯在风里咔咔作响——它在腐肉堆里长大，闻到了活人的水汽。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f1, death: "sq_50_death",
        }),
        ("sq_f1_mut", FightCfg {
            name: "迷航幸存者·变异", hp: 42, dmg: (10, 16), reward: 25, reward_why: "终结了变异幸存者的痛苦",
            intro: "信号舱里那个蜷缩的人形忽然扭过头——干瘪的脸上半是绿皮苔藓，双目空洞。它曾是落在这颗星的幸存者，如今已和绿潮长在了一起。",
            rage_at: Some(20), rage_text: "变异幸存者发出一声不属于人类的嘶鸣，扑咬速度骤增！", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f1, death: "sq_50_death",
        }),
        ("sq_f2_sprout", FightCfg {
            name: "绿潮吞噬体·幼体", hp: 45, dmg: (8, 14), reward: 15, reward_why: "摆脱绿潮吞噬体缠绕",
            intro: "一团圆滚滚的绿色肉质球滚到你脚边，触须缠了上来，黏液拉丝，还在模仿着无害花苞的模样——绿潮最擅长用美伪装杀意。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f2, death: "sq_50_death",
        }),
        ("sq_f2_vine", FightCfg {
            name: "藤蔓猎手", hp: 52, dmg: (11, 17), reward: 25, reward_why: "斩断藤蔓猎手的缠绕",
            intro: "半植物半螳螂的身影从毒沼边缘窜出，藤蔓节肢、叶片刀刃前肢，带着绿腥气贴地俯冲——它是这片战场的猎手，专猎走动的「水源」。",
            rage_at: Some(24), rage_text: "藤蔓猎手震地裂开，藤鞭连击如雨点般砸来！", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f2, death: "sq_50_death",
        }),
        ("sq_f2_spore", FightCfg {
            name: "孢子喷射沼怪", hp: 60, dmg: (10, 16), reward: 25, reward_why: "击破孢子喷射沼怪",
            intro: "一个鼓胀的菌体从毒沼里浮起，顶部的喷腔渗出黄色的孢子雾。它不扑你，只是朝你喷出使人窒息的孢子——想榨干你肺里最后那口水汽。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f2, death: "sq_50_death",
        }),
        ("sq_f2_wrangler", FightCfg {
            name: "缉捕虫（精英）", hp: 88, dmg: (13, 19), reward: 80, reward_why: "击溃缉捕虫·精英",
            intro: "一头巨大的甲虫挡在营地的废墟外，苔纹甲壳泛着微光，虹吸口器对准了你——它是绿潮放出来的缉捕者，专门猎杀进入战场的活物。",
            rage_at: Some(40), rage_text: "缉捕虫甲壳硬化，足刺根根立起，攻势骤然凌厉！", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f2, death: "sq_50_death",
        }),
        ("sq_f3_larva", FightCfg {
            name: "幼虫蛆体", hp: 40, dmg: (7, 12), reward: 12, reward_why: "踩碎蠕动幼虫",
            intro: "孵化腔的黏壁上涌出几团白色黏蛆，透明体腔里五脏可见，顺着你鞋底往上爬。它们还小，但它们的母亲就在这墙后面。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f3, death: "sq_50_death",
        }),
        ("sq_f3_sguard", FightCfg {
            name: "孢子囊卫兵", hp: 58, dmg: (11, 17), reward: 20, reward_why: "解除孢子囊卫兵的封锁",
            intro: "一个被绿植整个包裹成植物人形的卫兵立在孢子囊走廊，头顶的孢囊一鼓一鼓，像在替母体站岗。它不移动，只是把出口堵得死死的。",
            rage_at: Some(25), rage_text: "卫兵孢囊炸开，喷出大片孢子云，侵蚀你的呼吸！", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f3, death: "sq_50_death",
        }),
        ("sq_f3_piercer", FightCfg {
            name: "膜腔穿刺者", hp: 66, dmg: (12, 18), reward: 25, reward_why: "避其针芒，击退穿刺者",
            intro: "狭长的膜翼在阴影里一收一合，半透明的肉质上拖着口器长针。它安静得像一条缝在墙上的影子，等你背对它时才缓缓探针。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f3, death: "sq_50_death",
        }),
        ("sq_f3_lpack", FightCfg {
            name: "幼虫群·增援", hp: 45, dmg: (8, 13), reward: 12, reward_why: "杀出幼虫群的包围",
            intro: "孵化腔口骤然涌出一整群幼虫，层层叠叠朝你滚来——母体感知到有人闯入，派出了增援。它们的数量比你的弹药多。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f3, death: "sq_50_death",
        }),
        ("sq_f3_soldier", FightCfg {
            name: "母巢侍卫（精英）", hp: 105, dmg: (15, 22), reward: 120, reward_why: "击溃母巢侍卫·精英",
            intro: "一头复合共生体战卫挡在子宫口前：藤甲裹着兽骨，兽骨里缝着人形骨骼，几双眼在甲缝里同时看你——这是母体的亲卫，绿潮最后的防线。",
            rage_at: Some(50), rage_text: "侍卫藤甲大片剥落，露出下面炽热的活肉，怒火倾泻而出！", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f3, death: "sq_50_death",
        }),
        ("sq_f4_echo", FightCfg {
            name: "深渊回声兽", hp: 90, dmg: (13, 19), reward: 100, reward_why: "令深渊回声兽沉寂",
            intro: "一头回声蝠形的半透明绿影在深渊回廊里荡来，声波纹一圈圈扩散，把你的惊叫加倍放大成恐惧。它只活在声音里，你安静它就消散。",
            rage_at: None, rage_text: "", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f4, death: "sq_50_death",
        }),
        ("sq_f4_knight", FightCfg {
            name: "水蛭骑士（精英）", hp: 110, dmg: (14, 21), reward: 120, reward_why: "击败水蛭骑士·精英",
            intro: "一个披着共生水蛭甲的人形骑士拦在你的竞技场西侧，脊背隆起大大的水囊，吸饱了水。它舔舐着干裂的口器——它不是在战斗，是在向你讨水。",
            rage_at: Some(45), rage_text: "水蛭骑士身上的水蛭同时蠕动吸血，重新爬满甲缝！", on_rage: sq_rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: sq_win_f4, death: "sq_50_death",
        }),
        ("sq_boss_king", FightCfg {
            name: "渴水兽王", hp: 240, dmg: (18, 28), reward: 600, reward_why: "击杀渴水兽王，夺回升空之井",
            intro: "王座石台的阴影里，那头庞然大物终于动了——半植物半兽的巨躯，外皮是墨绿苔壳，骨骼与脏器在壳下隐约可见，胸前裂口如同干涸的河床。它缓缓朝你转过三只眼，眼底没有杀意，只有……渴。这整颗星球都在替它喊渴。",
            rage_at: Some(96), rage_text: "渴水兽王体表苔壳爆裂，荧光绿血涌出，体型暴涨一格——它开始从地底的汲取池猛吸绿潮！",
            on_rage: sq_rage_none,
            finisher_if: |st, _| inv(st, "it_sq_lure") && boss_hp(st) < 120,
            finisher_name: |_| "倾倒诱水剂·脱水重创".to_string(),
            finisher_desc: |_| "你把一整瓶诱水剂泼进兽王胸前的河床裂口——它一生的渴在这一瞬被塞满，又在一瞬榨干。绿潮回涌戛然而止，巨兽如枯柴般一寸寸塌陷。".to_string(),
            win: sq_win_boss, death: "sq_50_death",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn shaqiu_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    shaqiu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   结算辅助（对齐 compute_settlement：total = points + 存活队友×100 + 支线flag×200）
   ===================================================================== */
fn shaqiu_side_count(st: &GameState) -> i32 {
    ["sq_side_survivor", "sq_side_autopsy", "sq_side_battery", "sq_side_trap", "sq_relic_seed"]
        .iter().filter(|k| st.flag(k)).count() as i32
}

/// 结算评级：S≥1600 / A≥1300 / B≥1000 / C≥700
fn sq_grade(total: i32) -> char {
    if total >= 1600 { 'S' } else if total >= 1300 { 'A' } else if total >= 1000 { 'B' } else if total >= 700 { 'C' } else { 'D' }
}

fn sq_route_settle(st: &mut GameState) -> String {
    let side = shaqiu_side_count(st) * 200;
    let alive = st.alive_count() * 100;
    let total = st.points + alive + side;
    st.sp_grade = Some(sq_grade(total));
    "sq_45_card".to_string()
}

fn sq_route_to_f2(st: &mut GameState) -> String { oxy_tick(st); "sq_10_f2".to_string() }
fn sq_route_to_f3(st: &mut GameState) -> String { oxy_tick(st); "sq_20_f3".to_string() }
fn sq_route_to_f4(st: &mut GameState) -> String { oxy_tick(st); "sq_30_f4".to_string() }
fn sq_route_hub1(st: &mut GameState) -> String { oxy_tick(st); "sq_01_hub".to_string() }
fn sq_route_hub2(st: &mut GameState) -> String { oxy_tick(st); "sq_10_f2".to_string() }
fn sq_route_hub3(st: &mut GameState) -> String { oxy_tick(st); "sq_20_f3".to_string() }
fn sq_route_hub4(st: &mut GameState) -> String { oxy_tick(st); "sq_30_f4".to_string() }
fn sq_route_stay(st: &mut GameState) -> String { let _ = st; "sq_10_f2".to_string() }

/* =====================================================================
   剧情场景（id 全部 sq_ 前缀）
   ===================================================================== */
pub static SHAQIU_SCENES: &[SceneDef] = &[

    /* ================= 幕1 · 开场：坠落 ================= */
    SceneDef {
        id: "sq_00_intro", bg: Some("shaqiu_bg.png"), loc: Some("坠毁穿梭机残骸 · 西北沙丘"),
        mood: "fear", speaker: Some("系统播报 → 黑铁"), voice: Some("vo_sq_intro"),
        text: TextSpec::Static(&[
            "主神空间投放完成。穿梭机残骸在身后冒着青烟，黄沙正一寸寸吞没它的轮廓。氧气表开始滴答——你们只剩下压缩空气罐里那点生命。",
            "黑铁突然抬头：『……有东西在往这边爬，不快，但是很多。』黄沙的地平线上，那抹绿色像退潮的浪，一层层漫过来。",
            "<em>「绿潮吞没显示器之前，你先看清了它有多美。」</em>",
        ]),
        choices: &[
            ChoiceDef { label: "先侦查信号方向", sub: "Eff: SetFlag recon + Points 20", cond: None,
                effects: &[Eff::SetFlag("sq_recon"), Eff::Points(20)], route: Route::To("sq_01_hub") },
            ChoiceDef { label: "立刻搜索近处舱段", sub: "Points 10", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("sq_01_hub") },
            ChoiceDef { label: "原地架设警戒（耗时）", sub: "Points 5 · San -5（绿潮逼近感）", cond: None,
                effects: &[Eff::Points(5), Eff::San(-5)], route: Route::To("sq_01_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F1 · 坠毁穿梭机残骸 ================= */
    SceneDef {
        id: "sq_01_hub", bg: Some("shaqiu_bg.png"), loc: Some("F1 · 坠毁穿梭机残骸"),
        mood: "fear", speaker: Some("黑铁"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_oxy_3") {
                format!("残骸里的空气越来越稀薄，氧气表针已经压进红色区。你还能在这片铁壳里待一会，但沙海深处——那里的空气更贵，也更危险。")
            } else {
                "你站在残骸的骨架里。四处散落着焦黑的舱段：驾驶舱的黑匣子、生活舱的解剖台、储物柜，以及信号舱那盏仍在一明一灭的信标。黄沙在舷窗外缓慢地流。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "调查黑匣子", sub: "坠落前 40 秒的真相", cond: None, effects: &NO_EFF, route: Route::To("sq_02_blackbox") },
            ChoiceDef { label: "查看解剖台", sub: "支线线索", cond: None, effects: &NO_EFF, route: Route::To("sq_03_autopsy") },
            ChoiceDef { label: "翻找储物柜", sub: "Item it_sq_pry", cond: None, effects: &NO_EFF, route: Route::To("sq_05_locker") },
            ChoiceDef { label: "靠近信号发射器", sub: "求救信号", cond: None, effects: &NO_EFF, route: Route::To("sq_04_signal") },
            ChoiceDef { label: "翻越北侧残骸，进入沙海", sub: "pt_sq_12 · 前往 F2 绿潮战场", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_to_f2) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_02_blackbox", bg: Some("shaqiu_bg.png"), loc: Some("F1 · 驾驶舱 · 黑匣子"),
        mood: "cold", speaker: Some("数据终端 → 波丽"), voice: Some("vo_sq_cockpit"),
        text: TextSpec::Static(&[
            "黑匣子回放着坠毁前 40 秒：不是故障——是被拖下去的。那团绿色从地平线漫过来，像海，把穿梭机整个囫囵吞了下去。",
            "信号发射器仍在广播：『……沙丘魔海，着陆点 K7，幸存者 3 人……』僵尸般重复。",
        ]),
        choices: &[
            ChoiceDef { label: "解析灰盒（波丽伴线）", sub: "SetFlag decrypt_ok · Points 15", cond: None,
                effects: &[Eff::SetFlag("sq_decrypt_ok"), Eff::Points(15)], route: Route::Dyn(sq_route_hub1) },
            ChoiceDef { label: "先处理解剖台", sub: "SetFlag autopsy_hint", cond: None,
                effects: &[Eff::SetFlag("sq_autopsy_hint")], route: Route::Dyn(sq_route_hub1) },
            ChoiceDef { label: "直接上路", sub: "返回北侧沙丘", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub1) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_03_autopsy", bg: Some("shaqiu_bg.png"), loc: Some("F1 · 生活舱 · 解剖台"),
        mood: "cold", speaker: Some("依凡"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_autopsy_hint") {
                "解剖台上残留着细小的绿色组织切片，依凡的笔记写着：『共生体组织对母体炎症血清敏感。若有足够样本，或能合成亲和通道。』你已记下这条线索。".to_string()
            } else {
                "解剖台上摊着几片风干的绿色组织，刀痕干净利落——是依凡留下的研究现场。他让你记下：「卵堆里的炎症血清，也许能骗过母体。」".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "记下解剖台（开启卵堆解剖链）", sub: "SetFlag autopsy_hint", cond: None,
                effects: &[Eff::SetFlag("sq_autopsy_hint"), Eff::Points(10)], route: Route::Dyn(sq_route_hub1) },
            ChoiceDef { label: "返回", sub: "回到残骸主厅", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub1) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_04_signal", bg: Some("shaqiu_bg.png"), loc: Some("F1 · 信号舱 · 信号发射器"),
        mood: "mystery", speaker: Some("广播里的幸存者"), voice: Some("vo_sq_signal"),
        text: TextSpec::Static(&[
            "信标在静电里一明一灭，重复着那句求救。信号源从沙海深处传来——沿着它，你可能会找到第二个出生在绿潮里的人。你记下了方位。",
            "『……着陆点 K7，幸存者 3 人……』",
        ]),
        choices: &[
            ChoiceDef { label: "记下求救信号", sub: "Points 10", cond: None,
                effects: &[Eff::Points(10), Eff::SetFlag("sq_signal_heard")], route: Route::Dyn(sq_route_hub1) },
            ChoiceDef { label: "返回", sub: "回到残骸主厅", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub1) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_05_locker", bg: Some("shaqiu_bg.png"), loc: Some("F1 · 生活舱 · 储物柜"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["储物柜的水汽锈住合页，你用力撬开——里面斜躺着一根粗壮的「磁力撬具」。它比撬柜子有用得多：南侧那个塌方的闸门，正等着它。"]),
        choices: &[
            ChoiceDef { label: "撬出磁力撬具", sub: "Item it_sq_pry · 可开 g1 塌方闸门", cond: None,
                effects: &[Eff::AddItem("it_sq_pry"), Eff::MarkPoint("pt_sq_locker")], route: Route::Dyn(sq_route_hub1) },
            ChoiceDef { label: "返回", sub: "回到残骸主厅", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub1) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F2 · 沙漠地表 · 绿潮战场 ================= */
    SceneDef {
        id: "sq_10_f2", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 营地B · 补给站废墟"),
        mood: "fear", speaker: Some("格列弗 → 全体"), voice: Some("vo_sq_onslaught"),
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_oxy_3") {
                "沙海上的空气滚烫而稀薄，氧气表已压进红色区。远处那片狰狞之绿还在蔓延——你得快点，要么找到面罩和氧气，要么尽快穿过这块活着的土地。".to_string()
            } else {
                "你穿过风蚀隧道，踏入真正的沙地。废屋、半埋的补给箱、以及远处那堵正在一格格吞噬地平线的「狰狞之绿」幕墙。营地里还困着人。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "翻找医疗箱", sub: "Item it_sq_mask 滤毒面罩", cond: None, effects: &NO_EFF, route: Route::To("sq_11_medbox") },
            ChoiceDef { label: "靠近被困幸存者", sub: "幕3 · 救人抉择", cond: None, effects: &NO_EFF, route: Route::To("sq_12_survivor") },
            ChoiceDef { label: "检查蓄电池库", sub: "幕4 · 三选一生存抉择", cond: None, effects: &NO_EFF, route: Route::To("sq_13_battery") },
            ChoiceDef { label: "听鹰·清晰者指路", sub: "角色 · 补氧信息", cond: None, effects: &NO_EFF, route: Route::To("sq_npc_hawk") },
            ChoiceDef { label: "在通风台休整", sub: "不推进氧耗 · Points 5", cond: None, effects: &NO_EFF, route: Route::To("sq_14_vent") },
            ChoiceDef { label: "深入绿潮母巢入口", sub: "pt_sq_23 · 前往 F3 共生体母巢", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_to_f3) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_11_medbox", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 营地A · 医疗箱"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["医疗箱里静静躺着一副「滤毒面罩」。套上它，你能在沼泽孢子毒气区里直穿而不被榨干肺里的水汽——那是穿过这片战场最省氧的路。"]),
        choices: &[
            ChoiceDef { label: "戴上滤毒面罩", sub: "Item it_sq_mask · 不推进氧耗", cond: None,
                effects: &[Eff::AddItem("it_sq_mask"), Eff::MarkPoint("pt_sq_medbox")], route: Route::Dyn(sq_route_stay) },
            ChoiceDef { label: "返回", sub: "回到营地", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_stay) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_12_survivor", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 营地A · 被吞没的哨站"),
        mood: "fear", speaker: Some("格列弗 → 全体"), voice: Some("vo_sq_gliff"),
        text: TextSpec::Static(&[
            "你抵达时，哨站正被绿潮一点一点嚼碎。铁皮房梁下压着一只仍在挥动的手——那是信号里的幸存者格列弗。幕墙正在合拢，你只有一次选择的机会。",
            "格列弗虚弱地喊：『别管我……那绿色会喝水！你们身上每滴都是它的水！』",
        ]),
        choices: &[
            ChoiceDef { label: "冲进去救人", sub: "SetFlag side_survivor · Points 30 · Item mask · San -10", cond: None,
                effects: &[Eff::SetFlag("sq_side_survivor"), Eff::Points(30), Eff::AddItem("it_sq_mask"), Eff::San(-10)],
                route: Route::Dyn(sq_route_hub2) },
            ChoiceDef { label: "保持距离绕行", sub: "错过 side_survivor · 不额外耗氧", cond: None,
                effects: &NO_EFF, route: Route::Dyn(sq_route_hub2) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_13_battery", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 营地B · 补给站废墟"),
        mood: "cold", speaker: Some("引擎终端 → 全体"), voice: Some("vo_sq_battery"),
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_battery_saved") || st.flag("sq_gate_powered") || st.flag("sq_bait_lure") {
                "蓄电池库空了，你把最后那点电能喂给了它该去的地方。营地的电表归零，风沙重新灌进废墟。".to_string()
            } else {
                "发电机还留着一格电。蓄电池库只有一块可用的外接电池。氧气表已进入黄色区——走廊尽头的闸门、求救信标、防潮幕帘，只能喂饱一样。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "给信标供电", sub: "SetFlag battery_saved · Points 40 · 结局救援加成", cond: None,
                effects: &[Eff::SetFlag("sq_battery_saved"), Eff::Points(40)], route: Route::Dyn(sq_route_hub2) },
            ChoiceDef { label: "给南侧闸门供电", sub: "SetFlag gate_powered · Points 20 · 降低后续氧耗", cond: None,
                effects: &[Eff::SetFlag("sq_gate_powered"), Eff::Points(20)], route: Route::Dyn(sq_route_hub2) },
            ChoiceDef { label: "做成绿潮诱饵", sub: "SetFlag bait_lure · Points 20 · 母巢减扰", cond: None,
                effects: &[Eff::SetFlag("sq_bait_lure"), Eff::Points(20)], route: Route::Dyn(sq_route_hub2) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_14_vent", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 沼泽孢子毒气区 · 中央通风台"),
        mood: "calm", speaker: Some("黑铁"), voice: None,
        text: TextSpec::Static(&["你爬到毒气区中央那座安全通风台上，灌了两口净水，氧气表针缓了一缓。这台子四周的绿潮像不敢近前，只在几步外静静涌着。短暂的安宁。"]),
        choices: &[
            ChoiceDef { label: "休整片刻", sub: "Points 5 · 不推进氧耗", cond: None,
                effects: &[Eff::Points(5)], route: Route::Dyn(sq_route_stay) },
            ChoiceDef { label: "返回营地B", sub: "回到补给站废墟", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_stay) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_npc_hawk", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 北缘观景点 · 鹰"),
        mood: "mystery", speaker: Some("鹰·清晰者"), voice: None,
        text: TextSpec::Static(&["一只灰羽的鹰立在北缘断天线顶，用一种几乎懂你说话的眼神看你。它啄了啄自己的羽翼，朝母巢方向偏了偏头——它在告诉你，条条大路都通那片绿色更深处。"]),
        choices: &[
            ChoiceDef { label: "顺着鹰的指向望去", sub: "Points 10 · 不推进氧耗", cond: None,
                effects: &[Eff::Points(10), Eff::SetFlag("sq_hawk_met")], route: Route::Dyn(sq_route_stay) },
            ChoiceDef { label: "返回营地B", sub: "回到补给站废墟", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_stay) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F3 · 共生体母巢 ================= */
    SceneDef {
        id: "sq_20_f3", bg: Some("shaqiu_bg.png"), loc: Some("F3 · 共生体母巢 · 入口肉厅"),
        mood: "cold", speaker: Some("黑铁"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_spore_serum") {
                "你攥着那管孢子血清，亮绿色的脉动像握着一小颗心脏。母体的通道认出了亲缘，在你面前缓缓让开。子宫口就在走廊尽头。".to_string()
            } else {
                "活体肉厅在四壁微微搏动，像呼吸。孢子囊走廊通向深处，孵化腔的卵堆隐隐透着光。空气里是黏腻的潮味。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "解剖孵化腔卵堆", sub: "支线 · 需 autopsy_hint", cond: None, effects: &NO_EFF, route: Route::To("sq_21_eggnest") },
            ChoiceDef { label: "检查孵化膜", sub: "母巢情报 · 弱酸溶解线索", cond: None, effects: &NO_EFF, route: Route::To("sq_22_hatch") },
            ChoiceDef { label: "深入子宫口", sub: "pt_sq_34 · 前往 F4 沙丘洞穴", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_to_f4) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_21_eggnest", bg: Some("shaqiu_bg.png"), loc: Some("F3 · 孵化腔 · 卵堆"),
        mood: "cold", speaker: Some("依凡"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_autopsy_hint") {
                "有了 F1 解剖台上的笔记，你辨认出卵堆里那粒半透明的囊——取一些组织，依凡的配方能把它炼成一管「孢子血清」，母体亲和的凭证。".to_string()
            } else {
                "卵堆在你眼前轻微搏动，像无数颗回望你的眼睛。你隐约觉得该在解剖台上先学会辨认它们，但眼下无从下手。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "解剖卵堆（合成孢子血清）", sub: "SetFlag spore_serum · Item it_sq_serum", cond: Some(cond_autopsy_hint),
                effects: &[Eff::SetFlag("sq_spore_serum"), Eff::SetFlag("sq_side_autopsy"), Eff::AddItem("it_sq_serum"), Eff::Points(20)],
                route: Route::Dyn(sq_route_hub3) },
            ChoiceDef { label: "暂不知如何下手", sub: "仍需 autopsy_hint 线索", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub3) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_22_hatch", bg: Some("shaqiu_bg.png"), loc: Some("F3 · 孵化膜"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["一层极薄的孵化膜挡在通道某处，你看出它怕酸——只要兑上某种能溶解黏蛋白的液体，它就会软化成可以挤过去的果冻。你把这记在心上。"]),
        choices: &[
            ChoiceDef { label: "触碰孵化膜", sub: "Points 10", cond: None,
                effects: &[Eff::Points(10), Eff::SetFlag("sq_hatch_seen")], route: Route::Dyn(sq_route_hub3) },
            ChoiceDef { label: "返回", sub: "回到入口肉厅", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub3) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F4 · 沙丘洞穴 · 深渊回响 ================= */
    SceneDef {
        id: "sq_30_f4", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 沙丘洞穴 · 前厅"),
        mood: "fear", speaker: Some("黑铁"), voice: Some("vo_sq_whisper"),
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_boss_secret") {
                "你看过墙上的壁画，知道这头「渴水兽王」正沉在极深的缺水里——把它体内的水榨干，它就只是一堆枯柴。前方便是它的王座。".to_string()
            } else {
                "穿过子宫口，抵达地底的沙丘洞穴。石笋与深渊回廊在前，回廊尽头的王座石台下，隐约有一道会呼吸的巨大阴影。风里带着一丝潮腥。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "查看壁画", sub: "SetFlag boss_secret · 揭示脱水弱点", cond: None, effects: &NO_EFF, route: Route::To("sq_31_mural") },
            ChoiceDef { label: "触碰王座石台", sub: "BOSS 前奏氛围", cond: None, effects: &NO_EFF, route: Route::To("sq_32_throne") },
            ChoiceDef { label: "走向深渊回廊尽头", sub: "幕5 · 沉洞骗局", cond: None, effects: &NO_EFF, route: Route::To("sq_33_trap") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_31_mural", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 回音长廊 · 壁画前"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["壁画上，远古生物把成桶的水倒进王座下的裂缝，那头「王」便沉沉睡去。你在刻痕旁补了一句批注：水，是它唯一的命门。"]),
        choices: &[
            ChoiceDef { label: "记下弱水之秘", sub: "SetFlag boss_secret · Points 15", cond: None,
                effects: &[Eff::SetFlag("sq_boss_secret"), Eff::Points(15), Eff::MarkPoint("pt_sq_mural")], route: Route::Dyn(sq_route_hub4) },
            ChoiceDef { label: "返回", sub: "回到前厅", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub4) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_32_throne", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 王座石台"),
        mood: "mystery", speaker: Some("黑铁"), voice: None,
        text: TextSpec::Static(&["王座石台四周散着大片的干涸裂缝，你甚至能听见地底深处有汩汩的、近似渴望的声响。那不是水声——是某头渴极了的东西，翻了个身。"]),
        choices: &[
            ChoiceDef { label: "记住这份渴", sub: "Points 5", cond: None,
                effects: &[Eff::Points(5), Eff::SetFlag("sq_throne_seen")], route: Route::Dyn(sq_route_hub4) },
            ChoiceDef { label: "返回", sub: "回到前厅", cond: None, effects: &NO_EFF, route: Route::Dyn(sq_route_hub4) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_33_trap", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 回音长廊深处"),
        mood: "cold", speaker: Some("假『水玉文明』全息贵族"), voice: Some("vo_sq_trap"),
        text: TextSpec::Static(&[
            "空气突然清亮。一幅全息影像升起：自称『水玉文明』的发光贵族，用完美的人类语邀请你们『献上航标，换取取之不尽的净化水』。它太礼貌了，礼貌得像一段广告。",
            "黑铁低声说：『它没有恶意——它根本没有善意。它在馋我们身上的水分。』",
        ]),
        choices: &[
            ChoiceDef { label: "拆穿骗局（需 decrypt_ok）", sub: "SetFlag side_trap · Points 80 · 规避伏击", cond: Some(has_decrypt),
                effects: &[Eff::SetFlag("sq_side_trap"), Eff::Points(80)], route: Route::To("sq_34_whisper") },
            ChoiceDef { label: "假装顺从", sub: "San -15 · 引蛇出洞情报 · 40 点", cond: None,
                effects: &[Eff::San(-15), Eff::Points(40)], route: Route::To("sq_34_whisper") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_34_whisper", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 回音长廊 · 壁画前"),
        mood: "cold", speaker: Some("黑铁"), voice: Some("vo_sq_whisper"),
        text: TextSpec::Dyn(|st| {
            if st.flag("sq_boss_secret") {
                "黑铁按住太阳穴：『渴。它快渴死了。这整颗星球都在替它喊渴。』你看着壁画，心中已有了完整的一击：『把它体内的水榨干，它就只是一堆枯柴。』".to_string()
            } else {
                "黑铁按住太阳穴：『渴。它快渴死了。』他的声音沉下来：「它朝水来——你们可以顺它的意，也可以反着来。」".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "制作诱水剂（蓄电池余料+绿潮汁液）", sub: "SetFlag lure_made · Item it_sq_lure", cond: None,
                effects: &[Eff::SetFlag("sq_lure_made"), Eff::AddItem("it_sq_lure")], route: Route::To("sq_40_boss_intro") },
            ChoiceDef { label: "放弃取巧，硬拼", sub: "Points 20 · 全体 HP+10", cond: None,
                effects: &[Eff::Points(20), Eff::Hurt(-10, "")], route: Route::To("sq_40_boss_intro") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= BOSS · 渴水兽王 ================= */
    SceneDef {
        id: "sq_40_boss_intro", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 王座擂地 · 渴水兽王"),
        mood: "danger", speaker: Some("渴水兽王"), voice: Some("vo_sq_boss"),
        text: TextSpec::Static(&[
            "王座下的裂缝轰然炸开，绿潮从地底喷涌而出，那头巨兽撑起半植物半兽的躯体，苔壳寸寸隆起。它没有咆哮——它只是缓缓朝你伸出前爪，裂口微微开合，像在标本里呼出的最后一口气也要掬起来。",
            "黑铁的声音在脑内回响：「它在渴。」",
        ]),
        choices: &[
            ChoiceDef { label: "【面向渴水兽王】", sub: "进入选择驱动决战", cond: None,
                effects: &NO_EFF, route: Route::Dyn(start_boss) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_41_round", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 王座擂地 · 决战"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            let hp = boss_hp(st);
            let max = st.fight.as_ref().map(|f| f.max_hp).unwrap_or(240);
            let mode = if st.flag("sq_dehydrated") {
                "——绿潮已断，兽王失去再生，正像枯柴般塌陷——"
            } else if st.flag("sq_raged") {
                "——狂暴再生：苔壳爆裂，绿潮自汲取池回涌 +15——"
            } else {
                "——常态 · 半植物半兽的巨躯，眼底仍是那份渴——"
            };
            format!("渴水兽王 HP {}/{}。\n{mode}\n(弱火 ×1.3 / 弱电 ×1.3；持有诱水剂且 HP<120 时可终结)",
                hp.max(0), max)
        }),
        choices: &[
            ChoiceDef { label: "火焰冲击", sub: "弱火 ×1.3 · 伤害 40-50", cond: None,
                effects: &NO_EFF, route: Route::Dyn(atk_fire) },
            ChoiceDef { label: "电弧放电", sub: "弱电 ×1.3 · 伤害 38-46", cond: None,
                effects: &NO_EFF, route: Route::Dyn(atk_elect) },
            ChoiceDef { label: "普通猛攻", sub: "无克制 · 伤害 30-40", cond: None,
                effects: &NO_EFF, route: Route::Dyn(atk_normal) },
            ChoiceDef { label: "【倾倒诱水剂·脱水重创】", sub: "需诱水剂 + HP<50% · 60 固定伤 · 永久断再生", cond: Some(cond_lure_finisher),
                effects: &NO_EFF, route: Route::Dyn(finisher_lure) },
            ChoiceDef { label: "侧身蓄力·回避", sub: "本回合免伤", cond: None,
                effects: &NO_EFF, route: Route::Dyn(atk_guard) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_42_boss_down", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 王座擂地 · 兽王既殁"),
        mood: "calm", speaker: Some("希望号通讯"), voice: Some("vo_sq_down"),
        text: TextSpec::Static(&[
            "渴水兽王倒下的最后一刻，整座洞穴的绿潮同时枯萎了一瞬——像是行星叹了口气。它胸前那条干涸河床缓缓合拢，变得滚烫而干。",
            "圣物室的潮锁门在酸液中溶解，石台上躺着一枚沉甸甸的种子。你听见更远处的井口，有风正在灌下来。",
        ]),
        choices: &[
            ChoiceDef { label: "（走近圣物室）", sub: "潮锁门已因王尸酸液洞开", cond: None, effects: &NO_EFF, route: Route::To("sq_43_relic") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_43_relic", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 圣物室 · 遗泽种子"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["圣物室的正中，石台上静静躺着一枚沉甸甸的「遗泽种子」，像一粒小太阳。它不属于草木，也不属于机械——它是这颗星最深处那份求生的意志，凝成的一枚种子。"]),
        choices: &[
            ChoiceDef { label: "取走遗泽种子并升空", sub: "SetFlag relic_seed · Item it_sq_relic_seed", cond: None,
                effects: &[Eff::SetFlag("sq_relic_seed"), Eff::AddItem("it_sq_relic_seed"), Eff::MarkPoint("pt_sq_relic")],
                route: Route::To("sq_44_victory") },
            ChoiceDef { label: "升空（放弃种子，速逃）", sub: "少一枚结算支线物", cond: None,
                effects: &NO_EFF, route: Route::To("sq_44_victory") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "sq_44_victory", bg: Some("shaqiu_bg.png"), loc: Some("F4 · 沙丘洞穴 · 升空井"),
        mood: "calm", speaker: Some("希望号通讯 → 全体"), voice: Some("vo_sq_rise"),
        text: TextSpec::Dyn(|st| {
            let extra = if st.flag("sq_battery_saved") {
                "信标指引下，救援舱精准降落在坠毁点，把你们近乎是在升起的瞬间就接上了船。"
            } else if st.flag("sq_side_trap") {
                "被拆穿的骗局，成为希望号对星海的第一课——船长在频道里郑重地记下了你们的名字。"
            } else { "" };
            format!(
                "升空井在脚下亮起一道灼目的光柱，希望号的救援信号穿破尘暴：『重生点已锁定，欢迎回家。』\n{}",
                extra
            )
        }),
        choices: &[
            ChoiceDef { label: "（按照落着想，升空）", sub: "按全程结算", cond: None,
                effects: &NO_EFF, route: Route::Dyn(sq_route_settle) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 结算卡片（主神空间衔接由主线接线 __enter_nexus__） ================= */
    SceneDef {
        id: "sq_45_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_sq_settle"), death: None,
            card: |st| crate::state::Card {
                title: "沙 丘 魔 海 · 遗 泽".into(), good: true,
                body_html: format!(
                    "<p>在绿潮吞没屏幕之前，你看清了它有多美。你从坠毁的残骸出发，穿过绿潮战场与共生体母巢，掘出渴水兽王，带着「遗泽种子」升上归途。</p>\
                     <p style='color:#4fbf4a'>「这颗星球的绿色会吞掉一切为它解渴的东西——而你把它的渴，一并带回了家。」</p>\
                     <table class='statTable'>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>支线完成</td><td>{} / 5 项</td></tr>\
                     <tr><td>遗泽评级</td><td style='color:#ffd76a'>{}</td></tr>\
                     </table>\
                     <p style='color:#9ff'>{}</p>",
                    st.points,
                    shaqiu_side_count(st),
                    if st.sp_grade == Some('S') { "S · 全遗也" } else if st.sp_grade == Some('A') { "A · 殊途而归" }
                        else if st.sp_grade == Some('B') { "B · 有始有终" } else { "C · 惊鸿一瞥" },
                    if st.flag("sq_relic_seed") { "【遗泽种子已携回 —— 可在主神空间上架二级文明·高斯武器线】".to_string() } else { "未携回遗泽种子，高斯武器线保持『目击未交付』锁定态。".to_string() }
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= NPC 对话（talk 节点） ================= */
    SceneDef {
        id: "sq_npc_gliff", bg: Some("shaqiu_bg.png"), loc: Some("F2 · 获救后的格列弗"),
        mood: "cold", speaker: Some("格列弗"), voice: None,
        text: TextSpec::Static(&["获救的格列弗蜷在通风台边灌着水，抬头看你：「那绿色……会喝水。你们身上的每滴都是它的水。想要穿过它，就把它眼里最值钱的东西——水——反着用。」"]),
        choices: &[ChoiceDef { label: "记下他的话", sub: "Points 10", cond: None,
            effects: &[Eff::Points(10)], route: Route::Dyn(sq_route_hub2) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 死亡档案 ================= */
    SceneDef {
        id: "sq_50_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("沙丘魔海 · 任务世界 阵亡", "被绿潮榨干了最后一口水汽")), card: |_st| crate::state::Card {
                title: "沙 丘 之 渴".into(), good: false,
                body_html: r#"<p>你的肺里最后一口水汽被这片绿潮抽走，意识像被黄沙一寸寸埋过。沙丘魔海依旧在渴望——而你的「渴」，化进了它的潮里。</p>
<p style='color:#ff8a8a'>【死亡档案 · 沙丘魔海 · 渴毙者】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。）</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];
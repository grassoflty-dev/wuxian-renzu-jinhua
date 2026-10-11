//! 《天蛇族地下实验室·零号基地》任务世界 · 全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/honghuang_li/tianshe_lab.md §4/§5/§6/§7/§8。
//! 本文件为全新新增文件，只导出静态数据（TIANSHE_SCENES / tianshe_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/tianshe_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `ts_` 前缀，与既有 SCENES 无重名。
//! 残页收集用方案 A：连号 flag `ts_archive_1..8`，结算计数用 flag 数（ts_archive_count）。
//! 镜像 flag 链：mirror_1/2/3 → ts_mirror_line_3（结算 +200）。
//! BOSS 二段战：阶段一 `mulaba`（标准 fight，finisher=HP<150 逼退）→ 转场 ts_boss1_retreat
//!   → 阶段二 `apocalypse_snake`（选择驱动遭遇，样本共鸣终结 vs 灭世蜕皮 3 回合倒计时灭团）。

use crate::defs::*;
use crate::state::GameState;
use rand::Rng;

/// 空 effect / choice 惯用静态（同 scenes.rs / scenes_zhouyuan.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rnd_ts(a: i32, b: i32) -> i32 {
    rand::thread_rng().gen_range(a..=b)
}

/// 持有初蛇基因样本
fn has_sample(st: &GameState) -> bool {
    st.inventory.iter().any(|i| i == "item_chushe_sample")
}
/// 持有草药麻醉剂（miniboss 终结技条件）
fn has_anesthetic(st: &GameState) -> bool {
    st.inventory.iter().any(|i| i == "item_anesthetic")
}

/// 初蛇聚合体当前 HP
fn snake_hp(st: &GameState) -> i32 {
    st.fight.as_ref().map(|f| f.hp).unwrap_or(540)
}
/// 灭世蜕皮倒计时层数（0..3）
fn apoc_round(st: &GameState) -> usize {
    ["ts_apoc_1", "ts_apoc_2", "ts_apoc_3"].iter().filter(|k| st.flag(k)).count()
}
/// 样本共鸣可用：持样本 且 幼体 HP<100
fn cond_sample_resonance(st: &GameState) -> bool {
    has_sample(st) && snake_hp(st) < 100
}
/// 理智足以同时交两个狱友（San≥50）
fn cond_psych(st: &GameState) -> bool { st.san >= 50 }
/// 开场选择装死 → 可走通风口捷径
fn cond_wake_quiet(st: &GameState) -> bool { st.flag("wake_quiet") }
/// 已信任阿莲 → 幕 4 可先救流水线上的她
fn cond_trust_alien(st: &GameState) -> bool { st.flag("trust_alien_1") }
/// 已持有希望模板残光 → 幕 6 隐藏支线
fn cond_hope_light(st: &GameState) -> bool {
    st.inventory.iter().any(|i| i == "item_hope_light")
}

/// 残页收集计数（方案 A 连号 flag）
pub fn ts_archive_count(st: &GameState) -> usize {
    (1..=8).filter(|n| st.flag(&format!("ts_archive_{n}"))).count()
}

/// 收集任一残页后：满 8 张自动挂 aggregate flag（结算项）
fn route_archive_count(st: &mut GameState) -> String {
    if ts_archive_count(st) >= 8 {
        st.set_flag("ts_archive_all");
    }
    "ts_hall".to_string()
}

/// 阶段二初始化：用 apocalypse_snake 的 FightCfg 建 Fight（参考 zhouyuan start_kayako）
fn start_snake(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("apocalypse_snake") {
            st.fight = Some(crate::power::scaled_fight("apocalypse_snake", cfg, st, vec![]));
        }
    }
    st.set_flag("boss1_retreated");
    "ts_boss2_round".to_string()
}

/// 阶段二：一\"回\"。幼体 HP<100 且无样本 → 灭世蜕皮护体（打不动，倒计时推进，3 回灭团）。
/// 可正常伤害 → HP≤0 胜利；持样本后玩家可命中直至 HP<100 再样本共鸣。
fn route_snake_attack(st: &mut GameState, dmg: i32) -> String {
    let low = snake_hp(st) < 100;
    let shed = low && !has_sample(st);
    if shed {
        // 灭世蜕皮：幼体被蛇蜕护住，无法造成伤害；倒计时 1→2→3，到 3 灭团
        if !st.flag("ts_apoc_1") {
            st.set_flag("ts_apoc_1");
        } else if !st.flag("ts_apoc_2") {
            st.set_flag("ts_apoc_2");
        } else if !st.flag("ts_apoc_3") {
            st.set_flag("ts_apoc_3"); // 倒计时 {seen}/3 达 3 → 即刻灭团
            return "ts_boss2_wipe".to_string();
        }
    } else {
        if let Some(f) = st.fight.as_mut() {
            f.hp = (f.hp - dmg.max(0)).max(0);
        }
        if snake_hp(st) <= 0 {
            return "ts_boss2_win".to_string();
        }
    }
    // 幼体反击
    let p = rnd_ts(18, 30);
    st.hp = (st.hp - p).max(0);
    if st.hp <= 0 {
        return "ts_boss2_lose".to_string();
    }
    "ts_boss2_round".to_string()
}

/// 阶段二：样本共鸣（直接结算胜利）
fn route_snake_resonance(st: &mut GameState) -> String {
    st.set_flag("sample_resonance");
    "ts_boss2_win".to_string()
}

/// 阶段二回合文本
fn txt_boss2_round(st: &GameState) -> String {
    let hp = snake_hp(st);
    let seen = apoc_round(st);
    let head = if seen > 0 {
        format!(
            "<span style='color:#ff6a6a'>【灭世蜕皮】幼体全身蛇蜕暴涨护体，倒计时 {seen}/3——若无法共鸣，3 回后你将被抹去。</span>"
        )
    } else if hp < 100 && !has_sample(st) {
        "幼体的气息骤变，鳞甲开始蜕裂……（下一回将进入「灭世蜕皮」倒计时）".to_string()
    } else {
        "初蛇聚合体盘踞熔炉之上，多首低垂，万族面孔的浮雕在血肉里蠕动。".to_string()
    };
    format!(
        "{head}\n\n<b>初蛇基因聚合体 · 灭世之蛇幼体</b>　HP {hp}/540\n\n（蛇蜕摩擦声、粘稠蠕动在空气中蔓延。）"
    )
}

/// 结算：写 sp_grade='D'（Route::Dyn 落地），置 dungeon_cleared
fn route_finalize(st: &mut GameState) -> String {
    st.sp_grade = Some('D');
    st.set_flag("dungeon_cleared");
    if ts_archive_count(st) >= 8 {
        st.set_flag("ts_archive_all");
    }
    "ts_finish".to_string()
}

/* =====================================================================
   战斗配置表（TS 专属；导出供主线把 query 扩展进来）
   ===================================================================== */
fn ts_rage_common(st: &mut GameState, log: &mut Vec<String>) {
    let _ = st;
    log.push("<span class='crit'>蛇纹燃亮，鳞甲下的暴虐彻底释放——它的攻势更烈了！</span>".into());
}
fn win_hall(_st: &GameState) -> String { "ts_hall".into() }

pub fn tianshe_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        // ---- L1 ----
        ("tianshe_guard", FightCfg {
            name: "天蛇族哨卫·劣化体", hp: 48, dmg: (9, 15), reward: 15, reward_why: "击倒天蛇族哨卫",
            intro: "墨绿鳞甲的哨卫拖着骨制长矛，竖瞳锁死你的咽喉——它吹响了呼唤的短尾音。",
            rage_at: Some(25), rage_text: "嘶吼呼唤增援，哨卫的鳞甲间渗出腥气——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |_| "ts_guard_win".into(), death: "ts_death",
        }),
        ("tianshe_hound", FightCfg {
            name: "蛇犬·驯化劣种", hp: 42, dmg: (8, 13), reward: 12, reward_why: "击杀蛇犬",
            intro: "被驯化的人形劣种四肢伏地，喉间发出蛇信的沙沙声。",
            rage_at: Some(20), rage_text: "撕咬加速，它化作一道残影——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("cell_rioter", FightCfg {
            name: "囚笼暴徒·劣化实验品", hp: 45, dmg: (9, 14), reward: 12, reward_why: "制服囚笼暴徒",
            intro: "眼前这具扭曲的躯体曾也是人——他咆哮着，十指带着缝合线朝你抓来。",
            rage_at: Some(20), rage_text: "狂暴乱抓，缝合线崩裂——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        // ---- L2 ----
        ("licker_x", FightCfg {
            name: "暴走实验体 X", hp: 120, dmg: (15, 23), reward: 120, reward_why: "暴走实验体 X · 清除",
            intro: "舔食者同构的实验体四肢伏地，脊柱从皮肉里拱起，淌着新鲜的粘液。",
            rage_at: Some(60), rage_text: "四肢伏地突进——它快得只剩残影！", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("blood_swallower", FightCfg {
            name: "血池吞噬体", hp: 95, dmg: (14, 21), reward: 90, reward_why: "血池吞噬体 · 清除",
            intro: "血池里浮起一团畸形的巨口，嘴角挂满半溶化的肢体。",
            rage_at: Some(45), rage_text: "它吐出半消化的残肢向你掷来——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("snake_overseer", FightCfg {
            name: "蛇监工·改造线工头", hp: 70, dmg: (12, 19), reward: 50, reward_why: "蛇监工 · 击倒",
            intro: "蛇监工甩动鞭哨，催促着身后的暴走体向你扑来。",
            rage_at: Some(35), rage_text: "鞭哨催促，暴走体动作骤然加速——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        // miniboss：零号试做型
        ("zero_proto", FightCfg {
            name: "暴走实验体·零号试做型", hp: 230, dmg: (18, 30), reward: 350, reward_why: "压制零号试做型",
            intro: "脊柱外翻撑出第二对臂，蛇尾刺鞭在地上划出火线——它曾为\"人\"，如今只剩编号 000。",
            rage_at: Some(90), rage_text: "第二对臂破体而出，蛇尾化作刺鞭——", on_rage: ts_rage_common,
            finisher_if: |st: &GameState, _ehp: i32| has_anesthetic(st),
            finisher_name: |_| "草药麻醉 · 强制压制".into(),
            finisher_desc: |_| "你掷出阿莲给的草药麻醉剂，药雾笼住它的鼻隙——那具挣动的躯体终于缓缓伏了下去，被锁链重新钉回血池边。".into(),
            win: |_| "ts_zero_win".into(), death: "ts_death",
        }),
        // ---- L3 ----
        ("snake_guard", FightCfg {
            name: "蛇卫·符文持矛", hp: 150, dmg: (17, 26), reward: 150, reward_why: "蛇卫 · 击破",
            intro: "符文甲蛇卫横持石矛，鳞甲上的金色纹路随心跳明灭。",
            rage_at: Some(75), rage_text: "符文燃亮，石矛突刺——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("chushe_tentacle", FightCfg {
            name: "初蛇触手簇", hp: 130, dmg: (16, 24), reward: 130, reward_why: "斩断初蛇触手簇",
            intro: "惨白荧绿的触手从培养槽壁疯长出来，黏腥的气味扑面。",
            rage_at: Some(65), rage_text: "触手疯长缠绕——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("rabid_guard", FightCfg {
            name: "狂化蛇卫·图腾附体", hp: 170, dmg: (18, 27), reward: 170, reward_why: "斩杀狂化蛇卫",
            intro: "图腾纹在蛇卫身上烧成血线，它嘶吼着，灵魂早已被祭柱榨干。",
            rage_at: Some(85), rage_text: "图腾纹燃血，它扑来的动作成为本能——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        // ---- L4 ----
        ("royal_guard", FightCfg {
            name: "族长亲卫·蜕皮蛇卫", hp: 190, dmg: (19, 29), reward: 180, reward_why: "击杀族长亲卫",
            intro: "金纹亲卫褪下一层蛇蜕，露出比踏入前更年轻也更凶戾的鳞甲。",
            rage_at: Some(95), rage_text: "蜕皮再生，鳞甲重铸——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("wangxue_exp", FightCfg {
            name: "狂化实验体·王血级", hp: 165, dmg: (17, 26), reward: 160, reward_why: "斩杀王血级实验体",
            intro: "王血在它血管里沸腾，堪堪撑出圣位以下的极限暴烈。",
            rage_at: Some(80), rage_text: "王血沸腾，它全身透出凶光——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        ("nest_tentacle", FightCfg {
            name: "初蛇触手簇·王巢变体", hp: 175, dmg: (18, 27), reward: 180, reward_why: "斩断王巢触手",
            intro: "暗金色的触手拖拽着破碎残骸，啃噬着战场上的尸骸回了口气。",
            rage_at: Some(85), rage_text: "吞噬残骸回血——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hall, death: "ts_death",
        }),
        // BOSS 阶段一：穆拉巴（终结=逼退，非击杀）
        ("mulaba", FightCfg {
            name: "天蛇族长·穆拉巴", hp: 360, dmg: (22, 34), reward: 500, reward_why: "逼退天蛇族长·穆拉巴",
            intro: "王座前，穆拉巴褪下蛇蜕披风，露出满身金纹。骨杖上的\"初蛇之眼\"幽幽盯着你。",
            rage_at: Some(150), rage_text: "他祭出圣人符箓残响——符箓金光虚影升腾，你的理智被灼烧（San-15）！", on_rage: |st, log| {
                st.san = (st.san - 15).clamp(0, 100);
                log.push("<span class='crit'>圣人符箓残响压下，San -15。</span>".into());
            },
            // 逼退：HP<150 触发「弃战献祭」→ 转场阶段二
            finisher_if: |_, ehp| ehp < 150,
            finisher_name: |_| "弃战献祭 · 逼退".into(),
            finisher_desc: |_| "你将他逼至墙角的刹那，穆拉巴撕碎蜕下的蛇蜕，朝核心熔炉狂奔而去——他要把自己与全部\"零号计划\"一起，献祭进熔炉！".into(),
            win: |_| "ts_boss1_retreat".into(), death: "ts_death",
        }),
        // BOSS 阶段二：初蛇聚合体（选择驱动遭遇，终结条件见 route_snake_* ）
        ("apocalypse_snake", FightCfg {
            name: "初蛇基因聚合体·灭世之蛇幼体", hp: 540, dmg: (26, 40), reward: 800, reward_why: "终结灭世之蛇幼体",
            intro: "熔炉轰然炸开，万族的血与人族的魂凝成一头多首盘聚的\"伪初蛇\"——它睁开的每一只眼睛里，都是一张被记录进零号计划的名册。",
            rage_at: Some(200), rage_text: "完全蜕皮！多首触手全展，血肉上浮现万族面孔的浮雕——", on_rage: ts_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |_| "ts_boss2_win".into(), death: "ts_death",
        }),
    ]
}

/// 查询辅助（主线合并查询扩展时可直接调用）
pub fn ts_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    tianshe_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/// 场景查询辅助
pub fn ts_scene(id: &str) -> Option<&'static SceneDef> {
    TIANSHE_SCENES.iter().find(|s| s.id == id)
}

/* =====================================================================
   剧情场景（id 全部 ts_ 前缀）
   ===================================================================== */
pub static TIANSHE_SCENES: &[SceneDef] = &[

/* ---- 幕 1 · 开场「血池倒影」 ---- */
SceneDef {
    id: "ts_open", bg: Some("tianshe_bg.png"), loc: Some("L1 · 牢房"), mood: "danger",
    speaker: Some("内心"), voice: None,
    text: TextSpec::Static(&[
        "你在腥甜的气味里醒来。铁栏外是暗绿的荧光，远处有滴答的水声——那不是水，是某种粘稠的液体。",
        "你的记忆只剩半截：摇摇晃晃的牢车、被蛇纹罩袍的人推进甬道、还有……血池里自己的倒影。",
        "墙上有人用血画了行字：<b>「别信祭坛，信火种。」</b>（镜像线 · 刻字一）",
    ]),
    choices: &[
        ChoiceDef { label: "【先观察环境，保持安静】", sub: "San+5 · 发现墙上刻字 mirror_1", cond: None,
            effects: &[Eff::San(5), Eff::SetFlag("mirror_1"), Eff::SetFlag("wake_calm")], route: Route::To("ts_act2_cell") },
        ChoiceDef { label: "【大声呼救】", sub: "引蛇哨卫巡逻 · 强制战斗", cond: None,
            effects: &[Eff::SetFlag("wake_loud")], route: Route::To("ts_guard_fight") },
        ChoiceDef { label: "【装死再听一会儿】", sub: "解锁通风口低风险进入", cond: None,
            effects: &[Eff::SetFlag("wake_quiet")], route: Route::To("ts_act2_cell") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 2 「狱友」 ---- */
SceneDef {
    id: "ts_act2_cell", bg: Some("tianshe_bg.png"), loc: Some("L1 · 牢房区"), mood: "cold",
    speaker: Some("阿莲"), voice: None,
    text: TextSpec::Static(&[
        "隔着铁栏，你看见一张瘦削的脸——是个妇人，头发散乱，眼神却亮。「也是……材料？」她自嘲地笑，「我叫阿莲，江东谷地的药婆。」",
        "更里面那间重刑牢房，一个铁塔般的汉子沉默地靠墙坐着——老石，双臂被符文铁链锁着。",
    ]),
    choices: &[
        ChoiceDef { label: "【和阿莲交谈，问基地的事】", sub: "trust_alien_1 · 情报 intel_1", cond: None,
            effects: &[Eff::SetFlag("trust_alien_1"), Eff::SetFlag("intel_1")], route: Route::To("ts_act3_convoy") },
        ChoiceDef { label: "【和老石交谈（隔栏）】", sub: "trust_stone_1 · 血池下有暗河", cond: None,
            effects: &[Eff::SetFlag("trust_stone_1")], route: Route::To("ts_act3_convoy") },
        ChoiceDef { label: "【两个都聊】（需 San≥50）", sub: "双 trust flag", cond: Some(cond_psych),
            effects: &[Eff::SetFlag("trust_alien_1"), Eff::SetFlag("trust_stone_1"), Eff::SetFlag("intel_1")], route: Route::To("ts_act3_convoy") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 3 「越狱夜」 ---- */
SceneDef {
    id: "ts_act3_convoy", bg: Some("tianshe_bg.png"), loc: Some("L1 · 体检室→甬道"), mood: "danger",
    speaker: Some("蛇监工"), voice: None,
    text: TextSpec::Static(&[
        "你被拖上体检台，蛇监工翻着名册：「编号 087，劣品，右臂无法再生……」",
        "话音未落，整座基地猛地一震——<b>A 翼方向传来闷雷般的爆炸与嘶吼</b>，警报尖啸，蛇人们纷纷朝北冲去。",
        "混乱中，铁栏被人从外面撬开了一角——墙上有人用血画了箭头，指向监工室。",
    ]),
    choices: &[
        ChoiceDef { label: "【趁乱夺令】", sub: "正面对决监工室蛇哨卫 → 监工令牌 G1", cond: None,
            effects: &NO_EFF, route: Route::To("ts_guard_fight") },
        ChoiceDef { label: "【从通风口钻出去】", sub: "需 wake_quiet · San-10 · 走 P2 捷径", cond: Some(cond_wake_quiet),
            effects: &[Eff::San(-10), Eff::SetFlag("vent_route")], route: Route::To("ts_act4_pool") },
        ChoiceDef { label: "【顺从地被推上体检台】", sub: "发现残页·名册 · Hurt(8)", cond: None,
            effects: &[Eff::Hurt(8, "ts_death"), Eff::SetFlag("exam_route"), Eff::SetFlag("ts_archive_1"), Eff::AddItem("item_record_1"), Eff::Points(30)],
            route: Route::Dyn(route_archive_count) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_guard_fight", bg: Some("tianshe_bg.png"), loc: Some("L1 · 监工室"), mood: "danger",
    speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["监工室的蛇哨卫拔出骨矛，挡在你与令牌箱之间——"]),
    choices: &NO_CH, fight_id: Some("tianshe_guard"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_guard_win", bg: Some("tianshe_bg.png"), loc: Some("L1 · 监工室"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你放倒蛇哨卫，撬开令牌箱取走<b>监工令牌</b>。甬道尽头的刻字在血光里亮起：<b>「出口不在电梯，在血池底下。别信蛇蜕。」</b>（镜像线 · 刻字二）",
    ]),
    choices: &[
        ChoiceDef { label: "（循令牌下电梯）", sub: "token_route · mirror_2 · 取得监工令牌", cond: None,
            effects: &[Eff::SetFlag("token_route"), Eff::SetFlag("mirror_2"), Eff::AddItem("item_guard_token")], route: Route::To("ts_act4_pool") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 4 「血池之下」（miniboss） ---- */
SceneDef {
    id: "ts_act4_pool", bg: Some("tianshe_bg.png"), loc: Some("L2 · 血池车间"), mood: "danger",
    speaker: Some("阿莲"), voice: None,
    text: TextSpec::Static(&[
        "血池在脚下翻涌，池壁挂满白骨与拖痕。吊钩上，一具具\"材料\"被送上改造线。",
        "流水线尽头，一个被锁链拴住的巨大身影正在挣动：<b>暴走实验体·零号试做型</b>。它曾经也是人。",
        "血池里的每一具\"失败品\"，都曾经是一个有名字的人。",
    ]),
    choices: &[
        ChoiceDef { label: "【正面对决零号试做型】", sub: "强制遭遇 miniboss zero_proto", cond: None,
            effects: &NO_EFF, route: Route::To("ts_zero_fight") },
        ChoiceDef { label: "【调查血池阀门，放池冲乱杂兵】", sub: "blood_valve · Points+20", cond: None,
            effects: &[Eff::SetFlag("blood_valve"), Eff::Points(20)], route: Route::To("ts_zero_fight") },
        ChoiceDef { label: "【先救流水线上的阿莲】（需信任）", sub: "alien_saved · 得草药麻醉剂", cond: Some(cond_trust_alien),
            effects: &[Eff::SetFlag("alien_saved"), Eff::AddItem("item_anesthetic")], route: Route::To("ts_zero_fight") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_zero_fight", bg: Some("tianshe_bg.png"), loc: Some("L2 · 血池车间 · miniboss"), mood: "danger",
    speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["锁链崩断，零号试做型挣动着朝你扑来——你只有两个选择：药，或刀。"]),
    choices: &NO_CH, fight_id: Some("zero_proto"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_zero_win", bg: Some("tianshe_bg.png"), loc: Some("L2 · 血池车间"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["零号试做型终于不动了。血池归于平静。你从池边拾起一块犹带体温的徽记——那是它的编号，也是它曾有过的名字的余响。"]),
    choices: &[
        ChoiceDef { label: "（继续深入基因区）", sub: "zero_prototype_killed", cond: None,
            effects: &[Eff::SetFlag("zero_prototype_killed")], route: Route::To("ts_act5_temple") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 5 「初蛇基因库」 ---- */
SceneDef {
    id: "ts_act5_temple", bg: Some("tianshe_bg.png"), loc: Some("L3 · 祭祀场→母体槽室"), mood: "angry",
    speaker: Some("老石"), voice: Some("vo_tianshe_laoshi"),
    text: TextSpec::Static(&[
        "图腾柱不是木头——是<b>万族的头骨与脊椎</b>，一层层叠成蛇形。北壁符箓金光下，一行小字：「代价：三百个魂，一座城？」",
        "母体槽旁，老石被绑在祭柱上，咧嘴一笑：「我这一身力气，原是打铁的。今日，给你们打条活路。」",
    ]),
    choices: &[
        ChoiceDef { label: "【割断祭柱，救下老石】", sub: "stone_saved", cond: None,
            effects: &[Eff::SetFlag("stone_saved")], route: Route::To("ts_act6_core") },
        ChoiceDef { label: "【调查祭坛取血契+样本】", sub: "初蛇血契 G4 · 初蛇基因样本", cond: None,
            effects: &[Eff::SetFlag("chushe_sample"), Eff::AddItem("item_chushe_blood"), Eff::AddItem("item_chushe_sample")], route: Route::To("ts_act6_core") },
        ChoiceDef { label: "【让老石断后，自己保存战力】", sub: "stone_sacrifice · Points+50", cond: None,
            effects: &[Eff::SetFlag("stone_sacrifice"), Eff::Points(50)], route: Route::To("ts_act6_core") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 6 「零号核心」（最终二段战） ---- */
SceneDef {
    id: "ts_act6_core", bg: Some("tianshe_bg.png"), loc: Some("L4 · 大厅→核心熔炉"), mood: "danger",
    speaker: Some("穆拉巴"), voice: Some("vo_tianshe_mulaba_name"),
    text: TextSpec::Static(&[
        "王座上，穆拉巴褪下蛇蜕披风，露出满身金纹。「你们这些没有能量循环的虫子……也配谈名字？」",
        "他指向熔炉：「看看零号计划——万族的血、人族的魂、初蛇的基因，铸成吾族的'新初蛇'。天蛇族，当出圣人！」",
        "你忽然明白：<b>零号基地里真正被实验的，从来不止是人族。</b>",
    ]),
    choices: &[
        ChoiceDef { label: "【揭穿他：符箓的代价是三百个魂】", sub: "zero_plan_known · +30 · San+10", cond: None,
            effects: &[Eff::SetFlag("zero_plan_known"), Eff::Points(30), Eff::SetFlag("morale_break")], route: Route::To("ts_boss1_fight") },
        ChoiceDef { label: "【直接开战】", sub: "zero_plan_known · 无士气加成", cond: None,
            effects: &[Eff::SetFlag("zero_plan_known")], route: Route::To("ts_boss1_fight") },
        ChoiceDef { label: "【（隐藏）亮出希望模板残光】", sub: "穆拉巴惊惧 · 压制 2 回合", cond: Some(cond_hope_light),
            effects: &[Eff::SetFlag("zero_plan_known"), Eff::SetFlag("hope_reveal")], route: Route::To("ts_boss1_fight") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_boss1_fight", bg: Some("tianshe_bg.png"), loc: Some("L4 · 族长寝巢 · 阶段一"), mood: "danger",
    speaker: Some("⚔ BOSS"), voice: None,
    text: TextSpec::Static(&["穆拉巴怒吼着扑来，骨杖上的\"初蛇之眼\"亮起豺狼般的凶光。"]),
    choices: &NO_CH, fight_id: Some("mulaba"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_boss1_retreat", bg: Some("tianshe_bg.png"), loc: Some("L4 · 族长寝巢 → 熔炉"), mood: "danger",
    speaker: Some("穆拉巴"), voice: Some("vo_tianshe_mulaba_snake"),
    text: TextSpec::Static(&[
        "穆拉巴「弃战献祭」——他撕碎蛇蜕，朝核心熔炉狂奔。熔炉轰然吞噬了他的身影与全部\"零号计划\"的成果。",
        "<b>「既然要死，就一起做我的……初蛇！」</b>他的话语被熔炉轰鸣吞没。",
    ]),
    choices: &[
        ChoiceDef { label: "（举步走进熔炉核心）", sub: "进入阶段二", cond: None, effects: &NO_EFF, route: Route::Dyn(start_snake) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_boss2_start", bg: Some("tianshe_bg.png"), loc: Some("L4 · 核心熔炉 · 阶段二"), mood: "danger",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&["熔炉深处，万族的血与人族的魂翻涌凝形——初蛇聚合体苏醒了。" ]),
    choices: &[ChoiceDef { label: "（逼近聚合体）", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_snake) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_boss2_round", bg: Some("tianshe_bg.png"), loc: Some("L4 · 核心熔炉 · 决战"), mood: "danger",
    speaker: Some("⚔ 决战"), voice: None,
    text: TextSpec::Dyn(txt_boss2_round),
    choices: &[
        ChoiceDef { label: "【重击（强攻）】", sub: "伤害 28-40 · 易露破绽", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st: &mut GameState| route_snake_attack(st, rnd_ts(28, 40))) },
        ChoiceDef { label: "【连击（迅疾）】", sub: "伤害 20-28 · 稳", cond: None,
            effects: &NO_EFF, route: Route::Dyn(|st: &mut GameState| route_snake_attack(st, rnd_ts(20, 28))) },
        ChoiceDef { label: "【样本共鸣（持初蛇基因样本）】", sub: "幼体 HP<100 时终结", cond: Some(cond_sample_resonance),
            effects: &NO_EFF, route: Route::Dyn(route_snake_resonance) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_boss2_win", bg: Some("tianshe_bg.png"), loc: Some("L4 · 核心熔炉"), mood: "calm",
    speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "样本共鸣——你体内的初蛇基因与这头\"伪初蛇\"同频共振。/幼体在共鸣里坍缩成一枚温热的<b>初蛇基因样本</b>与<b>零号核心结晶</b>。",
        "熔炉过载，金色的裂痕爬满整座基地。",
    ]),
    choices: &[
        ChoiceDef { label: "【拾取战利品，冲出熔炉】", sub: "boss2_defeated · +800 · 结晶+样本", cond: None,
            effects: &[Eff::SetFlag("boss2_defeated"), Eff::Points(800), Eff::AddItem("item_core_crystal"), Eff::AddItem("item_chushe_sample")],
            route: Route::To("ts_finale") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_boss2_wipe", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("灭世蜕皮"), death: Some(("灭世蜕皮·全灭", "未携初蛇基因样本，被灭世之蛇幼体抹去")), card: |_st| crate::state::Card {
            title: "灭 世 蜕 皮".into(), good: false,
            body_html: r#"<p>倒计时数到三。幼体完全蜕皮，无数触手与万族面孔将你与整支小队吞没——你成了零号计划名册上又一个过期编号。</p>
<p style='color:#ff8a8a'>【死亡档案 · 灭团】灭团扣 300 点，支线 flag 清零重打。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "ts_boss2_lose", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("熔炉的记录", "被核心熔炉吞噬")), card: |_st| crate::state::Card {
            title: "熔 炉 的 记 录".into(), good: false,
            body_html: r#"<p>熔炉将你拖入深处——你的名字被记进\"零号计划\"的名册，编号之后画上一个勾。</p>
<p style='color:#ff8a8a'>【死亡档案】个体死亡，回主神空间扣 100 点复活。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- 幕 7 结局「崩塌与黎明」 ---- */
SceneDef {
    id: "ts_finale", bg: Some("tianshe_bg.png"), loc: Some("L4 · 逃生竖井"), mood: "cold",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "你带着活下来的人冲进逃生竖井。背后，零号基地在一连串轰鸣中塌入地底。",
        "裂缝尽头，晨光刺眼。远方的荒原上，一个高大身影正扛着另一个人，一步一步走向地平线——那是 A 翼的越狱者。",
        "竖井壁上，最后一行刻字：<b>「记住名字。别让他们白死。」</b>",
    ]),
    choices: &[
        ChoiceDef { label: "【走主通道撤离】", sub: "escape_route_1 · 遇蛇卫潮", cond: None,
            effects: &[Eff::SetFlag("escape_route_1"), Eff::PointsIfFlag("ts_mirror_line_3", 200), Eff::PointsIfFlag("ts_archive_all", 200)],
            route: Route::Dyn(route_finalize) },
        ChoiceDef { label: "【赌一把，走塌方侧缝】", sub: "escape_route_2 · Hurt(15) San-5 · 无战斗", cond: None,
            effects: &[Eff::Hurt(15, "ts_death"), Eff::San(-5), Eff::SetFlag("escape_route_2"), Eff::PointsIfFlag("ts_mirror_line_3", 200), Eff::PointsIfFlag("ts_archive_all", 200)],
            route: Route::Dyn(route_finalize) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_finish", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("vo_tianshe_liming"), death: None,
        card: |st| crate::state::Card {
            title: "任 务 完 成 · 零 号 基 地".into(), good: true,
            body_html: format!(
                "<p>洪荒历 y+j 年，天蛇族零号基地从记录中消失。那夜，两批'材料'学会了奔跑。</p>\
                 <table class='statTable'>\
                 <tr><td>奖励点数</td><td>{}</td></tr>\
                 <tr><td>残页收集档案</td><td>{}/8{}（成功 {}</td></tr>\
                 <tr><td>支线评级</td><td style='color:#ffd76a'>D 级（题干·通用）</td></tr>\
                 <tr><td>镜像越狱</td><td>{}</td></tr>\
                 </table>\
                 <p style='color:#8fd0a8'>主神：零号核心结晶可兑换后天灵宝碎片（1/3）；初蛇基因样本是后续低纬度副本的门券。——解散。</p>",
                st.points,
                ts_archive_count(st), 8,
                if st.flag("ts_archive_all") { "集齐＋200" } else { "未集齐" },
                if st.flag("ts_mirror_line_3") { "完整 ＋200" } else { "缺失" },
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: Some("vo_tianshe_liming"),
        },
    }),
},

/* ---- 通用死亡档案 ---- */
SceneDef {
    id: "ts_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: Some("李铭（死亡档案）"), death: Some(("零号基地的死亡记录", "战死于天蛇族地下实验室")), card: |_st| crate::state::Card {
            title: "死 亡 档 案".into(), good: false,
            body_html: r#"<p>你倒在了零号基地阴暗的甬道里。</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},

/* ---- 中立枢纽 scene_ts_hall（收集/调查/NPC 返回点） ---- */
SceneDef {
    id: "ts_hall", bg: Some("tianshe_bg.png"), loc: Some("零号基地 · 探索间"), mood: "cold",
    speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "零号基地的暗绿灯光在头顶明灭。你可以继续在当前的楼层里调查，或循主线向前推进。",
    ]),
    choices: &[
        ChoiceDef { label: "（继续探索本层）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ts_hall") },
        ChoiceDef { label: "（回到囚笼区探索）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ts_act2_cell") },
        ChoiceDef { label: "（前往基因实验区主线）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ts_act4_pool") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 失败品记录残页 · 收集点（方案 A 连号 flag ts_archive_1..8） ---- */
SceneDef {
    id: "ts_roster", bg: Some("tianshe_bg.png"), loc: Some("L1 · 体检台 · 押运名册"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·名册：押运名单上，17 个名字被勾掉。批注——<b>「人族，无能量循环，可作载体」</b>。",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·名册】", sub: "ts_archive_1 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_1"), Eff::AddItem("item_record_1"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_flow", bg: Some("tianshe_bg.png"), loc: Some("L2 · 改造流水线 · 记录"), mood: "angry",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·流水线：改造记录——<b>「希望他/她能撑过第七次修正」</b>，署名被蛇纹盖住。",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·流水线】", sub: "ts_archive_2 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_2"), Eff::AddItem("item_record_2"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_prayer", bg: Some("tianshe_bg.png"), loc: Some("L2 · 血池回廊 · 墙刻祈祷"), mood: "sad",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·祈祷：囚犯刻在墙上的祈祷残句——<b>「若有人逃出去，请告诉江东……」</b>",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·祈祷】", sub: "ts_archive_3 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_3"), Eff::AddItem("item_record_3"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_mother", bg: Some("tianshe_bg.png"), loc: Some("L2 · 样本库 · 培养槽日志"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·母体：培养槽日志——<b>「第 41 次尝试，母体拒绝人族基因。它记得什么？」</b>",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·母体】", sub: "ts_archive_4 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_4"), Eff::AddItem("item_record_4"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_seal", bg: Some("tianshe_bg.png"), loc: Some("L3 · 祭祀场 · 符箓抄本"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·符箓：圣人符箓抄本残页，边角——<b>「代价：三百个魂，一座城？」</b>",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·符箓】", sub: "ts_archive_5 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_5"), Eff::AddItem("item_record_5"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_escape", bg: Some("tianshe_bg.png"), loc: Some("L3 · 母体槽室 · 钧的刻字"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·越狱（镜像）：钧的笔迹——<b>「出口不在电梯，在血池底下。别信蛇蜕。」</b>",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·越狱】", sub: "ts_archive_6 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_6"), Eff::AddItem("item_record_6"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_name", bg: Some("tianshe_bg.png"), loc: Some("L4 · 核心区 · 实验室名单"), mood: "sad",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·名字：实验室名单里\"盘部落-覃\"的条目，已做已故标记——那是古线中一个未能走更远的人。",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·名字】", sub: "ts_archive_7 · Points+30", cond: None,
        effects: &[Eff::SetFlag("ts_archive_7"), Eff::AddItem("item_record_7"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_heart", bg: Some("tianshe_bg.png"), loc: Some("L4 · 逃生竖井旁 · 无名氏记录"), mood: "sad",
    speaker: None, voice: None,
    text: TextSpec::Static(&[
        "残页·心跳：无名氏的最后记录——<b>「今天又有人被拖走了。我数着数着，忘了自己的名字。」</b>",
    ]),
    choices: &[ChoiceDef { label: "【收下残页·心跳】", sub: "ts_archive_8 · Points+30（集齐 archive_all）", cond: None,
        effects: &[Eff::SetFlag("ts_archive_8"), Eff::AddItem("item_record_8"), Eff::Points(30)], route: Route::Dyn(route_archive_count) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 调查点（门禁道具 / 机关 / 情报） ---- */
SceneDef {
    id: "ts_tokenbox", bg: Some("tianshe_bg.png"), loc: Some("L1 · 监工室 · 令牌箱"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["令牌箱锁着，但箱身上的感应纹路正是 G1 闸门的型号。你需要先放倒守卫的蛇哨卫才能撬开。"]),
    choices: &[ChoiceDef { label: "（返回）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_foreman", bg: Some("tianshe_bg.png"), loc: Some("L1 · 监工室 · 书桌"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["监工室的桌面上摊着一份排班表——监工室令牌在保险箱里，体检室在右，往北是通往基因实验区的电梯。"]),
    choices: &[ChoiceDef { label: "（记下情报）", sub: "intel_1 · Points+5", cond: None,
        effects: &[Eff::SetFlag("intel_1"), Eff::Points(5)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_wall", bg: Some("tianshe_bg.png"), loc: Some("L1 · 牢房 · 墙上刻字"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["染血的刻字：「别信祭坛，信火种。」——那是一双与你一样逃亡的手留下的。（镜像 · 刻字一）"]),
    choices: &[ChoiceDef { label: "（记下刻字）", sub: "mirror_1", cond: None,
        effects: &[Eff::SetFlag("mirror_1")], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_pool", bg: Some("tianshe_bg.png"), loc: Some("L2 · 血池 · 翻涌池面"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["血池深处仿佛有无数双眼睛与你对视。白骨在池壁下的暗河方向延伸——那也许就是老石说的暗河。"]),
    choices: &[ChoiceDef { label: "（退开）", sub: "San-5", cond: None, effects: &[Eff::San(-5)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_console", bg: Some("tianshe_bg.png"), loc: Some("L2 · 监工站 · 控制台"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["控制台闪着血红的光。你抽出那张<b>基因样本卡</b>——它能开启样本库的大门（G2）。"]),
    choices: &[ChoiceDef { label: "【取走基因样本卡】", sub: "G2 解锁 · +10", cond: None,
        effects: &[Eff::AddItem("item_gene_card"), Eff::Points(10)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_safe", bg: Some("tianshe_bg.png"), loc: Some("L2 · 样本库 · 保险柜"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["保险柜里躺着一枚<b>封界通行符</b>，旁边一张便签：某试验品侧缝，能取到初蛇基因样本。"]),
    choices: &[ChoiceDef { label: "【取走封界通行符】", sub: "G3 解锁 · +15", cond: None,
        effects: &[Eff::AddItem("item_seal_pass"), Eff::Points(15)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_rune", bg: Some("tianshe_bg.png"), loc: Some("L2 · 符文切割场 · 符文装置"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["你参透潮汐符文的规律——血池涨落的间隙，正是横穿符文切割场的安全时机。（rune_secret）"]),
    choices: &[ChoiceDef { label: "【参透规律】", sub: "rune_secret · +30", cond: None,
        effects: &[Eff::SetFlag("rune_secret"), Eff::Points(30)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_valve", bg: Some("tianshe_bg.png"), loc: Some("L2 · 血池 · 排水阀门"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["你按下阀门，血池翻涌着向暗河倾泻，冲散了周围的杂兵。可零号试做型不惧血池。"]),
    choices: &[ChoiceDef { label: "【放池】", sub: "blood_valve · +20", cond: None,
        effects: &[Eff::SetFlag("blood_valve"), Eff::Points(20)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_altar", bg: Some("tianshe_bg.png"), loc: Some("L3 · 祭坛 · 初蛇血契"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["祭坛血槽里凝着一滴<b>初蛇血契</b>。取走它，就能开启祭祀场南门直下族长巢穴（L4）。"]),
    choices: &[ChoiceDef { label: "【取走初蛇血契】", sub: "G4 · +15", cond: None,
        effects: &[Eff::SetFlag("chushe_blood"), Eff::AddItem("item_chushe_blood"), Eff::Points(15)], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_mothercurve", bg: Some("tianshe_bg.png"), loc: Some("L3 · 初蛇母体 · 巨槽"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["你在母体巨槽的侧缝取出一管温热的<b>初蛇基因样本</b>——它是终结第二段 BOSS 的关键，也是跨副本的兑换券。"]),
    choices: &[ChoiceDef { label: "【取走初蛇基因样本】", sub: "样本共鸣关键", cond: None,
        effects: &[Eff::SetFlag("chushe_sample"), Eff::AddItem("item_chushe_sample")], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_temple", bg: Some("tianshe_bg.png"), loc: Some("L3 · 图腾柱 · 暗格"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["你撬开图腾柱底部的暗格，里面刻着一条渗血的侧翼路线——被标记：<b>直入核心区侧翼</b>（P6 绕行捷径）。"]),
    choices: &[ChoiceDef { label: "【记下暗格秘密】", sub: "temple_secret · P6 捷径", cond: None,
        effects: &[Eff::SetFlag("temple_secret")], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_hope", bg: Some("tianshe_bg.png"), loc: Some("L3 · 母体槽壁 · 血液刻印"), mood: "cold",
    speaker: None, voice: None,
    text: TextSpec::Static(&["母体槽壁的血液刻印发烫亮起——那是钧的血液印记（镜像 · 刻字三）。刻印与你的血共鸣，化作一团微光：<b>希望模板残光</b>。"]),
    choices: &[ChoiceDef { label: "【引燃希望模板残光】", sub: "hope_light · mirror_3", cond: None,
        effects: &[Eff::AddItem("item_hope_light")], route: Route::Dyn(route_hope) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_furnace", bg: Some("tianshe_bg.png"), loc: Some("L4 · 基因熔炉"), mood: "danger",
    speaker: None, voice: None,
    text: TextSpec::Static(&["熔炉在疯狂运转。你知道，走到这一步，穆拉巴与零号计划的真相，都在那融化的初蛇之血里。"]),
    choices: &[ChoiceDef { label: "（退回大厅）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ts_act6_core") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- NPC 对话 ---- */
SceneDef {
    id: "ts_npc_alien", bg: Some("tianshe_bg.png"), loc: Some("L1 · 牢房区 · 阿莲"), mood: "cold",
    speaker: Some("阿莲"), voice: Some("vo_tianshe_alian"),
    text: TextSpec::Static(&[
        "「药材认得全，符文未必。若你信得过我，这次若能逃出去，我就把'再生'之法教给同样被抓的人。」",
        "（若已逃出）「他们叫我们'材料'。可材料，也会记得回家的路。」",
    ]),
    choices: &[ChoiceDef { label: "（继续交谈）", sub: "trust_alien_1", cond: None,
        effects: &[Eff::SetFlag("trust_alien_1"), Eff::SetFlag("intel_1")], route: Route::To("ts_act2_cell") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_npc_stone", bg: Some("tianshe_bg.png"), loc: Some("L1 · 牢房区 · 老石"), mood: "cold",
    speaker: Some("老石"), voice: None,
    text: TextSpec::Static(&["「我这身力气，原是打铁的。铁匠的直觉告诉我——往北的电梯不是出口。血池底下，有暗河。」（镜像伏笔）"]),
    choices: &[ChoiceDef { label: "（继续交谈）", sub: "trust_stone_1", cond: None,
        effects: &[Eff::SetFlag("trust_stone_1")], route: Route::To("ts_act2_cell") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ts_npc_jun", bg: Some("tianshe_bg.png"), loc: Some("L3 · 下层回廊 · 钧留音"), mood: "calm",
    speaker: Some("钧（镜像留音）"), voice: Some("vo_tianshe_jun"),
    text: TextSpec::Static(&["「别信祭坛，信火种。出口，在血池底下。」——那声音平静得不像一个正在越狱的人，像早已想通了生死。"]),
    choices: &[ChoiceDef { label: "（让刻印沉入心底）", sub: "mirror_3", cond: None,
        effects: &[Eff::SetFlag("mirror_3")], route: Route::To("ts_hall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
];

/// ts_hope：取得希望模板残光，完成镜像线第三段；若 mirror_1/2/3 集齐 → ts_mirror_line_3
fn route_hope(st: &mut GameState) -> String {
    st.set_flag("hope_light");
    st.set_flag("mirror_3");
    if st.flag("mirror_1") && st.flag("mirror_2") && st.flag("mirror_3") {
        st.set_flag("ts_mirror_line_3");
    }
    "ts_act5_temple".to_string()
}
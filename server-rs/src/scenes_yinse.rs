//! 《银色大地 · 地灵族机界遗迹》任务世界 · 全部剧情场景与战斗配置。
//! 设计依据 design/zhttty_universe/honghuang_li/yinse_dadi.md §4/§5/§6/§7/§8。
//! 本文件是全新新增文件，只导出静态数据（YINSE_SCENES / yinse_figths / 查询辅助），
//! 不写入 scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()
//! 同时检索本文件表（见 tools/design/yinse_impl_log.md ★外部依赖）。
//!
//! 场景 id 全部 `ys_` 前缀，与既有 SCENES 无重名。
//! 战斗 id 全部 `ws_` 前缀（敌 BOSS 两段式 wo_waro_r1/r2）。
//!
//! 多段 BOSS（机界升华体·瓦罗残响，§5）：一段战胜利 → 转场演出（东天二皇投影）→ 放二段战斗场景，用"场景链"衔接，
//! 无 next_fight 引擎字段（设计 §10.2 取舍，参考 scenes_zhouyuan.rs 的 BOSS 场景链写法）。
//! 圣位演出红线：东天二皇投影 / 龙族高圣尸骸只做演出级场景文本，不进 fight 数值（§1 铁律，设计 §10.4）。
//! 机关链（L2 三配电点 master→B→C 顺序，§10.1）：错序即触发电偶战斗（自然惩罚，零引擎改动）。
//! sp_grade 用 Route::Dyn 写 `st.sp_grade = Some('D')`。

use crate::defs::*;
use crate::state::GameState;

/// 空 effect / choice 惯用静态（同 scenes.rs / scenes_zhouyuan.rs）
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   条件谓词（cond，全部具名：CondFn 为 fn 指针不能捕获闭包）
   ===================================================================== */
fn cond_has_worm_corpse(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "item_worm_corpse") }
fn cond_has_diling(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "item_diling") }
fn cond_power_restored(st: &GameState) -> bool { st.flag("ys_l2_power_restored") }
fn cond_has_jiche(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "item_jiche") }
fn cond_has_peace(st: &GameState) -> bool { st.flag("ys_waro_peace") }
fn cond_rift_open(st: &GameState) -> bool { st.flag("ys_rift_open") }
fn cond_core_open(st: &GameState) -> bool { st.flag("ys_core_open") }

/// G3 升华装置启动间门禁（scene 侧双条件：truth 或机核）
fn cond_sublime_unlocked(st: &GameState) -> bool { cond_has_jiche(st) }

/// 撤离信标提前结算仅在有主神信标时可用
fn cond_has_beacon(st: &GameState) -> bool { st.inventory.iter().any(|i| i == "item_beacon") }

/* =====================================================================
   动态文本辅助
   ===================================================================== */
fn txt_home_bones(_st: &GameState) -> String {
    "长街两侧的居民骸骨以同一种姿势定格——向上伸着手。两百年前，他们把身体交给了升华。\n\n<b>「升华，原来是把人变成材料。」</b>（San -8）".to_string()
}

/// 阶段进度文本（BOSS 两段式总血量 680）
fn txt_boss_r1(_st: &GameState) -> String {
    "机界升华体·瓦罗残响（一形态）——半圣躯壳外覆机械装甲，背后机械翼垂落，面部被圣光蚀刻成一片空白。他抬起头，数十条管线同时绷直。\n\n<b>「人类……又是人类的火种。」</b>".to_string()
}
fn txt_boss_r2(_st: &GameState) -> String {
    "法阵坍缩的瞬间，瓦罗残响的机壳从内部炸开。墨紫裂隙物质裹缠而上，触须、眼柄漫布——只剩一只人类的眼睛在分崩离析的躯壳中央，还亮着。\n\n<b>「……把碎片，还给我。」</b>".to_string()
}

/* =====================================================================
   战斗配置表（YS 专属；导出供主线把查询扩展进来）
   ===================================================================== */
fn ys_rage_common(_st: &mut GameState, log: &mut Vec<String>) {
    log.push("<span class='crit'>机械轰鸣暴涨——它的出力再度抬升。</span>".into());
}

/* ---- 胜利/死亡路由（两段式 BOSS 场景链衔接） ---- */
/// 一段 BOSS 胜利路由：进入转场演出场景（放二段）
fn ys_win_r1(st: &GameState) -> String {
    let _ = st;
    "ys_waR0_cast".into()
}
/// 二段 BOSS 胜利路由：按结局分支（复仇/和平归还/私藏碎片）
fn ys_win_r2(st: &GameState) -> String {
    if st.flag("ys_waro_peace") {
        "ys_15_ending_peace".into()
    } else if st.inventory.iter().any(|i| i == "item_jiche") {
        // 私藏碎片 → 火种结局
        "ys_15_ending_fire".into()
    } else {
        "ys_15_ending_venge".into()
    }
}
/// L1 普通战斗胜利（回到战壕汇合点）
fn ys_win_common(_st: &GameState) -> String { "ys_04_trench".into() }

/* ---- 战斗配置表（id 全部 ws_ 前缀，引用 enemy_* 复用 / boss_waro_*.png） ---- */
pub fn yinse_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("ws_scav", FightCfg {
            name: "古兽人拾荒者", hp: 52, dmg: (10, 18), reward: 18, reward_why: "古兽人拾荒者 · 清理",
            intro: "一名灰绿粗糙身形的古兽人拾荒者从废墟后转出，缠满地灵族管线的兽骨棒扛在肩头，獠牙毕露。",
            rage_at: Some(25), rage_text: "<b>狂暴@25</b>：兽性爆发，伤害+3。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_brute", FightCfg {
            name: "战潮碎骨者", hp: 68, dmg: (13, 20), reward: 30, reward_why: "战潮碎骨者 · 击破",
            intro: "精壮双獠牙兽人拖着右臂的机械臂拱步逼近，符文锈蚀的关节发出刺耳的吱呀。",
            rage_at: Some(30), rage_text: "<b>狂暴@30</b>:双持撞击，追加 1 击。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_worm", FightCfg {
            name: "银色机械蠕虫", hp: 42, dmg: (8, 15), reward: 15, reward_why: "银色机械蠕虫 · 擒杀",
            intro: "白银鳞甲的长虫从尸堆间弹起，腹部透出蓝白符文光带，三瓣机械爪的口器一张一合。",
            rage_at: Some(20), rage_text: "<b>狂暴@20</b>：自爆溅射（San -2）。", on_rage: |st, log| {
                st.san = (st.san - 2).clamp(0, 100);
                log.push("<span class='crit'>银蚴自爆，碎片溅射——你的理智被刮去一线（San -2）。</span>".into());
            },
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_lou", FightCfg {
            name: "古兽人战潮王 · 髅", hp: 220, dmg: (16, 26), reward: 150, reward_why: "战潮王・髅 · 枭首",
            intro: "近三米巨型兽人主将背插残破战旗，白骨面具下，一排獠牙朝你咧开。他把地灵族的机械臂当串肉签，插在尸堆上。",
            rage_at: Some(100), rage_text: "<b>狂暴@100</b>：战旗一挥，召唤拾荒者×1！", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: |st| { let _ = st; "ys_05_lou_win".into() },
            death: "ys_lose_common",
        }),
        ("ws_servant", FightCfg {
            name: "失控地灵机仆", hp: 96, dmg: (12, 20), reward: 60, reward_why: "失控地灵机仆 · 拆解",
            intro: "圆头单眼的矮机仆拖着锈蚀漆皮朝你走来，地灵符文刻印在关节处闪烁，猩红单眼一眯。",
            rage_at: Some(45), rage_text: "<b>狂暴@45</b>：管线爆浆，全场溅射。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_golem", FightCfg {
            name: "符文电偶", hp: 148, dmg: (16, 26), reward: 160, reward_why: "符文电偶 · 过载",
            intro: "三米人形铁像全身符文回路流光，面部是空洞的放电法阵——它朝你举手，蓝紫电弧在指尖汇聚。",
            rage_at: Some(70), rage_text: "<b>狂暴@70</b>：符文过载，电击连线。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_fused", FightCfg {
            name: "机械缝合体", hp: 126, dmg: (15, 24), reward: 130, reward_why: "机械缝合体 · 肢解",
            intro: "居民骸骨、管线、甲片缝合而成的人形残骸，以向上伸手的姿态朝你扑来——胸腔里一枚「灵核」红光闪烁。",
            rage_at: Some(55), rage_text: "<b>狂暴@55</b>：夺舍扑咬（San -5）。", on_rage: |st, log| {
                st.san = (st.san - 5).clamp(0, 100);
                log.push("<span class='crit'>缝合体扑上来死死咬住你——理智被抽走一线（San -5）。</span>".into());
            },
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_nest", FightCfg {
            name: "银蚴巢群", hp: 108, dmg: (13, 21), reward: 180, reward_why: "银蚴巢群 · 扫平",
            intro: "南区污水渠里黑压压挤着一整窝银色机械蠕虫，触须般的体节在黑暗里此起彼伏。",
            rage_at: Some(50), rage_text: "<b>狂暴@50</b>：增员 2 只！", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_guardline", FightCfg {
            name: "生产线守卫机仆", hp: 118, dmg: (14, 23), reward: 100, reward_why: "生产线守卫机仆 · 拆卸",
            intro: "厂房里的守卫机仆与流水线焊接在一起，机械臂还在按两百年前的节拍校验——校准的对象已无人。",
            rage_at: Some(50), rage_text: "<b>狂暴@50</b>：流水线自动装填，连击。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_proto", FightCfg {
            name: "三神兵 · 试作残骸", hp: 190, dmg: (18, 30), reward: 260, reward_why: "试作残骸 · 镇压",
            intro: "巨型刀、剑、盾的三件半成品焊接体被吊在流水线上缓慢转动——半睁的机械眼朝你投来一束熔炉橙红的光。",
            rage_at: Some(85), rage_text: "<b>狂暴@85</b>：半成品刃口崩解，残片雨。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_pupa", FightCfg {
            name: "低纬度灾厄之蛹", hp: 230, dmg: (17, 28), reward: 320, reward_why: "灾厄之蛹 · 净化",
            intro: "触须、符文与眼柄聚合的蛹体悬浮在裂缝口，几何结构反逻辑地折叠——你多看一眼，理智就开始松动。",
            rage_at: Some(110), rage_text: "<b>狂暴@110</b>：眼柄展开，次声波（San -6）。", on_rage: |st, log| {
                st.san = (st.san - 6).clamp(0, 100);
                log.push("<span class='crit'>灾厄之蛹的眼柄同时睁开，低频次声波在你的颅骨里共振（San -6）。</span>".into());
            },
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_abyss_snake", FightCfg {
            name: "裂缝银蛇", hp: 150, dmg: (15, 26), reward: 210, reward_why: "裂缝银蛇 · 斩除",
            intro: "银鳞微型符文的蛇形寄生体自裂缝滑出，口腔裂隙深处是低纬度的虚空，直直朝你弹射。",
            rage_at: Some(70), rage_text: "<b>狂暴@70</b>：蜕皮重生（回 20%HP）。", on_rage: |st, log| { let _ = st; log.push("<span class='crit'>裂缝银蛇蜕下一层银皮，伤口处重新生长（回血）。</span>".into()); },
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        ("ws_warden", FightCfg {
            name: "机界守墓枢机", hp: 170, dmg: (16, 27), reward: 220, reward_why: "机界守墓枢机 · 镇压",
            intro: "银白流光甲片的枢机人形持长戟立在墓道，胸口「镇压」古符文亮起——守在此地两百年，等一个该来的名字。",
            rage_at: Some(80), rage_text: "<b>狂暴@80</b>：圣痕回路全开，+2 速。", on_rage: ys_rage_common,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: ys_win_common, death: "ys_lose_common",
        }),
        /* ---- BOSS 两段式（§5，机界升华体・瓦罗残响） ---- */
        ("ws_waro_r1", FightCfg {
            name: "机界升华体 · 瓦罗残响（一形态）", hp: 380, dmg: (20, 32), reward: 500, reward_why: "瓦罗残响 · 一段肃清",
            intro: "升华法阵在他脚下亮起，<b>尸骸升天</b>。半圣躯壳外覆机界升华装甲，机械翼向两侧展开——他抬手，发射阵列对准你。",
            rage_at: Some(160), rage_text: "<b>银色飛升</b>：机械臂全开，每回合指令阶段追加一次「升华脉冲」（全队 dmg 6~10 无视护甲）。",
            on_rage: |st, log| {
                st.hp = (st.hp - rnd_ys(6, 10)).max(0);
                log.push("<span class='crit'>銀屑飛升——升华脉冲穿透护甲，灼伤全队（dmg 6~10）。</span>".into());
            },
            finisher_if: |_, ehp| ehp <= 60,
            finisher_name: |st| { if st.flag("ys_waro_cast_stopped") { "断开升华 · 打断".into() } else { "机界升华 · 降临".into() } },
            finisher_desc: |st| {
                if st.flag("ys_waro_cast_stopped") {
                    "你在机界升华法阵蓄力至最高点前冲入祭坛，把地灵符文一把按下。法阵的光如同被掐灭的引线，「嗡」地一滞——升华降临被打断了。".into()
                } else {
                    "他全身的法阵同时亮到最刺目的一刻——『机界升华・降临』！一线银白将你与整片祭坛钉在原地（dmg 28×2 + San -10）。".into()
                }
            },
            win: ys_win_r1, death: "ys_lose_r1",
        }),
        ("ws_waro_r2", FightCfg {
            name: "机界升华体 · 瓦罗残响（二形态）", hp: 300, dmg: (24, 38), reward: 700, reward_why: "瓦罗残响 · 两段平定",
            intro: "瓦罗的最终残响只剩那唯一的人类眼睛还亮着，墨紫裂隙物质缠绕成新的躯壳，低纬度涌出的触须在你脚下盘成一圈圈。",
            rage_at: Some(120), rage_text: "<b>低纬度凝视</b>：裂缝涌入银蛇×2 增员；每次命中携带 San -3。",
            on_rage: |st, log| {
                let _ = st;
                log.push("<span class='crit'>裂缝震颤，两条裂缝银蛇自墨紫裂隙中涌出——每一次命中都携带低纬度凝视（San -3）。</span>".into());
            },
            finisher_if: |_, _| true,
            finisher_name: |st| {
                if st.flag("ys_waro_peace") { "把碎片还给他 · 和平".into() }
                else if st.flag("ys_rift_open") { "裂缝侧门关闭 · 危机".into() }
                else { "强杀 · 平定".into() }
            },
            finisher_desc: |st| {
                if st.flag("ys_waro_peace") {
                    "你走上前，把三神兵·机核碎片轻轻放在法阵中央。瓦罗残响那唯一的人类眼睛忽然一颤——他伸出被裂隙物质裹满的手，不是抓你，而是把碎片拢回自己胸口。".to_string()
                } else if st.flag("ys_rift_open") {
                    "你按下裂缝机关，紧急关闭了通往低纬度的入口。裂隙物质失去供养，瓦罗残响的躯壳开始崩解。".to_string()
                } else {
                    "你咬牙将最后一击贯入他胸口的‘镇压’圣痕。瓦罗残响发出最后一声不属于机械的叹息。".to_string()
                }
            },
            win: ys_win_r2, death: "ys_lose_r2",
        }),
    ]
}

fn rnd_ys(a: i32, b: i32) -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(a..=b)
}

/* =====================================================================
   剧情场景（id 全部 ys_ 前缀）
   ===================================================================== */
pub static YINSE_SCENES: &[SceneDef] = &[

/* ---- 幕 0 ・ 开场：主神空间入口 ---- */
SceneDef {
    id: "ys_00", bg: Some("yinse_bg.png"), loc: Some("主神广场 · 新任务门"),
    mood: "danger", speaker: Some("李铭（记录员）"), voice: Some("vo_yinse_liming_start"),
    text: TextSpec::Static(&[
        "<b>【主线任务】</b>深入银色大地调查 y+E 年地灵族『机界升华』被镇压的真相，夺回（或归还）地灵族三神兵留下的机核碎片，并把地灵族的火种带出遗迹。",
        "「历史被篡改四十八亿次，我们勉强维持在十三次。这一次，轮到你们去修正 y+E 年之后的那片银色大地。」",
        "「档案编号零零二——地灵族机界遗迹。被镇压的文明，会在两百年后，等来一双人类的眼睛。」",
    ]),
    choices: &[
        ChoiceDef { label: "【接受任务】", sub: "接受 → 降落白银荒原", cond: None,
            effects: &[Eff::SetFlag("ys_misson"), Eff::Points(0)], route: Route::To("ys_01_drop") },
        ChoiceDef { label: "【问：瓦罗是谁？】", sub: "旁白给设定 · San-2", cond: None,
            effects: &[Eff::San(-2)], route: Route::To("ys_00_waro") },
        ChoiceDef { label: "【拒绝接受】", sub: "李铭：修正任务不可拒绝 · San-4", cond: None,
            effects: &[Eff::San(-4)], route: Route::To("ys_00_refuse") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_00_waro", bg: Some("yinse_bg.png"), loc: Some("主神广场 · 新任务门"),
    mood: "cold", speaker: Some("李铭（记录员）"), voice: None,
    text: TextSpec::Static(&[
        "「瓦罗——地灵之祖，人类『机械计划』自救分支中最成功的一支。他以圣位之身，断开了外位面的道路。」",
        "「可惜，人类自救的种子，总被万族争霸的巨轮碾进土里。」",
    ]),
    choices: &[ChoiceDef { label: "（回到任务门）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ys_00") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_00_refuse", bg: Some("yinse_bg.png"), loc: Some("主神广场 · 新任务门"),
    mood: "danger", speaker: Some("李铭（记录员）"), voice: Some("vo_yinse_liming_refuse"),
    text: TextSpec::Static(&[
        "「修正任务不可拒绝。」李铭的声音不带温度。「修改的刀刃有差，文明的火种便会熄灭。你只能去。」",
    ]),
    choices: &[ChoiceDef { label: "（被强制弹入传送）", sub: "强制接受", cond: None, effects: &[Eff::Points(0)], route: Route::To("ys_01_drop") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 1 ・ 降落白银荒原（L1）：阿桑遇伏 ---- */
SceneDef {
    id: "ys_01_drop", bg: Some("yinse_bg.png"), loc: Some("L1 白银荒原 · 降落点"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "脚下是不再反光的白银。风里有铁锈、尸脂，和某种仍然通电的嗡鸣。",
        "远处战壕传来金属刮擦声——有人被围了。一名瘦小的身影被两个灰绿色兽人逼到断墙边。",
    ]),
    choices: &[
        ChoiceDef { label: "【冲上去救人】", sub: "遭遇 ys_scav ×2", cond: None,
            effects: &[Eff::MarkPoint("ys_pt_drop"), Eff::Points(20)], route: Route::To("ys_03_asang_save") },
        ChoiceDef { label: "【用铁棒敲击引开】", sub: "需理智≥40 判定 · 失败则阿桑掉血", cond: None,
            effects: &[Eff::San(-5)], route: Route::Dyn(route_taotao) },
        ChoiceDef { label: "【绕行（放弃阿桑）】", sub: "阿桑战死 · San-10", cond: None,
            effects: &[Eff::SetFlag("ys_asang_dead"), Eff::San(-10)], route: Route::To("ys_02_war_flags") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_03_asang_save", bg: Some("yinse_bg.png"), loc: Some("L1 战壕 · 救人"),
    mood: "danger", speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["两名古兽人拾荒者转过身——你坏了它们的好事。它们逼近了。"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_scav ×2（连战）", cond: None, effects: &NO_EFF, route: Route::To("ys_03_asang_win") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_03_asang_win", bg: Some("yinse_bg.png"), loc: Some("L1 战壕 · 救人"),
    mood: "calm", speaker: Some("阿桑"), voice: Some("vo_yinse_asang"),
    text: TextSpec::Static(&[
        "拾荒者溃逃之后，那名瘦削的少年站起身，灰扑扑的脸上还带着惊魂未定。",
        "「我……我叫阿桑，人族劫掠队就剩我一个了。他们说瓦罗是叛徒……可我爸说过，叛徒不会把地灵族的种子，埋在被镇压的城里。」",
    ]),
    choices: &[
        ChoiceDef { label: "【收入队】", sub: "支线 save_asang +200 · 阿桑入队", cond: None,
            effects: &[Eff::SetFlag("ys_save_asang"), Eff::SetFlag("ys_asang_ally"), Eff::PointsIfFlag("ys_save_asang", 200)], route: Route::To("ys_04_trench") },
        ChoiceDef { label: "（先去北废墟查看战旗）", sub: "情报线", cond: None, effects: &NO_EFF, route: Route::To("ys_02_war_flags") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_03_asang", bg: Some("yinse_bg.png"), loc: Some("L1 · 阿桑（NPC）"),
    mood: "cold", speaker: Some("阿桑"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ys_save_asang") {
            "「机核碎片……我听我爸提过，是地灵族三神兵的核心。别的我不敢保证，但我知道它被藏在都市某处，得先让城里的灯重新亮起来。」".to_string()
        } else {
            "「那家伙……战潮王・髅，是古兽人余孽里最疯的一个。电梯井被它的人守着，除非你有地灵方解石……或者从它尸体上搜。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "（继续）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ys_04_trench") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 2 ・ 战壕：战潮王・髅（miniboss） —— */
SceneDef {
    id: "ys_02_war_flags", bg: Some("yinse_bg.png"), loc: Some("L1 北废墟 · 战旗"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "北废墟里立着一面破败的战旗。旗面上用兽血绘着一颗颅骨——那是战潮王・髅的纹章。",
        "旗杆下压着一块残旧的机械臂铁皮，上面用刀刻着断续的坐标：指向南侧的电梯塔，标注着『地灵方解石』四个字。",
        "（电梯井情报 + 髅的来历。）",
    ]),
    choices: &[ChoiceDef { label: "（走去战壕汇合点）", sub: "触发战潮王", cond: None, effects: &[Eff::Points(10)], route: Route::To("ys_04_trench") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_04_trench", bg: Some("yinse_bg.png"), loc: Some("L1 战壕汇合点"),
    mood: "danger", speaker: Some("阿桑"), voice: None,
    text: TextSpec::Dyn(|st| {
        let head = "一具背插战旗的巨影坐在尸堆上，把地灵族的机械臂当串肉签。他听见你的脚步，缓缓抬起头。";
        let hint = if st.flag("ys_save_asang") {
            "阿桑小声说：「那是战潮王・髅，古兽人余孽里最疯的一个。」"
        } else {
            "你孤身一人。战旗上的颅骨纹章在风里翻卷——战潮王·髅，没有同伴敢走在它前面。"
        };
        format!("{head}\n\n{hint}\n\n那是一尊近三米的巨型兽人主将——白骨面具，背后残破战旗，手持巨大骨锯。G1 电梯井的铁闸需要地灵方解石。")
    }),
    choices: &[
        ChoiceDef { label: "【迎战（战潮王·髅）】", sub: "miniboss ys_lou · 胜得方解石+髅骨", cond: None,
            effects: &NO_EFF, route: Route::To("ys_05_lou_enter") },
        ChoiceDef { label: "【丢下银蚴尸体诱饵绕过】", sub: "需银色机械蠕虫·残骸", cond: Some(cond_has_worm_corpse),
            effects: &[Eff::SetFlag("ys_lou_bypassed"), Eff::San(-5)], route: Route::To("ys_05_bypass") },
        ChoiceDef { label: "（无诱饵）强行绕过", sub: "San-8 · 触发额外遭遇", cond: None,
            effects: &[Eff::San(-8)], route: Route::To("ys_05_bypass_forced") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_05_lou_enter", bg: Some("yinse_bg.png"), loc: Some("L1 · 战潮王・髅战场"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你拔出武器。白骨面具下的那排獠牙，朝你咧开了。「古兽人的地盘——人类，你也算一条命。」"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_lou", cond: None, effects: &NO_EFF, route: Route::To("ys_05_lou_win") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_05_lou_win", bg: Some("yinse_bg.png"), loc: Some("L1 · 战潮王・髅战场"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["骨锯砸在白银大地上，扬起一片灰。你在尸堆里翻出一枚银白色结晶体——地灵方解石，还有一节髅骨。"]),
    choices: &[ChoiceDef { label: "（走向电梯井）", sub: "G1 · 得地灵方解石+髅骨", cond: None,
        effects: &[Eff::SetFlag("ys_war_lord_slain"), Eff::AddItem("item_diling"), Eff::AddItem("item_lou_bone")], route: Route::To("ys_05_ele1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_05_bypass", bg: Some("yinse_bg.png"), loc: Some("L1 战壕 · 绕过"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你丢下银蚴的尸体。战潮王嗅到同类的死气，迟疑片刻，最终挥挥手让开了路——但电梯井仍需地灵方解石。"]),
    choices: &[ChoiceDef { label: "（走向电梯井）", sub: "G1", cond: None,
        effects: &[Eff::SetFlag("ys_lou_bypassed")], route: Route::To("ys_05_ele1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_05_bypass_forced", bg: Some("yinse_bg.png"), loc: Some("L1 战壕 · 被堵"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你试图沿塌方斜道绕行，却被两名拾荒者与一条银蚴堵了正着——没有诱饵，它们发现了你。"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_scav + ys_worm", cond: None, effects: &NO_EFF, route: Route::To("ys_05_ele1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* G1 电梯井 */
SceneDef {
    id: "ys_05_ele1", bg: Some("yinse_bg.png"), loc: Some("L1 南电梯塔 · 电梯井"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ys_lou_bypassed") {
            "你从髅的残躯旁绕过，没能得到地灵方解石。电梯井的铁闸纹丝不动——你只能绕行塌方斜道（多 2 场遭遇）。".to_string()
        } else {
            "你握着地灵方解石来到电梯井。南侧塔楼里的轿厢早已坠毁，铁闸横在井口——门禁 G1。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【（用地灵方解石）开电梯井】", sub: "G1", cond: Some(cond_has_diling),
            effects: &NO_EFF, route: Route::To("ys_05_ele1_open") },
        ChoiceDef { label: "【绕行塌方斜道】", sub: "多 2 场遭遇 · San-6", cond: None,
            effects: &[Eff::San(-6)], route: Route::Dyn(route_slope) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_05_ele1_open", bg: Some("yinse_bg.png"), loc: Some("L1 → L2 电梯井下行"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["地灵方解石贴合凹槽，「咔」一声轻响，铁闸升起。你踏入黑暗的电梯井——脚下传来失重的一瞬，是下降。"]),
    choices: &[ChoiceDef { label: "（下行 L2）", sub: "pt_down1 → L2(36,4)", cond: None, effects: &NO_EFF, route: Route::To("ys_06_city") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_05_ele1_open_slope", bg: Some("yinse_bg.png"), loc: Some("L1 南侧塌方斜道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["塌方斜道上的两名拾荒者与一条银蚴挡路——你只能杀过去。"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_scav + ys_worm", cond: None, effects: &NO_EFF, route: Route::To("ys_06_city") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 3 ・ 都市遗迹（L2）：居民骸骨 / 机关链 / 小枢 / 库房 ---- */
SceneDef {
    id: "ys_06_city", bg: Some("yinse_bg.png"), loc: Some("L2 都市遗迹 · 电梯到站台"),
    mood: "danger", speaker: Some("旁白"), voice: None,
    text: TextSpec::Static(&[
        "电梯在都市遗迹中央停下。街区还亮着40%的符文灯——两百年没坏，因为它们在等人回来供电。",
        "（配电塔总控在西北角；长街上的居民骸骨是升华现场。）",
    ]),
    choices: &[
        ChoiceDef { label: "【去配电塔总控（机关链起点）】", sub: "Z1", cond: None, effects: &NO_EFF, route: Route::To("ys_06_power_master") },
        ChoiceDef { label: "【去居民骸骨长街】", sub: "san-8 · 真相线索", cond: None, effects: &NO_EFF, route: Route::To("ys_07_home_bones") },
        ChoiceDef { label: "【找小枢（地灵族遗民）】", sub: "NPC · 密钥/工厂图", cond: None, effects: &NO_EFF, route: Route::To("ys_07_xiaoshu") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 机关链 master→B→C（顺序，错序触发电偶） */
SceneDef {
    id: "ys_06_power_master", bg: Some("yinse_bg.png"), loc: Some("L2 配电塔总控"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "配电塔总控台的符文屏还亮着最后一线蓝光，一封两百年前的调度日志弹出来：『先合主闸，次合 B 支路，再合 C 支路——顺序不可乱。』",
        "（机关链起点：先拉总控。）",
    ]),
    choices: &[
        ChoiceDef { label: "【拉下总控（master）】", sub: "Z1 完成", cond: None,
            effects: &[Eff::SetFlag("ys_l2_power_master")], route: Route::To("ys_06_power_mid") },
        ChoiceDef { label: "（回头先看长街）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ys_07_home_bones") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_power_mid", bg: Some("yinse_bg.png"), loc: Some("L2 配电塔 · 拔闸提示"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["总控已合。调度日志亮起下一行指示：『次合 B 支路（东侧 (24,9)），最后合 C 支路（南侧 (16,22)）。』"]),
    choices: &[ChoiceDef { label: "（去配电点 B）", sub: "Z2", cond: None, effects: &NO_EFF, route: Route::To("ys_06_power_b") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_power_b", bg: Some("yinse_bg.png"), loc: Some("L2 配电点 B"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["配电点 B 的拉闸卡在铁锈里。你在正确顺序中（先 master 后 B）。"]),
    choices: &[
        ChoiceDef { label: "【拉下配电点 B】", sub: "Z2 完成", cond: None,
            effects: &[Eff::SetFlag("ys_l2_power_b")], route: Route::To("ys_06_power_c") },
        ChoiceDef { label: "【先动 C（错序）】", sub: "错序触发电偶战斗", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_wrong_order) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_power_c", bg: Some("yinse_bg.png"), loc: Some("L2 配电点 C"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["配电点 C 的拉闸。A 与 B 已依序合上，只剩最后一步。"]),
    choices: &[
        ChoiceDef { label: "【拉下配电点 C】", sub: "机关链完成 → l2_power_restored", cond: None,
            effects: &[Eff::SetFlag("ys_l2_power_restored"), Eff::PointsIfFlag("ys_l2_power_restored", 200)], route: Route::To("ys_06_power_done") },
        ChoiceDef { label: "【先回总控（错序）】", sub: "错序触发电偶战斗", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_wrong_order) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_golem_fight", bg: Some("yinse_bg.png"), loc: Some("L2 · 错序 · 符文电偶"),
    mood: "danger", speaker: Some("⚔ 遭遇"), voice: None,
    text: TextSpec::Static(&["操作顺序出错，配电塔的保护回路暴走——一尊三米符文电偶自塔内升起，电光四溅。"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_golem", cond: None, effects: &NO_EFF, route: Route::To("ys_06_golem_win") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_golem_win", bg: Some("yinse_bg.png"), loc: Some("L2 配电塔"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["电偶坍倒，电弧归于死寂。调度日志依旧固执地闪烁：『请按 master→B→C 顺序。』"]),
    choices: &[ChoiceDef { label: "（回配电塔总控重新操作）", sub: "Z1", cond: None, effects: &NO_EFF, route: Route::To("ys_06_power_master") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_power_done", bg: Some("yinse_bg.png"), loc: Some("L2 都市街道 · 全面送电"),
    mood: "calm", speaker: Some("小枢"), voice: None,
    text: TextSpec::Static(&[
        "三处配电点依序闭合的刹那，整座都市的符文灯一明，尔后全数亮起——两百年的夜晚，第一次被点亮。",
        "几十根符文灯柱同时亮了。远处某处传来一声机械的、像叹息又像欢呼的『呜——』。",
    ]),
    choices: &[ChoiceDef { label: "（前去符文闸门 G2）", sub: "G2 已开", cond: None,
        effects: &[Eff::SetFlag("ys_06_power_restored_note")], route: Route::To("ys_06_gate2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_06_gate2", bg: Some("yinse_bg.png"), loc: Some("L2 符文闸门前"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["供电恢复，符文闸门（G2）自下而上亮起，轰然升开。都市中心大道、隐藏库房与回程吊索全部畅通。"]),
    choices: &[ChoiceDef { label: "（前往居民骸骨长街 / 小枢 / 隐藏库房）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ys_06_city") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 居民骸骨长街（san-8 真相） */
SceneDef {
    id: "ys_07_home_bones", bg: Some("yinse_bg.png"), loc: Some("L2 居民骸骨长街"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(txt_home_bones),
    choices: &[ChoiceDef { label: "（凝视片刻，后退）", sub: "San-8 · 真相线索", cond: None,
        effects: &[Eff::San(-8)], route: Route::To("ys_06_city") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 小枢（NPC） */
SceneDef {
    id: "ys_07_xiaoshu", bg: Some("yinse_bg.png"), loc: Some("L2 · 小枢"),
    mood: "cold", speaker: Some("小枢（机械幼体意识）"), voice: Some("vo_yinse_xiaoshu"),
    text: TextSpec::Static(&[
        "一尊圆头单眼的小机械体从配电塔阴影里探出来，独眼的红光明明灭灭，努力用机械音吐出断续的话：",
        "「你……是人类。老师说过，升华成功那天，我们就不用再吃人血了……可我们最后，吃的是自己。」",
        "「门口那间库房……钥匙在这里。带我去……去安全的地方，好不好。」",
    ]),
    choices: &[
        ChoiceDef { label: "【轻抚她，答应带她走】", sub: "支线 diling_friend +200 · 赠地灵密钥", cond: None,
            effects: &[Eff::SetFlag("ys_diling_friend"), Eff::SetFlag("ys_asang_ally"), Eff::PointsIfFlag("ys_diling_friend", 200), Eff::AddItem("item_key")],
            route: Route::To("ys_07_xiaoshu_ok") },
        ChoiceDef { label: "（请她指路后暂不随行）", sub: "仍可得密钥线索", cond: None,
            effects: &[Eff::AddItem("item_key")], route: Route::To("ys_06_city") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_07_xiaoshu_ok", bg: Some("yinse_bg.png"), loc: Some("L2 · 小枢随行"),
    mood: "calm", speaker: Some("小枢"), voice: None,
    text: TextSpec::Static(&["小枢的独眼眨了眨，机械音里竟听出几分欣喜：「好……我记下你的方向。你们要去的是那间库房——地灵密钥给你，里面有老师留的东西。」"]),
    choices: &[ChoiceDef { label: "（前往隐藏库房）", sub: "需地灵密钥 + l2_power_restored", cond: None,
        effects: &NO_EFF, route: Route::To("ys_08_vault") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 隐藏库房（三神兵·机核碎片） */
SceneDef {
    id: "ys_08_vault", bg: Some("yinse_bg.png"), loc: Some("L2 隐藏库房"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["库房的门缝里漏出幽蓝圣光。你用随身的地灵密钥拧开门——满屋的蓝图和齿轮间，一枚不规则机械核心静静躺在托架上。"]),
    choices: &[
        ChoiceDef { label: "【取走三神兵·机核碎片】", sub: "关键道具 · San-3 · 每持 1 幕 -2", cond: None,
            effects: &[Eff::AddItem("item_jiche"), Eff::San(-3)], route: Route::To("ys_08_vault_taken") },
        ChoiceDef { label: "（先不取，退走）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ys_06_city") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_08_vault_taken", bg: Some("yinse_bg.png"), loc: Some("L2 隐藏库房 · 取走碎片"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你托起那枚不规则机核碎片，触手微烫，颅内掠过一声不属于你的叹息——那是两百年前，一个文明最后的『我还在』。",
        "（持有碎片：每经历 1 幕再 San -2；可交还、带回主神，或用于 G3 真相回放。）",
    ]),
    choices: &[ChoiceDef { label: "（前往 L3 工厂）", sub: "pt_down2", cond: None,
        effects: &[Eff::San(-2)], route: Route::To("ys_09_factory") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 4 ・ 工厂（L3）：生产线 / 升华装置 / 裂缝 ---- */
SceneDef {
    id: "ys_09_factory", bg: Some("yinse_bg.png"), loc: Some("L3 机界升华工厂 · 入口"),
    mood: "danger", speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "流水线还在转——没人维护的机械臂，仍在把『人形材料』压进模子里。两百年前，它们压的是地灵族自己。",
        "东侧的大裂缝泛着墨紫色的光；北中处是升华装置启动间（G3）；底部中央是通往瓦罗之墓的主升降井（G4）。",
    ]),
    choices: &[
        ChoiceDef { label: "【调查三神兵生产线】", sub: "三神兵来历 + 试作残骸", cond: None, effects: &NO_EFF, route: Route::To("ys_09_assembly_line") },
        ChoiceDef { label: "【去升华装置启动间（G3）】", sub: "需机核碎片 · 真相回放", cond: Some(cond_sublime_unlocked), effects: &NO_EFF, route: Route::To("ys_10_sublime") },
        ChoiceDef { label: "【去低纬度裂缝】", sub: "灾厄区 · 机关拉杆", cond: None, effects: &NO_EFF, route: Route::To("ys_11_rift") },
        ChoiceDef { label: "（G4 已开）主升降井下行", sub: "fL4", cond: Some(cond_core_open), effects: &NO_EFF, route: Route::To("ys_12_tomb") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_09_assembly_line", bg: Some("yinse_bg.png"), loc: Some("L3 三神兵生产线"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "巨型刀、剑、盾的三件半成品焊接体还挂在流水线上——攻/防/机，地灵族三神兵的雏形。",
        "旁边的残破铭牌记载：『攻守兼备者，守护者；机动穿梭者，破阵者；其上，为一族之心。』",
        "生产线深处传来锈蚀的机械运转声——试作残骸被惊醒了。",
    ]),
    choices: &[ChoiceDef { label: "【迎战三神兵·试作残骸】", sub: "ys_proto", cond: None,
        effects: &[Eff::Points(10)], route: Route::To("ys_09_proto_enter") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_09_proto_enter", bg: Some("yinse_bg.png"), loc: Some("L3 三神兵生产线 · 试作"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["试作残骸的半睁机械眼投来熔炉橙红的光——它从流水线解下，重力落地，地面龟裂。"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_proto", cond: None, effects: &NO_EFF, route: Route::To("ys_09_factory") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* G3 升华装置启动间 · 真相回放（Sequence Cine） */
SceneDef {
    id: "ys_10_sublime", bg: Some("yinse_bg.png"), loc: Some("L3 升华装置启动间"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "机核碎片嵌入，装置启动。眼前炸开一连串圣光影像——",
        "影像一（y+d 年）：瓦罗断开外位面道路，地灵族计划成功——『我们不是叛徒，我们是人类被万族打碎后，自己焊起来的机械骨头。』",
        "影像二（y+E 年）：古兽人叩城。一只白羽巨鸟立于云端，『指点』瓦罗沟通低纬度——『升上去，就再也没人敢吃你们。』",
        "影像三：东天二皇天倾镇压——「镇压——凡升者，必坠。」",
    ]),
    choices: &[
        ChoiceDef { label: "【洞悉真相】", sub: "flag waro_truth +200 · 揭示鲲鹏算计", cond: None,
            effects: &[Eff::SetFlag("ys_waro_truth"), Eff::SetFlag("ys_waro_truth_seed"), Eff::PointsIfFlag("ys_waro_truth", 200), Eff::SetFlag("ys_core_open")],
            route: Route::To("ys_10_truth_found") },
        ChoiceDef { label: "【记录影像后退出】", sub: "不标记真相", cond: None,
            effects: &NO_EFF, route: Route::To("ys_09_factory") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_10_truth_found", bg: Some("yinse_bg.png"), loc: Some("L3 升华装置启动间 · 真相"),
    mood: "danger", speaker: Some("瓦罗残响（预录）"), voice: None,
    text: TextSpec::Static(&[
        "影像最后定格在瓦罗那张只剩一条缝的圣面上。他临终的声音破开两百年：『人类……我不是自愿背叛。我认得我焊的骨头。』",
        "『鲲鹏骗了我，低纬度诱惑了我，东天二皇镇压了我。可我把机核藏进都市——我等的人，总会来。』",
        "启动间深处，圣力感应到你已洞悉真相，通往瓦罗之墓的主门（G4）封印松动。",
    ]),
    choices: &[ChoiceDef { label: "（前行 L4）", sub: "G4 已开", cond: None, effects: &NO_EFF, route: Route::To("ys_12_tomb") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 低纬度裂缝（灾厄区 + 机关拉杆） */
SceneDef {
    id: "ys_11_rift", bg: Some("yinse_bg.png"), loc: Some("L3 低纬度裂缝"),
    mood: "danger", speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "一道墨紫色的巨大裂缝撕开厂房东壁，逆几何的虚空在裂缝里缓缓坍缩，漂浮着零碎的机械残骸。",
        "裂缝口趴着一只低纬度灾厄之蛹，两条裂缝银蛇缠绕吐信。裂缝侧还有一具古旧的拉杆——通向未知。",
    ]),
    choices: &[
        ChoiceDef { label: "【迎战裂缝守卫】", sub: "ys_pupa + ys_abyss_snake ×2", cond: None, effects: &NO_EFF, route: Route::To("ys_11_rift_fight") },
        ChoiceDef { label: "【拉动机关拉杆（机关链末端）】", sub: "二选一：开/关裂缝", cond: None, effects: &NO_EFF, route: Route::To("ys_11_rift_lever") },
        ChoiceDef { label: "（裂缝彩蛋）凝望裂缝深处", sub: "无战斗 · 骨的身影", cond: None, effects: &NO_EFF, route: Route::To("ys_11_rift_easter") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_11_rift_fight", bg: Some("yinse_bg.png"), loc: Some("L3 裂缝口 · 战斗"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["灾厄之蛹的眼柄同时睁开，两条银蛇自裂缝涌出——它们扑来！"]),
    choices: &[ChoiceDef { label: "战斗", sub: "ys_pupa + 银蛇×2", cond: None, effects: &NO_EFF, route: Route::To("ys_11_rift_lever") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_11_rift_lever", bg: Some("yinse_bg.png"), loc: Some("L3 裂缝机关拉杆"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["拉杆分两个方向：向右，激活裂缝传送（可直达瓦罗之墓的裂隙台）；向左，关闭裂缝，让灾厄停息。只能选一次。"]),
    choices: &[
        ChoiceDef { label: "【激活裂缝传送（开）】", sub: "支线 rift_open +200 · 可绕 G4", cond: None,
            effects: &[Eff::SetFlag("ys_rift_open"), Eff::PointsIfFlag("ys_rift_open", 200)], route: Route::To("ys_11_rift_open") },
        ChoiceDef { label: "【关闭裂缝（合）】", sub: "支线 rift_closed +200", cond: None,
            effects: &[Eff::SetFlag("ys_rift_closed"), Eff::PointsIfFlag("ys_rift_closed", 200), Eff::SetFlag("ys_core_open")], route: Route::To("ys_11_rift_closed") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_11_rift_open", bg: Some("yinse_bg.png"), loc: Some("L3 裂缝传送激活"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你把拉杆推向右侧。裂缝亮起墨紫色的传送门——可直接上瓦罗之墓的裂隙台，但走此路会 San -15，并遭遇灾厄。",
        "（也可仍走主门 G4——若你已洞悉真相或持有机核。）",
    ]),
    choices: &[
        ChoiceDef { label: "（踏进裂缝传送 → L4 裂隙台）", sub: "rift_open · San-15", cond: None,
            effects: &[Eff::San(-15), Eff::SetFlag("ys_entered_side")], route: Route::To("ys_12_tomb_side") },
        ChoiceDef { label: "（改走主升降井）", sub: "G4", cond: None, effects: &NO_EFF, route: Route::Dyn(route_core) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_11_rift_closed", bg: Some("yinse_bg.png"), loc: Some("L3 裂缝关闭"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&["你向左按下拉杆。裂缝无声合拢，灾厄的笔墨紫虚空像一道被拉上的拉链——厂房重新安静下来。低纬度的吠鸣消失了。"]),
    choices: &[ChoiceDef { label: "（转往主升降井 G4）", sub: "关闭裂缝同样强化了核心封印判断", cond: None, effects: &NO_EFF, route: Route::Dyn(route_core) }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 裂缝彩蛋：骨（机械升华体） */
SceneDef {
    id: "ys_11_rift_easter", bg: Some("yinse_bg.png"), loc: Some("L3 裂缝 · 彩蛋"),
    mood: "cold", speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "你凝望裂缝深处。逆几何的虚空里，一道与瓦罗相似的机械升华体身影一闪而过——他没有被镇压，也没有陨落，而是朝裂缝更深处走去。",
        "李铭的声音停在半空：「那是……后来的贤者，骨。原来升华并非全灭。有人带着那份力量，叛出了这里。」",
    ]),
    choices: &[ChoiceDef { label: "（记住这道身影）", sub: "", cond: None, effects: &[Eff::Points(10)], route: Route::To("ys_11_rift") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 5 ・ 转折：瓦罗的真相（L4 门外） ---- */
SceneDef {
    id: "ys_12_tomb", bg: Some("yinse_bg.png"), loc: Some("L4 环形墓道 · 决战祭坛前"),
    mood: "danger", speaker: Some("瓦罗残响"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ys_entered_side") {
            "你自低纬度裂隙台坠入墓室。一路的震荡让瓦罗残响的机壳猛烈颤动——它被裂缝侧门惊动，恨意提前蒸腾。".to_string()
        } else {
            "环形墓道尽头，祭坛前。石门后传来一个 200 年没有声带却仍在『说话』的声音：「人类……又是人类的火种。」".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【听他说完（真相）】", sub: "waro_truth +200 · 机密与真相演出", cond: None,
            effects: &[Eff::SetFlag("ys_waro_truth"), Eff::PointsIfFlag("ys_waro_truth", 200)], route: Route::To("ys_12_truth_talk") },
        ChoiceDef { label: "【直接开战】", sub: "跳过真相 · r1 狂暴阈值 -20%", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_fight_no_truth) },
        ChoiceDef { label: "【先关闭升华装置】", sub: "需回 L3 · 多 2 场遭遇", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_stop_machine) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_12_truth_talk", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛前 · 真相"),
    mood: "calm", speaker: Some("瓦罗残响"), voice: Some("vo_yinse_waro_truth"),
    text: TextSpec::Static(&[
        "石门无声滑开一线。你听完了瓦罗的独白——被鲲鹏算计、被低纬度诱惑、被东天二皇镇压，而非自愿堕落。",
        "「人类……我又见到人类了。告诉后来者——地灵族，从来不是叛徒。我们只是，被这个世界骗着，先走了一步。」",
        "若你藏有碎片，他那只人类眼睛微微一动——他认得自己种的火。",
    ]),
    choices: &[
        ChoiceDef { label: "【持有碎片 → 把机核还给他】", sub: "waro_peace +200 · 和平路线", cond: Some(cond_has_jiche),
            effects: &[Eff::SetFlag("ys_waro_peace"), Eff::PointsIfFlag("ys_waro_peace", 200)], route: Route::To("ys_12_peace") },
        ChoiceDef { label: "【记下真相（不还）】", sub: "正常进决战", cond: None,
            effects: &NO_EFF, route: Route::Dyn(route_fight_truth) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_12_peace", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛前 · 交还"),
    mood: "calm", speaker: Some("瓦罗残响"), voice: None,
    text: TextSpec::Static(&[
        "你把三神兵·机核碎片举到祭坛前。瓦罗残响的机壳短暂浮出『平静』的面容。「原来……人类的火，还亮着。」",
        "交还碎片后，BOSS 二阶段的狂暴阈值降至 100，末日技变为可谈判——『把碎片还给我』。",
    ]),
    choices: &[ChoiceDef { label: "（步入决战祭坛）", sub: "进入二段可谈判战", cond: None,
        effects: &[Eff::SetFlag("ys_waro_returned")], route: Route::To("ys_13_fight_r1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_12_machine_off", bg: Some("yinse_bg.png"), loc: Some("L3 升华装置启动间 · 强行关闭"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你折返 L3，强行重启升华装置并立即断开熔断器。机器发出濒死的轰鸣，转子缓缓停转。裂缝里传来一声短促的怒吼。",
        "你来时多遭遇了两场守卫机仆；归来时，瓦罗残响的伤害被全局 -3。",
    ]),
    choices: &[ChoiceDef { label: "（战斗：守卫机仆）", sub: "ys_guardline ×2", cond: None,
        effects: &[Eff::SetFlag("ys_waro_machine_off"), Eff::PointsIfFlag("ys_waro_machine_off", 200)], route: Route::To("ys_12_machine_off_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_12_machine_off_back", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛前"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["你回到祭坛前。装置已停，裂缝的光线黯淡，瓦罗残响的躯壳比先前少了三分凶戾。"]),
    choices: &[ChoiceDef { label: "（步入决战祭坛）", sub: "waro_machine_off · BOSS dmg -3", cond: None,
        effects: &[Eff::SetFlag("ys_waro_machine_off")], route: Route::To("ys_13_fight_r1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_12_tomb_side", bg: Some("yinse_bg.png"), loc: Some("L4 裂隙台 · 侧门落地"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你从裂隙台的边缘踉跄落地，额头冷汗直流。瓦罗残响的机壳在祭坛中央轰然立起——仇恨被裂缝侧门的闯入彻底点燃，它开局便狂暴。"]),
    choices: &[ChoiceDef { label: "（正面迎战）", sub: "r1 狂暴阈值 @170", cond: None,
        effects: &[Eff::SetFlag("ys_entered_side")], route: Route::To("ys_13_fight_r1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_12_stele", bg: Some("yinse_bg.png"), loc: Some("L4 瓦罗石碑"),
    mood: "cold", speaker: None, voice: None,
    text: TextSpec::Static(&["石碑上刻着一行诗谜：「白银大地葬的不是龙，是把自己焊接成骨头的一族。火种不熄，便不算枉死。」（San -2）"]),
    choices: &[ChoiceDef { label: "（记住碑文）", sub: "San-2", cond: None,
        effects: &[Eff::San(-2), Eff::SetFlag("ys_waro_truth_seed_extra")], route: Route::To("ys_12_tomb") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 幕 6 ・ 决战：机界升华体 两段式（场景链） ---- */
SceneDef {
    id: "ys_13_fight_r1", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛 · 一形态"),
    mood: "danger", speaker: Some("瓦罗残响"), voice: None,
    text: TextSpec::Dyn(txt_boss_r1),
    choices: &[
        ChoiceDef { label: "【开始战斗（一形态）】", sub: "ws_waro_r1", cond: None,
            effects: &NO_EFF, route: Route::Dyn(start_waro_r1) },
        ChoiceDef { label: "（检查祭坛符文）", sub: "打断升华需 l2_power_restored", cond: None,
            effects: &NO_EFF, route: Route::To("ys_13_cast_check") },
    ],
    fight_id: None, video: None, cine_label: Some("决战 · 机界升华体"), overlay: None,
},
SceneDef {
    id: "ys_13_cast_check", bg: Some("yinse_bg.png"), loc: Some("L4 祭坛 · 符文检查"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("ys_l2_power_restored") {
            "你认出了祭坛符文与都市配电塔同源——都市的电能可以逆向注入，打断升华蓄力！".to_string()
        } else {
            "祭坛符文陌生而古老，你无从下手。要打断升华，也许需先恢复都市供电。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "【强大：注入都市电能使升华可被打断】", sub: "需 l2_power_restored · 蓄力轮打断生效", cond: Some(cond_power_restored),
            effects: &[Eff::SetFlag("ys_waro_cast_stopped")], route: Route::Dyn(start_waro_r1) },
        ChoiceDef { label: "（无法打断，直接开战）", sub: "无打断机制", cond: None,
            effects: &NO_EFF, route: Route::Dyn(start_waro_r1) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 一段战 · 选择驱动回合 */
SceneDef {
    id: "ys_13_round_r1", bg: Some("yinse_bg.png"), loc: Some("L4 · 决战 · 一形态"),
    mood: "danger", speaker: Some("⚔ 决战"), voice: None,
    text: TextSpec::Dyn(txt_round_r1),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 34-52", cond: None, effects: &NO_EFF,
            route: Route::Dyn(|st| route_r1_attack(st, rnd_ys(34, 52))) },
        ChoiceDef { label: "【注入都市电能打断升华】", sub: "蓄力轮 · 需 l2_power_restored", cond: Some(cond_power_restored),
            effects: &NO_EFF, route: Route::Dyn(|st| { st.set_flag("ys_waro_cast_stopped"); route_r1_attack(st, rnd_ys(20, 30)) }) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* 转场演出：东天二皇投影（演出级，不可战） */
SceneDef {
    id: "ys_waR0_cast", bg: Some("yinse_bg.png"), loc: Some("L4 祭坛 · 转场演出"),
    mood: "danger", speaker: Some("东皇太一投影"), voice: Some("vo_yinse_taiyi"),
    text: TextSpec::Static(&[
        "一形态溃灭的刹那，祭坛上的升华法阵彻底崩解。一线巨影自天而降——",
        "双重日冕剪影：帝俊执河图洛书，太一执东皇钟。太一一字一顿：「镇压——凡升者，必坠。」",
        "话音未落，低纬度裂缝自法阵废墟中撕开，把两皇投影连同一整个时代，一并吞没。余韵里只剩一句：瓦罗残响的机壳，在升华的能量被攫走之后，蓄满了第二次伏起——<b>真理形态，天翻地覆。</b>",
    ]),
    choices: &[
        ChoiceDef { label: "（凝望投影被裂缝吞没）", sub: "演出 · 转入二形态", cond: None,
            effects: &NO_EFF, route: Route::Dyn(start_waro_r2) },
    ],
    fight_id: None, video: Some("cine_huang_yaji.002"), cine_label: Some("东天二皇 · 镇压"), overlay: None,
},
/* 二段战 · 入口 */
SceneDef {
    id: "ys_14_fight_r2", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛 · 二形态"),
    mood: "danger", speaker: Some("瓦罗残响"), voice: None,
    text: TextSpec::Dyn(txt_boss_r2),
    choices: &[
        ChoiceDef { label: "【开始决战（二形态）】", sub: "ws_waro_r2", cond: None,
            effects: &NO_EFF, route: Route::To("ys_14_round_r2") },
        ChoiceDef { label: "（若 private 伤害叠加提示）", sub: "末技低纬度凝视 San-5/击", cond: None,
            effects: &NO_EFF, route: Route::To("ys_14_round_r2") },
    ],
    fight_id: None, video: None, cine_label: Some("决战 · 真理形态"), overlay: None,
},
/* 二段战 · 选择驱动回合 */
SceneDef {
    id: "ys_14_round_r2", bg: Some("yinse_bg.png"), loc: Some("L4 · 决战 · 二形态"),
    mood: "danger", speaker: Some("⚔ 决战"), voice: None,
    text: TextSpec::Dyn(txt_round_r2),
    choices: &[
        ChoiceDef { label: "重击（强攻）", sub: "伤害 30-48", cond: None, effects: &NO_EFF,
            route: Route::Dyn(|st| route_r2_attack(st, rnd_ys(30, 48))) },
        ChoiceDef { label: "【把碎片交还（和平谈判）】", sub: "需 waro_peace · 提前结束", cond: Some(cond_has_peace),
            effects: &NO_EFF, route: Route::Dyn(route_r2_surrender) },
        ChoiceDef { label: "【裂缝机关紧急关闭】", sub: "需 rift_open · 压制天翻地覆", cond: Some(cond_rift_open),
            effects: &NO_EFF, route: Route::Dyn(|st| { st.set_flag("ys_rift_closed_emergency"); route_r2_attack(st, rnd_ys(40, 60)) }) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* ---- 结局（按 flag 分支） ---- */
SceneDef {
    id: "ys_15_ending_peace", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛 · 撤离传送门"),
    mood: "calm", speaker: Some("瓦罗残响（遗言）"), voice: None,
    text: TextSpec::Static(&[
        "你把三神兵·机核碎片交还。瓦罗残响那唯一的人类眼睛慢慢阖上，躯壳化作亿万银白色光点，沉入祭坛，沉入大地。",
        "裂缝合拢。银色大地第一次泛起别的颜色——是水汽，是天光，是两百年前就该有的黎明。",
        "李铭：「修正完成——瓦罗·地灵之祖，档案中『叛徒』一词，被我们划掉了。」撤离传送门激活。「三神兵·机核碎片已归还——文明的种子，回到它应属的地方。」",
    ]),
    choices: &[ChoiceDef { label: "（踏入撤离传送门 → 主神空间）", sub: "pt_exit · 结算", cond: None,
        effects: &[Eff::SetFlag("ys_cleared"), Eff::SetFlag("ys_waro_returned")], route: Route::To("ys_16_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_15_ending_fire", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛 · 撤离传送门"),
    mood: "calm", speaker: Some("吴明（录音旁白）"), voice: Some("vo_yinse_wuming"),
    text: TextSpec::Static(&[
        "你反手扣住三神兵·机核碎片，瓦罗残响一滞。李铭：「文明的种子，加一。至于两千名在银色大地上战死的先遣者……他们不叫英雄，他们叫『人』。」",
        "吴明的录音自胸口响起：「踏过银色大地的时候记住：两百年前，有人为了『不再吃人血』启动了升华。别让他们的死，只变成一句传说。」",
    ]),
    choices: &[ChoiceDef { label: "（携带碎片撤离 → 主神空间）", sub: "pt_exit · 结算", cond: None,
        effects: &[Eff::SetFlag("ys_cleared"), Eff::SetFlag("ys_probe_kept")], route: Route::To("ys_17_settle_fire") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "ys_15_ending_venge", bg: Some("yinse_bg.png"), loc: Some("L4 决战祭坛 · 撤离传送门"),
    mood: "danger", speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "你亲手送葬了这团执念。仪式没有和平，也没有私藏——你只是把它从历史里抹去。",
        "李铭：「有时候，让一个文明安静地死，也是一种修正。」",
        "（未洞悉真相的沉默杀伐：无和平/火种结局，「瓦罗之墓」估值偏冷。）",
    ]),
    choices: &[ChoiceDef { label: "（撤离 → 主神空间）", sub: "pt_exit · 结算", cond: None,
        effects: &[Eff::SetFlag("ys_cleared")], route: Route::To("ys_16_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* ---- 结算卡（s_nexus 结算） ---- */
SceneDef {
    id: "ys_16_settle", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "修 正 完 成".into(), good: true,
            body_html: format!(
                "<p>银色大地在你们身后合拢。撤离传送门把你们送回主神空间。</p>\
                 <p style='color:#9a958a'>《银色大地 · 地灵族机界遗迹》副本 · 已完成</p>\
                 <table class='statTable'><tr><td>奖励点数</td><td>{}</td></tr><tr><td>理智</td><td>{}</td></tr></table>\
                 <p style='color:#ffd76a'>支线评级：{}　结算（存活×100 + 支线flag×200）见主神．</p>\
                 <p style='color:#b0c4de'>地灵族的火种，已带出（或归还）。历史修正<span class='crit'>像一场葬礼</span>。</p>",
                st.points, st.san.max(0),
                st.sp_grade.map(|g| format!("{g} 级")).unwrap_or_else(|| "暂无".into())
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: Some("vo_yinse_liming_done"),
        },
    }),
},
SceneDef {
    id: "ys_17_settle_fire", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |st| crate::state::Card {
            title: "文 明 的 种 子，加 一".into(), good: true,
            body_html: format!(
                "<p>你携着三神兵·机核碎片回到主神空间。那团低纬度的冷意，在银白的大地上，被你焐成了火种。</p>\
                 <p style='color:#9a958a'>《银色大地》副本 · 已完成（火种结局：私藏碎片）</p>\
                 <table class='statTable'><tr><td>奖励点数</td><td>{}</td></tr><tr><td>理智</td><td>{}</td></tr></table>\
                 <p style='color:#ffd76a'>持有碎片：可用于兑换『机械动力核心』或留作低纬度副本门禁券（§8.4）。</p>",
                st.points, st.san.max(0)
            ),
            buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
            voice: Some("vo_yinse_liming_done"),
        },
    }),
},
/* ---- 失败 / 死亡档案 ---- */
SceneDef {
    id: "ys_lose_r1", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被机界升华定格", "升华法阵把你锻成机械尸骸")), card: |_st| crate::state::Card {
            title: "被 机 界 升 华 定 格".into(), good: false,
            body_html: r#"<p>升华法阵的银白之光陡然吞没你的身影。你发觉自己正在被「锻」——骨骼开始机械化，皮肤泛起金属光泽。</p>
<p style='color:#ff8a8a'>【死亡档案 · 被机界升华定格】＃0002</p>
<p style='color:#666'>(复活：回主神空间扣 400 点，复活系统接线。本条死亡历史已由记录员修正。)</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
SceneDef {
    id: "ys_lose_r2", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("被低纬度吞没", "墨紫裂隙物质缠裹，理智与躯体一并沉入低纬度")), card: |st| {
            let special = st.flag("ys_waro_truth");
            crate::state::Card {
                title: if special { "被 推 出 裂 缝（幸存）".into() } else { "低 纬 度 吞 没".into() },
                good: false,
                body_html: if special {
                    r#"<p>瓦罗残响看穿了你洞悉真相的眼睛。它没有杀你——用最后一丝意志，把你们整队推出裂缝。</p>
<p style='color:#ff8a8a'>【特殊失败 · 已洞悉真相】瓦罗以残余意志放你们一马：只扣 200 点，全部 flag 保留。</p>"#.to_string()
                } else {
                    r#"<p>墨紫裂隙物质缠上你的脚踝，把你拖进无边界的低纬度。耳边只剩一句不属于任何种族的话。</p>
<p style='color:#ff8a8a'>【死亡档案 · 被低纬度吞没】＃0002</p>
<p style='color:#666'>(复活：回主神空间扣 400 点。本条死亡历史已由记录员修正。)</p>"#.to_string()
                },
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            }
        },
    }),
},
SceneDef {
    id: "ys_lose_common", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("战死于银色大地", "倒在白银荒原")), card: |_st| crate::state::Card {
            title: "战 死 于 白 银 大 地".into(), good: false,
            body_html: r#"<p>你在银色大地倒下。血色在锈白的大地上，很快就会被风蚀成灰。</p>
<p style='color:#ff8a8a'>【死亡档案 · 战死于银色大地】</p>
<p style='color:#666'>(复活：回主神空间扣 400 点。本条死亡历史已由记录员修正。)</p>"#.to_string(),
            buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
            voice: None,
        },
    }),
},
/* ---- 撤离信标（任意层可提前结算） ---- */
SceneDef {
    id: "ys_evac_beacon", bg: Some("yinse_bg.png"), loc: Some("撤离信标 · 提前结算"),
    mood: "cold", speaker: Some("李铭"), voice: None,
    text: TextSpec::Static(&[
        "你抵达撤离信标。李铭的声音传来：「提前撤离将按当前已达成内容评级结算。确定吗？」",
        "（若 BOSS 未击杀即撤退：奖励按已完成支线计。主神信标将你接回主神空间。）",
    ]),
    choices: &[
        ChoiceDef { label: "【激活信标撤离】", sub: "提前结算", cond: Some(cond_has_beacon),
            effects: &[Eff::SetFlag("ys_evac"), Eff::SetFlag("ys_cleared")], route: Route::To("ys_16_settle") },
        ChoiceDef { label: "（继续探索）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("ys_01_drop") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
];

/* =====================================================================
   龙尸坑演出（圣位红线：龙族高圣尸骸只做演出文本，不入战斗）
   ===================================================================== */
fn route_slope(st: &mut GameState) -> String {
    if st.flag("ys_slope_encounter_done") {
        "ys_06_city".to_string()
    } else {
        st.set_flag("ys_slope_encounter_done");
        st.points += 20;
        st.san = (st.san - 6).clamp(0, 100);
        "ys_05_ele1_open_slope".to_string()
    }
}

fn route_taotao(st: &mut GameState) -> String {
    // 用钢棒敲击引开（设计 §6 Act1 选项 B：需理智≥40 判定）
    if st.san >= 40 {
        st.set_flag("ys_save_asang");
        st.set_flag("ys_asang_ally");
        st.points += 200;
        "ys_03_asang_win".to_string()
    } else {
        // 失败：阿桑掉血但仍可救（此处简化为直接进入救人战斗）
        "ys_03_asang_save".to_string()
    }
}

/* ---- （从 YINSE_SCENES 数组内移出，供 Route::Dyn 引用） ---- */
fn route_wrong_order(st: &mut GameState) -> String {
    // 错序即触发：符文电偶战斗（自然惩罚，§10.1 案 A）
    st.set_flag("ys_wrong_order_golem");
    st.san = (st.san - 4).clamp(0, 100);
    "ys_06_golem_fight".to_string()
}
fn route_core(st: &mut GameState) -> String {
    // G4 主门判定：truth 或持机核 → 已通过 ys_core_open 设 flag；若无则需裂缝侧门
    if st.flag("ys_core_open") {
        "ys_12_tomb".to_string()
    } else {
        st.set_flag("ys_entered_side");
        st.san = (st.san - 15).clamp(0, 100);
        "ys_12_tomb_side".to_string()
    }
}
fn route_fight_no_truth(st: &mut GameState) -> String {
    st.set_flag("ys_no_truth_fight");
    st.san = (st.san - 5).clamp(0, 100);
    "ys_13_fight_r1".to_string()
}
fn route_fight_truth(st: &mut GameState) -> String {
    st.set_flag("ys_truth_known_fight");
    "ys_13_fight_r1".to_string()
}
fn route_stop_machine(st: &mut GameState) -> String {
    // 折返 L3 强行关闭升华装置：多 2 场遭遇，归来 BOSS 伤害 -3（以支线 flag 记录）
    if st.flag("ys_waro_machine_off") {
        "ys_13_fight_r1".to_string() // 已关过
    } else {
        st.san = (st.san - 6).clamp(0, 100);
        "ys_12_machine_off".to_string()
    }
}
/// 初始化一段 BOSS 会话（从 ws_waro_r1 的 FightCfg 建 Fight）
fn start_waro_r1(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("ws_waro_r1") {
            st.fight = Some(crate::power::scaled_fight("ws_waro_r1", cfg, st, vec![]));
        }
    }
    "ys_13_round_r1".to_string()
}
fn txt_round_r1(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(380);
    let raged = st.fight.as_ref().map(|f| f.hp <= 160).unwrap_or(false);
    let head = if raged {
        "他身上的机界升华装甲全数展开，银色飞屑漫天——每回合指令阶段都将追加一次『升华脉冲』（dmg 6~10 无视护甲）。"
    } else {
        "升华法阵被他一次次推向更高的完成度——若你已恢复都市供电，可在蓄力轮注入电能打断。"
    };
    format!("<b>机界升华体 · 瓦罗残响</b> · 一形态　HP {hp}/380\n\n{head}")
}
fn route_r1_attack(st: &mut GameState, dmg: i32) -> String {
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    // 一段全局受 waro_machine_off 影响较小；此处做玩家反击
    let p_dmg = rnd_ys(10, 18);
    st.hp = (st.hp - p_dmg).max(0);
    if st.hp <= 0 { return "ys_lose_r1".to_string(); }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        // 一段战胜利 → 转场演出 → 放二段
        st.points += 500;
        return "ys_waR0_cast".to_string();
    }
    "ys_13_round_r1".to_string()
}
/// 初始化二阶段 BOSS（承接一段胜利奖励，重开法阵）
fn start_waro_r2(st: &mut GameState) -> String {
    if let Some(cfg) = crate::scenes::fight_cfg("ws_waro_r2") {
        st.fight = Some(crate::power::scaled_fight("ws_waro_r2", cfg, st, vec![]));
    }
    "ys_14_fight_r2".to_string()
}
fn txt_round_r2(st: &GameState) -> String {
    let hp = st.fight.as_ref().map(|f| f.hp).unwrap_or(300);
    let raged = st.fight.as_ref().map(|f| f.hp <= 120).unwrap_or(false);
    let peace = st.flag("ys_waro_peace");
    let mut note = String::new();
    if raged {
        note.push_str("裂缝涌入银蛇，低纬度凝视携带 San -3/命中。");
    }
    if peace {
        note.push_str("狂暴阈值已降至 100，末日技『天翻地覆』变为可谈判——把碎片还给他。");
    }
    if st.inventory.iter().any(|i| i == "item_jiche") {
        note.push_str("你私藏碎片——每次攻击额外触发低纬度凝视（San -5）。");
    }
    if st.flag("ys_rift_open") {
        note.push_str("裂缝传送仍开启，可用裂缝机关紧急关闭『天翻地覆』。");
    }
    format!("<b>机界升华体 · 瓦罗残响</b> · 二形态　HP {hp}/300\n\n{}", if note.is_empty() { "墨紫裂隙物质在你们脚下盘成一圈圈死结。" } else { &note })
}
fn route_r2_attack(st: &mut GameState, dmg: i32) -> String {
    // 私藏碎片：每次命中携带低维度凝视
    if st.inventory.iter().any(|i| i == "item_jiche") {
        st.san = (st.san - 5).clamp(0, 100);
    }
    if st.fight.as_ref().map(|f| f.hp <= 120).unwrap_or(false) {
        st.san = (st.san - 3).clamp(0, 100);
    }
    if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg.max(0)).max(0); }
    let p_dmg = if st.flag("ys_waro_machine_off") { rnd_ys(19, 35) } else { rnd_ys(22, 38) }; // machine_off 全局 -3
    st.hp = (st.hp - p_dmg).max(0);
    if st.hp <= 0 { return "ys_lose_r2".to_string(); }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        st.points += 700;
        st.set_flag("ys_waro_defeated");
        crate::world::add_item(st, "item_walo_tear");
        st.sp_grade = Some('D');
        if st.flag("ys_waro_peace") {
            // 交还碎片：和平结局（掉落同胜利，碎片消耗）
            return "ys_15_ending_peace".to_string();
        } else if st.inventory.iter().any(|i| i == "item_jiche") {
            return "ys_15_ending_fire".to_string();
        } else {
            return "ys_15_ending_venge".to_string();
        }
    }
    "ys_14_round_r2".to_string()
}
fn route_r2_surrender(st: &mut GameState) -> String {
    // 交还碎片 → 提前结束战斗（和平结局，掉落同胜利）
    st.set_flag("ys_waro_defeated");
    st.points += 700;
    crate::world::add_item(st, "item_walo_tear");
    st.sp_grade = Some('D');
    "ys_15_ending_peace".to_string()
}

/// 本文件场景查询辅助（主线合并查询扩展时可直接使用）
pub fn ys_scene(id: &str) -> Option<&'static SceneDef> {
    YINSE_SCENES.iter().find(|s| s.id == id)
}

/// 查询辅助（主线合并 fight_cfg 扩展时可直接调用）
pub fn ys_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    yinse_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}
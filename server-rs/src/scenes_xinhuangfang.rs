//! 《心慌方》CUBE 副本——全部剧情场景与战斗配置。
//! 机关流求生副本：没有单体守方，而是迷宫本身施予惩罚。陷阱=San 惩罚 + 死亡档案留痕；
//! 门禁=数字对路（flag）；BOSS 无实体（可 1 个幸存的「考验者」HP120 可选战，也可和平交流）。
//! 结局开放：脱出 / 回到原点 / 永远迷失（sp_grade Some('D')）。
//! 钩子：「每个房间都一样，除了死法。」
//!
//! 本文件只导出静态数据（XINHUANGFANG_SCENES / xinhuangfang_figths / 查询辅助），不写入
//! scenes.rs 的 SCENES/FIGHTS 静态表；合并阶段由主线扩展 scenes::scene() / scenes::fight_cfg()。
//! 场景 id 全 `xf_` 前缀；fight id 全 `xf_` 前缀。
//!
//! ★待素材替换清单（新 bg 落地后把 bg= 换新图；当前用现有图占位）：
//!   F1 启动层 bg xf_bg_start   （现用 img_laser.png 占位）
//!   F2 中层    bg xf_bg_mid    （现用 img_redqueen.png 占位）
//!   F3 出口层  bg xf_bg_exit   （现用 img_zhuyuan_book.png 占位）
//!   考验者     bg xf_bg_kanshi （现用 img_laser.png 占位）
//! 敌人立绘复用：mech→铁灰巡卫/编号守望者、zombie→出口徘徊者、hunter→游荡回声/隙间潜伏者；
//! 新美术由主 agent 统一生图替换。

use crate::defs::*;
use crate::state::GameState;
use rand::Rng;

/// 空 effect / choice 惯用静态
static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* =====================================================================
   通用小工具 / 条件谓词
   ===================================================================== */
/// 物品栏是否持有
fn inv(st: &GameState, item: &str) -> bool { st.inventory.iter().any(|i| i == item) }

fn cond_has_num1(st: &GameState) -> bool { st.flag("xf_num_1") }
fn cond_has_num2(st: &GameState) -> bool { st.flag("xf_num_2") }
fn cond_has_num3(st: &GameState) -> bool { st.flag("xf_num_3") }

/* =====================================================================
   战斗配置表（id 全 xf_ 前缀）。
   机关流副本敌人主要是迷宫异兽（遭遇战）；「考验者」为可选战 BOSS。
   ===================================================================== */
fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

fn win_hub(_st: &GameState) -> String { "xf_00".to_string() }
fn win_f2(_st: &GameState) -> String { "xf_20_arrive".to_string() }
fn win_f3(_st: &GameState) -> String { "xf_30_arrive".to_string() }
fn win_kanshi(_st: &GameState) -> String { "xf_41_kanshi_down".to_string() }

pub fn xinhuangfang_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("xf_guard", FightCfg {
            name: "铁灰巡卫", hp: 42, dmg: (6, 10), reward: 40, reward_why: "掰断铁灰巡卫的机械指节",
            intro: "黑暗中一对红灯亮起。铁灰巡卫从不主动找你——它只在你挡了它的路时，缓慢地转身，齿轴发出令人牙酸的绞动。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hub, death: "xf_90_trap_death",
        }),
        ("xf_echo", FightCfg {
            name: "游荡回声", hp: 36, dmg: (5, 9), reward: 35, reward_why: "穿过游荡回声的呜咽",
            intro: "一个透明的、反复呢喃的人形轮廓在走廊尽头晃动。它不是活的，却在学会你走路的声音——你越怕，它就越像你。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_hub, death: "xf_90_trap_death",
        }),
        ("xf_fixer", FightCfg {
            name: "编号守望者", hp: 88, dmg: (9, 15), reward: 70, reward_why: "避开编号守望者的瞪视",
            intro: "一个静止在房间正中的生物，浑身覆满褪色的编号。它不攻击，只是死死盯着每一个想离开的活物——目光所及之处，地面开始松动。",
            rage_at: Some(40), rage_text: "编号守望者被激怒，房间四壁开始向中央合拢！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f2, death: "xf_90_trap_death",
        }),
        ("xf_lurker", FightCfg {
            name: "隙间潜伏者", hp: 60, dmg: (8, 14), reward: 55, reward_why: "甩掉隙间潜伏者",
            intro: "身后传来砖块摩擦的声音——某个东西正从墙缝里拼凑自己。等你回头，缝隙已经空无一物，只在更近的角落，多了一双半睁的眼。",
            rage_at: None, rage_text: "", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f2, death: "xf_90_trap_death",
        }),
        ("xf_wanderer", FightCfg {
            name: "出口徘徊者", hp: 72, dmg: (10, 16), reward: 65, reward_why: "停下出口徘徊者的踱步",
            intro: "它守在通往更深处的地方，来回踱步，数着一生都解不开的算术。看见你时它停下来，像是在问：你也解不出来，对么？",
            rage_at: Some(34), rage_text: "徘徊者发出低吼，出口的光在一瞬间熄灭又亮起！", on_rage: rage_none,
            finisher_if: |_, _| false, finisher_name: |_| String::new(), finisher_desc: |_| String::new(),
            win: win_f3, death: "xf_90_trap_death",
        }),
        ("xf_kanshi", FightCfg {
            name: "幸存的考验者", hp: 120, dmg: (12, 18), reward: 160, reward_why: "制服幸存的考验者，从他手中夺走完整的数字批注",
            intro: "一个浑身浴血的活人靠墙坐着，瞳孔涣散却仍握着半截烧焦的笔。听见你走近，他猛地抬头：<em>「你是想逃出去，还是想死得体面一点？」</em>",
            rage_at: Some(55), rage_text: "考验者嘶吼着扑来，笔尖在他手中闪着铁灰的冷光——他早已分不清你是敌人，还是记忆里某个没能带出去的人。", on_rage: rage_none,
            finisher_if: |st, _| st.fight.as_ref().map(|f| f.raged).unwrap_or(false) && inv(st, "it_xf_num_note"),
            finisher_name: |_| "出示残缺的数字批注".to_string(),
            finisher_desc: |_| "你把那几页揉皱的数字批注举到他眼前。他浑身的颤抖慢慢平复，笔从手中滑落——原来他不是要杀你，他只是要把唯一记得的东西交出去。".to_string(),
            win: win_kanshi, death: "xf_90_trap_death",
        }),
    ]
}

/// 查询辅助
pub fn xf_fight_cfg(id: &str) -> Option<&'static FightCfg> {
    xinhuangfang_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v)
}

/* =====================================================================
   幸存的考验者（选择驱动遭遇；可和平交流或交手）
   血量存 st.fight（xf_40 的 Route::Dyn 初始化，引用 xf_kanshi 的 FightCfg）。
   ===================================================================== */
fn start_kanshi(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("xf_kanshi") {
            st.fight = Some(crate::power::scaled_fight("xf_kanshi", cfg, st, vec![cfg.intro.to_string()]));
            st.set_flag("xf_kanshi_start");
        }
    }
    "xf_42_kanshi_round".to_string()
}

fn kanshi_win(st: &mut GameState) -> String {
    crate::world::add_item(st, "it_xf_num_note");
    st.points += 160;
    st.set_flag("xf_kanshi_down");
    st.sp_grade = Some('D');
    st.set_flag("xf_nums_done"); // 从他手中夺得的完整批注补全数字对路
    "xf_41_kanshi_down".to_string()
}

fn kanshi_peace(st: &mut GameState) -> String {
    crate::world::add_item(st, "it_xf_num_note");
    st.points += 120;
    st.set_flag("xf_kanshi_peace");
    st.set_flag("xf_nums_done");
    "xf_43_kanshi_peace".to_string()
}

fn kanshi_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 {
        if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); }
    }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return kanshi_win(st); }
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    if !raged && st.fight.as_ref().map(|f| f.hp <= 55).unwrap_or(false) {
        if let Some(f) = st.fight.as_mut() { f.raged = true; }
    }
    let raged = st.fight.as_ref().map(|f| f.raged).unwrap_or(false);
    let raw = if raged { rng(18, 24) } else { rng(12, 18) };
    if !guard {
        st.hp = (st.hp - raw).max(0);
    }
    if st.hp <= 0 { return "xf_90_trap_death".to_string(); }
    "xf_42_kanshi_round".to_string()
}

fn rng(a: i32, b: i32) -> i32 { rand::thread_rng().gen_range(a..=b) }

/* =====================================================================
   剧情场景（id 全 xf_ 前缀）
   ===================================================================== */
pub static XINHUANGFANG_SCENES: &[SceneDef] = &[

    /* ================= 序 · 醒来 ================= */
    SceneDef {
        id: "xf_00", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 铁灰房间"),
        mood: "danger", speaker: Some("旁白"), voice: Some("vo_xf_open"),
        text: TextSpec::Static(&[
            "冰凉的地板把你硌醒。你睁开眼，头顶一盏同样铁灰的灯，房间六面光滑，只有一面开着一道门。",
            "你试着回忆自己是怎么进来的——一片空白。门外是另一间一模一样的房间，只是墙上多了一枚你绕不过去的画号。",
            "<em>「每个房间都一样，除了死法。」</em>",
        ]),
        choices: &[
            ChoiceDef { label: "环顾这间房间", sub: "打量牢笼的边界", cond: None, effects: &NO_EFF, route: Route::To("xf_01_first") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_01_first", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 铁灰房间"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["房间中央横着一具不会动的冰冷躯体——某个先你一步走进这里的人。他衣兜里露出一角揉皱的纸，纸边被血浸透，字迹仍在。"]),
        choices: &[
            ChoiceDef { label: "取走纸上的数字批注", sub: "Item it_xf_num_note · 残缺批注", cond: None,
                effects: &[Eff::AddItem("it_xf_num_note")], route: Route::To("xf_10_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F1 启动层 · hub ================= */
    SceneDef {
        id: "xf_10_arrive", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 洗牌走廊"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("xf_f1_probe") {
                "走廊尽头偶尔响起砖块摩擦的洗牌声——房间在你身后悄无声息地换了位。墙上画号是唯一的坐标，可你并不确定它们在骗你多少。".to_string()
            } else {
                "启动层的走廊延伸向肉眼可见的错觉尽头。每扇门通向的都不是直线，而是另一个与你脚下这间几乎相同的房间。你决定先摸清这里。".to_string()
            }
        }),
        choices: &[
            ChoiceDef { label: "铁灰房间壹", sub: "调查点 · 画号", cond: None, effects: &NO_EFF, route: Route::To("xf_01_room1") },
            ChoiceDef { label: "铁灰房间贰", sub: "调查点 · 机关", cond: None, effects: &NO_EFF, route: Route::To("xf_02_room2") },
            ChoiceDef { label: "铁灰房间叁", sub: "调查点 · 死法", cond: None, effects: &NO_EFF, route: Route::To("xf_03_room3") },
            ChoiceDef { label: "墙上画号", sub: "数字线索（一）", cond: None, effects: &NO_EFF, route: Route::To("xf_04_draw1") },
            ChoiceDef { label: "夹层记号", sub: "机关侧记", cond: None, effects: &NO_EFF, route: Route::To("xf_05_draw2") },
            ChoiceDef { label: "褪色序列", sub: "数字线索（二）", cond: None, effects: &NO_EFF, route: Route::To("xf_06_draw3") },
            ChoiceDef { label: "血滴轨迹", sub: "前人的路", cond: None, effects: &NO_EFF, route: Route::To("xf_07_floor1") },
            ChoiceDef { label: "空床位", sub: "数数他们剩几个", cond: None, effects: &NO_EFF, route: Route::To("xf_08_floor2") },
            ChoiceDef { label: "揉皱纸条", sub: "数字线索（三）", cond: None, effects: &NO_EFF, route: Route::To("xf_09_floor3") },
            ChoiceDef { label: "与铁灰巡卫擦肩", sub: "异兽 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("xf_10_fight") },
            ChoiceDef { label: "走向更深处", sub: "推到画号门甲 → 中层", cond: Some(cond_has_num1), effects: &NO_EFF, route: Route::To("xf_12_gate") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ---- F1 调查点 ---- */
    SceneDef {
        id: "xf_01_room1", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 铁灰房间壹"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["这间房的画号是 <em>03</em>。墙角有一道很浅的刻痕，像有人用指甲反复写过：『07 · 14』。你不确定这是答案，还是又一个诱饵。"]),
        choices: &[ChoiceDef { label: "记下 07 · 14", sub: "数字线索 · 机关追责", cond: None,
            effects: &[Eff::SetFlag("xf_num_1"), Eff::Points(5)], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_02_room2", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 铁灰房间贰 · 致命机关"),
        mood: "danger", speaker: None, voice: Some("vo_xf_trap"),
        text: TextSpec::Static(&["脚步声一重，地面中央的薄板微微一沉——你触到机关了。金属尖刺从四壁缝隙中缓慢伸出一寸又缩回，像在丈量你逃开的速度。（致命机关）"]),
        choices: &[
            ChoiceDef { label: "屏息退开", sub: "躲过 · San 撕裂", cond: None,
                effects: &[Eff::San(-15), Eff::SetFlag("xf_trap_2")], route: Route::To("xf_10_arrive") },
            ChoiceDef { label: "硬闯过去", sub: "受伤 · 可能归档", cond: None,
                effects: &[Eff::Hurt(22, "xf_90_trap_death")], route: Route::To("xf_10_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_03_room3", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 铁灰房间叁 · 死法"),
        mood: "danger", speaker: None, voice: Some("vo_xf_trap"),
        text: TextSpec::Static(&["房间一侧的墙上贴着一块早被磨出深痕的告示：<em>『防止移动天花板压人 · 需按下号键』</em>。你意识到，这里每一个死法都被精心设计成「可以躲开」，前提是你读得懂它们。"]),
        choices: &[
            ChoiceDef { label: "环视告示良久", sub: "看清死法 · 心有余悸", cond: None,
                effects: &[Eff::San(-10), Eff::SetFlag("xf_probe_3")], route: Route::To("xf_10_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_04_draw1", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 墙上画号（一）"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["画号 <em>05</em>。你踮脚看，编号下方被人用焦炭补了一行小字：『质数之和，二位——别信他的减法』。不知道『他』是谁，但这条笔记救过某个人。"]),
        choices: &[ChoiceDef { label: "抄下这条笔记", sub: "数字线索 · 门禁铺垫", cond: None,
            effects: &[Eff::SetFlag("xf_num_2"), Eff::Points(5)], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_05_draw2", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 夹层记号"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["夹层里塞着一只断了一截的铅笔和一张更旧的图。图上有三条并行的路径，每条尽头都画着一个叉——只有最底下那条，末尾圈着一个圈。你把它折好，和批注放在一处。"]),
        choices: &[ChoiceDef { label: "收好图纸", sub: "Item it_xf_num_note 内容补全", cond: None,
            effects: &[Eff::Points(10), Eff::SetFlag("xf_f1_probe")], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_06_draw3", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 褪色序列"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["墙上一列几乎要消失的序列：<em>0,1,1,2,3,5,…</em>。第七个数是多少？你盯着看，忽然明白这不是装饰——它是指向出路的方向标，只在肯数数的人面前显形。"]),
        choices: &[ChoiceDef { label: "推算出数列第七项", sub: "记下 13 · 触发机关警觉", cond: None,
            effects: &[Eff::San(-5), Eff::SetFlag("xf_num_3")], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_07_floor1", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 血滴轨迹"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["暗色的血点断断续续地拖向一个方向。它不是逃跑的痕迹，而是倒着拖出去的——有人被往回拽过。你顺着它反方向走，来到更深那扇门前。"]),
        choices: &[ChoiceDef { label: "顺着反方向探索", sub: "发现通往中层方向", cond: None,
            effects: &[Eff::Points(10), Eff::SetFlag("xf_f1_probe")], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_08_floor2", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 空床位"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["墙根整整齐齐码着一排空床位——像是给谁准备的，又像是从来没人睡过。你在其中一张床垫下摸到一粒硬物：一枚沾灰的旧钥匙。"]),
        choices: &[ChoiceDef { label: "收下旧钥匙", sub: "Item it_xf_key · 备用开门", cond: None,
            effects: &[Eff::AddItem("it_xf_key"), Eff::Points(5)], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_09_floor3", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 揉皱纸条"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["纸条上重复着一句话，字迹从工整写到歪斜：<em>『房间会换，数字不会作假』『房间会换，数字不会』『房间会换』</em>——最后一行只写到一半。你莫名觉得冷。"]),
        choices: &[ChoiceDef { label: "把纸条收进批注", sub: "数字对路的最后一块拼图", cond: None,
            effects: &[Eff::San(-10), Eff::SetFlag("xf_f1_probe")], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_10_fight", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 走廊 · 擦肩"),
        mood: "danger", speaker: None, voice: Some("vo_xf_guard"),
        text: TextSpec::Static(&["铁灰巡卫的红灯扫过，你贴着墙一动不动。它没有停留，机械地朝更深处走去，留下一串碾过石砖的响声。（遭遇）"]),
        choices: &[], fight_id: Some("xf_guard"), video: None, cine_label: None, overlay: None,
    },

    /* ---- F1 门禁 · 画号门甲 ---- */
    SceneDef {
        id: "xf_12_gate", bg: Some("xinhuangfang_bg.png"), loc: Some("启动层 · 画号门甲（G1）"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["当你凑齐 07 · 14 与那行焦炭小字的批注，门闸的纹路恰好嵌合成一道间隙。中层在门缝后露出一角更深的铁灰——那是你不确定该不该踏进的地方。"]),
        choices: &[ChoiceDef { label: "（推门踏入中层）", sub: "pt_xf_1 单向 · 进 F2", cond: None,
            effects: &NO_EFF, route: Route::To("xf_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F2 中层 · hub ================= */
    SceneDef {
        id: "xf_20_arrive", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 编号回廊"),
        mood: "cold", speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&["中层的走廊比启动层更密，每个房间墙角都钉着一枚编号。数字不再只是线索，它们开始像评语一样，给每个经过的活物打分。这里住着一个还没断气的住客——幸存的考验者。"]),
        choices: &[
            ChoiceDef { label: "中心枢纽", sub: "生存者营地 · 调查", cond: None, effects: &NO_EFF, route: Route::To("xf_20_center") },
            ChoiceDef { label: "回廊暗格", sub: "数字线索补全", cond: None, effects: &NO_EFF, route: Route::To("xf_21_right") },
            ChoiceDef { label: "幸存者留言", sub: "前人的嘱托", cond: None, effects: &NO_EFF, route: Route::To("xf_22_note") },
            ChoiceDef { label: "与编号守望者周旋", sub: "异兽 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("xf_20_fight") },
            ChoiceDef { label: "深入编号回廊", sub: "推到画号门乙 → 出口层", cond: Some(cond_has_num2), effects: &NO_EFF, route: Route::To("xf_22_gate") },
            ChoiceDef { label: "回头折返启动层", sub: "回到原点 · 重新数一次", cond: None, effects: &NO_EFF, route: Route::To("xf_20_back") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_20_center", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 中心枢纽 · 生存者营地"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["几块门板搭成的掩体后，散落着罐头空壳与半截蜡烛。有人在这里活了不短的时间——直到他决定把所有数字交给下一个来者。你在营地角落摸到一支还能用的手电。"]),
        choices: &[ChoiceDef { label: "拿起手电", sub: "Item it_xf_flashlight · 照亮前路", cond: None,
            effects: &[Eff::AddItem("it_xf_flashlight"), Eff::Points(10)], route: Route::To("xf_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_21_right", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 回廊暗格"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["暗格里的数字批注与你在启动层读到的恰好互补。合上它，门禁上的『质数之和』终于有了活路——介乎 9 与 17 的质数只有一对，而门只要那一个答案。"]),
        choices: &[ChoiceDef { label: "补全批注 · 数字对路（二）", sub: "xf_num_2 已足 · 前路开", cond: None,
            effects: &[Eff::SetFlag("xf_num_2"), Eff::Points(10)], route: Route::To("xf_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_22_note", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 幸存者留言"),
        mood: "cold", speaker: Some("留言"), voice: None,
        text: TextSpec::Static(&["留言用烧焦的笔迹写着：<em>『别信墙上的死因，信夹层的地图。出口需要三组数字，最后那组在出口层——前提是你别在见到它之前疯掉。』</em>"]),
        choices: &[ChoiceDef { label: "默记嘱托", sub: "心理锚点 · 数字线索（三）", cond: None,
            effects: &[Eff::SetFlag("xf_num_3"), Eff::Points(5)], route: Route::To("xf_20_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_20_fight", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 编号回廊 · 交锋"),
        mood: "danger", speaker: None, voice: Some("vo_xf_guard"),
        text: TextSpec::Static(&["编号守望者的目光落定在你身上，四壁开始极缓慢地松动。你没有退路——只能从它身边夺路而过。（遭遇）"]),
        choices: &[], fight_id: Some("xf_fixer"), video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_22_gate", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 画号门乙（G2）"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Static(&["当第二组数字补全，门乙的锁舌无声收回。出口层的空气从门缝里渗进来，带着一丝诡异的光。你深呼吸，推开它。"]),
        choices: &[ChoiceDef { label: "（踏入出口层）", sub: "pt_xf_2 单向 · 进 F3", cond: None,
            effects: &NO_EFF, route: Route::To("xf_30_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_20_back", bg: Some("xinhuangfang_bg.png"), loc: Some("中层 · 回廊返程"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["你折返启动层，把已经摸清的走廊又走了一遍。房间在你身后继续洗牌，原有的记号渐渐失去意义——你开始怀疑，是不是从一开始就没有真正的『更深处』。"]),
        choices: &[ChoiceDef { label: "回到启动层洗牌走廊", sub: "回到原点 · 重新数一次", cond: None,
            effects: &[Eff::San(-5)], route: Route::To("xf_10_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= F3 出口层 · hub ================= */
    SceneDef {
        id: "xf_30_arrive", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 出口之室"),
        mood: "danger", speaker: Some("旁白"), voice: Some("vo_xf_exit"),
        text: TextSpec::Static(&["尽头那道门亮着不容质疑的白光。可你很快就发现，自己在原地绕了三圈才真正接近它——数字若对不齐，这道光只是迷宫给自己照的灯。角落里有个人影。"]),
        choices: &[
            ChoiceDef { label: "出口之室", sub: "调查尽头的那道门", cond: None, effects: &NO_EFF, route: Route::To("xf_30_room") },
            ChoiceDef { label: "中心暗房", sub: "检查门侧编号", cond: None, effects: &NO_EFF, route: Route::To("xf_31_mid") },
            ChoiceDef { label: "编号墙", sub: "数字线索（三）拼图", cond: None, effects: &NO_EFF, route: Route::To("xf_32_num") },
            ChoiceDef { label: "尽头的门", sub: "尝试推开出口之门 G3", cond: Some(cond_has_num3), effects: &NO_EFF, route: Route::To("xf_33_gate") },
            ChoiceDef { label: "靠近幸存的考验者", sub: "角色 · 交流或交手", cond: None, effects: &NO_EFF, route: Route::To("xf_40_kanshi") },
            ChoiceDef { label: "与出口徘徊者交锋", sub: "异兽 · 遭遇", cond: None, effects: &NO_EFF, route: Route::To("xf_30_fight") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_30_room", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 出口之室"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["白光来自门缝里一片刺目的空旷。你没有立刻推门——因为门侧的铭牌上，被人用指甲划掉了一行字，又写了另一行上去。它似乎也在犹豫。"]),
        choices: &[ChoiceDef { label: "记下门侧铭牌", sub: "出口线索 · 渐明", cond: None,
            effects: &[Eff::Points(10), Eff::SetFlag("xf_exit_probe")], route: Route::To("xf_30_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_31_mid", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 中心暗房"),
        mood: "danger", speaker: None, voice: Some("vo_xf_trap"),
        text: TextSpec::Static(&["你弯腰探进暗房，脚下的感应板一沉。天花板上方传来迟缓的液压声——这间不是出口，是另一道机关。你屏住呼吸，等它缩回去。"]),
        choices: &[
            ChoiceDef { label: "屏息等液压退去", sub: "躲过 · San 撕裂", cond: None,
                effects: &[Eff::San(-12), Eff::SetFlag("xf_trap_mid")], route: Route::To("xf_30_arrive") },
            ChoiceDef { label: "抢在液压落下前滚出", sub: "受伤 · 可能归档", cond: None,
                effects: &[Eff::Hurt(20, "xf_90_trap_death")], route: Route::To("xf_30_arrive") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_32_num", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 编号墙"),
        mood: "cold", speaker: None, voice: None,
        text: TextSpec::Static(&["编号墙上钉着一排完整的数字，唯独离开口最近的那枚被人摘下带走。你想起那条留言——第三组数字在这里。你把它从墙后摸出来，握进手心。"]),
        choices: &[ChoiceDef { label: "取得第三组数字", sub: "数字对路（三） · 出口之门将开", cond: None,
            effects: &[Eff::SetFlag("xf_num_3"), Eff::Points(10)], route: Route::To("xf_30_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_30_fight", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 必经交锋"),
        mood: "danger", speaker: None, voice: Some("vo_xf_wanderer"),
        text: TextSpec::Static(&["出口徘徊者停下踱步，挡在你与白光之间。它似乎只想要一个你也没有的答案——可它挡了路，就得让开。（遭遇）"]),
        choices: &[], fight_id: Some("xf_wanderer"), video: None, cine_label: None, overlay: None,
    },

    /* ---- F3 出口之门 ---- */
    SceneDef {
        id: "xf_33_gate", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 出口之门（G3）"),
        mood: "mystery", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            if st.flag("xf_nums_done") {
                "三组数字同时嵌入门缝，白光轰然漫过头顶。你被冲进一片刺目的空旷里——那也许就是出口，也许只是另一间更大更亮的房间。".to_string()
            } else {
                format!("门上的字被磨平，只剩你手里推演出的数字：<em>01 · 01 · 13</em>。你需要用一路上凑齐的三组数字去说服它（当前集齐：{}{}{}）。",
                    if st.flag("xf_num_1") { "壹" } else { "·" },
                    if st.flag("xf_num_2") { "贰" } else { "·" },
                    if st.flag("xf_num_3") { "叁" } else { "·" })
            }
        }),
        choices: &[
            ChoiceDef { label: "嵌入三组数字", sub: "数字对路 · 出口之门", cond: Some(cond_has_num3), effects: &NO_EFF, route: Route::To("xf_34_open") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_34_open", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 白光之后"),
        mood: "awe", speaker: Some("旁白"), voice: Some("vo_xf_exit_open"),
        text: TextSpec::Static(&["白光漫过你的一瞬，身后的洗牌声停住了。不是因为终结，而是因为你终于站在了这局算术的所答——门之后没有仪器，只有更亮的一片空旷，像某个兑现的承诺。"]),
        choices: &[ChoiceDef { label: "（迈出最后一步）", sub: "结局 · 脱出", cond: None,
            effects: &NO_EFF, route: Route::Dyn(xf_route_exit) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 考验者（可选战/和平） ================= */
    SceneDef {
        id: "xf_40_kanshi", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 角落 · 幸存者"),
        mood: "danger", speaker: Some("幸存的考验者"), voice: Some("vo_xf_kanshi"),
        text: TextSpec::Static(&[
            "他靠在门侧，浑浊的眼睛透过乱发看着你。<em>「你是想逃出去，还是想死得体面一点？」</em>他问，语气平静得不像话。",
            "你看到他怀里那叠完整的数字批注——只要你出得去，他或许愿意把『正确的那条路』告诉你，也可能，什么都不说。",
        ]),
        choices: &[
            ChoiceDef { label: "试着与他交谈", sub: "和平 · 把批注交给下一个来者", cond: None, effects: &NO_EFF, route: Route::To("xf_41_kanshi_peace") },
            ChoiceDef { label: "逼他交出数字", sub: "可选战 · 交手", cond: None, effects: &NO_EFF, route: Route::Dyn(start_kanshi) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_41_kanshi_peace", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 幸存者 · 交托"),
        mood: "calm", speaker: Some("幸存的考验者"), voice: None,
        text: TextSpec::Static(&["他沉默很久，然后从怀里取出那叠批注，放在地上推向你。<em>「我数到第三组就停不下来了。」</em>他扯出一个难看的笑，<em>「你帮我把它数完吧——如果门后面真是出口。」</em>"]),
        choices: &[ChoiceDef { label: "收下完整批注", sub: "Item it_xf_num_note · 补全数字对路", cond: None,
            effects: &NO_EFF, route: Route::Dyn(kanshi_peace) }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_41_kanshi_down", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 幸存者 · 制服"),
        mood: "cold", speaker: None, voice: Some("vo_xf_kanshi_down"),
        text: TextSpec::Static(&["你制服了他，从他颤抖的手里夺下完整的数字批注。他瘫坐在地，反而笑了。<em>「对……拿走它，逃出去。替我数完那组数。」</em>他把最后一枚编号墙上的数字也塞给你，闭上眼。"]),
        choices: &[ChoiceDef { label: "拾起完整批注与数字", sub: "Item it_xf_num_note · 数字对路完备", cond: None,
            effects: &NO_EFF, route: Route::To("xf_30_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    /* 考验者战斗回合（选择驱动） */
    SceneDef {
        id: "xf_42_kanshi_round", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 与考验者交手"),
        mood: "danger", speaker: None, voice: None,
        text: TextSpec::Dyn(|st| {
            format!("幸存的考验者 剩余 {} / 120 血，你 HP {}。他握着一截烧焦的笔，像握着一把随时会落下判决的刀。",
                st.fight.as_ref().map(|f| f.hp.max(0)).unwrap_or(0), st.hp)
        }),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤害 · 逼他退让", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| kanshi_act(st, rng(30, 40), false)) },
            ChoiceDef { label: "防御", sub: "本回合免伤", cond: None,
                effects: &NO_EFF, route: Route::Dyn(|st| kanshi_act(st, 0, true)) },
            ChoiceDef { label: "把残缺批注还给他", sub: "出示批注 · 收手和解（需批注）", cond: Some(cond_has_num1),
                effects: &NO_EFF, route: Route::Dyn(|st| kanshi_act(st, 0, false) ) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_43_kanshi_peace", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 幸存者 · 让路"),
        mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&["他没有再说话，只把门侧的编号墙往你这边推了推，露出那枚被他藏起的数字。你拾起它，他替你阖上门，像送走最后一个记得他的人。"]),
        choices: &[ChoiceDef { label: "（走向出口之门）", sub: "数字对路完备 · 可开门", cond: None,
            effects: &NO_EFF, route: Route::To("xf_30_arrive") }],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 结局（开放：脱出 / 回到原点 / 永远迷失） ================= */
    SceneDef {
        id: "xf_38_ending", bg: Some("xinhuangfang_bg.png"), loc: Some("出口层 · 白光尽头 · 抉择"),
        mood: "mystery", speaker: Some("旁白"), voice: Some("vo_xf_ending"),
        text: TextSpec::Static(&[
            "白光尽头的空旷在你脚下铺开。你终于站在了这段算术的尽头——但答案是否值得去对，只有你自己知道。",
            "你可以迈出去，也可以转身，把这一整个立方体连同那些数字一起，留在身后。",
        ]),
        choices: &[
            ChoiceDef { label: "迈出白光（脱出）", sub: "把握住的数字带出去", cond: None,
                effects: &[Eff::SetFlag("xf_end_escape"), Eff::Points(120), Eff::SetFlag("xf_ending_done")], route: Route::To("xf_50_card") },
            ChoiceDef { label: "回到原点（返身）", sub: "从 F1 启动层重新开始", cond: None,
                effects: &[Eff::SetFlag("xf_end_return"), Eff::San(-10), Eff::SetFlag("xf_ending_done")], route: Route::Dyn(xf_route_return) },
            ChoiceDef { label: "留在白光里（永远迷失）", sub: "不再数了", cond: None,
                effects: &[Eff::SetFlag("xf_end_lost"), Eff::Points(60), Eff::SetFlag("xf_ending_done")], route: Route::To("xf_51_lost") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "xf_50_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: Some("vo_xf_settle"), death: None,
            card: |st| crate::state::Card {
                title: "心 慌 方 · 三 组 数 字".into(), good: true,
                body_html: format!(
                    "<p>你带着三组数字走出白光，身后那座铁灰的立方体在视野里缩成一点。没有人鼓掌，其实也没有人真的相信你数完了——但你出去了。</p>\
                     <p style='color:#9a958a'>「每个房间都一样，除了死法。」可你把房门，掰成了出口。</p>\
                     <table class='statTable'>\
                     <tr><td>存活点数</td><td>{}</td></tr>\
                     <tr><td>探界评级</td><td style='color:#ffd76a'>D 级（数到尽头的出逃者）</td></tr>\
                     </table>",
                    st.points
                ),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "xf_51_lost", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("心慌方 · 永远迷失者", "在一道永远推不开的白光前停了下来")),
            card: |st| crate::state::Card {
                title: "永 远 迷 失".into(), good: false,
                body_html: format!(
                    r#"<p>你停在白光前，不再往前走，也不再回头。房间在你身后继续洗牌，一轮又一轮，直到你分不清自己是第几号。</p>
<p style='color:#ff8a8a'>【死亡档案 · 心慌方永远迷失者】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。积分仍计 {}。）</p>"#,
                    st.points
                ),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 死亡档案（机关归档） ================= */
    SceneDef {
        id: "xf_90_trap_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("心慌方 · 机关下葬者", "死在某个铁灰房间的致命机关之下")),
            card: |st| crate::state::Card {
                title: "某 个 死 法".into(), good: false,
                body_html: format!(
                    r#"<p>这一间房间的机关，恰好撞上了你。立方体不会因为少一个活物而停下洗牌——它只会把这一幕，换成下一间一模一样的房间。</p>
<p style='color:#ff8a8a'>【死亡档案 · 心慌方机关下葬者】</p>
<p style='color:#666'>（复活：回主神空间扣 300 点，由主线复活系统接线。积分仍计 {}。）</p>"#,
                    st.points
                ),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

/* =====================================================================
   Route::Dyn 路由函数
   ===================================================================== */
/// 脱出结算：写 sp_grade D → 卡片
fn xf_route_exit(st: &mut GameState) -> String {
    st.set_flag("xf_end_escape");
    st.set_flag("xf_ending_done");
    st.points += 120;
    st.sp_grade = Some('D');
    "xf_50_card".to_string()
}

/// 回到原点：绕回 F1 启动层 hub（不置结局 flag，可继续探索）
fn xf_route_return(st: &mut GameState) -> String {
    st.points += 5;
    "xf_10_arrive".to_string()
}
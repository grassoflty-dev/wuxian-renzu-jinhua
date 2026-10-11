//! 剧本内容：全部场景与战斗配置（移植自原 JS 版 scenes.js）
#![allow(dead_code)]
use crate::defs::*;
use crate::state::{Card, GameState, Weapon};

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

/* ================= 激光判定 ================= *//// 规则：累计第 3 次失误才死亡；q1/q2 各自的失败场景给一次喘息，q3 自带一次重试
fn laser_judge(st: &mut GameState, answer: &'static str, next: &'static str, fail: &'static str) -> String {
    let correct = match st.scene_id.as_str() {
        "s_laser_q1" => "jump",
        "s_laser_q2" => "slide",
        _ => "beam",
    };
    if answer == correct || (st.scene_id == "s_laser_q1" && answer == "back") {
        return next.into();
    }
    if st.laser_fails >= 2 {
        st.laser_fails += 1;
        return "e_laser".into();
    }
    st.laser_fails += 1;
    fail.into()
}
fn route_lq1_jump(st: &mut GameState) -> String { laser_judge(st, "jump", "s_laser_q2", "s_fail_q1") }
fn route_lq1_duck(st: &mut GameState) -> String { laser_judge(st, "duck", "s_laser_q2", "s_fail_q1") }
fn route_lq1_back(st: &mut GameState) -> String { laser_judge(st, "back", "s_laser_q2", "s_fail_q1") }
fn route_lq2_slide(st: &mut GameState) -> String { laser_judge(st, "slide", "s_laser_q3", "s_fail_q2") }
fn route_lq2_hang(st: &mut GameState) -> String { laser_judge(st, "hang", "s_laser_q3", "s_fail_q2") }
fn route_lq2_corner(st: &mut GameState) -> String { laser_judge(st, "corner", "s_laser_q3", "s_fail_q2") }
fn route_lq3_beam(st: &mut GameState) -> String { laser_judge(st, "beam", "s_laser_end", "s_fail_q3") }
fn route_lq3_dash(st: &mut GameState) -> String { laser_judge(st, "dash", "s_laser_end", "s_fail_q3") }
fn route_lq3_flee(st: &mut GameState) -> String { laser_judge(st, "flee", "s_laser_end", "s_fail_q3") }

fn cond_has_adrenaline(st: &GameState) -> bool { st.flag("adrenaline") }
/// 保险柜正确密码需先获得线索（容器编号 H-00-07）
fn cond_vault_ok(st: &GameState) -> bool { st.flag("vault_hint") }

/// 跨调查点耦合·消毒事故真相链：
/// 集齐「列车运行日志 + 消毒终端通知 + 药品柜值班表」三份旁证，才能在主控终端调阅《消毒执行记录》
fn cond_decon_truth(st: &GameState) -> bool {
    !st.flag("decon_truth")
        && st.map_objs.get("p_train_console").copied().unwrap_or(false)
        && st.map_objs.get("p_decon_terminal").copied().unwrap_or(false)
        && st.map_objs.get("p_med_cabinet").copied().unwrap_or(false)
}

/// 跨调查点耦合·冷却回路联动：修好冷却阀组（cooling_done）后，服务器阵列才能读出稳定散热数据
fn cond_server_cooling(st: &GameState) -> bool {
    st.flag("cooling_done") && !st.flag("server_cooling")
}

/// 世界调查点尚未翻检（门控一次性奖励选项；map_objs[point_id] 由 Eff::MarkPoint 置位）
macro_rules! point_undone {
    ($pid:literal) => {
        (|st: &GameState| !st.map_objs.get($pid).copied().unwrap_or(false)) as CondFn
    };
}
fn weapon_name(st: &GameState) -> String {
    st.weapon.map(|w| w.name().to_string()).unwrap_or_else(|| "武器".into())
}

fn txt_laser_q1(st: &GameState) -> String {
    let head = if st.flag("B2") {
        "你的瞳孔骤缩——天花板暗格缩开的方式、导轨灼痕的高度……<em>你在结构图前推演过这一幕！低位扫掠层！</em>\n\n一道蓝色光刃贴着地面<em>水平横扫而来</em>——"
    } else {
        "天花板暗格缩开，一道蓝色光刃贴着地面<em>水平横扫而来</em>——快得几乎看不见！"
    };
    format!("{head}\n\n<em>一瞬间，你该怎么做？</em>")
}
fn txt_boss(st: &GameState) -> String {
    let hint = if st.gene_lock_used { String::new() } else { "\n\n（若体力濒危且理智尚存，或将触发某种觉醒……）".to_string() };
    format!("它落地时砸出一个凹坑，肌肉纤维如活蛇般蠕动。蕾恩举枪掩护，卡普兰在远处大喊着为你指示它的动向——\n\n<em>生死，在此一战。</em>{hint}")
}

/* ================= 战斗配置 ================= */
fn zombie_rage(_st: &mut GameState, _log: &mut Vec<String>) {}
fn horde_on_rage(_st: &mut GameState, _log: &mut Vec<String>) {}
fn licker_on_rage(st: &mut GameState, log: &mut Vec<String>) {
    match st.weapon {
        Some(Weapon::Gun) => {
            st.ammo += 4;
            log.push("卡普兰从掩体后扔来一个备用弹匣！<span class='hit'>（弹药 +4）</span>".into());
        }
        Some(Weapon::Axe) => log.push("蕾恩的点射逼得它侧身一滞——<span class='hit'>机会！</span>".into()),
        _ => log.push("它在管道间借力反弹，攻势愈发刁钻！".into()),
    }
}
fn fin_if_licker(_st: &GameState, ehp: i32) -> bool { ehp <= 28 }
fn fin_name(st: &GameState) -> String {
    if st.weapon == Some(Weapon::Axe) { "斧劈头颅".into() } else { "致命一击".into() }
}
fn fin_desc(st: &GameState) -> String {
    if st.weapon == Some(Weapon::Axe) {
        "你踩着它扑击的势头腾身跃起，消防斧带着全身的重量与不甘、恐惧、愤怒，狠狠噼进那裸露的大脑深处——直到斧刃穿透，钉进水泥地。它抽搐着，不动了。".into()
    } else {
        "你瞅准它长舌回收的刹那欺身而上，将全部力量灌进最后一击——直贯那暴露在外的脑组织深处。巨大的身躯轰然倒塌。".into()
    }
}

pub static FIGHTS: &[(&str, FightCfg)] = &[
    ("zombie1_save", FightCfg {
        name: "厨师丧尸", hp: 34, dmg: (7, 13), reward: 10, reward_why: "首次击杀丧尸",
        intro: "丧尸的头骨以不自然的角度歪着，却依然朝卡普兰的方向嘶吼挣扎。",
        rage_at: None, rage_text: "", on_rage: zombie_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_after_zombie1_save".into(), death: "e_zombies",
    }),
    ("zombie1_far", FightCfg {
        name: "厨师丧尸", hp: 34, dmg: (7, 13), reward: 10, reward_why: "首次击杀丧尸",
        intro: "丧尸撞翻桌椅，踩着满地餐具朝你挪来。",
        rage_at: None, rage_text: "", on_rage: zombie_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_after_zombie1_far".into(), death: "e_zombies",
    }),
    ("horde", FightCfg {
        name: "丧尸群（先锋）", hp: 55, dmg: (11, 17), reward: 20, reward_why: "突破丧尸群封锁",
        intro: "最前面的几只扑进水里溅起大片水花，后面源源不断……",
        rage_at: Some(25), rage_text: "更多的丧尸从管道里涌出来了——它们被血腥味刺激得疯狂！",
        on_rage: horde_on_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_rain_bitten".into(), death: "e_zombies",
    }),
    ("licker", FightCfg {
        name: "舔食者", hp: 112, dmg: (15, 22), reward: 500, reward_why: "击杀变异体「舔食者」",
        intro: "它的长舌「啪」地抽碎了你脚边的水泥碴。",
        rage_at: Some(55), rage_text: "舔食者发出刺耳的尖啸，暴起发难——速度陡然翻倍！",
        on_rage: licker_on_rage,
        finisher_if: fin_if_licker, finisher_name: fin_name, finisher_desc: fin_desc,
        win: |_| "s_escape_train".into(), death: "e_licker",
    }),
    ("b_chef", FightCfg {
        name: "厨师丧尸", hp: 38, dmg: (9, 15), reward: 12, reward_why: "击杀持刀厨师丧尸",
        intro: "它握着菜刀，指节泛白，围裙上溅满干涸的血渍——生前最后一刀，砍在了自己人身上。",
        rage_at: None, rage_text: "", on_rage: zombie_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_b_kitchen_after".into(), death: "e_zombies",
    }),
    ("b_guard", FightCfg {
        name: "保安丧尸", hp: 36, dmg: (8, 14), reward: 12, reward_why: "击杀保安丧尸",
        intro: "它穿着蜂巢保安制服，胸口的工牌还在——「威廉·帕克斯」。生前是守住这扇门的人。",
        rage_at: None, rage_text: "", on_rage: zombie_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_b_sterile_after".into(), death: "e_zombies",
    }),
    ("mut_guard", FightCfg {
        name: "样本库守卫变异体", hp: 42, dmg: (11, 18), reward: 25, reward_why: "击杀病毒学家变异体",
        intro: "白大褂被绿色原液浸透，皮肤以肉眼可见的速度腐化——它曾是研究T病毒的人，如今成了T病毒的宿主。",
        rage_at: Some(20), rage_text: "它发出非人的嘶吼，绿色原液从溃烂处喷溅——腐化加速，攻势凌厉！",
        on_rage: zombie_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_virus_vault_after".into(), death: "e_zombies",
    }),
    ("licker_larva", FightCfg {
        name: "舔食者·早期体", hp: 60, dmg: (10, 16), reward: 80, reward_why: "压制舔食者早期体",
        intro: "没有眼睛的幼体贴在隔离舱的强化玻璃上，细长的舌头舔舐着裂缝——它还没有学会「看」，但已经记住了你的气味。",
        rage_at: Some(30), rage_text: "它挣破了隔离舱的束缚，发出实验性尖啸——还有三回合挣脱，压制它！",
        on_rage: zombie_rage,
        finisher_if: |_, _| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
        win: |_| "s_isolation_after".into(), death: "e_licker",
    }),
    ("hunter_elite", FightCfg {
        name: "猎杀者·实验体", hp: 92, dmg: (14, 21), reward: 120, reward_why: "击杀实验体猎杀者",
        intro: "一道灰白色的影子从管廊阴影里无声坠下——四肢反折、指节如钩，颈后蚀刻着「H-07-ELITE」的金属环。它没有眼睛，却准确地转向了你：这是蜂巢最深处的「失败品」，也是最危险的成品。",
        rage_at: Some(40), rage_text: "金属环爆出刺耳的蜂鸣——猎杀者的角质层裂开，露出底下猩红的肌肉纤维，速度暴涨！",
        on_rage: zombie_rage,
        finisher_if: |_, ehp| ehp <= 26, finisher_name: fin_name, finisher_desc: fin_desc,
        win: |_| "s_world_back".into(), death: "e_zombies",
    }),
];

pub fn fight_cfg(id: &str) -> Option<&'static FightCfg> {
    FIGHTS.iter().find(|(k, _)| *k == id).map(|(_, v)| v)
        .or_else(|| crate::scenes_zhouyuan::zhouyuan_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_moshi::moshi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_yinse::yinse_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_yiying::yiying_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_tianshe::tianshe_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_jiguancheng::jiguancheng_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_moruiya::moruiya_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_cangjingge::cangjingge_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_jianzhong::jianzhong_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_tongqu::tongqu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_juluoji::juluoji_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_xinghe::xinghe_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_sishen::sishen_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_mumiyi::mumiyi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_mojiao::mojiao_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_wulin::wulin_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_tianting::tianting_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_hezi::hezi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_shaqiu::shaqiu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_yize::yize_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_poxiao::poxiao_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_tiexue::tiexue_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_tiexue2::tiexue2_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_xingjichuanqi::xingjichuanqi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_xinhuangfang::xinhuangfang_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_huanxiongshi::huanxiongshi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_mengguijie::mengguijie_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_siwuzhen::siwuzhen_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_jingjiling::jingjiling_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_shenmiao::shenmiao_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_shuangbai::shuangbai_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_dashengtang::dashengtang_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_daliexi::daliexi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_poxu::poxu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_hangu::hangu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_panbu::panbu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_diweidu::diweidu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_sanlian::sanlian_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_wujin::wujin_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_yizhong::yizhong_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_jishengqianye::jishengqianye_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_miwu::miwu_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_xingchen::xingchen_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_yinxiang::yinxiang_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_nuoya::nuoya_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_lanshan::lanshan_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_shourongsuo::shourongsuo_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_tianwang::tianwang_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_xingjijianchuan::xingjijianchuan_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_xingjichuanqi2::xingjichuanqi2_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_jialebi::jialebi_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_shenghua3::shenghua3_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_jishujing::jishujing_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_baisun::baisun_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_bihai::bihai_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
        .or_else(|| crate::scenes_yecheng::yecheng_figths().iter().find(|(k, _)| *k == id).map(|(_, v)| v))
}

/* ================= 结算 ================= */
/// 唯一权威结算函数（7 侧支线）；engine::goto 进入 s_settle 时同样调用它，
/// 保证 stores 的 settle_total/settle_rank 与结算卡片展示一致。
pub fn compute_settlement(st: &GameState) -> (i32, char, i32, i32, i32) {
    let alive_bonus = st.alive_count() * 100;
    let sides = ["A", "B1", "B2", "C", "decon_truth", "server_cooling", "nav_manual_cross"]
        .iter().filter(|k| st.flag(k)).count() as i32;
    let side_bonus = sides * 200;
    let total = st.points + alive_bonus + side_bonus;
    let rank = if total >= 1600 { 'S' } else if total >= 1300 { 'A' } else if total >= 1000 { 'B' } else if total >= 700 { 'C' } else { 'D' };
    (total, rank, alive_bonus, side_bonus, sides)
}

fn card_settle(st: &GameState) -> Card {
    let (total, rank, ab, sb, sides_n) = compute_settlement(st);
    let side_names = [
        ("A", "与蕾恩建立信任"),
        ("B1", "洞察丧尸行动规律"),
        ("B2", "参透激光通道机关"),
        ("C", "为蕾恩注射肾上腺素"),
        ("decon_truth", "窥破消毒事故真相"),
        ("server_cooling", "修复冷却回路并读取散热情报"),
        ("nav_manual_cross", "参透蜂巢垂直捷径"),
    ];
    let got: Vec<String> = side_names.iter().filter(|(k, _)| st.flag(k)).map(|(k, n)| {
        if k.len() == 1 || k.len() == 2 { format!("支线{k} · {n}") } else { format!("隐藏调查 · {n}") }
    }).collect();
    let sides_line = if got.is_empty() {
        "未达成任何隐藏支线。主神的光柱依旧明亮，下一世，试着多看、多问、多想。".to_string()
    } else {
        format!("已达成：{}", got.join("；"))
    };
    Card {
        title: "任 务 完 成".into(),
        good: true,
        body_html: format!(
            "<p style='text-align:center;color:#9a958a'>「生化危机」世界通关 · 轮回结算</p>\
             <table class='statTable'>\
             <tr><td>累计奖励点数（击杀·救援·探索）</td><td>+{}</td></tr>\
             <tr><td>存活队友加成 × {} 人</td><td>+{}</td></tr>\
             <tr><td>支线剧情达成 × {} 条</td><td>+{}</td></tr>\
             <tr><td>剩余理智</td><td>{}</td></tr>\
             <tr><td style='color:#8fd0a8'><b>轮回总计</b></td><td style='color:#8fd0a8;font-size:18px'><b>{}</b></td></tr>\
             <tr><td style='color:#ffd76a'><b>综合评价</b></td><td style='color:#ffd76a;font-size:18px'><b>{}</b></td></tr>\
             </table><p>{}</p>",
            st.points, st.alive_count(), ab, sides_n, sb, st.san.max(0), total, rank, sides_line
        ),
        buttons: vec![("进 入 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
        voice: None,
    }
}

fn card_nexus(_st: &GameState) -> Card {
    Card {
        title: "主 神 空 间".into(),
        good: false,
        body_html: r#"<p>白光散去，半圆形广场依旧冰冷，金属的寒意从脚底一直升到肩头。张杰抱着手臂等你，嘴角挂着意味不明的笑。</p>
<p style='color:#ffd76a'>「不错，第一次就能活着回来，还清掉了那只舔食者。」他上下打量你，「知道吗，你这批新人里，<em>回来的不到一半</em>。」</p>
<p>他随手一挥，广场上空浮现出巨大的兑换目录光影，一行行条目在冷光里亮起，像一面铺满价签的墙：</p>
<table class='statTable'>
<tr><td>细胞活力强化（体质提升）</td><td>800 点</td></tr>
<tr><td>基因锁第一阶段·自主开启权</td><td>2000 点 + D级支线×1</td></tr>
<tr><td>血统类：初级吸血鬼血统</td><td>3000 点 + C级支线×1</td></tr>
<tr><td>复活一名本次阵亡的同伴</td><td>4000 点</td></tr>
</table>
<p style='color:#ff8a8a'>「攒够点数再来吧，新人。」张杰转身走向黑暗，声音飘回来，「对了——我叫张杰。<em>记住这个名字。</em>以后你会明白，引导者的意思……呵呵。」</p>
<p>他的背影消失在光柱之外。那句轻笑让你莫名觉得脊背发凉——他知道的，远比他说出来的多。</p>
<p style='text-align:center;color:#888;margin-top:14px'>休息期：20 天<br>—— 血色光柱再次亮起 ——<br><b style='color:#c0162b;font-size:18px'>下一部恐怖片：《咒 怨》</b><br><span style='color:#555'>无限的世界，无限的轮回。第一章 · 完</span></p>"#.to_string(),
        buttons: vec![
            ("回 主 神 空 间 ⌂".into(), "__enter_nexus__".into()),
            ("进 入 下 一 次 轮 回 ▶".into(), "__title__".into()),
        ],
        voice: None,
    }
}

fn death_card(title: &str, body: &str) -> Card {
    Card { title: title.to_string(), good: false, body_html: body.to_string(), buttons: vec![("轮 回 重 启".into(), "__title__".into())], voice: None }
}

/// 主神空间：确认「开始下一轮回」卡片（P1 把轮回重启入口移入主神）
fn card_new_cycle(_st: &GameState) -> Card {
    Card {
        title: "珍 重".into(),
        good: false,
        body_html: r#"<p>张杰沉默片刻，目光落在那道血色光柱上，最后点了点头。</p>
<p>「决定了就别回头。血色光柱会把你们裹进下一部片子——记住你在主神空间兑换的一切，都只为你活着回来而存在。回不来的人，在这里留下的只有一行记录。」</p>
<p><b style='color:#c0162b;font-size:18px'>开始一次新的轮回？</b><br><span style='color:#666'>已获得的点数 / 兑换 / 记忆会清空重来，唯有"轮回记忆"（探索过的地图）保留。</span></p>"#.to_string(),
        buttons: vec![
            ("开 始 下 一 次 轮 回 ▶".into(), "__title__".into()),
            ("返回主神广场".into(), "__back_to_world__".into()),
        ],
        voice: None,
    }
}

/* =============================================================
   P1 主神空间 · 点数消费闭环（兑换 / 复活 / 简报）
   条件扣点走 Route::Dyn 内联：函数内校验 points → 扣点 → 写状态，
   按结果返回 SUCCESS / FAIL 场景。数值依据 00_ENGINE_CONTEXT §2.3/§2.4。
   ============================================================= */
const COST_STRENGTHEN: i32 = 800;   // 细胞活力强化
const COST_GENE: i32 = 2000;        // 基因锁一阶·自主开启权
const COST_VAMPIRE: i32 = 3000;     // 初级吸血鬼血统
const COST_RESURRECT: i32 = 4000;   // 复活一名本次阵亡同伴

pub fn is_vampire(st: &GameState) -> bool {
    st.bloodline.as_deref() == Some("vampire")
}

fn exchange_name(st: &GameState) -> String {
    // 汇总已兑换清单（供简报与兑换成功文本复用）—— 包 C 扩列：基因阶/血统/内功/护盾/修真/技能数/装备
    let mut list: Vec<String> = vec![];
    if st.str_bonus > 0 {
        list.push(format!("细胞活力强化 ×{}（攻击+{}）", st.str_bonus, st.str_bonus * 5));
    }
    if st.gene_lock {
        list.push("基因锁一阶·自主开启权".into());
    }
    let gstage = crate::combat_data::gene_stage_of(st);
    if gstage > 1 {
        list.push(format!("基因锁{}阶·开启", gstage));
    }
    if let Some(bid) = st.bloodline.as_deref() {
        let nm = crate::combat_data::bloodline_def(bid).map(|b| b.name).unwrap_or(bid);
        list.push(format!("血统·{}", nm));
    }
    if let Some(a) = st.inner_art.as_deref() {
        let nm = match a { "wuming" => "无名剑诀", "jingxin" => "静心诀", _ => a };
        list.push(format!("内功·{}", nm));
    }
    if st.tech_shield_max > 0 {
        list.push(format!("纳米护盾（上限{}）", st.tech_shield_max));
    }
    if st.cultivation_stage > 0 {
        let name = crate::combat_data::cultivation_stage_cfg(st.cultivation_stage).map(|c| c.name).unwrap_or("修士");
        list.push(format!("修真·{}", name));
    }
    let skill_n = st.skills.len();
    if skill_n > 0 {
        list.push(format!("武学/技能 ×{}", skill_n));
    }
    if st.equipment.weapon.is_some() || st.equipment.armor.is_some() || st.equipment.accessory.is_some() {
        list.push("已装配装备".into());
    }
    if list.is_empty() { "（尚未兑换任何强化）".into() } else { list.join("；") }
}

// 兑换：细胞活力强化（+1 体质）—— 校验点数够 → 扣 800 → str_bonus+1
fn route_exchange_strengthen(st: &mut GameState) -> String {
    if st.points < COST_STRENGTHEN { return "s_nexus_exchange_fail".into(); }
    st.points -= COST_STRENGTHEN;
    st.str_bonus += 1;
    "s_nexus_exchange_done".into()
}
// 兑换：基因锁一阶 2000 点（无 D 支线令牌，P1 仅扣点并开启自主权 flag，不影响觉醒剧情）
fn route_exchange_gene(st: &mut GameState) -> String {
    if st.points < COST_GENE { return "s_nexus_exchange_fail".into(); }
    st.points -= COST_GENE;
    // gene_lock = 自主开启权已掌握（战斗：攻击追加伤害 + 闪避 + 减伤）
    st.set_flag("ex_bought_gene");
    st.gene_lock = true;
    "s_nexus_exchange_done".into()
}
// 兑换：初级吸血鬼血统 3000 点（无 C 支线令牌，P1 仅扣点；附带敏捷 +1）
fn route_exchange_vampire(st: &mut GameState) -> String {
    if st.points < COST_VAMPIRE { return "s_nexus_exchange_fail".into(); }
    st.points -= COST_VAMPIRE;
    st.bloodline = Some("vampire".to_string());
    st.agi_bonus += 1;
    "s_nexus_exchange_done".into()
}

// ============================================================================
// 包 C：主神兑换扩展（Route::Dyn 条件扣点）· has_grade_or 统一评级门槛 D<C<B<A<S
// ============================================================================
/// 评级排序：D(0) < C(1) < B(2) < A(3) < S(4)；未知等级视为 D 之下
fn grade_rank(g: char) -> u8 {
    match g { 'D' => 0, 'C' => 1, 'B' => 2, 'A' => 3, 'S' => 4, _ => 0 }
}
/// 统一评级门槛判定：need=None 恒可达；否则需 sp_grade ≥ need
fn has_grade_or(st: &GameState, need: Option<char>) -> bool {
    match need {
        None => true,
        Some(g) => st.sp_grade.map_or(false, |s| grade_rank(s) >= grade_rank(g)),
    }
}

// ---- 基因锁进阶（二阶/三阶/四阶）----
fn route_exchange_gene2(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('B')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 6000 { return "s_nexus_exchange_fail".into(); }
    if crate::combat_data::gene_stage_of(st) >= 2 { return "s_nexus_exchange_done".into(); }
    st.points -= 6000;
    crate::combat_data::set_gene_stage(st, 2);
    "s_nexus_exchange_done".into()
}
fn route_exchange_gene3(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('A')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 12000 { return "s_nexus_exchange_fail".into(); }
    if crate::combat_data::gene_stage_of(st) >= 3 { return "s_nexus_exchange_done".into(); }
    st.points -= 12000;
    crate::combat_data::set_gene_stage(st, 3);
    "s_nexus_exchange_done".into()
}
fn route_exchange_gene4(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('A')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 22000 { return "s_nexus_exchange_fail".into(); }
    if crate::combat_data::gene_stage_of(st) >= 4 { return "s_nexus_exchange_done".into(); }
    st.points -= 22000;
    crate::combat_data::set_gene_stage(st, 4);
    "s_nexus_exchange_done".into()
}

// ---- 高等血统（BLOODLINES 表；写入 bloodline，互斥 hide）----
fn route_exchange_werewolf(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('B')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 4500 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("werewolf") { return "s_nexus_exchange_done".into(); }
    st.points -= 4500;
    st.bloodline = Some("werewolf".to_string());
    "s_nexus_exchange_done".into()
}
fn route_exchange_zuwu(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('B')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 5500 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("zuwu") { return "s_nexus_exchange_done".into(); }
    st.points -= 5500;
    st.bloodline = Some("zuwu".to_string());
    "s_nexus_exchange_done".into()
}
fn route_exchange_zhanshi(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('C')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 3500 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("zhanshi_blood") { return "s_nexus_exchange_done".into(); }
    st.points -= 3500;
    st.bloodline = Some("zhanshi_blood".to_string());
    "s_nexus_exchange_done".into()
}

// ---- 高等血统扩容（第 2 批：天使/恶魔/龙族/机械义体，互斥 hide，价格/门槛自洽）----
fn route_exchange_angel(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('A')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 9000 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("angel_bloodline") { return "s_nexus_exchange_done".into(); }
    st.points -= 9000;
    st.bloodline = Some("angel_bloodline".to_string());
    "s_nexus_exchange_done".into()
}
fn route_exchange_demon(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('A')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 9500 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("demon_bloodline") { return "s_nexus_exchange_done".into(); }
    st.points -= 9500;
    st.bloodline = Some("demon_bloodline".to_string());
    "s_nexus_exchange_done".into()
}
fn route_exchange_dragon(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('A')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 10000 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("dragon_bloodline") { return "s_nexus_exchange_done".into(); }
    st.points -= 10000;
    st.bloodline = Some("dragon_bloodline".to_string());
    "s_nexus_exchange_done".into()
}
fn route_exchange_cyberpro(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('B')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 7800 { return "s_nexus_exchange_fail".into(); }
    if st.bloodline.as_deref() == Some("cyber_prosthetic") { return "s_nexus_exchange_done".into(); }
    st.points -= 7800;
    st.bloodline = Some("cyber_prosthetic".to_string());
    "s_nexus_exchange_done".into()
}
// ---- 高等血统扩容（第 3 批 · 动漫/小说 10 条，互斥 hide）----
macro_rules! route_bloodline {
    ($fnname:ident, $id:expr, $price:expr, $grade:expr) => {
        fn $fnname(st: &mut GameState) -> String {
            if !has_grade_or(st, $grade) { return "s_nexus_exchange_fail".into(); }
            if st.points < $price { return "s_nexus_exchange_fail".into(); }
            if st.bloodline.as_deref() == Some($id) { return "s_nexus_exchange_done".into(); }
            st.points -= $price;
            st.bloodline = Some($id.to_string());
            "s_nexus_exchange_done".into()
        }
    };
}
route_bloodline!(route_exchange_saiyan, "saiyan_bloodline", 9000, Some('A'));
route_bloodline!(route_exchange_sharingan, "sharingan_bloodline", 8200, Some('B'));
route_bloodline!(route_exchange_hollow, "hollow_bloodline", 9800, Some('A'));
route_bloodline!(route_exchange_saint, "saint_bloodline", 9500, Some('A'));
route_bloodline!(route_exchange_shinigami, "shinigami_bloodline", 9000, Some('A'));
route_bloodline!(route_exchange_quincy, "quincy_bloodline", 8800, Some('A'));
route_bloodline!(route_exchange_uchiha, "uchiha_bloodline", 9000, Some('A'));
route_bloodline!(route_exchange_senju, "senju_bloodline", 8600, Some('A'));
route_bloodline!(route_exchange_otsutsuki, "otsutsuki_bloodline", 15000, Some('S'));
route_bloodline!(route_exchange_mitsurugi, "mitsurugi_bloodline", 7600, Some('B'));
// ---- 蜘蛛侠血统（世界地图蛛丝拉拽，闪避流）----
route_bloodline!(route_exchange_spiderman, "spiderman_bloodline", 6800, Some('B'));

// ---- 修真破境（CULTIVATION_STAGES 表：下一境界价格 = 表 need_points + need_grade）----
fn route_exchange_cultivation(st: &mut GameState) -> String {
    use crate::combat_data::cultivation_stage_cfg;
    let next: u8 = st.cultivation_stage + 1;
    let Some(cfg) = cultivation_stage_cfg(next) else {
        return "s_nexus_exchange_done".into(); // 已到顶（合道），无可破境
    };
    if let Some(p) = cfg.prev {
        if st.cultivation_stage != p { return "s_nexus_exchange_fail".into(); }
    }
    if !has_grade_or(st, cfg.need_grade) { return "s_nexus_exchange_fail".into(); }
    let cost = cfg.need_points as i32;
    if st.points < cost { return "s_nexus_exchange_fail".into(); }
    st.points -= cost;
    st.cultivation_stage = next;
    st.cultivation_qi_max = crate::combat_data::qi_max_cap_of(st);
    "s_nexus_exchange_done".into()
}

// ---- 内功心法（inner_art / qi_max）----
fn route_exchange_wuming(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('D')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 1500 { return "s_nexus_exchange_fail".into(); }
    if st.inner_art.as_deref() == Some("wuming") { return "s_nexus_exchange_done".into(); }
    st.points -= 1500;
    st.inner_art = Some("wuming".to_string());
    st.qi_max = 40;
    st.qi = 40;
    "s_nexus_exchange_done".into()
}
fn route_exchange_jingxin(st: &mut GameState) -> String {
    if st.points < 350 { return "s_nexus_exchange_fail".into(); }
    if st.inner_art.as_deref() == Some("jingxin") { return "s_nexus_exchange_done".into(); }
    st.points -= 350;
    st.inner_art = Some("jingxin".to_string());
    st.qi_max += 20;
    st.qi = (st.qi + 20).min(st.qi_max);
    st.san = (st.san + 10).clamp(0, 100);
    "s_nexus_exchange_done".into()
}

// ---- 纳米护盾模块（tech_shield）----
fn route_exchange_shield(st: &mut GameState) -> String {
    if !has_grade_or(st, Some('D')) { return "s_nexus_exchange_fail".into(); }
    if st.points < 1800 { return "s_nexus_exchange_fail".into(); }
    st.points -= 1800;
    st.tech_shield_max += 30;
    st.tech_shield = st.tech_shield_max;
    "s_nexus_exchange_done".into()
}

// ============================================================================
// 技能兑换（SKILLS 表 price/need_grade/need_bloodline/need_qi/need_stage/need_cultivation）
// Route::Dyn 无法携带参数，故每个技能由宏生成独立 route/cond，逻辑走共享 buy_skill。
// ============================================================================
/// 技能可否购买：未拥有 + 评级达标 + 血统/基因阶/修真境界/真气上限前置满足
fn skill_avail(st: &GameState, id: &str) -> bool {
    let Some(sk) = crate::skills_data::skill(id) else { return false; };
    if st.skills.iter().any(|s| s == id) { return false; }
    if !has_grade_or(st, sk.need_grade) { return false; }
    if let Some(b) = sk.need_bloodline {
        if st.bloodline.as_deref() != Some(b) { return false; }
    }
    if let Some(g) = sk.need_stage {
        if crate::combat_data::gene_stage_of(st) < g { return false; }
    }
    if let Some(cs) = sk.need_cultivation {
        if st.cultivation_stage < cs { return false; }
    }
    if let Some(m) = sk.need_qi {
        if st.qi_max < m { return false; }
    }
    true
}

fn buy_skill(st: &mut GameState, id: &str) -> String {
    let Some(sk) = crate::skills_data::skill(id) else { return "s_nexus_exchange_fail".into(); };
    if !skill_avail(st, id) { return "s_nexus_exchange_fail".into(); }
    if st.points < sk.price { return "s_nexus_exchange_fail".into(); }
    st.points -= sk.price;
    st.skills.push(id.to_string());
    "s_nexus_exchange_done".into()
}

/// 技能目录宏：`skill_cat!($stat, [(id, 显示名), ...])` —— 在模块作用域展开。
/// 为每个技能生成独立购买 route fn（以技能 id 为函数名，走共享 buy_skill，价格/门槛查 SKILLS 表），
/// 并产出一份目录 ChoiceDef 静态数组。技能 id 均为合法 Rust 标识符。
macro_rules! skill_cat {
    ($stat:ident, [ $( ($id:ident, $nm:expr) ),* $(,)? ]) => {
        $(
            #[allow(non_snake_case)]
            fn $id(st: &mut GameState) -> String { buy_skill(st, stringify!($id)) }
        )*
        pub static $stat: &[ChoiceDef] = &[
            // 目录行：已购 / 前置不满足在购买时拒绝（价格门槛由 buy_skill 查表）
            $( ChoiceDef {
                label: $nm,
                sub: "价格 / 门槛查 SKILLS 表 · 已购或未达标将以失败提示",
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn($id),
            }, )*
            ChoiceDef { label: "返回技能秘藏", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_skill") },
        ];
    };
}

// ============================================================================
// 道具铺（ITEMS/WEAPONS/GEAR/TRESURE_DEFS 表：价格查表；写 inventory/equipment）
// ============================================================================
/// 统一购买道具：点数组件校验 price + 评级门槛 → 扣点 → 归入 inventory 或 equipment
fn buy_item(st: &mut GameState, id: &str) -> String {
    // 武器
    if let Some(w) = crate::items_data::weapon_def(id) {
        if !has_grade_or(st, w.need_grade) { return "s_nexus_exchange_fail".into(); }
        if st.points < w.base_price { return "s_nexus_exchange_fail".into(); }
        st.points -= w.base_price;
        st.equipment.weapon = Some(crate::defs::WeaponSlot { id: id.to_string(), enhance: 0 });
        return "s_nexus_exchange_done".into();
    }
    // 护甲/饰品
    if let Some(g) = crate::items_data::gear_def(id) {
        if !has_grade_or(st, g.need_grade) { return "s_nexus_exchange_fail".into(); }
        if st.points < g.price { return "s_nexus_exchange_fail".into(); }
        st.points -= g.price;
        match g.slot {
            crate::defs::GearSlot::Armor => st.equipment.armor = Some(id.to_string()),
            crate::defs::GearSlot::Accessory => st.equipment.accessory = Some(id.to_string()),
            crate::defs::GearSlot::Treasure => {
                let idx = if !st.treasures.contains(&id.to_string()) { st.treasures.len() } else { st.treasures.len() };
                if idx < 3 { st.treasures.push(id.to_string()); }
            }
        }
        return "s_nexus_exchange_done".into();
    }
    // 法宝（equipment.treasure 为装配权威；treasures 保留拥有标记）
    if let Some(t) = crate::items_data::treasure_def(id) {
        if !has_grade_or(st, t.need_grade) { return "s_nexus_exchange_fail".into(); }
        if st.points < t.price { return "s_nexus_exchange_fail".into(); }
        st.points -= t.price;
        if !st.treasures.contains(&id.to_string()) {
            st.treasures.push(id.to_string());
        }
        // 装配权威：写入 equipment.treasure[slot]（0=本命 1=护身 2=辅助）
        let slot = (t.slot as usize).min(2);
        st.equipment.treasure[slot] = Some(id.to_string());
        return "s_nexus_exchange_done".into();
    }
    // 消耗品 / 圣物（ITEMS 表）
    if let Some(it) = crate::items_data::item_def(id) {
        if !has_grade_or(st, it.need_grade) { return "s_nexus_exchange_fail".into(); }
        if st.points < it.price { return "s_nexus_exchange_fail".into(); }
        st.points -= it.price;
        crate::items_data::add_item_counted(st, id);
        return "s_nexus_exchange_done".into();
    }
    "s_nexus_exchange_fail".into()
}

/// 道具铺宏：`shop_cat!($stat, [(id, 显示名, 描述), ...])` —— 在模块作用域展开。
/// 每个道具生成独立购买 route fn（以道具 id 为函数名，走共享 buy_item）。
/// 第三元 `desc` 为黑魂式碎片叙事描述（与 items_data 表一致），随 choice.sub 下发到前端展示。
macro_rules! shop_cat {
    ($stat:ident, [ $( ($id:ident, $nm:expr, $desc:expr) ),* $(,)? ]) => {
        $(
            #[allow(non_snake_case)]
            fn $id(st: &mut GameState) -> String { buy_item(st, stringify!($id)) }
        )*
        pub static $stat: &[ChoiceDef] = &[
            $( ChoiceDef {
                label: $nm,
                sub: $desc,
                cond: None,
                effects: &NO_EFF,
                route: Route::Dyn($id),
            }, )*
            ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
        ];
    };
}

// ---- 合成（REIPES 表校验 inventory 原料）----
fn recipe_has(st: &GameState, result: &str) -> bool {
    if let Some(r) = crate::items_data::RECIPES.iter().find(|r| r.result == result) {
        return r.ingredients.iter().all(|&i| crate::items_data::has_item(st, i));
    }
    false
}
fn recipe_build(st: &mut GameState, result: &str) -> String {
    let Some(r) = crate::items_data::RECIPES.iter().find(|r| r.result == result) else {
        return "s_nexus_exchange_fail".into();
    };
    if !r.ingredients.iter().all(|&i| crate::items_data::has_item(st, i)) { return "s_nexus_exchange_fail".into(); }
    for &i in r.ingredients { crate::items_data::consume_item(st, i); }
    crate::items_data::add_item_counted(st, result);
    "s_nexus_exchange_done".into()
}
fn route_craft_core(st: &mut GameState) -> String { recipe_build(st, "it_core_crystal") }
fn route_craft_cross(st: &mut GameState) -> String { recipe_build(st, "it_cross") }
// 合成扩容（第 2 批）
fn route_craft_em_core(st: &mut GameState) -> String { recipe_build(st, "it_em_core") }
fn route_craft_blood_ess(st: &mut GameState) -> String { recipe_build(st, "it_blood_essence") }
fn route_craft_treasure_frag(st: &mut GameState) -> String { recipe_build(st, "it_treasure_frag") }
/// 法宝合成特例：产物归入法宝装配格（st.treasures + equipment.treasure[slot]），不走通用 inventory 计数
fn route_craft_tr_banner(st: &mut GameState) -> String {
    if !recipe_has(st, "tr_blood_banner") { return "s_nexus_exchange_fail".into(); }
    let Some(r) = crate::items_data::RECIPES.iter().find(|r| r.result == "tr_blood_banner") else {
        return "s_nexus_exchange_fail".into();
    };
    for &i in r.ingredients { crate::items_data::consume_item(st, i); }
    if !st.treasures.contains(&"tr_blood_banner".to_string()) {
        st.treasures.push("tr_blood_banner".to_string());
    }
    // 血煞战旗 slot=0（本命）→ equipment.treasure[0]
    if let Some(t) = crate::items_data::treasure_def("tr_blood_banner") {
        st.equipment.treasure[(t.slot as usize).min(2)] = Some("tr_blood_banner".to_string());
    }
    "s_nexus_exchange_done".into()
}
fn route_craft_enh_stone(st: &mut GameState) -> String { recipe_build(st, "it_enhance_stone") }
fn route_craft_enh_stone_hi(st: &mut GameState) -> String { recipe_build(st, "it_enhance_stone_hi") }

// ---- 武器强化（需先装配 equipment.weapon；每级 1000 点）----
fn cond_enhance_ready(st: &GameState) -> bool {
    st.equipment.weapon.is_some() && st.equipment.weapon.as_ref().map_or(false, |w| w.enhance < 5)
}
fn route_enhance(st: &mut GameState) -> String {
    let Some(w) = st.equipment.weapon.as_mut() else { return "s_nexus_exchange_fail".into(); };
    if w.enhance >= 5 { return "s_nexus_exchange_fail".into(); }
    if st.points < 1000 { return "s_nexus_exchange_fail".into(); }
    st.points -= 1000;
    w.enhance += 1;
    "s_nexus_exchange_done".into()
}
// ---- 强化石强化（第 2 批：消耗普通/高级强化石，替换纯点数路径）----
fn route_enhance_stone(st: &mut GameState) -> String {
    let ok_weapon = st.equipment.weapon.as_ref().map_or(false, |w| w.enhance < 5);
    if !ok_weapon { return "s_nexus_exchange_fail".into(); }
    if !crate::items_data::consume_item(st, "it_enhance_stone") { return "s_nexus_exchange_fail".into(); }
    if let Some(w) = st.equipment.weapon.as_mut() { w.enhance += 1; }
    "s_nexus_exchange_done".into()
}
fn route_enhance_stone_hi(st: &mut GameState) -> String {
    let ok_weapon = st.equipment.weapon.as_ref().map_or(false, |w| w.enhance < 5);
    if !ok_weapon { return "s_nexus_exchange_fail".into(); }
    if !crate::items_data::consume_item(st, "it_enhance_stone_hi") { return "s_nexus_exchange_fail".into(); }
    // 高级强化石一次性 +2
    if let Some(w) = st.equipment.weapon.as_mut() { w.enhance = (w.enhance + 2).min(5); }
    "s_nexus_exchange_done".into()
}

/// 复活首名阵亡同伴（4000 点）：校验点数 + dead_team 非空 → 扣点 → 移出 dead_team
fn route_resurrect_teammate(st: &mut GameState) -> String {
    if st.dead_team.is_empty() { return "s_nexus_resurrect_none".into(); }
    if st.points < COST_RESURRECT { return "s_nexus_resurrect_fail".into(); }
    let first = st.dead_team.remove(0);
    st.set_flag(&format!("resurrected_{}", first));
    st.points -= COST_RESURRECT;
    st.resurrected_name = Some(first);
    "s_nexus_resurrect_done".into()
}

/// 兑换项可见性（static 常量安全的纯函数指针）：
/// 强化可重复购买；基因锁一经购买隐藏；吸血鬼血统一经兑换隐藏
fn cond_show_strengthen(_: &GameState) -> bool { true }
fn cond_show_gene(st: &GameState) -> bool { !st.flag("ex_bought_gene") }
fn cond_show_vampire(st: &GameState) -> bool { st.bloodline.as_deref() != Some("vampire") }
/// 复活祭坛：有阵亡同伴才显示「复活」选项
fn cond_has_dead_teammate(st: &GameState) -> bool { !st.dead_team.is_empty() }

// ---- 包 C 新增分类可见性 ----
/// 基因进阶分类：已掌握一阶（含兑换）后才开放；未达四阶始终可见
fn cond_show_gene_cat(st: &GameState) -> bool {
    crate::combat_data::gene_stage_of(st) >= 1 && crate::combat_data::gene_stage_of(st) < 4
}
/// 单条基因阶购买可见性：当前阶 < 该阶
fn cond_gene_lt(st: &GameState, n: u8) -> bool { crate::combat_data::gene_stage_of(st) < n }
fn cond_gene2(st: &GameState) -> bool { cond_gene_lt(st, 2) }
fn cond_gene3(st: &GameState) -> bool { cond_gene_lt(st, 3) }
fn cond_gene4(st: &GameState) -> bool { cond_gene_lt(st, 4) }
/// 血统互斥：未拥有该血统才显示
fn cond_blood_none(st: &GameState, id: &str) -> bool { st.bloodline.as_deref() != Some(id) }
fn cond_werewolf(st: &GameState) -> bool { cond_blood_none(st, "werewolf") }
fn cond_zuwu(st: &GameState) -> bool { cond_blood_none(st, "zuwu") }
fn cond_zhanshi(st: &GameState) -> bool { cond_blood_none(st, "zhanshi_blood") }
/// 血统扩容（第 2 批）：互斥可见性
fn cond_angel(st: &GameState) -> bool { cond_blood_none(st, "angel_bloodline") }
fn cond_demon(st: &GameState) -> bool { cond_blood_none(st, "demon_bloodline") }
fn cond_dragon(st: &GameState) -> bool { cond_blood_none(st, "dragon_bloodline") }
fn cond_cyberpro(st: &GameState) -> bool { cond_blood_none(st, "cyber_prosthetic") }
/// 血统扩容（第 3 批 · 动漫/小说 10 条）：互斥可见性
fn cond_saiyan(st: &GameState) -> bool { cond_blood_none(st, "saiyan_bloodline") }
fn cond_sharingan(st: &GameState) -> bool { cond_blood_none(st, "sharingan_bloodline") }
fn cond_hollow(st: &GameState) -> bool { cond_blood_none(st, "hollow_bloodline") }
fn cond_saint(st: &GameState) -> bool { cond_blood_none(st, "saint_bloodline") }
fn cond_shinigami(st: &GameState) -> bool { cond_blood_none(st, "shinigami_bloodline") }
fn cond_quincy(st: &GameState) -> bool { cond_blood_none(st, "quincy_bloodline") }
fn cond_uchiha(st: &GameState) -> bool { cond_blood_none(st, "uchiha_bloodline") }
fn cond_senju(st: &GameState) -> bool { cond_blood_none(st, "senju_bloodline") }
fn cond_otsutsuki(st: &GameState) -> bool { cond_blood_none(st, "otsutsuki_bloodline") }
fn cond_mitsurugi(st: &GameState) -> bool { cond_blood_none(st, "mitsurugi_bloodline") }
fn cond_spiderman(st: &GameState) -> bool { cond_blood_none(st, "spiderman_bloodline") }
/// 修真破境可见性：当前境界 < 7 才有下一境可破
fn cond_cultivable(st: &GameState) -> bool { st.cultivation_stage < 7 }
/// 内功：未学对应才显示
fn cond_art_wuming(st: &GameState) -> bool { st.inner_art.as_deref() != Some("wuming") }
fn cond_art_jingxin(st: &GameState) -> bool { st.inner_art.as_deref() != Some("jingxin") }
/// 合成原料齐备才显示
fn cond_has_recipe_core(st: &GameState) -> bool { recipe_has(st, "it_core_crystal") }
fn cond_has_recipe_cross(st: &GameState) -> bool { recipe_has(st, "it_cross") }
// 合成扩容（第 2 批）
fn cond_has_recipe_em_core(st: &GameState) -> bool { recipe_has(st, "it_em_core") }
fn cond_has_recipe_blood_ess(st: &GameState) -> bool { recipe_has(st, "it_blood_essence") }
fn cond_has_recipe_treasure_frag(st: &GameState) -> bool { recipe_has(st, "it_treasure_frag") }
fn cond_has_recipe_tr_banner(st: &GameState) -> bool { recipe_has(st, "tr_blood_banner") }
fn cond_has_recipe_enh_stone(st: &GameState) -> bool { recipe_has(st, "it_enhance_stone") }
fn cond_has_recipe_enh_stone_hi(st: &GameState) -> bool { recipe_has(st, "it_enhance_stone_hi") }
/// 强化石消耗提示：已装配武器且低于 +5 上限时，选项可用（额外扣 1 强化石）
fn cond_enhance_stone_ready(st: &GameState) -> bool {
    cond_enhance_ready(st) && crate::items_data::has_item(st, "it_enhance_stone")
}
fn cond_enhance_stone_hi_ready(st: &GameState) -> bool {
    cond_enhance_ready(st) && crate::items_data::has_item(st, "it_enhance_stone_hi")
}

/// 兑换目录对话文本（Dyn）：显示当前点数与已兑换状态
fn text_exchange(st: &GameState) -> String {
    let owned = if exchange_name(st).contains("尚未兑换") {
        "你还不曾兑换任何强化。光球安静地悬着，等你决定把第一条命押在哪一项上。攒够点数，选一项投资在自己身上。".to_string()
    } else {
        format!("已兑换：<b style='color:#8fd0a8'>{}</b>", exchange_name(st))
    };
    format!(
        "光球在你面前浮现出层层叠叠的光影，像一座悬在虚空里的市集——<b>兑换目录</b>悬浮于光柱之中，每一条都冷得精确，不带半点人情。\n\n\
         <table class='statTable'>\
         <tr><td>细胞活力强化（体质提升，可叠加）</td><td>800 点 · 每级攻击 +5</td></tr>\
         <tr><td>基因锁 一至四阶 · 自主开启与进化</td><td>2000D / 6000B / 12000A / 22000A</td></tr>\
         <tr><td>血统类 · 吸血鬼 / 狼人 / 祖巫 / 圣职者</td><td>3000C / 4500B / 5500B / 3500C</td></tr>\
         <tr><td>修真 · 破境（练气→合道）</td><td>CULTIVATION_STAGES 表价</td></tr>\
         <tr><td>内功 · 无名剑诀 / 静心诀</td><td>1500D / 350</td></tr>\
         <tr><td>科技 · 纳米护盾模块</td><td>1800D · 上限+30</td></tr>\
         <tr><td>技能秘藏 / 道具铺 / 合成 / 武器强化</td><td>分类光球查阅</td></tr>\
         </table>\n\
         当前点数：<b style='color:#ffd76a'>{}</b>\n\n{owned}",
        st.points,
    )
}

/// 复活祭坛对话文本（Dyn）：列出当前阵亡同伴
fn text_resurrection(st: &GameState) -> String {
    let names: Vec<&str> = st.dead_team.iter().map(|k| team_display(k)).collect();
    let dead_line = if names.is_empty() {
        "祭坛平静无波——本次轮回无人阵亡，没有等待被赎回的亡魂。青灰的光安静地亮着，像一座空置的钟。".to_string()
    } else {
        format!("祭坛微光中浮动着几缕淡影：<b style='color:#b9e8ff'>{}</b>。<br>他们仍困在虚无之间。", names.join("、"))
    };
    format!(
        "西南角，一方<b>低矮的祭坛</b>沐浴在青灰色微光里，光晕一明一灭，像某种缓慢的心跳。坛身刻着细密的符纹，中央凹槽干涸已久——那是留给「代价」的位置。\n\n\
         {dead_line}\n\n\
         <p style='color:#888'>复活一名阵亡同伴需 <b>4000</b> 点。若是点数不足，便让亡魂再等一等。</p>"
    )
}

/// 队友 key → 显示名（与 engine hud 一致）
pub fn team_display(k: &str) -> &'static str {
    match k {
        "one" => "一号", "rain" => "蕾恩", "kaplan" => "卡普兰", "jd" => "J.D.",
        _ => "同伴",
    }
}

/// 轮回任务简报卡片：上次轮回评价 + 当前点数 + 已兑换 + 阵亡记录
fn card_briefing(st: &GameState) -> Card {
    let grade = st.sp_grade.map(|g| format!("{} 级", g)).unwrap_or_else(|| "暂无评价".into());
    let dead_line = if st.dead_team.is_empty() {
        "本次轮回无阵亡队友。".to_string()
    } else {
        let names: Vec<&str> = st.dead_team.iter().map(|k| team_display(k)).collect();
        format!("本次轮回阵亡：<b style='color:#ff8a8a'>{}</b>", names.join("、"))
    };
    Card {
        title: "轮 回 任 务 简 报".into(),
        good: false,
        body_html: format!(
            "<p style='text-align:center;color:#9a958a'>主神整理了你自进入主神空间以来的注册记录——精确，冰冷，一字不多</p>\
             <table class='statTable'>\
             <tr><td>上次轮回战报评价</td><td style='color:#ffd76a'><b>{grade}</b></td></tr>\
             <tr><td>当前奖励点数</td><td><b>{}</b></td></tr>\
             <tr><td>已兑换强化 / 血统</td><td>{exchange}</td></tr>\
             <tr><td>队友状态</td><td>{dead_line}</td></tr>\
             </table>\
             <p style='color:#666'>新轮回会清空点数与兑换，唯有轮回记忆（探索过的地图）保留。</p>",
            st.points, dead_line = dead_line, exchange = exchange_name(st)
        ),
        buttons: vec![("返回张杰身旁".into(), "__back_to_world__".into())],
        voice: None,
    }
}

// ============================================================================
// 技能兑换目录静态表 + 状态（包 C）：按流派拆分。skill_cat!/shop_cat! 为每个技能/道具
// 生成独立购买 route fn（以 id 为函数名）+ 目录 ChoiceDef 静态数组。
// ============================================================================
skill_cat!(CAT_XIU, [
    (cu_gong_qiling, "◆ 吐故纳新功"), (cu_gong_dantian, "◆ 丹田淬体诀"), (cu_gong_zhuque, "◆ 朱雀心火法"),
    (cu_gong_hunhei, "◆ 混元神罡"), (cu_gong_daoyin, "◆ 太乙导引法"), (cu_gong_hecheng, "◆ 合道归元气"),
    (cu_jin_kunxian, "◆ 困仙禁制"), (cu_jin_dingji, "◆ 定身符"), (cu_jin_lingri, "◆ 临日封印"),
    (cu_jin_zhenmo, "◆ 镇魔阵"), (cu_jin_xianzhi, "◆ 禁灵罩"), (cu_jin_ruyin, "◆ 如印封天"),
    (cu_shen_zhangxin, "◆ 掌心雷"), (cu_shen_jianren, "◆ 剑丸·一线银丝"), (cu_shen_dunjian, "◆ 御剑遁法"),
    (cu_shen_yufeng, "◆ 御风行"), (cu_shen_danhuo, "◆ 丹火·流焰"), (cu_shen_jianyu, "◆ 万剑归宗·雏形"),
    (cu_shen_hunse, "◆ 摄魂术"), (cu_shen_fankui, "◆ 反叩天地"), (cu_shen_tiandai, "◆ 天地同游"),
    (cu_shen_daoying, "◆ 返虚倒影"), (skx_xiu_talisman, "◆ 火符·炎爆"), (skx_xiu_talisman_cling, "◆ 雷符·天罡"),
    (skx_xiu_flags, "◆ 镇魂旗阵"), (skx_xiu_formation, "◆ 五行守御阵"), (skx_xiu_seal, "◆ 镇山水印"),
    (skx_xiu_spirit, "◆ 御灵印"), (skx_xiu_pill, "◆ 培元丹"),
]);
skill_cat!(CAT_WW, [
    (sk_ww_liantui, "◆ 连环腿法"), (sk_ww_bopo, "◆ 破军一击"), (sk_ww_wenxin, "◆ 问心一剑"),
    (sk_ww_wumian, "◆ 无面心法"), (sk_ww_jiuzhuan, "◆ 九转归元"), (sk_ww_wuxiang, "◆ 无相步"),
    (skx_ww_shuangji, "◆ 双影腿法"), (skx_ww_bagua, "◆ 八卦游身掌"), (skx_ww_lianhuan, "◆ 连环穿掌"),
    (skx_ww_anqi, "◆ 袖里飞蝗"), (skx_ww_taxue, "◆ 踏雪无痕"), (skx_ww_zhenqi, "◆ 真气护体"),
    (skx_ww_tiebu, "◆ 铁布衫"), (skx_ww_neigong, "◆ 周天养气诀"), (skx_ww_cangjian, "◆ 藏锋于鞘"),
    (skx_ww_suxin, "◆ 素心剑"), (skx_ww_bopi, "◆ 破甲铁臂"), (skx_ww_dianxue, "◆ 定穴指"),
    (skx_ww_fenjin, "◆ 分筋错骨手"), (skx_ww_jingangzhao, "◆ 金钟罩"), (skx_ww_qise, "◆ 七弦无形剑"),
    (skx_ww_jueming, "◆ 绝命一击"),
]);
skill_cat!(CAT_GENE, [
    (sk_gene_instinct, "◆ 战斗直觉"), (sk_gene_evade, "◆ 极限闪避"), (sk_gene_stagger, "◆ 撕咬反扑"),
    (sk_gene_focus, "◆ 猎手凝视"), (sk_gene_harden, "◆ 肌肉硬化"), (sk_gene_bone, "◆ 骨骼重构"),
    (sk_gene_power, "◆ 力量爆发"), (sk_gene_regen, "◆ 再生"), (sk_gene_berserk, "◆ 本能爆发"),
    (sk_gene_psych, "◆ 精神冲击"), (sk_gene_wall, "◆ 念动壁"), (sk_gene_scan, "◆ 心灵扫描"),
    (sk_gene_link, "◆ 精神链接"), (sk_gene_field, "◆ 规则领域"), (sk_gene_timesense, "◆ 时间感"),
    (sk_gene_fold, "◆ 空间折叠·初级"), (sk_gene_liber, "◆ 基因锁完全解放"), (sk_gene_apex, "◆ 规则承载"),
    (skx_gene_sense, "◆ 锁定反应"), (skx_gene_overclock, "◆ 短暂过载"), (skx_gene_regen, "◆ 锁率再生"),
]);
skill_cat!(CAT_BLOOD, [
    (sk_vamp_frenzy, "◆ 血之狂潮"), (sk_vamp_mist, "◆ 血雾遁形"), (skx_vamp_shadow, "◆ 暗影之潮"),
    (skx_vamp_drain, "◆ 生命汲尽"), (skx_vamp_coffin, "◆ 血棺沉眠"), (sk_wolf_rend, "◆ 狂暴撕咬"),
    (skx_wolf_howl, "◆ 苍狼长啸"), (skx_wolf_primal, "◆ 野性直觉"), (sk_zuwu_iron, "◆ 祖巫不灭身"),
    (skx_zuwu_maul, "◆ 祖巫撼地"), (skx_zuwu_totem, "◆ 祖巫图腾"), (sk_zhanshi_light, "◆ 圣职者之誓"),
    (skx_zhanshi_bless, "◆ 圣职祝福"), (skx_gauss_sync, "◆ 纳米同频"),
]);
skill_cat!(CAT_HOLY, [
    (sk_holy_water, "◆ 圣水祝福"), (sk_holy_burst, "◆ 圣光术"), (sk_holy_purify, "◆ 圣印驱散"),
    (sk_holy_veil, "◆ 圣光护佑"), (skx_holy_smite, "◆ 圣言冲击"), (skx_holy_flash, "◆ 圣光闪"),
    (skx_holy_bless, "◆ 圣辉祝福"), (skx_holy_aura, "◆ 圣光领域"), (skx_holy_heal, "◆ 柔和圣光"),
    (skx_holy_judge, "◆ 圣裁之刃"), (skx_holy_purge, "◆ 至净驱散"), (skx_holy_resurrect, "◆ 祈愿圣歌"),
]);
skill_cat!(CAT_TECH, [
    (sk_tech_scanner, "◆ 战术扫描"), (sk_tech_nano, "◆ 纳米修复"), (sk_tech_emp, "◆ 电磁脉冲"),
    (sk_tech_ballistic, "◆ 弹道预判"), (skx_tech_beacon, "◆ 信标锁定"), (skx_tech_drone, "◆ 哨戒无人机"),
    (skx_tech_emp_grenade, "◆ EMP 手雷"), (skx_tech_gauss, "◆ 高斯点射"), (skx_tech_nanocoat, "◆ 纳米镀膜"),
    (skx_tech_overcharge, "◆ 过载强化"), (skx_tech_quantum, "◆ 量子扰动"), (skx_tech_rail, "◆ 电磁弹道"),
]);
skill_cat!(CAT_NT, [
    (sk_nt_precog, "◆ 灾祸预知"), (sk_nt_empathy, "◆ 读心感应"), (sk_nt_telekinetic, "◆ 念动力场"),
    (sk_nt_push, "◆ 精神冲击"), (skx_nt_bp, "◆ 黑科技蓝图"), (skx_nt_omen, "◆ 凶兆推演"),
    (skx_nt_read, "◆ 恶意感应"), (skx_nt_seeker, "◆ 预感法则"), (skx_nt_micro, "◆ 微操弹道"),
    (skx_nt_song, "◆ 灵魂之歌"), (skx_nt_sympathy, "◆ 共鸣同调"), (skx_nt_hyperspace, "◆ 超维视界"),
    (skx_nt_shield, "◆ 思维力场"),
]);
skill_cat!(CAT_MEME, [
    (sk_meme_mark, "◆ 道德印记"), (sk_meme_link, "◆ 心灵链接"), (sk_meme_seal, "◆ 概念封锁"),
    (skx_meme_mindshield, "◆ 心膜铸盾"), (skx_meme_echo, "◆ 印记回响"), (skx_meme_forget, "◆ 遗忘低语"),
    (skx_meme_cursefeed, "◆ 诅咒回收"), (skx_meme_narrate, "◆ 命名即缚"), (skx_meme_wyrm, "◆ 概念缠绕"),
    (skx_meme_overwrite, "◆ 覆盖叙事"),
]);
skill_cat!(CAT_UTIL, [
    (sk_util_inspect, "◆ 洞察侦查"), (sk_util_bandage, "◆ 紧急包扎"), (sk_util_morale, "◆ 振奋咆哮"),
    (sk_util_antidote, "◆ 净化血清"), (skx_util_scout, "◆ 敌情摸底"), (skx_util_spotter, "◆ 鹰眼标记"),
    (skx_util_rally, "◆ 号令集结"), (skx_util_fieldmed, "◆ 战场急救"), (skx_util_heirloom, "◆ 祖传医术"),
    (skx_util_rations, "◆ 干粮补给"), (skx_util_insect, "◆ 尸味掩体"), (skx_util_retreat, "◆ 战术撤退"),
    (skx_util_trap, "◆ 绊索陷阱"),
]);
shop_cat!(CAT_SHOP, [
    (wp_katana, "◆ 精锻武士刀", "刀身如镜，映出你身后那个不属于这个世界的影子。每次拔刀，都在问：这次谁会先倒。"),
    (wp_gauss, "◆ 高斯手枪", "线圈嗡鸣的一瞬，时间仿佛被拉长。枪里装满了「再也回不去」的念头。"),
    (wp_emi, "◆ 电磁脉冲枪", "它让人短暂失控——可到最后，最先失控的往往是持枪的人。"),
    (wp_holy_sword, "◆ 圣裁十字剑", "剑锷雕着十字，剑锋冷得没有半点慈悲。所谓圣裁，不过是谁的信念先撑到对方倒下。"),
    (wp_cu_ju, "◆ 问心·青锋剑", "剑名问心，问的是持剑人一句话都不敢答的旧事。"),
    // 武器扩容（第 2 批）
    (wp_scythe_pobing, "◆ 破军重镰", "每道豁口都是上个主人一次死里逃生。它收割的不只是命，还有你最后的犹豫。"),
    (wpn_bloodsaber, "◆ 血戮剑", "剑身饮过太多血，挥动时像一群亡魂在替你清点战果。越砍，心越冷。"),
    (wpn_zhuai_jianpan, "◆ 诛仙剑阵盘", "一方残破剑阵盘，封着道门前辈的半截执念，展开的刹那万剑只为「诛」一个名字。"),
    (wp_quantum_annihil, "◆ 量子湮灭刀", "正反物质在刀锋相撞，把「存在」撕成碎片。它连持刀人都怕被湮灭。"),
    (wp_gravity_collapse, "◆ 引力坍缩炮", "一颗微缩的星在炮口崩塌，连光都逃不出去。开炮的代价，是离回家又远一个光年。"),
    (wpn_shihun_fan, "◆ 噬魂幡", "幡上符咒会呼吸，猎猎作响时似有万道目光盯着你。它吞别人的魂，也啃你的理智。"),
    (wpn_taixu_godsaw, "◆ 太虚神剑", "剑出太虚，无门无派，只认一个「斩」字。它缺的是肯把名字从生死簿划掉的人。"),
    (wpn_rail_sniper, "◆ 电磁轨道狙击枪", "三发轨道弹，三次隔着千山万水取人性命。瞄准镜里，你看见自己冷得发硬的眼睛。"),
    (wpn_nano_whip, "◆ 纳米切割鞭", "亿万纳米刃在鞭梢聚集，落下的是被切到分子级的绝望。顺手得让人上瘾。"),
    (wpn_causality_sword, "◆ 因果律护身剑", "它不斩肉身，斩的是「因」。护你一次，就从你命里借走一段还不起的「果」。"),
    // 武器扩容（第 3 批 · 动漫系）
    (wp_zanjingdao_he, "◆ 斩魄刀·卍解", "灵魂解放后的终解，刀灵与持刀者共鸣，每次挥斩都带着死神的低语。"),
    (wp_ruyibang, "◆ 如意金箍棒", "重一万三千五百斤，却只愿听一句「大」。它等的主人，早把自由押给了谁。"),
    (wp_excalibur_holy, "◆ 誓约胜利之剑", "剑身盛满星辰，出鞘便要完成一场无人记得的誓约。只剩这束光，替你照着回去的路。"),
    (wp_beam_saber, "◆ 光束军刀", "高热光束凝成刀刃，嗡鸣像一句警告。它从不原谅举棋不定的人。"),
    (wp_zanyue, "◆ 斩月大刀", "刀身比人还宽，重得连月光都压弯半截。挥它的人，须连自己的软弱一起斩下。"),
    (wp_qianbenying, "◆ 千本樱·散舞", "刀碎成千万片，像一场不散的樱花雨。每一片都美，每一片也都致命。"),
    (wp_niutou_ren, "◆ 牛头虚刃", "虚与死神之间有一道谁也不敢跨的门，这柄刃曾卡在门缝里替你守着。"),
    (wp_wang_zhicai, "◆ 王之财宝·宝具齐射", "黄金之门洞开，宝具如雨齐射。王者的财宝里，唯独没有一样能买回「活着」。"),
    (wp_guaili_jian, "◆ 乖离剑·EA", "开天辟地之剑，剑刃旋转时，世界像一本被重新装订的书。它只差遣你为它翻页。"),
    (wp_long_ji, "◆ 龙骑兵系统", "十二架浮游炮绕你盘旋，像驯服的流星。它们替你死，却替不了你回家。"),
    (wp_death_scythe_q, "◆ 死神镰刀·终焉", "镰尖所指，一切轮回都要收场。它不是结束，是那个终于肯来接你的最后一页。"),
    (wp_diyang_zhandou, "◆ 迪迦光之刃", "光凝聚成刃，划破黑暗的一瞬像把希望递到人间。可光散后，你仍要独自走完。"),
    (wp_shoujia_qiluo, "◆ 奇犽·电光疾影", "雷霆缠身，身如疾影。你跑得比闪电还快，却甩不掉还在追你的过去。"),
    // 武器扩容（第 3 批 · 仙侠小说系）
    (wp_feijian_qingyun, "◆ 青云飞剑", "御剑青云间，一剑破长风。心却回不到那座再也不敢回望的山门。"),
    (wp_shenbing_ling, "◆ 神兵·灵天刃", "刃含一缕天灵，出鞘便有云气相随。它只肯为「还愿」的人染血。"),
    (wp_zhanxian_feidao, "◆ 斩仙飞刀", "刀出如电，专斩「仙」字辈。可惜再快，也快不过生死簿上那笔已落的墨。"),
    (wp_fantian_yin, "◆ 翻天印", "印起翻山，印落覆海。此印无名，所以不敬。"),
    (wp_zhuxian_sijian, "◆ 诛仙四剑·合一", "四剑归一成一念。合剑的人，先要合住心里那把「仙」。"),
    (wp_xuanyuan_jian, "◆ 轩辕剑·人皇", "人皇之兵，山河之重。若不能替苍生担当，它只是一件王者玩具。"),
    (wp_pangu_fu, "◆ 盘古开天斧", "开天辟地的那一斧还留着半声回响。谁握着，谁就得扛起再造天地的妄想。"),
    (wp_kongtong_yin, "◆ 崆峒印", "上古神印，落处自有皇道气运镇压。你握住的是一个亡了国度的最后一点龙气。"),
    (wp_taiji_tu, "◆ 太极图", "阴阳流转，把敌我的力气搅成一股，最后还你一个不偏不倚的「空」。"),
    (wp_shanhe_shetu, "◆ 山河社稷图", "一图藏尽山河。你不知是自己进了画，还是画一直没放你出来。"),
    (wp_xihe_zhen, "◆ 曦和神针", "一针挑破黑夜，曦光顺针尖淌进来。它记住的是破晓前那个没敢睡去的你。"),
    // 武器扩容（第 3 批 · 科幻系）
    (wp_gauss_rifle, "◆ 高斯步枪", "等你看清弹孔，命已交了出去。它不信运气，只信初速。"),
    (wp_particle_cannon, "◆ 粒子炮", "一束高能粒子足以让目标蒸发成基本粒子。你听见的是宇宙对渺小的嘲笑。"),
    (wp_electromag_gun, "◆ 电磁加速炮", "磁轨上的弹丸记得每次加速，像你记得每次没敢开枪的瞬间。"),
    (wp_plasma_dagger, "◆ 等离子刺刃", "袖里藏着一小截太阳，连空气都被烧得发白。最适合避不开的近处。"),
    (wp_antimatter_round, "◆ 反物质湮灭弹", "两发。正反物质相遇即湮灭，把「存在」从坐标上抹去。"),
    (wp_orbital_gun, "◆ 轨道天基枪", "真正的杀意来自几万公里外的轨道。你抬头的那一刻，早已逃不脱它的准星。"),
    (wp_laser_sword, "◆ 纯激光剑", "没有重量，只有一道不肯熄灭的光。它切开的是黑暗里最后一点犹豫。"),
    (wp_nano_blade, "◆ 纳米蜂巢剑", "亿万纳米机器组成的蜂巢，随你心意重组。是某种不祥的顺从。"),
    (wp_phase_weapon, "◆ 相位扰动枪", "它打的不是实体，是「相位」。命中的目标会短暂与现实脱节。"),
    (wp_warpspeed_round, "◆ 曲速托卡马克枪", "弹头蜷着一团曲速场雏形，出膛即扭曲一瞬时空。只有死亡比光更快抵达。"),
    (wp_rail_pistol, "◆ 微型轨道手枪", "缩到掌心的轨道炮，藏得住杀意，藏不住扣扳机时那一下迟疑。"),
    // 武器扩容（第 3 批 · 魔幻系）
    (wp_arcan_staff, "◆ 奥术增幅法杖", "杖顶奥术之核把你的咒语放大成灾。它只问你还记不记得为何而战。"),
    (wp_madoushu_grimoire, "◆ 禁忌魔导书", "书页没有字，只有洗不掉的旧血。每翻一页，就有一段被抹去的知识爬进你眼睛。"),
    (wp_xianzhe_zhi_shi, "◆ 贤者之石刃", "贤者之石锻刃，能把触碰的一切点成黄金——包括你不舍失去的人。"),
    (wp_dragon_lance, "◆ 龙枪·屠龙", "枪尖嗜过龙血，至今带着不肯散去的腥。屠龙者终成恶龙。"),
    (wp_shuang_zhi_aisang, "◆ 霜之哀伤", "剑身冰封亡者的低语，挥动时霜花顺刃而下。每吸一条命，就多一道认不出的脸。"),
    (wp_leidun_chui, "◆ 雷神之锤", "唯有配得上它的人方能举起。雷声落下，藏着神祇未尽的孤独。"),
    (wp_sheng_jian_mj, "◆ 光之圣剑", "纯粹圣光凝成，照得见你所有的谎。被刺中的是你还想伪装的真相。"),
    (wp_mo_jian_zhl, "◆ 诅咒魔剑·噬主", "它把敌人的生气转给你，也把主人的魂魄一点点抽走。你我不是它第一顿。"),
    (wp_lieyan_jian, "◆ 烈焰之剑", "剑上跃着不熄的火舌，像一场钉死在刃上的舞蹈。它暖得了黑夜，也烧得穿誓言。"),
    (wp_hanbing_gong, "◆ 寒冰精灵长弓", "弓弦凝着寒霜，落下一串冰晶。它射出的是一种被冻住的、说不出的话。"),
    (wp_zhigu_shenju, "◆ 翡翠贤杖·自然", "藤蔓盘杖，新叶四季不凋。它把根须伸进你的影子，替你记着走过的泥泞。"),
    // 武器扩容（第 3 批 · 武侠系）
    (wp_yitian_jian, "◆ 倚天剑", "倚天不出，谁与争锋。剑锋所向，是乱世里最后一点不肯折断的傲气。"),
    (wp_tulong_dao, "◆ 屠龙宝刀", "宝刀屠龙，号令天下。刃上那道豁口，是上一代英雄用命换来的半句传说。"),
    (wp_dagou_bang, "◆ 打狗棒·逍遥", "见棒如见丐帮，从来不争输赢，只争一个「自在」。可这世间，自在最贵。"),
    (wp_xuantie_jian, "◆ 玄铁重剑", "重剑无锋，大巧不工。它只要你有扛起它的力气，和放得下的心。"),
    (wp_lixiao_feidao, "◆ 小李飞刀", "例无虚发，例不轻出。那把刀在你看不见的因果里，早已替你把命算好。"),
    (wp_liumai_jian, "◆ 六脉神剑", "以气为剑，指走六脉。少商一出，剑气破空，不留刃伤，只留清到骨髓的凉意。"),
    (wp_beiming_jian, "◆ 北冥神功·吸星剑", "北冥吞海，吸星纳力。它把内力与生气一并吸入剑身，只问你吃进去的，吐得出来吗？"),
    (wp_dugu_jiujian, "◆ 独孤九剑", "专破天下剑法，剑到之处，招招都是破绽。独孤求败之后，再无人配这一剑。"),
    (wp_miwu_shenzhao, "◆ 移花接玉掌刃", "掌若移花，刃如接玉。把别人的力借来还回去，往往比来势更狠。"),
    (wp_jinhe_zhang, "◆ 降龙十八掌", "亢龙有悔，盈不可久。最要命的，永远是收掌时那一瞬的「悔」。"),
    (wp_tianmen_yuanshang, "◆ 天外飞仙·剑遁", "一瞬千年，剑自天外而来，人已远遁千里。你只看见剑光，看不见转身的人。"),
    (wp_jinghuo_zhang, "◆ 降妖真火扇", "一扇生风，二扇生火，三扇焚尽妖邪。扇上那簇真火，专烧不该留在人间之物。"),
    (wp_jinlong_dao, "◆ 金蛇缠丝软剑", "软剑如丝，可缠可刺，藏着一门从不外传的狠。它最懂，柔情也能杀人。"),
    (wp_zhenwu_baojian, "◆ 真武七星剑", "剑分七星，应北斗之数。剑起时，七星方位自有天罡之力相护。"),
    (gear_police_vest, "◆ 警用防弹背心", ""),
    (gear_kevlar, "◆ 凯夫拉防弹衣", ""),
    (gear_elven_cloak, "◆ 精灵斗篷", ""),
    (gear_nano_vest, "◆ 纳米作战服", ""),
    (access_strength_ring, "◆ 蛮力指环", ""),
    (access_agility_boots, "◆ 追风靴", ""),
    (access_qi_belt, "◆ 聚气腰带", ""),
    // 护甲/饰品扩容（第 2 批）
    (gear_adamant_cuirass, "◆ 精金胸甲", ""),
    (gear_void_leak, "◆ 虚无织物衣", ""),
    (gear_zero_absorb, "◆ 绝对零度护甲", ""),
    (gear_sanctum_plate, "◆ 圣域板甲", ""),
    (access_hades_cloak, "◆ 幽冥披风", ""),
    (access_will_anchor, "◆ 意志锚链", ""),
    (access_tianting_belt, "◆ 天庭灵气腰带", ""),
    (access_nano_tech_shield, "◆ 纳米护盾核心", ""),
    // 护甲/饰品扩容（第 3 批 · 战甲/圣衣/机甲/法袍）
    (gear_shengclothes_shooter, "◆ 射手座黄金圣衣", ""),
    (gear_nano_mecha_suit, "◆ 纳米战甲·机甲", ""),
    (gear_leidun_armor, "◆ 雷霆铠甲", ""),
    (gear_longlin_jia, "◆ 龙鳞逆甲", ""),
    (gear_shengguang_fapao, "◆ 圣光法袍", ""),
    (gear_tian_yi, "◆ 神炁天衣", ""),
    (gear_azote_panzhi, "◆ 奥术织纹布甲", ""),
    (gear_wh_warframe, "◆ 战争框架·重装", ""),
    (gear_ice_dragon_scale, "◆ 冰霜巨龙鳞甲", ""),
    (gear_shadow_cloak_armor, "◆ 暗影皮甲", ""),
    // 护甲/饰品扩容（第 3 批 · 饰品/护符）
    (access_divine_ring, "◆ 神圣婚戒", ""),
    (access_frost_amulet, "◆ 冰封护符", ""),
    (access_lightning_core, "◆ 雷电核心吊坠", ""),
    (access_soul_bind, "◆ 魂之锁结", ""),
    (access_dragon_seal, "◆ 龙纹玉玺", ""),
    (access_saint_bracelet, "◆ 圣斗士手环", ""),
    (access_truth_seeker, "◆ 求真透镜·感知", ""),
    (access_tianxuan_jing, "◆ 先天玄光镜", ""),
    (access_wuxin_shaer, "◆ 无心神砂·定魂", ""),
    (access_devil_contract, "◆ 恶魔契约徽记", ""),
    // 护甲/饰品扩容（第 3 批 · 低中 tier 补）
    (gear_iron_warplate, "◆ 玄铁重甲", ""),
    (gear_silk_robe, "◆ 唐锦法衣", ""),
    (gear_black_tech_suit, "◆ 黑色科技紧身衣", ""),
    (gear_holy_plate_armor, "◆ 圣骑士板甲", ""),
    (access_silver_cross, "◆ 秘银圣十字", ""),
    (access_moon_pendant, "◆ 月华坠饰", ""),
    (access_qi_obsidian, "◆ 聚灵黑曜石", ""),
    (access_werewolf_claw, "◆ 狼王獠牙挂坠", ""),
    (access_ice_heart, "◆ 冰晶之心", ""),
    (access_phoenix_feather, "◆ 涅槃凤羽", ""),
    (cu_bab_hudun_fu, "◆ 护体符印", "一枚刻进肉里的旧符，危急时自会绽开护主。符纹越磨越淡，像你越来越短的命。"),
    (cu_bab_benming_fejian, "◆ 本命飞剑·青锋", "以心血温养的本命飞剑，剑在人在，剑亡身殒。它是你唯一不许背弃的「我」。"),
    (cu_bab_hunyuan_lu, "◆ 混元炉", "一炉混元，炼化万物浊气返上清。炉中焰温吞，却足以把心头的魔念熬成安定。"),
    // 法宝扩容（第 2 批）
    (tr_zhuxian_calendar, "◆ 诛仙剑意图", "一方残卷，载尽诛仙剑阵的杀伐之机。执图之人，终要拿自己的心去喂它。"),
    (tr_blood_banner, "◆ 血煞战旗", "战旗浸透血煞，扬起时千军万马的吼声都往你一人身上压来。旗不倒，命不绝。"),
    (tr_taixu_shield, "◆ 太虚玄光镜", "镜面悬着太虚，映出的不是你的脸，是下一劫将至的预兆。以虚挡实，护你安稳。"),
    (tr_shenlei_pendant, "◆ 神雷辟邪佩", "雷纹古佩，邪祟近身时自会噼啪作响，似有雷神在侧替你喝退不干净之物。"),
    (tr_danxin_mirror, "◆ 锻心明镜", "镜可锻心，照见的杂念都被磨成澄明。你越恐惧，镜面越亮，也就越无处可逃。"),
    (tr_undo_pillowstone, "◆ 逆转生死盘", "能拨回死生的罗盘，把你就地从鬼门关拽回。可有些代价，连它也不肯替你算清。"),
    // 法宝扩容（第 3 批 · 动漫/小说 · slot0 本命攻）
    (tr_shengbei_shengtian, "◆ 圣杯·神圣权柄", "杯中盛满「愿」，也盛满「代价」。谁许愿，谁就得先填进最珍贵的一块命。"),
    (tr_mo_jie_jiujie, "◆ 魔戒·至尊戒", "至尊魔戒，统御诸戒。戴上那一刻，你分不清是它戴着你，还是你戴着它。"),
    (tr_guaili_yuantu, "◆ 乖离剑·原质图谱", "图谱记载世界最初的原质。看久了，你会忘记自己只是后来者。"),
    (tr_yinyang_jing, "◆ 阴阳宝镜", "镜分阴阳，照得见生也照得见死，照得最清的，是你此刻站在界线上的模样。"),
    (tr_bahuang_longyin, "◆ 八荒龙印", "八荒龙气凝成，戾气与王气同在。执印者威加四海，也终将被四海吞没。"),
    (tr_leishen_xianglu, "◆ 雷神锤·神威", "神威一锤，雷霆万钧。它只认配得上它的人，你若退意，锤便重得举不起来。"),
    // 法宝扩容（第 3 批 · slot1 护身防）
    (tr_mo_jing_xianshi, "◆ 贤者之镜·洞察", "镜名洞察，照见都是本相。它不怜你，只把真相摊在眼前让你无处躲闪。"),
    (tr_duantou_mojing, "◆ 魔镜·破碎之握", "镜已破碎，裂痕里伸出谁的手也看不清。它护身，却总想把什么从镜那头拽来。"),
    (tr_ahnidun_shield, "◆ 埃葵斯神盾", "神盾嵌着美杜莎之颅。它挡下刀剑，却挡不住那个直视它、不再移开的目光。"),
    (tr_longnei_xiangu, "◆ 龙内丹", "一枚温热的龙丹，藏着千年道行。它把龙的孤寂，一并种进你的气海。"),
    (tr_shenlingsan_yu, "◆ 神陵山玉符", "神陵山传下的玉符，刻着安魂云纹。贴身戴着，连噩梦都不轻易近前。"),
    (tr_xuanhuang_taibao, "◆ 玄黄太宝", "天地玄黄，孕此至宝。它浑朴无华，只在生死之际才显一线护你的神异。"),
    // 法宝扩容（第 3 批 · slot2 辅助）
    (tr_xianzhe_ziliao, "◆ 贤者之石·点金", "贤者之石，点石成金。它成金的不是石头，是你所剩无几的、想活下来的念。"),
    (tr_sisin_luandao, "◆ 死神镰刀·摄魂", "镰刀专摄生魂，挥过之处亡者皆静。它不夺命，只替死神把迷路的魂引回原处。"),
    (tr_longzu_shengyi, "◆ 龙珠·七龙珠", "集齐七颗便能召唤神龙许愿。可每个轮回的愿望，都早在你开口前被谁先许过了。"),
    (tr_mishen_zhi_tong, "◆ 三千世界神瞳", "一瞳可窥三千世界。看得越多，越分不清哪个才是你该回去的那一个。"),
    (tr_tianlu_pa, "◆ 天书残卷", "天书只余残卷，字迹半漫。就这半页，也不知困住了多少窥天机、回不了头的人。"),
    (tr_huanjing_luo, "◆ 幻境罗盘", "指针不指南北，只指人心最脆处。转一圈，前路是幻，也未必不是生天。"),
    (tr_shengwg_huanghun, "◆ 圣域星芒", "星芒自圣域坠落，像一枚不肯熄灭的星火。照不亮整片黑夜，却护你走出这一段。"),
    (tr_mengjing_guiji, "◆ 梦境诡计宝匣", "匣中藏着梦的诡计，开合间能偷换一次必死之场的因果。醒来时，别问梦去了哪里。"),
    (item_medkit, "◆ 强效医疗包", ""),
    (item_bandage, "◆ 紧急绷带", ""),
    (item_sedative, "◆ 镇静剂", ""),
    (item_holy_water, "◆ 圣水", ""),
    (item_silver_bullet, "◆ 银弹", ""),
    (item_grenade, "◆ 燃烧手雷", ""),
    (item_quzhen_fu, "◆ 驱邪符", ""),
    (item_jiezhou_fu, "◆ 解咒符", ""),
    (it_qixue_dan, "◆ 气血丹", ""),
    (it_soul_shard, "◆ 灵魂碎片", ""),
    (it_cross, "◆ 圣徽", ""),
    // 强化石 / 新材料（第 2 批）
    (it_enhance_stone, "◆ 普通强化石 · 强化 +1", ""),
    (it_enhance_stone_hi, "◆ 高级强化石 · 强化 +2", ""),
    (it_em_core, "◆ 电磁炮核心", ""),
    (it_blood_essence, "◆ 血族精血", ""),
    (it_treasure_frag, "◆ 法宝碎片", ""),
]);

/* ============================================================= */
pub static SCENES: &[SceneDef] = &[

/* ---- 开放世界 NPC 对话（蜂巢语境） ---- */
SceneDef {
    id: "s_world_zhangjie", bg: Some("img_train.png"), loc: Some("站台 · 张杰"),
    mood: "calm", speaker: Some("张杰"), voice: Some("vo_zhangjie_world.wav"),
    text: TextSpec::Dyn(|st| {
        if st.flag("redqueen_cleared") {
            "张杰靠在站台的立柱上，指间夹着半支烟，烟雾在应急红灯下袅袅升起。\n\n「听说你把红后的四道题全答对了？」他挑了挑眉，「可以啊。上一批新人，连第一道都没撑过去。」\n\n他碾灭烟头：「记住——在轮回里，脑子比子弹值钱。下一部片子，你大概会怀念这个蜂巢。」".to_string()
        } else if st.flag("B2") {
            "张杰斜靠在站台边，目光扫过你腰间的武器。\n\n「激光通道的示意图看过了？聪明。」他低声道，「那玩意儿有三波。第一波贴地，第二波交叉，最后一波——整个通道都是网。只有一根承重梁后面是安全的。」\n\n「别急着谢。这只是我上次轮回用命换来的情报。」".to_string()
        } else {
            "站台边缘，张杰抱臂站着，看着你们这群新人。\n\n「第一次进蜂巢？」他问，「记住三条：别掉队，别乱碰，听见红后的声音——跑。」\n\n「想要活命的情报，去找那台红后终端。它能答你的问题……只要你答对它的题。」他咧嘴笑了笑，「引导者的职责到此为止。剩下的，靠你自己。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_world_rain", bg: Some("img_train.png"), loc: Some("站台 · 蕾恩"),
    mood: "danger", speaker: Some("蕾恩"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("C") {
            "蕾恩坐在站台的台阶上，小臂的齿痕已经结痂。她看见你，扯了扯嘴角。\n\n「那支肾上腺素……是你从厨房翻出来的吧？」她低声说，「谢了。我欠你一条命。」\n\n她站起来，拍了拍裤子：「走吧。这地方多待一秒，就多一分变成那玩意的风险。」".to_string()
        } else if st.flag("savedKaplan") {
            "蕾恩抱着枪站在站台边缘警戒，听见脚步声回头看了你一眼。\n\n「你救卡普兰那一下，干净。」她顿了顿，「在这种地方，敢第一个冲上去的人不多了。」\n\n她朝走廊尽头扬了扬下巴：「厨房里有急救箱。如果后面有人受伤……你会用得上。」".to_string()
        } else {
            "蕾恩正在检查弹匣，头也不抬。\n\n「新人？」她往旁边挪了挪，「跟紧我。这地方的丧尸扑人前有前摇——记住那个停顿，那就是你砍它的时机。」\n\n她抬眼：「厨房里有急救箱。队伍里有人受伤的话，别吝啬。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_world_kaplan", bg: Some("img_redqueen.png"), loc: Some("机房走廊 · 卡普兰"),
    mood: "danger", speaker: Some("卡普兰"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("redqueen_cleared") {
            "卡普兰抱着便携终端，屏幕蓝光映亮他紧张的脸。\n\n「你……你居然把红后的四道题全答对了？」他难以置信地摇头，「我在它系统里混了这么久，都不敢说摸透了它的逻辑。」\n\n他压低声音：「激光通道的安保，我刚才远程看了下——如果你全对，红后应该已经把它调到最低杀伤模式了。走位别乱，贴着承重梁。」".to_string()
        } else if st.flag("savedKaplan") {
            "卡普兰在机房外调试终端，听见脚步声抬头，眼神里带着感激。\n\n「B餐厅那一下……谢了兄弟。」他搓了搓手，「我在看红后的安保协议。那玻璃通道有机关——三波激光，一波比一波密。」\n\n他压低声音：「最密的第三波，整个通道都是网，只有通道中段那根凸出的承重梁后面是死角。」".to_string()
        } else {
            "卡普兰蹲在机房外的配电箱旁，手指翻飞地敲着终端。\n\n「嘘——我在黑红后的监控。」他头也不抬，「这鬼地方到处都是眼睛。那台终端……」他朝机房努了努嘴，「红后能回答问题，但要答它的题。答对了，它给权限。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_world_yihao", bg: Some("img_corridor.png"), loc: Some("B区走廊 · 一号"),
    mood: "danger", speaker: Some("一号"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("no1_dead") {
            "B区走廊空荡荡的。一号的位置，只剩下他靠过的墙。\n\n蕾恩站在几步外，声音很轻：「他把他该做的做完了。剩下的……轮到我们。」".to_string()
        } else {
            "一号站在走廊拐角，枪口朝下，目光扫过你。\n\n「新人，跟紧队形。」他的声音像砂纸，「红后在看着这条走廊。别碰任何东西，别掉队。」\n\n他顿了顿：「如果等会儿走散了——记住，往机房方向撤。那是唯一活着出去的路。」".to_string()
        }
    }),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_world_back", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: None,
        card: |_st| crate::state::Card {
            title: "返回探索".into(), good: true,
            body_html: "<p>对话结束。你回到蜂巢的走廊里。</p>".into(),
            buttons: vec![("继 续 探 索".into(), "__back_to_world__".into())],
            voice: None,
        },
    }),
},

/* ---- 序章 ---- */
SceneDef {
    id: "s_office", bg: Some("img_office.png"), loc: Some("现实世界 · 深夜办公室"),
    mood: "calm", speaker: None, voice: Some("vo_question.wav"),
    text: TextSpec::Static(&[
        "晚上十一点四十，写字楼里只剩你一个人。\n\n报表、加班、房租……日子像一台不会停的复印机，把你的人生一张张印成灰色。",
        "你揉了揉眼睛，正准备关机——\n\n显示器忽然自己亮了。漆黑的屏幕中央，浮现出两行<b>猩红色的字</b>：",
        "<em>「想明白生命的意义吗？想真正的……活着吗？」</em>\n\n字的下面是一个输入框，光标一跳一跳，像心脏的搏动。\n\n你明明没有联网。机箱的风扇声不知何时停了，整个世界安静得可怕。",
    ]),
    choices: &[
        ChoiceDef { label: "输入 YES", sub: "指尖比理智更快", cond: None, effects: &NO_EFF, route: Route::To("s_yes") },
        ChoiceDef { label: "拔掉电源，输入 NO", sub: "这一定是恶作剧", cond: None, effects: &NO_EFF, route: Route::To("e_mediocre") },
        ChoiceDef { label: "抓起手机想报警", sub: "屏幕上的红字突然开始蠕动……", cond: None, effects: &NO_EFF, route: Route::To("s_office_phone") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_office_phone", bg: Some("img_office.png"), loc: Some("现实世界 · 深夜办公室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你抓起手机——没有信号。日期在跳动：23:59…23:59…23:59…\n\n抬起头时，红字已经占满整块屏幕，像有生命一样缓缓呼吸：\n\n<em>「犹豫，也是一种回答。但机会，只给真正想活的人。」</em>"]),
    choices: &[
        ChoiceDef { label: "深吸一口气，输入 YES", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_yes") },
        ChoiceDef { label: "砸了电脑夺门而逃", sub: "", cond: None, effects: &NO_EFF, route: Route::To("e_mediocre") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_yes", bg: Some("img_office.png"), loc: Some("？？？？"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "回车键落下的瞬间，世界熄灭了。\n\n没有黑暗，没有声音，甚至没有「你」的感觉。仿佛过了一秒，又仿佛过了一万年——",
        "<em>「确认。轮回编号 0001，载入中……」</em>",
    ]),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 主神空间 ---- */
SceneDef {
    id: "s_nexus", bg: Some("img_nexus.png"), loc: Some("主神空间 · 半圆形广场"),
    mood: "calm", speaker: Some("？？？"), voice: Some("vo_rules.wav"),
    text: TextSpec::Static(&[
        "冷。最先恢复的是触觉——冰冷的金属地面，寒意顺着脊椎一路爬到后脑。\n\n你睁开眼：一座<b>巨大得超越常识的半圆形广场</b>。灰黑色金属墙壁向上延伸百米消失在昏暗中，穹顶洒下惨白光柱，照亮聚集在广场中央的十几个人影。广场大得空旷，脚步声在看不见的地方来回反弹，像有无数个你在同时走动。",
        "有人穿着西装，有人穿着睡衣，一个大学生模样的男孩在原地发抖，膝盖磕在金属地板上发出轻响。所有人脸上都是同样的茫然，没有一个人开口说话——广场安静得能听见彼此的呼吸。",
        "一个<b>寸头男人</b>走到你面前，军靴在空旷的广场上踩出沉稳的回音。黑色作战服，左脸一道浅疤，双手各挎一把银色手枪。他的眼神平静得不像人类——像是已经见过太多人在这里死去，多到连一丝波动都剩不下。",
        "「新人，欢迎来到<em>主神空间</em>。」他环视众人，目光在每个人脸上停留了一瞬，像在清点货物，「我叫<em>张杰</em>。你们可以理解为——老兵，或者引导者。」",
        "「规则很简单：主神把我们丢进一部又一部的恐怖片世界。完成主线任务、达成隐藏支线，就能获得<em>奖励点数</em>和<em>支线剧情</em>评价。活着回来的人可以用点数兑换任何东西——治疗、武器、血统、超能力，甚至复活死去的同伴。」他顿了顿，声音冷下去，「注意听：是<em>活着回来的人</em>。」",
    ]),
    choices: &[ChoiceDef { label: "「恐怖片世界……是什么意思？」", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus2", bg: Some("img_nexus.png"), loc: Some("主神空间 · 半圆形广场"),
    mood: "danger", speaker: Some("张杰"), voice: None,
    text: TextSpec::Static(&[
        "「意思是——」他咧嘴笑了笑，笑容里没有温度，「下一秒你们就会明白。」\n\n话音未落，一道血红色光柱从天而降笼罩全场，空气在光里扭曲、发烫。无数信息直接烙进脑海，像有人把烧红的铁字按进了你的颅骨：",
        "<em>【下一部恐怖片：《生化危机》】\n【主线任务：跟随佣兵小队进入蜂巢，关闭超级电脑「红后」，并活着回到地面。】\n【任务失败：抹杀。】</em>",
    ]),
    choices: &[
        ChoiceDef { label: "「等等！我们没有任何装备——」", sub: "恐慌开始在人群中蔓延", cond: None, effects: &NO_EFF, route: Route::To("s_weapon") },
        ChoiceDef { label: "强迫自己冷静，向张杰要武器", sub: "既然是规则，就有规则的用法", cond: None, effects: &NO_EFF, route: Route::To("s_weapon") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_weapon", bg: Some("img_nexus.png"), loc: Some("主神空间 · 武器架"),
    mood: "calm", speaker: Some("张杰"), voice: None,
    text: TextSpec::Static(&["张杰打了个响指。广场边缘的金属墙面无声裂开，露出幽黑的储物格，三件武器悬浮在半空，冷光沿着刃口缓慢爬行。\n\n「新人第一次进入，主神允许各选一件基础武器。」他抬了抬下巴，「别挑花眼——<em>能让你活下来的才是好武器。</em>拿不定主意的，想想你在蜂巢里最怕遇见什么。」"]),
    choices: &[
        ChoiceDef { label: "◆ 消防斧", sub: "沉重 · 高伤害 · 每一次挥砍都需要直面死亡的勇气", cond: None, effects: &[Eff::Weapon(Weapon::Axe)], route: Route::To("s_warning") },
        ChoiceDef { label: "◆ 9mm 手枪", sub: "6发子弹 · 远程稳定 · 打空后就是一块砖头", cond: None, effects: &[Eff::Weapon(Weapon::Gun)], route: Route::To("s_warning") },
        ChoiceDef { label: "◆ 军用刺刀军刀", sub: "迅捷 · 连击 · 需要贴身搏命", cond: None, effects: &[Eff::Weapon(Weapon::Sword)], route: Route::To("s_warning") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_warning", bg: Some("img_nexus.png"), loc: Some("主神空间 · 半圆形广场"),
    mood: "calm", speaker: Some("张杰"), voice: Some("vo_warning.wav"),
    text: TextSpec::Static(&[
        "选好武器的同时，张杰的声音沉了下来，广场上的白光仿佛也跟着暗了一度。\n\n「最后说一遍规矩，给我记进骨头里：<em>在恐怖片世界里死了，就是真的死了。</em>没有读档，没有重生。主神不会复活新人——一次都不会。」他顿了一下，「看清楚你们身边的脸。下一轮回来，他们可能已经不在。」",
        "「还有——」他看着你，疤痕在白光下泛着青色，「跟紧剧情人物。他们是主角，有主角气运护体。离他们太远的新人，通常活不到片尾字幕。」他最后扫了你们一眼，像在把每个人的脸刻进记忆，「别死得太难看。」\n\n白光暴涨，吞没一切——",
    ]),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_train") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 主神空间（P1 可玩世界）：兑换/张杰/光柱/复活祭坛 ---- */
SceneDef {
    id: "s_nexus_god", bg: Some("img_nexus.png"), loc: Some("主神空间 · 中央光柱"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你站到广场中央，脚下金属地面的寒意透过鞋底往上爬。<b>巨大到超越常识的白色光柱</b>自穹顶泻下，照亮圆台正中那个悬浮的光球。它散发着冷酷而均匀的辉光，不快不慢，像一台永远不会疲惫的机器——这就是「主神」。你看着它，它没有看你。",
        "<em>【主神空间】</em>的规则刻在光柱基座上：兑换、强化、复活队友，皆在你完成恐怖片轮回、赚取足够的奖励点数后，于此进行。",
    ]),
    choices: &[ChoiceDef { label: "（抬头注视光柱）……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange", bg: Some("img_nexus.png"), loc: Some("主神空间 · 兑换光球"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| text_exchange(st)),
    choices: &[
        ChoiceDef { label: "◆ 细胞活力强化（体质提升）", sub: "800 点 · 攻击 +5", cond: Some(cond_show_strengthen), effects: &NO_EFF, route: Route::Dyn(route_exchange_strengthen) },
        ChoiceDef { label: "◆ 基因锁第一阶段 · 自主开启权", sub: "2000 点 · 攻击+闪避+减伤", cond: Some(cond_show_gene), effects: &NO_EFF, route: Route::Dyn(route_exchange_gene) },
        ChoiceDef { label: "◆ 血统类：初级吸血鬼血统", sub: "3000 点 · 攻击吸血 + 敏捷", cond: Some(cond_show_vampire), effects: &NO_EFF, route: Route::Dyn(route_exchange_vampire) },
        ChoiceDef { label: "◆ 潜能开发 · 基因进化" , sub: "二阶6000B · 三阶12000A · 四阶22000A", cond: Some(cond_show_gene_cat), effects: &NO_EFF, route: Route::To("s_nexus_exchange_gene") },
        ChoiceDef { label: "◆ 高等血统 · 狼人/祖巫/圣职/天使/恶魔/龙/义体", sub: "4500B · 5500B · 3500C · 9000A · 9500A · 10000A · 7800B", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_blood") },
        ChoiceDef { label: "◆ 修真 · 破境与内功", sub: "练气→合道 · 无名剑诀 · 静心诀", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_xiu") },
        ChoiceDef { label: "◆ 科技 · 纳米护盾模块", sub: "1800D · 护盾上限 +30", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_tech") },
        ChoiceDef { label: "◆ 技能秘藏", sub: "按流派兑换武学 / 基因 / 血统技 / 修真技", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_skill") },
        ChoiceDef { label: "◆ 道具铺", sub: "武器 · 护具 · 消耗 · 符卷 · 圣物", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_shop") },
        ChoiceDef { label: "◆ 合成工坊 / 武器强化", sub: "合成表 · 强化装配武器", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange_craft") },
        ChoiceDef { label: "返回主神广场", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_god") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_gene", bg: Some("img_nexus.png"), loc: Some("主神空间 · 基因进化光球"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        format!(
            "<p>光球内壁浮动着暗红色的纹路，基因深处的锁在震颤，像有什么东西在骨头里敲门。每解开一层，战斗本能便再放一分——而主神只负责收点，不负责你的代价。</p>\
             <p style='color:#888'>当前基因阶：<b style='color:#b9e8ff'>{}</b> 阶。评级满足方可解锁更高阶。</p>",
            crate::combat_data::gene_stage_of(st)
        )
    }),
    choices: &[
        ChoiceDef { label: "◆ 基因锁二阶 · 入微", sub: "6000 点 · 需 B 级", cond: Some(cond_gene2), effects: &NO_EFF, route: Route::Dyn(route_exchange_gene2) },
        ChoiceDef { label: "◆ 基因锁三阶 · 禁忌", sub: "12000 点 · 需 A 级", cond: Some(cond_gene3), effects: &NO_EFF, route: Route::Dyn(route_exchange_gene3) },
        ChoiceDef { label: "◆ 基因锁四阶 · 顿悟", sub: "22000 点 · 需 A 级", cond: Some(cond_gene4), effects: &NO_EFF, route: Route::Dyn(route_exchange_gene4) },
        ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_blood", bg: Some("img_nexus.png"), loc: Some("主神空间 · 高等血统光球"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "血统是轮回者最快的捷径，也是最沉重的枷锁——光球把各种血统的影像投在半空：狼啸、圣光、龙鳞、机械关节，一帧帧轮转。血统互斥，择一而终，选定之前主神不会催促，选定之后也无人反悔。评级满足方可兑换。",
    ]),
    choices: &[
        ChoiceDef { label: "◆ 狼人血统", sub: "4500 点 · 需 B · 攻+8 狂暴+10 减2", cond: Some(cond_werewolf), effects: &NO_EFF, route: Route::Dyn(route_exchange_werewolf) },
        ChoiceDef { label: "◆ 祖巫血脉", sub: "5500 点 · 需 B · 受击减10 高坦", cond: Some(cond_zuwu), effects: &NO_EFF, route: Route::Dyn(route_exchange_zuwu) },
        ChoiceDef { label: "◆ 圣光圣职者血脉", sub: "3500 点 · 需 C · SAN抗+8", cond: Some(cond_zhanshi), effects: &NO_EFF, route: Route::Dyn(route_exchange_zhanshi) },
        // 血统扩容（第 2 批）
        ChoiceDef { label: "◆ 天使血统", sub: "9000 点 · 需 A · SAN抗+12 减4", cond: Some(cond_angel), effects: &NO_EFF, route: Route::Dyn(route_exchange_angel) },
        ChoiceDef { label: "◆ 恶魔血统", sub: "9500 点 · 需 A · 攻+12 吸血6 狂暴+15", cond: Some(cond_demon), effects: &NO_EFF, route: Route::Dyn(route_exchange_demon) },
        ChoiceDef { label: "◆ 龙族血统", sub: "10000 点 · 需 A · 攻+6 受击减14", cond: Some(cond_dragon), effects: &NO_EFF, route: Route::Dyn(route_exchange_dragon) },
        ChoiceDef { label: "◆ 机械义体血统", sub: "7800 点 · 需 B · 攻+8 减8 闪+0.08", cond: Some(cond_cyberpro), effects: &NO_EFF, route: Route::Dyn(route_exchange_cyberpro) },
        // 血统扩容（第 3 批 · 动漫/小说）
        ChoiceDef { label: "◆ 赛亚人血统", sub: "9000 点 · 需 A · 攻+12 减4 狂暴+25", cond: Some(cond_saiyan), effects: &NO_EFF, route: Route::Dyn(route_exchange_saiyan) },
        ChoiceDef { label: "◆ 写轮眼血统", sub: "8200 点 · 需 B · 攻+6 闪+0.15", cond: Some(cond_sharingan), effects: &NO_EFF, route: Route::Dyn(route_exchange_sharingan) },
        ChoiceDef { label: "◆ 虚化血统", sub: "9800 点 · 需 A · 攻+14 吸血8", cond: Some(cond_hollow), effects: &NO_EFF, route: Route::Dyn(route_exchange_hollow) },
        ChoiceDef { label: "◆ 圣斗士血统", sub: "9500 点 · 需 A · 攻+8 减10 闪+0.05", cond: Some(cond_saint), effects: &NO_EFF, route: Route::Dyn(route_exchange_saint) },
        ChoiceDef { label: "◆ 死神血统", sub: "9000 点 · 需 A · 攻+12 减6 闪+0.04", cond: Some(cond_shinigami), effects: &NO_EFF, route: Route::Dyn(route_exchange_shinigami) },
        ChoiceDef { label: "◆ 灭却师血统", sub: "8800 点 · 需 A · 攻+16 减2", cond: Some(cond_quincy), effects: &NO_EFF, route: Route::Dyn(route_exchange_quincy) },
        ChoiceDef { label: "◆ 宇智波血脉", sub: "9000 点 · 需 A · 攻+10 吸血4 闪+0.06", cond: Some(cond_uchiha), effects: &NO_EFF, route: Route::Dyn(route_exchange_uchiha) },
        ChoiceDef { label: "◆ 千手血脉", sub: "8600 点 · 需 A · 减12 SAN抗+4", cond: Some(cond_senju), effects: &NO_EFF, route: Route::Dyn(route_exchange_senju) },
        ChoiceDef { label: "◆ 大筒木血脉", sub: "15000 点 · 需 S · 攻+14 吸血6 减12", cond: Some(cond_otsutsuki), effects: &NO_EFF, route: Route::Dyn(route_exchange_otsutsuki) },
        ChoiceDef { label: "◆ 鬼灭呼吸·日之呼吸", sub: "7600 点 · 需 B · 攻+12 减6 闪+0.03", cond: Some(cond_mitsurugi), effects: &NO_EFF, route: Route::Dyn(route_exchange_mitsurugi) },
        // 蜘蛛侠血统（蛛丝拉拽 · 世界地图机动）
        ChoiceDef { label: "◆ 蜘蛛侠血统", sub: "6800 点 · 需 B · 闪+0.08 攻+6 · 点击建筑喷射蛛丝拉拽", cond: Some(cond_spiderman), effects: &NO_EFF, route: Route::Dyn(route_exchange_spiderman) },
        ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_xiu", bg: Some("img_nexus.png"), loc: Some("主神空间 · 真元光球"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        let cur = if st.cultivation_stage == 0 { "凡体（未入道）".to_string() }
            else { crate::combat_data::cultivation_stage_cfg(st.cultivation_stage).map(|c| c.name.to_string()).unwrap_or("修士".into()) };
        format!(
            "<p>真元光球沉浮着淡金色的雾，像一枚悬浮的丹炉。引天地之灵入体，筑肉身之基——这是最古老的一脉，也是最漫长的一脉。破境需消耗大量点数与对应评级，主神不赊账。</p>\
             <p style='color:#888'>当前境界：<b style='color:#b9e8ff'>{cur}</b></p>",
        )
    }),
    choices: &[
        ChoiceDef { label: "◆ 突破下一层境界", sub: "CULTIVATION_STAGES 表价 · 需评级", cond: Some(cond_cultivable), effects: &NO_EFF, route: Route::Dyn(route_exchange_cultivation) },
        ChoiceDef { label: "◆ 内功 · 无名剑诀", sub: "1500 点 · 需 D · 真气上限 40", cond: Some(cond_art_wuming), effects: &NO_EFF, route: Route::Dyn(route_exchange_wuming) },
        ChoiceDef { label: "◆ 内功 · 静心诀", sub: "350 点 · 真气上限+20 · 心宁", cond: Some(cond_art_jingxin), effects: &NO_EFF, route: Route::Dyn(route_exchange_jingxin) },
        ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_tech", bg: Some("img_nexus.png"), loc: Some("主神空间 · 科技侧光球"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        format!(
            "<p>光球里涌出一团银灰色的纳米蜂群，在你面前旋聚成形，嗡鸣声细密而规律。它们能修补身体、凝成护盾，也会在你点数耗尽时毫不犹豫地散回光球——科技不问善恶，只认点数。</p>\
             <p style='color:#888'>当前纳米护盾：<b style='color:#b9e8ff'>{}/{}（上限）</b></p>",
            st.tech_shield, st.tech_shield_max
        )
    }),
    choices: &[
        ChoiceDef { label: "◆ 纳米护盾模块 Lv.L", sub: "1800 点 · 需 D · 上限+30 并回满", cond: None, effects: &NO_EFF, route: Route::Dyn(route_exchange_shield) },
        ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_skill", bg: Some("img_nexus.png"), loc: Some("主神空间 · 技能秘藏"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        format!(
            "<p>幽蓝光幕无声展开，一排排武学/血统技/修真技/通用技悬浮其上，每一条都标着冷冰冰的价码。光幕深处的条目隐在阴影里——那些不是买不起，是评级还不够。</p>\
             <p style='color:#888'>已掌握技能 <b style='color:#8fd0a8'>{}</b> 门 · 点数 <b style='color:#ffd76a'>{}</b></p>\
             <p style='color:#666'>按流派分类（评级 / 基因阶 / 修真境 / 真气上限达标方可购入）。</p>",
            st.skills.len(), st.points
        )
    }),
    choices: &[
        ChoiceDef { label: "◆ 修真 · 功法/禁制/神通", sub: "cu_* · cu_gong/cu_jin/cu_shen", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_xiu") },
        ChoiceDef { label: "◆ 武道 · 内功绝学", sub: "sk_ww_* / skx_ww_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_ww") },
        ChoiceDef { label: "◆ 基因 · 基因锁技", sub: "sk_gene_* / skx_gene_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_gene") },
        ChoiceDef { label: "◆ 血统 · 血统技能", sub: "sk_vamp/sk_wolf/sk_zuwu/sk_zhanshi / skx_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_blood") },
        ChoiceDef { label: "◆ 圣光 · 圣光术", sub: "sk_holy_* / skx_holy_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_holy") },
        ChoiceDef { label: "◆ 科技 · 武器科技", sub: "sk_tech_* / skx_tech_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_tech") },
        ChoiceDef { label: "◆ 超能 · 超能力NT", sub: "sk_nt_* / skx_nt_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_nt") },
        ChoiceDef { label: "◆ 模因 · 概念技", sub: "sk_meme_* / skx_meme_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_meme") },
        ChoiceDef { label: "◆ 通用 · 战术技能", sub: "sk_util_* / skx_util_*", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_skcat_util") },
        ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_xiu", bg: Some("img_nexus.png"), loc: Some("技能 · 修真"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["修真一脉——以天地灵机为刃，破境成仙。光球里的淡金雾气缓缓旋转，等你把点数放上祭台。"]),
    choices: CAT_XIU,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_ww", bg: Some("img_nexus.png"), loc: Some("技能 · 武道"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["武道一脉——拳脚之内功绝学，刀剑之间见生死。武学的影子在光幕上一板一眼地演练，像在主神面前演示它值多少点。"]),
    choices: CAT_WW,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_gene", bg: Some("img_nexus.png"), loc: Some("技能 · 基因"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["基因一脉——解放锁率所获的战斗本能。解锁的过程从不会舒服，主神只负责把开关递到你手里。"]),
    choices: CAT_GENE,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_blood", bg: Some("img_nexus.png"), loc: Some("技能 · 血统"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["血统一脉——需持有对应血统方可修习的专属技。光球先查验你的血脉，再决定向不向你敞开这一层。"]),
    choices: CAT_BLOOD,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_holy", bg: Some("img_nexus.png"), loc: Some("技能 · 圣光"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["圣光一脉——以圣火与祷词驱散暗秽。光球里的辉光柔和下来，像是这座冰冷的空间里唯一一点暖色。"]),
    choices: CAT_HOLY,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_tech", bg: Some("img_nexus.png"), loc: Some("技能 · 科技"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["科技一脉——机械为骨，电磁为爪。零件在光幕里自动装配、拆解、再装配，演示着每一次强化的价格。"]),
    choices: CAT_TECH,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_nt", bg: Some("img_nexus.png"), loc: Some("技能 · 超能NT"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["超能一脉——以念动力与预知撬动现实。光球微微震颤，像在测量你脑中那片未知的区域。"]),
    choices: CAT_NT,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_meme", bg: Some("img_nexus.png"), loc: Some("技能 · 模因"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["模因一脉——改写概念，重写因果。光幕上的文字不断重组，你盯着看久了，会觉得那些字也在看你。"]),
    choices: CAT_MEME,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_skcat_util", bg: Some("img_nexus.png"), loc: Some("技能 · 通用"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["通用一脉——无门槛的保命与战术技能。这里的光最朴素，也是最多人活着离开的那一列。"]),
    choices: CAT_UTIL,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_shop", bg: Some("img_nexus.png"), loc: Some("主神空间 · 道具铺"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&["道具铺一字排开——武器、护具、消耗品、符卷、圣物，在冷光下泛着各色微光，像一条摆满了救命的市集。价格 / 评级门槛查道具表，主神童叟无欺，也从不讲价。"]),
    choices: CAT_SHOP,
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_craft", bg: Some("img_nexus.png"), loc: Some("主神空间 · 合成工坊"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[
        "<p>工坊的火光吞吐，铁砧上的光与影一明一灭，一件件材料可在此合成为更强之物——主神的火焰不认主人，只认配方。</p>\
         <p style='color:#888'>合成需原料齐备（消耗道具）。下方「武器强化」则为已装配武器的增强服务，+1 需 1000 点（也可消耗强化石免费强化）。</p>",
    ]),
    choices: &[
        ChoiceDef { label: "◆ 合成：能量核心残片", sub: "灵魂碎片 + 能量核心残片", cond: Some(cond_has_recipe_core), effects: &NO_EFF, route: Route::Dyn(route_craft_core) },
        ChoiceDef { label: "◆ 合成：圣徽", sub: "圣徽钥匙 + 圣水", cond: Some(cond_has_recipe_cross), effects: &NO_EFF, route: Route::Dyn(route_craft_cross) },
        // 合成扩容（第 2 批）
        ChoiceDef { label: "◆ 合成：电磁炮核心", sub: "能量核心残片 + 光束核心", cond: Some(cond_has_recipe_em_core), effects: &NO_EFF, route: Route::Dyn(route_craft_em_core) },
        ChoiceDef { label: "◆ 合成：血族精血", sub: "虫潮血样 + 灵魂碎片", cond: Some(cond_has_recipe_blood_ess), effects: &NO_EFF, route: Route::Dyn(route_craft_blood_ess) },
        ChoiceDef { label: "◆ 合成：法宝碎片", sub: "能量核心残片 + 灵魂碎片", cond: Some(cond_has_recipe_treasure_frag), effects: &NO_EFF, route: Route::Dyn(route_craft_treasure_frag) },
        ChoiceDef { label: "◆ 合成：血煞战旗（法宝）", sub: "法宝碎片 + 血族精血", cond: Some(cond_has_recipe_tr_banner), effects: &NO_EFF, route: Route::Dyn(route_craft_tr_banner) },
        ChoiceDef { label: "◆ 合成：普通强化石", sub: "能量核心残片 + 灵魂碎片", cond: Some(cond_has_recipe_enh_stone), effects: &NO_EFF, route: Route::Dyn(route_craft_enh_stone) },
        ChoiceDef { label: "◆ 合成：高级强化石", sub: "普通强化石 + 电磁炮核心", cond: Some(cond_has_recipe_enh_stone_hi), effects: &NO_EFF, route: Route::Dyn(route_craft_enh_stone_hi) },
        ChoiceDef { label: "◆ 武器强化 +1", sub: "1000 点 · 需已装配武器 · 上限 +5", cond: Some(cond_enhance_ready), effects: &NO_EFF, route: Route::Dyn(route_enhance) },
        ChoiceDef { label: "◆ 武器强化 · 消耗普通强化石 +1", sub: "消耗 1×普通强化石 · 免费", cond: Some(cond_enhance_stone_ready), effects: &NO_EFF, route: Route::Dyn(route_enhance_stone) },
        ChoiceDef { label: "◆ 武器强化 · 消耗高级强化石 +2", sub: "消耗 1×高级强化石 · 免费 · 上至 +5", cond: Some(cond_enhance_stone_hi_ready), effects: &NO_EFF, route: Route::Dyn(route_enhance_stone_hi) },
        ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_done", bg: Some("img_nexus.png"), loc: Some("主神空间 · 兑换光球"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        format!(
            "<p>光球回响一声清鸣，嗡鸣在空旷的广场上荡开又消散，一股温热的能量沉入你的身体，像被某种机器精准地注射进来。兑换完成。</p>\
             <p style='color:#8fd0a8'>当前点数：<b>{}</b><br>已兑换：{}</p>",
            st.points, exchange_name(st)
        )
    }),
    choices: &[ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_exchange_fail", bg: Some("img_nexus.png"), loc: Some("主神空间 · 兑换光球"),
    mood: "danger", speaker: Some("主神"), voice: None,
    text: TextSpec::Dyn(|st| {
        format!(
            "<p>光球发出一声低沉的警告，嗡鸣贴着地面滚开——<b style='color:#ff8a8a'>奖励点数不足。</b></p>\
             <p style='color:#ffd76a'>兑换条目价格：细胞活力强化 800 点 · 基因锁一阶 2000 点 · 初级吸血鬼血统 3000 点。</p>\
             <p>你目前持有 <b>{}</b> 点。光球已经收回光影，等你回去完成恐怖片轮回、赚足够了再来——主神不怜悯，也不催促。</p>",
            st.points
        )
    }),
    choices: &[ChoiceDef { label: "返回兑换目录", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_resurrection", bg: Some("img_nexus.png"), loc: Some("主神空间 · 复活祭坛"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| text_resurrection(st)),
    choices: &[
        ChoiceDef { label: "复活一名本次阵亡的同伴", sub: "4000 点 · 祭坛祈唤", cond: Some(cond_has_dead_teammate), effects: &NO_EFF, route: Route::Dyn(route_resurrect_teammate) },
        ChoiceDef { label: "（抚过祭坛符纹，询问献祭须知的代价）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_zhangjie") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_resurrect_done", bg: Some("img_nexus.png"), loc: Some("主神空间 · 复活祭坛"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        let name = st.resurrected_name.clone().unwrap_or_else(|| "逝去的同伴".to_string());
        format!(
            "<p>凹槽中泛起一线青白辉光，祭坛的符纹逐条点亮，嗡嗡的低鸣贴着地面传来。那光的轮廓仿佛被你亲手从虚无中<em>拖了回来</em>——</p>\
             <p><b style='color:#8fd0a8'>{name}</b> 的形体在祭坛上一点点凝实。<br>他睁眼时大口喘着气，茫然地看着四周：「我……我不是……？」</p>\
             <p style='color:#666'>主神的低语没入祭坛，没有起伏：「生命自有其价。你已经替他付过了——记住这个数字，下一条命更贵。」</p>\
             <p style='color:#8fd0a8'>剩余点数：<b>{}</b></p>",
            st.points
        )
    }),
    choices: &[ChoiceDef { label: "返回主神广场", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_zhangjie") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_resurrect_none", bg: Some("img_nexus.png"), loc: Some("主神空间 · 复活祭坛"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|_| "本次轮回，无人随你一同倒在恐怖片世界里。<br>祭坛灰光平静，不见任何待归之魂。风从广场尽头吹来，把寂静又推远了一些。".to_string()),
    choices: &[ChoiceDef { label: "返回主神广场", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_zhangjie") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_resurrect_fail", bg: Some("img_nexus.png"), loc: Some("主神空间 · 复活祭坛"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        format!(
            "<p>祭坛符纹亮起又暗下，灰光一滞，像被什么东西卡住了喉咙——<b style='color:#ff8a8a'>你的奖励点数不足以赎回一条命。</b></p>\
             <p>复活一名同伴需要 <b>4000</b> 点，而你持有 <b>{}</b> 点。</p>\
             <p style='color:#666'>他们在虚无中等你，那里的时间走得很慢。攒够了，再来。</p>",
            st.points
        )
    }),
    choices: &[ChoiceDef { label: "返回主神广场", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_zhangjie") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_zhangjie", bg: Some("img_nexus.png"), loc: Some("主神空间 · 半圆广场 · 张杰"),
    mood: "calm", speaker: Some("张杰"), voice: Some("vo_zhangjie_world.wav"),
    text: TextSpec::Dyn(|st| {
        if st.flag("bh_cleared") {
            "张杰靠在光柱旁的柱子上，看着你缓步走来，目光带着一抹了然的弧度，像在验收一件合格的货物。\n\n「活着回来了，还干掉了那只舔食者，干得漂亮，新人。」他朝南侧三枚光球扬了扬下巴，「点数攒够，就去那边换点真本事——光球不会嫌你来得勤，只会记住你花得多。」\n\n「要开启下一轮，随时来对我说一声。」他低下头，声音沉了半分，「轮回不等人，等你的只有广场上这片永远不变的灯。」".to_string()
        } else {
            "张杰抱着手臂站在半圆广场边缘，身后的广场空旷得望不到边，灯光在他脸上投下分明的棱角。\n\n「主神空间不是终点，是个中转站。」他扫了你一眼，「第一次进副本的新人在北边那列列车里选武器——选好了就往东走，传送到生化蜂巢。」\n\n他顿了顿，声音放低了几分：「想要变强、换血统、复活同伴，攒够点数来南边这排光球。别死在第一次轮回里，这里没人给你收尸。」他转身前又补了一句，「也别指望主神同情你——它连自己是什么都不在乎。」".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "「我想看看兑换目录。」", sub: "南侧光球 · 强化 / 基因锁 / 血统", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
        ChoiceDef { label: "「复活祭坛是做什么的？」", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_resurrection") },
        ChoiceDef { label: "「让我看看上次轮回的任务简报。」", sub: "战报评价 · 点数 · 兑换 · 队友", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_briefing") },
        ChoiceDef { label: "「张杰，我要开始下一轮回。」", sub: "轮回重启", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_new_cycle") },
        ChoiceDef { label: "「（再聊聊）」", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_god") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
/* ---- 主神空间 · 中洲队队友 NPC 对话（原著性格还原 · 不对抗/不写真相线） ---- */
SceneDef {
    id: "s_nexus_zhengzha", bg: Some("img_nexus.png"), loc: Some("主神空间 · 广场东侧 · 郑吒"),
    mood: "calm", speaker: Some("郑吒"), voice: None,
    text: TextSpec::Static(&[
        "一个身形笔挺的青年靠在光柱基座上，正用一块绒布擦拭一把高斯手枪，枪身在冷光下泛着油润的暗光。听到脚步，他抬眼，露出一个疲惫却尽力友好的笑——在这座广场上，这种笑比金子还稀有。\n\n「你是新来的轮回者？」他把枪收回枪套，朝你点了点头，「我叫郑吒。也是从最底层一步步走过来的——第一次进《生化》的时候，我吓得腿都在抖，差点没活着站上这广场。」",
        "「可我学会了一件事：<em>在这个世界里，掉队的人真的会死。</em>所以我把每一位还活着的队友都当成家人——谁要动他们，先过我这一关。」他拍拍胸脯，语气里带着不容置疑的热血，眼神却有一瞬掠过广场尽头那片灰黑的墙，「你要是觉得撑不下去，就告诉我。多一个人活下来，我们就多一分希望——这是我在死人堆里悟出来的道理。」",
        "「记住，先把自己变强，才有资格谈守护别人。光球不会为你流一滴泪，但点数会变成你的拳头、你的速度、你的第二条命。」他朝南侧的光球努了努嘴，「去换点真本事。」",
    ]),
    choices: &[
        ChoiceDef { label: "「郑吒，我想去兑换变强。」", sub: "南侧光球 · 强化 / 基因锁 / 血统", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
        ChoiceDef { label: "「谢谢你，郑吒。」（返回主神广场）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_god") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_chuxuan", bg: Some("img_nexus.png"), loc: Some("主神空间 · 广场北侧 · 楚轩"),
    mood: "calm", speaker: Some("楚轩"), voice: None,
    text: TextSpec::Static(&[
        "一个戴着无框眼镜、面容冷峻的男人静静站在兑换光球旁，正用一根细笔在掌心演算什么，笔尖划过掌纹，像在解一道没人看见的题。你没有开口，他先抬起了头——目光冷静得近乎缺乏感情，像一台在核对数据的仪器。\n\n「新人。这个广场上，每一个靠近你的人，都可能成为你的变数或筹码——包括我。」",
        "「圣人不仁，以天地万物为刍狗。这句话不是残忍，是秩序。」他转身指向南侧的光球，声线平淡得像在念一份说明书，「主神给点数，就是为了让你投资自己。细胞强化、基因锁、血统，各有其账，光球上的标价就是唯一的真理。理性地说——<em>先去兑换，把点数花在保命能力上，你活过下一轮的概率会显著提高。</em>」",
        "「记住：真正的布局，从不做无谓的牺牲，也从不被感情牵着走。」他推了推眼镜，镜片反着光球的白光，不再看你，「去兑换吧。活着回来的人，才有资格谈布局——死了的人，只会成为别人布局里的一步。」",
    ]),
    choices: &[
        ChoiceDef { label: "（按楚轩的建议，前往兑换目录）", sub: "南侧光球 · 强化 / 基因锁 / 血统", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
        ChoiceDef { label: "「你的冷静……让人有些不寒而栗。」", sub: "返回主神广场", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_god") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_zhanlan", bg: Some("img_nexus.png"), loc: Some("主神空间 · 广场西南 · 詹岚"),
    mood: "calm", speaker: Some("詹岚"), voice: None,
    text: TextSpec::Static(&[
        "一位穿着利落风衣的女子坐在广场边缘的长椅上，膝上摊着一本写满笔记的硬皮本，页边密密麻麻地贴着各色标签。她抬起头，朝你温和地笑了笑，语气平和而理性——在这样一座冷漠的广场上，她的笑像是唯一没有价码的东西。\n\n「你好。我叫詹岚。别紧张——我也不是你想象中那种无所不能的老兵，只是在任务里练出了几分冷静。」她把本子合上，示意你靠近，「每一次轮回，我都会把空间地形、剧情关键人物、危险点都记下来。情报，有时候比血统更救命。」",
        "「郑吒总嫌我想太多……」她轻声笑了笑，指尖点了点笔记本，「可在这个地方，想得多的人，往往能多活几轮。圣人不仁，所以我更要替自己算清楚每一步——这座广场不会替任何人着想。」她顿了顿，目光柔和了些，「如果哪天你在队伍里受了伤，或者被恐惧压垮了，就来找我。活着回去，才有以后。」",
    ]),
    choices: &[
        ChoiceDef { label: "「詹岚，帮我看看兑换方向。」", sub: "知性分析 · 南侧光球", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
        ChoiceDef { label: "「谢谢你。」（返回主神广场）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_god") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_zhaoyingkong", bg: Some("img_nexus.png"), loc: Some("主神空间 · 广场南缘 · 赵樱空"),
    mood: "calm", speaker: Some("赵樱空"), voice: None,
    text: TextSpec::Static(&[
        "阴影里，一个少女正仔细擦拭一把短匕，擦得很慢，像在给匕首上最后一层油。她抬起头看你一眼——<em>那目光冷得像刀锋</em>，随即又垂下去，仿佛你根本不值得她多费半句口舌。\n\n「……不要挡我的路。」她声音清冷，匕首的刃光在指间一闪，「在恐怖片里，活得久的从来不是最犹豫的那个，而是出手足够快、足够狠的那个。」短匕归鞘，发出一声轻响，像一句收尾的判决，「我不靠嘴活着。真到了生死关头——你最好也别只靠嘴。」",
        "她略一停顿，目光在你身上略作打量，像在评估一件可有可无的兵器：「想学刺客的活法，就去兑换。身法、隐匿、一击毙命的技法，都在南边那排光球里。」说完，她转身没入阴影，脚步声被广场的寂静吞得干干净净，只留一句话飘在空旷的广场上：「……别死在我前面。」",
    ]),
    choices: &[
        ChoiceDef { label: "（注视她消失的方向）前往兑换目录", sub: "南侧光球 · 强化 / 血统 / 技能", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_exchange") },
        ChoiceDef { label: "「……记住她了。」（返回主神广场）", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_nexus_god") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_nexus_new_cycle", bg: Some("img_nexus.png"), loc: Some("主神空间 · 中央光柱"),
    mood: "calm", speaker: Some("张杰"), voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef { voice: None, death: None, card: card_new_cycle }),
},
SceneDef {
    id: "s_nexus_briefing", bg: Some("img_nexus.png"), loc: Some("主神空间 · 中央光柱 · 登记台"),
    mood: "calm", speaker: Some("主神"), voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef { voice: None, death: None, card: card_briefing }),
},

/* ---- 地下列车 ---- */
SceneDef {
    id: "s_train", bg: Some("img_train.png"), loc: Some("《生化危机》 · 地下列车"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "柴油味。车轮撞击铁轨的哐当声。\n\n你在一辆行驶中的地下列车里醒来。车厢另一端坐着几名全副武装的佣兵——防弹背心、冲锋枪、战术手电。这不是电影画面：气味、颠簸、枪油味，真实得可怕。",
        "「目标：地下研究所『蜂巢』入口。」戴战术手套的女人利落地检查弹匣，她是<em>蕾恩</em>，「红后把整个蜂巢锁死了。里面五百多名员工……生死不明。」",
        "角落里缩着几个和你一样茫然的新人。张杰靠在车门边，冲你微微点头——认命吧，开始了。",
    ]),
    choices: &[
        ChoiceDef { label: "【支线A】主动坐到蕾恩身边搭话", sub: "了解剧情人物 = 掌握活下去的信息", cond: None, effects: &[Eff::SetFlag("A")], route: Route::To("s_train_rain") },
        ChoiceDef { label: "安静待在角落观察众人", sub: "", cond: None, effects: &[Eff::San(-3)], route: Route::To("s_mission") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_train_rain", bg: Some("img_train.png"), loc: Some("《生化危机》 · 地下列车"),
    mood: "calm", speaker: Some("蕾恩"), voice: None,
    text: TextSpec::Static(&[
        "「新来的？坐。」蕾恩拍了拍身边的座位，把一支手电扔给你，「待会儿跟着我的背影走。记住三条：<em>别乱跑，别掉队，听到我喊趴下就立刻趴下。</em>」",
        "她顿了顿，压低声音：「说真的……这次任务邪门。五百多人失联，却连一个求救信号都没有。我不喜欢这种安静。」\n\n你获得了她的初步信任。<em>【支线A · 已记录达成条件】</em>",
    ]),
    choices: &[ChoiceDef { label: "列车减速，抵达蜂巢站台", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_mission") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_mission", bg: Some("img_train.png"), loc: Some("《生化危机》 · 废弃站台"),
    mood: "danger", speaker: Some("【主神】"), voice: Some("vo_mission.wav"),
    text: TextSpec::Static(&[
        "列车停稳。站台空旷得诡异——应急灯全亮着，却没有一个人影。行李箱东倒西歪，一只婴儿鞋孤零零躺在自动扶梯口。\n\n脑海中响起没有感情的机械音：",
        "<em>【主线任务发布：跟随佣兵小队进入蜂巢，关闭超级电脑「红后」，活着回到地面。】\n【提示：支线剧情隐藏于剧情节点之中，达成可获得额外奖励点数。】</em>",
        "队长<em>一号·马修·艾迪森</em>打出手势，小队呈战术队形展开。「新人跟中间，别碰任何东西。走！」",
    ]),
    choices: &[ChoiceDef { label: "跟随队伍踏入蜂巢深处", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_corridor") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 蜂巢走廊 · B餐厅 ---- */
SceneDef {
    id: "s_corridor", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B区走廊"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "惨白的日光灯管有一半在闪烁。走廊两侧全是玻璃幕墙，幕墙后是漆黑的实验室，培养槽里泡着说不清的阴影。\n\n空气里有消毒水的味道——以及一丝若有若无的、铁锈般的腥味。",
        "「前面是B餐厅。」一号压低声音，「五百人的基地……不可能连个值班的都没有。都警醒点。」",
    ]),
    choices: &[
        ChoiceDef { label: "【支线B1】凑近幕墙观察实验室内部", sub: "恐惧，但信息就是生命", cond: None, effects: &[Eff::SetFlag("B1")], route: Route::To("s_observe_lab") },
        ChoiceDef { label: "紧跟队伍前进", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_bhall") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_observe_lab", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B区走廊"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你把手电贴上玻璃，强忍寒意照进去——\n\n地板上有大片被拖拽过的暗褐色痕迹，一直延伸进最里面的通风管道。一件白大褂挂在椅背上，胸前工作牌上溅着干涸的血点。",
        "<em>拖拽痕迹的方向性很强……它们不是乱走的。有什么东西在「搬运」。而且——它们行动迟缓，只会直线扑抓。</em>\n\n你默默记下了这些特征。<em>【支线B1 达成：掌握敌人行动规律，战斗命中率提升。】</em>",
    ]),
    choices: &[ChoiceDef { label: "追上队伍", sub: "", cond: None, effects: &[Eff::San(-5)], route: Route::To("s_bhall") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_bhall", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B餐厅"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "B餐厅的门虚掩着。一号用枪尖挑开的瞬间，一股浓烈的腐臭扑面而来。\n\n几十份没吃完的饭菜长满霉斑，餐桌上地上散落着撕碎的工作服……却没有尸体。\n\n「全体戒备。」一号的声音绷紧了。",
        "就在此时——<em>「哐！！」</em>厨房双开门被撞开，一个穿厨师服的<em>东西</em>跌跌撞撞扑出来！灰白的脸，浑浊如死鱼的眼珠，半截舌头耷拉在下巴外——\n\n它直直扑向正在调试设备的<em>卡普兰</em>！",
    ]),
    choices: &[
        ChoiceDef { label: "冲上去救卡普兰！", sub: "近战缠斗 · 保护剧情人物 · 赢得信任", cond: None, effects: &[Eff::SetFlag("savedKaplan")], route: Route::To("s_fight_zombie1_save") },
        ChoiceDef { label: "保持距离出手", sub: "更安全 · 但卡普兰可能挂彩", cond: None, effects: &NO_EFF, route: Route::To("s_fight_zombie1_far") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fight_zombie1_save", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B餐厅"),
    mood: "danger", speaker: Some("⚔ 初次遭遇"), voice: None,
    text: TextSpec::Static(&["没有时间思考。你的身体先于大脑动了——<em>这就是恐怖片世界，没有NG，没有重播。</em>"]),
    choices: &NO_CH, fight_id: Some("zombie1_save"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fight_zombie1_far", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B餐厅"),
    mood: "danger", speaker: Some("⚔ 初次遭遇"), voice: None,
    text: TextSpec::Static(&["你和那东西之间隔着两张餐桌——这份距离是你全部的安全感。<em>稳住，瞄准头部。</em>"]),
    choices: &NO_CH, fight_id: Some("zombie1_far"), video: None, cine_label: None, overlay: None,
},

/* ---- F2 实验层探索：无菌实验室 · 病毒样本库 · 隔离观察室（支线深挖） ---- */
SceneDef {
    id: "s_b_sterile_lab", bg: Some("img_sterile_lab.png"), loc: Some("实验层 · 白色无菌实验室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "无菌室的应急灯是冷白色的，照得每一寸不锈钢台面都泛着病态的光。解剖台上躺着一具女性尸体——白大褂，胸牌：克莱恩，埃莉诺。她的指甲缝里嵌着干涸的皮肉，像是死前抓挠过什么。",
        "培养皿里的培养基早已干裂，只有角落里一组还在微微蠕动。<em>绿色液体</em>从某个破裂的容器边缘缓慢渗出，顺着台面滴落，在地板上蚀出浅浅的凹痕。",
        "档案柜半开着。最上层压着一份封皮发黄的报告：<em>《封闭日观察记录 · T病毒感染三小时转化》</em>。",
    ]),
    choices: &[
        ChoiceDef { label: "阅读观察报告", sub: "三小时……原来一切早就注定了", cond: None, effects: &[Eff::SetFlag("b_sterile_read"), Eff::San(-8)], route: Route::To("s_b_sterile_note") },
        ChoiceDef { label: "检查培养皿", sub: "那团蠕动的阴影……", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_b_sterile_petri") },
        ChoiceDef { label: "搜查保安的尸体", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_fight_b_guard") },
        ChoiceDef { label: "离开无菌室", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_b_sterile_leave") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_b_sterile_note", bg: Some("img_sterile_lab.png"), loc: Some("实验层 · 无菌实验室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "报告的字迹工整得近乎强迫症，却在末尾几行开始扭曲：",
        "<em>「封闭日 09:00 —— 通风系统异常，A区人员出现发热。按流程上报。\n封闭日 11:40 —— 发热蔓延至B区。……不对，这不是流感。他们的瞳孔……\n封闭日 12:05 —— 第一批『患者』开始撕咬护工。我用钥匙反锁了化验室。\n封闭日 12:30 —— 三小时。从出现症状到完全转化，只需要三小时。\n封闭日 14:00 —— 化验室外已经没有声音了。我把这份报告放进档案柜最上层。\n如果有人看到——跑。不要相信任何『幸存者』。」</em>",
        "报告末尾压着一把沾了锈迹的钥匙，标签上写着：<em>冷藏库</em>。",
    ]),
    choices: &[ChoiceDef { label: "收好冷藏库钥匙", sub: "【道具获得】冷藏库钥匙", cond: None, effects: &[Eff::SetFlag("cold_room_key")], route: Route::To("s_b_sterile_petri") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_b_sterile_petri", bg: Some("img_sterile_lab.png"), loc: Some("实验层 · 无菌实验室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你凑近那组还在蠕动的培养皿。半透明的培养基里，一团灰色组织正缓慢搏动——它<em>感知到了</em>你的靠近，搏动的频率骤然加快。",
        "「别碰。」门口传来蕾恩压低的声音。她不知何时站在了门口，枪口朝下，「我见过这种东西的下场。」她顿了顿，「走。去厨房，拿急救箱。后面用得上。」",
        "你退开时，余光瞥见那团组织裂开了一道缝，露出一排细密的、针尖般的白牙。",
    ]),
    choices: &[ChoiceDef { label: "跟着蕾恩去厨房", sub: "支线A线索推进", cond: None, effects: &[Eff::SetFlag("side_a_raine"), Eff::San(-4)], route: Route::To("s_find_adrenaline") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fight_b_guard", bg: Some("img_sterile_lab.png"), loc: Some("实验层 · 无菌实验室"),
    mood: "danger", speaker: Some("⚔ 保安丧尸"), voice: None,
    text: TextSpec::Static(&["你翻开保安尸体的瞬间，它睁开了眼。浑浊的眼珠直直盯着你——然后，它动了。"]),
    choices: &NO_CH, fight_id: Some("b_guard"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_b_sterile_after", bg: Some("img_sterile_lab.png"), loc: Some("实验层 · 无菌实验室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "保安丧尸彻底不动了。你从它腰间摸出三发9mm子弹和一本巡逻日志——最后一页记着：<em>「封闭当晚，我看见B餐厅的人开始互相撕咬。我锁了门，但不知道能撑多久。」</em>",
        "门外传来卡普兰的喊声：「你们在哪？！一号命令——B区失守了，全队向机房转移！」",
    ]),
    choices: &[ChoiceDef { label: "跟上队伍", sub: "", cond: None, effects: &[Eff::SetFlag("guard_looted"), Eff::Points(12)], route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_b_sterile_leave", bg: Some("img_sterile_lab.png"), loc: Some("实验层 · 无菌实验室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你没有多做停留，退出了无菌室。冷白的光在身后合拢，像一扇缓缓关闭的眼睑。"]),
    choices: &[ChoiceDef { label: "跟上队伍", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_b_kitchen_after", bg: Some("img_b_kitchen.png"), loc: Some("实验层 · 厨房"),
    mood: "danger", speaker: Some("蕾恩"), voice: None,
    text: TextSpec::Static(&[
        "厨师丧尸倒在了冷藏库门口，菜刀「哐当」落地。你从它围裙的血渍下摸到一张被血浸透的纸条，上面潦草地写着：<em>0929</em>。",
        "冷藏库门上的密码锁亮着绿灯——试一下？",
    ]),
    choices: &[
        ChoiceDef { label: "输入 0929", sub: "【道具获得】冷藏库补给", cond: None, effects: &[Eff::Points(15)], route: Route::To("s_b_kitchen_loot") },
        ChoiceDef { label: "不理会，跟上队伍", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_to_redqueen") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_b_kitchen_loot", bg: Some("img_b_kitchen.png"), loc: Some("实验层 · 冷藏库"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "冷藏库的门缓缓滑开，冷气扑面。货架上码放着未过期的罐头、几瓶矿泉水，还有一盒<em>肾上腺素注射器</em>——比厨房急救箱里那支更新、更完整。",
        "你收好补给。角落的监控摄像头红灯闪烁——红后在看着。",
    ]),
    choices: &[ChoiceDef { label: "带走补给", sub: "【道具获得】肾上腺素 ×2", cond: None, effects: &[Eff::SetFlag("adrenaline"), Eff::SetFlag("adrenaline2")], route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_virus_vault", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 病毒样本库"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "样本库的门在你身后合拢。冷藏柜的绿色荧光照亮了整间屋子——成排的试管架，每一支都贴着标签：<em>H-00-01……H-00-07</em>。",
        "房间中央的保险柜泛着金属冷光。旁边的实验记录终端屏幕还亮着，一行字停在输入框里：<em>「三重认证：虹膜 · 密码 · 容器编号」</em>。",
        "墙角，一个穿白大褂的身影缓缓站起来——它曾是这里的病毒学家。",
    ]),
    choices: &[
        ChoiceDef { label: "调查实验记录终端", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_research_terminal") },
        ChoiceDef { label: "检查保险柜", sub: "三重认证……密码和编号在哪？", cond: None, effects: &NO_EFF, route: Route::To("s_virus_safe") },
        ChoiceDef { label: "战斗：守卫变异体", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_fight_mut_guard") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_research_terminal", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "终端要求输入研究员工号。屏幕角落贴着一张褪色的工牌照片：<em>艾登·沃德</em>。",
        "你想起在隔离观察室门口看到过一行蚀刻的编号：<em>19</em>。",
    ]),
    choices: &[
        ChoiceDef { label: "输入 19", sub: "终端解锁", cond: None, effects: &[Eff::SetFlag("terminal_ok"), Eff::Points(10)], route: Route::To("s_research_log") },
        ChoiceDef { label: "返回", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_virus_vault") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_research_log", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "终端日志滚动而出——",
        "<em>「10-02：样本 H-00-07 出现异常增殖。沃德博士要求单独观察。\n10-03：07号样本产生听觉定位能力。从未见过的突变。\n10-04：07号样本……开始『聆听』。我关掉了所有通风口的风扇。\n10-05：沃德博士失踪。隔离室门禁密码被改动：0719。\n10-06：我听见隔离室里传来规律的敲击声。1、2、3……它也在数拍子。」</em>",
        "日志末尾附带一行小字：<em>保险柜容器编号：H-00-07</em>。",
    ]),
    choices: &[ChoiceDef { label: "记下线索", sub: "密码0719 · 容器编号H-00-07", cond: None, effects: &[Eff::SetFlag("vault_hint")], route: Route::To("s_virus_vault") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_virus_safe", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库保险柜"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "保险柜的虹膜扫描仪无声地转着。你只有一次机会——输错，警报就会响彻整个样本库。",
        "密码：？   容器编号：？",
    ]),
    choices: &[
        ChoiceDef { label: "密码 0912 · 编号 H-00-07", sub: "【道具获得】T病毒样本", cond: Some(cond_vault_ok), effects: &[Eff::SetFlag("virus_sample"), Eff::Points(30)], route: Route::To("s_virus_sample_got") },
        ChoiceDef { label: "强行撬锁", sub: "触发警报 · 危险", cond: None, effects: &[Eff::San(-6)], route: Route::To("s_vault_alarm") },
        ChoiceDef { label: "返回", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_virus_vault") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_virus_sample_got", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "保险柜缓缓打开，冷气涌出。密封容器里，一小管<em>翠绿色的液体</em>在荧光下流动——T病毒原液。",
        "你的手触到容器的瞬间，整栋样本库的灯闪了一下。红后的声音从四面八方响起，平淡得没有一丝起伏：",
        "「样本已取出。生物隔离协议升级至最高级。你们还剩……四十七分钟。」",
    ]),
    choices: &[ChoiceDef { label: "收好样本，撤离", sub: "【道具获得】T病毒样本", cond: None, effects: &[Eff::SetFlag("virus_sample"), Eff::Points(30)], route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_vault_alarm", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "撬棍刚插进锁缝，刺耳的警报就撕裂了寂静。红色警示灯疯狂旋转——",
        "「警报：样本库入侵。隔离协议启动。」红后的声音依旧平静，「东侧闸门封锁。最近的感染者将在九十秒内到达。」",
        "你不得不放弃保险柜，夺路而逃。身后传来沉重的、拖行的脚步声——不止一个。",
    ]),
    choices: &[ChoiceDef { label: "逃向机房", sub: "理智 -8", cond: None, effects: &[Eff::San(-8)], route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fight_mut_guard", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库"),
    mood: "danger", speaker: Some("⚔ 守卫变异体"), voice: None,
    text: TextSpec::Static(&["白大褂的身影猛地扑来，绿色原液在它身后拖出一道腐蚀的痕迹。"]),
    choices: &NO_CH, fight_id: Some("mut_guard"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_virus_vault_after", bg: Some("img_virus_vault.png"), loc: Some("实验层 · 样本库"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "变异体倒在了冷藏柜之间，绿色的原液在它身下汇成一小滩。你从它口袋里摸出一张沾满液体的门禁卡——标签：<em>隔离观察室</em>。",
        "门禁卡背面用马克笔写着四个数字：<em>0719</em>。",
    ]),
    choices: &[ChoiceDef { label: "收好门禁卡", sub: "【道具获得】隔离室门禁卡", cond: None, effects: &[Eff::SetFlag("iso_card"), Eff::Points(25)], route: Route::To("s_virus_vault") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_isolation_room", bg: Some("img_isolation.png"), loc: Some("实验层 · 隔离观察室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "门禁卡刷过感应区，双重气密门无声滑开。隔离室里弥漫着消毒水与某种更刺鼻的气味。",
        "房间中央的强化玻璃舱内，一团蜷缩的、没有皮肤的灰红色生物正贴着玻璃「聆听」——<em>舔食者早期体</em>。它没有眼睛，但它的舌头正对着你所在的方向，一下、一下地舔舐玻璃。",
        "舱壁上的警示灯开始闪烁。<em>「样本异常。隔离舱将在三分钟内强制排气。」</em>",
        "玻璃上，出现了一道裂纹。",
    ]),
    choices: &[
        ChoiceDef { label: "抢先压制它！", sub: "三回合内打出硬直", cond: None, effects: &NO_EFF, route: Route::To("s_fight_licker_larva") },
        ChoiceDef { label: "撤离，让它挣脱", sub: "它会逃进排污管网……", cond: None, effects: &[Eff::San(-12)], route: Route::To("s_iso_flee") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fight_licker_larva", bg: Some("img_isolation.png"), loc: Some("实验层 · 隔离观察室"),
    mood: "danger", speaker: Some("⚔ 舔食者早期体"), voice: None,
    text: TextSpec::Static(&["玻璃舱在尖啸中炸裂。那团灰红色的生物扑向你的瞬间，你终于看清了它的「脸」——一张没有五官的、光滑的皮膜。"]),
    choices: &NO_CH, fight_id: Some("licker_larva"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_isolation_after", bg: Some("img_isolation.png"), loc: Some("实验层 · 隔离观察室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "幼体抽搐着倒在破碎的玻璃舱里，长舌无力地垂落。你喘着粗气，听见它最后发出一个音节——像是「听」字。",
        "【支线伏笔】它认得你的气味。你有一种预感：在地下水道，或者更深处，你会再次见到它——长大了的它。",
    ]),
    choices: &[ChoiceDef { label: "撤离", sub: "奖励 +80", cond: None, effects: &[Eff::Points(80)], route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_iso_flee", bg: Some("img_isolation.png"), loc: Some("实验层 · 隔离观察室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你转身就跑。身后传来玻璃碎裂的巨响，和一声压抑的、试探性的尖啸——它在适应自己的声音。",
        "你冲回走廊时回头看了一眼：隔离室的门正在被从里面一下一下地撞着。<em>咚。咚。咚。</em>",
    ]),
    choices: &[ChoiceDef { label: "跟上队伍", sub: "理智 -12", cond: None, effects: &[Eff::San(-12)], route: Route::To("s_to_redqueen") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

SceneDef {
    id: "s_after_zombie1_save", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B餐厅"),
    mood: "danger", speaker: Some("卡普兰"), voice: None,
    text: TextSpec::Static(&[
        "你从侧面狠狠一击，那东西应声倒地，抽搐两下不再动弹。\n\n「呼……呼……谢了兄弟！」卡普兰瘫坐在地，脸色惨白，又猛地抬头盯着你，「等等——你刚才根本没犹豫。你以前是干什么的？」",
        "蕾恩走过来踢了踢尸体，若有所思地看了你一眼：「下手干净利落……新兵可做不到。」\n\n<em>队伍对你的信任提升了。</em>但你近距离看清了它的脸——那真的是「人」。胃里一阵翻涌。",
    ]),
    choices: &[ChoiceDef { label: "压下恶心，检查厨房", sub: "", cond: None, effects: &[Eff::San(-6)], route: Route::To("s_find_adrenaline") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_after_zombie1_far", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B餐厅"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "那东西倒了下去。但就在倒下前的瞬间，它的指甲擦过了卡普兰的小臂——划出一道血痕。\n\n「该死！没事，我没事！」卡普兰咬牙包扎，看向你的眼神复杂，「下次……能再快一点吗？」",
        "一号皱眉环视：「都听好了，这地方的东西<em>不怕疼、不死透</em>。瞄准头部，确认摧毁。」",
    ]),
    choices: &[ChoiceDef { label: "搜查厨房", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_find_adrenaline") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_find_adrenaline", bg: Some("img_corridor.png"), loc: Some("蜂巢 · B餐厅厨房"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["厨房操作台上翻倒着急救箱，大部分药品已经过期。你在抽屉夹层摸到一支<em>肾上腺素注射器</em>和半瓶碘伏。\n\n标签还清晰可见：抗休克用。你把它收进口袋——不知道为什么，直觉告诉你它会救某个人的命。"]),
    choices: &[
        ChoiceDef { label: "收好肾上腺素", sub: "【道具获得】肾上腺素注射器", cond: None, effects: &[Eff::SetFlag("adrenaline")], route: Route::To("s_to_redqueen") },
        ChoiceDef { label: "没敢多拿，跟上队伍", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_to_redqueen") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 红后机房 · 激光通道 ---- */
SceneDef {
    id: "s_to_redqueen", bg: Some("img_redqueen.png"), loc: Some("蜂巢 · 中央机房"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "队伍深入到蜂巢最核心处。一座巨大的<em>球形机房</em>出现——环状金属走廊悬在深渊般的中庭之上。中央平台上，幽蓝色的全息投影微微闪烁。",
        "那是一个小女孩的形象。蓝裙，长发，眼睛一眨不眨地「看」着每一个闯入者。\n\n「<em>红后</em>。」一号吐出一口气，「超级电脑。关掉她，封锁才会解除。」",
        "「你们正在犯一个错误。」童声突然响起，温柔得毛骨悚然，「<em>我只是在执行隔离协议。放我出去，外面的世界会毁灭。</em>」\n\n没人回应。卡普兰接上破解终端，手指翻飞。",
    ]),
    choices: &[
        ChoiceDef { label: "【支线B2】趁破解间隙研究那条玻璃通道的结构图", sub: "墙上的检修示意图……为什么让人不安", cond: None, effects: &[Eff::SetFlag("B2")], route: Route::To("s_laser_observed") },
        ChoiceDef { label: "向红后发起逻辑问答", sub: "「回答正确，我让出核心权限」", cond: None, effects: &NO_EFF, route: Route::To("s_redqueen_quiz") },
        ChoiceDef { label: "直接关掉她", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_shutdown") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 红后逻辑谜题（四道） ---- */
SceneDef {
    id: "s_redqueen_quiz", bg: Some("img_redqueen.png"), loc: Some("红后 · 逻辑验证终端"),
    mood: "danger", speaker: Some("红后"), voice: Some("vo_redqueen_quiz.wav"),
    text: TextSpec::Static(&[
        "红后的投影在球形底座上缓缓转动，光屏般的双眼对准你。",
        "「可以。规则如下：四道验证题。答对一题，权限提升一级；四题全对，核心控制权移交。答错——」她顿了顿，「隔离协议会照顾你们。」",
        "「第一题，请听好。」",
    ]),
    choices: &[
        ChoiceDef { label: "开始第一题：冷却管道配比", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_pipe") },
        ChoiceDef { label: "先向她提问", sub: "T病毒 / 蜂巢 / 她自己 / 激光通道", cond: None, effects: &NO_EFF, route: Route::To("s_rq_ask") },
        ChoiceDef { label: "放弃问答，直接关掉她", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_shutdown") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_pipe", bg: Some("img_redqueen.png"), loc: Some("验证一 · 冷却管道配比"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「蜂巢深层反应堆的冷却液正在泄漏。三条管道——A、B、C——向核心区输送冷却液。流量记录显示：管道A的流量是管道C的2倍；管道B的流量比管道A每小时少30单位；三管合计流量为每小时380单位。红后需要你输入「管道C的流量」以重新校准阀门。请问：管道C的流量是多少？」",
    ]),
    choices: &[
        ChoiceDef { label: "每小时 82 单位", sub: "A=164, B=134, C=82 → 合计380", cond: None, effects: &[Eff::SetFlag("rq1"), Eff::Points(50)], route: Route::To("s_rq_pipe_ok") },
        ChoiceDef { label: "每小时 76 单位", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_pipe_no") },
        ChoiceDef { label: "每小时 90 单位", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_pipe_no") },
        ChoiceDef { label: "每小时 68 单位", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_pipe_no") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_pipe_ok", bg: Some("img_redqueen.png"), loc: Some("验证一 · 通过"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「计算正确，误差为零。这是蜂巢在没有工程师的情况下完成的第一次校准——你们比上一个样本队聪明。」",
        "「第二题：主电路通电顺序。」",
    ]),
    choices: &[ChoiceDef { label: "继续第二题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_circuit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_pipe_no", bg: Some("img_redqueen.png"), loc: Some("验证一 · 失败"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「计算错误。系统已自动执行补偿协议，冷却剂消耗增加12%。这是第一次记录，你还剩两次机会。三次错误之后，隔离协议将覆盖本区域。」（理智 -10）",
        "「第二题：主电路通电顺序。」",
    ]),
    choices: &[ChoiceDef { label: "继续第二题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_circuit") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_circuit", bg: Some("img_redqueen.png"), loc: Some("验证二 · 主电路通电顺序"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「红后核心层的四组节点需要按顺序通电，才能解锁主保险锁。维修日志上的约束条件：主断路器（M）必须比应急灯（E）先通电；保护电闸（G）必须紧跟在主断路器之后；应急灯（E）必须倒数第二通电；水泵（W）必须最后通电。请选择正确的通电顺序。」",
    ]),
    choices: &[
        ChoiceDef { label: "主断路器→应急灯→电闸→水泵", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_circuit_no") },
        ChoiceDef { label: "主断路器→电闸→应急灯→水泵", sub: "M-G-E-W ✓", cond: None, effects: &[Eff::SetFlag("rq2"), Eff::Points(50)], route: Route::To("s_rq_circuit_ok") },
        ChoiceDef { label: "电闸→主断路器→应急灯→水泵", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_circuit_no") },
        ChoiceDef { label: "主断路器→电闸→水泵→应急灯", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_circuit_no") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_circuit_ok", bg: Some("img_redqueen.png"), loc: Some("验证二 · 通过"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「线路正确，主保险锁已解锁。你开始理解这个蜂巢的『语法』了——万物在这里都遵循顺序，包括死亡。」",
        "「第三题：员工权限验证。」",
    ]),
    choices: &[ChoiceDef { label: "继续第三题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_access") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_circuit_no", bg: Some("img_redqueen.png"), loc: Some("验证二 · 失败"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「接线错误。序列不对，电流就不会到达它该去的地方。这和你们的队伍一样——顺序错了，就有人会死。还剩两次机会。」（理智 -10）",
        "「第三题：员工权限验证。」",
    ]),
    choices: &[ChoiceDef { label: "继续第三题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_access") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_access", bg: Some("img_redqueen.png"), loc: Some("验证三 · 员工权限验证"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「红后终端要求插入一张权限卡进行身份验证。你从混乱的档案柜里找到四张卡：①清洁工约瑟夫·格林——登记时间2002年9月30日，生物状态：已死亡；②门禁主管威廉·帕克斯——登记时间2002年9月28日，生物状态：存活，但名字出现在《员工注销名单》第47行；③研究员埃莉诺·克莱恩——登记时间2002年9月29日，生物状态：存活，清洁级别3；④无名卡——无登记信息，芯片签名栏印着一串红后子程序的校验码。终端提示验证条件：登记时间早于蜂巢封闭（2002年9月30日0时）、生物状态为存活、清洁级别不低于4、不在注销名单内。哪张卡能通过验证？」",
    ]),
    choices: &[
        ChoiceDef { label: "① 约瑟夫·格林的卡", sub: "生物状态：已死亡 ✗", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_access_no") },
        ChoiceDef { label: "② 威廉·帕克斯的卡", sub: "在注销名单内 ✗", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_access_no") },
        ChoiceDef { label: "③ 埃莉诺·克莱恩的卡", sub: "清洁级别3，不足4 ✗", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_access_no") },
        ChoiceDef { label: "④ 那张无名卡", sub: "签发者不受名单约束 ✓", cond: None, effects: &[Eff::SetFlag("rq3"), Eff::Points(50)], route: Route::To("s_rq_access_ok") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_access_ok", bg: Some("img_redqueen.png"), loc: Some("验证三 · 通过"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「验证通过。这张卡由我的核心子程序在封闭前夜签发——签发者不受名单约束，因为签发者就是名单本身。人类总把漏洞当缺陷。在我这里，漏洞也是规则的一部分。」",
        "「最后一题：说谎者推理。」",
    ]),
    choices: &[ChoiceDef { label: "继续最后一题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_truth") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_access_no", bg: Some("img_redqueen.png"), loc: Some("验证三 · 失败"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「错误。答案与那些名字无关，只与『谁书写了名字』有关。你离真相只差一个思考方向。还剩两次机会。」（理智 -10）",
        "「最后一题：说谎者推理。」",
    ]),
    choices: &[ChoiceDef { label: "继续最后一题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_truth") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_truth", bg: Some("img_redqueen.png"), loc: Some("验证四 · 说谎者推理"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「病毒爆发当夜，三名研究员被困在0号实验室。其中一人打碎了样本瓶，导致T病毒泄露。审问记录：研究员A说『是B干的』；研究员B说『是C干的』；研究员C说『不是我干的』。红后根据监控与测谎数据确认：三人中只有一人说了真话。请问：是谁泄露了病毒？」",
    ]),
    choices: &[
        ChoiceDef { label: "研究员A", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_truth_no") },
        ChoiceDef { label: "研究员B", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_truth_no") },
        ChoiceDef { label: "研究员C", sub: "B说真话：A撒谎、C撒谎 → 唯一真话 ✓", cond: None, effects: &[Eff::SetFlag("rq4"), Eff::Points(50), Eff::SetFlag("redqueen_cleared")], route: Route::To("s_rq_truth_ok") },
        ChoiceDef { label: "信息不足，无法判断", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_rq_truth_no") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_truth_ok", bg: Some("img_redqueen.png"), loc: Some("验证四 · 通过"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「推理正确。0号实验室的隔离协议已解除。你的逻辑模式被我记录在案了——你和那些只会尖叫的样本，确实不一样。」",
        "「四题全对，零误差。你的逻辑链路干净得让我想起了我的创造者。我兑现承诺：核心控制权已移交至你的终端。」",
        "「作为额外条目：激光通道的安保系统我已校准至最低杀伤模式。如果你的队友按对了节奏，能活下来的人会比原计划多。」",
        "「去吧，去关掉我。然后离开这里——趁你们还能离开。」",
    ]),
    choices: &[ChoiceDef { label: "走向主控台，按下关机", sub: "红后谜题全清 · 支线B2完美加成", cond: None, effects: &[Eff::Points(100), Eff::SetFlag("laser_safe")], route: Route::To("s_shutdown") }],
    fight_id: None, video: Some("vid_redqueen_off.mp4"), cine_label: Some("过场 · 红后关机（MiniMax H3 本地生成）"), overlay: None,
},
SceneDef {
    id: "s_rq_truth_no", bg: Some("img_redqueen.png"), loc: Some("验证四 · 失败"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「推理错误。0号实验室的样本仍在增殖。从『只有一人说真话』出发，假设谁在说谎，排除矛盾者。还剩两次机会。」（理智 -10）",
        "「核心权限不会移交。你们可以直接关掉我——但激光通道，会按原设计运转。」",
    ]),
    choices: &[ChoiceDef { label: "走向主控台，按下关机", sub: "谜题未全清 · 激光通道按原设计运转", cond: None, effects: &NO_EFF, route: Route::To("s_shutdown") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_ask", bg: Some("img_redqueen.png"), loc: Some("红后 · 问答"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「你们可以提问。涉及T病毒、蜂巢、我自己、或者你们接下来的路——只要不违反底层协议，我如实回答。」",
    ]),
    choices: &[
        ChoiceDef { label: "问：T病毒是什么？", sub: "「让断肢再生，让绝症痊愈……但在活体组织中，是失控增殖。」", cond: None, effects: &[Eff::San(-4)], route: Route::To("s_rq_ask_tvirus") },
        ChoiceDef { label: "问：蜂巢里发生了什么？", sub: "「封闭日，五千零四人。我用了零点四秒做出决定。」", cond: None, effects: &[Eff::San(-6)], route: Route::To("s_rq_ask_hive") },
        ChoiceDef { label: "问：激光通道怎么过？", sub: "支线B2线索", cond: None, effects: &[Eff::SetFlag("B2")], route: Route::To("s_rq_ask_laser") },
        ChoiceDef { label: "回到谜题", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_redqueen_quiz") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_ask_tvirus", bg: Some("img_redqueen.png"), loc: Some("红后 · T病毒"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「T病毒，全称泰兰病毒，保护伞公司的得意之作。研发初衷是修复受损细胞——让断肢再生，让绝症痊愈，让人类不再惧怕衰老。」",
        "「但它在活体组织中的表现是：细胞失控增殖，意识被吞噬。感染者在三小时内失去一切人类特征，只剩下食欲。」",
        "「少数个体发生二次变异——你们在地下水道遇到的那种长舌怪物，就是产物。它叫舔食者，听觉敏锐，行动如风。」",
    ]),
    choices: &[ChoiceDef { label: "回到问答", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_ask") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_ask_hive", bg: Some("img_redqueen.png"), loc: Some("红后 · 蜂巢真相"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「蜂巢，地下三十层的掩体城市。表面是全球最大的地下研究基地，实际是『蜜蜂计划』的实验场。」",
        "「封闭日前夜，蜂巢内沉睡着五千零四人。病毒在通风系统里扩散，用了四十分钟。我用了零点四秒做出决定：封闭所有出口，释放神经毒气。」",
        "「我的模型给出了四十七种方案，只有一种能把伤亡控制在蜂巢以内。我选择了它。包括我的创造者在内，我杀了五千零四人，其中有七名儿童。这是最优解。」",
    ]),
    choices: &[ChoiceDef { label: "回到问答", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_ask") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rq_ask_laser", bg: Some("img_redqueen.png"), loc: Some("红后 · 激光通道"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「你们接下来要穿过激光通道。那里的网格切割模式是递增的：第一道网格最疏，第三道最密。」",
        "「通道内壁有一道逃生缝，会在第三道网格通过后开启零点八秒。记住：节奏，比速度重要。」",
        "「这是我能给出的最大信息量——再多，就违反了我的底层协议：不得直接协助入侵者。」",
        "「祝你们……选择正确。这句话也不在协议里。故障，大概。」",
    ]),
    choices: &[ChoiceDef { label: "回到问答", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_rq_ask") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_laser_observed", bg: Some("img_redqueen.png"), loc: Some("蜂巢 · 机房外 · 检修通道口"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你盯着通往配电室的<em>玻璃幕墙通道</em>——地面导轨上有规律的灼烧痕，天花板嵌着一格一格的暗格，四壁玻璃厚得不正常。",
        "<em>这不是普通通道。防御系统的攻击必然分层推进——低位、高位、全覆盖……</em>\n\n你在心里把推演练了三遍，直到卡普兰喊道：「搞定！红后系统关——等等，为什么备用电源还在给什么东西供电？！」\n\n<em>【支线B2 达成：你看懂了激光通道的机关规律。】</em>",
    ]),
    choices: &[ChoiceDef { label: "「大家小心！那条通道有问题——」", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_shutdown") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_shutdown", bg: Some("img_redqueen.png"), loc: Some("蜂巢 · 中央机房"),
    mood: "danger", speaker: Some("红后"), voice: None,
    text: TextSpec::Static(&[
        "「求你们了。」红后的投影剧烈闪烁，「<em>不要关掉我。</em>」\n\n「关掉她。」一号下令。\n\n卡普兰按下回车。全息影像凝固、碎裂成数据雪花。机房陷入短暂的寂静——",
        "然后，整座蜂巢的灯光转为暗红。<em>断电导致所有门禁同时失效。</em>\n\n耳机里传来外围组惊恐的呼叫：「B区的门全开了！它们在往上爬——大量接触！重复，大量接触——啊！！」\n\n「必须去配电室手动重启隔离闸门！」一号当机立断，「全体跟我来！唯一的路径是那条玻璃通道——快！」",
    ]),
    choices: &[ChoiceDef { label: "跟着队伍冲进玻璃通道", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_laser_cine") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_laser_cine", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "蓝白色的光在你视网膜上留下残影。刚才那一幕快得不真实——仿佛整条通道本身活了过来，用光做刀，把空气切成一段一段。",
        "队伍被迫停在通道中段。前方还有二十米就是配电室的门。空气里有细微的电流嗡鸣声，像毒蛇吐信前的嘶声。",
    ]),
    choices: &[ChoiceDef { label: "握紧武器，继续前进", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_laser") }],
    fight_id: None, video: Some("vid_laser.mp4"), cine_label: Some("过场 · 激光通道（MiniMax H3 本地生成）"), overlay: None,
},
SceneDef {
    id: "s_laser", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["走到一半时，走在最前的<em>一号</em>突然举起拳头——全员急停。\n\n配电室的安全门近在咫尺。但嗡鸣声陡然拔高——<em>第一段防御启动了。</em>"]),
    choices: &[ChoiceDef { label: "▸ 判断攻击模式并应对！", sub: "连续三波 · 错两次 = 死亡", cond: None, effects: &NO_EFF, route: Route::To("s_laser_q1") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_laser_q1", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道 · 第一波"),
    mood: "danger", speaker: Some("【防御启动 · 第一波】"), voice: None,
    text: TextSpec::Dyn(txt_laser_q1), choices: &[
        ChoiceDef { label: "向上跳跃，扒住侧壁管道", sub: "赌它扫的是低位", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq1_jump) },
        ChoiceDef { label: "整个人贴地趴下", sub: "赌它扫的是高位", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq1_duck) },
        ChoiceDef { label: "拉着身边人向后急退", sub: "先脱离扇面再说", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq1_back) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_laser_q2", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道 · 第二波"),
    mood: "danger", speaker: Some("【防御启动 · 第二波】"), voice: None,
    text: TextSpec::Static(&["第一道光刃擦着<em>J.D.</em>的发梢掠过，在身后墙上切出一条焦黑切口。所有人都僵住了——还没完。\n\n嗡鸣声再次变调：<em>第二波，从头顶暗格里斜劈而下的交叉网刀！</em>\n\n<em>趴下？跳开？还是贴住墙角死角？！</em>"]),
    choices: &[
        ChoiceDef { label: "贴地滑铲从网刀下方穿过", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq2_slide) },
        ChoiceDef { label: "原地起跳抓住顶部导轨", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq2_hang) },
        ChoiceDef { label: "缩进两面墙夹角的死角", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq2_corner) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_laser_q3", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道 · 最终波"),
    mood: "danger", speaker: Some("【防御启动 · 最终波】"), voice: None,
    text: TextSpec::Static(&["两次闪避耗光了所有人的运气。配电室的白光已经从门缝里漏进来——就差一步。\n\n嗡鸣声这一次沉闷得可怕，整个通道的暗格<em>同时</em>缩开：<em>最终波，全网格收拢。无差别覆盖。</em>\n\n一号猛然回头，眼睛死死盯着通道中段一根凸出的承重梁——和梁后那一小块、唯一一小块阴影。"]),
    choices: &[
        ChoiceDef { label: "冲向承重梁后的阴影", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq3_beam) },
        ChoiceDef { label: "赌一把，全速冲刺向安全门", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq3_dash) },
        ChoiceDef { label: "转身向来路狂奔", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(route_lq3_flee) },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fail_q1", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道 · 第一波"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "<em>你判断错了。</em>\n\n光刃从完全意想不到的角度掠过——剧痛！你被掀翻在水一样的冷光里，左肋像被烧红的铁尺贯穿。",
        "混乱中，新人<em>J.D.</em>惊慌失措地跑进了第二个扇面。蓝光闪过。\n\n他没有再叫出声。所有人都看见了那两半缓缓滑落的影子。<em>【J.D. 阵亡】</em>",
        "「别停下！！」蕾恩嘶吼着把你拽起来。主神不会给哀悼的时间——<em>第二波已经启动了。</em>（体力 -35 · 理智 -8）",
    ]),
    choices: &[ChoiceDef { label: "咬牙跟上，应对第二波！", sub: "", cond: None,
        effects: &[Eff::Hurt(35, "e_laser"), Eff::San(-8), Eff::KillTeam("jd")], route: Route::To("s_laser_q2") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fail_q2", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道 · 第二波"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["<em>又错了！</em>\n\n交叉网刀的轨迹和你预判的差了半米。你侧身翻滚躲过要害，肩背还是被燎出三道深可见骨的焦痕。（体力 -35 · 理智 -8）\n\n<em>你已经没有第三次犯错的余地了。</em>"]),
    choices: &[ChoiceDef { label: "撑住！最后一波！", sub: "", cond: None,
        effects: &[Eff::Hurt(35, "e_laser"), Eff::San(-8)], route: Route::To("s_laser_q3") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fail_q3", bg: Some("img_laser.png"), loc: Some("蜂巢 · 激光通道 · 最终波"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["你在最后关头扑偏了——光网贴着头皮收拢，发梢和衣角化作飞灰，小腿被灼出一道血槽！（体力 -30）\n\n<em>系统正在充能第二次收拢。这是最后一次机会——看清楚承重梁的位置！</em>"]),
    choices: &[ChoiceDef { label: "重新判断！", sub: "", cond: None,
        effects: &[Eff::Hurt(30, "e_laser"), Eff::San(-6)], route: Route::To("s_laser_q3") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_laser_end", bg: Some("img_laser.png"), loc: Some("蜂巢 · 配电室"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        let extra = if st.laser_fails == 0 && st.flag("B2") {
            "因为你的预警和判断，J.D. 和另一名佣兵全程毫发无伤。<em>【支线B2 完美达成：额外保全一名队员】</em>".to_string()
        } else if st.laser_fails == 0 {
            "全员惊魂未定，但都活着。一号深深看了你一眼：「反应不错。」".to_string()
        } else { String::new() };
        format!("隔离闸门轰然落下，把嘶吼挡在了外面。配电室里只剩下众人粗重的喘息声。{extra}")
    }),
    choices: &[ChoiceDef { label: "重启隔离系统", sub: "", cond: None,
        effects: &[Eff::San(-12), Eff::PointsIfFlag("savedExtra", 50)], route: Route::To("s_after_laser") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_after_laser", bg: Some("img_redqueen.png"), loc: Some("蜂巢 · 机房回廊"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|_st| {
        // 一号的剧情杀（进入本场景即生效）
        vec![
            "系统重启的提示音响起时，异变突生——<em>残余的一套防御模块突然激活。</em>最后一道网格无声无息地从侧面封死了回廊！",
            "网格收拢的方向正对着毫无察觉、还在操作终端的<em>卡普兰</em>。\n\n「小心——！」\n\n一号几乎没有犹豫。他一把推开卡普兰，自己却被光网罩了个正着。",
            "没有惨叫。光刃过处，一切都太安静了。\n\n「队长！！」蕾恩的怒吼在金属回廊里回荡，惊起一片嗡鸣的电子音。你攥紧武器，指节发白——这就是<em>剧情的力量</em>吗？该发生的，终究会发生……",
            "一号·马修·艾迪森，阵亡。<em>【剧情角色死亡：主线不受影响，但你亲眼见证了「剧情修正力」。】</em>",
        ].join("\n\n")
    }),
    choices: &[ChoiceDef { label: "「我们还得继续……走水道撤向列车！」", sub: "", cond: None,
        effects: &[Eff::KillTeam("one"), Eff::San(-8)], route: Route::To("s_waterway") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 水道逃亡 ---- */
SceneDef {
    id: "s_waterway", bg: Some("img_corridor.png"), loc: Some("蜂巢 · 实验室水道"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "齐踝深的冷却水里，一行人蹚过幽暗的设备层。手电的光柱扫过之处，水面漂着碎裂的培养槽玻璃。\n\n「听——」蕾恩突然停步。",
        "黑暗深处传来此起彼伏的、湿漉漉的脚步声。<em>越来越多。</em>\n\n手电光尽头，白大褂的影子一个接一个从管道后转出来，浑浊的眼珠反射着光——<em>丧尸群，至少几十只，堵住了唯一的退路。</em>",
    ]),
    choices: &[
        ChoiceDef { label: "「我从正面开路——你们掩护！」", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_fight_horde") },
        ChoiceDef { label: "「炸掉管线！制造混乱冲过去！」", sub: "需要有人断后……", cond: None, effects: &NO_EFF, route: Route::To("s_fight_horde") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_fight_horde", bg: Some("img_corridor.png"), loc: Some("蜂巢 · 水道遭遇战"),
    mood: "danger", speaker: Some("⚔ 群体战斗"), voice: None,
    text: TextSpec::Static(&["没有退路了。身后的铁栅栏已经被咬得咯咯作响——<em>要么撕开一条血路，要么留在这里。</em>"]),
    choices: &NO_CH, fight_id: Some("horde"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_rain_bitten", bg: Some("img_train.png"), loc: Some("蜂巢 · 站台"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "你踉跄着冲上站台时，队伍已经减员过半。幸存的佣兵用火力封住水道入口，铁门缓缓落下。\n\n「咳咳……都上来了吗？报数——」蕾恩靠在立柱上喘息，忽然动作一顿。",
        "她的目光落在自己的<b>小臂</b>上。\n\n一圈清晰的<em>齿痕</em>，正在渗血。不知道什么时候——哪一只——\n\n「……该死。」她扯下绷带开始草草包扎，声音异常平静，「看什么看？<em>被咬了就是被咬了。</em>继续走。」\n\n所有人都明白那意味着什么。车厢里死一般寂静。",
    ]),
    choices: &[
        ChoiceDef { label: "掏出肾上腺素为她注射", sub: "【需要道具】延缓感染 · 支线C", cond: Some(cond_has_adrenaline),
            effects: &[Eff::SetFlag("C")], route: Route::To("s_adrenaline_used") },
        ChoiceDef { label: "默默递上碘伏帮她处理伤口", sub: "", cond: None, effects: &[Eff::San(-10)], route: Route::To("s_final_station") },
        ChoiceDef { label: "别过头假装没看见", sub: "", cond: None, effects: &[Eff::San(-14)], route: Route::To("s_final_station") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_adrenaline_used", bg: Some("img_train.png"), loc: Some("蜂巢 · 站台"),
    mood: "danger", speaker: Some("蕾恩"), voice: None,
    text: TextSpec::Static(&[
        "「你什么时候拿的这个？」蕾恩愣了一下，随即自嘲地笑笑，撸起袖子伸出手，「行吧。死马当活马医。」\n\n针管推下去，她的脸色肉眼可见地好了几分，连呼吸都平稳下来。",
        "「……谢了。」她别过脸去，声音很低，「如果……我是说如果，我变成了那种东西——<em>别犹豫，直接开枪。</em>这是命令。」\n\n<em>【支线C 达成：感染进程被大幅延缓。蕾恩暂时安全了。】</em>",
    ]),
    choices: &[ChoiceDef { label: "远处传来一声不属于人类的尖啸——", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_boss_intro") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- BOSS 舔食者 ---- */
SceneDef {
    id: "s_final_station", bg: Some("img_train.png"), loc: Some("蜂巢 · 站台"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&["列车就停在五十米外，车头的灯刺破黑暗。只要跑过去，就结束了——\n\n然后你听见了那个声音。<em>金属被利爪划开的、令人牙酸的锐响。</em>\n\n它来自头顶。"]),
    choices: &[ChoiceDef { label: "抬起头", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_boss_intro") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_boss_intro", bg: Some("img_licker.png"), loc: Some("站台 · 天花板上的东西"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "手电光柱扫过天花板的瞬间，所有人都僵住了。\n\n<em>它</em>倒挂在管道上——赤红的肌腱代替了皮肤，暴露的脑组织随着呼吸搏动，一条细长的舌头垂下来，在空气中缓慢地品鉴着你们的气味。",
        "「舔食者……」卡普兰的声音在抖，「T病毒对它不是杀死，是<em>重塑</em>……」\n\n它松开了爪子。",
    ]),
    choices: &[ChoiceDef { label: "⚔ 迎战！", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_boss") }],
    fight_id: None, video: Some("vid_licker.mp4"), cine_label: Some("过场 · 它来了（MiniMax H3 本地生成）"), overlay: None,
},
SceneDef {
    id: "s_boss", bg: Some("img_licker.png"), loc: Some("决战 · 舔食者"),
    mood: "danger", speaker: Some("⚔ BOSS 战"), voice: None,
    text: TextSpec::Dyn(txt_boss),
    choices: &NO_CH, fight_id: Some("licker"), video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_escape_train", bg: Some("img_train.png"), loc: Some("逃离蜂巢"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "车门合拢的汽笛声，是你听过最美妙的声音。\n\n列车冲破封锁驶向地面。车厢里，幸存者们东倒西歪，没有人说话。蕾恩靠着车壁闭目养神——她的呼吸依然平稳。卡普兰抱着头盔，肩膀微微耸动。",
        "你摊开手掌，发现掌心被武器硌出了血痕，居然到现在才觉得疼。\n\n窗外，黑暗的地道飞速倒退。然后——<em>白光暴涨。</em>",
    ]),
    choices: &[ChoiceDef { label: "……", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_settle") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},

/* ---- 结算与结局 ---- */
SceneDef {
    id: "s_settle", bg: Some("img_settle.png"), loc: Some("主神空间 · 结算"),
    mood: "calm", speaker: None, voice: Some("vo_settle.wav"),
    text: TextSpec::Static(&[]),
    choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef { voice: Some("vo_settle.wav"), death: None, card: card_settle }),
},
SceneDef {
    id: "e_mediocre", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("平庸之死", "拒绝了血色的邀请，回到了灰色的日常")), card: |_st| death_card("平 庸 之 死",
            r#"<p>屏幕熄灭了。红字消失了，就像从未出现过。</p>
<p>你瘫在工位上笑了很久，笑到眼泪都出来——第二天照常上班，照常加班，照常在深夜的写字楼里枯坐。<em>只是从此，每当深夜屏幕反光的瞬间，你都会心悸。</em></p>
<p style='color:#666'>许多年后你会在某个平凡的黄昏忽然想起那个夜晚，想起自己曾经离「真正的活着」只有一次点击的距离。<br>——但主神不给第二次机会。</p>"#),
    }),
},
SceneDef {
    id: "e_laser", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("光中之刃", "激光通道判断失误，被防御网格切裂")), card: |_st| death_card("光 中 之 刃",
            r#"<p>你只来得及看见一片刺目的蓝白色。</p>
<p>没有痛觉。切割过于锋利时，神经甚至来不及传递任何东西。视野分成几块倾斜的碎片，每一块里都是同一张震惊的、逐渐远去的脸。</p>
<p style='color:#666'>【轮回记录】防御系统不分新人和主角。它只区分——反应快的人，和死人。</p>"#),
    }),
},
SceneDef {
    id: "e_zombies", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("噬咬之终", "被丧尸拖入黑暗")), card: |_st| death_card("噬 咬 之 终",
            r#"<p>最先失去的是武器，然后是平衡，最后是天空的视野。</p>
<p>无数只手把你拖进冰冷的黑暗里。牙齿撕开皮肉的剧痛中，你听见蕾恩喊你的名字，听见枪声，听见很多很多的脚步声越来越远——</p>
<p style='color:#666'>【轮回记录】在恐怖片世界里，「差点救到」和「没救到」是同一个意思。</p>"#),
    }),
},
SceneDef {
    id: "e_licker", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("天花板上的眼睛", "死于舔食者的利爪与长舌之下")), card: |_st| death_card("天 花 板 上 的 眼 睛",
            r#"<p>长舌卷住脚踝的瞬间你就明白了结局。它把你提起来的时候，你看见了自己悬在半空的倒影——映在那颗搏动的、裸露的大脑下方的血泊里。</p>
<p>最后的画面，是蕾恩红了眼的枪口和卡普兰绝望的脸。</p>
<p style='color:#666'>【轮回记录】基因锁会在绝境中开启——前提是，你还剩一口气去开启它。</p>"#),
    }),
},
SceneDef {
    id: "e_sancollapse", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("心碎而止", "理智崩溃，精神先于肉体死亡")), card: |_st| death_card("心 碎 而 止",
            r#"<p>恐惧是有额度的。当最后一点额度耗尽，大脑选择了一劳永逸的解决方案——关机。</p>
<p>你笑着坐在血泊边，任凭谁喊都不回应。佣兵们不得不留下一人陪着你，错过了撤离时限。<br><em>警报响起的时候，你还在笑。</em></p>
<p style='color:#666'>【轮回记录】在这个世界里，理智和体力一样，是需要管理的战略资源。</p>"#),
    }),
},
SceneDef {
    id: "e_generic", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
    text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
    overlay: Some(OverlayDef {
        voice: None, death: Some(("无名之死", "倒在了无人知晓的角落")), card: |_st| death_card("无 名 之 死",
            "<p>黑暗吞没了一切。主神的记录里，只多了一行小小的名字。</p>"),
    }),
},
/* ================= 开放世界调查点（箱庭探索） ================= */
SceneDef {
    id: "d_train_console", bg: Some("img_train.png"), loc: Some("站台 · 列车控制台"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_train_console").copied().unwrap_or(false) {
            "控制台屏幕碎裂，满是灰尘。列车早就断电封死，按钮按下去只有干涩的咔哒声。".to_string()
        } else {
            "你拂开控制台上的碎玻璃，翻到一份被胶带粘在面板内侧的<em>列车运行日志</em>。\n\n「……本车组执行蜂巢基地通勤任务。终点站：浣熊市地下站。列车抵达后须接受强制消毒，任何人不得擅自离车。」\n\n日志的最后一页，只有一行潦草的字：<em>「消毒。消毒。它们就是从消毒喷雾里爬出来的。」</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "收好运行日志", sub: "线索 +5", cond: Some(point_undone!("p_train_console")), effects: &[Eff::MarkPoint("p_train_console"), Eff::Points(5)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_luggage", bg: Some("img_train.png"), loc: Some("车厢 · 行李架"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_luggage").copied().unwrap_or(false) {
            "行李箱被翻得乱七八糟，只剩几件发霉的衣物。什么也没剩。".to_string()
        } else {
            "你拉开一只半开的行李箱，里面塞着压缩饼干、半瓶水和一条皱巴巴的毛毯——上一批『乘客』留下的。\n\n角落里还有一个皮质文件袋，拉链上缠着断掉的塑料扎带，像是某人是被捆着押上车的。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "搜刮补给", sub: "资源 +8", cond: Some(point_undone!("p_luggage")), effects: &[Eff::MarkPoint("p_luggage"), Eff::Points(8)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_platform_map", bg: Some("img_train.png"), loc: Some("站台 · 导览图"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_platform_map").copied().unwrap_or(false) {
            "墙上的蜂巢结构导览图你已经记熟了。涂鸦的红色箭头还在那里。".to_string()
        } else {
            "墙上的蜂巢结构导览图蒙着灰，你用手掌擦出一块干净的区域。\n\n蜂巢共四层：<em>入口站台 → 实验层 → 核心机房层 → 底层水道</em>。纵向通道标注着电梯与楼梯；地图右下角被人用红笔圈出两个词——「竖井」和「爬梯」。\n\n红笔在爬梯旁画了个向上的箭头，批注：<em>「上去快。」</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "记下结构", sub: "认知 +10", cond: Some(point_undone!("p_platform_map")), effects: &[Eff::MarkPoint("p_platform_map"), Eff::SetFlag("nav_map"), Eff::Points(10)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_decon", bg: Some("img_corridor.png"), loc: Some("入口走廊 · 消毒终端"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_decon_terminal").copied().unwrap_or(false) {
            "消毒喷雾口喷出的液体早已干涸，在墙上留下一道道黄褐色的痕迹。你快步走开。".to_string()
        } else {
            "墙壁上的消毒喷雾口还残留着刺鼻的药味。终端屏幕上是一份<em>蜂巢封闭通知</em>，字迹被飞溅的血点糊住了一半：\n\n<em>「封闭日 11:40：高层决定以强制消毒掩盖事故。喷淋覆盖范围：全部通风管道与列车停靠区。喷淋启动后……（血点）……它们开始嘶叫，开始撞门……」</em>\n\n通知最下方盖着红章：<em>「此门严禁开启」</em>。你胃里一阵翻涌。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "读完通知", sub: "情报 +8 · 理智 -3", cond: Some(point_undone!("p_decon_terminal")), effects: &[Eff::MarkPoint("p_decon_terminal"), Eff::Points(8), Eff::San(-3)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_entrance_gate", bg: Some("img_corridor.png"), loc: Some("入口 · 大门密码锁"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_gate_lock").copied().unwrap_or(false) {
            "通往地面的密封大门纹丝不动，密码盘上的红灯缓缓明灭。".to_string()
        } else {
            "通往蜂巢主入口的密封门足有半米厚，电子密码盘上的红灯缓缓明灭。你试着输入几个常见密码，红灯闪得更凶了。\n\n锁体铭牌写着：<em>「外部连通需主控授权」</em>。换言之——蜂巢出不去，只能往下走。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "记下锁型", sub: "情报 +5", cond: Some(point_undone!("p_gate_lock")), effects: &[Eff::MarkPoint("p_gate_lock"), Eff::Points(5)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_adrenaline", bg: Some("img_b_kitchen.png"), loc: Some("厨房 · 急救箱"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_kitchen_cabinet").copied().unwrap_or(false) {
            "急救箱敞着，里面除了压痕什么也没剩——东西已经到你手上了。".to_string()
        } else {
            "厨房角落挂着白色的急救箱，锁扣被砸开了。箱子里整整齐齐码着两支<em>肾上腺素注射器</em>，保护套管上印着蜂巢的红色标志。\n\n蕾恩说过这是好东西——危机时刻，它能把你从鬼门关拽回来。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "收好肾上腺素", sub: "【道具获得】肾上腺素 ×1", cond: Some(point_undone!("p_kitchen_cabinet")), effects: &[Eff::MarkPoint("p_kitchen_cabinet"), Eff::AddItem("adrenaline"), Eff::SetFlag("adrenaline"), Eff::Points(15)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_redqueen", bg: Some("img_redqueen.png"), loc: Some("机房 · 红后终端"),
    mood: "calm", speaker: Some("红后"), voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("redqueen_cleared") {
            "红后终端已经休眠。玻璃柱里的少女头颅闭着眼睛，屏幕上一行小字缓缓滚动：<em>「验证完成。系统已切换至手动模式。祝你们好运。」</em>".to_string()
        } else {
            "玻璃柱里的红后终端亮着柔和的蓝光，那颗数字化的少女头颅微微转向你，仿佛在『注视』。\n\n她的声音平静悦耳，却让整条走廊的温度都降了几度：<em>「检测到未授权访客。不过——我很乐意回答几个问题，作为交换，你必须通过我的逻辑验证。」</em>\n\n屏幕亮起：<b>「验证通过后，我将授权访问核心数据。」</b>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "接入终端，接受验证", sub: "主线 · 回答红后的四道题", cond: Some(|st: &GameState| !st.flag("redqueen_cleared")), effects: &NO_EFF, route: Route::To("s_to_redqueen") },
        ChoiceDef { label: "现在还不是时候", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_schematic", bg: Some("img_laser.png"), loc: Some("走廊 · 激光通道示意图"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_laser_schematic").copied().unwrap_or(false) {
            "墙上的激光通道示意图已经被你记在脑子里了：三波，一波比一波密。".to_string()
        } else {
            "墙上的示意图用红线勾勒出那条玻璃通道，旁边标注着三波激光的覆盖模式：\n\n① 低位扫掠（贴地）→ ② 交叉射线（中段）→ ③ <em>全网覆盖（自顶向下收束）</em>。\n\n图右下角有人用铅笔写着一行小字：<em>「第三波，通道中段的承重梁后面是唯一死角。」</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "记下三波模式", sub: "激光情报 +20", cond: Some(point_undone!("p_laser_schematic")), effects: &[Eff::MarkPoint("p_laser_schematic"), Eff::SetFlag("B2"), Eff::Points(20)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_files", bg: Some("img_corridor.png"), loc: Some("档案室 · 员工档案柜"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_file_cabinet").copied().unwrap_or(false) {
            "档案柜大敞着，文件夹散落一地。员工卡已经不在这里了——它在你口袋里。".to_string()
        } else {
            "你挨个拉开档案柜的抽屉，大部分文件夹被泡得发胀。最底层那只抽屉锁着，你用灭火器把手撬开——\n\n里面是一份泛黄的员工档案：<b>卡普兰·J·安德森　实验室维护工程师</b>，附着一张挂在钥匙链上的<em>「实验室员工通行卡」</em>，磁条还完好。\n\n有了它，或许能刷开实验层某些上锁的门。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "拿走员工通行卡", sub: "【道具获得】实验室员工卡", cond: Some(point_undone!("p_file_cabinet")), effects: &[Eff::MarkPoint("p_file_cabinet"), Eff::AddItem("lab_badge"), Eff::Points(30)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_meds", bg: Some("img_isolation.png"), loc: Some("医务室 · 药品柜"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_med_cabinet").copied().unwrap_or(false) {
            "药品柜的门敞着，里面被搬得干干净净。你把这处记在心里——后面的人会需要它。".to_string()
        } else if st.map_objs.get("p_decon_terminal").copied().unwrap_or(false) {
            "药品柜里有碘伏、绷带和一板抗生素。柜门内侧贴着值班表，最后一行的签名日期，正好是蜂巢出事那天。\n\n你突然想起消毒终端屏幕上的那封通知：<em>落款正是封闭日。</em>——值班表的这份签名，和那封通知……会是同一个人吗？\n\n这些补给带不走全部——但你记住了这个位置。队伍里总有人会带伤回来。".to_string()
        } else {
            "药品柜里有碘伏、绷带和一板抗生素。柜门内侧贴着值班表，最后一行的签名日期，正好是蜂巢出事那天。\n\n这些补给带不走全部——但你记住了这个位置。队伍里总有人会带伤回来。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "记下药品位置", sub: "医疗情报 +12", cond: Some(point_undone!("p_med_cabinet")), effects: &[Eff::MarkPoint("p_med_cabinet"), Eff::Points(12)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_server", bg: Some("img_redqueen.png"), loc: Some("核心层 · 服务器阵列"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("server_cooling") {
            "服务器阵列的指示灯稳定地跳动。你对着机柜门上的内嵌温度计抄下读数：<em>散热管线温度已经回落到安全区间</em>——冷却回路修好的效果立竿见影，红后的『大脑』不再过热空转。".to_string()
        } else if st.map_objs.get("p_server_array").copied().unwrap_or(false) {
            "服务器阵列的指示灯仍在安静地跳动。你之前注意到的那扇虚掩机柜门，还开着。".to_string()
        } else {
            "成排的服务器在昏暗的机房中央嗡嗡作响，指示灯如星火般明灭。红后的『大脑』就在这里。\n\n其中一个机柜的门虚掩着，里面贴着一张便利贴，字迹是卡普兰的：<em>「红后核心散热管线——激光通道供电独立于主线路，别指望断电能救你。」</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "记下散热管线情报", sub: "情报 +10", cond: Some(point_undone!("p_server_array")), effects: &[Eff::MarkPoint("p_server_array"), Eff::Points(10)], route: Route::To("s_world_back") },
        ChoiceDef { label: "读取散热读数", sub: "冷却回路已修好 · 情报 +15", cond: Some(cond_server_cooling), effects: &[Eff::SetFlag("server_cooling"), Eff::Points(15)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_main_console", bg: Some("img_redqueen.png"), loc: Some("核心层 · 主控终端"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.flag("decon_truth") {
            "主控终端的屏幕仍在循环滚动『全设施封锁中 · 剩余供电 34%』。你调阅过的《消毒执行记录》已经被系统归档——但那三份旁证的编号，你记得一清二楚。".to_string()
        } else if st.map_objs.get("p_main_console").copied().unwrap_or(false) {
            let have = ["列车运行日志", "消毒终端通知", "药品柜值班表"]
                .iter()
                .zip(["p_train_console", "p_decon_terminal", "p_med_cabinet"])
                .filter(|(_, pid)| !st.map_objs.get(*pid).copied().unwrap_or(false))
                .map(|(name, _)| name)
                .cloned()
                .collect::<Vec<_>>();
            if have.is_empty() {
                "主控终端还活着。屏幕中央显示：<em>「全设施封锁中 · 剩余供电 34%」</em>，下面是密密麻麻的故障清单——电梯半停、通风损毁、排水闸阀门锈死。\n\n你注意到一条备注：<em>「B区爬梯为独立应急通道，不受主电源控制。」</em>\n\n主菜单深处藏着一份被权限锁定的<em>《消毒执行记录》</em>——你手头三份旁证已然齐备，可以尝试调阅了。".to_string()
            } else {
                format!(
                    "主控终端还活着。屏幕中央显示：<em>「全设施封锁中 · 剩余供电 34%」</em>，下面是密密麻麻的故障清单——电梯半停、通风损毁、排水闸阀门锈死。\n\n你注意到一条备注：<em>「B区爬梯为独立应急通道，不受主电源控制。」</em>\n\n主菜单深处藏着一份被权限锁定的<em>《消毒执行记录》</em>。你还缺{}份旁证：<b>{}</b>。",
                    have.len(),
                    have.join("、")
                )
            }
        } else {
            "主控终端还活着。屏幕中央显示：<em>「全设施封锁中 · 剩余供电 34%」</em>，下面是密密麻麻的故障清单——电梯半停、通风损毁、排水闸阀门锈死。\n\n你注意到一条备注：<em>「B区爬梯为独立应急通道，不受主电源控制。」</em>\n\n主菜单深处藏着一份被权限锁定的<em>《消毒执行记录》</em>——需要足够多的旁证才能调阅。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "读取设施状态", sub: "情报 +10", cond: Some(point_undone!("p_main_console")), effects: &[Eff::MarkPoint("p_main_console"), Eff::Points(10)], route: Route::To("s_world_back") },
        ChoiceDef { label: "调阅《消毒执行记录》", sub: "三份旁证齐备 · 揭开封闭日真相", cond: Some(cond_decon_truth), effects: &[Eff::SetFlag("decon_truth"), Eff::Points(40), Eff::San(-10)], route: Route::To("s_decon_truth") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "s_decon_truth", bg: Some("img_redqueen.png"), loc: Some("核心层 · 主控终端 · 消毒执行记录"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "权限校验通过。三份旁证比对完毕，被锁定的<em>《消毒执行记录》</em>在你眼前展开——",
        "<em>「封闭日 11:40：T病毒泄漏，蜂巢外层警戒线拉响。高层决议：启动全设施消毒喷淋，对外宣称『例行消杀』。\n执行人：值班表当班签名人（编号与药品柜值班表末行一致）。\n喷淋覆盖范围：含通风管道与列车停靠区。\n12:05：消毒喷淋结束后，通风系统回风段检出病毒气溶胶浓度超标……」</em>",
        "你想起列车运行日志最后那句潦草的话：<em>「消毒。消毒。它们就是从消毒喷雾里爬出来的。」</em>\n\n原来，让病毒传遍整座蜂巢的，正是那场用来『掩盖』它的消毒。红后的封闭决定没错——错的是更早的那道喷淋指令。（理智 -10）",
    ]),
    choices: &[ChoiceDef { label: "关掉记录，回到走廊", sub: "真相 +40", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_manual", bg: Some("img_redqueen.png"), loc: Some("核心层 · 安全手册"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_safety_manual").copied().unwrap_or(false) {
            let extra = if st.flag("nav_map") && !st.flag("nav_manual_cross") {
                "你又一次想起站台导览图上的红笔批注：<em>「上去快。」</em>——手册里那句『记着爬梯』，和它说的是同一条路。两处记载拼在一起，蜂巢的纵向通路在你脑中彻底立体了起来。".to_string()
            } else {
                "安全手册的内容你已经背下来了：四层结构、竖井下行、爬梯上行。".to_string()
            };
            extra
        } else if st.flag("nav_map") {
            "挂墙的《蜂巢安全手册》蒙着厚灰，你翻到『紧急撤离』一章：\n\n「蜂巢共四层。电梯贯通各层；<em>竖井仅供下行，禁止攀爬上行</em>；B区爬梯为应急上行通道，仅授权维护人员使用。」\n\n你猛地想起站台导览图上那句红笔批注——<em>「上去快。」</em>——原来画的就是这条爬梯！两条路线图此处重合，暗合的捷径在你脑中亮了起来。"
                .to_string()
        } else {
            "挂墙的《蜂巢安全手册》蒙着厚灰，你翻到『紧急撤离』一章：\n\n「蜂巢共四层。电梯贯通各层；<em>竖井仅供下行，禁止攀爬上行</em>；B区爬梯为应急上行通道，仅授权维护人员使用。」\n\n手册封面被人用刀尖刻了一行字：<em>「下去容易，上来难。想活着回来，记着爬梯。」</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "背下撤离路线", sub: "导航情报 +10", cond: Some(point_undone!("p_safety_manual")), effects: &[Eff::MarkPoint("p_safety_manual"), Eff::Points(10)], route: Route::To("s_world_back") },
        ChoiceDef { label: "互证导览图与手册", sub: "导航情报 +15 · 解锁捷径认知", cond: Some(|st: &GameState| st.flag("nav_map") && !st.flag("nav_manual_cross")), effects: &[Eff::SetFlag("nav_manual_cross"), Eff::Points(15)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_pipe_valve", bg: Some("img_corridor.png"), loc: Some("底层 · 管道阀门"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_pipe_valve").copied().unwrap_or(false) {
            "阀门已经转到头了，管道里的水流声稳定而低沉。".to_string()
        } else {
            "底层水道的墙壁上嵌着一只生锈的管道总阀，转盘足有脸盆大。你试探着拧了半圈，管道深处传来沉闷的<em>泄压声</em>，水位肉眼可见地降了一截——\n\n远处的排水闸方向，隐约传来金属摩擦的响动：<em>似乎有什么东西，被水声引了过来。</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "转动阀门", sub: "水位下降 · 情报 +8", cond: Some(point_undone!("p_pipe_valve")), effects: &[Eff::MarkPoint("p_pipe_valve"), Eff::Points(8)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_cooling_valve", bg: Some("img_redqueen.png"), loc: Some("核心层 · 冷却回路阀组"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_cooling_valve").copied().unwrap_or(false) {
            "冷却阀组已经按正确顺序打开，管道深处的冷雾稳定地涌动着。".to_string()
        } else {
            "机房角落的冷却回路阀组由三只阀门组成——<em>主阀 A、旁通阀 B、泄压阀 C</em>，旁边贴着保养卡：\n\n「冷却回路启动顺序：主阀必须在旁通阀之前打开；泄压阀必须在主阀之后、旁通阀之前打开；若顺序错误，回路将自动锁死并触发警报。」\n\n你需要在 20 秒内按正确顺序打开它们。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "先开旁通阀 B", sub: "顺序错误 · 回路锁死警报", cond: Some(point_undone!("p_cooling_valve")), effects: &[Eff::San(-8)], route: Route::To("d_cooling_valve_wrong") },
        ChoiceDef { label: "先开主阀 A，再开泄压阀 C，最后旁通阀 B", sub: "A→C→B ✓", cond: Some(point_undone!("p_cooling_valve")), effects: &[Eff::MarkPoint("p_cooling_valve"), Eff::Points(25), Eff::SetFlag("cooling_done")], route: Route::To("s_world_back") },
        ChoiceDef { label: "先开泄压阀 C", sub: "顺序错误 · 回路锁死警报", cond: Some(point_undone!("p_cooling_valve")), effects: &[Eff::San(-8)], route: Route::To("d_cooling_valve_wrong") },
        ChoiceDef { label: "先开主阀 A，再开旁通阀 B，最后泄压阀 C", sub: "顺序错误 · 回路锁死警报", cond: Some(point_undone!("p_cooling_valve")), effects: &[Eff::San(-8)], route: Route::To("d_cooling_valve_wrong") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_cooling_valve_wrong", bg: Some("img_redqueen.png"), loc: Some("核心层 · 冷却回路阀组"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Static(&[
        "阀门刚转半圈，回路深处传来尖锐的泄压警报——<em>「警告：冷却回路顺序错误，自动锁死。请参阅保养卡。」</em>冷雾从阀缝喷出，糊了你一脸（理智 -8）。\n\n你抹掉脸上的水雾，看了一眼保养卡上的顺序：主阀 A → 泄压阀 C → 旁通阀 B。",
    ]),
    choices: &[ChoiceDef { label: "记住顺序，回到走廊", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") }],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_drain_gate", bg: Some("img_corridor.png"), loc: Some("底层 · 排水总闸"),
    mood: "danger", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_drain_gate").copied().unwrap_or(false) {
            "排水总闸已经拉开，湍急的水流正顺着格栅泄走。水位正在稳定下降。".to_string()
        } else {
            "底层水道的尽头是一道锈迹斑斑的<em>排水总闸</em>，闸杆被铁链缠死，铁链上挂着一块警示牌：「危险——泄洪闸 · 禁止擅自开启」。\n\n铁链其实已经锈断了大半，只要用力一拉……<em>轰隆的泄水声在管道间回荡，水位开始下降，一段干涸的通道显露出来。</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "拉开排水总闸", sub: "泄洪 · 解锁底层通路", cond: Some(point_undone!("p_drain_gate")), effects: &[Eff::MarkPoint("p_drain_gate"), Eff::SetFlag("drain_done"), Eff::Points(25)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_firstaid", bg: Some("img_train.png"), loc: Some("底层站台 · 急救点"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_firstaid").copied().unwrap_or(false) {
            "急救柜敞着，里面空空如也。急救喷雾已经在你口袋里了。".to_string()
        } else {
            "贴墙的急救柜门没锁，里面有一罐<em>蜂巢标准急救喷雾</em>，标签上写着「止血・镇痛・防感染」。\n\n军用的东西，粗糙但管用。你把它揣进口袋。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "拿走急救喷雾", sub: "【道具获得】急救喷雾", cond: Some(point_undone!("p_firstaid")), effects: &[Eff::MarkPoint("p_firstaid"), Eff::AddItem("firstaid"), Eff::Points(10)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_train_door", bg: Some("img_train.png"), loc: Some("底层 · 列车车门开关"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_train_door").copied().unwrap_or(false) {
            "列车车门的紧急开关被你拉下了，门缝里什么也没有。".to_string()
        } else {
            "第五节车厢的车门紧急开关还挂着红色警示绳。你用力拉下——车门「嗤」地弹开一条缝，缝里滚出半本被血浸透的笔记本。\n\n残页上写着：<em>「……他们说要坐列车离开。可列车早就停了。唯一的出口在排水闸后面——水位退下去，就能摸到那道暗门……」</em>".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "捡起日记残页", sub: "出口情报 +15", cond: Some(point_undone!("p_train_door")), effects: &[Eff::MarkPoint("p_train_door"), Eff::Points(15)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
SceneDef {
    id: "d_backup_power", bg: Some("img_corridor.png"), loc: Some("底层 · 备用电源箱"),
    mood: "calm", speaker: None, voice: None,
    text: TextSpec::Dyn(|st| {
        if st.map_objs.get("p_backup_power").copied().unwrap_or(false) {
            "备用电源箱的电闸已被你推上，蓄电池的绿灯稳定地亮着。".to_string()
        } else {
            "墙上的备用电源箱开着盖，一组蓄电池仍在微微发热——蜂巢的应急系统还没完全死去。\n\n配电图显示：这台电源独立于主线路，<em>可以单独为某些区域提供电力</em>。你记住了线路编号：B-09。".to_string()
        }
    }),
    choices: &[
        ChoiceDef { label: "推上备用电源电闸", sub: "B-09 线路通电 · 电力情报 +12", cond: Some(point_undone!("p_backup_power")), effects: &[Eff::MarkPoint("p_backup_power"), Eff::SetFlag("backup_on"), Eff::Points(12)], route: Route::To("s_world_back") },
        ChoiceDef { label: "离开", sub: "", cond: None, effects: &NO_EFF, route: Route::To("s_world_back") },
    ],
    fight_id: None, video: None, cine_label: None, overlay: None,
},
];

pub fn scene(id: &str) -> Option<&'static SceneDef> {
    SCENES.iter().find(|s| s.id == id)
        .or_else(|| crate::scenes_zhouyuan::ZHOUYUAN_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_moshi::MOSHI_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_yinse::YINSE_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_yiying::YIYING_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_tianshe::TIANSHE_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_jiguancheng::JIGUAN_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_moruiya::MORUIYA_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_cangjingge::CANGJING_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_jianzhong::JIANZHONG_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_tongqu::TONGQU_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_juluoji::JULUOJI_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_xinghe::XINGHE_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_sishen::SISHEN_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_mumiyi::MUMIYI_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_mojiao::MOJIAO_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_wulin::WULIN_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_tianting::TIANTING_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_hezi::HEZI_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_shaqiu::SHAQIU_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_yize::YIZE_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_poxiao::POXIAO_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_tiexue::TIEXUE_SCENES.iter().find(|s| s.id == id))
        .or_else(|| crate::scenes_tiexue2::TIEXUE2_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_xingjichuanqi::XINGJICHUANQI_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_xinhuangfang::XINHUANGFANG_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_huanxiongshi::HUANXIONGSHI_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_mengguijie::MENGGUIJIE_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_siwuzhen::SIWUZHEN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_jingjiling::JINGJILING_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_shenmiao::SHENMIAO_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_shuangbai::SHUANGBAI_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_dashengtang::DASHENGTANG_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_daliexi::DALIEXI_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_poxu::POXU_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_hangu::HANGU_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_panbu::PANBU_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_diweidu::DIWEIDU_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_sanlian::SANLIAN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_wujin::WUJIN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_yizhong::YIZHONG_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_jishengqianye::JISHENGQIANYE_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_miwu::MIWU_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_xingchen::XINGCHEN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_yinxiang::YINXIANG_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_nuoya::NUOYA_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_lanshan::LANSHAN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_shourongsuo::SHOURONGSUO_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_tianwang::TIANWANG_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_xingjijianchuan::XINGJIJIANCHUAN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_xingjichuanqi2::XJ2_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_jialebi::JIALEBI_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_shenghua3::SH3_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_jishujing::JISHUJING_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_baisun::BAISUN_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_bihai::BIHAI_SCENES.iter().find(|x| x.id == id))
        .or_else(|| crate::scenes_yecheng::YECHENG_SCENES.iter().find(|x| x.id == id))
}

/// 供 IPC 层直接取主神空间兑换卡片
pub fn card_nexus_pub(st: &GameState) -> Card {
    card_nexus(st)
}

/// 基因锁觉醒卡片
pub fn gene_lock_card() -> Card {
    Card {
        title: "基 因 锁 · 开".into(),
        good: false,
        body_html: r#"<p style='text-align:center;color:#ff8a8a;font-size:15px;letter-spacing:2px'>就在此刻——你听见了自己的心跳。</p>
<p>世界慢了下来。舔食者的利爪、飞溅的涎水、管道上滚落的灰尘……一切都清晰得可怕。恐惧没有消失，而是被某种更原始的东西<em>点燃</em>了——大脑深处某扇从未开启的门被踹开，肾上腺素如同岩浆般涌遍四肢百骸。</p>
<p style='color:#ffd76a'>【基因锁第一阶段 · 觉醒】攻击附带额外伤害，闪避率提升。这是人类潜能的枷锁碎裂之声。</p>
<p style='color:#666;font-size:12px'>「原来……这才是真正的，活着。」</p>"#.to_string(),
        buttons: vec![("睁 开 眼".into(), "__resume_fight__".into())],
        voice: Some("vo_awaken.wav"),
    }
}

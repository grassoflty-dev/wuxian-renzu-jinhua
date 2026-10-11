//! 副本 scenes · 盘部落圣遗之夜（panbu / pb_）
//! 对应 worlds：PANBU_F1_MAP..F3_MAP + POINTS/ENEMIES/NPCS/ZONES/PORTALS/GATES
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_panbu(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS =====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("pb_boss") {
            st.fight = Some(crate::power::scaled_fight("pb_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "pb_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 24 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "pb_death".to_string(); }
    "pb_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("pb_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "pb_reward");
    "pb_card".to_string()
}

pub static PANBU_SCENES: &[SceneDef] = &[
    SceneDef {
        id: "pb_00", bg: Some("panbu_bg.png"), loc: Some("入夜营地"), mood: "calm",
        speaker: Some("守夜长老"), voice: Some("vo_panbuluo_1"),
        text: TextSpec::Static(&["夜色吞没了盘部落营地。", "长老压低声音：族长藏在蛇牙祭坛，等你去夺回圣骨。"]),
        choices: &[
            ChoiceDef { label: "接过火把", sub: "", cond: None, effects: &[Eff::Points(10)], route: Route::To("pb_01") },
            ChoiceDef { label: "先问图腾", sub: "", cond: None, effects: &NO_EFF, route: Route::To("pb_00") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pb_01", bg: Some("panbu_bg.png"), loc: Some("圣遗前庭"), mood: "danger",
        speaker: Some("灵蛇族长"), voice: None,
        text: TextSpec::Static(&["蛇牙在月光下泛着幽绿。", "族长盘踞在祭坛中央，吐信冷笑：圣骨是我的。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先找刻痕", sub: "侦察+分", cond: None, effects: &[Eff::Points(15)], route: Route::To("pb_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pb_round", bg: Some("panbu_bg.png"), loc: Some("蛇牙祭坛"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("灵蛇族长剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "祭火重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 32, false)) },
            ChoiceDef { label: "举盾防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "pb_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你取回了圣骨，驱散了盘部落的圣遗之夜。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "pb_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("盘部落 · 圣遗之夜", "你倒在了蛇牙祭坛前，血渗入石板缝。")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
];

pub fn panbu_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("pb_boss", FightCfg {
            name: "灵蛇族长蛇牙祭仪", hp: 200, dmg: (16, 24), reward: 500, reward_why: "击败 BOSS", intro: "随身之蛇翻涌成海，血盆大口朝你咬下！",
            rage_at: Some(60), rage_text: "蛇鳞炸开，祭火狂燃！", on_rage: rage_panbu,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "pb_card".to_string(), death: "pb_death",
        }),
    ]
}
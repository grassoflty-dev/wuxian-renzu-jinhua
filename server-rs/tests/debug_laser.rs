use wuxian_horror_ch1::{engine, scenes, state::{GameState, Mode}};

#[test]
fn debug_after_laser_kill() {
    let mut st = GameState::new();
    let mut d: Vec<(&'static str, &'static str)> = vec![];
    engine::goto(&mut st, "s_after_laser", &mut d);
    eprintln!("scene={} mode_normal={}", st.scene_id, matches!(st.mode, Mode::Normal));
    let sc = scenes::scene("s_after_laser").unwrap();
    for c in sc.choices { eprintln!("choice: {}", c.label); }
    engine::choose(&mut st, 0, &mut d);
    eprintln!("after choose: scene={} dead={:?} san={}", st.scene_id, st.dead_team, st.san);
    assert!(st.dead_team.contains(&"one".to_string()), "一号必须阵亡");
    assert_eq!(st.scene_id, "s_waterway");
}

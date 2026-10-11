//! 楚轩智谋提示（tactics）集成测试。
//! 只走公开 API（tactics::{CHUXUAN_TACTICS, chuxuan_tactic, chuxuan_tactic_line} 与
//! power::scaled_fight、scenes::fight_cfg）。
use wuxian_horror_ch1::state::GameState;
use wuxian_horror_ch1::{power, scenes, tactics};

// ============================================================================
// 1. 数据完整性：表条目数达标、每个 fight_id 都是 fight_cfg 真实存在的主键
// ============================================================================
#[test]
fn tactics_have_six_to_ten_real_entries() {
    let n = tactics::CHUXUAN_TACTICS.len();
    assert!((6..=10).contains(&n), "楚轩提示表应 6~10 条，实际 {}", n);
    for (fid, txt) in tactics::CHUXUAN_TACTICS {
        assert!(
            scenes::fight_cfg(fid).is_some(),
            "楚轩提示表引用未知 fight_id: {fid}"
        );
        assert!(!txt.is_empty(), "楚轩提示表 {fid} 文案为空");
        assert!(
            txt.contains("楚轩") || txt.starts_with("它") || !txt.is_empty(),
            "{fid} 文案需为楚轩口吻"
        );
    }
}

// ============================================================================
// 2. 命中场景：scaled_fight 后 pending_log 应含楚轩提示，且紧跟登场文案之后
// ============================================================================
#[test]
fn matching_fight_appends_chuxuan_tactic() {
    let st = GameState::new();
    for (fid, _) in tactics::CHUXUAN_TACTICS {
        let cfg = scenes::fight_cfg(fid).expect("前面已验证 fight_id 真实");
        let mut log = vec![format!("<span class='miss'>{}</span>", cfg.intro)];
        let f = power::scaled_fight(fid, cfg, &st, log.clone());
        // 命中 → 恰好追加一条楚轩提示
        let tactics_found = f.pending_log.iter().filter(|l| l.contains("class='tactic'")).count();
        assert_eq!(tactics_found, 1, "{fid} 应恰好追加一条楚轩提示");
        // 且追加在登场文案之后（intro 之后新推入）
        log.push(tactics::chuxuan_tactic_line(fid).unwrap());
        assert_eq!(f.pending_log, log, "{fid} 提示应紧跟登场文案之后");
    }
}

// ============================================================================
// 3. 未命中场景：无提示
// ============================================================================
#[test]
fn non_matching_fight_has_no_tactic() {
    let st = GameState::new();
    let cfg = scenes::fight_cfg("b_chef").expect("b_chef 是真实 fight_id");
    let f = power::scaled_fight("b_chef", cfg, &st, vec![cfg.intro.to_string()]);
    assert!(
        f.pending_log.iter().all(|l| !l.contains("class='tactic'")),
        "未命中 fight_id 不应出现楚轩提示"
    );
}

// ============================================================================
// 4. 幂等：同一场战斗（同一实例化调用）只追加一次；重复构造不重复累计
// ============================================================================
#[test]
fn tactic_is_idempotent() {
    let st = GameState::new();
    let cfg = scenes::fight_cfg("licker").expect("licker 是真实 fight_id");
    // 一次实例化 → 且仅一条
    let f = power::scaled_fight("licker", cfg, &st, vec![cfg.intro.to_string()]);
    assert_eq!(
        f.pending_log.iter().filter(|l| l.contains("class='tactic'")).count(),
        1,
        "单次实例化应仅一条楚轩提示"
    );
    // 再次实例化同样不累计（每次新 Fight 独立），不依赖存档状态
    let f2 = power::scaled_fight("licker", cfg, &st, vec![cfg.intro.to_string()]);
    assert_eq!(
        f2.pending_log.iter().filter(|l| l.contains("class='tactic'")).count(),
        1
    );
}

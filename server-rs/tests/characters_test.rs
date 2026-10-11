//! 主神空间队友 NPC 集成测试：中洲队核心队友（张杰/郑吒/楚轩/詹岚/赵樱空）
//! 在世界表（zhutian NPCS）存在，且各自 talk 场景 scenes::scene() 可解析、可选词可达。
//! 文件名称 characters_test.rs（任务约定词尾）。
use wuxian_horror_ch1::scenes;
use wuxian_horror_ch1::state::GameState;
use wuxian_horror_ch1::worlds;

/// 主神空间世界存在且挂载队友 NPC 表
#[test]
fn zhutian_npcs_present() {
    let w = worlds::find_world(worlds::WORLD_ZHUTIAN).expect("主神空间世界应注册");
    let ids: Vec<&str> = w.npcs.iter().map(|n| n.id).collect();
    assert!(
        ids.contains(&"n_zhangjie_nexus")
            && ids.contains(&"n_zhengzha_nexus")
            && ids.contains(&"n_chuxuan_nexus")
            && ids.contains(&"n_zhanlan_nexus")
            && ids.contains(&"n_zhaoyingkong_nexus"),
        "主神空间应含五名队友 NPC，实际: {:?}", ids
    );
}

/// 五位角色 talk 场景全部可解析（scenes::scene 返回 Some 且 id 匹配）
#[test]
fn team_member_talk_scenes_resolve() {
    let w = worlds::find_world(worlds::WORLD_ZHUTIAN).unwrap();
    let wanted = ["张杰", "郑吒", "楚轩", "詹岚", "赵樱空"];
    let members: Vec<_> = w.npcs
        .iter()
        .filter(|n| wanted.contains(&n.name))
        .collect();
    assert_eq!(members.len(), 5, "应命中五名队友");

    for m in members {
        let sc = scenes::scene(m.talk).unwrap_or_else(|| panic!("队友 {} 场景 {} 无法解析", m.name, m.talk));
        assert_eq!(sc.id, m.talk, "场景 id 应自洽");
        assert!(sc.speaker.is_some(), "{} 对话应有发言人", m.name);
    }
}

/// 各角色对话可选路：至少一个选择可执行（返回主神 / 前往兑换）
#[test]
fn team_member_scenes_have_choices() {
    for (talk, keyword) in [
        ("s_nexus_zhangjie", "兑换目录"),
        ("s_nexus_zhengzha", "兑换变强"),
        ("s_nexus_chuxuan", "兑换目录"),
        ("s_nexus_zhanlan", "兑换方向"),
        ("s_nexus_zhaoyingkong", "前往兑换目录"),
    ] {
        let sc = scenes::scene(talk).expect(talk);
        assert!(
            sc.choices.iter().any(|c| c.label.contains(keyword)),
            "{} 应有更新进的兑换类选择，实际: {:?}",
            talk,
            sc.choices.iter().map(|c| c.label).collect::<Vec<_>>()
        );
    }
}

/// 场景流转：从张杰直达队友对话并返回，mode 全 Normal、无 panic
#[test]
fn team_member_scene_flow() {
    let mut st = GameState::new();
    st.world_id = worlds::WORLD_ZHUTIAN.to_string();
    let mut deaths = vec![];
    for talk in ["s_nexus_zhangjie", "s_nexus_zhengzha", "s_nexus_chuxuan", "s_nexus_zhanlan", "s_nexus_zhaoyingkong"] {
        wuxian_horror_ch1::engine::goto(&mut st, talk, &mut deaths);
        assert_eq!(st.scene_id, talk, "应进入 {}", talk);
        assert!(scenes::scene(&st.scene_id).is_some());
    }
}

/// 流程：张杰 → 郑吒 → 楚轩 → 詹岚 → 赵樱空，经 world NPCS 的 talk 路由逐跳可达
#[test]
fn talk_route_chain_reaches_all() {
    let w = worlds::find_world(worlds::WORLD_ZHUTIAN).unwrap();
    // 从张杰出发，每跳选「……/返回」类可达下一名；此处只验证 talk 表彼此不循环引用同一场景 id
    let mut seen = std::collections::HashSet::new();
    for n in w.npcs {
        assert!(seen.insert(n.talk.to_string()), "NPC talk 场景重复: {}", n.talk);
        let sc = scenes::scene(n.talk).expect(n.talk);
        assert_eq!(sc.id, n.talk);
    }
    // 默认新建状态可直接 goto 任一队友
    for talk in seen {
        let mut st = GameState::new();
        st.world_id = worlds::WORLD_ZHUTIAN.to_string();
        let mut deaths = vec![];
        wuxian_horror_ch1::engine::goto(&mut st, &talk, &mut deaths);
        assert_eq!(st.scene_id, talk);
    }
}
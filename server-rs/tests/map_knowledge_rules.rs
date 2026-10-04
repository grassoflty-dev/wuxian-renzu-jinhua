use wuxian_horror_ch1::effects::{
    project_map_knowledge, CanonicalMap, CanonicalMapFeature, CanonicalMapRegion,
    MapExplorationState, MapPoint, MapPolygon, MapProjectionError,
};
use wuxian_horror_ch1::player_rules::{EffectivePlayerRules, KnowledgeChannel, KnowledgeLevel};

fn point(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn fixture() -> CanonicalMap {
    CanonicalMap {
        regions: vec![CanonicalMapRegion {
            id: "room-a".into(),
            channel: KnowledgeChannel::Topology,
            polygon: MapPolygon {
                vertices: vec![
                    point(0.0, 0.0),
                    point(10.0, 0.0),
                    point(10.0, 10.0),
                    point(0.0, 10.0),
                ],
            },
            terrain: None,
            boundary: None,
        }],
        features: vec![
            CanonicalMapFeature {
                id: "door-a".into(),
                channel: KnowledgeChannel::Connections,
                position: point(9.0, 5.0),
                region_id: Some("room-a".into()),
                terrain: None,
                hazard: None,
                obstacle: None,
                boundary: None,
            },
            CanonicalMapFeature {
                id: "objective-a".into(),
                channel: KnowledgeChannel::Objectives,
                position: point(3.0, 4.0),
                region_id: Some("room-a".into()),
                terrain: None,
                hazard: None,
                obstacle: None,
                boundary: None,
            },
            CanonicalMapFeature {
                id: "enemy-a".into(),
                channel: KnowledgeChannel::Enemies,
                position: point(4.0, 4.0),
                region_id: Some("room-a".into()),
                terrain: None,
                hazard: None,
                obstacle: None,
                boundary: None,
            },
            CanonicalMapFeature {
                id: "secret-a".into(),
                channel: KnowledgeChannel::Secrets,
                position: point(5.0, 4.0),
                region_id: Some("room-a".into()),
                terrain: None,
                hazard: None,
                obstacle: None,
                boundary: None,
            },
        ],
    }
}

#[test]
fn no_map_permission_projects_nothing() {
    let projection = project_map_knowledge(
        &fixture(),
        &MapExplorationState::default(),
        &EffectivePlayerRules::default(),
    )
    .unwrap();
    assert!(projection.regions.is_empty());
    assert!(projection.features.is_empty());
}

#[test]
fn full_topology_and_connections_do_not_leak_other_channels() {
    let mut rules = EffectivePlayerRules::default();
    rules.map.topology = KnowledgeLevel::Full;
    rules.map.connections = KnowledgeLevel::Full;
    let projection =
        project_map_knowledge(&fixture(), &MapExplorationState::default(), &rules).unwrap();
    assert_eq!(projection.regions.len(), 1);
    assert_eq!(projection.features.len(), 1);
    assert_eq!(projection.features[0].id.as_deref(), Some("door-a"));
    let serialized = serde_json::to_string(&projection).unwrap();
    for forbidden in ["enemy-a", "secret-a", "objective-a"] {
        assert!(!serialized.contains(forbidden));
    }
}

#[test]
fn local_map_only_projects_explored_geometry_and_authorized_objectives() {
    let mut rules = EffectivePlayerRules::default();
    rules.map.topology = KnowledgeLevel::ExploredOnly;
    rules.map.connections = KnowledgeLevel::ExploredOnly;
    rules.map.objectives = KnowledgeLevel::Known;
    let state = MapExplorationState {
        explored_region_ids: ["room-a".into()].into(),
        known_feature_ids: ["objective-a".into()].into(),
        ..MapExplorationState::default()
    };
    let projection = project_map_knowledge(&fixture(), &state, &rules).unwrap();
    assert_eq!(projection.regions.len(), 1);
    assert_eq!(projection.features.len(), 2);
    assert!(projection
        .features
        .iter()
        .any(|feature| feature.channel == KnowledgeChannel::Objectives));
    assert!(projection
        .features
        .iter()
        .any(|feature| feature.channel == KnowledgeChannel::Connections));
}

#[test]
fn detected_only_is_anonymous_and_distinct_from_known_or_full() {
    let map = fixture();
    let mut rules = EffectivePlayerRules::default();
    rules.map.enemies = KnowledgeLevel::DetectedOnly;
    let state = MapExplorationState {
        detected_feature_ids: ["enemy-a".into()].into(),
        ..MapExplorationState::default()
    };
    let detected = project_map_knowledge(&map, &state, &rules).unwrap();
    assert_eq!(detected.features.len(), 1);
    assert_eq!(detected.features[0].knowledge, KnowledgeLevel::DetectedOnly);
    assert_eq!(detected.features[0].id, None);
    assert!(!serde_json::to_string(&detected)
        .unwrap()
        .contains("enemy-a"));

    rules.map.enemies = KnowledgeLevel::Known;
    assert!(project_map_knowledge(&map, &state, &rules)
        .unwrap()
        .features
        .is_empty());
    let known_state = MapExplorationState {
        known_feature_ids: ["enemy-a".into()].into(),
        ..state
    };
    assert_eq!(
        project_map_knowledge(&map, &known_state, &rules)
            .unwrap()
            .features[0]
            .id
            .as_deref(),
        Some("enemy-a")
    );
    rules.map.enemies = KnowledgeLevel::Full;
    assert_eq!(
        project_map_knowledge(&map, &MapExplorationState::default(), &rules)
            .unwrap()
            .features[0]
            .knowledge,
        KnowledgeLevel::Full
    );
}

#[test]
fn bounded_reveal_radius_detects_nearby_entities_without_revealing_identity() {
    let mut rules = EffectivePlayerRules::default();
    rules.map.enemies = KnowledgeLevel::DetectedOnly;
    rules.map.reveal_radius_mm = 2_000;
    let state = MapExplorationState {
        player_position: Some(point(4.0, 3.0)),
        ..MapExplorationState::default()
    };
    let projection = project_map_knowledge(&fixture(), &state, &rules).unwrap();
    assert_eq!(projection.features.len(), 1);
    assert_eq!(projection.features[0].channel, KnowledgeChannel::Enemies);
    assert_eq!(projection.features[0].id, None);

    rules.map.reveal_radius_mm = 2_001;
    assert_eq!(
        project_map_knowledge(&fixture(), &state, &rules)
            .unwrap()
            .features
            .len(),
        1
    );
    rules.map.reveal_radius_mm = 999;
    assert!(project_map_knowledge(&fixture(), &state, &rules)
        .unwrap()
        .features
        .is_empty());
    rules.map.reveal_radius_mm = 100_001;
    assert_eq!(
        project_map_knowledge(&fixture(), &state, &rules),
        Err(MapProjectionError::InvalidRules)
    );
}

#[test]
fn malformed_geometry_duplicate_ids_and_unknown_tags_fail_closed() {
    let mut bad = fixture();
    bad.regions[0].polygon.vertices[2] = point(0.0, 10.0);
    assert_eq!(
        project_map_knowledge(
            &bad,
            &MapExplorationState::default(),
            &EffectivePlayerRules::default()
        ),
        Err(MapProjectionError::InvalidPolygon)
    );

    let mut duplicate = fixture();
    duplicate.features[1].id = duplicate.features[0].id.clone();
    assert_eq!(
        project_map_knowledge(
            &duplicate,
            &MapExplorationState::default(),
            &EffectivePlayerRules::default()
        ),
        Err(MapProjectionError::DuplicateId)
    );

    assert!(serde_json::from_str::<wuxian_horror_ch1::effects::TerrainTag>("\"lava\"").is_err());
    assert!(serde_json::from_str::<KnowledgeLevel>("\"unlimited\"").is_err());

    let mut misplaced_hazard = fixture();
    misplaced_hazard.features[2].hazard = Some(wuxian_horror_ch1::effects::HazardTag::Heat);
    assert_eq!(
        project_map_knowledge(
            &misplaced_hazard,
            &MapExplorationState::default(),
            &EffectivePlayerRules::default()
        ),
        Err(MapProjectionError::InvalidChannelData)
    );
}

#[test]
fn source_order_does_not_change_effective_map_rules() {
    use wuxian_horror_ch1::effects::{
        EffectLifetime, EffectResolver, EffectSource, EffectSourceKind, EffectSpec,
        MapKnowledgeEffect, MapKnowledgeTag, MapRevealRadiusEffect, StackRule,
    };
    let source = |id: &str, instance: &str, effects| EffectSource {
        source_kind: EffectSourceKind::Item,
        source_id: id.into(),
        instance_id: instance.into(),
        lifetime: EffectLifetime::Equipped,
        ui_category: None,
        effects,
    };
    let a = source(
        "map_a",
        "map-a",
        vec![EffectSpec::MapKnowledge(MapKnowledgeEffect {
            tag: MapKnowledgeTag::Enemies,
            stack: StackRule::Any,
        })],
    );
    let b = source(
        "scan_b",
        "scan-b",
        vec![EffectSpec::MapRevealRadius(MapRevealRadiusEffect {
            radius_mm: 5_000,
            stack: StackRule::Max,
        })],
    );
    assert_eq!(
        EffectResolver::resolve(&[a.clone(), b.clone()]).unwrap(),
        EffectResolver::resolve(&[b, a]).unwrap()
    );
}

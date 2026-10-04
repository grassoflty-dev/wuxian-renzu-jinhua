//! Validated, immutable registry for compiler-produced SceneDefinition content.

use crate::scene_runtime::{SceneDefinition, SceneRuntimeError};
use std::collections::{BTreeMap, BTreeSet};

pub const WORLD_IDS: [&str; 3] = ["grey_hive", "mist_harbor", "clockworks"];
pub const RETURN_STATION_ID: &str = "return_station";
pub const GREY_HIVE_SLICE_SCENE_IDS: [&str; 11] = [
    "gh_beacon",
    "gh_entry_maintenance",
    "gh_central_shaft",
    "gh_deep_decon",
    "gh_exit",
    "gh_gate_a",
    "gh_gate_b",
    "gh_power_room",
    "gh_lockdown",
    "gh_bio_isolation",
    "gh_sentinel_arena",
];
pub const MIST_HARBOR_SCENE_IDS: [&str; 9] = [
    "mh_fog_pier",
    "mh_tidal_warehouse",
    "mh_signal_yard",
    "mh_drowned_quay",
    "mh_breakwater",
    "mh_pump_station",
    "mh_resonance_tower",
    "mh_warden_arena",
    "mh_extraction",
];
pub const CLOCKWORKS_STAGED_SCENE_IDS: [&str; 9] = [
    "cw_entry_foundry",
    "cw_pressure_hall",
    "cw_conveyor_bridge",
    "cw_boiler_chamber",
    "cw_gear_shaft",
    "cw_furnace_heart",
    "cw_forged_guard_arena",
    "cw_regulator_core",
    "cw_shutdown_exit",
];
const PROGRESSION_JSON: &str = include_str!("../data/world_progression_v1.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegistryMode {
    FullCampaign,
    GreyHiveSlice,
    GreyHiveWithReturnStation,
    GreyHiveMistHarborWithReturnStation,
    StagedClockworksProduction,
    CompleteClockworksProduction,
}

#[derive(Clone, Debug)]
pub struct WorldRegistry {
    scenes: BTreeMap<String, SceneDefinition>,
    required_events: BTreeMap<String, BTreeSet<String>>,
    event_owner: BTreeMap<String, String>,
    mode: RegistryMode,
}

impl WorldRegistry {
    pub fn load<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::load_mode(
            scene_json,
            admitted_assets,
            known_entity_types,
            RegistryMode::FullCampaign,
        )
    }

    /// Load the explicitly authored Grey Hive production slice without claiming that the
    /// three-world campaign is complete.
    pub fn load_grey_hive_slice<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::load_mode(
            scene_json,
            admitted_assets,
            known_entity_types,
            RegistryMode::GreyHiveSlice,
        )
    }

    /// Load the locked Grey Hive production route plus native Return Station
    /// scene. Return Station is registered for state/save compatibility, but
    /// this mode does not create a route into it.
    pub fn load_grey_hive_with_return_station<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::load_mode(
            scene_json,
            admitted_assets,
            known_entity_types,
            RegistryMode::GreyHiveWithReturnStation,
        )
    }

    /// Register the locked Grey Hive, Mist Harbor, and Return Station scenes.
    /// This validates Mist Harbor content but does not make a world gate available.
    pub fn load_grey_hive_mist_harbor_with_return_station<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::load_mode(
            scene_json,
            admitted_assets,
            known_entity_types,
            RegistryMode::GreyHiveMistHarborWithReturnStation,
        )
    }

    /// Register the immutable 30-scene production bundle while Clockworks is
    /// still staged. This mode requires Clockworks valves and shutdown event
    /// sources, but permits its authored core event to remain absent.
    pub fn load_staged_clockworks_production<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::load_mode(
            scene_json,
            admitted_assets,
            known_entity_types,
            RegistryMode::StagedClockworksProduction,
        )
    }

    /// Validate the complete native campaign without relaxing the immutable
    /// scene set or accepting substitute progression emitters. Installing this
    /// registry is a separate startup decision from validating its content.
    pub fn load_complete_clockworks_production<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::load_mode(
            scene_json,
            admitted_assets,
            known_entity_types,
            RegistryMode::CompleteClockworksProduction,
        )
    }

    fn load_mode<I, S>(
        scene_json: I,
        admitted_assets: &BTreeSet<String>,
        known_entity_types: &BTreeSet<String>,
        mode: RegistryMode,
    ) -> Result<Self, SceneRuntimeError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut scenes = BTreeMap::new();
        let progression: ProgressionCatalog = serde_json::from_str(PROGRESSION_JSON)
            .map_err(|_| SceneRuntimeError::InvalidProgressionCatalog)?;
        if progression.schema_version != 1 || progression.worlds.len() != WORLD_IDS.len() {
            return Err(SceneRuntimeError::InvalidProgressionCatalog);
        }
        let mut required_events = BTreeMap::new();
        let mut event_owner = BTreeMap::new();
        for (index, world) in progression.worlds.into_iter().enumerate() {
            if world.world_id != WORLD_IDS[index] || world.required_events.is_empty() {
                return Err(SceneRuntimeError::InvalidProgressionCatalog);
            }
            let mut required = BTreeSet::new();
            for event in world.required_events {
                if !valid_id(&event)
                    || !required.insert(event.clone())
                    || event_owner.insert(event, world.world_id.clone()).is_some()
                {
                    return Err(SceneRuntimeError::InvalidProgressionCatalog);
                }
            }
            required_events.insert(world.world_id, required);
        }
        for raw in scene_json {
            let mut scene: SceneDefinition = serde_json::from_str(raw.as_ref())
                .map_err(|_| SceneRuntimeError::MalformedSceneDefinition)?;
            use sha2::Digest;
            scene.verified_source_sha256=format!("{:x}",sha2::Sha256::digest(raw.as_ref().as_bytes()));
            scene.validate(admitted_assets, known_entity_types, &required_events)?;
            if scenes.insert(scene.scene_id.clone(), scene).is_some() {
                return Err(SceneRuntimeError::DuplicateSceneId);
            }
        }
        if scenes.is_empty() {
            return Err(SceneRuntimeError::EmptyRegistry);
        }
        if mode == RegistryMode::GreyHiveSlice
            && (scenes.len() != GREY_HIVE_SLICE_SCENE_IDS.len()
                || GREY_HIVE_SLICE_SCENE_IDS
                    .iter()
                    .any(|scene_id| !scenes.contains_key(*scene_id))
                || scenes.values().any(|scene| scene.world_id != "grey_hive"))
        {
            return Err(SceneRuntimeError::InvalidGreyHiveSlice);
        }
        if matches!(
            mode,
            RegistryMode::GreyHiveWithReturnStation
                | RegistryMode::GreyHiveMistHarborWithReturnStation
                | RegistryMode::StagedClockworksProduction
                | RegistryMode::CompleteClockworksProduction
        ) && (scenes.len()
            != GREY_HIVE_SLICE_SCENE_IDS.len()
                + 1
                + match mode {
                    RegistryMode::GreyHiveMistHarborWithReturnStation => {
                        MIST_HARBOR_SCENE_IDS.len()
                    }
                    RegistryMode::StagedClockworksProduction
                    | RegistryMode::CompleteClockworksProduction => {
                        MIST_HARBOR_SCENE_IDS.len() + CLOCKWORKS_STAGED_SCENE_IDS.len()
                    }
                    _ => 0,
                }
            || !scene_ids_belong_to_world(&scenes, &GREY_HIVE_SLICE_SCENE_IDS, "grey_hive")
            || scenes
                .values()
                .filter(|scene| scene.world_id == "grey_hive")
                .count()
                != GREY_HIVE_SLICE_SCENE_IDS.len()
            || (mode == RegistryMode::GreyHiveMistHarborWithReturnStation
                && (!scene_ids_belong_to_world(&scenes, &MIST_HARBOR_SCENE_IDS, "mist_harbor")
                    || scenes
                        .values()
                        .filter(|scene| scene.world_id == "mist_harbor")
                        .count()
                        != MIST_HARBOR_SCENE_IDS.len()))
            || (matches!(
                mode,
                RegistryMode::StagedClockworksProduction | RegistryMode::CompleteClockworksProduction
            )
                && (!scene_ids_belong_to_world(&scenes, &MIST_HARBOR_SCENE_IDS, "mist_harbor")
                    || scenes
                        .values()
                        .filter(|scene| scene.world_id == "mist_harbor")
                        .count()
                        != MIST_HARBOR_SCENE_IDS.len()
                    || !scene_ids_belong_to_world(
                        &scenes,
                        &CLOCKWORKS_STAGED_SCENE_IDS,
                        "clockworks",
                    )
                    || scenes
                        .values()
                        .filter(|scene| scene.world_id == "clockworks")
                        .count()
                        != CLOCKWORKS_STAGED_SCENE_IDS.len()))
            || scenes.get("rs_core_room").is_none_or(|scene| {
                scene.world_id != RETURN_STATION_ID
                    || !scene.transition_targets().is_empty()
                    || scene
                        .interactions
                        .iter()
                        .any(|interaction| interaction.event.is_some())
            }))
        {
            return Err(SceneRuntimeError::InvalidNativeSceneSet);
        }
        if mode == RegistryMode::CompleteClockworksProduction {
            validate_clockworks_event_sources(&scenes)?;
        }
        let mut world_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut emitted: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for scene in scenes.values() {
            *world_counts.entry(scene.world_id.clone()).or_default() += 1;
            let ids = emitted.entry(scene.world_id.clone()).or_default();
            ids.extend(scene.emitted_event_ids());
        }
        if mode == RegistryMode::FullCampaign
            && world_counts.get(RETURN_STATION_ID).copied().unwrap_or(0) == 0
        {
            return Err(SceneRuntimeError::MissingReturnStation);
        }
        let worlds: &[&str] = match mode {
            RegistryMode::FullCampaign => &WORLD_IDS,
            RegistryMode::GreyHiveSlice | RegistryMode::GreyHiveWithReturnStation => &["grey_hive"],
            RegistryMode::GreyHiveMistHarborWithReturnStation => &["grey_hive", "mist_harbor"],
            RegistryMode::StagedClockworksProduction | RegistryMode::CompleteClockworksProduction => {
                &WORLD_IDS
            }
        };
        for world in worlds {
            if world_counts.get(*world).copied().unwrap_or(0) == 0 {
                return Err(SceneRuntimeError::MissingWorld((*world).into()));
            }
            let required = required_events.get(*world).expect("validated catalog");
            let staged_clockworks_required = BTreeSet::from([
                "clockworks_valves".to_owned(),
                "clockworks_shutdown".to_owned(),
            ]);
            let required_for_mode =
                if mode == RegistryMode::StagedClockworksProduction && *world == "clockworks" {
                    &staged_clockworks_required
                } else {
                    required
                };
            let none = BTreeSet::new();
            if !required_for_mode.is_subset(emitted.get(*world).unwrap_or(&none)) {
                return Err(SceneRuntimeError::MissingRequiredEvent((*world).into()));
            }
        }
        for scene in scenes.values() {
            for target in scene.transition_targets() {
                let Some(target_scene) = scenes.get(&target.scene_id) else {
                    return Err(SceneRuntimeError::UnknownTransitionTarget(target.scene_id));
                };
                if !target_scene
                    .spawns
                    .iter()
                    .any(|spawn| spawn.id == target.spawn_id && spawn.kind == "player")
                {
                    return Err(SceneRuntimeError::UnknownTransitionSpawn(target.spawn_id));
                }
                if matches!(
                    mode,
                    RegistryMode::GreyHiveWithReturnStation
                        | RegistryMode::GreyHiveMistHarborWithReturnStation
                        | RegistryMode::StagedClockworksProduction
                        | RegistryMode::CompleteClockworksProduction
                ) && target_scene.world_id != scene.world_id
                {
                    return Err(SceneRuntimeError::InvalidNativeSceneSet);
                }
            }
        }
        Ok(Self {
            scenes,
            required_events,
            event_owner,
            mode,
        })
    }

    pub fn scene(&self, scene_id: &str) -> Option<&SceneDefinition> {
        self.scenes.get(scene_id)
    }

    pub fn required_events(&self, world_id: &str) -> Option<&BTreeSet<String>> {
        self.required_events.get(world_id)
    }

    pub fn event_owner(&self, event_id: &str) -> Option<&str> {
        self.event_owner.get(event_id).map(String::as_str)
    }

    pub fn is_staged_clockworks_production(&self) -> bool {
        self.mode == RegistryMode::StagedClockworksProduction
    }

    pub fn is_complete_clockworks_production(&self) -> bool {
        self.mode == RegistryMode::CompleteClockworksProduction
    }

    pub fn is_required_event(&self, world_id: &str, event_id: &str) -> bool {
        self.required_events
            .get(world_id)
            .is_some_and(|events| events.contains(event_id))
    }

    pub fn scenes(&self) -> impl Iterator<Item = &SceneDefinition> {
        self.scenes.values()
    }
}

/// Progression sources are identities, not merely a set of emitted event names.
/// In particular, a trigger or a second terminal cannot substitute for the
/// pressure aggregate, the post-Regulator console, or the shutdown terminal.
fn validate_clockworks_event_sources(
    scenes: &BTreeMap<String, SceneDefinition>,
) -> Result<(), SceneRuntimeError> {
    const VALVES: [&str; 3] = [
        "cw_pressure_valve_01_staged",
        "cw_pressure_valve_02_staged",
        "cw_pressure_valve_03_staged",
    ];
    let expected = BTreeSet::from([
        ("cw_pressure_hall", "clockworks_valves", "cw_clockworks_valves_staged"),
        ("cw_regulator_core", "clockworks_core", "cw_regulator_core_console_staged"),
        ("cw_shutdown_exit", "clockworks_shutdown", "cw_master_shutdown_staged"),
    ]);
    let mut actual = BTreeSet::new();
    for scene in scenes.values() {
        for interaction in &scene.interactions {
            let Some(event) = interaction.event.as_deref() else { continue; };
            if !event.starts_with("clockworks_") { continue; }
            if scene.world_id != "clockworks"
                || !matches!(event, "clockworks_core" | "clockworks_shutdown")
                || interaction.kind != "terminal"
                || !actual.insert((scene.scene_id.as_str(), event, interaction.id.as_str()))
            {
                return Err(SceneRuntimeError::InvalidNativeSceneSet);
            }
        }
        for aggregate in &scene.interaction_aggregates {
            if !aggregate.event.starts_with("clockworks_") { continue; }
            let members: BTreeSet<_> = aggregate.member_ids.iter().map(String::as_str).collect();
            if scene.world_id != "clockworks"
                || aggregate.event != "clockworks_valves"
                || aggregate.member_ids.len() != VALVES.len()
                || members != BTreeSet::from(VALVES)
                || !scene.interactions.iter().any(|item| {
                    item.id == aggregate.marker_id
                        && item.kind == "three_valve_sequence_staged_marker"
                        && item.event.is_none()
                })
                || VALVES.iter().any(|id| !scene.interactions.iter().any(|item| {
                    item.id == *id && item.kind == "valve_control" && item.event.is_none()
                }))
                || !actual.insert((scene.scene_id.as_str(), aggregate.event.as_str(), aggregate.marker_id.as_str()))
            {
                return Err(SceneRuntimeError::InvalidNativeSceneSet);
            }
        }
        if scene.triggers.iter().any(|trigger| trigger.event.starts_with("clockworks_")) {
            return Err(SceneRuntimeError::InvalidNativeSceneSet);
        }
    }
    if actual != expected {
        return Err(SceneRuntimeError::InvalidNativeSceneSet);
    }
    Ok(())
}

fn scene_ids_belong_to_world(
    scenes: &BTreeMap<String, SceneDefinition>,
    scene_ids: &[&str],
    world_id: &str,
) -> bool {
    scene_ids.iter().all(|scene_id| {
        scenes
            .get(*scene_id)
            .is_some_and(|scene| scene.world_id == world_id)
    })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressionCatalog {
    schema_version: u32,
    worlds: Vec<ProgressionWorld>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressionWorld {
    world_id: String,
    required_events: Vec<String>,
}

pub(crate) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'))
}

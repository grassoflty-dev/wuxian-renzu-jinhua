//! Durable world state shared by scene visits and Save V6.

use crate::effects::TerrainTag;
use serde::{Deserialize, Serialize};

pub const CLOCKWORKS_PRESSURE_VALVE_IDS: [&str; 3] = [
    "cw_pressure_valve_01_staged",
    "cw_pressure_valve_02_staged",
    "cw_pressure_valve_03_staged",
];

pub const MIST_HARBOR_ID: &str = "mist_harbor";
pub const PUMP_CONTROL_ID: &str = "mh_pump_control_primary";
pub const PUMP_CONTROL_POSITION_M: [f32; 3] = [17.0, 0.0, 7.0];
pub const PUMP_EAST_WATER_ID: &str = "mh_pump_east_channel_water";
pub const DROWNED_QUAY_WATER_ID: &str = "mh_water_depth_region";
pub const MIST_BEACON_EAST_EVENT_ID: &str = "mist_beacon_east";
pub const PUMP_DRAIN_DURATION_MS: u64 = 6_000;
pub const MIST_HARBOR_EXPLORATION_REGION_IDS: [&str; 27] = [
    "mh_fp_entry_berth",
    "mh_fp_foghorn_pier",
    "mh_fp_warehouse_approach",
    "mh_tw_west_loading",
    "mh_tw_storage_floor",
    "mh_tw_beacon_bay",
    "mh_sy_west_yard",
    "mh_sy_interference_field",
    "mh_sy_quay_approach",
    "mh_dq_west_quay",
    "mh_dq_flooded_channel",
    "mh_dq_east_quay",
    "mh_bw_west_seawall",
    "mh_bw_center_seawall",
    "mh_bw_east_beacon",
    "mh_ps_intake",
    "mh_ps_control_hall",
    "mh_ps_east_channel",
    "mh_rt_entry",
    "mh_rt_resonance_floor",
    "mh_rt_signal_section",
    "mh_wa_entry",
    "mh_wa_arena_core",
    "mh_wa_exit",
    "mh_ex_arrival",
    "mh_ex_extraction_pad",
    "mh_ex_exit_section",
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PumpStatus {
    #[default]
    Ready,
    Draining,
    Drained,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PumpPersistentState {
    pub state: PumpStatus,
    pub drain_complete_at_world_time_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MistHarborPersistentState {
    /// Exact existing Wraith controller version in the four authored MH scenes.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub signal_wraith_controller_version: u32,
    /// Quay, Breakwater and Tower Tidebound roster version only.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub tidebound_actor_roster_version: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub warden_defeated: bool,
    #[serde(default)]
    pub pump: PumpPersistentState,
    #[serde(default)]
    pub explored_region_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GreyHivePersistentState {
    /// GH-BEACON-1: absent historical state never implies ownership.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_beacon")]
    pub beacon: Option<GreyHiveBeaconState>,
    /// Version of the expanded Gate B Security2 + Brute1 roster only.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub gate_b_actor_roster_version: u32,
    /// Version of the expanded Lockdown/Deep Decon Swarm rosters only.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub swarm_actor_roster_version: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sentinel_first_kill: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GreyHiveBeaconStage { Uncollected, Carried, Mounted }
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GreyHiveBeaconAction { Collect, Mount }
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GreyHiveBeaconReceipt {
    pub action: GreyHiveBeaconAction,
    pub request_id: String,
    pub world_epoch: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GreyHiveBeaconState {
    pub schema_version: u32,
    pub state: GreyHiveBeaconStage,
    pub receipts: Vec<GreyHiveBeaconReceipt>,
}
impl GreyHiveBeaconState {
    pub fn new_uncollected() -> Self { Self { schema_version: 1, state: GreyHiveBeaconStage::Uncollected, receipts: vec![] } }
    pub fn validate(&self) -> bool {
        let expected = match self.state { GreyHiveBeaconStage::Uncollected => 0, GreyHiveBeaconStage::Carried => 1, GreyHiveBeaconStage::Mounted => 2 };
        self.schema_version == 1 && self.receipts.len() == expected
            && self.receipts.iter().enumerate().all(|(i,r)| {
                r.action == if i == 0 { GreyHiveBeaconAction::Collect } else { GreyHiveBeaconAction::Mount }
                    && crate::scene_registry::valid_id(&r.request_id)
                    && r.world_epoch > 0 && r.world_epoch <= 9_007_199_254_740_991
            })
            && self.receipts.windows(2).all(|pair| pair[0].request_id != pair[1].request_id && pair[0].world_epoch <= pair[1].world_epoch)
    }
    pub fn owned(&self) -> bool { self.validate() && self.state != GreyHiveBeaconStage::Uncollected }
}
fn deserialize_beacon<'de,D:serde::Deserializer<'de>>(deserializer:D)->Result<Option<GreyHiveBeaconState>,D::Error>{
    // Explicit null is malformed current state, whereas omission stays legacy.
    GreyHiveBeaconState::deserialize(deserializer).map(Some)
}

fn is_false(value: &bool) -> bool {
    !value
}

impl GreyHivePersistentState {
    pub fn mark_sentinel_first_kill(&mut self) -> bool {
        if self.sentinel_first_kill {
            return false;
        }
        self.sentinel_first_kill = true;
        true
    }

    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClockworksPersistentState {
    /// Canonical Gear Shaft support layout; zero is the historical floor scene.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub gear_shaft_support_version: u32,
    /// Exact authored CW roster version; zero denotes historical content.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub actor_roster_version: u32,
    /// Canonically sorted source receipts, never inferred from route progress.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pressure_valve_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub core_console_confirmed: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub forged_guard_elite_first_kill: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub regulator_phase2_active: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub regulator_phase3_active: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub regulator_defeated: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regulator_phase3_started_at_ms: u64,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub regulator_hazard_next_damage_at_ms: std::collections::BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regulator_heat_warning_until_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regulator_heat_next_damage_at_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regulator_heat_cooled_until_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regulator_valve_cooldown_until_ms: u64,
}

fn is_zero_u32(value: &u32) -> bool { *value == 0 }

fn is_zero(value: &u64) -> bool {
    *value == 0
}

impl ClockworksPersistentState {
    pub fn mark_forged_guard_elite_first_kill(&mut self) -> bool {
        if self.forged_guard_elite_first_kill {
            return false;
        }
        self.forged_guard_elite_first_kill = true;
        true
    }

    pub fn reset_regulator_encounter(&mut self) {
        self.regulator_phase2_active = false;
        self.regulator_phase3_active = false;
        self.regulator_phase3_started_at_ms = 0;
        self.regulator_hazard_next_damage_at_ms.clear();
        self.regulator_heat_warning_until_ms = 0;
        self.regulator_heat_next_damage_at_ms = 0;
        self.regulator_heat_cooled_until_ms = 0;
        self.regulator_valve_cooldown_until_ms = 0;
    }

    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldPersistentState {
    #[serde(default, skip_serializing_if = "crate::environment_hazards::EnvironmentPersistentState::is_default")]
    pub environment: crate::environment_hazards::EnvironmentPersistentState,
    #[serde(default, skip_serializing_if = "GreyHivePersistentState::is_default")]
    pub grey_hive: GreyHivePersistentState,
    #[serde(default, skip_serializing_if = "ClockworksPersistentState::is_default")]
    pub clockworks: ClockworksPersistentState,
    #[serde(default)]
    pub mist_harbor: MistHarborPersistentState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PumpStartResult {
    Started,
    AlreadyDraining,
    AlreadyDrained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PumpStateError {
    WrongWorld,
    EastBeaconIncomplete,
    InvalidState,
    TimeOverflow,
}

impl PumpPersistentState {
    pub fn start(
        &mut self,
        world_id: &str,
        east_beacon_complete: bool,
        world_time_ms: u64,
    ) -> Result<PumpStartResult, PumpStateError> {
        if world_id != MIST_HARBOR_ID {
            return Err(PumpStateError::WrongWorld);
        }
        match self.state {
            PumpStatus::Drained => return Ok(PumpStartResult::AlreadyDrained),
            PumpStatus::Draining => return Ok(PumpStartResult::AlreadyDraining),
            PumpStatus::Ready => {}
        }
        if !east_beacon_complete {
            return Err(PumpStateError::EastBeaconIncomplete);
        }
        let deadline = world_time_ms
            .checked_add(PUMP_DRAIN_DURATION_MS)
            .ok_or(PumpStateError::TimeOverflow)?;
        self.state = PumpStatus::Draining;
        self.drain_complete_at_world_time_ms = Some(deadline);
        Ok(PumpStartResult::Started)
    }

    /// Resolves an expired drain at the authoritative saved/server world time.
    pub fn resolve_at(&mut self, world_time_ms: u64) -> Result<bool, PumpStateError> {
        match (self.state, self.drain_complete_at_world_time_ms) {
            (PumpStatus::Ready, None) | (PumpStatus::Drained, Some(_)) => Ok(false),
            (PumpStatus::Draining, Some(deadline)) if world_time_ms >= deadline => {
                self.state = PumpStatus::Drained;
                Ok(true)
            }
            (PumpStatus::Draining, Some(_)) => Ok(false),
            _ => Err(PumpStateError::InvalidState),
        }
    }

    pub fn validate(&self) -> Result<(), PumpStateError> {
        match (self.state, self.drain_complete_at_world_time_ms) {
            (PumpStatus::Ready, None) | (PumpStatus::Draining | PumpStatus::Drained, Some(_)) => {
                Ok(())
            }
            _ => Err(PumpStateError::InvalidState),
        }
    }
}

impl WorldPersistentState {
    pub fn reset_new_journey(&mut self) {
        *self = Self::default();
    }

    pub fn resolve_terrain_tag(
        &self,
        world_id: &str,
        region_id: &str,
        authored_tag: TerrainTag,
    ) -> TerrainTag {
        if world_id == MIST_HARBOR_ID
            && self.mist_harbor.pump.state == PumpStatus::Drained
            && ((region_id == PUMP_EAST_WATER_ID && authored_tag == TerrainTag::WaterDeep)
                || (region_id == DROWNED_QUAY_WATER_ID && authored_tag == TerrainTag::WaterShallow))
        {
            TerrainTag::WetFloor
        } else {
            authored_tag
        }
    }

    pub fn validate(&self) -> Result<(), PumpStateError> {
        self.mist_harbor.pump.validate()?;
        if self.grey_hive.beacon.as_ref().is_some_and(|state| !state.validate()) { return Err(PumpStateError::InvalidState); }
        let mut seen = std::collections::BTreeSet::new();
        if self.mist_harbor.explored_region_ids.iter().any(|id| {
            !MIST_HARBOR_EXPLORATION_REGION_IDS.contains(&id.as_str()) || !seen.insert(id)
        }) {
            return Err(PumpStateError::InvalidState);
        }
        Ok(())
    }

    pub fn resolve_at(&mut self, world_time_ms: u64) -> Result<bool, PumpStateError> {
        self.mist_harbor.pump.resolve_at(world_time_ms)
    }

    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn mark_explored(&mut self, region_id: &str) -> Result<bool, PumpStateError> {
        if !MIST_HARBOR_EXPLORATION_REGION_IDS.contains(&region_id) {
            return Err(PumpStateError::InvalidState);
        }
        let ids = &mut self.mist_harbor.explored_region_ids;
        if ids.iter().any(|id| id == region_id) {
            return Ok(false);
        }
        ids.push(region_id.to_owned());
        ids.sort();
        Ok(true)
    }

    pub fn is_explored(&self, region_id: &str) -> bool {
        self.mist_harbor
            .explored_region_ids
            .iter()
            .any(|id| id == region_id)
    }
}

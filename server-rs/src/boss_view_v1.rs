//! Pure optional Boss wire DTOs. No runtime ownership or external asset admission.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BossEncounterView {
    pub entity_id: String,
    pub entity_type: String,
    pub display_name: String,
    pub current_hp: u32,
    pub max_hp: u32,
    pub phase: u8,
    pub state: String,
    pub temporary_visual: bool,
    pub public_release_eligible: bool,
    /// Resolved current permission, restricted to an ongoing true windup.
    pub mapped_true_source: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<BossWarningView>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BossWarningView {
    pub attack_serial: u64,
    pub kind: String,
    pub origin_m: [f32; 3],
    pub direction_rad: f32,
    pub radius_m: f32,
    pub half_angle_rad: f32,
    pub remaining_ms: u64,
    pub ordinary_visible: bool,
}

//! Pure inventory/equipment wire DTOs shared by runtime and path-included protocol tests.
//! No runtime owner, command, grant, or persistence authority lives in this module.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildProjection {
    pub schema_version: u32,
    pub revision: u64,
    pub items: Vec<InventoryItemProjection>,
    pub equipment: Vec<EquipmentProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryItemProjection {
    pub item_id: String,
    pub quantity: u32,
    pub slot_id: String,
    pub label: String,
    pub description: String,
    pub equipped: bool,
    pub release_eligible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquipmentProjection {
    pub slot_id: String,
    pub item_id: String,
    pub label: String,
}

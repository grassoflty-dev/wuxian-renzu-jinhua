//! Internal-runtime inventory projection and ID-only, epoch-bound equipment commands.
//! Public redistribution admission is metadata, never a replacement for runtime ownership.
use super::{build_v6, project_scene_view, FormalRuntime, RuntimeState};
pub use crate::build_projection::{BuildProjection, EquipmentProjection, InventoryItemProjection};
use crate::world_v3::CommandReceipt;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::OnceLock,
};

pub(super) const MAX_SAFE_REVISION: u64 = 9_007_199_254_740_991;
const RECEIPT_LIMIT: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildCommandRequest {
    pub request_id: String,
    pub world_epoch: u64,
    pub expected_build_revision: u64,
    pub action: BuildAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum BuildAction {
    Equip {
        item_id: String,
        #[serde(deserialize_with = "required_expected_item")]
        expected_item_id: Option<String>,
    },
    Unequip {
        slot_id: String,
        expected_item_id: String,
    },
}

fn required_expected_item<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

#[derive(Clone, Default)]
pub(crate) struct BuildCommandState {
    pub revision: u64,
    receipts: VecDeque<BuildCommandRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PresentationCatalog {
    schema_version: u32,
    items: BTreeMap<String, PresentationItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PresentationItem {
    label: String,
    description: String,
    release_eligible: bool,
}

struct AdmittedItem {
    slot: String,
    presentation: PresentationItem,
}

fn catalog() -> Result<&'static BTreeMap<String, AdmittedItem>, &'static str> {
    static CATALOG: OnceLock<Result<BTreeMap<String, AdmittedItem>, &'static str>> =
        OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let presentation: PresentationCatalog =
                serde_json::from_str(include_str!("../../data/formal_build_presentation_v1.json"))
                    .map_err(|_| "E_BUILD_PRESENTATION_INVALID")?;
            let slots = build_v6::trusted_item_slots().map_err(|_| "E_BUILD_CONTENT_INVALID")?;
            if presentation.schema_version != 1 || presentation.items.len() != slots.len() {
                return Err("E_BUILD_PRESENTATION_INVALID");
            }
            let mut items = BTreeMap::new();
            for (id, presentation) in presentation.items {
                let slot = slots
                    .get(&id)
                    .ok_or("E_BUILD_PRESENTATION_INVALID")?
                    .clone();
                if presentation.label.trim().is_empty()
                    || presentation.label.len() > 192
                    || presentation.description.trim().is_empty()
                    || presentation.description.len() > 1024
                    || presentation
                        .label
                        .chars()
                        .chain(presentation.description.chars())
                        .any(char::is_control)
                {
                    return Err("E_BUILD_PRESENTATION_INVALID");
                }
                items.insert(id, AdmittedItem { slot, presentation });
            }
            Ok(items)
        })
        .as_ref()
        .map_err(|error| *error)
}

pub(super) fn project_build(state: &RuntimeState) -> Option<BuildProjection> {
    let catalog = catalog().ok()?;
    let items = state
        .progression_v6
        .inventory
        .iter()
        .filter_map(|(id, quantity)| {
            let definition = catalog.get(id)?;
            if *quantity == 0 {
                return None;
            }
            Some(InventoryItemProjection {
                item_id: id.clone(),
                quantity: *quantity,
                slot_id: definition.slot.clone(),
                label: definition.presentation.label.clone(),
                description: definition.presentation.description.clone(),
                equipped: state.progression_v6.equipment.get(&definition.slot) == Some(id),
                release_eligible: definition.presentation.release_eligible,
            })
        })
        .collect();
    let equipment = state
        .progression_v6
        .equipment
        .iter()
        .filter_map(|(slot, id)| {
            let definition = catalog.get(id)?;
            if definition.slot != *slot || !state.progression_v6.inventory.contains_key(id) {
                return None;
            }
            Some(EquipmentProjection {
                slot_id: slot.clone(),
                item_id: id.clone(),
                label: definition.presentation.label.clone(),
            })
        })
        .collect();
    Some(BuildProjection {
        schema_version: 1,
        revision: state.build_commands.revision,
        items,
        equipment,
    })
}

fn valid_id(id: &str, max: usize) -> bool {
    !id.is_empty()
        && id.len() <= max
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._:-".contains(&b))
}

impl BuildCommandRequest {
    fn validate(&self) -> Result<(), &'static str> {
        if !valid_id(&self.request_id, 96)
            || self.world_epoch == 0
            || self.world_epoch > MAX_SAFE_REVISION
            || self.expected_build_revision > MAX_SAFE_REVISION
        {
            return Err("E_BUILD_COMMAND_INVALID");
        }
        let valid = match &self.action {
            BuildAction::Equip {
                item_id,
                expected_item_id,
            } => {
                valid_id(item_id, 128)
                    && expected_item_id.as_ref().is_none_or(|id| valid_id(id, 128))
            }
            BuildAction::Unequip {
                slot_id,
                expected_item_id,
            } => valid_id(slot_id, 128) && valid_id(expected_item_id, 128),
        };
        if !valid {
            return Err("E_BUILD_COMMAND_INVALID");
        }
        Ok(())
    }
}

impl FormalRuntime {
    /// Same internal-runtime admissibility as trusted equip, with transport concurrency guards.
    /// The source docs impose no additional hub-only or pause requirement for Inventory.
    pub fn apply_build_command(
        &self,
        request: BuildCommandRequest,
    ) -> Result<CommandReceipt, String> {
        request.validate().map_err(str::to_owned)?;
        let mut state = self.state.lock().map_err(|_| "E_RUNTIME_LOCK_POISONED")?;
        // Acquire both locks before mutation, so a projection failure cannot hide a committed change.
        let scene = self
            .scene_runtime
            .lock()
            .map_err(|_| "E_SCENE_RUNTIME_LOCK_POISONED")?;
        let command_id = format!("build:{}", request.request_id);
        let result = apply_request(&mut state, &request);
        let view = project_scene_view(&state, scene.as_ref());
        Ok(match result {
            Ok(applied) => CommandReceipt::outcome(command_id, applied, !applied, None, view),
            Err(error) => CommandReceipt::outcome(command_id, false, false, Some(error), view),
        })
    }
}

fn apply_request(state: &mut RuntimeState, request: &BuildCommandRequest) -> Result<bool, String> {
    if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
    if state.world.player_hp == 0 { return Err("E_RUNTIME_DEAD".into()); }
    if request.world_epoch != state.world.revision.world_epoch {
        return Err("E_BUILD_STALE_EPOCH".into());
    }
    if let Some(previous) = state.build_commands.receipts.iter().find(|entry| {
        entry.world_epoch == request.world_epoch && entry.request_id == request.request_id
    }) {
        return if previous == request {
            Ok(false)
        } else {
            Err("E_BUILD_REQUEST_CONFLICT".into())
        };
    }
    if request.expected_build_revision != state.build_commands.revision {
        return Err("E_BUILD_STALE_REVISION".into());
    }
    if state.last_owner_error.is_some() {
        return Err("E_BUILD_OWNER_UNAVAILABLE".into());
    }
    let catalog = catalog().map_err(str::to_owned)?;
    let mut candidate = state.progression_v6.clone();
    match &request.action {
        BuildAction::Equip {
            item_id,
            expected_item_id,
        } => {
            let item = catalog.get(item_id).ok_or("E_BUILD_CONTENT_NOT_FOUND")?;
            if candidate.equipment.get(&item.slot) != expected_item_id.as_ref() {
                return Err("E_BUILD_SLOT_CHANGED".into());
            }
            if expected_item_id.as_ref() == Some(item_id) {
                return Err("E_BUILD_ALREADY_EQUIPPED".into());
            }
            candidate.equip(item_id)?;
        }
        BuildAction::Unequip {
            slot_id,
            expected_item_id,
        } => {
            let item = catalog
                .get(expected_item_id)
                .ok_or("E_BUILD_CONTENT_NOT_FOUND")?;
            if item.slot != *slot_id || candidate.equipment.get(slot_id) != Some(expected_item_id) {
                return Err("E_BUILD_SLOT_CHANGED".into());
            }
            candidate.unequip(slot_id)?;
        }
    }
    state.install_build_command(candidate)?;
    state
        .build_commands
        .receipts
        .retain(|entry| entry.world_epoch == request.world_epoch);
    state.build_commands.receipts.push_back(request.clone());
    if state.build_commands.receipts.len() > RECEIPT_LIMIT {
        state.build_commands.receipts.pop_front();
    }
    // Older exact replays remain harmless after receipt eviction: their Build revision is stale.
    Ok(true)
}

#[cfg(test)]
mod tests;

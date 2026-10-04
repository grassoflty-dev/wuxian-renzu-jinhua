//! Authoritative character progression and trusted content resolution for Save V6.
use crate::effects::{
    CapabilityCategory, EffectLifetime, EffectResolver, EffectSource, EffectSourceKind, EffectSpec,
};
use crate::player_rules::EffectivePlayerRules;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const BUILD_CONTENT: &str = include_str!("../../data/formal_build_content_v1.json");
const LEGACY_BUILD_CONTENT: &str =
    include_str!("../../data/formal_build_content_legacy_save_v6.json");
const MAX_ENTRIES: usize = 512;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerProgressionV6 {
    pub inventory: BTreeMap<String, u32>,
    pub equipment: BTreeMap<String, String>,
    pub owned_skills: Vec<String>,
    pub equipped_skills: Vec<String>,
    pub skill_proficiency: BTreeMap<String, u32>,
    pub category_proficiency: BTreeMap<String, u32>,
    pub bloodline: Option<String>,
    pub bloodline_tier: u32,
    pub bloodline_proficiency: u32,
    pub treasures: Vec<String>,
    pub gene_stage: u32,
    pub cultivation: BTreeMap<String, u32>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContentCatalog {
    schema_version: u32,
    items: BTreeMap<String, ContentItem>,
    #[serde(default)]
    bloodlines: BTreeMap<String, ContentBloodline>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
// Internal sample balance is catalog data, not a world reward or a public content admission.
// The current samples start at tier 1/proficiency 0; only authored tiers change effects.
struct ContentBloodline {
    ui_category: CapabilityCategory,
    initial_tier: u32,
    tiers: BTreeMap<u32, Vec<EffectSpec>>,
}

impl ContentBloodline {
    fn owned_source(&self, id: &str, tier: u32) -> Result<EffectSource, String> {
        Ok(EffectSource {
            source_kind: EffectSourceKind::Bloodline,
            source_id: id.into(),
            instance_id: format!("bloodline:{id}"),
            lifetime: EffectLifetime::Owned,
            ui_category: Some(self.ui_category),
            effects: self
                .tiers
                .get(&tier)
                .ok_or("E_BUILD_BLOODLINE_TIER_INVALID")?
                .clone(),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContentItem {
    slot: String,
    ui_category: CapabilityCategory,
    effects: Vec<EffectSpec>,
}

impl ContentItem {
    fn equipped_source(&self, item_id: &str) -> EffectSource {
        EffectSource {
            source_kind: EffectSourceKind::Equipment,
            source_id: item_id.into(),
            instance_id: format!("equipment:{}:{item_id}", self.slot),
            lifetime: EffectLifetime::Equipped,
            ui_category: Some(self.ui_category),
            effects: self.effects.clone(),
        }
    }
}

impl PlayerProgressionV6 {
    pub fn sync_capabilities(&mut self, authority: &crate::capability_v1::CapabilityState) {
        self.capabilities = authority
            .grants
            .iter()
            .map(|grant| grant.capability_id.clone())
            .collect();
        self.capabilities.sort();
    }

    pub fn validate(&self) -> Result<(), String> {
        self.validate_with_catalog(&catalog()?)
    }

    fn validate_with_catalog(&self, catalog: &ContentCatalog) -> Result<(), String> {
        if self.inventory.len() > MAX_ENTRIES
            || self.equipment.len() > 32
            || self.owned_skills.len() > MAX_ENTRIES
            || self.equipped_skills.len() > 32
            || self.skill_proficiency.len() > MAX_ENTRIES
            || self.category_proficiency.len() > 64
            || self.treasures.len() > MAX_ENTRIES
            || self.cultivation.len() > 64
            || self.capabilities.len() > MAX_ENTRIES
        {
            return Err("E_BUILD_LIMIT_EXCEEDED".into());
        }
        for (item, quantity) in &self.inventory {
            if !valid_id(item) || *quantity == 0 {
                return Err("E_BUILD_INVENTORY_INVALID".into());
            }
        }
        for (slot, item) in &self.equipment {
            if !valid_id(slot)
                || !catalog
                    .items
                    .get(item)
                    .is_some_and(|definition| definition.slot == *slot)
                || self.inventory.get(item).copied().unwrap_or_default() == 0
            {
                return Err("E_BUILD_EQUIPMENT_INVALID".into());
            }
        }
        for values in [
            &self.owned_skills,
            &self.equipped_skills,
            &self.treasures,
            &self.capabilities,
        ] {
            let unique: std::collections::BTreeSet<_> = values.iter().collect();
            if unique.len() != values.len() || values.iter().any(|value| !valid_id(value)) {
                return Err("E_BUILD_ID_LIST_INVALID".into());
            }
        }
        if self
            .equipped_skills
            .iter()
            .any(|skill| !self.owned_skills.contains(skill))
            || self
                .bloodline
                .as_deref()
                .is_some_and(|value| !valid_id(value))
            || self.bloodline_tier > 100
            || self.bloodline_proficiency > 1_000_000
            || self.gene_stage > 100
            || self
                .skill_proficiency
                .values()
                .any(|value| *value > 1_000_000)
            || self
                .category_proficiency
                .values()
                .any(|value| *value > 1_000_000)
            || self.cultivation.values().any(|value| *value > 1_000_000)
        {
            return Err("E_BUILD_PROGRESSION_INVALID".into());
        }
        for key in self
            .skill_proficiency
            .keys()
            .chain(self.category_proficiency.keys())
            .chain(self.cultivation.keys())
        {
            if !valid_id(key) {
                return Err("E_BUILD_ID_INVALID".into());
            }
        }
        // Unknown legacy IDs (and None with old metadata) remain inert and round-trippable.
        // A known definition must use an authored tier; proficiency is stored, not a multiplier.
        if let Some(definition) = self
            .bloodline
            .as_ref()
            .and_then(|id| catalog.bloodlines.get(id))
        {
            if !definition.tiers.contains_key(&self.bloodline_tier) {
                return Err("E_BUILD_BLOODLINE_TIER_INVALID".into());
            }
        }
        Ok(())
    }

    pub fn grant_trusted_item(&mut self, item_id: &str) -> Result<(), String> {
        let catalog = catalog()?;
        if !catalog.items.contains_key(item_id) {
            return Err("E_BUILD_CONTENT_NOT_FOUND".into());
        }
        let quantity = self.inventory.entry(item_id.into()).or_default();
        *quantity = quantity
            .checked_add(1)
            .ok_or("E_BUILD_QUANTITY_EXHAUSTED")?;
        self.validate()
    }

    pub fn equip(&mut self, item_id: &str) -> Result<(), String> {
        self.validate()?;
        let catalog = catalog()?;
        let item = catalog
            .items
            .get(item_id)
            .ok_or("E_BUILD_CONTENT_NOT_FOUND")?;
        if self.inventory.get(item_id).copied().unwrap_or_default() == 0 {
            return Err("E_BUILD_ITEM_NOT_OWNED".into());
        }
        self.equipment.insert(item.slot.clone(), item_id.into());
        self.validate()
    }

    pub fn unequip(&mut self, slot: &str) -> Result<(), String> {
        self.validate()?;
        if self.equipment.remove(slot).is_none() {
            return Err("E_BUILD_SLOT_EMPTY".into());
        }
        Ok(())
    }

    /// Trusted acquisition only: no source metadata/effects or client-selected tier is accepted.
    pub fn grant_trusted_bloodline(&mut self, id: &str) -> Result<(), String> {
        self.validate()?;
        if self.bloodline.is_some() {
            return Err("E_BUILD_BLOODLINE_OCCUPIED".into());
        }
        self.install_trusted_bloodline(id)
    }

    /// Explicit replacement discards the previous lineage's tier/proficiency. It is not a grant
    /// alias: the caller must name the current lineage, and ordinary acquisition cannot overwrite.
    pub fn replace_trusted_bloodline(&mut self, expected_id: &str, id: &str) -> Result<(), String> {
        self.validate()?;
        self.require_current_bloodline(expected_id)?;
        if expected_id == id {
            return Err("E_BUILD_BLOODLINE_ALREADY_ACTIVE".into());
        }
        self.install_trusted_bloodline(id)
    }

    pub fn remove_trusted_bloodline(&mut self, expected_id: &str) -> Result<(), String> {
        self.validate()?;
        self.require_current_bloodline(expected_id)?;
        self.bloodline = None;
        self.bloodline_tier = 0;
        self.bloodline_proficiency = 0;
        Ok(())
    }

    fn require_current_bloodline(&self, expected_id: &str) -> Result<(), String> {
        match self.bloodline.as_deref() {
            None => Err("E_BUILD_BLOODLINE_EMPTY".into()),
            Some(id) if id != expected_id => Err("E_BUILD_BLOODLINE_CHANGED".into()),
            Some(_) => Ok(()),
        }
    }

    fn install_trusted_bloodline(&mut self, id: &str) -> Result<(), String> {
        let catalog = catalog()?;
        let definition = catalog
            .bloodlines
            .get(id)
            .ok_or("E_BUILD_CONTENT_NOT_FOUND")?;
        let mut candidate = self.clone();
        candidate.bloodline = Some(id.into());
        candidate.bloodline_tier = definition.initial_tier;
        candidate.bloodline_proficiency = 0;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Current equipment/bloodline reconstruction; acquisition sources require authority below.
    pub fn resolve_rules(&self) -> Result<(Vec<EffectSource>, EffectivePlayerRules), String> {
        self.resolve_with_catalog(&catalog()?)
    }

    /// Exact frozen pre-capability source profile, used only by the strict legacy save reader.
    pub(crate) fn resolve_legacy_rules(
        &self,
    ) -> Result<(Vec<EffectSource>, EffectivePlayerRules), String> {
        self.resolve_with_catalog(&parse_catalog(LEGACY_BUILD_CONTENT)?)
    }

    pub fn resolve_rules_with_capabilities(
        &self,
        authority: &crate::capability_v1::CapabilityState,
        rear_view: &crate::world_v3::RearViewAuthorization,
    ) -> Result<(Vec<EffectSource>, EffectivePlayerRules), String> {
        let mut ids: Vec<_> = authority
            .grants
            .iter()
            .map(|grant| grant.capability_id.clone())
            .collect();
        ids.sort();
        if self.capabilities != ids {
            return Err("E_BUILD_CAPABILITY_AUTHORITY_MISMATCH".into());
        }
        let (mut sources, _) = self.resolve_rules()?;
        sources.extend(super::capability_sources::applicable_sources(
            authority, rear_view,
        )?);
        let rules = EffectResolver::resolve(&sources)
            .map_err(|error| format!("E_BUILD_EFFECT_RESOLVE: {error:?}"))?;
        Ok((sources, rules))
    }

    fn resolve_with_catalog(
        &self,
        catalog: &ContentCatalog,
    ) -> Result<(Vec<EffectSource>, EffectivePlayerRules), String> {
        self.validate_with_catalog(catalog)?;
        let mut sources = Vec::new();
        for item_id in self.equipment.values() {
            let definition = catalog
                .items
                .get(item_id)
                .ok_or("E_BUILD_CONTENT_NOT_FOUND")?;
            sources.push(definition.equipped_source(item_id));
        }
        // Keep existing equipment source order/bytes stable for old V6 saves. A legacy unknown
        // lineage does not become an executable source; saved sources are still compared exactly.
        if let Some(id) = &self.bloodline {
            if let Some(definition) = catalog.bloodlines.get(id) {
                sources.push(definition.owned_source(id, self.bloodline_tier)?);
            }
        }
        let rules = EffectResolver::resolve(&sources)
            .map_err(|error| format!("E_BUILD_EFFECT_RESOLVE: {error:?}"))?;
        Ok((sources, rules))
    }
}

fn catalog() -> Result<ContentCatalog, String> {
    parse_catalog(BUILD_CONTENT)
}

fn parse_catalog(content: &str) -> Result<ContentCatalog, String> {
    let catalog: ContentCatalog =
        serde_json::from_str(content).map_err(|_| "E_BUILD_CONTENT_INVALID".to_string())?;
    if catalog.schema_version != 1 {
        return Err("E_BUILD_CONTENT_VERSION_UNSUPPORTED".into());
    }
    if catalog.items.is_empty()
        || catalog.items.len() > MAX_ENTRIES
        || catalog.bloodlines.len() > MAX_ENTRIES
    {
        return Err("E_BUILD_CONTENT_INVALID".into());
    }
    // Validate every trusted definition, including unequipped content. Save data supplies
    // only ownership/slot IDs; source identity, category, lifetime and effects come here.
    for (item_id, definition) in &catalog.items {
        if !valid_id(item_id) || !valid_id(&definition.slot) {
            return Err("E_BUILD_CONTENT_INVALID".into());
        }
        EffectResolver::resolve(&[definition.equipped_source(item_id)])
            .map_err(|_| "E_BUILD_CONTENT_INVALID".to_string())?;
    }
    for (id, definition) in &catalog.bloodlines {
        if !valid_id(id)
            || definition.tiers.is_empty()
            || definition.tiers.len() > 100
            || !definition.tiers.contains_key(&definition.initial_tier)
        {
            return Err("E_BUILD_CONTENT_INVALID".into());
        }
        for tier in definition.tiers.keys() {
            if !(1..=100).contains(tier) {
                return Err("E_BUILD_CONTENT_INVALID".into());
            }
            EffectResolver::resolve(&[definition.owned_source(id, *tier)?])
                .map_err(|_| "E_BUILD_CONTENT_INVALID".to_string())?;
        }
    }
    Ok(catalog)
}

/// Runtime admission is the existing trusted catalog, independent of public release eligibility.
pub(super) fn trusted_item_slots() -> Result<BTreeMap<String, String>, String> {
    Ok(catalog()?
        .items
        .into_iter()
        .map(|(id, definition)| (id, definition.slot))
        .collect())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._:-".contains(&byte)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn test_catalog(effect: Value) -> Value {
        json!({
            "schemaVersion": 1,
            "items": {
                "arbitrary_content_id": {
                    "slot": "test_slot",
                    "uiCategory": "mobility",
                    "effects": [effect]
                }
            }
        })
    }

    #[test]
    fn native_effects_and_source_category_are_data_authored_without_item_id_branches() {
        let value = test_catalog(json!({
            "terrain_movement_modifier": {
                "tag": "terrain.water_shallow", "multiplier_bps": 8_000, "stack": "max"
            }
        }));
        let catalog = parse_catalog(&value.to_string()).unwrap();
        let source = catalog.items["arbitrary_content_id"].equipped_source("arbitrary_content_id");
        assert_eq!(source.source_kind, EffectSourceKind::Equipment);
        assert_eq!(source.source_id, "arbitrary_content_id");
        assert_eq!(
            source.instance_id,
            "equipment:test_slot:arbitrary_content_id"
        );
        assert_eq!(source.lifetime, EffectLifetime::Equipped);
        assert_eq!(source.ui_category, Some(CapabilityCategory::Mobility));
        let rules = EffectResolver::resolve(&[source]).unwrap();
        assert_eq!(
            rules
                .terrain
                .movement_multiplier_bps(crate::effects::TerrainTag::WaterShallow),
            8_000
        );
    }

    #[test]
    fn catalog_rejects_invalid_native_effects_before_any_equipment_is_selected() {
        for effect in [
            json!({"world_boundary_bypass": {"stack": "any"}}),
            json!({"terrain_penalty_ignore": {"tag": "world", "stack": "any"}}),
            json!({"map_knowledge_level": {"tag": "topology", "level": "none", "stack": "max"}}),
            json!({"map_knowledge_level": {"tag": "topology", "level": "full", "stack": "add"}}),
            json!({"hazard_resistance": {"tag": "heat", "reduction_bps": 10_000, "stack": "multiply_remaining"}}),
            json!({"terrain_movement_modifier": {"tag": "terrain.water_shallow", "multiplier_bps": 0, "stack": "max"}}),
            json!({"terrain_movement_modifier": {"tag": "terrain.water_shallow", "multiplier_bps": 10_001, "stack": "max"}}),
            json!({"terrain_movement_modifier": {"tag": "terrain.water_shallow", "multiplier_bps": 8_000, "stack": "multiply"}}),
            json!({"perception": {"tag": "rear_view", "stack": "any", "trusted": true}}),
        ] {
            let value = test_catalog(effect);
            assert_eq!(
                parse_catalog(&value.to_string()).unwrap_err(),
                "E_BUILD_CONTENT_INVALID",
                "accepted {value}"
            );
        }
    }

    #[test]
    fn catalog_rejects_invalid_metadata_and_empty_definitions() {
        let valid = test_catalog(json!({"perception": {"tag": "rear_view", "stack": "any"}}));
        for (field, invalid) in [
            ("slot", json!("Invalid Slot")),
            ("slot", json!("a".repeat(128))),
            ("uiCategory", json!("unknown")),
            ("effects", json!([])),
            ("sourceKind", json!("innate_capability")),
            ("lifetime", json!("permanent")),
        ] {
            let mut value = valid.clone();
            value["items"]["arbitrary_content_id"][field] = invalid;
            assert!(
                parse_catalog(&value.to_string()).is_err(),
                "accepted {value}"
            );
        }
        let mut unknown_version = valid;
        unknown_version["schemaVersion"] = json!(2);
        assert_eq!(
            parse_catalog(&unknown_version.to_string()).unwrap_err(),
            "E_BUILD_CONTENT_VERSION_UNSUPPORTED"
        );
        assert!(parse_catalog(r#"{"schemaVersion":1,"items":{}}"#).is_err());
        assert!(catalog().is_ok());
    }

    fn bloodline_catalog() -> Value {
        let mut value = test_catalog(json!({"perception": {"tag": "rear_view", "stack": "any"}}));
        value["bloodlines"] = json!({"arbitrary_lineage": {
            "uiCategory": "body", "initialTier": 1,
            "tiers": {
                "1": [{"hazard_resistance": {"tag": "heat", "reduction_bps": 5000, "stack": "multiply_remaining"}}],
                "2": [{"hazard_resistance": {"tag": "heat", "reduction_bps": 7500, "stack": "multiply_remaining"}}]
            }
        }});
        value
    }

    #[test]
    fn bloodline_tiers_and_category_are_data_authored_with_stable_owned_identity() {
        let catalog = parse_catalog(&bloodline_catalog().to_string()).unwrap();
        let definition = &catalog.bloodlines["arbitrary_lineage"];
        let first = definition.owned_source("arbitrary_lineage", 1).unwrap();
        let second = definition.owned_source("arbitrary_lineage", 2).unwrap();
        assert_eq!(first.source_kind, EffectSourceKind::Bloodline);
        assert_eq!(first.source_id, "arbitrary_lineage");
        assert_eq!(first.instance_id, "bloodline:arbitrary_lineage");
        assert_eq!(first.instance_id, second.instance_id);
        assert_eq!(first.lifetime, EffectLifetime::Owned);
        assert_eq!(first.ui_category, Some(CapabilityCategory::Body));
        assert_eq!(
            EffectResolver::resolve(&[first])
                .unwrap()
                .hazards
                .remaining_damage_bps(crate::effects::HazardTag::Heat),
            5000
        );
        assert_eq!(
            EffectResolver::resolve(&[second])
                .unwrap()
                .hazards
                .remaining_damage_bps(crate::effects::HazardTag::Heat),
            2500
        );
        assert_eq!(
            definition.owned_source("arbitrary_lineage", 3).unwrap_err(),
            "E_BUILD_BLOODLINE_TIER_INVALID"
        );
    }

    #[test]
    fn unowned_bloodline_catalog_rejects_invalid_metadata_tiers_and_effects() {
        for (field, invalid) in [
            ("uiCategory", json!("unknown")),
            ("initialTier", json!(0)),
            ("initialTier", json!(3)),
            ("initialTier", json!(101)),
            ("tiers", json!({})),
            ("tiers", json!({"1": []})),
            (
                "tiers",
                json!({"0": [{"perception": {"tag": "rear_view", "stack": "any"}}]}),
            ),
            (
                "tiers",
                json!({"1": [{"hazard_resistance": {"tag": "heat", "reduction_bps": 10000, "stack": "multiply_remaining"}}]}),
            ),
            (
                "tiers",
                json!({"1": [{"hazard_resistance": {"tag": "heat", "reduction_bps": 5000, "stack": "add"}}]}),
            ),
            ("tiers", json!({"1": [{"unknown": {}}]})),
            ("sourceKind", json!("equipment")),
            ("instanceId", json!("forged")),
            ("lifetime", json!("permanent")),
        ] {
            let mut value = bloodline_catalog();
            value["bloodlines"]["arbitrary_lineage"][field] = invalid;
            assert!(
                parse_catalog(&value.to_string()).is_err(),
                "accepted {value}"
            );
        }
        // A valid initial tier cannot hide a corrupt unselected higher tier.
        let mut value = bloodline_catalog();
        value["bloodlines"]["arbitrary_lineage"]["tiers"]["2"] = json!([]);
        assert!(parse_catalog(&value.to_string()).is_err());
        for id in ["Invalid ID".to_owned(), "a".repeat(128)] {
            let mut value = bloodline_catalog();
            let definition = value["bloodlines"]["arbitrary_lineage"].take();
            value["bloodlines"] = json!({id: definition});
            assert!(parse_catalog(&value.to_string()).is_err());
        }
    }
}

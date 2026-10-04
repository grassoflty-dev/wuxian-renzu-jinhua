//! Trusted applicability adapter for the six existing acquired capabilities.
//! Grant/selection authority stays in CapabilityState; sources never create acquisition rows.
use crate::{
    capability_v1::{
        CapabilityState, CAPABILITY_SCHEMA_VERSION, CAP_ACOUSTIC_MAPPING, CAP_AIR_STEP,
        CAP_ENEMY_VITALS, CAP_LOCAL_MAP, CAP_REAR_VIEW, CAP_REGENERATION,
    },
    effects::{
        CapabilityCategory, EffectLifetime, EffectResolver, EffectSource, EffectSourceKind,
        EffectSpec,
    },
    world_v3::RearViewAuthorization,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const CONTENT: &str = include_str!("../../data/formal_capability_effects_v1.json");
const CANONICAL: [&str; 6] = [
    CAP_LOCAL_MAP,
    CAP_REAR_VIEW,
    CAP_ENEMY_VITALS,
    CAP_REGENERATION,
    CAP_ACOUSTIC_MAPPING,
    CAP_AIR_STEP,
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Catalog {
    schema_version: u32,
    capabilities: BTreeMap<String, Definition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Definition {
    ui_category: CapabilityCategory,
    effects: Vec<EffectSpec>,
}

fn selected_policy(id: &str) -> bool {
    matches!(id, CAP_LOCAL_MAP | CAP_ACOUSTIC_MAPPING)
}

fn source(id: &str, definition: &Definition) -> EffectSource {
    EffectSource {
        source_kind: EffectSourceKind::InnateCapability,
        source_id: id.into(),
        instance_id: format!("capability:{id}"),
        lifetime: if selected_policy(id) {
            EffectLifetime::Selected
        } else {
            EffectLifetime::Permanent
        },
        ui_category: Some(definition.ui_category),
        effects: definition.effects.clone(),
    }
}

fn catalog(raw: &str) -> Result<Catalog, String> {
    let catalog: Catalog =
        serde_json::from_str(raw).map_err(|_| "E_CAPABILITY_EFFECT_CONTENT_INVALID")?;
    let expected: BTreeSet<_> = CANONICAL.into_iter().collect();
    let actual: BTreeSet<_> = catalog.capabilities.keys().map(String::as_str).collect();
    if catalog.schema_version != 1 || actual != expected {
        return Err("E_CAPABILITY_EFFECT_CONTENT_INVALID".into());
    }
    for (id, definition) in &catalog.capabilities {
        EffectResolver::resolve(&[source(id, definition)])
            .map_err(|_| "E_CAPABILITY_EFFECT_CONTENT_INVALID")?;
    }
    Ok(catalog)
}

/// Sources are applicable now, not every owned row. Selected lifetime alone does not filter
/// EffectResolver inputs; omitting inactive sources here preserves existing V1 selection policy.
pub(super) fn applicable_sources(
    authority: &CapabilityState,
    rear_view: &RearViewAuthorization,
) -> Result<Vec<EffectSource>, String> {
    let catalog = catalog(CONTENT)?;
    if authority.schema_version != CAPABILITY_SCHEMA_VERSION {
        return Err("E_BUILD_CAPABILITY_AUTHORITY_INVALID".into());
    }
    let mut grants = BTreeSet::new();
    for grant in &authority.grants {
        if !catalog.capabilities.contains_key(&grant.capability_id)
            || !grants.insert(grant.capability_id.as_str())
        {
            return Err("E_BUILD_CAPABILITY_AUTHORITY_INVALID".into());
        }
    }
    let selected: BTreeSet<_> = authority.selected.iter().map(String::as_str).collect();
    if selected.len() != authority.selected.len() || !selected.is_subset(&grants) {
        return Err("E_BUILD_CAPABILITY_AUTHORITY_INVALID".into());
    }
    // Preserve the legacy denial of a spoofed projection/authorization pair. An independent
    // equipment source may still grant rear view through the same resolver.
    let valid_rear_view = authority.rear_view_authorization.granted
        && authority.rear_view_authorization.grant_id.as_deref() == Some(CAP_REAR_VIEW)
        && authority
            .rear_view_authorization
            .granted_at_revision
            .is_some()
        && rear_view == &authority.rear_view_authorization;
    Ok(grants
        .into_iter()
        .filter(|id| {
            (!selected_policy(id) || selected.contains(id))
                && (*id != CAP_REAR_VIEW || valid_rear_view)
        })
        .map(|id| source(id, &catalog.capabilities[id]))
        .collect())
}

#[cfg(test)]
mod tests;

//! The absent discriminator is one exact frozen wire profile, never a recovery heuristic.
use super::{validate_authority, SaveV6, EFFECT_SOURCES_VERSION, SAVE_V6_SCHEMA_VERSION};
use crate::{
    effects::EffectSource, formal_runtime::build_v6::PlayerProgressionV6, save_v5::SaveV5,
    world_persistent_v1::WorldPersistentState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Profile {
    Legacy,
    Current,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacySaveV6 {
    schema_version: u32,
    save: SaveV5,
    progression: PlayerProgressionV6,
    effect_sources: Vec<EffectSource>,
    #[serde(default, skip_serializing_if = "WorldPersistentState::is_default")]
    world_persistent_v1: WorldPersistentState,
}

pub(crate) fn decode(value: Value) -> Result<(SaveV6, Profile), String> {
    let object = value.as_object().ok_or("E_SAVE_CORRUPT")?;
    if let Some(version) = object.get("effectSourcesVersion") {
        if version.as_u64() != Some(EFFECT_SOURCES_VERSION as u64) {
            return Err("E_SAVE_EFFECT_SOURCES_VERSION_UNSUPPORTED".into());
        }
        let save: SaveV6 = strict(value)?;
        save.validate()?;
        return Ok((save, Profile::Current));
    }
    // SaveV5 is shared by current and frozen legacy DTOs. Presence of a new
    // support field is forbidden in the absent-discriminator profile, even null.
    if object.get("save").and_then(Value::as_object)
        .is_some_and(|save| save.contains_key("movingSupportFrame")) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    // Keep the absent-discriminator profile frozen even when current persistent
    // state gains additive fields. Presence is rejected, including null/empty.
    if object
        .get("worldPersistentV1")
        .and_then(Value::as_object)
        .is_some_and(|persistent| {
            persistent
                .keys()
                .any(|key| !matches!(key.as_str(), "greyHive" | "clockworks" | "mistHarbor"))
        })
    {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    if object
        .get("worldPersistentV1")
        .and_then(|p| p.get("mistHarbor"))
        .and_then(Value::as_object)
        .is_some_and(|p| p.contains_key("wardenDefeated") || p.contains_key("tideboundActorRosterVersion") || p.contains_key("signalWraithControllerVersion"))
        || object
            .get("save")
            .and_then(|s| s.get("genericActors"))
            .and_then(Value::as_array)
            .is_some_and(|actors| {
                actors
                    .iter()
                    .any(|a| a.as_object().is_some_and(|a| a.contains_key("warden") || a.contains_key("ordinary") || a.contains_key("group") || a.contains_key("controllerVariant")))
            })
    {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    if object
        .get("worldPersistentV1")
        .and_then(|p| p.get("clockworks"))
        .and_then(Value::as_object)
        .is_some_and(|p| p.contains_key("pressureValveIds") || p.contains_key("coreConsoleConfirmed") || p.contains_key("actorRosterVersion") || p.contains_key("gearShaftSupportVersion"))
    {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    if object.get("worldPersistentV1").and_then(|p| p.get("greyHive"))
        .and_then(Value::as_object).is_some_and(|p| p.contains_key("beacon") || p.contains_key("gateBActorRosterVersion") || p.contains_key("swarmActorRosterVersion")) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    if object.get("save").and_then(|s|s.get("actors")).and_then(Value::as_array).is_some_and(|actors|actors.iter().any(|actor|
        actor.as_object().is_some_and(|a|a.contains_key("chargeController") || a.get("state").and_then(Value::as_str)
            .is_some_and(|s|matches!(s,"chargeWindup"|"charge"|"stagger"))))) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    let old: LegacySaveV6 = strict(value)?;
    if old.schema_version != SAVE_V6_SCHEMA_VERSION {
        return Err("E_SAVE_VERSION_UNSUPPORTED".into());
    }
    validate_authority(&old.save, &old.progression, &old.world_persistent_v1)?;
    // This validates progression with the frozen c571 catalog, including source order/effects.
    let (sources, _) = old.progression.resolve_legacy_rules()?;
    if sources != old.effect_sources {
        return Err("E_SAVE_EFFECT_SOURCES_INVALID".into());
    }
    let normalized = SaveV6::from_authority(old.save, old.progression, old.world_persistent_v1)?;
    Ok((normalized, Profile::Legacy))
}

fn strict<T: for<'de> Deserialize<'de> + Serialize>(value: Value) -> Result<T, String> {
    let decoded: T = serde_json::from_value(value.clone()).map_err(|_| "E_SAVE_CORRUPT")?;
    let known = serde_json::to_value(&decoded).map_err(|_| "E_SAVE_CORRUPT")?;
    if !crate::save_v5::same_schema_shape(&value, &known) {
        return Err("E_SAVE_SCHEMA_INVALID".into());
    }
    Ok(decoded)
}

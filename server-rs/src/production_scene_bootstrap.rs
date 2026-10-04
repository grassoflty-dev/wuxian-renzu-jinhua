//! Build-verified, statically embedded thirty-scene native campaign bundle.
//!
//! Startup checks locked scene SHA-256, canonical world membership, and the
//! exact Clockworks progression emitters before admitting authored world gates.
//! Fresh journeys begin in Return Station; content admission is not a claim of
//! native visual, installer, performance, or genuine campaign-route acceptance.

use crate::{formal_runtime::FormalRuntime, world_v3::WorldView};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/production_scene_bundle.rs"));
}

pub use generated::{EMBEDDED_SCENE_IDS, EMBEDDED_SCENE_SHA256};

fn canonical_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut canonical = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
            canonical.push(b'\n');
            index += 2;
        } else {
            canonical.push(bytes[index]);
            index += 1;
        }
    }
    canonical
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Validate a candidate embedded bundle against the build-locked identities
/// and SHA-256 digests. Exposed for fail-closed regression tests.
pub fn validate_scene_bundle(scene_json: &[&str]) -> Result<(), String> {
    if scene_json.len() != generated::EMBEDDED_SCENE_IDS.len()
        || scene_json.len() != generated::EMBEDDED_SCENE_SHA256.len()
        || scene_json.len() != 30
        || scene_json.len() != generated::EMBEDDED_SCENE_WORLD_IDS.len()
    {
        return Err("E_PRODUCTION_SCENE_BUNDLE_COUNT".into());
    }
    for (index, raw) in scene_json.iter().enumerate() {
        let expected_id = generated::EMBEDDED_SCENE_IDS[index];
        let (hash_id, expected_sha) = generated::EMBEDDED_SCENE_SHA256[index];
        if hash_id != expected_id {
            return Err("E_PRODUCTION_SCENE_BUNDLE_LOCK".into());
        }
        let actual_sha = sha256(&canonical_bytes(raw.as_bytes()));
        if actual_sha != expected_sha {
            return Err(format!("E_PRODUCTION_SCENE_SHA256:{expected_id}"));
        }
        let scene: serde_json::Value =
            serde_json::from_str(raw).map_err(|_| "E_PRODUCTION_SCENE_JSON")?;
        if scene.get("sceneId").and_then(serde_json::Value::as_str) != Some(expected_id)
            || scene.get("worldId").and_then(serde_json::Value::as_str)
                != Some(generated::EMBEDDED_SCENE_WORLD_IDS[index])
        {
            return Err(format!("E_PRODUCTION_SCENE_IDENTITY:{expected_id}"));
        }
    }
    Ok(())
}

pub fn embedded_scene_json() -> Result<&'static [&'static str], String> {
    validate_scene_bundle(generated::EMBEDDED_SCENE_JSON)?;
    Ok(generated::EMBEDDED_SCENE_JSON)
}

/// Initialize the authoritative runtime from the immutable desktop bundle.
/// No working-directory or external content path is consulted at runtime.
pub fn install(runtime: &FormalRuntime) -> Result<WorldView, String> {
    let scene_json = embedded_scene_json()?;
    let admitted_assets: BTreeSet<String> = generated::EMBEDDED_ADMITTED_ASSET_IDS
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    let entity_types: BTreeSet<String> = generated::EMBEDDED_ENTITY_TYPES
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    runtime.load_production_scene_registry(
        scene_json.iter().copied(),
        &admitted_assets,
        &entity_types,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_bundle_contains_the_locked_gh_mh_scenes_and_return_station() {
        validate_scene_bundle(generated::EMBEDDED_SCENE_JSON).unwrap();
        assert_eq!(generated::EMBEDDED_SCENE_JSON.len(), 30);
        assert_eq!(generated::EMBEDDED_SCENE_IDS[11], "rs_core_room");
        assert_eq!(generated::EMBEDDED_SCENE_WORLD_IDS[11], "return_station");
        assert_eq!(generated::EMBEDDED_SCENE_IDS[12], "mh_fog_pier");
        assert_eq!(generated::EMBEDDED_SCENE_IDS[20], "mh_extraction");
        assert_eq!(generated::EMBEDDED_SCENE_IDS[21], "cw_entry_foundry");
        assert_eq!(
            generated::EMBEDDED_SCENE_IDS.last(),
            Some(&"cw_shutdown_exit")
        );
        assert!(generated::EMBEDDED_SCENE_WORLD_IDS[12..]
            .iter()
            .take(9)
            .all(|world| *world == "mist_harbor"));
        assert!(generated::EMBEDDED_SCENE_WORLD_IDS[21..]
            .iter()
            .all(|world| *world == "clockworks"));
        let expected_clockworks_pins = [
            (
                "cw_entry_foundry",
                "d90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1",
            ),
            (
                "cw_pressure_hall",
                "53a7dc2925d5210da366abe8d03c07334c39cfe8b7da12f238090e27b1e5017a",
            ),
            (
                "cw_conveyor_bridge",
                "845b664b2212c5d9bf55cf72d34c9c69bf0caf345fa90ed9a7173a1df276e0ae",
            ),
            (
                "cw_boiler_chamber",
                "04b00520e16435d159f1a459016083020fd40d11a2f8086d2f78dcbd0e483b39",
            ),
            (
                "cw_gear_shaft",
                "91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9",
            ),
            (
                "cw_furnace_heart",
                "f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5",
            ),
            (
                "cw_forged_guard_arena",
                "38ae047e46e980f953377121c78713a79b69f84b994e2dd1984e1404fe77ccf5",
            ),
            (
                "cw_regulator_core",
                "b9b352865c5b4acdddbc52abe19abce7fea560687607b268dbe41dd456000fc7",
            ),
            (
                "cw_shutdown_exit",
                "4e24942874fb43b0111eea4c4ac6035fd4ccd65e5b45b845ca4063f63ab2692b",
            ),
        ];
        assert_eq!(
            &generated::EMBEDDED_SCENE_SHA256[21..],
            &expected_clockworks_pins
        );
        assert_eq!(generated::EMBEDDED_ADMITTED_ASSET_IDS.len(), 43);
        let expected_entities = BTreeSet::from([
            "enemy.clockworks.forged_guard",
            "enemy.clockworks.forged_guard_elite",
            "enemy.clockworks.prime_regulator",
            "enemy.clockworks.pressure_drone",
            "enemy.clockworks.furnace_hound",
            "enemy.grey_hive.sentinel",
            "enemy.grey_hive.brute",
            "enemy.grey_hive.swarm",
            "enemy.mist_harbor.drowned",
            "enemy.mist_harbor.signal_wraith",
            "enemy.mist_harbor.tidebound",
            "enemy.mist_harbor.resonance_warden",
            "grey_hive.infected_maintenance_worker",
            "grey_hive.infected_security",
            "npc.baizhi",
        ]);
        let embedded_entities: BTreeSet<_> =
            generated::EMBEDDED_ENTITY_TYPES.iter().copied().collect();
        assert_eq!(embedded_entities, expected_entities);
    }
}

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

#[path = "build_support/bundle_identity.rs"]
mod bundle_identity;

const EXPECTED_SCENES: [(&str, &str); 11] = [
    (
        "gh_beacon.json",
        "3cab424771d10799ee82a453a72117affb0ebca09dd814f25308be70310da9ee",
    ),
    (
        "gh_bio_isolation.json",
        "4ed45ceaa1d96cd03cafb4f9f3027b3bd4e294dcb104d2de5b266364d9791137",
    ),
    (
        "gh_central_shaft.json",
        "806ef2be417d59ab7041002cecfb26c1fc383db920157feb29c2669c343df5da",
    ),
    (
        "gh_deep_decon.json",
        "65e0cbf11183fe114259d680b0cd84a23a7f8619a25e628f4b30a7a84ce870a8",
    ),
    (
        "gh_entry_maintenance.json",
        "5deb6c01b2ad81144144ae2526c06f8c337839e8d880aeb1b3af3a225afe787b",
    ),
    (
        "gh_exit.json",
        "6ab262467e1992feac94f27d8bf571dcf77ddbc8c1693a79c57b3aeebff0c54c",
    ),
    (
        "gh_gate_a.json",
        "3c62f3c35a2c9f2e935707e7a93c34a8d6f5696ed730b8487c4efe0b6d1ee484",
    ),
    (
        "gh_gate_b.json",
        "a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1",
    ),
    (
        "gh_lockdown.json",
        "718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850",
    ),
    (
        "gh_power_room.json",
        "3d05edeca4a119bc7f2db7aa094041a7ad61380b00015a613fda787025e67854",
    ),
    (
        "gh_sentinel_arena.json",
        "3e3e3a52dca003ec13c7b3698317f6a32627091eeda833a49896910e521f45bb",
    ),
];
// Reviewed non-Grey-Hive content. Mist Harbor is embedded in the native
// registry and only its Rust-authorized Return Station route is playable.
// Clockworks is embedded for explicit staged-production registration; this
// does not create a route into the world or complete its required-event set.
const REVIEWED_SCENES: [(&str, &str, &str); 18] = [
    (
        "cw_entry_foundry.json",
        "d90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1",
        "clockworks",
    ),
    (
        "cw_pressure_hall.json",
        "53a7dc2925d5210da366abe8d03c07334c39cfe8b7da12f238090e27b1e5017a",
        "clockworks",
    ),
    (
        "cw_conveyor_bridge.json",
        "845b664b2212c5d9bf55cf72d34c9c69bf0caf345fa90ed9a7173a1df276e0ae",
        "clockworks",
    ),
    (
        "cw_boiler_chamber.json",
        "04b00520e16435d159f1a459016083020fd40d11a2f8086d2f78dcbd0e483b39",
        "clockworks",
    ),
    (
        "cw_gear_shaft.json",
        "91afdb035c3df824806884c49d1e8f71034ff8f51b7bf90280bf71594896fea9",
        "clockworks",
    ),
    (
        "cw_furnace_heart.json",
        "f16f3941767aed71023cd8b9ce75642345a103eabf5c77682e358c96c2e219d5",
        "clockworks",
    ),
    (
        "cw_forged_guard_arena.json",
        "38ae047e46e980f953377121c78713a79b69f84b994e2dd1984e1404fe77ccf5",
        "clockworks",
    ),
    (
        "cw_regulator_core.json",
        "b9b352865c5b4acdddbc52abe19abce7fea560687607b268dbe41dd456000fc7",
        "clockworks",
    ),
    (
        "cw_shutdown_exit.json",
        "4e24942874fb43b0111eea4c4ac6035fd4ccd65e5b45b845ca4063f63ab2692b",
        "clockworks",
    ),
    (
        "mh_fog_pier.json",
        "1f12ceae6565dcfb8f3b9f497840f8027748e6109208e788ea6d7579fff00946",
        "mist_harbor",
    ),
    (
        "mh_tidal_warehouse.json",
        "bcf83727cbf17118a534547e8d50abd65f8c606a342290b66e5a778a0c2d61d9",
        "mist_harbor",
    ),
    (
        "mh_signal_yard.json",
        "14f708142f6d4c15b8abdb3671e5357f845d96228c5b9aed719e19895e3a66d8",
        "mist_harbor",
    ),
    (
        "mh_drowned_quay.json",
        "e315ef1be88ab2f1ff8defcf7050beef439b7793cda3d121e4e97b4bdf9b41a9",
        "mist_harbor",
    ),
    (
        "mh_breakwater.json",
        "2de6f7b471b6b873fafad7e943a0f163e057ffca0787823625bf37bb23bd0b14",
        "mist_harbor",
    ),
    (
        "mh_pump_station.json",
        "557c180648bc8c73621b444dd108f2207535750947e9a7738a5716dd1b2a5437",
        "mist_harbor",
    ),
    (
        "mh_resonance_tower.json",
        "61d42c94a06204b1c4ddac4f29577d3e031a14f1b6fcc66613d625006fd94452",
        "mist_harbor",
    ),
    (
        "mh_warden_arena.json",
        "bb0ddca043c57ef4d0d1b327ace455e7f5f8c870845c113064144f64006df86b",
        "mist_harbor",
    ),
    (
        "mh_extraction.json",
        "3476501f29ea441c51caeab61633b929ef7e7abc3c887691a7557098986a47aa",
        "mist_harbor",
    ),
];
const NATIVE_AUXILIARY_SCENES: [(&str, &str, &str); 1] = [(
    "rs_core_room.json",
    "927f8dc466403837b081027bafb04c3c66a6ca17ee71bf5ce0a7d891e4fb3ad9",
    "return_station",
)];
const EXPECTED_INPUTS: [(&str, &str); 4] = [
    (
        "governance/assets/RUNTIME_ASSET_MANIFEST.json",
        "081bebad5eede90e6ed50f2f7cacb930a67f2295961d405f58141d23874ec414",
    ),
    (
        "governance/assets/AI_ASSET_RELEASE_MANIFEST.json",
        "c644b5953e9265d45796dfb8a6dad1c0fd8abfbffb3eb4ace84987d8238cee2d",
    ),
    (
        "content/enemies/entity-types.json",
        "de1ff6836b1a9ab216161c468688c4924f929f2036ccf1cb9c4d9948fada7e1a",
    ),
    (
        "server-rs/data/world_progression_v1.json",
        "f29b58ad1c345a7c2d84fcc71e6ad640911514ced58d1f81a789593d93a1084d",
    ),
];

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

fn read_json(path: &Path) -> Result<(Vec<u8>, Value), String> {
    let bytes = canonical_bytes(
        &fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?,
    );
    let json = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))?;
    Ok((bytes, json))
}

fn string_field<'a>(value: &'a Value, key: &str, from: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{from} missing string field {key}"))
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}

fn write_native_bundle_identity(
    out_dir: &Path,
    identity: &bundle_identity::ValidatedBundleIdentity,
) -> Result<(), String> {
    let build_identity = serde_json::to_string(&identity.build_identity)
        .map_err(|error| format!("cannot serialize native bundle build identity: {error}"))?;
    let mut generated = String::from("// Generated by server-rs/build.rs; do not edit.\n");
    generated.push_str(&format!(
        "pub const NATIVE_BUILD_IDENTITY_JSON: &str = {};\n",
        rust_string(&build_identity)
    ));
    generated.push_str(&format!(
        "pub const NATIVE_BUNDLE_SIDECAR_SHA256: &str = {};\n",
        rust_string(&identity.sidecar_sha256)
    ));
    generated.push_str(&format!(
        "pub const NATIVE_BUNDLE_ENTRY_PATH: &str = {};\n",
        rust_string(&identity.entry.path)
    ));
    generated.push_str(&format!(
        "pub const NATIVE_BUNDLE_ENTRY_SIZE_BYTES: u64 = {};\n",
        identity.entry.size_bytes
    ));
    generated.push_str(&format!(
        "pub const NATIVE_BUNDLE_ENTRY_SHA256: &str = {};\n",
        rust_string(&identity.entry.sha256)
    ));
    generated.push_str(&format!(
        "pub const NATIVE_BUNDLE_FILE_COUNT: u64 = {};\n",
        identity.file_count
    ));
    fs::write(out_dir.join("native_bundle_identity.rs"), generated)
        .map_err(|error| format!("cannot generate native bundle identity: {error}"))
}

fn run() -> Result<(), String> {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing CARGO_MANIFEST_DIR")?);
    let repo_root = manifest_dir
        .parent()
        .ok_or("server-rs must be directly under the repository root")?;
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);

    let identity_module = manifest_dir.join("build_support/bundle_identity.rs");
    println!("cargo:rerun-if-changed={}", identity_module.display());
    let bundle_identity = bundle_identity::validate_repo_frontend_dist(
        &repo_root.join("apps/web/dist"),
        repo_root,
        env!("CARGO_PKG_VERSION"),
    )?;
    write_native_bundle_identity(&out_dir, &bundle_identity)?;

    let mut input_json = BTreeMap::new();
    for (relative, expected_sha) in EXPECTED_INPUTS {
        let path = repo_root.join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
        let (bytes, json) = read_json(&path)?;
        let actual_sha = sha256(&bytes);
        if actual_sha != expected_sha {
            return Err(format!(
                "locked production input SHA-256 mismatch for {relative}: expected {expected_sha}, got {actual_sha}"
            ));
        }
        input_json.insert(relative, json);
    }

    let runtime = &input_json["governance/assets/RUNTIME_ASSET_MANIFEST.json"];
    let release = &input_json["governance/assets/AI_ASSET_RELEASE_MANIFEST.json"];
    let runtime_assets = runtime
        .get("assets")
        .and_then(Value::as_array)
        .ok_or("runtime manifest has no assets array")?;
    let release_assets = release
        .get("assets")
        .and_then(Value::as_array)
        .ok_or("B0 release manifest has no assets array")?;
    if runtime_assets.len() != 43 || release_assets.len() != 94 {
        return Err(format!(
            "unexpected B0/runtime asset counts: runtime={}, release={}",
            runtime_assets.len(),
            release_assets.len()
        ));
    }

    let mut approved_release = BTreeMap::new();
    let mut concept = 0;
    let mut rejected = 0;
    for row in release_assets {
        let id = string_field(row, "asset_id", "B0 row")?.to_owned();
        match string_field(row, "release_status", "B0 row")? {
            "release_approved" => {
                let path = string_field(row, "relative_path", "B0 row")?.to_owned();
                let hash = string_field(row, "sha256", "B0 row")?.to_owned();
                if approved_release.insert(id, (path, hash)).is_some() {
                    return Err("duplicate asset id in B0 release manifest".into());
                }
            }
            "concept_only" => concept += 1,
            "reject" => rejected += 1,
            other => return Err(format!("unknown B0 release status: {other}")),
        }
    }
    if approved_release.len() != 43 || concept != 30 || rejected != 21 {
        return Err(format!(
            "unexpected B0 admission counts: approved={}, concept_only={concept}, reject={rejected}",
            approved_release.len()
        ));
    }

    let mut admitted_assets = BTreeSet::new();
    for item in runtime_assets {
        if string_field(item, "admission", "runtime asset")? != "release_approved" {
            return Err("runtime manifest contains a non-approved asset".into());
        }
        let id = string_field(item, "assetId", "runtime asset")?;
        let source_path = string_field(item, "sourcePath", "runtime asset")?;
        let source_hash = string_field(item, "sourceSha256", "runtime asset")?;
        let Some((release_path, release_hash)) = approved_release.get(id) else {
            return Err(format!("runtime asset is absent from B0 approvals: {id}"));
        };
        if source_path != release_path || source_hash != release_hash {
            return Err(format!("B0 provenance mismatch for runtime asset {id}"));
        }
        admitted_assets.insert(id.to_owned());
    }
    if admitted_assets.len() != approved_release.len() {
        return Err("runtime manifest and B0 approved asset IDs differ".into());
    }

    let entity_catalog = &input_json["content/enemies/entity-types.json"];
    if entity_catalog.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("unsupported entity catalog schema".into());
    }
    let entity_types: BTreeSet<String> = entity_catalog
        .get("entityTypes")
        .and_then(Value::as_array)
        .ok_or("entity catalog has no entityTypes array")?
        .iter()
        .map(|entity| {
            entity
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "entity catalog contains a non-string entry".to_owned())
        })
        .collect::<Result<_, _>>()?;
    let expected_entities = BTreeSet::from([
        "enemy.clockworks.forged_guard".to_owned(),
        "enemy.clockworks.forged_guard_elite".to_owned(),
        "enemy.clockworks.prime_regulator".to_owned(),
        "enemy.clockworks.pressure_drone".to_owned(),
        "enemy.clockworks.furnace_hound".to_owned(),
        "enemy.grey_hive.sentinel".to_owned(),
        "enemy.grey_hive.brute".to_owned(),
        "enemy.grey_hive.swarm".to_owned(),
        "grey_hive.infected_maintenance_worker".to_owned(),
        "grey_hive.infected_security".to_owned(),
        "enemy.mist_harbor.drowned".to_owned(),
        "enemy.mist_harbor.signal_wraith".to_owned(),
        "enemy.mist_harbor.tidebound".to_owned(),
        "enemy.mist_harbor.resonance_warden".to_owned(),
    ]);
    if entity_types != expected_entities {
        return Err(
            "entity catalog differs from the authored Grey Hive, Mist Harbor, and staged Clockworks actor slice".into(),
        );
    }

    let scene_dir = repo_root.join("content/scenes/compiled");
    println!("cargo:rerun-if-changed={}", scene_dir.display());
    let json_files: BTreeSet<String> = fs::read_dir(&scene_dir)
        .map_err(|error| format!("cannot read {}: {error}", scene_dir.display()))?
        .map(|entry| entry.map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|entry| {
            (entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("json"))
            .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect();
    let expected_files: BTreeSet<String> = EXPECTED_SCENES
        .iter()
        .map(|(file, _)| (*file).to_owned())
        .chain(
            REVIEWED_SCENES
                .iter()
                .map(|(file, _, _)| (*file).to_owned()),
        )
        .chain(
            NATIVE_AUXILIARY_SCENES
                .iter()
                .map(|(file, _, _)| (*file).to_owned()),
        )
        .collect();
    if json_files != expected_files {
        return Err(format!(
            "production compiled scene set mismatch: expected {expected_files:?}, found {json_files:?}"
        ));
    }

    let production_dir = out_dir.join("production-scenes");
    fs::create_dir_all(&production_dir)
        .map_err(|error| format!("cannot create embedded scene output: {error}"))?;
    let mut scene_rows = Vec::new();
    for (file_name, expected_sha) in EXPECTED_SCENES {
        let path = scene_dir.join(file_name);
        println!("cargo:rerun-if-changed={}", path.display());
        let (bytes, scene) = read_json(&path)?;
        let actual_sha = sha256(&bytes);
        if actual_sha != expected_sha {
            return Err(format!(
                "compiled production scene SHA-256 mismatch for {file_name}: expected {expected_sha}, got {actual_sha}"
            ));
        }
        let scene_id = file_name.trim_end_matches(".json");
        if scene.get("schemaVersion").and_then(Value::as_u64) != Some(1)
            || string_field(&scene, "sceneId", "scene")? != scene_id
            || string_field(&scene, "worldId", "scene")? != "grey_hive"
        {
            return Err(format!("scene identity/schema mismatch in {file_name}"));
        }
        fs::write(production_dir.join(file_name), &bytes)
            .map_err(|error| format!("cannot embed {file_name}: {error}"))?;
        scene_rows.push((
            scene_id.to_owned(),
            expected_sha.to_owned(),
            file_name.to_owned(),
            "grey_hive".to_owned(),
        ));
    }
    for (file_name, expected_sha, expected_world) in NATIVE_AUXILIARY_SCENES {
        let path = scene_dir.join(file_name);
        println!("cargo:rerun-if-changed={}", path.display());
        let (bytes, scene) = read_json(&path)?;
        let actual_sha = sha256(&bytes);
        if actual_sha != expected_sha {
            return Err(format!(
                "native auxiliary scene SHA-256 mismatch for {file_name}: expected {expected_sha}, got {actual_sha}"
            ));
        }
        let scene_id = file_name.trim_end_matches(".json");
        if scene.get("schemaVersion").and_then(Value::as_u64) != Some(1)
            || string_field(&scene, "sceneId", "native auxiliary scene")? != scene_id
            || string_field(&scene, "worldId", "native auxiliary scene")? != expected_world
        {
            return Err(format!(
                "native auxiliary scene identity/schema mismatch in {file_name}"
            ));
        }
        fs::write(production_dir.join(file_name), &bytes)
            .map_err(|error| format!("cannot embed {file_name}: {error}"))?;
        scene_rows.push((
            scene_id.to_owned(),
            expected_sha.to_owned(),
            file_name.to_owned(),
            expected_world.to_owned(),
        ));
    }
    for (file_name, expected_sha, expected_world) in REVIEWED_SCENES {
        let path = scene_dir.join(file_name);
        println!("cargo:rerun-if-changed={}", path.display());
        let (bytes, scene) = read_json(&path)?;
        let actual_sha = sha256(&bytes);
        if actual_sha != expected_sha {
            return Err(format!(
                "reviewed scene SHA-256 mismatch for {file_name}: expected {expected_sha}, got {actual_sha}"
            ));
        }
        let scene_id = file_name.trim_end_matches(".json");
        if scene.get("schemaVersion").and_then(Value::as_u64) != Some(1)
            || string_field(&scene, "sceneId", "reviewed scene")? != scene_id
            || string_field(&scene, "worldId", "reviewed scene")? != expected_world
        {
            return Err(format!(
                "reviewed scene identity/schema mismatch in {file_name}"
            ));
        }
        if expected_world == "mist_harbor" {
            fs::write(production_dir.join(file_name), &bytes)
                .map_err(|error| format!("cannot embed {file_name}: {error}"))?;
            scene_rows.push((
                scene_id.to_owned(),
                expected_sha.to_owned(),
                file_name.to_owned(),
                expected_world.to_owned(),
            ));
        }
    }
    for (file_name, expected_sha, expected_world) in REVIEWED_SCENES {
        if expected_world != "clockworks" {
            continue;
        }
        let path = scene_dir.join(file_name);
        println!("cargo:rerun-if-changed={}", path.display());
        let (bytes, scene) = read_json(&path)?;
        let actual_sha = sha256(&bytes);
        if actual_sha != expected_sha {
            return Err(format!(
                "reviewed scene SHA-256 mismatch for {file_name}: expected {expected_sha}, got {actual_sha}"
            ));
        }
        let scene_id = file_name.trim_end_matches(".json");
        if scene.get("schemaVersion").and_then(Value::as_u64) != Some(1)
            || string_field(&scene, "sceneId", "reviewed scene")? != scene_id
            || string_field(&scene, "worldId", "reviewed scene")? != "clockworks"
        {
            return Err(format!(
                "reviewed scene identity/schema mismatch in {file_name}"
            ));
        }
        fs::write(production_dir.join(file_name), &bytes)
            .map_err(|error| format!("cannot embed {file_name}: {error}"))?;
        scene_rows.push((
            scene_id.to_owned(),
            expected_sha.to_owned(),
            file_name.to_owned(),
            "clockworks".to_owned(),
        ));
    }

    let mut generated = String::from("// Generated by server-rs/build.rs; do not edit.\n");
    generated.push_str("pub const EMBEDDED_SCENE_IDS: &[&str] = &[\n");
    for (scene_id, _, _, _) in &scene_rows {
        generated.push_str(&format!("    {},\n", rust_string(scene_id)));
    }
    generated.push_str("];\npub const EMBEDDED_SCENE_SHA256: &[(&str, &str)] = &[\n");
    for (scene_id, hash, _, _) in &scene_rows {
        generated.push_str(&format!(
            "    ({}, {}),\n",
            rust_string(scene_id),
            rust_string(hash)
        ));
    }
    generated.push_str("];\npub const EMBEDDED_SCENE_WORLD_IDS: &[&str] = &[\n");
    for (_, _, _, world_id) in &scene_rows {
        generated.push_str(&format!("    {},\n", rust_string(world_id)));
    }
    generated.push_str("];\npub const EMBEDDED_SCENE_JSON: &[&str] = &[\n");
    for (_, _, file_name, _) in &scene_rows {
        generated.push_str(&format!(
            "    include_str!(concat!(env!(\"OUT_DIR\"), \"/production-scenes/{}\")),\n",
            file_name
        ));
    }
    generated.push_str("];\npub const EMBEDDED_ADMITTED_ASSET_IDS: &[&str] = &[\n");
    for asset in &admitted_assets {
        generated.push_str(&format!("    {},\n", rust_string(asset)));
    }
    generated.push_str("];\npub const EMBEDDED_ENTITY_TYPES: &[&str] = &[\n");
    for entity in &entity_types {
        generated.push_str(&format!("    {},\n", rust_string(entity)));
    }
    generated.push_str("];\n");
    fs::write(out_dir.join("production_scene_bundle.rs"), generated)
        .map_err(|error| format!("cannot generate embedded scene bundle: {error}"))?;

    tauri_build::build();
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        panic!("production scene bootstrap build validation failed: {error}");
    }
}

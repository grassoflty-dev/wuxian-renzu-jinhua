//! A separate, opt-in provisional asset lane. This never changes B0 admission.
use super::{is_link_or_reparse, sha256, FileEntry};
use std::{fs, path::{Path, PathBuf}};

pub const FLAG: &str = "RETURN_STATION_CANDIDATE_BUILD";
pub const MANIFEST_SOURCE: &str = "governance/assets/RETURN_STATION_CANDIDATE_MANIFEST.json";
pub const MANIFEST_OUTPUT: &str = "candidate/return-station/manifest.json";
pub const MANIFEST_SHA256: &str = "6f610ece4a5dbebd42bd139da5020927bc1e77f5395fa014047cfd0ed432317f";
const RETIRED_FLOOR_MANIFEST_SHA256: &str = "2581af4c48084603e88d4c0a95954263434c2422c0aeb5369fe1dfa4711090de";
pub const PNG_PATH: &str = "assets/candidate/return-station/rs-floor-provisional-v1.png";
pub const PNG_SHA256: &str = "98c68b4227095292fb40ecc836b1c9be96c25f641ad92e54aac9e88ee5f9c087";
const PNG_SIZE: usize = 2_998_663;
pub const BACKWALL_PATH: &str = "assets/candidate/return-station/rs-backwall-provisional-v1.png";
pub const BACKWALL_SHA256: &str = "5aeb0fcefe0f91a26396db2b99b9de05d30c80080bdf02732c45b9a8862b81c2";
const BACKWALL_SIZE: usize = 3_177_492;
const RESERVED_CANDIDATE_HASHES: [&str; 4] = [MANIFEST_SHA256, RETIRED_FLOOR_MANIFEST_SHA256, PNG_SHA256, BACKWALL_SHA256];

pub fn parse_flag(value: Option<&str>) -> Result<bool, String> {
    match value {
        None | Some("") | Some("0") => Ok(false),
        Some("1") => Ok(true),
        _ => Err("E_RS_CANDIDATE_FLAG".into()),
    }
}

pub fn environment_enabled() -> Result<bool, String> {
    match std::env::var(FLAG) {
        Ok(value) => parse_flag(Some(&value)),
        Err(std::env::VarError::NotPresent) => Ok(false),
        Err(_) => Err("E_RS_CANDIDATE_FLAG_ENCODING".into()),
    }
}

fn candidate_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path == "candidate" || path.starts_with("candidate/") ||
        path == "assets/candidate" || path.starts_with("assets/candidate/") ||
        path == MANIFEST_SOURCE.to_ascii_lowercase()
}

fn read_strict(root: &Path, relative: &str) -> Result<(PathBuf, Vec<u8>), String> {
    let mut path = root.to_path_buf();
    let parts: Vec<_> = relative.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        let meta = fs::symlink_metadata(&path)
            .map_err(|error| format!("E_RS_CANDIDATE_FILE_MISSING:{relative}:{error}"))?;
        if is_link_or_reparse(&path, &meta) ||
            (if index + 1 == parts.len() { !meta.is_file() } else { !meta.is_dir() }) {
            return Err(format!("E_RS_CANDIDATE_FILE_TYPE:{relative}"));
        }
    }
    let actual = fs::canonicalize(&path)
        .map_err(|error| format!("E_RS_CANDIDATE_REALPATH:{relative}:{error}"))?;
    if !actual.starts_with(root) {
        return Err(format!("E_RS_CANDIDATE_ESCAPE:{relative}"));
    }
    let bytes = fs::read(&actual)
        .map_err(|error| format!("E_RS_CANDIDATE_READ:{relative}:{error}"))?;
    Ok((actual, bytes))
}

fn validate_manifest(bytes: &[u8]) -> Result<(), String> {
    if sha256(bytes) != MANIFEST_SHA256 {
        return Err("E_RS_CANDIDATE_MANIFEST_SHA".into());
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| "E_RS_CANDIDATE_MANIFEST_JSON".to_owned())?;
    let expected = serde_json::json!({
        "schemaId": "return-station-candidate-assets/1",
        "schemaVersion": 1,
        "status": "PROVISIONAL_UNVERIFIED",
        "admission": "candidate_only",
        "worldId": "return_station",
        "sceneId": "rs_core_room",
        "sourceCommit": "5697a4c1fa46b2ef8b3a9e70dffe2da65875a3f5",
        "assets": [{
            "assetId": "candidate.return_station.floor.provisional.v1",
            "role": "floor", "path": PNG_PATH, "sha256": PNG_SHA256,
            "width": 1536, "height": 1024, "colorType": 2, "bitDepth": 8,
            "plane": "XZ", "boundsM": {"x": 0, "z": 0, "width": 24, "depth": 16, "y": 0}
        }, {
            "assetId": "candidate.return_station.backwall.provisional.v1",
            "role": "backwall-material", "path": BACKWALL_PATH, "sha256": BACKWALL_SHA256,
            "width": 1536, "height": 1024, "colorType": 2, "bitDepth": 8,
            "maskAssetId": "runtime2d.grey_hive.wall_tiles.v1", "maskAtlasPage": 10,
            "maskAtlasFrame": [38, 1067, 310, 470], "anchor": [0.5, 0.9],
            "displaySizeM": {"width": 1.65, "height": 2.6}
        }]
    });
    if value != expected {
        return Err("E_RS_CANDIDATE_MANIFEST_SCHEMA".into());
    }
    Ok(())
}

fn validate_png(bytes: &[u8], backwall: bool) -> Result<(), String> {
    let (expected_size, expected_sha) = if backwall { (BACKWALL_SIZE, BACKWALL_SHA256) } else { (PNG_SIZE, PNG_SHA256) };
    if bytes.len() != expected_size || sha256(bytes) != expected_sha {
        return Err("E_RS_CANDIDATE_PNG_SHA".into());
    }
    // Hash pins the complete provisional bytes, not just this header.
    if bytes.len() < 33 || bytes[..8] != [137, 80, 78, 71, 13, 10, 26, 10] ||
        bytes[8..12] != [0, 0, 0, 13] || &bytes[12..16] != b"IHDR" ||
        bytes[16..20] != 1536_u32.to_be_bytes() || bytes[20..24] != 1024_u32.to_be_bytes() ||
        bytes[24..29] != [8, 2, 0, 0, 0] {
        return Err("E_RS_CANDIDATE_PNG_FORMAT".into());
    }
    Ok(())
}

pub fn validate_bundle(
    repo_root: &Path,
    files: &[FileEntry],
    enabled: bool,
    watched: &mut Vec<PathBuf>,
) -> Result<Option<String>, String> {
    let mut candidates: Vec<_> = files.iter().filter(|file|
        candidate_path(&file.path) || RESERVED_CANDIDATE_HASHES.contains(&file.sha256.as_str())
    ).cloned().collect();
    candidates.sort_by(|a, b| a.path.cmp(&b.path));
    if !enabled {
        if !candidates.is_empty() { return Err("E_RS_CANDIDATE_DISABLED_OUTPUT".into()); }
        return Ok(None);
    }
    let (manifest_path, manifest_bytes) = read_strict(repo_root, MANIFEST_SOURCE)?;
    let (png_path, png_bytes) = read_strict(repo_root, PNG_PATH)?;
    let (backwall_path, backwall_bytes) = read_strict(repo_root, BACKWALL_PATH)?;
    validate_manifest(&manifest_bytes)?;
    validate_png(&png_bytes, false)?;
    validate_png(&backwall_bytes, true)?;
    let mut expected = vec![
        FileEntry { path: MANIFEST_OUTPUT.into(), size_bytes: manifest_bytes.len() as u64, sha256: MANIFEST_SHA256.into() },
        FileEntry { path: PNG_PATH.into(), size_bytes: png_bytes.len() as u64, sha256: PNG_SHA256.into() },
        FileEntry { path: BACKWALL_PATH.into(), size_bytes: backwall_bytes.len() as u64, sha256: BACKWALL_SHA256.into() },
    ];
    expected.sort_by(|a, b| a.path.cmp(&b.path));
    if candidates != expected { return Err("E_RS_CANDIDATE_OUTPUT_CLOSURE".into()); }
    watched.extend([manifest_path, png_path, backwall_path]);
    Ok(Some(MANIFEST_SHA256.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()).unwrap()
    }
    fn candidate_files() -> Vec<FileEntry> {
        vec![
            FileEntry { path: MANIFEST_OUTPUT.into(), size_bytes: fs::metadata(root().join(MANIFEST_SOURCE)).unwrap().len(), sha256: MANIFEST_SHA256.into() },
            FileEntry { path: PNG_PATH.into(), size_bytes: PNG_SIZE as u64, sha256: PNG_SHA256.into() },
            FileEntry { path: BACKWALL_PATH.into(), size_bytes: BACKWALL_SIZE as u64, sha256: BACKWALL_SHA256.into() },
        ]
    }

    #[test]
    fn flag_is_explicit_and_fail_closed() {
        for value in [None, Some(""), Some("0")] { assert_eq!(parse_flag(value), Ok(false)); }
        assert_eq!(parse_flag(Some("1")), Ok(true));
        for value in ["true", "yes", "01", " 1", "2"] { assert!(parse_flag(Some(value)).is_err()); }
    }

    #[test]
    fn normal_mode_rejects_candidate_namespaces_and_renamed_pinned_bytes() {
        assert_eq!(validate_bundle(&root(), &[], false, &mut vec![]).unwrap(), None);
        for path in [PNG_PATH, BACKWALL_PATH, MANIFEST_OUTPUT, MANIFEST_SOURCE, "ASSETS/CANDIDATE/extra.json", "candidate/extra.txt", "assets/renamed.png"] {
            let file = FileEntry { path: path.into(), size_bytes: 1, sha256: PNG_SHA256.into() };
            assert!(validate_bundle(&root(), &[file], false, &mut vec![]).unwrap_err().contains("DISABLED_OUTPUT"));
        }
    }

    #[test]
    fn candidate_mode_pins_sources_and_exact_three_output_records() {
        let files = candidate_files();
        assert_eq!(validate_bundle(&root(), &files, true, &mut vec![]).unwrap(), Some(MANIFEST_SHA256.into()));
        assert!(validate_bundle(&root(), &[], true, &mut vec![]).unwrap_err().contains("OUTPUT_CLOSURE"));
        assert!(validate_bundle(&root(), &files[..2], true, &mut vec![]).is_err());
        let retired = FileEntry { path: "misc/retired-floor-manifest.json".into(), size_bytes: 773, sha256: RETIRED_FLOOR_MANIFEST_SHA256.into() };
        assert!(validate_bundle(&root(), &[retired.clone()], false, &mut vec![]).is_err());
        let mut with_retired = files.clone(); with_retired.push(retired);
        assert!(validate_bundle(&root(), &with_retired, true, &mut vec![]).is_err());
        let mut bad = files.clone(); bad[0].sha256 = "0".repeat(64);
        assert!(validate_bundle(&root(), &bad, true, &mut vec![]).is_err());
        let mut extra = files.clone(); extra.push(FileEntry { path: "candidate/return-station/extra.json".into(), size_bytes: 0, sha256: sha256(b"") });
        assert!(validate_bundle(&root(), &extra, true, &mut vec![]).is_err());
        for (index, path) in [(0, "misc/rs-copy.json"), (1, "misc/rs-copy.bin"), (2, "misc/backwall-copy.bin")] {
            let mut renamed = files[index].clone(); renamed.path = path.into();
            assert!(validate_bundle(&root(), &[renamed.clone()], false, &mut vec![]).is_err());
            let mut duplicated = files.clone(); duplicated.push(renamed);
            assert!(validate_bundle(&root(), &duplicated, true, &mut vec![]).is_err());
        }
    }

    #[test]
    fn mutated_manifest_or_png_is_never_admitted() {
        let mut manifest = fs::read(root().join(MANIFEST_SOURCE)).unwrap();
        validate_manifest(&manifest).unwrap(); manifest.push(b' ');
        assert!(validate_manifest(&manifest).unwrap_err().contains("MANIFEST_SHA"));
        let mut png = fs::read(root().join(PNG_PATH)).unwrap(); validate_png(&png, false).unwrap();
        png[20] ^= 1;
        assert!(validate_png(&png, false).unwrap_err().contains("PNG_SHA"));
        let mut wall = fs::read(root().join(BACKWALL_PATH)).unwrap(); validate_png(&wall, true).unwrap();
        assert!(validate_png(&wall, false).is_err());
        wall[20] ^= 1;
        assert!(validate_png(&wall, true).unwrap_err().contains("PNG_SHA"));
    }
}

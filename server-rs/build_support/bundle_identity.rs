use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub const SIDECAR_FILE_NAME: &str = "bundle-identity.json";
const SIDECAR_SCHEMA_ID: &str = "native-web-bundle-identity/1";
const MAX_SIDECAR_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildIdentity {
    pub schema_version: u32,
    pub git_sha: String,
    pub app_version: String,
    pub content_version: String,
    pub save_v6_schema_version: u32,
    pub runtime_asset_manifest_sha256: String,
    pub scene_definition_manifest_sha256: String,
    pub built_at_utc: String,
    pub has_tracked_diff: bool,
    pub has_untracked_files: bool,
    pub source_tree_dirty: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sidecar {
    pub schema_id: String,
    pub schema_version: u32,
    pub build_identity: BuildIdentity,
    pub entry: FileEntry,
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedBundleIdentity {
    pub build_identity: BuildIdentity,
    pub sidecar_sha256: String,
    pub entry: FileEntry,
    pub file_count: u64,
}

#[derive(Clone, Debug, Default)]
struct SourceIdentity {
    git_sha: String,
    app_version: String,
    content_version: String,
    save_v6_schema_version: u32,
    runtime_asset_manifest_sha256: String,
    scene_definition_manifest_sha256: String,
    has_tracked_diff: bool,
    has_untracked_files: bool,
    source_tree_dirty: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneRow {
    world_id: String,
    scene_id: String,
    path: String,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneManifest {
    schema_version: u32,
    scenes: Vec<SceneRow>,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_version(value: &str) -> bool {
    let core = value.split(['-', '+']).next().unwrap_or_default();
    let mut parts = core.split('.');
    let numeric = (0..3).all(|_| {
        parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
    }) && parts.next().is_none();
    numeric
        && !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".+-".contains(&byte))
        && !value.starts_with(['-', '+'])
        && !value.ends_with(['-', '+', '.'])
}

fn is_content_version(value: &str) -> bool {
    (1..=96).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

fn is_timestamp(value: &str) -> bool {
    let Some(body) = value.strip_suffix('Z') else {
        return false;
    };
    let Some((date, time)) = body.split_once('T') else {
        return false;
    };
    let date_parts: Vec<_> = date.split('-').collect();
    let time_parts: Vec<_> = time.split(':').collect();
    if date_parts.len() != 3
        || time_parts.len() != 3
        || date_parts[0].len() != 4
        || date_parts[1].len() != 2
        || date_parts[2].len() != 2
        || time_parts[0].len() != 2
        || time_parts[1].len() != 2
    {
        return false;
    }
    let seconds = time_parts[2]
        .split_once('.')
        .map_or(time_parts[2], |(whole, fraction)| {
            if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
                ""
            } else {
                whole
            }
        });
    seconds.len() == 2
        && date_parts
            .iter()
            .chain(time_parts.iter().take(2))
            .all(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
        && seconds.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_safe_path(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains(['\\', ':', '\0'])
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/._-".contains(&byte))
    {
        return false;
    }
    value
        .split('/')
        .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn is_link_or_reparse(path: &Path, metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        let _ = path;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

fn canonical_dist_root(
    path: &Path,
    repo_root: &Path,
    require_repo_frontend_dist: bool,
) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| format!("E_BUNDLE_IDENTITY_DIST_CWD:{error}"))?
            .join(path)
    };
    if absolute
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err("E_BUNDLE_IDENTITY_DIST_PATH_COMPONENT".into());
    }
    let mut current = PathBuf::new();
    for component in absolute.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if is_link_or_reparse(&current, &metadata) => {
                return Err(format!(
                    "E_BUNDLE_IDENTITY_DIST_PARENT_LINK:{}",
                    current.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "E_BUNDLE_IDENTITY_DIST_PARENT_LSTAT:{}:{error}",
                    current.display()
                ))
            }
        }
    }
    let metadata = fs::symlink_metadata(&absolute)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_DIST_MISSING:{error}"))?;
    if !metadata.is_dir() || is_link_or_reparse(&absolute, &metadata) {
        return Err(format!(
            "E_BUNDLE_IDENTITY_DIST_TYPE:{}",
            absolute.display()
        ));
    }
    let canonical = fs::canonicalize(&absolute)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_DIST_REALPATH:{error}"))?;
    if require_repo_frontend_dist && canonical != repo_root.join("apps/web/dist") {
        return Err(format!(
            "E_BUNDLE_IDENTITY_DIST_ESCAPE:{}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn relative_bundle_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "E_BUNDLE_IDENTITY_PATH_ESCAPE".to_owned())?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| "E_BUNDLE_IDENTITY_PATH_UTF8".to_owned())?;
                parts.push(part.to_owned());
            }
            _ => return Err("E_BUNDLE_IDENTITY_PATH_COMPONENT".to_owned()),
        }
    }
    let value = parts.join("/");
    if !is_safe_path(&value) {
        return Err(format!("E_BUNDLE_IDENTITY_PATH:{value}"));
    }
    Ok(value)
}

fn walk_payload(
    root: &Path,
    current: &Path,
    files: &mut Vec<FileEntry>,
    watched: &mut Vec<PathBuf>,
    directories: &mut Vec<PathBuf>,
) -> Result<(), String> {
    directories.push(current.to_path_buf());
    for item in fs::read_dir(current)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_READ_DIR:{}:{error}", current.display()))?
    {
        let item = item.map_err(|error| format!("E_BUNDLE_IDENTITY_READ_ENTRY:{error}"))?;
        let path = item.path();
        let relative = relative_bundle_path(root, &path)?;
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("E_BUNDLE_IDENTITY_LSTAT:{relative}:{error}"))?;
        if is_link_or_reparse(&path, &metadata) {
            return Err(format!("E_BUNDLE_IDENTITY_LINK:{relative}"));
        }
        if metadata.is_dir() {
            walk_payload(root, &path, files, watched, directories)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("E_BUNDLE_IDENTITY_FILE_TYPE:{relative}"));
        }
        watched.push(path.clone());
        if relative == SIDECAR_FILE_NAME {
            continue;
        }
        let real = fs::canonicalize(&path)
            .map_err(|error| format!("E_BUNDLE_IDENTITY_REALPATH:{relative}:{error}"))?;
        if !real.starts_with(root) {
            return Err(format!("E_BUNDLE_IDENTITY_LINK_ESCAPE:{relative}"));
        }
        let bytes = fs::read(&real)
            .map_err(|error| format!("E_BUNDLE_IDENTITY_READ_FILE:{relative}:{error}"))?;
        files.push(FileEntry {
            path: relative,
            size_bytes: bytes.len() as u64,
            sha256: sha256(&bytes),
        });
    }
    Ok(())
}

fn git_output(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|error| format!("E_BUNDLE_IDENTITY_GIT_START:{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "E_BUNDLE_IDENTITY_GIT_FAILED:{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text =
        String::from_utf8(output.stdout).map_err(|_| "E_BUNDLE_IDENTITY_GIT_UTF8".to_owned())?;
    Ok(text.trim().to_owned())
}

fn hash_required(repo_root: &Path, relative: &str) -> Result<(PathBuf, Vec<u8>, String), String> {
    let path = repo_root.join(relative);
    let bytes = fs::read(&path)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SOURCE_MISSING:{relative}:{error}"))?;
    let hash = sha256(&bytes);
    Ok((path, bytes, hash))
}

fn parse_quoted_constant(source: &str, declaration: &str, label: &str) -> Result<String, String> {
    let tail = source
        .split_once(declaration)
        .map(|(_, tail)| tail)
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?;
    let tail = tail.trim_start();
    let tail = tail
        .strip_prefix(':')
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?;
    let tail = tail
        .split_once('=')
        .map(|(_, value)| value)
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?
        .trim_start();
    let tail = tail
        .strip_prefix('"')
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?;
    let value = tail
        .split_once('"')
        .map(|(value, _)| value)
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?;
    Ok(value.to_owned())
}

fn parse_u32_constant(source: &str, declaration: &str, label: &str) -> Result<u32, String> {
    let tail = source
        .split_once(declaration)
        .map(|(_, tail)| tail)
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?;
    let tail = tail.trim_start();
    let tail = tail
        .strip_prefix(':')
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?;
    let tail = tail
        .split_once('=')
        .map(|(_, value)| value)
        .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))?
        .trim_start();
    let digits: String = tail
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    digits
        .parse()
        .map_err(|_| format!("E_BUNDLE_IDENTITY_SOURCE_CONSTANT:{label}"))
}

fn scene_manifest_sha(repo_root: &Path, watched: &mut Vec<PathBuf>) -> Result<String, String> {
    let scene_root = repo_root.join("content/scenes/compiled");
    let metadata = match fs::symlink_metadata(&scene_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let bytes = serde_json::to_vec_pretty(&SceneManifest {
                schema_version: 1,
                scenes: Vec::new(),
            })
            .map_err(|error| error.to_string())?;
            return Ok(sha256(&[bytes, b"\n".to_vec()].concat()));
        }
        Err(error) => return Err(format!("E_BUNDLE_IDENTITY_SCENE_SOURCE:{error}")),
    };
    if !metadata.is_dir() || is_link_or_reparse(&scene_root, &metadata) {
        return Err("E_BUNDLE_IDENTITY_SCENE_SOURCE_TYPE".into());
    }
    watched.push(scene_root.clone());
    let real_root = fs::canonicalize(&scene_root)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_SOURCE_REALPATH:{error}"))?;
    let mut rows = Vec::new();
    let mut identities = BTreeSet::new();
    for entry in fs::read_dir(&real_root)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_SOURCE_READ:{error}"))?
    {
        let entry =
            entry.map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_SOURCE_READ:{error}"))?;
        let path = entry.path();
        let relative = relative_bundle_path(&real_root, &path)?;
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_LSTAT:{relative}:{error}"))?;
        if is_link_or_reparse(&path, &metadata) {
            return Err(format!("E_BUNDLE_IDENTITY_SCENE_LINK:{relative}"));
        }
        if !metadata.is_file() || !relative.ends_with(".json") {
            return Err(format!("E_BUNDLE_IDENTITY_SCENE_FILE:{relative}"));
        }
        watched.push(path.clone());
        let bytes = fs::read(&path)
            .map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_READ:{relative}:{error}"))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_JSON:{relative}:{error}"))?;
        let scene_id = value
            .get("sceneId")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SCENE_ID:{relative}"))?;
        let world_id = value
            .get("worldId")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("E_BUNDLE_IDENTITY_SCENE_WORLD:{relative}"))?;
        if value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
            || entry.file_name().to_string_lossy() != format!("{scene_id}.json")
            || !["grey_hive", "mist_harbor", "clockworks", "return_station"].contains(&world_id)
            || scene_id.is_empty()
            || scene_id.len() > 128
            || !scene_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_.:-".contains(&byte))
            || !identities.insert(format!("{world_id}\0{scene_id}"))
        {
            return Err(format!("E_BUNDLE_IDENTITY_SCENE_SCHEMA:{relative}"));
        }
        rows.push(SceneRow {
            world_id: world_id.to_owned(),
            scene_id: scene_id.to_owned(),
            path: format!("scene-definitions/compiled/{world_id}/{scene_id}.json"),
            sha256: sha256(&bytes),
        });
    }
    rows.sort_by(|left, right| {
        left.world_id
            .cmp(&right.world_id)
            .then_with(|| left.scene_id.cmp(&right.scene_id))
    });
    let manifest = SceneManifest {
        schema_version: 1,
        scenes: rows,
    };
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SCENE_SERIALIZE:{error}"))?;
    Ok(sha256(&[bytes, b"\n".to_vec()].concat()))
}

fn source_identity(
    repo_root: &Path,
    cargo_version: &str,
    watched: &mut Vec<PathBuf>,
) -> Result<SourceIdentity, String> {
    let git_sha = git_output(repo_root, &["rev-parse", "HEAD"])?;
    if !is_sha(&git_sha, 40) {
        return Err("E_BUNDLE_IDENTITY_GIT_SHA".into());
    }
    let status = git_output(
        repo_root,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    let has_tracked_diff = status
        .lines()
        .any(|line| !line.is_empty() && !line.starts_with("??"));
    let has_untracked_files = status.lines().any(|line| line.starts_with("??"));

    let (tauri_path, tauri_bytes, _) = hash_required(repo_root, "server-rs/tauri.conf.json")?;
    let tauri: serde_json::Value = serde_json::from_slice(&tauri_bytes)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_TAURI_JSON:{error}"))?;
    let app_version = tauri
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "E_BUNDLE_IDENTITY_TAURI_VERSION".to_owned())?
        .to_owned();
    if !is_version(&app_version) || app_version != cargo_version {
        return Err("E_BUNDLE_IDENTITY_APP_VERSION_MISMATCH".into());
    }
    watched.push(tauri_path);

    let (cargo_path, _, _) = hash_required(repo_root, "server-rs/Cargo.toml")?;
    watched.push(cargo_path);
    let (web_package_path, web_package_bytes, _) =
        hash_required(repo_root, "apps/web/package.json")?;
    let web_package: serde_json::Value = serde_json::from_slice(&web_package_bytes)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_WEB_PACKAGE_JSON:{error}"))?;
    if let Some(web_version) = web_package
        .get("version")
        .and_then(serde_json::Value::as_str)
    {
        if web_version != app_version {
            return Err("E_BUNDLE_IDENTITY_WEB_APP_VERSION_MISMATCH".into());
        }
    }
    watched.push(web_package_path);

    let (save_v5_path, save_v5, _) = hash_required(repo_root, "server-rs/src/save_v5.rs")?;
    let content_version = parse_quoted_constant(
        std::str::from_utf8(&save_v5).map_err(|_| "E_BUNDLE_IDENTITY_SAVE_V5_UTF8")?,
        "SAVE_V5_CONTENT_VERSION",
        "contentVersion",
    )?;
    if !is_content_version(&content_version) {
        return Err("E_BUNDLE_IDENTITY_CONTENT_VERSION".into());
    }
    watched.push(save_v5_path);
    let (save_v6_path, save_v6, _) = hash_required(repo_root, "server-rs/src/save_v6.rs")?;
    let save_v6_schema_version = parse_u32_constant(
        std::str::from_utf8(&save_v6).map_err(|_| "E_BUNDLE_IDENTITY_SAVE_V6_UTF8")?,
        "SAVE_V6_SCHEMA_VERSION",
        "saveV6SchemaVersion",
    )?;
    if save_v6_schema_version == 0 {
        return Err("E_BUNDLE_IDENTITY_SAVE_V6_SCHEMA".into());
    }
    watched.push(save_v6_path);

    let (asset_path, _, runtime_asset_manifest_sha256) =
        hash_required(repo_root, "governance/assets/RUNTIME_ASSET_MANIFEST.json")?;
    watched.push(asset_path);
    let scene_definition_manifest_sha256 = scene_manifest_sha(repo_root, watched)?;
    let source_tree_dirty = has_tracked_diff || has_untracked_files;
    Ok(SourceIdentity {
        git_sha,
        app_version,
        content_version,
        save_v6_schema_version,
        runtime_asset_manifest_sha256,
        scene_definition_manifest_sha256,
        has_tracked_diff,
        has_untracked_files,
        source_tree_dirty,
    })
}

fn check_identity(identity: &BuildIdentity, current: &SourceIdentity) -> Result<(), String> {
    if identity.schema_version != 1
        || !is_sha(&identity.git_sha, 40)
        || !is_version(&identity.app_version)
        || !is_content_version(&identity.content_version)
        || identity.save_v6_schema_version == 0
        || !is_sha(&identity.runtime_asset_manifest_sha256, 64)
        || !is_sha(&identity.scene_definition_manifest_sha256, 64)
        || !is_timestamp(&identity.built_at_utc)
        || identity.source_tree_dirty != (identity.has_tracked_diff || identity.has_untracked_files)
    {
        return Err("E_BUNDLE_IDENTITY_BUILD_IDENTITY_SCHEMA".into());
    }
    if identity.git_sha != current.git_sha {
        return Err("E_BUNDLE_IDENTITY_GIT_STALE".into());
    }
    if identity.app_version != current.app_version {
        return Err("E_BUNDLE_IDENTITY_APP_VERSION_STALE".into());
    }
    if identity.content_version != current.content_version {
        return Err("E_BUNDLE_IDENTITY_CONTENT_VERSION_STALE".into());
    }
    if identity.save_v6_schema_version != current.save_v6_schema_version {
        return Err("E_BUNDLE_IDENTITY_SAVE_VERSION_STALE".into());
    }
    if identity.runtime_asset_manifest_sha256 != current.runtime_asset_manifest_sha256 {
        return Err("E_BUNDLE_IDENTITY_ASSET_MANIFEST_STALE".into());
    }
    if identity.scene_definition_manifest_sha256 != current.scene_definition_manifest_sha256 {
        return Err("E_BUNDLE_IDENTITY_SCENE_MANIFEST_STALE".into());
    }
    if identity.has_tracked_diff != current.has_tracked_diff
        || identity.has_untracked_files != current.has_untracked_files
        || identity.source_tree_dirty != current.source_tree_dirty
    {
        return Err("E_BUNDLE_IDENTITY_SOURCE_DIRTY_STALE".into());
    }
    Ok(())
}

fn check_profile_cleanliness(
    profile: &str,
    identity: &BuildIdentity,
    current: &SourceIdentity,
) -> Result<(), String> {
    match profile {
        "debug" => Ok(()),
        "release" => {
            let sidecar_dirty = identity.has_tracked_diff
                || identity.has_untracked_files
                || identity.source_tree_dirty;
            let current_dirty = current.has_tracked_diff
                || current.has_untracked_files
                || current.source_tree_dirty;
            if sidecar_dirty || current_dirty {
                return Err("E_BUNDLE_IDENTITY_RELEASE_DIRTY".into());
            }
            Ok(())
        }
        _ => Err(format!("E_BUNDLE_IDENTITY_PROFILE:{profile}")),
    }
}

fn validate_file_entries(files: &[FileEntry]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    #[cfg(windows)]
    let mut casefolded = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for file in files {
        if !is_safe_path(&file.path) || file.path == SIDECAR_FILE_NAME || !is_sha(&file.sha256, 64)
        {
            return Err(format!("E_BUNDLE_IDENTITY_FILE_ENTRY:{}", file.path));
        }
        if !seen.insert(file.path.as_str()) {
            return Err(format!("E_BUNDLE_IDENTITY_DUPLICATE_PATH:{}", file.path));
        }
        #[cfg(windows)]
        if !casefolded.insert(file.path.to_ascii_lowercase()) {
            return Err(format!("E_BUNDLE_IDENTITY_CASE_COLLISION:{}", file.path));
        }
        if previous.is_some_and(|last| last >= file.path.as_str()) {
            return Err(format!("E_BUNDLE_IDENTITY_FILE_ORDER:{}", file.path));
        }
        previous = Some(&file.path);
    }
    Ok(())
}

pub fn serialize_sidecar(sidecar: &Sidecar) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(sidecar)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SERIALIZE:{error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_dist_inner(
    dist_root: &Path,
    repo_root: &Path,
    cargo_version: &str,
    require_repo_frontend_dist: bool,
    profile: Option<&str>,
) -> Result<ValidatedBundleIdentity, String> {
    let repo_root = fs::canonicalize(repo_root)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_REPO_ROOT:{error}"))?;
    println!("cargo:rerun-if-changed={}", dist_root.display());
    println!(
        "cargo:rerun-if-changed={}",
        dist_root.join(SIDECAR_FILE_NAME).display()
    );
    for relative in [
        "apps/web/package.json",
        "apps/web/package-lock.json",
        "apps/web/vite.config.ts",
        "apps/web/scripts/build-identity.mjs",
        "apps/web/scripts/bundle-identity.mjs",
        "apps/web/scripts/scene-definition-bundle.mjs",
        "server-rs/Cargo.toml",
        "server-rs/tauri.conf.json",
        "server-rs/src/save_v5.rs",
        "server-rs/src/save_v6.rs",
        "governance/assets/RUNTIME_ASSET_MANIFEST.json",
        "content/scenes/compiled",
    ] {
        println!(
            "cargo:rerun-if-changed={}",
            repo_root.join(relative).display()
        );
    }
    let root = canonical_dist_root(dist_root, &repo_root, require_repo_frontend_dist)?;
    let sidecar_path = root.join(SIDECAR_FILE_NAME);
    let sidecar_metadata = fs::symlink_metadata(&sidecar_path)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SIDECAR_MISSING:{error}"))?;
    if !sidecar_metadata.is_file() || is_link_or_reparse(&sidecar_path, &sidecar_metadata) {
        return Err("E_BUNDLE_IDENTITY_SIDECAR_TYPE".into());
    }
    if sidecar_metadata.len() > MAX_SIDECAR_BYTES {
        return Err("E_BUNDLE_IDENTITY_SIDECAR_SIZE".into());
    }
    let sidecar_bytes = fs::read(&sidecar_path)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SIDECAR_READ:{error}"))?;
    let sidecar: Sidecar = serde_json::from_slice(&sidecar_bytes)
        .map_err(|error| format!("E_BUNDLE_IDENTITY_SIDECAR_JSON:{error}"))?;
    if sidecar.schema_id != SIDECAR_SCHEMA_ID || sidecar.schema_version != 1 {
        return Err("E_BUNDLE_IDENTITY_SCHEMA".into());
    }
    if serialize_sidecar(&sidecar)? != sidecar_bytes {
        return Err("E_BUNDLE_IDENTITY_NONCANONICAL".into());
    }
    validate_file_entries(&sidecar.files)?;
    if !is_safe_path(&sidecar.entry.path)
        || !sidecar.entry.path.to_ascii_lowercase().ends_with(".js")
    {
        return Err("E_BUNDLE_IDENTITY_ENTRY_PATH".into());
    }
    let matching_entries: Vec<_> = sidecar
        .files
        .iter()
        .filter(|file| file.path == sidecar.entry.path)
        .collect();
    if matching_entries.len() != 1 || *matching_entries[0] != sidecar.entry {
        return Err("E_BUNDLE_IDENTITY_ENTRY_RECORD".into());
    }

    let mut watched = vec![sidecar_path.clone()];
    let current = source_identity(&repo_root, cargo_version, &mut watched)?;
    check_identity(&sidecar.build_identity, &current)?;

    let mut actual_files = Vec::new();
    let mut payload_watched = Vec::new();
    let mut directories = Vec::new();
    walk_payload(
        &root,
        &root,
        &mut actual_files,
        &mut payload_watched,
        &mut directories,
    )?;
    actual_files.sort_by(|left, right| left.path.cmp(&right.path));
    if actual_files != sidecar.files {
        return Err("E_BUNDLE_IDENTITY_PAYLOAD_CLOSURE".into());
    }
    if actual_files
        .iter()
        .filter(|file| file.path == sidecar.entry.path)
        .count()
        != 1
    {
        return Err("E_BUNDLE_IDENTITY_ENTRY_MISSING".into());
    }
    if let Some(profile) = profile {
        check_profile_cleanliness(profile, &sidecar.build_identity, &current)?;
    }

    watched.extend(payload_watched);
    watched.extend(directories);
    watched.extend([
        repo_root.join("apps/web/dist"),
        repo_root.join("apps/web/package.json"),
        repo_root.join("apps/web/package-lock.json"),
        repo_root.join("apps/web/vite.config.ts"),
        repo_root.join("apps/web/scripts/build-identity.mjs"),
        repo_root.join("apps/web/scripts/bundle-identity.mjs"),
        repo_root.join("apps/web/scripts/scene-definition-bundle.mjs"),
        repo_root.join("server-rs/Cargo.toml"),
        repo_root.join("server-rs/tauri.conf.json"),
        repo_root.join("server-rs/src/save_v5.rs"),
        repo_root.join("server-rs/src/save_v6.rs"),
        repo_root.join("governance/assets/RUNTIME_ASSET_MANIFEST.json"),
        repo_root.join("content/scenes/compiled"),
    ]);
    for path in watched {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    for git_item in ["HEAD", "index"] {
        if let Ok(relative) = git_output(&repo_root, &["rev-parse", "--git-path", git_item]) {
            let path = if Path::new(&relative).is_absolute() {
                PathBuf::from(relative)
            } else {
                repo_root.join(relative)
            };
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    Ok(ValidatedBundleIdentity {
        build_identity: sidecar.build_identity,
        sidecar_sha256: sha256(&sidecar_bytes),
        entry: sidecar.entry,
        file_count: actual_files.len() as u64,
    })
}

#[cfg(test)]
pub fn validate_dist(
    dist_root: &Path,
    repo_root: &Path,
    cargo_version: &str,
) -> Result<ValidatedBundleIdentity, String> {
    validate_dist_inner(dist_root, repo_root, cargo_version, false, None)
}

#[cfg(test)]
pub fn validate_dist_with_profile(
    dist_root: &Path,
    repo_root: &Path,
    cargo_version: &str,
    profile: &str,
) -> Result<ValidatedBundleIdentity, String> {
    validate_dist_inner(dist_root, repo_root, cargo_version, false, Some(profile))
}

pub fn validate_repo_frontend_dist(
    dist_root: &Path,
    repo_root: &Path,
    cargo_version: &str,
) -> Result<ValidatedBundleIdentity, String> {
    let profile = std::env::var("PROFILE")
        .map_err(|_| "E_BUNDLE_IDENTITY_PROFILE_MISSING".to_owned())?;
    validate_dist_inner(dist_root, repo_root, cargo_version, true, Some(&profile))
}

#[cfg(test)]
pub fn current_identity(repo_root: &Path, cargo_version: &str) -> Result<BuildIdentity, String> {
    let mut watched = Vec::new();
    let current = source_identity(repo_root, cargo_version, &mut watched)?;
    Ok(BuildIdentity {
        schema_version: 1,
        git_sha: current.git_sha,
        app_version: current.app_version,
        content_version: current.content_version,
        save_v6_schema_version: current.save_v6_schema_version,
        runtime_asset_manifest_sha256: current.runtime_asset_manifest_sha256,
        scene_definition_manifest_sha256: current.scene_definition_manifest_sha256,
        built_at_utc: "2026-01-01T00:00:00.000Z".into(),
        has_tracked_diff: current.has_tracked_diff,
        has_untracked_files: current.has_untracked_files,
        source_tree_dirty: current.source_tree_dirty,
    })
}

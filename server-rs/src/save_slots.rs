//! Isolated, validated save slots. Legacy single-file saves are surfaced for
//! explicit import but are never written by the slot store.
use crate::{
    save_v3,
    save_v5::{self, SaveV5},
    world_v3::Vec3,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const RESERVED_SLOT_ID: &str = "legacy-save-v3";
const LEGACY_SLOT_FILE_NAME: &str = "slot-v4.json";
const SLOT_FILE_NAME: &str = "slot-v5.json";
const SLOT_V6_FILE_NAME: &str = "slot-v6.json";
const SLOT_V5_SCHEMA_VERSION: u32 = 2;
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SlotDocumentV6 {
    schema_version: u32,
    slot_id: String,
    display_name: String,
    updated_at_ms: u64,
    save: crate::save_v6::SaveV6,
}

// Preserve the nested raw profile until outer shape, version, identity, and name pass.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawSlotDocumentV6 {
    schema_version: u32,
    slot_id: String,
    display_name: String,
    updated_at_ms: u64,
    save: serde_json::Value,
}

fn validate_v6_envelope(
    version: u32,
    id: &str,
    expected_id: &str,
    name: &str,
) -> Result<(), String> {
    if version != 3 {
        return Err("E_SLOT_VERSION_UNSUPPORTED".into());
    }
    validate_slot_id(id)?;
    if id != expected_id {
        return Err("E_SLOT_ID_MISMATCH".into());
    }
    if name.trim().is_empty() || name.chars().count() > 48 {
        return Err("E_SLOT_NAME_INVALID".into());
    }
    Ok(())
}

fn decode_slot_v6(
    bytes: &[u8],
    slot_id: &str,
) -> Result<(SlotDocumentV6, crate::save_v6::Profile), String> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| "E_SLOT_CORRUPT")?;
    let raw: RawSlotDocumentV6 =
        serde_json::from_value(value.clone()).map_err(|_| "E_SLOT_CORRUPT")?;
    if !save_v5::same_schema_shape(
        &value,
        &serde_json::to_value(&raw).map_err(|_| "E_SLOT_CORRUPT")?,
    ) {
        return Err("E_SLOT_SCHEMA_INVALID".into());
    }
    validate_v6_envelope(raw.schema_version, &raw.slot_id, slot_id, &raw.display_name)?;
    let (save, profile) = crate::save_v6::decode_value(raw.save)?;
    Ok((
        SlotDocumentV6 {
            schema_version: raw.schema_version,
            slot_id: raw.slot_id,
            display_name: raw.display_name,
            updated_at_ms: raw.updated_at_ms,
            save,
        },
        profile,
    ))
}

pub fn create_slot_v6(
    root: &Path,
    slot_id: &str,
    display_name: &str,
    save: &crate::save_v6::SaveV6,
) -> Result<(), String> {
    validate_writable_slot_id(slot_id)?;
    let display_name = display_name.trim();
    validate_v6_envelope(3, slot_id, slot_id, display_name)?;
    save.validate()?;
    let dir = slots_root(root).join(slot_id);
    fs::create_dir_all(slots_root(root)).map_err(|e| format!("E_SLOT_DIR: {e}"))?;
    fs::create_dir(&dir).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            "E_SLOT_EXISTS".into()
        } else {
            format!("E_SLOT_DIR: {e}")
        }
    })?;
    let result = write_slot_v6(
        &dir.join(SLOT_V6_FILE_NAME),
        &SlotDocumentV6 {
            schema_version: 3,
            slot_id: slot_id.into(),
            display_name: display_name.into(),
            updated_at_ms: now_ms(),
            save: save.clone(),
        },
        true,
    );
    if result.is_err() {
        let _ = fs::remove_dir(&dir);
    }
    result
}

pub fn overwrite_slot_v6(
    root: &Path,
    slot_id: &str,
    display_name: &str,
    save: &crate::save_v6::SaveV6,
) -> Result<(), String> {
    validate_writable_slot_id(slot_id)?;
    let display_name = display_name.trim();
    validate_v6_envelope(3, slot_id, slot_id, display_name)?;
    save.validate()?;
    let target = slots_root(root).join(slot_id).join(SLOT_V6_FILE_NAME);
    let new = !crate::save_v6::path_present(&target);
    crate::save_v6::recovery::require_no_recovery(&target, "E_SLOT")?;
    if new {
        // V5 slots are advertised as writable. Validate that exact source, but
        // persist only the current game as V6; never pre-migrate or replace V5.
        // A present newer path always wins, including corrupt/unreadable files.
        let source = read_document(&slot_path(root, slot_id)?)?;
        if source.slot_id != slot_id {
            return Err("E_SLOT_ID_MISMATCH".into());
        }
    }
    write_slot_v6(
        &target,
        &SlotDocumentV6 {
            schema_version: 3,
            slot_id: slot_id.into(),
            display_name: display_name.into(),
            updated_at_ms: now_ms(),
            save: save.clone(),
        },
        new,
    )
}

/// Rest/Evolution/Scanner must honor the selected slot's read-only intent,
/// even where a writable parent directory would permit replacing its file.
pub(crate) fn require_writable_current_source(root: &Path, slot_id: &str) -> Result<(), String> {
    let v6 = slot_v6_path(root, slot_id)?;
    let source = if crate::save_v6::path_present(&v6) { v6 } else { slot_path(root, slot_id)? };
    let metadata = fs::metadata(&source).map_err(|error| format!("E_SLOT_READ: {error}"))?;
    if !metadata.is_file() { return Err("E_SLOT_NOT_REGULAR".into()); }
    if metadata.permissions().readonly() { return Err("E_SLOT_READ_ONLY".into()); }
    Ok(())
}

fn write_slot_v6(target: &Path, doc: &SlotDocumentV6, new: bool) -> Result<(), String> {
    crate::save_v6::recovery::require_no_recovery(target, "E_SLOT")?;
    write_slot_v6_inner(target, doc, new, None)
}

fn write_slot_v6_inner(target: &Path, doc: &SlotDocumentV6, new: bool, backup: Option<&crate::save_v6::recovery::Backup>) -> Result<(), String> {
    validate_writable_slot_id(&doc.slot_id)?;
    validate_v6_envelope(
        doc.schema_version,
        &doc.slot_id,
        &doc.slot_id,
        &doc.display_name,
    )?;
    doc.save.validate()?;
    let original = if new {
        None
    } else {
        let bytes = crate::save_v6::read_bounded(target, "E_SLOT", "E_SLOT_NOT_FOUND")?;
        if decode_slot_v6(&bytes, &doc.slot_id)?.1 == crate::save_v6::Profile::Legacy {
            crate::save_v6::retain_legacy_backup(target, &bytes, "E_SLOT")?;
        }
        Some(bytes)
    };
    let bytes = serde_json::to_vec_pretty(doc).map_err(|e| format!("E_SLOT_ENCODE: {e}"))?;
    crate::save_v6::atomic_write_verified_guarded(target, &bytes, original.as_deref(), "E_SLOT", || {
        if let Some(backup) = backup { crate::save_v6::recovery::require_unchanged(target, backup, "E_SLOT")?; }
        Ok(())
    }, |raw| {
        // Current DTO only: normalization must never conceal a mismatched temp/commit.
        let check: SlotDocumentV6 = serde_json::from_slice(raw).map_err(|_| "E_SLOT_CORRUPT")?;
        validate_v6_envelope(
            check.schema_version,
            &check.slot_id,
            &doc.slot_id,
            &check.display_name,
        )?;
        check.save.validate()?;
        if check != *doc {
            return Err("E_SLOT_VERIFY_MISMATCH".into());
        }
        Ok(())
    })
}

fn slot_v6_path(root: &Path, slot_id: &str) -> Result<PathBuf, String> {
    validate_slot_id(slot_id)?;
    Ok(slots_root(root).join(slot_id).join(SLOT_V6_FILE_NAME))
}

/// Read/migrate in memory only. Availability and candidate construction never write.
pub(crate) fn read_slot_v6_candidate(root: &Path, slot_id: &str) -> Result<(String, crate::save_v6::SaveV6), String> {
    read_slot_v6_source(root, slot_id).map(|(name, save, _)| (name, save))
}

pub(crate) fn read_slot_v6_source(root: &Path, slot_id: &str) -> Result<(String, crate::save_v6::SaveV6, Option<crate::save_v6::recovery::Backup>), String> {
    let path = slot_v6_path(root, slot_id)?;
    if crate::save_v6::path_present(&path) {
        let bytes = crate::save_v6::read_bounded(&path, "E_SLOT", "E_SLOT_NOT_FOUND")?;
        let (doc, _) = decode_slot_v6(&bytes, slot_id)?;
        return Ok((doc.display_name, doc.save, None));
    }
    if let Some(backup) = crate::save_v6::recovery::backup(&path, "E_SLOT")? {
        let (doc, _) = decode_slot_v6(&backup.bytes, slot_id)?;
        return Ok((doc.display_name, doc.save, Some(backup)));
    }
    let path = slot_path(root, slot_id)?;
    let (name, save) = if crate::save_v6::path_present(&path) {
        let doc = read_document(&path)?;
        if doc.slot_id != slot_id { return Err("E_SLOT_ID_MISMATCH".into()); }
        (doc.display_name, doc.save)
    } else {
        let doc = read_legacy_document(&legacy_slot_path(root, slot_id)?)?;
        if doc.slot_id != slot_id { return Err("E_SLOT_ID_MISMATCH".into()); }
        let mut save = save_v5::from_v4(&doc.save)?;
        let provenance = save.migration_provenance.as_mut().ok_or("E_SAVE_MIGRATION_PROVENANCE_INVALID")?;
        provenance.source_format = "slot-v4.json".into();
        provenance.method = "validated-v4-slot-copy; Grey Hive checkpoint mapping; original retained".into();
        (doc.display_name, save)
    };
    Ok((name, crate::save_v6::SaveV6::from_v5(save)?, None))
}
pub(crate) fn slot_v6_present(root: &Path, slot_id: &str) -> Result<bool, String> {
    Ok(crate::save_v6::path_present(&slot_v6_path(root, slot_id)?))
}
/// Called only after full explicit Continue validation. Original lower-version files remain intact.
pub(crate) fn write_continued_slot_v6(root: &Path, slot_id: &str, display_name: &str, save: &crate::save_v6::SaveV6, backup: Option<&crate::save_v6::recovery::Backup>) -> Result<(), String> {
    validate_writable_slot_id(slot_id)?;
    let target = slot_v6_path(root, slot_id)?;
    if let Some(backup) = backup {
        crate::save_v6::recovery::require_unchanged(&target, backup, "E_SLOT")?;
        decode_slot_v6(&backup.bytes, slot_id)?;
        return write_slot_v6_inner(&target, &SlotDocumentV6 {
            schema_version: 3, slot_id: slot_id.into(), display_name: display_name.into(),
            updated_at_ms: now_ms(), save: save.clone(),
        }, true, Some(backup));
    }
    crate::save_v6::recovery::require_no_recovery(&target, "E_SLOT")?;
    // Verify the actual winning source, including a corrupt newer file, before replacement.
    read_slot_v6_candidate(root, slot_id)?;
    if !crate::save_v6::path_present(&target) && !crate::save_v6::path_present(&slot_path(root, slot_id)?) {
        crate::save_v6::retain_v4_backup(&legacy_slot_path(root, slot_id)?, "E_SLOT")?;
    }
    write_slot_v6(&target, &SlotDocumentV6 {
        schema_version: 3, slot_id: slot_id.into(), display_name: display_name.into(),
        updated_at_ms: now_ms(), save: save.clone(),
    }, !crate::save_v6::path_present(&target))
}

pub fn read_slot_v6(
    root: &Path,
    slot_id: &str,
) -> Result<(String, crate::save_v6::SaveV6), String> {
    validate_slot_id(slot_id)?;
    let path = slots_root(root).join(slot_id).join(SLOT_V6_FILE_NAME);
    if crate::save_v6::path_present(&path) {
        let bytes = crate::save_v6::read_bounded(&path, "E_SLOT", "E_SLOT_NOT_FOUND")?;
        let (doc, _) = decode_slot_v6(&bytes, slot_id)?;
        return Ok((doc.display_name, doc.save));
    }
    if let Some(backup) = crate::save_v6::recovery::backup(&path, "E_SLOT")? {
        let (doc, _) = decode_slot_v6(&backup.bytes, slot_id)?;
        return Ok((doc.display_name, doc.save));
    }
    let (name, old) = read_slot(root, slot_id)?;
    let migrated = crate::save_v6::SaveV6::from_v5(old)?;
    let doc = SlotDocumentV6 {
        schema_version: 3,
        slot_id: slot_id.into(),
        display_name: name.clone(),
        updated_at_ms: now_ms(),
        save: migrated.clone(),
    };
    write_slot_v6(&path, &doc, true)?;
    Ok((name, migrated))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SlotDocument {
    schema_version: u32,
    slot_id: String,
    display_name: String,
    updated_at_ms: u64,
    save: SaveV5,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacySlotDocument {
    schema_version: u32,
    slot_id: String,
    display_name: String,
    updated_at_ms: u64,
    save: save_v3::SaveV3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotSummary {
    pub slot_id: String,
    pub display_name: String,
    pub updated_at_ms: u64,
    pub world_id: Option<String>,
    pub checkpoint_id: Option<String>,
    pub player_position_m: Option<Vec3>,
    pub current_hp: Option<u32>,
    pub max_hp: Option<u32>,
    pub current_energy: Option<u32>,
    pub max_energy: Option<u32>,
    pub gate_open: Option<bool>,
    pub completed_events: Vec<String>,
    pub read_only: bool,
    #[serde(default)]
    pub recoverable: bool,
    pub valid: bool,
    pub error_code: Option<String>,
}

pub fn validate_slot_id(slot_id: &str) -> Result<(), String> {
    if slot_id.is_empty()
        || slot_id.len() > 48
        || !slot_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("E_SLOT_ID_INVALID".into());
    }
    Ok(())
}

pub(crate) fn is_reserved_slot_id(slot_id: &str) -> bool {
    // The shipped Windows filesystem aliases ASCII case variants of this path.
    #[cfg(windows)]
    { slot_id.eq_ignore_ascii_case(RESERVED_SLOT_ID) }
    #[cfg(not(windows))]
    { slot_id == RESERVED_SLOT_ID }
}

fn validate_writable_slot_id(slot_id: &str) -> Result<(), String> {
    validate_slot_id(slot_id)?;
    if is_reserved_slot_id(slot_id) { return Err("E_SLOT_ID_RESERVED".into()); }
    Ok(())
}

/// Any physical reserved path conflicts with the virtual default-save alias.
/// Do not follow symlinks or hide unreadable, corrupt, or future-version evidence.
pub(crate) fn require_unambiguous_legacy_alias(root: &Path) -> Result<(), String> {
    match fs::symlink_metadata(slots_root(root).join(RESERVED_SLOT_ID)) {
        Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) => Ok(()),
        _ => Err("E_SLOT_ID_CONFLICT".into()),
    }
}

fn slots_root(root: &Path) -> PathBuf {
    root.join("slots")
}

fn slot_path(root: &Path, slot_id: &str) -> Result<PathBuf, String> {
    validate_slot_id(slot_id)?;
    Ok(slots_root(root).join(slot_id).join(SLOT_FILE_NAME))
}

fn legacy_slot_path(root: &Path, slot_id: &str) -> Result<PathBuf, String> {
    validate_slot_id(slot_id)?;
    Ok(slots_root(root).join(slot_id).join(LEGACY_SLOT_FILE_NAME))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn document(slot_id: &str, display_name: &str, save: SaveV5) -> Result<SlotDocument, String> {
    validate_slot_id(slot_id)?;
    let display_name = display_name.trim();
    if display_name.is_empty() || display_name.chars().count() > 48 {
        return Err("E_SLOT_NAME_INVALID".into());
    }
    save.validate()?;
    Ok(SlotDocument {
        schema_version: SLOT_V5_SCHEMA_VERSION,
        slot_id: slot_id.into(),
        display_name: display_name.into(),
        updated_at_ms: now_ms(),
        save,
    })
}

pub fn create_slot(
    root: &Path,
    slot_id: &str,
    display_name: &str,
    save: &SaveV5,
) -> Result<(), String> {
    if is_reserved_slot_id(slot_id) {
        return Err("E_SLOT_ID_RESERVED".into());
    }
    let doc = document(slot_id, display_name, save.clone())?;
    let dir = slots_root(root).join(slot_id);
    fs::create_dir_all(slots_root(root)).map_err(|e| format!("E_SLOT_DIR: {e}"))?;
    fs::create_dir(&dir).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "E_SLOT_EXISTS".to_string()
        } else {
            format!("E_SLOT_DIR: {error}")
        }
    })?;
    let path = dir.join(SLOT_FILE_NAME);
    let result = write_new_verified(&path, &doc);
    if result.is_err() {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
    result
}

pub fn overwrite_slot(
    root: &Path,
    slot_id: &str,
    display_name: &str,
    save: &SaveV5,
) -> Result<(), String> {
    if is_reserved_slot_id(slot_id) {
        return Err("E_SLOT_ID_RESERVED".into());
    }
    let doc = document(slot_id, display_name, save.clone())?;
    let target = slot_path(root, slot_id)?;
    if !target.is_file() {
        return Err("E_SLOT_NOT_FOUND".into());
    }
    write_replace_verified(&target, &doc)
}

fn write_new_verified(path: &Path, doc: &SlotDocument) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(doc).map_err(|e| format!("E_SLOT_ENCODE: {e}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("E_SLOT_CREATE: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("E_SLOT_WRITE: {e}"))?;
    file.sync_all().map_err(|e| format!("E_SLOT_SYNC: {e}"))?;
    drop(file);
    let check = read_document(path)?;
    if &check != doc {
        return Err("E_SLOT_VERIFY_MISMATCH".into());
    }
    Ok(())
}

fn write_replace_verified(path: &Path, doc: &SlotDocument) -> Result<(), String> {
    let parent = path.parent().ok_or("E_SLOT_PATH_INVALID")?;
    let token = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let temp = parent.join(format!(".slot.{}.{}.tmp", std::process::id(), token));
    let backup = parent.join(format!(".slot.{}.{}.bak", std::process::id(), token));
    let result = (|| {
        write_new_verified(&temp, doc)?;
        fs::rename(path, &backup).map_err(|e| format!("E_SLOT_BACKUP: {e}"))?;
        if let Err(error) = fs::rename(&temp, path) {
            let _ = fs::rename(&backup, path);
            return Err(format!("E_SLOT_COMMIT: {error}"));
        }
        let persisted = read_document(path)?;
        if persisted != *doc {
            let _ = fs::remove_file(path);
            let _ = fs::rename(&backup, path);
            return Err("E_SLOT_POSTWRITE_MISMATCH".into());
        }
        let _ = fs::remove_file(backup);
        Ok(())
    })();
    if temp.exists() {
        let _ = fs::remove_file(temp);
    }
    result
}

fn read_document(path: &Path) -> Result<SlotDocument, String> {
    let mut file = File::open(path).map_err(|_| "E_SLOT_NOT_FOUND".to_string())?;
    let size = file
        .metadata()
        .map_err(|e| format!("E_SLOT_READ: {e}"))?
        .len();
    if size > 8 * 1024 * 1024 {
        return Err("E_SLOT_TOO_LARGE".into());
    }
    let mut raw = String::with_capacity(size as usize);
    file.read_to_string(&mut raw)
        .map_err(|e| format!("E_SLOT_READ: {e}"))?;
    let raw_value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "E_SLOT_CORRUPT".to_string())?;
    let doc: SlotDocument =
        serde_json::from_value(raw_value.clone()).map_err(|_| "E_SLOT_CORRUPT".to_string())?;
    if !save_v5::same_schema_shape(
        &raw_value,
        &serde_json::to_value(&doc).map_err(|_| "E_SLOT_CORRUPT".to_string())?,
    ) {
        return Err("E_SLOT_SCHEMA_INVALID".into());
    }
    if doc.schema_version != SLOT_V5_SCHEMA_VERSION {
        return Err("E_SLOT_VERSION_UNSUPPORTED".into());
    }
    validate_slot_id(&doc.slot_id)?;
    if doc.display_name.trim().is_empty() || doc.display_name.chars().count() > 48 {
        return Err("E_SLOT_NAME_INVALID".into());
    }
    doc.save.validate()?;
    Ok(doc)
}

pub fn read_slot(root: &Path, slot_id: &str) -> Result<(String, SaveV5), String> {
    let v5_path = slot_path(root, slot_id)?;
    if v5_path.is_file() {
        let doc = read_document(&v5_path)?;
        if doc.slot_id != slot_id {
            return Err("E_SLOT_ID_MISMATCH".into());
        }
        return Ok((doc.display_name, doc.save));
    }
    migrate_v4_slot(root, slot_id)
}

fn read_legacy_document(path: &Path) -> Result<LegacySlotDocument, String> {
    let mut file = File::open(path).map_err(|_| "E_SLOT_NOT_FOUND".to_string())?;
    let size = file
        .metadata()
        .map_err(|e| format!("E_SLOT_READ: {e}"))?
        .len();
    if size > 8 * 1024 * 1024 {
        return Err("E_SLOT_TOO_LARGE".into());
    }
    let mut raw = String::with_capacity(size as usize);
    file.read_to_string(&mut raw)
        .map_err(|e| format!("E_SLOT_READ: {e}"))?;
    let raw_value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "E_SLOT_CORRUPT".to_string())?;
    let doc: LegacySlotDocument =
        serde_json::from_value(raw_value.clone()).map_err(|_| "E_SLOT_CORRUPT".to_string())?;
    if !save_v5::same_schema_shape(
        &raw_value,
        &serde_json::to_value(&doc).map_err(|_| "E_SLOT_CORRUPT".to_string())?,
    ) {
        return Err("E_SLOT_SCHEMA_INVALID".into());
    }
    if doc.schema_version != 1 {
        return Err("E_SLOT_VERSION_UNSUPPORTED".into());
    }
    validate_slot_id(&doc.slot_id)?;
    if doc.display_name.trim().is_empty() || doc.display_name.chars().count() > 48 {
        return Err("E_SLOT_NAME_INVALID".into());
    }
    doc.save.validate()?;
    if doc.save.schema_version != 4 {
        return Err("E_SAVE_MIGRATION_VERSION_UNSUPPORTED".into());
    }
    Ok(doc)
}

fn migrate_v4_slot(root: &Path, slot_id: &str) -> Result<(String, SaveV5), String> {
    validate_writable_slot_id(slot_id)?;
    let source = legacy_slot_path(root, slot_id)?;
    let doc = read_legacy_document(&source)?;
    if doc.slot_id != slot_id {
        return Err("E_SLOT_ID_MISMATCH".into());
    }
    let backup = source.with_file_name(format!("{LEGACY_SLOT_FILE_NAME}.bak"));
    if !backup.exists() {
        fs::copy(&source, &backup).map_err(|e| format!("E_SLOT_MIGRATION_BACKUP: {e}"))?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&backup)
            .and_then(|f| f.sync_all())
            .map_err(|e| format!("E_SLOT_MIGRATION_BACKUP_SYNC: {e}"))?;
    } else if fs::read(&backup).map_err(|e| format!("E_SLOT_MIGRATION_BACKUP_READ: {e}"))?
        != fs::read(&source).map_err(|e| format!("E_SLOT_MIGRATION_SOURCE_READ: {e}"))?
    {
        return Err("E_SLOT_MIGRATION_BACKUP_MISMATCH".into());
    }
    let mut save = save_v5::from_v4(&doc.save)?;
    let provenance = save
        .migration_provenance
        .as_mut()
        .ok_or("E_SAVE_MIGRATION_PROVENANCE_INVALID")?;
    provenance.source_format = "slot-v4.json".into();
    provenance.method =
        "validated-v4-slot-copy; Grey Hive checkpoint mapping; original retained".into();
    save.validate()?;
    let v5 = document(slot_id, &doc.display_name, save.clone())?;
    write_new_verified(&slot_path(root, slot_id)?, &v5)?;
    Ok((doc.display_name, save))
}

fn summary(
    slot_id: &str,
    display_name: &str,
    updated_at_ms: u64,
    save: &SaveV5,
    read_only: bool,
) -> SlotSummary {
    let progress = save
        .progression
        .progress
        .iter()
        .find(|item| item.world_id == save.world_id);
    let events = progress
        .map(|item| item.completed_events.clone())
        .unwrap_or_default();
    SlotSummary {
        slot_id: slot_id.into(),
        display_name: display_name.into(),
        updated_at_ms,
        world_id: Some(save.world_id.clone()),
        checkpoint_id: save.checkpoint_id.clone(),
        player_position_m: Some(save.player.position_m),
        current_hp: Some(save.player.current_hp),
        max_hp: Some(save.player.max_hp),
        current_energy: Some(save.player.current_energy),
        max_energy: Some(save.player.max_energy),
        gate_open: Some(events.iter().any(|event| event == "hive_power")),
        completed_events: events,
        read_only,
        recoverable: false,
        valid: true,
        error_code: None,
    }
}

pub fn list_slots(root: &Path) -> Vec<SlotSummary> {
    let mut result = Vec::new();
    let alias_conflict = require_unambiguous_legacy_alias(root).is_err();
    if alias_conflict { result.push(corrupt_summary(RESERVED_SLOT_ID, "E_SLOT_ID_CONFLICT")); }
    if let Ok(entries) = fs::read_dir(slots_root(root)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_reserved_slot_id(&name) || validate_slot_id(&name).is_err() || !entry.path().is_dir() {
                continue;
            }
            let path = entry.path().join(SLOT_FILE_NAME);
            let v6_path = entry.path().join(SLOT_V6_FILE_NAME);
            let legacy_path = entry.path().join(LEGACY_SLOT_FILE_NAME);
            if crate::save_v6::path_present(&v6_path) {
                match read_slot_v6(root, &name) {
                    Ok((display_name, save)) => result.push(summary(
                        &name,
                        &display_name,
                        fs::metadata(&v6_path)
                            .and_then(|m| m.modified())
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_millis().min(u64::MAX as u128) as u64)
                            .unwrap_or_default(),
                        &save.save,
                        false,
                    )),
                    Err(error) => result.push(corrupt_summary(&name, &error)),
                }
            } else if match crate::save_v6::recovery::backup(&v6_path, "E_SLOT") {
                Ok(Some(backup)) => {
                    match decode_slot_v6(&backup.bytes, &name) {
                        Ok((doc, _)) => {
                            let mut row = summary(&name, &doc.display_name, doc.updated_at_ms, &doc.save.save, true);
                            row.recoverable = true;
                            result.push(row);
                        }
                        Err(error) => result.push(corrupt_summary(&name, &error)),
                    }
                    true
                }
                Err(error) => { result.push(corrupt_summary(&name, &error)); true }
                Ok(None) => false,
            } {
                // The recovery decision already supplied a row; never fall through to V5.
            } else if crate::save_v6::path_present(&path) {
                match read_document(&path) {
                    Ok(doc) if doc.slot_id == name => result.push(summary(
                        &name,
                        &doc.display_name,
                        doc.updated_at_ms,
                        &doc.save,
                        false,
                    )),
                    Ok(_) => result.push(corrupt_summary(&name, "E_SLOT_ID_MISMATCH")),
                    Err(error) => result.push(corrupt_summary(&name, &error)),
                }
            } else if crate::save_v6::path_present(&legacy_path) {
                match read_legacy_document(&legacy_path) {
                    Ok(doc) if doc.slot_id == name => result.push(summary_v4_readonly(&name, &doc)),
                    Ok(_) => result.push(corrupt_summary(&name, "E_SLOT_ID_MISMATCH")),
                    Err(error) => result.push(corrupt_summary(&name, &error)),
                }
            }
        }
    }
    if !alias_conflict {
        if let Ok(save) = save_v3::read_save(root) {
            if let Ok(Some((candidate, Some(_)))) = crate::save_v6::read_candidate(root) {
                // The virtual alias will Continue the recovered default, never this older file.
                let mut row = summary(RESERVED_SLOT_ID, "自动存档", 0, &candidate.save, true);
                row.recoverable = true;
                result.push(row);
            } else {
                result.push(summary_legacy_save(RESERVED_SLOT_ID, &save));
            }
        }
    }
    result.sort_by(|a, b| {
        b.updated_at_ms
            .cmp(&a.updated_at_ms)
            .then(a.slot_id.cmp(&b.slot_id))
    });
    result
}

fn summary_legacy_save(slot_id: &str, save: &save_v3::SaveV3) -> SlotSummary {
    let progress = save
        .route
        .progress
        .iter()
        .find(|item| item.world_id == save.world_id);
    let events = progress
        .map(|item| item.completed_events.clone())
        .unwrap_or_default();
    SlotSummary {
        slot_id: slot_id.into(),
        display_name: "旧版单文件存档（只读）".into(),
        updated_at_ms: 0,
        world_id: Some(save.world_id.clone()),
        checkpoint_id: Some(save.checkpoint_id.clone()),
        player_position_m: Some(save.player.position_m),
        current_hp: Some(save.player.current_hp),
        max_hp: Some(save.player.max_hp),
        current_energy: Some(save.player.current_energy),
        max_energy: Some(save.player.max_energy),
        gate_open: Some(events.iter().any(|e| e == "hive_power")),
        completed_events: events,
        read_only: true,
        recoverable: false,
        valid: true,
        error_code: None,
    }
}

fn summary_v4_readonly(slot_id: &str, doc: &LegacySlotDocument) -> SlotSummary {
    let progress = doc
        .save
        .route
        .progress
        .iter()
        .find(|item| item.world_id == doc.save.world_id);
    let events = progress
        .map(|item| item.completed_events.clone())
        .unwrap_or_default();
    SlotSummary {
        slot_id: slot_id.into(),
        display_name: doc.display_name.clone(),
        updated_at_ms: doc.updated_at_ms,
        world_id: Some(doc.save.world_id.clone()),
        checkpoint_id: Some(doc.save.checkpoint_id.clone()),
        player_position_m: Some(doc.save.player.position_m),
        current_hp: Some(doc.save.player.current_hp),
        max_hp: Some(doc.save.player.max_hp),
        current_energy: Some(doc.save.player.current_energy),
        max_energy: Some(doc.save.player.max_energy),
        gate_open: Some(events.iter().any(|e| e == "hive_power")),
        completed_events: events,
        read_only: true,
        recoverable: false,
        valid: true,
        error_code: None,
    }
}

fn corrupt_summary(slot_id: &str, error: &str) -> SlotSummary {
    SlotSummary {
        slot_id: slot_id.into(),
        display_name: "无法验证的存档槽".into(),
        updated_at_ms: 0,
        world_id: None,
        checkpoint_id: None,
        player_position_m: None,
        current_hp: None,
        max_hp: None,
        current_energy: None,
        max_energy: None,
        gate_open: None,
        completed_events: Vec::new(),
        read_only: true,
        recoverable: false,
        valid: false,
        error_code: Some(error.into()),
    }
}

pub fn import_legacy(root: &Path, slot_id: &str, display_name: &str) -> Result<(), String> {
    let old = save_v5::read_v4(root)?;
    let save = save_v5::from_v4(&old)?;
    create_slot(root, slot_id, display_name, &save)
}

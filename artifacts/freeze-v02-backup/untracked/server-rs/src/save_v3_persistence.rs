//! Explicit-path V3 save commit and recovery boundary.
//!
//! This module is deliberately not wired to CampaignStore or startup. Every
//! operation requires a source, target, and existing restricted root. The
//! mapper remains pure; this layer owns bytes, SHA-256, backups, pending files,
//! sync, reload validation, and same-directory rename.

use crate::save_v3_migration::{
    migrate_v2_json, validate_envelope, SaveV3Envelope, SaveV3MigrationError,
};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const BACKUP_SUFFIX: &str = ".bak";
const PENDING_SUFFIX: &str = ".pending";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistenceFault {
    None,
    SourceRead,
    BackupWrite,
    PendingWrite,
    PendingReload,
    BeforeRename,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveV3PersistenceError {
    RootInvalid,
    SourceMissing,
    SourceNotRegular,
    SourceRead,
    PathOutsideRoot,
    PathSymlink,
    PathSame,
    PathDifferentDirectory,
    PathCrossVolume,
    BackupConflict,
    BackupSourceMismatch,
    BackupWrite,
    PendingExists,
    PendingWrite,
    PendingReload,
    TargetInvalid,
    TargetSourceMismatch,
    TargetWrite,
    CommitInterrupted,
    CommitRename,
    DirectorySync,
    Migration(SaveV3MigrationError),
}

impl SaveV3PersistenceError {
    pub fn code(&self) -> String {
        match self {
            Self::RootInvalid => "E_ROOT_INVALID".into(),
            Self::SourceMissing => "E_SOURCE_MISSING".into(),
            Self::SourceNotRegular => "E_SOURCE_NOT_REGULAR".into(),
            Self::SourceRead => "E_SOURCE_READ".into(),
            Self::PathOutsideRoot => "E_PATH_OUTSIDE_ROOT".into(),
            Self::PathSymlink => "E_PATH_SYMLINK".into(),
            Self::PathSame => "E_PATH_SAME".into(),
            Self::PathDifferentDirectory => "E_PATH_DIFFERENT_DIRECTORY".into(),
            Self::PathCrossVolume => "E_PATH_CROSS_VOLUME".into(),
            Self::BackupConflict => "E_BACKUP_CONFLICT".into(),
            Self::BackupSourceMismatch => "E_BACKUP_SOURCE_MISMATCH".into(),
            Self::BackupWrite => "E_BACKUP_WRITE".into(),
            Self::PendingExists => "E_PENDING_EXISTS".into(),
            Self::PendingWrite => "E_PENDING_WRITE".into(),
            Self::PendingReload => "E_PENDING_RELOAD".into(),
            Self::TargetInvalid => "E_TARGET_INVALID".into(),
            Self::TargetSourceMismatch => "E_TARGET_SOURCE_MISMATCH".into(),
            Self::TargetWrite => "E_TARGET_WRITE".into(),
            Self::CommitInterrupted => "E_COMMIT_INTERRUPTED".into(),
            Self::CommitRename => "E_COMMIT_RENAME".into(),
            Self::DirectorySync => "E_DIRECTORY_SYNC".into(),
            Self::Migration(error) => error.code(),
        }
    }
}

impl std::fmt::Display for SaveV3PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code())
    }
}

impl std::error::Error for SaveV3PersistenceError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveV3PersistenceReport {
    pub status: String,
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub pending_path: PathBuf,
    pub target_path: PathBuf,
    pub source_sha256: String,
    pub backup_sha256: String,
    pub target_sha256: String,
    pub source_unchanged: bool,
    pub pending_exists: bool,
    pub target_exists: bool,
    pub transform_none: bool,
    pub gate_b: String,
    pub choice_state: String,
    pub directory_sync_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEvidence {
    pub exists: bool,
    pub regular_file: bool,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistenceEvidence {
    pub source: FileEvidence,
    pub backup: FileEvidence,
    pub pending: FileEvidence,
    pub target: FileEvidence,
}

#[derive(Clone, Debug)]
struct ValidatedPaths {
    source: PathBuf,
    backup: PathBuf,
    pending: PathBuf,
    target: PathBuf,
    directory: PathBuf,
}

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn file_with_suffix(path: &Path, suffix: &str) -> Result<PathBuf, SaveV3PersistenceError> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(SaveV3PersistenceError::PathOutsideRoot)?;
    Ok(path.with_file_name(format!("{}{}", name, suffix)))
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

fn volume_key(path: &Path) -> Option<std::ffi::OsString> {
    path.components().find_map(|component| match component {
        Component::Prefix(prefix) => Some(prefix.as_os_str().to_os_string()),
        _ => None,
    })
}

fn reject_symlink_components(path: &Path) -> Result<(), SaveV3PersistenceError> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if fs::symlink_metadata(candidate)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(SaveV3PersistenceError::PathSymlink);
        }
        let parent = candidate.parent();
        if parent == Some(candidate) {
            break;
        }
        current = parent;
    }
    Ok(())
}

fn validate_paths(
    source: &Path,
    target: &Path,
    restricted_root: &Path,
) -> Result<ValidatedPaths, SaveV3PersistenceError> {
    reject_symlink_components(restricted_root)?;
    let root = restricted_root
        .canonicalize()
        .map_err(|_| SaveV3PersistenceError::RootInvalid)?;
    if !root.is_dir() {
        return Err(SaveV3PersistenceError::RootInvalid);
    }
    reject_symlink_components(&root)?;

    reject_symlink_components(source)?;
    let source = source
        .canonicalize()
        .map_err(|_| SaveV3PersistenceError::SourceMissing)?;
    if !source.is_file() {
        return Err(SaveV3PersistenceError::SourceNotRegular);
    }
    reject_symlink_components(&source)?;
    let source_directory = source
        .parent()
        .ok_or(SaveV3PersistenceError::PathOutsideRoot)?
        .to_path_buf();
    if !path_is_within(&source, &root) || !path_is_within(&source_directory, &root) {
        return Err(SaveV3PersistenceError::PathOutsideRoot);
    }

    reject_symlink_components(target)?;
    let target_directory = target
        .parent()
        .ok_or(SaveV3PersistenceError::PathOutsideRoot)?
        .canonicalize()
        .map_err(|_| SaveV3PersistenceError::PathOutsideRoot)?;
    reject_symlink_components(&target_directory)?;
    let target_name = target
        .file_name()
        .ok_or(SaveV3PersistenceError::PathOutsideRoot)?;
    let target = target_directory.join(target_name);
    reject_symlink_components(&target)?;
    if !path_is_within(&target_directory, &root) || !path_is_within(&target, &root) {
        return Err(SaveV3PersistenceError::PathOutsideRoot);
    }
    if source_directory != target_directory {
        if volume_key(&source_directory) != volume_key(&target_directory) {
            return Err(SaveV3PersistenceError::PathCrossVolume);
        }
        return Err(SaveV3PersistenceError::PathDifferentDirectory);
    }
    if source == target {
        return Err(SaveV3PersistenceError::PathSame);
    }

    Ok(ValidatedPaths {
        backup: {
            let backup = file_with_suffix(&source, BACKUP_SUFFIX)?;
            reject_symlink_components(&backup)?;
            backup
        },
        pending: {
            let pending = file_with_suffix(&target, PENDING_SUFFIX)?;
            reject_symlink_components(&pending)?;
            pending
        },
        source,
        target,
        directory: source_directory,
    })
}

fn write_create_new(
    path: &Path,
    bytes: &[u8],
    write_error: SaveV3PersistenceError,
) -> Result<(), SaveV3PersistenceError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| write_error.clone())?;
    file.write_all(bytes).map_err(|_| write_error.clone())?;
    file.flush().map_err(|_| write_error.clone())?;
    file.sync_all().map_err(|_| write_error)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<bool, SaveV3PersistenceError> {
    match File::open(path).and_then(|directory| directory.sync_all()) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn source_id(target: &Path) -> Result<String, SaveV3PersistenceError> {
    target
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .map(str::to_owned)
        .ok_or(SaveV3PersistenceError::PathOutsideRoot)
}

fn validate_existing_target(
    paths: &ValidatedPaths,
    source_hash: &str,
    expected: &SaveV3Envelope,
) -> Result<Option<SaveV3PersistenceReport>, SaveV3PersistenceError> {
    if !paths.target.exists() {
        return Ok(None);
    }
    let target_bytes =
        fs::read(&paths.target).map_err(|_| SaveV3PersistenceError::TargetInvalid)?;
    let envelope: SaveV3Envelope =
        serde_json::from_slice(&target_bytes).map_err(|_| SaveV3PersistenceError::TargetInvalid)?;
    validate_envelope(&envelope).map_err(|_| SaveV3PersistenceError::TargetInvalid)?;
    if envelope.source.source_sha256.as_deref() != Some(source_hash) {
        return Err(SaveV3PersistenceError::TargetSourceMismatch);
    }
    if envelope != *expected {
        return Err(SaveV3PersistenceError::TargetInvalid);
    }
    let source_after = fs::read(&paths.source).map_err(|_| SaveV3PersistenceError::SourceRead)?;
    let target_hash = sha256(&target_bytes);
    let backup_bytes =
        fs::read(&paths.backup).map_err(|_| SaveV3PersistenceError::BackupConflict)?;
    Ok(Some(SaveV3PersistenceReport {
        status: "already_present".into(),
        source_path: paths.source.clone(),
        backup_path: paths.backup.clone(),
        pending_path: paths.pending.clone(),
        target_path: paths.target.clone(),
        source_sha256: source_hash.into(),
        backup_sha256: sha256(&backup_bytes),
        target_sha256: target_hash,
        source_unchanged: sha256(&source_after) == source_hash,
        pending_exists: false,
        target_exists: true,
        transform_none: envelope.world_resume.transform.is_none(),
        gate_b: envelope.campaign.gate_facts.gate_b,
        choice_state: envelope.campaign.gate_facts.hive_choice_state,
        directory_sync_supported: false,
    }))
}

fn evidence_for(path: &Path) -> FileEvidence {
    match fs::symlink_metadata(path) {
        Ok(metadata) => FileEvidence {
            exists: true,
            regular_file: metadata.is_file(),
            sha256: if metadata.is_file() {
                fs::read(path).ok().map(|bytes| sha256(&bytes))
            } else {
                None
            },
        },
        Err(_) => FileEvidence {
            exists: false,
            regular_file: false,
            sha256: None,
        },
    }
}

pub fn inspect_transaction(source: &Path, target: &Path) -> PersistenceEvidence {
    let backup =
        file_with_suffix(source, BACKUP_SUFFIX).unwrap_or_else(|_| source.with_extension("bak"));
    let pending = file_with_suffix(target, PENDING_SUFFIX)
        .unwrap_or_else(|_| target.with_extension("pending"));
    PersistenceEvidence {
        source: evidence_for(source),
        backup: evidence_for(&backup),
        pending: evidence_for(&pending),
        target: evidence_for(target),
    }
}

pub fn commit_v2_to_v3(
    source: &Path,
    target: &Path,
    restricted_root: &Path,
) -> Result<SaveV3PersistenceReport, SaveV3PersistenceError> {
    commit_v2_to_v3_with_fault(source, target, restricted_root, PersistenceFault::None)
}

pub fn commit_v2_to_v3_with_fault(
    source: &Path,
    target: &Path,
    restricted_root: &Path,
    fault: PersistenceFault,
) -> Result<SaveV3PersistenceReport, SaveV3PersistenceError> {
    let paths = validate_paths(source, target, restricted_root)?;
    if paths.pending.exists() {
        return Err(SaveV3PersistenceError::PendingExists);
    }
    if fault == PersistenceFault::SourceRead {
        return Err(SaveV3PersistenceError::SourceRead);
    }
    let source_before = fs::read(&paths.source).map_err(|_| SaveV3PersistenceError::SourceRead)?;
    let source_hash = sha256(&source_before);
    let envelope = migrate_v2_json(
        &source_before,
        &source_id(&paths.target)?,
        Some(&source_hash),
    )
    .map_err(SaveV3PersistenceError::Migration)?;
    validate_envelope(&envelope).map_err(SaveV3PersistenceError::Migration)?;

    if paths.backup.exists() {
        if !paths.backup.is_file() {
            return Err(SaveV3PersistenceError::BackupConflict);
        }
        let backup_hash =
            sha256(&fs::read(&paths.backup).map_err(|_| SaveV3PersistenceError::BackupConflict)?);
        if backup_hash != source_hash {
            return Err(SaveV3PersistenceError::BackupSourceMismatch);
        }
    }
    if let Some(report) = validate_existing_target(&paths, &source_hash, &envelope)? {
        return Ok(report);
    }
    if fault == PersistenceFault::BackupWrite {
        return Err(SaveV3PersistenceError::BackupWrite);
    }
    if !paths.backup.exists() {
        write_create_new(
            &paths.backup,
            &source_before,
            SaveV3PersistenceError::BackupWrite,
        )?;
    }
    let backup_hash =
        sha256(&fs::read(&paths.backup).map_err(|_| SaveV3PersistenceError::BackupConflict)?);
    if backup_hash != source_hash {
        return Err(SaveV3PersistenceError::BackupSourceMismatch);
    }

    let encoded =
        serde_json::to_vec_pretty(&envelope).map_err(|_| SaveV3PersistenceError::PendingWrite)?;
    if fault == PersistenceFault::PendingWrite {
        return Err(SaveV3PersistenceError::PendingWrite);
    }
    write_create_new(
        &paths.pending,
        &encoded,
        SaveV3PersistenceError::PendingWrite,
    )?;
    if fault == PersistenceFault::PendingReload {
        return Err(SaveV3PersistenceError::PendingReload);
    }
    let pending_bytes =
        fs::read(&paths.pending).map_err(|_| SaveV3PersistenceError::PendingReload)?;
    let pending_envelope: SaveV3Envelope = serde_json::from_slice(&pending_bytes)
        .map_err(|_| SaveV3PersistenceError::PendingReload)?;
    validate_envelope(&pending_envelope).map_err(|_| SaveV3PersistenceError::PendingReload)?;
    if pending_envelope != envelope
        || pending_envelope.world_resume.transform.is_some()
        || pending_envelope.source.source_sha256.as_deref() != Some(source_hash.as_str())
    {
        return Err(SaveV3PersistenceError::PendingReload);
    }
    if fault == PersistenceFault::BeforeRename {
        return Err(SaveV3PersistenceError::CommitInterrupted);
    }
    fs::rename(&paths.pending, &paths.target).map_err(|_| SaveV3PersistenceError::CommitRename)?;
    let directory_sync_supported = sync_directory(&paths.directory)?;

    let target_bytes =
        fs::read(&paths.target).map_err(|_| SaveV3PersistenceError::TargetInvalid)?;
    let committed: SaveV3Envelope =
        serde_json::from_slice(&target_bytes).map_err(|_| SaveV3PersistenceError::TargetInvalid)?;
    validate_envelope(&committed).map_err(|_| SaveV3PersistenceError::TargetInvalid)?;
    if committed != envelope {
        return Err(SaveV3PersistenceError::TargetInvalid);
    }
    let source_after = fs::read(&paths.source).map_err(|_| SaveV3PersistenceError::SourceRead)?;
    Ok(SaveV3PersistenceReport {
        status: "committed".into(),
        source_path: paths.source,
        backup_path: paths.backup,
        pending_path: paths.pending,
        target_path: paths.target,
        source_sha256: source_hash.clone(),
        backup_sha256: backup_hash,
        target_sha256: sha256(&target_bytes),
        source_unchanged: sha256(&source_after) == source_hash,
        pending_exists: false,
        target_exists: true,
        transform_none: committed.world_resume.transform.is_none(),
        gate_b: committed.campaign.gate_facts.gate_b,
        choice_state: committed.campaign.gate_facts.hive_choice_state,
        directory_sync_supported,
    })
}

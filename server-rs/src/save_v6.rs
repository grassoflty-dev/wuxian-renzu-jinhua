//! Strict durable Save V6 wrapper. Effective rules are always resolved on load.
mod legacy_profile;
pub(crate) mod recovery;
use crate::{
    effects::EffectSource,
    formal_runtime::{build_v6::PlayerProgressionV6, RuntimeState},
    save_v5::SaveV5,
    world_persistent_v1::WorldPersistentState,
};
pub(crate) use legacy_profile::Profile;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const SAVE_V6_SCHEMA_VERSION: u32 = 6;
pub const EFFECT_SOURCES_VERSION: u32 = 2;
pub const SAVE_V6_FILE_NAME: &str = "formal-save-v6.json";
const MAX_SAVE_BYTES: u64 = 8 * 1024 * 1024;
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveV6 {
    pub schema_version: u32,
    // Required for every current object; only the private legacy DTO permits its absence.
    pub effect_sources_version: u32,
    pub save: SaveV5,
    pub progression: PlayerProgressionV6,
    pub effect_sources: Vec<EffectSource>,
    #[serde(default, skip_serializing_if = "WorldPersistentState::is_default")]
    pub world_persistent_v1: WorldPersistentState,
}

impl SaveV6 {
    pub(crate) fn capture(
        state: &RuntimeState,
        scene: Option<&crate::scene_runtime::SceneRuntime>,
    ) -> Result<Self, String> {
        let save = SaveV5::capture(state, scene)?;
        let mut progression = state.progression_v6.clone();
        progression.sync_capabilities(&state.capabilities);
        let mut world_persistent_v1 = state.world_persistent_v1.clone();
        if let Some(scene) = scene.filter(|scene| scene.is_complete_clockworks_production()) {
            world_persistent_v1
                .validate()
                .map_err(|_| "E_SAVE_WORLD_PERSISTENT_INVALID")?;
            let [active_scene_state] = save.scene_states.as_slice() else {
                return Err("E_SAVE_SCENE_HISTORY_ACTIVE_ROW_INVALID".into());
            };
            world_persistent_v1.upsert_scene_history_row(active_scene_state.clone())?;
            scene.validate_scene_history(&world_persistent_v1)?;
        }
        Self::from_authority(save, progression, world_persistent_v1)
    }

    /// A shared current constructor, used only after the caller has established authority.
    /// It deliberately does not repair the progression mirror or mutate capability state.
    pub(crate) fn from_authority(
        save: SaveV5,
        progression: PlayerProgressionV6,
        world_persistent_v1: WorldPersistentState,
    ) -> Result<Self, String> {
        validate_authority(&save, &progression, &world_persistent_v1)?;
        let (effect_sources, _) =
            progression.resolve_rules_with_capabilities(&save.capabilities, &save.rear_view)?;
        let value = Self {
            schema_version: SAVE_V6_SCHEMA_VERSION,
            effect_sources_version: EFFECT_SOURCES_VERSION,
            save,
            progression,
            effect_sources,
            world_persistent_v1,
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn from_v5(save: SaveV5) -> Result<Self, String> {
        save.validate()?;
        let mut progression = PlayerProgressionV6 {
            inventory: save
                .inventory
                .items
                .iter()
                .map(|item| (item.item_id.clone(), item.quantity))
                .collect(),
            ..PlayerProgressionV6::default()
        };
        progression.sync_capabilities(&save.capabilities);
        Self::from_authority(save, progression, WorldPersistentState::default())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SAVE_V6_SCHEMA_VERSION {
            return Err("E_SAVE_VERSION_UNSUPPORTED".into());
        }
        if self.effect_sources_version != EFFECT_SOURCES_VERSION {
            return Err("E_SAVE_EFFECT_SOURCES_VERSION_UNSUPPORTED".into());
        }
        if self.world_persistent_v1.map_knowledge_archive_v1.as_ref().is_some_and(|a|a.schema_version!=1) {
            return Err("E_SAVE_MAP_ARCHIVE_VERSION_UNSUPPORTED".into());
        }
        if self.world_persistent_v1.map_knowledge_archive_v2.as_ref().is_some_and(|a|a.schema_version!=2) {
            return Err("E_SAVE_GH_MAP_ARCHIVE_VERSION_UNSUPPORTED".into());
        }
        let marker=self.world_persistent_v1.shaft_recipe_format_v2.as_ref();
        for frame in [self.save.moving_support_frame.as_ref(),self.save.shaft_inactive_history.as_ref().and_then(|h|h.frame.as_ref())].into_iter().flatten() {
            if frame.shaft_lower_recipe.as_ref().is_some_and(|id|Some(id)!=marker) {return Err("E_SAVE_SHAFT_RECIPE_FORMAT_MISMATCH".into());}
        }
        validate_authority(&self.save, &self.progression, &self.world_persistent_v1)?;
        let (sources, _) = self
            .progression
            .resolve_rules_with_capabilities(&self.save.capabilities, &self.save.rear_view)?;
        if sources != self.effect_sources {
            return Err("E_SAVE_EFFECT_SOURCES_INVALID".into());
        }
        Ok(())
    }

    pub fn resolve_rules(&self) -> Result<crate::player_rules::EffectivePlayerRules, String> {
        self.validate()?;
        self.progression
            .resolve_rules_with_capabilities(&self.save.capabilities, &self.save.rear_view)
            .map(|(_, rules)| rules)
    }

    pub(crate) fn restore_state(&self) -> Result<RuntimeState, String> {
        self.validate()?;
        let mut state = self.save.restore_state()?;
        state.install_build(self.progression.clone())?;
        state.world_persistent_v1 = self.world_persistent_v1.clone();
        state
            .world_persistent_v1
            .resolve_at(state.world.server_time_ms)
            .map_err(|error| format!("E_SAVE_WORLD_PERSISTENT_INVALID: {error:?}"))?;
        Ok(state)
    }
}

fn validate_authority(
    save: &SaveV5,
    progression: &PlayerProgressionV6,
    persistent: &WorldPersistentState,
) -> Result<(), String> {
    save.validate()?;
    // History hydration requires an authored active scene. Legacy world-only
    // snapshots have no SceneRuntime to receive even an empty history record.
    if persistent.scene_history_v1.is_some() && save.scene_id == save.world_id {
        return Err("E_SAVE_SCENE_HISTORY_ACTIVE_ROW_INVALID".into());
    }
    crate::formal_runtime::warden_encounter::validate_saved(save, persistent)?;
    crate::formal_runtime::clockworks_controls::validate_saved(save, &persistent.clockworks)?;
    crate::formal_runtime::clockworks_roster::validate_saved(save, &persistent.clockworks)?;
    crate::formal_runtime::gear_shaft_support::validate_saved(save, &persistent.clockworks)?;
    crate::formal_runtime::grey_hive_roster::validate_saved(save, &persistent.grey_hive)?;
    crate::formal_runtime::grey_hive_beacon::validate_saved(save, &persistent.grey_hive)?;
    crate::formal_runtime::swarm_roster::validate_saved(save, &persistent.grey_hive)?;
    crate::formal_runtime::tidebound_roster::validate_saved(save, &persistent.mist_harbor)?;
    crate::formal_runtime::signal_wraith_roster::validate_saved(save, &persistent.mist_harbor)?;
    crate::formal_runtime::environmental_hazards::validate_saved(
        &persistent.environment, save.server_time_ms,
    )?;
    persistent
        .validate()
        .map_err(|error| format!("E_SAVE_WORLD_PERSISTENT_INVALID: {error:?}"))?;
    let mut authority: Vec<_> = save
        .capabilities
        .grants
        .iter()
        .map(|grant| grant.capability_id.clone())
        .collect();
    authority.sort();
    if progression.capabilities != authority {
        return Err("E_SAVE_CAPABILITY_BUILD_MISMATCH".into());
    }
    Ok(())
}

pub fn save_path(root: &Path) -> PathBuf {
    root.join(SAVE_V6_FILE_NAME)
}

pub(crate) fn decode_value(value: serde_json::Value) -> Result<(SaveV6, Profile), String> {
    legacy_profile::decode(value)
}

fn decode_bytes(bytes: &[u8]) -> Result<(SaveV6, Profile), String> {
    let value = serde_json::from_slice(bytes).map_err(|_| "E_SAVE_CORRUPT")?;
    decode_value(value)
}

pub fn read_save(root: &Path) -> Result<SaveV6, String> {
    read_candidate(root)?.map(|(save, _)| save).ok_or_else(|| "E_NO_SAVE".into())
}

/// A normalized candidate plus the exact backup proof; never writes or grants.
pub(crate) fn read_candidate(root: &Path) -> Result<Option<(SaveV6, Option<recovery::Backup>)>, String> {
    Ok(read_candidate_with_raw(root)?.map(|(save,backup,_)|(save,backup)))
}

/// Retain the exact original decode buffer for a deferred Exit admission.
pub(crate) fn read_candidate_with_raw(root: &Path) -> Result<Option<(SaveV6, Option<recovery::Backup>, Option<Vec<u8>>)>, String> {
    let target = save_path(root);
    if path_present(&target) {
        let bytes = read_bounded(&target, "E_SAVE", "E_NO_SAVE")?;
        return Ok(Some((decode_bytes(&bytes)?.0, None, Some(bytes))));
    }
    match recovery::backup(&target, "E_SAVE")? {
        Some(backup) => Ok(Some((decode_bytes(&backup.bytes)?.0, Some(backup), None))),
        None => Ok(None),
    }
}

pub fn read_or_migrate(root: &Path) -> Result<SaveV6, String> {
    // A present but invalid V6 path never falls through to an older source.
    if path_present(&save_path(root)) {
        return read_save(root);
    }
    recovery::require_no_recovery(&save_path(root), "E_SAVE")?;
    let save = SaveV6::from_v5(crate::save_v5::read_or_migrate(root)?)?;
    write_save(root, &save)?;
    Ok(save)
}

pub fn write_save(root: &Path, save: &SaveV6) -> Result<(), String> {
    recovery::require_no_recovery(&save_path(root), "E_SAVE")?;
    write_save_inner(root, save, None, None)
}

pub(crate) fn write_recovered_save(root: &Path, save: &SaveV6, backup: &recovery::Backup) -> Result<(), String> {
    recovery::require_unchanged(&save_path(root), backup, "E_SAVE")?;
    write_save_inner(root, save, Some(backup), None)
}

pub(crate) fn write_continued_guarded(root: &Path, save: &SaveV6, expected: &[u8]) -> Result<(), String> {
    recovery::require_no_recovery(&save_path(root), "E_SAVE")?;
    write_save_inner(root,save,None,Some(expected))
}

pub(crate) fn lock_continued_source_path(target: &Path, expected: &[u8], prefix: &str) -> Result<File, String> {
    recovery::reject_links(target,prefix)?;
    #[cfg(not(windows))]
    { let _=(expected,prefix); return Err("E_EXIT_PREVIEW_SOURCE_LOCK_UNSUPPORTED".into()); }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let mut file=OpenOptions::new().read(true).share_mode(1).open(target).map_err(|e|format!("{prefix}_READ: {e}"))?;
        let metadata=file.metadata().map_err(|e|format!("{prefix}_READ: {e}"))?;
        if !metadata.is_file() || metadata.len()>MAX_SAVE_BYTES {return Err(format!("{prefix}_READ_NOT_REGULAR"));}
        let mut bytes=Vec::new();
        (&mut file).take(MAX_SAVE_BYTES+1).read_to_end(&mut bytes).map_err(|e|format!("{prefix}_READ: {e}"))?;
        if bytes!=expected {return Err(format!("{prefix}_SOURCE_CHANGED"));}
        Ok(file)
    }
}

pub(crate) fn require_continued_source(root: &Path, expected: &[u8]) -> Result<File, String> {
    let target=save_path(root);
    recovery::require_no_recovery(&target,"E_SAVE")?;
    lock_continued_source_path(&target,expected,"E_SAVE")
}

fn write_save_inner(root: &Path, save: &SaveV6, backup: Option<&recovery::Backup>, expected: Option<&[u8]>) -> Result<(), String> {
    save.validate()?;
    fs::create_dir_all(root).map_err(|e| format!("E_SAVE_DIR: {e}"))?;
    let target = save_path(root);
    let original = if backup.is_some() {
        if path_present(&target) { return Err("E_SAVE_RECOVERY_SOURCE_CHANGED".into()); }
        None
    } else if path_present(&target) {
        let bytes = read_bounded(&target, "E_SAVE", "E_NO_SAVE")?;
        if expected.is_some_and(|guard| guard!=bytes.as_slice()) {return Err("E_SAVE_SOURCE_CHANGED".into());}
        let (previous,profile)=decode_bytes(&bytes)?;
        if profile == Profile::Legacy {
            retain_legacy_backup(&target, &bytes, "E_SAVE")?;
        }
        if previous.world_persistent_v1.map_knowledge_archive_v1.is_none()
            && save.world_persistent_v1.map_knowledge_archive_v1.is_some() {
            retain_archive_upgrade_backup(&target,&bytes,"E_SAVE")?;
        }
        if previous.world_persistent_v1.map_knowledge_archive_v2.is_none()
            && save.world_persistent_v1.map_knowledge_archive_v2.is_some() {
            retain_gh_archive_upgrade_backup(&target,&bytes,"E_SAVE")?;
        }
        if previous.world_persistent_v1.shaft_recipe_format_v2.is_none()
            && save.world_persistent_v1.shaft_recipe_format_v2.is_some() {
            retain_shaft_recipe_upgrade_backup(&target,&bytes,"E_SAVE")?;
        }
        if previous.world_persistent_v1.lower_service_map_v1.is_none() && save.world_persistent_v1.lower_service_map_v1.is_some() {
            retain_lower_service_map_upgrade_backup(&target,&bytes,"E_SAVE")?;
        }
        Some(bytes)
    } else {
        if expected.is_some() {return Err("E_SAVE_SOURCE_CHANGED".into());}
        None
    };
    let bytes = serde_json::to_vec_pretty(save).map_err(|e| format!("E_SAVE_ENCODE: {e}"))?;
    atomic_write_verified_guarded(&target, &bytes, original.as_deref(), "E_SAVE", || {
        if let Some(backup) = backup {
            recovery::require_unchanged(&target, backup, "E_SAVE")?;
            let (previous,_)=decode_bytes(&backup.bytes)?;
            if previous.world_persistent_v1.shaft_recipe_format_v2.is_none() && save.world_persistent_v1.shaft_recipe_format_v2.is_some() {
                retain_shaft_recipe_upgrade_backup(&target,&backup.bytes,"E_SAVE")?;
            }
            if previous.world_persistent_v1.lower_service_map_v1.is_none() && save.world_persistent_v1.lower_service_map_v1.is_some() {
                retain_lower_service_map_upgrade_backup(&target,&backup.bytes,"E_SAVE")?;
            }
        }
        Ok(())
    }, |raw| {
        // Decode only the current DTO here. Never normalize a temp/commit equality check.
        let persisted: SaveV6 = serde_json::from_slice(raw).map_err(|_| "E_SAVE_VERIFY")?;
        persisted.validate()?;
        if persisted != *save {
            return Err("E_SAVE_VERIFY_MISMATCH".into());
        }
        Ok(())
    })
}

// Unlike Path::exists, preserve precedence for dangling symlinks and unreadable paths.
pub(crate) fn path_present(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
    }
}

pub(crate) fn read_bounded(path: &Path, prefix: &str, missing: &str) -> Result<Vec<u8>, String> {
    recovery::reject_links(path, prefix)?;
    let meta = fs::symlink_metadata(path).map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { missing.to_string() } else { format!("{prefix}_READ: {e}") })?;
    if !meta.file_type().is_file() { return Err(format!("{prefix}_READ_NOT_REGULAR")); }
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            missing.to_string()
        } else {
            format!("{prefix}_READ: {e}")
        }
    })?;
    let metadata = file.metadata().map_err(|e| format!("{prefix}_READ: {e}"))?;
    if !metadata.is_file() { return Err(format!("{prefix}_READ_NOT_REGULAR")); }
    let size = metadata.len();
    if size > MAX_SAVE_BYTES {
        return Err(format!("{prefix}_TOO_LARGE"));
    }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(MAX_SAVE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{prefix}_READ: {e}"))?;
    if bytes.len() as u64 > MAX_SAVE_BYTES {
        return Err(format!("{prefix}_TOO_LARGE"));
    }
    Ok(bytes)
}

/// A retained migration backup is independent of the transient rollback file.
/// Existing bytes must match exactly; an unrelated backup is never replaced.
pub(crate) fn retain_legacy_backup(
    target: &Path,
    bytes: &[u8],
    prefix: &str,
) -> Result<(), String> {
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("E_SAVE_PATH_INVALID")?;
    let backup = target.with_file_name(format!("{name}.legacy-effect-sources-v1.bak"));
    retain_backup_at(target, &backup, bytes, prefix)
}

pub(crate) fn retain_archive_upgrade_backup(target:&Path,bytes:&[u8],prefix:&str)->Result<(),String>{
    let name=target.file_name().and_then(|s|s.to_str()).ok_or("E_SAVE_PATH_INVALID")?;
    retain_backup_at(target,&target.with_file_name(format!("{name}.pre-map-archive-v1.bak")),bytes,prefix)
}
pub(crate) fn retain_shaft_recipe_upgrade_backup(target:&Path,bytes:&[u8],prefix:&str)->Result<(),String>{
    retain_numbered_backup(target,bytes,prefix,"pre-shaft-recipe-v2")
}
pub(crate) fn retain_lower_service_map_upgrade_backup(target:&Path,bytes:&[u8],prefix:&str)->Result<(),String>{
    retain_numbered_backup(target,bytes,prefix,"pre-lower-service-map-v1")
}
fn retain_numbered_backup(target:&Path,bytes:&[u8],prefix:&str,tag:&str)->Result<(),String>{
    let name=target.file_name().and_then(|s|s.to_str()).ok_or("E_SAVE_PATH_INVALID")?;
    // A new journey may introduce the same format again. Never replace an
    // earlier backup; retain exact bytes under the first available name.
    let mut index=0_u32;
    loop {
        let suffix=if index==0 {format!(".{tag}.bak")}
            else {format!(".{tag}.{index}.bak")};
        let path=target.with_file_name(format!("{name}{suffix}"));
        match retain_backup_at(target,&path,bytes,prefix) {
            Ok(())=>return Ok(()),
            Err(e) if e==format!("{prefix}_MIGRATION_BACKUP_MISMATCH")=>{
                index=index.checked_add(1).ok_or_else(||format!("{prefix}_MIGRATION_BACKUP_EXHAUSTED"))?;
            },
            Err(e)=>return Err(e),
        }
    }
}
pub(crate) fn retain_gh_archive_upgrade_backup(target:&Path,bytes:&[u8],prefix:&str)->Result<(),String>{
    let name=target.file_name().and_then(|s|s.to_str()).ok_or("E_SAVE_PATH_INVALID")?;
    retain_backup_at(target,&target.with_file_name(format!("{name}.pre-map-archive-v2.bak")),bytes,prefix)
}

/// Preserve the established V4 backup name without writing an intermediate migrated save.
pub(crate) fn retain_v4_backup(target: &Path, prefix: &str) -> Result<(), String> {
    let bytes = read_bounded(target, prefix, "E_SAVE_MIGRATION_SOURCE_MISSING")?;
    let name = target.file_name().and_then(|s| s.to_str()).ok_or("E_SAVE_PATH_INVALID")?;
    retain_backup_at(target, &target.with_file_name(format!("{name}.bak")), &bytes, prefix)
}

fn retain_backup_at(target: &Path, backup: &Path, bytes: &[u8], prefix: &str) -> Result<(), String> {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
    {
        Ok(mut file) => {
            file.write_all(bytes)
                .map_err(|e| format!("{prefix}_MIGRATION_BACKUP_WRITE: {e}"))?;
            file.sync_all()
                .map_err(|e| format!("{prefix}_MIGRATION_BACKUP_SYNC: {e}"))?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(format!("{prefix}_MIGRATION_BACKUP: {e}")),
    }
    // A retained symlink could follow the target across replacement and lose the old bytes.
    // Accept only our newly-created regular file or an existing matching regular file.
    let metadata = fs::symlink_metadata(&backup)
        .map_err(|e| format!("{prefix}_MIGRATION_BACKUP_METADATA: {e}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!("{prefix}_MIGRATION_BACKUP_NOT_REGULAR"));
    }
    let stored = read_bounded(&backup, prefix, "E_SAVE_MIGRATION_BACKUP_MISSING")?;
    if stored != bytes {
        return Err(format!("{prefix}_MIGRATION_BACKUP_MISMATCH"));
    }
    // Reused matching bytes must be durable too; a directory sync alone does not flush data.
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(&backup)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("{prefix}_MIGRATION_BACKUP_SYNC: {e}"))?;
    sync_directory(target.parent().ok_or("E_SAVE_PATH_INVALID")?, prefix)?;
    Ok(())
}

fn sync_directory(path: &Path, prefix: &str) -> Result<(), String> {
    #[cfg(unix)]
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("{prefix}_DIR_SYNC: {e}"))?;
    #[cfg(not(unix))]
    let _ = (path, prefix);
    Ok(())
}

/// Verify both the bytes and current DTO before and after commit. Every failure after
/// moving the original attempts rollback, and a failed rollback is explicitly reported.
pub(crate) fn atomic_write_verified(
    target: &Path,
    bytes: &[u8],
    original: Option<&[u8]>,
    prefix: &str,
    verify: impl Fn(&[u8]) -> Result<(), String>,
) -> Result<(), String> {
    atomic_write_verified_guarded(target, bytes, original, prefix, || Ok(()), verify)
}

pub(crate) fn atomic_write_verified_guarded(
    target: &Path, bytes: &[u8], original: Option<&[u8]>, prefix: &str,
    precommit: impl Fn() -> Result<(), String>,
    verify: impl Fn(&[u8]) -> Result<(), String>,
) -> Result<(), String> {
    recovery::reject_links(target, prefix)?;
    let parent = target.parent().ok_or("E_SAVE_PATH_INVALID")?;
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("E_SAVE_PATH_INVALID")?;
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let temp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), id));
    let rollback = parent.join(format!(".{name}.{}.{}.bak", std::process::id(), id));
    let mut owns_temp = false;
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("{prefix}_TEMP: {e}"))?;
        owns_temp = true;
        file.write_all(bytes)
            .map_err(|e| format!("{prefix}_WRITE: {e}"))?;
        file.sync_all().map_err(|e| format!("{prefix}_SYNC: {e}"))?;
        drop(file);
        let prepared = read_bounded(&temp, prefix, "E_SAVE_VERIFY_MISSING")?;
        if prepared != bytes {
            return Err(format!("{prefix}_VERIFY_MISMATCH"));
        }
        verify(&prepared)?;
        precommit()?;
        #[cfg(test)] recovery_test_fault("prepared")?;
        if let Some(old) = original {
            if read_bounded(target, prefix, "E_SAVE_TARGET_MISSING")? != old {
                return Err(format!("{prefix}_TARGET_CHANGED"));
            }
            if path_present(&rollback) {
                return Err(format!("{prefix}_BACKUP_EXISTS"));
            }
            fs::rename(target, &rollback).map_err(|e| format!("{prefix}_BACKUP: {e}"))?;
        } else if path_present(target) {
            return Err(format!("{prefix}_TARGET_EXISTS"));
        }
        let committed = (|| {
            sync_directory(parent, prefix)?;
            if let Some(old) = original {
                if read_bounded(&rollback, prefix, "E_SAVE_BACKUP_MISSING")? != old {
                    return Err(format!("{prefix}_BACKUP_MISMATCH"));
                }
            }
            #[cfg(test)] recovery_test_fault("commit")?;
            fs::rename(&temp, target).map_err(|e| format!("{prefix}_COMMIT: {e}"))?;
            #[cfg(test)] RECOVERY_TEST_COMMITS.with(|count| count.set(count.get() + 1));
            #[cfg(test)] recovery_test_fault("postwrite")?;
            let actual = read_bounded(target, prefix, "E_SAVE_POSTWRITE_MISSING")?;
            if actual != bytes {
                return Err(format!("{prefix}_POSTWRITE_MISMATCH"));
            }
            verify(&actual)?;
            #[cfg(test)] recovery_test_fault("verified")?;
            sync_directory(parent, prefix)
        })();
        if let Err(error) = committed {
            let recovery = (|| -> Result<(), String> {
                if path_present(target) {
                    fs::remove_file(target)
                        .map_err(|e| format!("{prefix}_ROLLBACK_REMOVE: {e}"))?;
                }
                if let Some(old) = original {
                    fs::rename(&rollback, target).map_err(|e| format!("{prefix}_ROLLBACK: {e}"))?;
                    if read_bounded(target, prefix, "E_SAVE_ROLLBACK_MISSING")? != old {
                        return Err(format!("{prefix}_ROLLBACK_MISMATCH"));
                    }
                }
                sync_directory(parent, prefix)?;
                Ok(())
            })();
            return Err(match recovery {
                Ok(()) => error,
                Err(recovery) => format!("{error}; {recovery}"),
            });
        }
        // The replacement is already verified and durable. Cleanup cannot turn a
        // successful transaction into an error; keep a leftover rollback on failure.
        if original.is_some() && fs::remove_file(&rollback).is_ok() {
            let _ = sync_directory(parent, prefix);
        }
        Ok(())
    })();
    if owns_temp && path_present(&temp) {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
thread_local! { pub(crate) static BEFORE_RECOVERY_CONTINUE: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = std::cell::RefCell::new(None); }
#[cfg(test)]
pub(crate) fn recovery_test_before_continue() {
    BEFORE_RECOVERY_CONTINUE.with(|hook| if let Some(hook) = hook.borrow_mut().take() { hook(); });
}

#[cfg(test)]
thread_local! { pub(crate) static RECOVERY_TEST_COMMITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static RECOVERY_TEST_FAULT: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) }; }
#[cfg(test)]
fn recovery_test_fault(stage: &str) -> Result<(), String> {
    if RECOVERY_TEST_FAULT.with(|fault| fault.get() == Some(stage)) {
        Err(format!("E_SAVE_INJECTED_{stage}"))
    } else { Ok(()) }
}

#[cfg(test)]
mod persistence_fault_tests {
    use super::*;
    use std::cell::Cell;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "save-v6-fault-{}-{}",
                std::process::id(),
                NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }
        fn target(&self) -> PathBuf {
            self.0.join("target.json")
        }
        fn transient(&self, suffix: &str) -> PathBuf {
            fs::read_dir(&self.0)
                .unwrap()
                .map(|e| e.unwrap().path())
                .find(|p| {
                    p.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with(".target.json.")
                        && p.extension().unwrap() == suffix
                })
                .unwrap()
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn map_archive_wire_compatibility_and_upgrade_backup_guard() {
        let state=crate::formal_runtime::archive_test_initial_state().unwrap();
        let old=SaveV6::capture(&state,None).unwrap();
        let mut old_json=serde_json::to_value(&old).unwrap();
        assert!(old.world_persistent_v1.map_knowledge_archive_v1.is_none());
        assert!(decode_bytes(&serde_json::to_vec(&old_json).unwrap()).unwrap().0.world_persistent_v1.map_knowledge_archive_v1.is_none());
        assert!(old_json.get("worldPersistentV1").and_then(|v|v.get("mapKnowledgeArchiveV1")).is_none());
        let mut null=old_json.clone();null["worldPersistentV1"]=serde_json::json!({"mapKnowledgeArchiveV1":null});
        assert!(decode_bytes(&serde_json::to_vec(&null).unwrap()).is_err());
        let mut legacy=null.clone();legacy.as_object_mut().unwrap().remove("effectSourcesVersion");
        assert!(decode_bytes(&serde_json::to_vec(&legacy).unwrap()).is_err());
        let record=crate::world_persistent_v1::MapKnowledgeRecord{world_id:"mist_harbor".into(),scene_id:"unknown".into(),scene_source_sha256:"0".repeat(64),provider_family:"unknown".into(),layer_id:"ground_y0".into(),frame_id:"unknown".into(),parent_id:"unknown".into(),rows:vec![crate::world_v3::ExploredRoom{room_id:"unknown:part0".into(),outline_m:vec![crate::world_v3::Vec3::new(0.,0.,0.).unwrap(),crate::world_v3::Vec3::new(1.,0.,0.).unwrap(),crate::world_v3::Vec3::new(1.,0.,1.).unwrap(),crate::world_v3::Vec3::new(0.,0.,1.).unwrap()]}]};
        let mut next=old.clone();next.world_persistent_v1.map_knowledge_archive_v1=Some(crate::world_persistent_v1::MapKnowledgeArchiveV1{schema_version:1,records:vec![record]});
        next.validate().unwrap();let mut unsupported=next.clone();unsupported.world_persistent_v1.map_knowledge_archive_v1.as_mut().unwrap().schema_version=2;
        assert_eq!(unsupported.validate().unwrap_err(),"E_SAVE_MAP_ARCHIVE_VERSION_UNSUPPORTED");
        let mut malformed=serde_json::to_value(&next).unwrap();malformed["worldPersistentV1"]["mapKnowledgeArchiveV1"]["records"][0]["rows"][0]["outlineM"][0]["extra"]=serde_json::json!(1);
        assert!(decode_bytes(&serde_json::to_vec(&malformed).unwrap()).is_err());
        let mut too_many=serde_json::to_value(&next).unwrap();let first=too_many["worldPersistentV1"]["mapKnowledgeArchiveV1"]["records"][0].clone();
        too_many["worldPersistentV1"]["mapKnowledgeArchiveV1"]["records"]=serde_json::Value::Array(vec![first;39]);
        assert!(decode_bytes(&serde_json::to_vec(&too_many).unwrap()).is_err());
        let dir=TestDir::new();write_save(&dir.0,&old).unwrap();let target=save_path(&dir.0);let original=fs::read(&target).unwrap();
        let backup=dir.0.join("formal-save-v6.json.pre-map-archive-v1.bak");fs::write(&backup,b"unrelated").unwrap();
        assert!(write_save(&dir.0,&next).is_err());assert_eq!(fs::read(&target).unwrap(),original);assert_eq!(fs::read(&backup).unwrap(),b"unrelated");
        fs::remove_file(&backup).unwrap();write_save(&dir.0,&next).unwrap();assert_eq!(fs::read(&backup).unwrap(),original);
        assert_eq!(read_save(&dir.0).unwrap(),next);
        crate::save_slots::create_slot_v6(&dir.0,"archive-test","Archive test",&old).unwrap();
        let slot=dir.0.join("slots").join("archive-test").join("slot-v6.json");let original_slot=fs::read(&slot).unwrap();
        let slot_backup=dir.0.join("slots").join("archive-test").join("slot-v6.json.pre-map-archive-v1.bak");fs::write(&slot_backup,b"unrelated").unwrap();
        assert!(crate::save_slots::overwrite_slot_v6(&dir.0,"archive-test","Archive test",&next).is_err());
        assert_eq!(fs::read(&slot).unwrap(),original_slot);assert_eq!(fs::read(&slot_backup).unwrap(),b"unrelated");
        fs::remove_file(&slot_backup).unwrap();crate::save_slots::overwrite_slot_v6(&dir.0,"archive-test","Archive test",&next).unwrap();
        assert_eq!(fs::read(&slot_backup).unwrap(),original_slot);
        old_json.as_object_mut().unwrap().remove("effectSourcesVersion");
    }
    #[test]
    fn shaft_recipe_recovery_keeps_known_backups_and_rejects_residue() {
        let state=crate::formal_runtime::archive_test_initial_state().unwrap();let old=SaveV6::capture(&state,None).unwrap();
        let mut next=old.clone();next.world_persistent_v1.shaft_recipe_format_v2=Some(crate::moving_support::ShaftLowerRecipeIdentity{recipe_version:2,scene_source_sha256:"6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832".into()});
        let dir=TestDir::new();write_save(&dir.0,&old).unwrap();let target=save_path(&dir.0);let bytes=fs::read(&target).unwrap();
        let transaction=dir.0.join(".formal-save-v6.json.123.4.bak");fs::rename(&target,&transaction).unwrap();
        let retained=dir.0.join("formal-save-v6.json.pre-shaft-recipe-v2.bak");fs::write(&retained,b"invalid").unwrap();
        assert!(recovery::backup(&target,"E_SAVE").is_err());fs::write(&retained,&bytes).unwrap();
        fs::write(dir.0.join("formal-save-v6.json.pre-map-archive-v2.bak"),&bytes).unwrap();
        let residue=dir.0.join("formal-save-v6.json.unknown.bak");fs::write(&residue,&bytes).unwrap();assert!(recovery::backup(&target,"E_SAVE").is_err());fs::remove_file(residue).unwrap();
        let selected=recovery::backup(&target,"E_SAVE").unwrap().unwrap();assert_eq!(selected.bytes,bytes);write_recovered_save(&dir.0,&next,&selected).unwrap();assert_eq!(fs::read(&retained).unwrap(),bytes);
        #[cfg(windows)] {
            let link_dir=TestDir::new();let link_target=save_path(&link_dir.0);let ordinary=link_dir.0.join("ordinary.json");fs::write(&ordinary,&bytes).unwrap();
            let link=link_dir.0.join("formal-save-v6.json.pre-shaft-recipe-v2.bak");
            if std::os::windows::fs::symlink_file(&ordinary,&link).is_ok() {assert!(recovery::backup(&link_target,"E_SAVE").is_err());}
            else {eprintln!("fixture symlink unavailable; source guard only");}
        }
        crate::save_slots::create_slot_v6(&dir.0,"shaft-recover","Shaft recovery",&old).unwrap();let slot=dir.0.join("slots/shaft-recover/slot-v6.json");let slot_bytes=fs::read(&slot).unwrap();
        let transaction=dir.0.join("slots/shaft-recover/.slot-v6.json.123.4.bak");fs::rename(&slot,&transaction).unwrap();
        fs::write(dir.0.join("slots/shaft-recover/slot-v6.json.pre-map-archive-v1.bak"),&slot_bytes).unwrap();
        let selected=recovery::backup(&slot,"E_SLOT").unwrap().unwrap();crate::save_slots::write_continued_slot_v6(&dir.0,"shaft-recover","Shaft recovery",&next,Some(&selected)).unwrap();
        assert_eq!(fs::read(dir.0.join("slots/shaft-recover/slot-v6.json.pre-shaft-recipe-v2.bak")).unwrap(),slot_bytes);
    }
    #[test]
    fn shaft_recipe_format_marker_preserves_old_shape_and_upgrade_backup() {
        let state=crate::formal_runtime::archive_test_initial_state().unwrap();let old=SaveV6::capture(&state,None).unwrap();
        let old_json=serde_json::to_value(&old).unwrap();assert!(old_json["worldPersistentV1"].get("shaftRecipeFormatV2").is_none());
        let mut null=old_json.clone();null["worldPersistentV1"]["shaftRecipeFormatV2"]=serde_json::Value::Null;
        assert!(decode_bytes(&serde_json::to_vec(&null).unwrap()).is_err());
        let marker=crate::moving_support::ShaftLowerRecipeIdentity{recipe_version:2,scene_source_sha256:"6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832".into()};
        let mut next=old.clone();next.world_persistent_v1.shaft_recipe_format_v2=Some(marker);next.validate().unwrap();
        let mut legacy=serde_json::to_value(&next).unwrap();legacy.as_object_mut().unwrap().remove("effectSourcesVersion");assert!(decode_bytes(&serde_json::to_vec(&legacy).unwrap()).is_err());
        let mut invalid=next.clone();invalid.world_persistent_v1.shaft_recipe_format_v2.as_mut().unwrap().recipe_version=3;assert!(invalid.validate().is_err());
        let mut mismatch=old.clone();mismatch.save.moving_support_frame=Some(crate::moving_support::SupportFrame{
            world_epoch:mismatch.save.revision.world_epoch,server_time_ms:mismatch.save.server_time_ms,catalog_sha256:"0".repeat(64),
            poses:vec![crate::formal_runtime::shaft_lift::ShaftClock::at_upper(mismatch.save.server_time_ms).pose()],
            shaft_lower_recipe:next.world_persistent_v1.shaft_recipe_format_v2.clone()});
        assert_eq!(mismatch.validate().unwrap_err(),"E_SAVE_SHAFT_RECIPE_FORMAT_MISMATCH");
        let dir=TestDir::new();write_save(&dir.0,&old).unwrap();let target=save_path(&dir.0);let original=fs::read(&target).unwrap();let backup=dir.0.join("formal-save-v6.json.pre-shaft-recipe-v2.bak");
        fs::write(&backup,b"unrelated").unwrap();write_save(&dir.0,&next).unwrap();assert_eq!(fs::read(&backup).unwrap(),b"unrelated");
        let numbered=dir.0.join("formal-save-v6.json.pre-shaft-recipe-v2.1.bak");assert_eq!(fs::read(&numbered).unwrap(),original);
        write_save(&dir.0,&next).unwrap();assert!(!dir.0.join("formal-save-v6.json.pre-shaft-recipe-v2.2.bak").exists());
        let mut another=old.clone();another.save.server_time_ms+=1;write_save(&dir.0,&another).unwrap();let second_original=fs::read(&target).unwrap();
        let mut next_journey=another.clone();next_journey.world_persistent_v1.shaft_recipe_format_v2=next.world_persistent_v1.shaft_recipe_format_v2.clone();write_save(&dir.0,&next_journey).unwrap();
        assert_eq!(fs::read(dir.0.join("formal-save-v6.json.pre-shaft-recipe-v2.2.bak")).unwrap(),second_original);assert_eq!(fs::read(&numbered).unwrap(),original);
        crate::save_slots::create_slot_v6(&dir.0,"shaft-recipe","Shaft recipe",&old).unwrap();let slot=dir.0.join("slots/shaft-recipe/slot-v6.json");let old_slot=fs::read(&slot).unwrap();let slot_backup=dir.0.join("slots/shaft-recipe/slot-v6.json.pre-shaft-recipe-v2.bak");
        fs::write(&slot_backup,b"unrelated").unwrap();crate::save_slots::overwrite_slot_v6(&dir.0,"shaft-recipe","Shaft recipe",&next).unwrap();assert_eq!(fs::read(&slot_backup).unwrap(),b"unrelated");
        assert_eq!(fs::read(dir.0.join("slots/shaft-recipe/slot-v6.json.pre-shaft-recipe-v2.1.bak")).unwrap(),old_slot);
        crate::save_slots::overwrite_slot_v6(&dir.0,"shaft-recipe","Shaft recipe",&next).unwrap();assert!(!dir.0.join("slots/shaft-recipe/slot-v6.json.pre-shaft-recipe-v2.2.bak").exists());
        let parent="grey_hive.gh_central_shaft.lower_service_v2.pipe_fittings";
        let archive=crate::world_persistent_v1::LowerServiceMapArchive{schema_version:1,recipe:next.world_persistent_v1.shaft_recipe_format_v2.clone().unwrap(),records:vec![crate::world_persistent_v1::LowerServiceMapRecord{parent_id:parent.into(),rows:crate::formal_runtime::shaft_lower_service::candidate_rows(parent).unwrap()}]};
        let mut mapped=next_journey.clone();mapped.world_persistent_v1.lower_service_map_v1=Some(archive.clone());mapped.validate().unwrap();
        let mut no_marker=mapped.clone();no_marker.world_persistent_v1.shaft_recipe_format_v2=None;assert!(no_marker.validate().is_err());
        let mut duplicate=mapped.clone();duplicate.world_persistent_v1.lower_service_map_v1.as_mut().unwrap().records.push(archive.records[0].clone());assert!(duplicate.validate().is_err());
        let mut partial=mapped.clone();partial.world_persistent_v1.lower_service_map_v1.as_mut().unwrap().records[0].rows.pop();assert!(partial.validate().is_err());
        let mut unknown=mapped.clone();unknown.world_persistent_v1.lower_service_map_v1.as_mut().unwrap().records[0].parent_id="unknown".into();assert!(unknown.validate().is_err());
        let previous=fs::read(&target).unwrap();write_save(&dir.0,&mapped).unwrap();assert_eq!(fs::read(dir.0.join("formal-save-v6.json.pre-lower-service-map-v1.bak")).unwrap(),previous);
        let previous_slot=fs::read(&slot).unwrap();crate::save_slots::overwrite_slot_v6(&dir.0,"shaft-recipe","Shaft recipe",&mapped).unwrap();assert_eq!(fs::read(dir.0.join("slots/shaft-recipe/slot-v6.json.pre-lower-service-map-v1.bak")).unwrap(),previous_slot);
    }
    #[test]
    fn gh_archive_v2_preserves_v1_and_guards_default_and_slot_upgrades() {
        let state=crate::formal_runtime::archive_test_initial_state().unwrap();let mut old=SaveV6::capture(&state,None).unwrap();
        old.world_persistent_v1.map_knowledge_archive_v1=Some(crate::world_persistent_v1::MapKnowledgeArchiveV1{schema_version:1,records:vec![]});old.validate().unwrap();
        let mut next=old.clone();next.world_persistent_v1.map_knowledge_archive_v2=Some(crate::world_persistent_v1::GhMapKnowledgeArchiveV2{schema_version:2,records:vec![crate::formal_runtime::gh_map_archive::test_record()],shaft_upper_observation:None,lockdown_observation:None});next.validate().unwrap();
        let mut null=serde_json::to_value(&old).unwrap();null["worldPersistentV1"]["mapKnowledgeArchiveV2"]=serde_json::Value::Null;assert!(decode_bytes(&serde_json::to_vec(&null).unwrap()).is_err());
        let mut legacy_null=null.clone();legacy_null.as_object_mut().unwrap().remove("effectSourcesVersion");assert!(decode_bytes(&serde_json::to_vec(&legacy_null).unwrap()).is_err());
        let mut legacy=serde_json::to_value(&next).unwrap();legacy.as_object_mut().unwrap().remove("effectSourcesVersion");assert!(decode_bytes(&serde_json::to_vec(&legacy).unwrap()).is_err());
        let mut unknown=next.clone();unknown.world_persistent_v1.map_knowledge_archive_v2.as_mut().unwrap().schema_version=3;assert_eq!(unknown.validate().unwrap_err(),"E_SAVE_GH_MAP_ARCHIVE_VERSION_UNSUPPORTED");
        let dir=TestDir::new();write_save(&dir.0,&old).unwrap();let target=save_path(&dir.0);let original=fs::read(&target).unwrap();let backup=dir.0.join("formal-save-v6.json.pre-map-archive-v2.bak");
        fs::write(&backup,b"unrelated").unwrap();assert!(write_save(&dir.0,&next).is_err());assert_eq!(fs::read(&target).unwrap(),original);assert_eq!(fs::read(&backup).unwrap(),b"unrelated");
        fs::remove_file(&backup).unwrap();write_save(&dir.0,&next).unwrap();assert_eq!(fs::read(&backup).unwrap(),original);assert_eq!(read_save(&dir.0).unwrap(),next);
        crate::save_slots::create_slot_v6(&dir.0,"gh-archive","GH archive",&old).unwrap();let slot=dir.0.join("slots/gh-archive/slot-v6.json");let old_slot=fs::read(&slot).unwrap();let slot_backup=dir.0.join("slots/gh-archive/slot-v6.json.pre-map-archive-v2.bak");
        fs::write(&slot_backup,b"unrelated").unwrap();assert!(crate::save_slots::overwrite_slot_v6(&dir.0,"gh-archive","GH archive",&next).is_err());assert_eq!(fs::read(&slot).unwrap(),old_slot);assert_eq!(fs::read(&slot_backup).unwrap(),b"unrelated");
        fs::remove_file(&slot_backup).unwrap();crate::save_slots::overwrite_slot_v6(&dir.0,"gh-archive","GH archive",&next).unwrap();assert_eq!(fs::read(&slot_backup).unwrap(),old_slot);
        assert_eq!(next.world_persistent_v1.map_knowledge_archive_v1,old.world_persistent_v1.map_knowledge_archive_v1);
    }

    #[test]
    fn failed_commit_verification_rolls_back_exact_original_bytes() {
        let dir = TestDir::new();
        let target = dir.target();
        let old = b"  exact old bytes\n";
        fs::write(&target, old).unwrap();
        retain_legacy_backup(&target, old, "E_SAVE").unwrap();
        let calls = Cell::new(0);
        let result = atomic_write_verified(&target, b"new", Some(old), "E_SAVE", |_| {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                Err("injected postwrite verification failure".into())
            } else {
                Ok(())
            }
        });
        assert_eq!(
            result.unwrap_err(),
            "injected postwrite verification failure"
        );
        assert_eq!(fs::read(&target).unwrap(), old);
        assert_eq!(
            fs::read(dir.0.join("target.json.legacy-effect-sources-v1.bak")).unwrap(),
            old
        );
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 2);
    }

    #[test]
    fn failed_commit_rename_restores_original_and_retained_backup() {
        let dir = TestDir::new();
        let target = dir.target();
        let old = b"original";
        fs::write(&target, old).unwrap();
        retain_legacy_backup(&target, old, "E_SAVE").unwrap();
        let result = atomic_write_verified(&target, b"replacement", Some(old), "E_SAVE", |_| {
            fs::remove_file(dir.transient("tmp")).unwrap();
            Ok(())
        });
        assert!(result.unwrap_err().starts_with("E_SAVE_COMMIT:"));
        assert_eq!(fs::read(&target).unwrap(), old);
        assert_eq!(
            fs::read(dir.0.join("target.json.legacy-effect-sources-v1.bak")).unwrap(),
            old
        );
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 2);
    }

    #[test]
    fn failed_new_file_postwrite_verification_removes_failed_target() {
        let dir = TestDir::new();
        let calls = Cell::new(0);
        let result = atomic_write_verified(&dir.target(), b"new", None, "E_SAVE", |_| {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                Err("injected verification failure".into())
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        assert!(!dir.target().exists());
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
    }

    #[test]
    fn rollback_cleanup_failure_does_not_report_an_already_committed_write_as_failed() {
        let dir = TestDir::new();
        let target = dir.target();
        let old = b"old";
        fs::write(&target, old).unwrap();
        let calls = Cell::new(0);
        atomic_write_verified(&target, b"new", Some(old), "E_SAVE", |_| {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                // Inject a directory at the rollback path so remove_file fails cross-platform.
                let rollback = dir.transient("bak");
                fs::rename(&rollback, dir.0.join("preserved-original")).unwrap();
                fs::create_dir(rollback).unwrap();
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(target).unwrap(), b"new");
        assert_eq!(fs::read(dir.0.join("preserved-original")).unwrap(), old);
        assert!(dir.transient("bak").is_dir());
    }
}

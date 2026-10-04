//! Conservative, read-only discovery of interrupted V6 replacement transactions.
//! Canonical is always authoritative. A backup is only a Continue candidate.
use super::{path_present, read_bounded};
use std::{fs, path::{Path, PathBuf}};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Backup {
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
}

/// Check every existing path component without following links/reparse points.
/// Missing paths remain the caller's responsibility (including new-file writes).
pub(super) fn reject_links(path: &Path, prefix: &str) -> Result<(), String> {
    for component in path.ancestors() {
        match fs::symlink_metadata(component) {
            Ok(meta) => {
                #[cfg(windows)]
                let reparse = { use std::os::windows::fs::MetadataExt; meta.file_attributes() & 0x400 != 0 };
                #[cfg(not(windows))]
                let reparse = false;
                if meta.file_type().is_symlink() || reparse { return Err(format!("{prefix}_NOT_REGULAR")); }
                if component != path && !meta.is_dir() { return Err(format!("{prefix}_PATH_INVALID")); }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
            Err(e) => return Err(format!("{prefix}_READ: {e}")),
        }
    }
    Ok(())
}

fn same_filename(left: &str, right: &str) -> bool {
    #[cfg(windows)] { left.eq_ignore_ascii_case(right) }
    #[cfg(not(windows))] { left == right }
}
#[cfg(any(windows, test))]
fn windows_associated_variant(entry: &str, name: &str) -> bool {
    let lower = entry.to_ascii_lowercase(); let name = name.to_ascii_lowercase();
    lower.starts_with(&format!(".{name}.")) || lower.strip_prefix(&format!("{name}."))
        .is_some_and(|suffix| suffix.split('.').any(|part| matches!(part, "bak" | "tmp")))
}

fn transaction_suffix<'a>(entry: &'a str, name: &str) -> Option<&'a str> {
    entry.strip_prefix(&format!(".{name}."))
}
fn strict(suffix: &str, extension: &str) -> bool {
    let Some(numbers) = suffix.strip_suffix(extension) else { return false; };
    let mut parts = numbers.split('.');
    let (Some(pid), Some(counter), None) = (parts.next(), parts.next(), parts.next()) else { return false; };
    !pid.is_empty() && !counter.is_empty()
        && pid.bytes().all(|b| b.is_ascii_digit()) && counter.bytes().all(|b| b.is_ascii_digit())
        && pid.parse::<u32>().is_ok() && counter.parse::<u64>().is_ok()
}

pub(crate) fn backup(target: &Path, prefix: &str) -> Result<Option<Backup>, String> {
    reject_links(target, prefix)?;
    if path_present(target) { return Ok(None); }
    let parent = target.parent().ok_or("E_SAVE_PATH_INVALID")?;
    let name = target.file_name().and_then(|s| s.to_str()).ok_or("E_SAVE_PATH_INVALID")?;
    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{prefix}_RECOVERY_READ: {e}")),
    };
    let mut backups = Vec::new(); let mut temporary = false; let mut suspected = false;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{prefix}_RECOVERY_READ: {e}"))?;
        let file_name = entry.file_name(); let entry_name = file_name.to_string_lossy();
        // These separately retained migration backups never belong to this protocol.
        if same_filename(&entry_name, &format!("{name}.bak")) || same_filename(&entry_name, &format!("{name}.legacy-effect-sources-v1.bak")) { continue; }
        if let Some(suffix) = transaction_suffix(&entry_name, name) {
            if strict(suffix, ".bak") { backups.push(entry.path()); }
            else if strict(suffix, ".tmp") { temporary = true; }
            else { suspected = true; }
        } else if entry_name.strip_prefix(&format!("{name}."))
            .is_some_and(|suffix| suffix.split('.').any(|part| matches!(part, "bak" | "tmp"))) {
            // Clearly target-associated, but missing the protocol's leading dot.
            suspected = true;
        } else {
            #[cfg(windows)]
            if windows_associated_variant(&entry_name, name) { suspected = true; }
        }
    }
    // Count every strict backup, including corrupt/future/nonregular ones.
    if backups.len() > 1 { return Err(format!("{prefix}_RECOVERY_AMBIGUOUS")); }
    if suspected { return Err(format!("{prefix}_TRANSACTION_RESIDUE_SUSPECTED")); }
    if let Some(path) = backups.pop() {
        let bytes = read_bounded(&path, prefix, "E_SAVE_RECOVERY_SOURCE_MISSING")?;
        return Ok(Some(Backup { path, bytes }));
    }
    if temporary { return Err(format!("{prefix}_RECOVERY_REQUIRED")); }
    Ok(None)
}

pub(crate) fn require_unchanged(target: &Path, expected: &Backup, prefix: &str) -> Result<(), String> {
    if path_present(target) || backup(target, prefix)?.as_ref() != Some(expected) {
        return Err(format!("{prefix}_RECOVERY_SOURCE_CHANGED"));
    }
    Ok(())
}

/// Ordinary save/migration APIs may not silently replace interrupted transactions.
pub(crate) fn require_no_recovery(target: &Path, prefix: &str) -> Result<(), String> {
    if backup(target, prefix)?.is_some() { return Err(format!("{prefix}_RECOVERY_REQUIRED")); }
    Ok(())
}


#[cfg(test)]
mod filename_tests {
    use super::*;
    #[test]
    fn windows_case_variants_are_associated_but_never_strict_candidates() {
        assert!(windows_associated_variant(".SLOT-V6.JSON.12.2.BAK", "slot-v6.json"));
        assert!(windows_associated_variant("SLOT-V6.JSON.12.2.TMP.PARTIAL", "slot-v6.json"));
        assert!(!windows_associated_variant("unrelated.12.2.BAK", "slot-v6.json"));
        assert!(transaction_suffix(".SLOT-V6.JSON.12.2.BAK", "slot-v6.json").is_none());
        assert!(!strict("12.2.BAK", ".bak"));
    }
}

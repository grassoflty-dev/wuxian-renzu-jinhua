//! Offline, pre-start extraction of compiled content. No runtime, save, IPC or network access.
//! All payload names are generated here, never interpreted as filesystem paths from an asset.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub const MAX_FILES: usize = 4096;
pub const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_EXE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Gameplay,
    Export(PathBuf),
}

/// Only the one explicit offline mode accepts arguments. Never execute supplied commands.
pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Mode, String> {
    let args: Vec<_> = args.into_iter().collect();
    if args.is_empty() {
        return Ok(Mode::Gameplay);
    }
    if args.len() != 2 || args[0] != "--export-release-audit" {
        return Err("E_AUDIT_ARGUMENTS".into());
    }
    let path = PathBuf::from(&args[1]);
    validate_output_path(&path)?;
    Ok(Mode::Export(path))
}

fn validate_output_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.as_os_str().len() > 4096 || path.file_name().is_none() {
        return Err("E_AUDIT_OUTPUT_PATH".into());
    }
    for part in path.components() {
        match part {
            Component::Normal(name)
                if name
                    .to_str()
                    .is_some_and(|s| !s.is_empty() && !s.contains([':', '\0'])) => {}
            Component::RootDir => {}
            #[cfg(windows)]
            Component::Prefix(prefix) if matches!(prefix.kind(), std::path::Prefix::Disk(_)) => {}
            _ => return Err("E_AUDIT_OUTPUT_PATH".into()),
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    WebAsset,
    CompiledScene,
    WindowIconRgba,
}

pub struct Input<'a> {
    pub logical_path: String,
    pub role: Role,
    pub bytes: Cow<'a, [u8]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRow {
    pub logical_path: String,
    pub role: Role,
    pub storage_name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Executable {
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeConfig {
    pub has_external_resources: bool,
    pub has_external_binaries: bool,
    pub embedded_frontend: bool,
    pub icon_width: u32,
    pub icon_height: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub schema_id: &'static str,
    pub native_identity: serde_json::Value,
    pub executable: Executable,
    pub native_config: NativeConfig,
    pub files: Vec<FileRow>,
    pub total_bytes: u64,
    pub public_release_approved: bool,
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn executable_identity(path: &Path) -> Result<Executable, String> {
    let mut file = File::open(path).map_err(|_| "E_AUDIT_EXECUTABLE_OPEN")?;
    let metadata = file.metadata().map_err(|_| "E_AUDIT_EXECUTABLE_METADATA")?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_EXE_BYTES {
        return Err("E_AUDIT_EXECUTABLE_SIZE".into());
    }
    let mut hasher = Sha256::new();
    let mut block = [0; 64 * 1024];
    let mut size = 0_u64;
    loop {
        let count = file
            .read(&mut block)
            .map_err(|_| "E_AUDIT_EXECUTABLE_READ")?;
        if count == 0 {
            break;
        }
        size = size
            .checked_add(count as u64)
            .ok_or("E_AUDIT_EXECUTABLE_SIZE")?;
        if size > MAX_EXE_BYTES {
            return Err("E_AUDIT_EXECUTABLE_SIZE".into());
        }
        hasher.update(&block[..count]);
    }
    if size != metadata.len() {
        return Err("E_AUDIT_EXECUTABLE_CHANGED".into());
    }
    Ok(Executable {
        size_bytes: size,
        sha256: format!("{:x}", hasher.finalize()),
    })
}

fn valid_logical_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 512
        && path.is_ascii()
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// Capture only bytes passed from compiled content. The sole input file read is current_exe,
/// supplied by main, and all output files use create-new semantics in an anchored directory.
/// On error a partial directory can remain, but capture.json is written only after completion.
pub fn export<'a>(
    output: &Path,
    executable: &Path,
    native_identity: serde_json::Value,
    native_config: NativeConfig,
    inputs: impl IntoIterator<Item = Result<Input<'a>, String>>,
) -> Result<Capture, String> {
    validate_output_path(output)?;
    let executable_before = executable_identity(executable)?;
    let directory = secure_output::Directory::create(output)?;
    let mut files = Vec::new();
    let mut identities = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for input in inputs {
        let input = input?;
        if files.len() >= MAX_FILES || input.bytes.len() > MAX_FILE_BYTES {
            return Err("E_AUDIT_LIMIT".into());
        }
        if !valid_logical_path(&input.logical_path)
            || !identities.insert(input.logical_path.to_ascii_lowercase())
        {
            return Err("E_AUDIT_ASSET_PATH_OR_DUPLICATE".into());
        }
        total_bytes = total_bytes
            .checked_add(input.bytes.len() as u64)
            .ok_or("E_AUDIT_LIMIT")?;
        if total_bytes > MAX_TOTAL_BYTES {
            return Err("E_AUDIT_LIMIT".into());
        }
        let storage_name = format!("payload-{:04}.bin", files.len());
        directory.write_new(&storage_name, &input.bytes)?;
        files.push(FileRow {
            logical_path: input.logical_path,
            role: input.role,
            storage_name,
            size_bytes: input.bytes.len() as u64,
            sha256: sha(&input.bytes),
        });
    }
    if files.is_empty() {
        return Err("E_AUDIT_EMPTY".into());
    }
    files.sort_by(|a, b| a.logical_path.cmp(&b.logical_path));
    let after = executable_identity(executable)?;
    if after.sha256 != executable_before.sha256 || after.size_bytes != executable_before.size_bytes
    {
        return Err("E_AUDIT_EXECUTABLE_CHANGED".into());
    }
    let capture = Capture {
        schema_id: "native-release-capture/1",
        native_identity,
        executable: after,
        native_config,
        files,
        total_bytes,
        public_release_approved: false,
    };
    let bytes = serde_json::to_vec_pretty(&capture).map_err(|_| "E_AUDIT_REPORT_JSON")?;
    directory.write_new("capture.json", &bytes)?;
    Ok(capture)
}

#[cfg(target_os = "linux")]
mod secure_output {
    use super::*;
    use std::{
        ffi::CString,
        os::fd::{AsRawFd, FromRawFd},
        os::unix::ffi::OsStrExt,
    };
    const O_DIRECTORY: i32 = 0o200000;
    const O_NOFOLLOW: i32 = 0o400000;
    const O_CLOEXEC: i32 = 0o2000000;
    const O_WRONLY: i32 = 1;
    const O_CREAT: i32 = 0o100;
    const O_EXCL: i32 = 0o200;
    unsafe extern "C" {
        fn openat(dirfd: i32, path: *const std::ffi::c_char, flags: i32, ...) -> i32;
        fn mkdirat(dirfd: i32, path: *const std::ffi::c_char, mode: u32) -> i32;
    }
    pub struct Directory {
        handle: File,
    }
    impl Directory {
        pub fn create(path: &Path) -> Result<Self, String> {
            let mut parent = File::open("/").map_err(|_| "E_AUDIT_PARENT_OPEN")?;
            for component in path.parent().ok_or("E_AUDIT_OUTPUT_PATH")?.components() {
                if let Component::Normal(name) = component {
                    let name = CString::new(name.as_bytes()).map_err(|_| "E_AUDIT_OUTPUT_PATH")?;
                    let fd = unsafe {
                        openat(
                            parent.as_raw_fd(),
                            name.as_ptr(),
                            O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
                        )
                    };
                    if fd < 0 {
                        return Err("E_AUDIT_PARENT_NOT_SAFE_DIRECTORY".into());
                    }
                    parent = unsafe { File::from_raw_fd(fd) };
                }
            }
            let name = CString::new(path.file_name().ok_or("E_AUDIT_OUTPUT_PATH")?.as_bytes())
                .map_err(|_| "E_AUDIT_OUTPUT_PATH")?;
            if unsafe { mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
                return Err("E_AUDIT_OUTPUT_MUST_BE_NEW".into());
            }
            let fd = unsafe {
                openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err("E_AUDIT_OUTPUT_NOT_SAFE_DIRECTORY".into());
            }
            Ok(Self {
                handle: unsafe { File::from_raw_fd(fd) },
            })
        }
        pub fn write_new(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
            let name = CString::new(name).map_err(|_| "E_AUDIT_OUTPUT_NAME")?;
            let fd = unsafe {
                openat(
                    self.handle.as_raw_fd(),
                    name.as_ptr(),
                    O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC,
                    0o600u32,
                )
            };
            if fd < 0 {
                return Err("E_AUDIT_OUTPUT_CREATE".into());
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            file.write_all(bytes)
                .and_then(|_| file.sync_all())
                .map_err(|_| "E_AUDIT_OUTPUT_WRITE".into())
        }
    }
}

#[cfg(windows)]
mod secure_output {
    use super::*;
    use std::{
        fs::OpenOptions,
        os::windows::fs::{MetadataExt, OpenOptionsExt},
    };
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x02000000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x00200000;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    pub struct Directory {
        path: PathBuf,
        _locked_ancestors: Vec<File>,
    }
    fn lock_directory(path: &Path) -> Result<File, String> {
        // Deny both WRITE and DELETE sharing: rename and in-place reparse mutation
        // must not acquire a concurrent handle while an ancestor is held.
        let handle = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "E_AUDIT_DIRECTORY_LOCK")?;
        let meta = handle
            .metadata()
            .map_err(|_| "E_AUDIT_DIRECTORY_METADATA")?;
        if !meta.is_dir() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err("E_AUDIT_REPARSE_DIRECTORY".into());
        }
        Ok(handle)
    }
    impl Directory {
        pub fn create(path: &Path) -> Result<Self, String> {
            let mut current = PathBuf::new();
            let mut locks = Vec::new();
            for component in path.parent().ok_or("E_AUDIT_OUTPUT_PATH")?.components() {
                current.push(component);
                if current.is_absolute() {
                    locks.push(lock_directory(&current)?);
                }
            }
            std::fs::create_dir(path).map_err(|_| "E_AUDIT_OUTPUT_MUST_BE_NEW")?;
            locks.push(lock_directory(path)?);
            if std::fs::read_dir(path)
                .map_err(|_| "E_AUDIT_OUTPUT_READ")?
                .next()
                .is_some()
            {
                return Err("E_AUDIT_OUTPUT_NOT_EMPTY".into());
            }
            Ok(Self {
                path: path.to_owned(),
                _locked_ancestors: locks,
            })
        }
        pub fn write_new(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(self.path.join(name))
                .map_err(|_| "E_AUDIT_OUTPUT_CREATE")?;
            file.write_all(bytes)
                .and_then(|_| file.sync_all())
                .map_err(|_| "E_AUDIT_OUTPUT_WRITE".into())
        }
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod secure_output {
    use super::*;
    pub struct Directory;
    impl Directory {
        pub fn create(_: &Path) -> Result<Self, String> {
            Err("E_AUDIT_UNSUPPORTED_PLATFORM".into())
        }
        pub fn write_new(&self, _: &str, _: &[u8]) -> Result<(), String> {
            Err("E_AUDIT_UNSUPPORTED_PLATFORM".into())
        }
    }
}

//! The production offline helper, not a substitute DTO or runtime.
#[path = "../src/native_release_audit.rs"]
mod audit;
use audit::{export, parse_args, Input, Mode, NativeConfig, Role};
use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    exe: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "native-audit-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let exe = root.join("fixture.exe");
        fs::write(&exe, b"synthetic executable bytes, never executed").unwrap();
        Self { root, exe }
    }
    fn run(
        &self,
        output: &str,
        inputs: Vec<Result<Input<'_>, String>>,
    ) -> Result<audit::Capture, String> {
        export(
            &self.root.join(output),
            &self.exe,
            serde_json::json!({"test": true}),
            NativeConfig {
                has_external_resources: false,
                has_external_binaries: false,
                embedded_frontend: true,
                icon_width: 1,
                icon_height: 1,
            },
            inputs,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn input(path: &str, bytes: &[u8]) -> Result<Input<'static>, String> {
    Ok(Input {
        logical_path: path.into(),
        role: Role::WebAsset,
        bytes: Cow::Owned(bytes.into()),
    })
}

#[test]
fn only_explicit_bounded_mode_is_accepted() {
    assert_eq!(parse_args(Vec::new()).unwrap(), Mode::Gameplay);
    let root = Fixture::new();
    let output = root.root.join("new");
    assert_eq!(
        parse_args([
            "--export-release-audit".into(),
            output.clone().into_os_string()
        ])
        .unwrap(),
        Mode::Export(output)
    );
    for args in [
        vec!["--export-release-audit"],
        vec!["--export-release-audit", "relative"],
        vec!["--run", "something"],
        vec!["--export-release-audit", "/tmp/new", "--grant-all"],
    ] {
        assert!(parse_args(args.into_iter().map(Into::into)).is_err());
    }
    assert!(parse_args([
        "--export-release-audit".into(),
        root.root.join("../escape").into_os_string()
    ])
    .is_err());
}

#[test]
fn exact_payloads_are_exported_flat_and_report_is_sorted_without_save_writes() {
    let f = Fixture::new();
    let capture = f
        .run(
            "output",
            vec![
                input("web/z.js", b"z"),
                input("web/a.js", b"a"),
                Ok(Input {
                    logical_path: "scenes/test.json".into(),
                    role: Role::CompiledScene,
                    bytes: Cow::Borrowed(b"{}"),
                }),
                Ok(Input {
                    logical_path: "native/default-window-icon.rgba".into(),
                    role: Role::WindowIconRgba,
                    bytes: Cow::Borrowed(&[0, 1, 2, 255]),
                }),
            ],
        )
        .unwrap();
    assert_eq!(capture.files.len(), 4);
    assert_eq!(capture.total_bytes, 8);
    assert!(!capture.public_release_approved);
    assert_eq!(
        capture.files[0].logical_path,
        "native/default-window-icon.rgba"
    );
    assert_eq!(
        fs::read(f.root.join("output/payload-0000.bin")).unwrap(),
        b"z"
    );
    assert_eq!(
        fs::read(f.root.join("output/payload-0001.bin")).unwrap(),
        b"a"
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(f.root.join("output/capture.json")).unwrap()).unwrap();
    assert_eq!(report["schemaId"], "native-release-capture/1");
    assert_eq!(
        report["files"][3]["sha256"],
        "594e519ae499312b29433b7dd8a97ff068defcba9755b6d5d00e84c524d67b06"
    );
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 2);
    assert!(!f.root.join("data").exists());
}

#[test]
fn existing_output_is_never_reused_or_overwritten() {
    let f = Fixture::new();
    fs::create_dir(f.root.join("output")).unwrap();
    fs::write(f.root.join("output/sentinel"), b"preserve").unwrap();
    assert_eq!(
        f.run("output", vec![input("web/index.js", b"x")])
            .unwrap_err(),
        "E_AUDIT_OUTPUT_MUST_BE_NEW"
    );
    assert_eq!(
        fs::read(f.root.join("output/sentinel")).unwrap(),
        b"preserve"
    );
    assert_eq!(fs::read_dir(f.root.join("output")).unwrap().count(), 1);
}

#[test]
fn duplicate_escaping_or_failed_asset_never_produces_success_report() {
    for (index, inputs) in [
        vec![input("web/A.js", b"a"), input("web/a.js", b"b")],
        vec![input("web/../escape", b"x")],
        vec![input("/absolute", b"x")],
        vec![input("web/a\\b", b"x")],
        vec![input("web/ok", b"ok"), Err("E_DECODE".into())],
        vec![],
    ]
    .into_iter()
    .enumerate()
    {
        let f = Fixture::new();
        let name = format!("output{index}");
        assert!(f.run(&name, inputs).is_err());
        assert!(!f.root.join(name).join("capture.json").exists());
    }
}

#[test]
fn bounded_file_count_and_size_fail_before_success_report() {
    let f = Fixture::new();
    let inputs = (0..=audit::MAX_FILES)
        .map(|n| input(&format!("web/{n}"), b"x"))
        .collect();
    assert_eq!(f.run("many", inputs).unwrap_err(), "E_AUDIT_LIMIT");
    assert!(!f.root.join("many/capture.json").exists());
    let too_big = vec![0; audit::MAX_FILE_BYTES + 1];
    assert_eq!(
        f.run("big", vec![input("web/big", &too_big)]).unwrap_err(),
        "E_AUDIT_LIMIT"
    );
    assert!(!f.root.join("big/capture.json").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn symlink_parent_and_output_are_rejected() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let elsewhere = f.root.join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    symlink(&elsewhere, f.root.join("alias")).unwrap();
    assert_eq!(
        f.run("alias/escaped", vec![input("web/x", b"x")])
            .unwrap_err(),
        "E_AUDIT_PARENT_NOT_SAFE_DIRECTORY"
    );
    assert!(!elsewhere.join("escaped").exists());
    symlink(&elsewhere, f.root.join("output")).unwrap();
    assert_eq!(
        f.run("output", vec![input("web/x", b"x")]).unwrap_err(),
        "E_AUDIT_OUTPUT_MUST_BE_NEW"
    );
    assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
}

#[cfg(windows)]
#[test]
fn directory_junction_is_rejected_without_writing_target() {
    let f = Fixture::new();
    let target = f.root.join("target");
    fs::create_dir(&target).unwrap();
    let alias = f.root.join("alias");
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&alias)
        .arg(&target)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(f.run("alias/escaped", vec![input("web/x", b"x")]).is_err());
    assert!(!target.join("escaped").exists());
    fs::remove_dir(alias).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn anchored_output_never_follows_a_path_replacement_between_payloads() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let output = f.root.join("output");
    let moved = f.root.join("moved");
    let elsewhere = f.root.join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    let inputs = std::iter::once_with(|| {
        fs::rename(&output, &moved).unwrap();
        symlink(&elsewhere, &output).unwrap();
        input("web/safe", b"compiled bytes")
    });
    export(
        &output,
        &f.exe,
        serde_json::json!({"test":true}),
        NativeConfig {
            has_external_resources: false,
            has_external_binaries: false,
            embedded_frontend: true,
            icon_width: 1,
            icon_height: 1,
        },
        inputs,
    )
    .unwrap();
    assert!(moved.join("capture.json").is_file());
    assert_eq!(
        fs::read(moved.join("payload-0000.bin")).unwrap(),
        b"compiled bytes"
    );
    assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
}

#[cfg(windows)]
#[test]
fn locked_output_refuses_rename_during_capture() {
    let f = Fixture::new();
    let output = f.root.join("output");
    let inputs = std::iter::once_with(|| {
        assert!(fs::rename(&output, f.root.join("moved")).is_err());
        input("web/safe", b"compiled bytes")
    });
    export(
        &output,
        &f.exe,
        serde_json::json!({"test":true}),
        NativeConfig {
            has_external_resources: false,
            has_external_binaries: false,
            embedded_frontend: true,
            icon_width: 1,
            icon_height: 1,
        },
        inputs,
    )
    .unwrap();
    assert!(output.join("capture.json").is_file());
}

#[test]
fn repeated_capture_of_same_compiled_inputs_has_identical_report_bytes() {
    let f = Fixture::new();
    f.run(
        "first",
        vec![input("web/a.js", b"a"), input("web/b.js", b"b")],
    )
    .unwrap();
    f.run(
        "second",
        vec![input("web/a.js", b"a"), input("web/b.js", b"b")],
    )
    .unwrap();
    assert_eq!(
        fs::read(f.root.join("first/capture.json")).unwrap(),
        fs::read(f.root.join("second/capture.json")).unwrap()
    );
}

#[cfg(windows)]
#[test]
fn write_sharing_denial_blocks_in_place_junction_mutation() {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, path::Path};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *const c_void,
            disposition: u32,
            flags: u32,
            template: *mut c_void,
        ) -> *mut c_void;
        fn DeviceIoControl(
            handle: *mut c_void,
            control: u32,
            input: *const c_void,
            input_len: u32,
            output: *mut c_void,
            output_len: u32,
            returned: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    fn mutate(path: &Path, target: &Path) -> Result<bool, i32> {
        let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                0x4000_0000,
                7,
                std::ptr::null(),
                3,
                0x0220_0000,
                std::ptr::null_mut(),
            )
        };
        if handle as isize == -1 {
            return Err(std::io::Error::last_os_error().raw_os_error().unwrap());
        }
        let substitute: Vec<u16> = format!("\\??\\{}", target.display())
            .encode_utf16()
            .collect();
        let print: Vec<u16> = target.as_os_str().encode_wide().collect();
        let data_len = 8 + (substitute.len() + print.len() + 2) * 2;
        let mut buffer = Vec::new();
        buffer.extend(0xa000_0003_u32.to_le_bytes());
        buffer.extend((data_len as u16).to_le_bytes());
        buffer.extend(0_u16.to_le_bytes());
        buffer.extend(0_u16.to_le_bytes());
        buffer.extend(((substitute.len() * 2) as u16).to_le_bytes());
        buffer.extend(((substitute.len() * 2 + 2) as u16).to_le_bytes());
        buffer.extend(((print.len() * 2) as u16).to_le_bytes());
        for code in substitute
            .into_iter()
            .chain(Some(0))
            .chain(print)
            .chain(Some(0))
        {
            buffer.extend(code.to_le_bytes());
        }
        let mut returned = 0;
        let result = unsafe {
            DeviceIoControl(
                handle,
                0x0009_00a4,
                buffer.as_ptr().cast(),
                buffer.len() as u32,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        unsafe {
            CloseHandle(handle);
        }
        Ok(result != 0)
    }
    let f = Fixture::new();
    let target = f.root.join("mutation-target");
    fs::create_dir(&target).unwrap();
    let control = f.root.join("unlocked-control");
    fs::create_dir(&control).unwrap();
    assert_eq!(
        mutate(&control, &target),
        Ok(true),
        "control proves the OS permits in-place junction mutation without our locks"
    );
    fs::remove_dir(&control).unwrap();
    let output = f.root.join("locked-output");
    let inputs = std::iter::once_with(|| {
        assert_eq!(
            mutate(&output, &target),
            Err(32),
            "FILE_SHARE_READ only must reject a concurrent GENERIC_WRITE handle"
        );
        input("web/safe", b"compiled bytes")
    });
    export(
        &output,
        &f.exe,
        serde_json::json!({"test":true}),
        NativeConfig {
            has_external_resources: false,
            has_external_binaries: false,
            embedded_frontend: true,
            icon_width: 1,
            icon_height: 1,
        },
        inputs,
    )
    .unwrap();
    assert!(output.join("capture.json").is_file());
    assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
}

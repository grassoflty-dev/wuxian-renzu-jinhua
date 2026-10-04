#[path = "../build_support/bundle_identity.rs"]
mod bundle_identity;

use bundle_identity::{
    current_identity, serialize_sidecar, validate_dist, validate_dist_with_profile,
    BuildIdentity, FileEntry, Sidecar, SIDECAR_FILE_NAME,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        for _ in 0..100 {
            let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "wuxian-bundle-identity-{label}-{}-{id}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create isolated bundle identity fixture directory: {error}"),
            }
        }
        panic!("could not allocate an exclusive bundle identity fixture directory");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let temp_root = std::env::temp_dir();
        if self.0.parent() == Some(temp_root.as_path())
            && self
                .0
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("wuxian-bundle-identity-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

struct FixtureLink(PathBuf);

impl FixtureLink {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Drop for FixtureLink {
    fn drop(&mut self) {
        // Remove only the test-created link itself; never recurse into its target.
        #[cfg(windows)]
        let _ = fs::remove_dir(&self.0);
        #[cfg(unix)]
        let _ = fs::remove_file(&self.0);
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server-rs has a repository parent")
        .to_path_buf()
}

fn cargo_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn source_identity() -> BuildIdentity {
    current_identity(&repo_root(), cargo_version()).expect("read current source identity")
}

fn file_entry(path: &str, bytes: &[u8]) -> FileEntry {
    FileEntry {
        path: path.into(),
        size_bytes: bytes.len() as u64,
        sha256: bundle_identity::sha256(bytes),
    }
}

fn write_fixture(root: &Path, build_identity: BuildIdentity) -> Sidecar {
    fs::create_dir_all(root.join("assets")).expect("create fixture assets directory");
    let css = b"body{color:#ddd}\n";
    let js = b"export const fixture = true;\n";
    fs::write(root.join("assets/theme.css"), css).expect("write fixture style");
    fs::write(root.join("index.js"), js).expect("write fixture entry");
    let files = vec![
        file_entry("assets/theme.css", css),
        file_entry("index.js", js),
    ];
    let sidecar = Sidecar {
        schema_id: "native-web-bundle-identity/1".into(),
        schema_version: 1,
        build_identity,
        entry: files[1].clone(),
        files,
    };
    write_sidecar(root, &sidecar);
    sidecar
}

fn write_sidecar(root: &Path, sidecar: &Sidecar) {
    fs::write(
        root.join(SIDECAR_FILE_NAME),
        serialize_sidecar(sidecar).expect("serialize canonical sidecar"),
    )
    .expect("write fixture sidecar");
}

fn assert_rejected(root: &Path, code: &str) {
    let error = validate_dist(root, &repo_root(), cargo_version())
        .expect_err("invalid bundle must fail closed");
    assert!(error.starts_with(code), "expected {code}, got {error}");
}

fn copy_file_to_repo(source_root: &Path, fixture_root: &Path, relative: &str) {
    let source = source_root.join(relative);
    let destination = fixture_root.join(relative);
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::copy(source, destination).unwrap();
}

fn make_clean_repo(root: &Path) {
    let source_root = repo_root();
    for relative in [
        "apps/web/package.json",
        "server-rs/Cargo.toml",
        "server-rs/tauri.conf.json",
        "server-rs/src/save_v5.rs",
        "server-rs/src/save_v6.rs",
        "governance/assets/RUNTIME_ASSET_MANIFEST.json",
    ] {
        copy_file_to_repo(&source_root, root, relative);
    }
    let source_scenes = source_root.join("content/scenes/compiled");
    let destination_scenes = root.join("content/scenes/compiled");
    fs::create_dir_all(&destination_scenes).unwrap();
    for entry in fs::read_dir(source_scenes).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), destination_scenes.join(entry.file_name())).unwrap();
    }
    fs::create_dir_all(root.join("apps/web")).unwrap();
    fs::write(root.join(".gitignore"), "apps/web/dist/\n").unwrap();
    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .expect("run isolated fixture git command");
        assert!(status.success(), "git fixture command failed: {args:?}");
    };
    run_git(&["init", "--quiet"]);
    run_git(&["add", "--all"]);
    run_git(&[
        "-c",
        "user.name=Bundle Identity Fixture",
        "-c",
        "user.email=bundle-identity-fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "fixture sources",
    ]);
}

#[test]
fn current_web_dist_is_a_verified_positive_fixture() {
    let root = repo_root().join("apps/web/dist");
    let result = validate_dist_with_profile(&root, &repo_root(), cargo_version(), "debug")
        .expect("validate actual built dist in debug profile");
    assert_eq!(result.build_identity.git_sha.len(), 40);
    assert_eq!(result.sidecar_sha256.len(), 64);
    assert_eq!(
        result.entry.path.to_ascii_lowercase().ends_with(".js"),
        true
    );
    assert!(result.entry.size_bytes > 0);
    assert!(result.file_count > 1);
    assert_eq!(
        result.build_identity.source_tree_dirty,
        result.build_identity.has_tracked_diff || result.build_identity.has_untracked_files
    );
}

#[test]
fn clean_source_bundle_is_accepted_for_release_profile() {
    let temp = TempDir::new("release-clean");
    make_clean_repo(temp.path());
    let root = temp.path().join("apps/web/dist");
    let identity =
        current_identity(temp.path(), cargo_version()).expect("read clean fixture identity");
    assert!(!identity.has_tracked_diff);
    assert!(!identity.has_untracked_files);
    assert!(!identity.source_tree_dirty);
    write_fixture(&root, identity);

    let result = validate_dist_with_profile(&root, temp.path(), cargo_version(), "release")
        .expect("clean source identity is valid for release");
    assert!(!result.build_identity.source_tree_dirty);
}

#[test]
fn tracked_dirty_source_bundle_is_rejected_for_release_profile() {
    let temp = TempDir::new("release-tracked-dirty");
    make_clean_repo(temp.path());
    let tracked_file = temp.path().join("server-rs/src/save_v5.rs");
    let mut bytes = fs::read(&tracked_file).unwrap();
    bytes.extend_from_slice(b"\n// tracked-dirty profile fixture\n");
    fs::write(tracked_file, bytes).unwrap();

    let root = temp.path().join("apps/web/dist");
    let identity = current_identity(temp.path(), cargo_version()).expect("read dirty identity");
    assert!(identity.has_tracked_diff);
    assert!(!identity.has_untracked_files);
    assert!(identity.source_tree_dirty);
    write_fixture(&root, identity);

    let error = validate_dist_with_profile(&root, temp.path(), cargo_version(), "release")
        .expect_err("tracked dirty source must not enter a release build");
    assert!(error.starts_with("E_BUNDLE_IDENTITY_RELEASE_DIRTY"), "{error}");
}

#[test]
fn untracked_dirty_source_bundle_is_rejected_for_release_profile() {
    let temp = TempDir::new("release-untracked-dirty");
    make_clean_repo(temp.path());
    fs::write(temp.path().join("release-fixture-untracked.txt"), b"untracked\n").unwrap();

    let root = temp.path().join("apps/web/dist");
    let identity = current_identity(temp.path(), cargo_version()).expect("read dirty identity");
    assert!(!identity.has_tracked_diff);
    assert!(identity.has_untracked_files);
    assert!(identity.source_tree_dirty);
    write_fixture(&root, identity);

    let error = validate_dist_with_profile(&root, temp.path(), cargo_version(), "release")
        .expect_err("untracked dirty source must not enter a release build");
    assert!(error.starts_with("E_BUNDLE_IDENTITY_RELEASE_DIRTY"), "{error}");
}

#[test]
fn dirty_debug_identity_is_accepted_and_preserved() {
    let temp = TempDir::new("debug-untracked-dirty");
    make_clean_repo(temp.path());
    fs::write(temp.path().join("debug-fixture-untracked.txt"), b"untracked\n").unwrap();

    let root = temp.path().join("apps/web/dist");
    let identity = current_identity(temp.path(), cargo_version()).expect("read dirty identity");
    assert!(!identity.has_tracked_diff);
    assert!(identity.has_untracked_files);
    assert!(identity.source_tree_dirty);
    write_fixture(&root, identity);

    let result = validate_dist_with_profile(&root, temp.path(), cargo_version(), "debug")
        .expect("dirty debug identity remains buildable");
    assert!(!result.build_identity.has_tracked_diff);
    assert!(result.build_identity.has_untracked_files);
    assert!(result.build_identity.source_tree_dirty);
}

#[test]
fn unknown_build_profile_fails_closed() {
    let temp = TempDir::new("unknown-profile");
    make_clean_repo(temp.path());
    let root = temp.path().join("apps/web/dist");
    let identity =
        current_identity(temp.path(), cargo_version()).expect("read clean fixture identity");
    write_fixture(&root, identity);

    for profile in ["", "release-candidate"] {
        let error = validate_dist_with_profile(&root, temp.path(), cargo_version(), profile)
            .expect_err("unknown or empty profile must fail closed");
        assert!(error.starts_with("E_BUNDLE_IDENTITY_PROFILE:"), "{error}");
    }
}

#[test]
fn clean_source_checkout_accepts_a_clean_bundle_identity() {
    let temp = TempDir::new("clean-source");
    make_clean_repo(temp.path());
    let root = temp.path().join("apps/web/dist");
    let identity =
        current_identity(temp.path(), cargo_version()).expect("read clean fixture identity");
    assert!(!identity.has_tracked_diff);
    assert!(!identity.has_untracked_files);
    assert!(!identity.source_tree_dirty);
    write_fixture(&root, identity);
    let result =
        validate_dist(&root, temp.path(), cargo_version()).expect("validate clean fixture bundle");
    assert!(!result.build_identity.source_tree_dirty);
}

#[test]
fn fixture_rejects_bad_schema_noncanonical_json_and_unsafe_paths() {
    let temp = TempDir::new("schema");
    let root = temp.path().join("dist");
    let mut sidecar = write_fixture(&root, source_identity());
    sidecar.schema_id = "wrong-schema".into();
    write_sidecar(&root, &sidecar);
    assert_rejected(&root, "E_BUNDLE_IDENTITY_SCHEMA");

    sidecar.schema_id = "native-web-bundle-identity/1".into();
    sidecar.files[0].path = "../outside.css".into();
    write_sidecar(&root, &sidecar);
    assert_rejected(&root, "E_BUNDLE_IDENTITY_FILE_ENTRY");

    let good = write_fixture(&root, source_identity());
    let mut bytes = serialize_sidecar(&good).unwrap();
    bytes.insert(0, b' ');
    fs::write(root.join(SIDECAR_FILE_NAME), bytes).unwrap();
    assert_rejected(&root, "E_BUNDLE_IDENTITY_NONCANONICAL");
}

#[test]
fn fixture_rejects_old_git_identity_and_wrong_entry() {
    let temp = TempDir::new("identity");
    let root = temp.path().join("dist");
    let mut sidecar = write_fixture(&root, source_identity());
    sidecar.build_identity.git_sha = "0000000000000000000000000000000000000000".into();
    write_sidecar(&root, &sidecar);
    assert_rejected(&root, "E_BUNDLE_IDENTITY_GIT_STALE");

    sidecar.build_identity = source_identity();
    sidecar.entry.path = "missing.js".into();
    write_sidecar(&root, &sidecar);
    assert_rejected(&root, "E_BUNDLE_IDENTITY_ENTRY_RECORD");
}

#[test]
fn fixture_rejects_missing_extra_and_rewritten_payload_files() {
    let temp = TempDir::new("closure");
    let root = temp.path().join("dist");
    let sidecar = write_fixture(&root, source_identity());
    fs::remove_file(root.join("assets/theme.css")).unwrap();
    assert_rejected(&root, "E_BUNDLE_IDENTITY_PAYLOAD_CLOSURE");

    write_fixture(&root, source_identity());
    fs::write(root.join("extra.bin"), b"not declared").unwrap();
    assert_rejected(&root, "E_BUNDLE_IDENTITY_PAYLOAD_CLOSURE");

    fs::remove_file(root.join("extra.bin")).unwrap();
    write_fixture(&root, sidecar.build_identity);
    fs::write(root.join("index.js"), b"rewritten entry bytes\n").unwrap();
    assert_rejected(&root, "E_BUNDLE_IDENTITY_PAYLOAD_CLOSURE");
}

#[test]
fn fixture_rejects_symlinked_payloads() {
    let temp = TempDir::new("link");
    let root = temp.path().join("dist");
    let outside = temp.path().join("outside");
    let _sidecar = write_fixture(&root, source_identity());
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("outside.css"), b"outside\n").unwrap();
    let link = root.join("assets/linked");
    let _link_guard = FixtureLink::new(link.clone());

    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        if symlink_dir(&outside, &link).is_err() {
            let status = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command"])
                .arg("New-Item -ItemType Junction -Path $env:WUXIAN_TEST_LINK -Target $env:WUXIAN_TEST_TARGET | Out-Null")
                .env("WUXIAN_TEST_LINK", &link)
                .env("WUXIAN_TEST_TARGET", &outside)
                .status()
                .expect("create Windows junction fixture");
            assert!(
                status.success(),
                "test environment must permit a symlink or junction fixture"
            );
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, root.join("assets/linked")).unwrap();
    }

    assert_rejected(&root, "E_BUNDLE_IDENTITY_LINK:");
}

#[test]
fn fixture_rejects_a_parent_junction_that_redirects_dist() {
    let temp = TempDir::new("parent-link");
    let real = temp.path().join("real");
    let real_dist = real.join("dist");
    write_fixture(&real_dist, source_identity());
    let alias = temp.path().join("alias");
    let _link_guard = FixtureLink::new(alias.clone());

    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        if symlink_dir(&real, &alias).is_err() {
            let status = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command"])
                .arg("New-Item -ItemType Junction -Path $env:WUXIAN_TEST_LINK -Target $env:WUXIAN_TEST_TARGET | Out-Null")
                .env("WUXIAN_TEST_LINK", &alias)
                .env("WUXIAN_TEST_TARGET", &real)
                .status()
                .expect("create Windows parent junction fixture");
            assert!(
                status.success(),
                "test environment must permit a symlink or junction fixture"
            );
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&real, &alias).unwrap();
    }

    assert_rejected(&alias.join("dist"), "E_BUNDLE_IDENTITY_DIST_PARENT_LINK:");
}

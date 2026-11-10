//! Test-only assist switch (human decision 2026-11-10 11:30, Lead §25.1).
//!
//! Default OFF. It turns on only when BOTH of these hold:
//!   1. `WUXIAN_TEST_ASSIST=invuln` is present in the environment of the
//!      isolated loopback authority (`wuxian_v1_authority`, `src/godot_main.rs`), and
//!   2. that authority was started with an explicit `--session-dir` namespace that
//!      lives outside the real user profile (`<profile>\AppData`).
//!
//! The released/native entry point (`src/main.rs`, the Tauri app) never calls
//! `activate`, so a normal player process cannot enable it even when the variable
//! is present in its environment.
//!
//! While active: the authoritative damage pipeline forces damage that would land
//! on the player to 0. Enemy statistics, spawn tables and AI are untouched (they
//! still attack and still apply their own states); only the player's incoming
//! damage is dropped, and it is dropped on the authority, not in the client.
//! Save data and the wire schema are not affected.
//!
//! Policy: this switch must never appear in release notes and must never be part
//! of an uploaded/published build description. Its record stays local.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Environment variable that requests the assist.
pub const ENV_VAR: &str = "WUXIAN_TEST_ASSIST";
/// The only supported mode.
pub const INVULN: &str = "invuln";

static ACTIVE: AtomicBool = AtomicBool::new(false);
static HITS: AtomicU64 = AtomicU64::new(0);
static DAMAGE: AtomicU64 = AtomicU64::new(0);

/// True when the authority must ignore damage aimed at the player.
#[inline]
pub fn invulnerable() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// Player damage after the assist rule (0 while the switch is active).
#[inline]
pub fn player_damage(applied: u32) -> u32 {
    if !invulnerable() {
        return applied;
    }
    if applied > 0 {
        let hits = HITS.fetch_add(1, Ordering::Relaxed) + 1;
        DAMAGE.fetch_add(u64::from(applied), Ordering::Relaxed);
        // Rate-limited: the first three, then every 100th, so an enemy crowd cannot
        // flood stderr while still proving that real attacks arrived.
        if hits <= 3 || hits % 100 == 0 {
            eprintln!("TEST-ASSIST absorbed {applied} dmg hit={hits}");
        }
    }
    0
}

fn normalised(value: &std::ffi::OsStr) -> String {
    value.to_string_lossy().replace('\\', "/").to_ascii_lowercase()
}

/// Pure isolation predicate, so it can be tested without touching the process env.
/// A session inside `<profile>/AppData` is the human's real area and is refused.
fn outside_real_profile(session: &str, profile: &str, user: &str) -> bool {
    let raw = session.to_ascii_lowercase();
    let mut roots: Vec<String> = Vec::new();
    if !profile.trim().is_empty() {
        roots.push(format!("{}/appdata", profile.trim_end_matches('/')));
    }
    if !user.trim().is_empty() {
        roots.push(format!("/users/{}/appdata", user.trim_matches('/')));
    }
    !roots
        .iter()
        .any(|root| raw == *root || raw.starts_with(&format!("{root}/")))
}

/// Called once by the isolated loopback authority after its own namespace checks.
/// Returns the active mode, or `None` when the switch is absent, unsupported or
/// refused. Any refusal is loud, because a silent refusal would look like a bug.
pub fn activate(session: &Path) -> Option<&'static str> {
    let requested = std::env::var(ENV_VAR).ok()?;
    let requested = requested.trim().to_owned();
    if requested.is_empty() {
        return None;
    }
    if requested != INVULN {
        eprintln!(
            "TEST-ASSIST OFF: {ENV_VAR}={requested} is not a supported mode (only `{INVULN}`)"
        );
        return None;
    }
    let profile = std::env::var_os("USERPROFILE").map(|v| normalised(&v)).unwrap_or_default();
    let user = std::env::var_os("USERNAME").map(|v| normalised(&v)).unwrap_or_default();
    if !outside_real_profile(&normalised(session.as_os_str()), &profile, &user) {
        eprintln!(
            "TEST-ASSIST REFUSED: session dir {} is inside the real user profile; assist stays OFF",
            session.display()
        );
        return None;
    }
    ACTIVE.store(true, Ordering::SeqCst);
    eprintln!("==============================================================");
    eprintln!("TEST-ASSIST ACTIVE: invulnerable");
    eprintln!("  entry     : isolated loopback authority (explicit --session-dir)");
    eprintln!("  session   : {}", session.display());
    eprintln!("  effect    : authoritative damage to the player is forced to 0");
    eprintln!("  untouched : enemy stats, spawn counts, AI, save data, wire schema");
    eprintln!("  policy    : test only; never in release notes, never in uploads");
    eprintln!("==============================================================");
    Some(INVULN)
}

/// Loud one-line summary for the authority's shutdown (also proves the effect ran).
pub fn summary() -> Option<String> {
    if !invulnerable() {
        return None;
    }
    Some(format!(
        "TEST-ASSIST SUMMARY: invulnerable hits={} damage_nullified={}",
        HITS.load(Ordering::Relaxed),
        DAMAGE.load(Ordering::Relaxed)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_profile_paths_are_refused() {
        let profile = "c:/users/gwl";
        let user = "gwl";
        assert!(!outside_real_profile(
            "c:/users/gwl/appdata/roaming/wuxianv1campaign141",
            profile,
            user
        ));
        assert!(!outside_real_profile("c:/users/gwl/appdata", profile, user));
        assert!(!outside_real_profile("/users/gwl/appdata/local/wuxian", profile, user));
        // A sibling directory that merely starts with the same letters is NOT the
        // profile area, so it stays allowed; only the real AppData subtree is refused.
        assert!(outside_real_profile("c:/users/gwl/appdatax", profile, user));
    }

    #[test]
    fn isolated_namespaces_are_allowed() {
        let profile = "c:/users/gwl";
        let user = "gwl";
        assert!(outside_real_profile("c:/wuxianchief20/session20-deep20a", profile, user));
        assert!(outside_real_profile("d:/tmp/session/saves", profile, user));
    }

    #[test]
    fn inactive_by_default_and_damage_passes_through() {
        assert!(!invulnerable());
        assert_eq!(player_damage(7), 7);
        assert_eq!(summary(), None);
    }
}

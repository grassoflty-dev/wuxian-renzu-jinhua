#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    process::Command,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, production_scene_bootstrap};

const TEST_NAME: &str = "delayed_new_journey_reset_returns_after_owner_ticks";
const CHILD_MARKER: &str = "WUXIAN_DELAYED_NEW_JOURNEY_TEST_CHILD";

#[test]
fn delayed_new_journey_reset_returns_after_owner_ticks() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        let token = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let save_dir = std::env::temp_dir().join(format!(
            "wuxian-delayed-new-journey-{}-{token}",
            std::process::id()
        ));
        let runtime = FormalRuntime::new_with_save_dir(save_dir).expect("runtime");
        let startup = production_scene_bootstrap::install(&runtime).expect("production bundle");
        assert_eq!(startup.scene_id, "rs_core_room");
        println!("native-new-journey: production registry installed");
        thread::sleep(Duration::from_millis(300));
        assert_eq!(runtime.snapshot().expect("owner tick snapshot").scene_id, "rs_core_room");
        println!("native-new-journey: owner tick snapshot returned");
        let fresh = acknowledge_ready(&runtime, runtime.reset_new().expect("delayed reset"));
        assert_eq!(fresh.scene_id, "rs_core_room");
        assert_eq!(fresh.world_id, "return_station");
        println!("native-new-journey: delayed reset returned");
        return;
    }

    let mut child = Command::new(std::env::current_exe().expect("test binary path"))
        .arg("--exact")
        .arg(TEST_NAME)
        .arg("--nocapture")
        .env(CHILD_MARKER, "1")
        .spawn()
        .expect("spawn isolated delayed reset test");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.try_wait().expect("poll isolated test") {
            assert!(status.success(), "isolated delayed reset failed: {status}");
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("terminate only isolated test child");
            let _ = child.wait();
            panic!("delayed new journey reset exceeded 15 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
}

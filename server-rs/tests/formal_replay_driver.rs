use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use wuxian_horror_ch1::formal_runtime::FormalRuntime;

// Tested by the default server package and included by the feature-enabled CLI.
#[test]
fn ordinary_constructor_retains_independent_wall_clock_owner() {
    let runtime =
        FormalRuntime::new_with_save_dir(PathBuf::from("unused-live-clock-save-root")).unwrap();
    let initial = runtime.snapshot().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let next = runtime.snapshot().unwrap();
        if next.server_tick > initial.server_tick {
            assert!(next.authority_revision > initial.authority_revision);
            assert_eq!(next.ack_seq, 0);
            assert_eq!(
                next.player.transform.position_m,
                initial.player.transform.position_m
            );
            break;
        }
        assert!(
            Instant::now() < deadline,
            "ordinary runtime stopped advancing without input"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

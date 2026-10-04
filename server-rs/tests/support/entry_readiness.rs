//! Headless harness acknowledgement, explicitly standing in for the renderer's first-frame proof.
//! This is not renderer/native acceptance. It uses the public readiness command and never bypasses its checks.
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, world_v3::WorldView};

pub fn acknowledge_ready(runtime: &FormalRuntime, prepared: WorldView) -> WorldView {
    let token = prepared.entry_token.as_ref().expect("route must provide a prepared entry token");
    assert_eq!(token.world_id, prepared.world_id);
    assert_eq!(token.scene_id, prepared.scene_id);
    assert_eq!(token.world_epoch, prepared.world_epoch);
    let before = runtime.snapshot().expect("prepared owner snapshot");
    assert_eq!(before.entry_token.as_ref(), Some(token));
    assert_eq!(before.server_tick, prepared.server_tick, "owner stays paused before readiness");
    assert_eq!(runtime.resume().unwrap_err(), "E_SCENE_ENTRY_NOT_READY",
        "ordinary resume cannot bypass the preparation barrier");
    let still_paused = runtime.snapshot().expect("paused owner snapshot");
    assert_eq!(still_paused, before, "failed resume must not mutate prepared state");
    let ready = runtime.scene_ready(token, false).expect("explicit exact-token headless readiness");
    assert!(ready.entry_token.is_none());
    assert_eq!(ready.world_epoch, prepared.world_epoch);
    assert_eq!(ready.world_id, prepared.world_id);
    assert_eq!(ready.scene_id, prepared.scene_id);
    assert_eq!(ready.server_tick, before.server_tick);
    assert_eq!(ready.player.current_hp, before.player.current_hp);
    ready
}

//! A reserved alias must never silently select a different physical slot.
use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, production_scene_bootstrap as production, save_slots, save_v6};
static NEXT: AtomicU64 = AtomicU64::new(0);
const ID: &str = "legacy-save-v3";
struct Saves(PathBuf);
impl Saves {
    fn new() -> Self { Self(std::env::temp_dir().join(format!("reserved-slot-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)))) }
    fn runtime(&self) -> FormalRuntime { let r=FormalRuntime::new_with_save_dir(self.0.clone()).unwrap(); production::install(&r).unwrap(); r }
    fn seed(&self) -> FormalRuntime { let r=self.runtime(); let v=r.reset_new().unwrap(); r.scene_ready(v.entry_token.as_ref().unwrap(),true).unwrap(); r.save().unwrap(); r }
    fn reserved(&self) -> PathBuf { self.0.join("slots").join(ID) }
}
impl Drop for Saves { fn drop(&mut self) { let _=fs::remove_dir_all(&self.0); } }
#[test]
fn rejected_create_and_overwrite_do_not_mutate_files_or_runtime() {
    let s=Saves::new(); let r=s.seed(); let save=save_v6::read_save(&s.0).unwrap(); let before=fs::read(save_v6::save_path(&s.0)).unwrap(); let world=r.snapshot().unwrap();
    for create in [true,false] {
        assert_eq!(r.save_slot(ID,"Reserved",create).unwrap_err(),"E_SLOT_ID_RESERVED");
        assert_eq!(r.snapshot().unwrap(),world);
    }
    assert_eq!(save_slots::create_slot_v6(&s.0,ID,"Reserved",&save).unwrap_err(),"E_SLOT_ID_RESERVED");
    assert_eq!(save_slots::overwrite_slot_v6(&s.0,ID,"Reserved",&save).unwrap_err(),"E_SLOT_ID_RESERVED");
    assert_eq!(fs::read(save_v6::save_path(&s.0)).unwrap(),before); assert!(!s.0.join("slots").exists());
}
#[test]
fn every_physical_reserved_path_blocks_alias_without_modification() {
    for kind in ["valid","corrupt","future","directory","file","empty"] {
        let s=Saves::new(); let r=s.seed(); let before=fs::read(save_v6::save_path(&s.0)).unwrap(); let world=r.snapshot().unwrap();
        if kind=="file" { fs::create_dir_all(s.0.join("slots")).unwrap(); fs::write(s.reserved(),b"physical collision").unwrap(); }
        else {
            fs::create_dir_all(s.reserved()).unwrap(); let p=s.reserved().join("slot-v6.json");
            match kind {
                "valid"|"future" => { let mut save=save_v6::read_save(&s.0).unwrap(); save.save.player.current_hp=17; let v=serde_json::json!({"schemaVersion":if kind=="future"{999}else{3},"slotId":ID,"displayName":"Physical 17 HP","updatedAtMs":1,"save":save});fs::write(p,serde_json::to_vec(&v).unwrap()).unwrap(); }
                "corrupt" => fs::write(p,b"bad json").unwrap(),
                "directory" => fs::create_dir(p).unwrap(), _=>{}
            }
        }
        let rows=r.list_save_slots(); assert_eq!(rows.len(),1,"{kind}"); assert_eq!(rows[0].slot_id,ID); assert!(!rows[0].valid); assert_eq!(rows[0].error_code.as_deref(),Some("E_SLOT_ID_CONFLICT"));
        assert_eq!(r.continue_slot(ID).unwrap_err(),"E_SLOT_ID_CONFLICT"); assert_eq!(r.snapshot().unwrap(),world); assert_eq!(fs::read(save_v6::save_path(&s.0)).unwrap(),before);
        assert!(r.has_default_save()); assert_eq!(r.continue_saved().unwrap().player.current_hp,100);
    }
}
#[test]
fn alias_without_physical_collision_keeps_default_semantics() {
    let s=Saves::new(); let r=s.seed(); assert_eq!(r.continue_slot(ID).unwrap().player.current_hp,100); assert!(!s.reserved().exists());
}
#[cfg(unix)]
#[test]
fn dangling_reserved_link_is_still_a_conflict() {
    let s=Saves::new(); let r=s.seed(); fs::create_dir_all(s.0.join("slots")).unwrap(); std::os::unix::fs::symlink("missing",s.reserved()).unwrap();
    assert_eq!(r.continue_slot(ID).unwrap_err(),"E_SLOT_ID_CONFLICT"); assert_eq!(r.list_save_slots()[0].error_code.as_deref(),Some("E_SLOT_ID_CONFLICT")); assert_eq!(fs::read_link(s.reserved()).unwrap(),PathBuf::from("missing"));
}
#[cfg(windows)]
#[test]
fn windows_case_aliases_cannot_mint_or_continue_reserved_physical_slots() {
    let s=Saves::new(); let r=s.seed();
    for id in ["Legacy-Save-V3","LEGACY-SAVE-V3"] {
        assert_eq!(r.save_slot(id,"Reserved",true).unwrap_err(),"E_SLOT_ID_RESERVED");
        assert_eq!(r.save_slot(id,"Reserved",false).unwrap_err(),"E_SLOT_ID_RESERVED");
    }
    fs::create_dir_all(s.0.join("slots/Legacy-Save-V3")).unwrap();
    let rows=r.list_save_slots(); assert_eq!(rows.len(),1); assert!(!rows[0].valid);
    assert_eq!(r.continue_slot("Legacy-Save-V3").unwrap_err(),"E_SLOT_ID_CONFLICT");
}
#[test]
fn physical_collision_suppresses_real_legacy_alias_and_keeps_all_bytes() {
    let s=Saves::new(); let r=s.seed();
    fs::write(wuxian_horror_ch1::save_v3::save_path(&s.0),include_bytes!("fixtures/capability-save-profiles/legacy-v4.json")).unwrap();
    fs::create_dir_all(s.reserved()).unwrap(); let path=s.reserved().join("slot-v4.json");
    let v=serde_json::json!({"schemaVersion":1,"slotId":ID,"displayName":"reserved legacy","updatedAtMs":1,"save":serde_json::from_slice::<serde_json::Value>(include_bytes!("fixtures/capability-save-profiles/legacy-v4.json")).unwrap()});
    let bytes=serde_json::to_vec(&v).unwrap(); fs::write(&path,&bytes).unwrap();
    let rows=r.list_save_slots(); assert_eq!(rows.len(),1); assert!(!rows[0].valid); assert_eq!(rows[0].error_code.as_deref(),Some("E_SLOT_ID_CONFLICT"));
    assert_eq!(save_slots::read_slot(&s.0,ID).unwrap_err(),"E_SLOT_ID_RESERVED"); assert_eq!(save_slots::read_slot_v6(&s.0,ID).unwrap_err(),"E_SLOT_ID_RESERVED");
    assert_eq!(r.continue_slot(ID).unwrap_err(),"E_SLOT_ID_CONFLICT"); assert_eq!(fs::read(path).unwrap(),bytes); assert_eq!(fs::read_dir(s.reserved()).unwrap().count(),1);
}
#[test]
fn non_directory_slots_container_does_not_invent_a_physical_alias_collision() {
    let s=Saves::new();let r=s.seed(); fs::write(s.0.join("slots"),b"not a slot directory").unwrap();
    assert_eq!(r.continue_slot(ID).unwrap().player.current_hp,100); assert_eq!(fs::read(s.0.join("slots")).unwrap(),b"not a slot directory");
}

//! Constructed rename-gap fixtures on the host filesystem, not Windows crash/power-loss tests.
use std::{collections::BTreeMap, fs, path::{Path,PathBuf}, sync::atomic::{AtomicU64,Ordering}};
use sha2::{Digest,Sha256};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, production_scene_bootstrap as production, save_slots, save_v5, save_v6};
static NEXT: AtomicU64=AtomicU64::new(0);
struct Saves { root: PathBuf, named: bool }
impl Saves {
    fn new(named:bool)->Self { Self{root:std::env::temp_dir().join(format!("save-recovery-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed))),named} }
    fn runtime(&self)->FormalRuntime { let r=FormalRuntime::new_with_save_dir(self.root.clone()).unwrap();production::install(&r).unwrap();r }
    fn seed(&self)->FormalRuntime { let r=self.runtime();let v=r.reset_new().unwrap();r.scene_ready(v.entry_token.as_ref().unwrap(),true).unwrap(); if self.named {r.save_slot("named","Named",true).unwrap();}else{r.save().unwrap();} r }
    fn target(&self)->PathBuf {if self.named{self.root.join("slots/named/slot-v6.json")}else{save_v6::save_path(&self.root)}}
    fn artifact(&self,suffix:&str)->PathBuf { let p=self.target();p.with_file_name(format!(".{}.424242.{suffix}",p.file_name().unwrap().to_str().unwrap())) }
    fn gap(&self)->(PathBuf,PathBuf){let target=self.target();let old=fs::read(&target).unwrap();let mut v:serde_json::Value=serde_json::from_slice(&old).unwrap();if self.named{v["save"]["save"]["player"]["currentHp"]=17.into();}else{v["save"]["player"]["currentHp"]=17.into();}let tmp=self.artifact("1.tmp");fs::write(&tmp,serde_json::to_vec(&v).unwrap()).unwrap();let bak=self.artifact("1.bak");fs::rename(target,&bak).unwrap();(bak,tmp)}
    fn older(&self){let mut save=if self.named{save_slots::read_slot_v6(&self.root,"named").unwrap().1}else{save_v6::read_save(&self.root).unwrap()};save.save.player.current_hp=33;if self.named{let v=serde_json::json!({"schemaVersion":2,"slotId":"named","displayName":"old33","updatedAtMs":1,"save":save.save});fs::write(self.root.join("slots/named/slot-v5.json"),serde_json::to_vec(&v).unwrap()).unwrap();}else{save_v5::write_save(&self.root,&save.save).unwrap();}}
    fn continue_it(&self,r:&FormalRuntime)->Result<wuxian_horror_ch1::world_v3::WorldView,String>{if self.named{r.continue_slot("named")}else{r.continue_saved()}}
    fn available(&self,r:&FormalRuntime)->bool {if self.named{r.list_save_slots().iter().any(|s|s.slot_id=="named"&&s.valid)}else{r.has_default_save()}}
}
impl Drop for Saves{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
fn hashes(root:&Path)->BTreeMap<String,String>{fn visit(root:&Path,at:&Path,out:&mut BTreeMap<String,String>){if let Ok(entries)=fs::read_dir(at){for e in entries {let p=e.unwrap().path();let m=fs::symlink_metadata(&p).unwrap();let key=p.strip_prefix(root).unwrap().to_string_lossy().into_owned();if m.file_type().is_symlink(){out.insert(key,format!("link:{:?}",fs::read_link(p).unwrap()));}else if m.is_dir(){out.insert(key,"dir".into());visit(root,&p,out);}else{out.insert(key,format!("{:x}",Sha256::digest(fs::read(p).unwrap())));}}}}let mut out=BTreeMap::new();visit(root,root,&mut out);out}
#[test]
fn unique_backup_probes_are_read_only_and_explicit_continue_commits_once(){
 for named in [false,true]{for older in [false,true]{let s=Saves::new(named);let r=s.seed();if older{s.older();}let(bak,tmp)=s.gap();let before=hashes(&s.root);let world=r.snapshot().unwrap();
  for _ in 0..3 {assert!(s.available(&r));assert!(r.has_save());if named{let row=r.list_save_slots().into_iter().find(|s|s.slot_id=="named").unwrap();assert!(row.recoverable&&row.read_only);assert_eq!(row.current_hp,Some(100));}assert_eq!(hashes(&s.root),before);assert_eq!(r.snapshot().unwrap(),world);}
  assert_eq!(s.continue_it(&r).unwrap().player.current_hp,100);let after=hashes(&s.root);assert_eq!(after.len(),before.len()+1);for(path,hash)in &before{assert_eq!(after.get(path),Some(hash));}assert!(bak.exists()&&tmp.exists());
  for _ in 0..2{assert_eq!(s.continue_it(&r).unwrap().player.current_hp,100);assert_eq!(hashes(&s.root),after);}if named{assert!(!r.list_save_slots()[0].recoverable);}
 }}
}
#[test]
fn canonical_is_authoritative_even_with_ambiguous_artifacts(){
 for named in [false,true]{for kind in ["valid","corrupt","future","directory"]{let s=Saves::new(named);let r=s.seed();s.older();let raw=fs::read(s.target()).unwrap();for counter in [1,2]{fs::write(s.artifact(&format!("{counter}.bak")),&raw).unwrap();}
  match kind{"corrupt"=>fs::write(s.target(),b"bad json").unwrap(),"future"=>{let mut v:serde_json::Value=serde_json::from_slice(&raw).unwrap();v["schemaVersion"]=999.into();fs::write(s.target(),serde_json::to_vec(&v).unwrap()).unwrap();},"directory"=>{fs::remove_file(s.target()).unwrap();fs::create_dir(s.target()).unwrap();},_=>{}}
  let before=hashes(&s.root);if kind=="valid"{assert_eq!(s.continue_it(&r).unwrap().player.current_hp,100);}else{assert!(!s.available(&r));assert!(s.continue_it(&r).is_err());}assert_eq!(hashes(&s.root),before);
 }}
}
#[test]
fn multiple_backups_never_select_one_even_if_others_are_invalid(){
 for named in [false,true]{for other in ["valid","corrupt","future","directory"]{let s=Saves::new(named);let r=s.seed();s.older();let(bak,_)=s.gap();let second=s.artifact("2.bak");match other{"valid"=>{fs::copy(&bak,&second).unwrap();},"corrupt"=>fs::write(&second,b"bad").unwrap(),"future"=>{let mut v:serde_json::Value=serde_json::from_slice(&fs::read(bak).unwrap()).unwrap();v["schemaVersion"]=999.into();fs::write(&second,serde_json::to_vec(&v).unwrap()).unwrap();},_=>fs::create_dir(&second).unwrap()}
 let before=hashes(&s.root);assert!(!s.available(&r));assert!(s.continue_it(&r).unwrap_err().contains("RECOVERY_AMBIGUOUS"));assert_eq!(hashes(&s.root),before);assert!(!s.target().exists());
 }}
}
#[test]
fn tmp_only_and_related_malformed_residue_block_v5_but_unrelated_and_history_do_not(){
 for named in [false,true]{for pattern in ["tmp-only","missing-pid","bad-number","wrong-suffix","missing-dot","both-malformed","unrelated","history"]{let s=Saves::new(named);let r=s.seed();s.older();let(bak,tmp)=s.gap();fs::remove_file(&bak).unwrap();if pattern!="tmp-only"{fs::remove_file(&tmp).unwrap();let target=s.target();let name=target.file_name().unwrap().to_str().unwrap();let filename=match pattern{"missing-pid"=>format!(".{name}.bak"),"bad-number"=>format!(".{name}.pid.2.bak"),"wrong-suffix"=>format!(".{name}.12.2.bak.partial"),"missing-dot"=>format!("{name}.12.2.bak"),"both-malformed"=>format!("{name}.12.2.bak.partial"),"unrelated"=>"xxx.tmp".into(),_=>format!("{name}.legacy-effect-sources-v1.bak")};fs::write(target.with_file_name(filename),b"leave this evidence").unwrap();}
 let before=hashes(&s.root);if ["unrelated","history"].contains(&pattern){assert!(s.available(&r));assert_eq!(hashes(&s.root),before);assert_eq!(s.continue_it(&r).unwrap().player.current_hp,33);}else{assert!(!s.available(&r));assert!(s.continue_it(&r).is_err());assert_eq!(hashes(&s.root),before);assert!(!s.target().exists());}
 }}
}
#[test]
fn unique_backup_must_pass_every_existing_schema_identity_and_restore_check(){
 for named in [false,true]{for kind in ["bad-json","future","nested-future","schema","dead","scene","too-large","identity"]{if !named&&kind=="identity"{continue}let s=Saves::new(named);let r=s.seed();s.older();let(bak,_)=s.gap();let mut v:serde_json::Value=serde_json::from_slice(&fs::read(&bak).unwrap()).unwrap();
 match kind{"bad-json"=>fs::write(&bak,b"bad").unwrap(),"too-large"=>fs::write(&bak,vec![b' ';8*1024*1024+1]).unwrap(),_=>{let dto=if named{&mut v["save"]}else{&mut v};match kind{"future"=>dto["schemaVersion"]=999.into(),"nested-future"=>dto["save"]["schemaVersion"]=999.into(),"schema"=>{dto.as_object_mut().unwrap().remove("effectSources");},"dead"=>dto["save"]["player"]["currentHp"]=0.into(),"scene"=>dto["save"]["sceneId"]="missing-scene".into(),"identity"=>v["slotId"]="different".into(),_=>unreachable!()};fs::write(&bak,serde_json::to_vec(&v).unwrap()).unwrap();}}
 let before=hashes(&s.root);let world=r.snapshot().unwrap();assert!(!s.available(&r),"{named}/{kind}");assert!(s.continue_it(&r).is_err(),"{named}/{kind}");assert_eq!(hashes(&s.root),before);assert_eq!(r.snapshot().unwrap(),world);
 }}
}
#[test]
fn ordinary_read_migration_and_save_cannot_commit_over_pending_recovery(){
 for named in [false,true]{let s=Saves::new(named);let r=s.seed();s.older();let save=if named{save_slots::read_slot_v6(&s.root,"named").unwrap().1}else{save_v6::read_save(&s.root).unwrap()};s.gap();let before=hashes(&s.root);
 if named{assert_eq!(save_slots::read_slot_v6(&s.root,"named").unwrap().1,save);assert!(save_slots::overwrite_slot_v6(&s.root,"named","overwrite",&save).unwrap_err().contains("RECOVERY_REQUIRED"));}else{assert_eq!(save_v6::read_save(&s.root).unwrap(),save);assert!(save_v6::read_or_migrate(&s.root).unwrap_err().contains("RECOVERY_REQUIRED"));assert!(r.save().unwrap_err().contains("RECOVERY_REQUIRED"));}assert_eq!(hashes(&s.root),before);
 }
}
#[cfg(unix)]
#[test]
fn canonical_backup_and_parent_links_are_never_followed(){
 for named in [false,true]{for linked in ["canonical","backup","parent"]{let s=Saves::new(named);let r=s.seed();s.older();let(bak,_)=s.gap();if linked=="canonical"{std::os::unix::fs::symlink(&bak,s.target()).unwrap();}else if linked=="backup"{let held=s.root.join("held.json");fs::rename(&bak,&held).unwrap();std::os::unix::fs::symlink(&held,&bak).unwrap();}else if named{let real=s.root.join("real-slot");fs::rename(s.root.join("slots/named"),&real).unwrap();std::os::unix::fs::symlink(real,s.root.join("slots/named")).unwrap();}else{continue;}
 let before=hashes(&s.root);assert!(!s.available(&r));assert!(s.continue_it(&r).is_err());assert_eq!(hashes(&s.root),before);
 }}
}
#[test]
fn recovery_never_accepts_slot_path_traversal(){let s=Saves::new(true);let r=s.seed();s.gap();let before=hashes(&s.root);for id in ["../named","named/..","/named","named\\.."]{assert!(r.continue_slot(id).unwrap_err().contains("ID_INVALID"));}assert_eq!(hashes(&s.root),before);}
#[test]
fn legacy_alias_recovery_summary_comes_from_backup_not_older_single_file(){
 let s=Saves::new(false);let r=s.seed();fs::write(wuxian_horror_ch1::save_v3::save_path(&s.root),include_bytes!("fixtures/capability-save-profiles/legacy-v4.json")).unwrap();
 s.gap();let before=hashes(&s.root);let rows=r.list_save_slots();assert_eq!(rows.len(),1);let row=&rows[0];assert_eq!(row.slot_id,"legacy-save-v3");assert!(row.valid&&row.recoverable&&row.read_only);assert_eq!(row.current_hp,Some(100));assert_eq!(row.world_id.as_deref(),Some("return_station"));assert_eq!(hashes(&s.root),before);assert_eq!(r.continue_slot("legacy-save-v3").unwrap().player.current_hp,100);
}
#[cfg(windows)]
#[test]
fn windows_case_variant_transaction_residue_blocks_legacy_fallback(){
 for named in [false,true]{let s=Saves::new(named);let r=s.seed();s.older();let(bak,tmp)=s.gap();fs::remove_file(bak).unwrap();fs::remove_file(tmp).unwrap();let name=s.target().file_name().unwrap().to_str().unwrap().to_ascii_uppercase();fs::write(s.target().with_file_name(format!(".{name}.12.2.BAK")),b"unknown").unwrap();let before=hashes(&s.root);assert!(!s.available(&r));assert!(s.continue_it(&r).unwrap_err().contains("RESIDUE_SUSPECTED"));assert_eq!(hashes(&s.root),before);}
}

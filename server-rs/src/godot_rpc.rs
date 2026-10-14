//! Strict player-facing adapter. No debug, trusted-grant, replay or raw route endpoints.
use serde::{de::{self, MapAccess, SeqAccess, Visitor}, Deserialize, Deserializer, Serialize};
use serde_json::{json, Map, Value};
use std::fmt;
use wuxian_horror_ch1::{formal_runtime::{self, FormalRuntime, SessionContext}, scene_route_commands as routes, world_v3::{CommandReceipt, WorldSnapshot}};
pub const PROTOCOL: &str = "wuxian.godot-v1.rpc";
pub const MAX_SAFE: u64 = 9_007_199_254_740_991;

// Value's default map parser overwrites duplicates; this visitor rejects them at every depth.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self,D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result {f.write_str("duplicate-free JSON")}
            fn visit_bool<E:de::Error>(self,v:bool)->Result<Unique,E>{Ok(Unique(Value::Bool(v)))}
            fn visit_i64<E:de::Error>(self,v:i64)->Result<Unique,E>{Ok(Unique(v.into()))}
            fn visit_u64<E:de::Error>(self,v:u64)->Result<Unique,E>{Ok(Unique(v.into()))}
            fn visit_f64<E:de::Error>(self,v:f64)->Result<Unique,E>{serde_json::Number::from_f64(v).map(|n|Unique(Value::Number(n))).ok_or_else(||E::custom("nonfinite"))}
            fn visit_str<E:de::Error>(self,v:&str)->Result<Unique,E>{Ok(Unique(v.into()))}
            fn visit_string<E:de::Error>(self,v:String)->Result<Unique,E>{Ok(Unique(v.into()))}
            fn visit_unit<E:de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
            fn visit_none<E:de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
            fn visit_seq<A:SeqAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{let mut v=Vec::new();while let Some(Unique(x))=a.next_element()?{v.push(x);}Ok(Unique(Value::Array(v)))}
            fn visit_map<A:MapAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{let mut m=Map::new();while let Some(k)=a.next_key::<String>()?{if m.contains_key(&k){return Err(de::Error::custom("duplicate key"));}let Unique(v)=a.next_value()?;m.insert(k,v);}Ok(Unique(Value::Object(m)))}
        }
        d.deserialize_any(V)
    }
}
pub fn parse(bytes:&[u8])->Result<Value,String>{
    if bytes.is_empty() || bytes.len()>65536 || bytes.starts_with(&[239,187,191]) {return Err("E_RPC_LINE".into());}
    serde_json::from_slice::<Unique>(bytes).map(|u|u.0).map_err(|_|"E_RPC_JSON".into())
}
pub fn keys(v:&Value,expected:&[&str])->Result<(),String>{let m=v.as_object().ok_or("E_RPC_PARAMS")?;if m.len()!=expected.len() || expected.iter().any(|k|!m.contains_key(*k)){return Err("E_RPC_PARAMS".into());}Ok(())}
pub fn integer(v:&Value,min:u64,max:u64)->Result<u64,String>{
    let n=if let Some(n)=v.as_u64(){n}else{let f=v.as_f64().ok_or("E_RPC_NUMBER")?;if !f.is_finite() || f.fract()!=0.0 || f<0.0 || f>MAX_SAFE as f64{return Err("E_RPC_NUMBER".into());}f as u64};
    if n<min || n>max {return Err("E_RPC_NUMBER".into());}Ok(n)
}
fn normalize(v:&mut Value)->Result<(),String>{
    match v {
        Value::Object(m)=>for(k,x)in m.iter_mut(){if matches!(k.as_str(),"protocolVersion"|"worldEpoch"|"seq"|"clientTimeMs"|"requestId"|"generation"|"commandSequence"|"pauseCommandSequence"|"expectedBuildRevision") && x.is_number(){*x=Value::from(integer(x,0,MAX_SAFE)?);}else{normalize(x)?;}},
        Value::Array(a)=>for x in a{normalize(x)?;}, _=>{}
    }Ok(())
}
fn typed<T:serde::de::DeserializeOwned>(v:&Value)->Result<T,String>{let mut v=v.clone();normalize(&mut v)?;serde_json::from_value(v).map_err(|_|"E_RPC_PARAMS".into())}
fn string<'a>(p:&'a Value,k:&str)->Result<&'a str,String>{let s=p[k].as_str().ok_or("E_RPC_PARAMS")?;if s.is_empty() || s.len()>256{return Err("E_RPC_PARAMS".into());}Ok(s)}
fn boolean(p:&Value,k:&str)->Result<bool,String>{p[k].as_bool().ok_or_else(||"E_RPC_PARAMS".into())}
fn redact_errors(v:&mut Value){match v {Value::Object(m)=>for(k,x)in m{if k=="errorCode" {if let Some(s)=x.as_str(){*x=Value::String(safe_error(s));}}else{redact_errors(x);}},Value::Array(a)=>for x in a{redact_errors(x);},_=>{}}}
fn encode<T:Serialize>(v:T)->Result<Value,String>{let mut value=serde_json::to_value(v).map_err(|_|"E_RPC_SERIALIZE")?;redact_errors(&mut value);Ok(value)}
fn receipt(id:&str,v:Result<wuxian_horror_ch1::world_v3::WorldView,String>)->Result<Value,String>{encode(CommandReceipt::from_view(id,v?))}
pub fn safe_error(error:&str)->String {
    let code=error.split(':').next().unwrap_or("");
    if code.starts_with("E_") && code.len()<=96 && code.bytes().all(|c|c.is_ascii_uppercase() || c.is_ascii_digit() || c==b'_'){code.into()}else{"E_RPC_CORE".into()}
}
pub fn response(id:u64,result:Result<Value,String>)->Value{match result {Ok(v)=>json!({"protocol":PROTOCOL,"protocolVersion":1,"id":id,"ok":true,"result":v,"error":null}),Err(e)=>json!({"protocol":PROTOCOL,"protocolVersion":1,"id":id,"ok":false,"result":null,"error":{"code":safe_error(&e)}})}}
pub fn hold(runtime:&FormalRuntime)->Result<(),String>{
    let s=runtime.bridge_session_status()?;
    let context=SessionContext{world_id:s["worldId"].as_str().ok_or("E_RPC_STATUS")?.into(),scene_id:s["sceneId"].as_str().ok_or("E_RPC_STATUS")?.into(),world_epoch:integer(&s["worldEpoch"],1,MAX_SAFE)?};
    let seq=integer(&s["lastLifecycleCommandSequence"],0,MAX_SAFE)?.checked_add(1).filter(|n|*n<=MAX_SAFE).ok_or("E_LIFECYCLE_STALE_COMMAND")?;
    runtime.pause_context_ordered(&context,seq)?;Ok(())
}
pub fn dispatch(runtime:&FormalRuntime,method:&str,p:&Value)->Result<Value,String>{
    match method {
        "formal_snapshot"=>{keys(p,&[])?;encode(WorldSnapshot::from_view(runtime.snapshot()?))},
        "bridge_session_status"=>{keys(p,&[])?;runtime.bridge_session_status()},
        "formal_new"=>{keys(p,&[])?;receipt("new",runtime.reset_new())},
        "formal_submit_input"=>{
            keys(p,&["sample","combat"])?;if p["combat"].as_array().is_none_or(|a|!a.is_empty()){return Err("E_RPC_COMBAT_LEGACY".into());}
            keys(&p["sample"],&["protocol","protocolVersion","worldEpoch","seq","clientTimeMs","moveX","moveZ","aimX","aimZ"])?;
            let sample:wuxian_horror_ch1::continuous_input::InputSample=typed(&p["sample"])?;
            let id=format!("input:{}:{}",sample.world_epoch,sample.seq);receipt(&id,runtime.submit_input(sample,vec![]))
        },
        "formal_submit_action"=>{keys(p,&["action"])?;keys(&p["action"],&["protocolVersion","worldEpoch","requestId","clientTimeMs","kind"])?;
            if !matches!(p["action"]["kind"].as_str(),Some("primaryAttack"|"dash"|"pulse"|"guardStart"|"guardEnd"|"pierce"|"contextTraversal")){return Err("E_RPC_ACTION".into());}
            let a:formal_runtime::ActionCommandRequest=typed(&p["action"])?;let id=format!("action:{}",a.request_id);receipt(&id,runtime.submit_action(a))},
        "formal_interact"=>{keys(p,&["actorId","requestId","worldEpoch"])?;encode(routes::interaction(runtime,string(p,"actorId")?,string(p,"requestId")?,Some(integer(&p["worldEpoch"],1,MAX_SAFE)?))?)},
        "formal_scene_transition"|"formal_scene_checkpoint"|"formal_scene_trigger"|"formal_environment_control"|"formal_world_gate"|"formal_mission_terminal_status"|"formal_capability_terminal_status"|"formal_save_rest_terminal"=>{
            keys(p,&["id","requestId","worldEpoch"])?;let id=string(p,"id")?;let request=string(p,"requestId")?;let epoch=integer(&p["worldEpoch"],1,MAX_SAFE)?;
            encode(match method {"formal_scene_transition"=>routes::transition(runtime,id,request,epoch),"formal_scene_checkpoint"=>routes::checkpoint(runtime,id,request,epoch),"formal_scene_trigger"=>routes::trigger(runtime,id,request,epoch),"formal_environment_control"=>routes::environment_control(runtime,id,request,epoch),"formal_world_gate"=>routes::world_gate(runtime,id,request,epoch),"formal_mission_terminal_status"=>routes::mission_terminal_status(runtime,id,request,epoch),"formal_capability_terminal_status"=>routes::capability_terminal_status(runtime,id,request,epoch),_=>routes::save_rest_terminal(runtime,id,request,epoch)}?)
        },
        "formal_build_command"=>{keys(p,&["command"])?;encode(runtime.apply_build_command(typed(&p["command"])?)?)},
        "formal_capability_command"=>{keys(p,&["command"])?;match p["command"]["kind"].as_str(){Some("select")=>keys(&p["command"],&["kind","capabilityIds"])? ,Some("activate")=>keys(&p["command"],&["kind","capabilityId","requestId"])? ,_=>return Err("E_RPC_CAPABILITY".into())};receipt("capability-command",runtime.apply_capability_command(typed(&p["command"])?))},
        "formal_choose_first_enhancement"=>{keys(p,&["capabilityId","context"])?;let id=string(p,"capabilityId")?;receipt(&format!("enhancement:{id}"),runtime.choose_first_enhancement(id,&typed(&p["context"])?))},
        "formal_baizhi_begin"=>{keys(p,&["request"])?;encode(runtime.baizhi_begin(typed(&p["request"])?)?)},
        "formal_baizhi_commit"=>{keys(p,&["ticket","requestId","choice"])?;encode(runtime.baizhi_commit(&typed(&p["ticket"])?,string(p,"requestId")?,typed(&p["choice"])?)?)},
        "formal_baizhi_close"=>{keys(p,&["ticket"])?;encode(runtime.baizhi_close(&typed(&p["ticket"])?)?)},
        "formal_has_save"=>{keys(p,&["defaultOnly"])?;Ok(Value::Bool(if boolean(p,"defaultOnly")?{runtime.has_default_save()}else{runtime.has_save()}))},
        "formal_continue"=>{keys(p,&[])?;receipt("continue",runtime.continue_saved())},
        "formal_save"=>{keys(p,&[])?;receipt("save",runtime.save())},
        "formal_return"=>{keys(p,&[])?;receipt("return",runtime.return_to_hub())},
        "formal_list_save_slots"=>{keys(p,&[])?;encode(runtime.list_save_slots())},
        "formal_save_slot"|"formal_return_slot"=>{keys(p,&["slotId","displayName","create"])?;let id=string(p,"slotId")?;let name=string(p,"displayName")?;let create=boolean(p,"create")?;if method=="formal_save_slot"{receipt("save-slot",runtime.save_slot(id,name,create))}else{receipt("return-slot",runtime.return_to_hub_slot(id,name,create))}},
        "formal_continue_slot"=>{keys(p,&["slotId"])?;receipt("continue-slot",runtime.continue_slot(string(p,"slotId")?))},
        "formal_pause"|"formal_resume"=>{keys(p,&["context","commandSequence"])?;let context:SessionContext=typed(&p["context"])?;let seq=integer(&p["commandSequence"],1,MAX_SAFE)?;if method=="formal_pause"{receipt("pause",runtime.pause_context_ordered(&context,seq))}else{receipt("resume",runtime.resume_context_ordered(&context,seq))}},
        "formal_scene_ready"=>{keys(p,&["token","remainPaused"])?;let token:wuxian_horror_ch1::world_v3::SceneEntryToken=typed(&p["token"])?;receipt(&format!("scene-ready:{}",token.generation),runtime.scene_ready(&token,boolean(p,"remainPaused")?))},
        "formal_presentation_events"|"formal_sound_cues"=>{keys(p,&["worldEpoch","afterEventId"])?;let epoch=integer(&p["worldEpoch"],1,MAX_SAFE)?;let cursor=integer(&p["afterEventId"],0,MAX_SAFE)?;if method=="formal_presentation_events"{encode(runtime.presentation_events_since(epoch,cursor)?)}else{encode(runtime.sound_cues_since(epoch,cursor)?)}},
        _=>Err("E_RPC_METHOD".into())
    }
}

//! Owned, loopback-only campaign service. Compilation never runs this entry.
mod godot_rpc;
use godot_rpc as rpc;
use serde_json::{json, Value};
use std::{collections::VecDeque, fs::{self,OpenOptions}, io::{self,Read,Write}, net::{TcpListener,TcpStream}, path::{Component,Path,PathBuf}, thread, time::{Duration,Instant}};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, production_scene_bootstrap};
const MAX_LINE:usize=65536;
const MAX_RESPONSE:usize=2*1024*1024;
const MAX_QUEUE:usize=4*1024*1024;

fn ordinary(path:&Path,missing_leaf:bool)->Result<(),String>{
    if !path.is_absolute(){return Err("E_SESSION_PATH".into());}
    let raw=path.to_str().ok_or("E_SESSION_PATH")?;
    let drive=raw.as_bytes();
    if drive.len()<3 || !drive[0].is_ascii_alphabetic() || drive[1]!=b':' || !matches!(drive[2],b'/'|b'\\') || raw[2..].contains(':') || raw.split(['/', '\\']).any(|s|s=="." || s=="..") {return Err("E_SESSION_PATH".into());}
    let mut current=PathBuf::new();let components:Vec<_>=path.components().collect();
    for (i,c) in components.iter().enumerate(){
        match c {Component::Normal(s)=>{let text=s.to_str().ok_or("E_SESSION_PATH")?;if text.ends_with('.') || text.ends_with(' ') || text.chars().any(char::is_control){return Err("E_SESSION_PATH".into());}},Component::ParentDir|Component::CurDir=>return Err("E_SESSION_PATH".into()),_=>{}}
        current.push(c.as_os_str());
        match fs::symlink_metadata(&current){
            Ok(m)=>{
                #[cfg(windows)] {use std::os::windows::fs::MetadataExt;if m.file_attributes()&0x400!=0{return Err("E_SESSION_REPARSE".into());}}
                if m.file_type().is_symlink() || (i+1<components.len() && !m.is_dir()){return Err("E_SESSION_PATH".into());}
            },
            Err(e) if e.kind()==io::ErrorKind::NotFound && missing_leaf && i+1==components.len()=>{},
            Err(_)=>return Err("E_SESSION_PATH".into())
        }
    }Ok(())
}
fn arguments()->Result<(PathBuf,PathBuf,String),String>{
    let mut session=None;let mut save=None;let mut token=None;let mut a=std::env::args().skip(1);
    while let Some(k)=a.next(){let v=a.next().ok_or("E_SESSION_ARGS")?;match k.as_str(){
        "--session-dir" if session.is_none()=>session=Some(PathBuf::from(v)),
        "--save-dir" if save.is_none()=>save=Some(PathBuf::from(v)),
        "--session-token" if token.is_none()=>token=Some(v), _=>return Err("E_SESSION_ARGS".into())
    }}
    let session=session.ok_or("E_SESSION_ARGS")?;let save=save.ok_or("E_SESSION_ARGS")?;let token:String=token.ok_or("E_SESSION_ARGS")?;
    if !(32..=128).contains(&token.len()) || !token.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'){return Err("E_SESSION_TOKEN".into());}
    // Explicit future-owner namespaces only; no directory/save ownership is inferred here.
    ordinary(&session,false)?;if !session.is_dir() || save!=session.join("saves"){return Err("E_SESSION_NAMESPACE".into());}
    ordinary(&save,true)?;ordinary(&session.join("run"),true)?;
    match fs::symlink_metadata(session.join("run/bridge-ready.json")){Ok(_)=>return Err("E_READY_OCCUPIED".into()),Err(e) if e.kind()==io::ErrorKind::NotFound=>{},Err(_)=>return Err("E_READY_PATH".into())}
    Ok((session,save,token))
}
struct Connection {stream:TcpStream,line:Vec<u8>,pending:VecDeque<Vec<u8>>,offset:usize,queued:usize,last_id:u64,authenticated:bool,closing:bool,last_progress:Instant}
impl Connection {
    fn new(stream:TcpStream)->Result<Self,String>{stream.set_nonblocking(true).map_err(|_|"E_SOCKET")?;stream.set_nodelay(true).map_err(|_|"E_SOCKET")?;Ok(Self{stream,line:Vec::new(),pending:VecDeque::new(),offset:0,queued:0,last_id:0,authenticated:false,closing:false,last_progress:Instant::now()})}
    fn enqueue(&mut self,id:u64,result:Result<Value,String>)->Result<(),String>{
        let mut bytes=serde_json::to_vec(&rpc::response(id,result)).map_err(|_|"E_RPC_SERIALIZE")?;bytes.push(b'\n');
        if bytes.len()>MAX_RESPONSE {bytes=serde_json::to_vec(&rpc::response(id,Err("E_RPC_RESPONSE_LIMIT_MUTATION_UNKNOWN".into()))).map_err(|_|"E_RPC_SERIALIZE")?;bytes.push(b'\n');}
        if self.queued+bytes.len()>MAX_QUEUE{return Err("E_RPC_QUEUE".into());}
        self.queued+=bytes.len();self.pending.push_back(bytes);Ok(())
    }
    fn request(&mut self,runtime:&FormalRuntime,token:&str,started:&mut bool)->Result<bool,String>{
        let v=rpc::parse(&self.line)?;rpc::keys(&v,&["protocol","protocolVersion","id","method","params"])?;
        if v["protocol"].as_str()!=Some(rpc::PROTOCOL) || rpc::integer(&v["protocolVersion"],1,1)?!=1{return Err("E_RPC_PROTOCOL".into());}
        let id=rpc::integer(&v["id"],1,rpc::MAX_SAFE)?;if id<=self.last_id{return Err("E_RPC_SEQUENCE".into());}self.last_id=id;
        let method=v["method"].as_str().filter(|s|s.len()<=96).ok_or("E_RPC_METHOD")?;let p=&v["params"];if !p.is_object(){return Err("E_RPC_PARAMS".into());}
        if !self.authenticated {
            let result=if method!="hello" {Err("E_RPC_HELLO_REQUIRED".into())}else{
                rpc::keys(p,&["sessionToken","clientProtocolVersion"])?;
                if p["sessionToken"].as_str()!=Some(token) || rpc::integer(&p["clientProtocolVersion"],1,1)?!=1{Err("E_RPC_AUTH".into())}
                else{self.authenticated=true;Ok(json!({"protocolVersion":1,"service":"wuxian-v1-formal","snapshotProtocolVersion":3}))}
            };self.enqueue(id,result)?;return Ok(false);
        }
        if method=="session_close" {rpc::keys(p,&[])?;self.closing=true;self.enqueue(id,Ok(json!({"closing":true})))?;return Ok(true);}
        let entry=matches!(method,"formal_new"|"formal_continue"|"formal_continue_slot");
        let readable=matches!(method,"formal_snapshot"|"bridge_session_status"|"formal_has_save"|"formal_list_save_slots"|"formal_presentation_events"|"formal_sound_cues");
        let result=if !*started && !entry && !readable {Err("E_SESSION_ENTRY_REQUIRED".into())}else{rpc::dispatch(runtime,method,p)};
        if entry && result.as_ref().is_ok_and(|v|v["applied"].as_bool()==Some(true) && v["errorCode"].is_null()){*started=true;}
        self.enqueue(id,result)?;Ok(false)
    }
    fn update(&mut self,runtime:&FormalRuntime,token:&str,started:&mut bool)->Result<bool,String>{
        while let Some(front)=self.pending.front(){match self.stream.write(&front[self.offset..]){
            Ok(0)=>return Err("E_SOCKET_CLOSED".into()),Ok(n)=>{self.offset+=n;self.queued-=n;self.last_progress=Instant::now();if self.offset==front.len(){self.pending.pop_front();self.offset=0;}},
            Err(e) if e.kind()==io::ErrorKind::WouldBlock=>break,Err(e) if e.kind()==io::ErrorKind::Interrupted=>continue,Err(_)=>return Err("E_SOCKET_WRITE".into())
        }}
        if self.closing{return Ok(self.pending.is_empty());}
        // Reserve a full response before reading/dispatching another mutation.
        if self.queued>MAX_QUEUE-MAX_RESPONSE {if self.last_progress.elapsed()>Duration::from_secs(30){return Err("E_SOCKET_BACKPRESSURE".into());}return Ok(false);}
        let mut byte=[0u8;1];let mut processed=0;
        for _ in 0..8192 {
            if self.queued>MAX_QUEUE-MAX_RESPONSE || processed>=16 {break;}
            match self.stream.read(&mut byte){
                Ok(0)=>return Err("E_SOCKET_CLOSED".into()),Ok(_)=>{
                    if byte[0]==b'\n'{if self.line.last()==Some(&b'\r'){return Err("E_RPC_LF".into());}let close=self.request(runtime,token,started)?;self.line.clear();processed+=1;self.last_progress=Instant::now();if close{break;}}
                    else{if self.line.len()>=MAX_LINE{return Err("E_RPC_LINE".into());}self.line.push(byte[0]);}
                },
                Err(e) if e.kind()==io::ErrorKind::WouldBlock=>break,Err(e) if e.kind()==io::ErrorKind::Interrupted=>continue,Err(_)=>return Err("E_SOCKET_READ".into())
            }
        }
        if !self.authenticated && self.last_progress.elapsed()>Duration::from_secs(30){return Err("E_RPC_AUTH_TIMEOUT".into());}
        Ok(false)
    }
}
fn run()->Result<(),String>{
    let (session,save,token)=arguments()?;
    fs::create_dir_all(session.join("run")).map_err(|_|"E_SESSION_CREATE")?;
    fs::create_dir_all(&save).map_err(|_|"E_SESSION_CREATE")?;ordinary(&save,false)?;ordinary(&session.join("run"),false)?;
    let listener=TcpListener::bind(("127.0.0.1",0)).map_err(|_|"E_SOCKET_BIND")?;listener.set_nonblocking(true).map_err(|_|"E_SOCKET")?;
    let runtime=FormalRuntime::new_with_save_dir(save.clone())?;production_scene_bootstrap::install(&runtime)?;rpc::hold(&runtime)?;
    let ready=json!({"schema":"wuxian.godot-v1.ready/v1","protocolVersion":1,"host":"127.0.0.1","port":listener.local_addr().map_err(|_|"E_SOCKET")?.port(),"sessionToken":token,"processId":std::process::id(),"sessionDir":session,"saveDir":save});
    let mut ready_file=OpenOptions::new().write(true).create_new(true).open(session.join("run/bridge-ready.json")).map_err(|_|"E_READY_CREATE")?;
    let mut bytes=serde_json::to_vec(&ready).map_err(|_|"E_READY_SERIALIZE")?;bytes.push(b'\n');ready_file.write_all(&bytes).and_then(|_|ready_file.sync_all()).map_err(|_|"E_READY_WRITE")?;drop(ready_file);
    let mut client:Option<Connection>=None;let mut started=false;
    loop {
        if client.is_none(){match listener.accept(){Ok((s,address))=>{if address.ip().is_loopback(){client=Some(Connection::new(s)?);}},Err(e) if e.kind()==io::ErrorKind::WouldBlock=>{},Err(e) if e.kind()==io::ErrorKind::Interrupted=>{},Err(_)=>return Err("E_SOCKET_ACCEPT".into())}}
        if let Some(c)=client.as_mut(){match c.update(&runtime,&token,&mut started){Ok(true)=>{rpc::hold(&runtime)?;break;},Ok(false)=>{},Err(_)=>{let explicitly_closing=c.closing;rpc::hold(&runtime)?;client=None;if explicitly_closing{break;}}}}
        thread::sleep(Duration::from_millis(5));
    }
    // FormalRuntime::drop requests stop and joins its actual simulation owner.
    drop(client);drop(runtime);Ok(())
}
fn main(){if let Err(error)=run(){eprintln!("{}",rpc::safe_error(&error));std::process::exit(2);}}

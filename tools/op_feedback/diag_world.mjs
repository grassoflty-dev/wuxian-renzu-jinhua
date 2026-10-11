// diag_world.mjs — 启动新局, dump 世界结构供排查交互/战斗可达性
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const SAVE = path.join(ROOT, 'target', 'release', 'data', 'save.json');
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
function runSync(cmd){try{return execSync(cmd,{stdio:'pipe'}).toString();}catch{return '';}}
async function killPort(){try{const out=runSync(`netstat -ano | findstr :${PORT}`);for(const l of out.split(/\r?\n/)){const m=l.trim().match(/(\d+)\s*$/);if(m)try{runSync(`taskkill /PID ${m[1]} /T /F 2>nul`);}catch{}}}catch{}runSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul');await sleep(600);}
let ws, nextId=1;
async function getPage(){for(let i=0;i<120;i++){try{const r=await fetch(`http://127.0.0.1:${PORT}/json/list`);const l=await r.json();const p=l.find(x=>x.type==='page');if(p)return p;}catch{}await sleep(500);}throw new Error('no page');}
async function connect(p){ws=new WebSocket(p.webSocketDebuggerUrl);await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});}
function send(o){ws.send(JSON.stringify(o));}
function cdp(method,params={}){return new Promise((resolve,reject)=>{const id=nextId++;const t=setTimeout(()=>reject(new Error('to '+method)),30000);const on=ev=>{let m;try{m=JSON.parse(ev.data);}catch{return;}if(m.id!==id)return;clearTimeout(t);ws.removeEventListener('message',on);m.error?reject(new Error('err')):resolve(m.result);};ws.addEventListener('message',on);send({id,method,params});});}
async function evalJs(expression){const r=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true,userGesture:true});if(r.exceptionDetails)return {__err:(r.exceptionDetails.exception?.description||'').slice(0,300)};return r.result?.value;}
const inv=(c,a)=>evalJs(`(async function(){try{return await window.__TAURI__.core.invoke(${JSON.stringify(c)},${JSON.stringify(a||{})});}catch(e){return {__err:String((e&&e.message)||e)};}})()`);

(async()=>{
  await killPort();
  try{fs.rmSync(SAVE,{force:true});}catch{}
  const child=spawn(EXE,[],{env:{...process.env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${PORT}`},stdio:'ignore'});
  child.unref();
  const page=await getPage();
  await connect(page);
  await sleep(2000);
  // 进世界
  await evalJs("(function(){var b=document.getElementById('btnNew');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(2000);
  // 跳过新手
  await evalJs("(function(){var b=document.getElementById('btnTutSkip');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(800);
  const w=await inv('api_world');
  if(!w||w.__err){console.log('NO WORLD',JSON.stringify(w));}
  else{
    const H=w.tiles?w.tiles.length:0; const W=H?(w.tiles[0]||'').length:0;
    console.log('tiles',W+'x'+H,'player',w.px,w.py,'floor',w.floor,'world',w.world&&w.world.id);
    console.log('ENEMIES',JSON.stringify((w.enemies||[]).map(e=>({id:e.id,x:e.x,y:e.y,alive:e.alive})),null,0));
    console.log('GATES',JSON.stringify((w.gates||[]).map(g=>({id:g.id,x:g.x,y:g.y,locked:g.locked}))));
    console.log('PORTALS',JSON.stringify((w.portals||[]).map(p=>({id:p.id,x:p.x,y:p.y}))));
    console.log('POINTS',JSON.stringify((w.points||[]).map(p=>({id:p.id||p.obj_id,x:p.x,y:p.y}))));
    console.log('NEARBY',JSON.stringify(w.nearby||[]));
    const h=await inv('api_world');
  }
  // 主神空间视角
  const ne=await inv('api_nexus_enter');
  await sleep(1200);
  const w2=await inv('api_world');
  if(w2&&!w2.__err){
    console.log('NEXUS tiles',(w2.tiles?w2.tiles[0].length:0)+'x'+(w2.tiles?w2.tiles.length:0),'player',w2.px,w2.py,'world',w2.world&&w2.world.id);
    console.log('NEXUS GATES',JSON.stringify((w2.gates||[]).map(g=>({id:g.id,x:g.x,y:g.y,locked:g.locked}))));
    console.log('NEXUS PORTALS',JSON.stringify((w2.portals||[]).map(p=>({id:p.id,x:p.x,y:p.y}))));
    console.log('NEXUS POINTS',JSON.stringify((w2.points||[]).map(p=>({id:p.id||p.obj_id,x:p.x,y:p.y}))));
    console.log('NEXUS NEARBY',JSON.stringify(w2.nearby||[]));
    console.log('NEXUS ENEMIES',JSON.stringify((w2.enemies||[]).map(e=>({id:e.id,x:e.x,y:e.y}))));
  }
  try{ws.close();}catch{}
  await killPort();
  process.exit(0);
})().catch(e=>{console.error('FATAL',e.message);try{ws&&ws.close();}catch{}process.exit(2);});

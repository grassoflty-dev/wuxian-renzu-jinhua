// diag_battle.mjs — 聚焦: 新局后 BFS 走到最近敌人并一步踩上, 观察是否触发 encounter → zone
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
function runSync(c){try{return execSync(c,{stdio:'pipe'}).toString();}catch{return '';}}
async function killPort(){try{const o=runSync(`netstat -ano | findstr :${PORT}`);for(const l of o.split(/\r?\n/)){const m=l.trim().match(/(\d+)\s*$/);if(m)try{runSync(`taskkill /PID ${m[1]} /T /F 2>nul`);}catch{}}}catch{}runSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul');await sleep(600);}
let ws,nextId=1;
async function getPage(){for(let i=0;i<120;i++){try{const r=await fetch(`http://127.0.0.1:${PORT}/json/list`);const l=await r.json();const p=l.find(x=>x.type==='page');if(p)return p;}catch{}await sleep(500);}throw new Error('nopage');}
async function connect(p){ws=new WebSocket(p.webSocketDebuggerUrl);await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});}
function send(o){ws.send(JSON.stringify(o));}
function cdp(m,params={}){return new Promise((res,rej)=>{const id=nextId++;const t=setTimeout(()=>rej(new Error('to '+m)),30000);const on=ev=>{let x;try{x=JSON.parse(ev.data);}catch{return;}if(x.id!==id)return;clearTimeout(t);ws.removeEventListener('message',on);x.error?rej(new Error('err')):res(x.result);};ws.addEventListener('message',on);send({id,method:m,params});});}
async function evalJs(e){const r=await cdp('Runtime.evaluate',{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});if(r.exceptionDetails)return {__err:(r.exceptionDetails.exception?.description||'').slice(0,300)};return r.result?.value;}
const inv=(c,a)=>evalJs(`(async function(){try{return await window.__TAURI__.core.invoke(${JSON.stringify(c)},${JSON.stringify(a||{})});}catch(e){return {__err:String((e&&e.message)||e)};}})()`);

(async()=>{
  await killPort();
  try{fs.rmSync(SAVE,{force:true});}catch{}
  const child=spawn(EXE,[],{env:{...process.env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${PORT}`},stdio:'ignore'});
  child.unref();
  const page=await getPage(); await connect(page); await sleep(2000);
  await evalJs("(function(){var b=document.getElementById('btnNew');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(2000);
  await evalJs("(function(){var b=document.getElementById('btnTutSkip');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(700);
  let w=await inv('api_world');
  console.log('start',w&&w.px,w&&w.py,'enemies',JSON.stringify((w.enemies||[]).filter(e=>e.alive).map(e=>e.id+':'+e.x+','+e.y)));
  // 直接调用 window.worldInteract(enemyId) 触发副本(不走寻路)
  const eid = 'e_f1_z3';
  const r0 = await evalJs(`(async function(){ try { window.worldInteract(${JSON.stringify(eid)}); return 'invoked'; } catch(e){ return 'err:'+String(e.message||e); } })()`);
  await sleep(2000);
  const zoneActive = await evalJs(`(function(){ return window.ZoneActive===true || document.getElementById('zoneView').style.display==='flex'; })()`);
  console.log('direct worldInteract =>', r0, 'zoneActive', zoneActive);
  const title = await evalJs(`(function(){ return (document.getElementById('zoneTitle')||{}).textContent||''; })()`);
  console.log('zoneTitle', title);
  try{ws.close();}catch{}
  await killPort();
  process.exit(0);
})().catch(e=>{console.error('FATAL',e);try{ws&&ws.close();}catch{}process.exit(2);});

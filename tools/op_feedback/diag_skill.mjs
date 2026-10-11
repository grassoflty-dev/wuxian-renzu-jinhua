// diag_skill.mjs — 检查体验账号解锁的技能 + 装备技能是否持久化
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
function cdp(m,params={}){return new Promise((res,rej)=>{const id=nextId++;const t=setTimeout(()=>rej(new Error('to '+m)),40000);const on=ev=>{let x;try{x=JSON.parse(ev.data);}catch{return;}if(x.id!==id)return;clearTimeout(t);ws.removeEventListener('message',on);x.error?rej(new Error('err')):res(x.result);};ws.addEventListener('message',on);send({id,method:m,params});});}
async function evalJs(e){const r=await cdp('Runtime.evaluate',{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});if(r.exceptionDetails)return {__err:(r.exceptionDetails.exception?.description||'').slice(0,300)};return r.result?.value;}
const inv=(c,a)=>evalJs(`(async function(){try{return await window.__TAURI__.core.invoke(${JSON.stringify(c)},${JSON.stringify(a||{})});}catch(e){return {__err:String((e&&e.message)||e)};}})()`);

(async()=>{
  await killPort(); try{fs.rmSync(SAVE,{force:true});}catch{}
  const child=spawn(EXE,[],{env:{...process.env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${PORT}`},stdio:'ignore'}); child.unref();
  const page=await getPage(); await connect(page); await sleep(2000);
  await evalJs("(function(){var b=document.getElementById('btnNew');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(1200);
  await evalJs("(function(){var b=document.getElementById('btnTutSkip');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(500);
  // 体验账号
  await evalJs("(function(){document.getElementById('settingsOverlay').classList.add('on');var b=document.getElementById('btnExpAccount');if(b)b.click();return 1;})()");
  await sleep(1200);
  await evalJs("(function(){document.getElementById('settingsOverlay').classList.remove('on');return 1;})()");
  await sleep(300);
  // 技能列表
  const sk=await inv('api_get_skills');
  console.log('skills count', sk&&sk.skills&&sk.skills.length, 'equipped', JSON.stringify(sk&&sk.equipped_skills));
  const acts=(sk&&sk.skills||[]).filter(s=>!s.kind||s.kind==='Active');
  console.log('Actives', JSON.stringify(acts.slice(0,6).map(s=>s.id)));
  // 直接 SkillSys.equipSkill 前2个 Active
  if(acts.length>=2){
    const e1=await evalJs(`(async function(){ try{ const ok=await window.SkillSys.equipSkill(${JSON.stringify(acts[0].id)}); return JSON.stringify({ok, eq:window.SkillSys.getEquipped()}); }catch(e){ return 'err:'+String(e.message||e);} })()`);
    console.log('equip #1', e1);
    const e2=await evalJs(`(async function(){ try{ const ok=await window.SkillSys.equipSkill(${JSON.stringify(acts[1].id)}); return JSON.stringify({ok, eq:window.SkillSys.getEquipped()}); }catch(e){ return 'err:'+String(e.message||e);} })()`);
    console.log('equip #2', e2);
    const back=await inv('api_get_skills');
    console.log('backend equipped after', JSON.stringify(back&&back.equipped_skills));
    const hud=await inv('api_set_equipped_skills',{skills:[acts[0].id,acts[1].id]});
    console.log('api_set_equipped_skills resp err', hud&&hud.__err||'none', 'hudSkillCount', hud&&hud.skillCount);
    const back2=await inv('api_get_skills');
    console.log('backend equipped after direct set', JSON.stringify(back2&&back2.equipped_skills));
  }
  try{ws.close();}catch{} await killPort(); process.exit(0);
})().catch(e=>{console.error('FATAL',e);try{ws&&ws.close();}catch{}process.exit(2);});

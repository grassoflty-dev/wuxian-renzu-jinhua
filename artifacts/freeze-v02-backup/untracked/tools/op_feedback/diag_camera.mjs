// diag_camera.mjs — 确认 world3d 渲染画布与事件触发是否生效 + 极限滚轮/旋转对比
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const SAVE = path.join(ROOT, 'target', 'release', 'data', 'save.json');
const SHOTS = path.join(__dirname, 'shots');
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
function runSync(c){try{return execSync(c,{stdio:'pipe'}).toString();}catch{return '';}}
async function killPort(){try{const o=runSync(`netstat -ano | findstr :${PORT}`);for(const l of o.split(/\r?\n/)){const m=l.trim().match(/(\d+)\s*$/);if(m)try{runSync(`taskkill /PID ${m[1]} /T /F 2>nul`);}catch{}}}catch{}runSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul');await sleep(600);}
let ws,nextId=1;
async function getPage(){for(let i=0;i<120;i++){try{const r=await fetch(`http://127.0.0.1:${PORT}/json/list`);const l=await r.json();const p=l.find(x=>x.type==='page');if(p)return p;}catch{}await sleep(500);}throw new Error('nopage');}
async function connect(p){ws=new WebSocket(p.webSocketDebuggerUrl);await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});}
function send(o){ws.send(JSON.stringify(o));}
function cdp(m,params={}){return new Promise((res,rej)=>{const id=nextId++;const t=setTimeout(()=>rej(new Error('to '+m)),40000);const on=ev=>{let x;try{x=JSON.parse(ev.data);}catch{return;}if(x.id!==id)return;clearTimeout(t);ws.removeEventListener('message',on);x.error?rej(new Error('err')):res(x.result);};ws.addEventListener('message',on);send({id,method:m,params});});}
async function evalJs(e){const r=await cdp('Runtime.evaluate',{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});if(r.exceptionDetails)return {__err:(r.exceptionDetails.exception?.description||'').slice(0,200)};return r.result?.value;}
const inv=(c,a)=>evalJs(`(async function(){try{return await window.__TAURI__.core.invoke(${JSON.stringify(c)},${JSON.stringify(a||{})});}catch(e){return {__err:String((e&&e.message)||e)};}})()`);
async function shot(tag){const r=await cdp('Page.captureScreenshot',{format:'png'});const out=path.join(SHOTS,tag+'.png');fs.writeFileSync(out,Buffer.from(r.data,'base64'));return out;}

(async()=>{
  await killPort();
  try{fs.rmSync(SAVE,{force:true});}catch{}
  const child=spawn(EXE,[],{env:{...process.env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${PORT}`},stdio:'ignore'});
  child.unref();
  const page=await getPage(); await connect(page); await sleep(2000);
  await evalJs("(function(){var b=document.getElementById('btnNew');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(2000);
  await evalJs("(function(){var b=document.getElementById('btnTutSkip');if(b&&b.offsetParent!==null){b.click();return 1;}return 0;})()");
  await sleep(800);
  // 检查 worldCanvasWrap 内 canvas 构成
  const info=await evalJs(`JSON.stringify((function(){
    const wrap=document.getElementById('worldCanvasWrap');
    const kids=wrap?[...wrap.children].map(el=>({tag:el.tagName,id:el.id,cls:el.className,w:el.width,h:el.height,sw:el.style&&el.style.width,sh:el.style&&el.style.height})):[];
    const hiddenPlaceholder=document.getElementById('worldCanvas');
    return {wrapExists:!!wrap, kids, hiddenPlaceholder:{w:hiddenPlaceholder&&hiddenPlaceholder.width,h:hiddenPlaceholder&&hiddenPlaceholder.height,parent:hiddenPlaceholder&&hiddenPlaceholder.parentNode&&hiddenPlaceholder.parentNode.id}};
  })())`);
  console.log('CANVAS INFO:', info);
  // 缩放: 极限滚轮
  await shot('cam_zoom_before');
  const zr=await evalJs(`(function(){
    const wrap=document.getElementById('worldCanvasWrap'); const c=wrap&&wrap.querySelector('canvas');
    let n=0; const ev=new WheelEvent('wheel',{deltaY:400,bubbles:true,cancelable:true});
    for(let i=0;i<80;i++){ c&&c.dispatchEvent(ev); n++; }
    return n; })()`);
  await sleep(1000);
  await shot('cam_zoom_after');
  console.log('ZOOM dispatched',zr);
  // 旋转: 极限右键拖
  const rr=await evalJs(`(function(){
    const wrap=document.getElementById('worldCanvasWrap'); const c=wrap&&wrap.querySelector('canvas');
    c&&c.dispatchEvent(new MouseEvent('mousedown',{button:2,clientX:100,clientY:450,bubbles:true,cancelable:true}));
    for(let i=0;i<20;i++) window.dispatchEvent(new MouseEvent('mousemove',{clientX:100+i*60,clientY:450,bubbles:true,cancelable:true}));
    window.dispatchEvent(new MouseEvent('mouseup',{button:2,bubbles:true,cancelable:true}));
    return 1; })()`);
  await sleep(800);
  await shot('cam_rot_after');
  console.log('ROTATE dispatched',rr);
  try{ws.close();}catch{}
  await killPort();
  process.exit(0);
})().catch(e=>{console.error('FATAL',e);try{ws&&ws.close();}catch{}process.exit(2);});

import fs from 'node:fs';
const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),25000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
const on=id=>ev(`document.getElementById(${JSON.stringify(id)}).classList.contains("on")`);

// 回世界：点击 story 里的“返回地图”按钮
const clicked = await ev(`(()=>{const b=Array.from(document.querySelectorAll("button")).find(x=>/返回地图|进入轮回/.test(x.textContent||""));if(b){b.click();return b.textContent}return "NO_BTN"})()`);
console.log("clicked:",clicked);
for(let i=0;i<40;i++){ if(await disp("worldView")==="block") break; await sleep(500); }
await sleep(800);
console.log("now: world=",await disp("worldView"),"title=",await disp("titleScreen"),"story=",await disp("story"));
console.log("bigmap before test:",await on("bigmapOverlay"));

// 测试1: 合成 KeyboardEvent
async function domTapM(){ return ev(`(()=>{const d=new KeyboardEvent("keydown",{key:"m",keyCode:77,bubbles:true,cancelable:true});const u=new KeyboardEvent("keyup",{key:"m",keyCode:77,bubbles:true});window.dispatchEvent(d);document.dispatchEvent(d);window.dispatchEvent(u);document.dispatchEvent(u);return "domtap"})()`); }
await domTapM(); await sleep(500);
console.log("after DOM M:",await on("bigmapOverlay"));
await domTapM(); await sleep(300);
console.log("after DOM M close:",await on("bigmapOverlay"));

// 测试2: CDP Input.dispatchKeyEvent
async function inputTapM(){ await cdp("Input.dispatchKeyEvent",{type:"keyDown",key:"m",code:"KeyM",nativeVirtualKeyCode:77}); await cdp("Input.dispatchKeyEvent",{type:"keyUp",key:"m",code:"KeyM",nativeVirtualKeyCode:77}); }
await cdp("Page.bringToFront");
await inputTapM(); await sleep(500);
console.log("after Input M:",await on("bigmapOverlay"));
await inputTapM(); await sleep(300);
console.log("after Input M close:",await on("bigmapOverlay"));

// ensure closed
await ev(`(()=>{const el=document.getElementById("bigmapOverlay");if(el)el.classList.remove("on");return 1})()`);
console.log("final world:",await disp("worldView"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

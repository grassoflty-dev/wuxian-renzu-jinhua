import fs from 'node:fs';
const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),25000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
const info=async()=>{
  return ev(`(()=>{
    const d=id=>{const el=document.getElementById(id);return el?getComputedStyle(el).display:"missing"};
    const on=id=>{const el=document.getElementById(id);return !!el&&el.classList.contains("on")};
    return {title:d("titleScreen"),world:d("worldView"),zone:d("zoneView"),story:d("story"),hud:d("hud"),
      bigmapOn:on("bigmapOverlay"),skillOn:on("skillPanelOverlay"),settingsOn:on("settingsOverlay"),
      tutOn:on("tutorialOverlay"), endOn:on("endOverlay"), zoneActive:!!window.ZoneActive,
      gameTitle: (window.__gameTitle) };
  })()`);
};
console.log("STATE:", JSON.stringify(await info()));
// cleanup overlays
await ev(`(()=>{["bigmapOverlay","skillPanelOverlay","settingsOverlay","tutorialOverlay","endOverlay"].forEach(id=>{const el=document.getElementById(id);if(el)el.classList.remove("on")});return "cleaned"})()`);
console.log("after cleanup:", JSON.stringify(await info()));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

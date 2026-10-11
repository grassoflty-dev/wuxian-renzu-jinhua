import fs from 'node:fs';
const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),25000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
const on=id=>ev(`document.getElementById(${JSON.stringify(id)}).classList.contains("on")`);

// 重载页面回标题
await cdp("Page.reload",{ignoreCache:true});
await sleep(2500);
console.log("after reload: title=",await disp("titleScreen"),"world=",await disp("worldView"));
// 清 overlay
await ev(`(()=>{["bigmapOverlay","skillPanelOverlay","settingsOverlay","tutorialOverlay","endOverlay"].forEach(id=>{const el=document.getElementById(id);if(el)el.classList.remove("on")});return 1})()`);
const contDisabled = await ev(`document.getElementById("btnContinue").disabled`);
console.log("btnContinue disabled:",contDisabled);
await ev(`document.getElementById(${contDisabled?"btnNew":"btnContinue"}).click()`);
console.log("clicked enter");
let world=false;
for(let i=0;i<40;i++){ if(await disp("worldView")==="block"){world=true;break;} await sleep(500); }
await sleep(1000);
console.log("after enter: world=",await disp("worldView"),"title=",await disp("titleScreen"),"story=",await disp("story"));

// --- 测试手动 open bigmap（等价 M 键 handler）---
await ev(`(()=>{const el=document.getElementById("bigmapOverlay");el.classList.add("on");if(window.World3D&&window.World3D.drawBigmap)window.World3D.drawBigmap();return 1})()`);
await sleep(900);
console.log("manual bigmap on:",await on("bigmapOverlay"));
const s=await cdp("Page.captureScreenshot",{format:"png"});
fs.writeFileSync("C:/Users/PUBLIC_USER/Desktop/itwillclaude/games/wuxian-horror-ch1/tools/shots/sweep/bigmap_manual.png",Buffer.from(s.data,"base64"));
console.log("saved bigmap_manual.png");
await ev(`(()=>{const el=document.getElementById("bigmapOverlay");el.classList.remove("on");return 1})()`);
await sleep(300);
console.log("closed bigmap:",await on("bigmapOverlay"));
console.log("final world:",await disp("worldView"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

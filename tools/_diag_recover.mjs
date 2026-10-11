import fs from 'node:fs';
const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const OUT="C:/Users/PUBLIC_USER/Desktop/itwillclaude/games/wuxian-horror-ch1/tools/shots/sweep";
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to "+m)),35000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
const on=id=>ev(`(()=>{const el=document.getElementById(${JSON.stringify(id)});return !!el&&el.classList.contains("on")})()`);
async function center(sel){ return ev(`(()=>{const el=document.querySelector(${JSON.stringify(sel)});if(!el)return null;const r=el.getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2,v:r.width*r.height&&getComputedStyle(el).visibility!=="hidden"}})()`); }
async function clickSel(sel){ const pt=await center(sel); if(!pt||!pt.v) return false; await cdp("Input.dispatchMouseEvent",{type:"mouseMoved",x:pt.x,y:pt.y}); await cdp("Input.dispatchMouseEvent",{type:"mousePressed",x:pt.x,y:pt.y,button:"left",clickCount:1}); await sleep(50); await cdp("Input.dispatchMouseEvent",{type:"mouseReleased",x:pt.x,y:pt.y,button:"left",clickCount:1}); return true; }
async function inputKeyM(){ await cdp("Input.dispatchKeyEvent",{type:"keyDown",key:"m",code:"KeyM",nativeVirtualKeyCode:77}); await cdp("Input.dispatchKeyEvent",{type:"keyUp",key:"m",code:"KeyM",nativeVirtualKeyCode:77}); await sleep(700); }

await cdp("Runtime.enable"); await cdp("Page.enable"); await cdp("Page.bringToFront");
// reset: close any overlay
await ev(`(()=>{["bigmapOverlay","skillPanelOverlay","settingsOverlay","tutorialOverlay","endOverlay","cineWrap"].forEach(id=>{const el=document.getElementById(id);if(el)el.classList.remove("on")});return 1})()`);

let mode = await ev(`(()=>{const d=id=>{const e=document.getElementById(id);return e?getComputedStyle(e).display:"x"};const m=d("titleScreen")==="flex"?"title":d("worldView")==="block"?"world":d("zoneView")==="flex"?"zone":d("story")==="block"?"story":d("zoneView");return m})()`);
console.log("mode now:",mode);

if(mode!=="world"){
  // ensure title
  if(mode!=="title"){ await cdp("Page.reload",{ignoreCache:true}); await sleep(3000); console.log("reloaded; title=",await disp("titleScreen")); }
  const needNew = await ev(`document.getElementById("btnContinue").disabled`);
  console.log("btnContinue disabled:",needNew);
  const ok=await clickSel(needNew ? "#btnNew" : "#btnContinue");
  console.log("clicked enter:",ok);
}

// 推进开场：cine / narr / choices / tutorial 直到 world
let reached=false;
for(let i=0;i<120;i++){
  const m=await ev(`(()=>{const d=id=>{const e=document.getElementById(id);return e?getComputedStyle(e).display:"x"};return d("worldView").trim()==="block"?"world":d("titleScreen").trim()==="flex"?"title":d("story").trim()==="block"?"story":d("zoneView").trim()==="flex"?"zone":"other"})()`);
  if(m==="world"){reached=true;break;}
  // cine active?
  const cine=await on("cineWrap");
  if(cine){ await clickSel("#cineSkip"); await sleep(200); continue; }
  // tutorial overlay?
  const tut=await on("tutorialOverlay");
  if(tut){ await clickSel("#btnTutSkip"); await sleep(300); continue; }
  // story: 尝试“返回地图”choice 或点 narrBox
  if(m==="story"||m==="other"){
    const hasBack=await ev(`(()=>{const b=Array.from(document.querySelectorAll("#story button")).find(x=>/返回地图/.test(x.textContent||"")||/进入轮回/.test(x.textContent||""));return !!b})()`);
    if(hasBack){ await clickSel("#story button"); await sleep(400); continue; }
    await clickSel("#narrBox"); await sleep(150); continue;
  }
  if(m==="title"){ // shouldn't happen mid-flow
    await clickSel("#btnContinue"); await sleep(300); continue;
  }
  await sleep(300);
}
console.log("after advance loop: reached world =",reached,"world=",await disp("worldView"),"story=",await disp("story"));
await sleep(1000);
// cleanup tutorial again
await ev(`(()=>{const el=document.getElementById("tutorialOverlay");if(el)el.classList.remove("on");return 1})()`);
console.log("mode final:", await ev(`(()=>{const d=id=>{const e=document.getElementById(id);return e?getComputedStyle(e).display:"x"};return d("worldView").trim()==="block"?"world":d("story").trim()==="block"?"story":d("titleScreen").trim()==="flex"?"title":"other"})()`));

// ---- 测试 M key (Input) open bigmap ----
console.log("bigmap on before Input m:",await on("bigmapOverlay"));
await inputKeyM();
console.log("bigmap on after Input m:",await on("bigmapOverlay"));
if(await on("bigmapOverlay")){
  const s=await cdp("Page.captureScreenshot",{format:"png"}); fs.writeFileSync(OUT+"/bigmap_test2.png",Buffer.from(s.data,"base64")); console.log("saved bigmap_test2.png");
  await inputKeyM(); console.log("closed via M:",await on("bigmapOverlay"));
} else {
  // fallback manual
  await ev(`(()=>{const el=document.getElementById("bigmapOverlay");el.classList.add("on");if(window.World3D&&window.World3D.drawBigmap)window.World3D.drawBigmap();return 1})()`);
  await sleep(900);
  const s=await cdp("Page.captureScreenshot",{format:"png"}); fs.writeFileSync(OUT+"/bigmap_manual2.png",Buffer.from(s.data,"base64")); console.log("saved bigmap_manual2.png (fallback)");
  await ev(`(()=>{const el=document.getElementById("bigmapOverlay");el.classList.remove("on");return 1})()`);
}
console.log("final world:",await disp("worldView"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

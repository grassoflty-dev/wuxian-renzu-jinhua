import fs from 'node:fs';
const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),25000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
const on=id=>ev(`document.getElementById(${JSON.stringify(id)}).classList.contains("on")`);

console.log("before: title=",await disp("titleScreen"),"world=",await disp("worldView"));
// 尝试进世界：优先 btnContinue（不破坏存档）
const contText = await ev(`document.getElementById("btnContinue").textContent`);
const contDisabled = await ev(`document.getElementById("btnContinue").disabled`);
console.log("btnContinue:", JSON.stringify(contText),"disabled:",contDisabled);
let entered=false;
if(!contDisabled){
  await ev(`document.getElementById("btnContinue").click()`);
  entered=true;
} else {
  await ev(`document.getElementById("btnNew").click()`);
  entered=true;
}
console.log("clicked enter, entered=",entered);
// 等待世界进入
let world=false;
for(let i=0;i<30;i++){ const d=await disp("worldView"); if(d==="block"){world=true;break;} await sleep(500); }
console.log("after enter: world=",await disp("worldView"),"title=",await disp("titleScreen"));
await sleep(1000);

// 用 CDP Input.dispatchKeyEvent 发 m
async function inputKey(keyName, code, keyCode){
  await cdp("Input.dispatchKeyEvent",{type:"keyDown",key:keyName,code,text:keyName.length===1?keyName:undefined,composedText:undefined,nativeVirtualKeyCode:keyCode});
  await cdp("Input.dispatchKeyEvent",{type:"keyUp",key:keyName,code,nativeVirtualKeyCode:keyCode});
}
console.log("bigmap on before Input m:",await on("bigmapOverlay"));
await inputKey("m","KeyM",77);
await sleep(800);
console.log("bigmap on after Input m:",await on("bigmapOverlay"));
// screenshot if open
if(await on("bigmapOverlay")){
  const s=await cdp("Page.captureScreenshot",{format:"png"});
  fs.writeFileSync("C:/Users/PUBLIC_USER/Desktop/itwillclaude/games/wuxian-horror-ch1/tools/shots/sweep/bigmap_test.png",Buffer.from(s.data,"base64"));
  console.log("bigmap screenshot saved");
  await inputKey("m","KeyM",77); await sleep(300);
  console.log("bigmap on after close m:",await on("bigmapOverlay"));
} else {
  // 降级：手动 toggle attempt
  await ev(`(()=>{const el=document.getElementById("bigmapOverlay");el.classList.add("on");if(window.World3D&&window.World3D.drawBigmap)window.World3D.drawBigmap();return 1})()`);
  await sleep(800);
  console.log("manual fallback bigmap on:",await on("bigmapOverlay"));
}
console.log("world still:",await disp("worldView"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

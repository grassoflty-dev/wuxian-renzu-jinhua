const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),30000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);

async function realCenterClick(idOrSel){
  const pt=await ev(`(()=>{const el=document.querySelector(${JSON.stringify(idOrSel)});if(!el)return null;const r=el.getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2}})()`);
  if(!pt) return false;
  await cdp("Input.dispatchMouseEvent",{type:"mouseMoved",x:pt.x,y:pt.y});
  await cdp("Input.dispatchMouseEvent",{type:"mousePressed",x:pt.x,y:pt.y,button:"left",clickCount:1});
  await sleep(50);
  await cdp("Input.dispatchMouseEvent",{type:"mouseReleased",x:pt.x,y:pt.y,button:"left",clickCount:1});
  return true;
}

console.log("start: story=",await disp("story"),"world=",await disp("worldView"));
let clickedBack=false;
for(let i=0;i<30;i++){
  const w=await disp("worldView");
  if(w==="block"){console.log("reached WORLD at iter"+i);break;}
  // 尝试找“返回地图”按钮并点
  const hasBack=await ev(`(()=>{const b=Array.from(document.querySelectorAll("#story button")).find(x=>/返回地图|进入轮回/.test(x.textContent||""));return !!b})()`);
  if(hasBack){ await realCenterClick('#story button'); clickedBack=true; console.log("clicked back button iter"+i); }
  else {
    // 点文字框推进打字
    await realCenterClick('#narrBox'); console.log("clicked narrBox iter"+i);
  }
  await sleep(400);
}
console.log("done: world=",await disp("worldView"),"story=",await disp("story"),"title=",await disp("titleScreen"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

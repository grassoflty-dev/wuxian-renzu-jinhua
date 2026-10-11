const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),25000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
async function clickChoice(sel){ const pt=await ev(`(()=>{const els=document.querySelectorAll(${JSON.stringify(sel)});const el=els[0];if(!el)return null;const r=el.getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2}})()`); if(!pt)return false; await cdp("Input.dispatchMouseEvent",{type:"mouseMoved",x:pt.x,y:pt.y}); await cdp("Input.dispatchMouseEvent",{type:"mousePressed",x:pt.x,y:pt.y,button:"left",clickCount:1}); await sleep(40); await cdp("Input.dispatchMouseEvent",{type:"mouseReleased",x:pt.x,y:pt.y,button:"left",clickCount:1}); return true; }

async function pick(){ return ev(`(()=>{const s=document.getElementById("story");const cs=Array.from(document.querySelectorAll("#story .choice")).map(b=>b.textContent.trim().slice(0,30));return {story:getComputedStyle(s).display, choices:cs}})()`); }
console.log("before:",JSON.stringify(await pick()));
// 点 choice 推进，多次直到 world
for(let i=0;i<40;i++){
  if(await disp("worldView")==="block"){console.log("WORLD at i="+i);break;}
  const c=await ev(`document.querySelectorAll("#story .choice").length`);
  if(c>0){ await clickChoice("#story .choice"); console.log("clicked a choice i="+i); }
  else { console.log("no choice, narr click i="+i); await clickChoice("#narrBox"); }
  await sleep(500);
}
console.log("FINAL world=",await disp("worldView"),"story=",await disp("story"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

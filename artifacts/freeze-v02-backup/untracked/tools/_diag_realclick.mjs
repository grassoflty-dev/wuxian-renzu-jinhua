const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),30000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
console.log("title:",await disp("titleScreen"),"world:",await disp("worldView"));
const rect=await ev(`(()=>{const b=document.getElementById("btnContinue");if(!b)return null;if(b.disabled)return {disabled:true};const r=b.getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2,w:r.width,h:r.height}})()`);
console.log("btnContinue rect:",JSON.stringify(rect));
if(rect && !rect.disabled){
  await cdp("Page.bringToFront");
  await cdp("Input.dispatchMouseEvent",{type:"mouseMoved",x:rect.x,y:rect.y});
  await cdp("Input.dispatchMouseEvent",{type:"mousePressed",x:rect.x,y:rect.y,button:"left",clickCount:1});
  await sleep(60);
  await cdp("Input.dispatchMouseEvent",{type:"mouseReleased",x:rect.x,y:rect.y,button:"left",clickCount:1});
  console.log("real mouse clicked btnContinue");
}
for(let i=0;i<40;i++){ if(await disp("worldView")==="block") break; await sleep(500); }
console.log("after real click world=",await disp("worldView"),"title=",await disp("titleScreen"),"story=",await disp("story"));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

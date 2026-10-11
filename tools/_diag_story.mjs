const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),25000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true,userGesture:true});return r.result?.value};
console.log("=== STORY dump ===");
console.log(await ev(`(()=>{
  const s=document.getElementById("story");
  const choices=Array.from(document.querySelectorAll("#story button")).map(b=>b.textContent.trim());
  const narr=document.getElementById("narrText"); 
  const cineOn=document.getElementById("cineWrap").classList.contains("on");
  const tutOn=document.getElementById("tutorialOverlay").classList.contains("on");
  const sv=document.getElementById("cineVid");
  return {storyDisplay:getComputedStyle(s).display, choices, narr:(narr?narr.textContent.slice(0,120):""), cineOn, tutOn, cinePlaying: !!sv&&!sv.paused};
})()`));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

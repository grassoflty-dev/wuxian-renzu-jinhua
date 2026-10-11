const PORT=9702; const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
const r=await fetch(`http://127.0.0.1:${PORT}/json/list`); const list=await r.json(); const page=list.find(x=>x.type==="page");
const ws=new WebSocket(page.webSocketDebuggerUrl); await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
let n=1; const cdp=(m,p={})=>new Promise((res,rej)=>{const id=n++;const t=setTimeout(()=>rej(new Error("to")),20000);const on=ev=>{let mm;try{mm=JSON.parse(ev.data)}catch{return}if(mm.id===id){clearTimeout(t);ws.removeEventListener("message",on);mm.error?rej(new Error(JSON.stringify(mm.error))):res(mm.result)}};ws.addEventListener("message",on);ws.send(JSON.stringify({id,method:m,params:p}))});
const ev=async e=>{const r=await cdp("Runtime.evaluate",{expression:e,returnByValue:true,awaitPromise:true});return r.result?.value};
const disp=id=>ev(`getComputedStyle(document.getElementById(${JSON.stringify(id)})).display`);
console.log("title:",await disp("titleScreen"),"world:",await disp("worldView"),"zone:",await disp("zoneView"),"story:",await disp("story"));
console.log("btnNew exists:", await ev(`!!document.getElementById("btnNew")`));
console.log("btnNew text:", await ev(`document.getElementById("btnNew").textContent`));
console.log("btnContinue disabled:", await ev(`document.getElementById("btnContinue").disabled`));
ws.close();process.exit(0);
})().catch(e=>{console.error("ERR",e.message);process.exit(1)});

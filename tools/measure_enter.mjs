const sleep = ms => new Promise(r => setTimeout(r, ms));
async function getPage(){ for(let i=0;i<40;i++){ try{ const r=await fetch('http://127.0.0.1:9702/json/list'); const l=await r.json(); const p=l.find(x=>x.type==='page'); if(p) return p; }catch{} await sleep(500);} throw new Error('no page'); }
(async()=>{
  const page = await getPage();
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej;});
  let id=1;
  const cdp=(m,p={})=>new Promise((res,rej)=>{const i=id++;const t=setTimeout(()=>rej(new Error('timeout')),15000);const on=e=>{let d;try{d=JSON.parse(e.data);}catch{return}if(d.id!==i)return;clearTimeout(t);ws.removeEventListener('message',on);d.error?rej(new Error(JSON.stringify(d.error))):res(d.result)};ws.addEventListener('message',on);ws.send(JSON.stringify({id:i,method:m,params:p}));});
  const ev=(expr)=>cdp('Runtime.evaluate',{expression:expr,returnByValue:true,awaitPromise:true}).then(r=>r&&r.result&&r.result.value);
  const inv=(c,a)=>ev(`(async()=>{try{return await window.__TAURI__.core.invoke(${JSON.stringify(c)},${JSON.stringify(a||{})});}catch(e){return {__err:String(e.message||e)}}})()`);
  await sleep(1500);
  await inv('api_set_window_size', {width:1920, height:1080});
  await sleep(1200);
  await inv('api_new', {});
  await sleep(4000);
  const w = await inv('api_world', {});
  const tiles = (w && w.tiles) ? w.tiles : null;
  const info = tiles ? ('world=' + (w.world&&w.world.id||w.id||'?') + ' tiles_h=' + tiles.length + ' w=' + (tiles[0]||'').length) : 'NO_TILES ' + JSON.stringify(w).slice(0,120);
  console.log('MAP=' + info);
  console.log('READY');
  await sleep(90000);  // 保持运行 90 秒供采样
  ws.close(); process.exit(0);
})().catch(e=>{console.error('ERR',e.message);process.exit(1)});

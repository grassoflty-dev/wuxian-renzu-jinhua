// ⚠️【废弃标注】游戏已切换到 World2D/Zone2D 引擎（index.html 不再加载 js/postfx.js），window.PostFX 已不存在，本脚本的相机覆写/后处理链路失效，输出已无意义。画面评估请用 probe_world2d.mjs / probe_zone2d.mjs。
// 立绘 billboard 迁移真实游戏验证（v2）：
//   1) 世界地图(world3d)：进 biohazard，用 window.PostFX._scene 遍历实体，按 userData.spec.img 分类
//      玩家(pc_zhengzha)/敌人(enemy_*)/NPC(pc_*) 是否为立绘 billboard 平面，贴图是否加载成功；截图 verify_world_map.png
//   2) 战斗副本(zone3d)：走格触发舔食者遭遇，同样遍历 zone 场景检查 玩家/敌人 立绘；截图 verify_zone_fight.png
// 不改任何代码；只读脚本 + CDP 截图。输出 tools/shots/verify_*.png + tools/verify_scene_report.json
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const SAVE = path.join(ROOT, 'target', 'release', 'data', 'save.json');
const OUT = path.join(__dirname, 'shots');
const LOG = path.join(__dirname, 'logs', 'verify_game.log');
const PORT = 9702;
fs.mkdirSync(OUT, { recursive: true });
fs.mkdirSync(path.dirname(LOG), { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => { try { fs.appendFileSync(LOG, s + '\n'); } catch {} console.log(s); };

const REPORT = { launch: null, world: null, zone: null, shots: {}, files: {} };

function cleanLaunch() {
  try { execSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul', { stdio: 'pipe' }); } catch {}
  try {
    const out = execSync(`netstat -ano | findstr :${PORT}`, { stdio: 'pipe' }).toString();
    for (const line of out.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  fs.rmSync(SAVE, { force: true });
  fs.rmSync(path.join(OUT, 'verify_world_map.png'), { force: true });
  fs.rmSync(path.join(OUT, 'verify_zone_fight.png'), { force: true });
  const child = spawn(EXE, [], {
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: 'ignore',
  });
  child.unref();
  return child;
}
async function getPage() {
  for (let i = 0; i < 120; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}/json/list`);
      const list = await r.json();
      const p = list.find(x => x.type === 'page');
      if (p) return p;
    } catch {}
    await sleep(500);
  }
  throw new Error('page not found');
}
let ws, nextId = 1;
async function connect(page) {
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
}
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 25000);
    const onMsg = ev => {
      let m; try { m = JSON.parse(ev.data); } catch { return; }
      if (m.id !== id) return;
      clearTimeout(timer); ws.removeEventListener('message', onMsg);
      m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result);
    };
    ws.addEventListener('message', onMsg);
    send({ id, method, params });
  });
}
async function evalJs(expression, timeout = 25000) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) { log('evalExc: ' + JSON.stringify(r.exceptionDetails).slice(0, 400)); return { __exc: true }; }
  return r.result?.value;
}
async function shot(name) {
  await sleep(1300);
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const p = path.join(OUT, name);
  fs.writeFileSync(p, Buffer.from(r.data, 'base64'));
  const size = fs.statSync(p).size;
  log(`SHOT ${name} ${size}B ${size > 50 * 1024 ? '(OK)' : '(<50KB 可能黑屏!)'}`);
  return { name, path: p, size };
}
const inv = (cmd, args) => evalJs(`(async function(){ try { return await window.__TAURI__.core.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args || {})}); } catch(e){ return {__err:String(e.message||e)}; } })()`);
const readSave = () => { try { return JSON.parse(fs.readFileSync(SAVE, 'utf8')); } catch { return null; } };

// 注入：patch PostFX.render 以允许相机覆盖（世界/副本通用）
const INSTALL_CAM = `(function(){
  if (window.__camInstalledV) return 'already';
  window.__camInstalledV = true;
  window.__camOverrideV = null;
  const pf = window.PostFX;
  if (pf && pf.render) {
    const orig = pf.render.bind(pf);
    pf.render = function(){
      const o = window.__camOverrideV;
      if (o && this.ready && this._mainCam){
        this._mainCam.position.set(o.px, o.py, o.pz);
        this._mainCam.up.set(0,1,0);
        this._mainCam.lookAt(o.lx, o.ly, o.lz);
        if (o.fov && this._mainCam.fov !== o.fov){ this._mainCam.fov = o.fov; this._mainCam.aspect = this._mainCam.aspect || 1; this._mainCam.updateProjectionMatrix(); }
        this._mainCam.updateMatrixWorld && this._mainCam.updateMatrixWorld(true);
      }
      return orig.call(this);
    };
    return 'ok-patchno-postfx';
  }
  return 'ok-no-postfx';
})()`;
const setCam = (px, py, pz, lx, ly, lz, fov) => evalJs(`window.__camOverrideV = { px:${px}, py:${py}, pz:${pz}, lx:${lx}, ly:${ly}, lz:${lz}, fov:${fov || 0} }; 'set'`);

// 通用场景扫描：遍历 PostFX._scene，识别带 userData.spec.sprite 的实体组，按 spec.img 分类。
// 返回每个实体：分类类别(player/enemy/npc/item)、spec.img、是否立绘平面、贴图加载状态、位置、几何尺寸。
const SCENE_SCAN = `(function(){
  const pf = window.PostFX;
  if (!pf || !pf._scene) return { err: 'no scene', ready: !!(pf&&pf.ready) };
  const entities = [];
  const view = { hasSprite: 0 };
  pf._scene.traverse(o => {
    if (!o || !o.userData) return;
    const ud = o.userData;
    const spr = ud.sprite || null;
    if (!spr) return;                  // 只关注挂 sprite 的实体（玩家/敌人/NPC 立绘 or item billboard）
    view.hasSprite++;
    const img = (ud.spec && ud.spec.img) || (spr.userData && spr.userData._img) || '';
    let cls = 'unknown';
    if (/pc_zhengzha/i.test(img)) cls = 'player';
    else if (/pc_/i.test(img)) cls = 'npc';
    else if (/enemy_/i.test(img)) cls = 'enemy';
    else if (/item_/i.test(img)) cls = 'item';
    // 贴图加载判定
    const m = spr.material || {};
    const tex = m.map || null;
    let texLoaded = null, texImg = null;
    if (tex && tex.image) { const im = tex.image; texLoaded = !!(im.width > 0 && im.naturalWidth !== 0); texImg = { w: im.width || im.naturalWidth, h: im.height || im.naturalHeight }; }
    else if (tex) texLoaded = null;
    else texLoaded = false;
    const isPlane = !!(spr.geometry && spr.geometry.type === 'PlaneGeometry');
    const pw = (spr.geometry && spr.geometry.parameters) ? spr.geometry.parameters.width : null;
    const ph = (spr.geometry && spr.geometry.parameters) ? spr.geometry.parameters.height : null;
    entities.push({
      cls, img: img || null,
      enemyId: (ud.enemy && (ud.enemy.id || ud.enemy.name)) || (ud.id || null),
      npcId: (ud.npc && (ud.npc.name || ud.npc.id)) || null,
      isPlane, texLoaded, texImg,
      matTransparent: !!m.transparent, alphaTest: m.alphaTest != null ? m.alphaTest : null,
      spritePosY: spr.position ? spr.position.y : null,
      planeW: pw, planeH: ph,
      groupPos: o.position ? { x: +o.position.x.toFixed(2), y: +o.position.y.toFixed(2), z: +o.position.z.toFixed(2) } : null,
      childCount: o.children ? o.children.length : 0,
    });
  });
  // 额外检测：userData.enemy 在但无 sprite（= 体素兜底敌人），userData.spriteType false
  const voxels = [];
  pf._scene.traverse(o => {
    if (o && o.userData && o.userData.enemy && !o.userData.sprite) {
      voxels.push({ enemyId: o.userData.enemy.id || o.userData.enemy.name || o.userData.enemy, spriteType: o.userData.spriteType || false, childCount: o.children?o.children.length:0 });
    }
  });
  return { hasSprite: view.hasSprite, entities, voxels, totalChildren: pf._scene.children ? pf._scene.children.length : 0, renderer: { w: pf._renderer?pf._renderer.domElement.width:null, h: pf._renderer?pf._renderer.domElement.height:null }, ready: !!pf.ready };
})()`;

let world = null;
async function freshWorld() { world = await inv('api_world'); return world; }

async function main() {
  const child = cleanLaunch();
  try {
    let page;
    try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); REPORT.launch = { ok: false, err: 'getPage ' + e.message }; throw e; }
    await connect(page);
    await sleep(3000);
    log('readyState=' + await evalJs('document.readyState') + ' hasPostFX=' + await evalJs('!!window.PostFX'));
    // 设置 viewport 尺寸，并强制 WebGL renderer 画布铺满容器（世界阶段容器的尺寸已就绪）
    try { await cdp('Emulation.setDeviceMetricsOverride', { width: 1600, height: 900, deviceScaleFactor: 1, mobile: false }); } catch (e) {}
    await evalJs(`(function(){ try{ if(window.PostFX && window.PostFX._renderer){ const c=window.PostFX._renderer.domElement.parentElement; if(c){ window.PostFX._renderer.setSize(Math.max(100,c.clientWidth), Math.max(100,c.clientHeight), true);} } }catch(e){} window.dispatchEvent(new Event('resize')); return 'sized'; })()`);
    await sleep(800);
    const fr = await evalJs(`(function(){ return { worldMove: typeof window.worldMove==='function', enterZone: typeof window.enterZone==='function', worldInteract: typeof window.worldInteract==='function', World3D: !!window.World3D, Zone3D: !!window.Zone3D, tauri: !!(window.__TAURI__&&window.__TAURI__.core), PostFXReady: !!(window.PostFX&&window.PostFX.ready) }; })()`);
    log('frontReady=' + JSON.stringify(fr));
    REPORT.launch = { ok: true, frontReady: fr, exeExists: fs.existsSync(EXE) };

    // 关掉教程
    await evalJs(`(function(){ if(window.__showTutorial) window.__showTutorial=function(){}; return 'suppressed'; })()`);
    const clicked = await evalJs(`(function(){ const b=document.getElementById('btnNew'); if(b&&b.offsetParent!==null){ b.click(); return 1; } return 0; })()`);
    log('clicked btnNew=' + clicked);
    await sleep(2500);
    await evalJs(`(function(){ const o=document.getElementById('tutorialOverlay'); if(o) o.classList.remove('on'); if(window.__closeTutorial) try{window.__closeTutorial();}catch(e){} return 'closed'; })()`);
    await sleep(800);
    let st = readSave();
    log('[enter] world_id=' + (st && st.world_id) + ' scene=' + (st && st.scene_id));

    // 显式进生化世界（若缺省不是 biohazard）
    const intr = await evalJs(`(async function(){ try { await window.worldInteract('gw_biohazard'); return 'ok'; } catch(e){ return 'ERR:'+String(e.message||e); } })()`);
    log('worldInteract gw_biohazard -> ' + intr);
    await sleep(2500);

    // 安装相机覆盖
    await evalJs(INSTALL_CAM);

    // 等世界场景建好 & 扫描
    let scan = null;
    for (let i = 0; i < 25 && (scan==null || (scan&&scan.entities&&scan.entities.length===0)); i++) { await sleep(600); scan = await evalJs(SCENE_SCAN); }
    log('WORLD_SCAN=' + JSON.stringify(scan));
    REPORT.world = scan;

    // 世界地图截图前强制 renderer 画布铺满（世界阶段画布容器高度低，需显式 setSize 大尺寸，
    // 参考 model_shots_sim_v2 的 maximizeCanvas：硬编码渲染尺寸，使 WebGL 占据画面。）
    await evalJs(`(function(){
      try{
        if(window.PostFX && window.PostFX._renderer){
          window.PostFX._renderer.setPixelRatio(1);
          window.PostFX._renderer.setSize(1580, 820, true);
          if(window.PostFX._mainCam){ window.PostFX._mainCam.aspect = 1580/820; window.PostFX._mainCam.updateProjectionMatrix(); }
          const cv = window.PostFX._renderer.domElement;
          cv.style.margin = '0';
          window.dispatchEvent(new Event('resize'));
          return { w: cv.width, h: cv.height, cw: cv.clientWidth, ch: cv.clientHeight };
        }
        return null;
      }catch(e){ return { err: String(e.message||e) }; }
    })()`);
    await sleep(800);
    // 世界地图截图：默认姿态先截一张（保证有一张非黑屏的地图照）
    const sW0 = await shot('verify_world_map.png');
    REPORT.shots.verify_world_map = sW0;

    // 附加：尝试抬高的相机构图，同时容纳玩家 + 最近的已建敌人，存 verify_world_map_alt.png 备用
    const wmFrame = await evalJs(`(function(){
      const pf=window.PostFX; if(!pf||!pf._scene) return null;
      let P=null, E=null;
      pf._scene.traverse(o=>{ if(!o.userData) return; if(!P && o.userData.sprite && (/pc_zhengzha/.test((o.userData.spec&&o.userData.spec.img)||''))) P=o; if(!E && o.userData.enemy && o.userData.sprite) E=o; });
      if(!P) return null;
      const px=P.position.x,pz=P.position.z,py=1.0;
      let ex=px,ez=pz;
      if(E){ ex=E.position.x; ez=E.position.z; }
      const mx=(px+ex)/2, mz=(pz+ez)/2;
      return { camera:{x:mx,y:7.5,z:mz+8}, look:{x:mx,y:0.6,z:mz}, ppos:{x:px,z:pz,py:py}, epos:E?{x:ex,z:ez}:null };
    })()`);
    log('worldFrame='+JSON.stringify(wmFrame));
    if(wmFrame){
      await setCam(wmFrame.camera.x, wmFrame.camera.y, wmFrame.camera.z, wmFrame.look.x, wmFrame.look.y, wmFrame.look.z, 50);
      await sleep(900);
      const sAlt=await shot('verify_world_map_alt.png');
      REPORT.shots.verify_world_map_alt = sAlt;
      // 若 alt 构图成功（更大），用它作为主图
      if(sAlt && sAlt.size > sW0.size && sAlt.size > 80*1024){
        fs.copyFileSync(path.join(OUT,'verify_world_map_alt.png'), path.join(OUT,'verify_world_map.png'));
        sW0.size = sAlt.size; sW0.usedAlt = true;
        REPORT.shots.verify_world_map = { ...sW0, usedAlt: true };
      }
    }

    // ===== 战斗副本：把舔食者遭遇 =====
    const TARGET = { id:'e_licker', floor:1, x:35, y:22 };
    let entered = false, hitEnemy = null;
    function bfsPath(tx,ty,allowPortalFn){
      if(!world) return null;
      const W=world.w,H=world.h;
      const g=Array.from({length:H},()=>Array(W).fill(false));
      for(let y=0;y<H;y++)for(let x=0;x<W;x++) if(((world.tiles[y]||'')[x])==='#') g[y][x]=true;
      (world.gates||[]).forEach(gt=>{if(gt.locked)g[gt.y][gt.x]=true;});
      (world.enemies||[]).forEach(e=>{if(e.alive)g[e.y][e.x]=true;});
      (world.portals||[]).forEach(p=>{ if(!(allowPortalFn&&allowPortalFn(p.x,p.y))) g[p.y][p.x]=true; });
      const sx=world.px,sy=world.py; if(sx===tx&&sy===ty)return [];
      const prev=Array.from({length:H},()=>Array(W).fill(null));
      const dist=Array.from({length:H},()=>Array(W).fill(-1));
      const q=[[sx,sy]]; dist[sy][sx]=0;
      while(q.length){ const [x,y]=q.shift();
        for(const [dx,dy] of [[1,0],[-1,0],[0,1],[0,-1]]){ const nx=x+dx,ny=y+dy;
          if(nx<0||ny<0||nx>=W||ny>=H)continue; if(g[ny][nx]||dist[ny][nx]>=0)continue;
          dist[ny][nx]=dist[y][x]+1; prev[ny][nx]=[x,y]; q.push([nx,ny]);
          if(nx===tx&&ny===ty){ const p=[]; let cur=[tx,ty]; while(cur){p.unshift(cur); cur=prev[cur[1]][cur[0]];} return p; }
        } }
      return null;
    }
    async function rawMove(dx,dy){ return await inv('api_world_move',{dx,dy}); }
    async function tryEnterZone(r){
      if(await evalJs('window.ZoneActive===true')) return true;
      if(r){
        const rr=await evalJs(`(async function(){ try{ await window.enterZone({id:${JSON.stringify(r.enemy_id)},kind:'fight',ref:${JSON.stringify(r.fight_id)},name:${JSON.stringify(r.enemy_id)}},null); return 'ok'; }catch(e){ return 'ERR:'+String(e.message||e); } })()`);
        log('  enterZone -> '+rr);
        await sleep(2400);
      }
      return await evalJs('window.ZoneActive===true');
    }
    async function walkTo(tx,ty,allowPortalFn){
      let guard=0;
      while(guard++<800){
        await freshWorld();
        if(world.px===tx&&world.py===ty) return true;
        const p=bfsPath(tx,ty,allowPortalFn); if(!p||p.length===0){ log('  no path to ('+tx+','+ty+') floor '+world.floor+' @('+world.px+','+world.py+')'); return false; }
        const [tx2,ty2]=p.length>1?p[1]:p[0];
        const r=await rawMove(tx2-world.px,ty2-world.py);
        if(r&&r.encounter){ hitEnemy=r.encounter.enemy_id; log('  ### encounter '+hitEnemy); await tryEnterZone(r.encounter); return 'encounter'; }
      }
      return false;
    }

    await freshWorld();
    log('[zone] start floor='+world.floor+' @('+world.px+','+world.py+')');
    if(!entered && world.floor!==TARGET.floor){
      log('  switch to floor '+TARGET.floor);
      const portalCands=(world.portals||[]).filter(p=>Math.abs(p.to_floor-TARGET.floor)<4).sort((a,b)=> Math.abs(a.to_floor-TARGET.floor)-Math.abs(b.to_floor-TARGET.floor));
      for(const pt of portalCands){
        const w2=await walkTo(pt.x,pt.y,(x,y)=>x===pt.x&&y===pt.y);
        if(w2==='encounter'){entered=true;}
        await freshWorld();
        if(world.floor===TARGET.floor){ log('  on floor '+world.floor+' @('+world.px+','+world.py+')'); break; }
        if(entered) break;
      }
    }
    await freshWorld();
    if(!entered && world.floor===TARGET.floor){
      const near=[[TARGET.x-1,TARGET.y],[TARGET.x+1,TARGET.y],[TARGET.x,TARGET.y-1],[TARGET.x,TARGET.y+1]].filter(([x,y])=>{
        const t=(world.tiles[y]||'')[x]; if(!t||t==='#')return false;
        if((world.gates||[]).some(g=>g.locked&&g.x===x&&g.y===y))return false;
        if((world.enemies||[]).some(e=>e.alive&&e.x===x&&e.y===y))return false;
        return true;
      });
      log('[e_licker] near='+JSON.stringify(near));
      for(const [nx2,ny2] of near){
        await freshWorld();
        const w2=await walkTo(nx2,ny2,null);
        if(w2==='encounter'){entered=true;break;}
        if(await evalJs('window.ZoneActive===true')){entered=true;break;}
        if(w2!==true) continue;
        const r=await rawMove(TARGET.x-world.px,TARGET.y-world.py);
        if(await tryEnterZone(r)) entered=true;
        if(await evalJs('window.ZoneActive===true')){entered=true;break;}
        break;
      }
    }
    if(!entered){
      log('[zone] fallback enterZone e_licker');
      await tryEnterZone({enemy_id:'e_licker',fight_id:'licker'});
      entered=await evalJs('window.ZoneActive===true');
    }
    log('[zone] active='+entered+' hitEnemy='+hitEnemy);
    await sleep(1600);

    if(entered){
      const zscan=await evalJs(SCENE_SCAN);
      log('ZONE_SCAN='+JSON.stringify(zscan));
      REPORT.zone={ entered, hitEnemy, scene: zscan };
      // 先截默认战斗画面（开放场地，默认相机即见人+敌）
      const sZ=await shot('verify_zone_fight.png');
      REPORT.shots.verify_zone_fight = sZ;
      // 尝试把相机对准玩家与敌人朝中间，再截一张清晰立绘照（若 override 生效）
      const camRef=await evalJs(`(function(){
        const pf=window.PostFX; if(!pf||!pf._scene) return null;
        let P=null,E=null;
        pf._scene.traverse(o=>{ if(!o.userData||!o.userData.spec) return; const i=o.userData.spec.img||''; if(!P&&/pc_zhengzha/.test(i)) P=o; if(!E&&/enemy_/.test(i)) E=o; });
        if(!P) return null;
        const px=P.position.x,py=P.position.y+((P.userData.spec&&P.userData.spec.h)/2||1.6),pz=P.position.z;
        const ex=E?E.position.x:0, ey=E?(E.position.y+(E.userData.spec&&E.userData.spec.h)/2||1.6):1.4, ez=E?E.position.z:-9;
        const mx=(px+ex)/2, my=(py+ey)/2, mz=(pz+ez)/2;
        // 相机位于中点靠前上方，看向中点
        const dist=9;
        return { camera:{x:mx, y:my+3.5, z:mz+dist}, look:{x:mx,y:my+0.5,z:mz} };
      })()`);
      log('zoneCamRef='+JSON.stringify(camRef));
      if(camRef){
        await setCam(camRef.camera.x,camRef.camera.y,camRef.camera.z, camRef.look.x,camRef.look.y,camRef.look.z, 55);
        await sleep(900);
        const sZ2=await shot('verify_zone_fight.png'); // 覆盖更新同一文件名
        REPORT.shots.verify_zone_fight = sZ2;
      }
    } else {
      log('ZONE NOT ENTERED');
      REPORT.zone={ entered:false, hitEnemy, err:'could not enter zone' };
    }

  } catch (e) {
    log('FATAL ' + (e.stack || e.message));
    REPORT.launch = REPORT.launch || { ok:false };
    REPORT.launch.fatal = String(e.message || e);
  }
  // 最终文件存在性
  for (const n of ['verify_world_map.png','verify_zone_fight.png']) REPORT.files[n] = fs.existsSync(path.join(OUT,n)) ? fs.statSync(path.join(OUT,n)).size : 0;
  const jsPath = path.join(__dirname, 'verify_scene_report.json');
  fs.writeFileSync(jsPath, JSON.stringify(REPORT, null, 2));
  log('REPORT_WRITTEN ' + jsPath);
  try { ws && ws.close(); } catch {}
  try { child.kill(); } catch {}
  console.log('JSON>', JSON.stringify(REPORT));
  process.exit(0);
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); } catch {} console.error('FATAL', e); process.exit(2); });

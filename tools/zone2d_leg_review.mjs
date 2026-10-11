/* zone2d_leg_review.mjs — 用真实浏览器加载 server-rs/ui/js/zone2d.js，
 * 造一张平地地图、驱动角色 idle/走/跳/攻击，截图 + 裁剪放大，用于验收腿部渲染。
 * 用法: node tools/zone2d_leg_review.mjs [tag]
 * 说明：当前 exe（campaign_main.rs + release-ui）已不再加载 ui/js/zone2d.js，
 *       tools/nav_zombie.mjs 依赖的 api_world 命令已不存在，故改用本地等价浏览器验收。
 */
import { spawn } from "node:child_process";
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const UI = path.join(ROOT, "server-rs", "ui");
const TAG = process.argv[2] || "legs";
const OUT = path.join(ROOT, "tools", "shots", "zone2d_legs");
const PORT = 9761, DEBUG = 9762;
const sleep = ms => new Promise(r => setTimeout(r, ms));
const mime = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css", ".png": "image/png", ".jpg": "image/jpeg", ".json": "application/json" };
fs.mkdirSync(OUT, { recursive: true });

// 虚拟路径：/js/** 与 /assets/** 映射到 server-rs/ui（zone2d.js 的 ROOT 是相对页面的 "assets/img/"）
const server = http.createServer((req, res) => {
  let rel = decodeURIComponent((req.url || "/").split("?")[0]);
  let file;
  if (rel === "/zone2d_leg_review.html") file = path.join(ROOT, "tools", "zone2d_leg_review.html");
  else if (rel === "/js/zone2d_legacy.js") file = path.join(ROOT, "tools", "_zone2d_legacy.js");
  else if (rel.startsWith("/js/") || rel.startsWith("/assets/") || rel.startsWith("/vendor/")) file = path.join(UI, rel.replace(/^\//, ""));
  else file = path.resolve(ROOT, rel.replace(/^\/+/, ""));
  file = path.normalize(file);
  if (!file.startsWith(ROOT) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); res.end("not found: " + rel); return; }
  res.writeHead(200, { "Content-Type": mime[path.extname(file)] || "application/octet-stream", "Cache-Control": "no-store" });
  res.end(fs.readFileSync(file));
});
function cdp(ws, method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++cdp.next; const t = setTimeout(() => reject(new Error("timeout " + method)), 30000);
    const on = e => { let m; try { m = JSON.parse(e.data); } catch { return; } if (m.id !== id) return; clearTimeout(t); ws.removeEventListener("message", on); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); };
    ws.addEventListener("message", on); ws.send(JSON.stringify({ id, method, params }));
  });
}
cdp.next = 0;

const EDGE = ["C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe"].find(p => fs.existsSync(p));

async function main() {
  if (!EDGE) throw new Error("Edge 未安装，无法做浏览器验收");
  await new Promise(r => server.listen(PORT, "127.0.0.1", r));
  const profile = path.join(OUT, "edge-profile-" + Date.now());
  fs.mkdirSync(profile, { recursive: true });
  // 页面必须挂在站点根路径：zone2d.js 内部用相对路径 "assets/img/..." 取立绘
  // tag=before → 加载 tools/_zone2d_legacy.js（改动前的程序化梯形腿）做 A/B 对照
  const url = "http://127.0.0.1:" + PORT + "/zone2d_leg_review.html" + (TAG === "before" ? "?legacy=1" : "");
  const edge = spawn(EDGE, ["--headless=new", "--no-first-run", "--disable-extensions", "--remote-allow-origins=*",
    "--user-data-dir=" + profile, "--remote-debugging-port=" + DEBUG, "--disable-gpu-sandbox",
    "--disable-background-timer-throttling", "--disable-backgrounding-occluded-windows",
    "--disable-renderer-backgrounding", "--disable-features=CalculateNativeWinOcclusion",
    "--window-size=1280,720", url], { stdio: "ignore" });
  let target;
  for (let i = 0; i < 160; i++) { try { const xs = await (await fetch("http://127.0.0.1:" + DEBUG + "/json/list")).json(); target = xs.find(x => x.type === "page" && x.url === url); if (target) break; } catch {} await sleep(250); }
  if (!target) throw new Error("browser CDP page not found");
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  await cdp(ws, "Runtime.enable"); await cdp(ws, "Page.enable");
  // headless 下 rAF 会被节流；开 screencast 强制持续出帧
  try { await cdp(ws, "Page.startScreencast", { format: "png", everyNthFrame: 1, maxWidth: 1280, maxHeight: 720 }); } catch {}
  const errs = [];
  ws.addEventListener("message", ev => {
    let m; try { m = JSON.parse(ev.data); } catch { return; }
    if (m.method === "Runtime.exceptionThrown") errs.push("EXC " + (m.params.exceptionDetails.exception?.description || m.params.exceptionDetails.text));
    if (m.method === "Runtime.consoleAPICalled" && m.params.type === "error") errs.push("ERR " + (m.params.args || []).map(a => a.value || a.description || "").join(" "));
  });
  const evaluate = async expr => { const r = await cdp(ws, "Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.exceptionDetails) return "EXC:" + JSON.stringify(r.exceptionDetails).slice(0, 300); return r.result?.value; };
  const key = (k, down) => evaluate("window.dispatchEvent(new KeyboardEvent('" + (down ? "keydown" : "keyup") + "',{key:'" + k + "',bubbles:true})); true");
  // 取角色包围盒（世界坐标 → 屏幕坐标，TILE/ART = 1，故 scale = dpr * zoom）
  // 直接在画布像素里扫角色包围盒（亮于 165 的像素；地板最亮约 135，不会误判），
  // 换算成 CSS 像素，避免依赖 render() 内部变换公式。
  // 双保险：先按 render() 变换公式算屏幕位置（世界→CSS： (p-cam)*zoom + clientSize/2），
  // 再用画布像素扫描（亮于 70 的像素，地板顶部最亮 ~135，但角色所在行不带地板）做校验。
  // 位置来源：先在「地面带以上」扫描画布像素（>60），取角色包围盒——奔跑时角色每帧移动很大，
  // 用 debug() 预测点会因取帧延迟而裁空，扫描包围盒不受影响。扫描不到时退回预测点。
  const playerBox = () => evaluate("(function(){ try { var d=window.Zone2D.debug(); var cv=document.querySelector('canvas'); var dpr=window.devicePixelRatio||1; var px=(d.pos.x-d.cam.x)*d.zoom + cv.clientWidth/2; var py=(d.pos.y-d.cam.y)*d.zoom + cv.clientHeight/2; var g=cv.getContext('2d'); var lim=Math.max(1,Math.round((py-6)*dpr)); var img=g.getImageData(0,0,cv.width,lim).data; var minX=1e9,minY=1e9,maxX=-1,maxY=-1,n=0; for(var y=0;y<lim;y+=2){ for(var x=0;x<cv.width;x+=2){ var i=(y*cv.width+x)*4; var v=(img[i]+img[i+1]+img[i+2])/3; if(v>60){ n++; if(x<minX)minX=x; if(x>maxX)maxX=x; if(y<minY)minY=y; if(y>maxY)maxY=y; } } } var ok=n>150; var sx=ok?(minX+maxX)/2/dpr:px; var sy=py; return JSON.stringify({ok:ok,state:d.state,mode:d.cam&&d.cam.mode,dpr:dpr,vw:cv.clientWidth,vh:cv.clientHeight,zoom:d.zoom,px:d.pos.x,py:d.pos.y,cx:d.cam.x,cy:d.cam.y,predx:px,predy:py,sx:sx,sy:sy,n:n,bx0:minX/dpr,by0:minY/dpr,bx1:maxX/dpr,by1:maxY/dpr}); } catch(e){ return 'ERR '+e; } })()");
  const boxes = [];
  const shot = async name => {
    // 先取坐标（用整屏截图 + 离线裁剪，避免 CDP clip 在动画中的取帧问题）
    let b = null, info = "?";
    try { b = JSON.parse(await playerBox()); info = b.state; } catch (e) { info = "boxfail"; }
    const r = await cdp(ws, "Page.captureScreenshot", { format: "png" });
    fs.writeFileSync(path.join(OUT, TAG + "_" + name + ".png"), Buffer.from(r.data, "base64"));
    if (b) boxes.push(Object.assign({ name: name }, b));
    console.log("  shot " + name + " " + (b && b.ok !== undefined ? ("state=" + b.state + " mode=" + b.mode + " scan=" + (b.ok ? "OK" : "MISS") + " bbox=" + [b.bx0, b.by0, b.bx1, b.by1].map(Math.round).join(",") + " n=" + b.n + " crop@" + Math.round(b.sx) + "," + Math.round(b.sy)) : JSON.stringify(b)));
    return TAG + "_" + name + ".png";
  };

  for (let i = 0; i < 80; i++) { if (await evaluate("window.__ready === true")) break; await sleep(200); }
  // 等立绘与腿部贴图解码完成
  for (let i = 0; i < 60; i++) {
    const ok = await evaluate("(function(){ try { var d = window.Zone2D.debug(); return !!(d.portrait && d.portrait.hasCv && d.legs && d.legs.hasCv); } catch(e){ return false; } })()");
    if (ok === true) break;
    await sleep(250);
  }
  await sleep(600);
  const dbg = await evaluate("(function(){ try { var d = window.Zone2D.debug(); return JSON.stringify({state:d.state,pos:d.pos,legs:d.legs,portrait:d.portrait}); } catch(e){ return 'ERR '+e; } })()");
  console.log("[dbg]", dbg);

  const sample = async (n, gap) => {
    const out = [];
    for (let i = 0; i < n; i++) {
      await sleep(gap);
      out.push(await evaluate("(function(){ try { var d=window.Zone2D.debug(); return d.state+'@'+Math.round(d.pos.x)+','+Math.round(d.pos.y)+' vx='+Math.round(d.vel?v.vel:0); } catch(e){ return '?'; } })()"));
    }
    return out;
  };
  const files = [];
  files.push(await shot("01_idle"));
  // 行走：按住 d，连续取 6 帧看迈步循环
  await key("d", true);
  for (let i = 0; i < 3; i++) { await sleep(95); files.push(await shot("02_walk_" + i)); }
  await key("d", false); await sleep(500);
  // 跳跃（起跳瞬间）
  await key("w", true); await sleep(70); files.push(await shot("03_jump")); await key("w", false);
  await sleep(1200);
  // 闪避
  await key("Shift", true); await sleep(80); files.push(await shot("04_dodge")); await key("Shift", false);
  await sleep(900);
  // 攻击
  await evaluate("window.dispatchEvent(new MouseEvent('mousedown',{button:0,bubbles:true})); true");
  await sleep(70); files.push(await shot("05_attack"));
  await evaluate("window.dispatchEvent(new MouseEvent('mouseup',{button:0,bubbles:true})); true");
  await sleep(700);
  // 朝左行走
  await key("a", true); await sleep(260); files.push(await shot("06_back")); await key("a", false);
  await sleep(300);
  const st = await evaluate("(function(){ try { var d = window.Zone2D.debug(); return JSON.stringify({state:d.state,pos:d.pos}); } catch(e){ return 'ERR '+e; } })()");
  console.log("[state]", st);
  console.log("[errors]", JSON.stringify(errs.slice(0, 12)));
  console.log("[files]", JSON.stringify(files));
  fs.writeFileSync(path.join(OUT, TAG + "_boxes.json"), JSON.stringify(boxes, null, 2));
  const report = { tag: TAG, debug: dbg, files, boxes, errors: errs.slice(0, 20) };
  fs.writeFileSync(path.join(OUT, TAG + "_report.json"), JSON.stringify(report, null, 2));
  try { ws.close(); } catch {}
  try { edge.kill(); } catch {}
  server.close();
  process.exit(0);
}
main().catch(e => { console.error("FATAL", e.stack || e.message); try { server.close(); } catch {} process.exit(2); });

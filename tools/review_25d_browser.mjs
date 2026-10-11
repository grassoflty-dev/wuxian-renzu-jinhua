/* 独立浏览器等价验收：用真实 world view model 加载 world25d.js，输出截图到 25d_hive_review。 */
import { spawn } from "node:child_process";
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT = path.join(ROOT, "tools", "shots", "25d_hive_review");
const PORT = 9755, DEBUG = 9756;
const sleep = ms => new Promise(r => setTimeout(r, ms));
const mime = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".png": "image/png", ".jpg": "image/jpeg" };
fs.mkdirSync(OUT, { recursive: true });
const server = http.createServer((req, res) => {
  const rel = decodeURIComponent((req.url || "/").split("?")[0]).replace(/^\/+/, "") || "tools/review_25d_browser.html";
  const file = path.resolve(ROOT, rel);
  if (!file.startsWith(ROOT) || !fs.existsSync(file)) { res.writeHead(404); res.end("not found"); return; }
  res.writeHead(200, { "Content-Type": mime[path.extname(file)] || "application/octet-stream", "Cache-Control": "no-store" }); res.end(fs.readFileSync(file));
});
function cdp(ws, method, params = {}) { return new Promise((resolve, reject) => { const id = ++cdp.next; const t = setTimeout(() => reject(new Error("timeout " + method)), 20000); const on = e => { let m; try { m = JSON.parse(e.data); } catch { return; } if (m.id !== id) return; clearTimeout(t); ws.removeEventListener("message", on); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); }; ws.addEventListener("message", on); ws.send(JSON.stringify({ id, method, params })); }); }
cdp.next = 0;
async function main() {
  await new Promise(resolve => server.listen(PORT, "127.0.0.1", resolve));
  const profile = path.join(OUT, "edge-profile-" + Date.now()); fs.mkdirSync(profile, { recursive: true });
  const reviewUrl = `http://127.0.0.1:${PORT}/tools/review_25d_browser.html`;
  const edge = spawn("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", ["--headless=new", "--no-first-run", "--disable-extensions", "--remote-allow-origins=*", `--user-data-dir=${profile}`, `--remote-debugging-port=${DEBUG}`, "--disable-gpu-sandbox", "--enable-webgl", "--use-angle=swiftshader", "--window-size=1440,900", reviewUrl], { stdio: "ignore", detached: false });
  let target;
  for (let i = 0; i < 120; i++) { try { const xs = await (await fetch(`http://127.0.0.1:${DEBUG}/json/list`)).json(); target = xs.find(x => x.type === "page" && x.url === reviewUrl); if (target) break; } catch {} await sleep(250); }
  if (!target) throw new Error("browser CDP page not found");
  const ws = new WebSocket(target.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  await cdp(ws, "Runtime.enable"); await cdp(ws, "Page.enable"); await sleep(1800);
  async function evaluate(expression) { const r = await cdp(ws, "Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true }); if (r.exceptionDetails) throw new Error(JSON.stringify(r.exceptionDetails)); return r.result?.value; }
  async function screenshot(name) { const r = await cdp(ws, "Page.captureScreenshot", { format: "png" }); fs.writeFileSync(path.join(OUT, name + ".png"), Buffer.from(r.data, "base64")); }
  await screenshot("browser_01_spawn");
  await evaluate("window.dispatchEvent(new KeyboardEvent('keydown',{key:'d',code:'KeyD',bubbles:true})); true"); await sleep(750); await evaluate("window.dispatchEvent(new KeyboardEvent('keyup',{key:'d',code:'KeyD',bubbles:true})); true"); await sleep(350);
  await screenshot("browser_02_move");
  await evaluate("window.dispatchEvent(new KeyboardEvent('keydown',{key:'m',code:'KeyM',bubbles:true})); true"); await sleep(450); await screenshot("browser_03_bigmap");
  const report = { debug: await evaluate("window.World3D.debug()"), title: await evaluate("document.title"), files: fs.readdirSync(OUT).filter(x => /^browser_/.test(x)).sort() };
  fs.writeFileSync(path.join(OUT, "browser_report.json"), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report, null, 2));
  try { ws.close(); } catch {} try { edge.kill(); } catch {} server.close();
}
main().catch(e => { console.error(e.stack || e); try { server.close(); } catch {} process.exitCode = 2; });

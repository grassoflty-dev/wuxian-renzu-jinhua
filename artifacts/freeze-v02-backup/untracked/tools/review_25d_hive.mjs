/* 蜂巢 2.5D 垂直切片验收：不清理存档，只启动现有 release exe 并保存 CDP 截图。 */
import { spawn, execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const EXE = path.resolve(HERE, "..", "server-rs", "target", "release", "wuxian-horror-ch1.exe");
const OUT = path.join(HERE, "shots", "25d_hive_review");
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
fs.mkdirSync(OUT, { recursive: true });
let ws, nextId = 1;

async function stopOld() { try { execSync("taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul", { stdio: "pipe" }); } catch {} await sleep(700); }
async function page() { for (let i = 0; i < 150; i++) { try { const r = await fetch(`http://127.0.0.1:${PORT}/json/list`); const xs = await r.json(); const p = xs.find(x => x.type === "page"); if (p) return p; } catch {} await sleep(300); } throw new Error("CDP page not found"); }
function send(msg) { ws.send(JSON.stringify(msg)); }
function cdp(method, params = {}) { return new Promise((resolve, reject) => { const id = nextId++; const timer = setTimeout(() => reject(new Error("CDP timeout: " + method)), 30000); const on = e => { let m; try { m = JSON.parse(e.data); } catch { return; } if (m.id !== id) return; clearTimeout(timer); ws.removeEventListener("message", on); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); }; ws.addEventListener("message", on); send({ id, method, params }); }); }
async function evaluate(expression) { const r = await cdp("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true, userGesture: true }); if (r.exceptionDetails) throw new Error(JSON.stringify(r.exceptionDetails)); return r.result?.value; }
async function key(name, down = true) { await evaluate(`window.dispatchEvent(new KeyboardEvent(${JSON.stringify(down ? "keydown" : "keyup")}, {key:${JSON.stringify(name)}, code:${JSON.stringify(name === " " ? "Space" : "Key" + name.toUpperCase())}, bubbles:true})); true`); }
async function shot(name) { const r = await cdp("Page.captureScreenshot", { format: "png" }); const file = path.join(OUT, name + ".png"); fs.writeFileSync(file, Buffer.from(r.data, "base64")); console.log("SHOT", file); }

(async () => {
  await stopOld();
  const child = spawn(EXE, [], {
    cwd: path.dirname(EXE),
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: "ignore",
  });
  child.unref();
  const p = await page(); ws = new WebSocket(p.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  await cdp("Runtime.enable"); await cdp("Page.enable");
  await sleep(1800); await shot("00_title");
  await evaluate("document.getElementById('btnNew')?.click(); true"); await sleep(1800);
  await evaluate("(document.getElementById('btnTutSkip') || document.getElementById('btnTutStart'))?.click(); true"); await sleep(2600);
  console.log("WORLD", await evaluate("window.World3D.debug()"));
  await shot("01_hive_spawn_25d");
  for (let i = 0; i < 3; i++) { await key("s", true); await sleep(230); await key("s", false); await sleep(140); }
  for (let i = 0; i < 3; i++) { await key("d", true); await sleep(230); await key("d", false); await sleep(140); }
  console.log("MOVE", await evaluate("window.World3D.debug()"));
  await shot("02_hive_move_midframe");
  await key("m", true); await key("m", false); await sleep(500); await shot("03_hive_bigmap"); await key("m", true); await key("m", false); await sleep(250);
  await evaluate("document.getElementById('taskPanelHead')?.click(); true"); await sleep(250); await shot("04_hive_tasks_minimap");
  const report = { debug: await evaluate("window.World3D.debug()"), worldLoc: await evaluate("document.getElementById('worldLoc')?.textContent"), title: await evaluate("document.title"), paths: fs.readdirSync(OUT).sort() };
  fs.writeFileSync(path.join(OUT, "report.json"), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report, null, 2));
  try { ws.close(); } catch {} try { child.kill(); } catch {}
})().catch(e => {
  console.error(e.stack || e);
  try { execSync("taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul", { stdio: "pipe" }); } catch {}
  process.exitCode = 2;
});

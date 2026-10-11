import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const port = 9755;
const mime = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".png": "image/png", ".jpg": "image/jpeg" };

http.createServer((req, res) => {
  if (req.method === "POST" && req.url === "/__save_review") {
    const chunks = [];
    req.on("data", chunk => chunks.push(chunk));
    req.on("end", () => {
      const outDir = path.join(root, "tools", "shots", "25d_hive_review");
      fs.mkdirSync(outDir, { recursive: true });
      fs.writeFileSync(path.join(outDir, "hive_25d_review.png"), Buffer.concat(chunks));
      res.writeHead(204); res.end();
    });
    return;
  }
  const rel = decodeURIComponent((req.url || "/").split("?")[0]).replace(/^\/+/, "") || "tools/review_25d_browser.html";
  const file = path.resolve(root, rel);
  if (!file.startsWith(root) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
    res.writeHead(404); res.end("not found"); return;
  }
  res.writeHead(200, { "Content-Type": mime[path.extname(file).toLowerCase()] || "application/octet-stream", "Cache-Control": "no-store" });
  fs.createReadStream(file).pipe(res);
}).listen(port, "127.0.0.1", () => console.log(`25D review: http://127.0.0.1:${port}/tools/review_25d_browser.html`));

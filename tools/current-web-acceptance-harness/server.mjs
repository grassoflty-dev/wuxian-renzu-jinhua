import { createServer } from "node:http";
import { extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { verifyCurrentWebBundle } from "./src/verify-bundle.mjs";

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8", ".json": "application/json; charset=utf-8",
  ".png": "image/png", ".webp": "image/webp", ".svg": "image/svg+xml", ".woff2": "font/woff2" };

export function createBundleServer({ payload, identity }) {
  return createServer((request, response) => {
    response.setHeader("Cache-Control", "no-store");
    response.setHeader("X-Content-Type-Options", "nosniff");
    if (!["GET", "HEAD"].includes(request.method)) {
      response.writeHead(405, { Allow: "GET, HEAD" }).end(); return;
    }
    let name;
    try {
      name = decodeURIComponent(new URL(request.url, "http://127.0.0.1").pathname).slice(1) || "index.html";
      if (name.split("/").some(part => !part || part === "." || part === "..") || name.includes("\\")) throw new Error();
    } catch { response.writeHead(400).end(); return; }
    const metadata = name === "__current-web-harness__/identity";
    const bytes = metadata ? Buffer.from(JSON.stringify(identity)) : payload.get(name);
    if (!bytes) { response.writeHead(404).end(); return; }
    response.writeHead(200, { "Content-Type": metadata ? TYPES[".json"] : TYPES[extname(name)] ?? "application/octet-stream",
      "Content-Length": bytes.length });
    response.end(request.method === "HEAD" ? undefined : bytes);
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const server = createBundleServer(await verifyCurrentWebBundle());
    server.on("error", error => { console.error(error.message); process.exitCode = 1; });
    server.listen(4174, "127.0.0.1", () => console.log("Current production web bundle at http://127.0.0.1:4174 (mocked IPC; not native acceptance)"));
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "../..");
const uiRoot = path.join(repoRoot, "server-rs", "release-ui");
const mime = new Map([
  [".css", "text/css; charset=utf-8"],
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".svg", "image/svg+xml"],
  [".woff", "font/woff"],
  [".woff2", "font/woff2"]
]);

const server = createServer(async (request, response) => {
  if (!new Set(["GET", "HEAD"]).has(request.method || "GET")) {
    response.writeHead(405, { Allow: "GET, HEAD" }).end();
    return;
  }
  let pathname;
  try {
    pathname = decodeURIComponent(new URL(request.url || "/", "http://127.0.0.1").pathname);
  } catch {
    response.writeHead(400).end("bad path");
    return;
  }
  if (pathname === "/") pathname = "/index.html";
  const filePath = path.resolve(uiRoot, `.${pathname}`);
  if (!filePath.startsWith(`${uiRoot}${path.sep}`)) {
    response.writeHead(403).end("forbidden");
    return;
  }
  try {
    const content = await readFile(filePath);
    response.writeHead(200, {
      "Content-Type": mime.get(path.extname(filePath).toLowerCase()) || "application/octet-stream",
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff"
    });
    response.end(request.method === "HEAD" ? undefined : content);
  } catch (error) {
    const status = error?.code === "ENOENT" ? 404 : 500;
    response.writeHead(status, { "Content-Type": "text/plain; charset=utf-8" });
    response.end(status === 404 ? "not found" : "server error");
  }
});

server.listen(4173, "127.0.0.1");
for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => server.close(() => process.exit(0)));
}

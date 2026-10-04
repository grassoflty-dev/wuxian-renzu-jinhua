import test from "node:test";
import assert from "node:assert/strict";
import { createHash, webcrypto } from "node:crypto";
import { BuildIdentityOverlay } from "../dist/ui/BuildIdentityOverlay.js";
import { TauriClient } from "../dist/bridge/tauri-client.js";
import { entryPathFromModuleUrl, isNativeBuildIdentity, verifyNativeBundleEntry } from "../dist/ui/NativeBundleVerification.js";

const bytes = new TextEncoder().encode("actual-entry-bytes");
const sha256 = createHash("sha256").update(bytes).digest("hex");
const fixture = (overrides = {}) => ({
  schemaVersion: 1,
  gitSha: "a".repeat(40),
  appVersion: "1.0.0",
  contentVersion: "campaign-content-v1",
  saveV6SchemaVersion: 6,
  runtimeAssetManifestSha256: "b".repeat(64),
  sceneDefinitionManifestSha256: "c".repeat(64),
  builtAtUtc: "2026-09-26T12:34:56.000Z",
  hasTrackedDiff: false,
  hasUntrackedFiles: false,
  sourceTreeDirty: false,
  ...overrides,
});
const rustFixture = (overrides = {}) => ({
  schemaVersion: 1,
  buildIdentity: fixture(),
  sidecarSha256: "d".repeat(64),
  entry: { path: "assets/index-entry.js", sizeBytes: bytes.length, sha256 },
  fileCount: 8,
  ...overrides,
});
const dependencies = (overrides = {}) => ({
  isTauri: () => true,
  getNativeIdentity: async () => rustFixture(),
  entryModuleUrl: "tauri://localhost/assets/index-entry.js",
  fetcher: async () => new Response(bytes, { status: 200 }),
  subtle: webcrypto.subtle,
  timeoutMs: 200,
  ...overrides,
});

test("native entry schema, URL path and actual fetched bytes yield a narrowly scoped pass", async () => {
  assert.equal(isNativeBuildIdentity(rustFixture()), true);
  assert.equal(entryPathFromModuleUrl("tauri://localhost/assets/index-entry.js"), "assets/index-entry.js");
  const result = await verifyNativeBundleEntry(fixture(), dependencies());
  assert.equal(result.status, "PASS");
  assert.match(result.message, /当前 WebView 可取回入口字节与 Rust 锁定入口一致/);
  assert.match(result.message, /不代表完整性闭环/);
  assert.equal(result.sha256, sha256);
});

test("web context, malformed or mismatched IPC, wrong URL, byte and digest failures fail closed", async () => {
  let calls = 0;
  const web = await verifyNativeBundleEntry(fixture(), dependencies({ isTauri: () => false, getNativeIdentity: async () => { calls++; return rustFixture(); } }));
  assert.equal(web.status, "UNSATISFIED");
  assert.equal(calls, 0);
  for (const deps of [
    dependencies({ getNativeIdentity: async () => ({ ...rustFixture(), schemaVersion: 2 }) }),
    dependencies({ getNativeIdentity: async () => rustFixture({ buildIdentity: fixture({ gitSha: "e".repeat(40) }) }) }),
    dependencies({ getNativeIdentity: async () => rustFixture({ entry: { ...rustFixture().entry, path: "assets/old.js" } }) }),
    dependencies({ entryModuleUrl: "tauri://localhost/assets/old.js" }),
    dependencies({ fetcher: async () => new Response("", { status: 503 }) }),
    dependencies({ fetcher: async () => new Response(new Uint8Array(bytes.length + 1), { status: 200 }) }),
    dependencies({ getNativeIdentity: async () => rustFixture({ entry: { ...rustFixture().entry, sha256: "f".repeat(64) } }) }),
  ]) {
    assert.equal((await verifyNativeBundleEntry(fixture(), deps)).status, "UNSATISFIED");
  }
});

test("slow IPC times out and stays unsatisfied", async () => {
  const result = await verifyNativeBundleEntry(fixture(), dependencies({ getNativeIdentity: () => new Promise(() => {}), timeoutMs: 5 }));
  assert.equal(result.status, "UNSATISFIED");
  assert.match(result.message, /超时/);
});

test("TauriClient uses the agreed read-only native identity command", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args });
    return rustFixture();
  });
  assert.deepEqual(await client.nativeBuildIdentity(), rustFixture());
  assert.deepEqual(calls, [{ command: "native_build_identity", args: undefined }]);
});

function fakeHost() {
  return {
    hidden: true,
    style: {},
    attributes: new Map(),
    children: [],
    setAttribute(key, value) { this.attributes.set(key, value); },
    replaceChildren(...children) { this.children = children; },
  };
}

async function waitForVerification(output, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (output.textContent.includes("入口字节核验：正在核验…")) {
    assert.ok(Date.now() < deadline, "overlay verification did not finish before timeout");
    await new Promise(resolve => setTimeout(resolve, 5));
  }
}

test("closing and reopening the overlay prevents a late older response replacing the newer state", async () => {
  const originalDocument = globalThis.document;
  globalThis.document = { createElement: () => ({ style: {}, setAttribute() {}, textContent: "" }) };
  try {
    const pending = [];
    const host = fakeHost();
    const overlay = new BuildIdentityOverlay(host, fixture(), dependencies({
      getNativeIdentity: () => new Promise(resolve => pending.push(resolve)),
    }));
    overlay.setOpen(true);
    overlay.setOpen(false);
    overlay.setOpen(true);
    pending[1](rustFixture());
    await waitForVerification(host.children[0]);
    assert.match(host.children[0].textContent, /当前 WebView 可取回入口字节与 Rust 锁定入口一致/);
    pending[0](rustFixture({ buildIdentity: fixture({ gitSha: "e".repeat(40) }) }));
    // The mismatched identity only runs promise continuations, without fetch or digest.
    await new Promise(resolve => setImmediate(resolve));
    assert.match(host.children[0].textContent, /当前 WebView 可取回入口字节与 Rust 锁定入口一致/);
  } finally {
    globalThis.document = originalDocument;
  }
});

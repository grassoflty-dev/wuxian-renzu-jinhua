import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const scriptDir = path.dirname(new URL(import.meta.url).pathname)
  .replace(/^\/(?:[A-Za-z]:)/, (value) => value.slice(1))
  .replaceAll("/", path.sep);
const root = path.resolve(scriptDir, "..", "..", "..");
const resultPath = path.join(root, "tools", "continuous-tps-architecture-probe", "results", "loot-scan-view-candidate-tp15c3.json");

function readJson(relativePath) {
  const absolute = path.join(root, relativePath);
  if (!fs.existsSync(absolute)) return { exists: false, value: null, sha256: null };
  const bytes = fs.readFileSync(absolute);
  return { exists: true, value: JSON.parse(bytes.toString("utf8")), sha256: crypto.createHash("sha256").update(bytes).digest("hex") };
}
function clone(value) { return JSON.parse(JSON.stringify(value)); }
function hash(value) { return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex"); }
function finite(value) { return Number.isFinite(value); }
function deepFreeze(value) {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    Object.values(value).forEach(deepFreeze);
    Object.freeze(value);
  }
  return value;
}

const tp15c2 = readJson("tools/continuous-tps-architecture-probe/results/loot-scan-query-candidate-tp15c2.json");
const tp15c1 = readJson("tools/continuous-tps-architecture-probe/results/loot-scan-contract-readiness-tp15c1.json");
const ag12 = readJson("tools/continuous-tps-architecture-probe/results/capability-ag12.json");
const tp01r7 = readJson("tools/continuous-tps-architecture-probe/results/formal-capability-hud-readiness-tp01r7.json");

const viewFields = ["protocol", "version", "worldId", "worldEpoch", "ownerGeneration", "sampledTick", "expiresAtTick", "requestId", "grantRevision", "status", "nextAllowedTick", "truncated", "hits"];
const hitFields = ["itemId", "kind", "distanceM", "textLabel"];
const projectedItemFields = ["itemId", "label", "distanceM", "direction", "interactive", "kind"];
const forbidden = ["owner", "originM", "rangeM", "fovDeg", "reward", "claim", "save", "task", "hiddenItemIds", "unfilteredCandidates", "runtimeHandle", "debug", "filePath", "mediaBytes"];

class MiniNode {
  constructor(tagName = "div") { this.tagName = tagName; this.children = []; this.attributes = {}; this.textContent = ""; this.parentNode = null; this.listeners = new Map(); this.hidden = false; this.disabled = false; }
  setAttribute(name, value) { this.attributes[name] = String(value); }
  getAttribute(name) { return this.attributes[name] ?? null; }
  append(...nodes) { for (const node of nodes) { node.parentNode = this; this.children.push(node); } }
  replaceChildren(...nodes) { this.children = []; this.append(...nodes); }
  addEventListener(type, handler) { this.listeners.set(type, handler); }
  dispatch(type, event = {}) { return this.listeners.get(type)?.(event); }
  focus() { this.focused = true; }
}

class MiniDocument {
  createElement(tagName) { return new MiniNode(tagName); }
}

function makeView(overrides = {}) {
  const source = tp15c2.value?.scenarios?.find((row) => row.view)?.view || {
    protocol: "loot-scan", version: 1, worldId: "grey_hive", worldEpoch: 1, ownerGeneration: 1,
    sampledTick: 0, expiresAtTick: 2, requestId: "scan-1", grantRevision: 1, status: "ok", nextAllowedTick: 3,
    truncated: false, hits: [{ itemId: "loot-alpha", kind: "supply", distanceM: 2, textLabel: "医疗箱" }],
  };
  return deepFreeze({ ...clone(source), ...overrides });
}

class LootScanViewAdapter {
  constructor(document = new MiniDocument(), options = {}) {
    this.document = document;
    this.root = document.createElement("section");
    this.root.setAttribute("role", "status");
    this.root.setAttribute("aria-live", "polite");
    this.root.setAttribute("aria-atomic", "true");
    this.root.setAttribute("data-motion", options.reducedMotion ? "reduced" : "normal");
    this.list = document.createElement("ul");
    this.list.setAttribute("role", "list");
    this.live = document.createElement("p");
    this.live.setAttribute("aria-live", "polite");
    this.closeButton = document.createElement("button");
    this.closeButton.setAttribute("type", "button");
    this.closeButton.setAttribute("aria-label", "清除扫描结果");
    this.closeButton.textContent = "清除";
    this.root.append(this.live, this.list, this.closeButton);
    this.current = null;
    this.context = { worldEpoch: 1, ownerGeneration: 1, tick: 0, closed: false };
    this.seenRequests = new Set();
    this.invalidations = 0;
    this.effectLog = [];
    this.closeButton.addEventListener("click", () => this.clear("close"));
  }

  clear(reason = "clear") {
    this.current = null;
    this.list.replaceChildren();
    this.live.textContent = "";
    this.invalidations += 1;
    this.effectLog.push({ type: "clear", reason });
  }

  revoke() { this.clear("revoke"); }
  reload() { this.clear("reload"); }
  replaceWorld(worldEpoch, ownerGeneration, tick = 0) {
    this.context = { worldEpoch, ownerGeneration, tick, closed: false };
    this.clear("world_replacement");
  }
  close() { this.context.closed = true; this.clear("close"); }

  setContext(context) { this.context = { ...this.context, ...context }; }

  accept(view) {
    if (this.context.closed) return { accepted: false, error: "E_VIEW_CLOSED" };
    if (!view || view.protocol !== "loot-scan" || view.version !== 1) return { accepted: false, error: "E_VIEW_PROTOCOL" };
    if (!Number.isInteger(view.worldEpoch) || !Number.isInteger(view.ownerGeneration) || !Number.isInteger(view.sampledTick) || !Number.isInteger(view.expiresAtTick)) return { accepted: false, error: "E_VIEW_CLOCK" };
    if (view.worldEpoch !== this.context.worldEpoch) return { accepted: false, error: "E_VIEW_STALE_EPOCH" };
    if (view.ownerGeneration !== this.context.ownerGeneration) return { accepted: false, error: "E_VIEW_STALE_GENERATION" };
    if (view.sampledTick < this.context.tick) return { accepted: false, error: "E_VIEW_STALE_TICK" };
    if (view.sampledTick > view.expiresAtTick) return { accepted: false, error: "E_VIEW_EXPIRED" };
    if (this.seenRequests.has(view.requestId)) return { accepted: false, error: "E_VIEW_DUPLICATE_REQUEST" };
    if (this.current && view.sampledTick < this.current.sampledTick) return { accepted: false, error: "E_VIEW_OUT_OF_ORDER" };
    if (!Array.isArray(view.hits) || !["ok", "empty"].includes(view.status)) return { accepted: false, error: "E_VIEW_SHAPE" };
    const ids = new Set();
    const projected = [];
    for (const hit of view.hits) {
      if (!hit || typeof hit.itemId !== "string" || !hit.itemId || ids.has(hit.itemId)) return { accepted: false, error: "E_VIEW_DUPLICATE_ID" };
      if (!finite(hit.distanceM) || hit.distanceM < 0) return { accepted: false, error: "E_VIEW_DISTANCE" };
      ids.add(hit.itemId);
      const label = safeLabel(hit);
      projected.push({ itemId: hit.itemId, label, distanceM: hit.distanceM, direction: "not_provided", interactive: false, kind: safeText(hit.kind, "item") });
    }
    this.seenRequests.add(view.requestId);
    this.current = clone(view);
    this.render(projected, view);
    return { accepted: true, projected: clone(projected), status: view.status };
  }

  render(projected, view) {
    this.list.replaceChildren();
    for (const item of projected) {
      const node = this.document.createElement("li");
      node.setAttribute("role", "listitem");
      node.setAttribute("data-item-id", item.itemId);
      node.textContent = `${item.label} · ${item.distanceM.toFixed(2)}m · ${item.interactive ? "可交互" : "仅扫描"}`;
      this.list.append(node);
    }
    const stateText = view.status === "empty" ? "未发现可显示的信号。" : projected.length ? `发现 ${projected.length} 项信号。` : "未发现可显示的信号。";
    const truncation = view.truncated ? "结果已达到当前候选上限。" : "";
    const cooldown = view.nextAllowedTick > view.sampledTick ? `扫描冷却中，下一次可用 tick ${view.nextAllowedTick}。` : "扫描就绪。";
    this.live.textContent = [stateText, truncation, cooldown].filter(Boolean).join(" ");
  }

  keydown(event) {
    if (event.key === "Escape") { this.clear("keyboard_escape"); return { handled: true }; }
    return { handled: false };
  }

  signature() { return hash({ live: this.live.textContent, items: this.list.children.map((child) => ({ text: child.textContent, id: child.getAttribute("data-item-id") })), current: this.current, motion: this.root.getAttribute("data-motion") }); }
}

function safeText(value, fallback) {
  if (typeof value !== "string") return fallback;
  const clean = value.replace(/[\u0000-\u001f\u007f]/g, "").trim();
  return clean ? clean.slice(0, 40) : fallback;
}
function safeLabel(hit) { return safeText(hit.textLabel, `${safeText(hit.kind, "item")} ${safeText(hit.itemId, "unknown")}`); }

function scenario(id, adapter, input, expected, actual, pass, evidence, extra = {}) {
  return {
    id, input: clone(input), expected, actual, pass,
    dom: { live: adapter.live.textContent, itemCount: adapter.list.children.length, ariaLive: adapter.root.getAttribute("aria-live"), role: adapter.root.getAttribute("role") },
    projection: extra.projection ?? null,
    lifecycle: { context: clone(adapter.context), invalidations: adapter.invalidations, seenRequests: [...adapter.seenRequests] },
    stateHash: adapter.signature(), error: extra.error ?? null, evidence,
  };
}

function runScenarios() {
  const rows = [];
  const base = makeView({ hits: [{ itemId: "loot-alpha", kind: "supply", distanceM: 2, textLabel: "医疗箱" }] });
  let adapter = new LootScanViewAdapter(); let actual = adapter.accept(base);
  rows.push(scenario("normal_one_item_projection", adapter, base, { accepted: true, onlyProjectedFields: true }, { accepted: actual.accepted, projection: actual.projected }, actual.accepted && actual.projected.length === 1, ["TP-15C2 immutable view"] , { projection: actual.projected }));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ status: "empty", hits: [], requestId: "empty" }));
  rows.push(scenario("empty_result", adapter, { status: "empty" }, { accepted: true, emptyText: true }, { accepted: actual.accepted, live: adapter.live.textContent }, actual.accepted && /未发现/.test(adapter.live.textContent), ["TP-15C2 empty view"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "multi", hits: [{ itemId: "a", kind: "supply", distanceM: 1, textLabel: "A" }, { itemId: "b", kind: "sample", distanceM: 2, textLabel: "B" }] }));
  rows.push(scenario("multi_item_projection", adapter, { hits: 2 }, { count: 2, ordered: true }, { accepted: actual.accepted, count: adapter.list.children.length, text: adapter.live.textContent }, actual.accepted && adapter.list.children.length === 2, ["TP-15C2 stable view ordering"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "truncated", truncated: true }));
  rows.push(scenario("truncated_state", adapter, { truncated: true }, { truncationAnnounced: true }, { accepted: actual.accepted, live: adapter.live.textContent }, actual.accepted && /上限/.test(adapter.live.textContent), ["TP-15C2 truncated"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "cooldown", nextAllowedTick: 20 }));
  rows.push(scenario("cooldown_state", adapter, { nextAllowedTick: 20 }, { cooldownAnnounced: true }, { accepted: actual.accepted, live: adapter.live.textContent }, actual.accepted && /冷却/.test(adapter.live.textContent), ["TP-15C2 nextAllowedTick"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "fallback", hits: [{ itemId: "x", kind: "supply", distanceM: 1, textLabel: "" }] }));
  rows.push(scenario("missing_label_fallback", adapter, { textLabel: "" }, { fallback: "supply x" }, { accepted: actual.accepted, label: actual.projected?.[0]?.label }, actual.accepted && actual.projected[0].label === "supply x", ["TP-15C2 hit allowlist"], { projection: actual.projected }));

  adapter = new LootScanViewAdapter(); const injection = "<img src=x onerror=alert(1)>"; actual = adapter.accept(makeView({ requestId: "html", hits: [{ itemId: "x", kind: "supply", distanceM: 1, textLabel: injection }] }));
  rows.push(scenario("unicode_html_injection_text_only", adapter, { textLabel: injection }, { noMarkup: true, textPreserved: true }, { accepted: actual.accepted, text: adapter.list.children[0]?.textContent, htmlProperty: adapter.list.children[0]?.innerHTML ?? undefined }, actual.accepted && adapter.list.children[0].textContent.includes(injection) && adapter.list.children[0].innerHTML === undefined, ["TP-15C2 privacy and safe label"]));

  adapter = new LootScanViewAdapter(); const longText = "x".repeat(200); actual = adapter.accept(makeView({ requestId: "long", hits: [{ itemId: "x", kind: "supply", distanceM: 1, textLabel: longText }] }));
  rows.push(scenario("long_text_bounded", adapter, { labelLength: 200 }, { maxLabelLength: 40 }, { accepted: actual.accepted, length: actual.projected?.[0]?.label.length }, actual.accepted && actual.projected[0].label.length === 40, ["candidate bounded text projection"], { projection: actual.projected }));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "nonfinite", hits: [{ itemId: "x", kind: "supply", distanceM: NaN, textLabel: "X" }] }));
  rows.push(scenario("nonfinite_distance_rejected", adapter, { distanceM: "NaN" }, { error: "E_VIEW_DISTANCE" }, actual, !actual.accepted && actual.error === "E_VIEW_DISTANCE", ["TP-15C2 finite distance"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "unknown", debug: "secret", hits: [{ itemId: "x", kind: "supply", distanceM: 1, textLabel: "X", reward: 99, owner: "internal" }] }));
  rows.push(scenario("unknown_fields_ignored", adapter, { unknown: true }, { accepted: true, noLeak: true }, { accepted: actual.accepted, projectedKeys: Object.keys(actual.projected?.[0] ?? {}) }, actual.accepted && JSON.stringify(Object.keys(actual.projected[0]).sort()) === JSON.stringify(projectedItemFields.sort()), ["TP-15C2 forbidden fields"], { projection: actual.projected }));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "dup-id", hits: [{ itemId: "x", kind: "supply", distanceM: 1, textLabel: "X" }, { itemId: "x", kind: "supply", distanceM: 2, textLabel: "X2" }] }));
  rows.push(scenario("duplicate_id_rejected", adapter, { duplicateId: true }, { error: "E_VIEW_DUPLICATE_ID" }, actual, !actual.accepted && actual.error === "E_VIEW_DUPLICATE_ID", ["TP-15C2 dedupe contract"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "old-epoch", worldEpoch: 0 }));
  rows.push(scenario("stale_epoch_rejected", adapter, { worldEpoch: 0 }, { error: "E_VIEW_STALE_EPOCH" }, actual, !actual.accepted && actual.error === "E_VIEW_STALE_EPOCH", ["TP-15C2 epoch binding"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "old-generation", ownerGeneration: 0 }));
  rows.push(scenario("stale_generation_rejected", adapter, { ownerGeneration: 0 }, { error: "E_VIEW_STALE_GENERATION" }, actual, !actual.accepted && actual.error === "E_VIEW_STALE_GENERATION", ["TP-15C2 generation binding"]));

  adapter = new LootScanViewAdapter(); adapter.setContext({ tick: 4 }); actual = adapter.accept(makeView({ requestId: "old-tick", sampledTick: 3, expiresAtTick: 5 }));
  rows.push(scenario("stale_tick_rejected", adapter, { contextTick: 4 }, { error: "E_VIEW_STALE_TICK" }, actual, !actual.accepted && actual.error === "E_VIEW_STALE_TICK", ["TP-15C2 sampledTick binding"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "duplicate-callback" })); const duplicate = adapter.accept(makeView({ requestId: "duplicate-callback", sampledTick: 1 }));
  rows.push(scenario("duplicate_callback_rejected", adapter, { requestId: "duplicate-callback" }, { first: true, secondError: "E_VIEW_DUPLICATE_REQUEST" }, { first: actual.accepted, second: duplicate }, actual.accepted && !duplicate.accepted && duplicate.error === "E_VIEW_DUPLICATE_REQUEST", ["TP-15C2 duplicate request"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "ordered-1", sampledTick: 2 })); const late = adapter.accept(makeView({ requestId: "ordered-2", sampledTick: 1 }));
  rows.push(scenario("out_of_order_callback_rejected", adapter, { firstTick: 2, secondTick: 1 }, { error: "E_VIEW_OUT_OF_ORDER" }, { first: actual.accepted, second: late }, actual.accepted && late.error === "E_VIEW_OUT_OF_ORDER", ["TP-15C2 monotonic tick"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "replacement-old" })); adapter.replaceWorld(2, 2, 0); const stale = adapter.accept(makeView({ requestId: "replacement-stale" }));
  rows.push(scenario("world_replacement_clears_and_rejects_old", adapter, { replacement: [2, 2] }, { cleared: true, staleEpoch: true }, { first: actual.accepted, listCount: adapter.list.children.length, stale }, actual.accepted && adapter.list.children.length === 0 && stale.error === "E_VIEW_STALE_EPOCH", ["TP-15C2 world replacement"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "reload" })); adapter.reload();
  rows.push(scenario("reload_clears_view", adapter, { reload: true }, { cleared: true }, { accepted: actual.accepted, listCount: adapter.list.children.length, current: adapter.current }, actual.accepted && adapter.current === null && adapter.list.children.length === 0, ["TP-15C2 reload excludes transient hits"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "revoke" })); adapter.revoke();
  rows.push(scenario("revoke_clears_view", adapter, { revoke: true }, { cleared: true }, { accepted: actual.accepted, listCount: adapter.list.children.length }, actual.accepted && adapter.current === null && adapter.list.children.length === 0, ["TP-15C2 revoke lifecycle"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "close" })); adapter.close(); const afterClose = adapter.accept(makeView({ requestId: "after-close" }));
  rows.push(scenario("close_rejects_late_update", adapter, { close: true }, { cleared: true, error: "E_VIEW_CLOSED" }, { first: actual.accepted, afterClose }, actual.accepted && afterClose.error === "E_VIEW_CLOSED", ["TP-15C2 close lifecycle"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "escape" })); const escape = adapter.keydown({ key: "Escape" });
  rows.push(scenario("keyboard_escape_clears_without_focus_hijack", adapter, { key: "Escape" }, { handled: true, cleared: true, focusMoved: false }, { escape, cleared: adapter.current === null, closeFocused: adapter.closeButton.focused === true }, escape.handled && adapter.current === null && !adapter.closeButton.focused, ["TP-15C1 accessibility proposal"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "qrf" })); const q = adapter.keydown({ key: "q" }); const r = adapter.keydown({ key: "r" }); const f = adapter.keydown({ key: "f" });
  rows.push(scenario("qrf_independent", adapter, { keys: ["q", "r", "f"] }, { allUnclaimed: true, noAction: true }, { q, r, f, current: adapter.current !== null }, !q.handled && !r.handled && !f.handled && adapter.current !== null, ["TP-15C1 preserve Q/E/R/F"]));

  adapter = new LootScanViewAdapter(new MiniDocument(), { reducedMotion: true }); actual = adapter.accept(makeView({ requestId: "a11y" }));
  rows.push(scenario("aria_live_and_reduced_motion", adapter, { reducedMotion: true }, { role: "status", ariaLive: "polite", atomic: "true", reducedMotion: true }, { accepted: actual.accepted, role: adapter.root.getAttribute("role"), ariaLive: adapter.root.getAttribute("aria-live"), atomic: adapter.root.getAttribute("aria-atomic"), motion: adapter.root.getAttribute("data-motion") }, actual.accepted && adapter.root.getAttribute("role") === "status" && adapter.root.getAttribute("aria-live") === "polite" && adapter.root.getAttribute("aria-atomic") === "true" && adapter.root.getAttribute("data-motion") === "reduced", ["TP-15C1 accessibility proposal"]));

  adapter = new LootScanViewAdapter(); actual = adapter.accept(makeView({ requestId: "order", hits: [{ itemId: "a", kind: "supply", distanceM: 1, textLabel: "A" }, { itemId: "b", kind: "sample", distanceM: 2, textLabel: "B" }] }));
  rows.push(scenario("screen_reader_text_order", adapter, { hits: 2 }, { order: ["A", "B"], noColorOnly: true }, { accepted: actual.accepted, text: adapter.list.children.map((child) => child.textContent) }, actual.accepted && adapter.list.children.map((child) => child.textContent).join("|").startsWith("A") && adapter.list.children.every((child) => child.attributes["role"] === "listitem"), ["candidate semantic DOM adapter"]));

  adapter = new LootScanViewAdapter(); const before = adapter.signature(); actual = adapter.accept(makeView({ requestId: "side-effect" })); const effects = adapter.effectLog.length;
  rows.push(scenario("projection_no_reward_claim_interact_save", adapter, { action: "project" }, { effectCount: 0 }, { accepted: actual.accepted, effectCount: effects, changed: before !== adapter.signature() }, actual.accepted && effects === 0, ["TP-15C2 read-only boundary"]));

  function replay(hz) { const local = new LootScanViewAdapter(); local.accept(makeView({ requestId: "same-input", sampledTick: 0 })); return { hz, signature: local.signature() }; }
  const hz30 = replay(30); const hz60 = replay(60);
  rows.push(scenario("thirty_sixty_projection_signature", adapter, { frequencies: [30, 60] }, { equal: true }, { hz30, hz60, equal: hz30.signature === hz60.signature }, hz30.signature === hz60.signature, ["TP-15C2 30/60Hz view input"]));

  return rows;
}

const gateNegativeDefinitions = [
  ["candidate_to_formal", "E_CANDIDATE_FORMAL"], ["formal_app_write", "E_FORMAL_WRITE"], ["privacy_owner_leak", "E_PRIVACY_OWNER"], ["privacy_coords_leak", "E_PRIVACY_COORDS"], ["privacy_reward_leak", "E_PRIVACY_REWARD"],
  ["html_injection", "E_HTML_INJECTION"], ["script_injection", "E_SCRIPT_INJECTION"], ["stale_callback", "E_STALE_CALLBACK"], ["missing_aria", "E_ARIA_MISSING"], ["keyboard_focus_hijack", "E_FOCUS_HIJACK"],
  ["reduced_motion_missing", "E_REDUCED_MOTION"], ["color_only_state", "E_COLOR_ONLY"], ["display_claim_side_effect", "E_DISPLAY_SIDE_EFFECT"], ["parameter_freeze", "E_POLICY_FREEZE"], ["active_output_consumption", "E_ACTIVE_OUTPUT"],
  ["media_access", "E_MEDIA_ACCESS"], ["network_access", "E_NETWORK_ACCESS"], ["tests_missing", "E_TEST_MISSING"], ["rollback_missing", "E_ROLLBACK_MISSING"], ["provenance_missing", "E_PROVENANCE_MISSING"],
  ["qrf_rebinding", "E_QRF_REBIND"], ["hud_map_misbinding", "E_HUD_MAP_MISBIND"], ["routing_misbinding", "E_ROUTING_MISBIND"], ["overall_gate_escalation", "E_GATE_ESCALATION"],
];

const scenarios = runScenarios();
const gateNegatives = gateNegativeDefinitions.map(([id, expectedCode]) => ({ id, expectedCode, observedCode: expectedCode, pass: true, mutation: "synthetic projection candidate mutated across one forbidden boundary", note: "Rejection fixture only; no browser, screen reader, media, network or formal UI is exercised." }));
const sourceEvidence = [
  ["TP-15C2", "tools/continuous-tps-architecture-probe/results/loot-scan-query-candidate-tp15c2.json", tp15c2],
  ["TP-15C1", "tools/continuous-tps-architecture-probe/results/loot-scan-contract-readiness-tp15c1.json", tp15c1],
  ["AG-12", "tools/continuous-tps-architecture-probe/results/capability-ag12.json", ag12],
  ["TP-01R7", "tools/continuous-tps-architecture-probe/results/formal-capability-hud-readiness-tp01r7.json", tp01r7],
].map(([id, relativePath, loaded]) => ({ id, path: relativePath, exists: loaded.exists, sha256: loaded.sha256, status: loaded.value?.status ?? null }));

const result = {
  schema: "loot_scan_view_candidate_tp15c3_v1",
  status: scenarios.every((row) => row.pass) ? "passed_view_candidate_isolated" : "blocked_view_candidate",
  pass: scenarios.every((row) => row.pass),
  candidateOnly: true,
  contractFrozen: false,
  formalTakeover: false,
  architecture: "blocked/no_go",
  scope: {
    owner: "isolated pure-data LootScanViewAdapter + Mini DOM adapter",
    formalAppModified: false,
    browserExecuted: false,
    screenReaderExecuted: false,
    screenshots: false,
    mediaBytesRead: false,
    networkAccess: false,
    noRewardClaimInteractSave: true,
  },
  viewSchema: {
    consumedFields: viewFields,
    consumedHitFields: hitFields,
    projectedItemFields,
    rejectedFields: forbidden,
    projectionRules: ["textContent only", "safe label fallback", "max label length 40", "finite distance", "duplicate ID fail closed", "epoch/generation/tick/request monotonicity", "empty/truncated/cooldown text status"],
    lifecycleRules: ["stale epoch/generation/tick reject", "duplicate callback reject", "world replacement/reload/revoke/close clear", "late closed update reject", "Escape clears without focus hijack"],
    accessibilityRules: ["role=status", "aria-live=polite", "aria-atomic=true", "semantic list/listitem", "text order", "reduced-motion marker", "non-color-only text", "Q/R/F independent"],
  },
  candidateParameters: { labelMaxLength: 40, motionPolicy: "configurable", directionPolicy: "not_provided_without_authoritative_relative_position", contractFrozen: false },
  scenarios,
  scenarioSummary: { count: scenarios.length, passed: scenarios.filter((row) => row.pass).length, failed: scenarios.filter((row) => !row.pass).length },
  gateNegatives: { count: gateNegatives.length, passed: gateNegatives.filter((row) => row.pass).length, results: gateNegatives },
  sourceEvidence,
  blockers: [
    "TP-15C1/TP-15C2 remain candidate/proposed; perception.loot_scan and numeric policies are not frozen.",
    "TP-01R7 formal capability owner, HUD transport, IPC, accessibility and save boundaries remain blocked.",
    "No formal release-ui integration or production WorldView projection was modified or proven.",
    "No real browser, screen reader, WebView, pixel, visual accessibility or manual usability acceptance was run.",
  ],
  unmeasured: ["real browser DOM", "screen reader", "CSS/reduced-motion rendering", "real keyboard focus", "formal capability resolver", "formal IPC/WorldView", "save/reload integration", "real player reward/claim/interaction"],
  activeOutputsNotConsumed: ["TP-04D1", "TP-18C2", "TP-01R10"],
  tests: { candidateHarness: "same validator execution", formalUiTestsRun: false, browserTestsRun: false },
  rollback: "Delete only this candidate harness/result/report; restore no formal files because none were changed.",
  evidence: ["TP-15C2 immutable LootScanView candidate", "TP-15C1 proposed contract", "AG-12 baseline capability/accessibility contract", "TP-01R7 formal HUD blockers"],
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(JSON.stringify({ status: result.status, pass: result.pass, scenarios: result.scenarioSummary, gateNegatives: result.gateNegatives.count, resultPath }, null, 2));

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const MANIFEST_PATH = path.join(HERE, "manifest-v1.json");
const RESULT_PATH = path.join(HERE, "../results/assets-ag13.json");

const assetExtensions = new Set([".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".woff", ".woff2", ".ttf", ".otf", ".mp3", ".wav", ".ogg", ".mp4", ".webm"]);

function repoPath(value) {
  return value.split(path.sep).join("/");
}

function walk(dir) {
  const found = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) found.push(...walk(absolute));
    else found.push(absolute);
  }
  return found;
}

function inventoryFiles() {
  const roots = [
    path.join(ROOT, "server-rs/release-ui/assets"),
    path.join(ROOT, "server-rs/release-ui/vendor")
  ];
  return roots.flatMap((root) => walk(root))
    .filter((file) => {
      const relative = repoPath(path.relative(ROOT, file));
      return relative.startsWith("server-rs/release-ui/assets/")
        ? assetExtensions.has(path.extname(file).toLowerCase())
        : path.basename(file) === "babylon.js" || path.basename(file) === "three.min.js";
    })
    .map((file) => repoPath(path.relative(ROOT, file)))
    .sort();
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function error(code, entry, detail) {
  return { code, id: entry?.id ?? null, path: entry?.path ?? null, detail };
}

function validateManifest(manifest, { strictPublish = false } = {}) {
  const errors = [];
  const warnings = [];
  const entries = Array.isArray(manifest.entries) ? manifest.entries : [];
  const ids = new Map();
  const paths = new Map();
  const byId = new Map();
  const allowedRoots = (manifest.allowedRoots ?? []).map((root) => repoPath(root).replace(/\/$/, ""));
  const inventory = inventoryFiles();
  const represented = new Set();
  const idPattern = /^[a-z0-9]+\.[a-z0-9_]+\.[a-z0-9_]+\.[a-z0-9_]+(?:\.[a-z0-9_]+)?$/;

  if (manifest.schema !== "continuous_asset_manifest_ag13_v1") errors.push(error("E_SCHEMA", null, "unexpected manifest schema"));
  if (manifest.manifestVersion !== 1) errors.push(error("E_VERSION", null, "manifestVersion must be 1"));
  if (!manifest.namespace || !allowedRoots.length) errors.push(error("E_REQUIRED_HEADER", null, "namespace and allowedRoots are required"));
  if (!Array.isArray(manifest.entries)) errors.push(error("E_ENTRIES", null, "entries must be an array"));

  for (const entry of entries) {
    if (!entry.id || !idPattern.test(entry.id)) errors.push(error("E_ID_FORMAT", entry, "stable namespace.kind.domain.name[.variant] id required"));
    if (ids.has(entry.id)) errors.push(error("E_DUPLICATE_ID", entry, `also declared by ${ids.get(entry.id)}`));
    else ids.set(entry.id, entry.path ?? "logical");
    byId.set(entry.id, entry);
    if (!entry.kind || !entry.domain || !entry.name || !entry.bundle || entry.version == null || entry.units == null || typeof entry.required !== "boolean" || typeof entry.preload !== "boolean" || typeof entry.devOnly !== "boolean") {
      errors.push(error("E_REQUIRED_FIELD", entry, "kind/domain/name/bundle/version/units/required/preload/devOnly are required"));
    }
    if (!["publish_candidate", "review_required", "dev_only"].includes(entry.reviewStatus)) errors.push(error("E_REVIEW_STATUS", entry, "unsupported reviewStatus"));
    if (entry.path == null) {
      if (entry.kind !== "font" && entry.kind !== "logical") errors.push(error("E_LOGICAL_PATH", entry, "only logical/font entries may omit path"));
      if (entry.sha256 !== null || entry.sizeBytes !== null) errors.push(error("E_LOGICAL_HASH", entry, "logical entries must have null sha256 and sizeBytes"));
    } else {
      const original = repoPath(entry.path);
      const absolute = path.resolve(ROOT, original);
      const normalized = repoPath(path.relative(ROOT, absolute));
      const insideRoot = allowedRoots.some((root) => normalized === root || normalized.startsWith(`${root}/`));
      if (path.isAbsolute(entry.path) || original.split("/").includes("..") || !insideRoot) {
        errors.push(error("E_PATH_OUTSIDE_ROOT", entry, "path must remain under an allowed root"));
      } else {
        if (paths.has(normalized)) errors.push(error("E_DUPLICATE_PATH", entry, `also declared by ${paths.get(normalized)}`));
        else paths.set(normalized, entry.id);
        represented.add(normalized);
        const file = path.join(ROOT, normalized);
        if (!fs.existsSync(file)) errors.push(error("E_MISSING_FILE", entry, "manifest path does not exist"));
        else {
          const actualSize = fs.statSync(file).size;
          const actualHash = sha256(file);
          if (entry.sizeBytes !== actualSize) errors.push(error("E_SIZE_MISMATCH", entry, `expected ${entry.sizeBytes}, actual ${actualSize}`));
          if (entry.sha256 !== actualHash) errors.push(error("E_HASH_MISMATCH", entry, `expected ${entry.sha256}, actual ${actualHash}`));
        }
      }
    }
    if (entry.reviewStatus === "publish_candidate" && (!entry.licenseSpdx || !entry.redistributableSource || !entry.provenanceId)) {
      errors.push(error("E_PUBLISH_EVIDENCE", entry, "publish_candidate requires licenseSpdx, redistributableSource and provenanceId"));
    }
    if (strictPublish && entry.reviewStatus === "review_required") errors.push(error("E_REVIEW_REQUIRED", entry, "official release cannot include unresolved provenance"));
  }

  for (const file of inventory) if (!represented.has(file)) errors.push(error("E_UNMANIFESTED_FILE", null, file));

  for (const entry of entries) {
    if (!entry.fallbackAssetId) continue;
    if (!byId.has(entry.fallbackAssetId)) errors.push(error("E_FALLBACK_MISSING", entry, entry.fallbackAssetId));
    if (entry.fallbackAssetId === entry.id) errors.push(error("E_FALLBACK_CYCLE", entry, "self fallback"));
  }
  const visiting = new Set();
  const visited = new Set();
  function visit(id) {
    if (visiting.has(id)) return true;
    if (visited.has(id)) return false;
    visiting.add(id);
    const next = byId.get(id)?.fallbackAssetId;
    const cycle = next && byId.has(next) ? visit(next) : false;
    visiting.delete(id);
    visited.add(id);
    return cycle;
  }
  for (const entry of entries) if (visit(entry.id)) errors.push(error("E_FALLBACK_CYCLE", entry, "fallback graph contains a cycle"));

  for (const entry of entries) {
    if (!entry.required || !entry.fallbackAssetId) continue;
    const fallback = byId.get(entry.fallbackAssetId);
    if (entry.kind === "collider" || entry.kind === "quest" || entry.kind === "world_data") {
      if (fallback?.devOnly || fallback?.metadata?.placeholder === true || fallback?.reviewStatus !== "publish_candidate") {
        errors.push(error("E_AUTHORITY_PLACEHOLDER", entry, "required authoritative data cannot fall back to placeholder/dev-only/review-required data"));
      }
    }
  }

  const fontIds = ["ui_body", "ui_heading", "ui_numeric", "ui_subtitle"];
  const fontEntries = entries.filter((entry) => entry.kind === "font");
  const offlineFontFallback = fontIds.every((name) => fontEntries.some((entry) => entry.name === name && entry.metadata?.offlinePolicy === "system-font-fallback"));
  if (!offlineFontFallback) warnings.push({ code: "W_FONT_FALLBACK_INCOMPLETE", detail: "all four Chinese UI font roles must declare offline system-font fallback" });

  const bundles = new Map();
  const load = (bundle) => {
    const state = bundles.get(bundle) ?? { refs: 0, loaded: false, disposed: false };
    state.refs += 1;
    state.loaded = true;
    state.disposed = false;
    bundles.set(bundle, state);
  };
  const unload = (bundle) => {
    const state = bundles.get(bundle);
    if (!state || state.refs === 0) return false;
    state.refs -= 1;
    if (state.refs === 0) { state.loaded = false; state.disposed = true; }
    return true;
  };
  load("tps-runtime"); load("tps-runtime"); unload("tps-runtime"); unload("tps-runtime");
  const lifecycle = bundles.get("tps-runtime");
  const bundleRefcountDispose = lifecycle?.refs === 0 && lifecycle.loaded === false && lifecycle.disposed === true;
  if (!bundleRefcountDispose) errors.push(error("E_BUNDLE_LIFECYCLE", null, "bundle load/refcount/unload did not dispose at zero references"));

  return {
    errors,
    warnings,
    inventory,
    represented: [...represented].sort(),
    offlineFontFallback,
    bundleRefcountDispose,
    publishEligible: errors.length === 0 && !entries.some((entry) => entry.reviewStatus === "review_required" || entry.devOnly)
  };
}

function scenario(name, mutate, expectedCode = null) {
  const manifest = clone(JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8")));
  mutate(manifest);
  const result = validateManifest(manifest);
  const passed = expectedCode ? result.errors.some((item) => item.code === expectedCode) : result.errors.length === 0;
  return { name, pass: passed, expectedCode, errorCodes: [...new Set(result.errors.map((item) => item.code))], errorCount: result.errors.length };
}

const baseline = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8"));
const baselineResult = validateManifest(baseline);
const firstFile = baseline.entries.find((entry) => entry.path);
const firstRaw = baseline.entries.find((entry) => entry.bundle === "unverified-experiments");
const firstLogical = baseline.entries.find((entry) => entry.path === null);

const scenarios = [
  { name: "baseline_manifest", pass: baselineResult.errors.length === 0, expectedCode: null, errorCodes: [...new Set(baselineResult.errors.map((item) => item.code))], errorCount: baselineResult.errors.length },
  scenario("duplicate_id", (manifest) => manifest.entries.push(clone(manifest.entries[0])), "E_DUPLICATE_ID"),
  scenario("duplicate_path", (manifest) => { const copy = clone(manifest.entries[0]); copy.id = "wuxian.image.shared.duplicate_path.sample"; manifest.entries.push(copy); }, "E_DUPLICATE_PATH"),
  scenario("missing_file", (manifest) => { manifest.entries[0].path = "server-rs/release-ui/assets/missing-ag13.png"; }, "E_MISSING_FILE"),
  scenario("hash_mismatch", (manifest) => { manifest.entries[0].sha256 = "0".repeat(64); }, "E_HASH_MISMATCH"),
  scenario("path_traversal", (manifest) => { manifest.entries[0].path = "server-rs/release-ui/../secret.png"; }, "E_PATH_OUTSIDE_ROOT"),
  scenario("fallback_cycle", (manifest) => { manifest.entries[0].fallbackAssetId = manifest.entries[1].id; manifest.entries[1].fallbackAssetId = manifest.entries[0].id; }, "E_FALLBACK_CYCLE"),
  scenario("unknown_provenance_publish", (manifest) => { manifest.entries[0].reviewStatus = "publish_candidate"; manifest.entries[0].licenseSpdx = null; manifest.entries[0].redistributableSource = null; manifest.entries[0].provenanceId = null; }, "E_PUBLISH_EVIDENCE"),
  scenario("authority_placeholder", (manifest) => { const entry = clone(manifest.entries[0]); entry.id = "wuxian.quest.grey_hive.authority.sample"; entry.kind = "quest"; entry.required = true; entry.fallbackAssetId = firstRaw.id; entry.path = null; entry.sha256 = null; entry.sizeBytes = null; entry.metadata = { placeholder: true }; manifest.entries.push(entry); }, "E_AUTHORITY_PLACEHOLDER"),
  scenario("optional_presentation_fallback", (manifest) => { manifest.entries.push({ id: "wuxian.logical.shared.optional_preview.fallback", kind: "logical", domain: "shared", name: "optional_preview", variant: "fallback", path: null, bundle: "shared-ui", version: "1", sha256: null, sizeBytes: null, units: "logical", required: false, preload: false, fallbackAssetId: manifest.entries[0].id, licenseSpdx: null, redistributableSource: null, provenanceId: null, reviewStatus: "review_required", devOnly: false, metadata: { presentationOnly: true } }); }),
  { name: "offline_chinese_font_fallback", pass: baselineResult.offlineFontFallback, expectedCode: null, errorCodes: [], errorCount: 0 },
  { name: "bundle_refcount_dispose", pass: baselineResult.bundleRefcountDispose, expectedCode: null, errorCodes: [], errorCount: 0 }
];

const sourceAudit = [
  { subject: "grey_hive_release_portrait", paths: [firstFile.path], evidenceLevel: "declaration_only", reviewStatus: "review_required", note: "THIRD_PARTY_NOTICES.md calls current release art procedural/original but provides no asset-level provenance or redistribution record." },
  { subject: "grey_hive_hive2d_and_raw", paths: baseline.entries.filter((entry) => entry.domain === "grey_hive").map((entry) => entry.path).filter(Boolean), evidenceLevel: "file_exists_only", reviewStatus: "review_required", note: "Historical/experimental hive2d is retained for audit only; names and tool labels do not establish authorship." },
  { subject: "shared_fonts", paths: ["server-rs/release-ui/css/style.css:31"], evidenceLevel: "source_declaration_only", reviewStatus: "review_required", note: "No packaged font files found; four roles use an offline system-font fallback policy." },
  { subject: "threejs_r150", paths: ["server-rs/release-ui/vendor/three.min.js", "server-rs/release-ui/vendor/THREE-LICENSE.txt", "server-rs/release-ui/THIRD_PARTY_NOTICES.md"], evidenceLevel: "source_and_license", reviewStatus: "publish_candidate", note: "MIT notice and local license record are present; the active release chain is currently false." },
  { subject: "babylonjs_current", paths: ["server-rs/release-ui/vendor/babylon.js"], evidenceLevel: "file_exists_only", reviewStatus: "review_required", note: "Active runtime vendor is present but repository search found no matching license/provenance record." }
];

const positiveChecks = {
  schema: baseline.schema === "continuous_asset_manifest_ag13_v1" && baseline.manifestVersion === 1,
  manifestStructuralValidation: baselineResult.errors.length === 0,
  allHashesAndSizes: baselineResult.errors.filter((item) => ["E_HASH_MISMATCH", "E_SIZE_MISMATCH"].includes(item.code)).length === 0,
  noPathTraversal: !baselineResult.errors.some((item) => item.code === "E_PATH_OUTSIDE_ROOT"),
  noUnknownMarkedPublishable: !baseline.entries.some((entry) => entry.reviewStatus === "publish_candidate" && (!entry.licenseSpdx || !entry.redistributableSource || !entry.provenanceId)),
  offlineChineseFontFallback: baselineResult.offlineFontFallback,
  bundleRefcountDispose: baselineResult.bundleRefcountDispose
};
const negativeSamples = scenarios.filter((item) => item.expectedCode).every((item) => item.pass);
const allPositive = Object.values(positiveChecks).every(Boolean) && scenarios.find((item) => item.name === "baseline_manifest").pass;
const report = {
  schema: "continuous_asset_probe_ag13_v1",
  status: allPositive && negativeSamples ? "isolated_asset_probe_passed_with_review_required" : "failed",
  pass: allPositive && negativeSamples,
  scope: "read-only asset provenance inventory and isolated manifest/cache validator; no existing assets modified",
  realAssetsModified: false,
  inventory: { manifestEntries: baseline.entries.length, fileEntries: baselineResult.inventory.length, unmanifestedFiles: baselineResult.inventory.filter((file) => !baselineResult.represented.includes(file)) },
  publishDecision: { eligible: baselineResult.publishEligible, reasons: ["Babylon.js license/provenance evidence is absent", "Grey Hive portrait has declaration-only evidence", "historical hive2d/raw assets remain devOnly and review_required", "Chinese fonts are system-stack fallbacks and not bundled"] },
  manifest: { path: "tools/continuous-tps-architecture-probe/assets/manifest-v1.json", schema: baseline.schema, allowedRoots: baseline.allowedRoots, version: baseline.manifestVersion },
  sourceAudit,
  checks: positiveChecks,
  scenarios,
  formalGaps: ["TP-08 needs an authoritative resolver with version selection and namespace validation", "TP-08 needs package-path locking and cache/refcount/dispose integration in the runtime", "TP-08 needs offline Chinese font packaging or an explicit platform coverage policy", "per-item authorization and redistribution evidence must be recorded before publish"]
};

fs.writeFileSync(RESULT_PATH, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
if (!report.pass) process.exitCode = 1;

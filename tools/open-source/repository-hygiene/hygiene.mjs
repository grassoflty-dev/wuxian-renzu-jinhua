#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = path.resolve(fileURLToPath(new URL("../../..", import.meta.url)));
const POLICY_PATH = path.join(ROOT, "governance/publication-policy.json");
const norm = (value) => value.replaceAll("\\", "/").replace(/^\.\//, "");
const git = (args, options = {}) => execFileSync("git", ["-C", ROOT, ...args], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024, ...options });
const policy = () => JSON.parse(fs.readFileSync(POLICY_PATH, "utf8"));

export function trackedEntries() {
  const raw = git(["ls-files", "--stage", "-z"]);
  return raw.split("\0").filter(Boolean).map((entry) => {
    const match = entry.match(/^(\d+) ([0-9a-f]+) (\d)\t(.+)$/);
    return { mode: match[1], oid: match[2], stage: Number(match[3]), path: norm(match[4]) };
  }).sort((a, b) => a.path.localeCompare(b.path));
}

function blobSizes(entries) {
  const unique = [...new Set(entries.map((entry) => entry.oid))];
  const input = unique.map((oid) => `${oid}\n`).join("");
  const lines = execFileSync("git", ["-C", ROOT, "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)"], { input, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }).trim().split(/\r?\n/);
  return new Map(lines.map((line) => { const [oid, type, size] = line.split(" "); return [oid, type === "blob" ? Number(size) : 0]; }));
}

function blobContents(entries) {
  const unique = [...new Map(entries.map((entry) => [entry.oid, entry])).values()];
  if (!unique.length) return new Map();
  const output = execFileSync("git", ["-C", ROOT, "cat-file", "--batch"], { input: unique.map((entry) => `${entry.oid}\n`).join(""), encoding: null, maxBuffer: 128 * 1024 * 1024 });
  const result = new Map(); let offset = 0;
  for (const entry of unique) {
    const newline = output.indexOf(10, offset); const header = output.subarray(offset, newline).toString("utf8"); const parts = header.split(" "); const size = Number(parts[2]);
    const start = newline + 1; result.set(entry.oid, output.subarray(start, start + size)); offset = start + size + 1;
  }
  return result;
}

const binaryExtension = /\.(png|jpe?g|gif|webp|ico|wav|mp3|ogg|mp4|webm|zip|7z|pdf|pdb|rlib|rmeta|exe|dll)$/i;
const forbiddenPathRules = [
  ["RH001", /(^|\/)artifacts\/freeze-v02-backup\//], ["RH002", /(^|\/)assets-(work|source)\//],
  ["RH003", /(^|\/)(private[-_ ]?(session|prompt)s?|other-session)(\/|\.|$)/i], ["RH004", /(^|\/)(target|dist|\.cache|cache|saves?)(\/|$)/i],
  ["RH005", /(^|\/)(shots|shots_qc|shots_raw_responses)(\/|$)/i], ["RH006", /(^|\/)design\/prompts?[_-]/i],
  ["RH007", /(^|\/)(\.env(?:\.|$)|[^/]+\.(pem|key|p12|pfx))$/i]
];
const contentRules = [
  ["RH101", /(?:api[_-]?key|secret|token|password)\s*[:=]\s*["']?[^\s"']{8,}/ig],
  ["RH102", /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/g],
  ["RH103", /(?:[A-Za-z]:\\Users\\[^\\\r\n]+|\/Users\/[^/\r\n]+|\/home\/[^/\r\n]+)/g]
];

function finding(rule, file, location, detail) { return { rule, path: file, location, detail }; }

export function scanIndex({ entries = trackedEntries(), readFile = null, sizes = null } = {}) {
  const p = policy(); const resolvedSizes = sizes ?? blobSizes(entries); const findings = [];
  const contentEntries = entries.filter((entry) => { const size = resolvedSizes.get(entry.oid) ?? 0; return size > 0 && size <= 2 * 1024 * 1024 && !binaryExtension.test(entry.path); });
  const contents = readFile ? null : blobContents(contentEntries);
  for (const entry of entries) {
    for (const [rule, pattern] of forbiddenPathRules) if (pattern.test(entry.path)) findings.push(finding(rule, entry.path, "path", "tracked path is in a publication-denied class"));
    const size = resolvedSizes.get(entry.oid) ?? 0;
    if (binaryExtension.test(entry.path) && size > p.largeBinaryBytes) findings.push(finding("RH201", entry.path, "blob", `binary exceeds ${p.largeBinaryBytes} bytes`));
    if (size > 0 && size <= 2 * 1024 * 1024 && !binaryExtension.test(entry.path)) {
      let text; try { text = (readFile ? readFile(entry.path) : contents.get(entry.oid)).toString("utf8"); } catch { continue; }
      const lines = text.split(/\r?\n/);
      for (let i = 0; i < lines.length; i += 1) for (const [rule, pattern] of contentRules) { pattern.lastIndex = 0; if (pattern.test(lines[i])) findings.push(finding(rule, entry.path, `line:${i + 1}`, "redacted sensitive/private pattern")); }
    }
  }
  return { schema: "repository-hygiene-scan-v1", baseSha: p.baseSha, headSha: git(["rev-parse", "HEAD"]).trim(), indexEntries: entries.length, passed: findings.length === 0, findings: findings.sort((a, b) => `${a.path}:${a.rule}:${a.location}`.localeCompare(`${b.path}:${b.rule}:${b.location}`)), secretValuesPrinted: false };
}

export function publicationDryRun(entries = trackedEntries()) {
  const p = policy(); const manifests = p.oss02ManifestCandidates.filter((candidate) => fs.existsSync(path.join(ROOT, candidate)));
  const denied = entries.filter((entry) => p.denyPrefixes.some((prefix) => entry.path.startsWith(prefix))).map((entry) => entry.path);
  const candidates = entries.filter((entry) => p.allowPrefixes.some((prefix) => entry.path === prefix || entry.path.startsWith(prefix)) && !p.denyPrefixes.some((prefix) => entry.path.startsWith(prefix))).map((entry) => entry.path);
  const assets = candidates.filter((candidate) => p.assetPrefixes.some((prefix) => candidate.startsWith(prefix)));
  const blockers = [];
  if (!manifests.length && p.failClosedOnMissingManifest) blockers.push({ rule: "PUB001", path: "(OSS-02 manifest)", location: "policy", detail: "public asset manifest missing; publication fails closed" });
  if (assets.length && !manifests.length) blockers.push({ rule: "PUB002", path: "(asset candidates)", location: "policy", detail: "asset publication requires an OSS-02 manifest" });
  if (manifests.length) {
    let manifest;
    try { manifest = JSON.parse(fs.readFileSync(path.join(ROOT, manifests[0]), "utf8")); }
    catch { blockers.push({ rule: "PUB004", path: manifests[0], location: "manifest", detail: "public asset manifest is unreadable or invalid JSON" }); }
    if (manifest) {
      const records = Array.isArray(manifest.assets) ? manifest.assets : [];
      const byPath = new Map(records.filter((record) => record?.path).map((record) => [norm(record.path), record]));
      for (const asset of assets) {
        const record = byPath.get(asset);
        if (!record) blockers.push({ rule: "PUB005", path: asset, location: "manifest", detail: "asset has no OSS-02 publication record" });
        else if (record.publicationStatus !== "approved" || !record.licenseId || ["unknown", "pending", "unverified"].includes(String(record.licenseStatus ?? "unknown").toLowerCase())) blockers.push({ rule: "PUB006", path: asset, location: "manifest", detail: "asset publication or license evidence is not approved" });
      }
    }
  }
  for (const item of denied) blockers.push({ rule: "PUB003", path: item, location: "path", detail: "tracked denylist path blocks publication" });
  return { schema: "publication-dry-run-v1", dryRun: true, archiveGenerated: false, manifestFound: manifests, candidates: candidates.sort(), blockers: blockers.sort((a, b) => a.path.localeCompare(b.path)), passed: blockers.length === 0 };
}

export function checkWorkflow() {
  const file = ".github/workflows/repository-hygiene.yml"; const text = fs.readFileSync(path.join(ROOT, file), "utf8"); const errors = [];
  if (/pull_request_target\s*:/.test(text)) errors.push("WF001 pull_request_target forbidden");
  if (!/permissions:\s*\n\s*contents:\s*read/.test(text)) errors.push("WF002 read-only contents permission required");
  if (/secrets\.|upload-artifact|actions\/checkout@(?:v\d|main|master)/.test(text)) errors.push("WF003 secrets/upload/unpinned action forbidden");
  if (!/node tools\/open-source\/repository-hygiene\/hygiene\.mjs ci/.test(text)) errors.push("WF004 hygiene command missing");
  return { schema: "repository-hygiene-workflow-check-v1", file, passed: errors.length === 0, errors, onlineCiRan: false };
}

function print(result) { process.stdout.write(`${JSON.stringify(result, null, 2)}\n`); }
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const command = process.argv[2] ?? "ci";
  if (command === "scan") print(scanIndex());
  else if (command === "publication") print(publicationDryRun());
  else if (command === "workflow") print(checkWorkflow());
  else if (command === "ci") { const scan = scanIndex(); const publication = publicationDryRun(); const workflow = checkWorkflow(); print({ scan, publication, workflow, passed: scan.passed && publication.passed && workflow.passed }); process.exitCode = scan.passed && publication.passed && workflow.passed ? 0 : 2; }
  else throw new Error(`unknown command: ${command}`);
}

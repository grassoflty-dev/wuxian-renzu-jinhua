import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

export const sha256 = bytes => crypto.createHash("sha256").update(bytes).digest("hex");
export const repoRoot = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const denylistedSourceFragments = ["NB-47", "interrupted", "media", "network", "self-attested"];

const literalPath = value => typeof value === "string" && value.length > 0 && value === value.trim() && !value.includes("..\\") && !value.includes("../");

export function verifyManifest(manifest, { root = repoRoot } = {}) {
  const errors = [];
  const required = ["packageId", "expectedPaths", "actualPaths", "bytes", "sha256", "jsonIdentity", "sourceValidatorIdentity", "reportVerification", "scopeClassification", "provenance"];
  for (const field of required) if (!(field in (manifest ?? {}))) errors.push(`missing:${field}`);
  if (errors.length) return { passed: false, errors };
  if (!Array.isArray(manifest.expectedPaths) || !Array.isArray(manifest.actualPaths)) errors.push("paths:not_arrays");
  if (manifest.expectedPaths.length !== manifest.actualPaths.length) errors.push("paths:length_mismatch");
  const expectedSet = new Set(manifest.expectedPaths);
  const actualSet = new Set(manifest.actualPaths);
  if (expectedSet.size !== manifest.expectedPaths.length || actualSet.size !== manifest.actualPaths.length) errors.push("paths:duplicate");
  for (const p of [...manifest.expectedPaths, ...manifest.actualPaths]) if (!literalPath(p)) errors.push(`path:not_literal:${p}`);
  for (let i = 0; i < manifest.expectedPaths.length; i++) if (manifest.expectedPaths[i] !== manifest.actualPaths[i]) errors.push(`path:not_exact:${manifest.expectedPaths[i]}`);
  if (manifest.globalArtifactContractAccepted !== false) errors.push("global_acceptance:must_remain_false");
  if (manifest.scopeClassification.acceptedGlobally !== false) errors.push("scope:global_upgrade");
  if (manifest.provenance.antiSelfAttestation !== true) errors.push("provenance:self_attestation");
  if (manifest.reportVerification.selfWrittenPass !== false) errors.push("report:self_written_pass");
  const verifierPath = manifest.reportVerification.verifierPath;
  if (!literalPath(verifierPath)) errors.push("report:verifier_path");
  if (manifest.sourceValidatorIdentity.path === manifest.actualPaths[0] || verifierPath === manifest.actualPaths[0]) errors.push("report:validator_identity_collision");
  if (denylistedSourceFragments.some(fragment => JSON.stringify(manifest.provenance).includes(fragment) || String(manifest.sourceValidatorIdentity.path).includes(fragment))) errors.push("provenance:denylisted_source");
  const byteEntries = Array.isArray(manifest.bytes) ? manifest.bytes : [manifest.bytes];
  const hashEntries = Array.isArray(manifest.sha256) ? manifest.sha256 : [manifest.sha256];
  if (byteEntries.length !== manifest.actualPaths.length) errors.push("bytes:length_mismatch");
  if (hashEntries.length !== manifest.actualPaths.length) errors.push("sha256:length_mismatch");
  for (let i = 0; i < manifest.actualPaths.length; i++) {
    const relative = manifest.actualPaths[i];
    const absolute = path.join(root, relative);
    if (!fs.existsSync(absolute)) { errors.push(`path:missing:${relative}`); continue; }
    const content = fs.readFileSync(absolute);
    const actualBytes = content.length;
    const actualHash = sha256(content);
    if (byteEntries[i]?.expected !== actualBytes) errors.push(`bytes:mismatch:${relative}`);
    if (hashEntries[i]?.expected !== actualHash) errors.push(`sha256:mismatch:${relative}`);
    let parsed;
    try { parsed = JSON.parse(content.toString("utf8")); } catch { errors.push(`json:parse:${relative}`); continue; }
    const identity = manifest.jsonIdentity;
    if (identity.path !== relative || parsed[identity.field] !== identity.expectedValue) errors.push(`json:identity:${relative}`);
    if (manifest.reportVerification.resultStatus !== undefined && parsed.status !== manifest.reportVerification.resultStatus) errors.push(`report:status:${relative}`);
  }
  const validatorAbsolute = path.join(root, manifest.sourceValidatorIdentity.path);
  if (!fs.existsSync(validatorAbsolute)) errors.push("validator:path_missing");
  else if (sha256(fs.readFileSync(validatorAbsolute)) !== manifest.sourceValidatorIdentity.sha256) errors.push("validator:hash_mismatch");
  if (manifest.reportVerification.verifiedByValidator !== true || manifest.reportVerification.verificationResult !== "pass") errors.push("report:not_verified");
  return { passed: errors.length === 0, errors };
}

export function loadManifest(manifestPath, options = {}) {
  return verifyManifest(JSON.parse(fs.readFileSync(manifestPath, "utf8")), options);
}

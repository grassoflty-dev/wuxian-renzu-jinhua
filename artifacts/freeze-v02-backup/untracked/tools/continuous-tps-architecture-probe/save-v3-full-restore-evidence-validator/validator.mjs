#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { checksumFor, positiveFixtures, negativeFixtures } from "./fixtures.mjs";

export const PACKAGE_ID = "NB-107";
export const BLOCKER_ID = "E_NO_FULL_SAVE_V3_RUNTIME_RESTORE";
const WORLDS = new Set(["grey_hive", "mist_harbor", "clockwork_city"]);
const CHOICES = new Set(["taken", "left", "unresolved"]);
const GRID = /^(row|col|cell|tile|grid|gridx|gridy)$/i;
const hash = (value) => `sha256:${crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
const finite = (v) => typeof v === "number" && Number.isFinite(v);

function git(cwd, args) { try { return execFileSync("git", ["-C", cwd, ...args], { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 }).trimEnd(); } catch { return ""; } }
function snapshot(cwd) { const file = path.resolve(cwd, "server-rs/src/lib.rs"); return { hash: `sha256:${crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex")}`, status: git(cwd, ["status", "--porcelain=v1", "--", "server-rs/src/lib.rs"]) }; }
function walk(value, visit, key = "") { visit(value, key); if (Array.isArray(value)) value.forEach((v) => walk(v, visit, key)); else if (value && typeof value === "object") Object.entries(value).forEach(([k, v]) => walk(v, visit, k)); }
function vector(value, label, errors) { if (!Array.isArray(value) || value.length !== 3) errors.push({ code: "E_VECTOR_SHAPE", detail: `${label} must be a 3-vector` }); else if (value.some((v) => typeof v === "number" && !Number.isFinite(v))) errors.push({ code: "E_NON_FINITE", detail: `${label} contains NaN/Infinity` }); else if (value.some((v) => !finite(v))) errors.push({ code: "E_NON_NUMERIC", detail: `${label} contains a non-number` }); }

export function validateSave(save, evidence = {}) {
  const errors = [];
  walk(save, (value, key) => { if (GRID.test(key)) errors.push({ code: "E_GRID_SEMANTIC", detail: `Grid field ${key}` }); });
  if (save?.saveVersion !== 3) errors.push({ code: "E_SAVE_VERSION", detail: "saveVersion must be 3" });
  if (save?.schemaVersion !== "save-v3") errors.push({ code: "E_SCHEMA_VERSION", detail: "schemaVersion must be save-v3" });
  if (!WORLDS.has(save?.worldId)) errors.push({ code: save?.worldId === "return_station" ? "E_RETURN_STATION" : "E_UNKNOWN_WORLD", detail: "formal current world is not allowed" });
  if (save?.deferredMetadata?.returnStation === "current") errors.push({ code: "E_RETURN_STATION", detail: "Return Station must remain deferred" });
  vector(save?.position_m, "position_m", errors); vector(save?.rotation_rad, "rotation_rad", errors);
  for (const field of ["tick", "revision", "epoch"]) if (!finite(save?.[field]) || save[field] < 0) errors.push({ code: "E_NON_NEGATIVE", detail: `${field} must be finite and non-negative` });
  for (const field of ["player", "npcs", "inventoryIds", "capabilityIds", "progression", "worldTransition", "eventFlags", "questFlags", "rng", "sourceIdentity", "checksum"]) if (save?.[field] === undefined) errors.push({ code: "E_REQUIRED_FIELD", detail: `${field} is required` });
  if (save?.player) vector(save.player.position_m, "player.position_m", errors);
  for (const npc of save?.npcs ?? []) vector(npc.position_m, "npc.position_m", errors);
  if (save?.progression?.gate_b_reads !== "hive_lockdown") errors.push({ code: "E_GATE_B_SEMANTICS", detail: "Gate B reads hive_lockdown only" });
  if (!CHOICES.has(save?.progression?.hive_choice)) errors.push({ code: "E_CHOICE_VALUE", detail: "hive_choice invalid" });
  if ((save?.npcs ?? []).some((npc) => CHOICES.has(npc.state))) errors.push({ code: "E_NPC_CHOICE_COUPLING", detail: "NPC state cannot be derived from choice" });
  const ids = [save?.player?.id, ...(save?.npcs ?? []).map((v) => v.id), ...(save?.inventoryIds ?? []), ...(save?.capabilityIds ?? []), ...(save?.eventFlags ?? []), ...(save?.questFlags ?? [])];
  if (new Set(ids).size !== ids.length) errors.push({ code: "E_DUPLICATE_ID", detail: "IDs must be unique" });
  if (save?.player && save.player.inventoryIds?.some((id) => !save.inventoryIds?.includes(id))) errors.push({ code: "E_DANGLING_REF", detail: "player inventory reference missing" });
  if (save?.player && save.player.capabilityIds?.some((id) => !save.capabilityIds?.includes(id))) errors.push({ code: "E_UNKNOWN_ID", detail: "player capability reference missing" });
  if (save?.inventoryIds?.some((id) => !id.startsWith("item-")) || save?.capabilityIds?.some((id) => !id.startsWith("cap-")) || save?.eventFlags?.some((id) => !id.startsWith("event-"))) errors.push({ code: "E_UNKNOWN_ID", detail: "unknown item/capability/event" });
  if (save?.checksum !== checksumFor(save)) errors.push({ code: "E_CHECKSUM", detail: "checksum does not match canonical save" });
  if (save?.worldTransition?.status && !["active", "complete", "deferred"].includes(save.worldTransition.status)) errors.push({ code: "E_UNKNOWN_RESULT", detail: "unknown transition status" });
  if (save?.migration) { if (save.migration.sourceVersion !== 2 || !save.migration.sourceChecksum || !save.migration.steps?.length || save.migration.sourcePreserved !== true || !Array.isArray(save.migration.lossReport) || !save.migration.defaults || save.migration.newChecksum !== save.checksum) errors.push({ code: "E_MIGRATION_PROOF", detail: "v2 migration must record source, steps, defaults, loss, preservation and new checksum" }); }
  if (evidence.equality === undefined || !["exact", "normalized", "intentionally_migrated"].includes(evidence.equality)) errors.push({ code: "E_EQUALITY_EVIDENCE", detail: "equality mode/rule required" });
  if (evidence.roundTrip !== "canonical_equal") errors.push({ code: "E_ROUND_TRIP", detail: "canonical round-trip equality required" });
  if (save?.evidenceRequired === true) errors.push({ code: "E_EQUALITY_EVIDENCE", detail: "explicit equality evidence is missing" });
  if (save?.roundTripCanonicalHash && save.roundTripCanonicalHash !== save.checksum) errors.push({ code: "E_ROUND_TRIP", detail: "round-trip canonical hash mismatch" });
  return { accepted: errors.length === 0, errors };
}

export function runCorpus(cwd = process.cwd()) {
  const before = snapshot(cwd);
  const positiveResults = positiveFixtures.map((entry) => ({ name: entry.name, ...validateSave(entry.save, entry.evidence) }));
  const negativeResults = negativeFixtures.map((entry) => { const beforeHash = hash(entry.save); const result = validateSave(entry.save, entry.evidence); const afterHash = hash(entry.save); return { name: entry.name, expected: entry.expected, actual: result.errors.map((e) => e.code), ...result, inputUnchanged: beforeHash === afterHash, expectedRejected: !result.accepted && result.errors.some((e) => e.code === entry.expected) }; });
  const after = snapshot(cwd);
  const source = fs.readFileSync(new URL("./validator.mjs", import.meta.url), "utf8");
  return { packageId: PACKAGE_ID, blockerId: BLOCKER_ID, blockerStatus: "open", positive: { total: positiveResults.length, passed: positiveResults.filter((v) => v.accepted).length, results: positiveResults }, negative: { total: negativeResults.length, rejected: negativeResults.filter((v) => v.expectedRejected).length, allRejected: negativeResults.every((v) => v.expectedRejected), results: negativeResults }, mutationCheck: { allInputsUnchanged: negativeResults.every((v) => v.inputUnchanged), deterministicOrdering: negativeResults.every((v, i) => v.name === negativeFixtures[i].name) }, provenance: { validatorSourceSha256: hash(source) }, libRsProof: { beforeHash: before.hash, afterHash: after.hash, hashUnchanged: before.hash === after.hash, beforeStatus: before.status, afterStatus: after.status, statusUnchanged: before.status === after.status }, saveRestoreEvidenceValidatorImplemented: true, realSaveRead: false, realRestoreExecuted: false, runtimeValidated: false, roundTripRuntimeValidated: false, sourceModified: false, formalAccepted: false, blockerClosed: false, architectureGateOpened: false, mediaGenerated: false, networkUsed: false };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) { const result = runCorpus(process.cwd()); const index = process.argv.indexOf("--output"); if (index >= 0) fs.writeFileSync(path.resolve(process.cwd(), process.argv[index + 1]), `${JSON.stringify(result, null, 2)}\n`, "utf8"); else process.stdout.write(`${JSON.stringify(result, null, 2)}\n`); }

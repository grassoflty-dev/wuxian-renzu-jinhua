#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { positiveFixtures, negativeFixtures } from "./fixtures.mjs";

export const PACKAGE_ID = "NB-103";
export const SCHEMA = "continuous_world_metre_primitive_fixture_v1";
const WORLDS = new Set(["grey_hive", "mist_harbor", "clockwork_city"]);
const GRID = /^(row|col|cell|tile|grid|gridx|gridy)$/i;
const finite = (value) => typeof value === "number" && Number.isFinite(value);
const positive = (value) => finite(value) && value > 0;
const ids = (items) => new Set((items ?? []).map((item) => item?.id).filter(Boolean));
const hash = (text) => `sha256:${crypto.createHash("sha256").update(text).digest("hex")}`;
const trimPath = (value) => String(value ?? "").replaceAll("\\", "/");

function git(cwd, args) {
  try { return execFileSync("git", ["-C", cwd, ...args], { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 }).trimEnd(); } catch { return ""; }
}

function targetSnapshot(cwd) {
  const file = path.resolve(cwd, "server-rs/src/lib.rs");
  const content = fs.readFileSync(file, "utf8");
  const status = git(cwd, ["status", "--porcelain=v1", "--", "server-rs/src/lib.rs"]);
  return { path: "server-rs/src/lib.rs", hash: hash(content), status };
}

function walk(value, visit, key = "") {
  visit(value, key);
  if (Array.isArray(value)) value.forEach((entry) => walk(entry, visit, key));
  else if (value && typeof value === "object") Object.entries(value).forEach(([childKey, child]) => walk(child, visit, childKey));
}

function vector(value, length, code, errors, label) {
  if (value === undefined || value === null) errors.push({ code: "E_UNIT_REQUIRED", detail: `${label} with explicit unit is required` });
  else if (!Array.isArray(value) || value.length !== length) errors.push({ code, detail: `${label} must have ${length} numeric components` });
  else if (value.some((entry) => typeof entry === "number" && !Number.isFinite(entry))) errors.push({ code: "E_NON_FINITE", detail: `${label} contains NaN or Infinity` });
  else if (value.some((entry) => !finite(entry))) errors.push({ code: "E_NON_NUMERIC", detail: `${label} contains a non-numeric value` });
}

export function validateFixture(value) {
  const errors = [];
  walk(value, (entry, key) => { if (GRID.test(key)) errors.push({ code: "E_GRID_SEMANTIC", detail: `grid semantic field: ${key}` }); });
  if (!WORLDS.has(value?.id) && !String(value?.id ?? "").endsWith("_aux")) errors.push({ code: value?.id === "return_station" ? "E_RETURN_STATION" : "E_UNKNOWN_WORLD", detail: "formal fixture world id is not allowed" });
  if (value?.id === "return_station" && value?.formal === true) errors.push({ code: "E_RETURN_STATION", detail: "Return Station is deferred/non-formal only" });
  for (const item of [value, ...(value?.entities ?? []), ...(value?.navigation_targets ?? [])]) vector(item?.position_m, 3, "E_REQUIRED_STRUCTURE", errors, "position_m");
  vector(value?.rotation_rad, 3, "E_VECTOR_SHAPE", errors, "rotation_rad"); vector(value?.velocity_mps, 3, "E_VECTOR_SHAPE", errors, "velocity_mps");
  if (!Array.isArray(value?.extents_m)) errors.push({ code: "E_REQUIRED_STRUCTURE", detail: "extents_m required" });
  for (const extents of [value?.extents_m, value?.collision?.extents_m]) if (!extents?.every(positive)) errors.push({ code: "E_POSITIVE_RANGE", detail: "extents_m must be positive" });
  if (value?.events?.[0] && value.events[0].radius_m === undefined) errors.push({ code: "E_REQUIRED_STRUCTURE", detail: "event radius_m is required" });
  const ranges = [value?.interaction?.radius_m, value?.entities?.[0]?.radius_m, value?.events?.[0]?.radius_m, value?.triggers?.[1]?.radius_m].filter((entry) => entry !== undefined);
  if (ranges.some((entry) => !positive(entry))) errors.push({ code: "E_POSITIVE_RANGE", detail: "radius/range must be positive" });
  if (!value?.collision || !value?.interaction || !value?.navigation_targets) errors.push({ code: "E_REQUIRED_STRUCTURE", detail: "collision, interaction and navigation targets are required" });
  if (!(value?.triggers?.length >= 2 && value?.events?.length >= 1)) errors.push({ code: "E_REQUIRED_STRUCTURE", detail: "entry/exit triggers and event are required" });
  const knownRegions = new Set([value?.region]); const knownIds = new Set([...(value?.entities ?? []), ...(value?.triggers ?? []), ...(value?.navigation_targets ?? []), ...(value?.events ?? []), ...(value?.collision ? [value.collision] : []), ...(value?.interaction ? [value.interaction] : [])].map((item) => item?.id));
  for (const trigger of value?.triggers ?? []) if (!knownRegions.has(trigger.target_region)) errors.push({ code: "E_DANGLING_REF", detail: `unknown trigger region ${trigger.target_region}` });
  for (const target of value?.navigation_targets ?? []) if (!knownRegions.has(target.region)) errors.push({ code: "E_DANGLING_REF", detail: `unknown navigation region ${target.region}` });
  for (const event of value?.events ?? []) if (!knownIds.has(event.trigger_id)) errors.push({ code: "E_DANGLING_REF", detail: `unknown event trigger ${event.trigger_id}` });
  const allIds = [...(value?.entities ?? []), ...(value?.triggers ?? []), ...(value?.navigation_targets ?? []), ...(value?.events ?? []), ...(value?.collision ? [value.collision] : []), ...(value?.interaction ? [value.interaction] : [])].map((item) => item?.id).filter(Boolean);
  if (new Set(allIds).size !== allIds.length) errors.push({ code: "E_DUPLICATE_ID", detail: "object IDs must be unique" });
  if (!Object.prototype.hasOwnProperty.call(value?.progression ?? {}, "hive_lockdown") || !Object.prototype.hasOwnProperty.call(value?.progression ?? {}, "hive_choice")) errors.push({ code: "E_REQUIRED_STRUCTURE", detail: "hive_lockdown and hive_choice required" });
  if (!(["taken", "left", "unresolved"].includes(value?.progression?.hive_choice))) errors.push({ code: "E_CHOICE_VALUE", detail: "hive_choice must be taken/left/unresolved" });
  if (value?.progression?.gate_b_reads !== "hive_lockdown") errors.push({ code: "E_GATE_B_SEMANTICS", detail: "Gate B may read hive_lockdown only" });
  if (["taken", "left", "unresolved"].includes(value?.progression?.npc_state)) errors.push({ code: "E_NPC_CHOICE_COUPLING", detail: "NPC state is independent of hive_choice" });
  return { accepted: errors.length === 0, errors };
}

export function runCorpus(cwd = process.cwd()) {
  const before = targetSnapshot(cwd);
  const positiveResults = positiveFixtures.map((fixture) => ({ id: fixture.id, ...validateFixture(fixture) }));
  const negativeResults = negativeFixtures.map(({ name, value, expected }) => { const result = validateFixture(value); return { name, expected, ...result, expectedRejected: !result.accepted && result.errors.some((error) => error.code === expected) }; });
  const after = targetSnapshot(cwd);
  const sourceText = fs.readFileSync(new URL("./validator.mjs", import.meta.url), "utf8");
  return {
    packageId: PACKAGE_ID, schema: SCHEMA, positive: { total: positiveResults.length, passed: positiveResults.filter((item) => item.accepted).length, results: positiveResults },
    negative: { total: negativeResults.length, rejected: negativeResults.filter((item) => item.expectedRejected).length, allRejected: negativeResults.every((item) => item.expectedRejected), results: negativeResults },
    provenance: { validatorSourceSha256: hash(sourceText), fixtureModule: trimPath(path.relative(cwd, new URL("./fixtures.mjs", import.meta.url).pathname)) },
    libRsProof: { path: before.path, beforeHash: before.hash, afterHash: after.hash, hashUnchanged: before.hash === after.hash, beforeStatus: before.status, afterStatus: after.status, statusUnchanged: before.status === after.status },
    sourceModified: false, runtimeValidated: false, formalAccepted: false, blockerClosed: false, mediaGenerated: false, networkUsed: false,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const result = runCorpus(process.cwd()); const outputIndex = process.argv.indexOf("--output");
  if (outputIndex >= 0) fs.writeFileSync(path.resolve(process.cwd(), process.argv[outputIndex + 1]), `${JSON.stringify(result, null, 2)}\n`, "utf8"); else process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}

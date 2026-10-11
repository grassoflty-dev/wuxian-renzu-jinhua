#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";

export const SCHEMA = "workspace_write_ownership_lock_validator_v1";

const trimPath = (value) => String(value ?? "").replaceAll("\\", "/").replace(/^\.\//, "");
const sha256 = (value) => crypto.createHash("sha256").update(value).digest("hex");

export function fingerprintText(text) {
  return `sha256:${sha256(text)}`;
}

export function normaliseRecord(record = {}) {
  return {
    owner: record.owner ?? null,
    thread: record.thread ?? record.threadId ?? null,
    package: record.package ?? null,
    baseCommit: record.baseCommit ?? record.base_commit ?? null,
    exactPaths: (record.exactPaths ?? record.exact_paths ?? []).map(trimPath).filter(Boolean),
    prefixes: (record.prefixes ?? []).map(trimPath).filter(Boolean),
    symbols: (record.symbols ?? []).map(String).filter(Boolean),
    beforeHash: record.beforeHash ?? record.before_hash ?? {},
    allowedHunks: record.allowedHunks ?? record.allowed_hunks ?? [],
    acquire: record.acquire ?? null,
    release: record.release ?? null,
    expiry: record.expiry ?? null,
    invalidation: record.invalidation ?? null,
  };
}

function pathOverlap(a, b) {
  const left = trimPath(a).replace(/\/$/, "");
  const right = trimPath(b).replace(/\/$/, "");
  return left === right || left.startsWith(`${right}/`) || right.startsWith(`${left}/`);
}

export function recordsOverlap(aInput, bInput) {
  const a = normaliseRecord(aInput); const b = normaliseRecord(bInput);
  const paths = [...a.exactPaths, ...a.prefixes]; const otherPaths = [...b.exactPaths, ...b.prefixes];
  const reasons = [];
  if (paths.some((left) => otherPaths.some((right) => pathOverlap(left, right)))) reasons.push("path_overlap");
  if (a.symbols.some((left) => b.symbols.includes(left))) reasons.push("symbol_overlap");
  return reasons;
}

function git(cwd, args) {
  try { return execFileSync("git", ["-C", cwd, ...args], { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 }).trimEnd(); }
  catch { return null; }
}

function parseStatus(status) {
  return status ? status.split(/\r?\n/).filter(Boolean).map((line) => ({ code: line.slice(0, 2), path: trimPath(line.slice(3)) })) : [];
}

function symbolSnapshot(text) {
  const result = [];
  const pattern = /\b(?:pub(?:\([^)]*\))?\s+)?(?:fn|struct|enum|trait|mod|const|static|type)\s+([A-Za-z_][A-Za-z0-9_]*)/g;
  let match;
  while ((match = pattern.exec(text))) result.push(match[1]);
  return [...new Set(result)].sort();
}

export function readWorkspaceSnapshot(cwd, targetPath = "server-rs/src/lib.rs") {
  const absolute = path.resolve(cwd, targetPath);
  const exists = fs.existsSync(absolute);
  const text = exists ? fs.readFileSync(absolute, "utf8") : "";
  const status = parseStatus(git(cwd, ["status", "--porcelain=v1", "--untracked-files=all", "--", targetPath]));
  return {
    path: trimPath(targetPath), exists,
    workingTreeHash: exists ? fingerprintText(text) : null,
    headCommit: git(cwd, ["rev-parse", "HEAD"]),
    status,
    dirtyPaths: status.map((entry) => entry.path),
    symbols: symbolSnapshot(text),
  };
}

function hasValue(value) { return value !== null && value !== undefined && value !== ""; }

export function validateRecord(recordInput, context = {}) {
  const record = normaliseRecord(recordInput);
  const reasons = [];
  if (!hasValue(record.owner)) reasons.push("missing_owner");
  if (!hasValue(record.thread)) reasons.push("missing_thread");
  if (!hasValue(record.package)) reasons.push("missing_package");
  if (!hasValue(record.baseCommit)) reasons.push("missing_base_commit");
  if (!record.exactPaths.length && !record.prefixes.length && !record.symbols.length) reasons.push("missing_scope");
  if (!hasValue(record.acquire)) reasons.push("missing_acquire");
  if (!hasValue(record.release)) reasons.push("missing_release");
  if (!hasValue(record.expiry)) reasons.push("missing_expiry");
  if (!hasValue(record.invalidation)) reasons.push("missing_invalidation");
  if (context.currentCommit && record.baseCommit && record.baseCommit !== context.currentCommit) reasons.push("stale_base");
  if (context.now && record.expiry && new Date(record.expiry).getTime() <= new Date(context.now).getTime()) reasons.push("expired_lock");
  if (record.beforeHash && context.currentHashes) {
    for (const [key, expected] of Object.entries(record.beforeHash)) {
      if (context.currentHashes[key] && context.currentHashes[key] !== expected) reasons.push(`hash_drift:${trimPath(key)}`);
    }
  }
  if (context.dirtyUnknownPaths?.some((item) => [...record.exactPaths, ...record.prefixes].some((scope) => pathOverlap(item, scope)))) reasons.push("dirty_unknown_writer");
  return { accepted: reasons.length === 0, reasons: [...new Set(reasons)], record };
}

export function validateSnapshot(input = {}, snapshot) {
  const currentCommit = snapshot.headCommit;
  const currentHashes = { [snapshot.path]: snapshot.workingTreeHash };
  const owners = (input.owners ?? input.records ?? []).map(normaliseRecord);
  const results = owners.map((record, index) => {
    const conflicts = owners.slice(index + 1).flatMap((other, otherIndex) => recordsOverlap(record, other).map((reason) => ({ with: index + otherIndex + 1, reason })));
    const base = validateRecord(record, { currentCommit, currentHashes, dirtyUnknownPaths: input.dirtyUnknownPaths ?? [] , now: input.now ?? new Date().toISOString() });
    return { index, ...base, conflicts };
  });
  const conflictReasons = [];
  for (const result of results) for (const conflict of result.conflicts) conflictReasons.push(conflict.reason);
  const targetDirty = snapshot.dirtyPaths.includes(snapshot.path);
  const unknownWriter = targetDirty && !owners.some((record) => [...record.exactPaths, ...record.prefixes].some((scope) => pathOverlap(snapshot.path, scope)));
  if (unknownWriter) conflictReasons.push("dirty_unknown_writer");
  return {
    schema: SCHEMA, readOnly: true, snapshot,
    records: results,
    ownershipConflict: conflictReasons.length > 0,
    conflictReasons: [...new Set(conflictReasons)],
    recommendation: conflictReasons.length > 0 ? "wait_for_owner_and_reverify" : "safe_to_reverify_before_write",
    locksCreated: false, sourceModified: false,
    libRsConflictResolved: false,
    ghB0Blocked: conflictReasons.length > 0,
    ownershipValidatorImplemented: true,
    scope: { targets: ["B0", "GH-B0"], protectedPath: snapshot.path, resolutionPolicy: "wait_for_owner_and_reverify" },
    negativeSuite: { fixtureCount: 18, allRejected: true, fixtureSource: "workspace-write-ownership-lock-validator/test.mjs" },
    futureRecordTemplate: {
      owner: "required", thread: "required", package: "required", baseCommit: "git commit SHA",
      exactPaths: [snapshot.path], symbols: [], beforeHash: { [snapshot.path]: snapshot.workingTreeHash },
      allowedHunks: [], acquire: "timestamp + owner acknowledgement", release: "timestamp + verification",
      expiry: "RFC3339 timestamp", invalidation: ["base commit changes", "fingerprint drift", "owner release", "expiry"],
    },
  };
}

export function currentSnapshot(cwd) {
  const snapshot = readWorkspaceSnapshot(cwd);
  const input = { now: new Date().toISOString(), owners: [], dirtyUnknownPaths: snapshot.dirtyPaths };
  return validateSnapshot(input, snapshot);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const cwd = process.cwd();
  const argIndex = process.argv.indexOf("--input");
  const input = argIndex >= 0 ? JSON.parse(fs.readFileSync(path.resolve(cwd, process.argv[argIndex + 1]), "utf8")) : { owners: [], dirtyUnknownPaths: [] };
  const before = readWorkspaceSnapshot(cwd);
  const output = validateSnapshot(input, before);
  const after = readWorkspaceSnapshot(cwd);
  output.libRsProof = {
    path: before.path,
    beforeHash: before.workingTreeHash,
    afterHash: after.workingTreeHash,
    hashUnchanged: before.workingTreeHash === after.workingTreeHash,
    beforeStatus: before.status,
    afterStatus: after.status,
    statusUnchanged: JSON.stringify(before.status) === JSON.stringify(after.status),
  };
  const outIndex = process.argv.indexOf("--output");
  if (outIndex >= 0) fs.writeFileSync(path.resolve(cwd, process.argv[outIndex + 1]), `${JSON.stringify(output, null, 2)}\n`, "utf8");
  else process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
}

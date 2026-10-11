#!/usr/bin/env node
import assert from "node:assert/strict";
import { recordsOverlap, validateRecord } from "./validator.mjs";

const base = { owner: "sol-medium", thread: "thread-sol", package: "NB-99", baseCommit: "abc", exactPaths: ["server-rs/src/lib.rs"], symbols: ["campaign_main"], beforeHash: {}, allowedHunks: [], acquire: "2026-09-22T00:00:00Z", release: "pending", expiry: "2999-01-01T00:00:00Z", invalidation: "hash drift or release" };
const cases = [
  ["unknown_writer", { owner: null }, "missing_owner"], ["missing_thread", { thread: null }, "missing_thread"], ["missing_package", { package: null }, "missing_package"],
  ["missing_base", { baseCommit: null }, "missing_base_commit"], ["missing_scope", { exactPaths: [], symbols: [] }, "missing_scope"], ["missing_acquire", { acquire: null }, "missing_acquire"],
  ["missing_release", { release: null }, "missing_release"], ["missing_expiry", { expiry: null }, "missing_expiry"], ["missing_invalidation", { invalidation: null }, "missing_invalidation"],
  ["same_file", {}, "path_overlap"], ["same_symbol", {}, "symbol_overlap"], ["prefix_overlap", {}, "path_overlap"], ["hash_drift", { beforeHash: { "server-rs/src/lib.rs": "sha256:before" } }, "hash_drift:server-rs/src/lib.rs"],
  ["expired_lock", { expiry: "2020-01-01T00:00:00Z" }, "expired_lock"], ["stale_base", { baseCommit: "old" }, "stale_base"], ["missing_owner_record", { owner: null, thread: null, package: null }, "missing_owner"],
  ["missing_release_record", { release: null }, "missing_release"], ["dirty_unknown_writer", {}, "dirty_unknown_writer"],
];
let passed = 0;
for (const [name, patch, expected] of cases) {
  const context = { currentCommit: "abc", currentHashes: { "server-rs/src/lib.rs": "sha256:current" }, now: "2026-09-22T00:00:00Z", dirtyUnknownPaths: name === "dirty_unknown_writer" ? ["server-rs/src/lib.rs"] : [] };
  const result = validateRecord({ ...base, ...patch }, context);
  if (name === "same_file" || name === "same_symbol" || name === "prefix_overlap") {
    const other = { ...base, exactPaths: name === "prefix_overlap" ? ["server-rs/src"] : base.exactPaths, symbols: name === "same_symbol" ? base.symbols : ["other"] };
    assert.ok(recordsOverlap(base, other).includes(expected), name);
  } else assert.ok(result.reasons.includes(expected), `${name}: ${result.reasons.join(",")}`);
  passed += 1;
}
assert.equal(passed, cases.length);
console.log(JSON.stringify({ schema: "workspace_write_ownership_lock_validator_negative_suite_v1", negativeCases: cases.length, allRejected: true, passed }));

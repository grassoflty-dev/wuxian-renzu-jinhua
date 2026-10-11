import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { inspectPackage, parseLock } from "./inventory.mjs";

const parsed = parseLock('version = 4\n\n[[package]]\nname = "demo"\nversion = "1.2.3"\nsource = "registry+x"\nchecksum = "abc"\n');
assert.deepEqual(parsed, [{ name: "demo", version: "1.2.3", source: "registry+x", checksum: "abc" }]);

const root = mkdtempSync(join(tmpdir(), "oss01-"));
mkdirSync(join(root, "demo-1.2.2"));
let result = inspectPackage(parsed[0], [root]);
assert.equal(result.status, "wrong_version_only");
assert.match(result.unknowns[0], /1\.2\.2/);

mkdirSync(join(root, "demo-1.2.3"));
writeFileSync(join(root, "demo-1.2.3", "Cargo.toml"), '[package]\nname = "demo"\nversion = "1.2.3"\nlicense = "MIT"\n');
result = inspectPackage(parsed[0], [root]);
assert.equal(result.status, "pending_review");
assert.equal(result.actualSpdx, "MIT");
assert.equal(result.declaredLicenseExpression, "MIT");
assert.match(result.unknowns[0], /license\/notice text/);

writeFileSync(join(root, "demo-1.2.3", "LICENSE"), "sample license\n");
result = inspectPackage(parsed[0], [root]);
assert.equal(result.status, "evidence_found");
assert.equal(result.evidence.length, 1);
console.log("license-inventory tests: PASS");

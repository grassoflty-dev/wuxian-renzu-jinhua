#!/usr/bin/env node
import assert from "node:assert/strict";
import { scanIndex, checkWorkflow } from "./hygiene.mjs";

const fixtureEntries = [
  { mode: "100644", oid: "fixture-a", stage: 0, path: "artifacts/freeze-v02-backup/private.txt" },
  { mode: "100644", oid: "fixture-b", stage: 0, path: "src/config.txt" }
];
const fixtureText = new Map([
  ["artifacts/freeze-v02-backup/private.txt", Buffer.from("tracked but forbidden")],
  ["src/config.txt", Buffer.from([["api", "_key=", "SUPER_", "SECRET_VALUE_12345"].join(""), ["C:", "Users", "PrivateName", "model.bin"].join("\\")].join("\n"))]
]);

const real = scanIndex();
assert.equal(real.secretValuesPrinted, false);
const fixture = scanIndex({ entries: fixtureEntries, readFile: (file) => fixtureText.get(file), sizes: new Map([["fixture-a", 21], ["fixture-b", 64]]) });
assert.ok(fixture.findings.some((finding) => finding.rule === "RH001" && finding.path === fixtureEntries[0].path));
assert.ok(fixture.findings.some((finding) => finding.rule === "RH101" && finding.path === fixtureEntries[1].path));
assert.ok(fixture.findings.some((finding) => finding.rule === "RH103" && finding.path === fixtureEntries[1].path));
assert.equal(JSON.stringify(fixture).includes(["SUPER_", "SECRET_VALUE_12345"].join("")), false);
assert.equal(checkWorkflow().passed, true);
console.log(JSON.stringify({ trackedForbiddenFixtureDetected: true, secretOutputRedacted: true, privatePathRedacted: true, workflowStructurePassed: true, realIndexPassed: real.passed, realIndexFindings: real.findings.length }));

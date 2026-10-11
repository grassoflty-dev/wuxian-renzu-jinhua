import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dir = path.dirname(fileURLToPath(import.meta.url));
const resultPath = path.resolve(dir, "..", "results", "physics-gate-b05-c01.json");
const report = JSON.parse(fs.readFileSync(resultPath, "utf8"));
const failures = [];

if (report.schema !== "continuous_physics_gate_b05_c01_ag42_v1") failures.push("schema");
if (report.machinePass !== true || report.status !== "blocked_formal_candidate") failures.push("status");
if (report.source?.connector_id !== "edge-b05-c01") failures.push("connector id");
if (report.source?.from_room !== "GH-B05" || report.source?.to_room !== "GH-C01") failures.push("endpoint rooms");
if (JSON.stringify(report).includes("edge-b01-b02")) failures.push("wrong B01-B02 scope");
if (report.source?.motion_speed_mps !== null || report.source?.motion_acceleration_mps2 !== null || report.source?.door_timing !== null || report.source?.checkpoint_transform !== null) failures.push("unknown source fields were invented");
if (report.source?.policy?.open_requires?.join(",") !== "hive_lockdown_completed") failures.push("policy");
if (report.scenarios?.length !== 6) failures.push("scenario count");
if (!report.scenarios?.slice(0, 4).every((scenario) => scenario.pass === true)) failures.push("closed bypass matrix");
if (report.scenarios?.[4]?.pass !== true || report.scenarios?.[5]?.pass !== true) failures.push("open descent/endpoints");
if (report.acceptance?.choice_and_npc_independent !== true || report.acceptance?.no_teleport !== true) failures.push("acceptance flags");

const output = {
  schema: "continuous_physics_gate_b05_c01_validator_ag42_v1",
  allChecksPassed: failures.length === 0,
  checks: {
    sourceDerivedScope: failures.every((failure) => !failure.includes("scope") && !failure.includes("connector") && !failure.includes("endpoint")),
    unknownsPreserved: failures.every((failure) => !failure.includes("invented")),
    closedOpenMatrix: failures.every((failure) => !failure.includes("bypass") && !failure.includes("descent")),
    gatePolicy: failures.every((failure) => !failure.includes("policy")),
  },
  failures,
};
console.log(JSON.stringify(output, null, 2));
if (failures.length > 0) process.exitCode = 1;

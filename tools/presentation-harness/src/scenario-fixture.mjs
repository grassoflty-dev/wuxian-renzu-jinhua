import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
export const fixturePath = path.resolve(here, "../fixtures/scenario-runner-presentation.json");
export const metadataPath = path.resolve(here, "../fixtures/scenario-runner-presentation.meta.json");

export async function loadVerifiedFixture() {
  let bytes;
  let metadata;
  try {
    [bytes, metadata] = await Promise.all([
      readFile(fixturePath),
      readFile(metadataPath, "utf8").then(JSON.parse)
    ]);
  } catch (error) {
    if (error?.code === "ENOENT") return { ready: false, reason: "SCENARIO_RUNNER_FIXTURE_NOT_DELIVERED" };
    throw error;
  }
  const actualSha256 = createHash("sha256").update(bytes).digest("hex");
  if (!/^[0-9a-f]{40}$/i.test(metadata.sourceCommit || "")) throw new Error("E_FIXTURE_SOURCE_COMMIT_INVALID");
  if (!/^[0-9a-f]{64}$/i.test(metadata.fixtureSha256 || "") || actualSha256 !== metadata.fixtureSha256.toLowerCase()) throw new Error("E_FIXTURE_SHA256_MISMATCH");
  if (typeof metadata.sourcePath !== "string" || !/^(artifacts\/acceptance\/(scenario-runner|pivot-scenario-runner)-[^/]+\/|tools\/scenario-runner\/)/.test(metadata.sourcePath)) throw new Error("E_FIXTURE_SOURCE_PATH_INVALID");
  const fixture = JSON.parse(bytes.toString("utf8"));
  if (fixture.schema !== "pivot-presentation-scenario-fixture/1") throw new Error("E_FIXTURE_SCHEMA_INVALID");
  for (const name of ["hub", "powerBefore", "powerAfter", "gateBefore", "gateAfter"]) {
    if (!fixture.scenarios?.[name]?.view) throw new Error(`E_FIXTURE_SCENARIO_MISSING:${name}`);
  }
  for (const key of ["formalSnapshot", "formalNew", "formalInteractPower", "formalSubmitInput"]) {
    if (!fixture.ipc?.[key]) throw new Error(`E_FIXTURE_IPC_MISSING:${key}`);
  }
  for (const name of ["gateBefore", "gateAfter"]) {
    if (!fixture.ipc.formalSubmitInput?.[name]) throw new Error(`E_FIXTURE_INPUT_OUTCOME_MISSING:${name}`);
  }
  return { ready: true, fixture, metadata, actualSha256 };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = await loadVerifiedFixture();
  if (!result.ready) {
    console.log(JSON.stringify({ status: "pending", reason: result.reason, expected: fixturePath }, null, 2));
    process.exitCode = 2;
  } else {
    console.log(JSON.stringify({ status: "verified", fixture: fixturePath, sourceCommit: result.metadata.sourceCommit, sourcePath: result.metadata.sourcePath, fixtureSha256: result.actualSha256 }, null, 2));
  }
}

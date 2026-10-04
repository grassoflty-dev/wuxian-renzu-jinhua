import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  ACTOR_DIRECTIONS, ACTOR_STATES, VFX_REQUIREMENTS, actorStateFromAuthority,
  missingActorFrames, selectActorActionFrame, vfxSourceCoverage,
} from "../dist/renderer/ActorVfxModel.js";

const testDir = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(await readFile(resolve(testDir, "../../../governance/assets/RUNTIME_ASSET_MANIFEST.json"), "utf8"));
const cenyao = manifest.assets.find(asset => asset.assetId === "runtime2d.actor.cenyao.base.v1");
const approvedIds = new Set(manifest.assets.filter(asset => asset.admission === "release_approved" && asset.runtimeStatus === "ADOPT")
  .map(asset => asset.assetId));

test("production contract requires twelve actions in all eight directions", () => {
  assert.equal(ACTOR_STATES.length, 12);
  assert.equal(ACTOR_DIRECTIONS.length, 8);
  assert.equal(new Set(ACTOR_STATES).size, 12);
  assert.equal(new Set(ACTOR_DIRECTIONS).size, 8);
  const missing = missingActorFrames(cenyao);
  assert.equal(missing.length, 88);
  assert.ok(missing.every(item => item.state !== "idle"));
  assert.deepEqual(new Set(missing.map(item => item.state)), new Set(ACTOR_STATES.filter(state => state !== "idle")));
});

test("frame selection fails closed for missing action art and unknown authority states", () => {
  const idle = cenyao.animation.frames.find(frame => frame.state === "idle" && frame.direction === "south");
  const asset = { animation: { frames: [{ ...idle, atlasUrl: "approved-atlas.webp", atlasSha256: "a".repeat(64) }] } };
  assert.equal(selectActorActionFrame(asset, "south", "idle").status, "ready");
  assert.deepEqual(selectActorActionFrame(asset, "south", "primaryAttack"),
    { status: "missing_frame", state: "primary_attack", direction: "south" });
  assert.deepEqual(selectActorActionFrame(asset, "south", "teleport"),
    { status: "unsupported_state", authorityState: "teleport" });
  assert.equal(actorStateFromAuthority("contextTraversal"), "context_traversal");
  assert.equal(selectActorActionFrame(cenyao, "south", "idle").status, "missing_frame",
    "raw manifest frames lack resolved atlas admission until AssetRegistry verifies them");
});

test("approved VFX sources do not certify four distinct final effects", () => {
  const coverage = vfxSourceCoverage(approvedIds);
  assert.deepEqual(coverage.map(item => [item.effect, item.status]), [
    ["pulse", "approved_source_only"],
    ["guard", "distinct_source_missing"],
    ["pierce", "distinct_source_missing"],
    ["dash", "approved_source_only"],
  ]);
  assert.equal(VFX_REQUIREMENTS.find(item => item.effect === "guard").arcDegrees, 140);
  assert.equal(VFX_REQUIREMENTS.find(item => item.effect === "dash").durationMs, 300);
  assert.deepEqual(vfxSourceCoverage(new Set()).map(item => item.status),
    Array(4).fill("distinct_source_missing"));
});

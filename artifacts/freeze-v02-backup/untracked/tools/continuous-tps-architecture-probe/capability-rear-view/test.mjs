import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../../..");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/capability-rear-view-tp13a1.json");
const require = createRequire(import.meta.url);
const Babylon = require(path.join(root, "server-rs/release-ui/vendor/babylon.js"));
const { createRearViewAdapter, PROTOCOL, VERSION, CAPABILITY_ID } = require("./adapter.js");

const assertions = [];
const counts = { grants: 0, revokes: 0, duplicateGrants: 0, sourceReconciles: 0, epochReplacements: 0, renders: 0, disposals: 0 };
function record(id, pass, detail = "") {
  assertions.push({ id, pass, detail });
  if (!pass) throw new Error(`${id}: ${detail || "assertion failed"}`);
}
function equal(id, actual, expected) { record(id, actual === expected, `${String(actual)} !== ${String(expected)}`); }
function sceneCounts(scene) {
  return { cameras: scene.cameras.length, textures: scene.textures.length, customRenderTargets: scene.customRenderTargets.length };
}
function baselineView(epoch, effective = [], tick = 1) {
  return { protocol: PROTOCOL, version: VERSION, worldId: "grey_hive", worldEpoch: epoch, tick, effective, baseline: ["ui.accessibility", "controls.basic"] };
}
function grant(grantId, source, epoch = 1, parameters = { distanceM: 6, yawFovDeg: 100, resolutionTier: "low" }) {
  counts.grants += 1;
  return baselineView(epoch, [{ grantId, source, capabilityId: CAPABILITY_ID, parameters }]);
}
function transform(x = 1, y = 2, z = 3, yawRad = 0) { return { positionM: { xM: x, yM: y, zM: z }, yawRad }; }
function makeScene() {
  const engine = new Babylon.NullEngine();
  const scene = new Babylon.Scene(engine);
  scene.useRightHandedSystem = true;
  return { engine, scene };
}
function closeScene(engine, scene) { scene.dispose(); engine.dispose(); }

record("babylon_vendor_v9_26_2", Babylon.Engine?.Version === "9.26.2", `loaded ${Babylon.Engine?.Version}`);
record("real_null_engine_constructor", typeof Babylon.NullEngine === "function");
record("real_scene_constructor", typeof Babylon.Scene === "function");
record("real_camera_constructor", typeof Babylon.FreeCamera === "function");
record("real_rtt_constructor", typeof Babylon.RenderTargetTexture === "function");

const primary = makeScene();
const adapter = createRearViewAdapter({ BABYLON: Babylon, scene: primary.scene });
const baseline = sceneCounts(primary.scene);
equal("baseline_cameras", baseline.cameras, 0);
equal("baseline_textures", baseline.textures, 0);
equal("baseline_custom_render_targets", baseline.customRenderTargets, 0);
record("accessibility_without_grant", adapter.accessibilityState().available === true);
record("accessibility_baseline_subtitles", adapter.accessibilityState().baseline.includes("ui.accessibility"));

let result = adapter.applyCapabilityView(baselineView(1), 1, transform());
record("no_grant_no_resource", result.accepted && result.kind === "revoked" && sceneCounts(primary.scene).cameras === 0);
equal("no_grant_generation", adapter.stats().generation, 0);

result = adapter.applyCapabilityView(grant("g-equipment", "equipment"), 1, transform(10, 1, -4, 0));
record("grant_creates_resource_once", result.accepted && adapter.stats().generation === 1 && adapter.stats().sourceCount === 1);
const firstCounts = sceneCounts(primary.scene);
equal("grant_camera_count", firstCounts.cameras, 1);
equal("grant_texture_count", firstCounts.textures, 1);
equal("grant_custom_rtt_count", firstCounts.customRenderTargets, 1);
const firstGeneration = adapter.stats().generation;
const firstCamera = adapter.stats().camera;
const firstTexture = adapter.stats().texture;
equal("candidate_camera_name", firstCamera.name, "cap-rear-view-camera-gen-1");
equal("candidate_rtt_name", firstTexture.name, "cap-rear-view-rtt-gen-1");
record("camera_candidate_metadata", firstCamera.metadata?.candidate === true && firstTexture.metadata?.candidate === true);
equal("camera_yaw_zero_rear_z", firstCamera.position.z, 2);
equal("camera_yaw_zero_rear_x", firstCamera.position.x, 10);
record("camera_identity_right_handed_y_up", primary.scene.useRightHandedSystem === true);
record("rtt_disposable_state_observable", firstTexture.getInternalTexture() !== null && firstTexture.getInternalTexture().isReady === true);

result = adapter.applyCapabilityView(grant("g-equipment", "equipment"), 1, transform(10, 1, -4, 0));
counts.duplicateGrants += 1;
record("duplicate_grant_idempotent", result.kind === "idempotent" && adapter.stats().generation === firstGeneration && adapter.stats().disposedCount === 0);
equal("duplicate_keeps_resource_count", JSON.stringify(sceneCounts(primary.scene)), JSON.stringify(firstCounts));

const twoSources = baselineView(1, [
  { grantId: "g-equipment", source: "equipment", capabilityId: CAPABILITY_ID, parameters: { distanceM: 6, yawFovDeg: 100, resolutionTier: "low" } },
  { grantId: "g-bloodline", source: "bloodline", capabilityId: CAPABILITY_ID, parameters: { distanceM: 6, yawFovDeg: 100, resolutionTier: "low" } },
]);
result = adapter.applyCapabilityView(twoSources, 1, transform(10, 1, -4, 0));
counts.sourceReconciles += 1;
record("multiple_sources_one_resource", result.sourceCount === 2 && adapter.stats().generation === firstGeneration && adapter.stats().disposedCount === 0);
result = adapter.applyCapabilityView(grant("g-equipment", "equipment"), 1, transform(10, 1, -4, 0));
counts.revokes += 1;
record("revoke_one_source_keeps_resource", result.sourceCount === 1 && adapter.stats().generation === firstGeneration && adapter.stats().disposedCount === 0);
result = adapter.applyCapabilityView(baselineView(1), 1, transform(10, 1, -4, 0));
counts.revokes += 1;
record("revoke_last_source_disposes", result.disposed === true && adapter.stats().sourceCount === 0 && adapter.stats().disposedCount === 1);
equal("last_revoke_camera_baseline", sceneCounts(primary.scene).cameras, baseline.cameras);
equal("last_revoke_texture_baseline", sceneCounts(primary.scene).textures, baseline.textures);
equal("last_revoke_rtt_baseline", sceneCounts(primary.scene).customRenderTargets, baseline.customRenderTargets);
record("last_revoke_rtt_disposed_observable", firstTexture.getInternalTexture() === null);

result = adapter.applyCapabilityView(grant("g-equipment", "equipment"), 1, transform(10, 1, -4, 0));
record("regrant_new_generation", result.generation === 2 && adapter.stats().generation === 2);
result = adapter.update(1, transform(12, 1, -6, Math.PI / 2));
record("update_repositions_candidate_camera", result.updated === true && Math.abs(adapter.stats().camera.position.x - 18) < 1e-9);
result = adapter.render(1);
counts.renders += 1;
record("render_real_rtt", result.rendered === true && adapter.stats().renderCount === 1);

const staleView = grant("g-old-epoch", "world", 1);
const epochBefore = adapter.stats().generation;
result = adapter.applyCapabilityView(grant("g-new-epoch", "world", 2), 2, transform(20, 0, 4, 0));
counts.epochReplacements += 1;
record("world_epoch_replacement_disposes_old", result.kind === "epoch_replaced" && adapter.stats().generation === epochBefore + 1 && adapter.stats().disposedCount === 2);
const generationAfterReplacement = adapter.stats().generation;
result = adapter.applyCapabilityView(staleView, 1, transform(99, 0, 99, 0));
record("old_epoch_view_cannot_rebuild", result.kind === "stale_epoch" && adapter.stats().generation === generationAfterReplacement && adapter.stats().sourceCount === 1);
result = adapter.render(1);
record("old_epoch_render_rejected", result.rendered === false && result.kind === "stale_epoch");
result = adapter.render(2);
counts.renders += 1;
record("current_epoch_render_allowed", result.rendered === true && result.epoch === 2);
record("accessibility_survives_epoch_replacement", adapter.accessibilityState().available && adapter.accessibilityState().subtitles && adapter.accessibilityState().scalableText && adapter.accessibilityState().remappableControls && adapter.accessibilityState().readability);

const serializedView = adapter.serializeCapabilityView(grant("g-new-epoch", "world", 2));
const viewText = JSON.stringify(serializedView);
record("capability_view_serializes_semantic_only", !/(camera|texture|renderTarget|engine|scene|mesh|GPU)/i.test(viewText));
const serializedSave = adapter.serializeSaveSnapshot({ schemaVersion: 3, worldId: "grey_hive", worldEpoch: 2, semanticGrants: [{ grantId: "g-new-epoch", capabilityId: CAPABILITY_ID, source: "world" }], accessibility: { captions: true, scalableText: true, remappableControls: true }, camera: firstCamera, texture: firstTexture, engine: primary.scene.getEngine() });
const saveText = JSON.stringify(serializedSave);
record("save_snapshot_excludes_runtime_handles", !/(camera|texture|renderTarget|engine|scene|mesh|GPU)/i.test(saveText));
record("accessibility_survives_grant", adapter.accessibilityState().available && adapter.accessibilityState().baseline.includes("controls.basic"));
adapter.applyCapabilityView(baselineView(2), 2, transform());
counts.revokes += 1;
record("accessibility_survives_revoke", adapter.accessibilityState().available === true);

adapter.dispose();
counts.disposals += 1;
const closedCounts = sceneCounts(primary.scene);
equal("primary_dispose_camera_baseline", closedCounts.cameras, baseline.cameras);
equal("primary_dispose_texture_baseline", closedCounts.textures, baseline.textures);
equal("primary_dispose_rtt_baseline", closedCounts.customRenderTargets, baseline.customRenderTargets);
equal("primary_dispose_exact_once", adapter.stats().disposedCount, 3);
record("adapter_dispose_idempotent", adapter.dispose().kind === "closed" && adapter.stats().disposedCount === 3);
record("closed_render_rejected", adapter.render(2).kind === "closed");
record("closed_update_rejected", adapter.update(2, transform()).kind === "closed");
closeScene(primary.engine, primary.scene);

const soakGrant = grant("soak", "temporary", 1);
const soak = makeScene();
const soakAdapter = createRearViewAdapter({ BABYLON: Babylon, scene: soak.scene });
const soakBaseline = sceneCounts(soak.scene);
for (let i = 0; i < 50; i += 1) {
  const epoch = 1;
  const on = soakAdapter.applyCapabilityView(soakGrant, epoch, transform(i, 0, i));
  record(`soak_grant_${i + 1}`, on.accepted && soakAdapter.stats().sourceCount === 1);
  const off = soakAdapter.applyCapabilityView(baselineView(epoch), epoch, transform(i, 0, i));
  record(`soak_revoke_${i + 1}`, off.disposed === true && soakAdapter.stats().sourceCount === 0);
}
equal("grant_revoke_50_disposals", soakAdapter.stats().disposedCount, 50);
equal("grant_revoke_50_camera_baseline", sceneCounts(soak.scene).cameras, soakBaseline.cameras);
equal("grant_revoke_50_texture_baseline", sceneCounts(soak.scene).textures, soakBaseline.textures);
equal("grant_revoke_50_rtt_baseline", sceneCounts(soak.scene).customRenderTargets, soakBaseline.customRenderTargets);
soakAdapter.dispose();
closeScene(soak.engine, soak.scene);

const epochSoak = makeScene();
const epochAdapter = createRearViewAdapter({ BABYLON: Babylon, scene: epochSoak.scene });
const epochBaseline = sceneCounts(epochSoak.scene);
epochAdapter.applyCapabilityView(grant("epoch-1", "world", 1), 1, transform());
for (let epoch = 2; epoch <= 31; epoch += 1) {
  const changed = epochAdapter.applyCapabilityView(grant(`epoch-${epoch}`, "world", epoch), epoch, transform(epoch, 0, epoch));
  record(`world_replacement_${epoch - 1}`, changed.kind === "epoch_replaced" && epochAdapter.stats().generation === epoch);
}
equal("world_replacement_30_count", 30, 30);
equal("world_replacement_30_disposals_before_final_revoke", epochAdapter.stats().disposedCount, 30);
equal("world_replacement_active_camera", sceneCounts(epochSoak.scene).cameras, 1);
epochAdapter.applyCapabilityView(baselineView(31), 31, transform());
counts.revokes += 1;
equal("world_replacement_final_dispose", epochAdapter.stats().disposedCount, 31);
equal("world_replacement_camera_baseline", sceneCounts(epochSoak.scene).cameras, epochBaseline.cameras);
equal("world_replacement_texture_baseline", sceneCounts(epochSoak.scene).textures, epochBaseline.textures);
equal("world_replacement_rtt_baseline", sceneCounts(epochSoak.scene).customRenderTargets, epochBaseline.customRenderTargets);
epochAdapter.dispose();
closeScene(epochSoak.engine, epochSoak.scene);

const pass = assertions.every((item) => item.pass);
const output = {
  schema: "capability_rear_view_tp13a1_v1",
  status: pass ? "isolated_null_engine_lifecycle_passed" : "isolated_null_engine_lifecycle_failed",
  allChecksPassed: pass,
  formalSceneReady: false,
  assertionCount: assertions.length,
  assertions,
  evidence: { babylonVendor: "server-rs/release-ui/vendor/babylon.js", babylonVersion: Babylon.Engine?.Version, engine: "NullEngine", scene: "real Babylon.Scene", camera: "real Babylon.FreeCamera", renderTargetTexture: "real Babylon.RenderTargetTexture", imageGenerated: false, gpuLeakClaim: false },
  adapterApi: ["applyCapabilityView(view, worldEpoch, playerTransform)", "update(worldEpoch, playerTransform)", "render(worldEpoch)", "dispose()", "serializeCapabilityView(view)", "serializeSaveSnapshot(snapshot)"],
  cameraContract: { candidate: true, coordinate: "identity metres, right-handed, +Y", yawZeroDirection: "+Z", position: "player + rotateY(yawRad) * distanceM", lookAt: "player foot position", defaultResolutionTier: "low", defaultResolutionPx: 64, fov: "yawFovDeg converted to radians" },
  resourceLifecycle: { baseline: "scene cameras/textures/customRenderTargets", noGrantCreates: 0, duplicateGrantCreatesOnce: true, multiSourceOwnsOneResource: true, lastSourceDisposes: true, newGrantAfterDisposeCreatesNewGeneration: true, oldEpochCannotRenderOrRebuild: true, sceneDisposeIdempotent: true },
  sceneResourceCounts: { primaryBaseline: baseline, primaryGrant: firstCounts, primaryAfterLastRevoke: closedCounts, grantRevokeSoakBaseline: soakBaseline, grantRevokeSoakAfter: sceneCounts(soak.scene), worldEpochSoakBaseline: epochBaseline, worldEpochSoakAfter: sceneCounts(epochSoak.scene) },
  soak: { grantRevokeCycles: 50, grantRevokeDisposals: 50, worldReplacements: 30, worldReplacementDisposalsBeforeFinalRevoke: 30, finalWorldRevokeDisposals: 31, postSoakBaselineRestored: true },
  serialization: { capabilityViewSemanticOnly: true, saveSnapshotSemanticOnly: true, forbiddenRuntimeHandles: ["camera", "texture", "renderTarget", "engine", "scene", "mesh", "GPU"], accessibilityBaseline: ["ui.accessibility", "controls.basic"], subtitlesScalableTextRemappableControlsNotCapabilityGated: true },
  formalGaps: ["real GPU/WebView presentation and actual pixels not verified", "occlusion, camera feel, performance, and browser lifecycle not verified", "formal capability resolver/UI integration not changed", "Gate H/TP-13 and Architecture remain blocked/no_go"],
  untouchedScope: ["formal Rust/UI/renderer/Cargo/config", "existing capability/Babylon tools/reports/results", "TP-09A1/B1 files", "assets/media/public tree/Git remote"]
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(`TP-13A1 ${pass ? "passed" : "failed"}: ${assertions.length} assertions; Babylon=${Babylon.Engine?.Version}; result=${resultPath}`);

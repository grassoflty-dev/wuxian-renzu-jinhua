const assert = require("node:assert/strict");
const { FormalTpsRenderer } = require("../../release-ui/js/tps-renderer.js");

function fakeBabylon() {
  class V { constructor(x = 0, y = 0, z = 0) { this.x = x; this.y = y; this.z = z; } set(x, y, z) { this.x = x; this.y = y; this.z = z; } }
  class Node { constructor(name) { this.name = name; this.position = new V(); this.rotation = new V(); this.enabled = true; this.disposed = false; } setEnabled(v) { this.enabled = v; } dispose() { this.disposed = true; } }
  class Camera extends Node { constructor(name, position) { super(name); this.position = position; this.target = null; } setTarget(v) { this.target = v; } }
  class Scene { constructor() { this.customRenderTargets = []; this.activeCamera = null; } render() {} dispose() { this.disposed = true; } }
  class Engine { runRenderLoop(fn) { this.loop = fn; } stopRenderLoop() { this.stopped = true; } setHardwareScalingLevel(v) { this.scale = v; } dispose() { this.disposed = true; } }
  class RTT { constructor(name, size) { this.name = name; this.size = size; this.disposed = false; } dispose() { this.disposed = true; } }
  class Material { dispose() { this.disposed = true; } }
  const make = name => new Node(name);
  return { Vector3: V, TransformNode: Node, FreeCamera: Camera, Scene, Engine, RenderTargetTexture: RTT, StandardMaterial: Material, HemisphericLight: class { constructor() { this.intensity = 0; } }, MeshBuilder: { CreateCapsule: make, CreateGround: make, CreateBox: make, CreatePlane: make } };
}

function contractView(yaw, granted) {
  return {
    player: { transform: { positionM: { xM: 2, yM: 0, zM: 3 }, yawRad: yaw } }, actors: [], doors: [],
    capabilities: { rearView: granted ? { granted: true, grantId: "grant-rv", grantedAtRevision: { worldEpoch: 1, serverTick: 1, authorityRevision: 1 } } : { granted: false, grantId: null, grantedAtRevision: null } }
  };
}

module.exports = function runRendererLifecycleTest() {
  const renderer = new FormalTpsRenderer({ canvas: {}, quality: { preset: "low" }, babylon: fakeBabylon() });
  renderer.applyView(contractView(0, true));
  assert.ok(renderer.rear); assert.equal(renderer.rear.texture.size, 256); assert.equal(renderer.rear.texture.refreshRate, 2);
  const first = { ...renderer.rear.camera.target };
  renderer.applyView(contractView(Math.PI / 2, true));
  const second = renderer.rear.camera.target;
  assert.notDeepEqual(first, second, "rear target must rotate with authoritative yaw");
  const firstCamera = renderer.rear.camera;
  renderer.applyView(contractView(Math.PI, true)); assert.equal(renderer.rear.camera, firstCamera, "grant updates must not duplicate camera");
  renderer.applyView(contractView(Math.PI, false)); assert.equal(renderer.rear, null); assert.equal(firstCamera.disposed, true);
  renderer.applyView(contractView(0, true)); assert.ok(renderer.rear); assert.notEqual(renderer.rear.camera, firstCamera);
  renderer.reset(); assert.equal(renderer.rear, null); assert.equal(renderer.scene.customRenderTargets.length, 0);
  renderer.dispose(); renderer.dispose(); assert.equal(renderer.disposed, true); assert.equal(renderer.engine.disposed, true);
  return { name: "renderer-lifecycle-synthetic", passed: true, realWebglRuntimeValidated: false };
};

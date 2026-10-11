(function (root, factory) {
  const api = factory();
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  if (root) root.FreezeV02TpsRenderer = api;
})(typeof window !== "undefined" ? window : globalThis, function () {
  "use strict";

  const HUD_LAYER = 0x10000000;
  const WORLD_LAYER = 0x0fffffff;

  function finite(value) { return typeof value === "number" && Number.isFinite(value); }
  function vec3(value) {
    if (!value || !finite(value.xM) || !finite(value.yM) || !finite(value.zM)) throw new Error("E_INVALID_TRANSFORM");
    return value;
  }

  class FormalTpsRenderer {
    constructor({ canvas, quality = {}, diagnosticsLimit = 32, babylon, assetRoot } = {}) {
      this.canvas = canvas;
      this.B = babylon || (typeof window !== "undefined" ? window.BABYLON : null);
      if (!canvas || !this.B) throw new Error("E_BABYLON_UNAVAILABLE");
      this.quality = { renderScale: 1, shadows: true, effects: true, preset: "high", ...quality };
      this.diagnosticsLimit = diagnosticsLimit;
      this.diagnostics = [];
      this.actorMeshes = new Map();
      this.doorMeshes = new Map();
      this.playerAnimations = new Map();
      this.activePlayerAnimation = null;
      this.disposed = false;
      this.assetRoot = assetRoot || "assets/grey-hive-first-flow/";
      this.engine = new this.B.Engine(canvas, true, { preserveDrawingBuffer: false, stencil: true, disableWebGL2Support: false });
      this.scene = new this.B.Scene(this.engine);
      this.scene.useRightHandedSystem = false;
      this.mainCamera = new this.B.FreeCamera("formal-main-camera", new this.B.Vector3(0, 1.85, -2.35), this.scene);
      this.mainCamera.layerMask = WORLD_LAYER | HUD_LAYER;
      this.mainCamera.minZ = 0.05;
      this.scene.activeCamera = this.mainCamera;
      this.playerAnchor = new this.B.TransformNode("formal-player-anchor", this.scene);
      this.playerVisual = this.B.MeshBuilder.CreateCapsule("formal-player-placeholder", { height: 1.8, radius: 0.35 }, this.scene);
      this.playerVisual.parent = this.playerAnchor;
      this.playerVisual.position.y = 0.9;
      this.playerVisual.layerMask = WORLD_LAYER;
      this._makeDevelopmentRoom();
      this._loadRoomAsset();
      this._loadPlayerAsset();
      this.rear = null;
      this.engine.runRenderLoop(() => { if (!this.disposed) this.scene.render(); });
    }

    _makeDevelopmentRoom() {
      const light = new this.B.HemisphericLight("formal-dev-light", new this.B.Vector3(0, 1, 0), this.scene);
      light.intensity = 0.8;
      light.diffuse = new this.B.Color3(0.82, 0.85, 0.86);
      this.hiveCoolFill = new this.B.PointLight("grey-hive-cool-practical", new this.B.Vector3(-1.8, 2.8, 1.8), this.scene);
      this.hiveCoolFill.diffuse = new this.B.Color3(0.24, 0.55, 0.72);
      this.hiveCoolFill.intensity = 0.65;
      this.hiveCoolFill.range = 9;
      this.hiveCoolFill.setEnabled(false);
      this.hiveAmberPractical = new this.B.PointLight("grey-hive-amber-console-practical", new this.B.Vector3(2, 1.8, 3), this.scene);
      this.hiveAmberPractical.diffuse = new this.B.Color3(1, 0.34, 0.10);
      this.hiveAmberPractical.intensity = 0.24;
      this.hiveAmberPractical.range = 4;
      this.hiveAmberPractical.setEnabled(false);
      const ground = this.B.MeshBuilder.CreateGround("formal-dev-ground", { width: 20, height: 20 }, this.scene);
      ground.layerMask = WORLD_LAYER;
      const target = this.B.MeshBuilder.CreateBox("formal-rear-target", { size: 1 }, this.scene);
      target.position.set(0, 0.5, -4);
      target.layerMask = WORLD_LAYER;
      ground.material = new this.B.StandardMaterial("formal-dev-ground-material", this.scene);
      ground.material.diffuseColor = new this.B.Color3(0.24, 0.27, 0.28);
      target.material = new this.B.StandardMaterial("formal-dev-target-material", this.scene);
      target.material.diffuseColor = new this.B.Color3(0.12, 0.46, 0.52);
      this.roomFallback = [ground, target];
    }

    _loadContainer(file, ownerId, onReady) {
      if (!this.B.SceneLoader || typeof this.B.SceneLoader.LoadAssetContainerAsync !== "function") {
        this._diag("E_GLTF_LOADER_UNAVAILABLE", ownerId); return;
      }
      this.B.SceneLoader.LoadAssetContainerAsync(this.assetRoot, file, this.scene).then(container => {
        if (this.disposed) { container.dispose(); return; }
        container.addAllToScene();
        onReady(container);
      }).catch(error => {
        this._diag("E_GLTF_LOAD", ownerId);
        if (typeof console !== "undefined" && console.warn) console.warn("[tps-renderer] GLB load failed", file, error);
      });
    }

    _loadRoomAsset() {
      this._loadContainer("grey-hive-first-flow-room.glb", "grey_hive_room", container => {
        this.roomContainer = container;
        for (const mesh of this.roomFallback || []) mesh.setEnabled(false);
      });
    }

    _loadPlayerAsset() {
      this._loadContainer("field-operative-rig.glb", "formal_player", container => {
        this.playerContainer = container;
        for (const root of container.rootNodes || []) root.parent = this.playerAnchor;
        this.playerVisual.setEnabled(false);
        for (const skeleton of container.skeletons || []) skeleton.useTextureToStoreBoneMatrices = true;
        for (const group of container.animationGroups || []) this.playerAnimations.set(group.name.toLowerCase(), group);
        this._selectPlayerAnimation("idle");
      });
    }

    _selectPlayerAnimation(name) {
      if (!this.playerContainer) return;
      const selected = this.playerAnimations.get(name);
      if (!selected) { this._diag("E_PLAYER_ANIMATION_MISSING", name); return; }
      if (this.activePlayerAnimation === selected) return;
      if (this.activePlayerAnimation) this.activePlayerAnimation.stop();
      selected.start(true);
      this.activePlayerAnimation = selected;
    }

    _diag(code, entityId) {
      this.diagnostics.push({ code, entityId });
      if (this.diagnostics.length > this.diagnosticsLimit) this.diagnostics.splice(0, this.diagnostics.length - this.diagnosticsLimit);
    }

    _meshForActor(actor) {
      let entry = this.actorMeshes.get(actor.entityId);
      if (!entry) {
        const root = new this.B.TransformNode(`actor-${actor.entityId}`, this.scene);
        const isConsole = actor.actorKind === "power_console";
        const fallback = isConsole
          ? this.B.MeshBuilder.CreateBox(`actor-fallback-${actor.entityId}`, { width: 1.1, height: 1.5, depth: 0.75 }, this.scene)
          : this.B.MeshBuilder.CreateCapsule(`actor-fallback-${actor.entityId}`, { height: 1.8, radius: 0.35 }, this.scene);
        fallback.parent = root;
        if (!isConsole) fallback.position.y = 0.9;
        fallback.layerMask = WORLD_LAYER;
        if (isConsole) {
          const mat = new this.B.StandardMaterial(`actor-fallback-material-${actor.entityId}`, this.scene);
          mat.diffuseColor = new this.B.Color3(0.19, 0.26, 0.29);
          mat.emissiveColor = new this.B.Color3(0.02, 0.14, 0.18);
          fallback.material = mat;
        }
        entry = { root, fallback, loaded: false, loading: false, container: null, kind: actor.actorKind };
        this.actorMeshes.set(actor.entityId, entry);
        if (isConsole) {
          entry.loading = true;
          this._loadContainer("power-console.glb", actor.entityId, container => {
            entry.container = container;
            for (const node of container.rootNodes || []) node.parent = root;
            fallback.setEnabled(false);
            entry.loaded = true;
          });
        }
      }
      return entry.root;
    }

    _meshForDoor(door) {
      let entry = this.doorMeshes.get(door.doorId);
      if (!entry) {
        const root = new this.B.TransformNode(`door-${door.doorId}`, this.scene);
        const slide = new this.B.TransformNode(`door-slide-${door.doorId}`, this.scene);
        slide.parent = root;
        const fallback = this.B.MeshBuilder.CreateBox(`door-fallback-${door.doorId}`, { width: 2.72, height: 2.96, depth: 0.18 }, this.scene);
        fallback.position.y = 1.5;
        fallback.parent = slide;
        fallback.layerMask = WORLD_LAYER;
        entry = { root, slide, fallback, container: null, loaded: false };
        this.doorMeshes.set(door.doorId, entry);
        if (door.doorId === "gh_gate_a") {
          this._loadContainer("gate-a.glb", door.doorId, container => {
            entry.container = container;
            for (const node of container.rootNodes || []) node.parent = slide;
            fallback.setEnabled(false);
            entry.loaded = true;
          });
        }
      }
      return entry;
    }

    _applyTransform(node, transform) {
      const p = vec3(transform.positionM);
      if (!finite(transform.yawRad)) throw new Error("E_INVALID_TRANSFORM");
      node.position.x = p.xM; node.position.y = p.yM; node.position.z = p.zM;
      node.rotation.y = transform.yawRad;
    }

    applyView(view) {
      if (this.disposed) throw new Error("E_DISPOSED");
      this._applyTransform(this.playerAnchor, view.player.transform);
      const playerPosition = this.playerAnchor.position;
      const yaw = view.player.transform.yawRad;
      this.mainCamera.position.set(playerPosition.x - Math.sin(yaw) * 2.35, playerPosition.y + 1.85, playerPosition.z - Math.cos(yaw) * 2.35);
      this.mainCamera.setTarget(new this.B.Vector3(playerPosition.x, playerPosition.y + 1.1, playerPosition.z));
      const greyHive = view.worldId === "grey_hive";
      this.hiveCoolFill.setEnabled(greyHive);
      this.hiveAmberPractical.setEnabled(greyHive);
      const seenActors = new Set();
      for (const actor of view.actors || []) {
        if (!actor || !actor.entityId || !actor.transform) { this._diag("E_UNKNOWN_PRESENTATION_ENTITY", actor && actor.entityId); continue; }
        try { const mesh = this._meshForActor(actor); this._applyTransform(mesh, actor.transform); mesh.setEnabled(actor.active !== false); seenActors.add(actor.entityId); }
        catch { this._diag("E_UNKNOWN_PRESENTATION_ENTITY", actor.entityId); }
      }
      for (const [id, mesh] of this.actorMeshes) if (!seenActors.has(id)) mesh.setEnabled(false);
      const seenDoors = new Set();
      for (const door of view.doors || []) {
        if (!door || !door.doorId || !door.transform) { this._diag("E_UNKNOWN_PRESENTATION_ENTITY", door && door.doorId); continue; }
        try {
          const entry = this._meshForDoor(door);
          this._applyTransform(entry.root, door.transform);
          entry.slide.position.x = door.open ? 3.25 : 0;
          entry.root.setEnabled(true);
          if (entry.loaded) {
            for (const mesh of entry.container.meshes || []) {
              const mat = mesh.material;
              if (mat && /amber/i.test(mat.name)) mat.emissiveColor = door.locked ? new this.B.Color3(1, 0.22, 0.015) : new this.B.Color3(0.03, 0.48, 0.57);
            }
          }
          seenDoors.add(door.doorId);
        }
        catch { this._diag("E_UNKNOWN_PRESENTATION_ENTITY", door.doorId); }
      }
      for (const [id, entry] of this.doorMeshes) if (!seenDoors.has(id)) entry.root.setEnabled(false);
      const velocity = view.player.velocityMps || {};
      const speed = Math.hypot(Number(velocity.xM) || 0, Number(velocity.zM) || 0);
      this._selectPlayerAnimation(speed < 0.08 ? "idle" : speed < 2.2 ? "walk" : "run");
      this.setRearViewAuthorization(view.capabilities && view.capabilities.rearView, view.player.transform);
    }

    _rearSize() { return this.quality.preset === "low" ? 256 : 512; }

    _createRearView() {
      const camera = new this.B.FreeCamera("formal-rear-camera", new this.B.Vector3(0, 1.6, 0), this.scene);
      camera.layerMask = WORLD_LAYER;
      const texture = new this.B.RenderTargetTexture("formal-rear-rtt", this._rearSize(), this.scene, false, true);
      texture.activeCamera = camera;
      texture.refreshRate = this.quality.preset === "low" ? 2 : 1;
      this.scene.customRenderTargets.push(texture);
      const plane = this.B.MeshBuilder.CreatePlane("formal-rear-hud-plane", { width: 1.6, height: 0.9 }, this.scene);
      plane.parent = this.mainCamera;
      plane.position.set(-1.25, 0.72, 2.2);
      plane.layerMask = HUD_LAYER;
      plane.isPickable = false;
      const material = new this.B.StandardMaterial("formal-rear-hud-material", this.scene);
      material.disableLighting = true;
      material.emissiveTexture = texture;
      material.diffuseTexture = texture;
      material.backFaceCulling = false;
      plane.material = material;
      this.rear = { camera, texture, plane, material };
    }

    setRearViewAuthorization(authorization, playerTransform) {
      const granted = authorization && authorization.granted === true && typeof authorization.grantId === "string" && authorization.grantId.length > 0;
      if (!granted) { this._disposeRearView(); return; }
      if (!this.rear) this._createRearView();
      const p = vec3(playerTransform.positionM); const yaw = playerTransform.yawRad + Math.PI;
      this.rear.camera.position.set(p.xM, p.yM + 1.6, p.zM);
      this.rear.camera.setTarget(new this.B.Vector3(p.xM + Math.sin(yaw) * 5, p.yM + 1.6, p.zM + Math.cos(yaw) * 5));
    }

    _disposeRearView() {
      if (!this.rear) return;
      const index = this.scene.customRenderTargets.indexOf(this.rear.texture);
      if (index >= 0) this.scene.customRenderTargets.splice(index, 1);
      this.rear.plane.dispose(); this.rear.material.dispose(); this.rear.texture.dispose(); this.rear.camera.dispose();
      this.rear = null;
    }

    setQuality(next = {}) {
      this.quality = { ...this.quality, ...next };
      this.engine.setHardwareScalingLevel(1 / Math.max(0.25, this.quality.renderScale || 1));
      if (this.rear) { this.rear.texture.refreshRate = this.quality.preset === "low" ? 2 : 1; }
    }

    reset() {
      for (const entry of this.actorMeshes.values()) entry.root.dispose(false, true);
      for (const entry of this.doorMeshes.values()) entry.root.dispose(false, true);
      this.actorMeshes.clear(); this.doorMeshes.clear(); this._disposeRearView();
    }

    dispose() {
      if (this.disposed) return;
      this.disposed = true; this._disposeRearView(); this.engine.stopRenderLoop(); this.scene.dispose(); this.engine.dispose();
      this.actorMeshes.clear(); this.doorMeshes.clear();
    }
  }

  return { FormalTpsRenderer, HUD_LAYER, WORLD_LAYER };
});

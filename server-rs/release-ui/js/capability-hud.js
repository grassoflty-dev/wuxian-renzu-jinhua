(function (root, factory) {
  const api = factory();
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  if (root) root.FreezeV02CapabilityHud = api;
})(typeof window !== "undefined" ? window : globalThis, function () {
  "use strict";
  const TIERS = new Set(["healthy", "wounded", "severelyWounded", "critical"]);

  class CapabilityHud {
    constructor({ mapCanvas, documentRef } = {}) {
      this.document = documentRef || (typeof document !== "undefined" ? document : null);
      this.mapCanvas = mapCanvas || null;
      this.root = null; this.vitals = null; this.disposed = false;
      if (this.document) {
        this.root = this.document.createElement("section"); this.root.className = "capability-hud-v3"; this.root.setAttribute("aria-label", "能力信息");
        this.vitals = this.document.createElement("ul"); this.vitals.className = "capability-vitals"; this.root.appendChild(this.vitals);
        (this.document.body || this.document.documentElement).appendChild(this.root);
      }
    }

    apply(capabilities) {
      if (this.disposed || !capabilities) return;
      this.drawMap(capabilities.exploredMap);
      if (this.vitals) {
        this.vitals.replaceChildren();
        for (const item of capabilities.enemyVitals || []) {
          if (!item || typeof item.entityId !== "string" || !TIERS.has(item.tier)) continue;
          const li = this.document.createElement("li"); li.dataset.entityId = item.entityId; li.dataset.tier = item.tier;
          li.textContent = `${item.entityId}: ${this.tierLabel(item.tier)}`; this.vitals.appendChild(li);
        }
      }
    }

    tierLabel(tier) { return ({ healthy: "状态良好", wounded: "受伤", severelyWounded: "重伤", critical: "濒危" })[tier] || ""; }

    drawMap(map) {
      if (!this.mapCanvas || !map) return;
      const ctx = this.mapCanvas.getContext("2d"); if (!ctx) return;
      const width = this.mapCanvas.width || 240, height = this.mapCanvas.height || 180; ctx.clearRect(0, 0, width, height);
      const points = [];
      for (const room of map.rooms || []) for (const p of room.outlineM || []) if (Number.isFinite(p.xM) && Number.isFinite(p.zM)) points.push(p);
      if (!points.length) return;
      const minX = Math.min(...points.map(p => p.xM)), maxX = Math.max(...points.map(p => p.xM));
      const minZ = Math.min(...points.map(p => p.zM)), maxZ = Math.max(...points.map(p => p.zM));
      const scale = Math.min((width - 16) / Math.max(1, maxX - minX), (height - 16) / Math.max(1, maxZ - minZ));
      const project = p => [8 + (p.xM - minX) * scale, height - 8 - (p.zM - minZ) * scale];
      ctx.strokeStyle = "#79aeb6"; ctx.lineWidth = 1.5;
      const centers = new Map();
      for (const room of map.rooms || []) {
        if (!Array.isArray(room.outlineM) || room.outlineM.length < 2) continue;
        ctx.beginPath(); room.outlineM.forEach((p, i) => { const [x, y] = project(p); if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); }); ctx.closePath(); ctx.stroke();
        centers.set(room.roomId, { xM: room.outlineM.reduce((sum, p) => sum + p.xM, 0) / room.outlineM.length, zM: room.outlineM.reduce((sum, p) => sum + p.zM, 0) / room.outlineM.length });
      }
      ctx.strokeStyle = "#4f7379";
      for (const connection of map.connections || []) {
        const from = centers.get(connection.fromRoomId), to = centers.get(connection.toRoomId); if (!from || !to) continue;
        const [x1, y1] = project(from), [x2, y2] = project(to); ctx.beginPath(); ctx.moveTo(x1, y1); ctx.lineTo(x2, y2); ctx.stroke();
      }
      ctx.fillStyle = "#d8b25c";
      for (const objective of map.objectives || []) {
        const p = objective.positionM; if (!p || !Number.isFinite(p.xM) || !Number.isFinite(p.zM)) continue;
        const [x, y] = project(p); ctx.fillRect(x - 2, y - 2, 4, 4);
      }
      if (map.playerPositionM && Number.isFinite(map.playerPositionM.xM) && Number.isFinite(map.playerPositionM.zM)) {
        const [x, y] = project(map.playerPositionM); ctx.fillStyle = "#e4b85e"; ctx.beginPath(); ctx.arc(x, y, 3, 0, Math.PI * 2); ctx.fill();
      }
    }

    reset() { if (this.vitals) this.vitals.replaceChildren(); if (this.mapCanvas) this.mapCanvas.getContext("2d")?.clearRect(0, 0, this.mapCanvas.width, this.mapCanvas.height); }
    dispose() { if (this.disposed) return; this.disposed = true; this.root?.remove(); this.root = null; this.vitals = null; }
  }
  return { CapabilityHud, TIERS };
});

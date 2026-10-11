const assert = require("node:assert/strict");
const { CapabilityHud } = require("../../release-ui/js/capability-hud.js");

function element() { return { children: [], dataset: {}, appendChild(x) { this.children.push(x); }, replaceChildren(...x) { this.children = x; }, setAttribute() {}, remove() { this.removed = true; } }; }
module.exports = function runHudTest() {
  const body = element(); const documentRef = { body, createElement: () => element() };
  const calls = []; const ctx = { clearRect: (...x) => calls.push(["clear", ...x]), beginPath() {}, moveTo() {}, lineTo() {}, closePath() {}, stroke() {}, arc() {}, fill() {}, fillRect: (...x) => calls.push(["objective", ...x]) };
  const mapCanvas = { width: 240, height: 180, getContext: () => ctx };
  const hud = new CapabilityHud({ mapCanvas, documentRef });
  hud.apply({
    exploredMap: { playerPositionM: { xM: 1, zM: 1 }, rooms: [{ roomId: "GH-A01", outlineM: [{ xM: 0, zM: 0 }, { xM: 2, zM: 0 }, { xM: 2, zM: 2 }] }, { roomId: "GH-A02", outlineM: [{ xM: 3, zM: 0 }, { xM: 5, zM: 0 }, { xM: 5, zM: 2 }] }], connections: [{ connectionId: "a01-a02", fromRoomId: "GH-A01", toRoomId: "GH-A02" }], objectives: [{ objectiveId: "power", positionM: { xM: 4, yM: 0, zM: 1 } }] },
    enemyVitals: [{ entityId: "sentinel", tier: "critical", currentHp: 1, maxHp: 100 }, { entityId: "unknown", tier: "defeated" }]
  });
  assert.equal(hud.vitals.children.length, 1); assert.equal(hud.vitals.children[0].textContent, "sentinel: 濒危");
  assert.equal(hud.vitals.children[0].textContent.includes("100"), false, "exact HP must not leak");
  assert.ok(calls.some(x => x[0] === "clear")); assert.ok(calls.some(x => x[0] === "objective")); hud.dispose(); assert.equal(hud.disposed, true);
  return { name: "hud-contract-projection", passed: true, exactEnemyHpRendered: false };
};

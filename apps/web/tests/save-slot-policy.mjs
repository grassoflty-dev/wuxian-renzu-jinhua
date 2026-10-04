import test from "node:test";
import assert from "node:assert/strict";
import { canContinueSaveSlot, canOverwriteSaveSlot } from "../dist/bridge/save-slot-policy.js";

const good = { slotId: "slot-a", displayName: "存档", updatedAtMs: 0, worldId: "grey_hive", checkpointId: null,
  playerPositionM: null, currentHp: 1, maxHp: 1, currentEnergy: 1, maxEnergy: 1, gateOpen: false,
  completedEvents: [], readOnly: false, valid: true, errorCode: null };

test("create and overwrite require a writable selection; legacy read-only only permits explicit continue", () => {
  const legacy = { ...good, slotId: "legacy-save-v3", readOnly: true };
  assert.equal(canContinueSaveSlot(good), true);
  assert.equal(canOverwriteSaveSlot(good), true);
  assert.equal(canContinueSaveSlot(legacy), true);
  assert.equal(canOverwriteSaveSlot(legacy), false);
});

test("corrupt slots are neither continue nor overwrite targets", () => {
  const corrupt = { ...good, valid: false, readOnly: true, errorCode: "E_SLOT_CORRUPT" };
  assert.equal(canContinueSaveSlot(corrupt), false);
  assert.equal(canOverwriteSaveSlot(corrupt), false);
  assert.equal(canContinueSaveSlot(undefined), false);
});


test("dead and unknown-HP slots cannot be continued, even when their envelope is valid", () => {
  for (const currentHp of [0, -1, null, undefined, NaN, Infinity, "80"]) {
    assert.equal(canContinueSaveSlot({ ...good, currentHp }), false);
  }
  assert.equal(canContinueSaveSlot({ ...good, currentHp: 0.5 }), true);
});

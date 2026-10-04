import test from "node:test";
import assert from "node:assert/strict";
import {
  ACCESSIBILITY_PREFERENCES_STORAGE_KEY,
  AccessibilityPreferenceStore,
  bindAccessibilityPreferenceToggle,
} from "../dist/ui/AccessibilityPreferences.js";

class MemoryStorage {
  values = new Map();
  writes = [];

  getItem(key) { return this.values.has(key) ? this.values.get(key) : null; }
  setItem(key, value) {
    this.writes.push([key, value]);
    this.values.set(key, value);
  }
}

class FakeToggle {
  pressed = null;
  handler = null;

  setPressed(value) { this.pressed = value; }
  onToggle(handler) { this.handler = handler; }
  click() { this.handler?.(); }
}

function makeTarget() {
  return { dataset: {} };
}

test("startup restoration and settings controls share independent persisted state", () => {
  const storage = new MemoryStorage();
  const target = makeTarget();
  const preferences = new AccessibilityPreferenceStore(() => storage, target);
  assert.deepEqual(preferences.restore(), { highContrast: false, reducedMotion: false });
  assert.deepEqual(target.dataset, {});

  const contrast = new FakeToggle();
  const motion = new FakeToggle();
  bindAccessibilityPreferenceToggle(contrast, preferences, "highContrast");
  bindAccessibilityPreferenceToggle(motion, preferences, "reducedMotion");
  assert.equal(contrast.pressed, false);
  assert.equal(motion.pressed, false);

  contrast.click();
  assert.equal(contrast.pressed, true);
  assert.equal(target.dataset.contrast, "high");
  assert.equal(target.dataset.motion, undefined);
  motion.click();
  assert.equal(motion.pressed, true);
  assert.equal(target.dataset.motion, "reduced");
  assert.deepEqual(JSON.parse(storage.getItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY)), {
    schemaVersion: 2,
    textScalePercent: 100, uiScalePercent: 100,
    highContrast: true,
    reducedMotion: true,
  });

  const restartedTarget = makeTarget();
  const restarted = new AccessibilityPreferenceStore(() => storage, restartedTarget);
  assert.deepEqual(restarted.restore(), { highContrast: true, reducedMotion: true });
  assert.deepEqual(restartedTarget.dataset, { contrast: "high", motion: "reduced" });
  const reopenedContrast = new FakeToggle();
  bindAccessibilityPreferenceToggle(reopenedContrast, restarted, "highContrast");
  assert.equal(reopenedContrast.pressed, true);
});

test("clearing one preference preserves the other across a new store", () => {
  const storage = new MemoryStorage();
  const target = makeTarget();
  const preferences = new AccessibilityPreferenceStore(() => storage, target);
  preferences.restore();
  preferences.set("highContrast", true);
  preferences.set("reducedMotion", true);
  preferences.set("highContrast", false);

  assert.equal(target.dataset.contrast, undefined);
  assert.equal(target.dataset.motion, "reduced");
  const restored = new AccessibilityPreferenceStore(() => storage, makeTarget());
  assert.deepEqual(restored.restore(), { highContrast: false, reducedMotion: true });
});

test("corrupt and unknown-version records remain untouched while session toggles still work", () => {
  for (const raw of [
    "{not-json",
    JSON.stringify({ schemaVersion: 2, highContrast: true, reducedMotion: true }),
    JSON.stringify({ schemaVersion: 1, highContrast: true, reducedMotion: true, futureFlag: true }),
  ]) {
    const storage = new MemoryStorage();
    storage.values.set(ACCESSIBILITY_PREFERENCES_STORAGE_KEY, raw);
    const target = makeTarget();
    const preferences = new AccessibilityPreferenceStore(() => storage, target);
    assert.deepEqual(preferences.restore(), { highContrast: false, reducedMotion: false });
    assert.deepEqual(target.dataset, {});

    const contrast = new FakeToggle();
    bindAccessibilityPreferenceToggle(contrast, preferences, "highContrast");
    contrast.click();
    assert.equal(contrast.pressed, true);
    assert.equal(target.dataset.contrast, "high");
    assert.equal(storage.getItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY), raw);
    assert.equal(storage.writes.length, 0);
  }
});

test("throwing storage falls back to session-only state without blocking controls", () => {
  const readFailure = new AccessibilityPreferenceStore(() => ({
    getItem() { throw new Error("storage read blocked"); },
    setItem() { assert.fail("read failure must not write over unknown storage"); },
  }), makeTarget());
  assert.deepEqual(readFailure.restore(), { highContrast: false, reducedMotion: false });
  assert.deepEqual(readFailure.set("reducedMotion", true), { highContrast: false, reducedMotion: true });

  const target = makeTarget();
  const writeFailure = new AccessibilityPreferenceStore(() => ({
    getItem() { return null; },
    setItem() { throw new Error("storage write blocked"); },
  }), target);
  writeFailure.restore();
  const motion = new FakeToggle();
  bindAccessibilityPreferenceToggle(motion, writeFailure, "reducedMotion");
  motion.click();
  assert.equal(motion.pressed, true);
  assert.equal(target.dataset.motion, "reduced");
  assert.deepEqual(writeFailure.get(), { highContrast: false, reducedMotion: true });
});

test("valid v1 migrates on deliberate change and independent scales survive restart", () => {
  const storage=new MemoryStorage();const raw=JSON.stringify({schemaVersion:1,highContrast:true,reducedMotion:true});
  storage.values.set(ACCESSIBILITY_PREFERENCES_STORAGE_KEY,raw);
  const target=makeTarget();const store=new AccessibilityPreferenceStore(()=>storage,target);
  assert.deepEqual(store.restore(),{highContrast:true,reducedMotion:true});
  assert.deepEqual(store.getScale(),{textScalePercent:100,uiScalePercent:100});
  assert.equal(storage.getItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY),raw);
  store.setScale("textScalePercent",130);store.setScale("uiScalePercent",125);
  assert.deepEqual(target.dataset,{contrast:"high",motion:"reduced",textScale:"130",uiScale:"125"});
  assert.deepEqual(JSON.parse(storage.getItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY)),{schemaVersion:2,highContrast:true,reducedMotion:true,textScalePercent:130,uiScalePercent:125});
  const target2=makeTarget();const next=new AccessibilityPreferenceStore(()=>storage,target2);next.restore();
  assert.deepEqual(next.getScale(),{textScalePercent:130,uiScalePercent:125});
  next.setScale("textScalePercent",100);assert.equal(target2.dataset.textScale,undefined);assert.equal(target2.dataset.uiScale,"125");
  assert.deepEqual(next.get(),{highContrast:true,reducedMotion:true});
});

test("malformed v2 records stay intact while local controls remain usable", () => {
  const good={schemaVersion:2,highContrast:false,reducedMotion:true,textScalePercent:115,uiScalePercent:110};
  const missing={...good};delete missing.textScalePercent;
  for(const record of [missing,{...good,extra:1},{...good,schemaVersion:3},{...good,textScalePercent:114},{...good,uiScalePercent:130},{...good,textScalePercent:"115"},{...good,uiScalePercent:null},{...good,reducedMotion:0}]) {
    const storage=new MemoryStorage();const raw=JSON.stringify(record);storage.values.set(ACCESSIBILITY_PREFERENCES_STORAGE_KEY,raw);
    const store=new AccessibilityPreferenceStore(()=>storage,makeTarget());store.restore();assert.deepEqual(store.getScale(),{textScalePercent:100,uiScalePercent:100});
    store.setScale("textScalePercent",130);store.set("highContrast",true);assert.equal(storage.getItem(ACCESSIBILITY_PREFERENCES_STORAGE_KEY),raw);assert.equal(storage.writes.length,0);
  }
});

test("invalid commands never persist; blocked storage still applies session scale", () => {
  const storage=new MemoryStorage();const target=makeTarget();const store=new AccessibilityPreferenceStore(()=>storage,target);store.restore();
  for(const value of [NaN,Infinity,-Infinity,0,99,101,1.3,"130",null,undefined])store.setScale("textScalePercent",value);
  store.setScale("unknown",125);assert.equal(storage.writes.length,0);assert.deepEqual(store.getScale(),{textScalePercent:100,uiScalePercent:100});
  storage.setItem=()=>{throw Error("blocked");};store.setScale("uiScalePercent",125);assert.equal(target.dataset.uiScale,"125");
  const copy=store.getScale();copy.uiScalePercent=0;assert.equal(store.getScale().uiScalePercent,125);
});

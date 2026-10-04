import assert from "node:assert/strict";
import test from "node:test";
import { AUDIO_PREFERENCES_STORAGE_KEY, AudioCuePlayer } from "../dist/audio/AudioCuePlayer.js";

class MemoryStorage {
  values = new Map();
  writes = 0;
  getItem(key) { return this.values.get(key) ?? null; }
  setItem(key, value) { this.writes++; this.values.set(key, value); }
}

class FakeRuntime {
  status = "locked";
  unlockCalls = 0;
  played = [];
  ambience = [];
  stoppedVoices = 0;
  stoppedAmbient = 0;
  closed = false;
  state() { return this.status; }
  async unlock() { this.unlockCalls++; this.status = "running"; return true; }
  play(cue, volume) { this.played.push({ id: cue.id, volume }); return this.status === "running"; }
  startAmbient(cue, volume) { this.ambience.push({ id: cue.id, volume }); return this.status === "running"; }
  stopVoices() { this.stoppedVoices++; }
  stopAmbient() { this.stoppedAmbient++; }
  close() { this.closed = true; this.status = "unavailable"; }
}

function event(kind, worldEpoch = 4, eventId = 1, protocolVersion = 2) {
  return {
    protocolVersion, eventId, worldEpoch, serverTick: eventId, kind,
    positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 0, intensity: 1,
  };
}

function player(storage = new MemoryStorage(), runtime = new FakeRuntime()) {
  const cuePlayer = new AudioCuePlayer({ getStorage: () => storage, createRuntime: () => runtime });
  return { cuePlayer, storage, runtime };
}

test("first user gesture unlocks the injected runtime; locked cues are not replayed later", async () => {
  const { cuePlayer, runtime } = player();
  cuePlayer.setEpoch(4);
  assert.equal(cuePlayer.handlePresentationEvents([event("PulseCast", 4, 1)]), 0);
  assert.equal(runtime.played.length, 0);
  assert.equal(await cuePlayer.unlock(), true);
  assert.equal(runtime.unlockCalls, 1);
  assert.equal(cuePlayer.handlePresentationEvents([event("PulseCast", 4, 1)]), 0);
  assert.equal(cuePlayer.handlePresentationEvents([event("PulseCast", 4, 2)]), 1);
  assert.equal(cuePlayer.playUiError(), true);
  assert.equal(cuePlayer.playUiCancel(), true);
  assert.deepEqual(runtime.played.map(row => row.id), [
    "audio.skill.pulse.cast", "audio.ui.error", "audio.ui.cancel",
  ]);
});

test("only current-epoch authoritative events play once and pause stops active sounds", async () => {
  const { cuePlayer, runtime } = player();
  cuePlayer.setScene("grey_hive", "gh_entry_maintenance");
  assert.equal(await cuePlayer.unlock(), true);
  assert.equal(runtime.ambience.at(-1).id, "audio.gh.ambient.facility");
  cuePlayer.setEpoch(4);
  assert.equal(cuePlayer.handlePresentationEvents([event("GuardStarted", 3, 100)]), 0);
  assert.equal(cuePlayer.handlePresentationEvents([event("GuardStarted", 4, 7)]), 1);
  assert.equal(cuePlayer.handlePresentationEvents([event("GuardStarted", 4, 7)]), 0);
  cuePlayer.setEpoch(5);
  assert.ok(runtime.stoppedVoices > 0);
  assert.equal(cuePlayer.handlePresentationEvents([event("PierceHit", 4, 8)]), 0);
  assert.equal(cuePlayer.handlePresentationEvents([event("PierceHit", 5, 1)]), 1);
  cuePlayer.suspend();
  assert.equal(cuePlayer.handlePresentationEvents([event("PulseHit", 5, 2)]), 0);
  const stoppedAfterPause = runtime.stoppedVoices;
  cuePlayer.resume();
  assert.ok(runtime.stoppedAmbient > 0);
  assert.ok(runtime.ambience.length >= 2);
  assert.ok(runtime.stoppedVoices >= stoppedAfterPause);
  await cuePlayer.dispose();
  assert.equal(runtime.closed, true);
});

test("mute and master volume persist locally and affect both cues and ambience", async () => {
  const storage = new MemoryStorage();
  const runtime = new FakeRuntime();
  const { cuePlayer } = player(storage, runtime);
  assert.deepEqual(cuePlayer.settings, { muted: false, volume: 0.35 });
  cuePlayer.setScene("return_station", "rs_core_room");
  await cuePlayer.unlock();
  cuePlayer.setVolume(0);
  assert.equal(cuePlayer.playUiConfirm(), false);
  cuePlayer.setVolume(0.62);
  cuePlayer.setMuted(true);
  assert.equal(cuePlayer.playUiConfirm(), false);
  assert.ok(runtime.stoppedAmbient > 0);
  cuePlayer.setMuted(false);
  assert.equal(cuePlayer.playUiConfirm(), true);
  assert.equal(runtime.played.at(-1).volume, 0.62);
  const stored = JSON.parse(storage.getItem(AUDIO_PREFERENCES_STORAGE_KEY));
  assert.deepEqual(stored, { schemaVersion: 1, muted: false, volume: 0.62 });

  const restored = new AudioCuePlayer({ getStorage: () => storage, createRuntime: () => new FakeRuntime() });
  assert.deepEqual(restored.settings, { muted: false, volume: 0.62 });
  assert.match(restored.statusText, /首次操作/);
});

test("volume is clamped and corrupt preferences are preserved instead of overwritten", () => {
  const storage = new MemoryStorage();
  storage.values.set(AUDIO_PREFERENCES_STORAGE_KEY, "{bad preferences");
  const { cuePlayer } = player(storage);
  const writes = storage.writes;
  cuePlayer.setVolume(2);
  assert.equal(cuePlayer.settings.volume, 1);
  assert.equal(cuePlayer.playUiConfirm(), false);
  cuePlayer.setVolume(0);
  assert.equal(cuePlayer.settings.volume, 0);
  assert.equal(cuePlayer.playUiConfirm(), false);
  assert.equal(storage.getItem(AUDIO_PREFERENCES_STORAGE_KEY), "{bad preferences");
  assert.equal(storage.writes, writes);
  assert.match(cuePlayer.statusText, /本次运行/);
});

test("unsupported or failing audio environments report status without throwing", async () => {
  const unsupported = new AudioCuePlayer({ getStorage: () => new MemoryStorage(), createRuntime: () => null });
  assert.equal(await unsupported.unlock(), false);
  assert.match(unsupported.statusText, /不支持 Web Audio/);

  const failed = new AudioCuePlayer({
    getStorage: () => new MemoryStorage(),
    createRuntime: () => { throw new Error("no output"); },
  });
  assert.equal(await failed.unlock(), false);
  assert.match(failed.statusText, /输出不可用/);
});

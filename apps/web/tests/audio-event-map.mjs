import assert from "node:assert/strict";
import test from "node:test";
import { AUDIO_CUE_LIBRARY, PRESENTATION_AUDIO_IDS, ambientAudioCueForWorld, audioCueForPresentationEvent } from "../dist/audio/AudioEventMap.js";

function event(kind, protocolVersion = 2) {
  return {
    protocolVersion, eventId: 1, worldEpoch: 8, serverTick: 22, kind,
    positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 0, intensity: 1,
  };
}

test("authorized presentation kinds map to stable planned cue IDs", () => {
  assert.deepEqual(PRESENTATION_AUDIO_IDS, {
    ResonanceWardenDecoy: "audio.mh.warden.decoy",
    ResonanceWardenStrikeWindup: "audio.mh.warden.strike.warning",
    ResonanceWardenPulseWindup: "audio.mh.warden.pulse.warning",
    ResonanceWardenStrikeImpact: "audio.mh.warden.strike.impact",
    ResonanceWardenPulseImpact: "audio.mh.warden.pulse.impact",
    ResonanceWardenDeath: "audio.mh.warden.death",
    AttackStarted: "audio.player.attack",
    Hit: "audio.combat.hit",
    Damaged: "audio.combat.hurt",
    DamageAbsorbed: "audio.combat.absorbed",
    DashStarted: "audio.player.dash",
    PulseCast: "audio.skill.pulse.cast",
    PulseHit: "audio.skill.pulse.hit",
    GuardStarted: "audio.skill.guard.start",
    GuardImpact: "audio.skill.guard.hit",
    PierceStarted: "audio.skill.pierce.cast",
    PierceHit: "audio.skill.pierce.hit",
    SentinelAttackWindup: "audio.enemy.sentinel.warning.light",
    SentinelHeavyWindup: "audio.enemy.sentinel.warning.heavy",
    SentinelChargeWindup: "audio.enemy.sentinel.warning.charge",
  });
  for (const [kind, id] of Object.entries(PRESENTATION_AUDIO_IDS)) {
    const cue = audioCueForPresentationEvent(event(kind));
    assert.equal(cue?.id, id);
    assert.equal(cue?.temporary_audio, true);
  }
  assert.equal(audioCueForPresentationEvent(event("PulseCast", 1)), null);
  assert.equal(audioCueForPresentationEvent(event("EnergyChanged")), null);
  assert.equal(audioCueForPresentationEvent(event("FailedCommand")), null);
});

test("skill cues use distinguishable bounded synthesized recipes", () => {
  const ids = [
    "audio.skill.pulse.cast", "audio.skill.pulse.hit", "audio.skill.guard.start",
    "audio.skill.guard.hit", "audio.skill.pierce.cast", "audio.skill.pierce.hit", "audio.player.dash",
  ];
  const signatures = ids.map(id => JSON.stringify(AUDIO_CUE_LIBRARY[id].tones));
  assert.equal(new Set(signatures).size, signatures.length);
  for (const id of ids) {
    const cue = AUDIO_CUE_LIBRARY[id];
    assert.equal(cue.temporary_audio, true);
    assert.ok(cue.peakGain > 0 && cue.peakGain <= 0.22);
    assert.ok(cue.tones.every(tone => tone.durationMs <= 140 && tone.frequencyHz <= 1600));
  }
});

test("environment cues are scoped to Return Station and Grey Hive", () => {
  assert.equal(ambientAudioCueForWorld("return_station")?.id, "bgm.return_station");
  assert.equal(ambientAudioCueForWorld("grey_hive")?.id, "audio.gh.ambient.facility");
  assert.equal(ambientAudioCueForWorld("mist_harbor"), null);
  assert.equal(ambientAudioCueForWorld("clockworks"), null);
});

test("Sentinel windups use distinct short temporary warning cues", () => {
  const light = audioCueForPresentationEvent(event("SentinelAttackWindup"));
  const heavy = audioCueForPresentationEvent(event("SentinelHeavyWindup"));
  assert.equal(light.id, "audio.enemy.sentinel.warning.light");
  assert.equal(heavy.id, "audio.enemy.sentinel.warning.heavy");
  assert.notEqual(light.id, heavy.id);
  assert.equal(light.temporary_audio, true);
  assert.equal(heavy.temporary_audio, true);
  assert.ok(light.tones.reduce((end, tone) => Math.max(end, tone.offsetMs + tone.durationMs), 0) < 150);
  assert.ok(heavy.tones.reduce((end, tone) => Math.max(end, tone.offsetMs + tone.durationMs), 0) < 250);
  assert.equal(audioCueForPresentationEvent(event("SentinelDeath")), null);
});

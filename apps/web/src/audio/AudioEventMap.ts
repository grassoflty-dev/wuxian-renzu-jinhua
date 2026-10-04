import type { PresentationEvent } from "../protocol/types.js";

export type AudioCueId =
  | "audio.mh.wraith.cast.warning" | "audio.mh.wraith.cast.impact" | "audio.mh.wraith.blink"
  | "audio.ui.confirm"
  | "audio.ui.cancel"
  | "audio.ui.error"
  | "audio.ui.tab"
  | "audio.player.attack"
  | "audio.combat.hit"
  | "audio.combat.hurt"
  | "audio.combat.absorbed"
  | "audio.player.dash"
  | "audio.skill.pulse.cast"
  | "audio.skill.pulse.hit"
  | "audio.skill.guard.start"
  | "audio.skill.guard.hit"
  | "audio.skill.pierce.cast"
  | "audio.skill.pierce.hit"
  | "audio.enemy.sentinel.warning.light"
  | "audio.enemy.sentinel.warning.heavy"
  | "audio.enemy.sentinel.warning.charge"
  | "audio.gh.ambient.facility"
  | "bgm.return_station"
  | "audio.mh.warden.decoy"
  | "audio.mh.warden.strike.warning"
  | "audio.mh.warden.pulse.warning"
  | "audio.mh.warden.strike.impact"
  | "audio.mh.warden.pulse.impact"
  | "audio.mh.warden.death"
  | "audio.cw.enemy.drone.warning"
  | "audio.cw.enemy.hound.bite.warning"
  | "audio.cw.enemy.hound.leap.warning"
  | "audio.cw.enemy.drone.impact"
  | "audio.cw.enemy.hound.bite.impact"
  | "audio.cw.enemy.hound.leap.impact"
  | "audio.cw.enemy.alert"
  | "audio.cw.enemy.stagger"
  | "audio.cw.enemy.death"
  | "audio.gh.brute.charge.warning"
  | "audio.gh.brute.slam.warning"
  | "audio.gh.brute.charge.impact"
  | "audio.gh.brute.slam.impact"
  | "audio.gh.swarm.lunge.warning" | "audio.gh.swarm.lunge.impact" | "audio.gh.swarm.member.hit" | "audio.gh.swarm.member.disperse"
  | "audio.mh.tidebound.swing.warning" | "audio.mh.tidebound.charge.warning" | "audio.mh.tidebound.swing.impact" | "audio.mh.tidebound.charge.impact";

export interface AudioTone {
  frequencyHz: number;
  endFrequencyHz?: number;
  offsetMs: number;
  durationMs: number;
  waveform: OscillatorType;
}

export interface AudioCueDefinition {
  readonly id: AudioCueId;
  readonly temporary_audio: true;
  readonly peakGain: number;
  readonly tones: readonly AudioTone[];
}

function cue(
  id: AudioCueId,
  peakGain: number,
  tones: readonly AudioTone[],
): AudioCueDefinition {
  return Object.freeze({ id, temporary_audio: true as const, peakGain, tones: Object.freeze(tones) });
}

const tone = (
  frequencyHz: number,
  durationMs: number,
  offsetMs = 0,
  waveform: OscillatorType = "sine",
  endFrequencyHz?: number,
): AudioTone => endFrequencyHz === undefined
  ? { frequencyHz, durationMs, offsetMs, waveform }
  : { frequencyHz, durationMs, offsetMs, waveform, endFrequencyHz };

/** Stable IDs follow the planned audio registry; recipes are synthesized and temporary. */
export const AUDIO_CUE_LIBRARY: Readonly<Record<AudioCueId, AudioCueDefinition>> = Object.freeze({
  "audio.mh.wraith.cast.warning": cue("audio.mh.wraith.cast.warning", .15, [tone(440,180,0,"sine",880),tone(880,120,200,"sine",1100)]),
  "audio.mh.wraith.cast.impact": cue("audio.mh.wraith.cast.impact", .14, [tone(1100,110,0,"sine",220)]),
  "audio.mh.wraith.blink": cue("audio.mh.wraith.blink", .10, [tone(620,140,0,"sine",310)]),
  "audio.ui.confirm": cue("audio.ui.confirm", 0.16, [tone(620, 45), tone(830, 60, 46)]),
  "audio.ui.cancel": cue("audio.ui.cancel", 0.13, [tone(520, 55), tone(390, 75, 50)]),
  "audio.ui.error": cue("audio.ui.error", 0.17, [tone(310, 85, 0, "triangle"), tone(230, 105, 82, "triangle")]),
  "audio.ui.tab": cue("audio.ui.tab", 0.09, [tone(740, 28)]),
  "audio.player.attack": cue("audio.player.attack", 0.12, [tone(250, 75, 0, "triangle", 180)]),
  "audio.combat.hit": cue("audio.combat.hit", .17, [tone(560, 45, 0, "triangle", 160), tone(125, 65, 12, "sine", 70)]),
  "audio.combat.hurt": cue("audio.combat.hurt", .16, [tone(210, 120, 0, "triangle", 65)]),
  "audio.combat.absorbed": cue("audio.combat.absorbed", .07, [tone(640, 50, 0, "sine", 460)]),
  "audio.player.dash": cue("audio.player.dash", 0.16, [tone(380, 90, 0, "triangle", 980)]),
  "audio.skill.pulse.cast": cue("audio.skill.pulse.cast", 0.17, [tone(270, 100, 0, "sine", 690), tone(690, 115, 76, "sine", 920)]),
  "audio.skill.pulse.hit": cue("audio.skill.pulse.hit", 0.16, [tone(540, 65, 0, "triangle", 290), tone(370, 100, 48, "sine")]),
  "audio.skill.guard.start": cue("audio.skill.guard.start", 0.15, [tone(180, 95, 0, "triangle", 250), tone(250, 95, 74, "sine")]),
  "audio.skill.guard.hit": cue("audio.skill.guard.hit", 0.2, [tone(135, 110, 0, "triangle", 90), tone(430, 85, 18, "sine")]),
  "audio.skill.pierce.cast": cue("audio.skill.pierce.cast", 0.14, [tone(940, 75, 0, "sawtooth", 1520)]),
  "audio.skill.pierce.hit": cue("audio.skill.pierce.hit", 0.18, [tone(730, 65, 0, "triangle", 310), tone(155, 120, 30, "sine")]),
  "audio.enemy.sentinel.warning.light": cue("audio.enemy.sentinel.warning.light", 0.13,
    [tone(720, 92, 0, "triangle", 560)]),
  "audio.enemy.sentinel.warning.heavy": cue("audio.enemy.sentinel.warning.heavy", 0.17,
    [tone(245, 140, 0, "triangle", 165), tone(520, 85, 42, "sine")]),
  "audio.enemy.sentinel.warning.charge": cue("audio.enemy.sentinel.warning.charge", 0.17,
    [tone(180, 120, 0, "triangle", 360), tone(360, 100, 145, "sine", 720)]),
  "audio.gh.ambient.facility": cue("audio.gh.ambient.facility", 0.035, [tone(55, 1000, 0, "sine"), tone(73.4, 1000, 0, "sine")]),
  // Original temporary recipes, never admitted as final SFX.
  "audio.mh.warden.decoy": cue("audio.mh.warden.decoy", .13, [tone(480,100,0,"sine",300),tone(300,110,100,"sine",220)]),
  "audio.mh.warden.strike.warning": cue("audio.mh.warden.strike.warning", .18, [tone(260,160,0,"triangle",520),tone(520,160,170,"sine",780)]),
  "audio.mh.warden.pulse.warning": cue("audio.mh.warden.pulse.warning", .18, [tone(180,130,0,"triangle"),tone(240,130,170,"triangle"),tone(320,160,340,"sine")]),
  "audio.mh.warden.strike.impact": cue("audio.mh.warden.strike.impact", .20, [tone(700,90,0,"triangle",100)]),
  "audio.mh.warden.pulse.impact": cue("audio.mh.warden.pulse.impact", .20, [tone(120,220,0,"triangle",60),tone(380,150,40,"sine",90)]),
  "audio.mh.warden.death": cue("audio.mh.warden.death", .16, [tone(500,200,0,"sine",300),tone(300,220,200,"sine",100)]),
  "audio.cw.enemy.drone.warning": cue("audio.cw.enemy.drone.warning", .14, [tone(880, 100, 0, "sine", 1320), tone(1100, 80, 115, "sine")]),
  "audio.cw.enemy.hound.bite.warning": cue("audio.cw.enemy.hound.bite.warning", .16, [tone(180, 150, 0, "triangle", 260)]),
  "audio.cw.enemy.hound.leap.warning": cue("audio.cw.enemy.hound.leap.warning", .17, [tone(170, 100, 0, "triangle", 300), tone(240, 130, 120, "triangle", 420)]),
  "audio.cw.enemy.drone.impact": cue("audio.cw.enemy.drone.impact", .13, [tone(1300, 90, 0, "sine", 440)]),
  "audio.cw.enemy.hound.bite.impact": cue("audio.cw.enemy.hound.bite.impact", .17, [tone(240, 85, 0, "triangle", 90)]),
  "audio.cw.enemy.hound.leap.impact": cue("audio.cw.enemy.hound.leap.impact", .18, [tone(140, 145, 0, "triangle", 65), tone(300, 70, 20, "sine", 120)]),
  "audio.cw.enemy.alert": cue("audio.cw.enemy.alert", .10, [tone(620, 75, 0, "sine", 820)]),
  "audio.cw.enemy.stagger": cue("audio.cw.enemy.stagger", .12, [tone(400, 90, 0, "triangle", 150)]),
  "audio.cw.enemy.death": cue("audio.cw.enemy.death", .12, [tone(280, 180, 0, "triangle", 70)]),
  "audio.gh.brute.charge.warning": cue("audio.gh.brute.charge.warning", .17, [tone(105, 150, 0, "triangle", 190), tone(180, 180, 170, "triangle", 280)]),
  "audio.gh.brute.slam.warning": cue("audio.gh.brute.slam.warning", .18, [tone(160, 160, 0, "triangle", 95), tone(310, 100, 100, "sine", 180)]),
  "audio.gh.brute.charge.impact": cue("audio.gh.brute.charge.impact", .19, [tone(140, 140, 0, "triangle", 55)]),
  "audio.mh.tidebound.swing.warning": cue("audio.mh.tidebound.swing.warning", .16, [tone(240, 180, 0, "sine", 390), tone(155, 140, 120, "triangle", 110)]),
  "audio.mh.tidebound.charge.warning": cue("audio.mh.tidebound.charge.warning", .17, [tone(135, 170, 0, "triangle", 270), tone(270, 160, 160, "sine", 410)]),
  "audio.mh.tidebound.swing.impact": cue("audio.mh.tidebound.swing.impact", .17, [tone(220, 120, 0, "triangle", 65)]),
  "audio.mh.tidebound.charge.impact": cue("audio.mh.tidebound.charge.impact", .18, [tone(170, 170, 0, "triangle", 50), tone(410, 70, 0, "sine", 120)]),
  "audio.gh.swarm.lunge.warning": cue("audio.gh.swarm.lunge.warning", .15, [tone(520, 110, 0, "triangle", 260), tone(690, 130, 110, "triangle", 340)]),
  "audio.gh.swarm.lunge.impact": cue("audio.gh.swarm.lunge.impact", .16, [tone(180, 90, 0, "triangle", 75)]),
  "audio.gh.swarm.member.hit": cue("audio.gh.swarm.member.hit", .08, [tone(740, 55, 0, "triangle", 420)]),
  "audio.gh.swarm.member.disperse": cue("audio.gh.swarm.member.disperse", .10, [tone(850, 130, 0, "triangle", 140)]),
  "audio.gh.brute.slam.impact": cue("audio.gh.brute.slam.impact", .20, [tone(115, 200, 0, "triangle", 45), tone(380, 70, 10, "sine", 90)]),
  "bgm.return_station": cue("bgm.return_station", 0.025, [tone(110, 1000, 0, "sine"), tone(164.8, 1000, 0, "sine")]),
});

export const PRESENTATION_AUDIO_IDS: Readonly<Record<string, AudioCueId>> = Object.freeze({
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

/** Motion has no independent sound: warning/impact own the short synthesized lifecycle. */
export const CLOCKWORKS_ENEMY_AUDIO_IDS: Readonly<Record<string, AudioCueId | Partial<Record<"pressure_shot" | "bite" | "leap" | "charge" | "slam" | "lunge" | "tide_swing" | "tide_charge" | "signal_shot", AudioCueId>>>> = Object.freeze({
  SignalBlink: "audio.mh.wraith.blink",
  EnemyAlert: "audio.cw.enemy.alert", EnemyStagger: "audio.cw.enemy.stagger", EnemyDeath: "audio.cw.enemy.death",
  EnemyMemberHit: "audio.gh.swarm.member.hit",
  EnemyMemberDisperse: "audio.gh.swarm.member.disperse",
  EnemyAttackTelegraph: { signal_shot: "audio.mh.wraith.cast.warning", pressure_shot: "audio.cw.enemy.drone.warning", bite: "audio.cw.enemy.hound.bite.warning", leap: "audio.cw.enemy.hound.leap.warning", charge: "audio.gh.brute.charge.warning", slam: "audio.gh.brute.slam.warning", lunge: "audio.gh.swarm.lunge.warning", tide_swing: "audio.mh.tidebound.swing.warning", tide_charge: "audio.mh.tidebound.charge.warning" },
  EnemyAttackImpact: { signal_shot: "audio.mh.wraith.cast.impact", pressure_shot: "audio.cw.enemy.drone.impact", bite: "audio.cw.enemy.hound.bite.impact", leap: "audio.cw.enemy.hound.leap.impact", charge: "audio.gh.brute.charge.impact", slam: "audio.gh.brute.slam.impact", lunge: "audio.gh.swarm.lunge.impact", tide_swing: "audio.mh.tidebound.swing.impact", tide_charge: "audio.mh.tidebound.charge.impact" },
} as const);

export function audioCueForPresentationEvent(event: PresentationEvent): AudioCueDefinition | null {
  if (event.protocolVersion !== 2) return null;
  const ordinary = CLOCKWORKS_ENEMY_AUDIO_IDS[event.kind];
  const ordinaryId = typeof ordinary === "string" ? ordinary : event.attackKind ? ordinary?.[event.attackKind] : undefined;
  const id = ordinaryId ?? PRESENTATION_AUDIO_IDS[event.kind];
  return id ? AUDIO_CUE_LIBRARY[id] : null;
}

export function ambientAudioCueForWorld(worldId: string): AudioCueDefinition | null {
  if (worldId === "return_station") return AUDIO_CUE_LIBRARY["bgm.return_station"];
  if (worldId === "grey_hive") return AUDIO_CUE_LIBRARY["audio.gh.ambient.facility"];
  return null;
}

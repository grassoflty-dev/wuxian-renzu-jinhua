extends RefCounted
static func cue_for(e: Dictionary) -> String:
 if e.get('kind')=='ActionRejected':
  return 'audio.ui.error'
 var ordinary = CLOCKWORKS_ENEMY_AUDIO_IDS.get(e.get('kind'),'')
 if ordinary is Dictionary:
  return ordinary.get(e.get('attackKind'),'')
 return ordinary if ordinary!='' else PRESENTATION_AUDIO_IDS.get(e.get('kind'),'')
static func duration_ms(id: String) -> float:
 var duration = 0.0
 for t in AUDIO_CUE_LIBRARY.get(id,{}).get('tones',[]):
  duration = maxf(duration,t.offsetMs+t.durationMs)
 return duration
# Original temporary recipes; native synthesis is an approximation.
const temporary_audio = true
const AUDIO_CUE_LIBRARY = {
  "audio.mh.wraith.cast.warning": {
    "peakGain": 0.15,
    "tones": [
      {
        "frequencyHz": 440.0,
        "durationMs": 180.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 880.0
      },
      {
        "frequencyHz": 880.0,
        "durationMs": 120.0,
        "offsetMs": 200.0,
        "waveform": "sine",
        "endFrequencyHz": 1100.0
      }
    ]
  },
  "audio.mh.wraith.cast.impact": {
    "peakGain": 0.14,
    "tones": [
      {
        "frequencyHz": 1100.0,
        "durationMs": 110.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 220.0
      }
    ]
  },
  "audio.mh.wraith.blink": {
    "peakGain": 0.1,
    "tones": [
      {
        "frequencyHz": 620.0,
        "durationMs": 140.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 310.0
      }
    ]
  },
  "audio.ui.confirm": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 620.0,
        "durationMs": 45.0,
        "offsetMs": 0,
        "waveform": "sine"
      },
      {
        "frequencyHz": 830.0,
        "durationMs": 60.0,
        "offsetMs": 46.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.ui.cancel": {
    "peakGain": 0.13,
    "tones": [
      {
        "frequencyHz": 520.0,
        "durationMs": 55.0,
        "offsetMs": 0,
        "waveform": "sine"
      },
      {
        "frequencyHz": 390.0,
        "durationMs": 75.0,
        "offsetMs": 50.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.ui.error": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 310.0,
        "durationMs": 85.0,
        "offsetMs": 0.0,
        "waveform": "triangle"
      },
      {
        "frequencyHz": 230.0,
        "durationMs": 105.0,
        "offsetMs": 82.0,
        "waveform": "triangle"
      }
    ]
  },
  "audio.ui.tab": {
    "peakGain": 0.09,
    "tones": [
      {
        "frequencyHz": 740.0,
        "durationMs": 28.0,
        "offsetMs": 0,
        "waveform": "sine"
      }
    ]
  },
  "audio.player.attack": {
    "peakGain": 0.12,
    "tones": [
      {
        "frequencyHz": 250.0,
        "durationMs": 75.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 180.0
      }
    ]
  },
  "audio.combat.hit": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 560.0,
        "durationMs": 45.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 160.0
      },
      {
        "frequencyHz": 125.0,
        "durationMs": 65.0,
        "offsetMs": 12.0,
        "waveform": "sine",
        "endFrequencyHz": 70.0
      }
    ]
  },
  "audio.combat.hurt": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 210.0,
        "durationMs": 120.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 65.0
      }
    ]
  },
  "audio.combat.absorbed": {
    "peakGain": 0.07,
    "tones": [
      {
        "frequencyHz": 640.0,
        "durationMs": 50.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 460.0
      }
    ]
  },
  "audio.player.dash": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 380.0,
        "durationMs": 90.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 980.0
      }
    ]
  },
  "audio.skill.pulse.cast": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 270.0,
        "durationMs": 100.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 690.0
      },
      {
        "frequencyHz": 690.0,
        "durationMs": 115.0,
        "offsetMs": 76.0,
        "waveform": "sine",
        "endFrequencyHz": 920.0
      }
    ]
  },
  "audio.skill.pulse.hit": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 540.0,
        "durationMs": 65.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 290.0
      },
      {
        "frequencyHz": 370.0,
        "durationMs": 100.0,
        "offsetMs": 48.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.skill.guard.start": {
    "peakGain": 0.15,
    "tones": [
      {
        "frequencyHz": 180.0,
        "durationMs": 95.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 250.0
      },
      {
        "frequencyHz": 250.0,
        "durationMs": 95.0,
        "offsetMs": 74.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.skill.guard.hit": {
    "peakGain": 0.2,
    "tones": [
      {
        "frequencyHz": 135.0,
        "durationMs": 110.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 90.0
      },
      {
        "frequencyHz": 430.0,
        "durationMs": 85.0,
        "offsetMs": 18.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.skill.pierce.cast": {
    "peakGain": 0.14,
    "tones": [
      {
        "frequencyHz": 940.0,
        "durationMs": 75.0,
        "offsetMs": 0.0,
        "waveform": "sawtooth",
        "endFrequencyHz": 1520.0
      }
    ]
  },
  "audio.skill.pierce.hit": {
    "peakGain": 0.18,
    "tones": [
      {
        "frequencyHz": 730.0,
        "durationMs": 65.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 310.0
      },
      {
        "frequencyHz": 155.0,
        "durationMs": 120.0,
        "offsetMs": 30.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.enemy.sentinel.warning.light": {
    "peakGain": 0.13,
    "tones": [
      {
        "frequencyHz": 720.0,
        "durationMs": 92.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 560.0
      }
    ]
  },
  "audio.enemy.sentinel.warning.heavy": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 245.0,
        "durationMs": 140.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 165.0
      },
      {
        "frequencyHz": 520.0,
        "durationMs": 85.0,
        "offsetMs": 42.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.enemy.sentinel.warning.charge": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 180.0,
        "durationMs": 120.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 360.0
      },
      {
        "frequencyHz": 360.0,
        "durationMs": 100.0,
        "offsetMs": 145.0,
        "waveform": "sine",
        "endFrequencyHz": 720.0
      }
    ]
  },
  "audio.gh.ambient.facility": {
    "peakGain": 0.035,
    "tones": [
      {
        "frequencyHz": 55.0,
        "durationMs": 1000.0,
        "offsetMs": 0.0,
        "waveform": "sine"
      },
      {
        "frequencyHz": 73.4,
        "durationMs": 1000.0,
        "offsetMs": 0.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.mh.warden.decoy": {
    "peakGain": 0.13,
    "tones": [
      {
        "frequencyHz": 480.0,
        "durationMs": 100.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 300.0
      },
      {
        "frequencyHz": 300.0,
        "durationMs": 110.0,
        "offsetMs": 100.0,
        "waveform": "sine",
        "endFrequencyHz": 220.0
      }
    ]
  },
  "audio.mh.warden.strike.warning": {
    "peakGain": 0.18,
    "tones": [
      {
        "frequencyHz": 260.0,
        "durationMs": 160.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 520.0
      },
      {
        "frequencyHz": 520.0,
        "durationMs": 160.0,
        "offsetMs": 170.0,
        "waveform": "sine",
        "endFrequencyHz": 780.0
      }
    ]
  },
  "audio.mh.warden.pulse.warning": {
    "peakGain": 0.18,
    "tones": [
      {
        "frequencyHz": 180.0,
        "durationMs": 130.0,
        "offsetMs": 0.0,
        "waveform": "triangle"
      },
      {
        "frequencyHz": 240.0,
        "durationMs": 130.0,
        "offsetMs": 170.0,
        "waveform": "triangle"
      },
      {
        "frequencyHz": 320.0,
        "durationMs": 160.0,
        "offsetMs": 340.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.mh.warden.strike.impact": {
    "peakGain": 0.2,
    "tones": [
      {
        "frequencyHz": 700.0,
        "durationMs": 90.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 100.0
      }
    ]
  },
  "audio.mh.warden.pulse.impact": {
    "peakGain": 0.2,
    "tones": [
      {
        "frequencyHz": 120.0,
        "durationMs": 220.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 60.0
      },
      {
        "frequencyHz": 380.0,
        "durationMs": 150.0,
        "offsetMs": 40.0,
        "waveform": "sine",
        "endFrequencyHz": 90.0
      }
    ]
  },
  "audio.mh.warden.death": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 500.0,
        "durationMs": 200.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 300.0
      },
      {
        "frequencyHz": 300.0,
        "durationMs": 220.0,
        "offsetMs": 200.0,
        "waveform": "sine",
        "endFrequencyHz": 100.0
      }
    ]
  },
  "audio.cw.enemy.drone.warning": {
    "peakGain": 0.14,
    "tones": [
      {
        "frequencyHz": 880.0,
        "durationMs": 100.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 1320.0
      },
      {
        "frequencyHz": 1100.0,
        "durationMs": 80.0,
        "offsetMs": 115.0,
        "waveform": "sine"
      }
    ]
  },
  "audio.cw.enemy.hound.bite.warning": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 180.0,
        "durationMs": 150.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 260.0
      }
    ]
  },
  "audio.cw.enemy.hound.leap.warning": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 170.0,
        "durationMs": 100.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 300.0
      },
      {
        "frequencyHz": 240.0,
        "durationMs": 130.0,
        "offsetMs": 120.0,
        "waveform": "triangle",
        "endFrequencyHz": 420.0
      }
    ]
  },
  "audio.cw.enemy.drone.impact": {
    "peakGain": 0.13,
    "tones": [
      {
        "frequencyHz": 1300.0,
        "durationMs": 90.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 440.0
      }
    ]
  },
  "audio.cw.enemy.hound.bite.impact": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 240.0,
        "durationMs": 85.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 90.0
      }
    ]
  },
  "audio.cw.enemy.hound.leap.impact": {
    "peakGain": 0.18,
    "tones": [
      {
        "frequencyHz": 140.0,
        "durationMs": 145.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 65.0
      },
      {
        "frequencyHz": 300.0,
        "durationMs": 70.0,
        "offsetMs": 20.0,
        "waveform": "sine",
        "endFrequencyHz": 120.0
      }
    ]
  },
  "audio.cw.enemy.alert": {
    "peakGain": 0.1,
    "tones": [
      {
        "frequencyHz": 620.0,
        "durationMs": 75.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 820.0
      }
    ]
  },
  "audio.cw.enemy.stagger": {
    "peakGain": 0.12,
    "tones": [
      {
        "frequencyHz": 400.0,
        "durationMs": 90.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 150.0
      }
    ]
  },
  "audio.cw.enemy.death": {
    "peakGain": 0.12,
    "tones": [
      {
        "frequencyHz": 280.0,
        "durationMs": 180.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 70.0
      }
    ]
  },
  "audio.gh.brute.charge.warning": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 105.0,
        "durationMs": 150.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 190.0
      },
      {
        "frequencyHz": 180.0,
        "durationMs": 180.0,
        "offsetMs": 170.0,
        "waveform": "triangle",
        "endFrequencyHz": 280.0
      }
    ]
  },
  "audio.gh.brute.slam.warning": {
    "peakGain": 0.18,
    "tones": [
      {
        "frequencyHz": 160.0,
        "durationMs": 160.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 95.0
      },
      {
        "frequencyHz": 310.0,
        "durationMs": 100.0,
        "offsetMs": 100.0,
        "waveform": "sine",
        "endFrequencyHz": 180.0
      }
    ]
  },
  "audio.gh.brute.charge.impact": {
    "peakGain": 0.19,
    "tones": [
      {
        "frequencyHz": 140.0,
        "durationMs": 140.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 55.0
      }
    ]
  },
  "audio.mh.tidebound.swing.warning": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 240.0,
        "durationMs": 180.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 390.0
      },
      {
        "frequencyHz": 155.0,
        "durationMs": 140.0,
        "offsetMs": 120.0,
        "waveform": "triangle",
        "endFrequencyHz": 110.0
      }
    ]
  },
  "audio.mh.tidebound.charge.warning": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 135.0,
        "durationMs": 170.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 270.0
      },
      {
        "frequencyHz": 270.0,
        "durationMs": 160.0,
        "offsetMs": 160.0,
        "waveform": "sine",
        "endFrequencyHz": 410.0
      }
    ]
  },
  "audio.mh.tidebound.swing.impact": {
    "peakGain": 0.17,
    "tones": [
      {
        "frequencyHz": 220.0,
        "durationMs": 120.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 65.0
      }
    ]
  },
  "audio.mh.tidebound.charge.impact": {
    "peakGain": 0.18,
    "tones": [
      {
        "frequencyHz": 170.0,
        "durationMs": 170.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 50.0
      },
      {
        "frequencyHz": 410.0,
        "durationMs": 70.0,
        "offsetMs": 0.0,
        "waveform": "sine",
        "endFrequencyHz": 120.0
      }
    ]
  },
  "audio.gh.swarm.lunge.warning": {
    "peakGain": 0.15,
    "tones": [
      {
        "frequencyHz": 520.0,
        "durationMs": 110.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 260.0
      },
      {
        "frequencyHz": 690.0,
        "durationMs": 130.0,
        "offsetMs": 110.0,
        "waveform": "triangle",
        "endFrequencyHz": 340.0
      }
    ]
  },
  "audio.gh.swarm.lunge.impact": {
    "peakGain": 0.16,
    "tones": [
      {
        "frequencyHz": 180.0,
        "durationMs": 90.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 75.0
      }
    ]
  },
  "audio.gh.swarm.member.hit": {
    "peakGain": 0.08,
    "tones": [
      {
        "frequencyHz": 740.0,
        "durationMs": 55.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 420.0
      }
    ]
  },
  "audio.gh.swarm.member.disperse": {
    "peakGain": 0.1,
    "tones": [
      {
        "frequencyHz": 850.0,
        "durationMs": 130.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 140.0
      }
    ]
  },
  "audio.gh.brute.slam.impact": {
    "peakGain": 0.2,
    "tones": [
      {
        "frequencyHz": 115.0,
        "durationMs": 200.0,
        "offsetMs": 0.0,
        "waveform": "triangle",
        "endFrequencyHz": 45.0
      },
      {
        "frequencyHz": 380.0,
        "durationMs": 70.0,
        "offsetMs": 10.0,
        "waveform": "sine",
        "endFrequencyHz": 90.0
      }
    ]
  },
  "bgm.return_station": {
    "peakGain": 0.025,
    "tones": [
      {
        "frequencyHz": 110.0,
        "durationMs": 1000.0,
        "offsetMs": 0.0,
        "waveform": "sine"
      },
      {
        "frequencyHz": 164.8,
        "durationMs": 1000.0,
        "offsetMs": 0.0,
        "waveform": "sine"
      }
    ]
  }
}
const PRESENTATION_AUDIO_IDS = {
  "ResonanceWardenDecoy": "audio.mh.warden.decoy",
  "ResonanceWardenStrikeWindup": "audio.mh.warden.strike.warning",
  "ResonanceWardenPulseWindup": "audio.mh.warden.pulse.warning",
  "ResonanceWardenStrikeImpact": "audio.mh.warden.strike.impact",
  "ResonanceWardenPulseImpact": "audio.mh.warden.pulse.impact",
  "ResonanceWardenDeath": "audio.mh.warden.death",
  "AttackStarted": "audio.player.attack",
  "Hit": "audio.combat.hit",
  "Damaged": "audio.combat.hurt",
  "DamageAbsorbed": "audio.combat.absorbed",
  "DashStarted": "audio.player.dash",
  "PulseCast": "audio.skill.pulse.cast",
  "PulseHit": "audio.skill.pulse.hit",
  "GuardStarted": "audio.skill.guard.start",
  "GuardImpact": "audio.skill.guard.hit",
  "PierceStarted": "audio.skill.pierce.cast",
  "PierceHit": "audio.skill.pierce.hit",
  "SentinelAttackWindup": "audio.enemy.sentinel.warning.light",
  "SentinelHeavyWindup": "audio.enemy.sentinel.warning.heavy",
  "SentinelChargeWindup": "audio.enemy.sentinel.warning.charge",
}
const CLOCKWORKS_ENEMY_AUDIO_IDS = {
  "SignalBlink": "audio.mh.wraith.blink",
  "EnemyAlert": "audio.cw.enemy.alert", "EnemyStagger": "audio.cw.enemy.stagger", "EnemyDeath": "audio.cw.enemy.death",
  "EnemyMemberHit": "audio.gh.swarm.member.hit",
  "EnemyMemberDisperse": "audio.gh.swarm.member.disperse",
  "EnemyAttackTelegraph": { "signal_shot": "audio.mh.wraith.cast.warning", "pressure_shot": "audio.cw.enemy.drone.warning", "bite": "audio.cw.enemy.hound.bite.warning", "leap": "audio.cw.enemy.hound.leap.warning", "charge": "audio.gh.brute.charge.warning", "slam": "audio.gh.brute.slam.warning", "lunge": "audio.gh.swarm.lunge.warning", "tide_swing": "audio.mh.tidebound.swing.warning", "tide_charge": "audio.mh.tidebound.charge.warning" },
  "EnemyAttackImpact": { "signal_shot": "audio.mh.wraith.cast.impact", "pressure_shot": "audio.cw.enemy.drone.impact", "bite": "audio.cw.enemy.hound.bite.impact", "leap": "audio.cw.enemy.hound.leap.impact", "charge": "audio.gh.brute.charge.impact", "slam": "audio.gh.brute.slam.impact", "lunge": "audio.gh.swarm.lunge.impact", "tide_swing": "audio.mh.tidebound.swing.impact", "tide_charge": "audio.mh.tidebound.charge.impact" },
}

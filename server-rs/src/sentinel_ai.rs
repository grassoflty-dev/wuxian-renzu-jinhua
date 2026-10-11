use crate::{
    continuous_combat::{horizontal_distance, CombatEvent},
    world_v3::Vec3,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SentinelState {
    Chase,
    Attack,
    HeavyAttack,
    Recover,
    Hit,
    Death,
}

#[derive(Clone, Debug)]
pub struct Sentinel {
    pub entity_id: String,
    pub position_m: Vec3,
    pub hp: u32,
    pub state: SentinelState,
    pub state_remaining_ms: u64,
    pub attack_serial: u64,
    pub active: bool,
}
impl Sentinel {
    pub fn new(id: impl Into<String>, position_m: Vec3, hp: u32) -> Self {
        Self {
            entity_id: id.into(),
            position_m,
            hp,
            state: SentinelState::Chase,
            state_remaining_ms: 0,
            attack_serial: 0,
            active: true,
        }
    }
    pub fn take_damage(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
        if self.hp == 0 {
            self.state = SentinelState::Death;
            self.active = false;
            self.state_remaining_ms = 0;
        } else {
            self.state = SentinelState::Hit;
            self.state_remaining_ms = 120;
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum SentinelEvent {
    StateChanged {
        sentinel_id: String,
        state: SentinelState,
    },
    Moved {
        sentinel_id: String,
        position_m: Vec3,
    },
    DamagedPlayer {
        sentinel_id: String,
        damage: u32,
        attack_serial: u64,
    },
    Activated {
        sentinel_id: String,
    },
    Deactivated {
        sentinel_id: String,
    },
    Queried {
        sentinel_id: String,
        state: SentinelState,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SentinelCommand {
    Query,
    Activate { sentinel_id: String },
    Deactivate { sentinel_id: String },
}

pub fn tick_sentinel(
    s: &mut Sentinel,
    player: Vec3,
    dt_s: f32,
) -> (Vec<SentinelEvent>, Vec<CombatEvent>) {
    if !s.active || s.state == SentinelState::Death {
        return (vec![], vec![]);
    }
    let dt_ms = (dt_s * 1000.0).round() as u64;
    let mut events = vec![];
    let mut combat = vec![];
    if s.state_remaining_ms > 0 {
        s.state_remaining_ms = s.state_remaining_ms.saturating_sub(dt_ms);
        if s.state_remaining_ms > 0 {
            return (events, combat);
        }
    }
    let d = horizontal_distance(s.position_m, player);
    match s.state {
        SentinelState::Chase if d > 1.6 => {
            let dx = (player.x_m - s.position_m.x_m) / d;
            let dz = (player.z_m - s.position_m.z_m) / d;
            s.position_m.x_m += dx * 2.2 * dt_s;
            s.position_m.z_m += dz * 2.2 * dt_s;
            events.push(SentinelEvent::Moved {
                sentinel_id: s.entity_id.clone(),
                position_m: s.position_m,
            });
        }
        SentinelState::Chase => {
            s.state = SentinelState::Attack;
            s.state_remaining_ms = 250;
            events.push(SentinelEvent::StateChanged {
                sentinel_id: s.entity_id.clone(),
                state: s.state,
            });
        }
        SentinelState::Attack => {
            s.attack_serial += 1;
            if d <= 1.8 {
                combat.push(CombatEvent::PlayerDamaged {
                    source_id: s.entity_id.clone(),
                    damage: 8,
                });
                events.push(SentinelEvent::DamagedPlayer {
                    sentinel_id: s.entity_id.clone(),
                    damage: 8,
                    attack_serial: s.attack_serial,
                });
            }
            s.state = if s.attack_serial % 3 == 0 {
                SentinelState::HeavyAttack
            } else {
                SentinelState::Recover
            };
            s.state_remaining_ms = if s.state == SentinelState::HeavyAttack {
                400
            } else {
                300
            };
            events.push(SentinelEvent::StateChanged {
                sentinel_id: s.entity_id.clone(),
                state: s.state,
            });
        }
        SentinelState::HeavyAttack => {
            s.attack_serial += 1;
            if d <= 2.2 {
                combat.push(CombatEvent::PlayerDamaged {
                    source_id: s.entity_id.clone(),
                    damage: 16,
                });
                events.push(SentinelEvent::DamagedPlayer {
                    sentinel_id: s.entity_id.clone(),
                    damage: 16,
                    attack_serial: s.attack_serial,
                });
            }
            s.state = SentinelState::Recover;
            s.state_remaining_ms = 500;
            events.push(SentinelEvent::StateChanged {
                sentinel_id: s.entity_id.clone(),
                state: s.state,
            });
        }
        SentinelState::Recover | SentinelState::Hit => {
            s.state = SentinelState::Chase;
            events.push(SentinelEvent::StateChanged {
                sentinel_id: s.entity_id.clone(),
                state: s.state,
            });
        }
        SentinelState::Death => {}
    }
    (events, combat)
}

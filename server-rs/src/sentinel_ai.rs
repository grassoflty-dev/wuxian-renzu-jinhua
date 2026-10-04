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
    ChargeWindup,
    Charge,
    Stagger,
    Recover,
    Hit,
    Death,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sentinel {
    pub entity_id: String,
    pub position_m: Vec3,
    pub hp: u32,
    pub state: SentinelState,
    pub state_remaining_ms: u64,
    pub attack_serial: u64,
    pub active: bool,
    pub charge_controller: Option<ChargeController>,
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
            charge_controller: None,
        }
    }
    pub fn take_damage(&mut self, amount: u32) {
        self.take_damage_with_stagger(amount, 120);
    }

    pub fn take_damage_with_stagger(&mut self, amount: u32, stagger_ms: u64) {
        if !self.validate() || !self.active || self.hp == 0 { return; }
        self.cancel_charge();
        self.hp = self.hp.saturating_sub(amount);
        if self.hp == 0 {
            self.state = SentinelState::Death;
            self.active = false;
            self.state_remaining_ms = 0;
        } else {
            self.state = if self.charge_controller.is_some() && stagger_ms > 120 { SentinelState::Stagger } else { SentinelState::Hit };
            self.state_remaining_ms = stagger_ms;
        }
    }

    pub fn apply_stagger(&mut self, duration_ms: u64) {
        if !self.validate() || !self.active || self.state == SentinelState::Death || duration_ms == 0 {
            return;
        }
        self.cancel_charge();
        self.state = if self.charge_controller.is_some() { SentinelState::Stagger } else { SentinelState::Hit };
        self.state_remaining_ms = self.state_remaining_ms.max(duration_ms);
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
    kcc: &crate::continuous_kcc::StaticKccWorld,
) -> (Vec<SentinelEvent>, Vec<CombatEvent>) {
    if !s.validate() || !finite(player) || !dt_s.is_finite() || dt_s <= 0.0 || (s.charge_controller.is_some() && dt_s > 0.25)
        || !s.active || s.state == SentinelState::Death
        || !kcc.stable_actor_footprint(s.position_m,0.3) {
        return (vec![], vec![]);
    }
    // Do not release an attack that cannot produce a JS-safe current receipt.
    if s.charge_controller.is_some() && s.attack_serial >= 9_007_199_254_740_991
        && matches!(s.state, SentinelState::Attack | SentinelState::HeavyAttack) {return (vec![],vec![]);}
    let dt_ms = (dt_s * 1000.0).round() as u64;
    if let Some(output) = tick_charge(s, player, dt_ms, kcc) { return output; }
    let mut events = vec![];
    let mut combat = vec![];
    if s.state_remaining_ms > 0 {
        s.state_remaining_ms = s.state_remaining_ms.saturating_sub(dt_ms);
        if s.state_remaining_ms > 0 {
            return (events, combat);
        }
    }
    let d = horizontal_distance(s.position_m, player);
    let same_height = crate::continuous_combat::combat_vertical_overlap(s.position_m, player);
    match s.state {
        SentinelState::Chase if !same_height => {}
        SentinelState::Chase if d > 1.6 => {
            let dx = (player.x_m - s.position_m.x_m) / d;
            let dz = (player.z_m - s.position_m.z_m) / d;
            let candidate=Vec3{x_m:s.position_m.x_m+dx*2.2*dt_s,z_m:s.position_m.z_m+dz*2.2*dt_s,..s.position_m};
            if kcc.stable_actor_footprint(candidate,0.3) {s.position_m=candidate;}
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
            let Some(serial) = s.attack_serial.checked_add(1) else { return (events, combat) };
            s.attack_serial = serial;
            if same_height && d <= 1.8 {
                combat.push(CombatEvent::PlayerDamaged {
                    source_id: s.entity_id.clone(),
                    damage: 8,
                    contact: Some(crate::continuous_combat::CombatContact::new(s.position_m, player, None)),
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
            let Some(serial) = s.attack_serial.checked_add(1) else { return (events, combat) };
            s.attack_serial = serial;
            if same_height && d <= 2.2 {
                combat.push(CombatEvent::PlayerDamaged {
                    source_id: s.entity_id.clone(),
                    damage: 16,
                    contact: Some(crate::continuous_combat::CombatContact::new(s.position_m, player, None)),
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
        SentinelState::Recover | SentinelState::Hit | SentinelState::Stagger => {
            s.state = SentinelState::Chase;
            events.push(SentinelEvent::StateChanged {
                sentinel_id: s.entity_id.clone(),
                state: s.state,
            });
        }
        SentinelState::Death | SentinelState::ChargeWindup | SentinelState::Charge => {}
    }
    (events, combat)
}

mod charge;
pub use charge::{ChargeController, ChargeAttack, SentinelEncounterView, SentinelWarningView};
use charge::{finite, tick_charge};
#[cfg(test)] mod charge_tests;

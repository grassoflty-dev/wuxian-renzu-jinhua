use crate::world_v3::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombatIntent {
    Attack { request_id: u64 },
    Jump { request_id: u64 },
    Dash { request_id: u64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CombatEvent {
    AttackHit {
        request_id: u64,
        target_id: String,
        damage: u32,
    },
    AttackMiss {
        request_id: u64,
    },
    IntentRejected {
        request_id: u64,
        reason: String,
    },
    PlayerDamaged {
        source_id: String,
        damage: u32,
    },
}

#[derive(Clone, Debug, Default)]
pub struct CombatState {
    handled: BTreeSet<u64>,
}
impl CombatState {
    pub fn claim(&mut self, request_id: u64) -> bool {
        self.handled.insert(request_id)
    }

    /// Stable persistence boundary for request de-duplication.
    pub fn handled_request_ids(&self) -> Vec<u64> {
        self.handled.iter().copied().collect()
    }

    /// Rebuild a de-duplication ledger from a validated save payload.
    pub fn from_handled_request_ids(
        ids: impl IntoIterator<Item = u64>,
    ) -> Result<Self, &'static str> {
        let mut handled = BTreeSet::new();
        for id in ids {
            if !handled.insert(id) {
                return Err("duplicate combat request id");
            }
        }
        Ok(Self { handled })
    }
}

pub fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    ((a.x_m - b.x_m).powi(2) + (a.z_m - b.z_m).powi(2)).sqrt()
}

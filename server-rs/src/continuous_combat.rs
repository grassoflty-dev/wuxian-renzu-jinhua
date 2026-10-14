use crate::world_v3::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const COMBAT_V1_DATA: &str = include_str!("../data/combat_v1.json");

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatV1Data {
    pub schema_version: u32,
    pub stagger_ms_per_point: u64,
    pub invulnerability_ms: u64,
    pub guard_damage_reduction: f32,
    pub actions: BTreeMap<String, CombatActionData>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatActionData {
    pub energy: u32,
    pub cooldown_ms: u64,
    pub windup_ms: u64,
    pub active_ms: u64,
    pub recovery_ms: u64,
    pub range_m: f32,
    pub shape: String,
    pub min_facing_dot: Option<f32>,
    pub line_half_width_m: Option<f32>,
    pub damage: u32,
    pub stagger: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraversalMarker {
    #[serde(default)]
    pub from_height_range_m: Option<crate::moving_support::HeightRange>,
    pub id: String,
    pub from: Vec3,
    pub to: Vec3,
    pub range_m: f32,
    pub cooldown_ms: u64,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveTraversal {
    pub marker_index: usize,
    pub request_id: u64,
    pub start_position: Vec3,
    pub elapsed_ms: u64,
}

pub fn combat_v1() -> &'static CombatV1Data {
    static DATA: OnceLock<CombatV1Data> = OnceLock::new();
    DATA.get_or_init(|| {
        let data: CombatV1Data = serde_json::from_str(COMBAT_V1_DATA)
            .expect("combat_v1.json is validated static combat tuning");
        assert_eq!(data.schema_version, 1, "combat tuning schema mismatch");
        for name in ["primaryAttack", "dash", "pulse", "guard", "pierce"] {
            assert!(
                data.actions.contains_key(name),
                "missing combat action {name}"
            );
        }
        data
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatIntent {
    Attack {
        request_id: u64,
    },
    Jump {
        request_id: u64,
    },
    Dash {
        request_id: u64,
    },
    Pulse {
        request_id: u64,
    },
    GuardStart {
        request_id: u64,
    },
    GuardEnd {
        request_id: u64,
    },
    Pierce {
        request_id: u64,
    },
    ActionAttack {
        request_id: u64,
    },
    ActionDash {
        request_id: u64,
    },
    ContextTraversal {
        request_id: u64,
        marker_index: usize,
    },
}
impl CombatIntent {
    pub fn request_id(self) -> u64 {
        match self {
            Self::Attack { request_id }
            | Self::Jump { request_id }
            | Self::Dash { request_id }
            | Self::Pulse { request_id }
            | Self::GuardStart { request_id }
            | Self::GuardEnd { request_id }
            | Self::Pierce { request_id }
            | Self::ActionAttack { request_id }
            | Self::ActionDash { request_id }
            | Self::ContextTraversal { request_id, .. } => request_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CombatActionKind {
    PrimaryAttack,
    Dash,
    Pulse,
    Guard,
    Pierce,
}

impl CombatActionKind {
    pub fn data_key(self) -> &'static str {
        match self {
            Self::PrimaryAttack => "primaryAttack",
            Self::Dash => "dash",
            Self::Pulse => "pulse",
            Self::Guard => "guard",
            Self::Pierce => "pierce",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActiveCombatAction {
    pub kind: CombatActionKind,
    pub request_id: u64,
    pub elapsed_ms: u64,
    pub impact_resolved: bool,
    pub end_requested: bool,
}

/// Contact captured when the owner resolves combat, before death/movement changes actors.
/// This internal record is projected into amount-free presentation metadata.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatContact {
    pub position_m: Vec3,
    pub direction_rad: f32,
    pub request_id: Option<u64>,
}
impl CombatContact {
    pub fn new(origin: Vec3, target: Vec3, request_id: Option<u64>) -> Self {
        Self {
            position_m: target,
            direction_rad: (target.x_m - origin.x_m).atan2(target.z_m - origin.z_m),
            request_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CombatEvent {
    AttackHit {
        request_id: u64,
        target_id: String,
        damage: u32,
        #[serde(default)]
        modern: bool,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    AttackMiss {
        request_id: u64,
        #[serde(default)]
        modern: bool,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    ActionMiss {
        action: CombatActionKind,
        request_id: u64,
        contact: CombatContact,
    },
    IntentRejected {
        request_id: u64,
        reason: String,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    PlayerDamaged {
        source_id: String,
        damage: u32,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    ActionStarted {
        action: CombatActionKind,
        request_id: u64,
    },
    ActionImpact {
        action: CombatActionKind,
        target_id: String,
        damage: u32,
        stagger: u32,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    GuardImpact {
        source_id: String,
        stagger: u32,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    GuardStarted {
        request_id: u64,
    },
    GuardEnded {
        request_id: u64,
    },
    DamageAbsorbed {
        source_id: String,
        #[serde(default)]
        reason: String,
        #[serde(default)]
        contact: Option<CombatContact>,
    },
    TraversalStarted {
        request_id: u64,
        marker_id: String,
    },
    TraversalCompleted {
        request_id: u64,
        marker_id: String,
    },
    TraversalBlocked {
        request_id: u64,
        marker_id: String,
    },
}

impl CombatEvent {
    /// Pulse/Pierce miss reporting is presentation-only; adding it must not
    /// restart regeneration's existing combat delay.
    pub fn counts_as_combat_activity(&self) -> bool {
        !matches!(self, Self::ActionMiss { .. })
    }
}

#[derive(Clone, Debug)]
pub struct CombatState {
    handled: BTreeSet<u64>,
    last_request_id: u64,
    pub cooldowns_ms: BTreeMap<CombatActionKind, u64>,
    pub active_action: Option<ActiveCombatAction>,
    pub facing_x: f32,
    pub facing_z: f32,
    pub dash_x: f32,
    pub dash_z: f32,
    pub action_locks_facing: bool,
    pub invulnerability_remaining_ms: u64,
    pub qer_v1_active: bool,
    pub traversal_markers: Vec<TraversalMarker>,
    pub active_traversal: Option<ActiveTraversal>,
    pub traversal_cooldowns_ms: BTreeMap<String, u64>,
}
impl Default for CombatState {
    fn default() -> Self {
        Self {
            handled: BTreeSet::new(),
            last_request_id: 0,
            cooldowns_ms: BTreeMap::new(),
            active_action: None,
            facing_x: 0.0,
            facing_z: 1.0,
            dash_x: 0.0,
            dash_z: 1.0,
            action_locks_facing: false,
            invulnerability_remaining_ms: 0,
            qer_v1_active: false,
            traversal_markers: vec![],
            active_traversal: None,
            traversal_cooldowns_ms: BTreeMap::new(),
        }
    }
}
impl CombatState {
    pub fn claim(&mut self, request_id: u64) -> bool {
        if request_id == 0 || request_id <= self.last_request_id || !self.handled.insert(request_id)
        {
            return false;
        }
        self.last_request_id = request_id;
        true
    }

    pub fn tick_timers(&mut self, dt_ms: u64) {
        for remaining in self.cooldowns_ms.values_mut() {
            *remaining = remaining.saturating_sub(dt_ms);
        }
        self.invulnerability_remaining_ms = self.invulnerability_remaining_ms.saturating_sub(dt_ms);
        for remaining in self.traversal_cooldowns_ms.values_mut() {
            *remaining = remaining.saturating_sub(dt_ms);
        }
    }

    pub fn guard_active(&self, windup_ms: u64, active_ms: u64) -> bool {
        self.active_action.as_ref().is_some_and(|action| {
            action.kind == CombatActionKind::Guard
                && !action.end_requested
                && action.elapsed_ms >= windup_ms
                && action.elapsed_ms < windup_ms.saturating_add(active_ms)
        })
    }

    pub fn action_state(&self) -> &'static str {
        self.active_action
            .as_ref()
            .map_or("idle", |action| match action.kind {
                CombatActionKind::PrimaryAttack => "primaryAttack",
                CombatActionKind::Dash => "dash",
                CombatActionKind::Pulse => "pulse",
                CombatActionKind::Guard => "guard",
                CombatActionKind::Pierce => "pierce",
            })
    }

    /// Whole-action owner time, independent of renderer frames or wall clock.
    /// Releasing Guard changes its phase but preserves the original lock duration.
    pub fn action_presentation(&self) -> Option<crate::world_v3::ActionPresentation> {
        let active = self.active_action.as_ref()?;
        let spec = &combat_v1().actions[active.kind.data_key()];
        let active_end = spec.windup_ms.saturating_add(spec.active_ms);
        let duration_ms = active_end.saturating_add(spec.recovery_ms);
        if active.request_id == 0 || active.request_id > 9_007_199_254_740_991
            || active.elapsed_ms >= duration_ms {
            return None;
        }
        let phase = if active.kind == CombatActionKind::Guard && active.end_requested {
            crate::world_v3::ActionPhase::Recovery
        } else if active.elapsed_ms < spec.windup_ms {
            crate::world_v3::ActionPhase::Windup
        } else if active.elapsed_ms < active_end {
            crate::world_v3::ActionPhase::Active
        } else {
            crate::world_v3::ActionPhase::Recovery
        };
        Some(crate::world_v3::ActionPresentation {
            request_id: active.request_id, phase, elapsed_ms: active.elapsed_ms, duration_ms,
            range_m: spec.range_m, line_half_width_m: spec.line_half_width_m.unwrap_or(0.0),
        })
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
        let last_request_id = handled.iter().next_back().copied().unwrap_or(0);
        Ok(Self {
            handled,
            last_request_id,
            ..Self::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CombatState;

    #[test]
    fn request_ledger_rejects_duplicates_and_out_of_order_ids() {
        let mut state = CombatState::default();
        assert!(state.claim(12));
        assert!(!state.claim(12));
        assert!(!state.claim(11));
        assert!(state.claim(13));
    }
}

pub fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    ((a.x_m - b.x_m).powi(2) + (a.z_m - b.z_m).powi(2)).sqrt()
}

/// DEVELOPMENT TUNING: this 2.5D combat model permits at most one metre of
/// footpoint separation. Both attack directions use the same inclusive reach;
/// ordinary floor jumps remain eligible, while a 2m standing deck separates
/// combat. This is not a renderer-derived capsule or full 3D free-flight model.
pub const COMBAT_VERTICAL_REACH_M: f32 = 1.0;
pub fn combat_vertical_overlap(origin: Vec3, target: Vec3) -> bool {
    [origin.x_m, origin.y_m, origin.z_m, target.x_m, target.y_m, target.z_m]
        .iter().all(|value| value.is_finite())
        && (origin.y_m - target.y_m).abs() <= COMBAT_VERTICAL_REACH_M
}

/// Default footpoint band for existing floor-authored environmental effects.
/// Explicit elevated hazard bands are authored independently in SceneDefinition.
pub fn floor_height_overlap(height: f32) -> bool {
    height.is_finite() && (0.0..=COMBAT_VERTICAL_REACH_M).contains(&height)
}

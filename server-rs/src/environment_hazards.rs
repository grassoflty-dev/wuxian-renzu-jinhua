//! Generic simulation-clock environmental cycles. No world, scene, item or lineage IDs.
use crate::effects::HazardTag;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExposureConfig {
    pub max_units: u32,
    pub gain_per_second: u32,
    pub loss_per_second: u32,
    pub damage_threshold: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentHazardConfig {
    pub tag: HazardTag,
    pub warning_ms: u64,
    pub active_ms: u64,
    pub recovery_ms: u64,
    pub damage: u32,
    pub damage_interval_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure: Option<ExposureConfig>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentControlConfig {
    pub target_hazard_ids: Vec<String>,
    pub suppression_ms: u64,
    pub exposure_reduction_units: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentPhase {
    Warning,
    Active,
    Recovery,
    Suppressed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnvironmentFrame {
    pub phase: EnvironmentPhase,
    pub remaining_ms: u64,
    pub exposure_bps: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentHazardState {
    pub cycle_started_at_ms: u64,
    pub last_advanced_at_ms: u64,
    pub next_damage_at_ms: u64,
    pub suppressed_until_ms: u64,
    /// Integer fixed-point units avoid frame-rate-dependent exposure rounding.
    pub exposure_milliunits: u64,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentPersistentState {
    #[serde(default)]
    pub hazards: BTreeMap<String, EnvironmentHazardState>,
    #[serde(default)]
    pub control_cooldown_until_ms: BTreeMap<String, u64>,
}
impl EnvironmentPersistentState {
    pub fn is_default(&self) -> bool {
        self.hazards.is_empty() && self.control_cooldown_until_ms.is_empty()
    }
}
impl EnvironmentHazardConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(
            self.tag,
            HazardTag::Heat | HazardTag::Pressure | HazardTag::Toxin
        ) || !(1..=10_000).contains(&self.warning_ms)
            || !(100..=60_000).contains(&self.active_ms)
            || !(100..=60_000).contains(&self.recovery_ms)
            || !(1..=10_000).contains(&self.damage)
            || !(100..=60_000).contains(&self.damage_interval_ms)
        {
            return Err("E_ENV_HAZARD_CONFIG");
        }
        if let Some(e) = &self.exposure {
            if self.tag != HazardTag::Heat
                || !(1..=10_000).contains(&e.max_units)
                || !(1..=10_000).contains(&e.gain_per_second)
                || !(1..=10_000).contains(&e.loss_per_second)
                || e.damage_threshold == 0
                || e.damage_threshold > e.max_units
            {
                return Err("E_ENV_EXPOSURE_CONFIG");
            }
        }
        Ok(())
    }
    fn cycle_ms(&self) -> u64 {
        self.warning_ms + self.active_ms + self.recovery_ms
    }
}
impl EnvironmentControlConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        let unique: std::collections::BTreeSet<_> = self.target_hazard_ids.iter().collect();
        if self.target_hazard_ids.is_empty()
            || self.target_hazard_ids.len() > 16
            || unique.len() != self.target_hazard_ids.len()
            || self
                .target_hazard_ids
                .iter()
                .any(|id| !crate::scene_registry::valid_id(id))
            || !(100..=60_000).contains(&self.suppression_ms)
            || self.exposure_reduction_units > 10_000
        {
            return Err("E_ENV_CONTROL_CONFIG");
        }
        Ok(())
    }
}
impl EnvironmentHazardState {
    pub fn new(now: u64) -> Self {
        Self {
            cycle_started_at_ms: now,
            last_advanced_at_ms: now,
            next_damage_at_ms: now,
            suppressed_until_ms: 0,
            exposure_milliunits: 0,
        }
    }
    pub fn validate_at(
        &self,
        config: &EnvironmentHazardConfig,
        now: u64,
    ) -> Result<(), &'static str> {
        config.validate()?;
        if self.last_advanced_at_ms > now
            || self.cycle_started_at_ms > self.suppressed_until_ms.max(self.last_advanced_at_ms)
            || self.cycle_started_at_ms > self.suppressed_until_ms.max(now)
            || (self.suppressed_until_ms != 0
                && self.cycle_started_at_ms != self.suppressed_until_ms)
            || self.suppressed_until_ms.saturating_sub(now) > 60_000
            || self.next_damage_at_ms.saturating_sub(now) > config.damage_interval_ms
            || self.exposure_milliunits
                > config
                    .exposure
                    .as_ref()
                    .map_or(0, |e| u64::from(e.max_units) * 1000)
        {
            return Err("E_ENV_HAZARD_STATE");
        }
        Ok(())
    }
    pub fn frame(
        &self,
        config: &EnvironmentHazardConfig,
        now: u64,
    ) -> Result<EnvironmentFrame, &'static str> {
        self.validate_at(config, now)?;
        let exposure_bps = config.exposure.as_ref().map_or(0, |e| {
            (self.exposure_milliunits * 10 / u64::from(e.max_units)) as u32
        });
        if now < self.suppressed_until_ms {
            return Ok(EnvironmentFrame {
                phase: EnvironmentPhase::Suppressed,
                remaining_ms: self.suppressed_until_ms - now,
                exposure_bps,
            });
        }
        let t = now.saturating_sub(self.cycle_started_at_ms) % config.cycle_ms();
        let (phase, remaining_ms) = if t < config.warning_ms {
            (EnvironmentPhase::Warning, config.warning_ms - t)
        } else if t < config.warning_ms + config.active_ms {
            (
                EnvironmentPhase::Active,
                config.warning_ms + config.active_ms - t,
            )
        } else {
            (EnvironmentPhase::Recovery, config.cycle_ms() - t)
        };
        Ok(EnvironmentFrame {
            phase,
            remaining_ms,
            exposure_bps,
        })
    }
    /// One authoritative simulation step, with at most one hit and no overdue damage burst.
    /// Invalid clocks/arithmetic leave this state unchanged.
    pub fn advance(
        &mut self,
        config: &EnvironmentHazardConfig,
        now: u64,
        inside: bool,
    ) -> Result<bool, &'static str> {
        self.validate_at(config, now)?;
        let mut next = self.clone();
        let elapsed = now - next.last_advanced_at_ms;
        if let Some(e) = &config.exposure {
            let mut cursor = next.last_advanced_at_ms;
            if inside && elapsed > 60_000 {
                return Err("E_ENV_STEP_RANGE");
            }
            if !inside {
                let loss = elapsed
                    .checked_mul(u64::from(e.loss_per_second))
                    .ok_or("E_ENV_TIME_OVERFLOW")?;
                next.exposure_milliunits = next.exposure_milliunits.saturating_sub(loss);
                cursor = now;
            }
            // Integrate chronological phase segments so a safe warning cannot
            // subtract exposure that has not yet accumulated, and caps/decay
            // remain identical under any subdivision of the same input interval.
            while cursor < now {
                let frame = next.frame(config, cursor.max(next.last_advanced_at_ms))?;
                let end = cursor.saturating_add(frame.remaining_ms).min(now);
                let duration = end - cursor;
                if frame.phase == EnvironmentPhase::Active {
                    let gain = duration
                        .checked_mul(u64::from(e.gain_per_second))
                        .ok_or("E_ENV_TIME_OVERFLOW")?;
                    next.exposure_milliunits = next
                        .exposure_milliunits
                        .checked_add(gain)
                        .ok_or("E_ENV_TIME_OVERFLOW")?
                        .min(u64::from(e.max_units) * 1000);
                } else {
                    let loss = duration
                        .checked_mul(u64::from(e.loss_per_second))
                        .ok_or("E_ENV_TIME_OVERFLOW")?;
                    next.exposure_milliunits = next.exposure_milliunits.saturating_sub(loss);
                }
                cursor = end;
            }
        }
        let damaging = inside
            && next.frame(config, now)?.phase == EnvironmentPhase::Active
            && config
                .exposure
                .as_ref()
                .is_none_or(|e| next.exposure_milliunits >= u64::from(e.damage_threshold) * 1000)
            && now >= next.next_damage_at_ms;
        if damaging {
            next.next_damage_at_ms = now
                .checked_add(config.damage_interval_ms)
                .ok_or("E_ENV_TIME_OVERFLOW")?;
        }
        next.last_advanced_at_ms = now;
        *self = next;
        Ok(damaging)
    }
    /// Suppression ends in a complete fresh warning; coolant cannot produce an instant hit.
    pub fn suppress(
        &mut self,
        config: &EnvironmentHazardConfig,
        control: &EnvironmentControlConfig,
        now: u64,
    ) -> Result<(), &'static str> {
        self.validate_at(config, now)?;
        control.validate()?;
        let until = now
            .checked_add(control.suppression_ms)
            .ok_or("E_ENV_TIME_OVERFLOW")?;
        let mut next = self.clone();
        // A shorter independent coolant control must never remove an already
        // earned safe window. Both still reduce exposure and consume cooldown.
        next.suppressed_until_ms = next.suppressed_until_ms.max(until);
        next.cycle_started_at_ms = next.suppressed_until_ms;
        next.last_advanced_at_ms = now;
        next.exposure_milliunits = next
            .exposure_milliunits
            .saturating_sub(u64::from(control.exposure_reduction_units) * 1000);
        *self = next;
        Ok(())
    }
}

/// Saved keys are matched to trusted authored identities, never accepted from a save alone.
pub fn validate_persistence(
    state: &EnvironmentPersistentState,
    hazards: &BTreeMap<String, EnvironmentHazardConfig>,
    controls: &BTreeMap<String, u64>,
    now: u64,
) -> Result<(), &'static str> {
    for (key, value) in &state.hazards {
        let config = hazards.get(key).ok_or("E_ENV_SAVED_HAZARD_UNKNOWN")?;
        value.validate_at(config, now)?;
    }
    for (key, deadline) in &state.control_cooldown_until_ms {
        let cooldown = controls.get(key).ok_or("E_ENV_SAVED_CONTROL_UNKNOWN")?;
        if deadline.saturating_sub(now) > *cooldown {
            return Err("E_ENV_SAVED_COOLDOWN_INVALID");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> EnvironmentHazardConfig {
        EnvironmentHazardConfig {
            tag: HazardTag::Heat,
            warning_ms: 600,
            active_ms: 2000,
            recovery_ms: 1000,
            damage: 8,
            damage_interval_ms: 500,
            exposure: None,
        }
    }
    #[test]
    fn exact_warning_active_recovery_and_repeat_boundaries() {
        let c = config();
        let s = EnvironmentHazardState::new(100);
        for (t, p, left) in [
            (100, EnvironmentPhase::Warning, 600),
            (699, EnvironmentPhase::Warning, 1),
            (700, EnvironmentPhase::Active, 2000),
            (2699, EnvironmentPhase::Active, 1),
            (2700, EnvironmentPhase::Recovery, 1000),
            (3699, EnvironmentPhase::Recovery, 1),
            (3700, EnvironmentPhase::Warning, 600),
        ] {
            let f = s.frame(&c, t).unwrap();
            assert_eq!((f.phase, f.remaining_ms), (p, left));
        }
    }
    #[test]
    fn warning_recovery_outside_and_intervals_are_safe() {
        let c = config();
        let mut s = EnvironmentHazardState::new(0);
        for (t, inside, hit) in [
            (599, true, false),
            (600, false, false),
            (601, true, true),
            (1100, true, false),
            (1101, true, true),
            (2600, true, false),
            (3600, true, false),
            (4200, true, true),
        ] {
            assert_eq!(s.advance(&c, t, inside).unwrap(), hit, "{t}");
        }
    }
    #[test]
    fn fixed_point_exposure_accumulates_decays_and_matches_partitioned_steps() {
        let mut c = config();
        c.exposure = Some(ExposureConfig {
            max_units: 1000,
            gain_per_second: 101,
            loss_per_second: 53,
            damage_threshold: 100,
        });
        let mut a = EnvironmentHazardState::new(0);
        let mut b = a.clone();
        a.advance(&c, 1600, true).unwrap();
        for t in (0..1600).step_by(16).chain(std::iter::once(1600)) {
            b.advance(&c, t, true).unwrap();
        }
        assert_eq!(a.exposure_milliunits, 101000);
        assert_eq!(a.exposure_milliunits, b.exposure_milliunits);
        assert_eq!(a.frame(&c, 1600).unwrap().exposure_bps, 1010);
        a.advance(&c, 2600, false).unwrap();
        assert_eq!(a.exposure_milliunits, 48000);
    }
    #[test]
    fn cooling_reduces_exposure_and_restarts_warning_without_changing_damage_interval() {
        let mut c = config();
        c.exposure = Some(ExposureConfig {
            max_units: 1000,
            gain_per_second: 1000,
            loss_per_second: 100,
            damage_threshold: 100,
        });
        let mut s = EnvironmentHazardState::new(0);
        s.advance(&c, 1000, true).unwrap();
        let ctrl = EnvironmentControlConfig {
            target_hazard_ids: vec!["zone".into()],
            suppression_ms: 2000,
            exposure_reduction_units: 300,
        };
        s.suppress(&c, &ctrl, 1000).unwrap();
        assert_eq!(s.exposure_milliunits, 100000);
        assert!(!s.advance(&c, 2999, true).unwrap());
        assert_eq!(
            s.frame(&c, 2999).unwrap().phase,
            EnvironmentPhase::Suppressed
        );
        assert_eq!(s.frame(&c, 3000).unwrap().phase, EnvironmentPhase::Warning);
        assert!(!s.advance(&c, 3599, true).unwrap());
    }
    #[test]
    fn independent_shorter_coolant_never_shortens_existing_safety_and_still_reduces_exposure() {
        let mut c = config();
        c.exposure = Some(ExposureConfig {
            max_units: 100,
            gain_per_second: 1000,
            loss_per_second: 1,
            damage_threshold: 50,
        });
        let mut s = EnvironmentHazardState::new(0);
        s.advance(&c, 1600, true).unwrap();
        let long = EnvironmentControlConfig {
            target_hazard_ids: vec!["zone".into()],
            suppression_ms: 9000,
            exposure_reduction_units: 20,
        };
        let short = EnvironmentControlConfig {
            suppression_ms: 6000,
            exposure_reduction_units: 30,
            ..long.clone()
        };
        s.suppress(&c, &long, 1600).unwrap();
        assert_eq!(s.exposure_milliunits, 80000);
        s.suppress(&c, &short, 1700).unwrap();
        assert_eq!(s.suppressed_until_ms, 10600);
        assert_eq!(s.cycle_started_at_ms, 10600);
        assert_eq!(s.exposure_milliunits, 50000);
        assert_eq!(
            s.frame(&c, 10599).unwrap().phase,
            EnvironmentPhase::Suppressed
        );
        assert_eq!(s.frame(&c, 10600).unwrap().phase, EnvironmentPhase::Warning);
    }
    #[test]
    fn overflow_and_clock_rollback_are_atomic() {
        let c = config();
        let mut s = EnvironmentHazardState::new(u64::MAX - 1000);
        let before = s.clone();
        assert_eq!(s.advance(&c, u64::MAX, true), Err("E_ENV_TIME_OVERFLOW"));
        assert_eq!(s, before);
        assert_eq!(
            s.advance(&c, u64::MAX - 1001, true),
            Err("E_ENV_HAZARD_STATE")
        );
        assert_eq!(s, before);
        let ctrl = EnvironmentControlConfig {
            target_hazard_ids: vec!["zone".into()],
            suppression_ms: 2000,
            exposure_reduction_units: 0,
        };
        assert_eq!(
            s.suppress(&c, &ctrl, u64::MAX - 1000),
            Err("E_ENV_TIME_OVERFLOW")
        );
        assert_eq!(s, before);
    }
    #[test]
    fn malformed_config_and_state_fail_closed() {
        let mut c = config();
        c.active_ms = 0;
        assert!(c.validate().is_err());
        c = config();
        let mut s = EnvironmentHazardState::new(100);
        s.exposure_milliunits = 1;
        assert!(s.validate_at(&c, 100).is_err());
        let ctrl = EnvironmentControlConfig {
            target_hazard_ids: vec!["zone".into(), "zone".into()],
            suppression_ms: 2000,
            exposure_reduction_units: 0,
        };
        assert!(ctrl.validate().is_err());
        assert!(serde_json::from_value::<EnvironmentHazardState>(serde_json::json!({"cycleStartedAtMs":0,"lastAdvancedAtMs":0,"nextDamageAtMs":0,"suppressedUntilMs":0,"exposureMilliunits":0,"immune":true})).is_err());
    }
    #[test]
    fn persistence_rejects_forged_ids_values_and_control_deadlines() {
        let c = config();
        let mut s = EnvironmentPersistentState::default();
        let hazards = BTreeMap::from([("world/scene/zone".into(), c.clone())]);
        let controls = BTreeMap::from([("world/scene/valve".into(), 8000)]);
        s.hazards
            .insert("world/scene/zone".into(), EnvironmentHazardState::new(100));
        validate_persistence(&s, &hazards, &controls, 100).unwrap();
        s.hazards
            .insert("forged/scene/zone".into(), EnvironmentHazardState::new(100));
        assert_eq!(
            validate_persistence(&s, &hazards, &controls, 100),
            Err("E_ENV_SAVED_HAZARD_UNKNOWN")
        );
        s.hazards.remove("forged/scene/zone");
        s.control_cooldown_until_ms
            .insert("world/scene/valve".into(), 8101);
        assert_eq!(
            validate_persistence(&s, &hazards, &controls, 100),
            Err("E_ENV_SAVED_COOLDOWN_INVALID")
        );
        s.control_cooldown_until_ms
            .insert("world/scene/valve".into(), 8100);
        validate_persistence(&s, &hazards, &controls, 100).unwrap();
        s.control_cooldown_until_ms
            .insert("world/scene/forged".into(), 100);
        assert_eq!(
            validate_persistence(&s, &hazards, &controls, 100),
            Err("E_ENV_SAVED_CONTROL_UNKNOWN")
        );
    }
    #[test]
    fn forged_cycle_origin_after_last_step_fails_before_runtime_integration() {
        let mut c = config();
        c.exposure = Some(ExposureConfig {
            max_units: 100,
            gain_per_second: 10,
            loss_per_second: 10,
            damage_threshold: 10,
        });
        let mut state = EnvironmentHazardState::new(100);
        state.cycle_started_at_ms = 200;
        assert_eq!(state.validate_at(&c, 300), Err("E_ENV_HAZARD_STATE"));
        let before = state.clone();
        assert_eq!(state.advance(&c, 300, true), Err("E_ENV_HAZARD_STATE"));
        assert_eq!(state, before);
    }
    #[test]
    fn exposure_partitioning_across_repeated_cycles_caps_and_suppression_is_deterministic() {
        let mut c = config();
        c.exposure = Some(ExposureConfig {
            max_units: 100,
            gain_per_second: 250,
            loss_per_second: 73,
            damage_threshold: 50,
        });
        let mut once = EnvironmentHazardState::new(0);
        let mut steps = once.clone();
        for now in [3599, 11_317, 35_000] {
            let start = steps.last_advanced_at_ms;
            once.advance(&c, now, true).unwrap();
            for t in (start..now).step_by(17).chain(std::iter::once(now)) {
                steps.advance(&c, t, true).unwrap();
            }
            assert_eq!(once.exposure_milliunits, steps.exposure_milliunits);
            assert_eq!(once.frame(&c, now).unwrap(), steps.frame(&c, now).unwrap());
        }
        let control = EnvironmentControlConfig {
            target_hazard_ids: vec!["zone".into()],
            suppression_ms: 2700,
            exposure_reduction_units: 33,
        };
        once.suppress(&c, &control, 35_000).unwrap();
        steps.suppress(&c, &control, 35_000).unwrap();
        once.advance(&c, 42_193, true).unwrap();
        for t in (35_000..42_193).step_by(13).chain(std::iter::once(42_193)) {
            steps.advance(&c, t, true).unwrap();
        }
        assert_eq!(once.exposure_milliunits, steps.exposure_milliunits);
        // Damage remains at most one hit per authoritative step, deliberately not a catch-up burst.
    }
    #[test]
    fn optional_persistence_defaults_keep_old_world_state_canonical() {
        let old = r#"{"mistHarbor":{"pump":{"state":"ready","drainCompleteAtWorldTimeMs":null},"exploredRegionIds":[]}}"#;
        let world: crate::world_persistent_v1::WorldPersistentState =
            serde_json::from_str(old).unwrap();
        assert!(world.environment.is_default());
        assert_eq!(
            serde_json::to_value(&world).unwrap(),
            serde_json::from_str::<serde_json::Value>(old).unwrap()
        );
        let mut state = EnvironmentHazardState::new(100);
        state.advance(&config(), 700, true).unwrap();
        assert_eq!(
            serde_json::from_str::<EnvironmentHazardState>(&serde_json::to_string(&state).unwrap())
                .unwrap(),
            state
        );
    }
}

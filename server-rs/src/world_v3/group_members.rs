//! Profile-driven group hit points. Member indices and offsets are immutable
//! profile identity; only their HP is saved. No member receives an action twice.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupProfile {
    pub member_max_hp: u32,
    pub member_radius_m: f32,
    pub offsets_m: Vec<[f32; 2]>,
}
impl GroupProfile {
    pub fn validate(&self, profile: &ActorProfile) -> bool {
        let Some(ordinary) = &profile.ordinary else { return false; };
        self.member_max_hp > 0 && (2..=8).contains(&self.offsets_m.len())
            && self.member_max_hp.checked_mul(self.offsets_m.len() as u32) == Some(profile.max_hp)
            && self.member_radius_m.is_finite() && self.member_radius_m > 0.0
            && self.offsets_m.iter().enumerate().all(|(i, p)| {
                p.iter().all(|v| v.is_finite())
                    && p[0].hypot(p[1]) + self.member_radius_m <= ordinary.body_radius_m
                    && self.offsets_m[..i].iter().all(|q| p != q)
            })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupState { pub member_hp: Vec<u32> }
impl GroupState {
    pub(super) fn new(profile: &GroupProfile) -> Self {
        Self { member_hp: vec![profile.member_max_hp; profile.offsets_m.len()] }
    }
}
pub(super) fn validate(actor: &ActorRuntime, profile: &ActorProfile) -> bool {
    match (&profile.group, &actor.group) {
        (None, None) => true,
        (Some(config), Some(group)) => group.member_hp.len() == config.offsets_m.len()
            && group.member_hp.iter().all(|hp| *hp <= config.member_max_hp)
            && group.member_hp.iter().map(|hp| u64::from(*hp)).sum::<u64>() == u64::from(actor.hp)
            && (group.member_hp.iter().all(|hp| *hp == 0)) == (actor.state == ActorAiState::Dead),
        _ => false,
    }
}
fn member_position(actor: &ActorRuntime, offset: [f32; 2]) -> Vec3 {
    Vec3 { x_m: actor.position_m.x_m + offset[0], z_m: actor.position_m.z_m + offset[1], ..actor.position_m }
}
fn clear_member_line(from: Vec3, to: Vec3, kcc: &StaticKccWorld) -> bool {
    let steps = ((horizontal_distance(from, to) / 0.025).ceil() as usize).max(1);
    (0..=steps).all(|i| {
        let t = i as f32 / steps as f32;
        kcc.can_occupy(Vec3 { x_m: from.x_m + (to.x_m-from.x_m)*t,
            y_m: from.y_m + (to.y_m-from.y_m)*t, z_m: from.z_m + (to.z_m-from.z_m)*t }, 0.025)
    })
}
impl ActorRuntime {
    /// Authoritative, living member hit points; ordinary actors retain one point.
    /// Group body collision encloses every member, and member hits require LOS.
    pub fn combat_points(&self, from: Vec3, kcc: &StaticKccWorld) -> Vec<(Option<usize>, Vec3)> {
        if self.hp == 0 || !self.validate() || !finite(from)
            || !crate::continuous_combat::combat_vertical_overlap(from, self.position_m)
            || !kcc.stable_actor_footprint(self.position_m,self.body_radius_m()) { return vec![]; }
        let Some(config) = actor_profile(&self.entity_type).and_then(|p| p.group.as_ref()) else {
            return vec![(None, self.position_m)];
        };
        config.offsets_m.iter().enumerate().filter_map(|(i, offset)| {
            let p = member_position(self, *offset);
            (self.group.as_ref().unwrap().member_hp[i] > 0
                && kcc.can_occupy(p, config.member_radius_m) && clear_member_line(from, p, kcc))
                .then_some((Some(i), p))
        }).collect()
    }

    /// One accepted owner action submits its distinct geometric member hits once.
    /// Validate before cloning or mutation; duplicate indices cannot multiply damage.
    pub fn take_member_hits(&mut self, members: &[usize], damage: u32, stagger: u32) -> Result<Vec<ActorRuntimeEvent>, String> {
        let Some(group) = &self.group else { return Err("E_ACTOR_GROUP_REQUIRED".into()); };
        if !self.validate() || members.iter().any(|i| *i >= group.member_hp.len()) {
            return Err("E_ACTOR_GROUP_HIT_INVALID".into());
        }
        if members.is_empty() { return Ok(vec![]); }
        let unique: BTreeSet<_> = members.iter().copied().filter(|i| group.member_hp[*i] > 0).collect();
        if unique.is_empty() { return Ok(vec![]); }
        let changes = unique.into_iter().map(|i| (i, damage)).collect::<Vec<_>>();
        apply(self, &changes, stagger)
    }
}
fn apply(actor: &mut ActorRuntime, changes: &[(usize, u32)], stagger: u32) -> Result<Vec<ActorRuntimeEvent>, String> {
    if !actor.validate() { return Err("E_ACTOR_GROUP_HIT_INVALID".into()); }
    let profile = actor_profile(&actor.entity_type).ok_or("E_ACTOR_PROFILE_UNKNOWN")?;
    let config = profile.group.as_ref().ok_or("E_ACTOR_GROUP_REQUIRED")?;
    let mut next = actor.clone();
    let mut group = next.group.clone().ok_or("E_ACTOR_GROUP_REQUIRED")?;
    let mut member_events = vec![];
    for &(i, amount) in changes {
        let before = group.member_hp[i];
        group.member_hp[i] = before.saturating_sub(amount);
        if before != group.member_hp[i] {
            member_events.push(ActorRuntimeEvent::MemberHit {
                actor_id: actor.entity_id.clone(), member_id: format!("{}/member/{}", actor.entity_id, i+1),
                position_m: member_position(actor, config.offsets_m[i]), destroyed: group.member_hp[i] == 0,
            });
        }
    }
    let remaining: u32 = group.member_hp.iter().sum();
    let damage = next.hp - remaining;
    // Controller sees the old valid aggregate before the atomic member update.
    let mut events = ordinary_controller::take_damage(&mut next, profile, profile.ordinary.as_ref().unwrap(), damage, stagger);
    next.group = Some(group);
    if !next.validate() { return Err("E_ACTOR_GROUP_HIT_INVALID".into()); }
    member_events.append(&mut events);
    *actor = next;
    Ok(member_events)
}
pub(super) fn take_budget(actor: &mut ActorRuntime, damage: u32, stagger: u32) -> Vec<ActorRuntimeEvent> {
    if !actor.validate() { return vec![]; }
    let Some(group) = &actor.group else { return vec![]; };
    let mut remaining = damage;
    let changes = group.member_hp.iter().enumerate().map(|(i, hp)| {
        let amount = remaining.min(*hp); remaining -= amount; (i, amount)
    }).collect::<Vec<_>>();
    apply(actor, &changes, stagger).unwrap_or_default()
}
pub(super) fn scale_attack_damage(actor: &ActorRuntime, base: u32) -> u32 {
    actor.group.as_ref().map_or(base, |group| {
        if group.member_hp.is_empty() { return 0; }
        let living = group.member_hp.iter().filter(|hp| **hp > 0).count() as u32;
        base.saturating_mul(living).div_ceil(group.member_hp.len() as u32)
    })
}

/// Public, HP-free member geometry follows the same profile points used for hits.
pub(super) fn view_members(actor: &ActorRuntime) -> Option<Vec<super::super::ActorMemberView>> {
    if !actor.validate() { return None; }
    let config = actor_profile(&actor.entity_type)?.group.as_ref()?;
    let group = actor.group.as_ref()?;
    Some(config.offsets_m.iter().enumerate().map(|(i, offset)| super::super::ActorMemberView {
        member_id: format!("{}/member/{}", actor.entity_id, i+1), position_m: member_position(actor, *offset),
        radius_m: config.member_radius_m, active: group.member_hp[i] > 0,
    }).collect())
}

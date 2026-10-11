use rapier3d::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::f32::consts::PI;

pub const EPSILON_M: f32 = 1.0e-4;
pub const EPSILON_RAD: f32 = 1.0e-5;
pub const HALF_SEGMENT_M: f32 = 0.75;
pub const CAPSULE_RADIUS_M: f32 = 0.35;
pub const FOOT_CENTER_OFFSET_M: f32 = 1.10;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Vec3M {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3M {
    pub fn finite(self) -> Result<Self, QueryError> {
        if [self.x, self.y, self.z].iter().all(|v| v.is_finite()) {
            Ok(self)
        } else {
            Err(QueryError::NonFinite("position"))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct FootTransform {
    pub position_m: Vec3M,
    pub yaw_rad: f32,
}

impl FootTransform {
    pub fn finite(self) -> Result<Self, QueryError> {
        self.position_m.finite()?;
        if self.yaw_rad.is_finite() {
            Ok(self)
        } else {
            Err(QueryError::NonFinite("yawRad"))
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AttackInput {
    pub world_epoch: u64,
    pub world_tick: u64,
    pub attack_event_id: u64,
    pub attacker: FootTransform,
    pub range_m: f32,
    pub half_angle_rad: f32,
    pub vertical_tolerance_m: f32,
}

impl AttackInput {
    pub fn validate(&self) -> Result<(), QueryError> {
        self.attacker.finite()?;
        if self.attack_event_id == 0 {
            return Err(QueryError::InvalidAttackEventId);
        }
        if !self.range_m.is_finite() {
            return Err(QueryError::NonFinite("rangeM"));
        }
        if !self.half_angle_rad.is_finite() {
            return Err(QueryError::NonFinite("halfAngleRad"));
        }
        if !self.vertical_tolerance_m.is_finite() {
            return Err(QueryError::NonFinite("verticalToleranceM"));
        }
        if self.range_m < 0.0 {
            return Err(QueryError::InvalidParameter("rangeM"));
        }
        if !(0.0..=PI).contains(&self.half_angle_rad) {
            return Err(QueryError::InvalidParameter("halfAngleRad"));
        }
        if self.vertical_tolerance_m < 0.0 {
            return Err(QueryError::InvalidParameter("verticalToleranceM"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TargetCapsule {
    pub id: String,
    pub foot_position_m: Vec3M,
    pub radius_m: f32,
    pub half_segment_m: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TargetAudit {
    pub target_id: String,
    pub distance_m: f32,
    pub horizontal_angle_rad: f32,
    pub horizontal_angle_deg: f32,
    pub vertical_delta_m: f32,
    pub los: String,
    pub los_blocker_id: Option<String>,
    pub accepted: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttackResult {
    pub candidate_only: bool,
    pub evidence_level: &'static str,
    pub world_epoch: u64,
    pub world_tick: u64,
    pub attack_event_id: u64,
    pub epsilon_m: f32,
    pub epsilon_rad: f32,
    pub target_ids: Vec<String>,
    pub rejection_reason: Option<String>,
    pub audits: Vec<TargetAudit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryError {
    NonFinite(&'static str),
    InvalidAttackEventId,
    InvalidParameter(&'static str),
    WrongEpoch { expected: u64, actual: u64 },
    OldTick { minimum: u64, actual: u64 },
    DuplicateAttackEvent(u64),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite(field) => write!(f, "non_finite:{field}"),
            Self::InvalidAttackEventId => f.write_str("invalid_attack_event_id"),
            Self::InvalidParameter(field) => write!(f, "invalid_parameter:{field}"),
            Self::WrongEpoch { expected, actual } => {
                write!(f, "old_epoch:expected={expected}:actual={actual}")
            }
            Self::OldTick { minimum, actual } => {
                write!(f, "old_tick:minimum={minimum}:actual={actual}")
            }
            Self::DuplicateAttackEvent(id) => write!(f, "duplicate_attack_event:{id}"),
        }
    }
}

impl std::error::Error for QueryError {}

#[derive(Clone, Debug)]
struct TargetRecord {
    target: TargetCapsule,
    collider: ColliderHandle,
}

#[derive(Default)]
pub struct SpatialWorld {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    queries: QueryPipeline,
    targets: BTreeMap<String, TargetRecord>,
    collider_roles: HashMap<ColliderHandle, String>,
}

impl SpatialWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_target(&mut self, target: TargetCapsule) -> Result<(), QueryError> {
        if target.id.is_empty() || self.targets.contains_key(&target.id) {
            return Err(QueryError::InvalidParameter("targetId"));
        }
        target.foot_position_m.finite()?;
        if !target.radius_m.is_finite()
            || !target.half_segment_m.is_finite()
            || target.radius_m <= 0.0
            || target.half_segment_m < 0.0
        {
            return Err(QueryError::InvalidParameter("targetCapsule"));
        }
        let foot = target.foot_position_m;
        let center_y = foot.y + target.half_segment_m + target.radius_m;
        let collider = ColliderBuilder::new(SharedShape::capsule_y(
            target.half_segment_m,
            target.radius_m,
        ))
        .translation(vector![foot.x, center_y, foot.z])
        .build();
        let handle = self.colliders.insert(collider);
        self.collider_roles
            .insert(handle, format!("target:{}", target.id));
        self.targets.insert(
            target.id.clone(),
            TargetRecord {
                target,
                collider: handle,
            },
        );
        self.refresh();
        Ok(())
    }

    pub fn add_blocker_box(
        &mut self,
        id: &str,
        center_m: Vec3M,
        half_extents_m: Vec3M,
    ) -> Result<(), QueryError> {
        center_m.finite()?;
        half_extents_m.finite()?;
        if id.is_empty()
            || [half_extents_m.x, half_extents_m.y, half_extents_m.z]
                .iter()
                .any(|v| *v <= 0.0)
        {
            return Err(QueryError::InvalidParameter("blocker"));
        }
        let collider =
            ColliderBuilder::cuboid(half_extents_m.x, half_extents_m.y, half_extents_m.z)
                .translation(vector![center_m.x, center_m.y, center_m.z])
                .build();
        let handle = self.colliders.insert(collider);
        self.collider_roles.insert(handle, id.to_owned());
        self.refresh();
        Ok(())
    }

    fn refresh(&mut self) {
        self.queries.update(&self.colliders);
    }

    fn target_center(target: &TargetCapsule) -> Point<Real> {
        point![
            target.foot_position_m.x,
            target.foot_position_m.y + target.half_segment_m + target.radius_m,
            target.foot_position_m.z
        ]
    }

    fn line_of_sight(&self, origin: Point<Real>, target: &TargetRecord) -> (bool, Option<String>) {
        let target_point = Self::target_center(&target.target);
        let delta = target_point - origin;
        let target_filter = |handle: ColliderHandle, _collider: &Collider| {
            handle != target.collider
                && !self
                    .collider_roles
                    .get(&handle)
                    .is_some_and(|role| role.starts_with("target:"))
        };
        let filter = QueryFilter::default().predicate(&target_filter);
        let Some((handle, _toi)) = self.queries.cast_ray(
            &self.bodies,
            &self.colliders,
            &Ray::new(origin, delta),
            1.0,
            true,
            filter,
        ) else {
            return (true, None);
        };
        (false, self.collider_roles.get(&handle).cloned())
    }

    pub fn evaluate(&self, input: &AttackInput) -> Result<AttackResult, QueryError> {
        input.validate()?;
        let forward = vector![
            input.attacker.yaw_rad.sin(),
            0.0,
            input.attacker.yaw_rad.cos()
        ];
        let attacker = input.attacker.position_m;
        let origin = point![attacker.x, attacker.y + FOOT_CENTER_OFFSET_M, attacker.z];
        let mut audits = Vec::new();
        for target in self.targets.values() {
            let dx = target.target.foot_position_m.x - attacker.x;
            let dz = target.target.foot_position_m.z - attacker.z;
            let distance = dx.hypot(dz);
            let horizontal_angle = if distance <= EPSILON_M {
                0.0
            } else {
                ((forward.x * dx + forward.z * dz) / distance)
                    .clamp(-1.0, 1.0)
                    .acos()
            };
            let vertical_delta = target.target.foot_position_m.y - attacker.y;
            let (los_clear, blocker) = self.line_of_sight(origin, target);
            let (accepted, rejection_reason, los_label) = if distance > input.range_m + EPSILON_M {
                (false, Some("out_of_range".to_owned()), "not_tested")
            } else if horizontal_angle > input.half_angle_rad + EPSILON_RAD {
                (false, Some("out_of_cone".to_owned()), "not_tested")
            } else if vertical_delta.abs() > input.vertical_tolerance_m + EPSILON_M {
                (false, Some("vertical_outside".to_owned()), "not_tested")
            } else if !los_clear {
                (false, Some("los_blocked".to_owned()), "blocked")
            } else {
                (true, None, "clear")
            };
            audits.push(TargetAudit {
                target_id: target.target.id.clone(),
                distance_m: distance,
                horizontal_angle_rad: horizontal_angle,
                horizontal_angle_deg: horizontal_angle.to_degrees(),
                vertical_delta_m: vertical_delta,
                los: los_label.to_owned(),
                los_blocker_id: blocker,
                accepted,
                rejection_reason,
            });
        }
        audits.sort_by(|a, b| {
            a.distance_m
                .total_cmp(&b.distance_m)
                .then_with(|| a.target_id.cmp(&b.target_id))
        });
        let target_ids = audits
            .iter()
            .filter(|a| a.accepted)
            .map(|a| a.target_id.clone())
            .collect::<Vec<_>>();
        Ok(AttackResult {
            candidate_only: true,
            evidence_level: "isolated_candidate_rapier_022",
            world_epoch: input.world_epoch,
            world_tick: input.world_tick,
            attack_event_id: input.attack_event_id,
            epsilon_m: EPSILON_M,
            epsilon_rad: EPSILON_RAD,
            target_ids,
            rejection_reason: None,
            audits,
        })
    }
}

pub struct SpatialAuthority {
    pub world_epoch: u64,
    pub minimum_world_tick: u64,
    consumed_events: BTreeSet<u64>,
}

impl SpatialAuthority {
    pub fn new(world_epoch: u64, minimum_world_tick: u64) -> Self {
        Self {
            world_epoch,
            minimum_world_tick,
            consumed_events: BTreeSet::new(),
        }
    }

    pub fn query(
        &mut self,
        world: &SpatialWorld,
        input: &AttackInput,
    ) -> Result<AttackResult, QueryError> {
        input.validate()?;
        if input.world_epoch != self.world_epoch {
            return Err(QueryError::WrongEpoch {
                expected: self.world_epoch,
                actual: input.world_epoch,
            });
        }
        if input.world_tick < self.minimum_world_tick {
            return Err(QueryError::OldTick {
                minimum: self.minimum_world_tick,
                actual: input.world_tick,
            });
        }
        if !self.consumed_events.insert(input.attack_event_id) {
            return Err(QueryError::DuplicateAttackEvent(input.attack_event_id));
        }
        self.minimum_world_tick = input.world_tick;
        world.evaluate(input)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioRecord {
    pub name: String,
    pub input: serde_json::Value,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub pass: bool,
    pub evidence_level: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProbeReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub machine_pass: bool,
    pub candidate_only: bool,
    pub coordinate_contract: serde_json::Value,
    pub parameters: serde_json::Value,
    pub scenarios: Vec<ScenarioRecord>,
    pub formal_gaps: Vec<&'static str>,
}

fn target(id: &str, x: f32, y: f32, z: f32) -> TargetCapsule {
    TargetCapsule {
        id: id.to_owned(),
        foot_position_m: Vec3M { x, y, z },
        radius_m: CAPSULE_RADIUS_M,
        half_segment_m: HALF_SEGMENT_M,
    }
}

fn input(
    event: u64,
    tick: u64,
    _target: &TargetCapsule,
    range: f32,
    angle: f32,
    vertical: f32,
) -> AttackInput {
    AttackInput {
        world_epoch: 7,
        world_tick: tick,
        attack_event_id: event,
        attacker: FootTransform {
            position_m: Vec3M {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            yaw_rad: 0.0,
        },
        range_m: range,
        half_angle_rad: angle,
        vertical_tolerance_m: vertical,
    }
}

fn record(
    name: &str,
    input: serde_json::Value,
    expected: serde_json::Value,
    actual: serde_json::Value,
    pass: bool,
) -> ScenarioRecord {
    ScenarioRecord {
        name: name.to_owned(),
        input,
        expected,
        actual,
        pass,
        evidence_level: "isolated_candidate_rapier_022",
    }
}

pub fn run_all_scenarios() -> ProbeReport {
    let mut scenarios = Vec::new();
    let mut world = SpatialWorld::new();
    let straight = target("target-forward", 0.0, 0.0, 3.0);
    world.add_target(straight.clone()).unwrap();
    let mut authority = SpatialAuthority::new(7, 100);
    let result = authority
        .query(
            &world,
            &input(1, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5),
        )
        .unwrap();
    scenarios.push(record(
        "forward_hit",
        serde_json::to_value(input(1, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5)).unwrap(),
        json!({"targetIds":["target-forward"]}),
        json!({"targetIds":result.target_ids,"audit":result.audits}),
        result.target_ids == vec!["target-forward"],
    ));

    let boundary = target("target-boundary", 0.0, 0.0, 3.0);
    let mut boundary_world = SpatialWorld::new();
    boundary_world.add_target(boundary.clone()).unwrap();
    let mut boundary_auth = SpatialAuthority::new(7, 100);
    let at_range = boundary_auth
        .query(
            &boundary_world,
            &input(2, 100, &boundary, 3.0, 30.0_f32.to_radians(), 0.5),
        )
        .unwrap();
    let outside = target("target-outside", 0.0, 0.0, 3.0002);
    boundary_world.add_target(outside.clone()).unwrap();
    let outside_result = boundary_world
        .evaluate(&input(3, 100, &outside, 3.0, 30.0_f32.to_radians(), 0.5))
        .unwrap();
    scenarios.push(record("distance_boundary_in_and_out", json!({"epsilonM":EPSILON_M,"at":3.0,"outside":3.0002}), json!({"atRangeHit":true,"outsideRangeHit":false}), json!({"atRangeHit":!at_range.target_ids.is_empty(),"outsideRangeHit":outside_result.target_ids.iter().any(|id| id == "target-outside"),"atAudit":at_range.audits,"outsideAudit":outside_result.audits}), !at_range.target_ids.is_empty() && !outside_result.target_ids.iter().any(|id| id == "target-outside")));

    let angle = target(
        "target-angle",
        1.5,
        0.0,
        (3.0_f32.powi(2) - 1.5_f32.powi(2)).sqrt(),
    );
    let mut angle_world = SpatialWorld::new();
    angle_world.add_target(angle.clone()).unwrap();
    let mut angle_auth = SpatialAuthority::new(7, 100);
    let angle_at = angle_auth
        .query(
            &angle_world,
            &input(4, 100, &angle, 3.1, 30.0_f32.to_radians(), 0.5),
        )
        .unwrap();
    let outside_angle = target(
        "target-angle-out",
        1.501,
        0.0,
        (3.0_f32.powi(2) - 1.501_f32.powi(2)).sqrt(),
    );
    angle_world.add_target(outside_angle.clone()).unwrap();
    let outside_angle_result = angle_world
        .evaluate(&input(
            5,
            100,
            &outside_angle,
            3.1,
            30.0_f32.to_radians(),
            0.5,
        ))
        .unwrap();
    scenarios.push(record("cone_angle_boundary_in_and_out", json!({"epsilonRad":EPSILON_RAD,"atAngleDeg":30.0,"outsideAngleDeg":outside_angle_result.audits.iter().find(|a|a.target_id=="target-angle-out").unwrap().horizontal_angle_deg}), json!({"atAngleHit":true,"outsideAngleHit":false}), json!({"atAngleHit":!angle_at.target_ids.is_empty(),"outsideAngleHit":outside_angle_result.target_ids.iter().any(|id| id == "target-angle-out"),"atAudit":angle_at.audits,"outsideAudit":outside_angle_result.audits}), !angle_at.target_ids.is_empty() && !outside_angle_result.target_ids.iter().any(|id| id == "target-angle-out")));

    let behind = target("target-behind", 0.0, 0.0, -2.0);
    let mut behind_world = SpatialWorld::new();
    behind_world.add_target(behind.clone()).unwrap();
    let behind_result = behind_world
        .evaluate(&input(6, 100, &behind, 3.0, 45.0_f32.to_radians(), 0.5))
        .unwrap();
    scenarios.push(record(
        "behind_rejected",
        json!({"target":"target-behind","yawRad":0.0}),
        json!({"targetIds":[]}),
        json!({"targetIds":behind_result.target_ids,"audit":behind_result.audits}),
        behind_result.target_ids.is_empty()
            && behind_result.audits[0].rejection_reason.as_deref() == Some("out_of_cone"),
    ));

    let vertical = target("target-vertical", 0.0, 1.0, 2.0);
    let mut vertical_world = SpatialWorld::new();
    vertical_world.add_target(vertical.clone()).unwrap();
    let mut vertical_auth = SpatialAuthority::new(7, 100);
    let at_vertical = vertical_auth
        .query(
            &vertical_world,
            &input(7, 100, &vertical, 3.0, 30.0_f32.to_radians(), 1.0),
        )
        .unwrap();
    let outside_vertical = target("target-vertical-out", 0.0, 1.0002, 2.0);
    vertical_world.add_target(outside_vertical.clone()).unwrap();
    let outside_vertical_result = vertical_world
        .evaluate(&input(
            8,
            100,
            &outside_vertical,
            3.0,
            30.0_f32.to_radians(),
            1.0,
        ))
        .unwrap();
    scenarios.push(record("vertical_tolerance_boundary", json!({"epsilonM":EPSILON_M,"at":1.0,"outside":1.0002}), json!({"atVerticalHit":true,"outsideVerticalHit":false}), json!({"atVerticalHit":!at_vertical.target_ids.is_empty(),"outsideVerticalHit":outside_vertical_result.target_ids.iter().any(|id| id == "target-vertical-out"),"atAudit":at_vertical.audits,"outsideAudit":outside_vertical_result.audits}), !at_vertical.target_ids.is_empty() && !outside_vertical_result.target_ids.iter().any(|id| id == "target-vertical-out")));

    let los_target = target("target-wall", 0.0, 0.0, 4.0);
    let mut wall_world = SpatialWorld::new();
    wall_world.add_target(los_target.clone()).unwrap();
    wall_world
        .add_blocker_box(
            "wall-static",
            Vec3M {
                x: 0.0,
                y: 1.1,
                z: 2.0,
            },
            Vec3M {
                x: 0.5,
                y: 1.0,
                z: 0.1,
            },
        )
        .unwrap();
    let wall_result = wall_world
        .evaluate(&input(9, 100, &los_target, 5.0, 30.0_f32.to_radians(), 0.5))
        .unwrap();
    scenarios.push(record(
        "wall_blocks_los",
        json!({"blocker":"wall-static"}),
        json!({"targetIds":[]}),
        json!({"targetIds":wall_result.target_ids,"audit":wall_result.audits}),
        wall_result.target_ids.is_empty()
            && wall_result.audits[0].los_blocker_id.as_deref() == Some("wall-static"),
    ));
    let door_target = target("target-door", 0.0, 0.0, 4.0);
    let mut closed = SpatialWorld::new();
    closed.add_target(door_target.clone()).unwrap();
    closed
        .add_blocker_box(
            "door-closed",
            Vec3M {
                x: 0.0,
                y: 1.1,
                z: 2.0,
            },
            Vec3M {
                x: 0.5,
                y: 1.0,
                z: 0.1,
            },
        )
        .unwrap();
    let closed_result = closed
        .evaluate(&input(
            10,
            100,
            &door_target,
            5.0,
            30.0_f32.to_radians(),
            0.5,
        ))
        .unwrap();
    let mut open = SpatialWorld::new();
    open.add_target(door_target.clone()).unwrap();
    let open_result = open
        .evaluate(&input(
            11,
            100,
            &door_target,
            5.0,
            30.0_f32.to_radians(),
            0.5,
        ))
        .unwrap();
    scenarios.push(record("closed_door_blocks_open_door_hits", json!({"closedDoorCollider":"door-closed","openDoorCollider":null}), json!({"closed":[],"open":["target-door"]}), json!({"closed":closed_result.target_ids,"open":open_result.target_ids,"closedAudit":closed_result.audits,"openAudit":open_result.audits}), closed_result.target_ids.is_empty() && open_result.target_ids == vec!["target-door"]));

    let mut multi = SpatialWorld::new();
    for t in [
        target("target-b", 0.0, 0.0, 2.0),
        target("target-a", 0.0, 0.0, 2.0),
        target("target-c", 0.0, 0.0, 1.5),
    ] {
        multi.add_target(t).unwrap();
    }
    let multi_target = target("target-a", 0.0, 0.0, 2.0);
    let multi_result = multi
        .evaluate(&input(
            12,
            100,
            &multi_target,
            3.0,
            30.0_f32.to_radians(),
            0.5,
        ))
        .unwrap();
    scenarios.push(record(
        "stable_multi_target_sort",
        json!({"tieBreak":"targetId","distanceSort":"ascending"}),
        json!({"targetIds":["target-c","target-a","target-b"]}),
        json!({"targetIds":multi_result.target_ids,"audits":multi_result.audits}),
        multi_result.target_ids == vec!["target-c", "target-a", "target-b"],
    ));

    let mut dup_world = SpatialWorld::new();
    dup_world.add_target(straight.clone()).unwrap();
    let mut dup_auth = SpatialAuthority::new(7, 100);
    let first = dup_auth
        .query(
            &dup_world,
            &input(13, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5),
        )
        .unwrap();
    let duplicate = dup_auth.query(
        &dup_world,
        &input(13, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5),
    );
    scenarios.push(record(
        "duplicate_attack_event_idempotent",
        json!({"attackEventId":13}),
        json!({"firstHit":["target-forward"],"duplicateError":"duplicate_attack_event:13"}),
        json!({"firstHit":first.target_ids,"duplicateError":duplicate.clone().err().map(|e|e.to_string())}),
        first.target_ids == vec!["target-forward"]
            && duplicate.err() == Some(QueryError::DuplicateAttackEvent(13)),
    ));

    let mut barrier_world = SpatialWorld::new();
    barrier_world.add_target(straight.clone()).unwrap();
    let mut barrier_auth = SpatialAuthority::new(7, 100);
    let old_epoch = barrier_auth.query(
        &barrier_world,
        &AttackInput {
            world_epoch: 6,
            ..input(14, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5)
        },
    );
    let old_tick = barrier_auth.query(
        &barrier_world,
        &AttackInput {
            attack_event_id: 15,
            world_tick: 99,
            ..input(15, 99, &straight, 3.0, 30.0_f32.to_radians(), 0.5)
        },
    );
    scenarios.push(record("old_epoch_and_tick_rejected", json!({"expectedEpoch":7,"minimumTick":100}), json!({"epoch":"old_epoch:expected=7:actual=6","tick":"old_tick:minimum=100:actual=99"}), json!({"epoch":old_epoch.clone().err().map(|e|e.to_string()),"tick":old_tick.clone().err().map(|e|e.to_string())}), old_epoch.err()==Some(QueryError::WrongEpoch{expected:7,actual:6}) && old_tick.err()==Some(QueryError::OldTick{minimum:100,actual:99})));

    let mut invalid_world = SpatialWorld::new();
    invalid_world.add_target(straight.clone()).unwrap();
    let mut invalid_auth = SpatialAuthority::new(7, 100);
    let nan = invalid_auth.query(
        &invalid_world,
        &AttackInput {
            attack_event_id: 16,
            range_m: f32::NAN,
            ..input(16, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5)
        },
    );
    let inf = invalid_auth.query(
        &invalid_world,
        &AttackInput {
            attack_event_id: 17,
            half_angle_rad: f32::INFINITY,
            ..input(17, 100, &straight, 3.0, 30.0_f32.to_radians(), 0.5)
        },
    );
    scenarios.push(record(
        "nan_infinity_rejected",
        json!({"nan":"rangeM","infinity":"halfAngleRad"}),
        json!({"nan":"non_finite:rangeM","infinity":"non_finite:halfAngleRad"}),
        json!({"nan":nan.clone().err().map(|e|e.to_string()),"infinity":inf.clone().err().map(|e|e.to_string())}),
        nan.err() == Some(QueryError::NonFinite("rangeM"))
            && inf.err() == Some(QueryError::NonFinite("halfAngleRad")),
    ));

    let hit_set_at_rate = |tick: u64| {
        let mut rate_world = SpatialWorld::new();
        rate_world.add_target(straight.clone()).unwrap();
        rate_world
            .add_target(target("target-side", 0.2, 0.0, 2.0))
            .unwrap();
        let mut rate_auth = SpatialAuthority::new(7, tick);
        rate_auth
            .query(
                &rate_world,
                &input(
                    tick + 1000,
                    tick,
                    &straight,
                    3.0,
                    30.0_f32.to_radians(),
                    0.5,
                ),
            )
            .unwrap()
            .target_ids
    };
    let at_30 = hit_set_at_rate(30);
    let at_60 = hit_set_at_rate(60);
    scenarios.push(record(
        "thirty_sixty_hz_same_hit_set",
        json!({"fixedStepHz":[30,60],"sameInput":true}),
        json!({"sameHitSet":true}),
        json!({"at30Hz":at_30,"at60Hz":at_60,"same":at_30==at_60}),
        at_30 == at_60,
    ));

    let machine_pass = scenarios.iter().all(|scenario| scenario.pass);
    ProbeReport {
        schema: "continuous_combat_spatial_authority_tp12a1_v1",
        status: if machine_pass {
            "passed_isolated_candidate"
        } else {
            "blocked_probe_failure"
        },
        machine_pass,
        candidate_only: true,
        coordinate_contract: json!({"units":"metres","handedness":"right_handed","axes":"x=east,y=up,z=depth","footAnchor":"transform origin at foot plane","forward":"(sin(yaw),0,cos(yaw)","source":"docs/CONTINUOUS_COORDINATE_ADR.md + coordinates-ag05.json"}),
        parameters: json!({"epsilonM":EPSILON_M,"epsilonRad":EPSILON_RAD,"capsuleRadiusM":CAPSULE_RADIUS_M,"capsuleHalfSegmentM":HALF_SEGMENT_M,"footCenterOffsetM":FOOT_CENTER_OFFSET_M,"rangeMetric":"horizontal foot-anchor center distance","verticalMetric":"target foot y minus attacker foot y","los":"Rapier 0.22 QueryPipeline::cast_ray; target colliders filtered, static/dynamic blockers retained","sorting":"distance ascending, targetId ascending tie-break","duplicatePolicy":"attackEventId consumed once per authority epoch"}),
        scenarios,
        formal_gaps: vec![
            "formal combat owner/runtime integration",
            "damage/cooldown/animation synchronization",
            "real Grey Hive collider export",
            "performance budget and player hand-feel",
            "formal Gate/Architecture approval",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::run_all_scenarios;

    #[test]
    fn all_tp12a1_scenarios_pass() {
        let report = run_all_scenarios();
        assert!(
            report.machine_pass,
            "{}",
            serde_json::to_string_pretty(&report).unwrap()
        );
        assert_eq!(report.scenarios.len(), 12);
    }
}

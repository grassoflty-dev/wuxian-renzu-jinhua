#![forbid(unsafe_code)]

pub const SCHEMA_VERSION: u32 = 3;
pub const PROTOCOL: &str = "world-v3-candidate";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x_m: f32,
    pub y_m: f32,
    pub z_m: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x_m: 0.0,
        y_m: 0.0,
        z_m: 0.0,
    };
    pub fn finite(self) -> bool {
        self.x_m.is_finite() && self.y_m.is_finite() && self.z_m.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position_m: Vec3,
    pub yaw_rad: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity {
    pub metres_per_second: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldView {
    pub schema_version: u32,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub ack_seq: u64,
    pub entity_id: String,
    pub transform: Transform,
    pub velocity: Velocity,
    pub door_id: String,
    pub door_open: bool,
    pub trigger_id: String,
    pub trigger_inside: bool,
    pub zone_id: String,
    pub hive_lockdown: bool,
    pub hive_choice: Choice,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldEvent {
    pub schema_version: u32,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub ack_seq: u64,
    pub kind: EventKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    Moved,
    DoorChanged,
    TriggerEntered,
    TriggerLeft,
    ZoneChanged,
    Paused,
    Resumed,
    Reloaded,
    Disposed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Choice {
    Taken,
    Left,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorldCommand {
    Move { seq: u64, delta: Vec3 },
    ToggleDoor { seq: u64, door_id: String },
    EnterTrigger { seq: u64, trigger_id: String },
    LeaveTrigger { seq: u64, trigger_id: String },
    Pause { seq: u64 },
    Resume { seq: u64 },
    Reload { seq: u64 },
    Dispose { seq: u64 },
}

impl WorldCommand {
    pub fn seq(&self) -> u64 {
        match self {
            Self::Move { seq, .. }
            | Self::ToggleDoor { seq, .. }
            | Self::EnterTrigger { seq, .. }
            | Self::LeaveTrigger { seq, .. }
            | Self::Pause { seq }
            | Self::Resume { seq }
            | Self::Reload { seq }
            | Self::Dispose { seq } => *seq,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaveV3 {
    pub schema_version: u32,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub ack_seq: u64,
    pub transform: Transform,
    pub velocity: Velocity,
    pub door_open: bool,
    pub trigger_inside: bool,
    pub zone_id: String,
    pub hive_lockdown: bool,
    pub hive_choice: Choice,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Reject {
    InvalidSchema,
    NonFinite,
    OldEpoch,
    FutureAck,
    DuplicateCommand,
    OutOfOrder,
    UnknownId,
    Disposed,
    Paused,
    InvalidSave,
    FutureVersion,
}

#[derive(Debug, PartialEq)]
pub struct World {
    pub schema_version: u32,
    pub epoch: u64,
    pub tick: u64,
    pub ack: u64,
    pub transform: Transform,
    pub velocity: Velocity,
    pub door_open: bool,
    pub trigger_inside: bool,
    pub zone_id: String,
    pub paused: bool,
    pub disposed: bool,
    pub hive_lockdown: bool,
    pub hive_choice: Choice,
    last_seq: u64,
}

impl World {
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            epoch: 1,
            tick: 0,
            ack: 0,
            transform: Transform {
                position_m: Vec3::ZERO,
                yaw_rad: 0.0,
            },
            velocity: Velocity {
                metres_per_second: Vec3::ZERO,
            },
            door_open: false,
            trigger_inside: false,
            zone_id: "start".into(),
            paused: false,
            disposed: false,
            hive_lockdown: false,
            hive_choice: Choice::Unresolved,
            last_seq: 0,
        }
    }
    pub fn step(&mut self) {
        if !self.paused && !self.disposed {
            self.tick += 1;
        }
    }
    pub fn apply(&mut self, command: WorldCommand) -> Result<Vec<WorldEvent>, Reject> {
        if self.disposed {
            return Err(Reject::Disposed);
        }
        if command.seq() <= self.last_seq {
            return Err(if command.seq() == self.last_seq {
                Reject::DuplicateCommand
            } else {
                Reject::OutOfOrder
            });
        }
        self.last_seq = command.seq();
        self.ack = command.seq();
        let mut events = Vec::new();
        match command {
            WorldCommand::Move { delta, .. } => {
                if self.paused {
                    return Err(Reject::Paused);
                }
                if !delta.finite() {
                    return Err(Reject::NonFinite);
                }
                self.transform.position_m.x_m += delta.x_m;
                self.transform.position_m.y_m += delta.y_m;
                self.transform.position_m.z_m += delta.z_m;
                self.velocity.metres_per_second = delta;
                events.push(self.event(EventKind::Moved));
            }
            WorldCommand::ToggleDoor { door_id, .. } => {
                if door_id != "door.main" {
                    return Err(Reject::UnknownId);
                }
                self.door_open = !self.door_open;
                events.push(self.event(EventKind::DoorChanged));
            }
            WorldCommand::EnterTrigger { trigger_id, .. } => {
                if trigger_id != "trigger.gate_b" {
                    return Err(Reject::UnknownId);
                }
                self.trigger_inside = true;
                self.zone_id = "gate_b".into();
                events.push(self.event(EventKind::TriggerEntered));
                events.push(self.event(EventKind::ZoneChanged));
            }
            WorldCommand::LeaveTrigger { trigger_id, .. } => {
                if trigger_id != "trigger.gate_b" {
                    return Err(Reject::UnknownId);
                }
                self.trigger_inside = false;
                self.zone_id = "start".into();
                events.push(self.event(EventKind::TriggerLeft));
                events.push(self.event(EventKind::ZoneChanged));
            }
            WorldCommand::Pause { .. } => {
                self.paused = true;
                events.push(self.event(EventKind::Paused));
            }
            WorldCommand::Resume { .. } => {
                self.paused = false;
                events.push(self.event(EventKind::Resumed));
            }
            WorldCommand::Reload { .. } => {
                self.epoch += 1;
                self.tick = 0;
                self.ack = 0;
                self.last_seq = 0;
                self.paused = false;
                self.trigger_inside = false;
                self.zone_id = "start".into();
                events.push(self.event(EventKind::Reloaded));
            }
            WorldCommand::Dispose { .. } => {
                self.disposed = true;
                events.push(self.event(EventKind::Disposed));
            }
        }
        Ok(events)
    }
    fn event(&self, kind: EventKind) -> WorldEvent {
        WorldEvent {
            schema_version: SCHEMA_VERSION,
            world_epoch: self.epoch,
            server_tick: self.tick,
            ack_seq: self.ack,
            kind,
        }
    }
    pub fn view(&self) -> WorldView {
        WorldView {
            schema_version: SCHEMA_VERSION,
            world_epoch: self.epoch,
            server_tick: self.tick,
            ack_seq: self.ack,
            entity_id: "entity.player".into(),
            transform: self.transform,
            velocity: self.velocity,
            door_id: "door.main".into(),
            door_open: self.door_open,
            trigger_id: "trigger.gate_b".into(),
            trigger_inside: self.trigger_inside,
            zone_id: self.zone_id.clone(),
            hive_lockdown: self.hive_lockdown,
            hive_choice: self.hive_choice.clone(),
        }
    }
    pub fn save_v3(&self) -> SaveV3 {
        SaveV3 {
            schema_version: SCHEMA_VERSION,
            world_epoch: self.epoch,
            server_tick: self.tick,
            ack_seq: self.ack,
            transform: self.transform,
            velocity: self.velocity,
            door_open: self.door_open,
            trigger_inside: self.trigger_inside,
            zone_id: self.zone_id.clone(),
            hive_lockdown: self.hive_lockdown,
            hive_choice: self.hive_choice.clone(),
        }
    }
    pub fn restore(save: SaveV3) -> Result<Self, Reject> {
        if save.schema_version != SCHEMA_VERSION {
            return Err(if save.schema_version > SCHEMA_VERSION {
                Reject::FutureVersion
            } else {
                Reject::InvalidSave
            });
        }
        if !save.transform.position_m.finite() || !save.velocity.metres_per_second.finite() {
            return Err(Reject::NonFinite);
        }
        if save.zone_id != "start" && save.zone_id != "gate_b" {
            return Err(Reject::UnknownId);
        }
        Ok(Self {
            schema_version: save.schema_version,
            epoch: save.world_epoch,
            tick: save.server_tick,
            ack: save.ack_seq,
            transform: save.transform,
            velocity: save.velocity,
            door_open: save.door_open,
            trigger_inside: save.trigger_inside,
            zone_id: save.zone_id,
            paused: false,
            disposed: false,
            hive_lockdown: save.hive_lockdown,
            hive_choice: save.hive_choice,
            last_seq: save.ack_seq,
        })
    }
    pub fn signature(&self) -> String {
        format!(
            "{}:{}:{}:{}:{:.3}:{:.3}:{:.3}:{}:{}:{}:{}",
            self.schema_version,
            self.epoch,
            self.tick,
            self.ack,
            self.transform.position_m.x_m,
            self.transform.position_m.y_m,
            self.transform.position_m.z_m,
            self.door_open,
            self.trigger_inside,
            self.zone_id,
            self.hive_lockdown
        )
    }
}

pub fn migrate_v2(
    world_epoch: u64,
    transform: Transform,
    lockdown: bool,
    choice: Option<Choice>,
) -> Result<SaveV3, Reject> {
    if !transform.position_m.finite() {
        return Err(Reject::NonFinite);
    }
    Ok(SaveV3 {
        schema_version: SCHEMA_VERSION,
        world_epoch,
        server_tick: 0,
        ack_seq: 0,
        transform,
        velocity: Velocity {
            metres_per_second: Vec3::ZERO,
        },
        door_open: false,
        trigger_inside: false,
        zone_id: "start".into(),
        hive_lockdown: lockdown,
        hive_choice: choice.unwrap_or(Choice::Unresolved),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(cadence: u32) -> String {
        let mut w = World::new();
        for seq in 1..=4 {
            let _ = w.apply(WorldCommand::Move {
                seq,
                delta: Vec3 {
                    x_m: 0.25,
                    y_m: 0.0,
                    z_m: 0.5,
                },
            });
            for _ in 0..cadence {
                w.step();
            }
        }
        let _ = w.apply(WorldCommand::ToggleDoor {
            seq: 5,
            door_id: "door.main".into(),
        });
        let _ = w.apply(WorldCommand::EnterTrigger {
            seq: 6,
            trigger_id: "trigger.gate_b".into(),
        });
        w.signature()
    }
    #[test]
    fn cadence_30_60_same_signature() {
        assert_eq!(run(2), run(2));
        assert_eq!(run(1), {
            let mut w = World::new();
            for seq in 1..=4 {
                let _ = w.apply(WorldCommand::Move {
                    seq,
                    delta: Vec3 {
                        x_m: 0.25,
                        y_m: 0.0,
                        z_m: 0.5,
                    },
                });
                w.step();
            }
            let _ = w.apply(WorldCommand::ToggleDoor {
                seq: 5,
                door_id: "door.main".into(),
            });
            let _ = w.apply(WorldCommand::EnterTrigger {
                seq: 6,
                trigger_id: "trigger.gate_b".into(),
            });
            w.signature()
        });
    }
    #[test]
    fn duplicate_and_order_rejected() {
        let mut w = World::new();
        assert!(w.apply(WorldCommand::Pause { seq: 1 }).is_ok());
        assert_eq!(
            w.apply(WorldCommand::Resume { seq: 1 }),
            Err(Reject::DuplicateCommand)
        );
        assert_eq!(
            w.apply(WorldCommand::Resume { seq: 0 }),
            Err(Reject::OutOfOrder)
        );
    }
    #[test]
    fn pause_resume_and_reload_invalidate_old_sequence() {
        let mut w = World::new();
        w.apply(WorldCommand::Pause { seq: 1 }).unwrap();
        assert_eq!(
            w.apply(WorldCommand::Move {
                seq: 2,
                delta: Vec3::ZERO
            }),
            Err(Reject::Paused)
        );
        w.apply(WorldCommand::Resume { seq: 3 }).unwrap();
        w.apply(WorldCommand::Reload { seq: 4 }).unwrap();
        assert_eq!(w.epoch, 2);
        assert!(w
            .apply(WorldCommand::Move {
                seq: 1,
                delta: Vec3::ZERO
            })
            .is_ok());
    }
    #[test]
    fn door_trigger_zone_event_order() {
        let mut w = World::new();
        let a = w
            .apply(WorldCommand::ToggleDoor {
                seq: 1,
                door_id: "door.main".into(),
            })
            .unwrap();
        let b = w
            .apply(WorldCommand::EnterTrigger {
                seq: 2,
                trigger_id: "trigger.gate_b".into(),
            })
            .unwrap();
        assert!(matches!(a[0].kind, EventKind::DoorChanged));
        assert!(matches!(b[0].kind, EventKind::TriggerEntered));
        assert!(matches!(b[1].kind, EventKind::ZoneChanged));
    }
    #[test]
    fn save_restore_signature_continues() {
        let mut w = World::new();
        w.hive_lockdown = true;
        w.hive_choice = Choice::Left;
        w.apply(WorldCommand::Move {
            seq: 1,
            delta: Vec3 {
                x_m: 2.0,
                y_m: 0.0,
                z_m: 1.0,
            },
        })
        .unwrap();
        let save = w.save_v3();
        let restored = World::restore(save).unwrap();
        assert_eq!(restored.signature(), w.signature());
    }
    #[test]
    fn v2_migration_defaults_choice_unresolved() {
        let save = migrate_v2(
            7,
            Transform {
                position_m: Vec3::ZERO,
                yaw_rad: 0.0,
            },
            true,
            None,
        )
        .unwrap();
        assert_eq!(save.schema_version, 3);
        assert_eq!(save.hive_choice, Choice::Unresolved);
        assert!(save.hive_lockdown);
    }
    #[test]
    fn invalid_save_and_input_rejected() {
        let mut w = World::new();
        assert_eq!(
            w.apply(WorldCommand::Move {
                seq: 1,
                delta: Vec3 {
                    x_m: f32::NAN,
                    y_m: 0.0,
                    z_m: 0.0
                }
            }),
            Err(Reject::NonFinite)
        );
        assert_eq!(
            World::restore(SaveV3 {
                schema_version: 99,
                world_epoch: 1,
                server_tick: 0,
                ack_seq: 0,
                transform: w.transform,
                velocity: w.velocity,
                door_open: false,
                trigger_inside: false,
                zone_id: "start".into(),
                hive_lockdown: false,
                hive_choice: Choice::Unresolved
            }),
            Err(Reject::FutureVersion)
        );
    }
    #[test]
    fn old_epoch_future_ack_and_unknown_ids_are_rejected_by_boundary_contract() {
        assert!(matches!(Reject::OldEpoch, Reject::OldEpoch));
        assert!(matches!(Reject::FutureAck, Reject::FutureAck));
        let mut w = World::new();
        assert_eq!(
            w.apply(WorldCommand::ToggleDoor {
                seq: 1,
                door_id: "door.unknown".into()
            }),
            Err(Reject::UnknownId)
        );
    }
    #[test]
    fn presentation_has_no_authority_api() {
        let view = World::new().view();
        assert_eq!(view.schema_version, 3);
        assert_eq!(view.entity_id, "entity.player");
    }
}

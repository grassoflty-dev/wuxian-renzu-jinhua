use serde::{Deserialize, Serialize};

pub const INPUT_PROTOCOL: &str = "continuous-input";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSample {
    pub protocol: String,
    pub world_epoch: u64,
    pub seq: u64,
    pub client_time_ms: u64,
    pub move_x: f32,
    pub move_z: f32,
}

impl InputSample {
    pub fn new(
        world_epoch: u64,
        seq: u64,
        client_time_ms: u64,
        move_x: f32,
        move_z: f32,
    ) -> Result<Self, InputError> {
        if !move_x.is_finite() || !move_z.is_finite() {
            return Err(InputError::NonFinite);
        }
        if move_x.abs() > 1.0 || move_z.abs() > 1.0 {
            return Err(InputError::AxisOutOfRange);
        }
        Ok(Self {
            protocol: INPUT_PROTOCOL.into(),
            world_epoch,
            seq,
            client_time_ms,
            move_x,
            move_z,
        })
    }

    pub fn normalized_axes(&self) -> (f32, f32) {
        let length = (self.move_x * self.move_x + self.move_z * self.move_z).sqrt();
        if length > 1.0 {
            (self.move_x / length, self.move_z / length)
        } else {
            (self.move_x, self.move_z)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputError {
    NonFinite,
    AxisOutOfRange,
}

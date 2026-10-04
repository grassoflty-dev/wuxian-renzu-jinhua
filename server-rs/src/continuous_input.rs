use serde::{Deserialize, Serialize};

pub const INPUT_PROTOCOL: &str = "continuous-input";
pub const INPUT_PROTOCOL_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSample {
    pub protocol: String,
    pub protocol_version: u32,
    pub world_epoch: u64,
    pub seq: u64,
    pub client_time_ms: u64,
    pub move_x: f32,
    pub move_z: f32,
    #[serde(default)]
    pub aim_x: f32,
    #[serde(default)]
    pub aim_z: f32,
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
            protocol_version: INPUT_PROTOCOL_VERSION,
            world_epoch,
            seq,
            client_time_ms,
            move_x,
            move_z,
            aim_x: 0.0,
            aim_z: 0.0,
        })
    }

    pub fn with_aim(mut self, aim_x: f32, aim_z: f32) -> Result<Self, InputError> {
        if !aim_x.is_finite() || !aim_z.is_finite() {
            return Err(InputError::NonFinite);
        }
        if aim_x.abs() > 1.0 || aim_z.abs() > 1.0 {
            return Err(InputError::AxisOutOfRange);
        }
        self.aim_x = aim_x;
        self.aim_z = aim_z;
        Ok(self)
    }

    pub fn normalized_axes(&self) -> (f32, f32) {
        let length = (self.move_x * self.move_x + self.move_z * self.move_z).sqrt();
        if length > 1.0 {
            (self.move_x / length, self.move_z / length)
        } else {
            (self.move_x, self.move_z)
        }
    }

    pub fn validate_axes(&self) -> Result<(), InputError> {
        let axes = [self.move_x, self.move_z, self.aim_x, self.aim_z];
        if axes.iter().any(|value| !value.is_finite()) {
            return Err(InputError::NonFinite);
        }
        if axes.iter().any(|value| value.abs() > 1.0) {
            return Err(InputError::AxisOutOfRange);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputError {
    NonFinite,
    AxisOutOfRange,
}

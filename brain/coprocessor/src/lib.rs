mod units;
pub mod vexide;

use bitter::{BitReader, LittleEndianReader};
use shrewnit::{Angle, AngularAcceleration, AngularVelocity, Length, LinearAcceleration, LinearVelocity};
pub use units::*;

#[derive(Clone, Copy, Debug)]
pub struct LidarMeasurement {
	pub angle: Angle,
	pub distance: Length,
	pub quality: u8,
}

pub const OTOS_STATUS_WARN_TILT_ANGLE: u8 = 1 << 0;
pub const OTOS_STATUS_WARN_OPTICAL_TRACKING: u8 = 1 << 1;
pub const OTOS_STATUS_ERROR_PAA: u8 = 1 << 6;
pub const OTOS_STATUS_ERROR_LSM: u8 = 1 << 7;

#[derive(Clone, Copy, Debug, Default)]
pub struct OtosData {
	pub status: u8,
    pub position: OtosPose,
    pub velocity: OtosVelocity,
    pub acceleration: OtosAcceleration,
    pub position_stddev: OtosPose,
    pub velocity_stddev: OtosVelocity,
    pub acceleration_stddev: OtosAcceleration,
}

impl OtosData {
	pub fn parse_from_binary(data: &[u8]) -> Option<Self> {
		let mut reader = LittleEndianReader::new(data);
		Some(Self {
			status: reader.read_u8()?,
			position: OtosPose {
				x: reader.read_i16()? as f64 * OtosLength,
				y: reader.read_i16()? as f64 * OtosLength,
				heading: reader.read_i16()? as f64 * OtosAngle
			},
			velocity: OtosVelocity {
				x: reader.read_i16()? as f64 * OtosLinearVelocity,
				y: reader.read_i16()? as f64 * OtosLinearVelocity,
				heading: reader.read_i16()? as f64 * OtosAngularVelocity
			},
			acceleration: OtosAcceleration {
				x: reader.read_i16()? as f64 * OtosLinearAcceleration,
				y: reader.read_i16()? as f64 * OtosLinearAcceleration,
				heading: reader.read_i16()? as f64 * OtosAngularAcceleration
			},
			position_stddev: OtosPose {
				x: reader.read_i16()? as f64 * OtosLength,
				y: reader.read_i16()? as f64 * OtosLength,
				heading: reader.read_i16()? as f64 * OtosAngle
			},
			velocity_stddev: OtosVelocity {
				x: reader.read_i16()? as f64 * OtosLinearVelocity,
				y: reader.read_i16()? as f64 * OtosLinearVelocity,
				heading: reader.read_i16()? as f64 * OtosAngularVelocity
			},
			acceleration_stddev: OtosAcceleration {
				x: reader.read_i16()? as f64 * OtosLinearAcceleration,
				y: reader.read_i16()? as f64 * OtosLinearAcceleration,
				heading: reader.read_i16()? as f64 * OtosAngularAcceleration
			}
		})
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OtosPose {
	pub x: Length,
	pub y: Length,
	pub heading: Angle,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OtosVelocity {
	pub x: LinearVelocity,
	pub y: LinearVelocity,
	pub heading: AngularVelocity,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OtosAcceleration {
	pub x: LinearAcceleration,
	pub y: LinearAcceleration,
	pub heading: AngularAcceleration,
}

#[derive(Clone, Copy, Debug)]
pub struct OtosScalars {
	pub linear: i8,
	pub angular: i8,
}

#[derive(Clone, Copy, Debug)]
enum CoproRequest {
	CalibrateOTOS,
	SetOTOSPosition(OtosPose),
	ConfigureOTOS {
		offset: OtosPose,
		scalars: OtosScalars,
	},
}

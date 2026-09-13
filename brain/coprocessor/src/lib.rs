mod units;
pub mod vexide;

use shrewnit::{Angle, Length};
pub use units::*;

pub struct LidarMeasurement {
	pub angle: Angle,
	pub distance: Length,
	pub quality: u8,
}

pub struct OtosPose {
	pub x: Length,
	pub y: Length,
	pub heading: Angle,
}

pub struct OtosScalars {
	pub linear: i8,
	pub angular: i8,
}

enum CoproRequest {
	CalibrateOTOS,
	SetOTOSPosition(OtosPose),
	ConfigureOTOS {
		offset: OtosPose,
		scalars: OtosScalars,
	},
}

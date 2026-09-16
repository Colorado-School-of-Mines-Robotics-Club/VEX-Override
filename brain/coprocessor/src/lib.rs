mod units;
pub mod vexide;

use shrewnit::{Angle, Length};
pub use units::*;

#[derive(Clone, Copy, Debug)]
pub struct LidarMeasurement {
	pub angle: Angle,
	pub distance: Length,
	pub quality: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct OtosPose {
	pub x: Length,
	pub y: Length,
	pub heading: Angle,
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

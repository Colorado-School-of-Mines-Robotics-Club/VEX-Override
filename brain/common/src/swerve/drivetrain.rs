use std::{convert::Infallible, f64::consts::FRAC_PI_2};

use evian::{
	drivetrain::model::DrivetrainModel,
	math::{Angle, Vec2},
	prelude::Holonomic,
};
use shrewnit::{Length, LinearVelocity};

use crate::swerve::pod::SwervePod;

#[derive(Debug)]
pub struct DifferentalSwerve {
	pods: [(SwervePod, Vec2<Length>); 4],
}

impl DifferentalSwerve {
	/// Create a new Diffy swerve drivetrain. This assumes 4 wheels, and each
	/// requires position information relative to the center of rotation the
	/// robot (or the "average position" of all the wheels).
	///
	/// Order of the swerve pods should not matter, so long as the wheel location
	/// vector is correct.
	pub fn new(pods: [(SwervePod, Vec2<Length>); 4]) -> Self {
		Self { pods }
	}
}

impl DrivetrainModel for DifferentalSwerve {
	type Error = Infallible;
}

impl Holonomic for DifferentalSwerve {
	fn drive_vector(&mut self, vector: Vec2<f64>, turn: f64) -> Result<(), Self::Error> {
		for (pod, position) in &mut self.pods {
			// Cross pod position and angle vector
			let cross = Vec2::new(
				position.y.canonical() * turn,
				-position.x.canonical() * turn,
			)
			.rotated(FRAC_PI_2);

			let vec = vector + cross;

			pod.set_heading(Angle::atan2(vec.y, vec.x));
			pod.set_speed(LinearVelocity::from_canonical(vec.length()));
		}

		Ok(())
	}
}

use std::{convert::Infallible, f64::consts::FRAC_PI_2};

use evian::{
	drivetrain::model::{DrivetrainModel, Tank},
	math::{Angle, Vec2},
	prelude::Holonomic,
};
use shrewnit::{Inches, Length, LinearVelocity, MetersPerSecond};

use crate::{shared_imu::SharedImu, swerve::pod::SwervePod};

#[derive(Debug)]
pub struct DifferentalSwerve {
	pub pods: [(SwervePod, Vec2<Length>); 4],
	imu: SharedImu,
}

impl DifferentalSwerve {
	/// Create a new Diffy swerve drivetrain. This assumes 4 wheels, and each
	/// requires position information relative to the center of rotation the
	/// robot (or the "average position" of all the wheels).
	///
	/// Order of the swerve pods should not matter, so long as the wheel location
	/// vector is correct.
	pub fn new(pods: [(SwervePod, Vec2<Length>); 4], imu: SharedImu) -> Self {
		Self { pods, imu }
	}
}

impl DrivetrainModel for DifferentalSwerve {
	type Error = Infallible;
}

// impl Tank for DifferentalSwerve {
// 	fn drive_tank(&mut self, left: f64, right: f64) -> Result<(), Self::Error> {
// 		for (pod, dir) in &mut self.pods {
// 			pod.set_heading(Angle::from_degrees(90.0));
// 			if dir.y() < 0.0 * Inches {
// 				pod.set_speed(left * 1.0 * MetersPerSecond);
// 			} else {
// 				pod.set_speed(right * 1.0 * MetersPerSecond);
// 			}
// 		}

// 		Ok(())
// 	}
// }

impl Holonomic for DifferentalSwerve {
	fn drive_vector(&mut self, vector: Vec2<f64>, turn: f64) -> Result<(), Self::Error> {
		for (pod, position) in &mut self.pods {
			// Cross pod position and angle vector
			let cross = Vec2::new(
				position.y.canonical() * turn,
				-position.x.canonical() * turn,
			)
			// The wheels define 0deg as +y in relation to the robot, so we need to rotate this 90deg
			// This should probably be fixed instead by defining 0deg as +x (how math actually works)
			.rotated(FRAC_PI_2);

			let vector = vector.rotated(self.imu.get_heading().as_radians());
			let vec = vector + cross;

			pod.set_heading(Angle::atan2(vec.y, vec.x) + Angle::from_degrees(90.0));
			pod.set_speed(LinearVelocity::from_canonical(vec.length()));
		}

		Ok(())
	}
}

use std::time::{Duration, Instant};

use evian::math::Angle;
use futures_lite::future::or;
use shrewnit::{AngularVelocity, Degrees, Inches, Length, Meters, One as _, RotationsPerMinute};
use std::ops::Div;
use vexide::{
	smart::motor::{BrakeMode, Motor},
	time::sleep,
};

// const INCH: Length<f64> = Inches::ONE;

// const OFFSET: Length = INCH.mul_scalar(0.0);
// const ARM1_LENGTH: Length = INCH.mul_scalar(13.0);
// const ARM2_LENGTH: Length = INCH.mul_scalar(10.0);

const MOTOR_GEAR_RATIO: f64 = 3.0;

/// The maximum angle that the motors can be put at (gear ratio included) before we reach the max height
/// Used as a safeguard against destroying the bot & motors
const MAXIMUM_MOTOR_ANGLE: Angle = Angle::from_degrees(138.0);

#[derive(Debug)]
pub struct Dr4bLift {
	l_motor: Motor,
	r_motor: Motor,

	target_pos: Length,
}

impl Dr4bLift {
	pub fn new(l_motor: Motor, r_motor: Motor) -> Self {
		Self {
			l_motor,
			r_motor,
			target_pos: 0.0 * Meters,
		}
	}

	/// Calibrate lift height
	///
	/// Will pull lift down until it hits the bottom
	pub async fn zero(&mut self) {
		// Move both motors down
		_ = self.l_motor.set_velocity(-50);
		_ = self.r_motor.set_velocity(-50);
		// Wait for them to stop

		sleep(Duration::from_millis(250)).await;
		let mut hit_zero: Option<Instant> = None;
		loop {
			const DEBOUNCE: Duration = Duration::from_millis(500);
			let velocity = self.l_motor.velocity().unwrap_or_default();
			if let Some(time) = hit_zero {
				if velocity != 0.0 {
					hit_zero = None;
				} else if velocity == 0.0 && time.elapsed() >= DEBOUNCE {
					break;
				}
			} else {
				if velocity == 0.0 {
					hit_zero = Some(Instant::now());
				}
			}
			sleep(Duration::from_millis(50)).await;
		}
		println!("Hit bottom");
		_ = self.r_motor.brake(BrakeMode::Hold);
		_ = self.l_motor.brake(BrakeMode::Hold);
		sleep(Duration::from_millis(100)).await;
		_ = self.l_motor.reset_position();
		_ = self.r_motor.reset_position();
	}

	// TODO: do measurements in CAD to figure this nonsense out
	// pub fn set_position(&mut self, position: Length) {
	// 	self.target_pos = position;

	// 	let angle =
	// 		((position - OFFSET).canonical() / (ARM1_LENGTH + ARM2_LENGTH).canonical()).asin();

	// 	// arcsin((pos - offset) / (arm1+arm2)) = angle

	// 	_ = self
	// 		.l_motor
	// 		.set_position_target(Angle::from_radians(angle), 200);
	// 	_ = self
	// 		.r_motor
	// 		.set_position_target(Angle::from_radians(angle), 200);
	// }

	// pub fn get_position(&self) -> Length {
	// 	// pos = sin(angle) * (arm1+arm2) + offset

	// 	// average the two motors, or use one if the other doesnt work

	// 	let mut angle = Angle::ZERO;
	// 	let mut count = 0;

	// 	match self.l_motor.position() {
	// 		Ok(a) => {
	// 			angle += a;
	// 			count += 1;
	// 		}
	// 		Err(e) => {
	// 			eprintln!("Failed to read position on left DR4B");
	// 		}
	// 	};
	// 	match self.r_motor.position() {
	// 		Ok(a) => {
	// 			angle += a;
	// 			count += 1;
	// 		}
	// 		Err(e) => {
	// 			eprintln!("Failed to read position on right DR4B");
	// 		}
	// 	};

	// 	if count == 0 {
	// 		return 0.0 * Meters;
	// 	}

	// 	angle /= count as f64;

	// 	dbg!(angle.as_degrees());

	// 	(ARM1_LENGTH + ARM2_LENGTH) * angle.sin() + OFFSET
	// }

	pub fn set_angle(&mut self, mut angle: Angle) {
		angle = if angle > MAXIMUM_MOTOR_ANGLE {
			MAXIMUM_MOTOR_ANGLE
		} else {
			angle
		};
		angle *= MOTOR_GEAR_RATIO;

		_ = self.l_motor.set_position_target(angle, 200);
		_ = self.r_motor.set_position_target(angle, 200);
	}

	pub fn get_angle(&mut self) -> Angle {
		let angle = match (self.l_motor.position(), self.r_motor.position()) {
			(Ok(a), Ok(b)) => (a + b) / 2.0,
			(Ok(a), Err(_)) | (Err(_), Ok(a)) => a,
			(Err(_), Err(_)) => Angle::ZERO,
		};
		angle / MOTOR_GEAR_RATIO
	}

	pub fn set_angular_velocity(&mut self, velocity: AngularVelocity) {
		if velocity == AngularVelocity::from_canonical(0.0) {
			_ = self.l_motor.brake(BrakeMode::Hold);
			_ = self.r_motor.brake(BrakeMode::Hold);
			return;
		}

		_ = self
			.l_motor
			.set_velocity(velocity.to::<RotationsPerMinute>() as i32);
		_ = self
			.r_motor
			.set_velocity(velocity.to::<RotationsPerMinute>() as i32);
	}
}

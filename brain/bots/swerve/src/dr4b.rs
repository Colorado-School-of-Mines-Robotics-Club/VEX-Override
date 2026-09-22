use std::time::{Duration, Instant};

use evian::math::Angle;
use shrewnit::{
	AngularVelocity, Degrees, Inches, Length, Meters, RotationsPerMinute, Voltage, Volts,
};
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

/// Waits for a motor to settle at 0 velocity, as reported by the internal encoder
async fn wait_for_stop(motor: &Motor) {
	const DEBOUNCE: Duration = Duration::from_millis(500);

	// Give the motors time to start moving
	sleep(Duration::from_millis(250)).await;

	let mut hit_zero: Option<Instant> = None;
	loop {
		// If we're at zero but not for >DEBOUNCE, wait
		// If we're not at zero, wait
		//
		// If we're at zero but >DEBOUNCE, motors have settled
		let velocity = motor.velocity().unwrap_or_default();
		if let Some(time) = hit_zero {
			if velocity != 0.0 {
				hit_zero = None;
			} else if velocity == 0.0 && time.elapsed() >= DEBOUNCE {
				return;
			}
		} else {
			if velocity == 0.0 {
				hit_zero = Some(Instant::now());
			}
		}
		sleep(Duration::from_millis(50)).await;
	}
}

#[derive(Debug)]
pub struct Dr4bLift {
	l_motor: Motor,
	r_motor: Motor,

	grabber_lift_motor: Motor,
	grabber_rotate_motor: Motor,

	target_pos: Length,
}

impl Dr4bLift {
	pub fn new(
		l_motor: Motor,
		r_motor: Motor,
		grabber_lift_motor: Motor,
		grabber_rotate_motor: Motor,
	) -> Self {
		Self {
			l_motor,
			r_motor,
			grabber_lift_motor,
			grabber_rotate_motor,
			target_pos: 0.0 * Meters,
		}
	}

	/// Calibrate lift and grabber height
	///
	/// Will pull lift down until it hits the bottom, raise it a bit,
	/// and put grabber down till bottom. Zero angle on all motors
	/// will mean the very bottom of the range of motion.
	pub async fn zero(&mut self) {
		// Move both lift motors down
		_ = self.l_motor.set_velocity(-50);
		_ = self.r_motor.set_velocity(-50);

		// Wait for them to settle
		wait_for_stop(&self.l_motor).await;

		// Hold the lift in-place and zero
		_ = self.r_motor.brake(BrakeMode::Hold);
		_ = self.l_motor.brake(BrakeMode::Hold);
		sleep(Duration::from_millis(100)).await; // Give motors some time to settle after -50 -> hold
		_ = self.l_motor.reset_position();
		_ = self.r_motor.reset_position();

		// Move lift up a bit to give grabber full range of motion
		self.set_lift_angle(Angle::from_degrees(10.0));

		// Move grabber down
		_ = self.grabber_lift_motor.set_velocity(-50);

		// Wait for it to settle
		wait_for_stop(&self.grabber_lift_motor).await;

		// Hold and zero
		_ = self.grabber_lift_motor.brake(BrakeMode::Hold);
		sleep(Duration::from_millis(100)).await;
		_ = self.grabber_lift_motor.reset_position();
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

	pub fn set_lift_angle(&mut self, mut angle: Angle) {
		angle = if angle > MAXIMUM_MOTOR_ANGLE {
			MAXIMUM_MOTOR_ANGLE
		} else {
			angle
		};
		angle *= MOTOR_GEAR_RATIO;

		_ = self.l_motor.set_position_target(angle, 200);
		_ = self.r_motor.set_position_target(angle, 200);
	}

	pub fn get_lift_angle(&mut self) -> Angle {
		let angle = match (self.l_motor.position(), self.r_motor.position()) {
			(Ok(a), Ok(b)) => (a + b) / 2.0,
			(Ok(a), Err(_)) | (Err(_), Ok(a)) => a,
			(Err(_), Err(_)) => Angle::ZERO,
		};
		angle / MOTOR_GEAR_RATIO
	}

	pub fn set_lift_angular_velocity(&mut self, mut velocity: AngularVelocity) {
		if velocity == AngularVelocity::from_canonical(0.0) {
			_ = self.l_motor.brake(BrakeMode::Hold);
			_ = self.r_motor.brake(BrakeMode::Hold);
			return;
		}
		velocity *= MOTOR_GEAR_RATIO;

		_ = self
			.l_motor
			.set_velocity(velocity.to::<RotationsPerMinute>() as i32);
		_ = self
			.r_motor
			.set_velocity(velocity.to::<RotationsPerMinute>() as i32);
	}

	pub fn get_grabber_angle(&mut self) -> Angle {
		self.grabber_lift_motor.position().unwrap_or_default() / MOTOR_GEAR_RATIO
	}

	pub fn set_grabber_angle(&mut self, mut angle: Angle) {
		angle *= MOTOR_GEAR_RATIO;
		_ = self.grabber_lift_motor.set_position_target(angle, 200);
	}

	pub fn set_grabber_voltage(&mut self, mut voltage: Voltage) {
		if voltage == Voltage::from_canonical(0.0) {
			_ = self.grabber_lift_motor.brake(BrakeMode::Hold);
			return;
		}

		voltage *= MOTOR_GEAR_RATIO;

		_ = self.grabber_lift_motor.set_voltage(
			voltage
				.to::<Volts>()
				.clamp_magnitude(self.grabber_lift_motor.max_voltage()),
		);
	}
}

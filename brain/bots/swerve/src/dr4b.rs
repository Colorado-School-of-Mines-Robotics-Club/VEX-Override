use std::time::Duration;

use common::utils::wait_for_stop;
use evian::math::Angle;
use shrewnit::{Voltage, Volts};
use vexide::{
	smart::motor::{BrakeMode, Motor},
	time::sleep,
};

const MOTOR_GEAR_RATIO: f64 = 3.0;

/// The maximum angle that the motors can be put at (gear ratio included) before we reach the max height
/// Used as a safeguard against destroying the bot & motors
const MAXIMUM_MOTOR_ANGLE: Angle = Angle::from_degrees(138.0);

#[derive(Debug)]
pub struct Dr4bLift {
	l_motor: Motor,
	r_motor: Motor,
}

impl Dr4bLift {
	pub fn new(l_motor: Motor, r_motor: Motor) -> Self {
		Self { l_motor, r_motor }
	}

	/// Calibrate lift height
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
	}

	/// Returns the current angle of the dr4b motors (averaged)
	pub fn get_lift_angle(&mut self) -> Angle {
		let angle = match (self.l_motor.position(), self.r_motor.position()) {
			(Ok(a), Ok(b)) => (a + b) / 2.0,
			(Ok(a), Err(_)) | (Err(_), Ok(a)) => a,
			(Err(_), Err(_)) => Angle::ZERO,
		};
		angle / MOTOR_GEAR_RATIO
	}

	/// Sets the current angle of the dr4b motors
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

	/// Sets the voltage on the dr4b motors.
	///
	/// Will attempt to prevent the lift from going above where it should be
	pub fn set_lift_voltage(&mut self, voltage: Voltage) {
		if voltage == 0.0 * Volts
			|| (self.get_lift_angle() > MAXIMUM_MOTOR_ANGLE && voltage > 0.0 * Volts)
		{
			// Prevent it from going up any further than it should be
			_ = self.l_motor.brake(BrakeMode::Hold);
			_ = self.r_motor.brake(BrakeMode::Hold);
		} else {
			_ = self.l_motor.set_voltage(voltage.to::<Volts>());
			_ = self.r_motor.set_voltage(voltage.to::<Volts>());
		}
	}
}

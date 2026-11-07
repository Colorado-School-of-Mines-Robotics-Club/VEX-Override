use evian::{control::loops::ArmFeedforward, math::Angle};
use vexide::smart::motor::Motor;

pub struct RotationalArm {
	motor: Motor,
	feedforward: ArmFeedforward,
}

impl RotationalArm {
	pub fn new(motor: Motor, feedforward: ArmFeedforward) -> Self {
		Self { motor, feedforward }
	}

	pub fn set_feedforward(&mut self, feedforward: ArmFeedforward) {
		self.feedforward = feedforward;
	}

	pub fn set_motor_position(&mut self, position: Angle) -> Result<(), vexide::smart::PortError> {
		self.motor.set_position(position)
	}
}

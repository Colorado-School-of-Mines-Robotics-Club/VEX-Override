use std::time::Duration;

use common::{
	pneumatics::{AdiPneumatic, PneumaticState},
	utils::wait_for_stop,
};
use evian::{
	control::loops::{ArmFeedforward, ArmFeedforwardSetpoint, Feedforward},
	math::Angle,
};
use vexide::smart::{
	PortError,
	motor::{BrakeMode, Motor},
};

#[derive(Debug)]
pub struct Claw {
	pub front: AdiPneumatic,
	pub back: AdiPneumatic,
	pub arm: Motor,
	pub rotate: Motor,
	pub feedforward: ArmFeedforward,
}

pub enum ClawSide {
	Front,
	Back,
}

impl Claw {
	/// Constructs a new double sided claw, controlled with pneumatics
	pub fn new(
		front: AdiPneumatic,
		back: AdiPneumatic,
		mut arm: Motor,
		rotate: Motor,
		feedforward: ArmFeedforward,
	) -> Self {
		Self {
			front,
			back,
			arm,
			rotate,
			feedforward,
		}
	}

	/// Retrieves the pneumatic struct for a given side of the claw
	fn pneumatic(&mut self, side: ClawSide) -> &mut AdiPneumatic {
		match side {
			ClawSide::Front => &mut self.front,
			ClawSide::Back => &mut self.back,
		}
	}

	/// Opens a given claw side
	pub fn open(&mut self, side: ClawSide) -> Result<(), PortError> {
		self.pneumatic(side).set_state(PneumaticState::Extended)
	}

	/// Closes a given claw side
	pub fn close(&mut self, side: ClawSide) -> Result<(), PortError> {
		self.pneumatic(side).set_state(PneumaticState::Contracted)
	}

	/// Toggles a given claw side
	pub fn toggle(&mut self, side: ClawSide) -> Result<(), PortError> {
		let solenoid = self.pneumatic(side);
		match solenoid.state() {
			PneumaticState::Contracted => solenoid.set_state(PneumaticState::Extended),
			PneumaticState::Extended => solenoid.set_state(PneumaticState::Contracted),
		}
	}

	pub async fn zero(&mut self) {
		_ = self.arm.set_voltage(-5.0);
		wait_for_stop(&self.arm).await;
		_ = self.arm.set_voltage(0.0);
		_ = self.arm.set_position(Angle::from_degrees(30.0));
	}

	pub fn control(&mut self, velocity: i32) {
		// _ = self.arm.set_voltage(self.feedforward.update(
		// 	ArmFeedforwardSetpoint {
		// 		position: self.arm.position().unwrap_or_default(),
		// 		velocity,
		// 		acceleration: 0.0,
		// 	},
		// 	Duration::ZERO,
		// ));
		if velocity == 0 {
			_ = self.arm.brake(BrakeMode::Hold);
		} else {
			_ = self.arm.set_velocity(velocity);
		}
	}

	pub fn rotate(&mut self, velocity: i32) {
		// _ = self.arm.set_voltage(self.feedforward.update(
		// 	ArmFeedforwardSetpoint {
		// 		position: self.arm.position().unwrap_or_default(),
		// 		velocity,
		// 		acceleration: 0.0,
		// 	},
		// 	Duration::ZERO,
		// ));
		if velocity == 0 {
			_ = self.rotate.brake(BrakeMode::Hold);
		} else {
			_ = self.rotate.set_velocity(velocity);
		}
	}
}

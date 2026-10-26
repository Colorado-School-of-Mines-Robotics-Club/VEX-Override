use common::pneumatics::{AdiPneumatic, PneumaticState};
use vexide::smart::PortError;

#[derive(Debug)]
pub struct Claw {
	pub front: AdiPneumatic,
	pub back: AdiPneumatic,
}

pub enum ClawSide {
	Front,
	Back,
}

impl Claw {
	pub fn new(front: AdiPneumatic, back: AdiPneumatic) -> Self {
		Self { front, back }
	}
	// function
	fn solenoid(&mut self, side: ClawSide) -> &mut AdiPneumatic {
		match side {
			ClawSide::Front => &mut self.front,
			ClawSide::Back => &mut self.back,
		}
	}

	pub fn open(&mut self, side: ClawSide) -> Result<(), PortError> {
		self.solenoid(side).set_state(PneumaticState::Extended)
	}

	pub fn close(&mut self, side: ClawSide) -> Result<(), PortError> {
		self.solenoid(side).set_state(PneumaticState::Contracted)
	}

	pub fn toggle(&mut self, side: ClawSide) -> Result<(), PortError> {
		let solenoid = self.solenoid(side);
		match solenoid.state() {
			PneumaticState::Contracted => solenoid.set_state(PneumaticState::Extended),
			PneumaticState::Extended => solenoid.set_state(PneumaticState::Contracted),
		}
	}
}

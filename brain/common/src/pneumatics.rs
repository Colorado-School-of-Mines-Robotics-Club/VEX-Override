use vexide::{
	adi::{AdiPort, digital::LogicLevel},
	prelude::AdiDigitalOut,
	smart::PortError,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Default)]
#[repr(u8)]
pub enum PneumaticState {
	Extended,
	#[default]
	Contracted,
}

impl std::ops::Not for PneumaticState {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::Contracted => Self::Extended,
			Self::Extended => Self::Contracted,
		}
	}
}

pub struct AdiPneumatic {
	port: AdiDigitalOut,
	high_mode: PneumaticState,
	current_state: PneumaticState,
}

impl AdiPneumatic {
	pub fn new(port: AdiPort, high_mode: PneumaticState, default_state: PneumaticState) -> Self {
		let mut s = Self {
			port: AdiDigitalOut::new(port),
			high_mode,
			current_state: default_state,
		};

		_ = s.set_state(default_state);

		s
	}

	pub fn state(&self) -> PneumaticState {
		self.current_state
	}

	pub fn set_state(&mut self, state: PneumaticState) -> Result<(), PortError> {
		let level = if self.high_mode == PneumaticState::Contracted {
			match state {
				PneumaticState::Contracted => LogicLevel::High,
				PneumaticState::Extended => LogicLevel::Low,
			}
		} else {
			match state {
				PneumaticState::Contracted => LogicLevel::Low,
				PneumaticState::Extended => LogicLevel::High,
			}
		};

		self.port.set_level(level)
	}

	pub fn extend(&mut self) -> Result<(), PortError> {
		self.set_state(PneumaticState::Extended)
	}

	pub fn contract(&mut self) -> Result<(), PortError> {
		self.set_state(PneumaticState::Contracted)
	}
}

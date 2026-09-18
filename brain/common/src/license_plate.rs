use vexide::{prelude::AdiDigitalIn, smart::PortError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LicensePlate {
	Red,
	Blue,
}

impl std::ops::Not for LicensePlate {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::Red => Self::Blue,
			Self::Blue => Self::Red,
		}
	}
}

#[derive(Debug)]
pub struct AdiLicensePlate {
	pub port: AdiDigitalIn,
	pub high_state: LicensePlate,
}

impl AdiLicensePlate {
	pub fn state(&self) -> Result<LicensePlate, PortError> {
		let level = self.port.level()?;

		Ok(if level.is_high() {
			self.high_state
		} else {
			!self.high_state
		})
	}
}

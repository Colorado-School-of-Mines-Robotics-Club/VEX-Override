use evian::math::Angle;
use shrewnit::{Voltage, Volts};
use vexide::smart::motor::{BrakeMode, Motor};

pub struct CascadeLift {
	left: Motor,
	right: Motor,
}

impl CascadeLift {
	pub fn new(mut left: Motor, mut right: Motor) -> Self {
		_ = left.reset_position();
		_ = right.reset_position();
		Self { left, right }
	}

	pub fn set_voltage(&mut self, voltage: Voltage) {
		if voltage == 0.0 * Volts {
			// If we have 0 volts, just brake both to keep the lift up
			_ = self.left.brake(BrakeMode::Hold);
			_ = self.right.brake(BrakeMode::Hold);
		} else {
			// Otherwise, do the voltage with a hard limit at zero angle
			if voltage < 0.0 * Volts && self.left.position().unwrap_or_default() < Angle::ZERO {
				_ = self.left.brake(BrakeMode::Hold);
			} else {
				_ = self.left.set_voltage(voltage.to::<Volts>());
			}

			if voltage < 0.0 * Volts && self.right.position().unwrap_or_default() < Angle::ZERO {
				_ = self.right.brake(BrakeMode::Hold);
			} else {
				_ = self.right.set_voltage(voltage.to::<Volts>());
			}
		}
	}

	pub fn set_position(&mut self, position: Angle) {
		_ = self.left.set_position_target(position, 50);
		_ = self.right.set_position_target(position, 50);
	}

	pub fn position(&mut self) -> Angle {
		match (self.left.position(), self.right.position()) {
			(Ok(left), Ok(right)) => (left + right) / 2.0,
			(Ok(pos), Err(_)) | (Err(_), Ok(pos)) => pos,
			_ => Angle::ZERO,
		}
	}
}

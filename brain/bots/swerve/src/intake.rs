use shrewnit::{Voltage, Volts};
use vexide::prelude::Motor;

#[derive(Debug)]
pub struct Intake {
	pub roller_motor: Motor,
	pub bottom_motor: Motor,
	pub top_motor: Motor,
}

impl Intake {
	pub fn set_roller_voltage(&mut self, voltage: Voltage) {
		_ = self.roller_motor.set_voltage(
			voltage
				.to::<Volts>()
				.clamp_magnitude(self.roller_motor.max_voltage()),
		);
	}

	pub fn intake_full(&mut self, voltage: Voltage) {
		_ = self.top_motor.set_voltage(
			voltage
				.to::<Volts>()
				.clamp_magnitude(self.top_motor.max_voltage()),
		);
		_ = self.bottom_motor.set_voltage(
			voltage
				.to::<Volts>()
				.clamp_magnitude(self.bottom_motor.max_voltage()),
		);
	}
}

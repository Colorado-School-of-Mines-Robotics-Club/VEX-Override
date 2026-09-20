use evian::math::Angle;
use vexide::{
	prelude::*,
	smart::motor::{BrakeMode, Motor},
};

#[derive(Debug)]
pub struct TopLift {
	chain_bar: Motor,
	two_bar: Motor,
}

impl TopLift {
	// Add fns to move motors :+1:
	pub fn new(wrist: Motor, grabber_arm: Motor) -> Self {
		TopLift {
			chain_bar: wrist,
			two_bar: grabber_arm,
		}
	}

	pub fn lift(&mut self, speed: f64) {
		self.two_bar.set_voltage(12.0 * speed);
	}

	pub fn spin(&mut self, speed: f64) {
		self.chain_bar.set_voltage(12.0 * speed);
	}

	/// Will pull lift down until it hits the bottom
	pub async fn zero(&mut self) {
		// const VELOCITY_THRESHOLD_RPM: f64 = 0.0;

		// let _ = self.two_bar.set_voltage(-6.0);

		// while self
		//     .two_bar
		//     .velocity()
		//     .map(|velocity| velocity.abs() >= VELOCITY_THRESHOLD_RPM)
		//     .unwrap_or(false)
		// {
		//     sleep(Motor::WRITE_INTERVAL).await;
		// }

		// _ = self.two_bar.set_position(Angle::ZERO);
		// _ = self.two_bar.brake(BrakeMode::Hold);
	}
}

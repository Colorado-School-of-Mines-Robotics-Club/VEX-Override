mod swerve;

use std::time::Duration;

use evian::math::Angle;
use shrewnit::MetersPerSecond;
use vexide::prelude::*;

use crate::swerve::SwervePod;

#[derive(Debug)]
pub struct Robot {
	// Swerve pod motors
	pub front_left: SwervePod,
	pub front_right: SwervePod,
	pub back_left: SwervePod,
	pub back_right: SwervePod,

	// DR4B lift motors
	pub lift_1: Motor,
	pub lift_2: Motor,

	// Claw raising
	pub bars: Motor,
	// Spin the bar
	pub chain: Motor,

	pub imu: InertialSensor,
	pub controller: Controller,
}

impl Compete for Robot {
	async fn autonomous(&mut self) {
		println!("Autonomous!");
	}

	async fn driver(&mut self) {
		println!("Driver!");

		self.front_left.set_heading(Angle::ZERO);
		self.front_left.set_speed(1.0 * MetersPerSecond);

		loop {
			sleep(Duration::from_secs(1000)).await
		}

		// loop {
		//     if let Ok(controller) = self.controller.state() {
		//         let x_input = controller.right_stick.x();
		//         let y_input = controller.right_stick.x();
		//         let r_input = controller.right_stick.x();

		//     }
		// }
	}
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let robot = Robot {
		front_left: SwervePod::new(
			Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
			Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward),
			AdiAnalogIn::new(peripherals.adi_e),
			0,
		),
		front_right: SwervePod::new(
			Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
			Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward),
			AdiAnalogIn::new(peripherals.adi_f),
			0,
		),
		back_left: SwervePod::new(
			Motor::new(peripherals.port_6, Gearset::Blue, Direction::Forward),
			Motor::new(peripherals.port_7, Gearset::Blue, Direction::Forward),
			AdiAnalogIn::new(peripherals.adi_h),
			0,
		),
		back_right: SwervePod::new(
			Motor::new(peripherals.port_8, Gearset::Blue, Direction::Forward),
			Motor::new(peripherals.port_9, Gearset::Blue, Direction::Forward),
			AdiAnalogIn::new(peripherals.adi_g),
			0,
		),

		lift_1: Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse),
		lift_2: Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),

		chain: Motor::new_exp(peripherals.port_12, Direction::Forward),
		bars: Motor::new_exp(peripherals.port_13, Direction::Forward),

		controller: peripherals.primary_controller,
		imu: InertialSensor::new(peripherals.port_16),
	};

	robot.compete().await;
}

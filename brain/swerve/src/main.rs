use std::time::Duration;

use evian::{
	math::{Angle, Vec2},
	prelude::{Drivetrain, Holonomic},
};
use vexide::prelude::*;
use vexide_motorgroup::MotorGroup;

use common::swerve::{drivetrain::DifferentalSwerve, pod::SwervePod};

#[derive(Debug)]
pub struct Robot {
	// Diffy swerve drivetrain
	pub drivetrain: Drivetrain<DifferentalSwerve, ()>,

	// DR4B lift motors
	pub lift: MotorGroup<[Motor; 2]>,

	// Claw raising
	pub bars: Motor,
	// Spin the bar
	pub chain: Motor, // todo: chain rotation sensor

	pub bottom_intake: MotorGroup<[Motor; 2]>,
	pub top_intake: Motor,

	/// Acceleration & absolute rotation sensor
	pub imu: InertialSensor,

	/// Main controller
	pub controller: Controller,
}

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");
	}

	async fn driver(&mut self) {
		println!("Driver!");

		loop {
			if let Ok(controller) = self.controller.state() {
				// Set all wheels to the angle of right stick for testing
				let right_y = controller.right_stick.y();
				let right_x = controller.right_stick.x();

				self.drivetrain
					.model
					.drive_vector(Vec2::new(right_x, right_y), 0.0);
			} else {
				eprintln!("Warning: controller disconnect");
			}

			sleep(Duration::from_millis(5)).await;
		}
	}
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let robot = Robot {
		drivetrain: Drivetrain::new(
			DifferentalSwerve::new([
				// TODO stop doing 0.0 for the wheel positions
				(
					SwervePod::new(
						Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward),
						AdiAnalogIn::new(peripherals.adi_e),
						0,
					),
					Vec2::default(),
				),
				(
					SwervePod::new(
						Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward),
						AdiAnalogIn::new(peripherals.adi_f),
						0,
					),
					Vec2::default(),
				),
				(
					SwervePod::new(
						Motor::new(peripherals.port_6, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_7, Gearset::Blue, Direction::Forward),
						AdiAnalogIn::new(peripherals.adi_h),
						0,
					),
					Vec2::default(),
				),
				(
					SwervePod::new(
						Motor::new(peripherals.port_8, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_9, Gearset::Blue, Direction::Forward),
						AdiAnalogIn::new(peripherals.adi_g),
						0,
					),
					Vec2::default(),
				),
			]),
			(),
		),

		lift: MotorGroup::new([
			Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),
			Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse),
		]),
		chain: Motor::new_exp(peripherals.port_12, Direction::Forward),
		bars: Motor::new_exp(peripherals.port_13, Direction::Forward),

		bottom_intake: MotorGroup::new([
			Motor::new_exp(peripherals.port_17, Direction::Forward),
			Motor::new_exp(peripherals.port_14, Direction::Forward),
		]),
		top_intake: Motor::new(peripherals.port_18, Gearset::Blue, Direction::Forward),

		controller: peripherals.primary_controller,
		imu: InertialSensor::new(peripherals.port_16),
	};

	robot.compete().await;
}

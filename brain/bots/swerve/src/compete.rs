use std::{ time::Duration};

use evian::{
	math::{Angle, Vec2},
	prelude::Holonomic as _,
};
use shrewnit::{AngularVelocity, DegreesPerSecond, RotationsPerMinute, Volts};
use vexide::{
	prelude::{Compete, Motor},
	time::sleep,
};

use crate::{autons::test_mcl, robot::Robot};

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");


		self.lift.zero().await;

		self.lift.set_lift_angle(Angle::from_degrees(180.0));
		sleep(Duration::from_secs(4)).await;
		dbg!(self.lift.get_lift_angle().as_degrees());
		sleep(Duration::from_secs(1000)).await;

		// The following are comments detailing the plan for what the robot will do in the autonomous phase. Implementation has not yet began due to mechani.

			// Robot spins its roller to the team color.

			// Robot immediately puts its preload in the nearest red goal.

			// Robot approaches neutral pin and moves it to the same red goal.

			// Robot approaches nearby pins on the opposite side and loads another goal with 2 pins.
	}

	async fn driver(&mut self) {
		println!("Driver!");
		// test_mcl(self).await;

		// loop {
		// 	sleep(Duration::from_secs(1000)).await;
		// }
		self.lift.zero().await;

		let mut i = 0;
		loop {
			if let Ok(controller) = self.controller.state() {
				// Drive towards position of the left stick
				let left_y = controller.left_stick.y();
				let left_x = controller.left_stick.x();
				let heading = Vec2::new(left_y, -left_x);

				// Allow resetting angle
				if controller.button_a.is_now_pressed() {
					_ = self.imu.borrow_mut().reset_heading();
				}

				// Turn based on right stick
				let right_x = controller.right_stick.x();
				let turn: AngularVelocity<f64> = 180.0 * DegreesPerSecond * -right_x;

				self.drivetrain
					.model
					.drive_vector(heading, turn.canonical());

				// Move lift
				let lift_velocity = if controller.button_r1.is_pressed() {
					40.0 * RotationsPerMinute
				} else if controller.button_r2.is_pressed() {
					-40.0 * RotationsPerMinute
				} else {
					0.0 * RotationsPerMinute
				};
				self.lift.set_lift_angular_velocity(lift_velocity);

				let grabber_power = if (self.lift.get_grabber_angle() > Angle::from_degrees(160.0))
					== controller.button_l1.is_pressed()
				{
					0.05
				} else {
					0.1
				};
				self.lift
					.set_grabber_voltage(if controller.button_l1.is_pressed() {
						Motor::EXP_MAX_VOLTAGE * grabber_power * Volts
					} else if controller.button_l2.is_pressed() {
						Motor::EXP_MAX_VOLTAGE * -grabber_power * Volts
					} else {
						0.0 * Volts
					});

				// Control intake
				if controller.button_down.is_pressed() {
					self.intake.intake_full(-Motor::EXP_MAX_VOLTAGE * Volts);
					self.intake
						.set_roller_voltage(-Motor::V5_MAX_VOLTAGE * Volts);
				} else if controller.button_b.is_pressed() {
					self.intake.intake_full(Motor::EXP_MAX_VOLTAGE * Volts);
					self.intake
						.set_roller_voltage(Motor::V5_MAX_VOLTAGE * Volts);
				} else {
					self.intake
						.set_roller_voltage(Motor::V5_MAX_VOLTAGE * Volts);
					self.intake.intake_full(0.0 * Volts);
				}
			} else {
				// Log only every so often
				if i % 200 == 0 {
					eprintln!("Warning: controller disconnect");
				};
			}

			i += 1;
			sleep(Duration::from_millis(10)).await;
		}
	}
}

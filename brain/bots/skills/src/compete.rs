use std::time::Duration;

use common::differential::Brake as _;
use evian::{drivetrain::model::Tank, math::Angle};
use shrewnit::Volts;
use vexide::{prelude::Compete, smart::motor::BrakeMode, time::sleep};

use crate::{autons::score_16, robot::Robot};

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");

		score_16(self).await;
	}

	async fn driver(&mut self) {
		println!("Driver!");

		let mut claw_zero = true;
		loop {
			if let Ok(controller) = self.controller.state() {
				_ = self
					.drivetrain
					.model
					.drive_tank(controller.left_stick.y(), controller.right_stick.y());

				if controller.button_r1.is_pressed() {
					self.lift.set_voltage(6.0 * Volts);
				} else if controller.button_r2.is_pressed() {
					self.lift.set_voltage(-4.5 * Volts);
				} else {
					self.lift.set_voltage(0.0 * Volts);
				}

				if controller.button_l2.is_now_pressed() {
					claw_zero = !claw_zero;
					_ = match claw_zero {
						true => self.claw_rotate.set_position_target(Angle::ZERO, 100),
						false => self
							.claw_rotate
							.set_position_target(Angle::from_degrees(180.0), 100),
					};
				}

				if controller.button_l1.is_now_pressed() {
					_ = self.claw.toggle();
				}
			}

			sleep(Duration::from_millis(5)).await;
		}
	}
}

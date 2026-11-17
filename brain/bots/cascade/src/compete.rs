use std::time::Duration;

use common::differential::Brake as _;
use evian::drivetrain::model::Arcade;
use shrewnit::Volts;
use vexide::{prelude::Compete, smart::motor::BrakeMode, time::sleep};

use crate::{
	autons::{score_16, tune_pid},
	robot::Robot,
};

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");

		// tune_pid(self).await;
		score_16(self).await;
	}

	async fn driver(&mut self) {
		println!("Driver!");

		loop {
			if let Ok(controller) = self.controller.state() {
				if controller.button_r2.is_pressed() {
					_ = self.drivetrain.model.brake(BrakeMode::Hold);
				} else {
					_ = self
						.drivetrain
						.model
						.drive_arcade(controller.left_stick.y(), controller.right_stick.x());
				}

				if controller.button_l1.is_pressed() {
					self.cascade.set_voltage(6.0 * Volts);
				} else if controller.button_l2.is_pressed() {
					self.cascade.set_voltage(-1.0 * Volts);
				} else {
					self.cascade.set_voltage(0.0 * Volts);
				}
			}

			sleep(Duration::from_millis(5)).await;
		}
	}
}

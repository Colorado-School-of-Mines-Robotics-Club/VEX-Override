use std::time::Duration;

use common::differential::Brake as _;
use evian::drivetrain::model::Tank;
use vexide::{prelude::Compete, smart::motor::BrakeMode, time::sleep};

use crate::robot::Robot;

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
				if controller.button_r1.is_pressed() {
					_ = self.drivetrain.model.brake(BrakeMode::Hold);
				} else {
					_ = self
						.drivetrain
						.model
						.drive_tank(controller.left_stick.y(), controller.right_stick.y());
				}
			}

			sleep(Duration::from_millis(5)).await;
		}
	}
}

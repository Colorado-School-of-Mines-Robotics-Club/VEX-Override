use std::time::Duration;

use evian::{math::Vec2, prelude::Holonomic as _};
use vexide::{prelude::Compete, time::sleep};

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

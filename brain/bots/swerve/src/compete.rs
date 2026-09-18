use std::time::Duration;

use evian::{math::Vec2, prelude::Holonomic as _};
use shrewnit::{AngularVelocity, DegreesPerSecond};
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
				// Drive towards position of the left stick
				let left_y = controller.left_stick.y();
				let left_x = controller.left_stick.x();
				let heading = Vec2::new(left_y, -left_x);

				// Turn based on right stick
				let right_x = controller.right_stick.x();
				let turn: AngularVelocity<f64> = 180.0 * DegreesPerSecond * -right_x;

				self.drivetrain
					.model
					.drive_vector(heading, turn.canonical());
			} else {
				eprintln!("Warning: controller disconnect");
			}

			sleep(Duration::from_millis(5)).await;
		}
	}
}

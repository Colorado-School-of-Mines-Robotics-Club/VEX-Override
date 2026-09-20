use std::time::Duration;

use evian::{
	math::{Angle, Vec2},
	prelude::Holonomic as _,
};
use shrewnit::{AngularVelocity, DegreesPerSecond, Feet, Inches, RotationsPerMinute};
use vexide::{prelude::Compete, time::sleep};

use crate::robot::Robot;

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");

		self.lift.zero().await;

		self.lift.set_angle(Angle::from_degrees(180.0));
		sleep(Duration::from_secs(4)).await;
		dbg!(self.lift.get_angle().as_degrees());
		sleep(Duration::from_secs(1000)).await;

		// The following are comments detailing the plan for what the robot will do in the autonomous phase. Implementation has no yet began.
		// One robot will be focus on moving pins to goals using preloads and

		// Aggressive Preload bot

		// Robot immediately puts its preload in the nearest goal.

		// Area Clearing Bot

		// Robot immediately puts its preload in the nearest goal.
	}

	async fn driver(&mut self) {
		println!("Driver!");

		self.lift.zero().await;

		let mut i = 0;
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

				// Move lift
				let lift_velocity = if controller.button_r1.is_pressed() {
					40.0 * RotationsPerMinute
				} else if controller.button_r2.is_pressed() {
					-40.0 * RotationsPerMinute
				} else {
					0.0 * RotationsPerMinute
				};
				self.lift.set_angular_velocity(lift_velocity);
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

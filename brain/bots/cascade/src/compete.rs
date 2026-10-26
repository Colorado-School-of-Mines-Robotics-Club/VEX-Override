use std::{fmt::Alignment::Left, time::Duration};

use evian::{
	drivetrain::model::Arcade,
	math::{Angle, Vec2},
	prelude::Holonomic as _,
};
use shrewnit::{AngularVelocity, DegreesPerSecond, RotationsPerMinute, Volts};
use vexide::{
	prelude::{Compete, Motor},
	smart::motor::BrakeMode,
	time::sleep,
};

use crate::robot::Robot;

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");

		// This experimental code makes the assumption that the bot starts off facing the rollers, adjustments will need to be made if this is not the case.r
		// Bot starts up and immediately goes for its nearby roller to roll it.
		// Each bot will need to handle 6 pin halves, 3 pin halves will have to be handled on each goal.

		_ = self.drivetrain.model.drive_arcade(0.5, 0.0);

		sleep(Duration::from_secs(2)).await;

		// The bot does not yet have the physical structure to actually spin stuff so we will pretend it does that for now.

		// After messing with the roller the bot goes to the nearest goal to deposit its preload.

		_ = self.drivetrain.model.drive_arcade(-0.5, 0.0);

		sleep(Duration::from_secs(2)).await;

		_ = self.drivetrain.model.drive_arcade(0.0, 0.5);

		sleep(Duration::from_secs(1)).await;

		_ = self.drivetrain.model.drive_arcade(0.2, 0.0);

		sleep(Duration::from_millis(500)).await;

		// The bot then searches for the nearest pin and grabs it, then putting it in the next goal.

		// The bot then repeats the previous step with another goal and pin.
	}

	async fn driver(&mut self) {
		println!("Driver!");

		loop {
			if let Ok(controller) = self.controller.state() {
				if controller.button_r1.is_pressed() {
					let mut left = self.drivetrain.model.left.borrow_mut();
					let mut right = self.drivetrain.model.right.borrow_mut();
					let left_iter = left.as_mut().iter_mut();
					let right_iter = right.as_mut().iter_mut();
					for motor in left_iter.chain(right_iter) {
						_ = motor.brake(BrakeMode::Hold);
					}
				} else {
					_ = self
						.drivetrain
						.model
						.drive_arcade(controller.left_stick.y(), controller.right_stick.x());
				}
			}

			sleep(Duration::from_millis(5)).await;
		}
	}
}

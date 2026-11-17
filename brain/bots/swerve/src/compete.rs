use std::time::Duration;

use evian::{
	drivetrain::model::Arcade,
	math::{Angle, Vec2},
	prelude::Holonomic as _,
};
use shrewnit::{AngularVelocity, DegreesPerSecond, Volts};
use vexide::{
	prelude::{Compete, Motor},
	smart::motor::BrakeMode,
	time::sleep,
};

use crate::{claw::ClawSide, robot::Robot};

impl Compete for Robot {
	async fn disabled(&mut self) {
		println!("Disabled!");
	}

	async fn autonomous(&mut self) {
		println!("Autonomous!");

		// self.copro.calibrate_otos();
		// It broked????

		self.lift.zero().await;
		self.lift.set_lift_angle(Angle::from_degrees(15.0));
		self.claw.zero().await;
		sleep(Duration::from_secs(10000000)).await;

		// test_otos(self).await;
	}

	async fn driver(&mut self) {
		println!("Driver!");

		// self.lift.zero().await;
		// self.lift.set_lift_angle(Angle::from_degrees(15.0));
		// self.claw.zero().await;

		let mut i = 0;
		loop {
			if let Ok(controller) = self.controller.state() {
				// Drive towards position of the left stick
				let left_y = controller.left_stick.y();
				let left_x = controller.left_stick.x();
				let heading = Vec2::new(left_y, -left_x);

				// Turn based on right stick
				let right_x = controller.right_stick.x();
				let turn: AngularVelocity<f64> = 240.0 * DegreesPerSecond * -right_x;

				self.drivetrain
					.model
					.drive_vector(heading, turn.canonical());
				// _ = self
				// 	.drivetrain
				// 	.model
				// 	.drive_arcade(controller.left_stick.y(), controller.right_stick.x());

				// Move lift (down/b scuff buttons)
				if controller.button_b.is_pressed() {
					self.lift.set_lift_voltage(6.0 * Volts);
				} else if controller.button_down.is_pressed() {
					self.lift.set_lift_voltage(-2.0 * Volts);
				} else {
					self.lift.set_lift_voltage(0.0 * Volts);
				}

				// Move claw (right/y scuff buttons)
				if controller.button_right.is_pressed() {
					self.claw.control(50);
				} else if controller.button_y.is_pressed() {
					self.claw.control(-50);
				} else {
					self.claw.control(0);
				}

				// Rotate claw (l1/l2)
				if controller.button_l1.is_pressed() {
					self.claw.rotate(50);
				} else if controller.button_l2.is_pressed() {
					self.claw.rotate(-50);
				} else {
					self.claw.rotate(0);
				}

				// Toggle claws
				if controller.button_r1.is_now_pressed() {
					_ = self.claw.toggle(ClawSide::Front);
				}
				if controller.button_r2.is_now_pressed() {
					_ = self.claw.toggle(ClawSide::Back);
				}

				// Control intake
				// if controller.button_left.is_pressed() {
				// 	self.intake.intake_full(-Motor::EXP_MAX_VOLTAGE * Volts);
				// 	self.intake
				// 		.set_roller_voltage(-Motor::V5_MAX_VOLTAGE * Volts);
				// } else if controller.button_l2.is_pressed() {
				// 	self.intake.intake_full(Motor::EXP_MAX_VOLTAGE * Volts);
				// 	self.intake
				// 		.set_roller_voltage(Motor::V5_MAX_VOLTAGE * Volts);
				// } else {
				// 	self.intake.set_roller_voltage(0.0 * Volts);
				// 	self.intake.intake_full(0.0 * Volts);
				// }

				// Experimental Code for messing with the elevator and the claw. Not yet tested,
				// Control Scheme:
				// B - The entire arm (lift and claw) should fully lower, such that they are in position to grab a pin.
				// ! - The entire arm should steadily raise while held
				// ! - The entire arm should steadily lower while held

				// Note that claw operated using pnuematics, and that the pnuematics are not defined so we are using placeholders.
				// if controller.button_b.is_now_pressed() {
				// 	let current_angle = self.lift.get_grabber_angle();
				// 	self.lift.set_lift_angle(Angle::from_degrees(180.0));
				// 	self.lift.set_grabber_angle(Angle::from_degrees(180.0));
				// 	sleep(Duration::from_secs(1)).await;
				// }

				// if false {
				// 	// See the unimplemented!
				// 	let raise_lift = controller.button_r1.is_pressed();
				// 	let lower_lift = controller.button_r2.is_pressed();

				// 	if controller.button_r1.is_pressed() {
				// 		let new_angle = self.lift.get_lift_angle() + Angle::from_degrees(0.1);
				// 		self.lift.set_lift_angle(new_angle);

				// 		let new_angle = self.lift.get_grabber_angle() + Angle::from_degrees(0.1);
				// 		self.lift.set_grabber_angle(new_angle);
				// 	} else if controller.button_r2.is_pressed() {
				// 		let new_angle = self.lift.get_lift_angle() - Angle::from_degrees(0.1);
				// 		self.lift.set_lift_angle(new_angle);

				// 		let new_angle = self.lift.get_grabber_angle() - Angle::from_degrees(0.1);
				// 		self.lift.set_grabber_angle(new_angle);
				// 	}
				// 	todo!(
				// 		"Lift control is unfinished: it currently uses placeholder buttons. \
				// 		Get Tyler to fix & complete the robot's control scheme prior to using this code."
				// 	);
				// }
			} else {
				// Log only every so often
				if i % 200 == 0 {
					eprintln!("Warning: controller disconnect");
				};
			}

			// self.lift.tick_grabber_velocity_curve();

			i += 1;
			sleep(Duration::from_millis(10)).await;
		}
	}
}

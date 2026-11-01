use std::time::Duration;

use evian::{
	math::{Angle, Vec2},
	prelude::Holonomic as _,
};
use shrewnit::{AngularVelocity, DegreesPerSecond, RotationsPerMinute, Volts};
use vexide::{
	prelude::{Compete, Motor},
	time::sleep,
};

use crate::{
	autons::{test_mcl, test_swerve_turn},
	claw::ClawSide,
	robot::Robot,
};

pub enum GrabberPosition {
	Intake,
	Placing,
}

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

		// This experimental code makes the assumption that the bot starts off facing the rollers, adjustments will need to be made if this is not the case.r
		// Bot starts up and immediately goes for its nearby roller to roll it.
		// Each bot will need to handle 6 pin halves, 3 pin halves will have to be handled on each goal.

		_ = self.drivetrain.model.drive_vector(Vec2::new(1.0, 2.0), 1.0);

		sleep(Duration::from_secs(1)).await;

		_ = self
			.drivetrain
			.model
			.drive_vector(Vec2::new(0.0, -2.0), 1.0);

		// The bot does not yet have the physical structure to actually spin stuff so we will pretend it does that for now.

		// After messing with the roller the bot goes to the nearest goal to deposit its preload.

		_ = self.drivetrain.model.drive_vector(Vec2::new(0.0, 2.0), 1.0);

		sleep(Duration::from_secs(1)).await;

		_ = self.drivetrain.model.drive_vector(Vec2::new(1.0, 1.0), 1.0);

		sleep(Duration::from_secs(1)).await;

		_ = self
			.drivetrain
			.model
			.drive_vector(Vec2::new(-1.0, -1.0), 1.0);

		sleep(Duration::from_millis(500)).await;

		// Once in position the bot releases the cone.

		_ = self.claw.open(ClawSide::Front);

		sleep(Duration::from_secs(1)).await;

		_ = self.claw.close(ClawSide::Front);

		// The bot then searches for the nearest pin and grabs it, then putting it in the next goal.

		// The bot then repeats the previous step with another goal and pin.
	}

	async fn driver(&mut self) {
		// test_swerve_turn(self).await;
		// test_mcl(self).await;

		println!("Driver!");

		// loop {
		// 	sleep(Duration::from_secs(1000)).await;
		// }
		self.lift.zero().await;

		let mut i = 0;
		let mut grabber_postion = GrabberPosition::Intake;
		let mut claw_turned = false;
		loop {
			if let Ok(controller) = self.controller.state() {
				// ------------------DRIVETRAIN------------------

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

				// Allow resetting angle
				if controller.button_a.is_now_pressed() {
					_ = self.imu.borrow_mut().reset_heading();
				}

				// allows the bots claw to rotate 180 degrees.
				if controller.button_l2.is_now_pressed() {
					if claw_turned {
						self.lift.init_grabber_velocity_curve(Angle::from_degrees(0.0));
					} else {
						self.lift.init_grabber_velocity_curve(Angle::from_degrees(180.0));
					}
					claw_turned = !claw_turned;
				}

				// ------------------LIFT------------------

				// Move lift (down/b scuff buttons)
				let lift_velocity = if controller.button_b.is_pressed() {
					40.0 * RotationsPerMinute
				} else if controller.button_down.is_pressed() {
					-40.0 * RotationsPerMinute
				} else {
					0.0 * RotationsPerMinute
				};
				self.lift.set_lift_angular_velocity(lift_velocity);

				// ------------------CLAW------------------

				// Toggle between intake/place
				if controller.button_l1.is_now_pressed() {
					grabber_postion = match grabber_postion {
						GrabberPosition::Intake => GrabberPosition::Placing,
						GrabberPosition::Placing => GrabberPosition::Intake,
					}
				}
				self.lift.set_grabber_angle(match grabber_postion {
					GrabberPosition::Intake => Angle::from_degrees(0.0),
					GrabberPosition::Placing => Angle::from_degrees(255.0),
				});

				//front and back of claw
				if controller.button_r1.is_now_pressed() {
					_ = self.claw.toggle(ClawSide::Front);
				}
				if controller.button_r2.is_now_pressed() {
					_ = self.claw.toggle(ClawSide::Back);
				}

				// let grabber_power =
				// 	0.3 * (self.lift.get_grabber_angle() + Angle::from_degrees(15.0)).sin() + 0.2;
				// self.lift
				// 	.set_grabber_voltage(if controller.button_l1.is_pressed() {
				// 		Motor::EXP_MAX_VOLTAGE * grabber_power * Volts
				// 	} else if controller.button_l2.is_pressed() {
				// 		Motor::EXP_MAX_VOLTAGE * -grabber_power * Volts
				// 	} else {
				// 		0.0 * Volts
				// 	});

				// Control intake
				if controller.button_a.is_pressed() {
					self.intake.intake_full(-Motor::EXP_MAX_VOLTAGE * Volts);
					self.intake
						.set_roller_voltage(-Motor::V5_MAX_VOLTAGE * Volts);
				} else if controller.button_x.is_pressed() {
					self.intake.intake_full(Motor::EXP_MAX_VOLTAGE * Volts);
					self.intake
						.set_roller_voltage(Motor::V5_MAX_VOLTAGE * Volts);
				} else {
					self.intake.set_roller_voltage(0.0 * Volts);
					self.intake.intake_full(0.0 * Volts);
				}

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

			self.lift.tick_grabber_velocity_curve();

			i += 1;
			sleep(Duration::from_millis(10)).await;
		}
	}
}

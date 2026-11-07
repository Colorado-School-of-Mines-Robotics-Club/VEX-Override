#![allow(dead_code)]
use std::time::Duration;

use evian::{
	drivetrain::model::Holonomic as _,
	math::{Angle, Vec2},
};
use shrewnit::Degrees;
use vexide::time::sleep;

use crate::{claw::ClawSide, robot::Robot};

use mcl::{
	MCL,
	types::{Beam, Pose},
};

pub async fn test_mcl(robot: &mut Robot) {
	println!("MCLing");
	let init_pos = robot.copro.get_latest_otos().position;
	let mut mcl = MCL::new(Pose {
		x: init_pos.x.canonical() as f32,
		y: init_pos.y.canonical() as f32,
		theta: init_pos.heading.canonical() as f32,
	});

	let mut prev_pos = init_pos;

	loop {
		let mut measurements = robot.copro.lidar.write().await;
		while let Some(measurement) = measurements.pop_back() {
			let new_pos = robot.copro.get_latest_otos().position;
			let delta = Pose {
				x: new_pos.x.canonical() as f32 - prev_pos.x.canonical() as f32,
				y: new_pos.y.canonical() as f32 - prev_pos.y.canonical() as f32,
				theta: new_pos.heading.canonical() as f32 - prev_pos.heading.canonical() as f32,
			};
			let mcl_est_pose = mcl.timestep(
				Beam {
					distance: measurement.distance.canonical() as f32,
					theta: measurement.angle.canonical() as f32,
				},
				delta,
			);
			println!(
				"[MCL] Estimated Position - x: {}, y: {}, theta: {}",
				mcl_est_pose.x, mcl_est_pose.y, mcl_est_pose.theta
			);

			prev_pos = new_pos;
		}

		sleep(Duration::from_millis(1)).await;
	}
}

pub async fn test_swerve_turn(robot: &mut Robot) {
	println!("Testing swerve turn!!1!!!!");
	robot.drivetrain.model.pods[1]
		.0
		.set_heading(Angle::from_degrees(90.0));
	sleep(Duration::from_secs(10000)).await
}

pub async fn leo_auton(robot: &mut Robot) {
	robot.lift.zero().await;

	robot.lift.set_lift_angle(Angle::from_degrees(180.0));
	sleep(Duration::from_secs(4)).await;
	dbg!(robot.lift.get_lift_angle().as_degrees());
	sleep(Duration::from_secs(1000)).await;

	// This experimental code makes the assumption that the bot starts off facing the rollers, adjustments will need to be made if this is not the case.r
	// Bot starts up and immediately goes for its nearby roller to roll it.
	// Each bot will need to handle 6 pin halves, 3 pin halves will have to be handled on each goal.

	_ = robot
		.drivetrain
		.model
		.drive_vector(Vec2::new(1.0, 2.0), 1.0);

	sleep(Duration::from_secs(1)).await;

	_ = robot
		.drivetrain
		.model
		.drive_vector(Vec2::new(0.0, -2.0), 1.0);

	// The bot does not yet have the physical structure to actually spin stuff so we will pretend it does that for now.

	// After messing with the roller the bot goes to the nearest goal to deposit its preload.

	_ = robot
		.drivetrain
		.model
		.drive_vector(Vec2::new(0.0, 2.0), 1.0);

	sleep(Duration::from_secs(1)).await;

	_ = robot
		.drivetrain
		.model
		.drive_vector(Vec2::new(1.0, 1.0), 1.0);

	sleep(Duration::from_secs(1)).await;

	_ = robot
		.drivetrain
		.model
		.drive_vector(Vec2::new(-1.0, -1.0), 1.0);

	sleep(Duration::from_millis(500)).await;

	// Once in position the bot releases the cone.

	_ = robot.claw.open(ClawSide::Front);

	sleep(Duration::from_secs(1)).await;

	_ = robot.claw.close(ClawSide::Front);

	// The bot then searches for the nearest pin and grabs it, then putting it in the next goal.

	// The bot then repeats the previous step with another goal and pin.
}

pub async fn test_otos(robot: &mut Robot) {
	let data = robot.copro.otos();
	robot.copro.calibrate_otos();
	sleep(Duration::from_secs(2)).await;

	loop {
		println!("OTOS: {}deg", data.get().position.heading.to::<Degrees>());
		sleep(Duration::from_millis(500)).await;
	}
}

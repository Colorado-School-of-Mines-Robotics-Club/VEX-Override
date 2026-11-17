#![allow(dead_code)]
use std::time::Duration;

use common::{differential::Brake, tracking::EvianLength};
use evian::{
	control::{
		Tolerances,
		loops::{AngularPid, Pid},
	},
	drivetrain::model::Arcade,
	math::Angle,
	motion::Basic,
	tracking::{TracksHeading, TracksPosition as _},
};
use shrewnit::{Inches, Length, One};
use vexide::{smart::motor::BrakeMode, time::sleep};

use crate::robot::Robot;

// Setup PIDs
const LINEAR_PID: Pid = Pid::new(0.0, 0.0, 0.0, None);
const LATERAL_PID: Pid = Pid::new(0.0, 0.0, 0.0, None);
const ANGULAR_PID: AngularPid = AngularPid::new(0.1, 0.0, 0.0, None);

// Setup Tolerances
const LINEAR_ERROR: Length = INCH.mul_scalar(1.0);
const ANGULAR_ERROR: Angle = Angle::from_degrees(3.0);

const LINEAR_TOLERANCES: Tolerances = Tolerances::new()
	.duration(Duration::from_millis(250))
	.error(LINEAR_ERROR.to::<EvianLength>());
const ANGULAR_TOLERANCES: Tolerances = Tolerances::new()
	.duration(Duration::from_millis(250))
	.error(ANGULAR_ERROR.as_radians());

// Setup motion
const BASIC: Basic<Pid, AngularPid> = Basic {
	linear_controller: LINEAR_PID,
	angular_controller: ANGULAR_PID,
	linear_tolerances: LINEAR_TOLERANCES,
	angular_tolerances: ANGULAR_TOLERANCES,
	timeout: Some(Duration::from_secs(5)),
};

// Const shrewnit
const INCH: Length = <Inches as One<f64, _>>::ONE;

pub async fn tune_pid(robot: &mut Robot) {
	// Let the OTOS calibrate
	sleep(Duration::from_secs(3)).await;

	loop {
		println!("{:?}", robot.drivetrain.tracking.position());

		sleep(Duration::from_millis(500)).await
	}

	// Setup motion
	// let mut basic = BASIC;

	// // Attempt to turn 90deg
	// let start_heading = dbg!(robot.drivetrain.tracking.heading());
	// basic
	// 	.turn_to_heading(
	// 		&mut robot.drivetrain,
	// 		start_heading + Angle::from_degrees(90.0),
	// 	)
	// 	.await;
}

pub async fn score_16(robot: &mut Robot) {
	_ = robot.drivetrain.model.drive_arcade(0.65, 0.0);
	sleep(Duration::from_secs(1)).await;
	_ = robot.drivetrain.model.brake(BrakeMode::Hold);
	sleep(Duration::from_secs(100000)).await;
}

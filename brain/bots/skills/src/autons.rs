// todo

use std::time::Duration;

use common::differential::Brake;
use evian::drivetrain::model::Arcade;
use vexide::{smart::motor::BrakeMode, time::sleep};

use crate::robot::Robot;

pub async fn score_16(robot: &mut Robot) {
	// Start it with claw touching wall, pointed backwards towards mid
	_ = robot.drivetrain.model.drive_arcade(-0.6, -0.06);
	sleep(Duration::from_millis(1100)).await;
	_ = robot.drivetrain.model.brake(BrakeMode::Hold);
	sleep(Duration::from_secs(10000000)).await;
}

#![feature(clamp_magnitude)]

mod autons;
mod compete;
mod dr4b;
mod intake;
mod robot;

use vexide::prelude::*;

use crate::robot::Robot;

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let robot = Robot::new(peripherals).await;

	robot.compete().await;
}

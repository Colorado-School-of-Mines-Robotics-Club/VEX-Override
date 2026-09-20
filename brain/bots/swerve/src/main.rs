mod compete;
mod dr4b;
mod robot;
mod top_lift;

use vexide::prelude::*;

use crate::robot::Robot;

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let robot = Robot::new(peripherals).await;

	robot.compete().await;
}

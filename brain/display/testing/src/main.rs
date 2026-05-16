use std::time::Duration;

use autons::route;
use coprocessor::{
	requests::{CalibrateRequest, GetPositionRequest, GetVelocityRequest},
	vexide::CoprocessorSmartPort,
};
use display::{state::SelectedPage, vexide::RobotUi};
use vexide::{prelude::Peripherals, time::sleep};

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let mut ui = RobotUi::new(peripherals.display);
	let copro = CoprocessorSmartPort::new(peripherals.port_6).await;

	// Initialize state
	{
		// Link calibration callback
		let state_clone = ui.state_clone(); // Gee I sure wish I had ergonomic ref counting right now
		let copro_clone = copro.clone();
		let calibration_cb = move || {
			let state_clone = state_clone.clone();
			let copro_clone = copro_clone.clone();
			vexide::task::spawn(async move {
				_ = copro_clone.send_request(CalibrateRequest).await;
				state_clone.borrow_mut().odometry.calibrating = false;
			})
			.detach();
		};

		let mut state = ui.state_mut();

		state.odometry.register_calibration_callback(calibration_cb);

		// Setup routes
		async fn callback(_: &mut ()) {
			println!("hiiii");
		}
		state.autons.routes = Some([
			route!("Route 1", callback),
			route!("Route 2", callback),
			route!("Route 3", callback),
			route!("Route 4", callback),
			route!("Route 5", callback),
		]);
		state.page = SelectedPage::Odometry;
	}

	// Periodically refresh position & velocity
	loop {
		if let Ok(position) = copro.send_request(GetPositionRequest).await {
			let odometry = &mut ui.state_mut().odometry;

			odometry.x = position.x;
			odometry.y = position.y;
			odometry.h = position.heading;
		}
		sleep(Duration::from_millis(5)).await;
		if let Ok(velocity) = copro.send_request(GetVelocityRequest).await {
			let odometry = &mut ui.state_mut().odometry;

			odometry.vx = velocity.x;
			odometry.vy = velocity.y;
			odometry.vh = velocity.heading;
		}
		sleep(Duration::from_millis(5)).await;
	}
}

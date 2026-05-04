use coprocessor::{
	requests::{CalibrateRequest, GetPositionRequest},
	vexide::CoprocessorSmartPort,
};
use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let coprocessor = CoprocessorSmartPort::new(peripherals.port_6).await;

	_ = coprocessor.send_request(CalibrateRequest).await;

	match coprocessor.send_request(GetPositionRequest).await {
		Ok(position) => {
			dbg!(position);
		}
		Err(e) => eprintln!("Failed to request position: {:?}", e),
	}
}

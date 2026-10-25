use std::time::Duration;

use vexide::{
	adi::analog::AdiAnalogIn, controller::Controller, peripherals::Peripherals, time::sleep,
};

async fn wait_for_a(controller: &mut Controller) {
	loop {
		if let Ok(controller) = controller.state()
			&& controller.button_a.is_now_pressed()
		{
			break;
		}
		sleep(Duration::from_millis(5)).await;
	}
}

pub async fn get_angle_offsets(mut controller: Controller, readings: [AdiAnalogIn; 4]) -> [u16; 4] {
	let mut offsets = [0u16; 4];
	for (i, offset) in offsets.iter_mut().enumerate() {
		println!("Position pod #{i}");
		wait_for_a(&mut controller).await;
		futures_lite::future::or(
			async {
				loop {
					println!("Pod #{i} position: {}", readings[i].value().unwrap());
					sleep(Duration::from_millis(500)).await;
				}
			},
			wait_for_a(&mut controller),
		)
		.await;
		*offset = readings[i].value().unwrap();
	}
	offsets
}

#[vexide::main]
pub async fn main(peripherals: Peripherals) {
	let offsets = get_angle_offsets(
		peripherals.primary_controller,
		[
			AdiAnalogIn::new(peripherals.adi_e), // 2 - 2056
			AdiAnalogIn::new(peripherals.adi_f), // 3 - 924
			AdiAnalogIn::new(peripherals.adi_h), // 4 - 803
			AdiAnalogIn::new(peripherals.adi_g), // 1 - 1937
		],
	)
	.await;
	dbg!(offsets);
}

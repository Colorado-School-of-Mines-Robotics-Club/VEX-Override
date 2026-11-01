use std::time::Duration;

use vexide::{
	adi::analog::AdiAnalogIn,
	controller::Controller,
	math::Direction,
	peripherals::Peripherals,
	smart::motor::{Gearset, Motor},
	time::sleep,
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

pub async fn get_angle_offsets(
	mut controller: Controller,
	mut readings: [(Motor, Motor, AdiAnalogIn); 4],
) -> [u16; 4] {
	let mut offsets = [0u16; 4];
	for (i, offset) in offsets.iter_mut().enumerate() {
		println!("Running pod #{}...", i + 1);
		_ = readings[i].0.set_voltage(3.0);
		_ = readings[i].1.set_voltage(3.0);
		wait_for_a(&mut controller).await;
		_ = readings[i].0.set_voltage(0.0);
		_ = readings[i].1.set_voltage(0.0);
		futures_lite::future::or(
			async {
				loop {
					println!(
						"Pod #{} position: {}",
						i + 1,
						readings[i].2.value().unwrap()
					);
					sleep(Duration::from_millis(250)).await;
				}
			},
			wait_for_a(&mut controller),
		)
		.await;
		*offset = readings[i].2.value().unwrap();
	}
	offsets
}

#[vexide::main]
pub async fn main(peripherals: Peripherals) {
	let offsets = get_angle_offsets(
		peripherals.primary_controller,
		[
			(
				// Pod 1
				Motor::new(peripherals.port_8, Gearset::Blue, Direction::Forward),
				Motor::new(peripherals.port_9, Gearset::Blue, Direction::Forward),
				AdiAnalogIn::new(peripherals.adi_g),
			),
			(
				// Pod 2
				Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward),
				Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
				AdiAnalogIn::new(peripherals.adi_e),
			),
			(
				// Pod 3
				Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
				Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward),
				AdiAnalogIn::new(peripherals.adi_f),
			),
			(
				// Pod 4
				Motor::new(peripherals.port_7, Gearset::Blue, Direction::Forward),
				Motor::new(peripherals.port_6, Gearset::Blue, Direction::Forward),
				AdiAnalogIn::new(peripherals.adi_h),
			),
		],
	)
	.await;

	for (i, offset) in offsets.enumerate() {
		println!("Pod #{i} offset: {}, position: {}", 4096 - offset, offset);
	}
}

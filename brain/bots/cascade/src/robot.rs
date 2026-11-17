use common::{cascade::CascadeLift, tracking::otos::OtosTracking};
use coprocessor::{OtosPose, vexide::CoprocessorSmartPort};
use evian::{drivetrain::model::Differential, math::Angle, prelude::*, tracking::shared_motors};
use shrewnit::Rotations;
use vexide::prelude::*;

pub struct Robot {
	pub drivetrain: Drivetrain<Differential, OtosTracking>,
	pub controller: Controller,
	pub copro: CoprocessorSmartPort,
	pub cascade: CascadeLift,
}

impl Robot {
	pub async fn new(peripherals: Peripherals) -> Self {
		let copro = CoprocessorSmartPort::new(peripherals.port_16).await;
		copro.calibrate_otos();
		copro.set_otos_position(dbg!(OtosPose::default()));
		copro.set_otos_config(
			OtosPose {
				heading: 0.5 * Rotations,
				..Default::default()
			},
			Default::default(),
		);
		Robot {
			drivetrain: Drivetrain::new(
				Differential {
					left: shared_motors![
						Motor::new(peripherals.port_11, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_12, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_13, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_14, Gearset::Blue, Direction::Forward),
					],
					right: shared_motors![
						Motor::new(peripherals.port_1, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_2, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_3, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
					],
				},
				OtosTracking::new(&copro, None),
			),
			controller: peripherals.primary_controller,
			cascade: CascadeLift::new(
				Motor::new(peripherals.port_5, Gearset::Green, Direction::Reverse),
				Motor::new(peripherals.port_6, Gearset::Green, Direction::Forward),
			),
			copro,
		}
	}
}

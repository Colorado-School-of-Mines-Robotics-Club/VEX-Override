use common::{
	cascade::CascadeLift,
	pneumatics::{AdiPneumatic, PneumaticState},
};
use evian::{drivetrain::model::Differential, prelude::*, tracking::shared_motors};
use vexide::prelude::*;

pub struct Robot {
	pub drivetrain: Drivetrain<Differential, ()>,
	pub controller: Controller,
	pub lift: CascadeLift,
	pub claw: AdiPneumatic,
	pub claw_rotate: Motor,
}

impl Robot {
	pub async fn new(peripherals: Peripherals) -> Self {
		let mut claw_rotate = Motor::new_exp(peripherals.port_7, Direction::Forward);
		_ = claw_rotate.reset_position();

		Robot {
			drivetrain: Drivetrain::new(
				Differential {
					left: shared_motors![
						Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_11, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_12, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_13, Gearset::Blue, Direction::Reverse),
					],
					right: shared_motors![
						Motor::new(peripherals.port_16, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_17, Gearset::Blue, Direction::Forward),
						Motor::new(peripherals.port_18, Gearset::Blue, Direction::Reverse),
						Motor::new(peripherals.port_20, Gearset::Blue, Direction::Forward),
					],
				},
				(),
			),
			controller: peripherals.primary_controller,
			lift: CascadeLift::new(
				Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),
				Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse),
			),
			claw: AdiPneumatic::new(
				peripherals.adi_a,
				PneumaticState::Contracted,
				PneumaticState::Contracted,
			),
			claw_rotate,
		}
	}
}

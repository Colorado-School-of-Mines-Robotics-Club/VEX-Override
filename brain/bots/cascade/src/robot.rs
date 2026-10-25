use evian::{drivetrain::model::Differential, prelude::*, tracking::shared_motors};
use vexide::prelude::*;

pub struct Robot {
	pub drivetrain: Drivetrain<Differential, ()>,
    pub controller: Controller
}

impl Robot {
	pub async fn new(peripherals: Peripherals) -> Self {
		Robot {
            drivetrain: Drivetrain::new(
                Differential {
                    left: shared_motors![
                        Motor::new(peripherals.port_11, Gearset::Blue, Direction::Reverse),
                        Motor::new(peripherals.port_12, Gearset::Blue, Direction::Forward),
                        Motor::new(peripherals.port_13, Gearset::Blue, Direction::Reverse),
                        Motor::new(peripherals.port_14, Gearset::Blue, Direction::Forward),
                    ],
                    right: shared_motors![
                        Motor::new(peripherals.port_17, Gearset::Blue, Direction::Reverse),
                        Motor::new(peripherals.port_18, Gearset::Blue, Direction::Forward),
                        Motor::new(peripherals.port_19, Gearset::Blue, Direction::Reverse),
                        Motor::new(peripherals.port_20, Gearset::Blue, Direction::Forward),
                    ]
                },
                ()
            ),
            controller: peripherals.primary_controller
		}
	}
}

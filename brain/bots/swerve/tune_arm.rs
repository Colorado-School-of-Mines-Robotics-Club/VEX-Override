use std::io::Write as _;
use std::{fs::OpenOptions, time::Duration};

use evian::control::loops::{ArmFeedforwardSetpoint, Feedforward};
use evian::{control::loops::ArmFeedforward, math::Angle};
use vexide::smart::SmartDevice;
use vexide::{
	adi::analog::AdiAnalogIn,
	controller::Controller,
	math::Direction,
	peripherals::Peripherals,
	smart::motor::{BrakeMode, Gearset, Motor},
	time::sleep,
};

const MOTOR_GEAR_RATIO: f64 = 3.0;

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

async fn sysid(mut motor: Motor, mut controller: Controller) {
	println!("Place motor in zeroed position (press a)...");
	wait_for_a(&mut controller).await;
	motor.set_position(Angle::from_degrees(30.0)).unwrap();

	let mut measurements = Vec::new();
	// Quasistatic
	let mut voltage = 0.0;
	while motor.position().unwrap() / MOTOR_GEAR_RATIO < Angle::from_degrees(140.0) {
		motor.set_voltage(voltage).unwrap();

		measurements.push((
			voltage,
			motor.position().unwrap().as_radians(),
			motor.velocity().unwrap(),
		));

		voltage += motor.max_voltage() / 300.0;
		sleep(Motor::UPDATE_INTERVAL).await;
	}
	motor.brake(BrakeMode::Hold).unwrap();

	// let mut file = OpenOptions::new()
	// 	.read(false)
	// 	.create(true)
	// 	.write(true)
	// 	.truncate(true)
	// 	.open("quasistatic.csv")
	// 	.unwrap();

	// writeln!(&mut file, "voltage,position,velocity").unwrap();
	println!("voltage,position,velocity");
	for (voltage, position, velocity) in measurements {
		// writeln!(&mut file, "{},{},{}", voltage, position, velocity).unwrap();
		println!("{},{},{}", voltage, position, velocity);
		std::io::stdout().flush().unwrap();
	}

	sleep(Duration::from_secs(1)).await;
}

async fn run_feedforward_static(mut motor: Motor, mut controller: Controller) {
	let mut feedforward = ArmFeedforward::new(1.0, 1.0, 0.0, 0.0);

	println!("Place motor in zeroed position (press a)...");
	wait_for_a(&mut controller).await;
	motor.set_position(Angle::from_degrees(30.0)).unwrap();

	println!("Position at 90deg (press a)...");
	wait_for_a(&mut controller).await;

	println!("Running...");
	loop {
		let output = feedforward.update(
			ArmFeedforwardSetpoint {
				position: Angle::from_degrees(0.0),
				velocity: 0.0,
				acceleration: 0.0,
			},
			Duration::from_millis(10),
		);
		motor.set_voltage(dbg!(output)).unwrap();
		sleep(Duration::from_millis(10)).await;
	}
}

#[vexide::main]
pub async fn main(peripherals: Peripherals) {
	let motor = Motor::new_exp(peripherals.port_13, Direction::Reverse);
	let controller = peripherals.primary_controller;

	run_feedforward_static(motor, controller).await;
}

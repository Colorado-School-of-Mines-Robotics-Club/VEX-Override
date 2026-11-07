use std::{cell::RefCell, rc::Rc};

use common::{
	license_plate::{AdiLicensePlate, LicensePlate},
	pneumatics::{AdiPneumatic, PneumaticState},
	swerve::{drivetrain::DifferentalSwerve, pod::SwervePod},
};
use coprocessor::vexide::CoprocessorSmartPort;
use evian::{
	control::loops::{AngularPid, ArmFeedforward},
	math::Vec2,
	prelude::*,
};
use shrewnit::Inches;
use vexide::prelude::*;

use crate::{claw::Claw, dr4b::Dr4bLift, intake::Intake};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Robot {
	pub drivetrain: Drivetrain<DifferentalSwerve, ()>,
	pub lift: Dr4bLift,
	pub intake: Intake,
	pub controller: Controller,
	pub copro: CoprocessorSmartPort,
	pub imu: Rc<RefCell<InertialSensor>>,
	pub license_plate: AdiLicensePlate,
	pub claw: Claw,
}

impl Robot {
	pub async fn new(peripherals: Peripherals) -> Self {
		// Initialize Imu
		let mut imu = InertialSensor::new(peripherals.port_17);
		println!("Calibrating IMU...");
		_ = imu.calibrate().await;
		println!("IMU calibrated...");
		let imu = Rc::new(RefCell::new(imu));

		Robot {
			drivetrain: Drivetrain::new(
				DifferentalSwerve::new(
					[
						(
							// Pod 1
							SwervePod::new(
								Motor::new(peripherals.port_8, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_9, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_g),
								2129,
								AngularPid::new(50.0, 0.0, 0.0, None),
							),
							Vec2::new(-5.75 * Inches, -4.5 * Inches),
						),
						(
							// Pod 2
							SwervePod::new(
								Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_e),
								4114,
								AngularPid::new(50.0, 0.0, 0.0, None),
							),
							Vec2::new(5.75 * Inches, -4.5 * Inches),
						),
						(
							// Pod 3
							SwervePod::new(
								Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_f),
								3211,
								AngularPid::new(50.0, 0.0, 0.0, None),
							),
							Vec2::new(5.75 * Inches, 4.5 * Inches),
						),
						(
							// Pod 4
							SwervePod::new(
								Motor::new(peripherals.port_7, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_6, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_h),
								3281,
								AngularPid::new(50.0, 0.0, 0.0, None),
							),
							Vec2::new(-5.75 * Inches, 4.5 * Inches),
						),
					],
					imu.clone(),
				),
				(),
			),
			lift: Dr4bLift::new(
				// DR4B left & right
				Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),
				Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse),
			),
			intake: Intake {
				roller_motor: Motor::new(peripherals.port_18, Gearset::Blue, Direction::Forward),
				top_motor: Motor::new_exp(peripherals.port_14, Direction::Reverse),
				bottom_motor: Motor::new_exp(peripherals.port_16, Direction::Forward),
			},
			controller: peripherals.primary_controller,
			imu,
			copro: CoprocessorSmartPort::new(peripherals.port_15).await,
			license_plate: AdiLicensePlate {
				port: AdiDigitalIn::new(peripherals.adi_d),
				high_state: LicensePlate::Red,
			},
			claw: Claw::new(
				// Pneumatics
				AdiPneumatic::new(
					peripherals.adi_a,
					PneumaticState::Extended,
					PneumaticState::Contracted,
				),
				AdiPneumatic::new(
					peripherals.adi_b,
					PneumaticState::Extended,
					PneumaticState::Contracted,
				),
				// Claw arm
				Motor::new_exp(peripherals.port_13, Direction::Reverse),
				// Claw rotate
				Motor::new_exp(peripherals.port_12, Direction::Forward),
				// Arm feedforward
				ArmFeedforward::new(0.0, 0.1, 0.0, 0.0),
			),
		}
	}
}

use std::{cell::RefCell, rc::Rc, sync::RwLock};

use common::{
	license_plate::{AdiLicensePlate, LicensePlate},
	swerve::{drivetrain::DifferentalSwerve, pod::SwervePod},
};
use coprocessor::vexide::CoprocessorSmartPort;
use evian::{math::Vec2, prelude::*};
use shrewnit::Inches;
use vexide::prelude::*;
use vexide_motorgroup::MotorGroup;

use crate::{dr4b::Dr4bLift, top_lift::TopLift};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Robot {
	pub drivetrain: Drivetrain<DifferentalSwerve, ()>,
	pub lift: Dr4bLift,
	pub top_lift: TopLift,
	// pub bottom_intake: MotorGroup<[Motor; 2]>,
	// pub top_intake: Motor,
	pub controller: Controller,
	pub copro: CoprocessorSmartPort,
	pub imu: Rc<RefCell<InertialSensor>>,
	pub license_plate: AdiLicensePlate,
}

impl Robot {
	pub async fn new(peripherals: Peripherals) -> Self {
		// Initialize Imu
		let mut imu = InertialSensor::new(peripherals.port_16);
		println!("Calibrating IMU...");
		_ = imu.calibrate().await;
		println!("IMU calibrated...");
		let imu = Rc::new(RefCell::new(imu));

		// Setup DR4B
		let lift = Dr4bLift::new(
			Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),
			Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse),
		);

		Robot {
			drivetrain: Drivetrain::new(
				DifferentalSwerve::new(
					[
						// TODO stop doing 0.0 for the wheel positions
						(
							SwervePod::new(
								Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_e),
								2199,
							),
							Vec2::new(5.75 * Inches, -4.5 * Inches),
						),
						(
							SwervePod::new(
								Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_f),
								1130,
							),
							Vec2::new(5.75 * Inches, 4.5 * Inches),
						),
						(
							SwervePod::new(
								Motor::new(peripherals.port_6, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_7, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_h),
								1114,
							),
							Vec2::new(-5.75 * Inches, 4.5 * Inches),
						),
						(
							SwervePod::new(
								Motor::new(peripherals.port_8, Gearset::Blue, Direction::Forward),
								Motor::new(peripherals.port_9, Gearset::Blue, Direction::Forward),
								AdiAnalogIn::new(peripherals.adi_g),
								2016,
							),
							Vec2::new(-5.75 * Inches, -4.5 * Inches),
						),
					],
					imu.clone(),
				),
				(),
			),
			lift,
			top_lift: TopLift::new(
				Motor::new_exp(peripherals.port_12, Direction::Forward),
				Motor::new_exp(peripherals.port_13, Direction::Forward),
			),

			// bottom_intake: MotorGroup::new([
			// 	Motor::new_exp(peripherals.port_17, Direction::Forward),
			// 	Motor::new_exp(peripherals.port_14, Direction::Forward),
			// ]),
			// top_intake: Motor::new(peripherals.port_18, Gearset::Blue, Direction::Forward),
			controller: peripherals.primary_controller,
			imu,
			copro: CoprocessorSmartPort::new(peripherals.port_15).await,
			license_plate: AdiLicensePlate {
				port: AdiDigitalIn::new(peripherals.adi_d),
				high_state: LicensePlate::Red,
			},
		}
	}
}

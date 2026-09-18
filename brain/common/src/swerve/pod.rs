use std::{
	cell::Cell,
	f64,
	rc::Rc,
	time::{Duration, Instant},
};

use evian::control::loops::{AngularPid, Feedback as _};
use shrewnit::{
	Inches, Length, LinearVelocity, Meters, MetersPerSecond, One, RadiansPerSecond,
	RotationsPerMinute,
};
use vexide::{
	adi::{AdiDevice, analog::AdiAnalogIn},
	math::Angle,
	smart::motor::Motor,
	time::sleep,
};

/// Stores all the task's inner data
struct SwervePodInner {
	motor_a: Motor,
	motor_b: Motor,
	rotation: AdiAnalogIn,
	analog_offset: u16,

	// linear_pid: Pid,
	turn_pid: AngularPid,
}

/// A single 2-motor + rotation sensor differential swerve pod.
#[derive(Debug, Clone)]
pub struct SwervePod {
	target_heading: Rc<Cell<Angle>>,
	target_speed: Rc<Cell<LinearVelocity>>,
}

// const LINEAR_PID: Pid = Pid::new(0.02, 200.0, 0.0, Some(11.0));
const TURN_PID: AngularPid = AngularPid::new(30.0, 1.0, 2.5, None);
const WHEEL_RADIUS: Length = <Inches as One<f64, _>>::ONE.mul_scalar(2.75 / 2.0);
const LINEAR_GEAR_RATIO: f64 = 1.0;
const ANGULAR_GEAR_RATIO: f64 = 0.5;

impl SwervePod {
	/// Create a new SwervePod given the two differential motors, a rotation sensor, and a sensor offset
	///
	/// The offset should be chosen such that when added mod 4096 to the rotation input, zero means the
	/// wheel is facing directly forward on the robot. Which direction on the wheel is forward does not
	/// matter, the pod will always choose the closest path to the desired angle.
	pub fn new(motor_a: Motor, motor_b: Motor, rotation: AdiAnalogIn, analog_offset: u16) -> Self {
		let pod = Self {
			target_heading: Rc::new(Cell::new(Angle::ZERO)),
			target_speed: Rc::new(Cell::new(0.0 * MetersPerSecond)),
		};

		let inner = SwervePodInner {
			motor_a,
			motor_b,
			rotation,
			analog_offset,

			// linear_pid: LINEAR_PID,
			turn_pid: TURN_PID,
		};

		vexide::task::spawn(Self::task(pod.clone(), inner)).detach();

		pod
	}

	pub fn set_heading(&mut self, heading: Angle) {
		self.target_heading.set(heading);
	}

	pub fn set_speed(&mut self, speed: LinearVelocity) {
		self.target_speed.set(speed);
	}

	async fn task(pod: SwervePod, mut inner: SwervePodInner) {
		let mut timer = Instant::now();

		loop {
			// Get the current wheel heading measurement and the desired one
			let angle = if let Ok(a) = inner.rotation.value() {
				a
			} else {
				eprintln!(
					"Warning: failed to read rotation sensor on pod from port {}",
					inner.rotation.port_numbers()[0]
				);
				0
			};
			sleep(Duration::from_millis(100)).await;
			let angle =
				Angle::from_degrees((angle + inner.analog_offset % 4096) as f64 / 4096.0 * 360.0);
			let mut target_heading = pod.target_heading.get().wrapped_half();

			// Optimize the path to the target heading (check if it would be faster to go to target + 180.0 and drive backwards)
			let flipped = (angle - target_heading).wrapped_half().abs() > Angle::from_degrees(90.0);
			if flipped {
				target_heading += Angle::from_degrees(180.0);
			}

			// Feed current and desired angle into PID
			let mut turn = inner
				.turn_pid
				.update(angle, target_heading, timer.elapsed());

			// // Use PID with current wheel velocity and desired velocity
			// let current_speed = (inner.motor_a.velocity().unwrap_or(0.0)
			// 	- inner.motor_b.velocity().unwrap_or(0.0))
			// 	/ 2.0;

			// let target_speed = pod.target_speed.get().to::<MetersPerSecond>();
			// let linear =
			// 	inner
			// 		.linear_pid
			// 		.update(current_speed.abs(), target_speed, timer.elapsed());

			// Get the desired speed
			let linear = pod.target_speed.get().to::<MetersPerSecond>() / WHEEL_RADIUS.to::<Meters>() /* m/s / m = rad/s */;
			let linear = (linear * RadiansPerSecond).to::<RotationsPerMinute>();
			let linear = if flipped { -linear } else { linear };

			// Use cosine compensation (if we are at an incorrect angle, drive slower)
			let linear = linear * (angle - target_heading).cos();

			// If we don't want to move, just don't bother turning the wheel
			if linear < 1.0e-6 {
				turn = 0.0;
			}

			_ = inner
				.motor_a
				.set_velocity((turn / ANGULAR_GEAR_RATIO + linear / LINEAR_GEAR_RATIO) as i32);
			_ = inner
				.motor_b
				.set_velocity((turn / ANGULAR_GEAR_RATIO - linear / LINEAR_GEAR_RATIO) as i32);

			timer = Instant::now();
			sleep(Duration::from_millis(10)).await;
		}
	}
}

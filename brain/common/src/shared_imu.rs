use std::{cell::Cell, rc::Rc};

use evian::{math::Angle, tracking::Gyro};
use shrewnit::{AngularVelocity, LinearAcceleration, MetersPerSecondSquared, RadiansPerSecond};
use vexide::{
	math::Vector3,
	smart::{
		SmartDevice,
		imu::{CalibrateError, InertialSensor},
	},
	sync::Mutex,
	task::Task,
	time::sleep,
};

/// A wrapper around an [InertialSensor] with a cheap [Clone] implementation,
/// allowing for different parts of code to access the sensor.
///
/// Measurements are reported on a best-effort basis: if the sensor disconnects
/// or starts calibrating, measurements will not update and will return the last
/// good data.
pub struct SharedImu {
	imu: Rc<Mutex<InertialSensor>>,
	cached_heading: Rc<Cell<Angle>>,
	cached_angular_velocity: Rc<Cell<AngularVelocity>>,
	cached_acceleration: Rc<Cell<Vector3<LinearAcceleration>>>,
	_task: Rc<Task<()>>,
}

impl SharedImu {
	pub fn new(imu: InertialSensor) -> Self {
		let imu = Rc::new(Mutex::new(imu));
		let cached_heading = Rc::new(Cell::new(Default::default()));
		let cached_angular_velocity = Rc::new(Cell::new(Default::default()));
		let cached_acceleration = Rc::new(Cell::new(Vector3 {
			x: Default::default(),
			y: Default::default(),
			z: Default::default(),
		}));

		Self {
			_task: Rc::new(vexide::task::spawn(Self::task(
				imu.clone(),
				cached_heading.clone(),
				cached_angular_velocity.clone(),
				cached_acceleration.clone(),
			))),
			imu,
			cached_heading,
			cached_angular_velocity,
			cached_acceleration,
		}
	}

	async fn task(
		imu: Rc<Mutex<InertialSensor>>,
		cached_heading: Rc<Cell<Angle>>,
		cached_angular_velocity: Rc<Cell<AngularVelocity>>,
		cached_acceleration: Rc<Cell<Vector3<LinearAcceleration>>>,
	) {
		loop {
			if let Some(imu) = imu.try_lock() {
				if let Ok(heading) = imu.heading() {
					cached_heading.set(heading);
				}

				if let Ok(angular_velocity) = imu.angular_velocity() {
					cached_angular_velocity.set(angular_velocity * RadiansPerSecond);
				}

				if let Ok(acceleration) = imu.acceleration() {
					cached_acceleration.set(Vector3 {
						x: acceleration.x / 9.81 * MetersPerSecondSquared,
						y: acceleration.y / 9.81 * MetersPerSecondSquared,
						z: acceleration.z / 9.81 * MetersPerSecondSquared,
					});
				}
			}
			sleep(InertialSensor::UPDATE_INTERVAL).await
		}
	}

	/// Calibrates the sensor, waiting for it to finish
	///
	/// All retrieved headings during this period will be out of date
	pub async fn calibrate(&self) -> Result<(), CalibrateError> {
		self.imu.lock().await.calibrate().await
	}

	/// Retrieves the latest heading from the sensor
	pub fn get_heading(&self) -> Angle {
		self.cached_heading.get()
	}

	/// Retrieves the latest heading from the sensor
	pub fn get_angular_velocity(&self) -> AngularVelocity {
		self.cached_angular_velocity.get()
	}

	/// Retrieves the latest acceleration from the sensor
	pub fn get_acceleration(&self) -> Vector3<LinearAcceleration> {
		self.cached_acceleration.get()
	}
}

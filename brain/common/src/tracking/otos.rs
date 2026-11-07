use std::{cell::Cell, rc::Rc, time::Duration};

use coprocessor::{OtosData, vexide::CoprocessorSmartPort};
use evian::{
	math::{Angle, Vec2},
	tracking::{Tracking, TracksForwardTravel, TracksHeading, TracksPosition, TracksVelocity},
};
use shrewnit::{Length, Radians};
use vexide::time::sleep;

use crate::{
	shared_imu::SharedImu,
	tracking::{EvianAngularVelocity, EvianLength, EvianLinearVelocity},
};

/// An evian tracking implementation, using an OTOS sensor for position measurements.
///
/// If a [SharedImu] is provided, it will be used for heading measurements instead of
/// the OTOS sensor.
pub struct OtosTracking {
	otos: Rc<Cell<OtosData>>,
	forward_travel: Rc<Cell<Length>>,
	imu: Option<SharedImu>,
}

impl OtosTracking {
	pub fn new(copro: &CoprocessorSmartPort, imu: Option<SharedImu>) -> Self {
		let tracking = Self {
			otos: copro.otos(),
			forward_travel: Default::default(),
			imu,
		};

		let forward_travel = tracking.forward_travel.clone();
		let otos = copro.otos();
		vexide::task::spawn(async move {
			let last_position = otos.get().position;
			let mut last_position =
				Vec2::new(last_position.x.canonical(), last_position.y.canonical());
			loop {
				let current_travel = forward_travel.get();
				let current_position = otos.get().position;
				let (current_position, current_heading) = (
					Vec2::new(
						current_position.x.canonical(),
						current_position.y.canonical(),
					),
					Angle::from_radians(current_position.heading.to::<Radians>()),
				);

				// Calculate the direction in which me moved, and how far
				let movement = current_position - last_position;
				let angle = Angle::atan2(movement.y(), movement.x());
				let distance = Length::from_canonical(movement.length());

				// Calculate how far we traveled forward in the direction we went
				let forward_travel_delta =
					distance * (angle - (current_heading + Angle::from_degrees(90.0))).cos();

				last_position = current_position;
				forward_travel.set(current_travel + forward_travel_delta);

				sleep(Duration::from_millis(5)).await;
			}
		})
		.detach();

		tracking
	}
}

impl Tracking for OtosTracking {}

impl TracksPosition for OtosTracking {
	fn position(&self) -> evian::math::Vec2<f64> {
		let position = self.otos.get().position;
		Vec2::new(
			position.x.to::<EvianLength>(),
			position.y.to::<EvianLength>(),
		)
	}
}

impl TracksHeading for OtosTracking {
	fn heading(&self) -> Angle {
		if let Some(imu) = &self.imu {
			imu.get_heading()
		} else {
			Angle::from_radians(self.otos.get().position.heading.to::<Radians>())
		}
	}
}

impl TracksVelocity for OtosTracking {
	fn linear_velocity(&self) -> f64 {
		let velocity = self.otos.get().velocity;
		Vec2::new(
			velocity.x.to::<EvianLinearVelocity>(),
			velocity.y.to::<EvianLinearVelocity>(),
		)
		.length()
	}

	fn angular_velocity(&self) -> f64 {
		if let Some(imu) = &self.imu {
			imu.get_angular_velocity()
		} else {
			self.otos.get().velocity.heading
		}
		.to::<EvianAngularVelocity>()
	}
}

impl TracksForwardTravel for OtosTracking {
	fn forward_travel(&self) -> f64 {
		self.forward_travel.get().to::<EvianLength>()
	}
}

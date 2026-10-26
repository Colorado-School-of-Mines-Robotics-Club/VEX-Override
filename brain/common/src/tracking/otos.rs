use std::{cell::Cell, rc::Rc, time::Duration};

use coprocessor::{OtosData, vexide::CoprocessorSmartPort};
use evian::{
	math::{Angle, Vec2},
	tracking::{Tracking, TracksForwardTravel, TracksPosition, TracksVelocity},
};
use shrewnit::Length;
use vexide::{smart::imu::InertialSensor, sync::Mutex, time::sleep};

use crate::tracking::{EvianAngularVelocity, EvianLength, EvianLinearVelocity};

pub struct OtosTracking {
	otos: Rc<Cell<OtosData>>,
	forward_travel: Rc<Cell<Length>>,
}

impl OtosTracking {
	pub async fn main(copro: &CoprocessorSmartPort) -> Self {
		let tracking = Self {
			otos: copro.otos(),
			forward_travel: Default::default(),
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
				let current_position = Vec2::new(
					current_position.x.canonical(),
					current_position.y.canonical(),
				);

				// Calculate the direction in which me moved, and how far
				let movement = current_position - last_position;
				let angle = Angle::atan2(movement.y(), movement.x());
				let distance = Length::from_canonical(movement.length());

				// Calculate how far we traveled forward in the direction we went
				// let forward_travel_delta = distance * (angle - Angle::new)

				sleep(Duration::from_millis(5)).await;
			}
		})
		.await;

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
		self.otos
			.get()
			.velocity
			.heading
			.to::<EvianAngularVelocity>()
	}
}

impl TracksForwardTravel for OtosTracking {
	fn forward_travel(&self) -> f64 {
		0.0
	}
}

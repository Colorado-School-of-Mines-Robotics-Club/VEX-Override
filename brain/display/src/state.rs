use std::rc::Rc;

use autons::simple::Route;
use shrewnit::{Angle, AngularVelocity, Length, LinearVelocity};
use strum::{EnumCount, FromRepr, IntoStaticStr, VariantArray};

#[derive(Copy, Clone, PartialEq, Eq, Default, EnumCount, FromRepr, VariantArray, IntoStaticStr)]
pub enum SelectedPage {
	#[default]
	Autons,
	Odometry,
}

pub struct State<R, const N: usize> {
	pub page: SelectedPage,
	pub odometry: OdometryState,
	pub autons: AutonsState<R, N>,
}

impl<R, const N: usize> Default for State<R, N> {
	fn default() -> Self {
		Self {
			page: Default::default(),
			odometry: Default::default(),
			autons: Default::default(),
		}
	}
}

impl<R, const N: usize> Clone for State<R, N> {
	fn clone(&self) -> Self {
		Self {
			page: self.page,
			odometry: self.odometry.clone(),
			autons: self.autons.clone(),
		}
	}
}

#[derive(Clone)]
pub struct OdometryState {
	pub(crate) calibration_callback: Rc<dyn Fn()>,
	pub calibrating: bool,
	pub x: Length<f64>,
	pub y: Length<f64>,
	pub h: Angle<f64>,
	pub vx: LinearVelocity<f64>,
	pub vy: LinearVelocity<f64>,
	pub vh: AngularVelocity<f64>,
}

impl OdometryState {
	pub fn register_calibration_callback(&mut self, f: impl Fn() + 'static) {
		self.calibration_callback = Rc::new(f);
	}
}

impl Default for OdometryState {
	fn default() -> Self {
		Self {
			calibration_callback: Rc::new(|| ()),
			calibrating: Default::default(),
			x: Default::default(),
			y: Default::default(),
			h: Default::default(),
			vx: Default::default(),
			vy: Default::default(),
			vh: Default::default(),
		}
	}
}

pub struct AutonsState<R, const N: usize> {
	pub routes: Option<[Route<R>; N]>,
	pub selection: usize,
}

impl<R, const N: usize> Default for AutonsState<R, N> {
	fn default() -> Self {
		Self {
			routes: None,
			selection: 0,
		}
	}
}

impl<R, const N: usize> Clone for AutonsState<R, N> {
	fn clone(&self) -> Self {
		Self {
			routes: self.routes.clone(),
			selection: self.selection,
		}
	}
}

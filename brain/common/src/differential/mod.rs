use evian::drivetrain::model::{Differential, DrivetrainModel};
use vexide::smart::motor::BrakeMode;

pub trait Brake: DrivetrainModel {
	/// Brakes all the motors
	fn brake(&mut self, mode: BrakeMode) -> Result<(), Self::Error>;
}

impl Brake for Differential {
	fn brake(&mut self, mode: BrakeMode) -> Result<(), Self::Error> {
		let mut rtn = Ok(());

		for motor in self.left.borrow_mut().as_mut() {
			if let res = motor.brake(mode)
				&& res.is_err()
			{
				rtn = res;
			}
		}

		for motor in self.right.borrow_mut().as_mut() {
			if let res = motor.brake(mode)
				&& res.is_err()
			{
				rtn = res;
			}
		}

		rtn
	}
}

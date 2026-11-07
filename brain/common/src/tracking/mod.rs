use shrewnit::{
	Degrees, DegreesPerSecond, DegreesPerSecondSquared, Inches, InchesPerSecond,
	InchesPerSecondSquared,
};

pub mod otos;

// Define all the units being used here, for ease of use
pub type EvianLength = Inches;
pub type EvianLinearVelocity = InchesPerSecond;
pub type EvianLinearAcceleration = InchesPerSecondSquared;
pub type EvianAngle = Degrees;
pub type EvianAngularVelocity = DegreesPerSecond;
pub type EvianAngularAcceleration = DegreesPerSecondSquared;

use std::ops::Div;

use coprocessor::OtosPose;
use shrewnit::{Degrees, DegreesPerSecond, DegreesPerSecondSquared, Inches, InchesPerSecond, InchesPerSecondSquared, Length, Radians, Time};

pub mod otos;

pub type EvianLength = Inches;
pub type EvianLinearVelocity = InchesPerSecond;
pub type EvianLinearAcceleration = InchesPerSecondSquared;
pub type EvianAngle = Degrees;
pub type EvianAngularVelocity = DegreesPerSecond;
pub type EvianAngularAcceleration = DegreesPerSecondSquared;

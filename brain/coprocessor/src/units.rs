use shrewnit::{
	Angle, AngularAcceleration, AngularVelocity, Degrees, DegreesPerSecond,
	DegreesPerSecondSquared, Length, LinearAcceleration, LinearVelocity, Meters, MetersPerSecond,
	MetersPerSecondSquared,
};

const OTOS_I16_MAX: f64 = i16::MAX as f64 + 1.0;

// Shrewnit simple_unit!() doesn't support X per Y canonical so this is a workaround
macro_rules! impl_otos_reg_unit {
    ($($name:ident ($dimension:ident): $max:literal $base:ident),+) => {
        $(
            impl_otos_reg_unit!($name, $dimension, $max, $base);
        )+
    };
    ($name:ident, $dimension:ident, $max:literal, $base:ident) => {
        shrewnit::unit_type!(pub $name of dimension $dimension);

        impl<S: shrewnit::Scalar> shrewnit::UnitOf<S, $dimension<S>> for $name {
	        #[inline]
	        fn from_canonical(canonical: S) -> S {
				canonical * (S::from_f64(OTOS_I16_MAX).unwrap() / S::from_u64($max).unwrap())
	        }

	        #[inline]
	        fn to_canonical(converted: S) -> S {
                converted * (S::from_u64($max).unwrap() / S::from_f64(OTOS_I16_MAX).unwrap())
            }
        }

        // Abandon all hope, ye who wish to make a scalar blanket implementation for a trait with associated constants
        impl shrewnit::One<f32, $dimension<f32>> for $name {
            const ONE: $dimension<f32> = $dimension::from_canonical(
          		$max as f32 / OTOS_I16_MAX as f32
            );
            const ONE_CANONICAL: f32 = OTOS_I16_MAX as f32 / $max as f32;
        }

        impl shrewnit::One<f64, $dimension<f64>> for $name {
            const ONE: $dimension<f64> = $dimension::from_canonical(
          		$max as f64 / OTOS_I16_MAX
            );
            const ONE_CANONICAL: f64 = OTOS_I16_MAX / $max as f64;
        }
    };
}

// Implement all of the OTOS register units
impl_otos_reg_unit!(
	OtosLength (Length):                           10      Meters,
	OtosAngle (Angle):                             180     Degrees,
	OtosLinearVelocity (LinearVelocity):           5       MetersPerSecond,
	OtosAngularVelocity (AngularVelocity):         2_000   DegreesPerSecond,
	OtosLinearAcceleration (LinearAcceleration):   157     MetersPerSecondSquared,
	OtosAngularAcceleration (AngularAcceleration): 180_000 DegreesPerSecondSquared
);

#[cfg(test)]
mod tests {
	use std::f32::consts::PI;

	use shrewnit::{Dimension, One};

	use super::*;

	/// https://github.com/sparkfun/SparkFun_Optical_Tracking_Odometry_Sensor/blob/8b1340d613ee64cebb4bc283a44ce2752e6efa6e/Firmware/SFE/Inc/otosConstants.h#L11-L52
	///
	/// SparkFun code, firmware, and software is released under the MIT License(http://opensource.org/licenses/MIT).
	///
	/// The MIT License (MIT)
	/// Copyright (c) 2020 SparkFun Electronics
	/// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
	/// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
	/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
	///
	/// Syntax changed from C++ to rust, values untouched
	#[allow(non_upper_case_globals)]
	mod sparkfun {
		// Useful constants for rotation
		pub const kPi: f32 = 3.1415926535897932384626433832795_f32;
		pub const k2Pi: f32 = 2.0_f32 * kPi;
		pub const kPi2: f32 = kPi / 2.0_f32;
		pub const kRadToDeg: f32 = 180.0_f32 / kPi;
		pub const kDegToRad: f32 = kPi / 180.0_f32;

		// Conversion factor for the linear position registers. 16-bit signed registers
		// with a max value of 10 meters (394 inches) gives a resolution of about 0.0003
		// mps (0.012 ips)
		pub const kMeterToInt16: f32 = 32768.0_f32 / 10.0_f32;
		pub const kInt16ToMeter: f32 = 1.0_f32 / kMeterToInt16;

		// Conversion factor for the linear velocity registers. 16-bit signed registers
		// with a max value of 5 mps (197 ips) gives a resolution of about 0.00015 mps
		// (0.006 ips)
		pub const kMpsToInt16: f32 = 32768.0_f32 / 5.0_f32;
		pub const kInt16ToMps: f32 = 1.0_f32 / kMpsToInt16;

		// Conversion factor for the linear acceleration registers. 16-bit signed
		// registers with a max value of 157 mps^2 (16 g) gives a resolution of
		// about 0.0048 mps^2 (0.49 mg)
		pub const kMpssToInt16: f32 = 32768.0_f32 / (16.0_f32 * 9.80665_f32);
		pub const kInt16ToMpss: f32 = 1.0_f32 / kMpssToInt16;

		// Conversion factor for the angular position registers. 16-bit signed registers
		// with a max value of pi radians (180 degrees) gives a resolution of about
		// 0.00096 radians (0.0055 degrees)
		pub const kRadToInt16: f32 = 32768.0_f32 / kPi;
		pub const kInt16ToRad: f32 = 1.0_f32 / kRadToInt16;

		// Conversion factor for the angular velocity registers. 16-bit signed registers
		// with a max value of 34.9 rps (2000 dps) gives a resolution of about 0.0011
		// rps (0.061 degrees per second)
		pub const kRpsToInt16: f32 = 32768.0_f32 / (2000.0_f32 * kDegToRad);
		pub const kInt16ToRps: f32 = 1.0_f32 / kRpsToInt16;

		// Conversion factor for the angular acceleration registers. 16-bit signed
		// registers with a max value of 3141 rps^2 (180000 dps^2) gives a resolution of
		// about 0.096 rps^2 (5.5 dps^2)
		pub const kRpssToInt16: f32 = 32768.0_f32 / (kPi * 1000.0_f32);
		pub const kInt16ToRpss: f32 = 1.0_f32 / kRpssToInt16;
	}

	const LENGTHS: &[f32] = &[PI, 1.0, 2.25, 6.0, 10.0];
	const INT_LENGTHS: &[i16] = &[-26662, 2263, 18018, -27966, 19979];
	const VELOCITIES: &[f32] = &[1.0, 2.78, PI, 4.783];

	#[test]
	fn test_conversions() {
		for &len in LENGTHS {
			let otos = (len * Meters).to::<OtosLength>() as i16;
			// Ensure the firmware and rust implementations agree
			assert_eq!(otos, -1);
			assert_eq!(otos, (len * sparkfun::kMeterToInt16).trunc() as i16);
			// Ensure round trip works fine
			assert_eq!((otos as f32 * OtosLength).to::<Meters>(), len);
		}
		for &vel in VELOCITIES {
			assert_eq!(
				(vel * MetersPerSecond).to::<OtosLinearVelocity>(),
				vel * sparkfun::kMpsToInt16
			);
		}
	}

	#[test]
	fn test_one() {
		assert_eq!(
			<OtosLength as shrewnit::One<f32, Length<f32>>>::ONE.to::<Meters>(),
			1.0f32 * sparkfun::kInt16ToMeter
		);
		assert_eq!(
			<OtosLength as shrewnit::One<f32, Length<f32>>>::ONE_CANONICAL,
			1.0f32 * sparkfun::kMeterToInt16
		);
	}
}

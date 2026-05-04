use core::task::Poll;

use bytemuck::{AnyBitPattern, NoUninit};
use defmt::{error, info};
use embassy_futures::poll_once;
use embassy_rp::{
	i2c::{self, I2c},
	peripherals::I2C0,
};
use embassy_sync::blocking_mutex::{CriticalSectionMutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Ticker};
use embedded_rpc::RpcService;

use crate::{i2c_regs, peripherals::OtosI2C};

// https://github.com/sparkfun/SparkFun_Optical_Tracking_Odometry_Sensor/blob/main/Firmware/OTOS_Register_Map.pdf
i2c_regs!(
	0x17;
	// Product info
	PRODUCT_ID: 0x00,
	HARDWARE_VERSION: 0x01,
	FIRMWARE_VERSION: 0x02,
	// Scalars
	LINEAR_SCALAR: 0x04,
	ANGULAR_SCALAR: 0x05,
	// Control registers
	IMU_CALIBRATION: 0x06,
	RESET: 0x07,
	SIGNAL_PROCESS_CONFIG: 0x0E,
	SELF_TEST: 0x0F,
	// Offsets
	OFFSET_X_L: 0x10,
	OFFSET_X_H: OFFSET_X_L + 1,
	OFFSET_Y_L: OFFSET_X_L + 2,
	OFFSET_Y_H: OFFSET_X_L + 3,
	OFFSET_H_L: OFFSET_X_L + 4,
	OFFSET_H_H: OFFSET_X_L + 5,
	// Sensor status
	STATUS: 0x1F,
	// Position
	POSITION_X_L: 0x20,
	POSITION_X_H: POSITION_X_L + 1,
	POSITION_Y_L: POSITION_X_L + 2,
	POSITION_Y_H: POSITION_X_L + 3,
	POSITION_H_L: POSITION_X_L + 4,
	POSITION_H_H: POSITION_X_L + 5,
	// Velocity
	VELOCITY_X_L: 0x26,
	VELOCITY_X_H: VELOCITY_X_L + 1,
	VELOCITY_Y_L: VELOCITY_X_L + 2,
	VELOCITY_Y_H: VELOCITY_X_L + 3,
	VELOCITY_H_L: VELOCITY_X_L + 4,
	VELOCITY_H_H: VELOCITY_X_L + 5,
	// Acceleration
	ACCELERATION_X_L: 0x2C,
	ACCELERATION_X_H: ACCELERATION_X_L + 1,
	ACCELERATION_Y_L: ACCELERATION_X_L + 2,
	ACCELERATION_Y_H: ACCELERATION_X_L + 3,
	ACCELERATION_H_L: ACCELERATION_X_L + 4,
	ACCELERATION_H_H: ACCELERATION_X_L + 5,
	// Position standard deviation
	POSITION_STDDEV_X_L: 0x32,
	POSITION_STDDEV_X_H: POSITION_STDDEV_X_L + 1,
	POSITION_STDDEV_Y_L: POSITION_STDDEV_X_L + 2,
	POSITION_STDDEV_Y_H: POSITION_STDDEV_X_L + 3,
	POSITION_STDDEV_H_L: POSITION_STDDEV_X_L + 4,
	POSITION_STDDEV_H_H: POSITION_STDDEV_X_L + 5,
	// Position standard deviation
	VELOCITY_STDDEV_X_L: 0x38,
	VELOCITY_STDDEV_X_H: VELOCITY_STDDEV_X_L + 1,
	VELOCITY_STDDEV_Y_L: VELOCITY_STDDEV_X_L + 2,
	VELOCITY_STDDEV_Y_H: VELOCITY_STDDEV_X_L + 3,
	VELOCITY_STDDEV_H_L: VELOCITY_STDDEV_X_L + 4,
	VELOCITY_STDDEV_H_H: VELOCITY_STDDEV_X_L + 5,
	// Position standard deviation
	ACCELERATION_STDDEV_X_L: 0x3E,
	ACCELERATION_STDDEV_X_H: ACCELERATION_STDDEV_X_L + 1,
	ACCELERATION_STDDEV_Y_L: ACCELERATION_STDDEV_X_L + 2,
	ACCELERATION_STDDEV_Y_H: ACCELERATION_STDDEV_X_L + 3,
	ACCELERATION_STDDEV_H_L: ACCELERATION_STDDEV_X_L + 4,
	ACCELERATION_STDDEV_H_H: ACCELERATION_STDDEV_X_L + 5,
	// LSM6DS0 data
	LSM6DSO_OUTX_L_G: 0x44,
	LSM6DSO_OUTX_H_G: LSM6DSO_OUTX_L_G + 1,
	LSM6DSO_OUTY_L_G: LSM6DSO_OUTX_L_G + 2,
	LSM6DSO_OUTY_H_G: LSM6DSO_OUTX_L_G + 3,
	LSM6DSO_OUTZ_L_G: LSM6DSO_OUTX_L_G + 4,
	LSM6DSO_OUTZ_H_G: LSM6DSO_OUTX_L_G + 5,
	LSM6DSO_OUTX_L_A: LSM6DSO_OUTX_L_G + 6,
	LSM6DSO_OUTX_H_A: LSM6DSO_OUTX_L_G + 7,
	LSM6DSO_OUTY_L_A: LSM6DSO_OUTX_L_G + 8,
	LSM6DSO_OUTY_H_A: LSM6DSO_OUTX_L_G + 9,
	LSM6DSO_OUTZ_L_A: LSM6DSO_OUTX_L_G + 10,
	LSM6DSO_OUTZ_H_A: LSM6DSO_OUTX_L_G + 11,
	// PAA5160 data
	PAA5160_BURST_0: 0x50,
	PAA5160_BURST_1: PAA5160_BURST_0 + 1,
	PAA5160_BURST_2: PAA5160_BURST_0 + 2,
	PAA5160_BURST_3: PAA5160_BURST_0 + 3,
	PAA5160_BURST_4: PAA5160_BURST_0 + 4,
	PAA5160_BURST_5: PAA5160_BURST_0 + 5,
	PAA5160_BURST_6: PAA5160_BURST_0 + 6,
	PAA5160_BURST_7: PAA5160_BURST_0 + 7,
	PAA5160_BURST_8: PAA5160_BURST_0 + 8,
	PAA5160_BURST_9: PAA5160_BURST_0 + 9,
	PAA5160_BURST_10: PAA5160_BURST_0 + 10,
	PAA5160_BURST_11: PAA5160_BURST_0 + 11,
	PAA5160_BURST_12: PAA5160_BURST_0 + 12
);

pub const READINGS_LOWER: u8 = STATUS;
pub const READINGS_UPPER: u8 = ACCELERATION_STDDEV_H_H;

type ReadingsBuffer = [u8; { READINGS_UPPER - READINGS_LOWER + 1 } as usize];

pub static LATEST_READINGS: CriticalSectionMutex<ReadingsBuffer> =
	CriticalSectionMutex::new([0; _]);
pub static SERVICE: RpcService<CriticalSectionRawMutex, OtosAction, Result<(), ()>> =
	RpcService::new();

#[repr(C, align(1))]
#[derive(Clone, Copy, NoUninit, AnyBitPattern)]
pub struct OtosScalars {
	linear: i8,
	angular: i8,
}

impl From<&[u8; 2]> for OtosScalars {
	fn from(value: &[u8; 2]) -> Self {
		Self {
			linear: i8::from_le_bytes([value[0]]),
			angular: i8::from_le_bytes([value[1]]),
		}
	}
}

#[repr(C, align(1))]
#[derive(Clone, Copy, NoUninit, AnyBitPattern)]
pub struct OtosPose {
	pub x: i16,
	pub y: i16,
	pub h: i16,
}

impl From<&[u8; 6]> for OtosPose {
	fn from(value: &[u8; 6]) -> Self {
		Self {
			x: i16::from_le_bytes(value[0..2].try_into().unwrap()),
			y: i16::from_le_bytes(value[2..4].try_into().unwrap()),
			h: i16::from_le_bytes(value[4..6].try_into().unwrap()),
		}
	}
}

pub enum OtosAction {
	Calibrate,
	SetOffsets(OtosPose),
	SetPosition(OtosPose),
	SetScalars(OtosScalars),
}

#[embassy_executor::task]
pub async fn otos_task(mut i2c: I2c<'static, I2C0, i2c::Async>) {
	let mut ticker = Ticker::every(Duration::from_millis(5));
	let mut buf: ReadingsBuffer = [0u8; _];
	wait_for_otos(&mut i2c).await;
	_ = calibrate(&mut i2c).await;

	loop {
		match poll_once(SERVICE.serve()) {
			Poll::Ready((OtosAction::Calibrate, res)) => {
				res.respond(calibrate(&mut i2c).await.map_err(|_| ()));
			}
			Poll::Ready((
				a @ (OtosAction::SetOffsets(pose) | OtosAction::SetPosition(pose)),
				res,
			)) => {
				let mut i2c_write = [match a {
					OtosAction::SetOffsets(_) => OFFSET_X_L,
					OtosAction::SetPosition(_) => POSITION_X_L,
					_ => unreachable!(),
				}; 1 + size_of::<OtosPose>()];
				i2c_write[1..]
					.copy_from_slice(bytemuck::must_cast_slice(core::slice::from_ref(&pose)));
				if let Err(e) = i2c.write_async(ADDR, i2c_write).await {
					error!("I2C write on otos failed: {:?}", e);
					res.respond(Err(()));
					wait_for_otos(&mut i2c).await;
					continue;
				};
				res.respond(Ok(()));
			}
			Poll::Ready((OtosAction::SetScalars(scalars), res)) => {
				let mut i2c_write = [LINEAR_SCALAR; 1 + size_of::<OtosScalars>()];
				i2c_write[1..]
					.copy_from_slice(bytemuck::must_cast_slice(core::slice::from_ref(&scalars)));
				if let Err(e) = i2c.write_async(ADDR, i2c_write).await {
					error!("I2C write on otos failed: {:?}", e);
					res.respond(Err(()));
					wait_for_otos(&mut i2c).await;
					continue;
				};
				res.respond(Ok(()));
			}
			// Will be pending on no requests
			Poll::Pending => (),
		}

		// Update readings
		if let Err(e) = i2c.write_read_async(ADDR, [READINGS_LOWER], &mut buf).await {
			error!("I2C read on otos for sensor updates failed: {:?}", e);
			wait_for_otos(&mut i2c).await;
			continue;
		}

		let cl = |r: &mut ReadingsBuffer| *r = buf;
		// SAFETY: Calling lock_mut is completely safe, so long as it is not called within itself.
		unsafe {
			LATEST_READINGS.lock_mut(cl); // TODO: is dma worth it?
		};

		ticker.next().await;
	}
}

async fn calibrate<'a>(i2c: &mut I2c<'a, OtosI2C, i2c::Async>) -> Result<(), i2c::Error> {
	let mut samples = u8::MAX;
	if let Err(e) = i2c.write_async(ADDR, [IMU_CALIBRATION, samples]).await {
		error!("I2C write on otos for IMU calibration failed: {:?}", e);
		return Err(e);
	};

	// Wait for calibration to finish
	let mut ticker = Ticker::every(Duration::from_millis(5));
	while samples != 0 {
		if let Err(e) = i2c
			.write_read_async(ADDR, [IMU_CALIBRATION], core::slice::from_mut(&mut samples))
			.await
		{
			error!("I2C read on otos for IMU calibration failed: {:?}", e);
			return Err(e);
		};
		ticker.next().await;
	}

	// Reset position
	if let Err(e) = i2c
		.write_async(ADDR, [POSITION_X_L, 0, 0, 0, 0, 0, 0])
		.await
	{
		error!("I2C write on otos for position reset failed: {:?}", e);
		return Err(e);
	};

	// Reset scalars
	if let Err(e) = i2c.write_async(ADDR, [LINEAR_SCALAR, 0, 0]).await {
		error!("I2C write on otos for scalar reset failed: {:?}", e);
		return Err(e);
	};

	Ok(())
}

async fn wait_for_otos<'a>(i2c: &mut I2c<'a, OtosI2C, i2c::Async>) {
	let mut buf = 0;
	let mut ticker = Ticker::every(Duration::from_secs(1));
	loop {
		if let Err(e) = i2c
			.write_read_async(ADDR, [PRODUCT_ID], core::slice::from_mut(&mut buf))
			.await
		{
			error!("Unable to read product ID from OTOS sensor: {:?}", e);
			ticker.next().await;
			continue;
		}

		if buf != 0x5F {
			error!(
				"OTOS sensor responded with incorrect product ID: {:#04x}",
				buf
			);
			ticker.next().await;
			continue;
		}

		info!("OTOS sensor successfully connected!");
		return;
	}
}

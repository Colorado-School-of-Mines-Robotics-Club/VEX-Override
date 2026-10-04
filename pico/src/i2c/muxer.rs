use embassy_rp::{
	Peri,
	i2c::{self, Async, Config, I2c, Instance, InterruptHandler, SclPin, SdaPin},
	interrupt::typelevel::Binding,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embedded_hal_async::i2c::ErrorType;

pub struct I2cMuxer<'a, T: Instance, const ADDR: usize = 0b000> {
	inner: Mutex<NoopRawMutex, I2c<'a, T, Async>>,
}

impl<'a, T: Instance, const ADDR: usize> I2cMuxer<'a, T, ADDR> {
	pub fn new(
		peri: Peri<'a, T>,
		sda: Peri<'a, impl SdaPin<T>>,
		scl: Peri<'a, impl SclPin<T>>,
		irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
	) -> Self {
		Self {
			inner: Mutex::new(I2c::new_async(peri, scl, sda, irq, {
				let mut cfg = Config::default();
				cfg.frequency = 400_000;
				cfg
			})),
		}
	}

	pub fn get_channel<'b, const N: usize>(&'b self) -> I2cMuxedDevice<'a, 'b, T, N, ADDR> {
		I2cMuxedDevice { muxer: self }
	}
}

pub struct I2cMuxedDevice<'a, 'b, T: Instance, const N: usize, const ADDR: usize> {
	muxer: &'b I2cMuxer<'a, T, ADDR>,
}

impl<'a, 'b, T: Instance, const N: usize, const ADDR: usize> I2cMuxedDevice<'a, 'b, T, N, ADDR> {
	/// Form: 0b1110XXX
	/// Where X is the A0-A2 configuration
	const MUXER_ADDRESS: u16 = 0b1110 << 3 | ADDR as u16;

	#[allow(dead_code)]
	pub const TCA9548A_CHANNELS: usize = 8;

	/// Ensure we never create a muxed device higher than the possible ones
	const _ASSERT: () = assert!(N < Self::TCA9548A_CHANNELS);
}

impl<'a, 'b, T: Instance, const N: usize, const ADDR: usize> ErrorType
	for I2cMuxedDevice<'a, 'b, T, N, ADDR>
{
	type Error = i2c::Error;
}

impl<'a, 'b, T: Instance, const N: usize, const ADDR: usize> embedded_hal_async::i2c::I2c
	for I2cMuxedDevice<'a, 'b, T, N, ADDR>
{
	async fn transaction(
		&mut self,
		address: u8,
		operations: &mut [embedded_hal_async::i2c::Operation<'_>],
	) -> Result<(), Self::Error> {
		let mut inner = self.muxer.inner.lock().await;
		// Only enable the desired channel
		inner.write_async(Self::MUXER_ADDRESS, [0b1 << N]).await?;
		// Send the actual data
		inner.transaction(address, operations).await?;

		Ok(())
	}
}

use defmt::{error, info};
use embassy_futures::join::join4;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};

use crate::{
	i2c::{encoder::AS5600Encoder, muxer::I2cMuxer},
	peripherals::MuxerI2C,
};

pub static ANGLES: [Signal<CriticalSectionRawMutex, u16>; 4] =
	[Signal::new(), Signal::new(), Signal::new(), Signal::new()];

macro_rules! update_angle {
	($encoders:ident, $i: tt) => {
		async {
			$encoders
				.$i
				.read_angle()
				.await
				.and_then(|v| {
					ANGLES[$i].signal(v);
					Ok($i)
				})
				.map_err(|e| ($i, e))
		}
	};
}

#[embassy_executor::task]
pub async fn encoders_task(muxer: I2cMuxer<'static, MuxerI2C>) {
	let mut encoders = (
		AS5600Encoder::new(muxer.get_channel::<0>()),
		AS5600Encoder::new(muxer.get_channel::<1>()),
		AS5600Encoder::new(muxer.get_channel::<2>()),
		AS5600Encoder::new(muxer.get_channel::<3>()),
	);

	let mut ticker = Ticker::every(Duration::from_millis(10));
	let mut encoder_errors = 0u8;
	loop {
		// Update all the encoder values
		let results: [_; _] = join4(
			update_angle!(encoders, 0),
			update_angle!(encoders, 1),
			update_angle!(encoders, 2),
			update_angle!(encoders, 3),
		)
		.await
		.into();

		// Print errors if there are any
		for result in results {
			match result {
				// Was erroring, now works
				Ok(i) if encoder_errors & 1 << i != 0 => {
					encoder_errors ^= 1 << i;
					info!("Reading i2c encoder #{} now working", i)
				}
				// Was working, now failed
				Err((i, e)) if encoder_errors & 1 << i == 0 => {
					encoder_errors ^= 1 << i;
					error!("Reading i2c encoder #{} failed: {}", i, e)
				}
				// Continues working/erroring
				_ => (),
			}
		}

		ticker.next().await
	}
}

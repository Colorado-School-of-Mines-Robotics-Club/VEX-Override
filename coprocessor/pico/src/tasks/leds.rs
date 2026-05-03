use bytemuck::{AnyBitPattern, NoUninit};
use embassy_rp::{dma, pio::StateMachine};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

use crate::peripherals::{LEDS_STATE_MACHINE, LedsPIO};

#[repr(C, align(4))]
#[derive(Clone, Copy, NoUninit, AnyBitPattern)]
pub struct LedPixel {
	_padding: u8,
	blue: u8,
	red: u8,
	green: u8,
}

impl LedPixel {
	pub fn new(red: u8, green: u8, blue: u8) -> Self {
		Self {
			_padding: 0,
			red,
			green,
			blue,
		}
	}
}

impl From<u32> for LedPixel {
	fn from(value: u32) -> Self {
		Self::new(
			(value >> 16) as u8,
			(value >> 8 & 0xFF) as u8,
			(value & 0xFF) as u8,
		)
	}
}

pub const LED_COUNT: usize = 9;
pub static LEDS: Signal<CriticalSectionRawMutex, [LedPixel; LED_COUNT]> = Signal::new();

#[embassy_executor::task]
pub async fn leds_task(
	mut sm: StateMachine<'static, LedsPIO, LEDS_STATE_MACHINE>,
	mut dma: dma::Channel<'static>,
) {
	LEDS.signal([LedPixel::new(0, 1, 0); _]);
	Timer::after(Duration::from_secs(1)).await;
	loop {
		let leds = LEDS.wait().await;

		sm.tx()
			.dma_push(&mut dma, bytemuck::must_cast_slice::<_, u32>(&leds), false)
			.await;
	}
}

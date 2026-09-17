use core::num::NonZeroU32;

use embassy_rp::pio::StateMachine;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::peripherals::{BLINKER_STATE_MACHINE, BlinkerPIO};

pub static BLINK_RATE: Signal<CriticalSectionRawMutex, NonZeroU32> = Signal::new();

#[embassy_executor::task]
pub async fn blinker_task(mut sm: StateMachine<'static, BlinkerPIO, BLINKER_STATE_MACHINE>) {
	BLINK_RATE.signal(NonZeroU32::new(1).unwrap());
	loop {
		let blink_rate = BLINK_RATE.wait().await;

		sm.set_enable(false);
		sm.clear_fifos();
		sm.tx().push(blink_rate.get() - 1);
		sm.set_enable(true);
	}
}

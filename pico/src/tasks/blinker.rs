use defmt::debug;
use embassy_rp::pio::StateMachine;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::peripherals::{BLINKER_STATE_MACHINE, BlinkerPIO};

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum BlinkStatusUpdate {
	Otos(bool),
	Lidar(bool),
}

// Binary lol
//
// Effectively:
// 2 Blink = OTOS bad
// 3 Blink = Lidar bad
// 4 Blink = Both bad
#[derive(Clone, Copy, Debug, Default, defmt::Format)]
struct BlinkStatus {
	otos: bool,
	lidar: bool,
}
pub static STATUS_UPDATES: Channel<CriticalSectionRawMutex, BlinkStatusUpdate, 4> = Channel::new();

#[embassy_executor::task]
pub async fn blinker_task(mut sm: StateMachine<'static, BlinkerPIO, BLINKER_STATE_MACHINE>) {
	let mut status = BlinkStatus::default();
	loop {
		let update = STATUS_UPDATES.receive().await;
		match update {
			BlinkStatusUpdate::Otos(s) => status.otos = s,
			BlinkStatusUpdate::Lidar(s) => status.lidar = s,
		}
		debug!("{:?} {:?}", update, status);

		// There's probably a better way to do this
		let blinks = match status {
			BlinkStatus {
				// Both good
				otos: true,
				lidar: true,
			} => 1,
			BlinkStatus {
				// OTOS bad
				otos: false,
				lidar: true,
			} => 2,
			BlinkStatus {
				// Lidar bad
				otos: true,
				lidar: false,
			} => 3,
			BlinkStatus {
				// Both bad
				otos: false,
				lidar: false,
			} => 4,
		};

		sm.set_enable(false);
		sm.clear_fifos();
		sm.tx().push(blinks - 1);
		sm.set_enable(true);
	}
}

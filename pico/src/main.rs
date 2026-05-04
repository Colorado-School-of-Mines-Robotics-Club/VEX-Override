#![no_std]
#![no_main]
#![feature(split_array)]

mod lidar;
mod peripherals;
mod pio;
mod tasks;
mod utils;

use crate::tasks::{blinker::blinker_task, brain::brain_rx, leds::leds_task, otos::otos_task};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
	clocks::ClockConfig,
	config::Config,
	gpio::{self, Output},
	rom_data::reset_to_usb_boot,
	watchdog::{ResetReason, Watchdog},
};
#[cfg(feature = "usb")]
use embassy_rp::{peripherals::USB, usb};
use embassy_time::{Duration, Ticker};
use gpio::Input;

#[cfg(feature = "probe")]
use defmt_rtt as _;
use panic_probe as _;

macro_rules! spawn_tasks {
	($spawner:expr; $($(#[$attr:meta])* $fn:ident($($param:expr),*)),+) => {
		$(
			$(#[$attr])* {
				$spawner.spawn(unwrap!($fn($($param),*)));
			}
		)+
    };
}

const CLOCK_SPEED: u32 = 150_000_000; // MHz

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	// Configure rp2350 chip
	let config = Config::new(unwrap!(ClockConfig::system_freq(CLOCK_SPEED)));
	let p = embassy_rp::init(config);

	// Initialize peripherals
	let p = peripherals::setup_peripherals(p);

	// Spawn tasks
	spawn_tasks!(
		spawner;
		blinker_task(p.blinker_sm),
		otos_task(p.otos),
		brain_rx(p.brain_uart, p.brain_enable_pin),
		leds_task(p.leds_sm, p.leds_dma),
		secondary_bootsel(p.secondary_bootsel),
		watchdog_task(p.watchdog, p.led2),
		#[cfg(feature = "usb")] defmt_usb(p.usb),
		#[cfg(feature = "usb")] pinger()
	);
}

#[embassy_executor::task(pool_size = 1)]
async fn secondary_bootsel(mut pin: Input<'static>) {
	pin.wait_for_low().await;
	reset_to_usb_boot(0, 0);
}

#[embassy_executor::task]
async fn watchdog_task(mut watchdog: Watchdog, mut led: Output<'static>) {
	// If we have reset due to watchdog timeout, indicate via LED and stall
	if let Some(ResetReason::TimedOut) = watchdog.reset_reason() {
		led.set_high();
		loop {
			core::hint::spin_loop();
		}
	} else {
		led.set_low();
	}

	watchdog.start(Duration::from_millis(300));
	let mut ticker = Ticker::every(Duration::from_millis(250));
	loop {
		ticker.next().await;
		watchdog.feed(Duration::from_millis(300));
	}
}

#[embassy_executor::task]
async fn pinger() {
	let mut ticker = Ticker::every(Duration::from_millis(250));
	loop {
		debug!("Ping");
		ticker.next().await;
	}
}

#[embassy_executor::task]
#[cfg(feature = "usb")]
async fn defmt_usb(driver: usb::Driver<'static, USB>) {
	const USB_CONFIG: embassy_usb::Config<'static> = {
		let mut c = embassy_usb::Config::new(0x1209, 0xA5A5); // pid.codes unused PID, unregistered
		c.serial_number = Some("defmt");
		c.max_packet_size_0 = 64;
		c.composite_with_iads = true;
		c.device_class = 0xEF;
		c.device_sub_class = 0x02;
		c.device_protocol = 0x01;
		c
	};
	defmt_embassy_usbserial::run(driver, USB_CONFIG).await;
}

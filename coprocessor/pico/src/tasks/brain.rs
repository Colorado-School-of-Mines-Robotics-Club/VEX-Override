use defmt::{dbg, debug, error};
use embassy_futures::yield_now;
use embassy_rp::{
	gpio::Output,
	uart::{self, Uart},
};
use embassy_time::{Duration, Ticker};
use static_cell::StaticCell;

use super::otos::{self, OtosAction, OtosPose, OtosScalars};

const MTU: usize = 1024;

static RX_BUFFER: StaticCell<[u8; MTU]> = StaticCell::new();
static TX_BUFFER: StaticCell<[u8; MTU]> = StaticCell::new();
static COBS_BUFFER: StaticCell<[u8; cobs::max_encoding_length(MTU)]> = StaticCell::new();

// static CRC: crc::Algorithm<u16> = crc::Algorithm {
// 	width: 16,
// 	// (*op) C5; CRC-16F/4.2 ("321353")
// 	poly: 0xa2eb,
// 	// source: i made them up
// 	init: 0xff,
// 	refin: false,
// 	refout: false,
// 	xorout: 0x00,
// 	check: 0x2a50,
// 	residue: 0x00,
// };

macro_rules! read_cached_i2c {
	($start:ident..=$end:ident) => {
		crate::tasks::otos::LATEST_READINGS.lock(|r| {
			*r.rsplit_array_ref::<{ (crate::tasks::otos::READINGS_UPPER - crate::tasks::otos::$start + 1) as usize }>(
			)
			.1
			.split_array_ref::<{ (crate::tasks::otos::$end - crate::tasks::otos::$start + 1) as usize }>()
			.0
		})
	};
}

#[embassy_executor::task]
pub async fn brain_rx(mut uart: Uart<'static, uart::Async>, mut enable_pin: Output<'static>) {
	enable_pin.set_low();
	let rx_buf = RX_BUFFER.init([0u8; _]);
	let tx_buf = TX_BUFFER.init([0u8; _]);
	let cobs_buf = COBS_BUFFER.init([0u8; _]);
	let cobs_buf_len = cobs_buf.len();
	let mut cursor: usize = 0;
	loop {
		if cursor >= rx_buf.len() {
			cursor = 0;
			error!("Brain RX buffer overflow");
		}

		if let Err(e) = uart.read(&mut rx_buf[cursor..(cursor + 1)]).await {
			error!("Brain RX read error: {:?}", e);
			continue;
		}

		if let Some(0x00) = rx_buf.get(cursor) {
			cursor = 0;
			// TODO benchmark in-place vs normal decode
			match cobs::decode_in_place(&mut rx_buf[..]) {
				Ok(decoded) => {
					// Process packet and encode response
					let payload_len = handle_brain_packet(&rx_buf[0..decoded], tx_buf).await;
					let cobs_len =
						cobs::encode(&tx_buf[..payload_len], &mut cobs_buf[0..(cobs_buf_len - 1)]);
					cobs_buf[cobs_len] = 0; // Terminate it, because the library doesn't

					// Send response
					enable_pin.set_high();
					let write_result = uart.write(&cobs_buf[0..(cobs_len + 1)]).await;
					while uart.busy() {
						yield_now().await;
					}
					enable_pin.set_low();
					if let Err(e) = write_result {
						error!("Brain TX send error: {:?}", e);
						continue;
					}
				}
				Err(e) => {
					error!("Brain RX cobs decode error: {:?}", e);
					continue;
				}
			}
		} else {
			cursor += 1;
		}
	}
}

mod reqs {
	pub(super) const GET_POSITION: u8 = b'p';
	pub(super) const SET_POSITION: u8 = b'P';
	pub(super) const GET_VELOCITY: u8 = b'v';
	pub(super) const CALIBRATE: u8 = b'c';
	pub(super) const SET_OFFSETS: u8 = b'o';
	pub(super) const SET_SCALARS: u8 = b's';
	pub(super) const GET_STDDEV: u8 = b'S';
	pub(super) const PING: u8 = b'a';
	pub(super) const SET_LEDS: u8 = b'l';
}

/// Packet format:
/// [id: u8][payload][crc: u16]
///
/// No length is necessary for payload, framing takes care of that
pub async fn handle_brain_packet(packet: &[u8], buf: &mut [u8]) -> usize {
	let data: &[u8] = match packet[0] {
		reqs::GET_POSITION => &read_cached_i2c!(POSITION_X_L..=POSITION_H_H),
		reqs::SET_POSITION => {
			_ = otos::SERVICE
				.request(OtosAction::SetPosition(
					packet[1..7].as_array::<6>().unwrap().into(),
				))
				.await;
			b"d"
		}
		reqs::GET_VELOCITY => &read_cached_i2c!(VELOCITY_X_L..=VELOCITY_H_H),
		reqs::CALIBRATE => {
			_ = otos::SERVICE.request(OtosAction::Calibrate).await;
			_ = otos::SERVICE
				.request(OtosAction::SetPosition(OtosPose { x: 0, y: 0, h: 0 }))
				.await;
			b"d"
		}
		reqs::SET_OFFSETS => {
			_ = otos::SERVICE
				.request(OtosAction::SetOffsets(
					packet[1..7].as_array::<6>().unwrap().into(),
				))
				.await;
			b"d"
		}
		reqs::SET_SCALARS => {
			_ = otos::SERVICE
				.request(OtosAction::SetScalars(
					packet[1..3].as_array::<2>().unwrap().into(),
				))
				.await;
			b"d"
		}
		reqs::GET_STDDEV => &read_cached_i2c!(POSITION_STDDEV_X_L..=POSITION_STDDEV_H_H),
		reqs::PING => &[0u8; 32], // TODO
		reqs::SET_LEDS => b"d",   // TODO
		_ => return 0,
	};

	buf[..data.len()].copy_from_slice(data);

	data.len()
}

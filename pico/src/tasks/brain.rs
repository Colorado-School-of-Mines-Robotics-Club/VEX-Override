use cobs::CobsEncoder;
use embassy_futures::{
	join::join,
	select::{Either3, select3},
};
use embassy_rp::{
	gpio::Output,
	uart::{self, Uart},
};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

use crate::tasks::{
	lidar::LIDAR_MEASUREMENTS,
	otos::{LATEST_READINGS, OtosScalars},
};

use super::otos::{self, OtosAction, OtosPose};

const MTU: usize = 1024;

// static RX_BUFFER: StaticCell<[u8; MTU]> = StaticCell::new();
// static TX_BUFFER: StaticCell<[u8; MTU]> = StaticCell::new();
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

// macro_rules! read_cached_i2c {
// 	($start:ident..=$end:ident) => {
// 		crate::tasks::otos::LATEST_READINGS.lock(|r| {
// 			*r.rsplit_array_ref::<{ (crate::tasks::otos::READINGS_UPPER - crate::tasks::otos::$start + 1) as usize }>(
// 			)
// 			.1
// 			.split_array_ref::<{ (crate::tasks::otos::$end - crate::tasks::otos::$start + 1) as usize }>()
// 			.0
// 		})
// 	};
// }

#[embassy_executor::task]
pub async fn brain_rx(mut uart: Uart<'static, uart::Async>, mut enable_pin: Output<'static>) {
	enable_pin.set_low();
	// let rx_buf = RX_BUFFER.init([0u8; _]);
	// let tx_buf = TX_BUFFER.init([0u8; _]);
	let cobs_buf = COBS_BUFFER.init([0u8; _]);

	match select3(
		Timer::after(Duration::from_millis(250)),
		LIDAR_MEASUREMENTS.receive(),
		LATEST_READINGS.wait(),
	)
	.await
	{
		// Ping for updates from brain
		Either3::First(_) => {
			_ = uart.write(&[0x02, b'r', 0x00]).await;
			let mut cursor = 0;
			loop {
				_ = uart.read(&mut cobs_buf[cursor..(cursor + 1)]).await;
				if cobs_buf[cursor] == 0x00 {
					break;
				}
				cursor += 1;
			}
			let len = cobs::decode_in_place(&mut cobs_buf[..]).expect("todo don't panic");

			cursor = 0;
			while cursor < len {
				cursor += 1;
				match cobs_buf[cursor] {
					b'c' => {
						_ = otos::SERVICE.request(OtosAction::Calibrate).await;
					}
					b'p' => {
						_ = otos::SERVICE
							.request(OtosAction::SetPosition(bytemuck::must_cast::<
								[u8; 6],
								OtosPose,
							>(
								cobs_buf[cursor..(cursor + 6)].try_into().unwrap(),
							)))
							.await;
					}
					b's' => {
						_ =
							join(
								otos::SERVICE.request(OtosAction::SetScalars(
									bytemuck::must_cast::<[u8; 2], OtosScalars>(
										cobs_buf[cursor..(cursor + 2)].try_into().unwrap(),
									),
								)),
								otos::SERVICE.request(OtosAction::SetOffsets(
									bytemuck::must_cast::<[u8; 6], OtosPose>(
										cobs_buf[cursor..(cursor + 6)].try_into().unwrap(),
									),
								)),
							)
							.await;
					}
					_ => unreachable!(),
				}
			}

			todo!()
		}
		// Send lidar
		Either3::Second(mut measurement) => {
			// Encode message
			let mut cobs = CobsEncoder::new(&mut cobs_buf[..]);
			_ = cobs.push(b"l"); // Todo handle errors
			loop {
				_ = cobs.push(&bytemuck::must_cast::<_, [u8; size_of::<f32>()]>(
					measurement.angle,
				));
				_ = cobs.push(&bytemuck::must_cast::<_, [u8; size_of::<f32>()]>(
					measurement.distance,
				));
				_ = cobs.push(&[measurement.quality]);

				if let Ok(m) = LIDAR_MEASUREMENTS.try_peek() {
					measurement = m
				} else {
					break;
				}
			}
			let len = cobs.finalize();
			cobs_buf[len] = 0x00;
			// Send message
			_ = uart.write(&cobs_buf[..len]).await;
		}
		// Send OTOS
		Either3::Third(data) => {
			let mut cobs = CobsEncoder::new(&mut cobs_buf[..]);
			_ = cobs.push(b"o");
			_ = cobs.push(&data);
			let len = cobs.finalize();
			cobs_buf[len] = 0x00;
			_ = uart.write(&cobs_buf[..len]).await;
		}
	}
}

// mod reqs {
// pub(super) const GET_POSITION: u8 = b'p';
// pub(super) const SET_POSITION: u8 = b'P';
// pub(super) const GET_VELOCITY: u8 = b'v';
// pub(super) const CALIBRATE: u8 = b'c';
// pub(super) const SET_OFFSETS: u8 = b'o';
// pub(super) const SET_SCALARS: u8 = b's';
// pub(super) const GET_STDDEV: u8 = b'S';
// pub(super) const PING: u8 = b'a';
// pub(super) const SET_LEDS: u8 = b'l';
// }

// /// Packet format:
// /// [id: u8][payload][crc: u16]
// ///
// /// No length is necessary for payload, framing takes care of that
// pub async fn handle_brain_packet(packet: &[u8], buf: &mut [u8]) -> usize {
// 	let data: &[u8] = match packet[0] {
// 		reqs::GET_POSITION => &read_cached_i2c!(POSITION_X_L..=POSITION_H_H),
// 		reqs::SET_POSITION => {
// 			_ = otos::SERVICE
// 				.request(OtosAction::SetPosition(
// 					packet[1..7].as_array::<6>().unwrap().into(),
// 				))
// 				.await;
// 			b"d"
// 		}
// 		reqs::GET_VELOCITY => &read_cached_i2c!(VELOCITY_X_L..=VELOCITY_H_H),
// 		reqs::CALIBRATE => {
// 			_ = otos::SERVICE.request(OtosAction::Calibrate).await;
// 			_ = otos::SERVICE
// 				.request(OtosAction::SetPosition(OtosPose { x: 0, y: 0, h: 0 }))
// 				.await;
// 			b"d"
// 		}
// 		reqs::SET_OFFSETS => {
// 			_ = otos::SERVICE
// 				.request(OtosAction::SetOffsets(
// 					packet[1..7].as_array::<6>().unwrap().into(),
// 				))
// 				.await;
// 			b"d"
// 		}
// 		reqs::SET_SCALARS => {
// 			_ = otos::SERVICE
// 				.request(OtosAction::SetScalars(
// 					packet[1..3].as_array::<2>().unwrap().into(),
// 				))
// 				.await;
// 			b"d"
// 		}
// 		reqs::GET_STDDEV => &read_cached_i2c!(POSITION_STDDEV_X_L..=POSITION_STDDEV_H_H),
// 		reqs::PING => &[0u8; 32], // TODO
// 		reqs::SET_LEDS => b"d",   // TODO
// 		_ => return 0,
// 	};

// 	buf[..data.len()].copy_from_slice(data);

// 	data.len()
// }

use defmt::{info, warn};
use embassy_futures::{join::join, select::Either};
use embassy_time::{Duration, Ticker, WithTimeout};
use static_cell::StaticCell;

use crate::{tasks::otos::OtosScalars, utils::SharedBrainUart};

use super::otos::{self, OtosAction, OtosPose};

const MTU: usize = 1024;

static COBS_BUFFER: StaticCell<[u8; cobs::max_encoding_length(MTU) + 1]> = StaticCell::new();

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

#[embassy_executor::task]
pub async fn brain_rx(uart: SharedBrainUart) {
	let cobs_buf = COBS_BUFFER.init([0u8; _]);

	let mut request_ticker = Ticker::every(Duration::from_millis(1000));
	loop {
		request_ticker.next().await;

		// Request updates
		info!("Requesting brain updates");
		_ = uart.send(&[0x02, b'r', 0x00]).await;

		// Wait for updates
		let len = match uart
			.read_packet(&mut cobs_buf[..])
			.with_timeout(Duration::from_millis(1000))
			.await
		{
			Ok(Ok(len)) => len,
			Ok(Err(Either::First(e))) => {
				warn!("Brain UART error: {}", e);
				continue;
			}
			Ok(Err(Either::Second(e))) => {
				warn!("Brain COBS decoding error: {}", e);
				continue;
			}
			Err(_) => {
				warn!("Brain request timeout");
				continue;
			}
		};

		let mut cursor = 0;
		while cursor < len {
			cursor += 1;
			match cobs_buf[cursor] {
				b'c' => {
					_ = otos::SERVICE
						.request(OtosAction::Calibrate)
						.with_timeout(Duration::from_secs(2))
						.await;
				}
				b'p' => {
					_ = otos::SERVICE
						.request(OtosAction::SetPosition(bytemuck::must_cast::<
							[u8; 6],
							OtosPose,
						>(
							cobs_buf[cursor..(cursor + 6)].try_into().unwrap(),
						)))
						.with_timeout(Duration::from_secs(1))
						.await;
				}
				b's' => {
					_ = join(
						otos::SERVICE.request(OtosAction::SetScalars(bytemuck::must_cast::<
							[u8; 2],
							OtosScalars,
						>(
							cobs_buf[cursor..(cursor + 2)].try_into().unwrap(),
						))),
						otos::SERVICE.request(OtosAction::SetOffsets(bytemuck::must_cast::<
							[u8; 6],
							OtosPose,
						>(
							cobs_buf[cursor..(cursor + 6)].try_into().unwrap(),
						))),
					)
					.with_timeout(Duration::from_secs(1))
					.await;
				}
				unknown => warn!("Unexpected request packet: {}", unknown),
			}
		}
	}
}

// #[embassy_executor::task]
// pub async fn brain_rx(
// 	uart_tx: &'static SharedUart,
// 	mut uart_rx: UartRx<'static, uart::Async>,
// 	mut enable_pin: Output<'static>,
// ) {
// 	enable_pin.set_level(EN_PIN_RECIEVE);
// 	// let rx_buf = RX_BUFFER.init([0u8; _]);
// 	// let tx_buf = TX_BUFFER.init([0u8; _]);
// 	let cobs_buf = COBS_BUFFER.init([0u8; _]);

// 	let mut request_ticker = Ticker::every(Duration::from_millis(1000));
// 	loop {
// 		let select = select4(
// 			request_ticker.next(),
// 			LIDAR_MEASUREMENTS.ready_to_receive(),
// 			LATEST_READINGS.wait(),
// 			select_array(core::array::from_fn::<_, { ANGLES.len() }, _>(|i| {
// 				ANGLES[i].wait()
// 			})),
// 		)
// 		.await;
// 		match select {
// 			// Ping for updates from brain
// 			Either4::First(_) => {
// 				info!("Requesting brain updates");

// 				enable_pin.set_level(EN_PIN_SEND);
// 				Timer::after_millis(50).await;
// 				_ = uart.write(&[0x02, b'r', 0x00]).await;
// 				while uart.busy() {
// 					yield_now().await
// 				}
// 				enable_pin.set_level(EN_PIN_RECIEVE);
// 				if read_packet_from_brain(&mut uart, &mut cobs_buf[..])
// 					.with_timeout(Duration::from_millis(1000))
// 					.await
// 					.is_err()
// 				{
// 					warn!("Brain request timeout");
// 					continue;
// 				}
// 				let len = cobs::decode_in_place(&mut cobs_buf[..]).expect("todo don't panic");

// 				let mut cursor = 0;
// 				while cursor < len {
// 					cursor += 1;
// 					match cobs_buf[cursor] {
// 						b'c' => {
// 							_ = otos::SERVICE
// 								.request(OtosAction::Calibrate)
// 								.with_timeout(Duration::from_millis(250))
// 								.await;
// 						}
// 						b'p' => {
// 							_ = otos::SERVICE
// 								.request(OtosAction::SetPosition(bytemuck::must_cast::<
// 									[u8; 6],
// 									OtosPose,
// 								>(
// 									cobs_buf[cursor..(cursor + 6)].try_into().unwrap(),
// 								)))
// 								.with_timeout(Duration::from_millis(250))
// 								.await;
// 						}
// 						b's' => {
// 							_ = join(
// 								otos::SERVICE.request(OtosAction::SetScalars(
// 									bytemuck::must_cast::<[u8; 2], OtosScalars>(
// 										cobs_buf[cursor..(cursor + 2)].try_into().unwrap(),
// 									),
// 								)),
// 								otos::SERVICE.request(OtosAction::SetOffsets(
// 									bytemuck::must_cast::<[u8; 6], OtosPose>(
// 										cobs_buf[cursor..(cursor + 6)].try_into().unwrap(),
// 									),
// 								)),
// 							)
// 							.with_timeout(Duration::from_millis(250))
// 							.await;
// 						}
// 						unknown => warn!("Unexpected request packet: {}", unknown),
// 					}
// 				}
// 			}
// 			// Send lidar
// 			Either4::Second(_) => {
// 				info!("Sending LIDAR updates");

// 				// Encode message
// 				let mut cobs = CobsEncoder::new(&mut cobs_buf[..]);
// 				_ = cobs.push(b"l"); // Todo handle errors
// 				while let Ok(measurement) = LIDAR_MEASUREMENTS.try_receive() {
// 					_ = cobs.push(&bytemuck::must_cast::<_, [u8; size_of::<f32>()]>(
// 						measurement.angle,
// 					));
// 					_ = cobs.push(&bytemuck::must_cast::<_, [u8; size_of::<f32>()]>(
// 						measurement.distance,
// 					));
// 					_ = cobs.push(&[measurement.quality]);
// 				}
// 				let len = cobs.finalize();
// 				cobs_buf[len] = 0x00;
// 				// Send message
// 				enable_pin.set_level(EN_PIN_SEND);
// 				_ = uart.write(&cobs_buf[..=len]).await;
// 				enable_pin.set_level(EN_PIN_RECIEVE);
// 			}
// 			// Send OTOS
// 			Either4::Third(data) => {
// 				info!("Sending OTOS updates");

// 				let mut cobs = CobsEncoder::new(&mut cobs_buf[..]);
// 				_ = cobs.push(b"o");
// 				_ = cobs.push(&data);
// 				let len = cobs.finalize();
// 				cobs_buf[len] = 0x00;
// 				enable_pin.set_level(EN_PIN_SEND);
// 				_ = uart.write(&cobs_buf[..=len]).await;
// 				enable_pin.set_level(EN_PIN_RECIEVE);
// 			}
// 			// Send encoder position
// 			Either4::Fourth((angle, i)) => {
// 				info!("Sending encoder updates");

// 				let data = angle | ((i as u16) << 12);

// 				let mut cobs = CobsEncoder::new(&mut cobs_buf[..]);
// 				_ = cobs.push(b"a");
// 				_ = cobs.push(&data.to_le_bytes());
// 				let len = cobs.finalize();
// 				cobs_buf[len] = 0x00;
// 				enable_pin.set_level(EN_PIN_SEND);
// 				_ = uart.write(&cobs_buf[..=len]).await;
// 				enable_pin.set_level(EN_PIN_RECIEVE);
// 			}
// 		}
// 	}
// }

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

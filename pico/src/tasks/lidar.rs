use bitter::LittleEndianReader;
use defmt::{info, warn};
use embassy_futures::select::{Either, select};
use embassy_rp::uart::{Async, BufferedUartRx, UartTx};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::{BufRead as _, Read as _, ReadReady};

use rplidar::protocol::{
	BUFFER_SIZE, ParsingState, Request as _, Response, ResponseDescriptor, ResponseType,
	get_health::{GetHealthRequest, GetHealthResponse, HealthStatus},
	reset::ResetRequest,
	scan::{ScanRequest, ScanResponse},
};

const MEASUREMENT_COUNT: usize = 520; // Approximately the highest amount before a new scan
pub static LIDAR_MEASUREMENTS: Channel<CriticalSectionRawMutex, ScanResponse, MEASUREMENT_COUNT> =
	Channel::new();

/// Watches the incoming serial data from RPLIDAR, waiting for a specific response descriptor.
/// Will re-align parsing, so junk data doesn't cause problems, and times out after 1 second
async fn wait_response_descriptor(
	buf: &mut [u8],
	rx: &mut BufferedUartRx,
	kind: ResponseType,
) -> Option<ResponseDescriptor> {
	let fut = async {
		// info!("Looking for: {}", kind);
		let mut wanted_bytes = 1;
		let mut read = 0;
		loop {
			// Attempt to read one byte
			// info!(
			// 	"BEFORE: Wanted: {}, Read: {}, Slice: {:?}, Availible: {:?}",
			// 	wanted_bytes,
			// 	read,
			// 	&buf[..read],
			// 	rx.read_ready()
			// );
			if let Err(e) = rx.read_exact(&mut buf[read..(read + wanted_bytes)]).await {
				warn!(
					"UART error while waiting for LIDAR response descriptor: {}",
					e
				);
				continue;
			};
			read += wanted_bytes;

			// info!(
			// 	"AFTER: Wanted: {}, Read: {}, Slice: {:?}, Availible: {:?}",
			// 	wanted_bytes,
			// 	read,
			// 	&buf[..read],
			// 	rx.read_ready()
			// );

			// Try to parse our current run
			match ResponseDescriptor::parse(&mut LittleEndianReader::new(&buf[..read])) {
				// If it parses successfully and is the one we want, just return it
				ParsingState::Done(rd) if rd.response_type == kind => {
					// info!("Parsed full!");
					break rd;
				}
				// If it looks legit but unfinished, just keep reading bytes
				ParsingState::Unfinished(rest) => {
					// info!("Unfinished {}!", rest);
					{
						wanted_bytes = rest;
						continue;
					}
				}
				// If it doesn't look legit, or is an incorrect type reset the read count (ignore the data) and keep trying
				_ => {
					// info!("Invalid or incorrect type!");
					{
						read = 0;
						wanted_bytes = 1;
						continue;
					}
				}
			}
		}
	};

	// Return response descriptor or timeout after 1s
	match select(Timer::after(Duration::from_secs(3)), fut).await {
		Either::First(_) => {
			warn!("Reading {} response descriptor timed out", kind);
			None
		}
		Either::Second(rd) => Some(rd),
	}
}

#[embassy_executor::task]
pub async fn lidar_task(mut uart_tx: UartTx<'static, Async>, mut uart_rx: BufferedUartRx) {
	let mut buf = [0u8; BUFFER_SIZE];

	// Loop infinitely, if something goes wrong then just `continue 'outer` and it all resets
	'outer: loop {
		// Reset the lidar to make sure we're in a consistent state
		let len = ResetRequest.serialize(&mut buf);
		_ = uart_tx.write(&buf[..len]).await;

		// Ping the lidar until we get a healthy response
		loop {
			// Wait a sec to give it time to reboot and avoid spamming it
			Timer::after(Duration::from_millis(1000)).await;

			let len = GetHealthRequest.serialize(&mut buf);
			_ = uart_tx.write(&buf[..len]).await;

			// Wait for response descriptor
			let Some(rd) =
				wait_response_descriptor(&mut buf, &mut uart_rx, ResponseType::GetHealth).await
			else {
				warn!("LIDAR get health request timed out");
				continue;
			};

			// Read response
			if let Err(e) = uart_rx.read_exact(&mut buf[..rd.length]).await {
				warn!("Reading LIDAR get health response errored: {}", e);
				continue;
			}

			// Parse response
			let ParsingState::Done(health) =
				GetHealthResponse::parse(&mut LittleEndianReader::new(&buf[..rd.length]))
			else {
				warn!(
					"Parsing LIDAR get health response failed: {:?}",
					&buf[..rd.length]
				);
				continue;
			};

			// info!("Health: {:?}", health);
			// return;

			match health.status {
				HealthStatus::Good => {
					info!("LIDAR reported healthy status!");
					break;
				}
				HealthStatus::Warning => {
					warn!(
						"LIDAR returned warning status with error code: {}",
						health.error_code
					);
					break;
				}
				HealthStatus::Error => {
					warn!(
						"LIDAR reported error health with error code: {}",
						health.error_code
					);
					continue;
				}
			}
		}

		// Start scanning
		let len = ScanRequest.serialize(&mut buf);
		// info!("SCAN REQ: {:?}", &buf[..len]);
		_ = uart_tx.write(&buf[..len]).await;

		// Wait for response descriptor
		let Some(rd) = wait_response_descriptor(&mut buf, &mut uart_rx, ResponseType::Scan).await
		else {
			warn!("LIDAR scan request timed out");
			continue 'outer;
		};

		// Wait until we start getting data, with a longer timeout (wait for LIDAR to spin up)
		//
		// Kinda hacky, but the only way I see to await for data is to call fill_buf and never call consume
		// fill_buf will wait until at least some data is recieved, and then return a slice to it, without
		// actually marking it as read
		if let Either::Second(_) = select(uart_rx.fill_buf(), Timer::after_secs(5)).await {
			warn!("LIDAR failed to start sending scan data after 10s, resetting LIDAR");
			continue 'outer;
		}

		info!("Starting to read LIDAR data");
		let mut errors = 0;
		let mut start = Instant::now();
		let mut channel_filled = false;
		loop {
			// Read response with a timeout to catch if we get disconnected or LIDAR dies or something
			match select(
				uart_rx.read_exact(&mut buf[..rd.length]),
				Timer::after_secs(1),
			)
			.await
			{
				Either::First(Err(e)) => {
					warn!("Reading LIDAR get health response errored: {}", e);
					continue;
				}
				Either::Second(_) => {
					warn!("Reading LIDAR get health response timed out, resetting LIDAR");
					continue 'outer;
				}
				_ => (),
			}

			// Parse data
			let ParsingState::Done(data) =
				ScanResponse::parse(&mut LittleEndianReader::new(&buf[..rd.length]))
			else {
				warn!(
					"Parsing LIDAR scan response failed: {:?}",
					&buf[..rd.length]
				);
				errors += 1;
				if errors > 5 {
					warn!(
						"Parsing LIDAR scan response failed >5 consecutive times, resetting LIDAR"
					);
					continue 'outer;
				}
				continue;
			};
			errors = 0;

			// Push into channel
			if data.distance.is_finite() {
				match LIDAR_MEASUREMENTS.try_send(data) {
					Ok(_) if channel_filled => {
						info!(
							"LIDAR measurements channel has space, continuing to send measurements"
						);
						channel_filled = false
					}
					Err(_) if !channel_filled => {
						warn!("LIDAR measurements channel full, discarding measurements");
						channel_filled = true;
					}
					_ => (),
				}
			}

			// Print lidar measurements every so often
			if start.elapsed() > Duration::from_secs(1) {
				info!("Lidar: {:?}", data);
				start = Instant::now();
			}
		}
	}
}

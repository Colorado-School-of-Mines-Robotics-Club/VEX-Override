use bitter::LittleEndianReader;
use defmt::{info, warn};
use embassy_futures::select::{Either, select};
use embassy_rp::uart::{Async, BufferedUartRx, UartTx};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::Read as _;

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
		let mut wanted_bytes = 0;
		let mut read = 0;
		loop {
			// Attempt to read one byte
			if let Err(e) = rx.read_exact(&mut buf[read..wanted_bytes]).await {
				warn!(
					"UART error while waiting for LIDAR response descriptor: {}",
					e
				);
				continue;
			};
			read += 1;

			// Try to parse our current run
			match ResponseDescriptor::parse(&mut LittleEndianReader::new(&buf[..read])) {
				// If it looks legit, just keep reading bytes
				ParsingState::Unfinished(rest) => {
					wanted_bytes = rest;
					continue;
				}
				// If it doesn't look legit, reset the read count (ignore the data) and keep trying
				ParsingState::Invalid => {
					read = 0;
					wanted_bytes = 1;
					continue;
				}
				// If we parsed a response descriptor, but it is not what we expected, ignore it
				ParsingState::Done(rd) if rd.response_type != kind => {
					read = 0;
					wanted_bytes = 1;
					continue;
				}
				// If we parsed a response descriptor and it is what we expected, just return it :3
				ParsingState::Done(rd) => break rd,
			}
		}
	};

	// Return response descriptor or timeout after 1s
	match select(Timer::after(Duration::from_secs(1)), fut).await {
		Either::First(_) => None,
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
			Timer::after(Duration::from_millis(500)).await;

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

			match health.status {
				HealthStatus::Good => break,
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
		_ = uart_tx.write(&buf[..len]).await;

		// Wait for response
		let Some(rd) = wait_response_descriptor(&mut buf, &mut uart_rx, ResponseType::Scan).await
		else {
			warn!("LIDAR scan request timed out");
			continue 'outer;
		};

		info!("Starting to read LIDAR data");
		let mut errors = 0;
		let mut start = Instant::now();
		loop {
			// Parse data
			let ParsingState::Done(data) = ScanResponse::parse(&mut LittleEndianReader::new(&buf))
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
				if LIDAR_MEASUREMENTS.is_full() {
					warn!("LIDAR measurements channel full");
				}
				LIDAR_MEASUREMENTS.send(data).await;
			}

			// Print lidar measurements every so often
			if start.elapsed() > Duration::from_secs(1) {
				info!("Lidar: {:?}", data);
				start = Instant::now();
			}
		}
	}
}

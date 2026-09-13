use core::ops::Index;

use bitter::LittleEndianReader;
use defmt::{error, info, warn};
use embassy_futures::select::{Either, select};
use embassy_rp::uart::{Async, BufferedUart, BufferedUartRx, Uart, UartTx};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::{BufRead, Read as _, Write as _};

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
		let mut read = 0;
		loop {
			let len = match rx.fill_buf().await {
				Ok(data) => {
					if let Some(range) = buf.get_mut(read..(read + data.len())) {
						range.copy_from_slice(data);
					} else {
						warn!("Response descriptor buffer overflow");
						read = 0;
						buf[..data.len()].copy_from_slice(data);
					}
					read += data.len();
					data.len()
				}
				Err(e) => {
					warn!(
						"UART error while waiting for LIDAR response descriptor: {}",
						e
					);
					continue;
				}
			};

			// Attempt to parse a packet
			for start in (read - len)..read {
				if let ParsingState::Done(rd) = ResponseDescriptor::parse(
					&mut LittleEndianReader::new(&buf[(read - len)..read]),
				) {
					return rd;
				}
			}
		}
	};

	// Return found response descriptor or timeout
	match select(Timer::after(Duration::from_secs(1)), fut).await {
		Either::First(_) => None,
		Either::Second(rd) => Some(rd),
	}
}

#[embassy_executor::task]
pub async fn lidar_task(mut uart_tx: UartTx<'static, Async>, mut uart_rx: BufferedUartRx) {
	let mut buf = [0u8; BUFFER_SIZE];

	// Loop infinitely, if something goes wrong then just `continue 'outer` and it all resets
	loop {}
}

// #[embassy_executor::task]
// pub async fn lidar_task(mut uart: BufferedUart) {
// 	let mut buf = [0u8; BUFFER_SIZE];

// 	// Reset lidar in case it was already running
// 	let len = ResetRequest.serialize(&mut buf);
// 	_ = uart.write(&buf[..len]).await;

// 	// Wait for 1s, dumping out of uart in the meantime to clean up the buffer
// 	Timer::after(Duration::from_millis(1000)).await;
// 	_ = uart.fill_buf().await;

// 	// Attempt to ping the LIDAR until it indicates healthy status
// 	loop {
// 		Timer::after(Duration::from_secs(1)).await;

// 		// Send request
// 		let len = GetHealthRequest.serialize(&mut buf);
// 		if let Err(e) = uart.write(&buf[..len]).await {
// 			error!("Failed to write get info request to LIDAR: {:?}", e);
// 			continue;
// 		}

// 		// Read and parse response descriptor
// 		if let Err(e) = uart.read_exact(&mut buf[..ResponseDescriptor::SIZE]).await {
// 			error!(
// 				"Failed to read get health response descriptor from LIDAR: {:?}",
// 				e
// 			);
// 			continue;
// 		}

// 		let Some(response_descriptor) =
// 			ResponseDescriptor::parse(&mut LittleEndianReader::new(&buf))
// 		else {
// 			error!("Failed to parse get health response descriptor from LIDAR");
// 			continue;
// 		};

// 		// Read and parse response
// 		if let Err(e) = uart
// 			.read_exact(&mut buf[..response_descriptor.length])
// 			.await
// 		{
// 			error!("Failed to read get health response from LIDAR: {:?}", e);
// 			continue;
// 		}

// 		let Some(health) = GetHealthResponse::parse(&mut LittleEndianReader::new(&buf)) else {
// 			error!("Failed to parse get health response descritor from LIDAR");
// 			continue;
// 		};

// 		// Verify sensor healthy
// 		match health.status {
// 			HealthStatus::Good => {
// 				info!("LIDAR works!!!");
// 			}
// 			HealthStatus::Warning => {
// 				warn!("LIDAR reported warning: erorr code {}", health.error_code)
// 			}
// 			HealthStatus::Error => {
// 				error!("LIDAR reported error: erorr code {}", health.error_code);
// 				continue;
// 			}
// 		}

// 		break;
// 	}

// 	// Start LIDAR scan
// 	let len = ScanRequest.serialize(&mut buf);
// 	_ = uart.write(&buf[..len]).await;

// 	loop {
// 		// Read and parse response descriptor
// 		if let Err(e) = uart.read_exact(&mut buf[..ResponseDescriptor::SIZE]).await {
// 			error!(
// 				"Failed to read start scan response descriptor from LIDAR: {:?}",
// 				e
// 			);
// 			return;
// 		}

// 		let Some(rd) = ResponseDescriptor::parse(&mut LittleEndianReader::new(&buf)) else {
// 			info!("{:?}", &buf);
// 			error!("Failed to parse start scan response descriptor from LIDAR");
// 			return;
// 		};

// 		if rd.response_type == ResponseType::Scan {
// 			break;
// 		}
// 	}
// 	info!("Lidar started reading");

// 	// Start parsing data
// 	let mut start = Instant::now();
// 	loop {
// 		// Read data
// 		if let Err(e) = uart.read_exact(&mut buf[..5]).await {
// 			error!("Failed to read scan response from LIDAR: {:?}", e);
// 			continue;
// 		}

// 		// Parse data
// 		let Some(data) = ScanResponse::parse(&mut LittleEndianReader::new(&buf)) else {
// 			error!("Failed to parse scan response from LIDAR");
// 			continue;
// 		};

// 		// Push into channel
// 		if data.distance.is_finite() {
// 			if LIDAR_MEASUREMENTS.is_full() {
// 				warn!("Lidar channel filled!!!");
// 			}
// 			LIDAR_MEASUREMENTS.send(data).await;
// 		}

// 		if start.elapsed() > Duration::from_secs(1) {
// 			info!("Lidar: {:?}", data);
// 			start = Instant::now();
// 		}
// 	}
// }

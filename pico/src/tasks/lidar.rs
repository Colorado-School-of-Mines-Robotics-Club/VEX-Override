use bitter::LittleEndianReader;
use defmt::{error, warn};
use embassy_rp::uart::{self, Uart};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};

use rplidar::protocol::{
	BUFFER_SIZE, Request as _, Response, ResponseDescriptor,
	get_health::{GetHealthRequest, GetHealthResponse, HealthStatus},
	scan::{ScanRequest, ScanResponse},
};

const MEASUREMENT_COUNT: usize = (360.0 / 0.72) as usize; // Store one full rotation
static LIDAR_MEASUREMENTS: Channel<CriticalSectionRawMutex, ScanResponse, MEASUREMENT_COUNT> =
	Channel::new();

#[embassy_executor::task]
pub async fn lidar_task(mut uart: Uart<'static, uart::Async>) {
	let mut buf = [0u8; BUFFER_SIZE];

	// Attempt to ping the LIDAR until it indicates healthy status
	loop {
		Timer::after(Duration::from_secs(1)).await;

		// Send request
		let len = GetHealthRequest.serialize(&mut buf);
		if let Err(e) = uart.write(&buf[..len]).await {
			error!("Failed to write get info request to LIDAR: {:?}", e);
			continue;
		}

		// Read and parse response descriptor
		if let Err(e) = uart.read(&mut buf[..ResponseDescriptor::SIZE]).await {
			error!(
				"Failed to read get health response descriptor from LIDAR: {:?}",
				e
			);
			continue;
		}

		let Some(response_descriptor) =
			ResponseDescriptor::parse(&mut LittleEndianReader::new(&buf))
		else {
			error!("Failed to parse get health response descriptor from LIDAR");
			continue;
		};

		// Read and parse response
		if let Err(e) = uart.read(&mut buf[..response_descriptor.length]).await {
			error!("Failed to read get health response from LIDAR: {:?}", e);
			continue;
		}

		let Some(health) = GetHealthResponse::parse(&mut LittleEndianReader::new(&buf)) else {
			error!("Failed to parse get health response descritor from LIDAR");
			continue;
		};

		// Verify sensor healthy
		match health.status {
			HealthStatus::Good => (),
			HealthStatus::Warning => {
				warn!("LIDAR reported warning: erorr code {}", health.error_code)
			}
			HealthStatus::Error => {
				error!("LIDAR reported error: erorr code {}", health.error_code);
				continue;
			}
		}

		break;
	}

	// Start LIDAR scan
	let len = ScanRequest.serialize(&mut buf);
	_ = uart.write(&buf[..len]).await;

	// Read and parse response descriptor
	if let Err(e) = uart.read(&mut buf[..ResponseDescriptor::SIZE]).await {
		error!(
			"Failed to read start scan response descriptor from LIDAR: {:?}",
			e
		);
		return;
	}

	let Some(response_descriptor) = ResponseDescriptor::parse(&mut LittleEndianReader::new(&buf))
	else {
		error!("Failed to parse start scan response descriptor from LIDAR");
		return;
	};

	// Start parsing data
	loop {
		// Read data
		if let Err(e) = uart.read(&mut buf[..response_descriptor.length]).await {
			error!("Failed to read scan response from LIDAR: {:?}", e);
			continue;
		}

		// Parse data
		let Some(data) = ScanResponse::parse(&mut LittleEndianReader::new(&buf)) else {
			error!("Failed to parse scan response from LIDAR");
			continue;
		};

		// Push into channel
		LIDAR_MEASUREMENTS.send(data).await;
	}
}

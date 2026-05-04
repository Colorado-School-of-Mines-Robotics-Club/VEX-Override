use bitter::{BitReader as _, LittleEndianReader};

use crate::lidar::protocol::{Request, Response};

pub struct GetInfoRequest;

impl Request for GetInfoRequest {
	const TAG: u8 = 0x50;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

pub struct GetInfoResponse {
	/// The sub-model of the RPLIDAR
	///
	/// Example: RPLIDAR S2M1: 1
	pub sub_model: u8,
	/// The major model of the RPLIDAR
	///
	/// S-series should be subtracted by 5
	///
	/// Examples:
	/// - RPLIDAR S2M1: 7
	/// - RPLIDAR C1: 1
	pub major_model: u8,
	/// The decimal part of the firmware version number
	pub firmware_minor: u8,
	/// The integer part of the firmware version number
	pub firmware_major: u8,
	/// Undocumented
	pub hardware: u8,
	/// The serial number of this RPLIDAR
	///
	/// When converting to text in hex, the LSB prints first
	pub serial_number: [u8; 16],
}

impl Response for GetInfoResponse {
	fn parse(reader: &mut LittleEndianReader) -> Option<Self> {
		let sub_model = reader.read_bits(4)? as u8;
		let major_model = reader.read_bits(4)? as u8;
		let firmware_minor = reader.read_u8()?;
		let firmware_major = reader.read_u8()?;
		let hardware = reader.read_u8()?;
		let mut serial_number = [0u8; _];
		if !reader.read_bytes(&mut serial_number) {
			return None;
		};

		Some(Self {
			sub_model,
			major_model,
			firmware_minor,
			firmware_major,
			hardware,
			serial_number,
		})
	}
}

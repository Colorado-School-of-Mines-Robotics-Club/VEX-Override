use bitter::{BitReader as _, LittleEndianReader};

use crate::protocol::{Request, Response};

#[derive(Debug)]
pub struct GetInfoRequest;

impl Request for GetInfoRequest {
	const TAG: u8 = 0x50;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

#[derive(Debug)]
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
const C_MINIMUM_MAJOR_ID: u8 = 4;
const S_MINIMUM_MAJOR_ID: u8 = 6;
const T_MINIMUM_MAJOR_ID: u8 = 9;
const M_MINIMUM_MAJOR_ID: u8 = 12;

#[cfg(feature = "std")]
impl std::fmt::Display for GetInfoResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let major_model_category = match self.major_model {
			..C_MINIMUM_MAJOR_ID => 'A',
			C_MINIMUM_MAJOR_ID..S_MINIMUM_MAJOR_ID => 'C',
			S_MINIMUM_MAJOR_ID..T_MINIMUM_MAJOR_ID => 'S',
			T_MINIMUM_MAJOR_ID..M_MINIMUM_MAJOR_ID => 'T',
			M_MINIMUM_MAJOR_ID.. => 'M',
		};
		let major_model_version = match major_model_category {
			'A' => self.major_model,
			'S' => self.major_model - S_MINIMUM_MAJOR_ID + 1,
			'T' => self.major_model - T_MINIMUM_MAJOR_ID + 1,
			'M' => self.major_model - M_MINIMUM_MAJOR_ID + 1,
			'C' => self.major_model - C_MINIMUM_MAJOR_ID + 1,
			_ => unreachable!(), // todo enum
		};

		f.write_fmt(format_args!(
			"RPLIDAR {}{}M{}, firmware v{}.{}, hardware v{}, serial ",
			major_model_category,
			major_model_version,
			self.sub_model,
			self.firmware_major,
			self.firmware_minor,
			self.hardware,
		))?;
		for byte in self.serial_number {
			f.write_fmt(format_args!("{:X}", byte))?;
		}

		Ok(())
	}
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

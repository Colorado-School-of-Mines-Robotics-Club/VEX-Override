use bitter::{BitReader as _, LittleEndianReader};

use crate::lidar::protocol::{Request, Response};

pub struct GetSampleRateRequest;

impl Request for GetSampleRateRequest {
	const TAG: u8 = 0x59;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

pub struct GetSampleRateResponse {
	/// The sample rate of standard mode in microseconds
	pub standard: u16,
	/// The sample rate of express mode in microseconds
	pub express: u16,
}

impl Response for GetSampleRateResponse {
	fn parse(reader: &mut LittleEndianReader) -> Option<Self> {
		Some(Self {
			standard: reader.read_u16()?,
			express: reader.read_u16()?,
		})
	}
}

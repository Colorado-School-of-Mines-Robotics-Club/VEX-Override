use bitter::{BitReader as _, LittleEndianReader};

use crate::protocol::{ParsingState, Request, Response};

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
	fn parse(reader: &mut LittleEndianReader) -> ParsingState<Self> {
		let Some(standard) = reader.read_u16() else {
			return ParsingState::Unfinished(4 - reader.bytes_remaining());
		};
		let Some(express) = reader.read_u16() else {
			return ParsingState::Unfinished(2 - reader.bytes_remaining());
		};

		ParsingState::Done(Self { standard, express })
	}
}

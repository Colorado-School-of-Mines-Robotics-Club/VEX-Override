use bitter::{BitReader as _, LittleEndianReader};
use bytemuck::Contiguous;

use crate::protocol::{ParsingState, Request, Response};

pub struct GetHealthRequest;

impl Request for GetHealthRequest {
	const TAG: u8 = 0x52;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Contiguous, defmt::Format)]
pub enum HealthStatus {
	Good = 0x00,
	Warning = 0x01,
	Error = 0x02,
}

#[derive(Debug, defmt::Format)]
pub struct GetHealthResponse {
	pub status: HealthStatus,
	pub error_code: u16,
}

impl Response for GetHealthResponse {
	fn parse(reader: &mut LittleEndianReader) -> ParsingState<Self> {
		let Some(status) = reader.read_u8() else {
			return ParsingState::Unfinished(3);
		};
		let Some(status) = HealthStatus::from_integer(status) else {
			return ParsingState::Invalid;
		};
		let Some(error_code) = reader.read_u16() else {
			return ParsingState::Unfinished(2 - reader.bytes_remaining());
		};

		ParsingState::Done(Self { status, error_code })
	}
}

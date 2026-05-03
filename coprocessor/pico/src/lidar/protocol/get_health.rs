use bitter::{BitReader as _, LittleEndianReader};
use bytemuck::Contiguous;

use crate::lidar::protocol::{Request, Response};

pub struct GetHealthRequest;

impl Request for GetHealthRequest {
	const TAG: u8 = 0x52;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

#[repr(u8)]
#[derive(Clone, Copy, Contiguous)]
pub enum HealthStatus {
	Good = 0x00,
	Warning = 0x01,
	Error = 0x02,
}

pub struct GetHealthResponse {
	pub status: HealthStatus,
	pub error_code: u16,
}

impl Response for GetHealthResponse {
	fn parse(reader: &mut LittleEndianReader) -> Option<Self> {
		let status = HealthStatus::from_integer(reader.read_u8()?)?;
		let error_code = reader.read_u16()?;

		Some(Self { status, error_code })
	}
}

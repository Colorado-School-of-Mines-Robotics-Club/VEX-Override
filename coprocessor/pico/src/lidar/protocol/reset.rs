use crate::lidar::protocol::Request;

pub struct ResetRequest;

impl Request for ResetRequest {
	const TAG: u8 = 0x40;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

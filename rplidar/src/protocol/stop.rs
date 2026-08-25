use crate::protocol::Request;

pub struct StopRequest;

impl Request for StopRequest {
	const TAG: u8 = 0x25;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

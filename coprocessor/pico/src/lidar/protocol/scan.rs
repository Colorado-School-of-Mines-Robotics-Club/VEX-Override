use bitter::{BitReader as _, LittleEndianReader};
use bytemuck::Zeroable;

use crate::lidar::protocol::{Request, Response};

pub struct ScanRequest;

impl Request for ScanRequest {
	const TAG: u8 = 0x20;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

#[derive(Zeroable)]
pub struct ScanResponse {
	/// Whether this measurement is the start of a new scan
	pub start: bool,
	/// The quality of this measurement as determined by the reflected pulse strength
	pub quality: u8,
	/// The angle this measurement was taken at in degrees
	pub angle: f32,
	/// The distance this measurement determined in mm, or NaN if it failed
	pub distance: f32,
}

impl Response for ScanResponse {
	fn parse(reader: &mut LittleEndianReader) -> Option<Self> {
		// Get start flag
		let start = reader.read_bit()?;
		// Ensure inverse start flag is correct, or fail early
		if reader.read_bit()? != start {
			return None;
		};

		// Next 6 bits are quality, undetermined what they actually mean
		let quality = reader.read_bits(6)? as u8;

		// Next bit is a check bit, should always be 1
		if !reader.read_bit()? {
			return None;
		}

		// Next is 15 bits are angle, divide by 64.0 to get degrees
		let angle = reader.read_bits(15)? as f32 / 64.0 % 360.0;

		// Last 16 bits are the distance, or 0 for a failed measurement
		let distance = reader.read_u16()?;
		let distance = if distance == 0 {
			f32::NAN
		} else {
			distance as f32 / 4.0
		};

		Some(ScanResponse {
			start,
			quality,
			angle,
			distance,
		})
	}
}

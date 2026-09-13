use bitter::{BitReader as _, LittleEndianReader};
use bytemuck::Zeroable;

use crate::protocol::{ParsingState, Request, Response};

pub struct ScanRequest;

impl Request for ScanRequest {
	const TAG: u8 = 0x20;
	const MAX_PAYLOAD_LENGTH: u8 = 0;
}

#[derive(Debug, Zeroable, Clone, Copy, Default, defmt::Format)]
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

#[cfg(feature = "std")]
impl std::fmt::Display for ScanResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"({} deg, {} m, {} quality)",
			self.angle, self.distance, self.quality
		))
	}
}

impl Response for ScanResponse {
	fn parse(reader: &mut LittleEndianReader) -> ParsingState<Self> {
		// Check start flag and inverse start flag
		let start = match reader.read_bits(2) {
			Some(0b11 | 0b00) => return ParsingState::Invalid,
			Some(v) => (v >> 1) & 0b1 == 0b1,
			None => return ParsingState::Unfinished,
		};

		// Next 6 bits are quality, undetermined what they actually mean
		let Some(quality) = reader.read_bits(6).map(|v| v as u8) else {
			return ParsingState::Unfinished;
		};

		// Next bit is a check bit, should always be 1
		match reader.read_bit() {
			Some(true) => (),
			Some(false) => return ParsingState::Invalid,
			None => return ParsingState::Unfinished,
		};

		// Next is 15 bits are angle, divide by 64.0 to get degrees
		let Some(angle) = reader.read_bits(15).map(|v| v as f32 / 64.0 % 360.0) else {
			return ParsingState::Unfinished;
		};

		// Last 16 bits are the distance, or 0 for a failed measurement
		let distance = match reader.read_u16() {
			Some(0) => f32::NAN,
			Some(v) => v as f32 / 4.0,
			None => return ParsingState::Unfinished,
		};

		ParsingState::Done(ScanResponse {
			start,
			quality,
			angle,
			distance,
		})
	}
}

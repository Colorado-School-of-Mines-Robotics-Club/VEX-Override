//! Reference: https://bucket-download.slamtec.com/c5971f2703a8d014f3925694d798ea490a370efa/LR001_SLAMTEC_rplidar_S&C%20series_protocol_v2.8_en.pdf

use bitter::{BitReader, LittleEndianReader};

// TODO:
// 	- Write async request/response function using embedded-io-async
//  - Factor out into seperate library

pub mod get_health;
pub mod get_info;
// pub mod get_lidar_conf; this api is annoying to impl ill do it later
pub mod get_sample_rate;
pub mod motor_speed_ctrl;
pub mod reset;
pub mod scan;
pub mod stop;

const REQUEST_TAG: u8 = 0xA5;
const RESPONSE_DESCRIPTOR_TAG: [u8; 2] = [0xA5, 0x5A];

const SINGLE_REQUEST_SINGLE_RESPONSE: u8 = 0x0;
const SINGLE_REQUEST_MULTIPLE_RESPONSE: u8 = 0x1;

/// The recommended buffer size for doing I/O with the LIDAR. This is the maximum of the non-variable request and response sizes.
pub const BUFFER_SIZE: usize = 20;
// Known request sizes are completely bounded, with a max of 5 for express scan
pub const MAX_REQUEST_SIZE: usize = 5;
// The only unbounded response is a string which the max should be 8
// The maximum bounded response (including descriptor) would be GetInfoRequest with 20 bytes
pub const MAX_RESPONSE_SIZE: usize = 20;

/// An enum representing the state of a parser.
///
/// States:
/// 1. Unfinished - So far so good, but needs more data (minimum is returned)
/// 2. Invalid - An error was encountered, this doesn't look like valid data
/// 3. Done - Finished parsing successfully
pub enum ParsingState<T> {
	Unfinished(usize),
	Invalid,
	Done(T),
}

pub trait Response
where
	Self: Sized,
{
	fn parse(reader: &mut LittleEndianReader) -> ParsingState<Self>;
}

pub trait Request {
	/// The tag for this request
	const TAG: u8;
	/// The maximum length the payload can be for this request
	const MAX_PAYLOAD_LENGTH: u8;

	/// The maximum length of the serialized request
	const MAX_LENGTH: usize = 1 /* REQUEST_TAG */
		+ 1 /* Command */
		+ if Self::MAX_PAYLOAD_LENGTH != 0 {
			1 /* Payload length */
			+ Self::MAX_PAYLOAD_LENGTH as usize
			+ 1 /* Checksum */
		} else { 0 };

	/// Serializes the request into a given buffer
	///
	/// The provided buffer must be of size [Self::MAX_SIZE] or greater
	///
	/// Returns the amount of written bytes
	fn serialize(&self, mut buf: &mut [u8]) -> usize {
		let start_len = buf.len();

		// Add tags to the request, this is the bare minimum for a request
		buf[0] = REQUEST_TAG;
		buf[1] = Self::TAG;
		buf = &mut buf[2..];

		// If the request can have a payload, serialize it and a checksum
		if Self::MAX_PAYLOAD_LENGTH != 0 {
			// Serialize payload, skipping a byte (so we can write the length back to it)
			let payload_len = self.serialize_payload(&mut buf[1..]);
			buf[0] = payload_len as u8;

			// Calculate checksum
			let mut checksum = 0;
			checksum ^= REQUEST_TAG;
			checksum ^= Self::TAG;
			checksum ^= payload_len as u8;

			for byte in buf[1..(payload_len + 1)].iter() {
				checksum ^= *byte;
			}

			// Advance buffer the payload length
			buf = &mut buf[(payload_len + 1)..];

			// Write checksum to buffer
			buf[0] = checksum;
			buf = &mut buf[1..];
		}

		start_len - buf.len()
	}

	/// Serializes the payload for a request
	///
	/// This is the function that should be implemented for requests, but for payload-less types there is a default implementation
	///
	/// Returns the amount of written bytes
	#[allow(unused_variables)]
	fn serialize_payload(&self, buf: &mut [u8]) -> usize {
		0
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, defmt::Format)]
pub enum ResponseMode {
	SingleResponse,
	MultipleResponse,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, defmt::Format)]
pub enum ResponseType {
	Scan,
	GetInfo,
	GetHealth,
	GetSampleRate,
	GetLidarConf,
}

#[derive(Debug, defmt::Format)]
pub struct ResponseDescriptor {
	pub length: usize,
	pub mode: ResponseMode,
	pub response_type: ResponseType,
}

impl ResponseDescriptor {
	pub const SIZE: usize = 7;
}

impl Response for ResponseDescriptor {
	fn parse(reader: &mut LittleEndianReader) -> ParsingState<Self> {
		// let Some(tag_start) = reader.read_u8() else {
		// 	return ParsingState::Unfinished(7);
		// };
		// if tag_start != RESPONSE_DESCRIPTOR_TAG[0] {
		// 	return ParsingState::Invalid;
		// }
		// let Some(tag_end) = reader.read_u8() else {
		// 	return ParsingState::Unfinished(6);
		// };
		// if [tag_start, tag_end] != RESPONSE_DESCRIPTOR_TAG {
		// 	return ParsingState::Invalid;
		// };
		//
		for (i, &tag) in RESPONSE_DESCRIPTOR_TAG.iter().enumerate() {
			match reader.read_u8() {
				None => return ParsingState::Unfinished(7 - i),
				Some(v) if v != tag => return ParsingState::Invalid,
				_ => (),
			}
		}

		let Some(length) = reader.read_bits(30).map(|v| v as usize) else {
			return ParsingState::Unfinished(5 - reader.bytes_remaining());
		};

		ParsingState::Done(Self {
			length,
			mode: match reader.read_bits(2).map(|v| v as u8) {
				Some(SINGLE_REQUEST_SINGLE_RESPONSE) => ResponseMode::SingleResponse,
				Some(SINGLE_REQUEST_MULTIPLE_RESPONSE) => ResponseMode::MultipleResponse,
				Some(_) => return ParsingState::Invalid,
				None => return ParsingState::Unfinished(2),
			},
			response_type: match reader.read_u8() {
				Some(0x81) => ResponseType::Scan,
				Some(0x04) => ResponseType::GetInfo,
				Some(0x06) => ResponseType::GetHealth,
				Some(0x15) => ResponseType::GetSampleRate,
				Some(0x20) => ResponseType::GetLidarConf,
				Some(_) => return ParsingState::Invalid,
				None => return ParsingState::Unfinished(1),
			},
		})
	}
}

use crate::lidar::protocol::Request;

pub struct MotorSpeedCtrlRequest {
	/// The angular speed of the motor in RPM to use for the LIDAR
	///
	/// A value of zero will put the LIDAR into idle mode.
	pub rpm: u16,
}

impl Request for MotorSpeedCtrlRequest {
	const TAG: u8 = 0x02;
	const MAX_PAYLOAD_LENGTH: u8 = 2;

	fn serialize_payload(&self, buf: &mut [u8]) -> usize {
		buf.copy_from_slice(&self.rpm.to_le_bytes());

		2
	}
}

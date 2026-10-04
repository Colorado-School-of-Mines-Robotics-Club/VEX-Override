use embedded_hal_async::i2c::I2c;

use crate::i2c_regs;

// https://files.seeedstudio.com/wiki/Grove-12-bit-Magnetic-Rotary-Position-Sensor-AS5600/res/Magnetic%20Rotary%20Position%20Sensor%20AS5600%20Datasheet.pdf
i2c_regs!(
	0x36;
	// How many times ZPOS/MPOS have been burned, can only burn 3 times
	//
	// 2 bits
	ZMCO: 0x00,

	// The programmed start position of the angle range to measure
	// 12 bits
	ZPOS_H: 0x01,
	ZPOS_L: ZPOS_H + 1,

	// The programmed end position of the angle range to measure
	// 12 bits
	MPOS_H: 0x03,
	MPOS_L: MPOS_H + 1,

	// The size of the angle range to measure
	// 12 bits
	MANG_H: 0x05,
	MANG_L: MANG_H + 1,

	// 1:0 - Power mode (00 = NOM, 01 = LPM1, 10 = LPM2, 11 = LPM3)
	// 3:2 - Hysteris (00 = OFF, 01 = 1 LSB, 10 = 2 LSBs, 11 = 3 LSBs)
	// 5:4 - Output stage (00 = Analog, 01 = Reduced analog, 10 = PWM)
	// 7:6 - PWM frequency (00 = 115 Hz; 01 = 230 Hz; 10 = 460 Hz; 11 = 920 Hz)
	// 9:8 - Slow filter (00 = 16x, 01 = 8x, 10 = 4x, 11 = 2x)
	// 12:10 - Fast filter threshold (000 = slow filter only, 001 = 6 LSBs, 010 = 7 LSBs, 011 = 9 LSBs, 100 = 18 LSBs, 101 = 21 LSBs, 110 = 24 LSBs, 111 = 10 LSBs)
	// 13 - Watchdog (0 = OFF, 1 = ON)
	CONF_H: 0x07,
	CONF_L: CONF_H + 1,

	// The unscaled/unmodified angle measurement
	// 12 bits
	RAW_ANGLE_H: 0x0C,
	RAW_ANGLE_L: RAW_ANGLE_H + 1,

	// The angle measurement
	// 12 bits
	ANGLE_H: 0x0E,
	ANGLE_L: ANGLE_H + 1,

	// 0:2 - Undefined
	// 3 - AGC minimum gain overflow (magnet too strong)
	// 4 - AGC minimum gain underflow (magnet too weak)
	// 5 - Magnet detected
	// 6:8 - Undefined
	STATUS: 0x0B,

	// Current AGC gain value
	AGC: 0x1A,

	// Magnitude of internal CORDIC
	// 12 bits
	MAGNITUDE_H: 0x1B,
	MAGNITUDE_L: MAGNITUDE_H + 1,

	// Write to this register to burn values onto sensor
	// 0x80 - Burn angle (ZPOS, MPOS)
	// Can only be done 3 times
	//
	// 0x40 - Burn settings (MANG, CONF)
	// Can only be done once
	// MANG can only be burned if ZPOS and MPOS have never been burned
	BURN: 0xFF
);

pub struct AS5600Encoder<T: I2c> {
	inner: T,
}

impl<T: I2c> AS5600Encoder<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}

	pub async fn read_angle(&mut self) -> Result<u16, T::Error> {
		let mut buf = [0u8; 2];
		self.inner.write_read(ADDR, &[ANGLE_H], &mut buf).await?;

		Ok(u16::from_be_bytes(buf) & 0x0FFF)
	}
}

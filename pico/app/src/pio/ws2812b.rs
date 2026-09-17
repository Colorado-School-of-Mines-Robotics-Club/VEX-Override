use embassy_rp::{
	Peri,
	clocks::clk_sys_freq,
	pio::{self, FifoJoin, PioPin, ShiftConfig, ShiftDirection, StateMachine, program::pio_file},
};
use fixed::types::U24F8;

pub fn setup_ws2812b_sm<'a, PIO: pio::Instance, const SM: usize, Pin: PioPin>(
	pio: &mut pio::Common<'a, PIO>,
	sm: &mut StateMachine<'a, PIO, SM>,
	pin: Peri<'a, Pin>,
	data_rate: u32,
) {
	let program = pio_file!("src/pio/ws2812b.pio", select_program("ws2812"));

	let mut cfg = pio::Config::default();
	cfg.fifo_join = FifoJoin::TxOnly;
	cfg.use_program(
		&pio.load_program(&program.program),
		&[&pio.make_pio_pin(pin)],
	);
	cfg.shift_out = ShiftConfig {
		auto_fill: true,
		threshold: 24,
		direction: ShiftDirection::Left,
	};

	let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
	let ws2812_freq = U24F8::from_num(data_rate / 1000);
	let bit_freq = ws2812_freq
		* (program.public_defines.T1 + program.public_defines.T2 + program.public_defines.T3)
			as u32;

	cfg.clock_divider = clock_freq / bit_freq;

	sm.set_config(&cfg);
	sm.set_enable(true);
}

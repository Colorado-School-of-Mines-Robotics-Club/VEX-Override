use embassy_rp::{
	Peri,
	clocks::clk_sys_freq,
	pio::{self, FifoJoin, PioPin, StateMachine, program::pio_file},
};
use fixed::types::U24F8;

pub fn setup_blinker_sm<'a, PIO: pio::Instance, const SM: usize, Pin: PioPin>(
	pio: &mut pio::Common<'a, PIO>,
	sm: &mut StateMachine<'a, PIO, SM>,
	pin: Peri<'a, Pin>,
) {
	let program = pio_file!("src/pio/blinker.pio");

	let mut cfg = pio::Config::default();
	cfg.fifo_join = FifoJoin::TxOnly;
	cfg.use_program(&pio.load_program(&program.program), &[]);
	cfg.set_set_pins(&[&pio.make_pio_pin(pin)]);
	cfg.clock_divider =
		U24F8::strict_from_num(clk_sys_freq() as f64 / program.public_defines.FREQUENCY as f64);
	sm.set_config(&cfg);
}

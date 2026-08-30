use embassy_rp::{
	Peripherals, bind_interrupts, dma,
	gpio::{Input, Level, Output, Pull},
	i2c::{self, I2c},
	peripherals::*,
	pio::{self, Pio, StateMachine},
	uart::{self, DataBits, Parity, StopBits, Uart},
	usb,
	watchdog::{self, Watchdog},
};

use crate::pio::{blinker::setup_blinker_sm, ws2812b::setup_ws2812b_sm};

// Define aliases for the peripheral types to reduce duplication
pub type OtosI2C = I2C0;
pub type BrainUART = UART0;
pub type BrainUARTTxDMA = DMA_CH0;
pub type BrainUARTRxDMA = DMA_CH1;
pub type BlinkerPIO = PIO0;
pub const BLINKER_STATE_MACHINE: usize = 0;
pub type BlinkerPin = PIN_2;
pub type LedsPIO = PIO0;
pub const LEDS_STATE_MACHINE: usize = 1;
pub type LedsPin = PIN_6;
pub type LedsDMA = DMA_CH2;
pub type LidarUART = UART1;
pub type LidarUARTTxDMA = DMA_CH3;
pub type LidarUARTRxDMA = DMA_CH4;

// Link interrupts to their proper handlers
bind_interrupts!(pub struct Irq {
	UART0_IRQ => uart::InterruptHandler<BrainUART>;
	UART1_IRQ => uart::InterruptHandler<LidarUART>;
	DMA_IRQ_0 =>
		dma::InterruptHandler<BrainUARTTxDMA>,
		dma::InterruptHandler<BrainUARTRxDMA>,
		dma::InterruptHandler<LedsDMA>,
		dma::InterruptHandler<LidarUARTTxDMA>,
		dma::InterruptHandler<LidarUARTRxDMA>;
	I2C0_IRQ => i2c::InterruptHandler<OtosI2C>;
	PIO0_IRQ_0 => pio::InterruptHandler<BlinkerPIO>;
	#[cfg(feature = "usb")]
	USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

pub struct CoproPeripherals<'a> {
	/// USB for serial output (logging)
	#[cfg(feature = "usb")]
	pub usb: usb::Driver<'a, USB>,
	/// Watchdog timer to detect and reset on stalls
	pub watchdog: watchdog::Watchdog,
	/// Bottom LED
	pub led2: Output<'a>,
	/// SW2 button
	pub button: Input<'a>,
	/// OTOS over I2C
	pub otos: I2c<'a, OtosI2C, i2c::Async>,
	/// RS-485 tranciever UART
	pub brain_uart: Uart<'a, uart::Async>,
	/// RS-485 enable pin, High = Transmit, Low = Recieve
	pub brain_enable_pin: Output<'a>,
	/// PIO state machine for blinking an LED
	pub blinker_sm: StateMachine<'a, BlinkerPIO, BLINKER_STATE_MACHINE>,
	/// PIO state machine for controlling WS2812b RGB LEDs
	pub leds_sm: StateMachine<'a, LedsPIO, LEDS_STATE_MACHINE>,
	pub leds_dma: dma::Channel<'a>,
	/// Lidar sensor UART
	pub lidar_uart: Uart<'a, uart::Async>,
}

pub fn setup_peripherals(p: Peripherals) -> CoproPeripherals<'static> {
	// Configure PIO state machines
	let mut pio0 = Pio::new(p.PIO0, Irq);
	let mut blinker_sm = pio0.sm0;
	let mut leds_sm = pio0.sm1;
	setup_blinker_sm::<BlinkerPIO, BLINKER_STATE_MACHINE, BlinkerPin>(
		&mut pio0.common,
		&mut blinker_sm,
		p.PIN_2,
	);
	setup_ws2812b_sm::<LedsPIO, LEDS_STATE_MACHINE, LedsPin>(
		&mut pio0.common,
		&mut leds_sm,
		p.PIN_6,
		800_000,
	); // 800Kbps

	// Configure the rest
	CoproPeripherals {
		#[cfg(feature = "usb")]
		usb: usb::Driver::new(p.USB, Irq),
		watchdog: Watchdog::new(p.WATCHDOG),
		led2: Output::new(p.PIN_3, Level::Low),
		button: Input::new(p.PIN_12, Pull::Up),
		// secondary_bootsel: Input::new(p.PIN_17, Pull::Up),
		otos: I2c::new_async(p.I2C0, p.PIN_9, p.PIN_8, Irq, {
			// Use 1MBit/s speed for otos i2c
			let mut cfg = i2c::Config::default();
			cfg.frequency = 1_000_000;
			cfg
		}),
		brain_enable_pin: Output::new(p.PIN_14, Level::Low),
		brain_uart: Uart::new(p.UART0, p.PIN_0, p.PIN_1, Irq, p.DMA_CH0, p.DMA_CH1, {
			// 921600 8n1 UART for brain communication
			let mut cfg = uart::Config::default();
			cfg.baudrate = 921600;
			cfg.data_bits = DataBits::DataBits8;
			cfg.stop_bits = StopBits::STOP1;
			cfg.parity = Parity::ParityNone;
			cfg
		}),
		lidar_uart: Uart::new(p.UART1, p.PIN_4, p.PIN_5, Irq, p.DMA_CH3, p.DMA_CH4, {
			// 921600 8n1 UART for brain communication
			let mut cfg = uart::Config::default();
			cfg.baudrate = 460800;
			cfg.data_bits = DataBits::DataBits8;
			cfg.stop_bits = StopBits::STOP1;
			cfg.parity = Parity::ParityNone;
			cfg
		}),
		blinker_sm,
		leds_sm,
		leds_dma: dma::Channel::new(p.DMA_CH2, Irq),
	}
}

use embassy_futures::select::Either;
use embassy_rp::{
	gpio::{Level, Output},
	uart::{self, Async, Uart},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use static_cell::StaticCell;

#[macro_export]
macro_rules! i2c_regs {
	($addr:literal; $(
		// $(#[$attr:meta])*
		$name:ident: $reg_addr:expr
	),+$(,)?) => {
		#[allow(unused)]
		pub const ADDR: u8 = $addr;

		$(
			#[allow(unused)]
			// $(#[$attr])*
			pub const $name: u8 = $reg_addr;
		)+

		#[allow(unused)]
		pub const fn reg_name(reg_addr: u8) -> Option<&'static str> {
			match reg_addr {
				$(
					$name => Some(stringify!($name)),
				)+
				_ => None
			}
		}
	};
}

type InnerType = Mutex<CriticalSectionRawMutex, (Uart<'static, Async>, Output<'static>)>;
#[derive(Copy, Clone)]
pub struct SharedBrainUart {
	inner: &'static InnerType,
}

#[allow(dead_code)]
impl SharedBrainUart {
	const EN_PIN_RECEIVE: Level = Level::Low;
	const EN_PIN_SEND: Level = Level::High;

	pub fn new(uart: Uart<'static, Async>, en_pin: Output<'static>) -> Self {
		Self {
			inner: {
				static DATA: StaticCell<InnerType> = StaticCell::new();
				DATA.init(Mutex::new((uart, en_pin)))
			},
		}
	}

	pub async fn send(&self, buf: &[u8]) -> Result<(), uart::Error> {
		let mut lock = self.inner.lock().await;
		lock.1.set_level(Self::EN_PIN_SEND);
		let res = lock.0.write(buf).await;
		while lock.0.busy() {
			// Busy-loop until its done transmitting, to ensure the enable pin timings are good
		}
		lock.1.set_level(Self::EN_PIN_RECEIVE);
		res
	}

	pub async fn read(&self, buf: &mut [u8]) -> Result<(), uart::Error> {
		let mut lock = self.inner.lock().await;
		lock.0.read(buf).await
	}

	/// Reads a COBS packet from the brain, and decodes it in-place
	pub async fn read_packet(
		&self,
		buf: &mut [u8],
	) -> Result<usize, Either<uart::Error, cobs::DecodeError>> {
		let mut lock = self.inner.lock().await;
		let mut len = 0;

		loop {
			match lock.0.read(&mut buf[len..(len + 1)]).await {
				Ok(()) if buf[len] == 0x00 => {
					return match cobs::decode_in_place(buf) {
						Ok(len) => Ok(len),
						Err(e) => Err(Either::Second(e)),
					};
				}
				Ok(()) => len += 1,
				Err(e) => return Err(Either::First(e)),
			};
		}
	}
}

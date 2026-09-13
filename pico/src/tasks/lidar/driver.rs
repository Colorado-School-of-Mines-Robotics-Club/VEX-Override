use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::{
	Peri,
	dma::{ChannelInstance, InterruptHandler},
	interrupt::typelevel::Binding,
	uart::{
		self, Async, BufferedInterruptHandler, BufferedUartRx, DataBits, Parity, RxPin, StopBits,
		TxPin, UartTx,
	},
};
use embassy_sync::{
	blocking_mutex::raw::{NoopRawMutex, RawMutex},
	zerocopy_channel::{Channel, Receiver, Sender},
};
use embedded_rpc::RpcService;
use rplidar::protocol::{ResponseType, scan::ScanResponse};
use static_cell::StaticCell;

const MEASUREMENT_COUNT: usize = 520; // Approximately the highest amount before a new scan

type M = NoopRawMutex;
pub enum LidarResponse<'a> {
	Scan(Receiver<'a, M, ScanResponse>),
	GetInfo,
	GetHealth,
	GetSampleRate,
	GetLidarConf,
}

pub struct LidarDriver<'a> {
	uart_tx: UartTx<'a, Async>,
	requests: RpcService<M, ResponseType, LidarResponse<'a>>,
	scan_reciever: Receiver<'static, M, ScanResponse>,
}

impl<'a> LidarDriver<'a> {
	/// Create a new LidarDriver. Can only be called once due to statics
	pub fn new<Uart: uart::Instance, TxDma: ChannelInstance>(
		uart: Peri<'a, Uart>,
		tx: Peri<'a, impl TxPin<Uart>>,
		tx_dma: Peri<'a, TxDma>,
		rx: Peri<'a, impl RxPin<Uart>>,
		rx_buffer: &'a mut [u8],
		irq: impl Binding<Uart::Interrupt, BufferedInterruptHandler<Uart>>
		+ Binding<TxDma::Interrupt, InterruptHandler<TxDma>>
		+ 'a,
		spawner: &Spawner,
	) -> Self {
		let uart_cfg = {
			// 460800 8n1 UART for brain communication
			let mut cfg = uart::Config::default();
			cfg.baudrate = 460800;
			cfg.data_bits = DataBits::DataBits8;
			cfg.stop_bits = StopBits::STOP1;
			cfg.parity = Parity::ParityNone;
			cfg
		};

		let tx_uart = uart;
		// SAFETY: Each uart is only going to a TX or an RX, and gets permanantly moved into the TX/RX handlers
		let rx_uart = unsafe { tx_uart.clone_unchecked() };

		let (scan_sender, scan_reciever) = {
			static BUF: StaticCell<[ScanResponse; MEASUREMENT_COUNT]> = StaticCell::new();
			let buf = BUF.init([ScanResponse::default(); _]);
			static CHANNEL: StaticCell<Channel<'_, M, ScanResponse>> = StaticCell::new();
			CHANNEL.init(Channel::new(buf)).split()
		};

		spawner.spawn(unwrap!(driver_task(
			scan_sender,
			BufferedUartRx::new(rx_uart, irq, rx, rx_buffer, uart_cfg)
		)));

		let rpc = RpcService::new();
		rpc.

		Self {
			uart_tx: UartTx::new(tx_uart, tx, tx_dma, irq, uart_cfg),
			requests: rpc,
			scan_reciever,
		}
	}
}

#[embassy_executor::task]
async fn driver_task(sender: Sender<'static, M, ScanResponse>, uart: BufferedUartRx) {}

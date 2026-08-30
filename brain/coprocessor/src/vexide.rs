use std::{
	io::Write as _,
	rc::Rc,
	sync::mpsc::{Receiver, Sender},
	time::Duration,
};

use cobs::{CobsDecoderOwned, CobsEncoder};
use vexide::{prelude::SerialPort, smart::SmartPort, task::Task, time::sleep};

use crate::{CoproRequest, OtosAngle, OtosLength, OtosPose, OtosScalars};

#[derive(Clone)]
pub struct CoprocessorSmartPort {
	requests: Sender<CoproRequest>,
	_task: Rc<Task<()>>,
}

impl CoprocessorSmartPort {
	pub async fn new(port: SmartPort) -> Self {
		let (req_send, req_recv) = std::sync::mpsc::channel();
		Self {
			_task: Rc::new(vexide::task::spawn(Self::task(
				SerialPort::open(port, 921600).await,
				req_recv,
			))),
			requests: req_send,
		}
	}

	pub fn calibrate_otos(&self) {
		_ = self.requests.send(CoproRequest::CalibrateOTOS);
	}

	pub fn set_otos_config(&self, offset: OtosPose, scalars: OtosScalars) {
		_ = self
			.requests
			.send(CoproRequest::ConfigureOTOS { offset, scalars });
	}

	pub fn set_otos_position(&self, position: OtosPose) {
		_ = self.requests.send(CoproRequest::SetOTOSPosition(position));
	}

	async fn task(mut port: SerialPort, requests: Receiver<CoproRequest>) {
		let mut decoder = CobsDecoderOwned::new(1024);
		loop {
			if let Some(byte) = port.read_byte() {
				match decoder.feed(byte) {
					Ok(None) => continue, // Need more data
					Err(_) => {
						// Reset and try again on decode error
						decoder.reset();
						continue;
					}
					Ok(Some(len)) => {
						let decoded = &decoder.dest()[..len];
						match decoded[0] {
							b'l' => (),
							b'o' => (),
							b'r' => {
								// Reuse decoder buffer for encoding, since we don't need to read the decoded message anymore
								let mut encoder = CobsEncoder::new(decoder.dest_mut());
								for req in requests.try_iter() {
									match req {
										CoproRequest::CalibrateOTOS => {
											_ = encoder.push(b"c");
										}
										CoproRequest::SetOTOSPosition(otos_pose) => {
											_ = encoder.push(b"p");
											_ = encoder.push(
												&(otos_pose.x.to::<OtosLength>() as i16)
													.to_le_bytes(),
											);
											_ = encoder.push(
												&(otos_pose.y.to::<OtosLength>() as i16)
													.to_le_bytes(),
											);
											_ = encoder.push(
												&(otos_pose.heading.to::<OtosAngle>() as i16)
													.to_le_bytes(),
											);
										}
										CoproRequest::ConfigureOTOS { offset, scalars } => {
											_ = encoder.push(b"s");
											_ = encoder.push(
												&(offset.x.to::<OtosLength>() as i16).to_le_bytes(),
											);
											_ = encoder.push(
												&(offset.y.to::<OtosLength>() as i16).to_le_bytes(),
											);
											_ = encoder.push(
												&(offset.heading.to::<OtosAngle>() as i16)
													.to_le_bytes(),
											);
											_ = encoder.push(&scalars.linear.to_le_bytes());
											_ = encoder.push(&scalars.angular.to_le_bytes());
										}
									}
								}
								let len = encoder.finalize();
								decoder.dest_mut()[len] = 0x00;
								_ = port.write_all(&decoder.dest()[..(len + 1)]);
							}
							_ => unreachable!(),
						}
					}
				}
			} else {
				sleep(Duration::from_millis(1)).await;
			}
		}
	}
}

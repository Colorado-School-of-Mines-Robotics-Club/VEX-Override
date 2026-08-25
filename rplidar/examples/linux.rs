#![feature(read_array)]
use std::{
	cell::RefCell,
	collections::VecDeque,
	io::Read as _,
	sync::{Arc, LazyLock, Mutex},
	thread::sleep,
	time::Duration,
};

use bitter::LittleEndianReader;
use nannou::prelude::*;
use rplidar::protocol::{
	Request, Response, ResponseDescriptor, ResponseMode, ResponseType,
	get_info::{GetInfoRequest, GetInfoResponse},
	scan::{ScanRequest, ScanResponse},
	stop::StopRequest,
};

struct Settings {
	zoom: f32,
	rotation: f32,
}

struct Model {
	settings: Settings,
	window: Entity,
}

static POINTS: LazyLock<Arc<Mutex<VecDeque<ScanResponse>>>> =
	LazyLock::new(|| Arc::new(Mutex::new(VecDeque::with_capacity(500))));

fn model(app: &App) -> Model {
	let window = app.new_window().primary().view(view).build();

	Model {
		window,
		settings: Settings {
			zoom: 1.0,
			rotation: 0.0,
		},
	}
}

fn view(app: &App, model: &Model) {
	let settings = &model.settings;

	let win = app.window_rect();
	let draw = app.draw();
	draw.background().color(MIDNIGHT_BLUE);

	loop {
		if let Ok(points) = POINTS.try_lock()
			&& points.len() > 10
		{
			let mm_to_px = win.wh().min_element() / 24_000.0 * settings.zoom;
			draw.ellipse()
				.resolution(50.0)
				.color(BLUE)
				.radius(10.0)
				.xy(Vec2::ZERO);

			draw.line()
				.start(Vec2::ZERO)
				.end(pt2(
					(-settings.rotation.to_radians()).sin() * 12_000.0 * mm_to_px,
					(-settings.rotation.to_radians()).cos() * 12_000.0 * mm_to_px,
				))
				.stroke_weight(2.0)
				.color(ORANGE);

			for i in 1..=20 {
				draw.ellipse()
					.resolution(50.0)
					.stroke_color(GRAY)
					.radius(i as f32 * 609.6 * mm_to_px)
					.no_fill()
					.stroke_weight(1.0)
					.xy(Vec2::ZERO);
			}

			for point in points.iter().filter(|p| !p.distance.is_nan()) {
				draw.ellipse()
					.resolution(50.0)
					.color(Srgba {
						red: 1.0 - (point.quality as f32 / 58.0),
						green: point.quality as f32 / 58.0,
						blue: 0.0,
						alpha: 1.0,
					})
					.radius(5.0)
					.x(((point.angle - settings.rotation).to_radians()).sin()
						* point.distance * mm_to_px)
					.y(((point.angle - settings.rotation).to_radians()).cos()
						* point.distance * mm_to_px);
			}

			draw.text(
				"Up/Down = Zoom in/out\nLeft/Right = Rotate\nLines every 2 ft (approx. 1 field tile)",
			)
			.left_justify()
			.align_text_top()
			.xy(Vec2::ZERO)
			.font_size(25);

			break;
		}
		std::hint::spin_loop();
	}
}

fn update(app: &App, model: &mut Model) {
	let settings = &mut model.settings;

	for pressed in app.keys().get_just_pressed() {
		match pressed {
			KeyCode::ArrowUp => settings.zoom = (settings.zoom + 1.0).max(1.0),
			KeyCode::ArrowDown => settings.zoom = (settings.zoom - 1.0).max(1.0),
			KeyCode::ArrowLeft => settings.rotation -= 5.0,
			KeyCode::ArrowRight => settings.rotation += 5.0,
			_ => (),
		}
	}
}

fn main() {
	let mut port = serialport::new("/dev/ttyUSB0", 460_800)
		.timeout(Duration::from_millis(1000))
		.open()
		.expect("Serial port failed to open");

	let mut buf = [0u8; GetInfoRequest::MAX_LENGTH];
	let length = GetInfoRequest.serialize(&mut buf);
	port.write_all(&buf[..length]).expect("Unable to send data");

	// Read response descriptor
	let rd = ResponseDescriptor::read_from::<{ ResponseDescriptor::SIZE }>(&mut port)
		.expect("Unable to read response descriptor");

	// Quick sanity checks
	assert_eq!(rd.length, 20);
	assert_eq!(rd.response_type, ResponseType::GetInfo);
	assert_eq!(rd.mode, ResponseMode::SingleResponse);

	let lidar_info =
		GetInfoResponse::read_from::<20>(&mut port).expect("Unable to read response data");

	println!("Lidar info:\n{}", lidar_info);

	// Send start
	let mut buf = [0u8; ScanRequest::MAX_LENGTH];
	let length = ScanRequest.serialize(&mut buf);
	port.write_all(&buf[..length]).expect("Unable to send data");

	// Read response descriptor
	let rd = ResponseDescriptor::read_from::<{ ResponseDescriptor::SIZE }>(&mut port)
		.expect("Unable to read response descriptor");

	// Quick sanity checks
	assert_eq!(rd.length, 5);
	assert_eq!(rd.response_type, ResponseType::Scan);
	assert_eq!(rd.mode, ResponseMode::MultipleResponse);

	std::thread::spawn(move || {
		// Read measurement
		loop {
			if let Some(m) = ScanResponse::read_from::<5>(&mut port) {
				loop {
					if let Ok(mut p) = POINTS.try_lock() {
						if p.len() == 500 {
							p.pop_front();
						}
						p.push_back(m);
						break;
					}
				}
			}
		}
	});

	// Start gui
	nannou::app(model).update(update).run();
}

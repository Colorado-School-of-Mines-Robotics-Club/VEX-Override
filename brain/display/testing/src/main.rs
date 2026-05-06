use std::time::Instant;

use autons::route;
use buoyant::{
	app::{App, Harness},
	event::Event,
	primitives::Size,
	render_target::{EmbeddedGraphicsRenderTarget, RenderTarget},
};
use display::{State, top_level_view};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_touch::{Phase, Tool, Touch, TouchPoint};
use shrewnit::{Degrees, Inches};
use vexide::{
	display::{RenderMode, TouchEvent, TouchState},
	prelude::{Display, Peripherals},
	time::sleep,
};
use vexide_embedded_graphics::DisplayDriver;

struct TouchStateHandler {
	last: TouchEvent,
	touch_id: u8,
}

impl TouchStateHandler {
	fn new(event: TouchEvent) -> Self {
		Self {
			last: event,
			touch_id: 0,
		}
	}

	fn update(&mut self, event: TouchEvent) -> Option<Event> {
		let buoyant_event = match event.state {
			TouchState::Pressed if event.press_count > self.last.press_count => {
				Some(Event::Touch(Touch {
					id: self.touch_id,
					location: TouchPoint::new(event.point.x, event.point.y),
					phase: Phase::Started,
					tool: Tool::Finger,
				}))
			}
			TouchState::Held if event.point != self.last.point => Some(Event::Touch(Touch {
				id: self.touch_id,
				location: TouchPoint::new(event.point.x, event.point.y),
				phase: Phase::Moved,
				tool: Tool::Finger,
			})),
			TouchState::Released if event.release_count > self.last.release_count => {
				let touch_id = self.touch_id;
				self.touch_id = touch_id.wrapping_add(1);
				Some(Event::Touch(Touch {
					id: touch_id,
					location: TouchPoint::new(event.point.x, event.point.y),
					phase: Phase::Ended,
					tool: Tool::Finger,
				}))
			}
			_ => None,
		};

		self.last = event;

		buoyant_event
	}
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
	let mut display_driver = DisplayDriver::new(peripherals.display);
	let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display_driver, Rgb888::BLACK);
	target
		.display_mut()
		.set_render_mode(RenderMode::DoubleBuffered);

	let mut state = State::<(), 5>::default();
	state.odometry.x = -65.0 * Inches;
	state.odometry.y = 0.0 * Inches;
	state.odometry.h = 90.0 * Degrees;

	async fn callback(_: &mut ()) {
		println!("hiiii");
	}
	state.autons.routes = Some([
		route!("Route 1", callback),
		route!("Route 2", callback),
		route!("Route 3", callback),
		route!("Route 4", callback),
		route!("Route 5", callback),
	]);

	let app_start = Instant::now();
	let mut touch_state_handler = TouchStateHandler::new(target.display().touch_status());
	let mut app = App::new(
		state,
		Size::new(
			Display::HORIZONTAL_RESOLUTION as u32,
			Display::VERTICAL_RESOLUTION as u32,
		),
		top_level_view,
	);

	loop {
		app.set_time(app_start.elapsed());

		if let Some(e) = touch_state_handler.update(target.display().touch_status()) {
			app.send(e);
		}

		// Only render if active animation was reported or redraw is needed
		if app.should_redraw() || target.clear_animation_status() {
			// Render animated transition between source and target trees
			app.render_animated(&mut target, &Rgb888::WHITE);

			// Send to the display
			target.display_mut().render();
			target.clear(Rgb888::BLACK);
		} else {
			sleep(Display::REFRESH_INTERVAL).await;
		}
	}
}

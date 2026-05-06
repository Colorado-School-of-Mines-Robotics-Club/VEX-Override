use std::time::{Duration, Instant};

use autons::route;
use buoyant::{
	app::{App, Harness as _},
	event::simulator::MouseTracker,
	primitives::Size as BuoyantSize,
	render_target::{EmbeddedGraphicsRenderTarget, RenderTarget as _},
	view::prelude::*,
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
	BinaryColorTheme, OutputSettings, SimulatorDisplay, Window, sdl2::Keycode,
};

use display::{BACKGROUND_COLOR, DEFAULT_COLOR, State, top_level_view};

fn main() {
	let mut window = Window::new(
		"Hello World",
		&OutputSettings {
			scale: 1,
			pixel_spacing: 0,
			theme: BinaryColorTheme::Default,
		},
	);
	let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(480, 240));
	display.clear(BACKGROUND_COLOR);
	let mut target = EmbeddedGraphicsRenderTarget::new(&mut display);
	window.update(target.display());

	let mut state = State::<(), 5>::default();
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
	let mut mouse_tracker = MouseTracker::new();
	let mut app = App::new(state, BuoyantSize::new(480, 240), top_level_view);

	loop {
		app.set_time(app_start.elapsed());

		window
			.events()
			.filter_map(|event| {
				match event {
					embedded_graphics_simulator::SimulatorEvent::Quit
					| embedded_graphics_simulator::SimulatorEvent::KeyDown {
						keycode: Keycode::Q | Keycode::Escape,
						..
					} => std::process::exit(0),
					_ => (),
				}
				mouse_tracker.process_event(event)
			})
			.for_each(|event| {
				app.send(event);
			});

		// Only render if active animation was reported or redraw is needed
		if app.should_redraw() || target.clear_animation_status() {
			// Render animated transition between source and target trees
			app.render_animated(&mut target, &Rgb888::WHITE);

			// Send to the display
			window.update(target.display());
			// Clear for the next frame
			target.clear(Rgb888::BLACK);
		} else {
			std::thread::sleep(Duration::from_micros(16667));
		}
	}
}

use std::{
	cell::{Ref, RefCell, RefMut},
	ops::{Deref, DerefMut},
	rc::Rc,
	time::Instant,
};

use buoyant::{
	app::{App, Harness as _},
	event::Event,
	primitives::Size,
	render_target::{EmbeddedGraphicsRenderTarget, RenderTarget as _},
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::RgbColor as _};
use embedded_touch::{Phase, Tool, Touch, TouchPoint};
use vexide::{
	display::{RenderMode, TouchEvent, TouchState},
	prelude::Display,
	task::Task,
	time::sleep_until,
};
use vexide_embedded_graphics::DisplayDriver;

use crate::{State, top_level_view};

struct TouchStateHandler {
	touch: Option<Touch>,
}

impl TouchStateHandler {
	fn new() -> Self {
		Self { touch: None }
	}

	fn update(&mut self, event: TouchEvent) -> Option<Event> {
		let point = TouchPoint::new(event.point.x, event.point.y);
		match event.state {
			TouchState::Pressed /* if self.touch.is_none() */ => {
				let touch = Touch {
					id: 0,
					location: TouchPoint::new(event.point.x, event.point.y),
					phase: Phase::Started,
					tool: Tool::Finger,
				};
				self.touch = Some(touch.clone());

				Some(Event::Touch(touch))
			}
			TouchState::Held => match &mut self.touch {
				Some(touch) if touch.location != point => {
					touch.location = point;
					touch.phase = Phase::Moved;

					Some(Event::Touch(touch.clone()))
				}
				_ => None,
			},
			TouchState::Released if self.touch.is_some() => {
				self.touch = None;
				Some(Event::Touch(Touch {
					id: 0,
					location: point,
					phase: Phase::Ended,
					tool: Tool::Finger,
				}))
			}
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct TrackedState<R: 'static, const N: usize> {
	inner: State<R, N>,
	has_changed: bool,
}

impl<R, const N: usize> Deref for TrackedState<R, N> {
	type Target = State<R, N>;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<R, const N: usize> DerefMut for TrackedState<R, N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.has_changed = true;
		&mut self.inner
	}
}

impl<R, const N: usize> TrackedState<R, N> {
	fn update(&mut self, new: &State<R, N>) {
		self.inner = new.clone();
	}
}

#[derive(Clone)]
pub struct RobotUi<R: 'static, const N: usize> {
	state: Rc<RefCell<TrackedState<R, N>>>,
	_task: Rc<Task<()>>,
}

impl<R: 'static, const N: usize> RobotUi<R, N> {
	pub fn new(display: Display) -> Self {
		let initial_state = State::<R, N>::default();
		let state = Rc::new(RefCell::new(TrackedState {
			inner: initial_state,
			has_changed: false,
		}));

		Self {
			_task: Rc::new(vexide::task::spawn(Self::task(display, state.clone()))),
			state,
		}
	}

	pub fn state_clone(&self) -> Rc<RefCell<TrackedState<R, N>>> {
		self.state.clone()
	}

	pub fn state(&mut self) -> Ref<'_, TrackedState<R, N>> {
		self.state.borrow()
	}

	pub fn state_mut(&mut self) -> RefMut<'_, TrackedState<R, N>> {
		self.state.borrow_mut()
	}

	pub async fn task(display: Display, state: Rc<RefCell<TrackedState<R, N>>>) {
		let mut touch_state_handler = TouchStateHandler::new();
		let mut driver = DisplayDriver::new(display);
		driver.set_render_mode(RenderMode::DoubleBuffered);

		let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut driver, Rgb888::BLACK);

		let mut app = App::new(
			state.borrow().inner.clone(),
			Size::new(
				Display::HORIZONTAL_RESOLUTION as _,
				Display::VERTICAL_RESOLUTION as _,
			),
			top_level_view,
		);

		let app_start = Instant::now();
		loop {
			let render_start = Instant::now();

			// Update time for animations
			app.set_time(render_start.duration_since(app_start));

			// Update touch state
			if let Some(e) = touch_state_handler.update(target.display().touch_status()) {
				app.send(e);
			}

			// Update app state if necessary
			if let mut state = state.borrow_mut()
				&& state.has_changed
			{
				*app.state_mut() = state.inner.clone();
				state.has_changed = false;
			}

			// Rerender screen if necessary
			if app.should_redraw() || target.clear_animation_status() {
				// Render animated transition between source and target trees
				app.render_animated(&mut target, &Rgb888::WHITE);

				// Send to the display
				target.display_mut().render();
				target.clear(Rgb888::BLACK); // Clear framebuffer in advance

				// Ensure states are kept in sync
				// TODO: avoid this needless copying, make inner state type a refcell anyways
				state.borrow_mut().update(app.state());
			} else {
				sleep_until(render_start + Display::REFRESH_INTERVAL * 2).await;
			}
		}
	}
}

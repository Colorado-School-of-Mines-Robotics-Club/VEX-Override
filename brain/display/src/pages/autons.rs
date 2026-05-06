use buoyant::view::{Button, ForEach, Text, View, ZStack, prelude::ViewModifier, shape::Rectangle};
use embedded_graphics::{
	pixelcolor::Rgb888,
	prelude::{RgbColor, WebColors},
};

use crate::{ROBOTO, State};

fn calculate_cols(autons: usize) -> usize {
	(autons as f64 / 2.0).sqrt().ceil() as usize
}

const fn enumerate_array<const N: usize>() -> [usize; N] {
	let mut arr = [0; N];
	let mut i = 0;
	while i < N {
		arr[i] = i;
		i += 1;
	}
	arr
}

// Using the ceil(sqrt(N / 2)) method, the max rows and columns we allow
const MAX_AUTONS: usize = 18;
const MAX_COLS: usize = 3; // sqrt non-const, hardcode it
const MAX_ROWS: usize = MAX_AUTONS / MAX_COLS;

// Hacky? Yes. Does it work? Yes.
const MAX_COLS_ITER: [usize; MAX_COLS] = enumerate_array();
const MAX_ROWS_ITER: [usize; MAX_ROWS] = enumerate_array();

pub fn view<R, const N: usize>(state: &State<R, N>) -> impl View<Rgb888, State<R, N>> + use<R, N> {
	let selected = state.autons.selection;
	state
		.autons
		.routes
		.as_ref()
		.map(|routes| {
			let columns = calculate_cols(N);
			let rows = N.div_ceil(columns);
			let names = std::array::from_fn::<_, N, _>(|i| routes[i].name); // Copy names to avoid lifetime issues
			ForEach::<MAX_ROWS>::new_vertical(&MAX_ROWS_ITER[0..rows], move |row| {
				ForEach::<MAX_COLS>::new_horizontal(&MAX_COLS_ITER[0..columns], move |col| {
					let i = row * columns + col;
					let bg_color = if selected == i {
						Rgb888::CSS_GRAY
					} else {
						Rgb888::BLACK
					};
					Button::new(
						move |state: &mut State<R, N>| {
							if i < N {
								state.autons.selection = i
							}
						},
						move |_| {
							ZStack::new((
								Rectangle.foreground_color(bg_color),
								names.get(i).map(|name| {
									Text::new(*name, &*ROBOTO)
										.with_font_size(20)
										.hint_background_color(bg_color)
								}),
							))
						},
					)
				})
				.with_spacing(2)
			})
			.with_spacing(2)
		})
		.background_color(Rgb888::BLUE, Rectangle)
}

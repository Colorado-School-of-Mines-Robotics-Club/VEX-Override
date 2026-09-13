use buoyant::view::{Text, View, padding::Padding, prelude::*};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{ROBOTO, State};

pub fn view<R, const N: usize>(state: &State<R, N>) -> impl View<Rgb888, State<R, N>> + use<R, N> {
	Padding::new(
		Edges::All,
		10,
		ZStack::new((
			RoundedRectangle::new(15).foreground_color(Rgb888::CSS_DIM_GRAY),
			Text::new_fmt::<100>(
				format_args!(
					"Angle: {:.02}deg\nDistance: {:.02}mm\nQuality: {}",
					state.lidar.measurement.0, state.lidar.measurement.1, state.lidar.measurement.2
				),
				&*ROBOTO,
			)
			.with_font_size(24)
			.hint_background_color(Rgb888::CSS_DIM_GRAY),
		)),
	)
}

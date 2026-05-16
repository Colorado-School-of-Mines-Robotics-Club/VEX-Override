use std::sync::LazyLock;

use buoyant::{
	layout::HorizontalAlignment,
	view::{
		Button, HStack, Image, Spacer, Text, VStack, View, ZStack,
		padding::Padding,
		prelude::{Edges, ViewModifier},
		shape::{Circle, RoundedRectangle},
	},
};
use embedded_graphics::{image::ImageRaw, pixelcolor::Rgb888, prelude::WebColors};
use shrewnit::{Degrees, DegreesPerSecond, Inches, LinearVelocity, simple_unit};
use zune_jpeg::{JpegDecoder, zune_core::bytestream::ZCursor};

use crate::{ROBOTO, State};

simple_unit!(
	pub InchesPerSecond of dimension LinearVelocity = 39.37007874 per canonical
);

macro_rules! display_measurements {
	($state:expr; $($measurement:ident: $unit:ty => $unit_str:literal),+) => {{
		use ::buoyant::{view::{HStack, VStack, Spacer, Text}, layout::HorizontalAlignment};

		HStack::new((
			VStack::new((
				$(
					Text::new(concat!(stringify!($measurement), ":"), &*ROBOTO)
						.with_font_size(20)
						.hint_background_color(Rgb888::CSS_DIM_GRAY)
				),+
			)).with_alignment(HorizontalAlignment::Leading),
			Spacer::default(),
			HStack::new((
				VStack::new((
					$(
						Text::new_fmt::<10>(
							format_args!("{:.02}", $state.odometry.$measurement.to::<$unit>()),
							&*ROBOTO,
						)
						.with_font_size(20)
						.hint_background_color(Rgb888::CSS_DIM_GRAY)
					),+
				)).with_alignment(HorizontalAlignment::Leading),
				VStack::new((
					$(
						Text::new(
							$unit_str,
							&*ROBOTO,
						)
						.with_font_size(20)
						.hint_background_color(Rgb888::CSS_DIM_GRAY)
					),+
				)).with_alignment(HorizontalAlignment::Leading)
			)).with_spacing(5)
		))
	}};
}

static FIELD_IMAGE_BUF: LazyLock<[u8; 150 * 150 * 3]> = LazyLock::new(|| {
	let mut buffer = [0u8; _];
	JpegDecoder::new(ZCursor::new(include_bytes!(
		"../../static/field-150x150.jpg"
	)))
	.decode_into(&mut buffer)
	.unwrap();
	buffer
});
static FIELD_IMAGE: LazyLock<ImageRaw<Rgb888>> =
	LazyLock::new(|| ImageRaw::new(&*FIELD_IMAGE_BUF, 150));

pub fn view<R, const N: usize>(state: &State<R, N>) -> impl View<Rgb888, State<R, N>> + use<R, N> {
	let calibrating = state.odometry.calibrating;
	Padding::new(
		Edges::All,
		10,
		HStack::new((
			ZStack::new((
				RoundedRectangle::new(15).foreground_color(Rgb888::CSS_DIM_GRAY),
				Padding::new(
					Edges::All,
					10,
					VStack::new((
						display_measurements!(state;
							x: Inches => "in",
							y: Inches => "in",
							h: Degrees => "°",
							vx: InchesPerSecond => "in/s",
							vy: InchesPerSecond => "in/s",
							vh: DegreesPerSecond => "°/s"
						),
						Spacer::default(),
						Button::new(
							|state: &mut State<_, _>| {
								if !state.odometry.calibrating {
									state.odometry.calibrating = true;
									(state.odometry.calibration_callback)();
								}
							},
							move |_| {
								let (color, text) = if calibrating {
									(Rgb888::CSS_BLUE, "Calibrating...")
								} else {
									(Rgb888::CSS_GRAY, "Calibrate")
								};
								ZStack::new((
									RoundedRectangle::new(5)
										.foreground_color(color)
										.flex_infinite_width(HorizontalAlignment::Center)
										.with_max_height(40),
									Text::new(text, &*ROBOTO)
										.with_font_size(24)
										.hint_background_color(color),
								))
							},
						),
					)),
				),
			)),
			ZStack::new((
				RoundedRectangle::new(15).foreground_color(Rgb888::CSS_DIM_GRAY),
				ZStack::new((
					Image::new(&*FIELD_IMAGE),
					// TODO rotate based on heading
					// this probably needs a custom impl for that :(
					Circle.frame_sized(15, 15).offset(
						(state.odometry.x.to::<Inches>() * (150.0 / 144.0)) as i32,
						(state.odometry.y.to::<Inches>() * (150.0 / 144.0)) as i32,
					),
				)),
			)),
		))
		.with_spacing(10),
	)
}

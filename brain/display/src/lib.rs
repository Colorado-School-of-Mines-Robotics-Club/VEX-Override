use std::{ops::Rem, sync::LazyLock};

use autons::simple::Route;
use buoyant::{
	match_view,
	view::{Text, View, padding::Padding, prelude::*},
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use shrewnit::{Angle, AngularVelocity, Length, LinearVelocity};
use strum::{EnumCount, FromRepr, IntoStaticStr, VariantArray};

pub mod pages;

pub const BACKGROUND_COLOR: Rgb888 = Rgb888::BLACK;
pub const DEFAULT_COLOR: Rgb888 = Rgb888::WHITE;

static ROBOTO: LazyLock<rusttype::Font<'static>> = LazyLock::new(|| {
	let bytes = include_bytes!("../static/roboto-latin-400-normal.subset.ttf");
	rusttype::Font::try_from_bytes(bytes).unwrap()
});

#[derive(Copy, Clone, PartialEq, Eq, Default, EnumCount, FromRepr, VariantArray, IntoStaticStr)]
pub enum SelectedPage {
	#[default]
	Autons,
	Odometry,
}

#[derive(Clone, Default)]
pub struct State<R, const N: usize> {
	pub page: SelectedPage,
	pub odometry: OdometryState,
	pub autons: AutonsState<R, N>,
}

#[derive(Clone, Default)]
pub struct OdometryState {
	pub x: Length<f64>,
	pub y: Length<f64>,
	pub h: Angle<f64>,
	pub vx: LinearVelocity<f64>,
	pub vy: LinearVelocity<f64>,
	pub vh: AngularVelocity<f64>,
}

#[derive(Clone, Default)]
pub struct AutonsState<R, const N: usize> {
	pub routes: Option<[Route<R>; N]>,
	pub selection: usize,
}

pub fn page_selector<R, const N: usize>(
	state: &State<R, N>,
) -> impl View<Rgb888, State<R, N>> + use<R, N> {
	let current_page = state.page;
	ForEach::<{ SelectedPage::COUNT }>::new_horizontal(SelectedPage::VARIANTS, move |page| {
		Button::new(
			move |state: &mut State<R, N>| state.page = *page,
			move |_| {
				let color = if *page == current_page {
					Rgb888::CSS_FOREST_GREEN
				} else {
					Rgb888::CSS_DARK_GREEN
				};
				ZStack::new((
					Rectangle.foreground_color(color),
					Padding::new(
						Edges::All,
						5,
						Text::<&str, _>::new(page.into(), &*ROBOTO)
							.with_font_size(20)
							.hint_background_color(color),
					),
				))
			},
		)
	})
	.with_spacing(2)
	.fixed_size(false, true)
}

pub fn top_level_view<R, const N: usize>(
	state: &State<R, N>,
) -> impl View<Rgb888, State<R, N>> + use<R, N> {
	VStack::new((
		page_selector(state),
		match_view!(state.page, {
			SelectedPage::Autons => pages::autons::view(state),
			SelectedPage::Odometry => pages::odometry::view(state)
		}),
	))
}

use std::sync::LazyLock;

use buoyant::{
	match_view,
	view::{Text, View, padding::Padding, prelude::*},
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use strum::{EnumCount as _, VariantArray as _};

use crate::state::{SelectedPage, State};

pub mod pages;
pub mod state;
#[cfg(feature = "vexide")]
pub mod vexide;

pub const BACKGROUND_COLOR: Rgb888 = Rgb888::BLACK;
pub const DEFAULT_COLOR: Rgb888 = Rgb888::WHITE;

static ROBOTO: LazyLock<rusttype::Font<'static>> = LazyLock::new(|| {
	let bytes = include_bytes!("../static/roboto-latin-400-normal.subset.ttf");
	rusttype::Font::try_from_bytes(bytes).unwrap()
});

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

use buoyant::view::{Button, ForEach, Text, View, ZStack, prelude::ViewModifier, shape::Rectangle};
use embedded_graphics::{
	pixelcolor::Rgb888,
	prelude::{RgbColor, WebColors},
};

use crate::{ROBOTO, State};

pub fn view<R, const N: usize>(state: &State<R, N>) -> impl View<Rgb888, State<R, N>> + use<R, N> {}

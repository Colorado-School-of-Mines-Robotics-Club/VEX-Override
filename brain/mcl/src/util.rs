use std::f32::consts::{E, PI};

use crate::config::GC;

pub fn gaussian(x: f32) -> f32 {
    GC.mcl.gaussian_factor * E.powf(-0.5 * (x / GC.mcl.gaussian_stdev).powf(2.0))
        / (GC.mcl.gaussian_stdev * (2.0 * PI).sqrt())
}
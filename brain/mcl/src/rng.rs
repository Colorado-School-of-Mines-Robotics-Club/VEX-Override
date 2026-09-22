use rand::prelude::*;
use rand_pcg::Pcg32;

use crate::config::GC;

#[derive(Debug)]
pub struct RNGSuite {
    rng: Pcg32,
    xy_noise_distr: rand::distr::Uniform<f32>,
    theta_noise_distr: rand::distr::Uniform<f32>,
    particle_offset_distr: rand::distr::Uniform<f32>,
}
impl RNGSuite {
    pub fn new() -> Self {
        RNGSuite {
            rng: Pcg32::from_seed([0x56, 0x86, 0xdd, 0xe5, 0xc2, 0xca, 0x3b, 0x0a, 0x18, 0x2c, 0xae, 0xf4, 0x09, 0x96, 0x5b, 0x71]),
            xy_noise_distr: rand::distr::Uniform::<f32>::new(
                -GC.mcl.xy_noise,
                GC.mcl.xy_noise,
            )
            .unwrap(),
            theta_noise_distr: rand::distr::Uniform::<f32>::new(
                -GC.mcl.theta_noise,
                GC.mcl.theta_noise,
            )
            .unwrap(),
            particle_offset_distr: rand::distr::Uniform::<f32>::new(
                0.0,
                1.0 / GC.mcl.n_particles as f32,
            )
            .unwrap(),
        }
    }
    pub fn xy_noise(&mut self) -> f32 {
        self.rng.sample(self.xy_noise_distr)
    }
    pub fn theta_noise(&mut self) -> f32 {
        self.rng.sample(self.theta_noise_distr)
    }
    pub fn particle_offset(&mut self) -> f32 {
        self.rng.sample(self.particle_offset_distr)
    }
}

impl Default for RNGSuite {
    fn default() -> Self {
        Self::new()
    }
}

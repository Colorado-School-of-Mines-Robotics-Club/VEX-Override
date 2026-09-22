use toml_const::toml_const;

// This file is a bunch of horribleness, but it works well enough,
// and is far from a priority.

toml_const! {
    const CFG: "config.toml";
}

#[allow(nonstandard_style)]
pub struct _GC {
    pub arena_width: f32,
    pub arena_height: f32,
    pub n_beams: i32,
    pub mcl: _GC_MCL,
}

#[allow(nonstandard_style)]
pub struct _GC_MCL {
    pub n_particles: i32,
    pub gaussian_stdev: f32,
    pub gaussian_factor: f32,
    pub xy_noise: f32,
    pub theta_noise: f32,
}

pub const GC: _GC = _GC {
    arena_width: CFG.arena_width as f32,
    arena_height: CFG.arena_height as f32,
    n_beams: CFG.n_beams as i32,
    mcl: _GC_MCL {
        n_particles: CFG.mcl.n_particles as i32,
        gaussian_stdev: CFG.mcl.gaussian_stdev as f32,
        gaussian_factor: CFG.mcl.gaussian_factor as f32,
        xy_noise: CFG.mcl.xy_noise as f32,
        theta_noise: CFG.mcl.theta_noise as f32,
    },
};
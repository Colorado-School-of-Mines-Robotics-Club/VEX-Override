use std::iter::zip;

pub mod config;
pub mod types;
pub mod util;
pub mod rng;

use crate::config::GC;
use crate::rng::RNGSuite;
use crate::types::{Particle, Pose, ParticleArray, Beam};

#[derive(Debug)]
pub struct MCL {
    rng: RNGSuite,
    particles: ParticleArray,
    average_pose: Pose,
}
impl MCL {
    pub fn new(init_pose: Pose) -> MCL {
        MCL {
            rng: RNGSuite::new(),
            particles: [Particle {
                x: init_pose.x,
                y: init_pose.y,
                theta: init_pose.theta,
                weight: 0.0,
            }; _],
            average_pose: Pose {
                x: init_pose.x,
                y: init_pose.y,
                theta: init_pose.theta,
            },
        }
    }

    pub fn get_particle(&self, index: usize) -> Particle {
        self.particles[index]
    }

    // Delta represents the change in robot pose measured from motor odometry (delta)
    pub fn timestep(&mut self, beam: Beam, delta: Pose) -> Pose {
        //
        // 1. Offset all particles using odometry + add noise
        for particle in self.particles.iter_mut() {
            particle.update_delta_noise(&mut self.rng, delta);
        }
        //
        // 2. Calculate each particle's weight
        let mut weight_sum: f32 = 0.0;
        for particle in self.particles.iter_mut() {
            particle.update_weight(beam);
            weight_sum += particle.weight;
        }
        //
        // 3. idk anymore man
        let cumulative_weights: [f32; GC.mcl.n_particles as usize] = std::array::from_fn(|i| {
            self.particles[0..i]
                .iter()
                .fold::<f32, _>(0.0, |acc, particle| acc + particle.weight)
        });

        let random_offset = self.rng.particle_offset();
        let offsets: [f32; GC.mcl.n_particles as usize] = std::array::from_fn(|i| {
            weight_sum * (random_offset + i as f32 / GC.mcl.n_particles as f32)
        });

        let mut new_particles: ParticleArray = std::array::from_fn(|_| Particle::default());

        // black voodoo eldritch magic
        let mut push_index = 0;
        for offset in offsets {
            for (particle, cumulative_weight) in zip(&self.particles, &cumulative_weights) {
                if *cumulative_weight >= offset {
                    new_particles[push_index] = particle.clone();
                    push_index += 1;
                    break;
                }
            }
        }
        self.particles = new_particles;

        self.average_pose = self.particles.iter().fold(
            Pose {
                x: 0.0,
                y: 0.0,
                theta: 0.0,
            },
            |mut pose_acc, particle| {
                pose_acc.x += particle.x;
                pose_acc.y += particle.y;
                pose_acc.theta += particle.theta;
                pose_acc
            },
        );

        self.average_pose.x /= GC.mcl.n_particles as f32;
        self.average_pose.y /= GC.mcl.n_particles as f32;
        self.average_pose.theta /= GC.mcl.n_particles as f32;

        self.average_pose
    }
}
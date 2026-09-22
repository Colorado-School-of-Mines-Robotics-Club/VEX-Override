use crate::config::GC;
use crate::rng::RNGSuite;
use crate::util::gaussian;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
    pub weight: f32,
}
impl Particle {
    // Change the position of each particle by delta (odometry), and add some noise to the particle's pose
    pub fn update_delta_noise(&mut self, rng: &mut RNGSuite, delta: Pose) {
        self.x += rng.xy_noise() + delta.x;
        self.y += rng.xy_noise() + delta.y;
        self.theta += rng.theta_noise() + delta.theta;
    }
    // Assumming the particle is the location and rotation of the robot, get the pose at which the beam would be expected to land
    pub fn expected_pose(&mut self, beam: Beam) -> Pose {
        let global_theta = self.theta + beam.theta;
        Pose {
            x: self.x + beam.distance * global_theta.cos(),
            y: self.y + beam.distance * global_theta.sin(),
            theta: global_theta
        }
    }
    // Assumming the particle is the location and rotation of the robot, calculate the distance from the pose,
    // along the pose's direction (theta), to one of the arena's four walls
    // This implementation is kinda innacurate tbh
    pub fn distance_to_wall(&mut self, pose: Pose) -> f32 {
        [
            (pose.x - GC.arena_width / 2.0) / pose.theta.cos(),
            (pose.x + GC.arena_width / 2.0) / pose.theta.cos(),
            (pose.y - GC.arena_width / 2.0) / pose.theta.sin(),
            (pose.y + GC.arena_width / 2.0) / pose.theta.sin(),
        ]
        .into_iter()
        .reduce(|d1, d2| d1.min(d2))
        .unwrap()
    }

    pub fn update_weight(&mut self, beam: Beam) {
        let exp_pose = self.expected_pose(beam);
        self.weight = gaussian(self.distance_to_wall(exp_pose));
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Beam {
    pub distance: f32,
    pub theta: f32,
}

pub type ParticleArray = [Particle; GC.mcl.n_particles as usize];
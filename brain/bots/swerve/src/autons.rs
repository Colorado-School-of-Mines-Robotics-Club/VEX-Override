use std::time::Duration;
use std::array;

use vexide::time::sleep;

use crate::robot::Robot;

use mcl::{MCL, types::{Pose, Beam}};

pub async fn test_mcl(robot: &mut Robot) {
    println!("MCLing");
    let init_pos = robot.copro.get_latest_otos();
    let mut mcl = MCL::new(Pose { x: init_pos.x.canonical() as f32, y: init_pos.y.canonical() as f32, theta: init_pos.heading.canonical() as f32 });

    let mut prev_pos = init_pos;

    loop {
        let mut measurements = robot.copro.lidar.write().await;
        while let Some(measurement) = measurements.pop_back() {
            let new_pos = robot.copro.get_latest_otos();
            let delta = Pose {
                x: new_pos.x.canonical() as f32 - prev_pos.x.canonical() as f32,
                y: new_pos.y.canonical() as f32 - prev_pos.y.canonical() as f32,
                theta: new_pos.heading.canonical() as f32 - prev_pos.heading.canonical() as f32
            };
            let mcl_est_pose = mcl.timestep(Beam {distance: measurement.distance.canonical() as f32, theta: measurement.angle.canonical() as f32}, delta);
            println!("[MCL] Estimated Position - x: {}, y: {}, theta: {}", mcl_est_pose.x, mcl_est_pose.y, mcl_est_pose.theta);

            prev_pos = new_pos;
        }

        sleep(Duration::from_millis(1)).await;
    };
}
use std::{cell::RefCell, rc::Rc, time::{Duration, Instant}};

use evian::control::loops::{AngularPid, Feedback as _, Pid};
use shrewnit::{LinearVelocity, MetersPerSecond};
use vexide::{adi::analog::AdiAnalogIn, math::Angle, smart::motor::Motor, sync::RwLock, time::sleep};

pub type SwervePod = Rc<RwLock<SwervePodInner>>;

#[derive(Debug)]
pub struct SwervePodInner {
    motor_a: Motor,
    motor_b: Motor,
    rotation: AdiAnalogIn,
    analog_offset: u16,

    linear_pid: Pid,
    turn_pid: AngularPid,

    target_heading: Angle,
    target_speed: LinearVelocity,
}

const LINEAR_PID: Pid = Pid::new(0.02, 200.0, 0.0, Some(11.0));
const TURN_PID: AngularPid = AngularPid::new(5.0, 0.0, 0.2, None);

impl SwervePodInner {
    pub fn new(motor_a: Motor, motor_b: Motor, rotation: AdiAnalogIn, analog_offset: u16) -> Rc<RwLock<Self>> {
        let v = Rc::new(RwLock::new(Self {
            motor_a,
            motor_b,
            rotation,
            analog_offset,

            linear_pid: LINEAR_PID,
            turn_pid: TURN_PID,

            target_heading: Angle::ZERO,
            target_speed: 0.0 * MetersPerSecond
        }));
        
        vexide::task::spawn(Self::task(v.clone())).detach();

        v
    }

    pub fn set_heading(&mut self, heading: Angle) {
        self.target_heading = heading;
    }

    pub fn set_speed(&mut self, speed: LinearVelocity) {
        self.target_speed = speed;
    }

    async fn task(pod: Rc<RwLock<Self>>) {
        // double linearRPM = ((aMotorRPM - bMotorRPM) / 2) * 30/30 ; wheel size 2.75 in
        // double turnRPM = ((aMotorRPM + bMotorRPM) / 2) * 30/60;

        let mut timer = Instant::now();

        loop {
            let mut pod = pod.write().await;

            let angle = pod.rotation.value().expect("Failed to read angle encoder");
            let angle = Angle::from_degrees((angle + pod.analog_offset % 4096) as f64 / 4096.0 * 360.0);
            let target_heading = pod.target_heading;

            let turn = pod.turn_pid.update(angle, target_heading, timer.elapsed());

            let current_speed = (pod.motor_a.velocity().unwrap_or(0.0) - pod.motor_b.velocity().unwrap_or(0.0)) / 2.0;
            
            let target_speed = pod.target_speed.to::<MetersPerSecond>();
            let linear = pod.linear_pid.update(current_speed.abs(), target_speed, timer.elapsed());
            // let linear = 0.0;

            // dbg!(linear, turn);

            _ = pod.motor_a.set_velocity((turn + linear) as i32);
            _ = pod.motor_b.set_velocity((turn - linear) as i32);

            timer = Instant::now();
            std::mem::drop(pod);
            sleep(Duration::from_millis(10)).await;
        }
    }
}

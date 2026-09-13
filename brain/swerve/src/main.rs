use vexide::prelude::*;

#[derive(Debug)]
pub struct Robot {
    // Swerve pod motors
    front_left: (Motor, Motor),
    front_right: (Motor, Motor),
    back_left: (Motor, Motor),
    back_right: (Motor, Motor),

    // DR4B lift motors
    lift_1: Motor,
    lift_2: Motor,

    // Claw raising
    bars: Motor,
    // Spin the bar
    chain: Motor,

    imu: Imu,
    controller: Controller
}

impl Compete for Robot {
    async fn autonomous(&mut self) {
        println!("Autonomous!");
    }

    async fn driver(&mut self) {
        println!("Driver!");

        if let Ok(controller) = self.controller {
            
        }
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let robot = Robot {
        front_left: (
            Motor::new(peripherals.port_2, Gearset::Blue, Direction::Forward),
            Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward)
        ),
        front_right: (
            Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
            Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward)
        ),
        back_left: (
            Motor::new(peripherals.port_6, Gearset::Blue, Direction::Forward),
            Motor::new(peripherals.port_7, Gearset::Blue, Direction::Forward)
        ),
        back_right: (
            Motor::new(peripherals.port_8, Gearset::Blue, Direction::Forward),
            Motor::new(peripherals.port_9, Gearset::Blue, Direction::Forward)
        ),

        lift_1: Motor::new_exp(peripherals.port_10, Direction::Reverse),
        lift_2: Motor::new_exp(peripherals.port_1, Direction::Forward),

        chain: Motor::new_exp(peripherals.port_12, Direction::Forward),
        bars: Motor::new_exp(peripherals.port_13, Direction::Forward),

        controller: peripherals.master_controller,
        imu: Imu::new(peripherals.port_16)
    };

    robot.compete().await;
}

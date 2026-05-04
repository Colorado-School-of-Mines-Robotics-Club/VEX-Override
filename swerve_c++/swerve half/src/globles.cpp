#include "main.h"

pros::Motor a1(1, pros::v5::MotorGears::green);
pros::Motor a2(2, pros::v5::MotorGears::green);
pros::Motor b1(9, pros::v5::MotorGears::green);
pros::Motor b2(10, pros::v5::MotorGears::green);

pros::Controller master(pros::E_CONTROLLER_MASTER);
pros::Controller slave(pros::E_CONTROLLER_PARTNER);

pros::Imu imu(5);

pros::adi::AnalogIn rota('A');
pros::adi::AnalogIn rotb('B');
#include "main.h"

pros::Motor swerve_a1(2, pros::v5::MotorGears::blue);
pros::Motor swerve_a2(3, pros::v5::MotorGears::blue);
pros::Motor swerve_b1(4, pros::v5::MotorGears::blue);
pros::Motor swerve_b2(5, pros::v5::MotorGears::blue);
pros::Motor swerve_c1(6, pros::v5::MotorGears::blue);
pros::Motor swerve_c2(7, pros::v5::MotorGears::blue);
pros::Motor swerve_d1(8, pros::v5::MotorGears::blue);
pros::Motor swerve_d2(9, pros::v5::MotorGears::blue);
pros::Motor DR4B_R(1, pros::v5::MotorGears::green);
pros::Motor DR4B_L(-10, pros::v5::MotorGears::green);
pros::Motor Chain_bar(12, pros::v5::MotorGears::green);
pros::Motor Duo_bar(13, pros::v5::MotorGears::green);
pros::Motor Bottom_intake_a(14, pros::v5::MotorGears::green);
pros::Motor Bottom_intake_b(17, pros::v5::MotorGears::green);
pros::Motor Top_intake(18, pros::v5::MotorGears::blue);

pros::Controller master(pros::E_CONTROLLER_MASTER);
pros::Controller slave(pros::E_CONTROLLER_PARTNER);

pros::Imu imu(16);

pros::adi::AnalogIn rota('E');
pros::adi::AnalogIn rotb('F');
pros::adi::AnalogIn rotc('H');
pros::adi::AnalogIn rotd('G');

pros::adi::DigitalOut phematicks_a('A');
pros::adi::DigitalOut phematicks_b('B');
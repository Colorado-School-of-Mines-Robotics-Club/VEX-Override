#include "api.h"

extern pros::Motor swerve_a1;
extern pros::Motor swerve_a2;
extern pros::Motor swerve_b1;
extern pros::Motor swerve_b2;
extern pros::Motor swerve_c1;
extern pros::Motor swerve_c2;
extern pros::Motor swerve_d1;
extern pros::Motor swerve_d2;
extern pros::Motor DR4B_R;
extern pros::Motor DR4B_L;
extern pros::Motor Chain_bar;
extern pros::Motor Duo_bar;
extern pros::Motor Bottom_intake_a;
extern pros::Motor Bottom_intake_b;
extern pros::Motor Top_intake;

extern pros::adi::DigitalOut phematicks_a;
extern pros::adi::DigitalOut phematicks_b;

extern pros::adi::AnalogIn rota;
extern pros::adi::AnalogIn rotb;
extern pros::adi::AnalogIn rotc;
extern pros::adi::AnalogIn rotd;
extern pros::adi::AnalogIn rotchain;

extern pros::adi::DigitalIn licence_plate;

extern pros::Imu imu;

extern pros::Controller master;
extern pros::Controller slave;

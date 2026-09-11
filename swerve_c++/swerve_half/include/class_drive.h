#include "api.h"

class swerve_module {
private:
    pros::Motor* motora;
    pros::Motor* motorb;
    pros::adi::AnalogIn* rot;
    pros::Imu* imu;
    int offset;
    int id;
    double wanted_angle;
    double wanted_speed;
    double width;
    double length;  
    double turn_perfecktion;
public:
    swerve_module();
    swerve_module(pros::Motor* m1, pros::Motor* m2, pros::adi::AnalogIn* rot_in, pros::Imu* imu_in, int offset_in, int id_in);
    void update(double heading);
    void set_polor(double angle, double power);
};

class Drive{
private:
    pros::Motor* motor1;
    pros::Motor* motor2;
    pros::Motor* motor3;
    pros::Motor* motor4;
    pros::Imu* imu;
    pros::adi::AnalogIn* rota;
    pros::adi::AnalogIn* rotb;
    double b0 = 2320;
    double a0 = 3280;
    std::vector<double> wanted = {0, 0, 0}; // angle x want y want
    swerve_module mod1;
    swerve_module mod2;  
public:
    Drive();
    Drive(pros::Motor& m1, pros::Motor& m2, pros::Motor& m3, pros::Motor& m4, pros::Imu& imu_in, pros::adi::AnalogIn& rota_in, pros::adi::AnalogIn& rotb_in);
    void update();
    void set_veter(std::vector<double> new_wanted);
    void set_double(double angle, double speed, double turn);
};
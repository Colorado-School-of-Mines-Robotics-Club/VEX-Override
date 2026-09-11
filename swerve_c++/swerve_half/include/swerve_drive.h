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

public:
    swerve_module();
    swerve_module(pros::Motor* m1, pros::Motor* m2, pros::adi::AnalogIn* rot_in, pros::Imu* imu_in, int offset_in, int id_in);
    void update(double heading);
    void set_polor(double angle, double power);
};

class Drive{
private:
    pros::Motor* motora1;
    pros::Motor* motora2;
    pros::Motor* motorb1;
    pros::Motor* motorb2;
    pros::Motor* motorc1;
    pros::Motor* motorc2;
    pros::Motor* motord1;
    pros::Motor* motord2;
    pros::Imu* imu;
    pros::adi::AnalogIn* rota;
    pros::adi::AnalogIn* rotb;
    pros::adi::AnalogIn* rotc;
    pros::adi::AnalogIn* rotd;
    double b0 = 2320;
    double a0 = 3280;
    double c0 = 3280;
    double d0 = 2320;
    std::vector<double> wanted = {0, 0, 0}; // angle x want y want
    swerve_module mod1;
    swerve_module mod2;
    swerve_module mod3;
    swerve_module mod4;
public:
    Drive();
    Drive(pros::Motor& m1, pros::Motor& m2, pros::Motor& m3, pros::Motor& m4, pros::Motor& m5, pros::Motor& m6, pros::Motor& m7, pros::Motor& m8, pros::Imu& imu_in, pros::adi::AnalogIn& rota_in, pros::adi::AnalogIn& rotb_in, pros::adi::AnalogIn& rotc_in, pros::adi::AnalogIn& rotd_in);
    void update();
    void set_veter(std::vector<double> new_wanted);
    void set_double(double angle, double speed, double turn);
    void set_absolute(double tx, double ty, double turn);
};
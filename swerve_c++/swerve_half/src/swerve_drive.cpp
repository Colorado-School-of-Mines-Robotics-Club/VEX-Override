//#include "api.h"
#include "swerve_drive.h"
using namespace pros;

swerve_module::swerve_module(){
    motora = nullptr;
    motorb = nullptr;
    rot = nullptr;
    imu = nullptr;
    offset = 0;
    id = 0;
    wanted_angle = 0;
    wanted_speed = 0;
}

swerve_module::swerve_module(pros::Motor* m1, pros::Motor* m2, pros::adi::AnalogIn* rot_in, pros::Imu* imu_in, int offset_in, int id_in){
    motora = m1;
    motorb = m2;
    rot = rot_in;
    imu = imu_in;
    offset = 4096-offset_in;
    id = id_in;
}

void swerve_module::update(double heading) {
    int error = (rot->get_value() + offset) % 4096;
    double error2= error/4096.0*360;
    //double delta_angle = heading-error2;
    double delta_angle = 0-error2;
    if (delta_angle <0){
        delta_angle+=360;
    }
     
    double turn_error = delta_angle-wanted_angle;
    if(turn_error >= 360){
        turn_error -= 360;
    }
    if (turn_error > 180){
        turn_error = -1*(360-turn_error);
    }
    
   


    bool reverse = false;
    double actual_speed = 0;

    if (turn_error > 90 || turn_error < -90){
        turn_error = turn_error-180;
        actual_speed = -wanted_speed;
        if (turn_error < -180){
            turn_error += 360;
        }
    }else{
        actual_speed = wanted_speed;
    }




    motora->move(actual_speed + turn_error);
    motorb->move(-actual_speed + turn_error);

    //if (id == 1 ){printf("actual_speed: %f, turn_error: %f, id: %d\n", actual_speed, turn_error, id);}
}

void swerve_module::set_polor(double angle, double power){
    if (angle <0 ){
        angle += 360;
    }
    if (angle >= 360){
        angle -= 360;
    }
    wanted_angle = angle;
    wanted_speed = power;
}

Drive::Drive(){
    motora1 = nullptr;
    motora2 = nullptr;
    motorc1 = nullptr;
    motorc2 = nullptr;
    motorb1 = nullptr;
    motorb2 = nullptr;
    motord1 = nullptr;
    motord2 = nullptr;
    imu = nullptr;
    rota = nullptr;
    rotb = nullptr;
    mod1=swerve_module();
    mod2=swerve_module();
}

Drive::Drive(pros::v5::Motor& m1, pros::v5::Motor& m2, pros::v5::Motor& m3, pros::v5::Motor& m4, pros::v5::Motor& m5, pros::v5::Motor& m6, pros::v5::Motor& m7, pros::v5::Motor& m8, pros::Imu& imu_in, pros::adi::AnalogIn& rota_in, pros::adi::AnalogIn& rotb_in, pros::adi::AnalogIn& rotc_in, pros::adi::AnalogIn& rotd_in){
        motora1 = &m1;
        motora2 = &m2;
        motorb1 = &m3;
        motorb2 = &m4;
        motorc1 = &m5;
        motorc2 = &m6;
        motord1 = &m7;
        motord2 = &m8;
        imu = &imu_in;
        rota = &rota_in;
        rotb = &rotb_in;
        rotc = &rotc_in;
        rotd = &rotd_in;
        mod1=swerve_module( motora1, motora2, rota, imu, a0, 1);
        mod2=swerve_module(motorb1, motorb2, rotb, imu, b0, 2);
        mod1=swerve_module( motorc1, motorc2, rotc, imu, c0, 3);
        mod2=swerve_module(motord1, motord2, rotd, imu, d0, 4);
    }

    void Drive::update() {
        double heading = imu->get_heading();
        mod1.update(heading);
        mod2.update(heading);
        mod3.update(heading);
        mod4.update(heading);
    }

    void Drive::set_veter(std::vector<double> new_wanted) {
        wanted = new_wanted;
    }

void Drive::set_double(double angle, double speed, double turn) {

    if (speed == -1) {
        speed = 0;
    }

    // Convert heading angle to radians
    double rad = angle * M_PI / 180.0;

    // Translation vector
    double tx = cos(rad) * speed;
    double ty = sin(rad) * speed;

    set_absolute(tx, ty, turn);
    
}

void Drive::set_absolute(double tx, double ty, double turn) {

    // Half the distance between the modules
    double L = 8.05/2;
    double W = 11.5/2;

    double x[4] = {-L, L, -L, L};
    double y[4] = { W, W, -W, -W};

    for (int i = 0; i < 4; i++) {

        // Rotational velocity at this module
        double vx = tx - turn * y[i];
        double vy = ty + turn * x[i];

        // Resulting wheel velocity
        double power = sqrt(vx * vx + vy * vy);

        // Resulting wheel angle
        double module_angle = atan2(vy, vx) * 180.0 / M_PI;

        // Keep angle between 0 and 360
        if (module_angle < 0) {
            module_angle += 360;
        }

        // Send to module
        switch (i) {
            case 0:
                mod1.set_polor(module_angle, power);
                break;

            case 1:
                mod2.set_polor(module_angle, power);
                break;

            case 2:
                mod3.set_polor(module_angle, power);
                break;

            case 3:
                mod4.set_polor(module_angle, power);
                break;
        }
    }
}

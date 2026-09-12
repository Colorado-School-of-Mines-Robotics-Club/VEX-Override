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

Drive::Drive(pros::v5::Motor& top_left_m1, pros::v5::Motor& top_left_m2,
             pros::v5::Motor& top_right_m1, pros::v5::Motor& top_right_m2,
             pros::v5::Motor& bottom_left_m1, pros::v5::Motor& bottom_left_m2,
             pros::v5::Motor& bottom_right_m1, pros::v5::Motor& bottom_right_m2,
             pros::Imu& imu_in,
             pros::adi::AnalogIn& top_left_encoder,
             pros::adi::AnalogIn& top_right_encoder,
             pros::adi::AnalogIn& bottom_left_encoder,
             pros::adi::AnalogIn& bottom_right_encoder)
{
        motora1 = &top_left_m1;
        motora2 = &top_left_m2;
        motorb1 = &top_right_m1;
        motorb2 = &top_right_m2;
        motorc1 = &bottom_left_m1;
        motorc2 = &bottom_left_m2;
        motord1 = &bottom_right_m1;
        motord2 = &bottom_right_m2;
        imu = &imu_in;
        rota = &top_left_encoder;
        rotb = &top_right_encoder;
        rotc = &bottom_left_encoder;
        rotd = &bottom_right_encoder;
        initial_heading = imu_in.get_heading();
        mod1=swerve_module( motora1, motora2, rota, imu, a0, 1);
        mod2=swerve_module(motorb1, motorb2, rotb, imu, b0, 2);
        mod3=swerve_module( motorc1, motorc2, rotc, imu, c0, 3);
        mod4=swerve_module(motord1, motord2, rotd, imu, d0, 4);
        printf("flag c");
    }

void Drive::update() {
    double heading = imu->get_heading();
    mod1.update(heading);
    mod2.update(heading);
    mod3.update(heading);
    mod4.update(heading);
    printf("flag b");
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

void Drive::set_absolute(double ty, double tx, double turn) {

    printf("flag a");

    if (imu != nullptr) {
        double heading_delta = -imu->get_heading() - initial_heading;
        while (heading_delta > 180.0) heading_delta -= 360.0;
        while (heading_delta < -180.0) heading_delta += 360.0;

        double heading_rad = heading_delta * M_PI / 180.0;
        double cos_theta = cos(heading_rad);
        double sin_theta = sin(heading_rad);

        // Rotate the joystick vector back into the original field frame whose X
        // axis is the heading at the moment the Drive object was constructed.
        double tx_world = tx * cos_theta - ty * sin_theta;
        double ty_world = tx * sin_theta + ty * cos_theta;

        tx = tx_world;
        ty = ty_world;
    }

    // Half the distance between the modules
    double L = 8.05/2;
    double W = 11.5/2;

    double x[4] = {-L, L, -L, L};
    double y[4] = { W, W, -W, -W};

    for (int i = 0; i < 4; i++) {
        double turn_flip = (i == 1 || i == 2) ? -1.0 : 1.0;

        // Rotational velocity at this module
        double vx = tx - turn_flip * turn * y[i];
        double vy = ty + turn_flip * turn * x[i];

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
    printf("flag e");
}

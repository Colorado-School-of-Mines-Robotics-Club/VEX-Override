//#include "api.h"
#include "class_drive.h"
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
    width = 12;
    length = 12;
    turn_perfecktion = 0;
}

swerve_module::swerve_module(pros::Motor* m1, pros::Motor* m2, pros::adi::AnalogIn* rot_in, pros::Imu* imu_in, int offset_in, int id_in){
    motora = m1;
    motorb = m2;
    rot = rot_in;
    imu = imu_in;
    offset = 4096-offset_in;
    id = id_in;
    width = 12;
    length = 12;
    turn_perfecktion = tan(length/width)*90/M_PI;
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
    motor1 = nullptr;
    motor2 = nullptr;
    motor3 = nullptr;
    motor4 = nullptr;
    imu = nullptr;
    rota = nullptr;
    rotb = nullptr;
    mod1=swerve_module();
    mod2=swerve_module();
}

Drive::Drive(pros::v5::Motor& m1, pros::v5::Motor& m2, pros::v5::Motor& m3, pros::v5::Motor& m4, pros::Imu& imu_in, pros::adi::AnalogIn& rota_in, pros::adi::AnalogIn& rotb_in){
        motor1 = &m1;
        motor2 = &m2;
        motor3 = &m3;
        motor4 = &m4;
        imu = &imu_in;
        rota = &rota_in;
        rotb = &rotb_in;
        mod1=swerve_module( motor1, motor2, rota, imu, a0, 1);
        mod2=swerve_module(motor3, motor4, rotb, imu, b0, 2);
    }



    // Member function to display car details
    void Drive::update() {
        double heading = imu->get_heading();
        mod1.update(heading);
        mod2.update(heading);
    }

    void Drive::set_veter(std::vector<double> new_wanted) {
        wanted = new_wanted;
    }

    void Drive::set_double(double angle, double speed, double turn) {

        if (speed != -1){
            wanted[0] = angle;
            wanted[1] = speed;
        }else{
            wanted[1]=0;
        }
        wanted[2] = turn;
        
        mod1.set_polor(wanted[0], wanted[1]);
        mod2.set_polor(wanted[0], -wanted[1]);
    }

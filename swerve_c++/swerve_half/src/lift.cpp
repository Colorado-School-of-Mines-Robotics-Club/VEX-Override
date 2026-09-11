#include "api.h"
#include "lift.h"

using namespace pros;

Lift::Lift(){
    DR4B_R = nullptr;
    DR4B_L = nullptr;
    Chain_bar = nullptr;
    Duo_bar = nullptr;
    phematicks_a = nullptr;
    phematicks_b = nullptr;
    rotchain = nullptr;
    state = 0;
    wanted_height = 0;
    pin = 0;
    lift_pid = nullptr;
    chain_pid = nullptr;
    duo_pid = nullptr;
}

Lift::Lift(pros::Motor* DR4B_R_in, pros::Motor* DR4B_L_in, pros::Motor* Chain_bar_in, pros::Motor* Duo_bar_in, pros::adi::DigitalIn* phematicks_a_in, pros::adi::DigitalIn* phematicks_b_in, pros::adi::AnalogIn* rotchain_in){
    DR4B_R = DR4B_R_in;
    DR4B_L = DR4B_L_in;
    Chain_bar = Chain_bar_in;
    Duo_bar = Duo_bar_in;
    phematicks_a = phematicks_a_in;
    phematicks_b = phematicks_b_in;
    rotchain = rotchain_in;
    state = 0;
    wanted_height = 0;
    pin = 0;
    lift_pid = new PID(0, 0, 0, 0, 0, 0);
    chain_pid = new PID(0, 0, 0, 0, 0, 0);
    duo_pid = new PID(0, 0, 0, 0, 0, 0);
}

int Lift::get_height(){

    double angle = (DR4B_R->get_position()+DR4B_L->get_position())/6-DR4B_angle;

    double DR4B_height = sin(angle/180*M_PI)*(13+10);

    double duo_angle = Duo_bar->get_position()/3;

    double duo_height = sin(duo_angle/180*M_PI)*10;

    return DR4B_height+duo_height+base_height;
    
}

int Lift::get_max_height(){
    double angle = (DR4B_R->get_position()+DR4B_L->get_position())/6-DR4B_angle;

    double DR4B_height = sin(angle/180*M_PI)*(13+10);

    double duo_angle = Duo_bar->get_position()/3;

    double duo_height = sin(duo_angle/180*M_PI)*10;

    if (duo_height > 0) {
        return DR4B_height+duo_height+base_height;

    }else{
        return DR4B_height+base_height;
    }
}

int Lift::get_x_pos(){
    double angle = (DR4B_R->get_position()+DR4B_L->get_position())/6-DR4B_angle;

    double offset = cos(angle/180*M_PI)*(3);

    double duo_angle = Duo_bar->get_position()/3;

    double duo_height = cos(duo_angle/180*M_PI)*10;

    return offset+duo_height+back_offset;
}

bool Lift::initalise(){
    
    DR4B_R->set_brake_mode(pros::E_MOTOR_BRAKE_HOLD);
    DR4B_L->set_brake_mode(pros::E_MOTOR_BRAKE_HOLD);
    Chain_bar->set_brake_mode(pros::E_MOTOR_BRAKE_HOLD);
    Duo_bar->set_brake_mode(pros::E_MOTOR_BRAKE_HOLD);

    DR4B_R->move_velocity(-50);
    DR4B_L->move_velocity(-50);

    while(DR4B_L->get_actual_velocity()+DR4B_R->get_actual_velocity() > 0){
        pros::delay(10);
    }

    delay(50);

    DR4B_R->move_velocity(0);
    DR4B_L->move_velocity(0);

    delay(50);

    DR4B_R->tare_position();
    DR4B_L->tare_position();

    delay(50);

    Duo_bar->move_velocity(-50);
    while(Duo_bar->get_actual_velocity() > 0){
        pros::delay(10);
    }

    delay(50);

    Duo_bar->move_velocity(0);

    delay(50);

    Duo_bar->tare_position();

    return true;
}

void Lift::update(){
    switch (state){
        case 0: // intakeing case
            
        case 1: // scoreing case
            
        case 2: // docked case

        case 3: // under 18 case
            break;
    }
}
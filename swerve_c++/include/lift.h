#include "api.h"
#include "pros/adi.hpp"
#include "pid.h"

class Lift {
private:
    pros::Motor* DR4B_R;
    pros::Motor* DR4B_L;
    pros::Motor* Chain_bar;
    pros::Motor* Duo_bar;
    pros::adi::DigitalOut* phematicks_a;
    pros::adi::DigitalOut* phematicks_b;
    pros::adi::AnalogIn* rotchain;
    int state;
    int wanted_height;
    int pin;
    PID* lift_pid;
    PID* chain_pid;
    PID* duo_pid;
    int DR4B_angle = 30;
    int Chain_angle = 60;
    int base_height = 20;
    int back_offset = -6;

public:
    Lift();
    Lift(pros::Motor& DR4B_R, pros::Motor& DR4B_L, pros::Motor& Chain_bar, pros::Motor& Duo_bar, pros::adi::DigitalOut& phematicks_a, pros::adi::DigitalOut& phematicks_b, pros::adi::AnalogIn& rotchain);
    void update();
    bool initalise();
    void set_state(int new_state);
    void set_height(int height);
    void set_pin(int pin);
    void intake();
    int get_height();
    int get_max_height();
    int get_x_pos();

};
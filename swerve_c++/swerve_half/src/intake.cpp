//#include "api.h"
#include "intake.h"
using namespace pros;

intake::intake(){
    bottoma = nullptr;
    bottomb = nullptr;
    top = nullptr;
    state = 0;
}

intake::intake(pros::Motor* top_in, pros::Motor* bottoma_in, pros::Motor* bottomb_in){
    bottoma = bottoma_in;
    bottomb = bottomb_in;
    top = top_in;
    state = 0;
}

void intake::update(){
    switch (state){
        case 0:
            bottoma->move(0);
            bottomb->move(0);
            top->move(0);
            break;
        case 1:
            bottoma->move(127);
            bottomb->move(127);
            top->move(-127);
            break;
        case 2:
            bottoma->move(-127);
            bottomb->move(-127);
            top->move(127);
            break;
    }
}

void intake::set_state(int new_state){
    state = new_state;
}
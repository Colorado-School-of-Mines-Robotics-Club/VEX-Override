//#include "api.h"
#include "intake.h"
using namespace pros;

Intake::Intake(){
    bottoma = nullptr;
    bottomb = nullptr;
    top = nullptr;
    state = 0;
}

Intake::Intake(pros::Motor& top_in, pros::Motor& bottoma_in, pros::Motor& bottomb_in){
    bottoma = &bottoma_in;
    bottomb = &bottomb_in;
    top = &top_in;
    state = 0;
}

void Intake::update(){
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

void Intake::set_state(int new_state){
    state = new_state;
}

bool Intake::initalise(){
    return (true);
}
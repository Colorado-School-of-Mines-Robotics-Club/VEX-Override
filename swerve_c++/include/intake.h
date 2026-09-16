#include "api.h"

class Intake {
private:
    pros::Motor* bottoma;
    pros::Motor* bottomb;
    pros::Motor* top;
    int state;

public:
    Intake();
    Intake(pros::Motor& top, pros::Motor& bottoma, pros::Motor& bottomb);
    void update();
    void set_state(int new_state);
    bool initalise();
};
#include "api.h"

class intake {
private:
    pros::Motor* bottoma;
    pros::Motor* bottomb;
    pros::Motor* top;
    int state;

public:
    intake();
    intake(pros::Motor* top, pros::Motor* bottoma, pros::Motor* bottomb);
    void update();
    void set_state(int new_state);
};
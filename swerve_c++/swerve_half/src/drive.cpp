#include "main.h"

int a0 = 1750;
int b0 = 1780;

void update_drive(double direction, double speed, double turn) {
double heading= imu.get_heading();
double a_error= (rota.get_value() + a0)%4096;
double b_error= (rotb.get_value() + b0)%4096;

printf("heading: %f, a_error: %f, b_error: %f\n", heading, a_error, b_error);
}
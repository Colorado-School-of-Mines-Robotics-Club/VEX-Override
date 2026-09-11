#include "main.h"
#include "pros/misc.h"

/**
 * Runs initialization code. This occurs as soon as the program is started.
 *
 * All other competition modes are blocked by initialize; it is recommended
 * to keep execution time for this mode under a few seconds.
 */
void initialize() {

	printf("Hello person or thing!\n");

	imu.reset();
	int time = pros::millis();
	int iter = 0;
	while (imu.is_calibrating()) {
		printf("IMU calibrating... %d\n", iter);
		iter += 10;
		pros::delay(10);
	}
	printf("IMU is done calibrating (took %d ms)\n", iter - time);

	printf("Initialization complete!\n");
}

/**
 * Runs while the robot is in the disabled state of Field Management System or
 * the VEX Competition Switch, following either autonomous or opcontrol. When
 * the robot is enabled, this task will exit.
 */
void disabled() {
	printf("Disabled started!\n");
}

/**
 * Runs after initialize(), and before autonomous when connected to the Field
 * Management System or the VEX Competition Switch. This is intended for
 * competition-specific initialization routines, such as an autonomous selector
 * on the LCD.
 *
 * This task will exit when the robot is enabled and autonomous or opcontrol
 * starts.
 */
void competition_initialize() {
	printf("Competition initialize started!\n");
}

/**
 * Runs the user autonomous code. This function will be started in its own task
 * with the default priority and stack size whenever the robot is enabled via
 * the Field Management System or the VEX Competition Switch in the autonomous
 * mode. Alternatively, this function may be called in initialize or opcontrol
 * for non-competition testing purposes.
 *
 * If the robot is disabled or communications is lost, the autonomous task
 * will be stopped. Re-enabling the robot will restart the task, not re-start it
 * from where it left off.
 */
void autonomous() {
	printf("Autonomous started!\n");

}

/**
 * Runs the operator control code. This function will be started in its own task
 * with the default priority and stack size whenever the robot is enabled via
 * the Field Management System or the VEX Competition Switch in the operator
 * control mode.
 *
 * If no competition control is connected, this function will run immediately
 * following initialize().
 *
 * If the robot is disabled or communications is lost, the
 * operator control task will be stopped. Re-enabling the robot will restart the
 * task, not resume it from where it left off.
 */
void opcontrol() {
	printf("Driver started!\n");
	
	Drive drive(swerve_a1, swerve_a2, swerve_b1, swerve_b2, swerve_c1, swerve_c2, swerve_d1, swerve_d2, imu, rota, rotb, rotc, rotd);
	
	pros::delay(5000); // imu calabration

	while (true)
	{
		double turn_power = master.get_analog(pros::E_CONTROLLER_ANALOG_RIGHT_X);
		double deltax = -master.get_analog(pros::E_CONTROLLER_ANALOG_LEFT_X);
		double deltay = master.get_analog(pros::E_CONTROLLER_ANALOG_LEFT_Y);

		//double wanted_angle = atan2(deltay, deltax)*180/M_PI-90;
		//if (wanted_angle < 0){wanted_angle += 360;}
		//double wanted_speed = sqrt(deltay*deltay + deltax*deltax);
		//if (wanted_speed > 127){
		//	wanted_speed = 127;}
		//if (wanted_speed < 10){
		//	wanted_speed = -1;}
		//printf("wanted_angle: %f, wanted_speed: %f turn_power: %f\n", wanted_angle, wanted_speed, turn_power);
		//drive.set_double(wanted_angle, wanted_speed, turn_power);

		drive.set_absolute(deltax, deltay, turn_power);
		drive.update();
		pros::delay(20);
	}
}
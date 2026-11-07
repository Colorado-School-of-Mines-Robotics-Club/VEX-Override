use std::time::{Duration, Instant};

use vexide::{smart::motor::Motor, time::sleep};

/// Waits for a motor to settle at 0 velocity, as reported by the internal encoder
pub async fn wait_for_stop(motor: &Motor) {
	const DEBOUNCE: Duration = Duration::from_millis(500);

	// Give the motors time to start moving
	sleep(Duration::from_millis(250)).await;

	let mut hit_zero: Option<Instant> = None;
	loop {
		// If we're at zero but not for >DEBOUNCE, wait
		// If we're not at zero, wait
		//
		// If we're at zero but >DEBOUNCE, motors have settled
		let velocity = motor.velocity().unwrap_or_default();
		if let Some(time) = hit_zero {
			if velocity != 0.0 {
				hit_zero = None;
			} else if velocity == 0.0 && time.elapsed() >= DEBOUNCE {
				return;
			}
		} else {
			if velocity == 0.0 {
				hit_zero = Some(Instant::now());
			}
		}
		sleep(Duration::from_millis(50)).await;
	}
}

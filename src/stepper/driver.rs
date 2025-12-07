use crate::{Float, peripheral::port::PinState};

pub struct StepperDriver {
    pub step_pin_state: PinState,
    pub step: usize,
    pub microsteps: usize,
}

/// Model stepper motor driver (DRV8825)
impl StepperDriver {
    pub fn new(microsteps: usize) -> Self {
        Self {
            step_pin_state: PinState::Low,
            step: 0,
            microsteps: microsteps, // TODO(stepper): read from pins
        }
    }

    pub fn step(&mut self, step_pin_state: PinState, dir_pin_state: &PinState) {
        if matches!(self.step_pin_state, PinState::High) && matches!(step_pin_state, PinState::Low)
        {
            let n_steps = if self.microsteps == 2 {
                8
            } else if self.microsteps == 4 {
                16
            } else {
                panic!("unknown microsteps setting: {}", self.microsteps)
            };

            if matches!(dir_pin_state, PinState::High) {
                self.step = (self.step + 1) % n_steps;
            } else if matches!(dir_pin_state, PinState::Low) {
                self.step = (self.step + n_steps - 1) % n_steps;
            }
        }
        self.step_pin_state = step_pin_state;
    }

    pub fn currents(&self) -> (Float, Float) {
        match self.step_pin_state {
            PinState::Input => return (0., 0.),
            PinState::InputPullUp => return (0., 0.),
            _ => {}
        }
        let half_step_current = 1. / (2.0 as Float).sqrt();
        let cos_pi_over_8 = 0.924; // TODO(stepper): make more precise?
        let sin_pi_over_8 = 0.383;
        if self.microsteps == 2 {
            let current_seq = [
                (1., 0.),
                (half_step_current, half_step_current),
                (0., 1.),
                (-half_step_current, half_step_current),
                (-1., 0.),
                (-half_step_current, -half_step_current),
                (0., -1.),
                (half_step_current, -half_step_current),
            ];

            return current_seq[self.step];
        } else if self.microsteps == 4 {
            let current_seq = [
                (1., 0.),
                (cos_pi_over_8, sin_pi_over_8),
                (half_step_current, half_step_current),
                (sin_pi_over_8, cos_pi_over_8),
                (0., 1.),
                (-sin_pi_over_8, cos_pi_over_8),
                (-half_step_current, half_step_current),
                (-cos_pi_over_8, sin_pi_over_8),
                (-1., 0.),
                (-cos_pi_over_8, -sin_pi_over_8),
                (-half_step_current, -half_step_current),
                (-sin_pi_over_8, -cos_pi_over_8),
                (0., -1.),
                (sin_pi_over_8, -cos_pi_over_8),
                (half_step_current, -half_step_current),
                (cos_pi_over_8, -sin_pi_over_8),
            ];
            return current_seq[self.step];
        } else {
            panic!("unknown microsteps setting: {}", self.microsteps)
        }
    }
}

#[cfg(test)]
mod stepper_driver_tests {
    use crate::{
        Float, PI, assert_close,
        peripheral::port::PinState,
        stepper::{P, StepperMotor, driver::StepperDriver},
    };

    #[test]
    fn half_stepping() {
        // Arrange
        let mut driver = StepperDriver::new(2);
        let mut motor = StepperMotor::new();
        let full_step = (2. * PI) / (4. * P as Float);
        let half_step = full_step / 2.0;
        let angle_tolerance = 0.09 / 180. * PI;

        // Act
        let dir_pin = PinState::High;

        let dt = 1e-4;
        driver.step(PinState::High, &dir_pin);
        let currents = driver.currents();
        for _ in 0..10 {
            motor.step(dt, currents.0, currents.1, 0.);
        }

        driver.step(PinState::Low, &dir_pin);
        let currents = driver.currents();
        for _ in 0..10 {
            motor.step(dt, currents.0, currents.1, 0.);
        }

        // Assert
        assert_close!(motor.theta, half_step, angle_tolerance);
    }

    #[test]
    fn four_microstepping() {
        // Arrange
        let mut driver = StepperDriver::new(4);
        let mut motor = StepperMotor::new();
        let full_step = (2. * PI) / (4. * P as Float);
        let one_fourth_step = full_step / 4.0;
        let angle_tolerance = 0.09 / 180. * PI;

        // Act
        let dir_pin = PinState::High;

        let dt = 1e-4;
        driver.step(PinState::High, &dir_pin);
        let currents = driver.currents();
        for _ in 0..10 {
            motor.step(dt, currents.0, currents.1, 0.);
        }

        driver.step(PinState::Low, &dir_pin);
        let currents = driver.currents();
        for _ in 0..10 {
            motor.step(dt, currents.0, currents.1, 0.);
        }

        // Assert
        assert_close!(motor.theta, one_fourth_step, angle_tolerance);
    }

    #[test]
    fn continuous_stepping() {
        // Arrange
        let mut driver = StepperDriver::new(4);
        let mut motor = StepperMotor::new();
        let full_step = (2. * PI) / (4. * P as Float);
        let one_fourth_step = full_step / 4.0;
        let n_steps = 100;
        let angle_tolerance = 0.09 / 180. * PI;

        // Act
        let dir_pin = PinState::High;
        let dt = 1e-4;
        for _ in 0..n_steps {
            driver.step(PinState::High, &dir_pin);
            let currents = driver.currents();
            for _ in 0..10 {
                motor.step(dt, currents.0, currents.1, 0.);
            }

            driver.step(PinState::Low, &dir_pin);
            let currents = driver.currents();
            for _ in 0..10 {
                motor.step(dt, currents.0, currents.1, 0.);
            }
        }

        // Assert
        assert_close!(
            motor.theta,
            one_fourth_step * n_steps as Float,
            angle_tolerance
        );
    }

    #[test]
    fn reverse_direction() {
        // Arrange
        let mut driver = StepperDriver::new(4);
        let mut motor = StepperMotor::new();
        let full_step = (2. * PI) / (4. * P as Float);
        let one_fourth_step = full_step / 4.0;
        let n_steps = 100;
        let angle_tolerance = 0.09 / 180. * PI;

        // Act
        let dir_pin = PinState::Low;
        let dt = 1e-4;
        for _ in 0..n_steps {
            driver.step(PinState::High, &dir_pin);
            let currents = driver.currents();
            for _ in 0..10 {
                motor.step(dt, currents.0, currents.1, 0.);
            }

            driver.step(PinState::Low, &dir_pin);
            let currents = driver.currents();
            for _ in 0..10 {
                motor.step(dt, currents.0, currents.1, 0.);
            }
        }

        // Assert
        assert_close!(
            motor.theta,
            -one_fourth_step * n_steps as Float,
            angle_tolerance
        );
    }
}

use crate::{Float, peripheral::port::PinState};

pub struct StepperDriver {
    pub step_pin_state: PinState,

    pub step: usize,

    pub microsteps: usize,
}

/// Model stepper motor driver (DRV8825)
impl StepperDriver {
    pub fn new() -> Self {
        Self {
            step_pin_state: PinState::Low,
            step: 0,
            microsteps: 4, // TODO(stepper): read from pins
        }
    }

    pub fn step(&mut self, step_pin_state: PinState, dir_pin_state: PinState) {
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

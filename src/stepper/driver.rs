use crate::{Float, port::PinState};

pub struct StepperDriver {
    pub step_pin_state: PinState,

    pub step: usize,
}

impl StepperDriver {
    pub fn new() -> Self {
        Self {
            step_pin_state: PinState::Low,
            step: 0,
        }
    }

    pub fn step(&mut self, step_pin_state: PinState) {
        if matches!(self.step_pin_state, PinState::High) && matches!(step_pin_state, PinState::Low)
        {
            self.step = (self.step + 1) % 8;
        }
        self.step_pin_state = step_pin_state;
    }

    pub fn currents(&self) -> (Float, Float) {
        let half_step_current = 1. / (2.0 as Float).sqrt();
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
        current_seq[self.step]
    }
}

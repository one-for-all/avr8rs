use crate::Float;

pub mod driver;

const KT: Float = 10.0; // 1.0
const J: Float = 5.4e-5; // rotor inertia
const B: Float = 5e-1; // 5e-2 // viscous friction coefficient [N*m*s/rad]
const P: usize = 50; // pole-pairs

const I_MAX: Float = 1.0;

pub struct StepperVoltages {
    pub ap: Float,
    pub am: Float,
    pub bp: Float,
    pub bm: Float,
}

/// Simulate stepper motor
pub struct StepperMotor {
    pub omega: Float,
    pub theta: Float,
}

impl StepperMotor {
    pub fn new() -> Self {
        Self {
            omega: 0.,
            theta: 0.,
        }
    }

    fn eletromagnetic_torque(&self, ia: Float, ib: Float) -> Float {
        let theta = self.theta;
        KT * (-ia * (P as Float * theta).sin() + ib * (P as Float * theta).cos())
    }

    pub fn torque(&self, ia: Float, ib: Float) -> Float {
        self.eletromagnetic_torque(ia, ib) - B * self.omega
    }

    pub fn step(&mut self, dt: Float, ia: Float, ib: Float, load_torque: Float) {
        let domega_dt = (self.torque(ia, ib) - load_torque) / J;

        self.omega += domega_dt * dt;
        self.theta += self.omega * dt;
    }

    pub fn step_voltage(&mut self, dt: Float, voltages: &StepperVoltages, load_torque: Float) {
        let ia = self.current(voltages.ap, voltages.am);
        let ib = self.current(voltages.bp, voltages.bm);
        let torque = self.eletromagnetic_torque(ia, ib);

        let domega_dt = (torque - B * self.omega - load_torque) / J;

        self.omega += domega_dt * dt;
        self.theta += self.omega * dt;
    }

    fn current(&self, vp: Float, vm: Float) -> Float {
        I_MAX * (vp - vm)
    }
}

#[cfg(test)]
mod stepper_tests {

    use crate::{
        Float, PI, assert_close,
        plot::plot,
        stepper::{P, StepperMotor},
    };

    #[test]
    fn full_step() {
        // Arrange
        let mut stepper = StepperMotor::new();
        let full_step = PI / 2.0 / P as Float; // angle turned by a full-step
        stepper.theta = -full_step / 2.;
        let load_torque = 0.;

        // Act
        let mut data = vec![];
        let t_final = 1e-2;
        let dt = 1e-4;
        let n_steps = (t_final / dt) as usize;
        for _ in 0..n_steps {
            data.push(stepper.theta);
            stepper.step(dt, 1.0, 1.0, load_torque);
        }

        data.push(stepper.theta);

        // Assert
        // plot(&data, t_final, dt, n_steps, "stepper");
        println!("final theta: {}", stepper.theta);
        println!("expect theta: {}", full_step / 2.);
        assert_close!(stepper.theta, full_step / 2., 1e-5);
    }

    #[test]
    fn half_step() {
        // Arrange
        let mut stepper = StepperMotor::new();
        let full_step = PI / 2.0 / P as Float;
        let half_step = full_step / 2.0;
        let half_step_current = 1. / (2.0 as Float).sqrt();
        let load_torque = 0.;

        // Act
        let mut data = vec![];
        let t_final = 1e-2;
        let dt = 1e-4;
        let n_steps = (t_final / dt) as usize;
        for _ in 0..n_steps {
            data.push(stepper.theta);
            stepper.step(dt, half_step_current, half_step_current, load_torque);
        }

        // Assert
        // plot(&data, t_final, dt, n_steps, "stepper");
        println!("final theta: {}", stepper.theta);
        println!("expect theta: {}", half_step);
        assert_close!(stepper.theta, half_step, 1e-5);
    }
}

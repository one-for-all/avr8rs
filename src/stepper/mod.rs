use crate::Float;

pub mod driver;

const KT: Float = 5.0; // motor constant
const J: Float = 5.7e-6; // rotor inertia kg * m^2
const B: Float = 5e-2; // viscous friction coefficient [N*m*s/rad]
const P: usize = 50; // number of pole-pairs. gives P * 4 full steps

/// Simulate stepper motor (Nema 17 HS4023)
/// Spec:
///     holding torque: 0.13 Nm
///     weight: 132 g
///     step angle: 1.8 +- 0.09 degrees
///     rated voltage: 4.1 V
///     rated current: DC 1.0 amp / phase
///     speed: at 1.8 degrees increments, up to ~500 RPM (12 V, no loead), ~1800 RPM (24 V, no load)
/// https://www.datasheetcafe.com/wp-content/uploads/2021/03/17HS4023.pdf
/// https://www.aliexpress.com/item/1005003874936862.html
/// https://makersportal.com/shop/nema-17-stepper-motor-kit-17hs4023-drv8825-bridge
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
}

#[cfg(test)]
mod stepper_tests {

    use crate::{
        Float, PI, assert_close,
        stepper::{P, StepperMotor},
    };

    #[test]
    fn full_step() {
        // Arrange
        let mut stepper = StepperMotor::new();
        let full_step = (2. * PI) / (4. * P as Float); // angle turned by a full-step
        stepper.theta = -full_step / 2.;
        let angle_tolerance = 0.09 / 180. * PI;
        let speed = 500. * 2. * PI / 60.; // 500 rpm

        // Act
        // let mut data = vec![];
        let t_final = full_step / speed;
        let dt = 1e-4;
        let n_steps = (t_final / dt) as usize;
        for _ in 0..n_steps {
            // data.push(stepper.theta);
            stepper.step(dt, 1.0, 1.0, 0.);
        }

        // Assert
        // plot(&data, dt, "stepper");
        // println!("final theta: {}", stepper.theta);
        // println!("expect theta: {}", full_step / 2.);
        assert_close!(stepper.theta, full_step / 2., angle_tolerance);
    }

    #[test]
    fn half_step() {
        // Arrange
        let mut stepper = StepperMotor::new();
        let full_step = (2. * PI) / (4. * P as Float);
        let half_step = full_step / 2.0;
        let half_step_current = 1. / (2.0 as Float).sqrt();
        let angle_tolerance = 0.09 / 180. * PI;
        let speed = 500. * 2. * PI / 60.; // 500 rpm

        // Act
        // let mut data = vec![];
        let t_final = full_step / speed;
        let dt = 1e-4;
        let n_steps = (t_final / dt) as usize;
        for _ in 0..n_steps {
            // data.push(stepper.theta);
            stepper.step(dt, half_step_current, half_step_current, 0.);
        }

        // Assert
        // plot(&data, dt, "stepper");
        // println!("final theta: {}", stepper.theta);
        // println!("expect theta: {}", half_step);
        assert_close!(stepper.theta, half_step, angle_tolerance);
    }

    #[test]
    fn holding_torque() {
        // Arrange
        let mut stepper = StepperMotor::new();
        let holding_torque = 0.13;
        let angle_tolerance = 0.09 / 180. * PI;

        // Act
        let t_final = 1.0;
        let dt = 1e-4;
        let n_steps = (t_final / dt) as usize;
        for _ in 0..n_steps {
            stepper.step(dt, 1.0, 0.0, holding_torque);
        }

        // Assert
        assert_close!(stepper.theta, 0., angle_tolerance);
    }

    #[test]
    fn holding_torque_exceed() {
        // Arrange
        let mut stepper = StepperMotor::new();
        let large_torque = 0.13 * 10.; // 10 times the holding torque
        let angle_tolerance = 0.09 / 180. * PI;

        // Act
        let t_final = 1.0;
        let dt = 1e-4;
        let n_steps = (t_final / dt) as usize;
        for _ in 0..n_steps {
            stepper.step(dt, 1.0, 0.0, large_torque);
        }

        // Assert
        assert!(stepper.theta.abs() > angle_tolerance);
    }
}

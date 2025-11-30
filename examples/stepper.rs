use std::{
    fs::File,
    io::{BufReader, Read},
};

use avr8rs::{
    Float,
    plot::plot,
    runner::AVRRunner,
    stepper::{StepperMotor, driver::StepperDriver},
};

fn main() {
    let file = File::open("build/stepper.ino.hex").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: String = String::new();
    let _ = reader.read_to_string(&mut buf);

    let mut runner = AVRRunner::new(&buf);

    let mut driver = StepperDriver::new();
    let mut stepper = StepperMotor::new();

    let mut data: Vec<Float> = vec![];

    let final_time = 2.1; //2e-5; //
    let Hz = 16e6; // 16 MHz
    let dt = 1. / Hz;
    let n_steps = (final_time / dt) as usize;

    let load_torque = 0.;

    let mut s = 0;
    let mut motor_s = 0;
    while s < n_steps {
        // print!("cycle: {} ", s);

        let cycles = runner.atmega328p.cpu.cycles;
        runner.step(None);
        let delta_cycles = (runner.atmega328p.cpu.cycles - cycles) as usize;

        for _ in 0..delta_cycles {
            let step_pin = runner.atmega328p.port_pin_state("D", 3);
            let dir_pin = runner.atmega328p.port_pin_state("D", 2);
            driver.step(step_pin, dir_pin);
        }

        if s % 100 == 0 {
            let currents = driver.currents();
            stepper.step(
                dt * (s - motor_s) as Float,
                currents.0,
                currents.1,
                load_torque,
            );
            for _ in 0..s - motor_s {
                data.push(stepper.theta);
            }
            motor_s = s;
        }
        s += delta_cycles;
    }

    println!("final theta: {}", stepper.theta);
    // println!("step count: {}", count);
    // println!("PD3: {:?}", get_voltage(&runner.cpu, "D", 3));

    // println!("DDRC: {:08b}", runner.cpu.data[PORTC_CONFIG.DDR as usize]);
    // println!("PINC: {:08b}", runner.cpu.data[PORTC_CONFIG.PIN as usize]);
    // println!("DDRD: {:08b}", runner.cpu.data[PORTD_CONFIG.DDR as usize]);
    // println!("PIND: {:08b}", runner.cpu.data[PORTD_CONFIG.PIN as usize]);

    // plot(&data, final_time, dt, n_steps, "PD3");

    plot(&data, dt, "stepper motor");
}

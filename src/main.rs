use std::{
    any::Any,
    fs::File,
    io::{BufReader, Read},
};

use avr8rs::{
    Float,
    cpu::CPU,
    plot::plot,
    program::load_hex,
    runner::AVRRunner,
    stepper::{Stepper, StepperVoltages},
};

fn main() {
    let file = File::open("build/stepper.ino.hex").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: String = String::new();
    let _ = reader.read_to_string(&mut buf);

    let mut runner = AVRRunner::new(&buf);

    let mut stepper = Stepper::new();

    let mut data = vec![];

    let final_time = 100e-3; //2e-5; //
    let Hz = 16e6; // 16 MHz
    let dt = 1. / Hz;
    let n_steps = (final_time / dt) as usize;

    let mut s = 0;
    while s < n_steps {
        println!("cycle: {} ", s);

        let cycles = runner.cpu.cycles;
        runner.step();
        let delta_cycles = (runner.cpu.cycles - cycles) as usize;

        let ap = get_voltage(&runner.cpu, 2);
        let am = get_voltage(&runner.cpu, 3);
        let bp = get_voltage(&runner.cpu, 1);
        let bm = get_voltage(&runner.cpu, 0);

        let voltages = StepperVoltages { ap, am, bp, bm };
        for _ in 0..delta_cycles {
            stepper.step_voltage(dt, &voltages);
            data.push(stepper.theta);
        }

        let count = runner.cpu.timer0.tcnt;
        // println!("time: {} count", count);

        // for _ in 0..delta_cycles {
        //     // data.push(count as Float);

        //     data.push(bm);
        //     // data.push(runner.cpu.get_data(0x6e as u16) as Float);
        //     // data.push(runner.cpu.next_interrupt as Float);
        // }

        s += delta_cycles;
    }

    // println!("PB0: {:?}", runner.cpu.port_b_pin_state(0));
    plot(&data, final_time, dt, n_steps, "PB0");
}

fn get_voltage(cpu: &CPU, i: u8) -> Float {
    match cpu.port_b_pin_state(i) {
        avr8rs::port::PinState::Low => 0.,
        avr8rs::port::PinState::High => 1.,
        _ => 0.,
    }
}

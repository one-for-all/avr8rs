use std::{
    any::Any,
    fs::File,
    io::{BufReader, Read},
};

use avr8rs::{
    Float,
    cpu::{self, CPU},
    plot::plot,
    port::{PORTB_CONFIG, PORTC_CONFIG, PORTD_CONFIG},
    program::load_hex,
    runner::AVRRunner,
    stepper::{Stepper, StepperVoltages, driver::StepperDriver},
};

fn main() {
    let file = File::open("build/stepper.ino.hex").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: String = String::new();
    let _ = reader.read_to_string(&mut buf);

    let mut runner = AVRRunner::new(&buf);

    let mut stepper = Stepper::new();
    let mut driver = StepperDriver::new();

    let mut data: Vec<Float> = vec![];
    let mut data2 = vec![];

    let final_time = 0.1; //2e-5; //
    let Hz = 16e6; // 16 MHz
    let dt = 1. / Hz;
    let n_steps = (final_time / dt) as usize;

    let mut s = 0;
    let mut PD = 0.;
    let mut PB = 0.;
    let mut count = 0;

    let mut motor_s = 0;

    while s < n_steps {
        // print!("cycle: {} ", s);

        let cycles = runner.atmega328p.cpu.cycles;
        runner.step();
        let delta_cycles = (runner.atmega328p.cpu.cycles - cycles) as usize;

        // let ap = get_voltage(&runner.cpu, 2);
        // let am = get_voltage(&runner.cpu, 3);
        // let bp = get_voltage(&runner.cpu, 1);
        // let bm = get_voltage(&runner.cpu, 0);

        // let voltages = StepperVoltages { ap, am, bp, bm };
        for _ in 0..delta_cycles {
            // stepper.step_voltage(dt, &voltages);

            driver.step(runner.atmega328p.cpu.pin_state("D", 3));

            // let new_PD = get_voltage(&runner.cpu, "D", 3);
            // data.push(new_PD);
            // if PD == 0. && new_PD == 1. {
            //     println!("step: {}", s);
            //     count += 1;
            // }
            // PD = new_PD;

            let new_PB = get_voltage(&runner.atmega328p.cpu, "B", 3);
            if PB == 0. && new_PB == 1. {
                println!("step: {}", s);
            }
            data2.push(new_PB);
            PB = new_PB;
        }

        if runner.atmega328p.cpu.data[PORTB_CONFIG.DDR as usize] == 0b11111111 {
            println!("finished");
            return;
        }

        s += delta_cycles;

        if s % 100 == 0 {
            let currents = driver.currents();
            stepper.step(dt * (s - motor_s) as Float, currents.0, currents.1, 0.1);
            for _ in 0..s - motor_s {
                data2.push(stepper.theta);
            }
            motor_s = s;
        }
    }

    println!("SREG: {:08b}", runner.atmega328p.cpu.sreg());
    println!(
        "cycles per char: {}",
        runner.atmega328p.usart_parity_enabled()
    );

    // println!("theta: {}", stepper.theta);
    // println!("step count: {}", count);
    // println!("PD3: {:?}", get_voltage(&runner.cpu, "D", 3));

    // println!("DDRC: {:08b}", runner.cpu.data[PORTC_CONFIG.DDR as usize]);
    // println!("PINC: {:08b}", runner.cpu.data[PORTC_CONFIG.PIN as usize]);
    // println!("DDRD: {:08b}", runner.cpu.data[PORTD_CONFIG.DDR as usize]);
    // println!("PIND: {:08b}", runner.cpu.data[PORTD_CONFIG.PIN as usize]);

    // plot(&data, final_time, dt, n_steps, "PD3");
    // plot(&data2, final_time, dt, n_steps, "PB3");
}

fn get_voltage(cpu: &CPU, key: &str, i: u8) -> Float {
    match cpu.pin_state(key, i) {
        avr8rs::port::PinState::Low => 0.,
        avr8rs::port::PinState::High => 1.,
        _ => 0.,
    }
}

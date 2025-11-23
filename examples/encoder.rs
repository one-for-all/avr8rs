use std::{
    fs::File,
    io::{BufReader, Read},
};

use avr8rs::runner::AVRRunner;

fn main() {
    let file = File::open("build/encoder.ino.hex").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: String = String::new();
    let _ = reader.read_to_string(&mut buf);

    let mut runner = AVRRunner::new(&buf);

    let final_time = 1.0; //2e-5; //
    let Hz = 16e6; // 16 MHz
    let dt = 1. / Hz;
    let n_steps = (final_time / dt) as usize;

    let mut s = 0;
    while s < n_steps {
        let cycles = runner.atmega328p.cpu.cycles;
        runner.step();
        let delta_cycles = (runner.atmega328p.cpu.cycles - cycles) as usize;

        s += delta_cycles;
    }
}

use std::{
    fs::File,
    io::{BufReader, Read},
};

use avr8rs::{encoder::AS5600, peripheral::i2c::bus::I2CBus, runner::AVRRunner};

fn main() {
    let file = File::open("build/rand.ino.hex").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: String = String::new();
    let _ = reader.read_to_string(&mut buf);

    let mut runner = AVRRunner::new(&buf);
    let mut i2c_bus = I2CBus::new();
    let mut encoder = AS5600::new();

    let final_time = 2.0; //2e-5; //
    let Hz = 16e6; // 16 MHz
    let dt = 1. / Hz;
    let n_steps = (final_time / dt) as usize;

    let mut s = 0;
    while s < n_steps {
        let cycles = runner.atmega328p.cpu.cycles;
        runner.step(Some(&mut i2c_bus));
        let delta_cycles = (runner.atmega328p.cpu.cycles - cycles) as usize;

        encoder.step(&mut i2c_bus);

        s += delta_cycles;
    }
}

use std::{
    fs::File,
    io::{BufReader, Read},
};

use avr8rs::{
    peripheral::i2c::{self, bus::I2CBus},
    runner::AVRRunner,
};

fn main() {
    let file = File::open("build/encoder.ino.hex").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf: String = String::new();
    let _ = reader.read_to_string(&mut buf);

    let mut runner = AVRRunner::new(&buf);
    let mut i2c_bus = I2CBus::new();

    let final_time = 1.0; //2e-5; //
    let Hz = 16e6; // 16 MHz
    let dt = 1. / Hz;
    let n_steps = (final_time / dt) as usize;

    let mut s = 0;
    let mut register_address;
    while s < n_steps {
        let cycles = runner.atmega328p.cpu.cycles;
        runner.step(Some(&mut i2c_bus));
        let delta_cycles = (runner.atmega328p.cpu.cycles - cycles) as usize;

        match i2c_bus.status {
            i2c::bus::I2CBusStatus::ADDRESS => {
                if i2c_bus.address == 0x36 {
                    i2c_bus.acked = true;
                    if i2c_bus.read {
                        i2c_bus.data = 0x20;
                    }
                }
            }
            i2c::bus::I2CBusStatus::DATA => {
                if !i2c_bus.read {
                    register_address = i2c_bus.data;
                    println!("read from {:02x}", register_address);
                } else {
                    i2c_bus.data = 0x20;
                }
            }
            _ => {}
        }

        s += delta_cycles;
        // println!("{:?}", runner.atmega328p.port_pin_state("B", 0));
    }
}

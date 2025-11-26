use crate::{
    atmega328p::{ATMega328P, DEFAULT_FREQ},
    cpu::CPU,
    instruction::avr_instruction,
    peripheral::i2c::bus::I2CBus,
    program::load_hex,
};

pub struct AVRRunner {
    // pub cpu: CPU,
    pub atmega328p: ATMega328P,
}

impl AVRRunner {
    pub fn new(hex: &str) -> Self {
        let atmega328p = ATMega328P::new(hex, DEFAULT_FREQ);
        AVRRunner { atmega328p }
    }

    pub fn step(&mut self, i2c_bus: Option<&mut I2CBus>) {
        self.atmega328p.step(i2c_bus);
    }
}

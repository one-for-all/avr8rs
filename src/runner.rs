use crate::{
    atmega328p::{ATMega328P, DEFAULT_FREQ},
    cpu::CPU,
    instruction::avr_instruction,
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

    pub fn step(&mut self) {
        self.atmega328p.step();
    }
}

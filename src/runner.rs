use crate::{cpu::CPU, instruction::avr_instruction, program::load_hex};

pub struct AVRRunner {
    pub cpu: CPU,
}

impl AVRRunner {
    pub fn new(hex: &str) -> Self {
        let prog = load_hex(&hex);
        let cpu = CPU::new(prog);

        AVRRunner { cpu }
    }

    pub fn step(&mut self) {
        avr_instruction(&mut self.cpu);
        self.cpu.tick();
    }
}

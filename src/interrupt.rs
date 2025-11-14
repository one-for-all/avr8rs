use crate::{cpu::CPU, timer::AVRTimerConfig};

pub const MAX_INTERRUPTS: usize = 128; // Enough for ATMega2560

#[derive(Clone, Copy)]
pub struct AVRInterruptConfig {
    pub address: u8,
    pub enable_register: u16,
    pub enable_mask: u8,
    pub flag_register: u16,
    pub flag_mask: u8,
}

impl AVRInterruptConfig {
    pub fn new(config: &AVRTimerConfig) -> Self {
        AVRInterruptConfig {
            address: config.ovf_interrupt,
            enable_register: config.TIMSK as u16,
            enable_mask: config.TOIE,
            flag_register: config.TIFR as u16,
            flag_mask: config.TOV,
        }
    }
}

pub fn avr_interrupt(cpu: &mut CPU, addr: u8) {
    let sp = cpu.get_data_u16(93);
    cpu.set_data(sp, (cpu.pc & 0xff) as u8);
    cpu.set_data(sp - 1, ((cpu.pc >> 8) & 0xff) as u8);
    if cpu.pc_22_bits {
        cpu.set_data(sp - 2, ((cpu.pc >> 16) & 0xff) as u8);
    }
    cpu.set_data_u16(93, sp - (if cpu.pc_22_bits { 3 } else { 2 }));
    cpu.data[95] &= 0x7f; // clear global interrupt flag
    cpu.cycles += 2;
    cpu.pc = addr as u32;
}

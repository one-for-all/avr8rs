use std::collections::HashMap;

use crate::{
    clock::AVRClockEventType,
    cpu::{CPU, CPUMemoryHook},
    instruction::avr_instruction,
    program::load_hex,
    usart::{AVRUSART, UCSRB_TXEN, USART0_CONFIG},
};

pub const DEFAULT_FREQ: usize = 16_000_000; // 16Mhz

pub type PeripheralMemoryHook = Box<dyn Fn(&mut ATMega328P, u8, u8, u16, u8) -> bool>;

pub struct ATMega328P {
    pub cpu: CPU,

    // peripherals
    pub usart: AVRUSART,

    // data hooks
    pub write_hooks: HashMap<u16, PeripheralMemoryHook>,
}

impl ATMega328P {
    pub fn new(hex: &str, freq_hz: usize) -> Self {
        let prog = load_hex(&hex);
        let cpu = CPU::new(prog, freq_hz);

        let mut write_hooks: HashMap<u16, PeripheralMemoryHook> = HashMap::new();

        let usart = AVRUSART::new(USART0_CONFIG, freq_hz);
        usart.add_ucsrb_handler(&mut write_hooks);
        usart.add_udr_handler(&mut write_hooks);

        let atmega328p = Self {
            cpu,
            usart,
            write_hooks,
        };

        atmega328p
    }

    pub fn write_data(&mut self, addr: u16, data: u8) {
        self.write_data_with_mask(addr, data, 0xff);
    }

    pub fn write_data_with_mask(&mut self, addr: u16, data: u8, mask: u8) {
        if addr == self.usart.config.UDR as u16 || addr == self.usart.config.UCSRB as u16 {
            if let Some((addr, write_hook)) = self.write_hooks.remove_entry(&addr) {
                let cpu_data = self.cpu.get_data(addr);
                let result = write_hook(self, data, cpu_data, addr, mask);
                self.write_hooks.insert(addr, write_hook);
                if result {
                    return;
                }
            }
        } else {
            let cpu = &mut self.cpu;
            if let Some((addr, write_hook)) = cpu.write_hooks.remove_entry(&addr) {
                let result = write_hook(cpu, data, cpu.get_data(addr), addr, mask);
                cpu.write_hooks.insert(addr, write_hook);
                if result {
                    return;
                }
            }
        }
        self.cpu.set_data(addr, data);
    }

    pub fn step(&mut self) {
        avr_instruction(self);
        self.cpu.tick();
    }
}

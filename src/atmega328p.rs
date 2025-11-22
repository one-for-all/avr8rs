use std::collections::HashMap;

use crate::{
    cpu::CPU,
    instruction::avr_instruction,
    port::{AVRIOPort, PORTB_CONFIG, PORTC_CONFIG, PORTD_CONFIG},
    program::load_hex,
    usart::{AVRUSART, USART0_CONFIG},
};

pub const DEFAULT_FREQ: usize = 16_000_000; // 16Mhz

pub type PeripheralMemoryHook = Box<dyn Fn(&mut ATMega328P, u8, u8, u16, u8) -> bool>;

pub struct ATMega328P {
    pub cpu: CPU,

    // peripherals
    pub usart: AVRUSART,
    pub ports: [AVRIOPort; 3], // B, C, D

    // data hooks
    pub write_hooks: HashMap<u16, PeripheralMemoryHook>,
}

impl ATMega328P {
    pub fn new(hex: &str, freq_hz: usize) -> Self {
        let prog = load_hex(&hex);
        let cpu = CPU::new(prog, freq_hz);

        let mut write_hooks: HashMap<u16, PeripheralMemoryHook> = HashMap::new();

        // Universal Synchronous/Asynchronous Receiver Transmitter
        let usart = AVRUSART::new(USART0_CONFIG, freq_hz);
        usart.add_ucsrb_handler(&mut write_hooks);
        usart.add_udr_handler(&mut write_hooks);

        // GPIO Ports
        let port_b = AVRIOPort::new(PORTB_CONFIG);
        port_b.add_ddr_handler(&mut write_hooks, 0);
        port_b.add_port_handler(&mut write_hooks, 0);

        let port_c = AVRIOPort::new(PORTC_CONFIG);
        port_c.add_ddr_handler(&mut write_hooks, 1);
        port_c.add_port_handler(&mut write_hooks, 1);

        let port_d = AVRIOPort::new(PORTD_CONFIG);
        port_d.add_ddr_handler(&mut write_hooks, 2);
        port_d.add_port_handler(&mut write_hooks, 2);

        let ports = [port_b, port_c, port_d];

        let atmega328p = Self {
            cpu,
            usart,
            ports,
            write_hooks,
        };

        atmega328p
    }

    pub fn write_data(&mut self, addr: u16, data: u8) {
        self.write_data_with_mask(addr, data, 0xff);
    }

    pub fn write_data_with_mask(&mut self, addr: u16, data: u8, mask: u8) {
        if addr == self.usart.config.UDR as u16
            || addr == self.usart.config.UCSRB as u16
            || addr == self.ports[0].config.DDR as u16
            || addr == self.ports[0].config.PORT as u16
            || addr == self.ports[1].config.DDR as u16
            || addr == self.ports[1].config.PORT as u16
            || addr == self.ports[2].config.DDR as u16
            || addr == self.ports[2].config.PORT as u16
        {
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

use std::collections::HashMap;

use crate::{
    cpu::CPU,
    instruction::avr_instruction,
    interrupt::avr_interrupt,
    peripheral::{
        eeprom::{AVREEPROM, AVREEPROMConfig, EEPROM_CONFIG},
        i2c::{AVRI2C, TWI_CONFIG, TWIConfig, bus::I2CBus},
        port::{AVRIOPort, PORTB_CONFIG, PORTC_CONFIG, PORTD_CONFIG},
        timer::{AVRTimer, TIMER_0_CONFIG},
        usart::{AVRUSART, USART0_CONFIG},
    },
    program::load_hex,
};

pub const DEFAULT_FREQ: usize = 16_000_000; // 16Mhz

pub type PeripheralMemoryReadHook = Box<dyn Fn(&mut ATMega328P, u16) -> u8>;
pub type PeripheralMemoryWriteHook = Box<dyn Fn(&mut ATMega328P, u8, u8, u16, u8) -> bool>;

pub struct ATMega328P {
    pub cpu: CPU,

    // peripherals
    pub timer0: AVRTimer,
    pub usart: AVRUSART,
    pub ports: [AVRIOPort; 3], // B, C, D
    pub i2c: AVRI2C,
    pub eeprom: AVREEPROM,

    // data hooks
    pub read_hooks: HashMap<u16, PeripheralMemoryReadHook>,
    pub write_hooks: HashMap<u16, PeripheralMemoryWriteHook>,
}

impl ATMega328P {
    pub fn new(hex: &str, freq_hz: usize) -> Self {
        let prog = load_hex(&hex);
        let mut cpu = CPU::new(prog);

        let timer0 = AVRTimer::new(TIMER_0_CONFIG);
        let usart = AVRUSART::new(USART0_CONFIG, freq_hz);
        let port_b = AVRIOPort::new(PORTB_CONFIG);
        let port_c = AVRIOPort::new(PORTC_CONFIG);
        let port_d = AVRIOPort::new(PORTD_CONFIG);
        let i2c = AVRI2C::new(TWI_CONFIG, freq_hz, &mut cpu);
        let eeprom = AVREEPROM::new(EEPROM_CONFIG, 1024);

        let mut read_hooks: HashMap<u16, PeripheralMemoryReadHook> = HashMap::new();

        let mut write_hooks: HashMap<u16, PeripheralMemoryWriteHook> = HashMap::new();

        // Timer
        timer0.add_TCNT_read_hook(&mut read_hooks);
        timer0.add_TCNT_write_hook(&mut write_hooks);
        timer0.add_TCCRB_write_hook(&mut write_hooks);
        timer0.add_TIMSK_write_hook(&mut write_hooks);

        // Universal Synchronous/Asynchronous Receiver Transmitter
        usart.add_ucsrb_handler(&mut write_hooks);
        usart.add_udr_handler(&mut write_hooks);

        // GPIO Ports
        port_b.add_ddr_handler(&mut write_hooks, 0);
        port_b.add_port_handler(&mut write_hooks, 0);

        port_c.add_ddr_handler(&mut write_hooks, 1);
        port_c.add_port_handler(&mut write_hooks, 1);

        port_d.add_ddr_handler(&mut write_hooks, 2);
        port_d.add_port_handler(&mut write_hooks, 2);

        let ports = [port_b, port_c, port_d];

        // I2C interface
        i2c.add_TWCR_write_hook(&mut write_hooks);

        // EEPROM
        eeprom.add_EECR_write_hook(&mut write_hooks);

        let atmega328p = Self {
            cpu,
            timer0,
            usart,
            ports,
            i2c,
            eeprom,
            read_hooks,
            write_hooks,
        };

        atmega328p
    }

    pub fn read_data(&mut self, addr: u16) -> u8 {
        if addr >= 32
            && let Some((addr, read_hook)) = self.read_hooks.remove_entry(&addr)
        {
            let result = read_hook(self, addr);
            self.read_hooks.insert(addr, read_hook);
            return result;
        }
        self.cpu.get_data(addr)
    }

    pub fn write_data(&mut self, addr: u16, data: u8) {
        self.write_data_with_mask(addr, data, 0xff);
    }

    pub fn write_data_with_mask(&mut self, addr: u16, data: u8, mask: u8) {
        if let Some((addr, write_hook)) = self.write_hooks.remove_entry(&addr) {
            let cpu_data = self.cpu.get_data(addr);
            let result = write_hook(self, data, cpu_data, addr, mask);
            self.write_hooks.insert(addr, write_hook);
            if result {
                return;
            }
        }
        self.cpu.set_data(addr, data);
    }

    pub fn step(&mut self, i2c_bus: Option<&mut I2CBus>) {
        avr_instruction(self);
        self.tick(i2c_bus);
    }

    pub fn tick(&mut self, i2c_bus: Option<&mut I2CBus>) {
        if let Some(event) = self.cpu.next_clock_event.take() {
            // println!(
            //     "event cycles: {}, cpu cycles: {}",
            //     event.cycles, self.cycles
            // );
            if event.cycles <= self.cpu.cycles {
                self.cpu.next_clock_event = event.next;
                (event.callback)(self, i2c_bus, true, false);
            } else {
                self.cpu.next_clock_event = Some(event);
            }
        }

        let next_interrupt = self.cpu.next_interrupt;
        if self.cpu.interrupts_enabled() && next_interrupt >= 0 {
            assert!(self.cpu.pending_interrupts[next_interrupt as usize].is_some());
            let interrupt = self.cpu.pending_interrupts[next_interrupt as usize].unwrap();
            // println!("interrupt: {}", next_interrupt);
            avr_interrupt(&mut self.cpu, interrupt.address);
            self.cpu.clear_interrupt(&interrupt, true);
        }
    }
}

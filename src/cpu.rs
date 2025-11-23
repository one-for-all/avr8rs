use std::collections::HashMap;

use crate::{
    clock::{AVRClockEventCallback, AVRClockEventEntry, AVRClockEventType},
    interrupt::{AVRInterruptConfig, MAX_INTERRUPTS, avr_interrupt},
    ternary,
    timer::{AVRTimer, OCRUpdateMode, TIMER_0_CONFIG},
    usart::{AVRUSART, USART0_CONFIG},
};

const SRAM_BYTES: usize = 8192;
const REGISTER_SPACE: usize = 0x100;
const SREG: usize = 95;

pub struct CPU {
    pub data: Vec<u8>,
    pub prog_mem: Vec<u16>,
    pub prog_bytes: Vec<u8>,
    pub pc: u32,     // program counter
    pub cycles: u32, // clock cycle counter

    pub pending_interrupts: [Option<AVRInterruptConfig>; MAX_INTERRUPTS], // TODO: optimize this data structure for space
    pub next_clock_event: Option<Box<AVRClockEventEntry>>,

    pub pc_22_bits: bool, // Whether the program counter (PC) can address 22 bits (the default is 16)

    pub timer0: AVRTimer,

    pub usart: AVRUSART,
    pub next_interrupt: i16,
    max_interrupt: i16,
}

impl CPU {
    pub fn new(prog_bytes: Vec<u8>, freq_hz: usize) -> Self {
        // convert to Vec<u16>
        let prog_mem = prog_bytes
            .chunks(2)
            .map(|chunk| {
                let lo = chunk[0] as u16;
                let hi = chunk[1] as u16;
                (hi << 8) | lo // little endian
            })
            .collect();
        let pc_22_bits = prog_bytes.len() > 0x20000;

        let timer0 = AVRTimer::new(TIMER_0_CONFIG);

        let usart = AVRUSART::new(USART0_CONFIG, freq_hz);

        let mut cpu = Self {
            data: vec![0; SRAM_BYTES + REGISTER_SPACE],
            prog_mem,
            prog_bytes,
            pc: 0,
            cycles: 0,
            pending_interrupts: [None; MAX_INTERRUPTS],
            next_clock_event: None,
            pc_22_bits,
            timer0,
            usart,
            next_interrupt: -1,
            max_interrupt: 0,
        };

        cpu.reset();

        cpu
    }

    fn reset(&mut self) {
        self.set_sp((self.data.len() - 1) as u16);
        self.pc = 0;
        self.pending_interrupts = [None; MAX_INTERRUPTS];
        self.next_interrupt = -1;
        self.next_clock_event = None;
    }

    pub fn set_sp(&mut self, data: u16) {
        self.set_data_u16(93, data);
    }

    pub fn get_data(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn set_data(&mut self, addr: u16, data: u8) {
        self.data[addr as usize] = data
    }

    /// get u16 from consecutive two u8, w/ little-endian order
    pub fn get_data_u16(&self, addr: u16) -> u16 {
        let bytes: [u8; 2] = [self.get_data(addr), self.get_data(addr + 1)];
        u16::from_le_bytes(bytes)
    }

    pub fn set_data_u16(&mut self, addr: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.set_data(addr, bytes[0]);
        self.set_data(addr + 1, bytes[1]);
    }

    pub fn add_clock_event(
        &mut self,
        callback: AVRClockEventCallback,
        cycles: u32,
        event_type: AVRClockEventType,
    ) {
        // println!("add clock event, cycles: {}", cycles);
        let cycles = self.cycles + cycles.max(1);
        let mut entry = AVRClockEventEntry {
            cycles,
            callback,
            event_type,
            next: None,
        };
        if self
            .next_clock_event
            .as_ref()
            .is_none_or(|x| x.cycles >= cycles)
        {
            entry.next = self.next_clock_event.take();
            self.next_clock_event = Some(Box::new(entry));
        } else {
            let mut last_item = &mut self.next_clock_event;
            loop {
                assert!(last_item.is_some());
                let clock_event = last_item.as_mut().unwrap();
                assert!(clock_event.cycles < cycles);
                if clock_event.next.as_ref().is_some_and(|x| x.cycles < cycles) {
                    last_item = &mut clock_event.next;
                    continue;
                }
                // last_item is the last one whose cycles < cycles
                entry.next = clock_event.next.take();
                clock_event.next = Some(Box::new(entry));
                break;
            }
        }

        if let Some(x) = &self.next_clock_event {
            // println!("next clock event, cycles: {}", x.cycles);
        }
    }

    pub fn clear_clock_event(&mut self, event_type: AVRClockEventType) -> bool {
        if self.next_clock_event.is_none() {
            return false;
        }

        let mut ret_value = false; // whether any event type match
        while self
            .next_clock_event
            .as_ref()
            .is_some_and(|x| matches!(&x.event_type, event_type))
        {
            let event = self.next_clock_event.take();
            self.next_clock_event = event.unwrap().next;
            ret_value = true;
        }

        let mut last_item = &mut self.next_clock_event;
        if last_item.is_none() {
            return ret_value;
        }
        let mut clock_event = last_item.as_mut().unwrap();
        loop {
            assert!(!matches!(&clock_event.event_type, event_type));
            if clock_event.next.is_none() {
                break;
            }
            if clock_event
                .next
                .as_ref()
                .is_some_and(|x| matches!(&x.event_type, event_type))
            {
                let next = clock_event.next.take();
                clock_event.next = next.unwrap().next;
                ret_value = true;
            } else {
                last_item = &mut clock_event.next;
                clock_event = last_item.as_mut().unwrap();
            }
        }
        ret_value
    }

    pub fn update_clock_event(
        &mut self,
        callback: AVRClockEventCallback,
        event_type: AVRClockEventType,
        cycles: u32,
    ) -> bool {
        if self.clear_clock_event(event_type.clone()) {
            self.add_clock_event(callback, cycles, event_type);
            return true;
        }
        false
    }

    pub fn timer0_tccrb(&self) -> u8 {
        self.get_data(self.timer0.config.TCCRB as u16)
    }

    pub fn timer0_cs(&self) -> u8 {
        self.timer0_tccrb() & 0x7
    }

    pub fn set_interrupt_flag(&mut self, interrupt: AVRInterruptConfig) {
        let flag_register = interrupt.flag_register;
        let flag_mask = interrupt.flag_mask;
        self.data[flag_register as usize] |= flag_mask;
        // println!("set interrupt flag");

        // println!(
        //     "interrupt enable: {:#b}",
        //     self.get_data(interrupt.enable_register)
        // );
        if self.get_data(interrupt.enable_register) & interrupt.enable_mask != 0 {
            // println!("queue interrupt");
            self.queue_interrupt(interrupt);
        }
    }

    pub fn update_interrupt_enable(&mut self, interrupt: AVRInterruptConfig, register_value: u8) {
        if register_value & interrupt.enable_mask != 0 {
            let bit_set = self.get_data(interrupt.flag_register) & interrupt.flag_mask;
            if bit_set != 0 {
                self.queue_interrupt(interrupt);
            }
        } else {
            self.clear_interrupt(&interrupt, false);
        }
    }

    pub fn queue_interrupt(&mut self, interrupt: AVRInterruptConfig) {
        let address = interrupt.address;
        self.pending_interrupts[address as usize] = Some(interrupt);
        let address = address as i16;
        if self.next_interrupt == -1 || self.next_interrupt > address {
            self.next_interrupt = address;
        }
        if address > self.max_interrupt {
            self.max_interrupt = address;
        }
    }

    pub fn clear_interrupt(&mut self, interrupt: &AVRInterruptConfig, clear_flag: bool) {
        let address = interrupt.address;
        if clear_flag {
            self.data[interrupt.flag_register as usize] &= !interrupt.flag_mask;
        }
        let pending_interrupts = &mut self.pending_interrupts;
        if pending_interrupts[address as usize].is_none() {
            return;
        }
        pending_interrupts[address as usize] = None;
        if self.next_interrupt == address as i16 {
            self.next_interrupt = -1;
            for i in address + 1..=self.max_interrupt as u8 {
                if pending_interrupts[i as usize].is_some() {
                    self.next_interrupt = i as i16;
                    break;
                }
            }
        }
    }

    pub fn sreg(&self) -> u8 {
        self.data[SREG]
    }

    pub fn set_sreg(&mut self, data: u8) {
        self.data[SREG] = data;
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.sreg() & 0x80 != 0
    }
}

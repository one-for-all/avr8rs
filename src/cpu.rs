use std::collections::HashMap;

use crate::{
    clock::{AVRClockEventCallback, AVRClockEventEntry, AVRClockEventType},
    interrupt::{AVRInterruptConfig, MAX_INTERRUPTS, avr_interrupt},
    port::{AVRIOPort, PORTB_CONFIG, PORTC_CONFIG, PORTD_CONFIG, PinState},
    ternary,
    timer::{AVRTimer, OCRUpdateMode, TIMER_0_CONFIG},
    usart::{AVRUSART, UCSRB_TXEN, USART0_CONFIG},
};

const SRAM_BYTES: usize = 8192;
const REGISTER_SPACE: usize = 0x100;

type CPUMemoryReadHook = Box<dyn Fn(&mut CPU, u16) -> u8>;
type CPUMemoryHook = Box<dyn Fn(&mut CPU, u8, u8, u16, u8) -> bool>;

pub struct CPU {
    pub data: Vec<u8>,
    pub prog_mem: Vec<u16>,
    pub prog_bytes: Vec<u8>,
    pub pc: u32,     // program counter
    pub cycles: u32, // clock cycle counter

    read_hooks: HashMap<u16, CPUMemoryReadHook>,
    write_hooks: HashMap<u16, CPUMemoryHook>,

    pub pending_interrupts: [Option<AVRInterruptConfig>; MAX_INTERRUPTS], // TODO: optimize this data structure for space
    pub next_clock_event: Option<Box<AVRClockEventEntry>>,

    pub pc_22_bits: bool, // Whether the program counter (PC) can address 22 bits (the default is 16)

    pub ports: HashMap<String, AVRIOPort>,

    pub timer0: AVRTimer,

    pub usart: AVRUSART,

    pub next_interrupt: i16,
    max_interrupt: i16,
}

impl CPU {
    pub fn new(prog_bytes: Vec<u8>) -> Self {
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

        let port_keys = ["B", "C", "D"];
        let ports = HashMap::from([
            (port_keys[0].to_string(), AVRIOPort::new(PORTB_CONFIG)),
            (port_keys[1].to_string(), AVRIOPort::new(PORTC_CONFIG)),
            (port_keys[2].to_string(), AVRIOPort::new(PORTD_CONFIG)),
        ]);

        let usart = AVRUSART::new(USART0_CONFIG);

        let mut cpu = Self {
            data: vec![0; SRAM_BYTES + REGISTER_SPACE],
            prog_mem,
            prog_bytes,
            pc: 0,
            cycles: 0,
            read_hooks: HashMap::new(),
            write_hooks: HashMap::new(),
            pending_interrupts: [None; MAX_INTERRUPTS],
            next_clock_event: None,
            pc_22_bits,
            ports,
            timer0,
            usart,
            next_interrupt: -1,
            max_interrupt: 0,
        };

        // Timer0 setup
        cpu.read_hooks.insert(
            cpu.timer0.config.TCNT as u16,
            Box::new(|cpu, addr| {
                cpu.count(false, false);
                let data = (cpu.timer0.tcnt & 0xff) as u8;
                cpu.set_data(addr, data);
                data
            }),
        );

        cpu.write_hooks.insert(
            cpu.timer0.config.OCRA as u16,
            Box::new(|cpu, value, _, _, _| {
                cpu.timer0.next_ocra = ((cpu.timer0.high_byte_temp as u16) << 8) | value as u16;
                if matches!(cpu.timer0.ocr_update_mode, OCRUpdateMode::Immediate) {
                    cpu.timer0.ocra = cpu.timer0.next_ocra;
                }
                false
            }),
        );

        cpu.write_hooks.insert(
            cpu.timer0.config.TCCRA as u16,
            Box::new(|cpu, value, _, _, _| {
                cpu.set_data(cpu.timer0.config.TCCRA as u16, value);
                // TODO: update wgm config
                true
            }),
        );

        cpu.write_hooks.insert(
            cpu.timer0.config.TCCRB as u16,
            Box::new(|cpu, value, _, _, _| {
                // TODO: check force compare
                cpu.set_data(cpu.timer0.config.TCCRB as u16, value);
                cpu.timer0.update_divider = true;
                cpu.clear_clock_event(AVRClockEventType::Count);
                cpu.add_clock_event(Box::new(CPU::count), 0, AVRClockEventType::Count);
                // TODO: update wgm config
                true
            }),
        );

        cpu.write_hooks.insert(
            cpu.timer0.config.TIFR as u16,
            Box::new(|cpu, value, _, _, _| {
                println!("TIFR hook");
                cpu.set_data(cpu.timer0.config.TIFR as u16, value);
                // TODO: clear interrupt by flag
                true
            }),
        );

        cpu.write_hooks.insert(
            cpu.timer0.config.TIMSK as u16,
            Box::new(|cpu, value, _, _, _| {
                println!("TIMSK hook");
                cpu.update_interrupt_enable(cpu.timer0.ovf, value);
                false
            }),
        );

        for key in port_keys {
            let port = &cpu.ports[&key.to_string()];
            cpu.write_hooks.insert(
                port.config.DDR as u16,
                Box::new(|cpu, ddr_mask, _, _, _| {
                    let config = &cpu.ports[&key.to_string()].config;
                    let port = config.PORT;
                    let ddr = config.DDR;
                    let pin = config.PIN;

                    let port_value = cpu.data[port as usize];
                    cpu.data[ddr as usize] = ddr_mask;
                    let port = cpu.ports.get_mut(&key.to_string()).unwrap();
                    port.write_gpio(port_value, ddr_mask);
                    let new_pin = port.update_pin_register(ddr_mask);
                    cpu.data[pin as usize] = new_pin;

                    true
                }),
            );

            cpu.write_hooks.insert(
                port.config.PORT as u16,
                Box::new(|cpu, port_value, _, _, _| {
                    let config = &cpu.ports[&key.to_string()].config;
                    let port = config.PORT;
                    let ddr = config.DDR;

                    let ddr_mask = cpu.data[ddr as usize];
                    cpu.data[port as usize] = port_value;
                    let port = cpu.ports.get_mut(&key.to_string()).unwrap();
                    port.write_gpio(port_value, ddr_mask);
                    port.update_pin_register(ddr_mask);
                    true
                }),
            );
        }

        cpu.write_hooks.insert(
            cpu.usart.config.UCSRB as u16,
            Box::new(|cpu, value, old_value, _, _| {
                // cpu.update_interrupt_enable(cpu.usart.rxc, value);
                cpu.update_interrupt_enable(cpu.usart.udre, value);
                cpu.update_interrupt_enable(cpu.usart.txc, value);
                // if value & UCSRB_RXEN && old_value & UCSRB_RXEN {
                //     cpu.clear_interrupt(cpu.usart.rxc, true);
                // }
                if value & UCSRB_TXEN != 0 && old_value & UCSRB_TXEN == 0 {
                    // Enabling the transmission - mark UDR as empty
                    cpu.set_interrupt_flag(cpu.usart.udre);
                }
                cpu.data[cpu.usart.config.UCSRB as usize] = value;
                // cpu.onConfigurationChange

                true
            }),
        );

        cpu.write_hooks.insert(
            cpu.usart.config.UDR as u16,
            Box::new(|cpu, value, _, _, _| {
                println!("usart: {}", str::from_utf8(&[value]).unwrap());

                cpu.add_clock_event(
                    Box::new(|cpu: &mut CPU, _, _| {
                        cpu.set_interrupt_flag(cpu.usart.udre);
                        cpu.set_interrupt_flag(cpu.usart.txc);
                    }),
                    cpu.cycles_per_char(),
                    AVRClockEventType::USART,
                );
                let txc = cpu.usart.txc.clone();
                cpu.clear_interrupt(&txc, true);
                let urde = cpu.usart.udre.clone();
                cpu.clear_interrupt(&urde, true);

                false
            }),
        );

        cpu
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

    pub fn read_data(&mut self, addr: u16) -> u8 {
        if addr >= 32
            && let Some((addr, read_hook)) = self.read_hooks.remove_entry(&addr)
        {
            let result = read_hook(self, addr);
            self.read_hooks.insert(addr, read_hook);
            return result;
        }
        self.get_data(addr)
    }

    pub fn write_data(&mut self, addr: u16, data: u8, mask: u8) {
        if let Some((addr, write_hook)) = self.write_hooks.remove_entry(&addr) {
            let result = write_hook(self, data, self.get_data(addr), addr, mask);
            self.write_hooks.insert(addr, write_hook);
            if result {
                return;
            }
        }
        self.set_data(addr, data);
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

    pub fn clear_clock_event(&mut self, event_type: AVRClockEventType) {
        if self.next_clock_event.is_none() {
            return;
        }
        while self
            .next_clock_event
            .as_ref()
            .is_some_and(|x| matches!(&x.event_type, event_type))
        {
            let event = self.next_clock_event.take();
            self.next_clock_event = event.unwrap().next;
        }

        let mut last_item = &mut self.next_clock_event;
        if last_item.is_none() {
            return;
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
            } else {
                last_item = &mut clock_event.next;
                clock_event = last_item.as_mut().unwrap();
            }
        }
    }

    /// Get the state of a given GPIO pin
    ///
    /// @param index Pin index to return from 0 to 7
    /// @returns PinState.Low or PinState.High if the pin is set to output, PinState.Input if the pin is set
    /// to input, and PinState.InputPullUp if the pin is set to input and the internal pull-up resistor has
    /// been enabled.
    pub fn pin_state(&self, port_key: &str, index: u8) -> PinState {
        let p = &self.ports[port_key];
        let ddr = self.data[p.config.DDR as usize];
        let port = self.data[p.config.PORT as usize];
        let bit_mask: u8 = 1 << index;
        let open_state = ternary!(port & bit_mask, PinState::InputPullUp, PinState::Input);
        if ddr & bit_mask != 0 {
            let high_value = ternary!(p.open_collector & bit_mask, open_state, PinState::High);
            ternary!(p.last_value & bit_mask, high_value, PinState::Low)
        } else {
            open_state
        }
    }

    pub fn timer0_tccrb(&self) -> u8 {
        self.get_data(self.timer0.config.TCCRB as u16)
    }

    pub fn timer0_cs(&self) -> u8 {
        self.timer0_tccrb() & 0x7
    }

    pub fn count(&mut self, reschedule: bool, external: bool) {
        // println!("count");
        // println!("cpu cycles: {}", self.cycles);
        let divider = self.timer0.divider;
        let last_cycle = self.timer0.last_cycle;
        let cycles = self.cycles;
        let delta = (cycles - last_cycle) as u16;
        if (divider != 0 && delta >= divider) || external {
            let counter_delta = if external { 1 } else { delta / divider };
            self.timer0.last_cycle += counter_delta as u32 * divider as u32;
            let val = self.timer0.tcnt;
            // timer mode, assume is normal
            let TOP = self.timer0.top();
            let new_val = (val + counter_delta) % (TOP + 1);
            // println!("val: {}, new val: {}", val, new_val);
            let overflow = val + counter_delta > TOP;
            // A CPU write overrides all counter clear or count operations
            if true {
                // if !tcntUpdated
                self.timer0.tcnt = new_val;
                // if !phase pwm
                // self.timer0.timerUpdated(new_val, val);
            }

            // OCRUpdateMode.Bottom only occurs in Phase Correct modes, handled by phasePwmCount().
            // Thus we only handle TOVUpdateMode.Top or TOVUpdateMode.Max here.
            // println!("overflow: {}", overflow);
            // if overflow {
            //     println!("overflow");
            // }
            if overflow && TOP == self.timer0.max {
                let ovf = self.timer0.ovf;
                self.set_interrupt_flag(ovf);
            }
        }
        // TODO: handle if tcntUpdated
        if self.timer0.update_divider {
            let cs = self.timer0_cs();
            let timer01_dividers = [0, 1, 8, 64, 256, 1024, 0, 0];
            let new_divider = timer01_dividers[cs as usize];

            self.timer0.last_cycle = ternary!(new_divider, self.cycles, 0);
            self.timer0.update_divider = false;
            self.timer0.divider = new_divider;
            if new_divider != 0 {
                self.add_clock_event(
                    Box::new(CPU::count),
                    self.timer0.last_cycle + new_divider as u32 - self.cycles,
                    AVRClockEventType::Count,
                );
            }
            return;
        }
        if reschedule && divider != 0 {
            self.add_clock_event(
                Box::new(CPU::count),
                self.timer0.last_cycle + divider as u32 - self.cycles,
                AVRClockEventType::Count,
            );
        }
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
        self.data[95]
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.sreg() & 0x80 != 0
    }

    pub fn tick(&mut self) {
        if let Some(event) = self.next_clock_event.take() {
            // println!(
            //     "event cycles: {}, cpu cycles: {}",
            //     event.cycles, self.cycles
            // );
            if event.cycles <= self.cycles {
                self.next_clock_event = event.next;
                (event.callback)(self, true, false);
            } else {
                self.next_clock_event = Some(event);
            }
        }

        let next_interrupt = self.next_interrupt;
        if self.interrupts_enabled() && next_interrupt >= 0 {
            assert!(self.pending_interrupts[next_interrupt as usize].is_some());
            let interrupt = self.pending_interrupts[next_interrupt as usize].unwrap();
            // println!("interrupt: {}", next_interrupt);
            avr_interrupt(self, interrupt.address);
            self.clear_interrupt(&interrupt, true);
        }
    }
}

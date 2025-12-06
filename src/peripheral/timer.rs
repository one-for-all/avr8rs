use std::collections::HashMap;

use crate::{
    atmega328p::{ATMega328P, PeripheralMemoryReadHook, PeripheralMemoryWriteHook},
    clock::AVRClockEventType,
    cpu::CPU,
    interrupt::AVRInterruptConfig,
    peripheral::i2c::bus::I2CBus,
    ternary,
};

const CS00: u8 = 1 << 0; // Clock Select 0
const CS01: u8 = 1 << 1; // Clock Select 1

#[allow(non_snake_case)]
pub struct AVRTimerConfig {
    // Interrupt vectors
    pub ovf_interrupt: u8,

    // Register addresses
    pub TIFR: u8,
    pub TCNT: u8,
    pub OCRA: u8,
    pub TCCRA: u8,
    pub TCCRB: u8,
    pub TIMSK: u8,

    // TIFR bits
    pub TOV: u8,

    // TIMSK bits
    pub TOIE: u8,
}

pub enum OCRUpdateMode {
    Immediate,
}

pub struct AVRTimer {
    pub max: u16,

    pub config: AVRTimerConfig,
    pub last_cycle: u32,

    pub ocra: u16,
    pub next_ocra: u16,

    pub ocr_update_mode: OCRUpdateMode,

    pub tcnt: u16,
    pub tcnt_next: u16,
    pub tcnt_updated: bool,

    pub update_divider: bool,
    pub divider: u16,

    pub high_byte_temp: u8, // This is the temporary register used to access 16-bit registers (section 16.3 of the datasheet)

    pub ovf: AVRInterruptConfig,
}

impl AVRTimer {
    pub fn new(config: AVRTimerConfig) -> Self {
        let ovf = AVRInterruptConfig {
            address: config.ovf_interrupt,
            enable_register: config.TIMSK as u16,
            enable_mask: config.TOIE,
            flag_register: config.TIFR as u16,
            flag_mask: config.TOV,
            inverse_flag: false,
        };
        AVRTimer {
            max: 0xff, // config.bits === 16 ? 0xffff : 0xff;
            config,
            last_cycle: 0,
            ocra: 0,
            next_ocra: 0,
            ocr_update_mode: OCRUpdateMode::Immediate,
            tcnt: 0,
            tcnt_next: 0,
            tcnt_updated: false,
            update_divider: false,
            divider: 0,
            high_byte_temp: 0,
            ovf,
        }
    }

    /// TOP value of counter
    pub fn top(&self) -> u16 {
        // for now, assume to be 0xff
        0xff
    }

    #[allow(non_snake_case)]
    pub fn add_TCNT_read_hook(&self, read_hooks: &mut HashMap<u16, PeripheralMemoryReadHook>) {
        read_hooks.insert(
            self.config.TCNT as u16,
            Box::new(|atmega, addr| {
                atmega.count(None, false, false);
                let data = (atmega.timer0.tcnt & 0xff) as u8;
                atmega.cpu.set_data(addr, data);
                data
            }),
        );
    }

    #[allow(non_snake_case)]
    pub fn add_TCNT_write_hook(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.TCNT as u16,
            Box::new(|atmega, value, _, _, _| {
                atmega.timer0.tcnt_next = value as u16 | (atmega.timer0.high_byte_temp as u16) << 8;
                // atmega.cpu.timer0.counting_up = true;
                atmega.timer0.tcnt_updated = true;
                atmega.cpu.update_clock_event(
                    Box::new(ATMega328P::count),
                    crate::clock::AVRClockEventType::Count,
                    0,
                );
                // if atmega.cpu.timer0.divider != 0 {
                //     atmega.cpu.timer0.timer_updated
                // }
                false
            }),
        );
    }

    #[allow(non_snake_case)]
    pub fn add_TCCRB_write_hook(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.TCCRB as u16,
            Box::new(|atmega, value, _, _, _| {
                // TODO: check force compare
                atmega
                    .cpu
                    .set_data(atmega.timer0.config.TCCRB as u16, value);
                atmega.timer0.update_divider = true;
                atmega.cpu.clear_clock_event(AVRClockEventType::Count);
                atmega.cpu.add_clock_event(
                    Box::new(ATMega328P::count),
                    0,
                    AVRClockEventType::Count,
                );
                // TODO: update wgm config
                true
            }),
        );
    }

    #[allow(non_snake_case)]
    pub fn add_TIMSK_write_hook(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.TIMSK as u16,
            Box::new(|atmega, value, _, _, _| {
                println!("TIMSK hook");
                atmega.cpu.update_interrupt_enable(atmega.timer0.ovf, value);
                false
            }),
        );
    }

    pub fn tccrb(&self, cpu: &CPU) -> u8 {
        cpu.get_data(self.config.TCCRB as u16)
    }

    pub fn cs(&self, cpu: &CPU) -> u8 {
        self.tccrb(cpu) & 0x7
    }
}

impl ATMega328P {
    pub fn timer0_cs(&self) -> u8 {
        self.timer0.cs(&self.cpu)
    }

    pub fn count(&mut self, _: Option<&mut I2CBus>, reschedule: bool, external: bool) {
        // println!("count");
        // println!("cpu cycles: {}", self.cycles);
        let divider = self.timer0.divider;
        let last_cycle = self.timer0.last_cycle;
        let cycles = self.cpu.cycles;
        let delta = (cycles - last_cycle) as u16;
        // println!("delta: {} divider: {}", delta, divider);
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
                self.cpu.set_interrupt_flag(self.timer0.ovf);
            }
        }
        if self.timer0.tcnt_updated {
            self.timer0.tcnt = self.timer0.tcnt_next;
            self.timer0.tcnt_updated = false;
            // TODO: OCR updates
        }
        // TODO: handle if tcntUpdated
        if self.timer0.update_divider {
            let cs = self.timer0_cs();
            let timer01_dividers = [0, 1, 8, 64, 256, 1024, 0, 0];
            let new_divider = timer01_dividers[cs as usize];

            self.timer0.last_cycle = ternary!(new_divider, self.cpu.cycles, 0);
            self.timer0.update_divider = false;
            self.timer0.divider = new_divider;
            if new_divider != 0 {
                self.cpu.add_clock_event(
                    Box::new(Self::count),
                    self.timer0.last_cycle + new_divider as u32 - self.cpu.cycles,
                    AVRClockEventType::Count,
                );
            }
            return;
        }
        if reschedule && divider != 0 {
            self.cpu.add_clock_event(
                Box::new(ATMega328P::count),
                self.timer0.last_cycle + divider as u32 - self.cpu.cycles,
                AVRClockEventType::Count,
            );
        }
    }
}

pub const TIMER_0_CONFIG: AVRTimerConfig = AVRTimerConfig {
    ovf_interrupt: 0x20,

    TIFR: 0x35,
    TCNT: 0x46,
    OCRA: 0x47,
    TCCRA: 0x44,
    TCCRB: 0x45,
    TIMSK: 0x6e,

    TOV: 1,

    TOIE: 1,
};

#[cfg(test)]
mod timer_tests {
    use crate::{
        atmega328p::{ATMega328P, DEFAULT_FREQ},
        peripheral::timer::{CS00, CS01, TIMER_0_CONFIG},
    };

    #[test]
    fn timer_inc_when_tick_with_prescaler_1() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS00); // set prescaler to 1
        atmega.cpu.cycles = 1;
        atmega.tick(None); // first tick updates divider
        atmega.cpu.cycles = 1 + 1; // emulate cycles increment by instruction
        atmega.tick(None); // increment count

        // Assert
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 1);
    }

    #[test]
    fn timer_inc_every_64_ticks() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS01 | CS00); // set prescaler to 64
        atmega.cpu.cycles = 1;
        atmega.tick(None); // first tick updates divider
        atmega.cpu.cycles = 1 + 64; // emulate cycles increment by instruction
        atmega.tick(None); // increment count

        // Assert
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 1);
    }

    #[test]
    fn timer_no_inc_if_prescaler_0() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, 0); // set prescaler to 64
        atmega.cpu.cycles = 1;
        atmega.tick(None); // first tick updates divider
        atmega.cpu.cycles = 1_000; // set to a high cycles count
        atmega.tick(None);

        // Assert
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 0);
    }

    #[test]
    #[allow(non_snake_case)]
    fn set_TOV_if_overflow() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let top = 0xff;

        // Act & Assert
        atmega.write_data(TIMER_0_CONFIG.TCNT as u16, top);
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS00); // Set prescaler to 1
        atmega.cpu.cycles = 1;
        atmega.tick(None);
        // count value set, and overflow flag not yet
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, top);
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            0
        );

        atmega.cpu.cycles += 1;
        atmega.tick(None);
        // count wraps, and overflow flag set
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 0);
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            TIMER_0_CONFIG.TOV
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn set_TOV_even_if_skip_top() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let near_top = 0xfe;

        // Act & Assert
        atmega.write_data(TIMER_0_CONFIG.TCNT as u16, near_top);
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS00); // Set prescaler to 1
        atmega.cpu.cycles = 1;
        atmega.tick(None);
        // count value set, and overflow flag not yet
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, near_top);
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            0
        );

        atmega.cpu.cycles += 4;
        atmega.tick(None);
        // count wraps, and overflow flag set
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 2);
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            TIMER_0_CONFIG.TOV
        );
    }

    #[test]
    fn overflow_interrupt() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let top = 0xff;

        // Act
        atmega.write_data(TIMER_0_CONFIG.TCNT as u16, top);
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS00);
        atmega.cpu.cycles = 1;
        atmega.tick(None);
        atmega.write_data(TIMER_0_CONFIG.TIMSK as u16, TIMER_0_CONFIG.TOIE); // enable overflow interrupt
        atmega.cpu.set_sreg(1 << 7); // enable global interrupt
        atmega.cpu.cycles = 2;
        atmega.tick(None);

        // Assert
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 2); // 0xff + 1 + 2, where 2 is the 2 cycles for the interrupt
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            0
        ); // overflow flag cleared
        assert_eq!(atmega.cpu.pc, TIMER_0_CONFIG.ovf_interrupt as u32); // overflow interrupt handler address
        assert_eq!(atmega.cpu.cycles, 4);
    }

    #[test]
    fn no_overflow_interrupt_if_global_disabled() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let top = 0xff;

        // Act
        atmega.write_data(TIMER_0_CONFIG.TCNT as u16, top);
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS00);
        atmega.cpu.cycles = 1;
        atmega.tick(None);
        atmega.write_data(TIMER_0_CONFIG.TIMSK as u16, TIMER_0_CONFIG.TOIE); // enable overflow interrupt
        atmega.cpu.set_sreg(0); // disable global interrupt
        atmega.cpu.cycles = 2;
        atmega.tick(None);

        // Assert
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 0); // 0xff + 1
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            TIMER_0_CONFIG.TOV
        ); // overflow flag set
        assert_eq!(atmega.cpu.pc, 0); // unchanged
        assert_eq!(atmega.cpu.cycles, 2); // unchanged
    }

    #[test]
    #[allow(non_snake_case)]
    fn no_overflow_interrupt_if_TOIE_clear() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let top = 0xff;

        // Act
        atmega.write_data(TIMER_0_CONFIG.TCNT as u16, top);
        atmega.write_data(TIMER_0_CONFIG.TCCRB as u16, CS00);
        atmega.cpu.cycles = 1;
        atmega.tick(None);
        atmega.write_data(TIMER_0_CONFIG.TIMSK as u16, 0); // enable overflow interrupt
        atmega.cpu.set_sreg(1 << 7); // enable global interrupt
        atmega.cpu.cycles = 2;
        atmega.tick(None);

        // Assert
        let count = atmega.read_data(TIMER_0_CONFIG.TCNT as u16);
        assert_eq!(count, 0); // 0xff + 1
        assert_eq!(
            atmega.cpu.data[TIMER_0_CONFIG.TIFR as usize] & TIMER_0_CONFIG.TOV,
            TIMER_0_CONFIG.TOV
        ); // overflow flag set
        assert_eq!(atmega.cpu.pc, 0); // unchanged
        assert_eq!(atmega.cpu.cycles, 2); // unchanged
    }
}

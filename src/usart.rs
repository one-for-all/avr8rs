use std::collections::HashMap;

use crate::{
    atmega328p::{ATMega328P, PeripheralMemoryWriteHook},
    cpu::CPU,
    interrupt::AVRInterruptConfig,
    ternary,
};

// Register consts
pub const UCSRA_TXC: u8 = 0x40; // USART Transmit Complete, 1 << 6
const UCSRA_UDRE: u8 = 0x20; // USART Data Register Empty
pub const UCSRA_U2X: u8 = 0x2; // Double the USART Transmission Speed
const UCSRB_TXCIE: u8 = 0x40; // TX Complete Interrupt Enable
const UCSRB_UDRIE: u8 = 0x20; // USART Data Register Empty Interrupt Enable
pub const UCSRB_TXEN: u8 = 0x8; // Transmitter Enable
const UCSRB_UCSZ2: u8 = 1 << 2; // Character Size 2
const UCSRC_UPM1: u8 = 0x20; // Parity Mode 1
const UCSRC_USBS: u8 = 0x8; // Stop Bit Select
const UCSRC_UCSZ1: u8 = 0x4; // Character Size 1
const UCSRC_UCSZ0: u8 = 0x2; // Character Size 0

#[allow(non_snake_case)]
pub struct USARTConfig {
    pub data_register_empty_interrupt: u8, // interrupt hander address on data register empty
    pub tx_complete_interrupt: u8, // interrupt hander address on transmit complete for a frame

    pub UCSRA: u8, // register A address
    pub UCSRB: u8, // B address
    pub UCSRC: u8, // C address
    pub UBRRL: u8, // Baud rate (low bits) register address
    pub UBRRH: u8, // Baud rate (high bits) register address
    pub UDR: u8,   // Data register address
}

pub const USART0_CONFIG: USARTConfig = USARTConfig {
    data_register_empty_interrupt: 0x26,
    tx_complete_interrupt: 0x28,
    UCSRA: 0xc0,
    UCSRB: 0xc1,
    UCSRC: 0xc2,
    UBRRL: 0xc4,
    UBRRH: 0xc5,
    UDR: 0xc6,
};

pub struct AVRUSART {
    pub config: USARTConfig,
    pub freq_hz: usize, // clock frequency

    pub udre: AVRInterruptConfig,
    pub txc: AVRInterruptConfig,
}

impl AVRUSART {
    pub fn new(config: USARTConfig, freq_hz: usize) -> Self {
        let urde = AVRInterruptConfig {
            address: config.data_register_empty_interrupt,
            flag_register: config.UCSRA as u16,
            flag_mask: UCSRA_UDRE,
            enable_register: config.UCSRB as u16,
            enable_mask: UCSRB_UDRIE,
        };
        let txc = AVRInterruptConfig {
            address: config.tx_complete_interrupt,
            flag_register: config.UCSRA as u16,
            flag_mask: UCSRA_TXC,
            enable_register: config.UCSRB as u16,
            enable_mask: UCSRB_TXCIE,
        };
        Self {
            config,
            freq_hz,
            udre: urde,
            txc,
        }
    }

    /// USART Control and Status Register handler
    pub fn add_ucsrb_handler(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.UCSRB as u16,
            Box::new(|atmega, value, old_value, _, _| {
                // cpu.update_interrupt_enable(cpu.usart.rxc, value);
                atmega.cpu.update_interrupt_enable(atmega.usart.udre, value);
                atmega.cpu.update_interrupt_enable(atmega.usart.txc, value);
                // if value & UCSRB_RXEN && old_value & UCSRB_RXEN {
                //     cpu.clear_interrupt(cpu.usart.rxc, true);
                // }
                if value & UCSRB_TXEN != 0 && old_value & UCSRB_TXEN == 0 {
                    // Enabling the transmission - mark UDR as empty
                    atmega.cpu.set_interrupt_flag(atmega.usart.udre);
                }
                atmega.cpu.data[atmega.usart.config.UCSRB as usize] = value;
                // cpu.onConfigurationChange

                true
            }),
        );
    }

    pub fn add_udr_handler(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.UDR as u16,
            Box::new(|atmega, value, _, _, _| {
                println!("usart: {}", str::from_utf8(&[value]).unwrap());

                atmega.cpu.add_clock_event(
                    Box::new(|atmega: &mut ATMega328P, _, _| {
                        atmega.cpu.set_interrupt_flag(atmega.usart.udre);
                        atmega.cpu.set_interrupt_flag(atmega.usart.txc);
                    }),
                    atmega.usart_cycles_per_char(),
                    crate::clock::AVRClockEventType::USART,
                );
                let txc = atmega.usart.txc.clone();
                atmega.cpu.clear_interrupt(&txc, true);
                let urde = atmega.usart.udre.clone();
                atmega.cpu.clear_interrupt(&urde, true);

                false
            }),
        );
    }

    pub fn stop_bits(&self, data: &Vec<u8>) -> usize {
        ternary!(data[self.config.UCSRC as usize] & UCSRC_USBS, 2, 1)
    }

    pub fn bits_per_char(&self, data: &Vec<u8>) -> usize {
        let ucsz: u8 = ((data[self.config.UCSRC as usize] & (UCSRC_UCSZ1 | UCSRC_UCSZ0)) >> 1)
            | (data[self.config.UCSRB as usize] & UCSRB_UCSZ2);
        match ucsz {
            0 => 5,
            1 => 6,
            2 => 7,
            3 => 8,
            7 => 9,
            _ => panic!("invalid bits per char"),
        }
    }

    #[allow(non_snake_case)]
    pub fn UBRR(&self, data: &Vec<u8>) -> usize {
        let UBRRH = self.config.UBRRH;
        let UBRRL = self.config.UBRRL;
        (data[UBRRH as usize] as usize) << 8 | data[UBRRL as usize] as usize
    }

    pub fn multiplier(&self, data: &Vec<u8>) -> usize {
        ternary!(data[self.config.UCSRA as usize] & UCSRA_U2X, 8, 16)
    }

    pub fn parity_enabled(&self, data: &Vec<u8>) -> bool {
        data[self.config.UCSRC as usize] & UCSRC_UPM1 != 0
    }

    pub fn baud_rate(&self, data: &Vec<u8>) -> usize {
        self.freq_hz / (self.multiplier(data) * (1 + self.UBRR(data)))
    }

    pub fn cycles_per_char(&self, data: &Vec<u8>) -> usize {
        let symbols_per_char = 1
            + self.bits_per_char(data)
            + if self.parity_enabled(data) { 1 } else { 0 }
            + self.stop_bits(data);
        (self.UBRR(data) + 1) * self.multiplier(data) * symbols_per_char
    }
}

/// USART settings access methods
impl ATMega328P {
    pub fn usart_stop_bits(&self) -> usize {
        self.usart.stop_bits(&self.cpu.data)
    }

    pub fn usart_bits_per_char(&self) -> usize {
        self.usart.bits_per_char(&self.cpu.data)
    }

    pub fn usart_parity_enabled(&self) -> bool {
        self.usart.parity_enabled(&self.cpu.data)
    }

    pub fn usart_baud_rate(&self) -> usize {
        self.usart.baud_rate(&self.cpu.data)
    }

    pub fn usart_cycles_per_char(&self) -> u32 {
        self.usart.cycles_per_char(&self.cpu.data) as u32
    }
}

#[cfg(test)]
mod usart_tests {
    use crate::{
        atmega328p::{ATMega328P, DEFAULT_FREQ},
        usart::{
            UCSRA_TXC, UCSRA_U2X, UCSRA_UDRE, UCSRB_TXCIE, UCSRB_TXEN, UCSRB_UCSZ2, UCSRB_UDRIE,
            UCSRC_UCSZ0, UCSRC_UCSZ1, UCSRC_USBS, USART0_CONFIG,
        },
    };

    /// TX Complete Interrupt Enable bit
    #[test]
    #[allow(non_snake_case)]
    fn TXCIE_trigger() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let cycles: u32 = 1_000_000;

        // Act
        atmega.write_data(USART0_CONFIG.UCSRB as u16, UCSRB_TXCIE | UCSRB_TXEN);
        atmega.write_data(USART0_CONFIG.UDR as u16, 0x61);
        atmega.cpu.set_sreg(1 << 7);
        atmega.cpu.cycles = cycles;
        atmega.tick();

        // Assert
        assert_eq!(atmega.cpu.pc, USART0_CONFIG.tx_complete_interrupt as u32);
        assert_eq!(atmega.cpu.cycles, cycles + 2);
        assert_eq!(atmega.cpu.data[USART0_CONFIG.UCSRA as usize] & UCSRA_TXC, 0); // bit cleared after handling interrupt
    }

    /// Not trigger TX Complete interrrupt if UDR is not written to
    #[test]
    #[allow(non_snake_case)]
    fn TXCIE_no_trigger_if_no_data() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act
        atmega.write_data(USART0_CONFIG.UCSRB as u16, UCSRB_TXCIE | UCSRB_TXEN);
        atmega.cpu.set_sreg(1 << 7);
        atmega.tick();

        // Assert
        assert_eq!(atmega.cpu.pc, 0);
        assert_eq!(atmega.cpu.cycles, 0);
    }

    /// Not trigger any interrupts if interrupts disabled
    #[test]
    fn interrupts_disabled() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let cycles = 1_000_000;

        // Act
        atmega.write_data(USART0_CONFIG.UCSRB as u16, UCSRB_UDRIE | UCSRB_TXEN);
        atmega.write_data(USART0_CONFIG.UDR as u16, 0x61);
        atmega.cpu.set_sreg(0); // disable interrupts
        atmega.cpu.cycles = cycles;
        atmega.tick();

        // Assert
        assert_eq!(atmega.cpu.pc, 0);
        assert_eq!(atmega.cpu.cycles, cycles);
        assert_eq!(
            atmega.cpu.data[USART0_CONFIG.UCSRA as usize],
            UCSRA_TXC | UCSRA_UDRE
        );
    }

    /// Calculate the correct baud rate from UBRR (USART Baud Rate Register)
    #[test]
    #[allow(non_snake_case)]
    fn UBRR() {
        // Arrange
        let mut atmega = ATMega328P::new("", 11059200);
        atmega.write_data(USART0_CONFIG.UBRRH as u16, 0);
        atmega.write_data(USART0_CONFIG.UBRRL as u16, 5);

        // Act/Assert
        assert_eq!(atmega.usart_baud_rate(), 115200); // 11059200 / (16 * (1 + 5))
    }

    /// Calculate the correct baud rate in double-speed mode
    #[test]
    #[allow(non_snake_case)]
    fn UBRR_double_speed() {
        // Arrange
        let mut atmega = ATMega328P::new("", 16_000_000);
        atmega.write_data(USART0_CONFIG.UBRRH as u16, 3);
        atmega.write_data(USART0_CONFIG.UBRRL as u16, 64);
        atmega.write_data(USART0_CONFIG.UCSRA as u16, UCSRA_U2X);

        // Act/Assert
        assert_eq!(atmega.usart_baud_rate(), 2400); // 16000000 / (8 * (1 + 768 + 64]))
    }

    #[test]
    fn bits_per_char() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act/Assert
        atmega.write_data(USART0_CONFIG.UCSRC as u16, 0);
        assert_eq!(atmega.usart_bits_per_char(), 5);
        atmega.write_data(USART0_CONFIG.UCSRC as u16, UCSRC_UCSZ0);
        assert_eq!(atmega.usart_bits_per_char(), 6);
        atmega.write_data(USART0_CONFIG.UCSRC as u16, UCSRC_UCSZ1);
        assert_eq!(atmega.usart_bits_per_char(), 7);
        atmega.write_data(USART0_CONFIG.UCSRC as u16, UCSRC_UCSZ0 | UCSRC_UCSZ1);
        assert_eq!(atmega.usart_bits_per_char(), 8);
        atmega.write_data(USART0_CONFIG.UCSRC as u16, UCSRC_UCSZ0 | UCSRC_UCSZ1);
        atmega.write_data(USART0_CONFIG.UCSRB as u16, UCSRB_UCSZ2);
        assert_eq!(atmega.usart_bits_per_char(), 9);
    }

    #[test]
    fn stop_bits() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act/Assert
        atmega.write_data(USART0_CONFIG.UCSRC as u16, 0);
        assert_eq!(atmega.usart_stop_bits(), 1);
        atmega.write_data(USART0_CONFIG.UCSRC as u16, UCSRC_USBS);
        assert_eq!(atmega.usart_stop_bits(), 2);
    }
}

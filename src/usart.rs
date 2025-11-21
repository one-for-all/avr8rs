use crate::{cpu::CPU, interrupt::AVRInterruptConfig, ternary};

const UCSRA_TXC: u8 = 0x40; // USART Transmit Complete
const UCSRA_UDRE: u8 = 0x20; // USART Data Register Empty
const UCSRA_U2X: u8 = 0x2; // Double the USART Transmission Speed
const UCSRB_TXCIE: u8 = 0x40; // TX Complete Interrupt Enable
const UCSRB_UDRIE: u8 = 0x20; // USART Data Register Empty Interrupt Enable
pub const UCSRB_TXEN: u8 = 0x8; // Transmitter Enable
const UCSRB_UCSZ2: u8 = 0x4; // Character Size 2
const UCSRC_UPM1: u8 = 0x20; // Parity Mode 1
const UCSRC_USBS: u8 = 0x8; // Stop Bit Select
const UCSRC_UCSZ1: u8 = 0x4; // Character Size 1
const UCSRC_UCSZ0: u8 = 0x2; // Character Size 0

#[allow(non_snake_case)]
pub struct USARTConfig {
    pub data_register_empty_interrupt: u8,
    pub tx_complete_interrupt: u8,

    pub UCSRA: u8,
    pub UCSRB: u8,
    pub UCSRC: u8,
    pub UBRRL: u8,
    pub UBRRH: u8,
    pub UDR: u8,
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

    pub udre: AVRInterruptConfig,
    pub txc: AVRInterruptConfig,
}

impl AVRUSART {
    pub fn new(config: USARTConfig) -> Self {
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
            udre: urde,
            txc,
        }
    }
}

/// USART related functions
impl CPU {
    pub fn cycles_per_char(&self) -> u32 {
        let symbols_per_char =
            1 + self.bits_per_char() + self.stop_bits() + if self.parity_enabled() { 1 } else { 0 };
        ((self.UBRR() + 1) * self.multiplier() * symbols_per_char) as u32
    }

    #[allow(non_snake_case)]
    pub fn UBRR(&self) -> usize {
        let UBRRH = self.usart.config.UBRRH;
        let UBRRL = self.usart.config.UBRRL;
        (self.data[UBRRH as usize] as usize) << 8 | self.data[UBRRL as usize] as usize
    }

    pub fn multiplier(&self) -> usize {
        ternary!(
            self.data[self.usart.config.UCSRA as usize] & UCSRA_U2X,
            8,
            16
        )
    }

    pub fn bits_per_char(&self) -> usize {
        let ucsz: u8 =
            ((self.data[self.usart.config.UCSRC as usize] & (UCSRC_UCSZ1 | UCSRC_UCSZ0)) >> 1)
                | (self.data[self.usart.config.UCSRB as usize] & UCSRB_UCSZ2);
        match ucsz {
            0 => 5,
            1 => 6,
            2 => 7,
            3 => 8,
            7 => 9,
            _ => panic!("invalid bits per char"),
        }
    }

    pub fn stop_bits(&self) -> usize {
        ternary!(
            self.data[self.usart.config.UCSRC as usize] & UCSRC_USBS,
            2,
            1
        )
    }

    pub fn parity_enabled(&self) -> bool {
        self.data[self.usart.config.UCSRC as usize] & UCSRC_UPM1 != 0
    }
}

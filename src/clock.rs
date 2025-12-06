use crate::{atmega328p::ATMega328P, cpu::CPU, peripheral::i2c::bus::I2CBus};

#[derive(Clone)]
pub enum AVRClockEventType {
    Count,
    USART,
    I2C,
    EEPROMFinish,        // TODO(EEPROM): better naming
    EEPROMWriteComplete, // TODO(EEPROM): better naming
}

pub type AVRClockEventCallback = Box<dyn Fn(&mut ATMega328P, Option<&mut I2CBus>, bool, bool)>;

pub struct AVRClockEventEntry {
    pub cycles: u32,
    pub callback: AVRClockEventCallback,
    pub event_type: AVRClockEventType,
    pub next: Option<Box<AVRClockEventEntry>>,
}

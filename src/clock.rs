use crate::{atmega328p::ATMega328P, cpu::CPU};

#[derive(Clone)]
pub enum AVRClockEventType {
    Count,
    USART,
}

pub type AVRClockEventCallback = Box<dyn Fn(&mut ATMega328P, bool, bool)>;

pub struct AVRClockEventEntry {
    pub cycles: u32,
    pub callback: AVRClockEventCallback,
    pub event_type: AVRClockEventType,
    pub next: Option<Box<AVRClockEventEntry>>,
}

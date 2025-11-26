#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum I2CBusStatus {
    IDLE,
    START,
    STOP,
    ADDRESS,
    DATA_REQUEST,
    DATA_AVAILABLE, // Data has already been sent, either on write or read
}

pub struct I2CBus {
    pub status: I2CBusStatus,
    pub address: u8,
    pub data: u8,
    pub read: bool, // vs write; master's action

    pub acked: bool,
}

impl I2CBus {
    pub fn new() -> Self {
        Self {
            status: I2CBusStatus::IDLE,
            address: 0xff,
            data: 0xff,
            read: true,
            acked: false,
        }
    }
}

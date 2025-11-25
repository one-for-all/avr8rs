use std::panic;

use crate::atmega328p::ATMega328P;

const TWSR_TWPS1: u8 = 0x2; // TWI Prescaler Bits
const TWSR_TWPS0: u8 = 0x1; // TWI Prescaler Bits
const TWSR_TWPS_MASK: u8 = TWSR_TWPS1 | TWSR_TWPS0; // TWI Prescaler mask

pub struct TWIConfig {
    pub TWBR: u8,
    pub TWSR: u8,
    pub TWCR: u8,
    pub TWDR: u8,
}

/// I2C communication interface
pub struct AVRI2C {
    config: TWIConfig,
    freq_hz: usize, // clock frequency
}

impl AVRI2C {
    pub fn new(config: TWIConfig, freq_hz: usize) -> Self {
        Self { config, freq_hz }
    }

    pub fn scl_frequency(&self, data: &Vec<u8>) -> usize {
        self.freq_hz
            / (16 + 2 * data[self.config.TWBR as usize] as usize * self.prescaler(data)) as usize
    }

    pub fn prescaler(&self, data: &Vec<u8>) -> usize {
        match data[self.config.TWSR as usize] & TWSR_TWPS_MASK {
            0 => 1,
            1 => 4,
            2 => 16,
            3 => 64,
            _ => panic!("should not have values more than 2 bits"),
        }
    }
}

impl ATMega328P {
    pub fn i2c_scl_frequency(&self) -> usize {
        self.i2c.scl_frequency(&self.cpu.data)
    }
}

pub const TWI_CONFIG: TWIConfig = TWIConfig {
    TWBR: 0xb8,
    TWSR: 0xb9,
    TWCR: 0xbc,
    TWDR: 0xbb,
};

#[cfg(test)]
mod i2c_tests {
    use crate::{atmega328p::ATMega328P, peripheral::i2c::TWI_CONFIG};

    /// Correctly computes SCL (serial clock) frequency from TWBR register value
    #[test]
    fn SCL_freq_from_TWBR() {
        // Arrange
        let mut atmega = ATMega328P::new("", 16_000_000);

        // Act
        atmega.write_data(TWI_CONFIG.TWBR as u16, 72);
        atmega.write_data(TWI_CONFIG.TWSR as u16, 0); // set prescaler to 1

        // Arrange
        assert_eq!(atmega.i2c_scl_frequency(), 100_000); // 16000000 / (16 + 2 * 72)
    }
}

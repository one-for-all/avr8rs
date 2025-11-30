use crate::{
    Float, PI,
    peripheral::i2c::bus::{I2CBus, I2CBusStatus},
};

const ADDR_STATUS: u8 = 0x0b; // magnet status
const ADDR_MAGNITUDE: u8 = 0x1b; // magnitude of internal CORDIC - higher byte
const ADDR_MAGNITUDE_LOWER: u8 = ADDR_MAGNITUDE + 1; // magnitude of internal CORDIC - lower byte
const ADDR_RAW_ANGLE: u8 = 0x0c; // raw angle - higher byte;
const ADDR_RAW_ANGLE_LOWER: u8 = ADDR_RAW_ANGLE + 1; // raw angle - lower byte;

/// Simulate AS5600 magnetic rotary encoder
pub struct AS5600 {
    device_address: u8,   // fixed address of this device
    register_address: u8, // the register being addressed by master

    pub angle: Float, // angle sensed in radian
}

impl AS5600 {
    pub fn new() -> Self {
        Self {
            device_address: 0x36,
            register_address: 0xff,
            angle: 0.,
        }
    }

    pub fn step(&mut self, i2c_bus: &mut I2CBus) {
        match i2c_bus.status {
            I2CBusStatus::ADDRESS => {
                if i2c_bus.address == self.device_address {
                    i2c_bus.acked = true;
                    if i2c_bus.read {
                        i2c_bus.data = self.read_value();
                        i2c_bus.status = I2CBusStatus::DATA_AVAILABLE;
                    }
                }
            }
            I2CBusStatus::DATA_AVAILABLE => {
                if !i2c_bus.read {
                    self.register_address = i2c_bus.data;
                }
            }
            I2CBusStatus::DATA_REQUEST => {
                if i2c_bus.read {
                    if i2c_bus.acked {
                        // NACK means that last byte has been read
                        i2c_bus.data = self.read_value();
                        i2c_bus.status = I2CBusStatus::DATA_AVAILABLE;
                    } else {
                        // println!("last byte has been read");
                    }
                }
            }
            _ => {}
        }
    }

    /// Returns the value in the addressed register
    fn read_value(&mut self) -> u8 {
        // compute raw angle
        let raw = (self.angle / (2. * PI) * 4096.) as u16 % 4096;

        let register = self.register_address;
        let value = match register {
            ADDR_STATUS => 0x20,
            ADDR_MAGNITUDE => 0x0f,
            ADDR_MAGNITUDE_LOWER => 0x00,
            ADDR_RAW_ANGLE => (raw >> 8) as u8,
            ADDR_RAW_ANGLE_LOWER => raw as u8,
            _ => {
                panic!("unknown register address: {:02x}", self.register_address);
            }
        };
        self.register_address += 1;
        value
    }
}

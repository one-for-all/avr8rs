use std::{collections::HashMap, panic};

use crate::{
    atmega328p::{ATMega328P, PeripheralMemoryWriteHook},
    clock::AVRClockEventType,
    cpu::{self, CPU},
    interrupt::AVRInterruptConfig,
    peripheral::i2c::{self, bus::I2CBus},
};

pub mod bus;

const TWCR_TWINT: u8 = 0x80; // TWI Interrupt Flag
const TWCR_TWEA: u8 = 0x40; // TWI Enable Acknowledge Bit
const TWCR_TWSTA: u8 = 0x20; // TWI START Condition Bit
const TWCR_TWSTO: u8 = 0x10; // TWI STOP Condition Bit
const TWCR_TWIE: u8 = 0x1; // TWI Interrupt Enable
const TWCR_TWEN: u8 = 0x4; //  TWI Enable Bit
const TWSR_TWS_MASK: u8 = 0xf8; // TWI Status
const TWSR_TWPS1: u8 = 0x2; // TWI Prescaler Bits
const TWSR_TWPS0: u8 = 0x1; // TWI Prescaler Bits
const TWSR_TWPS_MASK: u8 = TWSR_TWPS1 | TWSR_TWPS0; // TWI Prescaler mask

// TWI statuses
const STATUS_IDLE: u8 = 0xf8;
// Master states
const STATUS_START: u8 = 0x08;
const STATUS_REPEATED_START: u8 = 0x10;
const STATUS_SLAW_ACK: u8 = 0x18;
const STATUS_SLAW_NACK: u8 = 0x20;
const STATUS_DATA_SENT_ACK: u8 = 0x28;
const STATUS_DATA_SENT_NACK: u8 = 0x30;

const STATUS_SLAR_ACK: u8 = 0x40;
const STATUS_SLAR_NACK: u8 = 0x48;
const STATUS_DATA_RECEIVED_ACK: u8 = 0x50;
const STATUS_DATA_RECEIVED_NACK: u8 = 0x58;

pub struct TWIConfig {
    twi_interrupt: u8,
    pub TWBR: u8,
    pub TWSR: u8,
    pub TWCR: u8,
    pub TWDR: u8,
}

pub const TWI_CONFIG: TWIConfig = TWIConfig {
    twi_interrupt: 0x30,
    TWBR: 0xb8,
    TWSR: 0xb9,
    TWCR: 0xbc,
    TWDR: 0xbb,
};

/// I2C communication interface
pub struct AVRI2C {
    config: TWIConfig,
    freq_hz: usize, // clock frequency

    twi: AVRInterruptConfig,

    pub busy: bool,
    pub wait_ack: bool,
}

impl AVRI2C {
    pub fn new(config: TWIConfig, freq_hz: usize, cpu: &mut CPU) -> Self {
        let twi = AVRInterruptConfig {
            address: config.twi_interrupt,
            flag_register: config.TWCR as u16,
            flag_mask: TWCR_TWINT,
            enable_register: config.TWCR as u16,
            enable_mask: TWCR_TWIE,
        };
        let mut i2c = Self {
            config,
            freq_hz,
            twi,
            busy: false,
            wait_ack: false,
        };
        i2c.update_status(cpu, STATUS_IDLE);
        i2c
    }

    pub fn add_TWCR_write_hook(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.TWCR as u16,
            Box::new(|atmega, value, _, _, _| {
                atmega.cpu.data[atmega.i2c.config.TWCR as usize] = value;
                atmega.cpu.clear_interrupt_by_flag(&atmega.i2c.twi, value);
                atmega.cpu.update_interrupt_enable(atmega.i2c.twi, value);
                let clear_interrupt = value & TWCR_TWINT != 0;
                if clear_interrupt && value & TWCR_TWEN != 0 && !atmega.i2c.busy {
                    atmega.cpu.add_clock_event(
                        Box::new(ATMega328P::i2c_op),
                        0,
                        crate::clock::AVRClockEventType::I2C,
                    );
                }
                true
            }),
        );
    }

    pub fn complete_start(&mut self, cpu: &mut CPU) {
        self.busy = false;
        let status = if self.status(&cpu.data) == STATUS_IDLE {
            STATUS_START
        } else {
            STATUS_REPEATED_START
        };
        self.update_status(cpu, status);
    }

    pub fn complete_stop(&mut self, cpu: &mut CPU) {
        self.busy = false;
        cpu.data[self.config.TWCR as usize] &= !TWCR_TWSTO;
        self.update_status(cpu, STATUS_IDLE);
    }

    pub fn complete_connect(&mut self, acked: bool, cpu: &mut CPU) {
        self.busy = false;
        if cpu.data[self.config.TWDR as usize] & 0x1 != 0 {
            self.update_status(
                cpu,
                if acked {
                    STATUS_SLAR_ACK
                } else {
                    STATUS_SLAR_NACK
                },
            );
        } else {
            self.update_status(
                cpu,
                if acked {
                    STATUS_SLAW_ACK
                } else {
                    STATUS_SLAW_NACK
                },
            );
        }
    }

    pub fn complete_write(&mut self, acked: bool, cpu: &mut CPU) {
        self.busy = false;
        self.update_status(
            cpu,
            if acked {
                STATUS_DATA_SENT_ACK
            } else {
                STATUS_DATA_SENT_NACK
            },
        );
    }

    pub fn complete_read(&mut self, ack: bool, cpu: &mut CPU) {
        self.busy = false;
        self.update_status(
            cpu,
            if ack {
                STATUS_DATA_RECEIVED_ACK
            } else {
                STATUS_DATA_RECEIVED_NACK
            },
        );
    }

    pub fn status(&self, data: &Vec<u8>) -> u8 {
        data[self.config.TWSR as usize] & TWSR_TWS_MASK
    }

    pub fn update_status(&mut self, cpu: &mut CPU, status: u8) {
        assert_eq!(status & 0x3, 0);

        let TWSR = self.config.TWSR;
        cpu.data[TWSR as usize] = (cpu.data[TWSR as usize] & !TWSR_TWS_MASK) | status;
        cpu.set_interrupt_flag(self.twi);
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

    pub fn i2c_status(&self) -> u8 {
        self.i2c.status(&self.cpu.data)
    }

    /// Operations performed by hardware to execute i2c communication
    pub fn i2c_op(&mut self, i2c_bus: Option<&mut I2CBus>, _: bool, _: bool) {
        let twcr_value = self.cpu.data[self.i2c.config.TWCR as usize];
        let twdr_value = self.cpu.data[self.i2c.config.TWDR as usize];
        let status = self.i2c.status(&self.cpu.data);
        // println!("status: {:02x}", status);
        if twcr_value & TWCR_TWSTA != 0 {
            self.i2c.busy = true;
            if let Some(i2c_bus) = i2c_bus {
                i2c_bus.status = bus::I2CBusStatus::START;
            }
            self.i2c.complete_start(&mut self.cpu);
        } else if twcr_value & TWCR_TWSTO != 0 {
            self.i2c.busy = true;
            if let Some(i2c_bus) = i2c_bus {
                i2c_bus.status = bus::I2CBusStatus::STOP;
            }
            self.i2c.complete_stop(&mut self.cpu);
        } else if status == STATUS_START || status == STATUS_REPEATED_START {
            self.i2c.busy = true;
            if let Some(i2c_bus) = i2c_bus {
                if !self.i2c.wait_ack {
                    i2c_bus.status = bus::I2CBusStatus::ADDRESS;
                    i2c_bus.address = twdr_value >> 1;
                    i2c_bus.read = (twdr_value & 0x1) != 0;
                    self.i2c.wait_ack = true;
                    self.cpu.add_clock_event(
                        Box::new(ATMega328P::i2c_op),
                        0,
                        AVRClockEventType::I2C,
                    ); // check for ack
                } else {
                    self.i2c.wait_ack = false;
                    let acked = i2c_bus.acked;
                    self.i2c.complete_connect(acked, &mut self.cpu);
                    i2c_bus.acked = false; // reset
                }
            } else {
                self.i2c.wait_ack = false;
                self.i2c.complete_connect(false, &mut self.cpu);
            }
        } else if status == STATUS_SLAW_ACK || status == STATUS_DATA_SENT_ACK {
            self.i2c.busy = true;
            if let Some(i2c_bus) = i2c_bus {
                assert_eq!(i2c_bus.read, false);
                if !self.i2c.wait_ack {
                    i2c_bus.status = bus::I2CBusStatus::DATA_AVAILABLE;
                    i2c_bus.data = twdr_value;
                    self.i2c.wait_ack = true;
                    self.cpu.add_clock_event(
                        Box::new(ATMega328P::i2c_op),
                        0,
                        AVRClockEventType::I2C,
                    ); // check for ack
                } else {
                    self.i2c.wait_ack = false;
                    let acked = i2c_bus.acked;
                    self.i2c.complete_write(acked, &mut self.cpu);
                    i2c_bus.acked = false; // reset
                }
            } else {
                self.i2c.wait_ack = false;
                self.i2c.complete_write(false, &mut self.cpu);
            }
        } else if status == STATUS_SLAR_ACK || status == STATUS_DATA_RECEIVED_ACK {
            self.i2c.busy = true;
            if let Some(i2c_bus) = i2c_bus {
                assert_eq!(i2c_bus.read, true);
                self.cpu.data[self.i2c.config.TWDR as usize] = i2c_bus.data; // read data
                let ack = twcr_value & TWCR_TWEA != 0;
                i2c_bus.status = bus::I2CBusStatus::DATA_REQUEST;
                i2c_bus.acked = ack;
                self.i2c.complete_read(ack, &mut self.cpu);
            } else {
                self.i2c.complete_read(false, &mut self.cpu);
            }
        }
    }
}

#[cfg(test)]
mod i2c_tests {
    use crate::{
        atmega328p::{self, ATMega328P, DEFAULT_FREQ},
        peripheral::i2c::{
            STATUS_DATA_SENT_ACK, STATUS_IDLE, STATUS_REPEATED_START, STATUS_SLAW_ACK,
            STATUS_START, TWCR_TWEN, TWCR_TWIE, TWCR_TWINT, TWCR_TWSTA, TWCR_TWSTO, TWI_CONFIG,
            bus::{I2CBus, I2CBusStatus},
        },
    };

    /// Correctly computes SCL (serial clock) frequency from TWBR register value
    #[test]
    fn SCL_freq_from_TWBR() {
        // Arrange
        let mut atmega = ATMega328P::new("", 16_000_000);

        // Act
        atmega.write_data(TWI_CONFIG.TWBR as u16, 72);
        atmega.write_data(TWI_CONFIG.TWSR as u16, 0); // set prescaler to 1

        // Assert
        assert_eq!(atmega.i2c_scl_frequency(), 100_000); // 16000000 / (16 + 2 * 72)
    }

    /// Correctly use prescaler in computing SCL frequency
    #[test]
    fn SCL_freq_with_prescaler() {
        // Arrange
        let mut atmega = ATMega328P::new("", 16_000_000);

        // Act
        atmega.write_data(TWI_CONFIG.TWBR as u16, 3);
        atmega.write_data(TWI_CONFIG.TWSR as u16, 0x01); // set prescaler to 4

        // Assert
        assert_eq!(atmega.i2c_scl_frequency(), 400_000); // 16000000 / (16 + 2 * 72)
    }

    #[test]
    fn initial_status_idle() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        atmega.write_data(TWI_CONFIG.TWBR as u16, 72);

        // Assert
        assert_eq!(atmega.i2c_status(), STATUS_IDLE);
    }

    #[test]
    fn i2c_start() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let mut i2c_bus = I2CBus::new();
        assert_eq!(i2c_bus.status, I2CBusStatus::IDLE);

        // Act
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWSTA | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        // Assert
        assert_eq!(i2c_bus.status, I2CBusStatus::START);
        assert_eq!(atmega.i2c.status(&atmega.cpu.data), STATUS_START);
    }

    /// Trigger interrupt on send START complete
    #[test]
    fn trigger_TWI_interrupt() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);

        // Act
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWIE);
        atmega.cpu.set_sreg(1 << 7); // global enable interrupt
        atmega.i2c.complete_start(&mut atmega.cpu);
        atmega.tick(None);

        // Assert
        assert_eq!(atmega.cpu.pc, TWI_CONFIG.twi_interrupt as u32);
        assert_eq!(atmega.cpu.cycles, 2);
        assert_eq!(atmega.cpu.data[TWI_CONFIG.TWCR as usize] & TWCR_TWINT, 0); // interrupt flag cleared after handling
    }

    #[test]
    fn i2c_stop() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let mut i2c_bus = I2CBus::new();
        assert_eq!(i2c_bus.status, I2CBusStatus::IDLE);

        // Act
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWSTO | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        // Assert
        assert_eq!(i2c_bus.status, I2CBusStatus::STOP);
    }

    #[test]
    fn i2c_connect() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let mut i2c_bus = I2CBus::new();

        // Act & Assert
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWSTA | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        let address = 0x36;
        atmega.write_data(TWI_CONFIG.TWDR as u16, address << 1 | 0x0); // write
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        assert_eq!(i2c_bus.address, address);
        assert_eq!(i2c_bus.read, false);

        i2c_bus.acked = true;
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        assert_eq!(atmega.i2c.status(&atmega.cpu.data), STATUS_SLAW_ACK);
    }

    #[test]
    fn i2c_write() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let mut i2c_bus = I2CBus::new();

        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWSTA | TWCR_TWEN); // start
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        let address = 0x36;
        atmega.write_data(TWI_CONFIG.TWDR as u16, address << 1 | 0x0); // write address
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        i2c_bus.acked = true; // ack address
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        // Act & Assert
        let data = 0x55;
        atmega.write_data(TWI_CONFIG.TWDR as u16, data); // write data
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        assert_eq!(i2c_bus.data, data);

        i2c_bus.acked = true; // ack data
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        assert_eq!(atmega.i2c.status(&atmega.cpu.data), STATUS_DATA_SENT_ACK);
    }

    #[test]
    fn repeated_start() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let mut i2c_bus = I2CBus::new();
        assert_eq!(i2c_bus.status, I2CBusStatus::IDLE);

        // Act
        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWSTA | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        atmega.write_data(TWI_CONFIG.TWCR as u16, TWCR_TWINT | TWCR_TWSTA | TWCR_TWEN);
        atmega.cpu.cycles += 1;
        atmega.tick(Some(&mut i2c_bus));

        // Assert
        assert_eq!(i2c_bus.status, I2CBusStatus::START);
        assert_eq!(atmega.i2c.status(&atmega.cpu.data), STATUS_REPEATED_START);
    }
}

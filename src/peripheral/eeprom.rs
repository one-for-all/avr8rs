use std::{collections::HashMap, mem};

use crate::{
    atmega328p::{ATMega328P, PeripheralMemoryWriteHook},
    interrupt::AVRInterruptConfig,
};

const EERE: u8 = 1 << 0;
const EEPE: u8 = 1 << 1; // Write Enable
const EEMPE: u8 = 1 << 2; // Master Write Enable
const EERIE: u8 = 1 << 3;
const EEPM0: u8 = 1 << 4;
const EEPM1: u8 = 1 << 5;
const EECR_WRITE_MASK: u8 = EEPE | EEMPE | EERIE | EEPM0 | EEPM1;

pub struct AVREEPROMConfig {
    eepromReadyInterrupt: u8,

    EECR: u8,
    EEDR: u8,
    EEARL: u8,
    EEARH: u8,

    /** The amount of clock cycles erase takes */
    erase_cycles: u32,
    /** The amount of clock cycles a write takes */
    write_cycles: u32,
}

pub const EEPROM_CONFIG: AVREEPROMConfig = AVREEPROMConfig {
    eepromReadyInterrupt: 0x2c,
    EECR: 0x3f,
    EEDR: 0x40,
    EEARL: 0x41,
    EEARH: 0x42,
    erase_cycles: 28800, // 1.8ms at 16MHz
    write_cycles: 28800, // 1.8ms at 16MHz
};

pub struct AVREEPROM {
    pub config: AVREEPROMConfig,
    eer: AVRInterruptConfig,

    write_enabled_cycles: u32,
    write_complete_cycles: u32,

    pub memory: Vec<u8>,
}

impl AVREEPROM {
    pub fn new(config: AVREEPROMConfig, memory_size: usize) -> Self {
        let eer = AVRInterruptConfig {
            address: config.eepromReadyInterrupt,
            flag_register: config.EECR as u16,
            flag_mask: EEPE,
            enable_register: config.EECR as u16,
            enable_mask: EERIE,
            // TODO: add constant and inverse flag
            inverse_flag: true,
        };
        Self {
            config,
            eer,
            write_enabled_cycles: 0,
            write_complete_cycles: 0,
            memory: vec![0xff; memory_size],
        }
    }

    pub fn add_EECR_write_hook(&self, write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>) {
        write_hooks.insert(
            self.config.EECR as u16,
            Box::new(|atmega, value, _, _, _| {
                let config = &atmega.eeprom.config;
                let addr = ((atmega.cpu.data[config.EEARH as usize] as u32) << 8)
                    | (atmega.cpu.data[config.EEARL as usize] as u32);

                atmega.cpu.data[config.EECR as usize] = (atmega.cpu.data[config.EECR as usize]
                    & !EECR_WRITE_MASK)
                    | (value & EECR_WRITE_MASK);
                atmega.cpu.update_interrupt_enable(atmega.eeprom.eer, value);

                if value & EERE != 0 {
                    atmega.cpu.clear_interrupt(&atmega.eeprom.eer, true);
                }

                if value & EEMPE != 0 {
                    let eempe_cycles = 4;
                    atmega.eeprom.write_enabled_cycles = atmega.cpu.cycles + eempe_cycles;
                    atmega.cpu.add_clock_event(
                        Box::new(|atmega, _, _, _| {
                            atmega.cpu.data[atmega.eeprom.config.EECR as usize] &= !EEMPE;
                        }),
                        eempe_cycles,
                        crate::clock::AVRClockEventType::EEPROMFinish,
                    );
                }

                // TODO: implement Read

                // Write
                if value & EEPE != 0 {
                    //  If EEMPE is zero, setting EEPE will have no effect.
                    if atmega.cpu.cycles >= atmega.eeprom.write_enabled_cycles {
                        atmega.cpu.data[config.EECR as usize] &= !EEPE;
                        return true;
                    }

                    // Check for write-in-progress
                    if atmega.cpu.cycles < atmega.eeprom.write_complete_cycles {
                        return true;
                    }

                    let eedr = atmega.cpu.data[config.EEDR as usize];
                    atmega.eeprom.write_complete_cycles = atmega.cpu.cycles;

                    // Erase
                    if value & EEPM1 == 0 {
                        atmega.eeprom.erase_memory(addr as u16);
                        atmega.eeprom.write_complete_cycles += atmega.eeprom.config.erase_cycles;
                    }

                    // Write
                    if value & EEPM0 == 0 {
                        atmega.eeprom.write_memory(addr as u16, eedr);
                        atmega.eeprom.write_complete_cycles += atmega.eeprom.config.write_cycles;
                    }

                    let config = &atmega.eeprom.config;
                    atmega.cpu.data[config.EECR as usize] |= EEPE;

                    atmega.cpu.add_clock_event(
                        Box::new(|atmega, _, _, _| {
                            atmega.cpu.set_interrupt_flag(atmega.eeprom.eer);
                        }),
                        atmega.eeprom.write_complete_cycles - atmega.cpu.cycles,
                        crate::clock::AVRClockEventType::EEPROMWriteComplete,
                    );

                    // When EEPE has been set, the CPU is halted for two cycles before the
                    // next instruction is executed.
                    atmega.cpu.cycles += 2;
                }

                true
            }),
        );
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] &= value;
    }

    fn erase_memory(&mut self, addr: u16) {
        self.memory[addr as usize] = 0xff;
    }
}

#[cfg(test)]
mod eeprom_tests {
    use plotters::data;

    use crate::{
        atmega328p::{ATMega328P, DEFAULT_FREQ},
        peripheral::eeprom::{EEMPE, EEPE, EEPROM_CONFIG},
    };

    #[test]
    fn write() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let data = 0x55;
        let addr = 15;

        // Act
        atmega.write_data(EEPROM_CONFIG.EEDR as u16, data);
        atmega.write_data(EEPROM_CONFIG.EEARL as u16, addr);
        atmega.write_data(EEPROM_CONFIG.EEARH as u16, 0);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEMPE);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEPE);
        atmega.tick(None);

        // Assert
        assert_eq!(atmega.cpu.cycles, 2);
        assert_eq!(atmega.eeprom.memory[addr as usize], data);
        assert_eq!(atmega.cpu.data[EEPROM_CONFIG.EECR as usize] & EEPE, EEPE);
    }

    #[test]
    fn write_two_bytes() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let data1 = 0x55;
        let addr1 = 15;
        let data2 = 0x66;
        let addr2 = 16;

        // Act
        // Write data1
        atmega.write_data(EEPROM_CONFIG.EEDR as u16, data1);
        atmega.write_data(EEPROM_CONFIG.EEARL as u16, addr1);
        atmega.write_data(EEPROM_CONFIG.EEARH as u16, 0);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEMPE);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEPE);
        atmega.tick(None);
        assert_eq!(atmega.cpu.cycles, 2);

        // wait long enough time for the first write to finish
        atmega.cpu.cycles += 10000000;
        atmega.tick(None);

        // Write data2
        atmega.write_data(EEPROM_CONFIG.EEDR as u16, data2);
        atmega.write_data(EEPROM_CONFIG.EEARL as u16, addr2);
        atmega.write_data(EEPROM_CONFIG.EEARH as u16, 0);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEMPE);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEPE);
        atmega.tick(None);

        // Assert
        assert_eq!(atmega.cpu.cycles, 10000000 + 2 + 2);
        assert_eq!(atmega.eeprom.memory[addr1 as usize], data1);
        assert_eq!(atmega.eeprom.memory[addr2 as usize], data2);
    }

    #[test]
    fn write_two_bytes_same_addr() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let data1 = 0x55;
        let data2 = 0x66;
        let addr = 15;

        // Act
        // Write data1
        atmega.write_data(EEPROM_CONFIG.EEDR as u16, data1);
        atmega.write_data(EEPROM_CONFIG.EEARL as u16, addr);
        atmega.write_data(EEPROM_CONFIG.EEARH as u16, 0);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEMPE);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEPE);
        atmega.tick(None);
        assert_eq!(atmega.cpu.cycles, 2);
        assert_eq!(atmega.eeprom.memory[addr as usize], data1);

        // wait long enough time for the first write to finish
        atmega.cpu.cycles += 10000000;
        atmega.tick(None);

        // Write data2
        atmega.write_data(EEPROM_CONFIG.EEDR as u16, data2);
        atmega.write_data(EEPROM_CONFIG.EEARL as u16, addr);
        atmega.write_data(EEPROM_CONFIG.EEARH as u16, 0);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEMPE);
        atmega.write_data(EEPROM_CONFIG.EECR as u16, EEPE);
        atmega.tick(None);

        // Assert
        assert_eq!(atmega.cpu.cycles, 10000000 + 2 + 2);
        assert_eq!(atmega.eeprom.memory[addr as usize], data2);
    }
}

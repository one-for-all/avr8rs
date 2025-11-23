use std::collections::HashMap;

use crate::{
    atmega328p::{ATMega328P, PeripheralMemoryWriteHook},
    ternary,
};

#[derive(Debug)]
pub enum PinState {
    Low,
    High,
    Input,
    InputPullUp,
}

#[allow(non_snake_case)]
pub struct AVRPortConfig {
    pub PIN: u8,  // Input register address
    pub DDR: u8,  // Direction register address
    pub PORT: u8, // Data register address
}

pub struct AVRIOPort {
    pub config: AVRPortConfig,

    pin_value: u8,

    override_mask: u8,
    override_value: u8,
    pub last_value: u8,
    last_ddr: u8,
    last_pin: u8,
    pub open_collector: u8,
}

impl AVRIOPort {
    pub fn new(config: AVRPortConfig) -> Self {
        AVRIOPort {
            config,
            pin_value: 0,
            override_mask: 0xff,
            override_value: 0,
            last_value: 0,
            last_ddr: 0,
            last_pin: 0,
            open_collector: 0,
        }
    }

    pub fn add_ddr_handler(
        &self,
        write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>,
        port_id: usize,
    ) {
        write_hooks.insert(
            self.config.DDR as u16,
            Box::new(move |atmega, ddr_mask, _, _, _| {
                let config = &atmega.ports[port_id].config;
                let port = config.PORT;
                let ddr = config.DDR;
                let pin = config.PIN;

                let port_value = atmega.cpu.data[port as usize];
                atmega.cpu.data[ddr as usize] = ddr_mask;

                let port = &mut atmega.ports[port_id];
                port.write_gpio(port_value, ddr_mask);
                let new_pin = port.update_pin_register(ddr_mask);
                atmega.cpu.data[pin as usize] = new_pin;

                true
            }),
        );
    }

    pub fn add_port_handler(
        &self,
        write_hooks: &mut HashMap<u16, PeripheralMemoryWriteHook>,
        port_id: usize,
    ) {
        write_hooks.insert(
            self.config.PORT as u16,
            Box::new(move |atmega, port_value, _, _, _| {
                let config = &atmega.ports[port_id].config;
                let port = config.PORT;
                let ddr = config.DDR;

                let ddr_mask = atmega.cpu.data[ddr as usize];
                atmega.cpu.data[port as usize] = port_value;

                let port = &mut atmega.ports[port_id];
                port.write_gpio(port_value, ddr_mask);
                port.update_pin_register(ddr_mask);
                true
            }),
        );
    }

    pub fn update_pin_register(&mut self, ddr: u8) -> u8 {
        let new_pin = (self.pin_value & !ddr) | (self.last_value & ddr);
        if self.last_pin != new_pin {
            for index in 0..8 {
                if (new_pin & (1 << index)) != (self.last_pin & (1 << index)) {
                    let value = (new_pin & (1 << index)) != 0;
                    // TODO: implement interrupt and listener
                    // self.toggleInterrupt(index, value);
                    // self.externalClockListeners[index]?.(value);
                }
            }
            self.last_pin = new_pin;
        }
        new_pin
    }

    pub fn write_gpio(&mut self, value: u8, ddr: u8) {
        let new_value =
            (((value & self.override_mask) | self.override_value) & ddr) | (value & !ddr);
        let prev_value = self.last_value;
        if new_value != prev_value || ddr != self.last_ddr {
            self.last_value = new_value;
            self.last_ddr = ddr;

            // TODO: implement GPIO listeners
            // for (const listener of this.listeners) {
            //   listener(newValue, prev_value);
            // }
        }
    }

    /// Get the state of a given GPIO pin
    ///
    /// @param index Pin index to return from 0 to 7
    /// @returns PinState.Low or PinState.High if the pin is set to output, PinState.Input if the pin is set
    /// to input, and PinState.InputPullUp if the pin is set to input and the internal pull-up resistor has
    /// been enabled.
    pub fn pin_state(&self, pin: u8, data: &Vec<u8>) -> PinState {
        let ddr = data[self.config.DDR as usize];
        let port = data[self.config.PORT as usize];
        let bit_mask: u8 = 1 << pin;
        let open_state = ternary!(port & bit_mask, PinState::InputPullUp, PinState::Input);
        if ddr & bit_mask != 0 {
            let high_value = ternary!(self.open_collector & bit_mask, open_state, PinState::High);
            ternary!(self.last_value & bit_mask, high_value, PinState::Low)
        } else {
            open_state
        }
    }
}

impl ATMega328P {
    pub fn port_pin_state(&self, port: &str, pin: u8) -> PinState {
        match port {
            "B" => self.ports[0].pin_state(pin, &self.cpu.data),
            "C" => self.ports[1].pin_state(pin, &self.cpu.data),
            "D" => self.ports[2].pin_state(pin, &self.cpu.data),
            _ => panic!("unknown port"),
        }
    }
}

pub const PORTB_CONFIG: AVRPortConfig = AVRPortConfig {
    PIN: 0x23,
    DDR: 0x24,
    PORT: 0x25,
};

pub const PORTC_CONFIG: AVRPortConfig = AVRPortConfig {
    PIN: 0x26,
    DDR: 0x27,
    PORT: 0x28,
};

pub const PORTD_CONFIG: AVRPortConfig = AVRPortConfig {
    PIN: 0x29,
    DDR: 0x2a,
    PORT: 0x2b,
};

#[cfg(test)]
mod port_tests {
    use crate::{
        atmega328p::{ATMega328P, DEFAULT_FREQ},
        port::{PORTB_CONFIG, PORTC_CONFIG, PORTD_CONFIG, PinState},
    };

    #[test]
    fn pin_default_input() {
        // Arrange
        let atmega = ATMega328P::new("", DEFAULT_FREQ);
        let pin = 4;

        // Act/Assert
        assert!(matches!(atmega.port_pin_state("B", pin), PinState::Input));
    }

    #[test]
    fn set_pin_high() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let pin = 3;

        // Act
        atmega.write_data(PORTB_CONFIG.DDR as u16, 1 << pin);
        atmega.write_data(PORTB_CONFIG.PORT as u16, 1 << pin);

        // Assert
        assert!(matches!(atmega.port_pin_state("B", pin), PinState::High));
    }

    #[test]
    fn set_pin_low() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let pin = 1;

        // Act
        atmega.write_data(PORTD_CONFIG.DDR as u16, 1 << pin);
        atmega.write_data(PORTD_CONFIG.PORT as u16, !(1 << pin));

        // Assert
        assert!(matches!(atmega.port_pin_state("D", pin), PinState::Low));
    }

    #[test]
    fn set_pin_pullup() {
        // Arrange
        let mut atmega = ATMega328P::new("", DEFAULT_FREQ);
        let pin = 2;

        // Act
        atmega.write_data(PORTC_CONFIG.PORT as u16, 1 << pin);

        // Assert
        assert!(matches!(
            atmega.port_pin_state("C", pin),
            PinState::InputPullUp
        ));
    }
}

use crate::cpu::CPU;

#[derive(Debug)]
pub enum PinState {
    Low,
    High,
    Input,
    InputPullUp,
}

pub struct AVRPortConfig {
    pub pin: u8,
    pub ddr: u8,
    pub port: u8,
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
}

pub const PORT_B_CONFIG: AVRPortConfig = AVRPortConfig {
    pin: 0x23,
    ddr: 0x24,
    port: 0x25,
};

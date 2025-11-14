use crate::interrupt::AVRInterruptConfig;

#[allow(non_snake_case)]
pub struct AVRTimerConfig {
    // Interrupt vectors
    pub ovf_interrupt: u8,

    // Register addresses
    pub TIFR: u8,
    pub TCNT: u8,
    pub OCRA: u8,
    pub TCCRA: u8,
    pub TCCRB: u8,
    pub TIMSK: u8,

    // TIFR bits
    pub TOV: u8,

    // TIMSK bits
    pub TOIE: u8,
}

pub enum OCRUpdateMode {
    Immediate,
}

pub struct AVRTimer {
    pub max: u16,

    pub config: AVRTimerConfig,
    pub last_cycle: u32,

    pub ocra: u16,
    pub next_ocra: u16,

    pub ocr_update_mode: OCRUpdateMode,

    pub tcnt: u16,

    pub update_divider: bool,
    pub divider: u16,

    pub high_byte_temp: u8, // This is the temporary register used to access 16-bit registers (section 16.3 of the datasheet)

    pub ovf: AVRInterruptConfig,
}

impl AVRTimer {
    pub fn new(config: AVRTimerConfig) -> Self {
        let ovf = AVRInterruptConfig::new(&config);
        AVRTimer {
            max: 0xff, // config.bits === 16 ? 0xffff : 0xff;
            config,
            last_cycle: 0,
            ocra: 0,
            next_ocra: 0,
            ocr_update_mode: OCRUpdateMode::Immediate,
            tcnt: 0,
            update_divider: false,
            divider: 0,
            high_byte_temp: 0,
            ovf,
        }
    }

    /// TOP value of counter
    pub fn top(&self) -> u16 {
        // for now, assume to be 0xff
        0xff
    }
}

pub const TIMER_0_CONFIG: AVRTimerConfig = AVRTimerConfig {
    ovf_interrupt: 0x20,

    TIFR: 0x35,
    TCNT: 0x46,
    OCRA: 0x47,
    TCCRA: 0x44,
    TCCRB: 0x45,
    TIMSK: 0x6e,

    TOV: 1,

    TOIE: 1,
};

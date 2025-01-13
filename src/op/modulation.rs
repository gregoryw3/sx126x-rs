pub struct ModParams {
    inner: [u8; 8],
    // pub lora: LoraModParams,
}

impl From<ModParams> for [u8; 8] {
    fn from(val: ModParams) -> Self {
        val.inner
    }
}

impl From<&ModParams> for [u8; 8] {
    fn from(val: &ModParams) -> Self {
        val.inner
    }
}

impl ModParams {
    pub fn get_spread_factor(&self) -> LoRaSpreadFactor {
        self.inner[0].into()
    }
    pub fn get_bandwidth(&self) -> LoRaBandWidth {
        self.inner[1].into()
    }
    pub fn get_coding_rate(&self) -> LoraCodingRate {
        self.inner[2].into()
    }
    pub fn get_low_dr_opt(&self) -> bool {
        self.inner[3] != 0
    }
}

pub use lora::*;

mod lora {
    use super::ModParams;
    #[derive(Copy, Clone)]
    #[repr(u8)]
    pub enum LoRaSpreadFactor {
        SF5 = 0x05,
        SF6 = 0x06,
        SF7 = 0x07,
        SF8 = 0x08,
        SF9 = 0x09,
        SF10 = 0x0A,
        SF11 = 0x0B,
        SF12 = 0x0C,
    }

    impl From<u8> for LoRaSpreadFactor {
        fn from(value: u8) -> Self {
            match value {
                0x05 => Self::SF5,
                0x06 => Self::SF6,
                0x07 => Self::SF7,
                0x08 => Self::SF8,
                0x09 => Self::SF9,
                0x0A => Self::SF10,
                0x0B => Self::SF11,
                0x0C => Self::SF12,
                _ => panic!("Invalid LoRa spread factor"),
            }
        }
    }

    #[derive(Copy, Clone, PartialEq)]
    #[repr(u8)]
    pub enum LoRaBandWidth {
        /// 7.81 kHz
        BW7 = 0x00,
        /// 10.42 kHz
        BW10 = 0x08,
        /// 15.63 kHz
        BW15 = 0x01,
        /// 20.83 kHz
        BW20 = 0x09,
        /// 31.25 kHz
        BW31 = 0x02,
        /// 41.67 kHz
        BW41 = 0x0A,
        /// 62.50 kHz
        BW62 = 0x03,
        /// 125 kHz
        BW125 = 0x04,
        /// 250 kHz
        BW250 = 0x05,
        /// 500 kHz
        BW500 = 0x06,
    }

    impl From<u8> for LoRaBandWidth {
        fn from(value: u8) -> Self {
            match value {
                0x00 => Self::BW7,
                0x08 => Self::BW10,
                0x01 => Self::BW15,
                0x09 => Self::BW20,
                0x02 => Self::BW31,
                0x0A => Self::BW41,
                0x03 => Self::BW62,
                0x04 => Self::BW125,
                0x05 => Self::BW250,
                0x06 => Self::BW500,
                _ => panic!("Invalid LoRa bandwidth"),
            }
        }
    }

    impl LoRaBandWidth {
        pub fn to_khz(&self) -> f32 {
            match self {
                LoRaBandWidth::BW7 => 7.81,
                LoRaBandWidth::BW10 => 10.42,
                LoRaBandWidth::BW15 => 15.63,
                LoRaBandWidth::BW20 => 20.83,
                LoRaBandWidth::BW31 => 31.25,
                LoRaBandWidth::BW41 => 41.67,
                LoRaBandWidth::BW62 => 62.50,
                LoRaBandWidth::BW125 => 125.0,
                LoRaBandWidth::BW250 => 250.0,
                LoRaBandWidth::BW500 => 500.0,
            }
        }
    }

    #[derive(Copy, Clone)]
    #[repr(u8)]
    pub enum LoraCodingRate {
        CR4_5 = 0x01,
        CR4_6 = 0x02,
        CR4_7 = 0x03,
        CR4_8 = 0x04,
    }

    impl From<u8> for LoraCodingRate {
        fn from(value: u8) -> Self {
            match value {
                0x01 => Self::CR4_5,
                0x02 => Self::CR4_6,
                0x03 => Self::CR4_7,
                0x04 => Self::CR4_8,
                _ => panic!("Invalid LoRa coding rate"),
            }
        }
    }

    pub struct LoraModParams {
        spread_factor: LoRaSpreadFactor,
        pub(crate) bandwidth: LoRaBandWidth,
        coding_rate: LoraCodingRate,
        low_data_rate_optimize: bool,
    }

    impl Default for LoraModParams {
        fn default() -> Self {
            Self {
                spread_factor: LoRaSpreadFactor::SF7,
                bandwidth: LoRaBandWidth::BW125,
                coding_rate: LoraCodingRate::CR4_5,
                low_data_rate_optimize: false,
            }
        }
    }

    impl LoraModParams {
        pub fn set_spread_factor(mut self, spread_factor: LoRaSpreadFactor) -> Self {
            self.spread_factor = spread_factor;
            self
        }
        pub fn set_bandwidth(mut self, bandwidth: LoRaBandWidth) -> Self {
            self.bandwidth = bandwidth;
            self
        }
        pub fn set_coding_rate(mut self, coding_rate: LoraCodingRate) -> Self {
            self.coding_rate = coding_rate;
            self
        }

        pub fn set_low_dr_opt(mut self, low_dr_opt: bool) -> Self {
            self.low_data_rate_optimize = low_dr_opt;
            self
        }
    }

    impl From<LoraModParams> for ModParams {
        fn from(val: LoraModParams) -> Self {
            ModParams {
                inner: [
                    val.spread_factor as u8,
                    val.bandwidth as u8,
                    val.coding_rate as u8,
                    val.low_data_rate_optimize as u8,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                ],
                // lora: val,
            }
        }
    }

    impl Default for ModParams {
        fn default() -> Self {
            LoraModParams::default().into()
        }
    }
}

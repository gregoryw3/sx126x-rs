/// Channel Activity Detection (CAD) commands
///
/// | Command      | Opcode | Parameters                                      | Description                                                                      |
/// |--------------|--------|-------------------------------------------------|----------------------------------------------------------------------------------|
/// | SetCadParams | 0x84   | cadSymbolNum, cadDetPeak, cadDetMin, cadExitMode, cadTimeout | Set the parameters which are used for performing a CAD (LoRa® only) |
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum CadCommands {
    /// 0x88, Op Code for setting the parameters which are used for performing a CAD (LoRa® only)
    SetCadParams = 0x88,
}

impl From<CadCommands> for u8 {
    fn from(val: CadCommands) -> Self {
        val as u8
    }
}

/// | cadSymbolNum   | Value | Number of Symbols used for CAD |
/// |----------------|-------|--------------------------------|
/// | CAD_ON_1_SYMB  | 0x00  | 1                              |
/// | CAD_ON_2_SYMB  | 0x01  | 2                              |
/// | CAD_ON_4_SYMB  | 0x02  | 4                              |
/// | CAD_ON_8_SYMB  | 0x03  | 8                              |
/// | CAD_ON_16_SYMB | 0x04  | 16                             |
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum CadSymbolNum {
    /// 0x00, Number of Symbols used for CAD: 1
    CAD_ON_1_SYMB = 0x00,
    /// 0x01, Number of Symbols used for CAD: 2
    CAD_ON_2_SYMB = 0x01,
    /// 0x02, Number of Symbols used for CAD: 4
    CAD_ON_4_SYMB = 0x02,
    /// 0x03, Number of Symbols used for CAD: 8
    CAD_ON_8_SYMB = 0x03,
    /// 0x04, Number of Symbols used for CAD: 16
    CAD_ON_16_SYMB = 0x04,
}

impl From<CadSymbolNum> for u8 {
    fn from(val: CadSymbolNum) -> Self {
        val as u8
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct CadDetPeak(u8);

impl CadDetPeak {
    pub const MIN: u8 = 18;
    pub const MAX: u8 = 35;

    pub const fn new(value: u8) -> Result<Self, &'static str> {
        if value >= Self::MIN && value <= Self::MAX {
            Ok(CadDetPeak(value))
        } else {
            Err("Value out of range")
        }
    }
}

impl From<CadDetPeak> for u8 {
    fn from(val: CadDetPeak) -> Self {
        val.0
    }
}

/// There is no need to use any value other than 10 (Semtech's response)
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct CadDetMin(u8);

impl CadDetMin {
    pub const MIN: u8 = 10;
    pub const MAX: u8 = 10;

    pub const fn new(value: u8) -> Result<Self, &'static str> {
        if value >= Self::MIN && value <= Self::MAX {
            Ok(CadDetMin(value))
        } else {
            Err("Value out of range")
        }
    }

    /// Not recommended to use this function but here for utility
    pub const fn new_override(value: u8) -> Result<Self, &'static str> {
        Ok(CadDetMin(value))
    }
}

impl From<CadDetMin> for u8 {
    fn from(val: CadDetMin) -> Self {
        val.0
    }
}

/// | cadExitMode | Value | Operation                                                                                           |
/// |-------------|-------|-----------------------------------------------------------------------------------------------------|
/// | CAD_ONLY    | 0x00  | The chip performs the CAD operation in LoRa®. Once done and whatever the activity on the channel, the chip goes back to STBY_RC mode. |
/// | CAD_RX      | 0x01  | The chip performs a CAD operation and if an activity is detected, it stays in RX until a packet is detected or the timer reaches the timeout defined by cadTimeout * 15.625 us |
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum CadExit {
    /// 0x00, The chip performs the CAD operation in LoRa®. Once done and whatever the activity on the channel, the chip goes back to STBY_RC mode.
    CAD_ONLY = 0x00,
    /// 0x01, The chip performs a CAD operation and if an activity is detected, it stays in RX until a packet is detected or the timer reaches the timeout defined by cadTimeout * 15.625 us.
    CAD_RX = 0x01,
}

impl From<CadExit> for u8 {
    fn from(val: CadExit) -> Self {
        val as u8
    }
}

/// cadTimeout: Timeout for the CAD operation in units of 15.625 us
/// 
/// The timeout is a 16-bit value, so the maximum value is 0xFFFF
/// 
/// It is auto converted to two u8 values for the command
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct CadTimeout(u32);

impl CadTimeout {
    pub const MIN: u32 = 0;
    pub const MAX: u32 = 0xFFFFFF; // 24-bit maximum value

    pub const fn new(value: u32) -> Result<Self, &'static str> {
        if value >= Self::MIN && value <= Self::MAX {
            Ok(CadTimeout(value))
        } else {
            Err("Value out of range")
        }
    }

    pub const fn split_u24(val: u32) -> (u8, u8, u8) {
        let byte0 = (val & 0xFF) as u8;
        let byte1 = ((val >> 8) & 0xFF) as u8;
        let byte2 = ((val >> 16) & 0xFF) as u8;
        (byte2, byte1, byte0)
    }

}

impl From<CadTimeout> for u32 {
    fn from(val: CadTimeout) -> Self {
        val.0
    }
}

pub struct CadParams {
    pub symbol_num: CadSymbolNum,
    pub det_peak: CadDetPeak,
    pub det_min: CadDetMin,
    pub exit_mode: CadExit,
    pub timeout: Option<CadTimeout>,
}
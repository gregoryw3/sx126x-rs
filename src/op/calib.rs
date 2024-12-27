#[derive(Copy, Clone)]
pub struct CalibParam {
    inner: u8,
}

impl From<CalibParam> for u8 {
    fn from(val: CalibParam) -> Self {
        val.inner
    }
}

impl From<u8> for CalibParam {
    fn from(val: u8) -> Self {
        Self { inner: val & 0x7F }
    }
}

impl CalibParam {
    /// Returns a `CalibParam` with all calibration parameters enabled.
    pub fn default() -> Self {
        Self::all()
    }

    /// Creates a new `CalibParam` with the specified calibration parameters.
    ///
    /// # Parameters
    ///
    /// * `rc64k_en` - Enable RC64K calibration. 64 kHz Resistance-Capacitance Oscillator
    /// * `rc13_en` - Enable RC13M calibration.  13 MHz Resistance-Capacitance Oscillator
    /// * `pll_en` - Enable PLL calibration.    Phase-Locked Loop
    /// * `adc_pulse_en` - Enable ADC pulse calibration.
    /// * `adc_bulk_n_en` - Enable ADC bulk N calibration.
    /// * `adc_bulk_p_en` - Enable ADC bulk P calibration.
    /// * `image_en` - Enable image calibration.
    ///
    /// | Bit  | CalibParam Calibration Setting            | Value | Description                        |
    /// |------|-------------------------------------------|-------|------------------------------------|
    /// | 0    | RC64k calibration                         | 0     | RC64k calibration disabled         |
    /// |      |                                           | 1     | RC64k calibration enabled          |
    /// | 1    | RC13M calibration                         | 0     | RC13M calibration disabled         |
    /// |      |                                           | 1     | RC13M calibration enabled          |
    /// | 2    | PLL calibration                           | 0     | PLL calibration disabled           |
    /// |      |                                           | 1     | PLL calibration enabled            |
    /// | 3    | ADC pulse calibration                     | 0     | ADC pulse calibration disabled     |
    /// |      |                                           | 1     | ADC pulse calibration enabled      |
    /// | 4    | ADC bulk N calibration                    | 0     | ADC bulk N calibration disabled    |
    /// |      |                                           | 1     | ADC bulk N calibration enabled     |
    /// | 5    | ADC bulk P calibration                    | 0     | ADC bulk P calibration disabled    |
    /// |      |                                           | 1     | ADC bulk P calibration enabled     |
    /// | 6    | Image calibration                         | 0     | Image calibration disabled         |
    /// |      |                                           | 1     | Image calibration enabled          |
    /// | 7    | RFU                                       | 0     | Reserved for future use            |
    ///
    /// The total calibration time if all blocks are calibrated is 3.5 ms. The calibration must be launched in STDBY_RC mode and the
    /// BUSY pins will be high during the calibration process. A falling edge of BUSY indicates the end of the procedure
    pub const fn new(
        rc64k_en: bool,
        rc13_en: bool,
        pll_en: bool,
        adc_pulse_en: bool,
        adc_bulk_n_en: bool,
        adc_bulk_p_en: bool,
        image_en: bool,
    ) -> Self {
        let inner = (rc64k_en as u8)
            | (rc13_en as u8) << 1
            | (pll_en as u8) << 2
            | (adc_pulse_en as u8) << 3
            | (adc_bulk_n_en as u8) << 4
            | (adc_bulk_p_en as u8) << 5
            | (image_en as u8) << 6
            | (0 as u8) << 7;
        Self { inner }
    }

    /// Returns a `CalibParam` with all calibration parameters enabled.
    pub const fn all() -> Self {
        Self::new(true, true, true, true, true, true, true)
    }
}

/// | Frequency Band (MHz) | Freq1 | Freq2 |
/// |----------------------|-------|-------|
/// | 430 - 440            | 0x6B  | 0x6F  |
/// | 470 - 510            | 0x75  | 0x81  |
/// | 779 - 787            | 0xC1  | 0xC5  |
/// | 863 - 870            | 0xD7  | 0xDB  |
/// | 902 - 928            | 0xE1 (default) | 0xE9 (default) |
#[derive(Copy, Clone)]
#[repr(u16)]
pub enum CalibImageFreq {
    MHz430_440 = 0x6B_6F,
    MHz470_510 = 0x75_81,
    MHz779_787 = 0xC1_C5,
    MHz863_870 = 0xD7_DB,
    MHz902_928 = 0xE1_E9,
}

impl From<CalibImageFreq> for [u8; 2] {
    fn from(val: CalibImageFreq) -> Self {
        (val as u16).to_be_bytes()
    }
}

impl CalibImageFreq {
    pub fn default() -> Self {
        Self::MHz902_928
    }

    pub const fn from_rf_frequency(rf_freq: u32) -> Self {
        match rf_freq / 1000000 {
            902..=928 => Self::MHz902_928,
            863..=870 => Self::MHz863_870,
            779..=787 => Self::MHz779_787,
            470..=510 => Self::MHz470_510,
            430..=440 => Self::MHz430_440,
            _ => Self::MHz902_928, // Default
        }
    }
}

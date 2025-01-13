
/// | Command                  | Opcode | Parameters                        | Description                                                                 |
/// |--------------------------|--------|-----------------------------------|-----------------------------------------------------------------------------|
/// | SetSleep                 | 0x84   | sleepConfig                       | Set Chip in SLEEP mode                                                     |
/// | SetStandby               | 0x80   | standbyConfig                     | Set Chip in STDBY_RC or STDBY_XOSC mode                                     |
/// | SetFs                    | 0xC1   | -                                 | Set Chip in Frequency Synthesis mode                                        |
/// | SetTx                    | 0x83   | timeout[23:0]                     | Set Chip in Tx mode                                                        |
/// | SetRx                    | 0x82   | timeout[23:0]                     | Set Chip in Rx mode                                                        |
/// | StopTimerOnPreamble      | 0x9F   | StopOnPreambleParam               | Stop Rx timeout on Sync Word/Header or preamble detection                   |
/// | SetRxDutyCycle           | 0x94   | rxPeriod[23:0], sleepPeriod[23:0] | Store values of RTC setup for listen mode and if period parameter is not 0, set chip into RX mode |
/// | SetCad                   | 0xC5   | -                                 | Set chip into RX mode with passed CAD parameters                            |
/// | SetTxContinuousWave      | 0xD1   | -                                 | Set chip into TX mode with infinite carrier wave settings                   |
/// | SetTxInfinitePreamble    | 0xD2   | -                                 | Set chip into TX mode with infinite preamble settings                       |
/// | SetRegulatorMode         | 0x96   | regModeParam                      | Select LDO or DC_DC+LDO for CFG_XOSC, FS, RX or TX mode                     |
/// | Calibrate                | 0x89   | calibParam                        | Calibrate the RC13, RC64, ADC, PLL, Image according to parameter            |
/// | CalibrateImage           | 0x98   | freq1, freq2                      | Launches an image calibration at the given frequencies                      |
/// | SetPaConfig              | 0x95   | paDutyCycle, HpMax, deviceSel, paLUT | Configure the Duty Cycle, Max output power, device for the PA for SX1261 or SX1262 |
/// | SetRxTxFallbackMode      | 0x93   | fallbackMode                      | Defines into which mode the chip goes after a TX / RX done                  |
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum OperatingModes {
    /// 0x84
    SetSleep = 0x84,
    /// 0x80
    SetStandby = 0x80,
    /// 0xC1
    SetFs = 0xC1,
    /// 0x83
    SetTx = 0x83,
    /// 0x82
    SetRx = 0x82,
    /// 0x9F
    StopTimerOnPreamble = 0x9F,
    /// 0x94
    SetRxDutyCycle = 0x94,
    /// 0xC5
    SetCad = 0xC5,
    /// 0xD1
    SetTxContinuousWave = 0xD1,
    /// 0xD2
    SetTxInfinitePreamble = 0xD2,
    /// 0x96
    SetRegulatorMode = 0x96,
    /// 0x89
    Calibrate = 0x89,
    /// 0x98
    CalibrateImage = 0x98,
    /// 0x95
    SetPaConfig = 0x95,
    /// 0x93
    SetRxTxFallbackMode = 0x93,
}

impl From<OperatingModes> for u8 {
    fn from(val: OperatingModes) -> Self {
        val as u8
    }
}
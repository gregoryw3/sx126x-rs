//! Wrapper for modem configuration parameters
use crate::calc_rf_freq;

use super::op::*;

/// Configuration parameters.
/// Used to initialize the SX126x modem
pub struct Config {
    /// Packet type
    pub packet_type: PacketType,
    /// LoRa sync word
    pub sync_word: u16,
    /// Calibration parameters
    pub calib_param: CalibParam,
    /// Modulation parameters
    pub mod_params: ModParams,
    /// Power-amplifier configuration
    pub pa_config: PaConfig,
    /// Packet parameters. Set tot none if you want to configure
    /// these later
    pub packet_params: Option<PacketParams>,
    /// TX parameters
    pub tx_params: TxParams,
    /// DIO1 IRQ mask
    pub dio1_irq_mask: IrqMask,
    /// DIO2 IRQ mask
    pub dio2_irq_mask: IrqMask,
    /// DIO3 IRW mask
    pub dio3_irq_mask: IrqMask,
    /// RF freq in MHz
    pub rf_frequency: f32,
    /// Crystal frequency
    pub freq_xtal: f32,
    /// True RF frequency calculated from `rf_frequency` and `freq_xtal` using `calc_rf_freq`
    pub(crate) rf_freq: u32,
    /// TCXO options. Set to None if not using TCXO
    pub tcxo_opts: Option<(TcxoVoltage, TcxoDelay)>,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            packet_type: PacketType::LoRa,
            sync_word: 0x3444,
            calib_param: CalibParam::default(),
            mod_params: ModParams::default(),
            pa_config: PaConfig::default(),
            packet_params: Some(PacketParams::default()),
            tx_params: TxParams::default(),
            dio1_irq_mask: IrqMask::default(),
            dio2_irq_mask: IrqMask::default(),
            dio3_irq_mask: IrqMask::default(),
            rf_frequency: 915.0,
            freq_xtal: 32.0 as f32,
            rf_freq: 0, // Temporary value
            tcxo_opts: None,
        };
        config.rf_freq = calc_rf_freq(config.rf_frequency, config.freq_xtal);
        config
    }
}

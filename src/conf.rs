//! Wrapper for modem configuration parameters
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
    /// RF freq, calculated using crate::calc_rf_freq
    pub rf_freq: u32,
    /// RF frequency in MHz
    pub rf_frequency: u32,
    /// TCXO options. Set to None if not using TCXO
    pub tcxo_opts: Option<(TcxoVoltage, TcxoDelay)>,
}

impl Default for Config {
    fn default() -> Self {
        let rf_freq = crate::calc_rf_freq(905.2, 32.0);
        let config = Config {
            packet_type: PacketType::LoRa,
            pa_config: PaConfig::default()
                .set_pa_duty_cycle(0x04)
                .set_hp_max(0x07)
                .set_device_sel(rxtx::DeviceSel::SX1262) // SX1262
                .set_enable_pa_clamp_fix(true),
            tx_params: TxParams::default()
                .set_power_dbm(0) // 0 dBm
                .set_ramp_time(RampTime::Ramp200u),
            mod_params: ModParams::from(
                LoraModParams::default()
                    // IREC will tell us what to set here
                    .set_bandwidth(LoRaBandWidth::BW250)
                    .set_coding_rate(LoraCodingRate::CR4_7)
                    .set_spread_factor(LoRaSpreadFactor::SF9)
                    .set_low_dr_opt(false),
            ),
            packet_params: Option::from(PacketParams::from(
                LoRaPacketParams::default()
                    .set_preamble_len(8)
                    .set_header_type(LoRaHeaderType::VarLen)
                    .set_payload_len(255)
                    .set_crc_type(LoRaCrcType::CrcOn)
                    // This flags the message as uplink (standard) or downlink (inverted)
                    .set_invert_iq(LoRaInvertIq::Standard),
            )),
            dio1_irq_mask: IrqMask::all(),
            dio2_irq_mask: IrqMask::none(),
            dio3_irq_mask: IrqMask::none(),
            tcxo_opts: Some((TcxoVoltage::Volt3_3, TcxoDelay::from_ms(1))),
            calib_param: CalibParam::new(
                true,
                true,
                true,
                true,
                true,
                true,
                true),
            sync_word: 0x12, // Private network 0x1424
            // sync_word: 0x34, // Public network 0x3444
            rf_frequency: rf_freq,
            rf_freq,
        };
        config
    }
}

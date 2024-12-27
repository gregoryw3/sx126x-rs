#![allow(async_fn_in_trait)]

pub(crate) mod err;
pub mod wait;

use crate::conf::Config;
use crate::op::*;
use crate::reg::*;
use crate::sx::wait::*;

use core::convert::Infallible;
use core::convert::TryInto;

use embedded_hal::digital::OutputPin;
// use embedded_hal::spi::Operation;
use embedded_hal_async::spi::SpiDevice;
use embedded_hal_async::spi::Operation;

use err::SpiError;
use err::PinError;

const NOP: u8 = 0x00;

/// Calculates the rf_freq value that should be passed to SX126x::set_rf_frequency
/// based on the desired RF frequency and the XTAL frequency.
///
/// Example calculation for 868MHz:
/// 13.4.1.: RFfrequecy = (RFfreq * Fxtal) / 2^25 = 868M
/// -> RFfreq =
/// -> RFfrequecy ~ ((RFfreq >> 12) * (Fxtal >> 12)) >> 1
pub fn calc_rf_freq(rf_frequency: f32, f_xtal: f32) -> u32 {
    (rf_frequency * (33554432. / f_xtal)) as u32
}

/// Wrapper around a Semtech SX1261/62 LoRa modem
/// 
/// [Datasheet (Semtech)](https://semtech.my.salesforce.com/sfc/p/#E0000000JelG/a/2R000000Un7F/yT.fKdAr9ZAo3cJLc4F2cBdUsMftpT2vsOICP7NmvMo)
/// 
/// [Datasheet (Mouser)](https://www.mouser.com/ds/2/761/DS_SX1261-2_V1.1-1307803.pdf)
/// 
/// | Pin Number | Pin Name | Type (I = input, O = Output) | Description |
/// |------------|----------|------------------------------|-------------|
/// | 0          | GND      | -                            | Exposed Ground pad |
/// | 1          | VDD_IN   | I                            | Input voltage for power amplifier regulator, VR_PA. SX1261: connected to pin 7. SX1262: connected to pin 10 |
/// | 2          | GND      | -                            | Ground |
/// | 3          | XTA      | -                            | Crystal oscillator connection, can be used to input external reference clock |
/// | 4          | XTB      | -                            | Crystal oscillator connection |
/// | 5          | GND      | -                            | Ground |
/// | 6          | DIO3     | I/O                          | Multipurpose digital I/O - external TCXO supply voltage |
/// | 7          | VREG     | O                            | Regulated output voltage from the internal regulator LDO / DC-DC |
/// | 8          | GND      | -                            | Ground |
/// | 9          | DCC_SW   | O                            | DC-DC Switcher Output |
/// | 10         | VBAT     | I                            | Supply for the RFIC |
/// | 11         | VBAT_IO  | I                            | Supply for the Digital I/O interface pins (except DIO3) |
/// | 12         | DIO2     | I/O                          | Multipurpose digital I/O / RF Switch control |
/// | 13         | DIO1     | I/O                          | Multipurpose digital IO |
/// | 14         | BUSY     | I/O                          | Busy indicator |
/// | 15         | NRESET   | I/O                          | Reset signal, active low |
/// | 16         | MISO     | O                            | SPI slave output |
/// | 17         | MOSI     | I                            | SPI slave input |
/// | 18         | SCK      | I                            | SPI clock |
/// | 19         | NSS      | I                            | SPI Slave Select |
/// | 20         | GND      | -                            | Ground |
/// | 21         | RFI_P    | I                            | RF receiver input |
/// | 22         | RFI_N    | I                            | RF receiver input |
/// | 23         | RFO      | O                            | RF transmitter output (SX1261 low power PA or SX1262 high power PA) |
/// | 24         | VR_PA    | -                            | Regulated power amplifier supply |
/// 
pub struct SX126x<SPI, NRST, BUSY, ANT, DIO1>
where
    SPI: SpiDevice,
    NRST: OutputPin<Error = Infallible>,
    BUSY: AnyWait,
    ANT: OutputPin<Error = Infallible>,
    DIO1: AnyWait,
{
    /// SpiDevice contains pins: MISO, MOSI, SCK, CS (NSS)
    spi: SPI,
    nrst_pin: NRST,
    busy_pin: BUSY,
    ant_pin: ANT,
    dio1_pin: DIO1,
}

impl<SPI, NRST, BUSY, ANT, DIO1> SX126x<SPI, NRST, BUSY, ANT, DIO1>
where
    SPI: SpiDevice,
    NRST: OutputPin<Error = Infallible>,
    BUSY: AnyWait,
    ANT: OutputPin<Error = Infallible>,
    DIO1: AnyWait,
{
    // Create a new SX126x
    pub fn new(spi: SPI, nrst_pin: NRST, busy_pin: BUSY, ant_pin: ANT, dio1_pin: DIO1) -> Self {
        // let (nrst_pin, busy_pin, ant_pin, dio1_pin) = pins;
        Self {
            spi,
            nrst_pin,
            busy_pin,
            ant_pin,
            dio1_pin,
        }
    }

    // Initialize and configure the SX126x using the provided Config
    pub async fn init_async(&mut self, conf: Config) -> Result<(), Infallible> {
        // Reset the sx
        self.reset().await;
        self.wait_on_busy_async().await.map_err(|_| SpiError::BusError);

        // 1. If not in STDBY_RC mode, then go to this mode with the command SetStandby(...)
        self.set_standby(StandbyConfig::StbyRc).await;
        self.wait_on_busy_async().await;

        // 2. Define the protocol (LoRa® or FSK) with the command SetPacketType(...)
        self.set_packet_type(conf.packet_type).await;
        self.wait_on_busy_async().await;

        // 3. Define the RF frequency with the command SetRfFrequency(...)
        self.set_rf_frequency(conf.rf_freq).await;
        self.wait_on_busy_async().await;

        if let Some((tcxo_voltage, tcxo_delay)) = conf.tcxo_opts {
            self.set_dio3_as_tcxo_ctrl(tcxo_voltage, tcxo_delay).await;
            self.wait_on_busy_async().await;
        }

        // Calibrate
        self.calibrate(conf.calib_param).await;
        self.wait_on_busy_async().await;
        self.calibrate_image(CalibImageFreq::from_rf_frequency(conf.rf_freq)).await;
        self.wait_on_busy_async().await;

        // 4. Define the Power Amplifier configuration with the command SetPaConfig(...)
        self.set_pa_config(conf.pa_config).await;
        self.wait_on_busy_async().await;

        // 5. Define output power and ramping time with the command SetTxParams(...)
        self.set_tx_params(conf.tx_params).await;
        self.wait_on_busy_async().await;

        // 6. Define where the data payload will be stored with the command SetBufferBaseAddress(...)
        self.set_buffer_base_address(0x00, 0x00).await;
        self.wait_on_busy_async().await;

        // 7. Send the payload to the data buffer with the command WriteBuffer(...)
        // This is done later in SX126x::write_bytes

        // 8. Define the modulation parameter according to the chosen protocol with the command SetModulationParams(...) 1
        self.set_mod_params(conf.mod_params).await;
        self.wait_on_busy_async().await;

        // 9. Define the frame format to be used with the command SetPacketParams(...) 2
        if let Some(packet_params) = conf.packet_params {
            self.set_packet_params(packet_params).await;
            self.wait_on_busy_async().await;
        }

        // 10. Configure DIO and IRQ: use the command SetDioIrqParams(...) to select TxDone IRQ and map this IRQ to a DIO (DIO1,
        // DIO2 or DIO3)
        self.set_dio_irq_params(
            conf.dio1_irq_mask,
            conf.dio1_irq_mask,
            conf.dio2_irq_mask,
            conf.dio3_irq_mask,
        )
        .await;
        self.wait_on_busy_async().await;
        self.set_dio2_as_rf_switch_ctrl(true).await;
        self.wait_on_busy_async().await;

        // 11. Define Sync Word value: use the command WriteReg(...) to write the value of the register via direct register access
        self.set_sync_word(conf.sync_word).await;
        self.wait_on_busy_async().await;

        // The rest of the steps are done by the user
        Ok(())
    }

    /// Set the LoRa Sync word
    /// Use 0x3444 for public networks like TTN
    /// Use 0x1424 for private networks
    pub async fn set_sync_word(&mut self, sync_word: u16) -> Result<(), SpiError> {
        self.write_register(Register::LoRaSyncWordMsb, &sync_word.to_be_bytes())
            .await
    }

    /// Set the modem packet type, which can be either GFSK of LoRa
    /// Note: GFSK is not fully supported by this crate at the moment
    pub async fn set_packet_type(
        &mut self,
        packet_type: PacketType,
    ) -> Result<(), SpiError> {
        self.spi
            .write(&[0x8A, packet_type as u8])
            .await
            .map_err(|_| SpiError::Write)
    }

    /// Put the modem in standby mode
    pub async fn set_standby(
        &mut self,

        standby_config: StandbyConfig,
    ) -> Result<(), SpiError> {
        self.spi
            .write(&[0x80, standby_config as u8])
            .await
            .map_err(|_| SpiError::Write)
    }

    /// Get the current status of the modem
    pub async fn get_status(&mut self) -> Result<Status, SpiError> {
        let mut result = [0xC0, NOP];
        self.spi
            .transfer_in_place(&mut result)
            .await
            .map_err(|_| SpiError::Transfer)?;

        Ok(result[1].into())
    }

    pub async fn set_fs(&mut self) -> Result<(), SpiError> {
        self.spi.write(&[0xC1])
        .await
        .map_err(|_| SpiError::Transfer)?;
        Ok(())
    }

    pub async fn get_stats(&mut self) -> Result<Stats, SpiError> {
        let mut result = [0x10, NOP, NOP, NOP, NOP, NOP, NOP, NOP];
        self.spi
            .transfer_in_place(&mut result)
            .await
            .map_err(|_| SpiError::Transfer)?;

        Ok(TryInto::<[u8; 7]>::try_into(&result[1..]).unwrap().into())
    }

    /// Calibrate image
    /// 
    /// | Byte | Data from host       |
    /// |------|----------------------|
    /// | 0    | Opcode = 0x98        |
    /// | 1    | freq1                |
    /// | 2    | freq2                |
    /// 
    /// The frequency is calculated as follows:
    /// 
    /// | Frequency Band (MHz) | Freq1 | Freq2 |
    /// |----------------------|-------|-------|
    /// | 430 - 440            | 0x6B  | 0x6F  |
    /// | 470 - 510            | 0x75  | 0x81  |
    /// | 779 - 787            | 0xC1  | 0xC5  |
    /// | 863 - 870            | 0xD7  | 0xDB  |
    /// | 902 - 928            | 0xE1 (default) | 0xE9 (default) |
    pub async fn calibrate_image(
        &mut self,

        freq: CalibImageFreq,
    ) -> Result<(), SpiError> {
        let freq: [u8; 2] = freq.into();
        let mut ops = [Operation::Write(&[0x98]), Operation::Write(&freq)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
    }

    /// Calibrate modem
    pub async fn calibrate(
        &mut self,
        calib_param: CalibParam,
    ) -> Result<(), SpiError> {
        self.spi
            .write(&[0x89, calib_param.into()])
            .await
            .map_err(|_| SpiError::Write)
    }

    /// Write data into a register
    pub async fn write_register(
        &mut self,
        register: Register,
        data: &[u8],
    ) -> Result<(), SpiError> {
        let start_addr = (register as u16).to_be_bytes();
        let mut ops = [
            Operation::Write(&[0x0D]),
            Operation::Write(&start_addr),
            Operation::Write(data),
        ];

        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)?;
        Ok(())
    }

    /// Read data from a register
    pub async fn read_register(
        &mut self,
        start_addr: u16,
        result: &mut [u8],
    ) -> Result<(), SpiError> {
        debug_assert!(!result.is_empty());
        let start_addr = start_addr.to_be_bytes();

        let mut ops = [
            Operation::Write(&[0x1D]),
            Operation::Write(&start_addr),
            Operation::Read(result),
        ];

        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Transfer)?;
        Ok(())
    }

    /// Write data into the buffer at the defined offset
    pub async fn write_buffer(
        &mut self,
        offset: u8,
        data: &[u8],
    ) -> Result<(), SpiError> {
        let header = [0x0E, offset];
        let mut ops = [Operation::Write(&header), Operation::Write(data)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Read data from the data from the defined offset
    pub async fn read_buffer(
        &mut self,
        offset: u8,
        result: &mut [u8],
    ) -> Result<(), SpiError> {
        let header = [0x1E, offset, NOP];
        let mut ops = [Operation::Write(&header), Operation::Read(result)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Transfer)
            
    }

    /// Configure the dio2 pin as RF control switch
    pub async fn set_dio2_as_rf_switch_ctrl(
        &mut self,
        enable: bool,
    ) -> Result<(), SpiError> {
        self.spi
            .write(&[0x9D, enable as u8])
            .await
            .map_err(|_| SpiError::Write)
            
    }

    pub async fn get_packet_status(&mut self) -> Result<PacketStatus, SpiError> {
        let header = [0x14, NOP];
        let mut result = [NOP; 3];
        let mut ops = [Operation::Write(&header), Operation::Read(&mut result)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Transfer)?;

        Ok(result.into())
    }

    /// Configure the dio3 pin as TCXO control switch
    pub async fn set_dio3_as_tcxo_ctrl(
        &mut self,
        tcxo_voltage: TcxoVoltage,
        tcxo_delay: TcxoDelay,
    ) -> Result<(), SpiError> {
        let header = [0x97, tcxo_voltage as u8];
        let tcxo_delay: [u8; 3] = tcxo_delay.into();
        let mut ops = [Operation::Write(&header), Operation::Write(&tcxo_delay)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Clear device error register
    pub async fn clear_device_errors(&mut self) -> Result<(), SpiError> {
        self.spi
            .write(&[0x07, NOP, NOP])
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Get current device errors
    pub async fn get_device_errors(&mut self) -> Result<DeviceErrors, SpiError> {
        let mut result = [0x17, NOP, NOP, NOP];
        self.spi
            .transfer_in_place(&mut result)
            .await
            .map_err(|_| SpiError::Transfer)?;
        Ok(DeviceErrors::from(u16::from_le_bytes(
            result[2..].try_into().unwrap(),
        )))
    }

    /// Reset the device py pulling nrst low for a while
    pub async fn reset(&mut self) -> Result<(), SpiError> {
        critical_section::with(|_| 
            self.nrst_pin
                .set_low()
                .map_err(|_| PinError::Output))
                .map_err(|_| SpiError::BusError)?;

        // 8.1: The pin should be held low for typically 100 μs for the Reset to happen
        self.spi
            .transaction(&mut [Operation::DelayNs(200_000)])
            .await
            .map_err(|_| SpiError::Write)?;

        critical_section::with(|_| 
            self.nrst_pin
                .set_high()
                .map_err(|_| PinError::Output))
                .map_err(|_| SpiError::BusError)?;

        Ok(())
    }

    /// Enable antenna
    pub async fn set_ant_enabled(&mut self, enabled: bool) -> Result<(), Infallible> {
        if enabled {
            self.ant_pin.set_high()
        } else {
            self.ant_pin.set_low()
        }
    }

    /// Configure IRQ
    pub async fn set_dio_irq_params(
        &mut self,

        irq_mask: IrqMask,
        dio1_mask: IrqMask,
        dio2_mask: IrqMask,
        dio3_mask: IrqMask,
    ) -> Result<(), SpiError> {
        let irq = (Into::<u16>::into(irq_mask)).to_be_bytes();
        let dio1 = (Into::<u16>::into(dio1_mask)).to_be_bytes();
        let dio2 = (Into::<u16>::into(dio2_mask)).to_be_bytes();
        let dio3 = (Into::<u16>::into(dio3_mask)).to_be_bytes();
        let mut ops = [
            Operation::Write(&[0x08]),
            Operation::Write(&irq),
            Operation::Write(&dio1),
            Operation::Write(&dio2),
            Operation::Write(&dio3),
        ];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Transfer)
            
    }

    /// Get the current IRQ status
    pub async fn get_irq_status(&mut self) -> Result<IrqStatus, SpiError> {
        let mut status = [NOP, NOP, NOP];
        let mut ops = [Operation::Write(&[0x12]), Operation::Read(&mut status)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Transfer)?;
        let irq_status: [u8; 2] = [status[1], status[2]];
        Ok(u16::from_be_bytes(irq_status).into())
    }

    /// Clear the IRQ status
    pub async fn clear_irq_status(
        &mut self,
        mask: IrqMask,
    ) -> Result<(), SpiError> {
        let mask = Into::<u16>::into(mask).to_be_bytes();
        let mut ops = [Operation::Write(&[0x02]), Operation::Write(&mask)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Put the device in TX mode. It will start sending the data written in the buffer,
    /// starting at the configured offset
    pub async fn set_tx(
        &mut self,
        timeout: RxTxTimeout,
    ) -> Result<Status, SpiError> {
        let mut buf = [0x83u8; 4];
        let timeout: [u8; 3] = timeout.into();
        buf[1..].copy_from_slice(&timeout);

        self.spi
            .transfer_in_place(&mut buf)
            .await
            .map_err(|_| SpiError::Transfer)?;
        Ok(timeout[1].into())
    }

    pub async fn set_rx(
        &mut self,
        timeout: RxTxTimeout,
    ) -> Result<Status, SpiError> {
        let mut buf = [0x82u8; 4];
        let timeout: [u8; 3] = timeout.into();
        buf[1..].copy_from_slice(&timeout);

        self.spi.write(&buf).await.map_err(|_| SpiError::Transfer)?;
        Ok(timeout[0].into())
    }

    /// Set packet parameters
    pub async fn set_packet_params(
        &mut self,
        params: PacketParams,
    ) -> Result<(), SpiError> {
        let params: [u8; 9] = params.into();
        let mut ops = [Operation::Write(&[0x8C]), Operation::Write(&params)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Set modulation parameters
    pub async fn set_mod_params(
        &mut self,
        params: ModParams,
    ) -> Result<(), SpiError> {
        let params: [u8; 8] = params.into();
        let mut ops = [Operation::Write(&[0x8B]), Operation::Write(&params)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Set TX parameters
    pub async fn set_tx_params(
        &mut self,
        params: TxParams,
    ) -> Result<(), SpiError> {
        let params: [u8; 2] = params.into();
        let mut ops = [Operation::Write(&[0x8E]), Operation::Write(&params)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Set RF frequency. This writes the passed rf_freq directly to the modem.
    /// Use sx1262::calc_rf_freq to calulate the correct value based
    /// On the XTAL frequency and the desired RF frequency
    pub async fn set_rf_frequency(
        &mut self,
        rf_freq: u32,
    ) -> Result<(), SpiError> {
        let rf_freq = rf_freq.to_be_bytes();
        let mut ops = [Operation::Write(&[0x86]), Operation::Write(&rf_freq)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Set Power Amplifier configuration
    pub async fn set_pa_config(
        &mut self,
        pa_config: PaConfig,
    ) -> Result<(), SpiError> {
        let pa_config: [u8; 4] = pa_config.into();
        let mut ops = [Operation::Write(&[0x95]), Operation::Write(&pa_config)];
        self.spi
            .transaction(&mut ops)
            .await
            .map_err(|_| SpiError::Write)
            
    }

    /// Configure the base addresses in the buffer
    pub async fn set_buffer_base_address(
        &mut self,
        tx_base_addr: u8,
        rx_base_addr: u8,
    ) -> Result<(), SpiError> {
        self.spi
            .write(&[0x8F, tx_base_addr, rx_base_addr])
            .await
            .map_err(|_| SpiError::Write)
    }

    /// High level method to send a message. This methods writes the data in the buffer,
    /// puts the device in TX mode, and waits until the devices
    /// is done sending the data or a timeout occurs.
    /// Please note that this method updates the packet params
    pub async fn write_bytes_async(
        &mut self,
        data: &[u8],
        timeout: RxTxTimeout,
        preamble_len: u16,
        crc_type: packet::LoRaCrcType,
    ) -> Result<Status, SpiError> {
        use packet::LoRaPacketParams;
        // Write data to buffer
        self.write_buffer(0x00, data).await?;

        // Set packet params
        let params = LoRaPacketParams::default()
            .set_preamble_len(preamble_len)
            .set_payload_len(data.len() as u8)
            .set_crc_type(crc_type)
            .into();

        self.set_packet_params(params).await?;

        // Set tx mode
        let status = self.set_tx(timeout).await?;
        // Wait for busy line to go low
        self.wait_on_busy_async().await.map_err(|_| SpiError::BusError)?;
        // Wait on dio1 going high
        self.wait_on_dio1_async().await.map_err(|_| SpiError::BusError)?;
        // Clear IRQ
        self.clear_irq_status(IrqMask::all()).await?;
        // Write completed!
        Ok(status)
    }

    /// Get Rx buffer status, containing the length of the last received packet
    /// and the address of the first byte received.
    pub async fn get_rx_buffer_status(
        &mut self,
    ) -> Result<RxBufferStatus, SpiError> {
        let mut result = [0x13, NOP, NOP, NOP];
        self.spi
            .transfer_in_place(&mut result)
            .await
            .map_err(|_| SpiError::Transfer)?;
        Ok(TryInto::<[u8; 2]>::try_into(&result[2..]).unwrap().into())
    }

    /// Busily wait for the busy pin to go low
    pub async fn wait_on_busy_async(&mut self) -> Result<(), PinError> {
        self.spi
            .transaction(&mut [Operation::DelayNs(1000)])
            .await
            .map_err(|_| SpiError::Transfer)
            .map_err(|_| PinError::Output)?;

        self.busy_pin
            .anywait_for_low()
            .await
            .map_err(|_| PinError::Input)
    }

    /// Busily wait for the dio1 pin to go high
    pub async fn wait_on_dio1_async(&mut self) -> Result<(), PinError> {
        self.dio1_pin
            .anywait_for_high()
            .await
            .map_err(|_| PinError::Input)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_calc_rf_freq() {
//         let f_xtal = 32.0;
//         let rf_freq = 868.0;
//         let expected = 0x6C_8A_00;
//         let result = calc_rf_freq(rf_freq, f_xtal);
//         assert_eq!(result, expected);
//     }
// }
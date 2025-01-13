#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum PacketType {
    GFSK = 0x00,
    LoRa = 0x01,
}

impl core::fmt::Debug for PacketType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PacketType::GFSK => write!(f, "GFSK"),
            PacketType::LoRa => write!(f, "LoRa"),
        }
    }
}

impl From<u8> for PacketType {
    fn from(b: u8) -> Self {
        match b {
            0x00 => PacketType::GFSK,
            0x01 => PacketType::LoRa,
            _ => unreachable!(),
        }
    }
}

pub struct PacketParams {
    inner: [u8; 9],
}

impl From<PacketParams> for [u8; 9] {
    fn from(val: PacketParams) -> Self {
        val.inner
    }
}

impl From<&PacketParams> for [u8; 9] {
    fn from(val: &PacketParams) -> Self {
        val.inner
    }
}

pub use lora::*;

mod lora {
    use super::PacketParams;

    #[repr(u8)]
    #[derive(Copy, Clone)]
    pub enum LoRaHeaderType {
        /// Variable length packet (explicit header)
        VarLen = 0x00,
        /// Fixed length packet (implicit header)
        FixedLen = 0x01,
    }

    #[repr(u8)]
    #[derive(Copy, Clone)]
    pub enum LoRaCrcType {
        /// CRC off
        CrcOff = 0x00,
        /// CRC on
        CrcOn = 0x01,
    }

    /// Only used in FSK mode
    #[repr(u8)]
    #[derive(Copy, Clone)]
    pub enum LoRaCrcTypeConfig {
        /// No CRC
        CrcOff = 0x01,
        /// CRC computed on 1 byte
        Crc1Byte = 0x00,
        /// CRC computed on 2 bytes
        Crc2Bytes = 0x02,
        /// CRC computed on 1 byte, inverted
        Crc1ByteInv = 0x04,
        /// CRC computed on 2 bytes, inverted
        Crc2BytesInv = 0x06,
    }

    #[repr(u8)]
    #[derive(Copy, Clone)]
    pub enum LoRaInvertIq {
        /// Standard IQ setup
        Standard = 0x00,
        /// Inverted IQ setup
        Inverted = 0x01,
    }

    pub struct LoRaPacketParams {
        /// preamble length: number of symbols sent as preamble
        /// The preamble length is a 16-bit value which represents
        /// the number of LoRa® symbols which will be sent by the radio.
        pub preamble_len: u16, // 1, 2
        /// Header type. When the byte headerType is at 0x00,
        /// the payload length, coding rate and the header
        /// CRC will be added to the LoRa® header and transported
        /// to the receiver.
        pub header_type: LoRaHeaderType, // 3
        /// Size of the payload (in bytes) to transmit or maximum size of the
        /// payload that the receiver can accept.
        pub payload_len: u8, // 4
        /// CRC type
        pub crc_type: LoRaCrcType, // 5
        /// Invert IW
        pub invert_iq: LoRaInvertIq,
    }

    impl From<LoRaPacketParams> for PacketParams {
        fn from(val: LoRaPacketParams) -> Self {
            let preamble_len = val.preamble_len.to_be_bytes();

            PacketParams {
                inner: [
                    preamble_len[0],
                    preamble_len[1],
                    val.header_type as u8,
                    val.payload_len,
                    val.crc_type as u8,
                    val.invert_iq as u8,
                    0x00,
                    0x00,
                    0x00,
                ],
            }
        }
    }

    impl From<&LoRaPacketParams> for PacketParams {
        fn from(val: &LoRaPacketParams) -> Self {
            let preamble_len = val.preamble_len.to_be_bytes();

            PacketParams {
                inner: [
                    preamble_len[0],
                    preamble_len[1],
                    val.header_type as u8,
                    val.payload_len,
                    val.crc_type as u8,
                    val.invert_iq as u8,
                    0x00,
                    0x00,
                    0x00,
                ],
            }
        }
    }

    impl Default for LoRaPacketParams {
        fn default() -> Self {
            Self {
                preamble_len: 0x0008,
                header_type: LoRaHeaderType::VarLen,
                payload_len: 0x00,
                crc_type: LoRaCrcType::CrcOff,
                invert_iq: LoRaInvertIq::Standard,
            }
        }
    }

    impl LoRaPacketParams {
        /// Valid from 0x0001 to 0xFFFF
        pub fn set_preamble_len(mut self, preamble_len: u16) -> Self {
            self.preamble_len = preamble_len;
            self
        }

        pub fn set_header_type(mut self, header_type: LoRaHeaderType) -> Self {
            self.header_type = header_type;
            self
        }

        pub fn set_payload_len(mut self, payload_len: u8) -> Self {
            self.payload_len = payload_len;
            self
        }

        pub fn set_crc_type(mut self, crc_type: LoRaCrcType) -> Self {
            self.crc_type = crc_type;
            self
        }

        pub fn set_invert_iq(mut self, invert_iq: LoRaInvertIq) -> Self {
            self.invert_iq = invert_iq;
            self
        }
    }
}

impl Default for PacketParams {
    fn default() -> Self {
        LoRaPacketParams::default().into()
    }
}

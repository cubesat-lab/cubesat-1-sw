#![no_std]

use crc::{Algorithm, Crc};

pub const SFP_FRAME_MARKER: [u8; 2] = [0xAA, 0xAA];
pub const SFP_FRAME_MARKER_SIZE: usize = SFP_FRAME_MARKER.len();
pub const SFP_DATA_LEN_MIN: usize = 1;
pub const SFP_DATA_LEN_MAX: usize = 1024;
pub const SFP_LEN_SIZE: usize = 2;
pub const SFP_CRC_SIZE: usize = 2;
pub const SFP_FRAME_LEN_MAX: usize =
    SFP_FRAME_MARKER_SIZE + SFP_LEN_SIZE + SFP_DATA_LEN_MAX + SFP_CRC_SIZE;

/// CRC-16/CCITT-FALSE algorithm parameters (a.k.a. CRC-16/IBM-3740)
/// Reference: <https://reveng.sourceforge.io/crc-catalogue/16.htm>
pub const CRC_16_CCITT_FALSE: Algorithm<u16> = Algorithm {
    width: 16,
    poly: 0x1021,
    init: 0xFFFF,
    refin: false,
    refout: false,
    xorout: 0x0000,
    check: 0x29B1,
    residue: 0x0000,
};

pub fn crc16(data: &[u8]) -> u16 {
    const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_CCITT_FALSE);
    CRC.checksum(data)
}

pub struct SfpFrame<'a> {
    pub data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub enum SfpFrameError {
    InvalidLength,
}

impl<'a> SfpFrame<'a> {
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SfpFrameError> {
        let length = self.data.len();
        let mut index = 0;

        if !(SFP_DATA_LEN_MIN..=SFP_DATA_LEN_MAX).contains(&length) {
            return Err(SfpFrameError::InvalidLength);
        }

        buffer[index..index + SFP_FRAME_MARKER_SIZE].copy_from_slice(&SFP_FRAME_MARKER);
        index += SFP_FRAME_MARKER_SIZE;

        buffer[index..index + SFP_LEN_SIZE].copy_from_slice(&(length as u16).to_be_bytes());
        index += SFP_LEN_SIZE;

        buffer[index..index + length].copy_from_slice(self.data);
        index += length;

        let crc = crc16(&buffer[SFP_FRAME_MARKER_SIZE..index]);
        buffer[index..index + SFP_CRC_SIZE].copy_from_slice(&crc.to_be_bytes());
        index += SFP_CRC_SIZE;

        // Return the total length of the serialized frame
        Ok(index)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseState {
    WaitMarker,
    ReadLen,
    ReadData,
    ReadCrc,
}

#[allow(clippy::large_enum_variant)]
pub enum ParseResult<'a> {
    Idle,
    Parsing,
    LengthError,
    CrcError,
    Success { length: u16, data: &'a [u8] },
}

pub struct SfpFrameParser<'a> {
    buffer: &'a [u8],
    counter: usize,
    length: u16,
    state: ParseState,
}

impl Default for SfpFrameParser<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SfpFrameParser<'a> {
    pub fn new() -> Self {
        Self {
            buffer: &[],
            counter: 0,
            length: 0,
            state: ParseState::WaitMarker,
        }
    }

    pub fn reset(&mut self) {
        self.buffer = &[];
        self.counter = 0;
        self.length = 0;
        self.state = ParseState::WaitMarker;
    }

    pub fn set_frame_start(&mut self, buffer: &'a [u8]) {
        self.reset();
        self.buffer = buffer;
    }

    pub fn parse_byte(&mut self, byte: u8) -> ParseResult {
        self.counter += 1;

        match &mut self.state {
            ParseState::WaitMarker => {
                let marker_index = self.counter - 1;
                if byte == SFP_FRAME_MARKER[marker_index] {
                    match marker_index {
                        0 => {
                            self.state = ParseState::WaitMarker;
                        }
                        1 => {
                            self.state = ParseState::ReadLen;
                        }
                        _ => unreachable!(),
                    }
                } else {
                    self.reset();
                    return ParseResult::Idle;
                }
            }
            ParseState::ReadLen => {
                if self.counter == (SFP_FRAME_MARKER_SIZE + SFP_LEN_SIZE) {
                    let mut length_buf: [u8; SFP_LEN_SIZE] = [0; SFP_LEN_SIZE];
                    length_buf.copy_from_slice(
                        &self.buffer[SFP_FRAME_MARKER_SIZE..SFP_FRAME_MARKER_SIZE + SFP_LEN_SIZE],
                    );
                    let length = u16::from_be_bytes(length_buf) as usize;

                    if !(SFP_DATA_LEN_MIN..=SFP_DATA_LEN_MAX).contains(&length) {
                        self.reset();
                        return ParseResult::LengthError;
                    } else {
                        self.length = length as u16;
                        self.state = ParseState::ReadData;
                    }
                }
            }
            ParseState::ReadData => {
                if (self.counter - (SFP_FRAME_MARKER_SIZE + SFP_LEN_SIZE)) == self.length.into() {
                    self.state = ParseState::ReadCrc;
                }
            }
            ParseState::ReadCrc => {
                let index_crc = SFP_FRAME_MARKER_SIZE + SFP_LEN_SIZE + self.length as usize;

                if self.counter == (index_crc + SFP_CRC_SIZE) {
                    let mut crc_buf: [u8; SFP_CRC_SIZE] = [0; SFP_CRC_SIZE];
                    crc_buf.copy_from_slice(&self.buffer[index_crc..self.counter]);

                    let received_crc = u16::from_be_bytes(crc_buf);
                    let calculated_crc = crc16(&self.buffer[SFP_FRAME_MARKER_SIZE..index_crc]);

                    let result_data = &self.buffer[SFP_FRAME_MARKER_SIZE + SFP_LEN_SIZE..index_crc];
                    let length = self.length;
                    self.reset();

                    if received_crc == calculated_crc {
                        return ParseResult::Success {
                            length,
                            data: result_data,
                        };
                    } else {
                        return ParseResult::CrcError;
                    }
                }
            }
        }

        ParseResult::Parsing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16() {
        let data = [0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39];
        let crc = crc16(&data);
        assert_eq!(crc, 0x29B1);
    }

    #[test]
    fn test_sfp_frame_serialize() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let frame = SfpFrame { data: &data };
        let mut buffer = [0u8; SFP_FRAME_LEN_MAX];
        let len = frame.serialize(&mut buffer).ok().unwrap();
        assert_eq!(len, data.len() + 6); // 2 bytes for marker, 2 bytes for length, 2 bytes for CRC

        // Verify the serialized frame structure
        assert_eq!(&buffer[0..2], &SFP_FRAME_MARKER); // Frame marker
        assert_eq!(
            u16::from_be_bytes([buffer[2], buffer[3]]) as usize,
            data.len()
        ); // Length
        assert_eq!(&buffer[4..4 + data.len()], &data); // Data
        let crc = crc16(&buffer[2..4 + data.len()]);
        assert_eq!(
            u16::from_be_bytes([buffer[4 + data.len()], buffer[5 + data.len()]]),
            crc
        ); // CRC
    }

    #[test]
    fn test_sfp_frame_serialize_error() {
        let data: [u8; 1025] = [0xAA; 1025];
        let frame = SfpFrame { data: &data };
        let mut buffer = [0u8; SFP_FRAME_LEN_MAX];
        let result = frame.serialize(&mut buffer).err().unwrap();
        assert_eq!(result, SfpFrameError::InvalidLength);
    }

    #[test]
    fn test_sfp_frame_parser() {
        let test_data: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let frame: [u8; 14] = [
            0xAA, 0xAA, // Frame marker
            0x00, 0x08, // Length (8 bytes)
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, // Data (DEADBEEFCAFEBABE)
            0xFE, 0x42, // CRC (0xFE42)
        ];

        let mut counter = 0;
        let mut parser = SfpFrameParser::default();
        parser.set_frame_start(&frame);

        for byte in frame {
            counter += 1;

            match parser.parse_byte(byte) {
                ParseResult::Idle => {
                    panic!("Unexpected idle state at byte {}", counter);
                }
                ParseResult::Parsing => {
                    assert_eq!(counter, parser.counter);
                }
                ParseResult::LengthError => {
                    panic!("Unexpected length error at byte {}", counter);
                }
                ParseResult::CrcError => {
                    panic!("Unexpected CRC error at byte {}", counter);
                }
                ParseResult::Success { length, data } => {
                    assert_eq!(data, &test_data);
                    assert_eq!(length, test_data.len() as u16);
                }
            }
        }
    }

    #[test]
    fn test_sfp_frame_parser_within_other_data() {
        let test_data: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let frame: [u8; 24] = [
            0xB8, 0x6C, 0x7F, 0x19, 0x4B, // Other data
            0xAA, 0xAA, // Frame marker
            0x00, 0x08, // Length (8 bytes)
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, // Data (DEADBEEFCAFEBABE)
            0xFE, 0x42, // CRC (0xFE42)
            0x45, 0x09, 0x05, 0xA8, 0xC9, // Other data
        ];

        let mut counter = 0;
        let mut parser = SfpFrameParser::default();
        parser.set_frame_start(&frame);

        for byte in frame {
            counter += 1;

            match parser.parse_byte(byte) {
                ParseResult::Idle => {
                    parser.set_frame_start(&frame[counter..]);
                }
                ParseResult::Parsing => {
                    assert_eq!(counter, (parser.counter + 5).into());
                }
                ParseResult::LengthError => {
                    panic!("Unexpected length error at byte {}", counter);
                }
                ParseResult::CrcError => {
                    panic!("Unexpected CRC error at byte {}", counter);
                }
                ParseResult::Success { length, data } => {
                    assert_eq!(data, &test_data);
                    assert_eq!(length, test_data.len() as u16);
                }
            }
        }
    }

    #[test]
    fn test_sfp_frame_parser_with_bad_marker() {
        let _test_data: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let frame: [u8; 14] = [
            0xAA, 0xAB, // Frame marker
            0x00, 0x08, // Length (8 bytes)
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, // Data (DEADBEEFCAFEBABE)
            0xFE, 0x42, // CRC (0xFE42)
        ];

        let mut counter = 0;
        let mut parser = SfpFrameParser::default();
        parser.set_frame_start(&frame);

        for byte in frame {
            counter += 1;

            match parser.parse_byte(byte) {
                ParseResult::Idle => {
                    // The parser should reset to Idle state after a bad marker
                    assert_eq!(parser.state, ParseState::WaitMarker);

                    parser.set_frame_start(&frame[counter..]);
                }
                ParseResult::Parsing => {
                    if counter != 1 {
                        panic!("Unexpected parsing state at byte {}", counter);
                    }
                }
                ParseResult::LengthError => {
                    panic!("Unexpected length error at byte {}", counter);
                }
                ParseResult::CrcError => {
                    panic!("Unexpected CRC error at byte {}", counter);
                }
                ParseResult::Success { length, data } => {
                    panic!(
                        "Unexpected success state with data {:?} of length {:?}",
                        data, length
                    );
                }
            }
        }
    }

    #[test]
    fn test_sfp_frame_parser_with_bad_length() {
        let _test_data: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let frame: [u8; 14] = [
            0xAA, 0xAA, // Frame marker
            0x04, 0x01, // Length (1025 bytes)
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, // Data (DEADBEEFCAFEBABE)
            0xFE, 0x42, // CRC (0xFE42)
        ];

        let mut counter = 0;
        let mut parser = SfpFrameParser::default();
        parser.set_frame_start(&frame);

        for byte in frame {
            counter += 1;

            match parser.parse_byte(byte) {
                ParseResult::Idle => {
                    // The parser should reset to Idle state after a bad length
                    assert_eq!(parser.state, ParseState::WaitMarker);

                    parser.set_frame_start(&frame[counter..]);
                }
                ParseResult::Parsing => {
                    if counter > 4 {
                        panic!("Unexpected parsing state at byte {}", counter);
                    }
                }
                ParseResult::LengthError => {
                    assert_eq!(counter, 4);
                    assert!(parser.buffer.is_empty());
                    assert_eq!(0, parser.counter);
                    assert_eq!(0, parser.length);
                    assert_eq!(ParseState::WaitMarker, parser.state);
                }
                ParseResult::CrcError => {
                    panic!("Unexpected CRC error at byte {}", counter);
                }
                ParseResult::Success { length, data } => {
                    panic!(
                        "Unexpected success state with data {:?} of length {:?}",
                        data, length
                    );
                }
            }
        }
    }

    #[test]
    fn test_sfp_frame_parser_with_bad_crc() {
        let _test_data: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let frame: [u8; 14] = [
            0xAA, 0xAA, // Frame marker
            0x00, 0x08, // Length (8 bytes)
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, // Data (DEADBEEFCAFEBABE)
            0x12, 0x34, // Incorrect CRC
        ];

        let mut counter = 0;
        let mut parser = SfpFrameParser::default();
        parser.set_frame_start(&frame);

        for byte in frame {
            counter += 1;

            match parser.parse_byte(byte) {
                ParseResult::Idle => {
                    panic!("Unexpected idle state at byte {}", counter);
                }
                ParseResult::Parsing => {
                    assert_eq!(counter, parser.counter);
                }
                ParseResult::LengthError => {
                    panic!("Unexpected length error at byte {}", counter);
                }
                ParseResult::CrcError => {
                    assert_eq!(counter, frame.len());
                    assert!(parser.buffer.is_empty());
                    assert_eq!(0, parser.counter);
                    assert_eq!(0, parser.length);
                    assert_eq!(ParseState::WaitMarker, parser.state);
                }
                ParseResult::Success { length, data } => {
                    panic!(
                        "Unexpected success state with data {:?} of length {:?}",
                        data, length
                    );
                }
            }
        }
    }
}

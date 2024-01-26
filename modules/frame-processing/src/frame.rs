use cortex_m_semihosting::hprintln;

use crc::{Crc, CRC_16_USB};

// Constants
const FRAME_HEADER_MIN_LENGTH: usize = 6;
const FRAME_START: u8 = 0xAA;
const FRAME_FIRST_BYTE: usize = 0;
const FRAME_SECOND_BYTE: usize = 1;
const LENGTH_FIRST_BYTE: usize = 2;
const LENGTH_SECOND_BYTE: usize = 3;
const FRAME_BEGIN_LENGTH: usize = 4;

// Define the Crc algorithm
const CRC_16: Crc<u16> = Crc::<u16>::new(&CRC_16_USB);

pub fn process_incoming_frame(buffer: &mut [u8], buffer_size: &mut usize) -> (bool, bool) {
    let mut is_frame_valid = false;
    let mut had_complete_frame = false;

    // If the buffer has a size which could mean a correct frame
    if *buffer_size >= FRAME_HEADER_MIN_LENGTH {
        // Search for the frame start
        if buffer[FRAME_FIRST_BYTE] == FRAME_START && buffer[FRAME_SECOND_BYTE] == FRAME_START {
            // Extract the two bytes which contain the size of actual message
            let data_len =
                ((buffer[LENGTH_FIRST_BYTE] as usize) << 8) | buffer[LENGTH_SECOND_BYTE] as usize;

            if *buffer_size >= data_len + FRAME_HEADER_MIN_LENGTH {
                // Extract the actual data from the frame
                let frame_data = &buffer[FRAME_BEGIN_LENGTH..FRAME_BEGIN_LENGTH + data_len];

                // Extract the CRC from the frame
                let frame_crc = ((buffer[FRAME_BEGIN_LENGTH + data_len] as u16) << 8)
                    | buffer[FRAME_BEGIN_LENGTH + data_len + 1] as u16;

                let nb_bytes_length = 2;
                let length_offset = 2;
                let data_for_crc =
                    &buffer[nb_bytes_length..nb_bytes_length + length_offset + data_len];

                // Calculate the CRC of the crc_data slice
                let computed_crc = CRC_16.checksum(data_for_crc);

                // Check if the computed CRC is the same as the one extracted from the frame
                // TODO - do something else here
                if frame_crc == computed_crc {
                    is_frame_valid = true;
                    hprintln!(
                        "Received valid frame:\nLength: {}\nUseful data: {:?}\nCRC: {:04X}\n\n",
                        data_len,
                        frame_data,
                        frame_crc
                    );
                } else {
                    is_frame_valid = false;
                    hprintln!("Received invalid frame:\nLength: {}\nUseful data: {:?}\nFRAME CRC: {:04X}\nREAL CRC: {:04X}\n\n",
                    data_len, frame_data, frame_crc, computed_crc);
                }

                had_complete_frame = true;

                // Remove this frame from the buffer
                *buffer_size = *buffer_size - (FRAME_HEADER_MIN_LENGTH + data_len);
                buffer.rotate_left(FRAME_HEADER_MIN_LENGTH + data_len);
            }
        } else {
            // Remove first byte to re-align frame search
            *buffer_size = *buffer_size - 1;
            buffer.rotate_left(1);
        }
    }

    // Return
    (had_complete_frame, is_frame_valid)
}

pub fn pack_frame(data: &[u8], buffer: &mut [u8; 1024]) -> usize {
    let data_len = data.len() as u16;

    // Start with two bytes of value 0xAA
    buffer[0] = 0xAA;
    buffer[1] = 0xAA;

    // Add two bytes representing the size of the input data
    buffer[2] = ((data_len >> 8) & 0xFF) as u8;
    buffer[3] = (data_len & 0xFF) as u8;

    // Append the content of the input data
    let start = 4;
    let end = start + data_len as usize;
    buffer[start..end].copy_from_slice(data);

    // Compute the CRC
    let mut crc = CRC_16.digest();
    crc.update(&buffer[2..end]);
    let calc_crc = crc.finalize();

    // Add CRC to the frame
    buffer[end] = ((calc_crc >> 8) & 0xFF) as u8;
    buffer[end + 1] = (calc_crc & 0xFF) as u8;

    // Return the size of the new frame
    end + 2
}

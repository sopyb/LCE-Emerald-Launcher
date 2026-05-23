use std::io::Read;
use flate2::read::ZlibDecoder;
use crate::console2lce::error::ConversionError;
use crate::console2lce::models::LegacyBlockState;
pub fn decode_xbox360_chunk(chunk_data: &[u8]) -> Result<Vec<u8>, ConversionError> {
    if chunk_data.len() < 5 {
        return Err(ConversionError::InvalidFormat("Chunk data too short".to_string()));
    }

    let _compressed_size = read_be_u32(&chunk_data[0..4]);
    let compression_scheme = chunk_data[4];
    let payload = &chunk_data[5..];
    match compression_scheme {
        0 => Ok(payload.to_vec()),
        1 => {
            let mut decoder = ZlibDecoder::new(payload);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        }
        2 => {
            let is_rle = payload.len() >= 4 && (payload[0..4].iter().any(|&b| b == 0xFF));
            if is_rle {
                decode_rle_chunk(payload)
            } else {
                Ok(payload.to_vec())
            }
        }
        _ => Err(ConversionError::DecompressionFailed(
            format!("Unknown chunk compression scheme: {}", compression_scheme)
        )),
    }
}

fn decode_rle_chunk(data: &[u8]) -> Result<Vec<u8>, ConversionError> {
    let mut output = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let b = data[pos];
        pos += 1;
        if b != 0xFF {
            output.push(b);
        } else {
            if pos + 1 >= data.len() {
                break;
            }
            let run_len = data[pos] as usize;
            let value = data[pos + 1];
            pos += 2;
            for _ in 0..run_len {
                output.push(value);
            }
        }
    }

    Ok(output)
}

pub fn try_decode_compressed_tile_storage(data: &[u8]) -> Result<Vec<LegacyBlockState>, ConversionError> {
    if data.len() < 2048 {
        return Err(ConversionError::InvalidFormat("Compressed tile storage data too short".to_string()));
    }

    let mut blocks: Vec<LegacyBlockState> = Vec::with_capacity(16384);
    let index_table = &data[0..2048];
    let mut data_offset = 2048;
    for entry in index_table.chunks(2) {
        let index_value = u16::from_be_bytes([entry[0], entry[1]]);
        let data_type = (index_value >> 13) & 0x07;
        let data_offset_val = (index_value & 0x1FFF) as usize;
        match data_type {
            0 => {
                for _ in 0..16 {
                    blocks.push(LegacyBlockState::new(0, 0));
                }
            }
            1 => {
                if data_offset + 4 <= data.len() {
                    for i in 0..16 {
                        let nibble = (data[data_offset + i / 2] >> (if i % 2 == 0 { 0 } else { 4 })) & 0x0F;
                        blocks.push(LegacyBlockState::new(0, nibble as u8));
                    }
                    data_offset += 4;
                }
            }
            2 => {
                if data_offset + 8 <= data.len() {
                    for i in 0..16 {
                        let nibble = if data_offset + i < data.len() {
                            data[data_offset + i]
                        } else {
                            0
                        };
                        blocks.push(LegacyBlockState::new(nibble, 0));
                    }
                    data_offset += 8;
                }
            }
            3 => {
                if data_offset + 16 <= data.len() {
                    for i in 0..16 {
                        if data_offset + i * 2 + 1 < data.len() {
                            let id = data[data_offset + i * 2];
                            let meta = data[data_offset + i * 2 + 1];
                            blocks.push(LegacyBlockState::new(id, meta));
                        } else {
                            blocks.push(LegacyBlockState::new(0, 0));
                        }
                    }
                    data_offset += 16;
                }
            }
            4 => {
                if data_offset_val > 0 && data_offset_val < data.len() {
                    let sparse_offset = data_offset_val;
                    let mut sparse_pos = sparse_offset;
                    while sparse_pos < data.len() && blocks.len() < 16384 {
                        if sparse_pos + 3 > data.len() {
                            break;
                        }
                        let location = data[sparse_pos] as usize;
                        let block_id = data[sparse_pos + 1];
                        let block_data = data[sparse_pos + 2];
                        sparse_pos += 3;
                        for _ in blocks.len()..location {
                            blocks.push(LegacyBlockState::new(0, 0));
                        }
                        if blocks.len() < 16384 {
                            blocks.push(LegacyBlockState::new(block_id, block_data));
                        }
                    }
                    while blocks.len() < 16384 {
                        blocks.push(LegacyBlockState::new(0, 0));
                    }
                } else {
                    for _ in 0..16 {
                        blocks.push(LegacyBlockState::new(0, 0));
                    }
                }
            }
            _ => {
                for _ in 0..16 {
                    blocks.push(LegacyBlockState::new(0, 0));
                }
            }
        }
    }

    Ok(blocks)
}

fn read_be_u32(bytes: &[u8]) -> u32 {
    if bytes.len() < 4 { return 0; }
    ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
}

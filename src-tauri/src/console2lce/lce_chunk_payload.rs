use std::collections::HashMap;
use std::io::Write;
use flate2::write::ZlibEncoder;
use flate2::Compression as FlateCompression;
use crate::console2lce::error::ConversionError;
use crate::console2lce::nbt_util::{NbtTag, write_nbt_le};
pub fn encode_legacy_nbt(
    x_pos: i32,
    z_pos: i32,
    blocks: &[u8; 65536],
    data: &[u8; 32768],
    sky_light: &[u8; 32768],
    _block_light: &[u8; 32768],
    height_map: &[u8; 256],
    biomes: &[u8; 256],
    last_update: i64,
    inhabited_time: i64,
    entities: Vec<NbtTag>,
    tile_entities: Vec<NbtTag>,
    tile_ticks: Option<Vec<NbtTag>>,
) -> Result<Vec<u8>, ConversionError> {
    let mut level_map = HashMap::new();
    level_map.insert("xPos".to_string(), NbtTag::Int(x_pos));
    level_map.insert("zPos".to_string(), NbtTag::Int(z_pos));
    level_map.insert("LastUpdate".to_string(), NbtTag::Long(last_update));
    level_map.insert("InhabitedTime".to_string(), NbtTag::Long(inhabited_time));
    level_map.insert("Blocks".to_string(), NbtTag::ByteArray(blocks.to_vec()));
    level_map.insert("Data".to_string(), NbtTag::ByteArray(data.to_vec()));
    level_map.insert("SkyLight".to_string(), NbtTag::ByteArray(sky_light.to_vec()));
    level_map.insert("BlockLight".to_string(), NbtTag::ByteArray(_block_light.to_vec()));
    level_map.insert("HeightMap".to_string(), NbtTag::ByteArray(height_map.to_vec()));
    level_map.insert("TerrainPopulatedFlags".to_string(), NbtTag::Short(0));
    level_map.insert("Biomes".to_string(), NbtTag::ByteArray(biomes.to_vec()));
    level_map.insert("Entities".to_string(), NbtTag::List(entities, 10));
    level_map.insert("TileEntities".to_string(), NbtTag::List(tile_entities, 10));
    if let Some(ticks) = tile_ticks {
        level_map.insert("TileTicks".to_string(), NbtTag::List(ticks, 10));
    }

    let root = NbtTag::Compound(level_map);
    let nbt_data = write_nbt_le(&root)?;
    let rle_compressed = rle_compress(&nbt_data);
    let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
    encoder.write_all(&rle_compressed)?;
    let zlib_data = encoder.finish()?;
    Ok(zlib_data)
}

pub fn encode_compressed_storage(
    x_pos: i32,
    z_pos: i32,
    blocks: &[u8; 65536],
    data: &[u8; 32768],
    sky_light: &[u8; 32768],
    _block_light: &[u8; 32768],
    height_map: &[u8; 256],
    biomes: &[u8; 256],
    last_update: i64,
) -> Result<Vec<u8>, ConversionError> {
    let sections = build_compressed_tile_storage_sections(blocks, data, sky_light);
    let mut level = HashMap::new();
    level.insert("xPos".to_string(), NbtTag::Int(x_pos));
    level.insert("zPos".to_string(), NbtTag::Int(z_pos));
    level.insert("LastUpdate".to_string(), NbtTag::Long(last_update));
    level.insert("Sections".to_string(), NbtTag::List(sections, 10));
    level.insert("HeightMap".to_string(), NbtTag::ByteArray(height_map.to_vec()));
    level.insert("Biomes".to_string(), NbtTag::ByteArray(biomes.to_vec()));
    let root = NbtTag::Compound(level);
    let nbt_data = write_nbt_le(&root)?;
    let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
    encoder.write_all(&nbt_data)?;
    let zlib_data = encoder.finish()?;
    Ok(zlib_data)
}

fn build_compressed_tile_storage_sections(
    blocks: &[u8; 65536],
    data: &[u8; 32768],
    sky_light: &[u8; 32768],
) -> Vec<NbtTag> {
    let mut sections = Vec::new();
    for section_y in 0..8 {
        let half_chunk_blocks = extract_half_chunk(blocks, section_y, 65536);
        let half_chunk_data = extract_half_chunk(data, section_y, 32768);
        let half_chunk_sky = extract_half_chunk(sky_light, section_y, 32768);
        let storage = build_single_compressed_storage(&half_chunk_blocks, &half_chunk_data, &half_chunk_sky);
        let mut section = HashMap::new();
        section.insert("Y".to_string(), NbtTag::Byte(section_y as i8));
        section.insert("Blocks".to_string(), NbtTag::ByteArray(storage));
        sections.push(NbtTag::Compound(section));
    }

    sections
}

fn extract_half_chunk(full: &[u8], section_y: usize, full_size: usize) -> Vec<u8> {
    let section_size = 16384;
    let mut result = vec![0u8; section_size];
    let base_y = section_y * 16;
    for x in 0..16 {
        for z in 0..16 {
            for y in 0..16 {
                let src_y = base_y + y;
                if src_y >= 128 {
                    continue;
                }
                let src_idx = ((x * 16) + z) * 256 + src_y;
                if src_idx < full_size {
                    let dst_idx = (y * 16 + z) * 16 + x;
                    if dst_idx < section_size {
                        result[dst_idx] = full[src_idx];
                    }
                }
            }
        }
    }

    result
}

fn build_single_compressed_storage(blocks: &[u8], data: &[u8], _sky_light: &[u8]) -> Vec<u8> {
    let mut storage = Vec::new();
    let num_indices = 1024;
    let mut index_table = vec![0u16; num_indices];
    let mut payload = Vec::new();
    for idx in 0..num_indices {
        let base = idx * 16;
        let mut uses_high_data = false;
        let mut max_id: u8 = 0;
        let mut max_data: u8 = 0;
        for j in 0..16 {
            let block_pos = base + j;
            if block_pos >= blocks.len() {
                break;
            }
            let b = blocks[block_pos];
            let d = if block_pos < data.len() {
                get_nibble(data, block_pos)
            } else {
                0
            };
            if b > max_id { max_id = b; }
            if d > max_data { max_data = d; }
            if d > 0 { uses_high_data = true; }
        }

        if max_id == 0 && max_data == 0 {
            index_table[idx] = 0;
            continue;
        }

        if max_id <= 15 && !uses_high_data {
            index_table[idx] = (1 << 13) | (payload.len() as u16);
            for j in 0..16 {
                let block_pos = base + j;
                if block_pos < blocks.len() {
                    let b = blocks[block_pos];
                    if j % 2 == 0 {
                        payload.push(b << 4);
                    } else {
                        let last = payload.last_mut().unwrap();
                        *last |= b & 0x0F;
                    }
                }
            }
            continue;
        }

        if max_data == 0 {
            index_table[idx] = (2 << 13) | (payload.len() as u16);
            for j in 0..16 {
                let block_pos = base + j;
                if block_pos < blocks.len() {
                    payload.push(blocks[block_pos]);
                }
            }
            continue;
        }

        if max_id > 0 || max_data > 0 {
            index_table[idx] = (3 << 13) | (payload.len() as u16);
            for j in 0..16 {
                let block_pos = base + j;
                if block_pos < blocks.len() {
                    payload.push(blocks[block_pos]);
                    if block_pos < data.len() {
                        payload.push(get_nibble(data, block_pos));
                    } else {
                        payload.push(0);
                    }
                }
            }
        }
    }

    for &entry in &index_table {
        storage.extend_from_slice(&entry.to_be_bytes());
    }
    storage.extend_from_slice(&payload);
    storage
}

fn get_nibble(arr: &[u8], index: usize) -> u8 {
    let b = arr[index >> 1];
    if (index & 1) == 0 { b & 0x0F } else { (b >> 4) & 0x0F }
}

pub fn rle_compress(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let run_start = pos;
        let mut run_len = 0u8;
        while run_start + (run_len as usize) < data.len() && run_len < 255 {
            if data[run_start] == data[run_start + (run_len as usize)] {
                run_len += 1;
            } else {
                break;
            }
        }

        if run_len >= 4 {
            output.push(0xFF);
            output.push(run_len);
            output.push(data[run_start]);
            pos = run_start + run_len as usize;
        } else {
            output.push(data[pos]);
            pos += 1;
        }
    }

    output
}

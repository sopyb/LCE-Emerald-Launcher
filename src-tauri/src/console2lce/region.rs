use crate::console2lce::error::ConversionError;
const SECTOR_SIZE: u32 = 4096;
const REGION_SIZE_X: usize = 32;
const REGION_SIZE_Z: usize = 32;
#[derive(Debug, Clone, Default)]
pub struct RegionEntry {
    pub offset: u32,
    pub sector_count: u32,
    pub timestamp: u32,
    pub exists: bool,
}

#[derive(Debug, Clone)]
pub struct Xbox360Region {
    pub entries: [[RegionEntry; REGION_SIZE_Z]; REGION_SIZE_X],
    pub raw_data: Vec<u8>,
}

pub fn parse_xbox360_region(data: &[u8]) -> Result<Xbox360Region, ConversionError> {
    if data.len() < 8192 {
        return Err(ConversionError::InvalidFormat("Region data too short, need at least 8192 bytes for header".to_string()));
    }

    let mut region = Xbox360Region {
        entries: Default::default(),
        raw_data: data.to_vec(),
    };

    for z in 0..REGION_SIZE_Z {
        for x in 0..REGION_SIZE_X {
            let index = z * REGION_SIZE_X + x;
            let offset_raw = read_be_u32(&data[index * 4..index * 4 + 4]);
            let timestamp_raw = read_be_u32(&data[4096 + index * 4..4096 + index * 4 + 4]);
            let offset = offset_raw >> 8;
            let sector_count = offset_raw & 0xFF;
            region.entries[x][z] = RegionEntry {
                offset,
                sector_count,
                timestamp: timestamp_raw,
                exists: offset_raw != 0 && sector_count != 0,
            };
        }
    }

    Ok(region)
}

pub fn read_region_chunk(region: &Xbox360Region, x: usize, z: usize) -> Result<Vec<u8>, ConversionError> {
    if x >= REGION_SIZE_X || z >= REGION_SIZE_Z {
        return Err(ConversionError::InvalidFormat(format!(
            "Chunk coordinates out of range: ({}, {}), max ({}, {})",
            x, z, REGION_SIZE_X, REGION_SIZE_Z
        )));
    }

    let entry = &region.entries[x][z];
    if !entry.exists {
        return Err(ConversionError::MissingData(format!(
            "Chunk ({}, {}) does not exist in region", x, z
        )));
    }

    let sector_offset = (entry.offset as usize) * SECTOR_SIZE as usize;
    let total_size = (entry.sector_count as usize) * SECTOR_SIZE as usize;
    if sector_offset + total_size > region.raw_data.len() {
        return Err(ConversionError::InvalidFormat(format!(
            "Chunk data at sector offset {} with size {} exceeds region data length {}",
            sector_offset, total_size, region.raw_data.len()
        )));
    }

    let sector_data = &region.raw_data[sector_offset..sector_offset + 5];
    let _compressed_size = read_be_u32(&sector_data[0..4]);
    let _compression_scheme = sector_data[4];
    let chunk_data = region.raw_data[sector_offset..sector_offset + total_size].to_vec();
    Ok(chunk_data)
}

pub fn read_java_mcr_region(data: &[u8]) -> Result<Vec<(usize, usize, Vec<u8>)>, ConversionError> {
    if data.len() < 8192 {
        return Err(ConversionError::InvalidFormat("MCR data too short".to_string()));
    }

    let mut chunks = Vec::new();
    for z in 0..REGION_SIZE_Z {
        for x in 0..REGION_SIZE_X {
            let index = z * REGION_SIZE_X + x;
            let offset_raw = read_be_u32(&data[index * 4..index * 4 + 4]);
            if offset_raw == 0 {
                continue;
            }

            let sector_offset = (offset_raw >> 8) as usize;
            let _sector_count = (offset_raw & 0xFF) as usize;
            let file_offset = sector_offset * SECTOR_SIZE as usize;
            if file_offset + 5 > data.len() {
                continue;
            }

            let chunk_len = read_be_u32(&data[file_offset..file_offset + 4]) as usize;
            let compression = data[file_offset + 4];
            if chunk_len == 0 || file_offset + 5 + chunk_len > data.len() {
                continue;
            }

            let mut chunk_data = vec![0u8; 5 + chunk_len];
            chunk_data[0..4].copy_from_slice(&data[file_offset..file_offset + 4]);
            chunk_data[4] = compression;
            chunk_data[5..].copy_from_slice(&data[file_offset + 5..file_offset + 5 + chunk_len]);
            chunks.push((x, z, chunk_data));
        }
    }

    Ok(chunks)
}

pub fn read_java_mca_region(data: &[u8]) -> Result<Vec<(usize, usize, Vec<u8>)>, ConversionError> {
    read_java_mcr_region(data)
}

fn read_be_u32(bytes: &[u8]) -> u32 {
    if bytes.len() < 4 { return 0; }
    ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
}

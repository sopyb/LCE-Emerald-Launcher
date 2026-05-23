use std::collections::BTreeMap;
use crate::console2lce::error::ConversionError;
const SECTOR_SIZE: usize = 4096;
const HEADER_SECTORS: usize = 2;
pub struct LceRegionFile {
    sectors: Vec<u8>,
    offsets: BTreeMap<(i32, i32), (u32, u32, u32)>,
    next_sector: u32,
}

impl LceRegionFile {
    pub fn new() -> Self {
        let header_size = HEADER_SECTORS * SECTOR_SIZE;
        LceRegionFile {
            sectors: vec![0u8; header_size],
            offsets: BTreeMap::new(),
            next_sector: HEADER_SECTORS as u32,
        }
    }

    pub fn add_chunk(&mut self, chunk_x: i32, chunk_z: i32, data: &[u8]) -> Result<(), ConversionError> {
        if data.len() > 1024 * 1024 {
            return Err(ConversionError::InvalidFormat(format!(
                "Chunk ({}, {}) data too large: {} bytes", chunk_x, chunk_z, data.len()
            )));
        }

        let total_size = 5 + data.len();
        let sectors_needed = ((total_size + SECTOR_SIZE - 1) / SECTOR_SIZE) as u32;
        let sector_start = self.next_sector;
        let padding = sectors_needed as usize * SECTOR_SIZE - total_size;
        let length_be = (data.len() as u32).to_be_bytes();
        self.sectors.extend_from_slice(&length_be);
        self.sectors.push(2);
        self.sectors.extend_from_slice(data);
        for _ in 0..padding {
            self.sectors.push(0);
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        self.offsets.insert((chunk_x, chunk_z), (sector_start, sectors_needed, timestamp));
        self.next_sector = sector_start + sectors_needed;
        Ok(())
    }

    pub fn build(mut self) -> Result<Vec<u8>, ConversionError> {
        for ((chunk_x, chunk_z), (sector_start, sector_count, timestamp)) in &self.offsets {
            let local_x = chunk_x.rem_euclid(32) as usize;
            let local_z = chunk_z.rem_euclid(32) as usize;
            let index = local_z * 32 + local_x;
            let offset_val = (sector_start << 8) | (sector_count & 0xFF);
            let offset_bytes = offset_val.to_be_bytes();
            let ts_bytes = timestamp.to_be_bytes();
            let off_pos = index * 4;
            let ts_pos = 4096 + index * 4;
            if off_pos + 4 <= self.sectors.len() {
                self.sectors[off_pos..off_pos + 4].copy_from_slice(&offset_bytes);
            }
            if ts_pos + 4 <= self.sectors.len() {
                self.sectors[ts_pos..ts_pos + 4].copy_from_slice(&ts_bytes);
            }
        }

        Ok(self.sectors)
    }
}

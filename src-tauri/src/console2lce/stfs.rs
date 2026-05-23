use crate::console2lce::error::ConversionError;
use lzxd::{Lzxd, WindowSize};

const CON_MAGIC: u32 = 0x434F4E20;

fn read_be_u32(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
}

fn read_be_u16(bytes: &[u8]) -> u16 {
    ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

fn read_le_u16(bytes: &[u8]) -> u16 {
    (bytes[0] as u16) | ((bytes[1] as u16) << 8)
}

fn read_le_u24(bytes: &[u8]) -> i32 {
    (bytes[0] as i32) | ((bytes[1] as i32) << 8) | ((bytes[2] as i32) << 16)
}

pub fn is_stfs_package(data: &[u8]) -> bool {
    if data.len() < 4 { return false; }
    read_be_u32(&data[0..4]) == CON_MAGIC
}

struct StfsVD {
    _size: u8,
    block_separation: u8,
    file_table_block_count: u16,
    file_table_block_num: i32,
    alloc_block_count: u32,
    _unallocated_block_count: u32,
}

fn read_stfs_vd(data: &[u8]) -> Result<StfsVD, ConversionError> {
    if data.len() < 0x24 {
        return Err(ConversionError::InvalidFormat("STFS VD too short".to_string()));
    }
    let size = data[0];
    let block_separation = data[2];
    let file_table_block_count = read_le_u16(&data[3..5]);
    let file_table_block_num = read_le_u24(&data[5..8]);
    let alloc_block_count = read_be_u32(&data[0x1C..0x20]);
    let unallocated_block_count = read_be_u32(&data[0x20..0x24]);
    Ok(StfsVD {
        _size: size,
        block_separation,
        file_table_block_count,
        file_table_block_num,
        alloc_block_count,
        _unallocated_block_count: unallocated_block_count,
    })
}

struct StfsPackage<'a> {
    data: &'a [u8],
    header_size: u32,
    vd: StfsVD,
    package_sex: u8,        //neo: 0=female, 1=male
    block_step: [u32; 2],
    first_hash_addr: u32,
    top_level: u8,
    top_table: Vec<HashEntry>,
    alloc_block_count: u32,
}

#[derive(Clone, Copy)]
struct HashEntry {
    _hash: [u8; 20],
    status: u8,
    next_block: i32,
}

impl<'a> StfsPackage<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, ConversionError> {
        if data.len() < 0x400 {
            return Err(ConversionError::InvalidFormat("STFS data too short for header".to_string()));
        }

        if !is_stfs_package(data) {
            return Err(ConversionError::InvalidFormat("Not a valid STFS package".to_string()));
        }

        let header_size = read_be_u32(&data[0x340..0x344]);
        let content_type = read_be_u32(&data[0x344..0x348]);
        if content_type != 1 {
            return Err(ConversionError::InvalidFormat(format!(
                "Not a savegame (content_type={})", content_type
            )));
        }

        let file_system = read_be_u32(&data[0x3A9..0x3AD]);
        if file_system != 0 {
            return Err(ConversionError::InvalidFormat("Not STFS format".to_string()));
        }

        let vd = read_stfs_vd(&data[0x379..])?;
        let alloc_block_count = vd.alloc_block_count;
        if alloc_block_count == 0 {
            return Err(ConversionError::InvalidFormat("No allocated blocks".to_string()));
        }

        let package_sex = (!vd.block_separation) & 1;
        let block_step: [u32; 2] = if package_sex == 0 {
            [0xAB, 0x718F]
        } else {
            [0xAC, 0x723A]
        };

        let first_hash_addr = (header_size + 0x0FFF) & !0x0FFF;
        let top_level = if alloc_block_count <= 0xAA {
            0
        } else if alloc_block_count <= 0x70E4 {
            1
        } else if alloc_block_count <= 0x4AF768 {
            2
        } else {
            return Err(ConversionError::InvalidFormat("Too many allocated blocks".to_string()));
        };

        let top_true_block = compute_level_n_backing_hash_block_number(0, top_level, &block_step, package_sex);
        let base_addr = (top_true_block << 12) + first_hash_addr;
        let top_addr = base_addr + (((vd.block_separation & 2) as u32) << 11);
        let top_addr = top_addr as usize;
        let data_blocks_per_level: [u32; 3] = [1, 0xAA, 0x70E4];
        let divisor = data_blocks_per_level[top_level as usize];
        let mut entry_count = alloc_block_count / divisor;
        if alloc_block_count > 0x70E4 && alloc_block_count % 0x70E4 != 0 {
            entry_count += 1;
        } else if alloc_block_count > 0xAA && alloc_block_count % 0xAA != 0 {
            entry_count += 1;
        }

        let mut top_table = Vec::with_capacity(entry_count as usize);
        for i in 0..entry_count {
            let off = top_addr + (i as usize) * 24;
            if off + 24 > data.len() {
                break;
            }
            let mut hash = [0u8; 20];
            hash.copy_from_slice(&data[off..off + 20]);
            let status = data[off + 20];
            let next_block = read_le_u24(&data[off + 21..off + 24]);
            top_table.push(HashEntry {
                _hash: hash,
                status,
                next_block,
            });
        }

        Ok(StfsPackage {
            data,
            header_size,
            vd,
            package_sex,
            block_step,
            first_hash_addr,
            top_level,
            top_table,
            alloc_block_count,
        })
    }

    fn block_to_address(&self, block_num: u32) -> u32 {
        (self.compute_backing_data_block_number(block_num) << 12) + self.first_hash_addr
    }

    fn compute_backing_data_block_number(&self, block_num: u32) -> u32 {
        let psex = self.package_sex as u32;
        let to_return = (((block_num + 0xAA) / 0xAA) << psex) + block_num;
        if block_num < 0xAA {
            return to_return;
        } else if block_num < 0x70E4 {
            return to_return + (((block_num + 0x70E4) / 0x70E4) << psex);
        } else {
            return (1 << psex) + (to_return + (((block_num + 0x70E4) / 0x70E4) << psex));
        }
    }

    fn get_hash_entry(&self, block_num: u32) -> Result<HashEntry, ConversionError> {
        if block_num >= self.alloc_block_count {
            return Err(ConversionError::InvalidFormat("Block number out of range".to_string()));
        }
        let hash_addr = self.get_hash_address_of_block(block_num);
        let off = hash_addr as usize;
        if off + 24 > self.data.len() {
            return Err(ConversionError::InvalidFormat("Hash address out of file bounds".to_string()));
        }
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&self.data[off..off + 20]);
        let status = self.data[off + 20];
        let next_block = read_le_u24(&self.data[off + 21..off + 24]);
        Ok(HashEntry { _hash: hash, status, next_block })
    }

    fn get_hash_address_of_block(&self, block_num: u32) -> u32 {
        let level0 = compute_level_0_backing_hash_block_number(block_num, &self.block_step, self.package_sex);
        let mut hash_addr = (level0 << 12) + self.first_hash_addr;
        hash_addr += (block_num % 0xAA) * 24;
        match self.top_level {
            0 => {
                hash_addr += (self.vd.block_separation as u32 & 2) << 11;
            }
            1 => {
                let idx = (block_num / 0xAA) as usize;
                if idx < self.top_table.len() {
                    hash_addr += ((self.top_table[idx].status & 0x40) as u32) << 6;
                }
            }
            2 => {
                let l1_idx = (block_num / 0x70E4) as usize;
                if l1_idx < self.top_table.len() {
                    let l1_off = (self.top_table[l1_idx].status as u32 & 0x40) << 6;
                    let l1_block = compute_level_1_backing_hash_block_number(block_num, &self.block_step, self.package_sex);
                    let pos = ((l1_block << 12) + self.first_hash_addr + l1_off) +
                              ((block_num % 0xAA) * 24);
                    let pos = pos as usize;
                    if pos + 21 <= self.data.len() {
                        hash_addr += (self.data[pos + 20] as u32 & 0x40) << 6;
                    }
                }
            }
            _ => {}
        }
        hash_addr
    }

    fn read_file_listing(&self) -> Result<Vec<StfsFileEntry>, ConversionError> {
        let mut entries = Vec::new();
        let block_count = self.vd.file_table_block_count as u32;
        let mut block = self.vd.file_table_block_num as u32;
        for _ in 0..block_count {
            let addr = self.block_to_address(block) as usize;
            if addr + 0x1000 > self.data.len() {
                break;
            }
            for i in 0..0x40u32 {
                let off = addr + (i as usize) * 0x40;
                if off + 0x40 > self.data.len() {
                    break;
                }
                let name_bytes = &self.data[off..off + 0x28];
                let name_len_byte = self.data[off + 0x28];
                let name_len = name_len_byte & 0x3F;
                if name_len == 0 {
                    continue;
                }
                let name = String::from_utf8_lossy(&name_bytes[..name_len as usize]).to_string();
                if name.is_empty() {
                    break;
                }
                let blocks_for_file = read_le_u24(&self.data[off + 0x29..off + 0x2C]);
                let starting_block_num = read_le_u24(&self.data[off + 0x2F..off + 0x32]);
                let path_indicator = read_be_u16(&self.data[off + 0x32..off + 0x34]);
                let file_size = read_be_u32(&self.data[off + 0x34..off + 0x38]);
                let created_ts = read_be_u32(&self.data[off + 0x38..off + 0x3C]);
                let accessed_ts = read_be_u32(&self.data[off + 0x3C..off + 0x40]);
                let flags = name_len_byte >> 6;
                entries.push(StfsFileEntry {
                    name,
                    flags,
                    blocks_for_file,
                    starting_block_num,
                    path_indicator,
                    file_size,
                    created_ts,
                    accessed_ts,
                });
            }

            let he = self.get_hash_entry(block)?;
            block = he.next_block as u32;
        }

        Ok(entries)
    }

    fn extract_file(&self, entry: &StfsFileEntry) -> Result<Vec<u8>, ConversionError> {
        let file_size = entry.file_size;
        if file_size == 0 {
            return Ok(Vec::new());
        }

        if entry.flags & 1 != 0 {
            let mut out = Vec::with_capacity(file_size as usize);
            let start_addr = self.block_to_address(entry.starting_block_num as u32);
            let pos = start_addr as usize;
            let block_count = (compute_level_0_backing_hash_block_number(
                entry.starting_block_num as u32, &self.block_step, self.package_sex
            ) + self.block_step[0]) - ((start_addr - self.first_hash_addr) >> 12);
            let initial = if (entry.blocks_for_file as u32) <= block_count {
                let sz = entry.file_size as usize;
                if pos + sz <= self.data.len() {
                    out.extend_from_slice(&self.data[pos..pos + sz]);
                }
                return Ok(out);
            } else {
                let amount = (block_count << 12) as usize;
                if pos + amount <= self.data.len() {
                    out.extend_from_slice(&self.data[pos..pos + amount]);
                }
                amount
            };

            let mut remaining = (file_size as usize).wrapping_sub(initial);
            let mut cur_pos = pos + initial;
            while remaining >= 0xAA000 {
                cur_pos += self.get_hash_table_skip_size(cur_pos as u32) as usize;
                let end = (cur_pos + 0xAA000).min(self.data.len());
                out.extend_from_slice(&self.data[cur_pos..end]);
                remaining = remaining.wrapping_sub(0xAA000);
                cur_pos = end;
            }
            if remaining > 0 {
                cur_pos += self.get_hash_table_skip_size(cur_pos as u32) as usize;
                let end = (cur_pos + remaining).min(self.data.len());
                out.extend_from_slice(&self.data[cur_pos..end]);
            }
            Ok(out)
        } else {
            let full_reads = file_size / 0x1000;
            let remainder = file_size % 0x1000;
            let mut out = Vec::with_capacity(file_size as usize);
            let mut block = entry.starting_block_num as u32;
            for _ in 0..full_reads {
                let addr = self.block_to_address(block) as usize;
                if addr + 0x1000 <= self.data.len() {
                    out.extend_from_slice(&self.data[addr..addr + 0x1000]);
                }
                block = self.get_hash_entry(block)?.next_block as u32;
            }
            if remainder > 0 {
                let addr = self.block_to_address(block) as usize;
                if addr + remainder as usize <= self.data.len() {
                    out.extend_from_slice(&self.data[addr..addr + remainder as usize]);
                }
            }
            Ok(out)
        }
    }

    fn get_hash_table_skip_size(&self, table_addr: u32) -> u32 {
        let psex = self.package_sex as u32;
        let true_block = (table_addr - self.first_hash_addr) >> 12;
        if true_block == 0 {
            return 0x1000 << psex;
        }
        let mut tb = true_block;
        if tb == self.block_step[1] {
            return 0x3000 << psex;
        } else if tb > self.block_step[1] {
            tb -= self.block_step[1] + (1 << psex);
        }
        if tb == self.block_step[0] || tb % self.block_step[1] == 0 {
            return 0x2000 << psex;
        }
        0x1000 << psex
    }
}

#[derive(Debug, Clone)]
struct StfsFileEntry {
    name: String,
    flags: u8,
    blocks_for_file: i32,
    starting_block_num: i32,
    path_indicator: u16,
    file_size: u32,
    created_ts: u32,
    accessed_ts: u32,
}

fn compute_level_n_backing_hash_block_number(
    block_num: u32, level: u8, block_step: &[u32; 2], package_sex: u8,
) -> u32 {
    match level {
        0 => compute_level_0_backing_hash_block_number(block_num, block_step, package_sex),
        1 => compute_level_1_backing_hash_block_number(block_num, block_step, package_sex),
        2 => block_step[1],
        _ => 0,
    }
}

fn compute_level_0_backing_hash_block_number(
    block_num: u32, block_step: &[u32; 2], package_sex: u8,
) -> u32 {
    let psex = package_sex as u32;
    if block_num < 0xAA { return 0; }
    let mut num = (block_num / 0xAA) * block_step[0];
    num += ((block_num / 0x70E4) + 1) << psex;
    if block_num / 0x70E4 == 0 { return num; }
    num + (1 << psex)
}

fn compute_level_1_backing_hash_block_number(
    block_num: u32, block_step: &[u32; 2], package_sex: u8,
) -> u32 {
    let psex = package_sex as u32;
    if block_num < 0x70E4 { return block_step[0]; }
    (1 << psex) + (block_num / 0x70E4) * block_step[1]
}

pub fn extract_savegame_from_stfs(data: &[u8]) -> Result<Vec<u8>, ConversionError> {
    let pkg = StfsPackage::parse(data)?;
    let listing = pkg.read_file_listing()?;
    let entry = listing.iter().find(|e| e.name == "savegame.dat")
        .ok_or_else(|| ConversionError::InvalidFormat("savegame.dat not found in STFS listing".to_string()))?;

    eprintln!("[stfs] found savegame.dat: size={} blocks={} start_block={}",
        entry.file_size, entry.blocks_for_file, entry.starting_block_num);

    let raw_data = pkg.extract_file(entry)?;
    eprintln!("[stfs] extracted {} bytes of savegame.dat", raw_data.len());

    Ok(raw_data)
}

pub fn try_decompress_stfs_savegame(data: &[u8]) -> Result<Vec<u8>, ConversionError> {
    if data.len() < 12 {
        return Err(ConversionError::InvalidFormat("STFS savegame data too short for header".to_string()));
    }
    let total_size = read_be_u32(&data[0..4]);
    let decompressed_size = u64::from(read_be_u32(&data[4..8])) << 32 | read_be_u32(&data[8..12]) as u64;
    let src_size = total_size.wrapping_sub(8) as usize;
    let decompressed_size = decompressed_size as usize;
    eprintln!("[stfs]   savegame header: total_size={} src_size={} decompressed_size={}",
        total_size, src_size, decompressed_size);

    if src_size == 0 || src_size > data.len().saturating_sub(12) {
        return Err(ConversionError::InvalidFormat(format!(
            "Invalid compressed size: {} (data len: {})", src_size, data.len()
        )));
    }
    if decompressed_size == 0 || decompressed_size > 100_000_000 {
        return Err(ConversionError::InvalidFormat(format!(
            "Invalid decompressed size: {}", decompressed_size
        )));
    }

    let compressed = &data[12..12 + src_size];
    decompress_lzx_chunked(compressed, decompressed_size)
}

pub fn decompress_lzx_chunked(data: &[u8], decompressed_size: usize) -> Result<Vec<u8>, ConversionError> {
    let mut lzxd = Lzxd::new(WindowSize::KB64);
    let mut output = Vec::with_capacity(decompressed_size);
    let mut pos = 0;
    let mut num_chunks = 0;
    let mut comp_count = 0;
    let mut uncomp_count = 0;
    let mut lzx_fallbacks = 0;
    while pos < data.len() && output.len() < decompressed_size {
        if pos + 2 > data.len() {
            break;
        }
        let chunk_size = read_be_u16(&data[pos..pos + 2]) as usize;
        pos += 2;
        if chunk_size == 0 {
            break;
        }

        let is_compressed = (chunk_size & 0x8000) != 0;
        let actual_size = chunk_size & 0x7FFF;
        num_chunks += 1;

        if pos + actual_size > data.len() {
            eprintln!("[lzx] chunk {} header overrun: need {} have {} at pos {}",
                num_chunks, actual_size, data.len() - pos + 2, pos - 2);
            break;
        }

        let chunk_data = &data[pos..pos + actual_size];
        pos += actual_size;

        if is_compressed {
            let out_len = if output.len() + 32768 > decompressed_size {
                decompressed_size - output.len()
            } else {
                32768
            };
            comp_count += 1;
            match lzxd.decompress_next(chunk_data, out_len) {
                Ok(decompressed) => {
                    eprintln!("[lzx] chunk {} C: compressed={} decompressed={} total_so_far={}",
                        num_chunks, actual_size, decompressed.len(), output.len() + decompressed.len());
                    output.extend_from_slice(decompressed);
                }
                Err(e) => {
                    return Err(ConversionError::DecompressionFailed(
                        format!("chunk {} LZXD failed at pos {}: {}", num_chunks, pos - actual_size - 2, e)
                    ));
                }
            }
        } else {
            let mut chunk_lzxd = Lzxd::new(WindowSize::KB32);
            match chunk_lzxd.decompress_next(chunk_data, 32768) {
                Ok(decompressed) => {
                    lzx_fallbacks += 1;
                    eprintln!("[lzx] chunk {} U->C (LZX fallback): {} -> {} total_so_far={}",
                        num_chunks, actual_size, decompressed.len(), output.len() + decompressed.len());
                    output.extend_from_slice(decompressed);
                }
                Err(_) => {
                    eprintln!("[lzx] chunk {} U: size={} total_so_far={}",
                        num_chunks, actual_size, output.len() + actual_size);
                    output.extend_from_slice(chunk_data);
                    uncomp_count += 1;
                }
            }
        }
    }
    eprintln!("[lzx] processed {} chunks ({} marked C + {} LZX-fallback + {} pure U), produced {} of {} expected bytes, consumed {}/{}",
        num_chunks, comp_count, lzx_fallbacks, uncomp_count, output.len(), decompressed_size, pos, data.len());

    output.truncate(decompressed_size);
    Ok(output)
}

pub fn try_decompress_savegame(data: &[u8]) -> Result<Vec<u8>, ConversionError> {
    if data.len() < 8 {
        return Err(ConversionError::InvalidFormat("Savegame data too short".to_string()));
    }
    let magic = &data[0..8];
    let expected_magic = [0x0B, 0xFC, 0x4A, 0x46, 0xAE, 0x7A, 0x43, 0x2B];
    if magic != expected_magic {
        return Err(ConversionError::InvalidFormat("Invalid savegame magic".to_string()));
    }

    if data.len() < 16 {
        return Err(ConversionError::InvalidFormat("Savegame data too short for header".to_string()));
    }

    let decompressed_size = read_be_u32(&data[8..12]) as usize;
    let compression_type = read_be_u32(&data[12..16]);
    let payload = &data[16..];
    match compression_type {
        0 => {
            if payload.len() >= decompressed_size {
                Ok(payload[..decompressed_size].to_vec())
            } else {
                Ok(payload.to_vec())
            }
        }
        1 => {
            decompress_lzx(payload, decompressed_size)
        }
        2 => {
            use std::io::Read;
            let mut decoder = flate2::read::ZlibDecoder::new(payload);
            let mut out = Vec::with_capacity(decompressed_size);
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        3 => {
            decode_rle(payload, decompressed_size)
        }
        _ => Err(ConversionError::DecompressionFailed(
            format!("Unknown compression type: {}", compression_type)
        )),
    }
}

fn decompress_lzx(data: &[u8], decompressed_size: usize) -> Result<Vec<u8>, ConversionError> {
    let mut output = Vec::with_capacity(decompressed_size);
    let mut pos = 0;
    while pos < data.len() && output.len() < decompressed_size {
        if pos + 2 > data.len() {
            break;
        }
        let chunk_size = read_be_u16(&data[pos..pos + 2]) as usize;
        pos += 2;
        if chunk_size == 0 {
            break;
        }

        let is_compressed = (chunk_size & 0x8000) != 0;
        let actual_size = chunk_size & 0x7FFF;
        if is_compressed {
            if pos + actual_size > data.len() {
                break;
            }
            let compressed_chunk = &data[pos..pos + actual_size];
            pos += actual_size;
            let mut lzxd = Lzxd::new(WindowSize::KB32);
            match lzxd.decompress_next(compressed_chunk, 32768) {
                Ok(decompressed) => {
                    output.extend_from_slice(decompressed);
                }
                Err(e) => {
                    eprintln!("[lzx] chunk LZXD decompress failed at pos {}: {}; filling with zeros",
                        pos - actual_size - 2, e);
                    output.extend_from_slice(&vec![0u8; 32768]);
                }
            }
        } else {
            if pos + actual_size > data.len() {
                break;
            }
            output.extend_from_slice(&data[pos..pos + actual_size]);
            pos += actual_size;
        }
    }

    output.truncate(decompressed_size);
    Ok(output)
}

pub fn decode_rle(data: &[u8], decompressed_size: usize) -> Result<Vec<u8>, ConversionError> {
    let mut output = Vec::with_capacity(decompressed_size);
    let mut pos = 0;
    while pos < data.len() && output.len() < decompressed_size {
        let b = data[pos];
        pos += 1;
        if b != 0xFF {
            if output.len() + 1 > decompressed_size {
                break;
            }
            output.push(b);
        } else {
            if pos + 1 >= data.len() {
                break;
            }
            let run_len = data[pos] as usize;
            let value = data[pos + 1];
            pos += 2;
            let actual_len = std::cmp::min(run_len, decompressed_size - output.len());
            for _ in 0..actual_len {
                output.push(value);
            }
        }
    }
    output.truncate(decompressed_size);
    Ok(output)
}

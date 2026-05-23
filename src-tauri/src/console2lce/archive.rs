use std::collections::HashMap;
use crate::console2lce::error::ConversionError;
use crate::console2lce::models::ArchiveEntry;
const ARCHIVE_MAGIC: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
pub fn parse_file_listing(data: &[u8]) -> Result<HashMap<String, Vec<u8>>, ConversionError> {
    if data.len() < 12 {
        return Err(ConversionError::InvalidFormat("File listing too short".to_string()));
    }

    let (header, le) = match try_parse_listing_header(data, true) {
        Some(h) => (h, false),
        None => match try_parse_listing_header(data, false) {
            Some(h) => (h, true),
            None => return Err(ConversionError::InvalidFormat("Invalid file listing header".to_string())),
        },
    };

    let footer_entry_size: usize = if header.latest <= 1 { 136 } else { 144 };
    let mut files = HashMap::new();
    for i in 0..header.file_count {
        let entry_off = header.index_offset as usize + i as usize * footer_entry_size;
        if entry_off + footer_entry_size > data.len() {
            break;
        }

        let name = decode_utf16le_padded(&data[entry_off..entry_off + 128]);
        let size_off = entry_off + 128;
        let ofs_off = size_off + 4;
        let file_size = if le {
            u32::from_le_bytes(data[size_off..size_off + 4].try_into().unwrap())
        } else {
            u32::from_be_bytes(data[size_off..size_off + 4].try_into().unwrap())
        };

        let file_ofs = if le {
            u32::from_le_bytes(data[ofs_off..ofs_off + 4].try_into().unwrap())
        } else {
            u32::from_be_bytes(data[ofs_off..ofs_off + 4].try_into().unwrap())
        };

        if name.is_empty() {
            continue;
        }

        let start = file_ofs as usize;
        let end = (start + file_size as usize).min(data.len());
        if start < end {
            files.insert(name, data[start..end].to_vec());
        }
    }

    Ok(files)
}

struct ListingHeader {
    index_offset: u32,
    file_count: u32,
    oldest: u16,
    latest: u16,
}

fn try_parse_listing_header(data: &[u8], be: bool) -> Option<ListingHeader> {
    let (index_offset, file_count) = if be {
        (u32::from_be_bytes(data[0..4].try_into().ok()?),
         u32::from_be_bytes(data[4..8].try_into().ok()?))
    } else {
        (u32::from_le_bytes(data[0..4].try_into().ok()?),
         u32::from_le_bytes(data[4..8].try_into().ok()?))
    };
    let oldest = if be {
        u16::from_be_bytes(data[8..10].try_into().ok()?)
    } else {
        u16::from_le_bytes(data[8..10].try_into().ok()?)
    };
    let latest = if be {
        u16::from_be_bytes(data[10..12].try_into().ok()?)
    } else {
        u16::from_le_bytes(data[10..12].try_into().ok()?)
    };

    if oldest > 13 || latest > 13 || latest < oldest {
        return None;
    }
    if index_offset < 12 || index_offset as usize >= data.len() {
        return None;
    }

    let (file_count, _footer_size) = if latest <= 1 {
        if file_count == 0 || file_count % 136 != 0 {
            return None;
        }
        (file_count / 136, 136usize)
    } else {
        (file_count, 144usize)
    };

    if file_count > 32768 {
        return None;
    }

    Some(ListingHeader { index_offset, file_count, oldest, latest })
}

fn decode_utf16le_padded(bytes: &[u8]) -> String {
    let mut chars = Vec::new();
    for chunk in bytes.chunks(2) {
        if chunk.len() < 2 { break; }
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if code_unit == 0 { break; }
        if let Some(c) = char::from_u32(code_unit as u32) {
            chars.push(c);
        }
    }
    chars.into_iter().collect()
}
pub fn is_minecraft_360_archive(data: &[u8]) -> bool {
    data.len() >= 8 && data[0..8] == ARCHIVE_MAGIC
}

pub fn parse_archive(data: &[u8]) -> Result<HashMap<String, ArchiveEntry>, ConversionError> {
    if data.len() < 8 {
        return Err(ConversionError::InvalidFormat("Archive data too short".to_string()));
    }
    if !is_minecraft_360_archive(data) {
        return Err(ConversionError::InvalidFormat("Invalid Minecraft 360 archive magic".to_string()));
    }

    let num_entries = read_be_u32(&data[8..12]) as usize;
    let _header_size = read_be_u32(&data[12..16]) as usize;
    let mut offset = 16usize;
    let mut entries = HashMap::new();
    for _ in 0..num_entries {
        if offset + 144 > data.len() {
            break;
        }

        let name_bytes = &data[offset..offset + 128];
        let name = decode_utf16_be_padded(name_bytes);
        let length = read_be_u32(&data[offset + 128..offset + 132]);
        let file_offset = read_be_u32(&data[offset + 132..offset + 136]) as u64;
        let timestamp = read_be_u64(&data[offset + 136..offset + 144]);
        offset += 144;
        entries.insert(name.clone(), ArchiveEntry {
            name,
            offset: file_offset,
            length,
            timestamp,
        });
    }

    Ok(entries)
}

fn decode_utf16_be_padded(bytes: &[u8]) -> String {
    let mut chars = Vec::new();
    for chunk in bytes.chunks(2) {
        if chunk.len() < 2 {
            break;
        }
        let code_unit = u16::from_be_bytes([chunk[0], chunk[1]]);
        if code_unit == 0 {
            break;
        }
        if let Some(c) = char::from_u32(code_unit as u32) {
            chars.push(c);
        }
    }
    chars.into_iter().collect()
}

fn read_be_u32(bytes: &[u8]) -> u32 {
    if bytes.len() < 4 { return 0; }
    ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
}

fn read_be_u64(bytes: &[u8]) -> u64 {
    if bytes.len() < 8 { return 0; }
    ((bytes[0] as u64) << 56) | ((bytes[1] as u64) << 48) | ((bytes[2] as u64) << 40) | ((bytes[3] as u64) << 32)
        | ((bytes[4] as u64) << 24) | ((bytes[5] as u64) << 16) | ((bytes[6] as u64) << 8) | (bytes[7] as u64)
}

pub fn extract_file_from_archive(data: &[u8], entry: &ArchiveEntry) -> Result<Vec<u8>, ConversionError> {
    let start = entry.offset as usize;
    let end = start + entry.length as usize;
    if end > data.len() {
        return Err(ConversionError::InvalidFormat(format!(
            "Archive entry '{}' at offset {} with length {} exceeds total size {}",
            entry.name, entry.offset, entry.length, data.len()
        )));
    }
    Ok(data[start..end].to_vec())
}

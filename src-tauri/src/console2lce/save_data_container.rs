use std::collections::HashMap;
use byteorder::{LittleEndian, WriteBytesExt};
use crate::console2lce::error::ConversionError;
const SAVE_MAGIC: [u8; 6] = [0x01, 0x00, 0x00, 0x00, 0x01, 0x00];
const CONTAINER_VERSION: u32 = 2;
pub fn write_save_data_container(
    files: &[(String, Vec<u8>)],
) -> Result<Vec<u8>, ConversionError> {
    let mut data = Vec::new();
    data.extend_from_slice(&SAVE_MAGIC);
    data.write_u32::<LittleEndian>(CONTAINER_VERSION)?;
    data.write_u32::<LittleEndian>(files.len() as u32)?;
    let mut file_offsets = Vec::new();
    for (name, content) in files {
        file_offsets.push((name.clone(), data.len() as u64, content.len() as u64));
        data.extend_from_slice(content);
        while data.len() % 4 != 0 {
            data.push(0);
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for (name, offset, length) in &file_offsets {
        data.write_u64::<LittleEndian>(*offset)?;
        data.write_u64::<LittleEndian>(*length)?;
        data.write_u64::<LittleEndian>(timestamp)?;
        let name_bytes = name.as_bytes();
        data.write_u16::<LittleEndian>(name_bytes.len() as u16)?;
        data.extend_from_slice(name_bytes);
        data.push(0);
        while data.len() % 4 != 0 {
            data.push(0);
        }
    }

    Ok(data)
}

pub fn write_save_data_ms(
    level_dat: &[u8],
    regions: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, ConversionError> {
    let mut files = Vec::new();
    files.push(("level.dat".to_string(), level_dat.to_vec()));
    for (path, region_data) in regions {
        files.push((path.clone(), region_data.clone()));
    }

    write_save_data_container(&files)
}

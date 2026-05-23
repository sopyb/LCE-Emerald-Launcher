use std::collections::HashMap;
#[derive(Debug, Clone, Copy)]
pub struct LegacyBlockState {
    pub id: u8,
    pub data: u8,
}

impl LegacyBlockState {
    pub fn new(id: u8, data: u8) -> Self {
        LegacyBlockState { id, data }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkConversionContext {
    pub preserve_dynamic_chunk_data: bool,
    pub global_modern_section_shift: Option<i32>,
    pub unknown_blocks: Vec<String>,
}

impl ChunkConversionContext {
    pub fn new() -> Self {
        ChunkConversionContext {
            preserve_dynamic_chunk_data: false,
            global_modern_section_shift: None,
            unknown_blocks: Vec::new(),
        }
    }

    pub fn record_unknown_modern_block(&mut self, name: &str) {
        if !self.unknown_blocks.iter().any(|s| s == name) {
            self.unknown_blocks.push(name.to_string());
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionOptions {
    pub profile: String,
    pub target_version: String,
    pub game_version: String,
    pub preserve_entities: bool,
    pub preview: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        ConversionOptions {
            profile: "large".to_string(),
            target_version: "TU19".to_string(),
            game_version: "1.13".to_string(),
            preserve_entities: false,
            preview: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub success: bool,
    pub message: String,
    pub chunk_count: usize,
    pub unknown_blocks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedJavaWorld {
    pub level_dat: HashMap<String, Vec<u8>>,
    pub region_files: Vec<JavaRegionFile>,
    pub dimension_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct JavaRegionFile {
    pub path: String,
    pub dimension: String,
    pub data: Vec<u8>,
    pub chunk_count: usize,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub offset: u64,
    pub length: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct StfsPackageDescriptor {
    pub magic: u32,
    pub content_type: u32,
    pub meta_data_size: u32,
    pub total_block_count: u32,
    pub block_allocation_table_size: u32,
}

#[derive(Debug, Clone)]
pub struct SavegameEnvelope {
    pub data: Vec<u8>,
    pub decompressed_size: u32,
    pub compression_type: String,
}

pub struct DecodedSection {
    pub section_y: i32,
    pub blocks: [u8; 4096],
    pub data: [u8; 2048],
    pub sky_light: Option<[u8; 2048]>,
    pub block_light: Option<[u8; 2048]>,
    pub non_air_count: i32,
}

pub const CHUNK_BLOCKS: usize = 65536;
pub const CHUNK_NIBBLES: usize = 32768;
pub const HEIGHTMAP_SIZE: usize = 256;
pub const BIOMES_SIZE: usize = 256;
pub struct SaveDataContainerHeader {
    pub magic: [u8; 6],
    pub version: u32,
    pub file_count: u32,
}

pub struct SaveDataContainerFileEntry {
    pub offset: u64,
    pub length: u64,
    pub timestamp: u64,
    pub name: String,
}

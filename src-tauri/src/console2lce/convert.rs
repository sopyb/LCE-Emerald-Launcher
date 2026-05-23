use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::console2lce::error::ConversionError;
use crate::console2lce::models::*;
use crate::console2lce::nbt_util::{self, NbtHelper};
use crate::console2lce::modern_chunk_writer;
use crate::console2lce::lce_chunk_payload;
use crate::console2lce::lce_region::LceRegionFile;
use crate::console2lce::level_dat;
use crate::console2lce::save_data_container;
use crate::console2lce::world_profile;
pub fn convert_xbox360_save_to_lce(
    input_path: &str,
    output_path: &str,
    _options: &ConversionOptions,
) -> Result<ConversionResult, ConversionError> {
    eprintln!("[convert] convert_xbox360_save_to_lce start");
    eprintln!("[convert]   input_path = {:?}", input_path);
    eprintln!("[convert]   output_path = {:?}", output_path);
    let data = std::fs::read(input_path)
        .map_err(|e| {
            eprintln!("[convert] ERROR reading input: {}", e);
            ConversionError::Io(e)
        })?;
    eprintln!("[convert]   read {} bytes from input", data.len());
    let decompressed = if crate::console2lce::stfs::is_stfs_package(&data) {
        eprintln!("[convert]   detected STFS package, extracting savegame.dat...");
        let savegame_dat = crate::console2lce::stfs::extract_savegame_from_stfs(&data)
            .map_err(|e| {
                eprintln!("[convert] ERROR extracting from STFS: {}", e);
                ConversionError::InvalidFormat(format!("STFS extraction failed: {}", e))
            })?;
        eprintln!("[convert]   savegame.dat extracted, {} bytes", savegame_dat.len());
        eprintln!("[convert]   decompressing STFS savegame data...");
        crate::console2lce::stfs::try_decompress_stfs_savegame(&savegame_dat)
            .map_err(|e| {
                eprintln!("[convert] ERROR decompressing STFS savegame: {}", e);
                ConversionError::DecompressionFailed(format!("STFS savegame decompression failed: {}", e))
            })?
    } else if (data.len() >= 8 && data[0..8] == [0x0B, 0xFC, 0x4A, 0x46, 0xAE, 0x7A, 0x43, 0x2B]) || input_path.ends_with(".dat") {
        eprintln!("[convert]   detected raw savegame.dat, decompressing directly...");
        crate::console2lce::stfs::try_decompress_savegame(&data)
            .map_err(|e| {
                eprintln!("[convert] ERROR decompressing savegame: {}", e);
                ConversionError::DecompressionFailed(format!("Savegame decompression failed: {}", e))
            })?
    } else {
        return Err(ConversionError::InvalidFormat(
            "Not an STFS package (.bin) or savegame.dat (.dat)".to_string()
        ));
    };
    eprintln!("[convert]   decompressed {} bytes", decompressed.len());
    eprintln!("[convert]   parsing file listing...");
    let files = crate::console2lce::archive::parse_file_listing(&decompressed)
        .map_err(|e| {
            eprintln!("[convert] ERROR parsing file listing: {}", e);
            ConversionError::InvalidFormat(format!("File listing parse failed: {}", e))
        })?;
    eprintln!("[convert]   file listing has {} entries", files.len());
    let mut level_dat_data: Vec<u8> = Vec::new();
    let mut regions: HashMap<String, Vec<u8>> = HashMap::new();
    let mut total_chunks = 0;
    for (name, file_data) in &files {
        eprintln!("[convert]     file: {:?} ({} bytes)", name, file_data.len());
        if name == "level.dat" || name.ends_with("/level.dat") {
            level_dat_data = file_data.clone();
        } else if name.ends_with(".mcr") || name.ends_with(".mca") {
            if let Ok(chunks) = crate::console2lce::region::read_java_mcr_region(file_data) {
                eprintln!("[convert]     region {} has {} chunks", name, chunks.len());
                let mut dim_region = LceRegionFile::new();
                for (local_x, local_z, chunk_raw) in &chunks {
                    if let Ok(decompressed) = crate::console2lce::nbt_util::read_mcr_chunk(chunk_raw, 0) {
                        if let Ok((_name, root_tag)) = crate::console2lce::nbt_util::read_nbt(&decompressed) {
                            let level = root_tag.as_compound()
                                .and_then(|m| m.get("Level").and_then(|t| t.as_compound()))
                                .or_else(|| root_tag.as_compound())
                                .cloned()
                                .unwrap_or_default();
                            let cx = crate::console2lce::nbt_util::NbtHelper::get_int(&level, "xPos").unwrap_or(*local_x as i32);
                            let cz = crate::console2lce::nbt_util::NbtHelper::get_int(&level, "zPos").unwrap_or(*local_z as i32);
                            if let Ok(payload) = crate::console2lce::lce_chunk_payload::encode_legacy_nbt(
                                cx, cz,
                                &[0u8; CHUNK_BLOCKS], &[0u8; CHUNK_NIBBLES],
                                &[0u8; CHUNK_NIBBLES], &[0u8; CHUNK_NIBBLES],
                                &[0u8; 256], &[1u8; 256],
                                0, 0,
                                Vec::new(), Vec::new(), None,
                            ) {
                                let _ = dim_region.add_chunk(cx, cz, &payload);
                                total_chunks += 1;
                            }
                        }
                    }
                }
                if let Ok(data) = dim_region.build() {
                    let region_key = format!("region/{}", name);
                    regions.insert(region_key, data);
                }
            }
        }
    }

    eprintln!("[convert]   writing output to {:?}", output_path);
    if total_chunks == 0 {
        eprintln!("[convert]   no chunks found. copying input directly");
        if let Some(parent) = std::path::Path::new(output_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(output_path, &data)
            .map_err(ConversionError::Io)?;
        return Ok(ConversionResult {
            success: true,
            message: "No chunks found, copied raw data".to_string(),
            chunk_count: 0,
            unknown_blocks: Vec::new(),
        });
    }

    let ms_data = save_data_container::write_save_data_ms(&level_dat_data, &regions)
        .map_err(|e| {
            eprintln!("[convert] ERROR building saveData.ms: {}", e);
            e
        })?;

    if let Some(parent) = std::path::Path::new(output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(output_path, &ms_data)
        .map_err(ConversionError::Io)?;

    eprintln!("[convert] convert_xbox360_save_to_lce done, {} chunks", total_chunks);
    Ok(ConversionResult {
        success: true,
        message: format!("Converted {} chunks from Xbox 360 save", total_chunks),
        chunk_count: total_chunks,
        unknown_blocks: Vec::new(),
    })
}

pub fn convert_java_world_to_lce(
    world_path: &str,
    output_path: &str,
    options: &ConversionOptions,
) -> Result<ConversionResult, ConversionError> {
    eprintln!("[convert] convert_java_world_to_lce start");
    eprintln!("[convert]   world_path = {:?}", world_path);
    eprintln!("[convert]   output_path = {:?}", output_path);
    eprintln!("[convert]   profile = {:?}", options.profile);
    let _profile = world_profile::get_profile(&options.profile)
        .ok_or_else(|| {
            eprintln!("[convert] ERROR: unknown profile {:?}", options.profile);
            ConversionError::InvalidFormat(format!("Unknown profile: {}", options.profile))
        })?;

    let world_dir = Path::new(world_path);
    eprintln!("[convert] world_dir resolved to {:?}", world_dir);
    eprintln!("[convert] world_dir.is_dir() = {:?}", world_dir.is_dir());
    if !world_dir.is_dir() {
        eprintln!("[convert] ERROR: world directory not found at {:?}", world_dir);
        eprintln!("[convert]   Checking existence: {:?}", world_dir.exists());
        return Err(ConversionError::InvalidFormat(format!("World directory not found: {}", world_path)));
    }

    let level_dat_path = world_dir.join("level.dat");
    eprintln!("[convert] reading level.dat: {:?}", level_dat_path);
    eprintln!("[convert]   level.dat exists: {:?}", level_dat_path.exists());
    let level_dat_data = fs::read(&level_dat_path)
        .map_err(|e| {
            eprintln!("[convert] ERROR reading level.dat: {}", e);
            e
        })?;
    eprintln!("[convert]   level.dat size: {} bytes", level_dat_data.len());
    let mut context = ChunkConversionContext::new();
    context.preserve_dynamic_chunk_data = options.preserve_entities;
    eprintln!("[convert] converting level.dat...");
    let lce_level_dat = level_dat::convert_level_dat(
        &level_dat_data, &options.profile, &options.game_version, &mut context,
    )?;
    eprintln!("[convert] level.dat converted, size: {} bytes", lce_level_dat.len());
    let dimensions = vec!["region".to_string(), "DIM-1/region".to_string(), "DIM1/region".to_string()];
    let mut regions: HashMap<String, Vec<u8>> = HashMap::new();
    let mut total_chunks = 0;
    for dim_dir in &dimensions {
        let dim_path = world_dir.join(dim_dir);
        eprintln!("[convert] checking dimension dir: {:?} (exists: {:?})", dim_path, dim_path.is_dir());
        if !dim_path.is_dir() {
            eprintln!("[convert]   skipping (not a directory)");
            continue;
        }

        let mut dim_region = LceRegionFile::new();
        let entries = match fs::read_dir(&dim_path) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[convert] ERROR reading dim dir {:?}: {}", dim_path, e);
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[convert] ERROR reading dir entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if !file_name.ends_with(".mca") && !file_name.ends_with(".mcr") {
                eprintln!("[convert]   skipping non-region file: {:?}", path);
                continue;
            }

            eprintln!("[convert]   processing region file: {:?}", path);
            let region_data = match fs::read(&path) {
                Ok(d) => {
                    eprintln!("[convert]     read {} bytes", d.len());
                    d
                }
                Err(e) => {
                    eprintln!("[convert]     ERROR reading region file: {}", e);
                    continue;
                }
            };

            let chunks = match crate::console2lce::region::read_java_mcr_region(&region_data) {
                Ok(c) => {
                    eprintln!("[convert]     found {} chunks", c.len());
                    c
                }
                Err(e) => {
                    eprintln!("[convert]     ERROR parsing region: {}", e);
                    continue;
                }
            };

            for (local_x, local_z, chunk_raw) in &chunks {
                eprintln!("[convert]     converting chunk ({}, {})", local_x, local_z);
                let decompressed = match nbt_util::read_mcr_chunk(chunk_raw, 0) {
                    Ok(d) => {
                        eprintln!("[convert]       decompressed {} bytes", d.len());
                        d
                    }
                    Err(e) => {
                        eprintln!("[convert]       ERROR decompressing chunk: {}", e);
                        continue;
                    }
                };

                let (name, root_tag) = match nbt_util::read_nbt(&decompressed) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[convert]       ERROR parsing chunk NBT: {}", e);
                        continue;
                    }
                };
                eprintln!("[convert]       chunk root name: {:?}", name);
                let level = root_tag.as_compound()
                    .and_then(|m| m.get("Level").and_then(|t| t.as_compound()))
                    .or_else(|| root_tag.as_compound())
                    .cloned()
                    .unwrap_or_default();

                let source_chunk_x = NbtHelper::get_int(&level, "xPos").unwrap_or(0);
                let source_chunk_z = NbtHelper::get_int(&level, "zPos").unwrap_or(0);
                eprintln!("[convert]       source chunk pos: ({}, {})", source_chunk_x, source_chunk_z);
                let new_chunk_x = source_chunk_x;
                let new_chunk_z = source_chunk_z;
                let has_sections = level.contains_key("Sections");
                let uses_modern_content = level.contains_key("Status") || level.contains_key("block_states");
                eprintln!("[convert]       has_sections: {}, uses_modern_content: {}", has_sections, uses_modern_content);
                let (blocks, data, _sky_light, _block_light) = if has_sections {
                    let shift = context.global_modern_section_shift;
                    modern_chunk_writer::flatten_anvil_sections(&level, shift, &mut context)
                } else {
                    let blocks_arr = NbtHelper::get_byte_array_or_default(&level, "Blocks", CHUNK_BLOCKS);
                    let data_arr = NbtHelper::get_byte_array_or_default(&level, "Data", CHUNK_NIBBLES);
                    let sky_arr = NbtHelper::get_byte_array_or_default(&level, "SkyLight", CHUNK_NIBBLES);
                    let block_arr = NbtHelper::get_byte_array_or_default(&level, "BlockLight", CHUNK_NIBBLES);
                    let mut bl = [0u8; CHUNK_BLOCKS];
                    let mut da = [0u8; CHUNK_NIBBLES];
                    let mut sk = [0u8; CHUNK_NIBBLES];
                    let mut blk = [0u8; CHUNK_NIBBLES];
                    bl[..blocks_arr.len().min(CHUNK_BLOCKS)].copy_from_slice(&blocks_arr[..blocks_arr.len().min(CHUNK_BLOCKS)]);
                    da[..data_arr.len().min(CHUNK_NIBBLES)].copy_from_slice(&data_arr[..data_arr.len().min(CHUNK_NIBBLES)]);
                    sk[..sky_arr.len().min(CHUNK_NIBBLES)].copy_from_slice(&sky_arr[..sky_arr.len().min(CHUNK_NIBBLES)]);
                    blk[..block_arr.len().min(CHUNK_NIBBLES)].copy_from_slice(&block_arr[..block_arr.len().min(CHUNK_NIBBLES)]);
                    (bl, da, sk, blk)
                };

                let height_map = NbtHelper::get_byte_array_or_default(&level, "HeightMap", HEIGHTMAP_SIZE);
                let biomes_data = NbtHelper::get_byte_array_or_default(&level, "Biomes", BIOMES_SIZE);
                let mut biomes_arr = [0u8; BIOMES_SIZE];
                let len = biomes_data.len().min(BIOMES_SIZE);
                biomes_arr[..len].copy_from_slice(&biomes_data[..len]);
                if uses_modern_content {
                    biomes_arr.fill(1);
                }

                let mut hm = [0u8; HEIGHTMAP_SIZE];
                let len = height_map.len().min(HEIGHTMAP_SIZE);
                hm[..len].copy_from_slice(&height_map[..len]);
                let last_update = NbtHelper::get_long(&level, "LastUpdate").unwrap_or(0);
                let inhabited_time = NbtHelper::get_long(&level, "InhabitedTime").unwrap_or(0);
                let entities = if context.preserve_dynamic_chunk_data && !uses_modern_content {
                    NbtHelper::get_list(&level, "Entities").unwrap_or_default()
                } else {
                    Vec::new()
                };

                let tile_entities = if context.preserve_dynamic_chunk_data {
                    NbtHelper::get_list(&level, "TileEntities").unwrap_or_default()
                } else {
                    Vec::new()
                };

                eprintln!("[convert]       encoding chunk NBT...");
                let chunk_payload = match lce_chunk_payload::encode_legacy_nbt(
                    new_chunk_x, new_chunk_z,
                    &blocks, &data, &[0u8; 32768], &[0u8; 32768],
                    &hm, &biomes_arr,
                    last_update, inhabited_time,
                    entities, tile_entities, None,
                ) {
                    Ok(p) => {
                        eprintln!("[convert]       encoded {} bytes", p.len());
                        p
                    }
                    Err(e) => {
                        eprintln!("[convert]       ERROR encoding chunk: {}", e);
                        continue;
                    }
                };

                if let Err(e) = dim_region.add_chunk(new_chunk_x, new_chunk_z, &chunk_payload) {
                    eprintln!("[convert]       ERROR adding chunk to region: {}", e);
                    continue;
                }
                total_chunks += 1;
            }
        }

        eprintln!("[convert]   building region file for {:?}...", dim_dir);
        let region_bytes = match dim_region.build() {
            Ok(b) => {
                eprintln!("[convert]   region built, {} bytes", b.len());
                b
            }
            Err(e) => {
                eprintln!("[convert]   ERROR building region: {}", e);
                continue;
            }
        };

        let region_key = if dim_dir == "region" {
            "region/r.0.0.mcr".to_string()
        } else {
            format!("{}/r.0.0.mcr", dim_dir)
        };

        regions.insert(region_key, region_bytes);
    }

    eprintln!("[convert] writing saveData.ms with {} chunks", total_chunks);
    let ms_data = match save_data_container::write_save_data_ms(&lce_level_dat, &regions) {
        Ok(d) => {
            eprintln!("[convert] saveData.ms content built, {} bytes", d.len());
            d
        }
        Err(e) => {
            eprintln!("[convert] ERROR building saveData.ms: {}", e);
            return Err(e);
        }
    };

    eprintln!("[convert] writing to {:?}", output_path);
    if let Err(e) = fs::write(output_path, &ms_data) {
        eprintln!("[convert] ERROR writing output file: {}", e);
        return Err(ConversionError::Io(e));
    }
    eprintln!("[convert] done!");

    Ok(ConversionResult {
        success: true,
        message: format!("Successfully converted {} chunks", total_chunks),
        chunk_count: total_chunks,
        unknown_blocks: context.unknown_blocks,
    })
}

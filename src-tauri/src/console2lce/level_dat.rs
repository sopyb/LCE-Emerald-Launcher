use std::collections::HashMap;
use crate::console2lce::error::ConversionError;
use crate::console2lce::nbt_util::{NbtTag, NbtHelper, write_zlibbed_nbt};
use crate::console2lce::world_profile::{get_profile, world_center_offset};
use crate::console2lce::models::ChunkConversionContext;
pub fn convert_level_dat(
    source_data: &[u8],
    profile_name: &str,
    game_version: &str,
    _context: &mut ChunkConversionContext,
) -> Result<Vec<u8>, ConversionError> {
    let (_name, root) = crate::console2lce::nbt_util::read_gzipped_nbt(source_data)?;
    let data = match root.as_compound() {
        Some(m) => m.clone(),
        None => return Err(ConversionError::InvalidFormat("level.dat root is not a compound".to_string())),
    };

    let data_tag = NbtHelper::get_compound(&data, "Data")
        .unwrap_or_else(|| data.clone());

    let profile = get_profile(profile_name).ok_or_else(|| {
        ConversionError::InvalidFormat(format!("Unknown profile: {}", profile_name))
    })?;

    let center_offset = world_center_offset(profile.xz_size);
    let spawn_x = NbtHelper::get_int(&data_tag, "SpawnX").unwrap_or(0) as i32;
    let spawn_z = NbtHelper::get_int(&data_tag, "SpawnZ").unwrap_or(0) as i32;
    let new_spawn_x = clamp_to_world(spawn_x, center_offset, profile.xz_size as i32);
    let new_spawn_z = clamp_to_world(spawn_z, center_offset, profile.xz_size as i32);
    let game_type = NbtHelper::get_int(&data_tag, "GameType").unwrap_or(0);
    let rain_time = NbtHelper::get_int(&data_tag, "rainTime").unwrap_or(0);
    let thunder_time = NbtHelper::get_int(&data_tag, "thunderTime").unwrap_or(0);
    let raining = NbtHelper::get_byte(&data_tag, "raining").unwrap_or(0) != 0;
    let thundering = NbtHelper::get_byte(&data_tag, "thundering").unwrap_or(0) != 0;
    let seed = NbtHelper::get_long(&data_tag, "RandomSeed").unwrap_or(0);
    let level_name = NbtHelper::get_string(&data_tag, "LevelName").unwrap_or_else(|| "Imported World".to_string());
    let day_time = NbtHelper::get_int(&data_tag, "DayTime").unwrap_or(0);
    let allow_commands = NbtHelper::get_byte(&data_tag, "allowCommands").unwrap_or(0) != 0;
    let difficulty = NbtHelper::get_byte(&data_tag, "Difficulty").unwrap_or(2);
    let difficulty_locked = NbtHelper::get_byte(&data_tag, "DifficultyLocked").unwrap_or(0) != 0;
    let hardcore = NbtHelper::get_byte(&data_tag, "hardcore").unwrap_or(0) != 0;
    let map_features = NbtHelper::get_byte(&data_tag, "MapFeatures").unwrap_or(1) != 0;
    let generator_name = NbtHelper::get_string(&data_tag, "generatorName").unwrap_or_else(|| "default".to_string());
    let generator_version = NbtHelper::get_int(&data_tag, "generatorVersion").unwrap_or(1);
    let generator_options = NbtHelper::get_string(&data_tag, "generatorOptions").unwrap_or_default();
    let initialized = NbtHelper::get_byte(&data_tag, "initialized").unwrap_or(1) != 0;
    let clear_weather = rained_since_last_thunder_cleanup(&data_tag);
    let mut game_rules = HashMap::new();
    if let Some(rules) = NbtHelper::get_compound(&data_tag, "GameRules") {
        for (k, v) in &rules {
            if let Some(s) = v.get_string() {
                game_rules.insert(k.clone(), NbtTag::String(s.to_string()));
            }
        }
    }

    let mut lce_data = HashMap::new();
    lce_data.insert("allowCommands".to_string(), NbtTag::Byte(if allow_commands { 1 } else { 0 }));
    lce_data.insert("clearedWeather".to_string(), NbtTag::Byte(if clear_weather { 1 } else { 0 }));
    lce_data.insert("clearWeatherTime".to_string(), NbtTag::Int(0));
    lce_data.insert("currentSaveVersion".to_string(), NbtTag::Int(9));
    lce_data.insert("DayTime".to_string(), NbtTag::Int(day_time));
    lce_data.insert("Difficulty".to_string(), NbtTag::Byte(difficulty));
    lce_data.insert("DifficultyLocked".to_string(), NbtTag::Byte(if difficulty_locked { 1 } else { 0 }));
    lce_data.insert("GameType".to_string(), NbtTag::Int(game_type));
    lce_data.insert("generatorName".to_string(), NbtTag::String(generator_name));
    lce_data.insert("generatorVersion".to_string(), NbtTag::Int(generator_version));
    lce_data.insert("generatorOptions".to_string(), NbtTag::String(generator_options));
    lce_data.insert("hardcore".to_string(), NbtTag::Byte(if hardcore { 1 } else { 0 }));
    lce_data.insert("HellScale".to_string(), NbtTag::Int(profile.hell_scale as i32));
    lce_data.insert("initialized".to_string(), NbtTag::Byte(if initialized { 1 } else { 0 }));
    lce_data.insert("LevelName".to_string(), NbtTag::String(level_name));
    lce_data.insert("MapFeatures".to_string(), NbtTag::Byte(if map_features { 1 } else { 0 }));
    lce_data.insert("originalSaveVersion".to_string(), NbtTag::Int(7));
    lce_data.insert("rainTime".to_string(), NbtTag::Int(rain_time));
    lce_data.insert("RandomSeed".to_string(), NbtTag::Long(seed));
    lce_data.insert("SpawnX".to_string(), NbtTag::Int(new_spawn_x));
    lce_data.insert("SpawnZ".to_string(), NbtTag::Int(new_spawn_z));
    lce_data.insert("thunderTime".to_string(), NbtTag::Int(thunder_time));
    lce_data.insert("XzSize".to_string(), NbtTag::Int(profile.xz_size as i32));
    lce_data.insert("raining".to_string(), NbtTag::Byte(if raining { 1 } else { 0 }));
    lce_data.insert("thundering".to_string(), NbtTag::Byte(if thundering { 1 } else { 0 }));
    if !game_rules.is_empty() {
        lce_data.insert("GameRules".to_string(), NbtTag::Compound(game_rules));
    }

    let version_info = build_version_info(game_version);
    lce_data.insert("Version".to_string(), NbtTag::Compound(version_info));
    let root = NbtTag::Compound({
        let mut m = HashMap::new();
        m.insert("Data".to_string(), NbtTag::Compound(lce_data));
        m
    });

    write_zlibbed_nbt("", &root)
}

fn clamp_to_world(value: i32, center: i32, xz_size: i32) -> i32 {
    let half = xz_size * 8;
    value.clamp(center - half, center + half - 1)
}

fn rained_since_last_thunder_cleanup(data: &HashMap<String, NbtTag>) -> bool {
    let rain_time = NbtHelper::get_int(data, "rainTime").unwrap_or(0);
    let thunder_time = NbtHelper::get_int(data, "thunderTime").unwrap_or(0);
    rain_time > 0 && thunder_time == 0
}

fn build_version_info(game_version: &str) -> HashMap<String, NbtTag> {
    let mut version = HashMap::new();
    version.insert("Id".to_string(), NbtTag::Int(19133));
    version.insert("Name".to_string(), NbtTag::String(game_version.to_string()));
    version.insert("Snapshot".to_string(), NbtTag::Byte(0));
    version
}

use std::collections::HashMap;
use crate::console2lce::models::{LegacyBlockState, ChunkConversionContext, CHUNK_BLOCKS, CHUNK_NIBBLES};
use crate::console2lce::nbt_util::{NbtTag, NbtHelper};
use crate::console2lce::block_mapping;
pub fn map_modern_block_state(name: &str, properties: &HashMap<String, String>, context: &mut ChunkConversionContext) -> LegacyBlockState {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    if let Some(result) = try_map_fluid_block(name, properties) {
        return result;
    }
    if let Some(result) = try_map_slab_block(name, properties) {
        return result;
    }
    if let Some(result) = try_map_directional_block(name, properties) {
        return result;
    }
    if let Some(result) = try_map_flattened_colored_block(name) {
        return result;
    }
    if let Some(legacy) = block_mapping::MODERN_DIRECT_MAP.get(name) {
        return *legacy;
    }
    if let Some(result) = try_map_colored_block(name, properties) {
        return result;
    }
    if let Some(result) = try_map_wood_block(name, properties) {
        return result;
    }
    if let Some(result) = try_map_variant_block(name, properties) {
        return result;
    }

    context.record_unknown_modern_block(name);
    LegacyBlockState::new(0, 0)
}

fn get_prop<'a>(properties: &'a HashMap<String, String>, key: &str) -> &'a str {
    properties.get(key).map(|s| s.as_str()).unwrap_or("")
}

fn get_bool_prop(properties: &HashMap<String, String>, key: &str) -> bool {
    get_prop(properties, key) == "true"
}

fn get_int_prop(properties: &HashMap<String, String>, key: &str, default: i32) -> i32 {
    get_prop(properties, key).parse::<i32>().unwrap_or(default)
}

fn try_map_fluid_block(name: &str, properties: &HashMap<String, String>) -> Option<LegacyBlockState> {
    if name != "water" && name != "lava" {
        return None;
    }
    let level = get_int_prop(properties, "level", 0).clamp(0, 15);
    let is_source = level == 0;
    if name == "water" {
        Some(LegacyBlockState::new(if is_source { 9 } else { 8 }, level as u8))
    } else {
        Some(LegacyBlockState::new(if is_source { 11 } else { 10 }, level as u8))
    }
}

fn try_map_slab_block(name: &str, properties: &HashMap<String, String>) -> Option<LegacyBlockState> {
    if !name.ends_with("_slab") {
        return None;
    }
    let slab_type = get_prop(properties, "type");
    let is_top = slab_type == "top";
    let is_double = slab_type == "double";
    if let Some(variant) = get_wood_slab_variant(name) {
        if is_double {
            return Some(LegacyBlockState::new(125, variant));
        }
        return Some(LegacyBlockState::new(126, variant | if is_top { 8 } else { 0 }));
    }

    if let Some(variant) = get_stone_slab_variant(name) {
        if is_double {
            return Some(LegacyBlockState::new(43, variant));
        }
        return Some(LegacyBlockState::new(44, variant | if is_top { 8 } else { 0 }));
    }

    if is_double {
        Some(LegacyBlockState::new(43, 0))
    } else {
        Some(LegacyBlockState::new(44, if is_top { 8 } else { 0 }))
    }
}

fn get_wood_slab_variant(name: &str) -> Option<u8> {
    match name {
        "oak_slab" => Some(0),
        "spruce_slab" => Some(1),
        "birch_slab" => Some(2),
        "jungle_slab" => Some(3),
        "acacia_slab" => Some(4),
        "dark_oak_slab" => Some(5),
        _ => None,
    }
}

fn get_stone_slab_variant(name: &str) -> Option<u8> {
    match name {
        "stone_slab" | "smooth_stone_slab" | "andesite_slab" | "polished_andesite_slab"
        | "diorite_slab" | "polished_diorite_slab" | "granite_slab" | "polished_granite_slab" => Some(0),
        "sandstone_slab" | "smooth_sandstone_slab" | "cut_sandstone_slab"
        | "red_sandstone_slab" | "smooth_red_sandstone_slab" | "cut_red_sandstone_slab" => Some(1),
        "cobblestone_slab" | "mossy_cobblestone_slab" => Some(3),
        "brick_slab" => Some(4),
        "stone_brick_slab" | "mossy_stone_brick_slab" => Some(5),
        "nether_brick_slab" | "red_nether_brick_slab" => Some(6),
        "quartz_slab" | "smooth_quartz_slab" | "purpur_slab"
        | "prismarine_slab" | "prismarine_brick_slab" | "dark_prismarine_slab" => Some(7),
        _ => None,
    }
}

fn try_map_directional_block(name: &str, properties: &HashMap<String, String>) -> Option<LegacyBlockState> {
    match name {
        "ladder" => {
            let data = map_ladder_facing(get_prop(properties, "facing"));
            Some(LegacyBlockState::new(65, data))
        }
        "vine" => {
            let data = map_vine_faces(properties);
            Some(LegacyBlockState::new(106, data))
        }
        "lever" => {
            let mut data = map_lever_data(get_prop(properties, "face"), get_prop(properties, "facing"));
            if get_bool_prop(properties, "powered") { data |= 8; }
            Some(LegacyBlockState::new(69, data))
        }
        _ => {
            if name.ends_with("_button") {
                let id = if is_wood_family(name) { 143 } else { 77 };
                let mut data = map_button_facing(get_prop(properties, "facing"));
                if get_bool_prop(properties, "powered") { data |= 8; }
                return Some(LegacyBlockState::new(id, data));
            }
            if name.ends_with("_fence_gate") {
                let mut data = map_fence_gate_facing(get_prop(properties, "facing"));
                if get_bool_prop(properties, "open") { data |= 4; }
                return Some(LegacyBlockState::new(107, data));
            }
            if name.ends_with("_pressure_plate") {
                let id = match name {
                    "light_weighted_pressure_plate" => 147,
                    "heavy_weighted_pressure_plate" => 148,
                    n if is_wood_family(n) => 72,
                    _ => 70,
                };
                let powered = get_bool_prop(properties, "powered") || get_int_prop(properties, "power", 0) > 0;
                return Some(LegacyBlockState::new(id, if powered { 1 } else { 0 }));
            }
            if name.ends_with("_door") || name == "iron_door" {
                let id = if name == "iron_door" { 71 } else { 64 };
                let half = get_prop(properties, "half");
                if half == "upper" {
                    let mut data: u8 = 8;
                    if get_prop(properties, "hinge") == "right" { data |= 1; }
                    if get_bool_prop(properties, "powered") { data |= 2; }
                    return Some(LegacyBlockState::new(id, data));
                }
                let mut data = map_door_facing(get_prop(properties, "facing"));
                if get_bool_prop(properties, "open") { data |= 4; }
                return Some(LegacyBlockState::new(id, data));
            }
            if let Some(stairs_id) = get_stairs_id(name) {
                let mut data = map_stairs_facing(get_prop(properties, "facing"));
                if get_prop(properties, "half") == "top" { data |= 4; }
                return Some(LegacyBlockState::new(stairs_id, data));
            }
            if name.ends_with("trapdoor") {
                let mut data = map_trapdoor_facing(get_prop(properties, "facing"));
                if get_bool_prop(properties, "open") { data |= 4; }
                if get_prop(properties, "half") == "top" { data |= 8; }
                return Some(LegacyBlockState::new(96, data));
            }
            if name == "dispenser" || name == "dropper" {
                let id = if name == "dropper" { 158 } else { 23 };
                let mut data = map_facing_data(get_prop(properties, "facing"));
                if get_bool_prop(properties, "triggered") { data |= 8; }
                return Some(LegacyBlockState::new(id, data));
            }
            if name == "piston" || name == "sticky_piston" {
                let id = if name == "sticky_piston" { 29 } else { 33 };
                let mut data = map_facing_data(get_prop(properties, "facing"));
                if get_bool_prop(properties, "extended") { data |= 8; }
                return Some(LegacyBlockState::new(id, data));
            }
            if name == "piston_head" {
                let mut data = map_facing_data(get_prop(properties, "facing"));
                if get_prop(properties, "type") == "sticky" { data |= 8; }
                return Some(LegacyBlockState::new(34, data));
            }
            if name == "redstone_wire" {
                let data = get_int_prop(properties, "power", 0).clamp(0, 15) as u8;
                return Some(LegacyBlockState::new(55, data));
            }
            if name == "repeater" {
                let id = if get_bool_prop(properties, "powered") { 94 } else { 93 };
                let dir = map_repeater_direction(get_prop(properties, "facing"));
                let delay = get_int_prop(properties, "delay", 1).clamp(1, 4);
                let data = dir | ((delay - 1) as u8) << 2;
                return Some(LegacyBlockState::new(id, data));
            }
            if name == "comparator" {
                let id = if get_bool_prop(properties, "powered") { 150 } else { 149 };
                let dir = map_repeater_direction(get_prop(properties, "facing"));
                let mut data = dir;
                if get_prop(properties, "mode") == "subtract" { data |= 4; }
                return Some(LegacyBlockState::new(id, data));
            }
            if name == "wall_torch" {
                let data = map_wall_torch_facing(get_prop(properties, "facing"));
                return Some(LegacyBlockState::new(50, data));
            }
            if name == "redstone_wall_torch" {
                let id = if get_bool_prop(properties, "lit") { 76 } else { 75 };
                let data = map_wall_torch_facing(get_prop(properties, "facing"));
                return Some(LegacyBlockState::new(id, data));
            }
            if name == "nether_wart" {
                let data = get_int_prop(properties, "age", 0).clamp(0, 3) as u8;
                return Some(LegacyBlockState::new(115, data));
            }
            if name == "pumpkin_stem" || name == "attached_pumpkin_stem" {
                let data = if name == "attached_pumpkin_stem" { 7 } else { get_int_prop(properties, "age", 0).clamp(0, 7) as u8 };
                return Some(LegacyBlockState::new(104, data));
            }
            if name == "melon_stem" || name == "attached_melon_stem" {
                let data = if name == "attached_melon_stem" { 7 } else { get_int_prop(properties, "age", 0).clamp(0, 7) as u8 };
                return Some(LegacyBlockState::new(105, data));
            }
            if name == "cocoa" {
                let age = get_int_prop(properties, "age", 0).clamp(0, 2) as u8;
                let dir = map_repeater_direction(get_prop(properties, "facing"));
                return Some(LegacyBlockState::new(127, (age << 2) | dir));
            }
            if name == "hay_block" {
                let data = match get_prop(properties, "axis") {
                    "x" => 4,
                    "z" => 8,
                    _ => 0,
                };
                return Some(LegacyBlockState::new(170, data));
            }
            if name == "quartz_pillar" {
                let data = match get_prop(properties, "axis") {
                    "x" => 3,
                    "z" => 4,
                    _ => 2,
                };
                return Some(LegacyBlockState::new(155, data));
            }
            if name == "nether_portal" {
                let data = if get_prop(properties, "axis") == "z" { 2 } else { 1 };
                return Some(LegacyBlockState::new(90, data));
            }
            None
        }
    }
}

fn is_wood_family(name: &str) -> bool {
    let woods = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];
    woods.iter().any(|w| name.starts_with(w))
}

fn get_stairs_id(name: &str) -> Option<u8> {
    match name {
        "oak_stairs" | "spruce_stairs" | "birch_stairs" | "jungle_stairs"
        | "acacia_stairs" | "dark_oak_stairs" => Some(53),
        "cobblestone_stairs" | "mossy_cobblestone_stairs" | "stone_stairs" => Some(67),
        "brick_stairs" => Some(108),
        "stone_brick_stairs" | "mossy_stone_brick_stairs" => Some(109),
        "nether_brick_stairs" => Some(114),
        "sandstone_stairs" | "red_sandstone_stairs" => Some(128),
        "quartz_stairs" | "purpur_stairs" => Some(156),
        _ => None,
    }
}

fn map_stairs_facing(facing: &str) -> u8 {
    match facing {
        "east" => 0, "west" => 1, "south" => 2, "north" => 3, _ => 0,
    }
}

fn map_ladder_facing(facing: &str) -> u8 {
    match facing {
        "north" => 2, "south" => 3, "west" => 4, "east" => 5, _ => 2,
    }
}

fn map_button_facing(facing: &str) -> u8 {
    match facing {
        "east" => 1, "west" => 2, "south" => 3, "north" => 4, _ => 1,
    }
}

fn map_fence_gate_facing(facing: &str) -> u8 {
    match facing {
        "south" => 0, "west" => 1, "north" => 2, "east" => 3, _ => 0,
    }
}

fn map_vine_faces(properties: &HashMap<String, String>) -> u8 {
    let mut data = 0u8;
    if get_bool_prop(properties, "south") { data |= 1; }
    if get_bool_prop(properties, "west") { data |= 2; }
    if get_bool_prop(properties, "north") { data |= 4; }
    if get_bool_prop(properties, "east") { data |= 8; }
    data
}

fn map_lever_data(face: &str, facing: &str) -> u8 {
    match face {
        "floor" => if facing == "east" || facing == "west" { 6 } else { 5 },
        "ceiling" => if facing == "east" || facing == "west" { 7 } else { 0 },
        _ => map_button_facing(facing),
    }
}

fn map_door_facing(facing: &str) -> u8 {
    match facing {
        "east" => 0, "south" => 1, "west" => 2, "north" => 3, _ => 0,
    }
}

fn map_trapdoor_facing(facing: &str) -> u8 {
    match facing {
        "north" => 0, "south" => 1, "west" => 2, "east" => 3, _ => 0,
    }
}

fn map_facing_data(facing: &str) -> u8 {
    match facing {
        "down" => 0, "up" => 1, "north" => 2, "south" => 3, "west" => 4, "east" => 5, _ => 3,
    }
}

fn map_repeater_direction(facing: &str) -> u8 {
    match facing {
        "south" => 0, "west" => 1, "north" => 2, "east" => 3, _ => 0,
    }
}

fn map_wall_torch_facing(facing: &str) -> u8 {
    match facing {
        "east" => 1, "west" => 2, "south" => 3, "north" => 4, _ => 1,
    }
}

fn try_map_colored_block(name: &str, properties: &HashMap<String, String>) -> Option<LegacyBlockState> {
    let color = get_prop(properties, "color");
    let color_data = block_mapping::get_color_data(color);

    match name {
        "wool" => Some(LegacyBlockState::new(35, color_data)),
        "stained_glass" => Some(LegacyBlockState::new(95, color_data)),
        "stained_glass_pane" => Some(LegacyBlockState::new(160, color_data)),
        "terracotta" | "stained_hardened_clay" => Some(LegacyBlockState::new(159, color_data)),
        "concrete" => Some(LegacyBlockState::new(172, color_data)),
        "concrete_powder" => Some(LegacyBlockState::new(12, color_data)),
        "glazed_terracotta" => Some(LegacyBlockState::new(159, color_data)),
        _ => None,
    }
}

fn try_map_flattened_colored_block(name: &str) -> Option<LegacyBlockState> {
    let (color_data, suffix) = split_color_prefix(name)?;
    match suffix {
        "wool" => Some(LegacyBlockState::new(35, color_data)),
        "stained_glass" | "glass" => Some(LegacyBlockState::new(95, color_data)),
        "stained_glass_pane" | "glass_pane" => Some(LegacyBlockState::new(160, color_data)),
        "terracotta" | "stained_hardened_clay" => Some(LegacyBlockState::new(159, color_data)),
        "concrete" => Some(LegacyBlockState::new(172, color_data)),
        "concrete_powder" => Some(LegacyBlockState::new(12, color_data)),
        "glazed_terracotta" => Some(LegacyBlockState::new(159, color_data)),
        "carpet" => Some(LegacyBlockState::new(171, color_data)),
        _ => None,
    }
}

fn split_color_prefix(name: &str) -> Option<(u8, &str)> {
    for color_name in block_mapping::COLOR_NAMES {
        if let Some(suffix) = name.strip_prefix(color_name).and_then(|s| s.strip_prefix('_')) {
            let color_data = block_mapping::get_color_name(color_name);
            return Some((color_data, suffix));
        }
    }
    None
}

fn try_map_wood_block(name: &str, properties: &HashMap<String, String>) -> Option<LegacyBlockState> {
    let wood_type = get_prop(properties, "variant");
    let wood_type = if wood_type.is_empty() {
        get_prefix_before_underscore(name)
    } else {
        wood_type.to_string()
    };
    let data = block_mapping::get_wood_data(&wood_type);

    if name.ends_with("_planks") || name == "planks" {
        return Some(LegacyBlockState::new(5, data));
    }
    if name.ends_with("_sapling") || name == "sapling" {
        return Some(LegacyBlockState::new(6, data));
    }
    if name.ends_with("_log") || name == "log" {
        return Some(LegacyBlockState::new(17, if data > 3 { 3 } else { data }));
    }
    if name.ends_with("_leaves") || name == "leaves" {
        return Some(LegacyBlockState::new(18, if data > 3 { 3 } else { data }));
    }
    if name.ends_with("_stairs") && name.contains("wood") {
        return Some(LegacyBlockState::new(53, 0));
    }
    if name.ends_with("_door") {
        return Some(LegacyBlockState::new(64, 0));
    }
    if name.ends_with("_fence") {
        return Some(LegacyBlockState::new(85, 0));
    }
    if name.ends_with("_fence_gate") {
        return Some(LegacyBlockState::new(107, 0));
    }
    if name.ends_with("_pressure_plate") && is_wood_family(name) {
        return Some(LegacyBlockState::new(72, 0));
    }
    None
}

fn get_prefix_before_underscore(name: &str) -> String {
    if let Some(idx) = name.find('_') {
        name[..idx].to_string()
    } else {
        String::new()
    }
}

fn try_map_variant_block(name: &str, properties: &HashMap<String, String>) -> Option<LegacyBlockState> {
    match name {
        "redstone_lamp" => {
            let lit = get_bool_prop(properties, "lit");
            Some(LegacyBlockState::new(if lit { 124 } else { 123 }, 0))
        }
        "deepslate" | "polished_deepslate" | "tuff" | "calcite" | "dripstone_block" => Some(LegacyBlockState::new(1, 0)),
        "cobbled_deepslate" => Some(LegacyBlockState::new(4, 0)),
        "deepslate_bricks" | "deepslate_tiles" => Some(LegacyBlockState::new(98, 0)),
        "cracked_deepslate_bricks" | "cracked_deepslate_tiles" => Some(LegacyBlockState::new(98, 2)),
        "chiseled_deepslate" => Some(LegacyBlockState::new(98, 3)),
        "deepslate_coal_ore" => Some(LegacyBlockState::new(16, 0)),
        "deepslate_iron_ore" | "deepslate_copper_ore" => Some(LegacyBlockState::new(15, 0)),
        "deepslate_gold_ore" => Some(LegacyBlockState::new(14, 0)),
        "deepslate_redstone_ore" => Some(LegacyBlockState::new(73, 0)),
        "deepslate_lapis_ore" => Some(LegacyBlockState::new(21, 0)),
        "deepslate_diamond_ore" => Some(LegacyBlockState::new(56, 0)),
        "deepslate_emerald_ore" => Some(LegacyBlockState::new(129, 0)),
        "cobblestone_wall" => Some(LegacyBlockState::new(139, 0)),
        "mossy_cobblestone_wall" => Some(LegacyBlockState::new(139, 1)),
        "mossy_stone_bricks" => Some(LegacyBlockState::new(98, 1)),
        "cracked_stone_bricks" => Some(LegacyBlockState::new(98, 2)),
        "chiseled_stone_bricks" => Some(LegacyBlockState::new(98, 3)),
        "smooth_sandstone" => Some(LegacyBlockState::new(24, 2)),
        "chiseled_sandstone" | "cut_sandstone" => Some(LegacyBlockState::new(24, 1)),
        "quartz_block" => Some(LegacyBlockState::new(155, 0)),
        "chiseled_quartz_block" => Some(LegacyBlockState::new(155, 1)),
        "prismarine" => Some(LegacyBlockState::new(168, 0)),
        "prismarine_bricks" => Some(LegacyBlockState::new(168, 1)),
        "dark_prismarine" => Some(LegacyBlockState::new(168, 2)),
        "poppy" | "sunflower" | "lilac" | "peony" | "rose_bush" => Some(LegacyBlockState::new(38, 0)),
        "blue_orchid" => Some(LegacyBlockState::new(38, 1)),
        "allium" => Some(LegacyBlockState::new(38, 2)),
        "azure_bluet" => Some(LegacyBlockState::new(38, 3)),
        "red_tulip" => Some(LegacyBlockState::new(38, 4)),
        "orange_tulip" => Some(LegacyBlockState::new(38, 5)),
        "white_tulip" => Some(LegacyBlockState::new(38, 6)),
        "pink_tulip" => Some(LegacyBlockState::new(38, 7)),
        "oxeye_daisy" => Some(LegacyBlockState::new(38, 8)),
        _ => None,
    }
}

pub fn get_nibble(arr: &[u8], index: usize) -> u8 {
    let b = arr[index >> 1];
    if (index & 1) == 0 { b & 0x0F } else { (b >> 4) & 0x0F }
}

pub fn set_nibble(arr: &mut [u8], index: usize, value: u8) {
    let i = index >> 1;
    let value = value & 0x0F;
    if (index & 1) == 0 {
        arr[i] = (arr[i] & 0xF0) | value;
    } else {
        arr[i] = (arr[i] & 0x0F) | (value << 4);
    }
}

pub fn flatten_anvil_sections(
    level: &HashMap<String, NbtTag>,
    section_shift: Option<i32>,
    context: &mut ChunkConversionContext,
) -> ([u8; CHUNK_BLOCKS], [u8; CHUNK_NIBBLES], [u8; CHUNK_NIBBLES], [u8; CHUNK_NIBBLES]) {
    let mut blocks = [0u8; CHUNK_BLOCKS];
    let mut data = [0u8; CHUNK_NIBBLES];
    let mut sky_light = [0xFFu8; CHUNK_NIBBLES];
    let mut block_light = [0u8; CHUNK_NIBBLES];
    let sections = NbtHelper::get_list(level, "Sections")
        .or_else(|| NbtHelper::get_list(level, "sections"));

    let sections = match sections {
        Some(s) => s,
        None => return (blocks, data, sky_light, block_light),
    };

    struct DecodedSectionData {
        section_y: i32,
        sblocks: [u8; 4096],
        sdata: [u8; 2048],
        sky: Option<[u8; 2048]>,
        block: Option<[u8; 2048]>,
        non_air: i32,
    }

    let mut decoded = Vec::new();
    for section_tag in &sections {
        let section = match section_tag.as_compound() {
            Some(s) => s,
            None => continue,
        };

        let section_y = NbtHelper::get_byte(section, "Y")
            .or_else(|| NbtHelper::get_int(section, "y").map(|v| v as i8))
            .unwrap_or(0) as i32;

        let (sblocks, sdata) = match try_decode_section_blocks(section, context) {
            Some(r) => r,
            None => continue,
        };

        let s_sky = NbtHelper::get_byte_array(section, "SkyLight")
            .or_else(|| NbtHelper::get_byte_array(section, "sky_light"))
            .map(|v| {
                let mut arr = [0u8; 2048];
                let len = v.len().min(2048);
                arr[..len].copy_from_slice(&v[..len]);
                arr
            });

        let s_block = NbtHelper::get_byte_array(section, "BlockLight")
            .or_else(|| NbtHelper::get_byte_array(section, "block_light"))
            .map(|v| {
                let mut arr = [0u8; 2048];
                let len = v.len().min(2048);
                arr[..len].copy_from_slice(&v[..len]);
                arr
            });

        let non_air = sblocks.iter().filter(|&&b| b != 0).count() as i32;
        decoded.push(DecodedSectionData {
            section_y,
            sblocks,
            sdata,
            sky: s_sky,
            block: s_block,
            non_air,
        });
    }

    if decoded.is_empty() {
        return (blocks, data, sky_light, block_light);
    }

    let mut effective_shift = section_shift.unwrap_or(0);
    let uses_negative_y = decoded.iter().any(|s| s.section_y < 0);
    if effective_shift == 0 && section_shift.is_none() {
        let anchor = decoded.iter()
            .max_by(|a, b| a.non_air.cmp(&b.non_air)
                .then_with(|| (a.section_y - 4).abs().cmp(&(b.section_y - 4).abs())))
            .map(|s| s.section_y)
            .unwrap_or(4);
        effective_shift = if uses_negative_y { anchor + 4 } else { anchor - 4 };
    }

    for section in &decoded {
        let remapped_y = section.section_y - effective_shift;
        if remapped_y < 0 || remapped_y > 15 {
            continue;
        }
        let base_y = remapped_y as usize * 16;
        for i in 0..4096 {
            let x = i & 0x0F;
            let z = (i >> 4) & 0x0F;
            let y = (i >> 8) & 0x0F;
            let global_y = base_y + y;
            let flat_index = ((x * 16) + z) * 256 + global_y;
            blocks[flat_index] = section.sblocks[i];
            set_nibble(&mut data, flat_index, get_nibble(&section.sdata, i));
            if let Some(ref sk) = section.sky {
                set_nibble(&mut sky_light, flat_index, get_nibble(sk, i));
            }
            if let Some(ref bl) = section.block {
                set_nibble(&mut block_light, flat_index, get_nibble(bl, i));
            }
        }
    }

    (blocks, data, sky_light, block_light)
}

fn try_decode_section_blocks(section: &HashMap<String, NbtTag>, context: &mut ChunkConversionContext) -> Option<([u8; 4096], [u8; 2048])> {
    let old_blocks = NbtHelper::get_byte_array(section, "Blocks");
    if let Some(ref old_b) = old_blocks {
        if old_b.len() >= 4096 {
            let mut blocks = [0u8; 4096];
            blocks.copy_from_slice(&old_b[..4096]);
            let data_arr = NbtHelper::get_byte_array_or_default(section, "Data", 2048);
            let mut data = [0u8; 2048];
            let len = data_arr.len().min(2048);
            data[..len].copy_from_slice(&data_arr[..len]);
            return Some((blocks, data));
        }
    }

    try_decode_palette_section(section, context)
}

fn try_decode_palette_section(section: &HashMap<String, NbtTag>, context: &mut ChunkConversionContext) -> Option<([u8; 4096], [u8; 2048])> {
    let block_states_container = NbtHelper::get_compound(section, "block_states");
    let palette = NbtHelper::get_list(section, "Palette")
        .or_else(|| block_states_container.as_ref().and_then(|c| NbtHelper::get_list(c, "palette")))?;

    if palette.is_empty() {
        return None;
    }

    let mut blocks = [0u8; 4096];
    let mut data = [0u8; 2048];
    if palette.len() == 1 {
        let entry = match &palette[0] {
            NbtTag::Compound(m) => m,
            _ => return None,
        };
        let legacy = map_palette_entry(entry, context);
        for b in blocks.iter_mut() {
            *b = legacy.id;
        }
        if legacy.data != 0 {
            for i in 0..4096 {
                set_nibble(&mut data, i, legacy.data);
            }
        }
        return Some((blocks, data));
    }

    let block_states = section.get("BlockStates")
        .and_then(|t| t.get_long_array())
        .or_else(|| block_states_container.as_ref().and_then(|c| c.get("data").and_then(|t| t.get_long_array())))?;

    if block_states.is_empty() {
        return None;
    }

    let bits_per_block = std::cmp::max(4, bits_required(palette.len() - 1));
    let values_per_long = std::cmp::max(1, 64 / bits_per_block);
    let expected_long_count = (4096 + values_per_long - 1) / values_per_long;
    let use_padded = block_states.len() == expected_long_count as usize;
    for i in 0..4096 {
        let palette_index = if use_padded {
            read_padded_block_state(block_states, bits_per_block, i)
        } else {
            read_compact_block_state(block_states, bits_per_block, i)
        };

        if palette_index >= palette.len() {
            continue;
        }

        let entry = match &palette[palette_index] {
            NbtTag::Compound(m) => m,
            _ => continue,
        };

        let legacy = map_palette_entry(entry, context);
        blocks[i] = legacy.id;
        if legacy.data != 0 {
            set_nibble(&mut data, i, legacy.data);
        }
    }

    Some((blocks, data))
}

fn map_palette_entry(entry: &HashMap<String, NbtTag>, context: &mut ChunkConversionContext) -> LegacyBlockState {
    let name = NbtHelper::get_string(entry, "Name").unwrap_or_default();
    let properties = NbtHelper::get_compound(entry, "Properties").unwrap_or_default();
    let mut prop_map = HashMap::new();
    for (k, v) in &properties {
        if let Some(s) = v.get_string() {
            prop_map.insert(k.clone(), s.to_string());
        }
    }
    map_modern_block_state(&name, &prop_map, context)
}

fn bits_required(value: usize) -> i32 {
    if value == 0 { return 1; }
    let mut bits = 0;
    let mut v = value;
    while v > 0 {
        bits += 1;
        v >>= 1;
    }
    bits
}

fn read_padded_block_state(states: &[i64], bits_per_block: i32, index: usize) -> usize {
    let values_per_long = std::cmp::max(1, 64 / bits_per_block);
    let long_index = index / values_per_long as usize;
    let bit_offset = (index % values_per_long as usize) * bits_per_block as usize;
    let mask = (1u64 << bits_per_block) - 1;
    (states[long_index] as u64 >> bit_offset & mask) as usize
}

fn read_compact_block_state(states: &[i64], bits_per_block: i32, index: usize) -> usize {
    let start_bit = index * bits_per_block as usize;
    let long_index = start_bit >> 6;
    let bit_offset = start_bit & 63;
    let mask = (1u64 << bits_per_block) - 1;
    let mut value = states[long_index] as u64 >> bit_offset;
    let bits_read = 64 - bit_offset;
    if (bits_read as usize) < bits_per_block as usize && long_index + 1 < states.len() {
        value |= (states[long_index + 1] as u64) << bits_read;
    }
    (value & mask) as usize
}

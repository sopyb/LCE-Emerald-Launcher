use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::console2lce::models::LegacyBlockState;
pub static MODERN_DIRECT_MAP: Lazy<HashMap<&'static str, LegacyBlockState>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("stone", LegacyBlockState::new(1, 0));
    m.insert("granite", LegacyBlockState::new(1, 1));
    m.insert("polished_granite", LegacyBlockState::new(1, 2));
    m.insert("diorite", LegacyBlockState::new(1, 3));
    m.insert("polished_diorite", LegacyBlockState::new(1, 4));
    m.insert("andesite", LegacyBlockState::new(1, 5));
    m.insert("polished_andesite", LegacyBlockState::new(1, 6));
    m.insert("grass_block", LegacyBlockState::new(2, 0));
    m.insert("dirt", LegacyBlockState::new(3, 0));
    m.insert("coarse_dirt", LegacyBlockState::new(3, 1));
    m.insert("podzol", LegacyBlockState::new(3, 2));
    m.insert("cobblestone", LegacyBlockState::new(4, 0));
    m.insert("bedrock", LegacyBlockState::new(7, 0));
    m.insert("sand", LegacyBlockState::new(12, 0));
    m.insert("red_sand", LegacyBlockState::new(12, 1));
    m.insert("gravel", LegacyBlockState::new(13, 0));
    m.insert("gold_ore", LegacyBlockState::new(14, 0));
    m.insert("iron_ore", LegacyBlockState::new(15, 0));
    m.insert("coal_ore", LegacyBlockState::new(16, 0));
    m.insert("sponge", LegacyBlockState::new(19, 0));
    m.insert("wet_sponge", LegacyBlockState::new(19, 1));
    m.insert("glass", LegacyBlockState::new(20, 0));
    m.insert("lapis_ore", LegacyBlockState::new(21, 0));
    m.insert("lapis_block", LegacyBlockState::new(22, 0));
    m.insert("sandstone", LegacyBlockState::new(24, 0));
    m.insert("chiseled_sandstone", LegacyBlockState::new(24, 1));
    m.insert("smooth_sandstone", LegacyBlockState::new(24, 2));
    m.insert("note_block", LegacyBlockState::new(25, 0));
    m.insert("cobweb", LegacyBlockState::new(30, 0));
    m.insert("short_grass", LegacyBlockState::new(31, 0));
    m.insert("fern", LegacyBlockState::new(31, 1));
    m.insert("dead_bush", LegacyBlockState::new(32, 0));
    m.insert("dandelion", LegacyBlockState::new(37, 0));
    m.insert("brown_mushroom", LegacyBlockState::new(39, 0));
    m.insert("red_mushroom", LegacyBlockState::new(40, 0));
    m.insert("gold_block", LegacyBlockState::new(41, 0));
    m.insert("iron_block", LegacyBlockState::new(42, 0));
    m.insert("bricks", LegacyBlockState::new(45, 0));
    m.insert("tnt", LegacyBlockState::new(46, 0));
    m.insert("bookshelf", LegacyBlockState::new(47, 0));
    m.insert("mossy_cobblestone", LegacyBlockState::new(48, 0));
    m.insert("obsidian", LegacyBlockState::new(49, 0));
    m.insert("torch", LegacyBlockState::new(50, 0));
    m.insert("fire", LegacyBlockState::new(51, 0));
    m.insert("mob_spawner", LegacyBlockState::new(52, 0));
    m.insert("chest", LegacyBlockState::new(54, 0));
    m.insert("trapped_chest", LegacyBlockState::new(146, 0));
    m.insert("diamond_ore", LegacyBlockState::new(56, 0));
    m.insert("diamond_block", LegacyBlockState::new(57, 0));
    m.insert("crafting_table", LegacyBlockState::new(58, 0));
    m.insert("wheat", LegacyBlockState::new(59, 0));
    m.insert("farmland", LegacyBlockState::new(60, 0));
    m.insert("furnace", LegacyBlockState::new(61, 0));
    m.insert("ladder", LegacyBlockState::new(65, 0));
    m.insert("rail", LegacyBlockState::new(66, 0));
    m.insert("snow", LegacyBlockState::new(78, 0));
    m.insert("ice", LegacyBlockState::new(79, 0));
    m.insert("snow_block", LegacyBlockState::new(80, 0));
    m.insert("cactus", LegacyBlockState::new(81, 0));
    m.insert("clay", LegacyBlockState::new(82, 0));
    m.insert("sugar_cane", LegacyBlockState::new(83, 0));
    m.insert("jukebox", LegacyBlockState::new(84, 0));
    m.insert("pumpkin", LegacyBlockState::new(86, 0));
    m.insert("netherrack", LegacyBlockState::new(87, 0));
    m.insert("soul_sand", LegacyBlockState::new(88, 0));
    m.insert("glowstone", LegacyBlockState::new(89, 0));
    m.insert("jack_o_lantern", LegacyBlockState::new(91, 0));
    m.insert("cake", LegacyBlockState::new(92, 0));
    m.insert("melon", LegacyBlockState::new(103, 0));
    m.insert("vine", LegacyBlockState::new(106, 0));
    m.insert("mycelium", LegacyBlockState::new(110, 0));
    m.insert("lily_pad", LegacyBlockState::new(111, 0));
    m.insert("nether_bricks", LegacyBlockState::new(112, 0));
    m.insert("nether_brick_fence", LegacyBlockState::new(113, 0));
    m.insert("enchanting_table", LegacyBlockState::new(116, 0));
    m.insert("brewing_stand", LegacyBlockState::new(117, 0));
    m.insert("cauldron", LegacyBlockState::new(118, 0));
    m.insert("end_portal", LegacyBlockState::new(119, 0));
    m.insert("end_portal_frame", LegacyBlockState::new(120, 0));
    m.insert("end_stone", LegacyBlockState::new(121, 0));
    m.insert("dragon_egg", LegacyBlockState::new(122, 0));
    m.insert("cocoa", LegacyBlockState::new(127, 0));
    m.insert("emerald_ore", LegacyBlockState::new(129, 0));
    m.insert("ender_chest", LegacyBlockState::new(130, 0));
    m.insert("tripwire_hook", LegacyBlockState::new(131, 0));
    m.insert("tripwire", LegacyBlockState::new(132, 0));
    m.insert("emerald_block", LegacyBlockState::new(133, 0));
    m.insert("command_block", LegacyBlockState::new(137, 0));
    m.insert("beacon", LegacyBlockState::new(138, 0));
    m.insert("flower_pot", LegacyBlockState::new(140, 0));
    m.insert("carrots", LegacyBlockState::new(141, 0));
    m.insert("potatoes", LegacyBlockState::new(142, 0));
    m.insert("anvil", LegacyBlockState::new(145, 0));
    m.insert("daylight_detector", LegacyBlockState::new(151, 0));
    m.insert("redstone_block", LegacyBlockState::new(152, 0));
    m.insert("nether_quartz_ore", LegacyBlockState::new(153, 0));
    m.insert("hopper", LegacyBlockState::new(154, 0));
    m.insert("slime_block", LegacyBlockState::new(165, 0));
    m.insert("iron_trapdoor", LegacyBlockState::new(167, 0));
    m.insert("sea_lantern", LegacyBlockState::new(169, 0));
    m.insert("hay_block", LegacyBlockState::new(170, 0));
    m.insert("dispenser", LegacyBlockState::new(23, 0));
    m.insert("dropper", LegacyBlockState::new(158, 0));
    m.insert("sticky_piston", LegacyBlockState::new(29, 0));
    m.insert("piston", LegacyBlockState::new(33, 0));
    m.insert("piston_head", LegacyBlockState::new(34, 0));
    m.insert("redstone_lamp", LegacyBlockState::new(123, 0));
    m.insert("red_nether_bricks", LegacyBlockState::new(112, 0));
    m.insert("bone_block", LegacyBlockState::new(1, 0));
    m.insert("observer", LegacyBlockState::new(1, 0));
    m.insert("shulker_box", LegacyBlockState::new(35, 0));
    m.insert("white_shulker_box", LegacyBlockState::new(35, 0));
    m.insert("orange_shulker_box", LegacyBlockState::new(35, 1));
    m.insert("magenta_shulker_box", LegacyBlockState::new(35, 2));
    m.insert("light_blue_shulker_box", LegacyBlockState::new(35, 3));
    m.insert("yellow_shulker_box", LegacyBlockState::new(35, 4));
    m.insert("lime_shulker_box", LegacyBlockState::new(35, 5));
    m.insert("pink_shulker_box", LegacyBlockState::new(35, 6));
    m.insert("gray_shulker_box", LegacyBlockState::new(35, 7));
    m.insert("light_gray_shulker_box", LegacyBlockState::new(35, 8));
    m.insert("cyan_shulker_box", LegacyBlockState::new(35, 9));
    m.insert("purple_shulker_box", LegacyBlockState::new(35, 10));
    m.insert("blue_shulker_box", LegacyBlockState::new(35, 11));
    m.insert("brown_shulker_box", LegacyBlockState::new(35, 12));
    m.insert("green_shulker_box", LegacyBlockState::new(35, 13));
    m.insert("red_shulker_box", LegacyBlockState::new(35, 14));
    m.insert("black_shulker_box", LegacyBlockState::new(35, 15));
    m
});

pub static COLOR_NAMES: &[&str] = &[
    "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
    "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black",
];

pub fn get_color_name(color: &str) -> u8 {
    match color {
        "white" => 0,
        "orange" => 1,
        "magenta" => 2,
        "light_blue" => 3,
        "yellow" => 4,
        "lime" => 5,
        "pink" => 6,
        "gray" => 7,
        "light_gray" | "silver" => 8,
        "cyan" => 9,
        "purple" => 10,
        "blue" => 11,
        "brown" => 12,
        "green" => 13,
        "red" => 14,
        "black" => 15,
        _ => 0,
    }
}

pub fn get_color_data(color: &str) -> u8 {
    get_color_name(color)
}

pub fn get_wood_data(wood_type: &str) -> u8 {
    match wood_type {
        "spruce" => 1,
        "birch" => 2,
        "jungle" => 3,
        "acacia" => 4,
        "dark_oak" => 5,
        _ => 0,
    }
}

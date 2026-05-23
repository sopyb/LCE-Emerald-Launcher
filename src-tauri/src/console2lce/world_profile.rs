#[derive(Debug, Clone, Copy)]
pub struct WorldProfile {
    pub name: &'static str,
    pub xz_size: u32,
    pub hell_scale: u32,
    pub flat: bool,
}

pub const PROFILES: &[WorldProfile] = &[
    WorldProfile { name: "classic", xz_size: 54, hell_scale: 54, flat: false },
    WorldProfile { name: "small", xz_size: 64, hell_scale: 64, flat: false },
    WorldProfile { name: "medium", xz_size: 192, hell_scale: 192, flat: false },
    WorldProfile { name: "large", xz_size: 320, hell_scale: 320, flat: false },
    WorldProfile { name: "flat", xz_size: 54, hell_scale: 54, flat: true },
    WorldProfile { name: "flat-small", xz_size: 64, hell_scale: 64, flat: true },
    WorldProfile { name: "flat-medium", xz_size: 192, hell_scale: 192, flat: true },
    WorldProfile { name: "flat-large", xz_size: 320, hell_scale: 320, flat: true },
];

pub fn get_profile(name: &str) -> Option<&'static WorldProfile> {
    PROFILES.iter().find(|p| p.name == name)
}

pub fn world_center_offset(xz_size: u32) -> i32 {
    (xz_size as i32) * 8
}

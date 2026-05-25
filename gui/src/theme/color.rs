use iced::Color;

// TODO: replace this with Color::from_rgba8
// pass hex string
const fn hex(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

// Surface layers
pub const BG_HARD: Color = hex(0x0d, 0x0f, 0x0e);
pub const BG: Color = hex(0x12, 0x14, 0x13);
pub const BG_LOW: Color = hex(0x1a, 0x1c, 0x1b);
pub const BG_CONT: Color = hex(0x1e, 0x20, 0x1f);
pub const BG_HIGH: Color = hex(0x29, 0x2a, 0x29);
pub const BG_HIGHEST: Color = hex(0x33, 0x35, 0x34);

// Stale-record row tint (dark warm red)
pub const BG_STALE: Color = hex(0x3c, 0x1f, 0x1e);

// Borders
pub const BORDER: Color = hex(0x41, 0x48, 0x45);

// Text
pub const FG: Color = hex(0xe3, 0xe2, 0xe0);
pub const FG2: Color = hex(0xc1, 0xc8, 0xc4);
pub const FG4: Color = hex(0x8b, 0x92, 0x8e);

// Accents
pub const PRIMARY: Color = hex(0xab, 0xce, 0xc0);
pub const PRIMARY_C: Color = hex(0x83, 0xa5, 0x98);
pub const GREEN: Color = hex(0xa1, 0xd4, 0x8e);
pub const YELLOW: Color = hex(0xd7, 0x99, 0x21);
pub const RED: Color = hex(0xcc, 0x24, 0x1d);
pub const RED_BRIGHT: Color = hex(0xfb, 0x49, 0x34);
pub const RED_CONT: Color = hex(0x93, 0x00, 0x0a);
pub const PURPLE: Color = hex(0xc2, 0x92, 0x8d);
pub const ORANGE: Color = hex(0xf4, 0x7b, 0x20);

// Runtime badge palette
pub const UNITY_BADGE_BG: Color = hex(0x24, 0x50, 0x1a);
pub const UNITY_BADGE_FG: Color = hex(0xbc, 0xf1, 0xa8);
pub const MONO_BADGE_BG: Color = hex(0x16, 0x36, 0x2c);
pub const RUNTIME_BADGE_BORDER: Color = hex(0x2d, 0x4d, 0x42);

// Log strip
pub const LOG_TIME: Color = hex(0xa8, 0x99, 0x84);
pub const LOG_INFO: Color = hex(0x83, 0xa5, 0x98);
pub const LOG_OK: Color = hex(0xb8, 0xbb, 0x26);
pub const LOG_WARN: Color = hex(0xfb, 0x49, 0x34);

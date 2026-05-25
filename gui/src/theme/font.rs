use iced::{
    Font,
    font::{Family, Stretch, Style, Weight},
};

pub const FONT_UI: Font = Font::with_name("Hanken Grotesk");
pub const FONT_MONO: Font = Font::with_name("JetBrains Mono");
pub const FONT_ICON: Font = Font::with_name("Material Symbols Outlined");

pub const FONT_UI_SEMIBOLD: Font = Font {
    family: Family::Name("Hanken Grotesk"),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_MONO_MEDIUM: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Medium,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

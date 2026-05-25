mod button;
mod color;
mod container;
mod font;
mod scrollable;
mod text;

pub use button::*;
pub use color::*;
pub use container::*;
pub use font::*;
pub use scrollable::*;
pub use text::*;

// Spacing (pixels)
pub const SP1: f32 = 4.0;
pub const SP2: f32 = 8.0;
pub const SP3: f32 = 12.0;
pub const SP4: f32 = 16.0;
pub const SP5: f32 = 24.0;

pub fn app_theme() -> iced::Theme {
    iced::Theme::custom(
        "mono-injector".to_string(),
        iced::theme::Palette {
            background: BG,
            text: FG,
            primary: PRIMARY,
            success: GREEN,
            danger: RED,
            warning: YELLOW,
        },
    )
}

use iced::widget::container;
use iced::{Background, Border, Color};

use super::{BG_CONT, BG_HARD, BG_HIGH, BG_HIGHEST, BG_LOW, BORDER, FG, FG2, RED_CONT};

pub fn panel_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_LOW)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(FG),
        ..Default::default()
    }
}

pub fn elevated_panel_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_CONT)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(FG),
        ..Default::default()
    }
}

pub fn panel_header_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_HIGH)),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        text_color: Some(FG2),
        ..Default::default()
    }
}

pub fn recessed_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_HARD)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn danger_section_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(RED_CONT.scale_alpha(0.1))),
        border: Border {
            color: RED_CONT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Bottom panel footer: flat (radius-0) border, used in the profile detail panel.
pub fn footer_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_LOW)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Underline-only border for section title rows.
pub fn section_title_style() -> container::Style {
    container::Style {
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Chip-style container for key/value runtime option pairs.
pub fn runtime_chip_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_HIGHEST)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

/// Circular indicator dot — the physical size is set via the containing `Space` widget.
pub fn dot_style(color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color,
            width: 0.0,
            radius: 999.0.into(),
        },
        ..Default::default()
    }
}

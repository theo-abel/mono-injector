use iced::widget::{container, scrollable};
use iced::{Background, Border, Color};

use super::{BG, BG_HARD, BG_HIGH, BORDER, FG4};

fn scrollable_rail(bg: Color) -> scrollable::Rail {
    scrollable::Rail {
        background: Some(Background::Color(bg)),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: 2.0.into(),
        },
        scroller: scrollable::Scroller {
            color: FG4,
            border: Border {
                color: BORDER,
                width: 0.0,
                radius: 2.0.into(),
            },
        },
    }
}

fn scrollable_style_with_bg(bg: Color) -> scrollable::Style {
    scrollable::Style {
        container: container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        },
        vertical_rail: scrollable_rail(BG_HIGH),
        horizontal_rail: scrollable_rail(Color::TRANSPARENT),
        gap: None,
    }
}

pub fn log_scrollable_style(
    _theme: &iced::Theme,
    _status: scrollable::Status,
) -> scrollable::Style {
    scrollable_style_with_bg(BG_HARD)
}

pub fn table_scrollable_style(
    _theme: &iced::Theme,
    _status: scrollable::Status,
) -> scrollable::Style {
    scrollable_style_with_bg(BG)
}

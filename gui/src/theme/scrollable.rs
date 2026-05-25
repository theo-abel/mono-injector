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
            background: Background::Color(FG4),
            border: Border {
                color: BORDER,
                width: 0.0,
                radius: 2.0.into(),
            },
        },
    }
}

fn scrollable_style_with_bg(
    theme: &iced::Theme,
    status: scrollable::Status,
    bg: Color,
) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    style.container = container::Style {
        background: Some(Background::Color(bg)),
        ..Default::default()
    };
    style.vertical_rail = scrollable_rail(BG_HIGH);
    style.horizontal_rail = scrollable_rail(Color::TRANSPARENT);
    style.gap = None;
    style
}

pub fn log_scrollable_style(theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    scrollable_style_with_bg(theme, status, BG_HARD)
}

pub fn table_scrollable_style(
    theme: &iced::Theme,
    status: scrollable::Status,
) -> scrollable::Style {
    scrollable_style_with_bg(theme, status, BG)
}

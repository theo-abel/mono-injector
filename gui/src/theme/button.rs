use iced::widget::button;
use iced::{Background, Border, Color};

use super::{
    BG_HARD, BG_HIGH, BG_HIGHEST, BORDER, FG, FG2, GREEN, LOG_OK, PRIMARY, PRIMARY_C, RED,
    RED_BRIGHT, RED_CONT, YELLOW,
};

pub fn ghost_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_HIGHEST,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: FG2,
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Button that sits flush next to a text input; uses the same `BG_HARD` fill
/// as inputs so the two elements look like a unified group.
pub fn input_adjacent_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_HIGHEST,
        _ => BG_HARD,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: FG2,
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn primary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => PRIMARY,
        _ => PRIMARY_C,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: BG_HARD,
        border: Border {
            color: PRIMARY_C,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn inject_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => GREEN,
        _ => LOG_OK,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: BG_HARD,
        border: Border {
            color: LOG_OK,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn danger_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => RED_BRIGHT,
        button::Status::Disabled => RED_CONT,
        button::Status::Active => RED,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: FG,
        border: Border {
            color: RED_CONT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn danger_outline_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let (bg, text) = match status {
        button::Status::Hovered | button::Status::Pressed => (RED, FG),
        _ => (Color::TRANSPARENT, RED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: text,
        border: Border {
            color: RED,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Rows in the inline process picker inside the Inject view.
pub fn process_list_row_button_style(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_HIGH,
        _ => BG_HARD,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: FG,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Yellow-outline button for dry-run / non-destructive preview actions.
pub fn dry_run_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_HIGH,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: YELLOW,
        border: Border {
            color: YELLOW,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Collapsible header button for the danger options section.
pub fn danger_header_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(RED_CONT.scale_alpha(0.12))),
        text_color: RED,
        border: Border {
            color: RED_CONT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Row button for the process browser table.
///
/// `bg` is the base (odd/even/selected) background; the row highlights to
/// `BG_HIGH` on hover. `selected` controls the primary-colour border.
pub fn table_row_button_style(
    bg: iced::Color,
    selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let active_bg = match status {
            button::Status::Hovered | button::Status::Pressed => BG_HIGH,
            _ => bg,
        };
        button::Style {
            background: Some(Background::Color(active_bg)),
            text_color: FG,
            border: Border {
                color: if selected {
                    PRIMARY_C
                } else {
                    Color::TRANSPARENT
                },
                width: if selected { 1.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Solid primary-colour button for "Send to Inject" actions.
pub fn send_to_inject_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(PRIMARY)),
        text_color: BG_HARD,
        border: Border {
            color: PRIMARY_C,
            width: 0.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

/// Profile sidebar list-item button. Selected items get a solid background and
/// a primary-colour border.
pub fn profile_list_item_button_style(
    selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = if selected {
            BG_HIGH
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => BG_HIGHEST,
                _ => Color::TRANSPARENT,
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if selected { PRIMARY } else { FG },
            border: Border {
                color: if selected {
                    PRIMARY_C
                } else {
                    Color::TRANSPARENT
                },
                width: if selected { 1.0 } else { 0.0 },
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

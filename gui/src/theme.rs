use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Font, font};

// TODO: replace this with Color::parse("#<hex_color>")
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

// Spacing (pixels)
pub const SP1: f32 = 4.0;
pub const SP2: f32 = 8.0;
pub const SP3: f32 = 12.0;
pub const SP4: f32 = 16.0;
pub const SP5: f32 = 24.0;

// Fonts
pub const FONT_UI: Font = Font::with_name("Hanken Grotesk");
pub const FONT_MONO: Font = Font::with_name("JetBrains Mono");
pub const FONT_ICON: Font = Font::with_name("Material Symbols Outlined");

pub const FONT_UI_SEMIBOLD: Font = Font {
    family: font::Family::Name("Hanken Grotesk"),
    weight: font::Weight::Semibold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

pub const FONT_MONO_MEDIUM: Font = Font {
    family: font::Family::Name("JetBrains Mono"),
    weight: font::Weight::Medium,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

pub fn app_theme() -> iced::Theme {
    iced::Theme::custom(
        "mono-injector".to_string(),
        iced::theme::Palette {
            background: BG,
            text: FG,
            primary: PRIMARY,
            success: GREEN,
            danger: RED,
        },
    )
}

// --- Container styles ---

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

// --- Text input styles ---

fn input_style_impl(border_color: Color, value_color: Color) -> text_input::Style {
    text_input::Style {
        background: Background::Color(BG_HARD),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: FG4,
        placeholder: FG4,
        value: value_color,
        selection: PRIMARY_C,
    }
}

pub fn input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border = if matches!(status, text_input::Status::Focused) {
        PRIMARY_C
    } else {
        BORDER
    };
    input_style_impl(border, FG)
}

pub fn mono_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border = if matches!(status, text_input::Status::Focused) {
        PRIMARY_C
    } else {
        BORDER
    };
    input_style_impl(border, PRIMARY_C)
}

pub fn purple_input_style(_theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(BG_HARD),
        border: Border {
            color: PURPLE,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: PURPLE,
        placeholder: FG4,
        value: PURPLE,
        selection: PURPLE,
    }
}

// --- Button styles ---

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
pub fn danger_header_button_style(
    _theme: &iced::Theme,
    _status: button::Status,
) -> button::Style {
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
                color: if selected { PRIMARY_C } else { Color::TRANSPARENT },
                width: if selected { 1.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Solid primary-colour button for "Send to Inject" actions.
pub fn send_to_inject_button_style(
    _theme: &iced::Theme,
    _status: button::Status,
) -> button::Style {
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
                color: if selected { PRIMARY_C } else { Color::TRANSPARENT },
                width: if selected { 1.0 } else { 0.0 },
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

// --- Scrollable styles ---

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

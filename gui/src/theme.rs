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

// Borders
pub const BORDER: Color = hex(0x41, 0x48, 0x45);
pub const BORDER_MID: Color = hex(0x8b, 0x92, 0x8e);

// Text
pub const FG: Color = hex(0xe3, 0xe2, 0xe0);
pub const FG2: Color = hex(0xc1, 0xc8, 0xc4);
pub const FG4: Color = hex(0x8b, 0x92, 0x8e);

// Accents
pub const PRIMARY: Color = hex(0xab, 0xce, 0xc0);
pub const PRIMARY_C: Color = hex(0x83, 0xa5, 0x98);
pub const GREEN: Color = hex(0xa1, 0xd4, 0x8e);
pub const GREEN_DIM: Color = hex(0x27, 0x53, 0x1c);
pub const YELLOW: Color = hex(0xd7, 0x99, 0x21);
pub const RED: Color = hex(0xcc, 0x24, 0x1d);
pub const RED_BRIGHT: Color = hex(0xfb, 0x49, 0x34);
pub const RED_CONT: Color = hex(0x93, 0x00, 0x0a);
pub const PURPLE: Color = hex(0xc2, 0x92, 0x8d);
pub const ORANGE: Color = hex(0xf4, 0x7b, 0x20);

// Log strip
pub const LOG_TIME: Color = hex(0xa8, 0x99, 0x84);
pub const LOG_INFO: Color = hex(0x83, 0xa5, 0x98);
pub const LOG_OK: Color = hex(0xb8, 0xbb, 0x26);
pub const LOG_WARN: Color = hex(0xfb, 0x49, 0x34);
pub const INJECT_BTN: Color = hex(0xb8, 0xbb, 0x26);

// Spacing (pixels)
pub const SP1: f32 = 4.0;
pub const SP2: f32 = 8.0;
pub const SP3: f32 = 12.0;
pub const SP4: f32 = 16.0;

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

pub const FONT_MONO_BOLD: Font = Font {
    family: font::Family::Name("JetBrains Mono"),
    weight: font::Weight::Bold,
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
        background: Some(Background::Color(Color {
            r: 0.576,
            g: 0.0,
            b: 0.039,
            a: 0.1,
        })),
        border: Border {
            color: RED_CONT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border_color = if matches!(status, text_input::Status::Focused) {
        PRIMARY_C
    } else {
        BORDER
    };
    text_input::Style {
        background: Background::Color(BG_HARD),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: FG4,
        placeholder: FG4,
        value: FG,
        selection: PRIMARY_C,
    }
}

pub fn mono_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border_color = if matches!(status, text_input::Status::Focused) {
        PRIMARY_C
    } else {
        BORDER
    };
    text_input::Style {
        background: Background::Color(BG_HARD),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: FG4,
        placeholder: FG4,
        value: PRIMARY_C,
        selection: PRIMARY_C,
    }
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

pub fn nav_active_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(BG_HIGH)),
        text_color: PRIMARY,
        border: Border {
            color: PRIMARY,
            width: 2.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn nav_inactive_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_HIGHEST,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: FG2,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
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
        _ => INJECT_BTN,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: BG_HARD,
        border: Border {
            color: INJECT_BTN,
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

fn rail_style(bg: Color) -> scrollable::Rail {
    scrollable::Rail {
        background: Some(Background::Color(bg)),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: 2.0.into(),
        },
        scroller: scrollable::Scroller {
            color: BORDER_MID,
            border: Border {
                color: BORDER,
                width: 0.0,
                radius: 2.0.into(),
            },
        },
    }
}

pub fn log_scrollable_style(
    _theme: &iced::Theme,
    _status: scrollable::Status,
) -> scrollable::Style {
    scrollable::Style {
        container: container::Style {
            background: Some(Background::Color(BG_HARD)),
            ..Default::default()
        },
        vertical_rail: rail_style(BG_HIGH),
        horizontal_rail: rail_style(Color::TRANSPARENT),
        gap: None,
    }
}

pub fn table_scrollable_style(
    _theme: &iced::Theme,
    _status: scrollable::Status,
) -> scrollable::Style {
    scrollable::Style {
        container: container::Style {
            background: Some(Background::Color(BG)),
            ..Default::default()
        },
        vertical_rail: rail_style(BG_HIGH),
        horizontal_rail: rail_style(Color::TRANSPARENT),
        gap: None,
    }
}

use iced::widget::text_input;
use iced::{Background, Border, Color};

use super::{BG_HARD, BORDER, FG, FG4, PRIMARY_C, PURPLE};

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

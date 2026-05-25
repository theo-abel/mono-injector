use iced::widget::{row, text, toggler};
use iced::{Color, Element, Length};

use crate::theme::{BORDER, FG, FONT_UI, PRIMARY_C, SP2};

/// A labelled toggle switch with optional custom track-on color.
///
/// Uses iced's built-in toggler, styled to match the design system.
pub fn toggle<'a, M: Clone + 'a>(
    label: &'a str,
    value: bool,
    on_toggle: impl Fn(bool) -> M + 'a,
    track_color: Option<Color>,
) -> Element<'a, M> {
    let on_color = track_color.unwrap_or(PRIMARY_C);
    row![
        text(label).size(14).font(FONT_UI).color(FG),
        iced::widget::horizontal_space(),
        toggler(value)
            .on_toggle(on_toggle)
            .size(16)
            .style(move |_theme, _status| toggler_style(value, on_color)),
    ]
    .spacing(SP2)
    .width(Length::Fill)
    .into()
}

fn toggler_style(is_on: bool, on_color: Color) -> iced::widget::toggler::Style {
    iced::widget::toggler::Style {
        background: if is_on {
            on_color
        } else {
            crate::theme::BG_HIGHEST
        },
        background_border_width: 1.0,
        background_border_color: BORDER,
        foreground: FG,
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
    }
}

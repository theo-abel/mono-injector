use iced::widget::container;
use iced::{Background, Border, Element, Length};

use crate::theme::{BG, BG_CONT, BG_HIGH, BORDER, FG, FG2, FONT_MONO, SP3};

/// Wraps content in a sticky header cell with the elevated header background.
pub fn header_cell<'a, M: 'a>(content: impl Into<Element<'a, M>>, flex: u16) -> Element<'a, M> {
    container(content)
        .width(Length::FillPortion(flex))
        .padding(SP3)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_HIGH)),
            border: Border {
                color: BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            text_color: Some(FG2),
            ..Default::default()
        })
        .into()
}

/// Wraps content in a data cell with alternating zebra row colouring.
pub fn data_cell<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    flex: u16,
    odd_row: bool,
) -> Element<'a, M> {
    let bg = if odd_row { BG_CONT } else { BG };
    data_cell_bg(content, flex, bg)
}

pub fn data_cell_bg<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    flex: u16,
    bg: iced::Color,
) -> Element<'a, M> {
    container(content)
        .width(Length::FillPortion(flex))
        .padding(SP3)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        })
        .into()
}

/// Returns label text styled as a table column header (caps, mono, muted).
pub fn header_label<'a, M: 'a>(label: &'a str) -> Element<'a, M> {
    iced::widget::text(label)
        .size(10)
        .font(FONT_MONO)
        .color(FG2)
        .into()
}

/// Returns value text styled as a monospace table cell value.
pub fn mono_value<'a, M: 'a>(value: &'a str) -> Element<'a, M> {
    iced::widget::text(value)
        .size(13)
        .font(FONT_MONO)
        .color(FG)
        .into()
}

/// Returns a small muted monospace annotation (PIDs, timestamps, etc.).
pub fn muted_mono<'a, M: 'a>(value: &'a str) -> Element<'a, M> {
    iced::widget::text(value)
        .size(11)
        .font(FONT_MONO)
        .color(FG2)
        .into()
}

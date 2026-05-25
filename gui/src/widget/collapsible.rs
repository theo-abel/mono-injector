use iced::widget::{button, column, container, horizontal_space, row, text};
use iced::{Border, Element, Length};

use crate::theme::{BG_HIGH, BG_HIGHEST, BG_LOW, BORDER, FG, FONT_UI_SEMIBOLD, SP3};
use crate::widget::icon;

/// A card panel with a clickable header that shows or hides a body element.
///
/// The header row always includes a chevron indicating expand state. Clicking
/// anywhere on the header fires `on_toggle`.
pub fn collapsible<'a, M: Clone + 'a>(
    title: &'a str,
    body: impl Into<Element<'a, M>>,
    expanded: bool,
    on_toggle: M,
) -> Element<'a, M> {
    let header = build_header(title, on_toggle);

    let card = if expanded {
        column![header, body.into()]
    } else {
        column![header]
    };

    container(card)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(BG_LOW)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn build_header<'a, M: Clone + 'a>(title: &'a str, on_toggle: M) -> Element<'a, M> {
    button(
        row![
            text(title).size(14).font(FONT_UI_SEMIBOLD).color(FG),
            horizontal_space(),
            icon::icon(icon::EXPAND_MORE, 20.0, FG),
        ]
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(on_toggle)
    .padding(SP3)
    .width(Length::Fill)
    .style(|_, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => BG_HIGHEST,
            _ => BG_HIGH,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: FG,
            border: Border {
                color: BORDER,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

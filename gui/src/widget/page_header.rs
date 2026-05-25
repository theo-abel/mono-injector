use iced::widget::{column, container, horizontal_rule, text};
use iced::{Element, Length};

use crate::theme::{FG, FG2, FONT_UI, FONT_UI_SEMIBOLD, SP3, SP4};

pub fn view<M: 'static>(title: &'static str, description: &'static str) -> Element<'static, M> {
    column![
        container(
            column![
                text(title).size(18).font(FONT_UI_SEMIBOLD).color(FG),
                text(description).size(13).font(FONT_UI).color(FG2),
            ]
            .spacing(4),
        )
        .padding([0.0, SP3]),
        horizontal_rule(1),
    ]
    .spacing(SP4)
    .width(Length::Fill)
    .into()
}

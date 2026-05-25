use iced::widget::{Space, container};
use iced::{Background, Border, Element, Length};

use crate::nav::View;
use crate::theme::{BG, BORDER, FG2};

pub fn view<M: 'static>(_active: View) -> Element<'static, M> {
    container(Space::new(Length::Fill, 1))
        .height(48)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(BG)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            text_color: Some(FG2),
            ..Default::default()
        })
        .into()
}

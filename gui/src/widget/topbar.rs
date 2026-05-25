use iced::widget::{container, row, text};
use iced::{Background, Border, Element, Length};

use crate::nav::View;
use crate::theme::{BG, BORDER, FG2, FONT_UI_SEMIBOLD, PRIMARY, SP2, SP3};
use crate::widget::icon;

pub fn view<M: 'static>(active: View) -> Element<'static, M> {
    let (title, glyph) = view_title(active);
    container(
        row![
            icon::icon(glyph, 22.0, PRIMARY),
            text(title).size(18).font(FONT_UI_SEMIBOLD).color(PRIMARY),
        ]
        .spacing(SP2)
        .align_y(iced::alignment::Vertical::Center),
    )
    .height(48)
    .width(Length::Fill)
    .padding([0.0, SP3])
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

fn view_title(active: View) -> (&'static str, &'static str) {
    match active {
        View::Inject => ("Inject Assembly", icon::INPUT),
        View::Eject => ("ASSEMBLY UNLOADER", icon::EJECT),
        View::Status => ("Active Injections", icon::QUERY_STATS),
        View::Processes => ("Running Processes", icon::MEMORY),
        View::Profiles => ("Profiles", icon::ACCOUNT_BOX),
    }
}

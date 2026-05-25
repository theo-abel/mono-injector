use iced::widget::{button, column, container, horizontal_rule, row, text, vertical_space};
use iced::{Background, Border, Color, Element, Length};

use crate::nav::View;
use crate::theme::{
    BG_CONT, BG_HIGH, BG_HIGHEST, BORDER, FG, FG2, FG4, FONT_MONO, FONT_UI, FONT_UI_SEMIBOLD,
    PRIMARY, RED, RED_BRIGHT, SP2, SP3,
};
use crate::widget::icon;

/// Messages emitted by the sidebar.
#[derive(Debug, Clone)]
pub enum Msg {
    Navigate(View),
    ClearLogs,
}

// All content in nav_button is 'static (literals + Copy enums), so we can
// return Element<'static, Msg> and avoid tying the sidebar's lifetime to a
// locally-allocated [NavItem; N] array.
fn nav_button(
    glyph: &'static str,
    label: &'static str,
    view: View,
    active: bool,
) -> Element<'static, Msg> {
    let label_color = if active { PRIMARY } else { FG2 };
    let accent = if active { PRIMARY } else { Color::TRANSPARENT };
    let button = button(
        row![
            icon::icon(glyph, 22.0, label_color),
            text(label).size(14).font(FONT_UI).color(label_color),
        ]
        .spacing(SP2),
    )
    .on_press(Msg::Navigate(view))
    .width(Length::Fill)
    .padding([12.0, SP3])
    .style(move |_, status| nav_button_style(active, status));

    row![
        container(iced::widget::Space::new(3, 1)).style(move |_| accent_style(accent)),
        button
    ]
    .height(48)
    .width(Length::Fill)
    .into()
}

fn accent_style(color: Color) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

fn nav_button_style(active: bool, status: button::Status) -> button::Style {
    let bg = if active {
        BG_HIGH
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        BG_HIGHEST
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: if active { PRIMARY } else { FG2 },
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn brand_icon() -> Element<'static, Msg> {
    icon::icon(icon::TERMINAL, 22.0, PRIMARY).into()
}

fn app_title() -> Element<'static, Msg> {
    container(
        column![
            row![
                brand_icon(),
                text("mono-injector")
                    .size(18)
                    .font(FONT_UI_SEMIBOLD)
                    .color(PRIMARY),
            ]
            .spacing(SP2),
            text(concat!("v", env!("CARGO_PKG_VERSION")))
                .size(11)
                .font(FONT_MONO)
                .color(FG4),
        ]
        .spacing(4),
    )
    .padding(SP3)
    .width(Length::Fill)
    .into()
}

fn clear_logs_button() -> Element<'static, Msg> {
    button(
        row![
            icon::icon(icon::DELETE, 20.0, RED),
            text("Clear Logs").size(13).font(FONT_UI).color(FG4),
        ]
        .spacing(SP2),
    )
    .on_press(Msg::ClearLogs)
    .width(Length::Fill)
    .padding([SP2, SP3])
    .style(|_, status| {
        let tc = match status {
            button::Status::Hovered | button::Status::Pressed => RED_BRIGHT,
            _ => FG4,
        };
        button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: tc,
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

/// Renders the 200px fixed-width navigation sidebar.
pub fn view(active: View) -> Element<'static, Msg> {
    let nav_items = [
        (icon::INPUT, "Inject", View::Inject),
        (icon::EJECT, "Eject", View::Eject),
        (icon::QUERY_STATS, "Status", View::Status),
        (icon::MEMORY, "Processes", View::Processes),
        (icon::ACCOUNT_BOX, "Profiles", View::Profiles),
    ];
    let nav = column(
        nav_items
            .iter()
            .map(|(ic, lb, v)| nav_button(ic, lb, *v, *v == active))
            .collect::<Vec<_>>(),
    )
    .spacing(0);

    container(
        column![
            app_title(),
            horizontal_rule(1),
            nav,
            vertical_space(),
            horizontal_rule(1),
            container(clear_logs_button()).padding(SP3)
        ]
        .spacing(0),
    )
    .width(200)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(BG_CONT)),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        text_color: Some(FG),
        ..Default::default()
    })
    .into()
}

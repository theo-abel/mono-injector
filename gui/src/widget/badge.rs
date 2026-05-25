use iced::advanced::text::IntoFragment;
use iced::widget::container;
use iced::{Background, Border, Color, Element, Padding};

use crate::theme::{
    self, FONT_MONO, MONO_BADGE_BG, RUNTIME_BADGE_BORDER, UNITY_BADGE_BG, UNITY_BADGE_FG,
};

/// A small pill-shaped label used for PIDs, runtime types, handles, etc.
///
/// Accepts both `&'static str` literals and owned `String` values so callers
/// do not have to worry about string lifetimes.
pub fn badge<'a, M: 'a + 'static>(
    label: impl IntoFragment<'static>,
    bg: Color,
    fg: Color,
    border: Color,
) -> Element<'a, M> {
    container(iced::widget::text(label).size(11).color(fg).font(FONT_MONO))
        .padding(Padding::from([2.0, 6.0]))
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border,
                width: 1.0,
                radius: 2.0.into(),
            },
            text_color: Some(fg),
            ..Default::default()
        })
        .into()
}

/// Convenience badge for process runtimes detected from loaded modules.
pub fn runtime_badge<'a, M: 'a + 'static>(runtime: &'static str) -> Element<'a, M> {
    let (bg, fg, border) = match runtime {
        "Unity" => (UNITY_BADGE_BG, UNITY_BADGE_FG, RUNTIME_BADGE_BORDER),
        "Mono" => (MONO_BADGE_BG, theme::PRIMARY, RUNTIME_BADGE_BORDER),
        _ => (Color::TRANSPARENT, theme::FG4, Color::TRANSPARENT),
    };
    badge(runtime, bg, fg, border)
}

/// Convenience badge for a hex handle value (purple-tinted).
pub fn handle_badge<'a, M: 'a + 'static>(handle: impl IntoFragment<'static>) -> Element<'a, M> {
    badge(handle, theme::BG_HIGH, theme::PURPLE, theme::PURPLE)
}

/// Convenience badge for "STALE" records.
pub fn stale_badge<'a, M: 'a + 'static>() -> Element<'a, M> {
    badge("STALE", theme::BG_HIGH, theme::YELLOW, theme::YELLOW)
}

/// Convenience badge for "DEAD" handles.
pub fn dead_badge<'a, M: 'a + 'static>() -> Element<'a, M> {
    badge("DEAD", Color::TRANSPARENT, theme::RED, theme::RED)
}

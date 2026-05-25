use std::time::{SystemTime, UNIX_EPOCH};

use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Background, Border, Color, Element, Length};

use crate::theme::{
    BG_HARD, BORDER, FG, FG4, FONT_MONO, LOG_INFO, LOG_OK, LOG_WARN, PRIMARY_C, RED, SP2, SP3,
};

pub type LogContent = text_editor::Content;

/// Severity classification for a log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Ok,
    Warn,
    Error,
}

/// A single timestamped log line shown in the bottom console strip.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Link {
    Documentation,
    Github,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Edit(text_editor::Action),
    Open(Link),
}

impl LogEntry {
    pub fn info(message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level: LogLevel::Info,
            message,
        }
    }

    pub fn ok(message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level: LogLevel::Ok,
            message,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level: LogLevel::Error,
            message,
        }
    }

    pub fn warn(message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level: LogLevel::Warn,
            message,
        }
    }
}

fn level_label(level: &LogLevel) -> (&'static str, Color) {
    match level {
        LogLevel::Info => ("[INFO]", LOG_INFO),
        LogLevel::Ok => ("[OK]", LOG_OK),
        LogLevel::Warn => ("[WARN]", LOG_WARN),
        LogLevel::Error => ("[ERROR]", RED),
    }
}

fn format_timestamp(time: SystemTime) -> String {
    let dur = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

fn format_line(entry: &LogEntry) -> String {
    let ts = format_timestamp(entry.timestamp);
    let (label, _) = level_label(&entry.level);
    format!("{ts} {label} {}", entry.message)
}

fn format_entries(entries: &[LogEntry]) -> String {
    entries
        .iter()
        .map(format_line)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn sync_content(content: &mut LogContent, entries: &[LogEntry]) {
    *content = text_editor::Content::with_text(&format_entries(entries));
}

fn strip_header() -> Element<'static, Msg> {
    container(
        row![
            text("EXECUTION LOG").size(10).font(FONT_MONO).color(FG4),
            Space::new().width(Length::Fill),
            link_button("Documentation", Link::Documentation),
            link_button("GitHub", Link::Github),
        ]
        .spacing(SP3)
        .padding([0.0, SP3]),
    )
    .width(Length::Fill)
    .padding([SP2, 0.0])
    .style(|_| container::Style {
        background: Some(Background::Color(BG_HARD)),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn link_button(label: &'static str, link: Link) -> Element<'static, Msg> {
    button(text(label).size(10).font(FONT_MONO).color(FG4))
        .on_press(Msg::Open(link))
        .padding(0)
        .style(|_, status| {
            let color = match status {
                button::Status::Hovered | button::Status::Pressed => crate::theme::PRIMARY,
                button::Status::Disabled | button::Status::Active => FG4,
            };
            button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: color,
                border: Border::default(),
                ..Default::default()
            }
        })
        .into()
}

/// Renders the fixed-height bottom log console strip.
pub fn view(content: &LogContent) -> Element<'_, Msg> {
    let body = text_editor(content)
        .on_action(Msg::Edit)
        .font(FONT_MONO)
        .size(11)
        .padding([SP2, SP3])
        .height(Length::Fill)
        .style(log_editor_style);

    let body = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_HARD)),
            ..Default::default()
        });

    container(column![strip_header(), body])
        .width(Length::Fill)
        .height(120)
        .style(|_| container::Style {
            background: Some(Background::Color(BG_HARD)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn log_editor_style(_theme: &iced::Theme, _status: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(BG_HARD),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: FG4,
        value: FG,
        selection: PRIMARY_C.scale_alpha(0.45),
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

use iced::widget::{column, container, horizontal_space, row, scrollable, text};
use iced::{Background, Border, Color, Element, Length};

use crate::theme::{
    BG_HARD, BORDER, FG, FG4, FONT_MONO, LOG_INFO, LOG_OK, LOG_TIME, LOG_WARN, RED, SP2, SP3,
};

/// Severity classification for a log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Ok,
    Warn,
    Error,
    System,
}

/// A single timestamped log line shown in the bottom console strip.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
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

    pub fn warn(message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level: LogLevel::Warn,
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

    pub fn system(message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level: LogLevel::System,
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
        LogLevel::System => ("[SYS]", FG4),
    }
}

fn format_timestamp(time: SystemTime) -> String {
    let secs = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let millis = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_millis());
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

fn log_line<'a, M: 'a>(entry: &'a LogEntry) -> Element<'a, M> {
    let ts = format_timestamp(entry.timestamp);
    let (label, color) = level_label(&entry.level);
    row![
        text(ts).size(11).font(FONT_MONO).color(LOG_TIME),
        text(label).size(11).font(FONT_MONO).color(color),
        text(entry.message.as_str())
            .size(11)
            .font(FONT_MONO)
            .color(FG),
    ]
    .spacing(SP2)
    .into()
}

fn strip_header<'a, M: 'a>() -> Element<'a, M> {
    container(
        row![
            text("EXECUTION LOG").size(10).font(FONT_MONO).color(FG4),
            horizontal_space(),
            text("System Ready | Latency: 4ms")
                .size(10)
                .font(FONT_MONO)
                .color(FG4),
        ]
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

/// Renders the fixed-height bottom log console strip.
pub fn view<M: Clone + 'static>(entries: &[LogEntry]) -> Element<'_, M> {
    let lines = if entries.is_empty() {
        column![system_line()]
    } else {
        column(entries.iter().map(log_line).collect::<Vec<_>>())
    }
    .spacing(1)
    .padding([SP2, SP3]);

    let scroll = scrollable(lines)
        .height(Length::Fill)
        .style(crate::theme::log_scrollable_style);

    container(column![strip_header(), scroll])
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

fn system_line<'a, M: 'a>() -> Element<'a, M> {
    row![
        text("System Ready | Latency: 4ms")
            .size(11)
            .font(FONT_MONO)
            .color(crate::theme::GREEN),
        horizontal_space(),
        text("Documentation   API Reference   GitHub")
            .size(11)
            .font(FONT_MONO)
            .color(FG4),
    ]
    .into()
}

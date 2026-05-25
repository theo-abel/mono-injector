use iced::Color;
use iced::widget::{Text, text};

use crate::theme::FONT_ICON;

pub const ACCOUNT_BOX: &str = "\u{e851}";
pub const ACCOUNT_TREE: &str = "\u{e97a}";
pub const ADD: &str = "\u{e145}";
pub const CHECK_CIRCLE: &str = "\u{f0be}";
pub const DELETE: &str = "\u{e92e}";
pub const DELETE_FOREVER: &str = "\u{e92b}";
pub const DELETE_SWEEP: &str = "\u{e16c}";
pub const EDIT: &str = "\u{f097}";
pub const EJECT: &str = "\u{e8fb}";
pub const EXPAND_MORE: &str = "\u{e5cf}";
pub const FOLDER: &str = "\u{e2c7}";
pub const INPUT: &str = "\u{e890}";
pub const MEMORY: &str = "\u{e322}";
pub const MY_LOCATION: &str = "\u{e55c}";
pub const PLAY_ARROW: &str = "\u{e037}";
pub const QUERY_STATS: &str = "\u{e4fc}";
pub const REFRESH: &str = "\u{e5d5}";
pub const SCIENCE: &str = "\u{ea4b}";
pub const SETTINGS: &str = "\u{e8b8}";
pub const TERMINAL: &str = "\u{eb8e}";
pub const WARNING: &str = "\u{e002}";

pub fn icon(name: &str, size: f32, color: Color) -> Text<'_> {
    text(name).size(size).font(FONT_ICON).color(color)
}

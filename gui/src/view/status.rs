use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Background, Element, Length, Task};
use mono_injector_core::process::ProcessInfo;
use mono_injector_core::state::InjectionRecord;

use crate::theme::{
    self, BG, BG_CONT, BG_STALE, BORDER, FG, FG2, FG4, FONT_MONO, FONT_MONO_MEDIUM, FONT_UI, RED,
    SP2, SP4,
};
use crate::util;
use crate::widget::{badge, icon, page_header, table};

/// An injection record annotated with live/stale status.
#[derive(Debug, Clone)]
pub struct RecordRow {
    pub record: InjectionRecord,
    pub is_stale: bool,
}

/// State for the Status (active injections) view.
#[derive(Debug, Default, Clone)]
pub struct StatusState {
    pub rows: Vec<RecordRow>,
    pub filter_text: String,
    pub loading: bool,
    pub confirm_clean_all: bool,
}

/// Messages handled by the Status view.
#[derive(Debug, Clone)]
pub enum StatusMsg {
    Load,
    Loaded(Result<Vec<RecordRow>, String>),
    FilterChanged(String),
    Refresh,
    CleanStale,
    CleanAll,
    ConfirmCleanAll,
    CancelCleanAll,
    Cleaned(Result<usize, String>),
    EjectRecord(InjectionRecord),
}

impl StatusMsg {
    pub fn log_entry(&self) -> Option<crate::widget::log_strip::LogEntry> {
        use crate::widget::log_strip::LogEntry;
        match self {
            Self::Cleaned(Ok(n)) => Some(LogEntry::ok(format!("Cleaned {n} injection record(s)"))),
            Self::Cleaned(Err(e)) => Some(LogEntry::error(format!("Clean failed: {e}"))),
            _ => None,
        }
    }
}

pub fn update(state: &mut StatusState, msg: StatusMsg) -> Task<StatusMsg> {
    match msg {
        StatusMsg::Load | StatusMsg::Refresh => {
            state.loading = true;
            load_records()
        }
        StatusMsg::Loaded(r) => {
            apply_loaded(state, r);
            Task::none()
        }
        StatusMsg::FilterChanged(f) => {
            state.filter_text = f;
            Task::none()
        }
        StatusMsg::CleanStale => clean_records(mono_injector_core::state::CleanMode::Stale),
        StatusMsg::CleanAll => {
            state.confirm_clean_all = true;
            Task::none()
        }
        StatusMsg::ConfirmCleanAll => {
            state.confirm_clean_all = false;
            clean_records(mono_injector_core::state::CleanMode::All)
        }
        StatusMsg::CancelCleanAll => {
            state.confirm_clean_all = false;
            Task::none()
        }
        // After any successful clean, reload from the database so the view
        // reflects the actual state regardless of which CleanMode was used.
        StatusMsg::Cleaned(r) => {
            if r.is_ok() {
                state.loading = true;
                load_records()
            } else {
                Task::none()
            }
        }
        StatusMsg::EjectRecord(_) => Task::none(), // routed up to app for navigation
    }
}

fn apply_loaded(state: &mut StatusState, result: Result<Vec<RecordRow>, String>) {
    state.loading = false;
    if let Ok(rows) = result {
        state.rows = rows;
    }
}

fn load_records() -> Task<StatusMsg> {
    Task::perform(util::run_blocking(build_record_rows), StatusMsg::Loaded)
}

fn build_record_rows() -> Result<Vec<RecordRow>, String> {
    let records = mono_injector_core::state::all().map_err(|e| e.to_string())?;
    let live = mono_injector_core::process::all_processes();
    Ok(records.into_iter().map(|r| make_row(r, &live)).collect())
}

fn make_row(record: InjectionRecord, live: &[ProcessInfo]) -> RecordRow {
    let is_stale = !live
        .iter()
        .any(|p| p.pid == record.pid && p.start_time == record.start_time);
    RecordRow { record, is_stale }
}

fn clean_records(mode: mono_injector_core::state::CleanMode) -> Task<StatusMsg> {
    Task::perform(
        util::run_blocking(move || {
            mono_injector_core::state::clean_stale_records(mode).map_err(|e| e.to_string())
        }),
        StatusMsg::Cleaned,
    )
}

pub fn view(state: &StatusState) -> Element<'_, StatusMsg> {
    let content = if state.rows.is_empty() && !state.loading {
        empty_state()
    } else {
        records_table(state)
    };

    let modal = state
        .confirm_clean_all
        .then(|| confirm_modal(state.rows.len()));

    let base = container(
        column![
            page_header::view(
                "Active Injections",
                "Review remembered injection handles and clean stale records.",
            ),
            toolbar(state),
            content
        ]
        .spacing(SP4)
        .height(Length::Fill),
    )
    .center_x(Length::Fill)
    .padding(SP4)
    .height(Length::Fill);

    if let Some(overlay) = modal {
        iced::widget::stack![base, overlay].into()
    } else {
        base.into()
    }
}

fn toolbar(state: &StatusState) -> Element<'_, StatusMsg> {
    container(
        row![
            text_input("Filter...", &state.filter_text)
                .on_input(StatusMsg::FilterChanged)
                .width(256)
                .style(theme::input_style),
            button(icon::icon(icon::REFRESH, 18.0, FG2))
                .on_press(StatusMsg::Refresh)
                .style(theme::ghost_button_style),
            iced::widget::Space::new().width(Length::Fill),
            button(
                row![
                    icon::icon(icon::DELETE_SWEEP, 16.0, RED),
                    text("Clean Stale").size(12).font(FONT_UI).color(RED)
                ]
                .spacing(4)
            )
            .on_press(StatusMsg::CleanStale)
            .style(theme::ghost_button_style),
            button(
                row![
                    icon::icon(icon::WARNING, 16.0, RED),
                    text("Clean All").size(12).font(FONT_UI).color(RED)
                ]
                .spacing(4)
            )
            .on_press(StatusMsg::CleanAll)
            .style(theme::danger_outline_button_style),
        ]
        .spacing(SP2),
    )
    .padding(SP2)
    .width(Length::Fill)
    .style(|_| theme::elevated_panel_style())
    .into()
}

fn table_header<'a>() -> Element<'a, StatusMsg> {
    let hd = |label, flex| table::header_cell(table::header_label(label), flex);
    row![
        hd("PROCESS", 20),
        hd("PID", 8),
        hd("ASSEMBLY", 22),
        hd("ENTRY POINT", 18),
        hd("HANDLE", 12),
        hd("INJECTED AT", 10),
        hd("ACTIONS", 10),
    ]
    .into()
}

fn records_table(state: &StatusState) -> Element<'_, StatusMsg> {
    let filter = state.filter_text.to_lowercase();
    let body = state
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            filter.is_empty() || r.record.process_name.to_lowercase().contains(&filter)
        })
        .map(|(i, r)| record_row(r, i % 2 == 1))
        .collect::<Vec<_>>();

    let rows_col = if body.is_empty() {
        column![text("No results match the filter").size(13).color(FG4)]
    } else {
        column(body).spacing(0)
    };

    container(
        column![
            table_header(),
            scrollable(rows_col)
                .height(Length::Fill)
                .style(theme::table_scrollable_style),
        ]
        .height(Length::Fill),
    )
    .height(Length::Fill)
    .style(|_| theme::panel_style())
    .into()
}

fn record_row(r: &RecordRow, odd: bool) -> Element<'_, StatusMsg> {
    let bg = if r.is_stale {
        BG_STALE
    } else if odd {
        BG_CONT
    } else {
        BG
    };

    let assembly_name = r
        .record
        .assembly_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map_or_else(|| "—".to_owned(), |n| n.to_string_lossy().into_owned());

    let stale_badge = r.is_stale.then(badge::stale_badge);
    let dead_handle = r.is_stale.then(badge::dead_badge);
    let handle_el = dead_handle.unwrap_or_else(|| badge::handle_badge(r.record.handle.clone()));

    let process_cell: Element<_> = if let Some(sb) = stale_badge {
        row![
            text(r.record.process_name.clone())
                .size(13)
                .font(FONT_MONO)
                .color(FG),
            sb
        ]
        .spacing(SP2)
        .into()
    } else {
        text(r.record.process_name.clone())
            .size(13)
            .font(FONT_MONO)
            .color(FG)
            .into()
    };

    container(row![
        table::data_cell_bg(process_cell, 20, bg),
        table::data_cell_bg(
            text(r.record.pid.to_string())
                .size(12)
                .font(FONT_MONO)
                .color(FG4),
            8,
            bg
        ),
        table::data_cell_bg(
            text(assembly_name).size(12).font(FONT_MONO).color(FG2),
            22,
            bg
        ),
        table::data_cell_bg(
            text(r.record.entry())
                .size(12)
                .font(FONT_MONO)
                .color(theme::PRIMARY_C),
            18,
            bg
        ),
        table::data_cell_bg(handle_el, 12, bg),
        table::data_cell_bg(
            text(util::relative_time(r.record.injected_at))
                .size(11)
                .font(FONT_MONO)
                .color(FG4),
            10,
            bg
        ),
        table::data_cell_bg(row_actions(&r.record, r.is_stale), 10, bg),
    ])
    .width(Length::Fill)
    .style(move |_| iced::widget::container::Style {
        background: Some(Background::Color(bg)),
        border: iced::Border {
            color: if r.is_stale { RED } else { BORDER },
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn row_actions(record: &InjectionRecord, is_stale: bool) -> Element<'_, StatusMsg> {
    if is_stale {
        button(icon::icon(icon::DELETE_FOREVER, 14.0, RED))
            .on_press(StatusMsg::EjectRecord(record.clone()))
            .style(theme::ghost_button_style)
            .into()
    } else {
        button(text("EJECT").size(10).font(FONT_MONO_MEDIUM).color(RED))
            .on_press(StatusMsg::EjectRecord(record.clone()))
            .style(theme::danger_outline_button_style)
            .into()
    }
}

fn empty_state<'a>() -> Element<'a, StatusMsg> {
    container(
        column![
            text("◈").size(48).color(FG4),
            text("No active injections.").size(14).color(FG4),
            text("Ready to operate.").size(14).color(FG4),
        ]
        .spacing(SP2)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .center(Length::Fill)
    .into()
}

fn confirm_modal(record_count: usize) -> Element<'static, StatusMsg> {
    let overlay = container(
        column![
            text("Confirm Clean All").size(18).color(RED),
            text(format!(
                "This will remove all {record_count} injection records including active ones."
            ))
            .size(14)
            .color(FG),
            row![
                button(text("Cancel").size(13))
                    .on_press(StatusMsg::CancelCleanAll)
                    .style(theme::ghost_button_style),
                button(text("Confirm").size(13).color(FG))
                    .on_press(StatusMsg::ConfirmCleanAll)
                    .style(theme::danger_button_style),
            ]
            .spacing(SP2),
        ]
        .spacing(SP4)
        .max_width(360),
    )
    .padding(SP4)
    .style(|_| theme::elevated_panel_style());

    container(overlay).center(Length::Fill).into()
}

use iced::widget::{
    button, column, container, row, scrollable, table as iced_table, text, text_input, toggler,
};
use iced::{Element, Length, Task};
use mono_injector_core::process::{ListOptions, ModuleFilter, ProcessListing};

use crate::theme::{self, BG, BG_CONT, BG_HIGH, FG, FG2, FG4, FONT_MONO, PRIMARY, SP2, SP4};
use crate::widget::{badge, icon, page_header};

// Width of the hidden space placeholder that reserves room for the send button.
const SEND_BUTTON_SLOT_WIDTH: u32 = 132;

#[derive(Debug, Clone)]
struct ProcessTableRow {
    process: ProcessListing,
    odd: bool,
    selected: bool,
}

impl ProcessTableRow {
    fn bg(&self) -> iced::Color {
        if self.selected {
            BG_HIGH
        } else if self.odd {
            BG_CONT
        } else {
            BG
        }
    }
}

/// Which runtime family to filter by in the process browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeFilter {
    #[default]
    All,
    Mono,
    Unity,
}

impl RuntimeFilter {
    fn as_str(self) -> &'static str {
        match self {
            RuntimeFilter::All => "All",
            RuntimeFilter::Mono => "Mono",
            RuntimeFilter::Unity => "Unity",
        }
    }
}

/// State for the Processes browser view.
#[derive(Debug, Default, Clone)]
pub struct ProcessesState {
    pub all_processes: Vec<ProcessListing>,
    pub filter_text: String,
    pub runtime_filter: RuntimeFilter,
    pub show_modules: bool,
    pub selected_pid: Option<u32>,
    pub loading: bool,
}

/// Messages handled by the Processes view.
#[derive(Debug, Clone)]
pub enum ProcessesMsg {
    Load,
    Refresh,
    Loaded(Vec<ProcessListing>),
    FilterChanged(String),
    RuntimeFilterChanged(RuntimeFilter),
    ToggleModules(bool),
    SelectPid(u32),
    SendToInject(ProcessListing),
}

pub fn update(state: &mut ProcessesState, msg: ProcessesMsg) -> Task<ProcessesMsg> {
    match msg {
        ProcessesMsg::Load | ProcessesMsg::Refresh => {
            state.loading = true;
            load_processes()
        }
        ProcessesMsg::Loaded(list) => {
            state.loading = false;
            state.all_processes = list;
            Task::none()
        }
        ProcessesMsg::FilterChanged(f) => {
            state.filter_text = f;
            Task::none()
        }
        ProcessesMsg::RuntimeFilterChanged(f) => {
            state.runtime_filter = f;
            Task::none()
        }
        ProcessesMsg::ToggleModules(v) => {
            state.show_modules = v;
            Task::none()
        }
        ProcessesMsg::SelectPid(pid) => {
            state.selected_pid = Some(pid);
            Task::none()
        }
        ProcessesMsg::SendToInject(_) => Task::none(), // handled in App
    }
}

fn load_processes() -> Task<ProcessesMsg> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(|| {
                mono_injector_core::process::list_processes(&ListOptions {
                    filter: None,
                    module_filter: ModuleFilter::MonoAndUnity,
                    include_modules: true,
                })
            })
            .await
            .unwrap_or_default()
        },
        ProcessesMsg::Loaded,
    )
}

fn runtime_type(p: &ProcessListing) -> Option<RuntimeFilter> {
    let has_unity = p
        .matched_modules
        .iter()
        .any(|m| m.to_lowercase().contains("unity"));
    let has_mono = p
        .matched_modules
        .iter()
        .any(|m| m.to_lowercase().contains("mono"));
    match (has_unity, has_mono) {
        (true, _) => Some(RuntimeFilter::Unity),
        (false, true) => Some(RuntimeFilter::Mono),
        _ => None,
    }
}

fn matches_runtime_filter(p: &ProcessListing, filter: RuntimeFilter) -> bool {
    match filter {
        RuntimeFilter::All => true,
        other => runtime_type(p) == Some(other),
    }
}

pub fn view(state: &ProcessesState) -> Element<'_, ProcessesMsg> {
    container(
        column![
            page_header::view(
                "Running Processes",
                "Find Mono or Unity runtime processes and send a target to injection.",
            ),
            toolbar(state),
            process_table(state)
        ]
        .spacing(SP4)
        .height(Length::Fill),
    )
    .center_x(Length::Fill)
    .padding(SP4)
    .height(Length::Fill)
    .into()
}

fn toolbar(state: &ProcessesState) -> Element<'_, ProcessesMsg> {
    container(
        row![
            text_input("Filter processes...", &state.filter_text)
                .on_input(ProcessesMsg::FilterChanged)
                .width(240)
                .style(theme::input_style),
            runtime_filter_group(state.runtime_filter),
            iced::widget::Space::new().width(Length::Fill),
            toggler(state.show_modules)
                .label("Show modules")
                .on_toggle(ProcessesMsg::ToggleModules),
            button(icon::icon(icon::REFRESH, 18.0, FG2))
                .on_press(ProcessesMsg::Refresh)
                .style(theme::ghost_button_style),
        ]
        .spacing(SP2)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(SP2)
    .width(Length::Fill)
    .style(|_| theme::elevated_panel_style())
    .into()
}

fn runtime_filter_group(active: RuntimeFilter) -> Element<'static, ProcessesMsg> {
    let btn = |label: &'static str, filter: RuntimeFilter| -> Element<'static, ProcessesMsg> {
        let is_active = active == filter;
        button(
            text(label)
                .size(12)
                .color(if is_active { PRIMARY } else { FG2 }),
        )
        .on_press(ProcessesMsg::RuntimeFilterChanged(filter))
        .style(move |_, status| {
            let bg = if is_active {
                theme::BG_HIGHEST
            } else {
                match status {
                    button::Status::Hovered | button::Status::Pressed => BG_HIGH,
                    _ => theme::BG_HARD,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if is_active { PRIMARY } else { FG2 },
                border: iced::Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            }
        })
        .padding([4, 8])
        .into()
    };

    row![
        btn("ALL", RuntimeFilter::All),
        btn("MONO", RuntimeFilter::Mono),
        btn("UNITY", RuntimeFilter::Unity)
    ]
    .spacing(1)
    .into()
}

fn process_table(state: &ProcessesState) -> Element<'_, ProcessesMsg> {
    container(process_rows(state))
        .height(Length::Fill)
        .style(|_| theme::panel_style())
        .into()
}

fn process_rows(state: &ProcessesState) -> Element<'static, ProcessesMsg> {
    let rows = filtered_rows(state);
    if rows.is_empty() {
        return empty_process_rows();
    }

    scrollable(process_table_widget(rows, state.show_modules))
        .height(Length::Fill)
        .style(theme::table_scrollable_style)
        .into()
}

fn empty_process_rows() -> Element<'static, ProcessesMsg> {
    container(text("No matching processes").size(13).color(FG4))
        .padding(SP4)
        .into()
}

fn filtered_rows(state: &ProcessesState) -> Vec<ProcessTableRow> {
    let filter = state.filter_text.to_lowercase();
    state
        .all_processes
        .iter()
        .filter(|p| matches_runtime_filter(p, state.runtime_filter))
        .filter(|p| filter.is_empty() || p.name.to_lowercase().contains(&filter))
        .enumerate()
        .map(|(i, p)| ProcessTableRow {
            process: p.clone(),
            odd: i % 2 == 1,
            selected: state.selected_pid == Some(p.pid),
        })
        .collect()
}

fn process_table_widget(
    rows: Vec<ProcessTableRow>,
    show_modules: bool,
) -> Element<'static, ProcessesMsg> {
    let mut columns = vec![
        pid_column(),
        name_column(show_modules),
        runtime_column(show_modules),
    ];
    if show_modules {
        columns.push(modules_column());
    }
    columns.push(action_column());

    iced_table::table(columns, rows)
        .width(Length::Fill)
        .padding(0)
        .separator(1)
        .into()
}

fn pid_column() -> iced_table::Column<'static, 'static, ProcessTableRow, ProcessesMsg> {
    iced_table::column(header_label("PID"), |row: ProcessTableRow| {
        selectable_cell(pid_label(row.process.pid), &row)
    })
    .width(Length::FillPortion(2))
}

fn name_column(
    show_modules: bool,
) -> iced_table::Column<'static, 'static, ProcessTableRow, ProcessesMsg> {
    let flex = if show_modules { 4 } else { 5 };
    iced_table::column(header_label("NAME"), |row: ProcessTableRow| {
        selectable_cell(process_name_label(row.process.name.clone()), &row)
    })
    .width(Length::FillPortion(flex))
}

fn runtime_column(
    show_modules: bool,
) -> iced_table::Column<'static, 'static, ProcessTableRow, ProcessesMsg> {
    let flex = if show_modules { 2 } else { 3 };
    iced_table::column(header_label("RUNTIME"), |row: ProcessTableRow| {
        selectable_cell(runtime_cell(&row.process), &row)
    })
    .width(Length::FillPortion(flex))
}

fn modules_column() -> iced_table::Column<'static, 'static, ProcessTableRow, ProcessesMsg> {
    iced_table::column(header_label("MODULES"), |row: ProcessTableRow| {
        selectable_cell(modules_label(&row.process), &row)
    })
    .width(Length::FillPortion(4))
}

fn action_column() -> iced_table::Column<'static, 'static, ProcessTableRow, ProcessesMsg> {
    iced_table::column(header_label(""), |row: ProcessTableRow| action_cell(&row))
        .width(SEND_BUTTON_SLOT_WIDTH + 24)
}

fn header_label(label: &'static str) -> Element<'static, ProcessesMsg> {
    container(text(label).size(10).font(FONT_MONO).color(FG2))
        .padding(SP2)
        .style(|_| theme::panel_header_style())
        .into()
}

fn selectable_cell(
    content: Element<'static, ProcessesMsg>,
    row: &ProcessTableRow,
) -> Element<'static, ProcessesMsg> {
    button(container(content).width(Length::Fill).padding(SP2))
        .on_press(ProcessesMsg::SelectPid(row.process.pid))
        .width(Length::Fill)
        .padding(0)
        .style(theme::table_row_button_style(row.bg(), row.selected))
        .into()
}

fn action_cell(row: &ProcessTableRow) -> Element<'static, ProcessesMsg> {
    let bg = row.bg();
    container(send_to_inject_button(&row.process, row.selected))
        .padding(SP2)
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        })
        .into()
}

fn pid_label(pid: u32) -> Element<'static, ProcessesMsg> {
    text(format!("0x{pid:X}"))
        .size(11)
        .font(FONT_MONO)
        .color(FG2)
        .into()
}

fn process_name_label(name: String) -> Element<'static, ProcessesMsg> {
    text(name).size(13).font(FONT_MONO).color(FG).into()
}

fn runtime_cell(p: &ProcessListing) -> Element<'static, ProcessesMsg> {
    runtime_type(p).map_or_else(|| text("").into(), |f| badge::runtime_badge(f.as_str()))
}

fn modules_label(p: &ProcessListing) -> Element<'static, ProcessesMsg> {
    text(p.matched_modules.join(", "))
        .size(11)
        .font(FONT_MONO)
        .color(FG4)
        .into()
}

fn send_to_inject_button(p: &ProcessListing, selected: bool) -> Element<'static, ProcessesMsg> {
    if !selected {
        return iced::widget::Space::new()
            .width(SEND_BUTTON_SLOT_WIDTH)
            .height(1)
            .into();
    }
    button(
        row![
            icon::icon(icon::MY_LOCATION, 14.0, theme::BG_HARD),
            text("SEND TO INJECT")
                .size(10)
                .font(FONT_MONO)
                .color(theme::BG_HARD),
        ]
        .spacing(4)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(ProcessesMsg::SendToInject(p.clone()))
    .padding([4.0, 10.0])
    .style(theme::send_to_inject_button_style)
    .into()
}

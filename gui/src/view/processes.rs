use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler};
use iced::{Element, Length, Task};
use mono_injector_core::process::{ListOptions, ModuleFilter, ProcessListing};

use crate::theme::{self, BG, BG_CONT, BG_HIGH, FG, FG2, FG4, FONT_MONO, PRIMARY, SP2, SP4};
use crate::widget::{badge, icon, page_header, table};

// Width of the hidden space placeholder that reserves room for the send button.
const SEND_BUTTON_SLOT_WIDTH: u16 = 132;

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
            iced::widget::horizontal_space(),
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
    container(column![table_header(state.show_modules), process_rows(state)].height(Length::Fill))
        .height(Length::Fill)
        .style(|_| theme::panel_style())
        .into()
}

fn table_header(show_modules: bool) -> Element<'static, ProcessesMsg> {
    let hd = |l, f| table::header_cell(table::header_label(l), f);
    if show_modules {
        row![
            hd("PID", 2),
            hd("NAME", 4),
            hd("RUNTIME", 2),
            hd("MODULES", 4)
        ]
        .into()
    } else {
        row![hd("PID", 2), hd("NAME", 5), hd("RUNTIME", 3)].into()
    }
}

fn process_rows(state: &ProcessesState) -> Element<'_, ProcessesMsg> {
    let filter = state.filter_text.to_lowercase();
    let body = state
        .all_processes
        .iter()
        .filter(|p| matches_runtime_filter(p, state.runtime_filter))
        .filter(|p| filter.is_empty() || p.name.to_lowercase().contains(&filter))
        .enumerate()
        .map(|(i, p)| {
            process_row(
                p,
                i % 2 == 1,
                state.selected_pid == Some(p.pid),
                state.show_modules,
            )
        })
        .collect::<Vec<_>>();

    let inner = if body.is_empty() {
        column![text("No matching processes").size(13).color(FG4)]
    } else {
        column(body).spacing(0)
    };

    scrollable(inner)
        .height(Length::Fill)
        .style(theme::table_scrollable_style)
        .into()
}

fn process_row(
    p: &ProcessListing,
    odd: bool,
    selected: bool,
    show_modules: bool,
) -> Element<'_, ProcessesMsg> {
    let bg = if selected {
        BG_HIGH
    } else if odd {
        BG_CONT
    } else {
        BG
    };
    let rt = runtime_type(p);
    let runtime_el: Element<_> =
        rt.map_or_else(|| text("").into(), |f| badge::runtime_badge(f.as_str()));

    let send_btn = send_to_inject_button(p, selected);

    let pid_cell = || {
        table::data_cell(
            text(format!("0x{:X}", p.pid))
                .size(11)
                .font(FONT_MONO)
                .color(FG2),
            2,
            odd,
        )
    };
    let name_cell = |flex| {
        table::data_cell(
            text(p.name.clone()).size(13).font(FONT_MONO).color(FG),
            flex,
            odd,
        )
    };
    let row_content: Element<_> = if show_modules {
        row![
            pid_cell(),
            name_cell(4),
            table::data_cell(runtime_el, 2, odd),
            table::data_cell(
                text(p.matched_modules.join(", "))
                    .size(11)
                    .font(FONT_MONO)
                    .color(FG4),
                4,
                odd
            ),
        ]
        .into()
    } else {
        row![
            pid_cell(),
            name_cell(5),
            table::data_cell(runtime_el, 3, odd)
        ]
        .into()
    };

    button(row![row_content, iced::widget::horizontal_space(), send_btn].spacing(SP2))
        .on_press(ProcessesMsg::SelectPid(p.pid))
        .width(Length::Fill)
        .padding(0)
        .style(theme::table_row_button_style(bg, selected))
        .into()
}

fn send_to_inject_button(p: &ProcessListing, selected: bool) -> Element<'_, ProcessesMsg> {
    if !selected {
        return iced::widget::Space::new(SEND_BUTTON_SLOT_WIDTH, 1).into();
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

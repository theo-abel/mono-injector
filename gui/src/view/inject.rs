use std::path::PathBuf;
use std::time::Duration;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task};
use mono_injector_core::operations::{InjectOptions, InjectOutput};
use mono_injector_core::process::{ListOptions, ModuleFilter, ProcessListing};
use mono_injector_core::profiles::Profile;
use mono_injector_core::runtime::RuntimeOptions;

use crate::theme::{
    self, BG_HIGH, FG, FG2, FG4, FONT_MONO, FONT_UI, FONT_UI_SEMIBOLD, ORANGE, PRIMARY_C, PURPLE,
    SP2, SP3, SP4, SP5, YELLOW,
};
use crate::util;
use crate::widget::{badge, collapsible, icon, page_header, toggle};

// Height of the scrollable process picker list.
const PROCESS_LIST_HEIGHT: u16 = 120;

/// Per-field state for the Inject panel.
#[derive(Debug, Clone)]
pub struct InjectState {
    pub process_input: String,
    pub process_filter: String,
    pub process_list: Vec<ProcessListing>,
    pub assembly_path: String,
    pub namespace_input: String,
    pub class_input: String,
    pub method_input: String,
    pub eject_method_input: String,
    pub profile_selection: Option<String>,
    pub profiles: Vec<String>,
    pub wait_for_process: bool,
    pub wait_timeout: String,
    pub poll_interval: String,
    pub wait_for_module: bool,
    pub wait_module_name: String,
    pub settle_ms: String,
    pub steam_enabled: bool,
    pub steam_app_id: String,
    pub timeout_ms: String,
    pub mono_module: String,
    pub base_dir: String,
    pub dry_run: bool,
    pub entry_point_expanded: bool,
    pub timing_expanded: bool,
    pub runtime_expanded: bool,
    pub profile_expanded: bool,
    pub inject_running: bool,
    pub last_error: Option<String>,
}

impl Default for InjectState {
    fn default() -> Self {
        Self {
            process_input: String::new(),
            process_filter: String::new(),
            process_list: Vec::new(),
            assembly_path: String::new(),
            namespace_input: String::new(),
            class_input: "Loader".to_owned(),
            method_input: "Init".to_owned(),
            eject_method_input: "Unload".to_owned(),
            profile_selection: None,
            profiles: Vec::new(),
            wait_for_process: false,
            wait_timeout: "120s".to_owned(),
            poll_interval: "1000ms".to_owned(),
            wait_for_module: false,
            wait_module_name: String::new(),
            settle_ms: "0".to_owned(),
            steam_enabled: false,
            steam_app_id: String::new(),
            timeout_ms: "5000".to_owned(),
            mono_module: "mono".to_owned(),
            base_dir: String::new(),
            dry_run: false,
            entry_point_expanded: false,
            timing_expanded: true,
            runtime_expanded: false,
            profile_expanded: false,
            inject_running: false,
            last_error: None,
        }
    }
}

/// Messages handled by the Inject view.
#[derive(Debug, Clone)]
pub enum InjectMsg {
    ProcessFilterChanged(String),
    ProcessSelected(String),
    AssemblyPathChanged(String),
    NamespaceChanged(String),
    ClassChanged(String),
    MethodChanged(String),
    EjectMethodChanged(String),
    WaitForProcessToggled(bool),
    WaitTimeoutChanged(String),
    PollIntervalChanged(String),
    WaitForModuleToggled(bool),
    WaitModuleChanged(String),
    SettleChanged(String),
    SteamToggled(bool),
    SteamAppIdChanged(String),
    TimeoutChanged(String),
    MonoModuleChanged(String),
    BaseDirChanged(String),
    DryRunToggled(bool),
    EntryPointToggled,
    TimingToggled,
    RuntimeToggled,
    ProfileToggled,
    RefreshProcesses,
    ProcessesLoaded(Vec<ProcessListing>),
    BrowseAssembly,
    BrowseResult(Option<PathBuf>),
    InjectClicked,
    DryRunClicked,
    InjectDone(Result<InjectOutput, String>),
    ProfileSelected(String),
    ProfileLoaded(Result<Profile, String>),
    ProfilesLoaded(Vec<String>),
}

impl InjectMsg {
    /// Returns a log entry for result messages so the app can record it.
    pub fn log_entry(&self) -> Option<crate::widget::log_strip::LogEntry> {
        use crate::widget::log_strip::LogEntry;
        match self {
            Self::InjectDone(Ok(o)) => Some(LogEntry::ok(format!(
                "Injected {} into {}",
                o.entry, o.process.name
            ))),
            Self::InjectDone(Err(e)) => Some(LogEntry::error(format!("Inject failed: {e}"))),
            _ => None,
        }
    }
}

pub fn update(state: &mut InjectState, msg: InjectMsg) -> Task<InjectMsg> {
    match msg {
        InjectMsg::BrowseAssembly => browse_assembly(),
        InjectMsg::BrowseResult(p) => {
            apply_browse(state, p);
            Task::none()
        }
        InjectMsg::RefreshProcesses => refresh_processes(),
        InjectMsg::ProcessesLoaded(list) => {
            state.process_list = list;
            Task::none()
        }
        InjectMsg::InjectClicked => perform_inject(state, false),
        InjectMsg::DryRunClicked => perform_inject(state, true),
        InjectMsg::InjectDone(r) => {
            handle_inject_done(state, r);
            Task::none()
        }
        InjectMsg::ProfileSelected(name) => load_profile(name),
        InjectMsg::ProfileLoaded(r) => {
            apply_profile(state, r);
            Task::none()
        }
        InjectMsg::ProfilesLoaded(names) => {
            state.profiles = names;
            Task::none()
        }
        other => {
            apply_field(state, other);
            Task::none()
        }
    }
}

fn apply_field(s: &mut InjectState, msg: InjectMsg) {
    match msg {
        InjectMsg::ProcessFilterChanged(v) => s.process_filter = v,
        InjectMsg::ProcessSelected(v) => s.process_input = v,
        InjectMsg::AssemblyPathChanged(v) => s.assembly_path = v,
        InjectMsg::NamespaceChanged(v) => s.namespace_input = v,
        InjectMsg::ClassChanged(v) => s.class_input = v,
        InjectMsg::MethodChanged(v) => s.method_input = v,
        InjectMsg::EjectMethodChanged(v) => s.eject_method_input = v,
        InjectMsg::WaitForProcessToggled(v) => s.wait_for_process = v,
        InjectMsg::WaitTimeoutChanged(v) => s.wait_timeout = v,
        InjectMsg::PollIntervalChanged(v) => s.poll_interval = v,
        InjectMsg::WaitForModuleToggled(v) => s.wait_for_module = v,
        InjectMsg::WaitModuleChanged(v) => s.wait_module_name = v,
        InjectMsg::SettleChanged(v) => s.settle_ms = v,
        InjectMsg::SteamToggled(v) => s.steam_enabled = v,
        InjectMsg::SteamAppIdChanged(v) => s.steam_app_id = v,
        InjectMsg::TimeoutChanged(v) => s.timeout_ms = v,
        InjectMsg::MonoModuleChanged(v) => s.mono_module = v,
        InjectMsg::BaseDirChanged(v) => s.base_dir = v,
        InjectMsg::DryRunToggled(v) => s.dry_run = v,
        InjectMsg::EntryPointToggled => s.entry_point_expanded = !s.entry_point_expanded,
        InjectMsg::TimingToggled => s.timing_expanded = !s.timing_expanded,
        InjectMsg::RuntimeToggled => s.runtime_expanded = !s.runtime_expanded,
        InjectMsg::ProfileToggled => s.profile_expanded = !s.profile_expanded,
        _ => {}
    }
}

fn handle_inject_done(state: &mut InjectState, result: Result<InjectOutput, String>) {
    state.inject_running = false;
    state.last_error = result.err();
}

fn apply_browse(state: &mut InjectState, path: Option<PathBuf>) {
    if let Some(p) = path {
        state.assembly_path = p.to_string_lossy().into_owned();
    }
}

fn apply_profile(state: &mut InjectState, result: Result<Profile, String>) {
    let Ok(p) = result else { return };
    if let Some(v) = p.process {
        state.process_input = v;
    }
    if let Some(v) = p.assembly {
        state.assembly_path = v.to_string_lossy().into_owned();
    }
    if let Some(v) = p.namespace {
        state.namespace_input = v;
    }
    if let Some(v) = p.class_name {
        state.class_input = v;
    }
    if let Some(v) = p.inject_method {
        state.method_input = v;
    }
    if let Some(v) = p.eject_method {
        state.eject_method_input = v;
    }
    if let Some(v) = p.mono_module {
        state.mono_module = v;
    }
    if let Some(v) = p.base_dir {
        state.base_dir = v;
    }
    if let Some(v) = p.timeout_ms {
        state.timeout_ms = v.to_string();
    }
    if let Some(v) = p.steam_app {
        state.steam_app_id = v.to_string();
    }
}

fn browse_assembly() -> Task<InjectMsg> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(|| {
                rfd::FileDialog::new()
                    .add_filter(".NET Assembly", &["dll"])
                    .pick_file()
            })
            .await
            .ok()
            .flatten()
        },
        InjectMsg::BrowseResult,
    )
}

fn refresh_processes() -> Task<InjectMsg> {
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
        InjectMsg::ProcessesLoaded,
    )
}

fn load_profile(name: String) -> Task<InjectMsg> {
    Task::perform(
        util::run_blocking(move || {
            mono_injector_core::profiles::get_profile(&name).map_err(|e| e.to_string())
        }),
        InjectMsg::ProfileLoaded,
    )
}

fn build_options(state: &InjectState) -> InjectOptions {
    InjectOptions {
        profile_name: state.profile_selection.clone(),
        process: util::non_empty(&state.process_input),
        assembly: util::non_empty(&state.assembly_path).map(PathBuf::from),
        namespace: util::non_empty(&state.namespace_input),
        class_name: util::non_empty(&state.class_input),
        inject_method: util::non_empty(&state.method_input),
        eject_method: util::non_empty(&state.eject_method_input),
        wait_for_process: state.wait_for_process,
        wait_timeout: parse_dur(&state.wait_timeout, Duration::from_mins(2)),
        poll_interval: parse_dur(&state.poll_interval, Duration::from_secs(1)),
        wait_module: state
            .wait_for_module
            .then(|| util::non_empty(&state.wait_module_name))
            .flatten(),
        disable_wait_module: false,
        settle_delay: state
            .settle_ms
            .parse::<u64>()
            .ok()
            .filter(|&ms| ms > 0)
            .map(Duration::from_millis),
        steam_app: state
            .steam_enabled
            .then(|| state.steam_app_id.parse::<u32>().ok())
            .flatten(),
        runtime: RuntimeOptions {
            timeout_ms: state.timeout_ms.parse().unwrap_or(5000),
            mono_module_hint: util::non_empty(&state.mono_module),
            base_dir: util::non_empty(&state.base_dir),
        },
    }
}

fn perform_inject(state: &mut InjectState, dry_run: bool) -> Task<InjectMsg> {
    state.inject_running = true;
    state.last_error = None;
    let opts = build_options(state);
    Task::perform(
        util::run_blocking(move || {
            if dry_run {
                mono_injector_core::operations::resolve_inject(&opts)
                    .map(mono_injector_core::operations::ResolvedInjectPlan::dry_run_output)
                    .map_err(|e| e.to_string())
            } else {
                mono_injector_core::operations::inject(&opts).map_err(|e| e.to_string())
            }
        }),
        InjectMsg::InjectDone,
    )
}

fn parse_dur(s: &str, default: Duration) -> Duration {
    humantime::parse_duration(s).unwrap_or(default)
}

pub fn view(state: &InjectState) -> Element<'_, InjectMsg> {
    container(
        column![
            page_header::view(
                "Inject Assembly",
                "Configure target process and assembly payload for injection.",
            ),
            form_grid(state)
        ]
        .spacing(SP5)
        .max_width(1120)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(SP4)
    .center_x(Length::Fill)
    .into()
}

fn form_grid(state: &InjectState) -> Element<'_, InjectMsg> {
    row![left_column(state), right_column(state)]
        .spacing(SP5)
        .width(Length::Fill)
        .into()
}

fn left_column(state: &InjectState) -> Element<'_, InjectMsg> {
    column![
        target_process_panel(state),
        assembly_payload_panel(state),
        entry_point_section(state),
    ]
    .spacing(SP4)
    .width(Length::FillPortion(7))
    .into()
}

fn right_column(state: &InjectState) -> Element<'_, InjectMsg> {
    column![
        timing_section(state),
        runtime_section(state),
        action_buttons(state),
    ]
    .spacing(SP4)
    .width(Length::FillPortion(5))
    .into()
}

fn target_process_panel(state: &InjectState) -> Element<'_, InjectMsg> {
    let search_row = row![
        text_input("Process name or PID...", &state.process_filter)
            .on_input(InjectMsg::ProcessFilterChanged)
            .padding([7.0, SP2])
            .style(theme::input_style),
        button(icon::icon(icon::REFRESH, 18.0, FG2))
            .on_press(InjectMsg::RefreshProcesses)
            .padding([7.0, SP3])
            .style(theme::ghost_button_style),
    ]
    .spacing(SP2)
    .align_y(iced::alignment::Vertical::Center);

    let list = process_picker(state);
    container(
        column![
            text("TARGET PROCESS").size(10).font(FONT_MONO).color(FG2),
            search_row,
            list,
        ]
        .spacing(SP2),
    )
    .padding(SP3)
    .width(Length::Fill)
    .style(|_| theme::panel_style())
    .into()
}

fn process_picker(state: &InjectState) -> Element<'_, InjectMsg> {
    let filter = state.process_filter.to_lowercase();
    let filtered: Vec<_> = state
        .process_list
        .iter()
        .filter(|p| filter.is_empty() || p.name.to_lowercase().contains(&filter))
        .collect();

    let rows = filtered.iter().map(|p| process_row(p)).collect::<Vec<_>>();
    let inner = if rows.is_empty() {
        column![text("No matching processes").size(12).color(FG4)]
    } else {
        column(rows).spacing(1)
    };

    container(scrollable(inner.padding([SP2, 0.0])).height(PROCESS_LIST_HEIGHT))
        .width(Length::Fill)
        .style(|_| theme::recessed_style())
        .into()
}

fn process_row(p: &ProcessListing) -> Element<'_, InjectMsg> {
    let label = util::format_process_label(&p.name, p.pid);
    button(
        row![
            runtime_dot(p),
            text(p.name.as_str()).size(12).font(FONT_MONO).color(FG),
            iced::widget::horizontal_space(),
            badge::badge(format!("PID {}", p.pid), BG_HIGH, FG2, theme::BORDER),
        ]
        .spacing(SP2),
    )
    .on_press(InjectMsg::ProcessSelected(label))
    .width(Length::Fill)
    .padding([2.0, SP2])
    .style(theme::process_list_row_button_style)
    .into()
}

fn runtime_dot(p: &ProcessListing) -> Element<'_, InjectMsg> {
    let color = if p
        .matched_modules
        .iter()
        .any(|m| m.to_lowercase().contains("unity"))
    {
        PURPLE
    } else if p
        .matched_modules
        .iter()
        .any(|m| m.to_lowercase().contains("mono"))
    {
        PRIMARY_C
    } else {
        FG4
    };
    container(iced::widget::Space::new(8, 8))
        .style(theme::dot_style(color))
        .into()
}

fn assembly_payload_panel(state: &InjectState) -> Element<'_, InjectMsg> {
    let path_row = row![
        icon::icon(icon::FOLDER, 20.0, FG2),
        text_input("Path to .dll assembly...", &state.assembly_path)
            .on_input(InjectMsg::AssemblyPathChanged)
            .padding([7.0, SP2])
            .style(theme::input_style),
        button(text("Browse").size(12).font(FONT_UI))
            .on_press(InjectMsg::BrowseAssembly)
            .padding([7.0, SP3])
            .style(theme::ghost_button_style),
    ]
    .spacing(SP2)
    .align_y(iced::alignment::Vertical::Center);

    let hint = if state.assembly_path.is_empty() {
        column![]
    } else {
        let name = PathBuf::from(&state.assembly_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        column![text(name).size(12).color(FG4)]
    };

    container(
        column![
            text("ASSEMBLY PAYLOAD").size(10).font(FONT_MONO).color(FG2),
            path_row,
            hint,
        ]
        .spacing(SP2),
    )
    .padding(SP3)
    .width(Length::Fill)
    .style(|_| theme::panel_style())
    .into()
}

fn entry_point_section(state: &InjectState) -> Element<'_, InjectMsg> {
    let body = entry_point_body(state);
    collapsible::collapsible(
        "Advanced Entry Point",
        body,
        state.entry_point_expanded,
        InjectMsg::EntryPointToggled,
    )
}

fn entry_point_body(state: &InjectState) -> Element<'_, InjectMsg> {
    column![
        labeled_input(
            "Namespace",
            &state.namespace_input,
            InjectMsg::NamespaceChanged
        ),
        labeled_input("Class", &state.class_input, InjectMsg::ClassChanged),
        labeled_input("Init Method", &state.method_input, InjectMsg::MethodChanged),
        labeled_input(
            "Unload Method",
            &state.eject_method_input,
            InjectMsg::EjectMethodChanged
        ),
    ]
    .spacing(SP2)
    .padding(SP3)
    .into()
}

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> InjectMsg + 'a,
) -> Element<'a, InjectMsg> {
    column![
        text(label).size(11).font(FONT_MONO).color(FG2),
        text_input("", value)
            .on_input(on_input)
            .style(theme::mono_input_style),
    ]
    .spacing(SP2)
    .into()
}

fn timing_section(state: &InjectState) -> Element<'_, InjectMsg> {
    let body = timing_body(state);
    collapsible::collapsible(
        "Timing & Launch",
        body,
        state.timing_expanded,
        InjectMsg::TimingToggled,
    )
}

fn timing_body(state: &InjectState) -> Element<'_, InjectMsg> {
    let mut col = column![
        toggle::toggle(
            "Wait for process",
            state.wait_for_process,
            InjectMsg::WaitForProcessToggled,
            None
        ),
        toggle::toggle(
            "Wait for module",
            state.wait_for_module,
            InjectMsg::WaitForModuleToggled,
            None
        ),
        labeled_input(
            "Settle delay (ms)",
            &state.settle_ms,
            InjectMsg::SettleChanged
        ),
        steam_row(state),
    ]
    .spacing(SP2)
    .padding(SP3);

    if state.wait_for_module {
        col = col.push(labeled_input(
            "Module name",
            &state.wait_module_name,
            InjectMsg::WaitModuleChanged,
        ));
    }

    col.into()
}

fn steam_row(state: &InjectState) -> Element<'_, InjectMsg> {
    let tog = toggle::toggle(
        "Steam App Launch",
        state.steam_enabled,
        InjectMsg::SteamToggled,
        Some(ORANGE),
    );
    if state.steam_enabled {
        column![
            tog,
            text_input("App ID", &state.steam_app_id)
                .on_input(InjectMsg::SteamAppIdChanged)
                .style(theme::mono_input_style),
        ]
        .spacing(SP2)
        .into()
    } else {
        tog
    }
}

fn runtime_section(state: &InjectState) -> Element<'_, InjectMsg> {
    let body = runtime_body(state);
    collapsible::collapsible(
        "Runtime Options",
        body,
        state.runtime_expanded,
        InjectMsg::RuntimeToggled,
    )
}

fn runtime_body(state: &InjectState) -> Element<'_, InjectMsg> {
    column![
        labeled_input("Timeout (ms)", &state.timeout_ms, InjectMsg::TimeoutChanged),
        labeled_input(
            "Mono Module Hint",
            &state.mono_module,
            InjectMsg::MonoModuleChanged
        ),
        labeled_input(
            "Base Directory Override",
            &state.base_dir,
            InjectMsg::BaseDirChanged
        ),
    ]
    .spacing(SP2)
    .padding(SP3)
    .into()
}

fn action_buttons(state: &InjectState) -> Element<'_, InjectMsg> {
    let inject_label = if state.inject_running {
        "INJECTING..."
    } else {
        "INJECT"
    };
    let inject_btn = button(centered_button_label(
        text(inject_label)
            .size(16)
            .font(FONT_UI_SEMIBOLD)
            .color(theme::BG_HARD),
    ))
    .width(Length::Fill)
    .padding(SP3)
    .style(theme::inject_button_style);

    let inject_btn = if state.inject_running {
        inject_btn
    } else {
        inject_btn.on_press(InjectMsg::InjectClicked)
    };

    let dry_btn = button(
        row![
            iced::widget::horizontal_space(),
            icon::icon(icon::SCIENCE, 18.0, YELLOW),
            text("Dry Run").size(13).font(FONT_UI).color(YELLOW),
            iced::widget::horizontal_space(),
        ]
        .spacing(SP2)
        .width(Length::Fill)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(InjectMsg::DryRunClicked)
    .width(Length::Fill)
    .padding(SP2)
    .style(theme::dry_run_button_style);

    let mut col = column![inject_btn, dry_btn].spacing(SP2);

    if let Some(ref err) = state.last_error {
        col = col.push(text(err.as_str()).size(12).color(theme::RED));
    }

    col.into()
}

fn centered_button_label(label: iced::widget::Text<'_>) -> Element<'_, InjectMsg> {
    row![
        iced::widget::horizontal_space(),
        label,
        iced::widget::horizontal_space()
    ]
    .width(Length::Fill)
    .into()
}

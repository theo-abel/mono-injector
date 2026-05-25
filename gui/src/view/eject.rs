use iced::widget::{button, column, container, row, text, text_input};
use iced::{Background, Border, Element, Length, Task};
use mono_injector_core::operations::{EjectOptions, EjectOutput};
use mono_injector_core::runtime::RuntimeOptions;
use mono_injector_core::state::InjectionRecord;

use crate::theme::{
    self, FG, FG2, FG4, FONT_MONO, FONT_UI_SEMIBOLD, PRIMARY_C, PURPLE, RED, SP2, SP3, SP4,
};
use crate::widget::{badge, icon, toggle};

/// Per-field state for the Eject panel.
#[derive(Debug, Clone)]
pub struct EjectState {
    pub process_input: String,
    pub handle_input: String,
    pub namespace_input: String,
    pub class_input: String,
    pub method_input: String,
    pub active_records: Vec<InjectionRecord>,
    pub resolved_record: Option<InjectionRecord>,
    pub force_enabled: bool,
    pub raw_handle_input: String,
    pub latest_enabled: bool,
    pub danger_expanded: bool,
    pub eject_running: bool,
    pub last_error: Option<String>,
}

impl Default for EjectState {
    fn default() -> Self {
        Self {
            process_input: String::new(),
            handle_input: String::new(),
            namespace_input: String::new(),
            class_input: String::new(),
            method_input: "Unload".to_owned(),
            active_records: Vec::new(),
            resolved_record: None,
            force_enabled: false,
            raw_handle_input: String::new(),
            latest_enabled: true,
            danger_expanded: true,
            eject_running: false,
            last_error: None,
        }
    }
}

/// Messages handled by the Eject view.
#[derive(Debug, Clone)]
pub enum EjectMsg {
    ProcessChanged(String),
    HandleChanged(String),
    NamespaceChanged(String),
    ClassChanged(String),
    MethodChanged(String),
    RawHandleChanged(String),
    ForceToggled(bool),
    LatestToggled(bool),
    DangerToggled,
    PickRecord(InjectionRecord),
    LoadRecords,
    RecordsLoaded(Result<Vec<InjectionRecord>, String>),
    EjectClicked,
    EjectDone(Result<EjectOutput, String>),
}

impl EjectMsg {
    pub fn log_entry(&self) -> Option<crate::widget::log_strip::LogEntry> {
        use crate::widget::log_strip::LogEntry;
        match self {
            Self::EjectDone(Ok(o)) => Some(LogEntry::ok(format!(
                "Ejected {} from {}",
                o.entry, o.process.name
            ))),
            Self::EjectDone(Err(e)) => Some(LogEntry::error(format!("Eject failed: {e}"))),
            _ => None,
        }
    }
}

pub fn update(state: &mut EjectState, msg: EjectMsg) -> Task<EjectMsg> {
    match msg {
        EjectMsg::ProcessChanged(v) => {
            state.process_input = v;
            Task::none()
        }
        EjectMsg::HandleChanged(v) => {
            resolve_handle(state, &v);
            state.handle_input = v;
            Task::none()
        }
        EjectMsg::NamespaceChanged(v) => {
            state.namespace_input = v;
            Task::none()
        }
        EjectMsg::ClassChanged(v) => {
            state.class_input = v;
            Task::none()
        }
        EjectMsg::MethodChanged(v) => {
            state.method_input = v;
            Task::none()
        }
        EjectMsg::RawHandleChanged(v) => {
            state.raw_handle_input = v;
            Task::none()
        }
        EjectMsg::ForceToggled(v) => {
            state.force_enabled = v;
            Task::none()
        }
        EjectMsg::LatestToggled(v) => {
            state.latest_enabled = v;
            Task::none()
        }
        EjectMsg::DangerToggled => {
            state.danger_expanded = !state.danger_expanded;
            Task::none()
        }
        EjectMsg::PickRecord(r) => {
            pick_record(state, r);
            Task::none()
        }
        EjectMsg::LoadRecords => load_records(),
        EjectMsg::RecordsLoaded(r) => {
            apply_records(state, r);
            Task::none()
        }
        EjectMsg::EjectClicked => perform_eject(state),
        EjectMsg::EjectDone(r) => {
            handle_done(state, r);
            Task::none()
        }
    }
}

fn resolve_handle(state: &mut EjectState, handle: &str) {
    state.resolved_record = state
        .active_records
        .iter()
        .find(|r| r.handle == handle)
        .cloned();
}

fn pick_record(state: &mut EjectState, r: InjectionRecord) {
    state.handle_input.clone_from(&r.handle);
    state.namespace_input.clone_from(&r.namespace);
    state.class_input.clone_from(&r.class_name);
    state.method_input.clone_from(&r.eject_method);
    state.resolved_record = Some(r);
}

fn apply_records(state: &mut EjectState, result: Result<Vec<InjectionRecord>, String>) {
    match result {
        Ok(recs) => state.active_records = recs,
        Err(_) => state.active_records.clear(),
    }
}

fn handle_done(state: &mut EjectState, result: Result<EjectOutput, String>) {
    state.eject_running = false;
    state.last_error = result.err();
    if state.last_error.is_none() {
        state.handle_input.clear();
        state.resolved_record = None;
    }
}

fn load_records() -> Task<EjectMsg> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(|| {
                mono_injector_core::state::all().map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r)
        },
        EjectMsg::RecordsLoaded,
    )
}

fn perform_eject(state: &mut EjectState) -> Task<EjectMsg> {
    state.eject_running = true;
    state.last_error = None;
    let opts = build_eject_options(state);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                mono_injector_core::operations::eject(&opts).map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r)
        },
        EjectMsg::EjectDone,
    )
}

fn build_eject_options(state: &EjectState) -> EjectOptions {
    let handle = non_empty(&state.handle_input);
    let raw = state
        .force_enabled
        .then(|| non_empty(&state.raw_handle_input))
        .flatten();
    EjectOptions {
        profile_name: None,
        process: non_empty(&state.process_input),
        handle,
        raw_handle: raw,
        namespace: non_empty(&state.namespace_input),
        class_name: non_empty(&state.class_input),
        method_name: non_empty(&state.method_input),
        latest: state.latest_enabled,
        force: state.force_enabled,
        runtime: RuntimeOptions::default(),
    }
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

// --- View ---

pub fn view(state: &EjectState) -> Element<'_, EjectMsg> {
    container(
        column![workspace_header(), eject_form(state)]
            .spacing(SP4)
            .max_width(960),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(SP4)
    .center_x(Length::Fill)
    .into()
}

fn workspace_header<'a>() -> Element<'a, EjectMsg> {
    column![
        text("Eject Assembly")
            .size(18)
            .font(FONT_UI_SEMIBOLD)
            .color(FG),
        text("Forcefully unload a .NET assembly from the target process space.")
            .size(13)
            .color(FG2),
    ]
    .spacing(4)
    .into()
}

fn eject_form(state: &EjectState) -> Element<'_, EjectMsg> {
    column![
        process_context_strip(state),
        handle_section(state),
        danger_section(state),
        eject_action_row(state)
    ]
    .spacing(SP4)
    .into()
}

fn process_context_strip(state: &EjectState) -> Element<'_, EjectMsg> {
    let name = if state.process_input.is_empty() {
        "—"
    } else {
        state.process_input.as_str()
    };
    container(
        row![
            icon::icon(icon::MEMORY, 20.0, crate::theme::GREEN),
            text("ATTACHED PROCESS").size(10).font(FONT_MONO).color(FG4),
            text(name).size(13).font(FONT_MONO).color(FG),
            iced::widget::horizontal_space(),
            text_input("Process name or PID...", &state.process_input)
                .on_input(EjectMsg::ProcessChanged)
                .style(theme::input_style),
        ]
        .spacing(SP2),
    )
    .padding(SP3)
    .width(Length::Fill)
    .style(|_| theme::elevated_panel_style())
    .into()
}

fn handle_section(state: &EjectState) -> Element<'_, EjectMsg> {
    let picker_rows = state
        .active_records
        .iter()
        .map(|r| record_pick_row(r))
        .collect::<Vec<_>>();

    let mut col = column![
        row![
            text("ASSEMBLY HANDLE").size(10).font(FONT_MONO).color(FG2),
            iced::widget::horizontal_space(),
            text("REQUIRED").size(10).font(FONT_MONO).color(PURPLE),
        ],
        text_input("0x...", &state.handle_input)
            .on_input(EjectMsg::HandleChanged)
            .style(theme::purple_input_style),
        row![
            text("Select from loaded assemblies or enter raw pointer.")
                .size(11)
                .font(FONT_MONO)
                .color(FG4),
            iced::widget::horizontal_space(),
            button(text("Pick...").size(12))
                .on_press(EjectMsg::LoadRecords)
                .style(theme::ghost_button_style),
        ],
    ]
    .spacing(SP2);

    if !picker_rows.is_empty() {
        col = col.push(
            container(iced::widget::column(picker_rows).spacing(1))
                .style(|_| theme::recessed_style()),
        );
    }

    if let Some(ref rec) = state.resolved_record {
        col = col.push(resolved_record_card(rec));
    }

    container(col.padding(SP3))
        .width(Length::Fill)
        .style(|_| theme::panel_style())
        .into()
}

fn record_pick_row(r: &InjectionRecord) -> Element<'_, EjectMsg> {
    button(
        row![
            badge::handle_badge(r.handle.clone()),
            text(r.entry()).size(12).font(FONT_MONO).color(PRIMARY_C),
            iced::widget::horizontal_space(),
            text(relative_time(r.injected_at))
                .size(11)
                .font(FONT_MONO)
                .color(FG4),
        ]
        .spacing(SP2),
    )
    .on_press(EjectMsg::PickRecord(r.clone()))
    .width(Length::Fill)
    .padding([4.0, SP2])
    .style(theme::ghost_button_style)
    .into()
}

fn resolved_record_card(rec: &InjectionRecord) -> Element<'_, EjectMsg> {
    container(
        column![
            row![
                icon::icon(icon::CHECK_CIRCLE, 18.0, PRIMARY_C),
                text("Resolved Reference").size(12).color(FG)
            ]
            .spacing(SP2),
            row![
                text("Class::").size(11).font(FONT_MONO).color(FG4),
                text(rec.entry()).size(11).font(FONT_MONO).color(FG),
            ]
            .spacing(SP2),
            row![
                text("Method::").size(11).font(FONT_MONO).color(FG4),
                text(rec.eject_method.clone())
                    .size(11)
                    .font(FONT_MONO)
                    .color(crate::theme::GREEN),
            ]
            .spacing(SP2),
        ]
        .spacing(SP2),
    )
    .padding(SP2)
    .style(|_| theme::elevated_panel_style())
    .into()
}

fn danger_section(state: &EjectState) -> Element<'_, EjectMsg> {
    let card = if state.danger_expanded {
        column![danger_header(), danger_body(state)]
    } else {
        column![danger_header()]
    };
    container(card)
        .width(Length::Fill)
        .style(|_| theme::danger_section_style())
        .into()
}

fn danger_header<'a>() -> Element<'a, EjectMsg> {
    button(
        row![
            icon::icon(icon::WARNING, 22.0, RED),
            text("Danger Options")
                .size(14)
                .font(FONT_UI_SEMIBOLD)
                .color(crate::theme::RED_BRIGHT),
            iced::widget::horizontal_space(),
            icon::icon(icon::EXPAND_MORE, 20.0, RED),
        ]
        .spacing(SP2)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(EjectMsg::DangerToggled)
    .width(Length::Fill)
    .padding(SP3)
    .style(|_, _| button::Style {
        background: Some(Background::Color(crate::theme::RED_CONT.scale_alpha(0.12))),
        text_color: RED,
        border: Border {
            color: crate::theme::RED_CONT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn danger_body(state: &EjectState) -> Element<'_, EjectMsg> {
    column![
        danger_toggle(
            "Force Unload",
            "Bypass AppDomain locking mechanisms. May crash target.",
            state.force_enabled,
            EjectMsg::ForceToggled
        ),
        danger_toggle(
            "Unload Latest Only",
            "If multiple versions exist, only target the most recent handle.",
            state.latest_enabled,
            EjectMsg::LatestToggled
        ),
        text("RAW UNLOAD HANDLE OVERRIDE")
            .size(10)
            .font(FONT_MONO)
            .color(crate::theme::RED_BRIGHT),
        text_input("0x...", &state.raw_handle_input)
            .on_input(EjectMsg::RawHandleChanged)
            .style(theme::purple_input_style),
    ]
    .spacing(SP2)
    .padding(SP3)
    .into()
}

fn danger_toggle<'a>(
    title: &'a str,
    hint: &'a str,
    value: bool,
    on_toggle: impl Fn(bool) -> EjectMsg + 'a,
) -> Element<'a, EjectMsg> {
    row![
        container(toggle::toggle("", value, on_toggle, Some(RED))).width(Length::Fixed(44.0)),
        column![
            text(title).size(13).color(crate::theme::RED_BRIGHT),
            text(hint).size(11).font(FONT_MONO).color(FG2),
        ]
        .spacing(2),
    ]
    .spacing(SP2)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

fn eject_action_row(state: &EjectState) -> Element<'_, EjectMsg> {
    let label = if state.eject_running {
        "EJECTING..."
    } else {
        "EJECT ASSEMBLY"
    };
    let btn = button(text(label).size(15).font(FONT_UI_SEMIBOLD).color(FG))
        .padding(SP3)
        .style(theme::danger_button_style);

    let btn = if state.eject_running {
        btn
    } else {
        btn.on_press(EjectMsg::EjectClicked)
    };

    let mut col = column![row![iced::widget::horizontal_space(), btn].spacing(SP2)].spacing(SP2);
    if let Some(ref err) = state.last_error {
        col = col.push(text(err.as_str()).size(12).color(RED));
    }
    col.into()
}

fn relative_time(unix_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let elapsed = now.saturating_sub(unix_secs);
    if elapsed < 60 {
        format!("{elapsed}s ago")
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else {
        format!("{}h ago", elapsed / 3600)
    }
}

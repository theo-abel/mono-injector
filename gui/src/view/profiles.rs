use std::path::PathBuf;

use iced::widget::{button, column, container, row, rule, scrollable, text, text_input};
use iced::{Element, Length, Task};
use mono_injector_core::profiles::{Profile, ProfileSummary};

use crate::theme::{
    self, FG, FG2, FG4, FONT_MONO, FONT_UI, FONT_UI_SEMIBOLD, GREEN, PRIMARY, PRIMARY_C, SP1, SP2,
    SP3, SP4,
};
use crate::util;
use crate::widget::{collapsible, icon, page_header};

/// Editable string form mirroring `Profile` fields.
#[derive(Debug, Default, Clone)]
pub struct ProfileDraft {
    pub name: String,
    pub process: String,
    pub assembly: String,
    pub namespace: String,
    pub class_name: String,
    pub inject_method: String,
    pub eject_method: String,
    pub mono_module: String,
    pub base_dir: String,
    pub timeout_ms: String,
    pub wait_module: String,
    pub settle_ms: String,
    pub steam_app: String,
}

impl From<(&str, &Profile)> for ProfileDraft {
    fn from((name, p): (&str, &Profile)) -> Self {
        Self {
            name: name.to_owned(),
            process: p.process.clone().unwrap_or_default(),
            assembly: p
                .assembly
                .as_ref()
                .map(|v| v.to_string_lossy().into_owned())
                .unwrap_or_default(),
            namespace: p.namespace.clone().unwrap_or_default(),
            class_name: p.class_name.clone().unwrap_or_default(),
            inject_method: p.inject_method.clone().unwrap_or_default(),
            eject_method: p.eject_method.clone().unwrap_or_default(),
            mono_module: p.mono_module.clone().unwrap_or_default(),
            base_dir: p.base_dir.clone().unwrap_or_default(),
            timeout_ms: p.timeout_ms.map(|v| v.to_string()).unwrap_or_default(),
            wait_module: p.wait_module.clone().unwrap_or_default(),
            settle_ms: p.settle_ms.map(|v| v.to_string()).unwrap_or_default(),
            steam_app: p.steam_app.map(|v| v.to_string()).unwrap_or_default(),
        }
    }
}

fn draft_to_profile(draft: &ProfileDraft) -> Profile {
    Profile {
        process: util::non_empty(&draft.process),
        assembly: util::non_empty(&draft.assembly).map(PathBuf::from),
        namespace: util::non_empty(&draft.namespace),
        class_name: util::non_empty(&draft.class_name),
        inject_method: util::non_empty(&draft.inject_method),
        eject_method: util::non_empty(&draft.eject_method),
        mono_module: util::non_empty(&draft.mono_module),
        base_dir: util::non_empty(&draft.base_dir),
        timeout_ms: draft.timeout_ms.parse().ok(),
        wait_module: util::non_empty(&draft.wait_module),
        settle_ms: draft.settle_ms.parse().ok(),
        steam_app: draft.steam_app.parse().ok(),
    }
}

/// State for the Profiles management view.
#[derive(Debug, Default, Clone)]
pub struct ProfilesState {
    pub profiles: Vec<ProfileSummary>,
    pub selected_index: Option<usize>,
    pub editing: bool,
    pub confirm_delete: bool,
    pub advanced_expanded: bool,
    pub draft: ProfileDraft,
    pub save_error: Option<String>,
}

/// Messages handled by the Profiles view.
#[derive(Debug, Clone)]
pub enum ProfilesMsg {
    Load,
    Loaded(Result<Vec<ProfileSummary>, String>),
    Select(usize),
    NewProfile,
    EditClicked,
    CancelEdit,
    SaveClicked,
    Saved(Result<(), String>),
    DeleteClicked,
    ConfirmDelete,
    CancelDelete,
    AdvancedToggled,
    Deleted(Result<(), String>),
    InjectWithProfile,
    DraftChanged(DraftField, String),
}

impl ProfilesMsg {
    pub fn log_entry(&self) -> Option<crate::widget::log_strip::LogEntry> {
        use crate::widget::log_strip::LogEntry;
        match self {
            Self::DeleteClicked => Some(LogEntry::warn("Confirm profile deletion".into())),
            Self::Saved(Ok(())) => Some(LogEntry::ok("Profile saved".into())),
            Self::Saved(Err(e)) => Some(LogEntry::error(format!("Save failed: {e}"))),
            Self::Deleted(Ok(())) => Some(LogEntry::ok("Profile deleted".into())),
            Self::Deleted(Err(e)) => Some(LogEntry::error(format!("Delete failed: {e}"))),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DraftField {
    Name,
    Process,
    Assembly,
    Namespace,
    ClassName,
    InjectMethod,
    EjectMethod,
    MonoModule,
    BaseDir,
    TimeoutMs,
    WaitModule,
    SettleMs,
    SteamApp,
}

pub fn update(state: &mut ProfilesState, msg: ProfilesMsg) -> Task<ProfilesMsg> {
    match msg {
        ProfilesMsg::Load => load_profiles(),
        ProfilesMsg::Loaded(r) => {
            apply_loaded(state, r);
            Task::none()
        }
        ProfilesMsg::Select(i) => {
            state.selected_index = Some(i);
            state.editing = false;
            Task::none()
        }
        ProfilesMsg::NewProfile => {
            start_new(state);
            Task::none()
        }
        ProfilesMsg::EditClicked => {
            start_edit(state);
            Task::none()
        }
        ProfilesMsg::CancelEdit => {
            state.editing = false;
            Task::none()
        }
        ProfilesMsg::AdvancedToggled => {
            state.advanced_expanded = !state.advanced_expanded;
            Task::none()
        }
        ProfilesMsg::SaveClicked => save_profile(state),
        ProfilesMsg::Saved(r) => {
            apply_saved(state, r);
            Task::done(ProfilesMsg::Load)
        }
        ProfilesMsg::DeleteClicked => {
            state.confirm_delete = true;
            Task::none()
        }
        ProfilesMsg::ConfirmDelete => {
            state.confirm_delete = false;
            delete_profile(state)
        }
        ProfilesMsg::CancelDelete => {
            state.confirm_delete = false;
            Task::none()
        }
        ProfilesMsg::Deleted(r) => {
            apply_deleted(state, &r);
            Task::done(ProfilesMsg::Load)
        }
        ProfilesMsg::InjectWithProfile => Task::none(), // handled in App
        ProfilesMsg::DraftChanged(field, value) => {
            apply_draft(state, field, value);
            Task::none()
        }
    }
}

fn apply_loaded(state: &mut ProfilesState, result: Result<Vec<ProfileSummary>, String>) {
    if let Ok(profiles) = result {
        state.profiles = profiles;
    }
}

fn start_new(state: &mut ProfilesState) {
    state.selected_index = None;
    state.draft = ProfileDraft::default();
    state.editing = true;
    state.advanced_expanded = false;
}

fn start_edit(state: &mut ProfilesState) {
    if let Some(i) = state.selected_index
        && let Some(summary) = state.profiles.get(i)
    {
        state.draft = ProfileDraft::from((summary.name.as_str(), &summary.profile));
        state.editing = true;
        state.advanced_expanded = false;
    }
}

fn apply_saved(state: &mut ProfilesState, result: Result<(), String>) {
    state.save_error = result.err();
    if state.save_error.is_none() {
        state.editing = false;
    }
}

fn apply_deleted(state: &mut ProfilesState, result: &Result<(), String>) {
    if result.is_ok() {
        state.selected_index = None;
        state.editing = false;
    }
}

fn apply_draft(state: &mut ProfilesState, field: DraftField, value: String) {
    let d = &mut state.draft;
    match field {
        DraftField::Name => d.name = value,
        DraftField::Process => d.process = value,
        DraftField::Assembly => d.assembly = value,
        DraftField::Namespace => d.namespace = value,
        DraftField::ClassName => d.class_name = value,
        DraftField::InjectMethod => d.inject_method = value,
        DraftField::EjectMethod => d.eject_method = value,
        DraftField::MonoModule => d.mono_module = value,
        DraftField::BaseDir => d.base_dir = value,
        DraftField::TimeoutMs => d.timeout_ms = value,
        DraftField::WaitModule => d.wait_module = value,
        DraftField::SettleMs => d.settle_ms = value,
        DraftField::SteamApp => d.steam_app = value,
    }
}

fn load_profiles() -> Task<ProfilesMsg> {
    Task::perform(
        util::run_blocking(|| {
            mono_injector_core::profiles::list_profiles().map_err(|e| e.to_string())
        }),
        ProfilesMsg::Loaded,
    )
}

fn save_profile(state: &ProfilesState) -> Task<ProfilesMsg> {
    let draft = state.draft.clone();
    Task::perform(
        util::run_blocking(move || {
            let profile = draft_to_profile(&draft);
            let mut file =
                mono_injector_core::profiles::load_profiles().map_err(|e| e.to_string())?;
            file.profiles.insert(draft.name.clone(), profile);
            mono_injector_core::profiles::save_profiles(&file).map_err(|e| e.to_string())
        }),
        ProfilesMsg::Saved,
    )
}

fn delete_profile(state: &ProfilesState) -> Task<ProfilesMsg> {
    let name = state
        .selected_index
        .and_then(|i| state.profiles.get(i))
        .map(|s| s.name.clone());
    let Some(name) = name else {
        return Task::none();
    };
    Task::perform(
        util::run_blocking(move || {
            let mut file =
                mono_injector_core::profiles::load_profiles().map_err(|e| e.to_string())?;
            file.profiles.remove(&name);
            mono_injector_core::profiles::save_profiles(&file).map_err(|e| e.to_string())
        }),
        ProfilesMsg::Deleted,
    )
}

pub fn view(state: &ProfilesState) -> Element<'_, ProfilesMsg> {
    column![
        page_header::view(
            "Profiles",
            "Manage saved injection profiles and reusable defaults.",
        ),
        row![profile_list_panel(state), profile_detail_panel(state)]
            .spacing(SP4)
            .height(Length::Fill)
    ]
    .spacing(SP4)
    .padding(SP4)
    .height(Length::Fill)
    .into()
}

fn profile_list_panel(state: &ProfilesState) -> Element<'_, ProfilesMsg> {
    let new_btn = button(
        row![
            icon::icon(icon::ADD, 16.0, GREEN),
            text("New Profile").size(12).font(FONT_UI).color(GREEN)
        ]
        .spacing(SP1)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(ProfilesMsg::NewProfile)
    .style(theme::ghost_button_style);

    let header = container(
        row![
            text("SAVED PROFILES").size(10).font(FONT_MONO).color(FG2),
            iced::widget::Space::new().width(Length::Fill),
            new_btn,
        ]
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(SP2)
    .width(Length::Fill)
    .style(|_| theme::panel_header_style());

    let items = state
        .profiles
        .iter()
        .enumerate()
        .map(|(i, s)| profile_list_item(s, i, state.selected_index == Some(i)))
        .collect::<Vec<_>>();
    let list = scrollable(column(items).spacing(1)).height(Length::Fill);

    container(column![header, list].spacing(0))
        .width(280)
        .height(Length::Fill)
        .style(|_| theme::panel_style())
        .into()
}

fn profile_list_item(
    summary: &ProfileSummary,
    index: usize,
    selected: bool,
) -> Element<'_, ProfilesMsg> {
    let process_hint = str_or_dash(summary.profile.process.as_deref());
    button(
        column![
            row![
                text(summary.name.clone())
                    .size(13)
                    .font(FONT_MONO)
                    .color(if selected { PRIMARY } else { FG }),
                iced::widget::Space::new().width(Length::Fill),
                selected_dot(selected),
            ],
            text(process_hint).size(11).font(FONT_MONO).color(FG4),
        ]
        .spacing(2),
    )
    .on_press(ProfilesMsg::Select(index))
    .width(Length::Fill)
    .padding(SP3)
    .style(theme::profile_list_item_button_style(selected))
    .into()
}

fn selected_dot<'a>(selected: bool) -> Element<'a, ProfilesMsg> {
    let color = if selected {
        PRIMARY_C
    } else {
        iced::Color::TRANSPARENT
    };
    container(iced::widget::Space::new().width(8).height(8))
        .style(theme::dot_style(color))
        .into()
}

fn profile_detail_panel(state: &ProfilesState) -> Element<'_, ProfilesMsg> {
    if state.editing {
        profile_edit_panel(state)
    } else if let Some(i) = state.selected_index {
        if let Some(summary) = state.profiles.get(i) {
            profile_read_panel(summary, state.confirm_delete)
        } else {
            empty_detail()
        }
    } else {
        empty_detail()
    }
}

fn profile_read_panel(summary: &ProfileSummary, confirm_delete: bool) -> Element<'_, ProfilesMsg> {
    let p = &summary.profile;
    container(
        column![
            profile_detail_header(summary),
            scrollable(profile_kv_grid(p)).height(Length::Fill),
            profile_footer(confirm_delete),
        ]
        .spacing(0)
        .height(Length::Fill),
    )
    .height(Length::Fill)
    .style(|_| theme::elevated_panel_style())
    .into()
}

fn profile_footer(confirm_delete: bool) -> Element<'static, ProfilesMsg> {
    let content = if confirm_delete {
        row![
            text("Delete this profile?")
                .size(13)
                .font(FONT_UI)
                .color(FG),
            iced::widget::Space::new().width(Length::Fill),
            button(text("Cancel").size(13).font(FONT_UI))
                .on_press(ProfilesMsg::CancelDelete)
                .style(theme::ghost_button_style),
            button(text("Delete").size(13).font(FONT_UI).color(FG))
                .on_press(ProfilesMsg::ConfirmDelete)
                .style(theme::danger_button_style),
        ]
    } else {
        row![
            edit_button(),
            iced::widget::Space::new().width(Length::Fill),
            delete_button(),
        ]
    };
    container(content.spacing(SP2))
        .padding(SP2)
        .width(Length::Fill)
        .style(|_| theme::footer_style())
        .into()
}

fn edit_button() -> Element<'static, ProfilesMsg> {
    button(
        row![
            icon::icon(icon::EDIT, 16.0, FG),
            text("Edit").size(13).font(FONT_UI)
        ]
        .spacing(SP2),
    )
    .on_press(ProfilesMsg::EditClicked)
    .style(theme::ghost_button_style)
    .into()
}

fn delete_button() -> Element<'static, ProfilesMsg> {
    button(
        row![
            icon::icon(icon::DELETE_FOREVER, 16.0, FG),
            text("Delete").size(13).font(FONT_UI).color(FG)
        ]
        .spacing(SP2),
    )
    .on_press(ProfilesMsg::DeleteClicked)
    .style(theme::danger_button_style)
    .into()
}

fn profile_detail_header(summary: &ProfileSummary) -> Element<'_, ProfilesMsg> {
    container(
        row![
            icon::icon(icon::ACCOUNT_TREE, 22.0, PRIMARY_C),
            column![
                text(summary.name.clone())
                    .size(18)
                    .font(FONT_UI_SEMIBOLD)
                    .color(FG),
                text("Stored injection defaults")
                    .size(11)
                    .font(FONT_MONO)
                    .color(FG4),
            ]
            .spacing(2),
            iced::widget::Space::new().width(Length::Fill),
            button(
                row![
                    icon::icon(icon::PLAY_ARROW, 16.0, theme::BG_HARD),
                    text("Inject with profile").size(12).font(FONT_UI)
                ]
                .spacing(SP2)
            )
            .on_press(ProfilesMsg::InjectWithProfile)
            .style(theme::primary_button_style),
        ]
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(SP3)
    .width(Length::Fill)
    .style(|_| theme::panel_header_style())
    .into()
}

fn str_or_dash(v: Option<&str>) -> String {
    v.map_or_else(|| "—".to_owned(), String::from)
}

fn num_or_dash<T: ToString>(v: Option<T>) -> String {
    v.map_or_else(|| "—".to_owned(), |n| n.to_string())
}

fn profile_kv_grid(p: &Profile) -> Element<'static, ProfilesMsg> {
    let assembly = p
        .assembly
        .as_ref()
        .map_or_else(|| "—".to_owned(), |v| v.to_string_lossy().into_owned());
    column![
        row![
            column![
                profile_section(
                    "TARGET PROCESS",
                    value_box(str_or_dash(p.process.as_deref()))
                ),
                profile_section("ASSEMBLY TO INJECT", value_box(assembly))
            ]
            .spacing(SP4)
            .width(Length::FillPortion(1)),
            profile_section("ENTRY POINT", entry_point_values(p)).width(Length::FillPortion(1)),
        ]
        .spacing(40),
        profile_section("RUNTIME OPTIONS", runtime_values(p)),
    ]
    .spacing(32)
    .padding(SP4)
    .into()
}

fn profile_section<'a>(
    title: &'static str,
    body: impl Into<Element<'a, ProfilesMsg>>,
) -> iced::widget::Column<'a, ProfilesMsg> {
    column![section_title(title), body.into()].spacing(SP3)
}

fn section_title<'a>(title: &'static str) -> Element<'a, ProfilesMsg> {
    container(text(title).size(10).font(FONT_MONO).color(FG2))
        .width(Length::Fill)
        .style(|_| theme::section_title_style())
        .into()
}

fn value_box(value: String) -> Element<'static, ProfilesMsg> {
    container(text(value).size(13).font(FONT_MONO).color(FG))
        .padding(SP3)
        .width(Length::Fill)
        .style(|_| theme::recessed_style())
        .into()
}

fn entry_point_values(p: &Profile) -> Element<'static, ProfilesMsg> {
    column![
        labeled_value("Namespace", str_or_dash(p.namespace.as_deref())),
        labeled_value("Class", str_or_dash(p.class_name.as_deref())),
        labeled_value("Method", str_or_dash(p.inject_method.as_deref())),
    ]
    .spacing(SP3)
    .into()
}

fn labeled_value(label: &'static str, value: String) -> Element<'static, ProfilesMsg> {
    column![
        text(label).size(11).font(FONT_MONO).color(FG4),
        value_box(value)
    ]
    .spacing(SP1)
    .into()
}

fn runtime_values(p: &Profile) -> Element<'static, ProfilesMsg> {
    container(
        row![
            runtime_chip("Timeout", num_or_dash(p.timeout_ms)),
            runtime_chip("Mono", str_or_dash(p.mono_module.as_deref())),
            runtime_chip("Wait Module", str_or_dash(p.wait_module.as_deref())),
            runtime_chip("Settle", num_or_dash(p.settle_ms)),
            runtime_chip("Steam", num_or_dash(p.steam_app)),
        ]
        .spacing(SP3),
    )
    .padding(SP3)
    .width(Length::Fill)
    .style(|_| theme::recessed_style())
    .into()
}

fn runtime_chip(label: &'static str, value: String) -> Element<'static, ProfilesMsg> {
    container(
        row![
            text(label).size(11).font(FONT_MONO).color(FG2),
            text(value).size(11).font(FONT_MONO).color(FG)
        ]
        .spacing(SP1),
    )
    .padding([SP1, SP2])
    .style(|_| theme::runtime_chip_style())
    .into()
}

fn profile_edit_panel(state: &ProfilesState) -> Element<'_, ProfilesMsg> {
    let d = &state.draft;
    let form = column![
        edit_identity_section(d),
        edit_target_section(d),
        edit_entry_section(d),
        edit_advanced_section(d, state.advanced_expanded),
        edit_action_row(),
    ]
    .spacing(SP3);

    container(scrollable(form.padding(SP3)).height(Length::Fill))
        .height(Length::Fill)
        .style(|_| theme::elevated_panel_style())
        .into()
}

fn edit_identity_section(d: &ProfileDraft) -> Element<'_, ProfilesMsg> {
    edit_section(
        "PROFILE",
        edit_field(
            "Name",
            "descriptive-profile-name",
            &d.name,
            DraftField::Name,
        ),
    )
    .into()
}

fn edit_target_section(d: &ProfileDraft) -> Element<'_, ProfilesMsg> {
    edit_section(
        "TARGET & ASSEMBLY",
        column![
            edit_field(
                "Process",
                "game.exe or PID",
                &d.process,
                DraftField::Process
            ),
            edit_field(
                "Assembly",
                "C:\\path\\Loader.dll",
                &d.assembly,
                DraftField::Assembly
            ),
        ]
        .spacing(SP2),
    )
    .into()
}

fn edit_entry_section(d: &ProfileDraft) -> Element<'_, ProfilesMsg> {
    edit_section(
        "ENTRY POINT",
        column![
            edit_field("Namespace", "optional", &d.namespace, DraftField::Namespace),
            row![
                edit_field("Class", "Loader", &d.class_name, DraftField::ClassName),
                edit_field("Inject", "Init", &d.inject_method, DraftField::InjectMethod),
                edit_field("Eject", "Unload", &d.eject_method, DraftField::EjectMethod),
            ]
            .spacing(SP2),
        ]
        .spacing(SP2),
    )
    .into()
}

fn edit_section<'a>(
    title: &'static str,
    body: impl Into<Element<'a, ProfilesMsg>>,
) -> iced::widget::Column<'a, ProfilesMsg> {
    column![edit_section_title(title), body.into()].spacing(SP3)
}

fn edit_section_title<'a>(title: &'static str) -> Element<'a, ProfilesMsg> {
    column![
        text(title).size(13).font(FONT_UI_SEMIBOLD).color(FG2),
        rule::horizontal(1),
    ]
    .spacing(SP1)
    .into()
}

fn edit_advanced_section(d: &ProfileDraft, expanded: bool) -> Element<'_, ProfilesMsg> {
    collapsible::collapsible(
        "Optional Runtime & Launch",
        edit_advanced_body(d),
        expanded,
        ProfilesMsg::AdvancedToggled,
    )
}

fn edit_advanced_body(d: &ProfileDraft) -> Element<'_, ProfilesMsg> {
    column![
        row![
            edit_field("Timeout", "5000", &d.timeout_ms, DraftField::TimeoutMs),
            edit_field("Mono", "mono", &d.mono_module, DraftField::MonoModule),
        ]
        .spacing(SP2),
        edit_field("Base Dir", "optional", &d.base_dir, DraftField::BaseDir),
        row![
            edit_field(
                "Wait Module",
                "d3d11.dll",
                &d.wait_module,
                DraftField::WaitModule
            ),
            edit_field("Settle", "8000", &d.settle_ms, DraftField::SettleMs),
            edit_field("Steam App", "480", &d.steam_app, DraftField::SteamApp),
        ]
        .spacing(SP2),
    ]
    .spacing(SP2)
    .padding(SP3)
    .into()
}

fn edit_field<'a>(
    label: &'static str,
    placeholder: &'static str,
    value: &'a str,
    field: DraftField,
) -> Element<'a, ProfilesMsg> {
    row![
        text(label)
            .size(12)
            .font(FONT_UI_SEMIBOLD)
            .color(FG)
            .width(86),
        text_input(placeholder, value)
            .on_input(move |v| ProfilesMsg::DraftChanged(field, v))
            .size(13)
            .padding([7.0, SP2])
            .style(theme::mono_input_style),
    ]
    .spacing(SP2)
    .align_y(iced::alignment::Vertical::Center)
    .width(Length::Fill)
    .into()
}

fn edit_action_row() -> Element<'static, ProfilesMsg> {
    let save_btn = button(text("Save").size(13).font(FONT_UI))
        .on_press(ProfilesMsg::SaveClicked)
        .style(theme::primary_button_style);
    let cancel_btn = button(text("Cancel").size(13).font(FONT_UI))
        .on_press(ProfilesMsg::CancelEdit)
        .style(theme::ghost_button_style);
    row![
        iced::widget::Space::new().width(Length::Fill),
        cancel_btn,
        save_btn
    ]
    .spacing(SP2)
    .into()
}

fn empty_detail<'a>() -> Element<'a, ProfilesMsg> {
    container(
        column![
            text("No profile selected").size(14).color(FG4),
            text("Select a profile or create a new one.")
                .size(12)
                .color(FG4),
        ]
        .spacing(SP2)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .center(Length::Fill)
    .height(Length::Fill)
    .style(|_| theme::elevated_panel_style())
    .into()
}

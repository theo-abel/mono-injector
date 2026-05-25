use std::time::Duration;

use iced::{Element, Subscription, Task, Theme};

use crate::util::open_link;
use crate::view;
use crate::view::eject::{EjectMsg, EjectState};
use crate::view::inject::{InjectMsg, InjectState};
use crate::view::processes::{ProcessesMsg, ProcessesState};
use crate::view::profiles::{ProfilesMsg, ProfilesState};
use crate::view::status::{StatusMsg, StatusState};
use crate::widget::log_strip::{self, LogEntry};
use crate::widget::sidebar;

// Maximum log entries kept in memory before oldest are evicted.
const MAX_LOG_ENTRIES: usize = 500;

/// Which panel is currently displayed in the main content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Inject,
    Eject,
    Status,
    Processes,
    Profiles,
}

/// Top-level message type routing to sub-view messages.
#[derive(Debug, Clone)]
pub enum Message {
    Navigate(View),
    Log(LogEntry),
    ClearLogs,
    Error(String),
    Inject(InjectMsg),
    Eject(EjectMsg),
    Status(StatusMsg),
    Processes(ProcessesMsg),
    Profiles(ProfilesMsg),
    LogStrip(log_strip::Msg),
}

/// Root application state holding every sub-view.
#[derive(Debug, Default)]
pub struct App {
    pub(crate) active_view: View,
    pub(crate) inject: InjectState,
    pub(crate) eject: EjectState,
    pub(crate) status: StatusState,
    pub(crate) processes: ProcessesState,
    pub(crate) profiles: ProfilesState,
    pub(crate) log_entries: Vec<LogEntry>,
}

impl App {
    pub fn theme(_: &Self) -> Theme {
        crate::theme::app_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.active_view {
            View::Processes => iced::time::every(Duration::from_secs(5))
                .map(|_| Message::Processes(ProcessesMsg::Refresh)),
            _ => Subscription::none(),
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Navigate(v) => self.handle_navigate(v),
            Message::Log(e) => self.push_log(e),
            Message::ClearLogs => {
                self.log_entries.clear();
                Task::none()
            }
            Message::Error(e) => self.push_log(LogEntry::error(e)),
            Message::Inject(m) => self.update_inject(m),
            Message::Eject(m) => self.update_eject(m),
            Message::Status(m) => self.update_status(m),
            Message::Processes(m) => self.update_processes(m),
            Message::Profiles(m) => self.update_profiles(m),
            Message::LogStrip(m) => Self::update_log_strip(m),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = sidebar::view(self.active_view).map(|msg| match msg {
            sidebar::Msg::Navigate(v) => Message::Navigate(v),
            sidebar::Msg::ClearLogs => Message::ClearLogs,
        });

        let content = self.current_view();
        let log = log_strip::view(&self.log_entries).map(Message::LogStrip);

        iced::widget::row![
            sidebar,
            iced::widget::column![content, log]
                .height(iced::Length::Fill)
                .spacing(0),
        ]
        .height(iced::Length::Fill)
        .into()
    }

    fn current_view(&self) -> Element<'_, Message> {
        match self.active_view {
            View::Inject => view::inject::view(&self.inject).map(Message::Inject),
            View::Eject => view::eject::view(&self.eject).map(Message::Eject),
            View::Status => view::status::view(&self.status).map(Message::Status),
            View::Processes => view::processes::view(&self.processes).map(Message::Processes),
            View::Profiles => view::profiles::view(&self.profiles).map(Message::Profiles),
        }
    }

    fn handle_navigate(&mut self, view: View) -> Task<Message> {
        self.active_view = view;
        match view {
            View::Status => Task::done(Message::Status(StatusMsg::Load)),
            View::Processes => Task::done(Message::Processes(ProcessesMsg::Load)),
            View::Profiles => Task::done(Message::Profiles(ProfilesMsg::Load)),
            View::Inject => Task::done(Message::Inject(InjectMsg::RefreshProcesses)),
            View::Eject => Task::done(Message::Eject(EjectMsg::LoadRecords)),
        }
    }

    fn push_log(&mut self, entry: LogEntry) -> Task<Message> {
        self.log_entries.push(entry);
        if self.log_entries.len() > MAX_LOG_ENTRIES {
            self.log_entries.remove(0);
        }
        Task::none()
    }

    fn update_inject(&mut self, msg: InjectMsg) -> Task<Message> {
        let navigate_on_success = matches!(msg, InjectMsg::InjectDone(Ok(_)));
        if let Some(entry) = msg.log_entry() {
            self.log_entries.push(entry);
        }
        let task = crate::view::inject::update(&mut self.inject, msg).map(Message::Inject);
        if navigate_on_success {
            Task::batch([task, Task::done(Message::Navigate(View::Status))])
        } else {
            task
        }
    }

    fn update_eject(&mut self, msg: EjectMsg) -> Task<Message> {
        let navigate_on_success = matches!(msg, EjectMsg::EjectDone(Ok(_)));
        if let Some(entry) = msg.log_entry() {
            self.log_entries.push(entry);
        }
        let task = crate::view::eject::update(&mut self.eject, msg).map(Message::Eject);
        if navigate_on_success {
            Task::batch([task, Task::done(Message::Navigate(View::Status))])
        } else {
            task
        }
    }

    fn update_status(&mut self, msg: StatusMsg) -> Task<Message> {
        // EjectRecord navigates away; handle it before consuming msg.
        if let StatusMsg::EjectRecord(ref rec) = msg {
            self.eject.handle_input.clone_from(&rec.handle);
            self.eject.process_input.clone_from(&rec.process_name);
            self.eject.namespace_input.clone_from(&rec.namespace);
            self.eject.class_input.clone_from(&rec.class_name);
            self.eject.method_input.clone_from(&rec.eject_method);
            return Task::done(Message::Navigate(View::Eject));
        }

        if let Some(entry) = msg.log_entry() {
            self.log_entries.push(entry);
        }

        crate::view::status::update(&mut self.status, msg).map(Message::Status)
    }

    fn update_processes(&mut self, msg: ProcessesMsg) -> Task<Message> {
        if let ProcessesMsg::SendToInject(ref proc) = msg {
            self.inject.process_input = crate::util::format_process_label(&proc.name, proc.pid);

            let nav = Task::done(Message::Navigate(View::Inject));
            let log = Task::done(Message::Log(LogEntry::info(format!(
                "Process {} selected from browser",
                proc.name
            ))));

            return Task::batch([nav, log]);
        }

        crate::view::processes::update(&mut self.processes, msg).map(Message::Processes)
    }

    fn update_profiles(&mut self, msg: ProfilesMsg) -> Task<Message> {
        let inject_profile = matches!(msg, ProfilesMsg::InjectWithProfile);

        if let Some(entry) = msg.log_entry() {
            self.log_entries.push(entry);
        }

        if inject_profile {
            if let Some(i) = self.profiles.selected_index
                && let Some(summary) = self.profiles.profiles.get(i)
            {
                self.inject.profile_selection = Some(summary.name.clone());
            }
            return Task::done(Message::Navigate(View::Inject));
        }

        crate::view::profiles::update(&mut self.profiles, msg).map(Message::Profiles)
    }

    fn update_log_strip(msg: log_strip::Msg) -> Task<Message> {
        match msg {
            log_strip::Msg::Open(link) => open_link(link),
        }
    }
}

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

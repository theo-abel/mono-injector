use thiserror::Error as ThisError;

/// Result type used by the reusable service layer.
pub type Result<T> = std::result::Result<T, Error>;

/// Failures that a frontend can map to user-facing messages or exit codes.
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("process '{0}' not found")]
    ProcessNotFound(String),

    #[error("invalid assembly handle '{0}': expected a hex integer like 0x7fff00001234")]
    InvalidHandle(String),

    #[error("failed to read assembly file: {0}")]
    AssemblyRead(std::io::Error),

    #[error("failed to inspect .NET metadata: {0}")]
    AssemblyMetadata(String),

    #[error("failed to access injection records: {0}")]
    InjectionRecords(std::io::Error),

    #[error("failed to parse injection records: {0}")]
    InjectionRecordsParse(serde_json::Error),

    #[error("failed to access profiles: {0}")]
    Profiles(std::io::Error),

    #[error("could not determine the current user's {kind} directory")]
    UserDirectoryUnavailable { kind: &'static str },

    #[error("failed to parse profiles: {0}")]
    ProfilesParse(toml::de::Error),

    #[error("profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("missing {name}; pass {flag} or use a profile that defines it")]
    MissingArgument {
        name: &'static str,
        flag: &'static str,
    },

    #[error("multiple recorded assemblies match {entry}; pass --latest or --assembly")]
    AmbiguousRecordedAssembly { entry: String },

    #[error("no recorded assembly handle for {process} ({pid}) and {entry}")]
    NoRecordedAssembly {
        process: String,
        pid: u32,
        entry: String,
    },

    #[error(
        "refusing to eject unrecorded assembly handle {handle} from {process} ({pid}); pass --force to bypass this guard"
    )]
    UnrecordedAssemblyHandle {
        handle: String,
        process: String,
        pid: u32,
    },

    #[error("--raw-handle requires force")]
    RawHandleRequiresForce,

    #[error("timed out waiting for process '{0}'")]
    ProcessWaitTimeout(String),

    #[error("timed out waiting for module '{module}' in {process} ({pid})")]
    ModuleWaitTimeout {
        process: String,
        pid: u32,
        module: String,
    },

    #[error("failed to launch Steam app {app_id}: {source}")]
    SteamLaunch { app_id: u32, source: std::io::Error },

    #[error("failed to serialize profiles: {0}")]
    ProfilesSerialize(String),

    #[error(transparent)]
    Inject(#[from] mono_injector::Error),
}

impl Error {
    /// Returns the process exit code used by the existing CLI contract.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ProcessNotFound(_) | Self::ProcessWaitTimeout(_) => 10,
            Self::AssemblyRead(_) | Self::AssemblyMetadata(_) => 12,
            Self::NoRecordedAssembly { .. }
            | Self::UnrecordedAssemblyHandle { .. }
            | Self::InvalidHandle(_)
            | Self::RawHandleRequiresForce => 16,
            Self::ModuleWaitTimeout { .. } => 17,
            Self::Inject(e) => injector_exit_code(e),
            _ => 1,
        }
    }
}

fn injector_exit_code(error: &mono_injector::Error) -> i32 {
    match error {
        mono_injector::Error::MonoModuleNotFound(_) => 11,
        mono_injector::Error::ClassNotFound { .. } => 13,
        mono_injector::Error::MethodNotFound(_) => 14,
        mono_injector::Error::ManagedException { .. } => 15,
        mono_injector::Error::InvalidAssemblyHandle { .. } => 16,
        mono_injector::Error::RemoteThreadTimeout(_) => 17,
        _ => 1,
    }
}

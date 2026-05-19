use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("process '{0}' not found")]
    ProcessNotFound(String),

    #[error("invalid assembly handle '{0}': expected a hex integer like 0x7fff00001234")]
    InvalidHandle(String),

    #[error("failed to read assembly file: {0}")]
    AssemblyRead(std::io::Error),

    #[error("failed to access injection records: {0}")]
    InjectionRecords(std::io::Error),

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

    #[error("{0}")]
    Inject(#[from] mono_injector::Error),
}

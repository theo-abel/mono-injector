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

    #[error("{0}")]
    Inject(#[from] mono_injector::Error),
}

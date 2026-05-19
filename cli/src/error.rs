use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to serialize output: {0}")]
    OutputSerialize(serde_json::Error),

    #[error("post-inject command failed: {0}")]
    PostCommand(std::io::Error),

    #[error(transparent)]
    Core(#[from] mono_injector_core::Error),
}

impl Error {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::Core(e) => e.exit_code(),
            _ => 1,
        }
    }
}

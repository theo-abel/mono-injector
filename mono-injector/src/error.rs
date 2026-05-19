use thiserror::Error;
use windows::core::Error as WindowsError;

use iced_x86::IcedError;
use std::io::Error as IoError;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to open process {pid}: {source}")]
    OpenProcess { pid: u32, source: WindowsError },

    #[error("failed to enumerate process modules: {0}")]
    EnumModules(WindowsError),

    #[error("mono module not found in process {0}")]
    MonoModuleNotFound(u32),

    #[error("failed to read mono module file `{path}`: {source}")]
    ModuleFileRead { path: PathBuf, source: IoError },

    #[error("PE parsing failed: {0}")]
    PeParse(String),

    #[error("ReadProcessMemory at {addr:#x}: {source}")]
    ReadMemory { addr: u64, source: WindowsError },

    #[error("WriteProcessMemory at {addr:#x}: {source}")]
    WriteMemory { addr: u64, source: WindowsError },

    #[error("VirtualAllocEx failed: {0}")]
    VirtualAlloc(WindowsError),

    #[error("CreateRemoteThread failed: {0}")]
    CreateRemoteThread(WindowsError),

    #[error("remote thread timed out after {0}ms")]
    RemoteThreadTimeout(u32),

    #[error("WaitForSingleObject failed: {0}")]
    WaitFailed(WindowsError),

    #[error("required Mono export `{0}` not found in target")]
    ExportNotFound(&'static str),

    #[error("shellcode assembly failed: {0}")]
    Assemble(String),

    #[error("mono_get_root_domain returned null")]
    NullRootDomain,

    #[error("mono_image_open_from_data failed ({status:?}): {message}")]
    ImageOpenFailed {
        status: mono_rt::MonoImageOpenStatus,
        message: String,
    },

    #[error("mono_assembly_load_from_full returned null")]
    AssemblyLoadFailed,

    #[error("mono_assembly_get_image returned null")]
    NullImage,

    #[error("assembly handle {handle:#018x} is not readable in the target process")]
    InvalidAssemblyHandle { handle: u64 },

    #[error("class `{namespace}.{name}` not found")]
    ClassNotFound { namespace: String, name: String },

    #[error("method `{0}` not found")]
    MethodNotFound(String),

    #[error("managed exception `{class_name}`: {message}")]
    ManagedException { class_name: String, message: String },

    #[error("assembly data is empty")]
    EmptyAssembly,

    #[error("string `{0}` contains a null byte")]
    NullByteInString(String),
}

impl From<IcedError> for Error {
    fn from(e: IcedError) -> Self {
        Self::Assemble(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_assembly_display_is_nonempty() {
        assert!(!Error::EmptyAssembly.to_string().is_empty());
    }

    #[test]
    fn null_byte_in_string_display_contains_name() {
        let e = Error::NullByteInString("bad\0arg".to_owned());
        assert!(e.to_string().contains("null byte"));
    }

    #[test]
    fn export_not_found_display_contains_name() {
        let e = Error::ExportNotFound("mono_get_root_domain");
        assert!(e.to_string().contains("mono_get_root_domain"));
    }

    #[test]
    fn class_not_found_display_is_nonempty() {
        let e = Error::ClassNotFound {
            namespace: "My.Mod".to_owned(),
            name: "Loader".to_owned(),
        };
        assert!(e.to_string().contains("My.Mod"));
        assert!(e.to_string().contains("Loader"));
    }
}

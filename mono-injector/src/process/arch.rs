use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::IsWow64Process;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arch {
    X86,
    X64,
}

impl Arch {
    /// Returns the pointer width in bytes for this architecture.
    pub(crate) const fn ptr_size(self) -> usize {
        match self {
            Self::X86 => 4,
            Self::X64 => 8,
        }
    }
}

/// Detects whether the process is 32-bit or 64-bit via `IsWow64Process`.
///
/// A WOW64 process (32-bit binary running on 64-bit Windows) maps to `Arch::X86`.
/// A native 64-bit process maps to `Arch::X64`.
pub(crate) fn detect(handle: HANDLE) -> Result<Arch> {
    let mut is_wow64 = windows::core::BOOL(0);
    unsafe { IsWow64Process(handle, &raw mut is_wow64) }
        .map_err(|e| Error::OpenProcess { pid: 0, source: e })?;

    if is_wow64.as_bool() {
        Ok(Arch::X86)
    } else {
        Ok(Arch::X64)
    }
}

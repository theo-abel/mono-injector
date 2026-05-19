use std::path::PathBuf;

use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};
use windows::Win32::System::ProcessStatus::LIST_MODULES_ALL;
use windows::Win32::System::ProcessStatus::{EnumProcessModulesEx, GetModuleFileNameExW};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::process::memory::SharedProcess;

/// A Mono module located in the target process.
#[derive(Debug, Clone)]
pub(crate) struct MonoModule {
    /// Base address of the module in the target process.
    pub(crate) base: u64,
    /// Absolute path to the DLL file on disk (used for PE export parsing).
    pub(crate) path: PathBuf,
}

/// Locates the Mono module in the target process.
///
/// Tries `CreateToolhelp32Snapshot` first and falls back to `EnumProcessModulesEx` on failure.
///
/// # Errors
///
/// Returns [`Error::MonoModuleNotFound`] if no matching module is found, or
/// [`Error::EnumModules`] if the OS scan itself fails.
pub(crate) fn find_mono_module(process: &SharedProcess, config: &Config) -> Result<MonoModule> {
    let hint = &config.mono_module_hint;
    if let Ok(Some(m)) = toolhelp_scan(process.pid, hint) {
        return Ok(m);
    }
    if let Some(m) = enum_modules_scan(process, hint)? {
        return Ok(m);
    }
    Err(Error::MonoModuleNotFound(process.pid))
}

/// RAII snapshot handle; closes on drop.
struct Snapshot(HANDLE);

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn toolhelp_scan(pid: u32, hint: &str) -> Result<Option<MonoModule>> {
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) }
        .map_err(Error::EnumModules)?;
    let _guard = Snapshot(snap);
    Ok(find_in_snapshot(snap, hint))
}

fn find_in_snapshot(snap: HANDLE, hint: &str) -> Option<MonoModule> {
    let mut entry = unsafe { std::mem::zeroed::<MODULEENTRY32W>() };
    entry.dwSize = u32::try_from(std::mem::size_of::<MODULEENTRY32W>()).unwrap_or(u32::MAX);

    if unsafe { Module32FirstW(snap, &raw mut entry) }.is_err() {
        return None;
    }
    loop {
        if let Some(m) = module_from_entry(&entry, hint) {
            return Some(m);
        }
        if unsafe { Module32NextW(snap, &raw mut entry) }.is_err() {
            break;
        }
    }
    None
}

fn module_from_entry(entry: &MODULEENTRY32W, hint: &str) -> Option<MonoModule> {
    let null = entry
        .szExePath
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(entry.szExePath.len());
    let path_str = String::from_utf16_lossy(&entry.szExePath[..null]);
    if !hint_matches(&path_str, hint) {
        return None;
    }
    Some(MonoModule {
        base: entry.modBaseAddr as u64,
        path: PathBuf::from(path_str.as_str()),
    })
}

fn enum_modules_scan(process: &SharedProcess, hint: &str) -> Result<Option<MonoModule>> {
    let handles = module_handles(process)?;
    Ok(handles
        .into_iter()
        .find_map(|hmod| module_from_handle(process, hmod, hint)))
}

fn module_handles(process: &SharedProcess) -> Result<Vec<HMODULE>> {
    let mut needed = 0u32;
    unsafe {
        EnumProcessModulesEx(
            process.raw,
            std::ptr::null_mut(),
            0,
            &raw mut needed,
            LIST_MODULES_ALL,
        )
    }
    .map_err(Error::EnumModules)?;

    let count = (needed as usize) / std::mem::size_of::<HMODULE>();
    let mut handles = vec![HMODULE::default(); count];
    unsafe {
        EnumProcessModulesEx(
            process.raw,
            handles.as_mut_ptr(),
            needed,
            &raw mut needed,
            LIST_MODULES_ALL,
        )
    }
    .map_err(Error::EnumModules)?;

    Ok(handles)
}

fn module_from_handle(process: &SharedProcess, hmod: HMODULE, hint: &str) -> Option<MonoModule> {
    let path_str = module_path(process, hmod)?;
    hint_matches(&path_str, hint).then(|| MonoModule {
        base: hmod.0 as u64,
        path: PathBuf::from(path_str.as_str()),
    })
}

fn module_path(process: &SharedProcess, hmod: HMODULE) -> Option<String> {
    let mut buf = [0u16; 260];
    let len = unsafe { GetModuleFileNameExW(Some(process.raw), Some(hmod), &mut buf) } as usize;
    (len != 0).then(|| String::from_utf16_lossy(&buf[..len]))
}

fn hint_matches(path: &str, hint: &str) -> bool {
    let hint = hint.as_bytes();
    if hint.is_empty() {
        return true;
    }

    path.as_bytes()
        .windows(hint.len())
        .any(|window| window.eq_ignore_ascii_case(hint))
}

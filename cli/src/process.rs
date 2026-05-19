use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

use crate::error::{Error, Result};

/// Resolves a target string to a process ID.
///
/// Parses `target` as a `u32` PID first; if that fails, scans running
/// processes for an exact name match (case-insensitive).
///
/// # Errors
///
/// Returns [`Error::ProcessNotFound`] if no process matches the name.
pub fn resolve_pid(target: &str) -> Result<u32> {
    if let Ok(pid) = target.parse::<u32>() {
        return Ok(pid);
    }

    find_by_name(target).ok_or_else(|| Error::ProcessNotFound(target.to_owned()))
}

/// Returns a sorted list of `(pid, name)` pairs for all running processes.
///
/// When `filter` is provided, only processes whose name contains `filter`
/// (case-insensitive substring) are included.
#[must_use]
pub fn list_processes(filter: Option<&str>) -> Vec<(u32, String)> {
    let lower_filter = filter.map(str::to_lowercase);
    let mut procs: Vec<(u32, String)> = snapshot()
        .into_iter()
        .filter(|(pid, name)| process_matches(*pid, name, lower_filter.as_deref()))
        .collect();

    procs.sort_by_key(|(pid, _)| *pid);
    procs
}

fn find_by_name(name: &str) -> Option<u32> {
    snapshot()
        .into_iter()
        .find(|(_, n)| n.eq_ignore_ascii_case(name))
        .map(|(pid, _)| pid)
}

fn snapshot() -> Vec<(u32, String)> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sys.processes()
        .values()
        .map(|p| (p.pid().as_u32(), p.name().to_string_lossy().into_owned()))
        .collect()
}

fn process_matches(pid: u32, name: &str, filter: Option<&str>) -> bool {
    filter.is_none_or(|f| name.to_lowercase().contains(f) || module_matches(pid, f))
}

fn module_matches(pid: u32, filter: &str) -> bool {
    let Ok(snapshot) =
        (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) })
    else {
        return false;
    };

    let _guard = Snapshot(snapshot);

    snapshot_contains(snapshot, filter)
}

fn snapshot_contains(snapshot: HANDLE, filter: &str) -> bool {
    let mut entry = unsafe { std::mem::zeroed::<MODULEENTRY32W>() };

    entry.dwSize = u32::try_from(std::mem::size_of::<MODULEENTRY32W>()).unwrap_or(u32::MAX);

    if unsafe { Module32FirstW(snapshot, &raw mut entry) }.is_err() {
        return false;
    }

    loop {
        if module_entry_matches(&entry, filter) {
            return true;
        }
        if unsafe { Module32NextW(snapshot, &raw mut entry) }.is_err() {
            return false;
        }
    }
}

fn module_entry_matches(entry: &MODULEENTRY32W, filter: &str) -> bool {
    utf16_buf_contains(&entry.szModule, filter) || utf16_buf_contains(&entry.szExePath, filter)
}

fn utf16_buf_contains(buf: &[u16], needle: &str) -> bool {
    let null = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());

    String::from_utf16_lossy(&buf[..null])
        .to_lowercase()
        .contains(needle)
}

struct Snapshot(HANDLE);

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessInfo {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) start_time: u64,
}

pub(crate) fn resolve_process(target: &str) -> Result<ProcessInfo> {
    let processes = snapshot();
    if let Ok(pid) = target.parse::<u32>() {
        return find_by_pid(&processes, pid, target);
    }

    find_by_name(&processes, target).ok_or_else(|| Error::ProcessNotFound(target.to_owned()))
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
        .filter(|process| process_matches(process, lower_filter.as_deref()))
        .map(|process| (process.pid, process.name))
        .collect();

    procs.sort_by_key(|(pid, _)| *pid);
    procs
}

fn find_by_pid(processes: &[ProcessInfo], pid: u32, target: &str) -> Result<ProcessInfo> {
    processes
        .iter()
        .find(|process| process.pid == pid)
        .cloned()
        .ok_or_else(|| Error::ProcessNotFound(target.to_owned()))
}

fn find_by_name(processes: &[ProcessInfo], name: &str) -> Option<ProcessInfo> {
    processes
        .iter()
        .find(|process| process.name.eq_ignore_ascii_case(name))
        .cloned()
}

fn snapshot() -> Vec<ProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sys.processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            start_time: p.start_time(),
        })
        .collect()
}

fn process_matches(process: &ProcessInfo, filter: Option<&str>) -> bool {
    filter.is_none_or(|f| process.name.to_lowercase().contains(f) || module_matches(process.pid, f))
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

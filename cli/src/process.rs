use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProcessInfo {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) start_time: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProcessListing {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) matched_modules: Vec<String>,
}

pub(crate) fn resolve_process(target: &str) -> Result<ProcessInfo> {
    let processes = all_processes();
    if let Ok(pid) = target.parse::<u32>() {
        return find_by_pid(&processes, pid, target);
    }
    find_by_name(&processes, target).ok_or_else(|| Error::ProcessNotFound(target.to_owned()))
}

pub(crate) fn wait_for_process(
    target: &str,
    timeout: Duration,
    poll: Duration,
) -> Result<ProcessInfo> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(process) = resolve_process(target) {
            return Ok(process);
        }
        if Instant::now() >= deadline {
            return Err(Error::ProcessWaitTimeout(target.to_owned()));
        }
        thread::sleep(poll);
    }
}

pub(crate) fn wait_for_module(
    process: &ProcessInfo,
    module: &str,
    timeout: Duration,
    poll: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if module_matches(process.pid, module) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(module_timeout(process, module));
        }
        thread::sleep(poll);
    }
}

#[must_use]
pub(crate) fn all_processes() -> Vec<ProcessInfo> {
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

#[must_use]
pub(crate) fn list_processes(
    filter: Option<&str>,
    mono_only: bool,
    unity_only: bool,
    include_modules: bool,
) -> Vec<ProcessListing> {
    let mut procs = all_processes()
        .into_iter()
        .filter_map(|process| listing(process, filter, mono_only, unity_only, include_modules))
        .collect::<Vec<_>>();
    procs.sort_by_key(|process| process.pid);
    procs
}

#[must_use]
pub(crate) fn module_names(pid: u32) -> Vec<String> {
    let Ok(snapshot) = module_snapshot(pid) else {
        return Vec::new();
    };
    let _guard = Snapshot(snapshot);
    collect_modules(snapshot)
}

#[must_use]
pub(crate) fn module_matches(pid: u32, filter: &str) -> bool {
    module_names(pid)
        .iter()
        .any(|module| contains_ci(module, filter))
}

fn listing(
    process: ProcessInfo,
    filter: Option<&str>,
    mono_only: bool,
    unity_only: bool,
    include_modules: bool,
) -> Option<ProcessListing> {
    let modules = selected_modules(&process, filter, mono_only, unity_only, include_modules);
    if process_matches(&process, filter, mono_only, unity_only, &modules) {
        Some(ProcessListing {
            pid: process.pid,
            name: process.name,
            matched_modules: modules,
        })
    } else {
        None
    }
}

fn selected_modules(
    process: &ProcessInfo,
    filter: Option<&str>,
    mono_only: bool,
    unity_only: bool,
    include_modules: bool,
) -> Vec<String> {
    let modules = module_names(process.pid);
    modules
        .into_iter()
        .filter(|module| module_selected(module, filter, mono_only, unity_only, include_modules))
        .collect()
}

fn module_selected(
    module: &str,
    filter: Option<&str>,
    mono_only: bool,
    unity_only: bool,
    include_modules: bool,
) -> bool {
    include_modules
        || filter.is_some_and(|f| contains_ci(module, f))
        || (mono_only && contains_ci(module, "mono"))
        || (unity_only && contains_ci(module, "unity"))
}

fn process_matches(
    process: &ProcessInfo,
    filter: Option<&str>,
    mono_only: bool,
    unity_only: bool,
    modules: &[String],
) -> bool {
    filter_match(process, filter, modules)
        && (!mono_only || module_matches(process.pid, "mono"))
        && (!unity_only || unity_match(process, modules))
}

fn filter_match(process: &ProcessInfo, filter: Option<&str>, modules: &[String]) -> bool {
    filter.is_none_or(|f| {
        contains_ci(&process.name, f) || modules.iter().any(|module| contains_ci(module, f))
    })
}

fn unity_match(process: &ProcessInfo, modules: &[String]) -> bool {
    contains_ci(&process.name, "unity") || modules.iter().any(|module| contains_ci(module, "unity"))
}

fn collect_modules(snapshot: HANDLE) -> Vec<String> {
    let Some(mut entry) = first_module(snapshot) else {
        return Vec::new();
    };
    let mut modules = Vec::new();
    loop {
        modules.push(module_name(&entry));
        if unsafe { Module32NextW(snapshot, &raw mut entry) }.is_err() {
            return modules;
        }
    }
}

fn first_module(snapshot: HANDLE) -> Option<MODULEENTRY32W> {
    let mut entry = unsafe { std::mem::zeroed::<MODULEENTRY32W>() };
    entry.dwSize = u32::try_from(std::mem::size_of::<MODULEENTRY32W>()).unwrap_or(u32::MAX);
    unsafe { Module32FirstW(snapshot, &raw mut entry) }
        .is_ok()
        .then_some(entry)
}

fn module_snapshot(pid: u32) -> windows::core::Result<HANDLE> {
    unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) }
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

fn module_name(entry: &MODULEENTRY32W) -> String {
    utf16_buf_to_string(&entry.szModule)
}

fn utf16_buf_to_string(buf: &[u16]) -> String {
    let null = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..null])
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn module_timeout(process: &ProcessInfo, module: &str) -> Error {
    Error::ModuleWaitTimeout {
        process: process.name.clone(),
        pid: process.pid,
        module: module.to_owned(),
    }
}

struct Snapshot(HANDLE);

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

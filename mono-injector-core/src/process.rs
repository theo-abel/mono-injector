use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

use crate::error::{Error, Result};

/// Identifies a process strongly enough to detect PID reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub start_time: u64,
}

/// Process-listing row with optional matched module names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessListing {
    pub pid: u32,
    pub name: String,
    pub matched_modules: Vec<String>,
}

/// Filters used when listing candidate processes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListOptions {
    pub filter: Option<String>,
    pub module_filter: ModuleFilter,
    pub include_modules: bool,
}

/// Selects process runtime/module families for process listing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleFilter {
    /// Include processes regardless of loaded Mono or Unity modules.
    #[default]
    Any,
    /// Include only processes with a Mono runtime module.
    Mono,
    /// Include only Unity processes or processes with Unity modules.
    Unity,
    /// Include only Unity processes that also have a Mono runtime module.
    MonoAndUnity,
}

/// Resolves either a PID string or an exact process name.
///
/// # Errors
///
/// Returns an error when the target process is not currently running.
pub fn resolve_process(target: &str) -> Result<ProcessInfo> {
    let processes = all_processes();
    if let Ok(pid) = target.parse::<u32>() {
        return find_by_pid(&processes, pid, target);
    }
    find_by_name(&processes, target).ok_or_else(|| Error::ProcessNotFound(target.to_owned()))
}

/// Polls until a process exists or the timeout elapses.
///
/// # Errors
///
/// Returns an error when the target cannot be found before the deadline.
pub fn wait_for_process(target: &str, timeout: Duration, poll: Duration) -> Result<ProcessInfo> {
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

/// Polls until a module name appears in the process module list.
///
/// # Errors
///
/// Returns an error when the module cannot be found before the deadline.
pub fn wait_for_module(
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

/// Returns all running processes visible to the current user.
#[must_use]
pub fn all_processes() -> Vec<ProcessInfo> {
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

/// Lists processes using the same filter semantics as the CLI.
#[must_use]
pub fn list_processes(options: &ListOptions) -> Vec<ProcessListing> {
    let mut procs = all_processes()
        .into_iter()
        .filter_map(|process| listing(process, options))
        .collect::<Vec<_>>();

    procs.sort_by_key(|process| process.pid);
    procs
}

/// Returns loaded module names for a process, or an empty list when unavailable.
#[must_use]
pub fn module_names(pid: u32) -> Vec<String> {
    let Ok(snapshot) = module_snapshot(pid) else {
        return Vec::new();
    };

    let _guard = Snapshot(snapshot);
    collect_modules(snapshot)
}

/// Tests whether a process has a module containing the given case-insensitive fragment.
#[must_use]
pub fn module_matches(pid: u32, filter: &str) -> bool {
    module_names(pid)
        .iter()
        .any(|module| contains_ci(module, filter))
}

fn listing(process: ProcessInfo, options: &ListOptions) -> Option<ProcessListing> {
    let modules = selected_modules(&process, options);

    if process_matches(&process, options, &modules) {
        Some(ProcessListing {
            pid: process.pid,
            name: process.name,
            matched_modules: modules,
        })
    } else {
        None
    }
}

fn selected_modules(process: &ProcessInfo, options: &ListOptions) -> Vec<String> {
    if !needs_modules(options) {
        return Vec::new();
    }

    module_names(process.pid)
        .into_iter()
        .filter(|module| module_selected(module, options))
        .collect()
}

fn needs_modules(options: &ListOptions) -> bool {
    options.include_modules || options.filter.is_some() || options.module_filter.needs_modules()
}

fn module_selected(module: &str, options: &ListOptions) -> bool {
    options.include_modules
        || options
            .filter
            .as_deref()
            .is_some_and(|f| contains_ci(module, f))
        || (options.module_filter.needs_mono() && contains_ci(module, "mono"))
        || (options.module_filter.needs_unity() && contains_ci(module, "unity"))
}

fn process_matches(process: &ProcessInfo, options: &ListOptions, modules: &[String]) -> bool {
    filter_match(process, options.filter.as_deref(), modules)
        && mono_filter_match(options.module_filter, modules)
        && unity_filter_match(process, options.module_filter, modules)
}

fn mono_filter_match(filter: ModuleFilter, modules: &[String]) -> bool {
    !filter.needs_mono() || modules.iter().any(|module| contains_ci(module, "mono"))
}

fn unity_filter_match(process: &ProcessInfo, filter: ModuleFilter, modules: &[String]) -> bool {
    !filter.needs_unity() || unity_match(process, modules)
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
    let needle = needle.as_bytes();
    needle.is_empty()
        || haystack
            .as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle))
}

fn module_timeout(process: &ProcessInfo, module: &str) -> Error {
    Error::ModuleWaitTimeout {
        process: process.name.clone(),
        pid: process.pid,
        module: module.to_owned(),
    }
}

impl ModuleFilter {
    const fn needs_modules(self) -> bool {
        !matches!(self, Self::Any)
    }

    const fn needs_mono(self) -> bool {
        matches!(self, Self::Mono | Self::MonoAndUnity)
    }

    const fn needs_unity(self) -> bool {
        matches!(self, Self::Unity | Self::MonoAndUnity)
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

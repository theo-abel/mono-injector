use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use directories::BaseDirs;
use mono_injector::AssemblyHandle;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::process::{ProcessInfo, all_processes};

const STATE_DIR: &str = "mono-injector";
const STATE_FILE: &str = "injections.json";
const LEGACY_STATE_FILE: &str = "injections.tsv";
const VERSION: u32 = 2;

/// Remembered assembly loaded into a specific process instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InjectionRecord {
    pub process_name: String,
    pub pid: u32,
    pub start_time: u64,
    pub handle: String,
    pub assembly_path: Option<PathBuf>,
    pub namespace: String,
    pub class_name: String,
    pub inject_method: String,
    pub eject_method: String,
    pub profile: Option<String>,
    pub injected_at: u64,
}

/// Data recorded after a successful injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectionInput {
    pub process: ProcessInfo,
    pub handle: AssemblyHandle,
    pub assembly_path: Option<PathBuf>,
    pub namespace: String,
    pub class_name: String,
    pub inject_method: String,
    pub eject_method: String,
    pub profile: Option<String>,
}

/// Selects which remembered injection records should be removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanMode {
    /// Remove only records whose process instance is no longer running.
    Stale,
    /// Remove every record, including records for live processes.
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StateFile {
    version: u32,
    injections: Vec<InjectionRecord>,
}

impl InjectionRecord {
    /// Parses the serialized handle back into the low-level handle type.
    #[must_use]
    pub fn handle_value(&self) -> Option<AssemblyHandle> {
        parse_handle(&self.handle).and_then(AssemblyHandle::from_raw)
    }

    /// Returns the cleanup entry point displayed by frontends.
    #[must_use]
    pub fn entry(&self) -> String {
        entry_name(&self.namespace, &self.class_name, &self.eject_method)
    }

    fn from_input(input: InjectionInput) -> Self {
        Self {
            process_name: input.process.name,
            pid: input.process.pid,
            start_time: input.process.start_time,
            handle: input.handle.to_string(),
            assembly_path: input.assembly_path,
            namespace: input.namespace,
            class_name: input.class_name,
            inject_method: input.inject_method,
            eject_method: input.eject_method,
            profile: input.profile,
            injected_at: now_secs(),
        }
    }

    fn matches_process(&self, process: &ProcessInfo) -> bool {
        self.pid == process.pid && self.start_time == process.start_time
    }

    fn matches_entry(&self, namespace: Option<&str>, class_name: Option<&str>) -> bool {
        namespace.is_none_or(|n| self.namespace == n)
            && class_name.is_none_or(|c| self.class_name == c)
    }
}

/// Stores or replaces a remembered injection record.
///
/// # Errors
///
/// Returns an error when the state file cannot be read or written.
pub fn remember(input: InjectionInput) -> Result<()> {
    let record = InjectionRecord::from_input(input);
    let mut state = load_state()?;

    state
        .injections
        .retain(|item| !same_injection(item, &record));

    state.injections.push(record);
    save_state(&state)
}

/// Verifies that a handle is recorded for the same process and entry point.
///
/// # Errors
///
/// Returns an error when state cannot be loaded or the handle is unrecorded.
pub fn ensure_recorded(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> Result<()> {
    is_recorded(process, handle, namespace, class_name)?
        .then_some(())
        .ok_or_else(|| unrecorded_error(process, handle))
}

/// Finds remembered records for a process and optional entry-point filters.
///
/// # Errors
///
/// Returns an error when state cannot be loaded.
pub fn matching(
    process: &ProcessInfo,
    namespace: Option<&str>,
    class_name: Option<&str>,
) -> Result<Vec<InjectionRecord>> {
    Ok(load_state()?
        .injections
        .into_iter()
        .filter(|record| record.matches_process(process))
        .filter(|record| record.matches_entry(namespace, class_name))
        .collect())
}

/// Returns every remembered injection record.
///
/// # Errors
///
/// Returns an error when state cannot be loaded.
pub fn all() -> Result<Vec<InjectionRecord>> {
    Ok(load_state()?.injections)
}

/// Removes one process/handle record after a successful ejection.
///
/// # Errors
///
/// Returns an error when state cannot be read or written.
pub fn forget(process: &ProcessInfo, handle: AssemblyHandle) -> Result<()> {
    let mut state = load_state()?;
    state
        .injections
        .retain(|record| !record_matches_handle(record, process, handle));

    save_state(&state)
}

/// Removes records using a caller-provided process snapshot.
///
/// # Errors
///
/// Returns an error when state cannot be read or written.
pub fn clean(live: &[ProcessInfo], mode: CleanMode) -> Result<usize> {
    let mut state = load_state()?;
    let before = state.injections.len();

    match mode {
        CleanMode::All => state.injections.clear(),
        CleanMode::Stale => state.injections.retain(|record| is_live(record, live)),
    }

    save_state(&state)?;
    Ok(before - state.injections.len())
}

/// Removes records using the current process list.
///
/// # Errors
///
/// Returns an error when state cannot be read or written.
pub fn clean_stale_records(mode: CleanMode) -> Result<usize> {
    clean(&all_processes(), mode)
}

fn is_recorded(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> Result<bool> {
    Ok(matching(process, Some(namespace), Some(class_name))?
        .iter()
        .any(|record| record.handle_value() == Some(handle)))
}

fn same_injection(left: &InjectionRecord, right: &InjectionRecord) -> bool {
    left.pid == right.pid
        && left.start_time == right.start_time
        && left.handle == right.handle
        && left.namespace == right.namespace
        && left.class_name == right.class_name
}

fn record_matches_handle(
    record: &InjectionRecord,
    process: &ProcessInfo,
    handle: AssemblyHandle,
) -> bool {
    record.matches_process(process) && record.handle_value() == Some(handle)
}

fn is_live(record: &InjectionRecord, live: &[ProcessInfo]) -> bool {
    live.iter()
        .any(|process| record.pid == process.pid && record.start_time == process.start_time)
}

fn load_state() -> Result<StateFile> {
    match fs::read_to_string(state_path()?) {
        Ok(content) => parse_state(&content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => load_legacy_state(),
        Err(e) => Err(Error::InjectionRecords(e)),
    }
}

fn load_legacy_state() -> Result<StateFile> {
    match fs::read_to_string(legacy_state_path()?) {
        Ok(content) => Ok(legacy_state(&content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(default_state()),
        Err(e) => Err(Error::InjectionRecords(e)),
    }
}

fn parse_state(content: &str) -> Result<StateFile> {
    serde_json::from_str(content).map_err(Error::InjectionRecordsParse)
}

fn save_state(state: &StateFile) -> Result<()> {
    let path = state_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(Error::InjectionRecords)?;
    }

    let content = serde_json::to_string_pretty(state).map_err(Error::InjectionRecordsParse)?;
    fs::write(path, content).map_err(Error::InjectionRecords)
}

fn legacy_state(content: &str) -> StateFile {
    StateFile {
        version: VERSION,
        injections: content.lines().filter_map(parse_legacy_record).collect(),
    }
}

fn parse_legacy_record(line: &str) -> Option<InjectionRecord> {
    let mut parts = line.split('\t');
    let pid = parts.next()?.parse().ok()?;
    let start_time = parts.next()?.parse().ok()?;
    let handle = parts.next()?.to_owned();
    let namespace = parts.next()?.to_owned();
    let class_name = parts.next()?.to_owned();

    Some(legacy_record(
        pid, start_time, handle, namespace, class_name,
    ))
}

fn legacy_record(
    pid: u32,
    start_time: u64,
    handle: String,
    namespace: String,
    class_name: String,
) -> InjectionRecord {
    InjectionRecord {
        process_name: format!("pid:{pid}"),
        pid,
        start_time,
        handle,
        assembly_path: None,
        namespace,
        class_name,
        inject_method: "Init".to_owned(),
        eject_method: "Unload".to_owned(),
        profile: None,
        injected_at: 0,
    }
}

fn unrecorded_error(process: &ProcessInfo, handle: AssemblyHandle) -> Error {
    Error::UnrecordedAssemblyHandle {
        handle: handle.to_string(),
        process: process.name.clone(),
        pid: process.pid,
    }
}

fn parse_handle(raw: &str) -> Option<u64> {
    let digits = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);

    u64::from_str_radix(digits, 16).ok()
}

pub(crate) fn entry_name(namespace: &str, class_name: &str, method_name: &str) -> String {
    if namespace.is_empty() {
        format!("{class_name}::{method_name}")
    } else {
        format!("{namespace}.{class_name}::{method_name}")
    }
}

fn default_state() -> StateFile {
    StateFile {
        version: VERSION,
        injections: Vec::new(),
    }
}

fn state_path() -> Result<PathBuf> {
    Ok(base_dir()?.join(STATE_DIR).join(STATE_FILE))
}

fn legacy_state_path() -> Result<PathBuf> {
    Ok(base_dir()?.join(STATE_DIR).join(LEGACY_STATE_FILE))
}

fn base_dir() -> Result<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.data_local_dir().to_owned())
        .ok_or(Error::UserDirectoryUnavailable { kind: "data" })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mono_injector::AssemblyHandle;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::process::ProcessInfo;

const STATE_DIR: &str = "mono-injector";
const STATE_FILE: &str = "injections.json";
const LEGACY_STATE_FILE: &str = "injections.tsv";
const VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct InjectionRecord {
    pub(crate) process_name: String,
    pub(crate) pid: u32,
    pub(crate) start_time: u64,
    pub(crate) handle: String,
    pub(crate) assembly_path: Option<PathBuf>,
    pub(crate) namespace: String,
    pub(crate) class_name: String,
    pub(crate) inject_method: String,
    pub(crate) eject_method: String,
    pub(crate) profile: Option<String>,
    pub(crate) injected_at: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct InjectionInput {
    pub(crate) process: ProcessInfo,
    pub(crate) handle: AssemblyHandle,
    pub(crate) assembly_path: Option<PathBuf>,
    pub(crate) namespace: String,
    pub(crate) class_name: String,
    pub(crate) inject_method: String,
    pub(crate) eject_method: String,
    pub(crate) profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StateFile {
    version: u32,
    injections: Vec<InjectionRecord>,
}

impl InjectionRecord {
    #[must_use]
    pub(crate) fn handle_value(&self) -> Option<AssemblyHandle> {
        parse_handle(&self.handle).and_then(AssemblyHandle::from_raw)
    }

    #[must_use]
    pub(crate) fn entry(&self) -> String {
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

pub(crate) fn remember(input: InjectionInput) -> Result<()> {
    let record = InjectionRecord::from_input(input);
    let mut state = load_state()?;
    state
        .injections
        .retain(|item| !same_injection(item, &record));
    state.injections.push(record);
    save_state(&state)
}

pub(crate) fn ensure_recorded(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> Result<()> {
    if is_recorded(process, handle, namespace, class_name)? {
        Ok(())
    } else {
        Err(unrecorded_error(process, handle))
    }
}

pub(crate) fn matching(
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

pub(crate) fn all() -> Result<Vec<InjectionRecord>> {
    Ok(load_state()?.injections)
}

pub(crate) fn forget(process: &ProcessInfo, handle: AssemblyHandle) -> Result<()> {
    let mut state = load_state()?;
    state
        .injections
        .retain(|record| !record_matches_handle(record, process, handle));
    save_state(&state)
}

pub(crate) fn clean(live: &[ProcessInfo], all: bool) -> Result<usize> {
    let mut state = load_state()?;
    let before = state.injections.len();
    if all {
        state.injections.clear();
    } else {
        state.injections.retain(|record| is_live(record, live));
    }
    save_state(&state)?;
    Ok(before - state.injections.len())
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
    match fs::read_to_string(state_path()) {
        Ok(content) => parse_state(&content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => load_legacy_state(),
        Err(e) => Err(Error::InjectionRecords(e)),
    }
}

fn load_legacy_state() -> Result<StateFile> {
    match fs::read_to_string(legacy_state_path()) {
        Ok(content) => Ok(legacy_state(&content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(default_state()),
        Err(e) => Err(Error::InjectionRecords(e)),
    }
}

fn parse_state(content: &str) -> Result<StateFile> {
    serde_json::from_str(content).map_err(Error::InjectionRecordsParse)
}

fn save_state(state: &StateFile) -> Result<()> {
    let path = state_path();
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

fn entry_name(namespace: &str, class_name: &str, method_name: &str) -> String {
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

fn state_path() -> PathBuf {
    base_dir().join(STATE_DIR).join(STATE_FILE)
}

fn legacy_state_path() -> PathBuf {
    base_dir().join(STATE_DIR).join(LEGACY_STATE_FILE)
}

fn base_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(std::env::temp_dir)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

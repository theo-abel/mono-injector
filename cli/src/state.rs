use std::fs;
use std::path::PathBuf;

use mono_injector::AssemblyHandle;

use crate::error::{Error, Result};
use crate::process::ProcessInfo;

const STATE_DIR: &str = "mono-injector";
const STATE_FILE: &str = "injections.tsv";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    pid: u32,
    start_time: u64,
    handle: u64,
    namespace: String,
    class_name: String,
}

impl Record {
    fn new(
        process: &ProcessInfo,
        handle: AssemblyHandle,
        namespace: &str,
        class_name: &str,
    ) -> Self {
        Self {
            pid: process.pid,
            start_time: process.start_time,
            handle: handle.as_raw(),
            namespace: namespace.to_owned(),
            class_name: class_name.to_owned(),
        }
    }

    fn matches(
        &self,
        process: &ProcessInfo,
        handle: AssemblyHandle,
        namespace: &str,
        class_name: &str,
    ) -> bool {
        self.pid == process.pid
            && self.start_time == process.start_time
            && self.handle == handle.as_raw()
            && self.namespace == namespace
            && self.class_name == class_name
    }

    fn matches_entry(&self, process: &ProcessInfo, namespace: &str, class_name: &str) -> bool {
        self.pid == process.pid
            && self.start_time == process.start_time
            && self.namespace == namespace
            && self.class_name == class_name
    }

    fn handle(&self) -> Option<AssemblyHandle> {
        AssemblyHandle::from_raw(self.handle)
    }
}

pub(crate) fn remember(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> std::io::Result<()> {
    let mut records = load()?;
    records.retain(|record| !record.matches(process, handle, namespace, class_name));
    records.push(Record::new(process, handle, namespace, class_name));
    save(&records)
}

pub(crate) fn ensure_recorded(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> Result<()> {
    if is_recorded(process, handle, namespace, class_name) {
        Ok(())
    } else {
        Err(unrecorded_error(process, handle))
    }
}

pub(crate) fn latest(
    process: &ProcessInfo,
    namespace: &str,
    class_name: &str,
) -> std::io::Result<Option<AssemblyHandle>> {
    Ok(load()?
        .iter()
        .rev()
        .find(|record| record.matches_entry(process, namespace, class_name))
        .and_then(Record::handle))
}

pub(crate) fn forget(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> std::io::Result<()> {
    let mut records = load()?;
    records.retain(|record| !record.matches(process, handle, namespace, class_name));
    save(&records)
}

fn is_recorded(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    namespace: &str,
    class_name: &str,
) -> bool {
    load().is_ok_and(|records| {
        records
            .iter()
            .any(|record| record.matches(process, handle, namespace, class_name))
    })
}

fn unrecorded_error(process: &ProcessInfo, handle: AssemblyHandle) -> Error {
    Error::UnrecordedAssemblyHandle {
        handle: handle.to_string(),
        process: process.name.clone(),
        pid: process.pid,
    }
}

fn load() -> std::io::Result<Vec<Record>> {
    match fs::read_to_string(path()) {
        Ok(content) => Ok(content.lines().filter_map(parse_record).collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

fn save(records: &[Record]) -> std::io::Result<()> {
    let path = path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, records.iter().map(format_record).collect::<String>())
}

fn parse_record(line: &str) -> Option<Record> {
    let mut parts = line.split('\t');
    Some(Record {
        pid: parts.next()?.parse().ok()?,
        start_time: parts.next()?.parse().ok()?,
        handle: parse_handle(parts.next()?)?,
        namespace: parts.next()?.to_owned(),
        class_name: parts.next()?.to_owned(),
    })
}

fn format_record(record: &Record) -> String {
    format!(
        "{}\t{}\t{:#x}\t{}\t{}\n",
        record.pid, record.start_time, record.handle, record.namespace, record.class_name
    )
}

fn parse_handle(raw: &str) -> Option<u64> {
    let digits = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    u64::from_str_radix(digits, 16).ok()
}

fn path() -> PathBuf {
    base_dir().join(STATE_DIR).join(STATE_FILE)
}

fn base_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(std::env::temp_dir)
}

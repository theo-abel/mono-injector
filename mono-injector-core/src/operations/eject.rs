use mono_injector::{AssemblyHandle, EjectRequest};

use crate::error::{Error, Result};
use crate::process::{ProcessInfo, resolve_process};
use crate::profiles::Profile;
use crate::state::{self, InjectionRecord};

use super::shared::{injector_for, load_profile, profile_process, required};
use super::{DEFAULT_EJECT_METHOD, EjectOptions, EjectOutput, ResolvedEjectPlan};

#[derive(Debug, Clone)]
struct PreparedEject {
    profile_name: Option<String>,
    profile: Option<Profile>,
    process: ProcessInfo,
    handle: AssemblyHandle,
    namespace: String,
    class_name: String,
    method_name: String,
    raw: bool,
}

/// Resolves an ejection plan without changing the target process.
///
/// # Errors
///
/// Returns an error when inputs are incomplete, ambiguous, or unsafe without force.
pub fn resolve_eject(options: &EjectOptions) -> Result<ResolvedEjectPlan> {
    let prepared = prepare_eject(options)?;
    enforce_record_guard(options, &prepared)?;
    Ok(eject_plan(&prepared))
}

/// Executes an ejection and removes the remembered handle.
///
/// # Errors
///
/// Returns an error when resolution, guard validation, ejection, or state updates fail.
pub fn eject(options: &EjectOptions) -> Result<EjectOutput> {
    let prepared = prepare_eject(options)?;
    enforce_record_guard(options, &prepared)?;

    let injector = injector_for(options, &prepared.process, prepared.profile.as_ref());
    injector.eject(&eject_request(&prepared))?;
    state::forget(&prepared.process, prepared.handle)?;

    Ok(eject_output("ejected", &prepared))
}

fn prepare_eject(options: &EjectOptions) -> Result<PreparedEject> {
    let profile = load_profile(options.profile_name.as_deref())?;
    let process = resolve_process(&eject_process_name(options, profile.as_ref())?)?;
    let handle = explicit_handle(options)?;
    let record = selected_record(options, &process, profile.as_ref(), handle)?;

    resolved_eject(options, profile, process, handle, record.as_ref())
}

fn resolved_eject(
    options: &EjectOptions,
    profile: Option<Profile>,
    process: ProcessInfo,
    handle: Option<(AssemblyHandle, bool)>,
    record: Option<&InjectionRecord>,
) -> Result<PreparedEject> {
    let (handle, raw) = handle
        .or_else(|| record_handle(record))
        .ok_or_else(|| no_record(&process, options))?;

    Ok(PreparedEject {
        namespace: eject_namespace(options, profile.as_ref(), record),
        class_name: eject_class_name(options, profile.as_ref(), record)?,
        method_name: eject_method_name(options, profile.as_ref(), record),
        profile_name: options.profile_name.clone(),
        profile,
        process,
        handle,
        raw,
    })
}

fn selected_record(
    options: &EjectOptions,
    process: &ProcessInfo,
    profile: Option<&Profile>,
    handle: Option<(AssemblyHandle, bool)>,
) -> Result<Option<InjectionRecord>> {
    if handle.is_some_and(|(_, raw)| raw) {
        return Ok(None);
    }

    let matches = matching_records(options, process, profile)?;

    select_record(options, handle.map(|(h, _)| h), matches)
}

fn select_record(
    options: &EjectOptions,
    handle: Option<AssemblyHandle>,
    records: Vec<InjectionRecord>,
) -> Result<Option<InjectionRecord>> {
    if let Some(handle) = handle {
        return Ok(records
            .into_iter()
            .find(|record| record.handle_value() == Some(handle)));
    }

    match records.len() {
        0 => Ok(None),
        1 => Ok(records.into_iter().next()),
        _ if options.latest => Ok(records.into_iter().last()),
        _ => Err(Error::AmbiguousRecordedAssembly {
            entry: entry_filter(options),
        }),
    }
}

fn matching_records(
    options: &EjectOptions,
    process: &ProcessInfo,
    profile: Option<&Profile>,
) -> Result<Vec<InjectionRecord>> {
    let namespace = options
        .namespace
        .as_deref()
        .or_else(|| profile.and_then(|p| p.namespace.as_deref()));

    let class_name = options
        .class_name
        .as_deref()
        .or_else(|| profile.and_then(|p| p.class_name.as_deref()));

    state::matching(process, namespace, class_name)
}

fn explicit_handle(options: &EjectOptions) -> Result<Option<(AssemblyHandle, bool)>> {
    if let Some(raw) = &options.raw_handle {
        if !options.force {
            return Err(Error::RawHandleRequiresForce);
        }

        return parse_handle(raw).map(|handle| Some((handle, true)));
    }

    options
        .handle
        .as_deref()
        .map(parse_handle)
        .transpose()
        .map(|handle| handle.map(|h| (h, false)))
}

fn enforce_record_guard(options: &EjectOptions, prepared: &PreparedEject) -> Result<()> {
    if options.force || prepared.raw {
        Ok(())
    } else {
        state::ensure_recorded(
            &prepared.process,
            prepared.handle,
            &prepared.namespace,
            &prepared.class_name,
        )
    }
}

fn eject_request(prepared: &PreparedEject) -> EjectRequest<'_> {
    EjectRequest {
        handle: prepared.handle,
        namespace: &prepared.namespace,
        class_name: &prepared.class_name,
        method_name: &prepared.method_name,
    }
}

fn eject_plan(prepared: &PreparedEject) -> ResolvedEjectPlan {
    ResolvedEjectPlan {
        entry: state::entry_name(
            &prepared.namespace,
            &prepared.class_name,
            &prepared.method_name,
        ),
        process: prepared.process.clone(),
        handle: prepared.handle.to_string(),
        namespace: prepared.namespace.clone(),
        class_name: prepared.class_name.clone(),
        method_name: prepared.method_name.clone(),
        profile: prepared.profile_name.clone(),
        raw: prepared.raw,
    }
}

fn eject_output(status: &str, prepared: &PreparedEject) -> EjectOutput {
    EjectOutput {
        status: status.to_owned(),
        process: prepared.process.clone(),
        handle: prepared.handle.to_string(),
        entry: state::entry_name(
            &prepared.namespace,
            &prepared.class_name,
            &prepared.method_name,
        ),
        profile: prepared.profile_name.clone(),
    }
}

fn eject_process_name(options: &EjectOptions, profile: Option<&Profile>) -> Result<String> {
    required(
        options
            .process
            .as_ref()
            .or_else(|| profile_process(profile)),
        "process",
        "-p",
    )
}

fn eject_class_name(
    options: &EjectOptions,
    profile: Option<&Profile>,
    record: Option<&InjectionRecord>,
) -> Result<String> {
    options
        .class_name
        .clone()
        .or_else(|| profile.and_then(|p| p.class_name.clone()))
        .or_else(|| record.map(|r| r.class_name.clone()))
        .ok_or(Error::MissingArgument {
            name: "class",
            flag: "-c",
        })
}

fn eject_namespace(
    options: &EjectOptions,
    profile: Option<&Profile>,
    record: Option<&InjectionRecord>,
) -> String {
    options
        .namespace
        .clone()
        .or_else(|| profile.and_then(|p| p.namespace.clone()))
        .or_else(|| record.map(|r| r.namespace.clone()))
        .unwrap_or_default()
}

fn eject_method_name(
    options: &EjectOptions,
    profile: Option<&Profile>,
    record: Option<&InjectionRecord>,
) -> String {
    options
        .method_name
        .clone()
        .or_else(|| profile.and_then(|p| p.eject_method.clone()))
        .or_else(|| record.map(|r| r.eject_method.clone()))
        .unwrap_or_else(|| DEFAULT_EJECT_METHOD.to_owned())
}

fn record_handle(record: Option<&InjectionRecord>) -> Option<(AssemblyHandle, bool)> {
    record?.handle_value().map(|handle| (handle, false))
}

fn no_record(process: &ProcessInfo, options: &EjectOptions) -> Error {
    Error::NoRecordedAssembly {
        process: process.name.clone(),
        pid: process.pid,
        entry: entry_filter(options),
    }
}

fn entry_filter(options: &EjectOptions) -> String {
    let namespace = options.namespace.as_deref().unwrap_or("*");
    let class_name = options.class_name.as_deref().unwrap_or("*");
    let method_name = options
        .method_name
        .as_deref()
        .unwrap_or(DEFAULT_EJECT_METHOD);

    state::entry_name(namespace, class_name, method_name)
}

fn parse_handle(raw: &str) -> Result<AssemblyHandle> {
    let digits = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);

    let ptr = u64::from_str_radix(digits, 16).map_err(|_| Error::InvalidHandle(raw.to_owned()))?;

    AssemblyHandle::from_raw(ptr).ok_or_else(|| Error::InvalidHandle(raw.to_owned()))
}

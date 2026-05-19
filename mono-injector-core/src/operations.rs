use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use mono_injector::{AssemblyHandle, EjectRequest, InjectRequest, Injector};
use serde::{Deserialize, Serialize};

use crate::dotnet;
use crate::error::{Error, Result};
use crate::process::{ProcessInfo, resolve_process, wait_for_module, wait_for_process};
use crate::profiles::{Profile, get_profile};
use crate::runtime::RuntimeOptions;
use crate::state::{self, InjectionInput, InjectionRecord};

/// Default loader class used when neither args nor profile provide one.
pub const DEFAULT_CLASS_NAME: &str = "Loader";
/// Default injection method used when neither args nor profile provide one.
pub const DEFAULT_INJECT_METHOD: &str = "Init";
/// Default ejection method used when neither args nor profile provide one.
pub const DEFAULT_EJECT_METHOD: &str = "Unload";
/// Default readiness module used for Steam-launched Unity games.
pub const DEFAULT_STEAM_WAIT_MODULE: &str = "d3d11.dll";
/// Default post-readiness delay used for Steam launches.
pub const DEFAULT_STEAM_SETTLE_MS: u64 = 8_000;

/// Profile-aware options for resolving or executing an injection.
#[derive(Debug, Clone)]
pub struct InjectOptions {
    pub profile_name: Option<String>,
    pub process: Option<String>,
    pub assembly: Option<PathBuf>,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
    pub inject_method: Option<String>,
    pub eject_method: Option<String>,
    pub wait_for_process: bool,
    pub wait_timeout: Duration,
    pub poll_interval: Duration,
    pub wait_module: Option<String>,
    pub disable_wait_module: bool,
    pub settle_delay: Option<Duration>,
    pub steam_app: Option<u32>,
    pub runtime: RuntimeOptions,
}

/// Profile-aware options for resolving or executing an ejection.
#[derive(Debug, Clone)]
pub struct EjectOptions {
    pub profile_name: Option<String>,
    pub process: Option<String>,
    pub handle: Option<String>,
    pub raw_handle: Option<String>,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub latest: bool,
    pub force: bool,
    pub runtime: RuntimeOptions,
}

/// Fully resolved injection plan suitable for dry-run output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedInjectPlan {
    pub process: ProcessInfo,
    pub assembly: PathBuf,
    pub namespace: String,
    pub class_name: String,
    pub method_name: String,
    pub eject_method: String,
    pub entry: String,
    pub wait_module: Option<String>,
    pub settle_ms: u64,
    pub steam_app: Option<u32>,
    pub profile: Option<String>,
}

/// Result returned after injection or dry-run planning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InjectOutput {
    pub status: String,
    pub process: ProcessInfo,
    pub assembly: PathBuf,
    pub entry: String,
    pub handle: Option<String>,
    pub profile: Option<String>,
}

/// Fully resolved ejection plan suitable for dry-run output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedEjectPlan {
    pub process: ProcessInfo,
    pub handle: String,
    pub namespace: String,
    pub class_name: String,
    pub method_name: String,
    pub entry: String,
    pub profile: Option<String>,
    pub raw: bool,
}

/// Result returned after ejection or dry-run planning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EjectOutput {
    pub status: String,
    pub process: ProcessInfo,
    pub handle: String,
    pub entry: String,
    pub profile: Option<String>,
}

#[derive(Debug, Clone)]
struct PreparedInject {
    profile_name: Option<String>,
    profile: Option<Profile>,
    process_name: String,
    assembly_path: PathBuf,
    assembly: Vec<u8>,
    namespace: String,
    class_name: String,
    method_name: String,
    eject_method: String,
    wait_module: Option<String>,
    settle_ms: u64,
    steam_app: Option<u32>,
}

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

/// Resolves an injection plan without changing the target process.
///
/// # Errors
///
/// Returns an error when inputs are incomplete, metadata is invalid, or the process is absent.
pub fn resolve_inject(options: &InjectOptions) -> Result<ResolvedInjectPlan> {
    let prepared = prepare_inject(options)?;
    let process = resolve_process(&prepared.process_name)?;

    Ok(inject_plan(process, &prepared))
}

/// Executes an injection and records the returned assembly handle.
///
/// # Errors
///
/// Returns an error when resolution, waiting, Steam launch, injection, or state recording fails.
pub fn inject(options: &InjectOptions) -> Result<InjectOutput> {
    let prepared = prepare_inject(options)?;
    maybe_launch(&prepared)?;

    let process = target_process(options, &prepared)?;
    wait_for_readiness(options, &prepared, &process)?;
    maybe_settle(prepared.settle_ms);

    let handle = run_inject(options, &prepared, &process)?;
    remember_handle(&prepared, &process, handle)?;

    Ok(inject_output("injected", process, &prepared, Some(handle)))
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

impl ResolvedInjectPlan {
    /// Converts the plan into the dry-run output shape used by frontends.
    #[must_use]
    pub fn dry_run_output(self) -> InjectOutput {
        InjectOutput {
            status: "dry-run".to_owned(),
            process: self.process,
            assembly: self.assembly,
            entry: self.entry,
            handle: None,
            profile: self.profile,
        }
    }
}

impl ResolvedEjectPlan {
    /// Converts the plan into the dry-run output shape used by frontends.
    #[must_use]
    pub fn dry_run_output(self) -> EjectOutput {
        EjectOutput {
            status: "dry-run".to_owned(),
            process: self.process,
            handle: self.handle,
            entry: self.entry,
            profile: self.profile,
        }
    }
}

fn prepare_inject(options: &InjectOptions) -> Result<PreparedInject> {
    let profile = load_profile(options.profile_name.as_deref())?;
    let process_name = inject_process_name(options, profile.as_ref())?;
    let class_name = inject_class_name(options, profile.as_ref());
    let explicit_namespace = inject_namespace(options, profile.as_ref());
    let assembly_path =
        inject_assembly_path(options, profile.as_ref(), &process_name, &class_name)?;
    let assembly = read_assembly(&assembly_path)?;
    let namespace = resolved_namespace(&assembly, explicit_namespace, &class_name)?;

    Ok(prepared_inject(
        options,
        profile,
        process_name,
        assembly_path,
        assembly,
        namespace,
    ))
}

fn prepared_inject(
    options: &InjectOptions,
    profile: Option<Profile>,
    process_name: String,
    assembly_path: PathBuf,
    assembly: Vec<u8>,
    namespace: String,
) -> PreparedInject {
    let steam_app = options
        .steam_app
        .or_else(|| profile.as_ref().and_then(|p| p.steam_app));

    PreparedInject {
        class_name: inject_class_name(options, profile.as_ref()),
        method_name: inject_method_name(options, profile.as_ref()),
        eject_method: inject_eject_method(options, profile.as_ref()),
        wait_module: inject_wait_module(options, profile.as_ref(), steam_app),
        settle_ms: inject_settle_ms(options, profile.as_ref(), steam_app),
        profile_name: options.profile_name.clone(),
        profile,
        process_name,
        assembly_path,
        assembly,
        namespace,
        steam_app,
    }
}

fn inject_assembly_path(
    options: &InjectOptions,
    profile: Option<&Profile>,
    process_name: &str,
    class_name: &str,
) -> Result<PathBuf> {
    options
        .assembly
        .clone()
        .or_else(|| profile.and_then(|p| p.assembly.clone()))
        .or_else(|| remembered_assembly(process_name, options.namespace.as_deref(), class_name))
        .ok_or(Error::MissingArgument {
            name: "assembly",
            flag: "-a",
        })
}

fn remembered_assembly(
    process_name: &str,
    namespace: Option<&str>,
    class_name: &str,
) -> Option<PathBuf> {
    let process = resolve_process(process_name).ok()?;
    state::matching(&process, namespace, Some(class_name))
        .ok()?
        .pop()?
        .assembly_path
}

fn resolved_namespace(
    assembly: &[u8],
    namespace: Option<String>,
    class_name: &str,
) -> Result<String> {
    if let Some(namespace) = namespace {
        dotnet::validate_assembly(assembly)?;
        return Ok(namespace);
    }

    Ok(dotnet::infer_namespace(assembly, class_name)?.unwrap_or_default())
}

fn target_process(options: &InjectOptions, prepared: &PreparedInject) -> Result<ProcessInfo> {
    if options.wait_for_process || prepared.steam_app.is_some() {
        wait_for_process(
            &prepared.process_name,
            options.wait_timeout,
            options.poll_interval,
        )
    } else {
        resolve_process(&prepared.process_name)
    }
}

fn wait_for_readiness(
    options: &InjectOptions,
    prepared: &PreparedInject,
    process: &ProcessInfo,
) -> Result<()> {
    if let Some(module) = &prepared.wait_module {
        wait_for_module(process, module, options.wait_timeout, options.poll_interval)?;
    }
    Ok(())
}

fn maybe_launch(prepared: &PreparedInject) -> Result<()> {
    if let Some(app_id) = prepared.steam_app
        && resolve_process(&prepared.process_name).is_err()
    {
        launch_steam(app_id)?;
    }
    Ok(())
}

fn maybe_settle(settle_ms: u64) {
    if settle_ms > 0 {
        thread::sleep(Duration::from_millis(settle_ms));
    }
}

fn run_inject(
    options: &InjectOptions,
    prepared: &PreparedInject,
    process: &ProcessInfo,
) -> Result<AssemblyHandle> {
    let injector = injector_for(options, process, prepared.profile.as_ref());
    Ok(injector.inject(&inject_request(prepared))?)
}

fn remember_handle(
    prepared: &PreparedInject,
    process: &ProcessInfo,
    handle: AssemblyHandle,
) -> Result<()> {
    state::remember(InjectionInput {
        process: process.clone(),
        handle,
        assembly_path: Some(prepared.assembly_path.clone()),
        namespace: prepared.namespace.clone(),
        class_name: prepared.class_name.clone(),
        inject_method: prepared.method_name.clone(),
        eject_method: prepared.eject_method.clone(),
        profile: prepared.profile_name.clone(),
    })
}

fn inject_request(prepared: &PreparedInject) -> InjectRequest<'_> {
    InjectRequest {
        assembly: &prepared.assembly,
        namespace: &prepared.namespace,
        class_name: &prepared.class_name,
        method_name: &prepared.method_name,
    }
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

fn inject_plan(process: ProcessInfo, prepared: &PreparedInject) -> ResolvedInjectPlan {
    ResolvedInjectPlan {
        entry: state::entry_name(
            &prepared.namespace,
            &prepared.class_name,
            &prepared.method_name,
        ),
        process,
        assembly: prepared.assembly_path.clone(),
        namespace: prepared.namespace.clone(),
        class_name: prepared.class_name.clone(),
        method_name: prepared.method_name.clone(),
        eject_method: prepared.eject_method.clone(),
        wait_module: prepared.wait_module.clone(),
        settle_ms: prepared.settle_ms,
        steam_app: prepared.steam_app,
        profile: prepared.profile_name.clone(),
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

fn inject_output(
    status: &str,
    process: ProcessInfo,
    prepared: &PreparedInject,
    handle: Option<AssemblyHandle>,
) -> InjectOutput {
    InjectOutput {
        status: status.to_owned(),
        process,
        assembly: prepared.assembly_path.clone(),
        entry: state::entry_name(
            &prepared.namespace,
            &prepared.class_name,
            &prepared.method_name,
        ),
        handle: handle.map(|h| h.to_string()),
        profile: prepared.profile_name.clone(),
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

fn injector_for(
    options: &impl RuntimeConfig,
    process: &ProcessInfo,
    profile: Option<&Profile>,
) -> Injector {
    Injector::with_config(process.pid, options.runtime().to_config(profile))
}

trait RuntimeConfig {
    fn runtime(&self) -> &RuntimeOptions;
}

impl RuntimeConfig for InjectOptions {
    fn runtime(&self) -> &RuntimeOptions {
        &self.runtime
    }
}

impl RuntimeConfig for EjectOptions {
    fn runtime(&self) -> &RuntimeOptions {
        &self.runtime
    }
}

fn launch_steam(app_id: u32) -> Result<()> {
    let uri = format!("steam://rungameid/{app_id}");
    Command::new("cmd")
        .args(["/C", "start", "", &uri])
        .spawn()
        .map_err(|source| Error::SteamLaunch { app_id, source })?;
    Ok(())
}

fn read_assembly(path: &PathBuf) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(Error::AssemblyRead)
}

fn load_profile(name: Option<&str>) -> Result<Option<Profile>> {
    name.map(get_profile).transpose()
}

fn inject_process_name(options: &InjectOptions, profile: Option<&Profile>) -> Result<String> {
    required(
        options
            .process
            .as_ref()
            .or_else(|| profile_process(profile)),
        "process",
        "-p",
    )
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

fn profile_process(profile: Option<&Profile>) -> Option<&String> {
    profile.and_then(|p| p.process.as_ref())
}

fn inject_class_name(options: &InjectOptions, profile: Option<&Profile>) -> String {
    options
        .class_name
        .clone()
        .or_else(|| profile.and_then(|p| p.class_name.clone()))
        .unwrap_or_else(|| DEFAULT_CLASS_NAME.to_owned())
}

fn inject_namespace(options: &InjectOptions, profile: Option<&Profile>) -> Option<String> {
    options
        .namespace
        .clone()
        .or_else(|| profile.and_then(|p| p.namespace.clone()))
}

fn inject_method_name(options: &InjectOptions, profile: Option<&Profile>) -> String {
    options
        .inject_method
        .clone()
        .or_else(|| profile.and_then(|p| p.inject_method.clone()))
        .unwrap_or_else(|| DEFAULT_INJECT_METHOD.to_owned())
}

fn inject_eject_method(options: &InjectOptions, profile: Option<&Profile>) -> String {
    options
        .eject_method
        .clone()
        .or_else(|| profile.and_then(|p| p.eject_method.clone()))
        .unwrap_or_else(|| DEFAULT_EJECT_METHOD.to_owned())
}

fn inject_wait_module(
    options: &InjectOptions,
    profile: Option<&Profile>,
    steam_app: Option<u32>,
) -> Option<String> {
    if options.disable_wait_module {
        return None;
    }
    options
        .wait_module
        .clone()
        .or_else(|| profile.and_then(|p| p.wait_module.clone()))
        .or_else(|| steam_app.map(|_| DEFAULT_STEAM_WAIT_MODULE.to_owned()))
}

fn inject_settle_ms(
    options: &InjectOptions,
    profile: Option<&Profile>,
    steam_app: Option<u32>,
) -> u64 {
    options
        .settle_delay
        .map(duration_ms)
        .or_else(|| profile.and_then(|p| p.settle_ms))
        .unwrap_or_else(|| steam_app.map_or(0, |_| DEFAULT_STEAM_SETTLE_MS))
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

fn required(value: Option<&String>, name: &'static str, flag: &'static str) -> Result<String> {
    value.cloned().ok_or(Error::MissingArgument { name, flag })
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

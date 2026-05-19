use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use mono_injector::{AssemblyHandle, InjectRequest};

use crate::dotnet;
use crate::error::{Error, Result};
use crate::process::{ProcessInfo, resolve_process, wait_for_module, wait_for_process};
use crate::profiles::Profile;
use crate::state::{self, InjectionInput};

use super::shared::{duration_ms, injector_for, load_profile, profile_process, required};
use super::{
    DEFAULT_CLASS_NAME, DEFAULT_EJECT_METHOD, DEFAULT_INJECT_METHOD, DEFAULT_STEAM_SETTLE_MS,
    DEFAULT_STEAM_WAIT_MODULE, InjectOptions, InjectOutput, ResolvedInjectPlan,
};

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

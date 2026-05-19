use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::thread;
use std::time::Duration;

use clap::Args as ClapArgs;
use mono_injector::{AssemblyHandle, InjectRequest};
use serde::Serialize;

use super::{RuntimeArgs, injector_for, profile_name};
use crate::context::Context;
use crate::dotnet;
use crate::error::{Error, Result};
use crate::process::{ProcessInfo, resolve_process, wait_for_module, wait_for_process};
use crate::profiles::{self, Profile};
use crate::state::{self, InjectionInput};
use crate::ui;

const DEFAULT_INJECT_METHOD: &str = "Init";
const DEFAULT_EJECT_METHOD: &str = "Unload";
const DEFAULT_CLASS_NAME: &str = "Loader";
const DEFAULT_STEAM_WAIT_MODULE: &str = "d3d11.dll";
const DEFAULT_STEAM_SETTLE_MS: u64 = 4_000;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Optional profile name.
    profile: Option<String>,

    /// Profile name alias for scripts that prefer flags.
    #[arg(long = "profile")]
    profile_alias: Option<String>,

    /// Target process id or exact process name.
    #[arg(short = 'p', long)]
    process: Option<String>,

    /// Managed assembly to load into the target process.
    #[arg(short = 'a', long)]
    assembly: Option<PathBuf>,

    /// Namespace containing the loader class.
    #[arg(short = 'n', long = "namespace")]
    namespace: Option<String>,

    /// Loader class name.
    #[arg(short = 'c', long = "class")]
    class_name: Option<String>,

    /// Loader method to invoke after loading the assembly.
    #[arg(short = 'm', long = "method")]
    method_name: Option<String>,

    /// Cleanup method recorded for later default ejection.
    #[arg(long)]
    eject_method: Option<String>,

    /// Wait for the target process before injecting.
    #[arg(long)]
    wait: bool,

    /// Seconds to wait for process/module readiness.
    #[arg(long, default_value_t = 120)]
    wait_timeout: u64,

    /// Milliseconds between process/module readiness checks.
    #[arg(long, default_value_t = 1_000)]
    poll_interval_ms: u64,

    /// Wait for a loaded module before injecting, for example UnityPlayer.dll.
    #[arg(long)]
    wait_module: Option<String>,

    /// Disable the default readiness-module wait used with --steam-app.
    #[arg(long)]
    no_wait_module: bool,

    /// Extra milliseconds to wait after readiness before injecting. Use 0 to disable.
    #[arg(long)]
    settle_ms: Option<u64>,

    /// Launch a Steam app before waiting for the process.
    #[arg(long)]
    steam_app: Option<u32>,

    /// Resolve inputs without calling Mono in the target process.
    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    runtime: RuntimeArgs,

    /// Command to run after successful injection. Pass after `--`.
    #[arg(last = true)]
    post_command: Vec<String>,
}

#[derive(Debug, Serialize)]
struct InjectOutput {
    status: &'static str,
    process: ProcessInfo,
    assembly: PathBuf,
    entry: String,
    handle: Option<String>,
    profile: Option<String>,
}

struct Resolved {
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

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let resolved = resolve(args)?;
    if args.dry_run {
        let process = resolve_process(&resolved.process_name)?;
        print_plan(ctx, &process, &resolved);
        return dry_run(ctx, process, resolved);
    }
    maybe_launch(&resolved)?;
    let process = resolve_target_process(args, &resolved)?;
    maybe_wait_module(&process, args, &resolved)?;
    maybe_settle(&resolved);
    print_plan(ctx, &process, &resolved);
    inject(ctx, args, process, resolved)
}

fn inject(ctx: Context, args: &Args, process: ProcessInfo, resolved: Resolved) -> Result<()> {
    let pb = ui::spinner();
    pb.set_message("injecting managed assembly...");
    let injector = injector_for(&process, &args.runtime, resolved.profile.as_ref());
    let handle = injector.inject(&request(&resolved))?;
    pb.finish_and_clear();
    remember_handle(&process, handle, &resolved)?;
    run_post_command(args, &process, handle, &resolved)?;
    print_success(ctx, process, resolved, handle)
}

fn resolve(args: &Args) -> Result<Resolved> {
    let profile_name = profile_name(args.profile.as_ref(), args.profile_alias.as_ref());
    let profile = load_profile(profile_name.as_deref())?;
    let process_name = resolved_process_name(args, profile.as_ref())?;
    let class_name = resolved_class_name(args, profile.as_ref());
    let explicit_namespace = namespace(args, profile.as_ref());
    let assembly_path = assembly_path(
        args,
        profile.as_ref(),
        &process_name,
        explicit_namespace.as_ref(),
        &class_name,
    )?;
    let assembly = read_assembly(&assembly_path)?;
    let namespace =
        explicit_namespace.unwrap_or_else(|| inferred_namespace(&assembly, &class_name));
    let steam_app = args
        .steam_app
        .or_else(|| profile.as_ref().and_then(|p| p.steam_app));
    Ok(Resolved {
        assembly_path,
        assembly,
        namespace,
        method_name: method_name(args, profile.as_ref()),
        eject_method: eject_method(args, profile.as_ref()),
        wait_module: wait_module(args, profile.as_ref(), steam_app),
        settle_ms: settle_ms(args, profile.as_ref(), steam_app),
        steam_app,
        profile_name,
        profile,
        process_name,
        class_name,
    })
}

fn resolved_process_name(args: &Args, profile: Option<&Profile>) -> Result<String> {
    required(
        args.process.as_ref().or_else(|| profile_process(profile)),
        "process",
        "-p",
    )
}

fn resolved_class_name(args: &Args, profile: Option<&Profile>) -> String {
    args.class_name
        .clone()
        .or_else(|| profile_class(profile).cloned())
        .unwrap_or_else(|| DEFAULT_CLASS_NAME.to_owned())
}

fn assembly_path(
    args: &Args,
    profile: Option<&Profile>,
    process_name: &str,
    namespace: Option<&String>,
    class_name: &str,
) -> Result<PathBuf> {
    args.assembly
        .clone()
        .or_else(|| profile.and_then(|p| p.assembly.clone()))
        .or_else(|| remembered_assembly(process_name, namespace.map(String::as_str), class_name))
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

fn resolve_target_process(args: &Args, resolved: &Resolved) -> Result<ProcessInfo> {
    if args.wait || resolved.steam_app.is_some() {
        ui::info(&format!("waiting for {}...", resolved.process_name));
        wait_for_process(&resolved.process_name, timeout(args), poll(args))
    } else {
        resolve_process(&resolved.process_name)
    }
}

fn maybe_wait_module(process: &ProcessInfo, args: &Args, resolved: &Resolved) -> Result<()> {
    if let Some(module) = &resolved.wait_module {
        ui::info(&format!("waiting for {module} in {}...", process.name));
        wait_for_module(process, module, timeout(args), poll(args))?;
    }
    Ok(())
}

fn maybe_settle(resolved: &Resolved) {
    if resolved.settle_ms == 0 {
        return;
    }
    ui::info(&format!(
        "waiting {}ms for the game to settle...",
        resolved.settle_ms
    ));
    thread::sleep(Duration::from_millis(resolved.settle_ms));
}

fn maybe_launch(resolved: &Resolved) -> Result<()> {
    if let Some(app_id) = resolved.steam_app
        && resolve_process(&resolved.process_name).is_err()
    {
        launch_steam(app_id)?;
    }
    Ok(())
}

fn dry_run(ctx: Context, process: ProcessInfo, resolved: Resolved) -> Result<()> {
    let output = output("dry-run", process, resolved, None);
    if ctx.json() {
        ctx.print_json(&output)
    } else {
        ui::success("dry run completed");
        Ok(())
    }
}

fn print_success(
    ctx: Context,
    process: ProcessInfo,
    resolved: Resolved,
    handle: AssemblyHandle,
) -> Result<()> {
    let output = output("injected", process, resolved, Some(handle));
    if ctx.json() {
        ctx.print_json(&output)
    } else {
        ui::success("injected successfully");
        println!("{handle}");
        Ok(())
    }
}

fn output(
    status: &'static str,
    process: ProcessInfo,
    resolved: Resolved,
    handle: Option<AssemblyHandle>,
) -> InjectOutput {
    InjectOutput {
        status,
        process,
        assembly: resolved.assembly_path,
        entry: entry_name(
            &resolved.namespace,
            &resolved.class_name,
            &resolved.method_name,
        ),
        handle: handle.map(|h| h.to_string()),
        profile: resolved.profile_name,
    }
}

fn remember_handle(
    process: &ProcessInfo,
    handle: AssemblyHandle,
    resolved: &Resolved,
) -> Result<()> {
    state::remember(InjectionInput {
        process: process.clone(),
        handle,
        assembly_path: Some(resolved.assembly_path.clone()),
        namespace: resolved.namespace.clone(),
        class_name: resolved.class_name.clone(),
        inject_method: resolved.method_name.clone(),
        eject_method: resolved.eject_method.clone(),
        profile: resolved.profile_name.clone(),
    })
}

fn run_post_command(
    args: &Args,
    process: &ProcessInfo,
    handle: AssemblyHandle,
    resolved: &Resolved,
) -> Result<()> {
    let Some((program, rest)) = args.post_command.split_first() else {
        return Ok(());
    };
    ProcessCommand::new(program)
        .args(rest)
        .env("MONO_INJECTOR_PROCESS", &process.name)
        .env("MONO_INJECTOR_PID", process.pid.to_string())
        .env("MONO_INJECTOR_HANDLE", handle.to_string())
        .env("MONO_INJECTOR_ASSEMBLY", &resolved.assembly_path)
        .status()
        .map_err(Error::PostCommand)?;
    Ok(())
}

fn request(resolved: &Resolved) -> InjectRequest<'_> {
    InjectRequest {
        assembly: &resolved.assembly,
        namespace: &resolved.namespace,
        class_name: &resolved.class_name,
        method_name: &resolved.method_name,
    }
}

fn print_plan(ctx: Context, process: &ProcessInfo, resolved: &Resolved) {
    if ctx.json() {
        return;
    }
    ui::label_value("Assembly:", &resolved.assembly_path.display().to_string());
    ui::label_value("Target:", &format!("{} ({})", process.name, process.pid));
    ui::label_value(
        "Entry:",
        &entry_name(
            &resolved.namespace,
            &resolved.class_name,
            &resolved.method_name,
        ),
    );
}

fn launch_steam(app_id: u32) -> Result<()> {
    let uri = format!("steam://rungameid/{app_id}");
    ProcessCommand::new("cmd")
        .args(["/C", "start", "", &uri])
        .spawn()
        .map_err(|source| Error::SteamLaunch { app_id, source })?;
    Ok(())
}

fn read_assembly(path: &PathBuf) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(Error::AssemblyRead)
}

fn load_profile(name: Option<&str>) -> Result<Option<Profile>> {
    name.map(profiles::get).transpose()
}

fn profile_process(profile: Option<&Profile>) -> Option<&String> {
    profile.and_then(|p| p.process.as_ref())
}

fn profile_class(profile: Option<&Profile>) -> Option<&String> {
    profile.and_then(|p| p.class_name.as_ref())
}

fn namespace(args: &Args, profile: Option<&Profile>) -> Option<String> {
    args.namespace
        .clone()
        .or_else(|| profile.and_then(|p| p.namespace.clone()))
}

fn inferred_namespace(assembly: &[u8], class_name: &str) -> String {
    dotnet::infer_namespace(assembly, class_name).unwrap_or_default()
}

fn method_name(args: &Args, profile: Option<&Profile>) -> String {
    args.method_name
        .clone()
        .or_else(|| profile.and_then(|p| p.inject_method.clone()))
        .unwrap_or_else(|| DEFAULT_INJECT_METHOD.to_owned())
}

fn eject_method(args: &Args, profile: Option<&Profile>) -> String {
    args.eject_method
        .clone()
        .or_else(|| profile.and_then(|p| p.eject_method.clone()))
        .unwrap_or_else(|| DEFAULT_EJECT_METHOD.to_owned())
}

fn wait_module(args: &Args, profile: Option<&Profile>, steam_app: Option<u32>) -> Option<String> {
    if args.no_wait_module {
        return None;
    }
    args.wait_module
        .clone()
        .or_else(|| profile.and_then(|p| p.wait_module.clone()))
        .or_else(|| steam_app.map(|_| DEFAULT_STEAM_WAIT_MODULE.to_owned()))
}

fn settle_ms(args: &Args, profile: Option<&Profile>, steam_app: Option<u32>) -> u64 {
    args.settle_ms
        .or_else(|| profile.and_then(|p| p.settle_ms))
        .unwrap_or_else(|| steam_app.map_or(0, |_| DEFAULT_STEAM_SETTLE_MS))
}

fn required(value: Option<&String>, name: &'static str, flag: &'static str) -> Result<String> {
    value.cloned().ok_or(Error::MissingArgument { name, flag })
}

fn timeout(args: &Args) -> Duration {
    Duration::from_secs(args.wait_timeout)
}

fn poll(args: &Args) -> Duration {
    Duration::from_millis(args.poll_interval_ms)
}

fn entry_name(namespace: &str, class_name: &str, method_name: &str) -> String {
    if namespace.is_empty() {
        format!("{class_name}::{method_name}")
    } else {
        format!("{namespace}.{class_name}::{method_name}")
    }
}

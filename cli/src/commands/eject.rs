use clap::Args as ClapArgs;
use mono_injector::{AssemblyHandle, EjectRequest};
use serde::Serialize;

use super::{RuntimeArgs, injector_for, profile_name};
use crate::context::Context;
use crate::error::{Error, Result};
use crate::process::{ProcessInfo, resolve_process};
use crate::profiles::{self, Profile};
use crate::state::{self, InjectionRecord};
use crate::ui;

const DEFAULT_EJECT_METHOD: &str = "Unload";

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

    /// Assembly handle returned by inject. Defaults to a matching remembered injection.
    #[arg(short = 'a', long = "assembly")]
    handle: Option<String>,

    /// Explicit unsafe handle mode; requires --force.
    #[arg(long, conflicts_with = "handle")]
    raw_handle: Option<String>,

    /// Namespace containing the loader class.
    #[arg(short = 'n', long = "namespace")]
    namespace: Option<String>,

    /// Loader class name.
    #[arg(short = 'c', long = "class")]
    class_name: Option<String>,

    /// Cleanup method to invoke before closing the assembly.
    #[arg(short = 'm', long = "method")]
    method_name: Option<String>,

    /// Use the latest matching remembered injection when several match.
    #[arg(long)]
    latest: bool,

    /// Bypass the local injection-record guard for advanced/manual ejection.
    #[arg(long)]
    force: bool,

    /// Resolve inputs without calling Mono in the target process.
    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    runtime: RuntimeArgs,
}

#[derive(Debug, Serialize)]
struct EjectOutput {
    status: &'static str,
    process: ProcessInfo,
    handle: String,
    entry: String,
    profile: Option<String>,
}

struct Resolved {
    profile_name: Option<String>,
    profile: Option<Profile>,
    process: ProcessInfo,
    handle: AssemblyHandle,
    namespace: String,
    class_name: String,
    method_name: String,
    raw: bool,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let resolved = resolve(args)?;
    print_plan(ctx, &resolved);
    if args.dry_run {
        return finish(ctx, "dry-run", resolved);
    }
    enforce_record_guard(args, &resolved)?;
    let injector = injector_for(&resolved.process, &args.runtime, resolved.profile.as_ref());
    injector.eject(&request(&resolved))?;
    state::forget(&resolved.process, resolved.handle)?;
    finish(ctx, "ejected", resolved)
}

fn resolve(args: &Args) -> Result<Resolved> {
    let profile_name = profile_name(args.profile.as_ref(), args.profile_alias.as_ref());
    let profile = load_profile(profile_name.as_deref())?;
    let process = resolve_process(&process_name(args, profile.as_ref())?)?;
    let handle = explicit_handle(args)?;
    let record = selected_record(args, &process, profile.as_ref(), handle)?;
    resolved(
        args,
        profile_name,
        profile,
        process,
        handle,
        record.as_ref(),
    )
}

fn resolved(
    args: &Args,
    profile_name: Option<String>,
    profile: Option<Profile>,
    process: ProcessInfo,
    handle: Option<(AssemblyHandle, bool)>,
    record: Option<&InjectionRecord>,
) -> Result<Resolved> {
    let handle = handle
        .or_else(|| record_handle(record))
        .ok_or_else(|| no_record(&process, args));
    let (handle, raw) = handle?;
    Ok(Resolved {
        namespace: namespace(args, profile.as_ref(), record),
        class_name: class_name(args, profile.as_ref(), record)?,
        method_name: method_name(args, profile.as_ref(), record),
        profile_name,
        profile,
        process,
        handle,
        raw,
    })
}

fn selected_record(
    args: &Args,
    process: &ProcessInfo,
    profile: Option<&Profile>,
    handle: Option<(AssemblyHandle, bool)>,
) -> Result<Option<InjectionRecord>> {
    if handle.is_some_and(|(_, raw)| raw) {
        return Ok(None);
    }
    let matches = matching_records(args, process, profile)?;
    select_record(args, handle.map(|(h, _)| h), matches)
}

fn select_record(
    args: &Args,
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
        _ if args.latest => Ok(records.into_iter().last()),
        _ => Err(Error::AmbiguousRecordedAssembly {
            entry: entry_filter(args),
        }),
    }
}

fn matching_records(
    args: &Args,
    process: &ProcessInfo,
    profile: Option<&Profile>,
) -> Result<Vec<InjectionRecord>> {
    let namespace = args
        .namespace
        .as_deref()
        .or_else(|| profile.and_then(|p| p.namespace.as_deref()));
    let class_name = args
        .class_name
        .as_deref()
        .or_else(|| profile.and_then(|p| p.class_name.as_deref()));
    state::matching(process, namespace, class_name)
}

fn explicit_handle(args: &Args) -> Result<Option<(AssemblyHandle, bool)>> {
    if let Some(raw) = &args.raw_handle {
        if !args.force {
            return Err(Error::RawHandleRequiresForce);
        }
        return parse_handle(raw).map(|handle| Some((handle, true)));
    }
    args.handle
        .as_deref()
        .map(parse_handle)
        .transpose()
        .map(|handle| handle.map(|h| (h, false)))
}

fn enforce_record_guard(args: &Args, resolved: &Resolved) -> Result<()> {
    if args.force || resolved.raw {
        ui::warn("bypassing local injection-record guard");
        Ok(())
    } else {
        state::ensure_recorded(
            &resolved.process,
            resolved.handle,
            &resolved.namespace,
            &resolved.class_name,
        )
    }
}

fn finish(ctx: Context, status: &'static str, resolved: Resolved) -> Result<()> {
    let output = EjectOutput {
        status,
        process: resolved.process,
        handle: resolved.handle.to_string(),
        entry: entry_name(
            &resolved.namespace,
            &resolved.class_name,
            &resolved.method_name,
        ),
        profile: resolved.profile_name,
    };
    if ctx.json() {
        ctx.print_json(&output)
    } else {
        ui::success(if status == "ejected" {
            "ejected successfully"
        } else {
            "dry run completed"
        });
        Ok(())
    }
}

fn request(resolved: &Resolved) -> EjectRequest<'_> {
    EjectRequest {
        handle: resolved.handle,
        namespace: &resolved.namespace,
        class_name: &resolved.class_name,
        method_name: &resolved.method_name,
    }
}

fn print_plan(ctx: Context, resolved: &Resolved) {
    if ctx.json() {
        return;
    }
    ui::label_value(
        "Target:",
        &format!("{} ({})", resolved.process.name, resolved.process.pid),
    );
    ui::label_value("Assembly:", &resolved.handle.to_string());
    ui::label_value(
        "Entry:",
        &entry_name(
            &resolved.namespace,
            &resolved.class_name,
            &resolved.method_name,
        ),
    );
}

fn process_name(args: &Args, profile: Option<&Profile>) -> Result<String> {
    args.process
        .clone()
        .or_else(|| profile.and_then(|p| p.process.clone()))
        .ok_or(Error::MissingArgument {
            name: "process",
            flag: "-p",
        })
}

fn class_name(
    args: &Args,
    profile: Option<&Profile>,
    record: Option<&InjectionRecord>,
) -> Result<String> {
    args.class_name
        .clone()
        .or_else(|| profile.and_then(|p| p.class_name.clone()))
        .or_else(|| record.map(|r| r.class_name.clone()))
        .ok_or(Error::MissingArgument {
            name: "class",
            flag: "-c",
        })
}

fn namespace(args: &Args, profile: Option<&Profile>, record: Option<&InjectionRecord>) -> String {
    args.namespace
        .clone()
        .or_else(|| profile.and_then(|p| p.namespace.clone()))
        .or_else(|| record.map(|r| r.namespace.clone()))
        .unwrap_or_default()
}

fn method_name(args: &Args, profile: Option<&Profile>, record: Option<&InjectionRecord>) -> String {
    args.method_name
        .clone()
        .or_else(|| profile.and_then(|p| p.eject_method.clone()))
        .or_else(|| record.map(|r| r.eject_method.clone()))
        .unwrap_or_else(|| DEFAULT_EJECT_METHOD.to_owned())
}

fn record_handle(record: Option<&InjectionRecord>) -> Option<(AssemblyHandle, bool)> {
    record?.handle_value().map(|handle| (handle, false))
}

fn no_record(process: &ProcessInfo, args: &Args) -> Error {
    Error::NoRecordedAssembly {
        process: process.name.clone(),
        pid: process.pid,
        entry: entry_filter(args),
    }
}

fn entry_filter(args: &Args) -> String {
    let namespace = args.namespace.as_deref().unwrap_or("*");
    let class_name = args.class_name.as_deref().unwrap_or("*");
    let method_name = args.method_name.as_deref().unwrap_or(DEFAULT_EJECT_METHOD);
    entry_name(namespace, class_name, method_name)
}

fn parse_handle(raw: &str) -> Result<AssemblyHandle> {
    let digits = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    let ptr = u64::from_str_radix(digits, 16).map_err(|_| Error::InvalidHandle(raw.to_owned()))?;
    AssemblyHandle::from_raw(ptr).ok_or_else(|| Error::InvalidHandle(raw.to_owned()))
}

fn load_profile(name: Option<&str>) -> Result<Option<Profile>> {
    name.map(profiles::get).transpose()
}

fn entry_name(namespace: &str, class_name: &str, method_name: &str) -> String {
    if namespace.is_empty() {
        format!("{class_name}::{method_name}")
    } else {
        format!("{namespace}.{class_name}::{method_name}")
    }
}

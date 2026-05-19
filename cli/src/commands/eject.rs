use clap::Args as ClapArgs;
use mono_injector::{AssemblyHandle, EjectRequest};

use super::{RuntimeArgs, injector_for};
use crate::error::{Error, Result};
use crate::process::resolve_process;
use crate::state;
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Target process id or exact process name.
    #[arg(short = 'p', long)]
    process: String,

    /// Assembly handle returned by the inject command. Defaults to the last recorded injection.
    #[arg(short = 'a', long = "assembly")]
    handle: Option<String>,

    /// Namespace containing the loader class.
    #[arg(short = 'n', long = "namespace", default_value = "")]
    namespace: String,

    /// Loader class name.
    #[arg(short = 'c', long = "class")]
    class_name: String,

    /// Cleanup method to invoke before closing the assembly.
    #[arg(short = 'm', long = "method")]
    method_name: String,

    /// Bypass the local injection-record guard for advanced/manual ejection.
    #[arg(long)]
    force: bool,

    #[command(flatten)]
    runtime: RuntimeArgs,
}

pub(crate) fn run(args: &Args) -> Result<()> {
    ui::label_value("Target:", &args.process);
    ui::label_value("Entry:", &entry_name(args));

    let pb = ui::spinner();
    pb.set_message("opening target process...");
    let process = resolve_process(&args.process)?;
    let handle = resolve_handle(&process, args)?;
    ui::label_value("Assembly:", &handle.to_string());
    enforce_record_guard(&process, handle, args)?;
    let injector = injector_for(&process, &args.runtime);

    pb.set_message("ejecting managed assembly...");
    injector.eject(&request(args, handle))?;

    pb.finish_and_clear();
    ui::success("ejected successfully");
    forget_handle(&process, handle, args);
    Ok(())
}

fn resolve_handle(process: &crate::process::ProcessInfo, args: &Args) -> Result<AssemblyHandle> {
    if let Some(raw) = &args.handle {
        parse_handle(raw)
    } else {
        latest_handle(process, args)
    }
}

fn latest_handle(process: &crate::process::ProcessInfo, args: &Args) -> Result<AssemblyHandle> {
    state::latest(process, &args.namespace, &args.class_name)
        .map_err(Error::InjectionRecords)?
        .ok_or_else(|| Error::NoRecordedAssembly {
            process: process.name.clone(),
            pid: process.pid,
            entry: entry_name(args),
        })
}

fn enforce_record_guard(
    process: &crate::process::ProcessInfo,
    handle: AssemblyHandle,
    args: &Args,
) -> Result<()> {
    if args.force {
        ui::warn("bypassing local injection-record guard");
        Ok(())
    } else {
        state::ensure_recorded(process, handle, &args.namespace, &args.class_name)
    }
}

fn forget_handle(process: &crate::process::ProcessInfo, handle: AssemblyHandle, args: &Args) {
    if let Err(e) = state::forget(process, handle, &args.namespace, &args.class_name) {
        ui::warn(&format!(
            "ejected but failed to clear local injection record: {e}"
        ));
    }
}

fn parse_handle(raw: &str) -> Result<AssemblyHandle> {
    let digits = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    let ptr = u64::from_str_radix(digits, 16).map_err(|_| Error::InvalidHandle(raw.to_owned()))?;
    AssemblyHandle::from_raw(ptr).ok_or_else(|| Error::InvalidHandle(raw.to_owned()))
}

fn request(args: &Args, handle: AssemblyHandle) -> EjectRequest<'_> {
    EjectRequest {
        handle,
        namespace: &args.namespace,
        class_name: &args.class_name,
        method_name: &args.method_name,
    }
}

fn entry_name(args: &Args) -> String {
    if args.namespace.is_empty() {
        format!("{}::{}", args.class_name, args.method_name)
    } else {
        format!(
            "{}.{}::{}",
            args.namespace, args.class_name, args.method_name
        )
    }
}

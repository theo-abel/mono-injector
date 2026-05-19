use std::path::PathBuf;

use clap::Args as ClapArgs;
use mono_injector::InjectRequest;

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

    /// Managed assembly to load into the target process.
    #[arg(short = 'a', long)]
    assembly: PathBuf,

    /// Namespace containing the loader class.
    #[arg(short = 'n', long = "namespace", default_value = "")]
    namespace: String,

    /// Loader class name.
    #[arg(short = 'c', long = "class")]
    class_name: String,

    /// Loader method to invoke after loading the assembly.
    #[arg(short = 'm', long = "method")]
    method_name: String,

    #[command(flatten)]
    runtime: RuntimeArgs,
}

pub(crate) fn run(args: &Args) -> Result<()> {
    ui::label_value("Assembly:", &args.assembly.display().to_string());
    ui::label_value("Target:", &args.process);
    ui::label_value("Entry:", &entry_name(args));

    let pb = ui::spinner();
    pb.set_message("reading assembly...");
    let assembly = read_assembly(&args.assembly)?;

    pb.set_message("opening target process...");
    let process = resolve_process(&args.process)?;
    let injector = injector_for(&process, &args.runtime);

    pb.set_message("injecting managed assembly...");
    let handle = injector.inject(&request(args, &assembly))?;

    pb.finish_and_clear();
    ui::success("injected successfully");
    remember_handle(&process, handle, args);
    println!("{handle}");
    Ok(())
}

fn remember_handle(
    process: &crate::process::ProcessInfo,
    handle: mono_injector::AssemblyHandle,
    args: &Args,
) {
    if let Err(e) = state::remember(process, handle, &args.namespace, &args.class_name) {
        ui::warn(&format!(
            "injected but failed to record handle for guarded eject: {e}"
        ));
    }
}

fn read_assembly(path: &PathBuf) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(Error::AssemblyRead)
}

fn request<'a>(args: &'a Args, assembly: &'a [u8]) -> InjectRequest<'a> {
    InjectRequest {
        assembly,
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

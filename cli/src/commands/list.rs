use clap::Args as ClapArgs;
use mono_injector_core::process::{ListOptions, ModuleFilter, ProcessListing, list_processes};

use crate::context::Context;
use crate::error::Result;
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Case-insensitive substring used to filter process and module names.
    #[arg(short, long)]
    filter: Option<String>,

    /// Show only processes with a Mono runtime module loaded.
    #[arg(long)]
    mono: bool,

    /// Show only Unity processes.
    #[arg(long)]
    unity: bool,

    /// Include matching or loaded module names in the output.
    #[arg(long)]
    modules: bool,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let processes = list_processes(&args.options());
    if ctx.json() {
        return ctx.print_json(&processes);
    }
    print_processes(&processes, args.modules);
    Ok(())
}

impl Args {
    fn options(&self) -> ListOptions {
        ListOptions {
            filter: self.filter.clone(),
            module_filter: module_filter(self),
            include_modules: self.modules,
        }
    }
}

fn module_filter(args: &Args) -> ModuleFilter {
    match (args.mono, args.unity) {
        (true, true) => ModuleFilter::MonoAndUnity,
        (true, false) => ModuleFilter::Mono,
        (false, true) => ModuleFilter::Unity,
        (false, false) => ModuleFilter::Any,
    }
}

fn print_processes(processes: &[ProcessListing], modules: bool) {
    if processes.is_empty() {
        ui::warn("no matching processes found");
        return;
    }
    ui::info("running processes");
    print_header(modules);
    for process in processes {
        print_process(process, modules);
    }
    ui::muted(&format!("{} process(es)", processes.len()));
}

fn print_header(modules: bool) {
    if modules {
        println!("{:<8} {:<32} MODULES", "PID", "NAME");
    } else {
        println!("{:<8} NAME", "PID");
    }
}

fn print_process(process: &ProcessListing, modules: bool) {
    if modules {
        println!(
            "{:<8} {:<32} {}",
            process.pid,
            process.name,
            process.matched_modules.join(", ")
        );
    } else {
        println!("{:<8} {}", process.pid, process.name);
    }
}

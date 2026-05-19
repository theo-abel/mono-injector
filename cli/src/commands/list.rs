use clap::Args as ClapArgs;

use crate::context::Context;
use crate::error::Result;
use crate::process::{ProcessListing, list_processes};
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
    let processes = list_processes(args.filter.as_deref(), args.mono, args.unity, args.modules);
    if ctx.json() {
        return ctx.print_json(&processes);
    }
    print_processes(&processes, args.modules);
    Ok(())
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

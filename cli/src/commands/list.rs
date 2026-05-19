use clap::Args as ClapArgs;

use crate::process::list_processes;
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Case-insensitive substring used to filter process names.
    #[arg(short, long)]
    filter: Option<String>,
}

pub(crate) fn run(args: &Args) {
    let processes = list_processes(args.filter.as_deref());
    if processes.is_empty() {
        ui::warn("no matching processes found");
        return;
    }

    ui::info("running processes");
    println!("{:<8} NAME", "PID");
    for (pid, name) in &processes {
        println!("{pid:<8} {name}");
    }
    ui::muted(&format!("{} process(es)", processes.len()));
}

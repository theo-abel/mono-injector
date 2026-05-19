use clap::Args as ClapArgs;

use crate::context::Context;
use crate::error::Result;
use crate::process::resolve_process;
use crate::state::{self, InjectionRecord};
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Optional profile name to resolve the target process.
    profile: Option<String>,

    /// Profile name alias for scripts that prefer flags.
    #[arg(long = "profile")]
    profile_alias: Option<String>,

    /// Target process id or exact process name.
    #[arg(short = 'p', long)]
    process: Option<String>,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let records = records(args)?;
    if ctx.json() {
        return ctx.print_json(&records);
    }
    print_records(&records);
    Ok(())
}

fn records(args: &Args) -> Result<Vec<InjectionRecord>> {
    if let Some(process) = process_name(args)? {
        let process = resolve_process(&process)?;
        state::matching(&process, None, None)
    } else {
        state::all()
    }
}

fn process_name(args: &Args) -> Result<Option<String>> {
    if let Some(process) = &args.process {
        return Ok(Some(process.clone()));
    }
    let name = super::profile_name(args.profile.as_ref(), args.profile_alias.as_ref());
    name.map(|profile| crate::profiles::get(&profile).map(|p| p.process))
        .transpose()
        .map(Option::flatten)
}

fn print_records(records: &[InjectionRecord]) {
    if records.is_empty() {
        ui::warn("no remembered injections");
        return;
    }
    for record in records {
        println!(
            "{} ({}) {} {}",
            record.process_name,
            record.pid,
            record.handle,
            record.entry()
        );
    }
}

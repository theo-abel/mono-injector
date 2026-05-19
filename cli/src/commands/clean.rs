use clap::Args as ClapArgs;
use serde::Serialize;

use crate::context::Context;
use crate::error::Result;
use crate::process::all_processes;
use crate::state;
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Remove all remembered injections, including live ones.
    #[arg(long)]
    all: bool,
}

#[derive(Debug, Serialize)]
struct CleanOutput {
    removed: usize,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let removed = state::clean(&all_processes(), args.all)?;
    let output = CleanOutput { removed };
    if ctx.json() {
        return ctx.print_json(&output);
    }
    ui::success(&format!("removed {removed} remembered injection(s)"));
    Ok(())
}

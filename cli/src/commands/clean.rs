use clap::Args as ClapArgs;
use mono_injector_core::state::{self, CleanMode};
use serde::Serialize;

use crate::context::Context;
use crate::error::Result;
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
    let removed = state::clean_stale_records(clean_mode(args))?;
    let output = CleanOutput { removed };
    if ctx.json() {
        return ctx.print_json(&output);
    }
    ui::success(&format!("removed {removed} remembered injection(s)"));
    Ok(())
}

fn clean_mode(args: &Args) -> CleanMode {
    if args.all {
        CleanMode::All
    } else {
        CleanMode::Stale
    }
}

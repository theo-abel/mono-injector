use clap::{Parser, Subcommand};

use crate::commands::{clean, eject, inject, list, profile, status};
use crate::context::Context;
use crate::error::Result;

#[derive(Debug, Parser)]
#[command(name = "mono-injector", version, propagate_version = true)]
pub(crate) struct Cli {
    /// Emit machine-readable JSON output where supported.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inject a managed assembly into a target process.
    Inject(inject::Args),
    /// Eject a previously injected assembly from a target process.
    Eject(eject::Args),
    /// List running processes.
    List(list::Args),
    /// Show remembered injections.
    Status(status::Args),
    /// Remove stale remembered injections.
    Clean(clean::Args),
    /// Inspect profile configuration.
    Profile(profile::Args),
}

/// Routes a parsed command to its handler.
pub(crate) fn dispatch(cli: Cli) -> Result<()> {
    let ctx = Context::new(cli.json);
    match cli.command {
        Command::Inject(args) => inject::run(ctx, &args),
        Command::Eject(args) => eject::run(ctx, &args),
        Command::List(args) => list::run(ctx, &args),
        Command::Status(args) => status::run(ctx, &args),
        Command::Clean(args) => clean::run(ctx, &args),
        Command::Profile(args) => profile::run(ctx, &args),
    }
}

use clap::{Parser, Subcommand};

use crate::commands::{eject, inject, list};
use crate::error::Result;

#[derive(Debug, Parser)]
#[command(name = "mono-injector", version, propagate_version = true)]
pub(crate) struct Cli {
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
}

/// Routes a parsed command to its handler.
pub(crate) fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Inject(args) => inject::run(&args),
        Command::Eject(args) => eject::run(&args),
        Command::List(args) => {
            list::run(&args);
            Ok(())
        }
    }
}

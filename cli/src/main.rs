mod cli;
mod commands;
mod context;
mod dotnet;
mod error;
mod process;
mod profiles;
mod state;
mod ui;

use std::process::exit;

use clap::Parser;

use cli::{Cli, dispatch};

fn main() {
    if let Err(e) = dispatch(Cli::parse()) {
        ui::error(&e.to_string());
        exit(e.exit_code());
    }
}

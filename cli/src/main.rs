mod cli;
mod commands;
mod error;
mod process;
mod state;
mod ui;

use std::process::exit;

use clap::Parser;

use cli::{Cli, dispatch};

fn main() {
    if let Err(e) = dispatch(Cli::parse()) {
        ui::error(&e.to_string());
        exit(1);
    }
}

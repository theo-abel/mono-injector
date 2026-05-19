use clap::{Args as ClapArgs, Subcommand};
use mono_injector_core::profiles;

use crate::context::Context;
use crate::error::Result;
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List configured profiles.
    List,
    /// Show one configured profile.
    Show { name: String },
    /// Print the profiles file path.
    Path,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    match &args.command {
        Command::List => list(ctx),
        Command::Show { name } => show(ctx, name),
        Command::Path => path(ctx),
    }
}

fn list(ctx: Context) -> Result<()> {
    let profiles = profiles::list_profiles()?;
    if ctx.json() {
        return ctx.print_json(&profiles);
    }
    for profile in &profiles {
        println!("{}", profile.name);
    }
    Ok(())
}

fn show(ctx: Context, name: &str) -> Result<()> {
    let profile = profiles::get_profile(name)?;
    if ctx.json() {
        return ctx.print_json(&profile);
    }
    println!("{profile:#?}");
    Ok(())
}

fn path(ctx: Context) -> Result<()> {
    let path = profiles::profiles_path()?;
    if ctx.json() {
        return ctx.print_json(&path);
    }
    ui::label_value("Profiles:", &path.display().to_string());
    Ok(())
}

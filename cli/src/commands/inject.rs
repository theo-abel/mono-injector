use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::time::Duration;

use clap::Args as ClapArgs;
use mono_injector_core::operations::{
    InjectOptions, InjectOutput, ResolvedInjectPlan, inject, resolve_inject,
};

use super::{RuntimeArgs, profile_name};
use crate::context::Context;
use crate::error::{Error, Result};
use crate::ui;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Optional profile name.
    profile: Option<String>,

    /// Profile name alias for scripts that prefer flags.
    #[arg(long = "profile")]
    profile_alias: Option<String>,

    /// Target process id or exact process name.
    #[arg(short = 'p', long)]
    process: Option<String>,

    /// Managed assembly to load into the target process.
    #[arg(short = 'a', long)]
    assembly: Option<PathBuf>,

    /// Namespace containing the loader class.
    #[arg(short = 'n', long = "namespace")]
    namespace: Option<String>,

    /// Loader class name.
    #[arg(short = 'c', long = "class")]
    class_name: Option<String>,

    /// Loader method to invoke after loading the assembly.
    #[arg(short = 'm', long = "method")]
    method_name: Option<String>,

    /// Cleanup method recorded for later default ejection.
    #[arg(long)]
    eject_method: Option<String>,

    /// Wait for the target process before injecting.
    #[arg(long)]
    wait: bool,

    /// Seconds to wait for process/module readiness.
    #[arg(long, default_value_t = 120)]
    wait_timeout: u64,

    /// Milliseconds between process/module readiness checks.
    #[arg(long, default_value_t = 1_000)]
    poll_interval_ms: u64,

    /// Wait for a loaded module before injecting, for example UnityPlayer.dll.
    #[arg(long)]
    wait_module: Option<String>,

    /// Disable the default readiness-module wait used with --steam-app.
    #[arg(long)]
    no_wait_module: bool,

    /// Extra milliseconds to wait after readiness before injecting. Use 0 to disable.
    #[arg(long)]
    settle_ms: Option<u64>,

    /// Launch a Steam app before waiting for the process.
    #[arg(long)]
    steam_app: Option<u32>,

    /// Resolve inputs without calling Mono in the target process.
    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    runtime: RuntimeArgs,

    /// Command to run after successful injection. Pass after `--`.
    #[arg(last = true)]
    post_command: Vec<String>,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let options = args.options();
    if args.dry_run {
        return dry_run(ctx, &options);
    }
    let output = inject_with_spinner(&options)?;
    print_output_plan(ctx, &output);
    run_post_command(args, &output)?;
    print_success(ctx, &output)
}

impl Args {
    fn options(&self) -> InjectOptions {
        InjectOptions {
            profile_name: profile_name(self.profile.as_ref(), self.profile_alias.as_ref()),
            process: self.process.clone(),
            assembly: self.assembly.clone(),
            namespace: self.namespace.clone(),
            class_name: self.class_name.clone(),
            inject_method: self.method_name.clone(),
            eject_method: self.eject_method.clone(),
            wait_for_process: self.wait,
            wait_timeout: Duration::from_secs(self.wait_timeout),
            poll_interval: Duration::from_millis(self.poll_interval_ms),
            wait_module: self.wait_module.clone(),
            disable_wait_module: self.no_wait_module,
            settle_delay: self.settle_ms.map(Duration::from_millis),
            steam_app: self.steam_app,
            runtime: self.runtime.options(),
        }
    }
}

fn dry_run(ctx: Context, options: &InjectOptions) -> Result<()> {
    let plan = resolve_inject(options)?;
    print_plan(ctx, &plan);
    let output = plan.dry_run_output();
    if ctx.json() {
        ctx.print_json(&output)
    } else {
        ui::success("dry run completed");
        Ok(())
    }
}

fn inject_with_spinner(options: &InjectOptions) -> Result<InjectOutput> {
    let pb = ui::spinner();
    pb.set_message("injecting managed assembly...");
    let result = inject(options);
    pb.finish_and_clear();
    Ok(result?)
}

fn print_success(ctx: Context, output: &InjectOutput) -> Result<()> {
    if ctx.json() {
        ctx.print_json(output)
    } else {
        ui::success("injected successfully");
        if let Some(handle) = &output.handle {
            println!("{handle}");
        }
        Ok(())
    }
}

fn run_post_command(args: &Args, output: &InjectOutput) -> Result<()> {
    let Some((program, rest)) = args.post_command.split_first() else {
        return Ok(());
    };
    ProcessCommand::new(program)
        .args(rest)
        .env("MONO_INJECTOR_PROCESS", &output.process.name)
        .env("MONO_INJECTOR_PID", output.process.pid.to_string())
        .env(
            "MONO_INJECTOR_HANDLE",
            output.handle.as_deref().unwrap_or_default(),
        )
        .env("MONO_INJECTOR_ASSEMBLY", &output.assembly)
        .status()
        .map_err(Error::PostCommand)?;
    Ok(())
}

fn print_plan(ctx: Context, plan: &ResolvedInjectPlan) {
    if ctx.json() {
        return;
    }
    ui::label_value("Assembly:", &plan.assembly.display().to_string());
    ui::label_value(
        "Target:",
        &format!("{} ({})", plan.process.name, plan.process.pid),
    );
    ui::label_value("Entry:", &plan.entry);
}

fn print_output_plan(ctx: Context, output: &InjectOutput) {
    if ctx.json() {
        return;
    }
    ui::label_value("Assembly:", &output.assembly.display().to_string());
    ui::label_value(
        "Target:",
        &format!("{} ({})", output.process.name, output.process.pid),
    );
    ui::label_value("Entry:", &output.entry);
}

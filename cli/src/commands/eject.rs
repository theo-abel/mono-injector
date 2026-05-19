use clap::Args as ClapArgs;
use mono_injector_core::operations::{
    EjectOptions, EjectOutput, ResolvedEjectPlan, eject, resolve_eject,
};

use super::{RuntimeArgs, profile_name};
use crate::context::Context;
use crate::error::Result;
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

    /// Assembly handle returned by inject. Defaults to a matching remembered injection.
    #[arg(short = 'a', long = "assembly")]
    handle: Option<String>,

    /// Explicit unsafe handle mode; requires --force.
    #[arg(long, conflicts_with = "handle")]
    raw_handle: Option<String>,

    /// Namespace containing the loader class.
    #[arg(short = 'n', long = "namespace")]
    namespace: Option<String>,

    /// Loader class name.
    #[arg(short = 'c', long = "class")]
    class_name: Option<String>,

    /// Cleanup method to invoke before closing the assembly.
    #[arg(short = 'm', long = "method")]
    method_name: Option<String>,

    /// Use the latest matching remembered injection when several match.
    #[arg(long)]
    latest: bool,

    /// Bypass the local injection-record guard for advanced/manual ejection.
    #[arg(long)]
    force: bool,

    /// Resolve inputs without calling Mono in the target process.
    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    runtime: RuntimeArgs,
}

pub(crate) fn run(ctx: Context, args: &Args) -> Result<()> {
    let options = args.options();
    if args.dry_run {
        return dry_run(ctx, &options);
    }
    warn_if_forced(args);
    let output = eject(&options)?;
    print_output_plan(ctx, &output);
    finish(ctx, &output)
}

impl Args {
    fn options(&self) -> EjectOptions {
        EjectOptions {
            profile_name: profile_name(self.profile.as_ref(), self.profile_alias.as_ref()),
            process: self.process.clone(),
            handle: self.handle.clone(),
            raw_handle: self.raw_handle.clone(),
            namespace: self.namespace.clone(),
            class_name: self.class_name.clone(),
            method_name: self.method_name.clone(),
            latest: self.latest,
            force: self.force,
            runtime: self.runtime.options(),
        }
    }
}

fn dry_run(ctx: Context, options: &EjectOptions) -> Result<()> {
    let plan = resolve_eject(options)?;
    print_plan(ctx, &plan);
    let output = plan.dry_run_output();
    if ctx.json() {
        ctx.print_json(&output)
    } else {
        ui::success("dry run completed");
        Ok(())
    }
}

fn finish(ctx: Context, output: &EjectOutput) -> Result<()> {
    if ctx.json() {
        ctx.print_json(output)
    } else {
        ui::success("ejected successfully");
        Ok(())
    }
}

fn warn_if_forced(args: &Args) {
    if args.force || args.raw_handle.is_some() {
        ui::warn("bypassing local injection-record guard");
    }
}

fn print_plan(ctx: Context, plan: &ResolvedEjectPlan) {
    if ctx.json() {
        return;
    }
    ui::label_value(
        "Target:",
        &format!("{} ({})", plan.process.name, plan.process.pid),
    );
    ui::label_value("Assembly:", &plan.handle);
    ui::label_value("Entry:", &plan.entry);
}

fn print_output_plan(ctx: Context, output: &EjectOutput) {
    if ctx.json() {
        return;
    }
    ui::label_value(
        "Target:",
        &format!("{} ({})", output.process.name, output.process.pid),
    );
    ui::label_value("Assembly:", &output.handle);
    ui::label_value("Entry:", &output.entry);
}

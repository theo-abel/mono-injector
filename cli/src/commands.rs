pub(crate) mod clean;
pub(crate) mod eject;
pub(crate) mod inject;
pub(crate) mod list;
pub(crate) mod profile;
pub(crate) mod status;

use clap::Args;
use mono_injector::{Config, Injector};

use crate::process::ProcessInfo;
use crate::profiles::Profile;

#[derive(Debug, Args)]
pub(crate) struct RuntimeArgs {
    /// Remote-thread wait timeout in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    timeout_ms: u32,

    /// Case-insensitive fragment used to find the target Mono module.
    #[arg(long = "mono-module")]
    mono_module_hint: Option<String>,

    /// Base directory passed to `mono_assembly_load_from_full`.
    #[arg(long)]
    base_dir: Option<String>,
}

impl RuntimeArgs {
    pub(crate) fn config(&self, profile: Option<&Profile>) -> Config {
        let mut builder = Config::builder().timeout_ms(self.timeout_ms(profile));

        if let Some(hint) = self.mono_module(profile) {
            builder = builder.mono_module_hint(hint);
        }

        if let Some(dir) = self.base_dir(profile) {
            builder = builder.base_dir(dir);
        }

        builder.build()
    }

    fn timeout_ms(&self, profile: Option<&Profile>) -> u32 {
        profile
            .and_then(|p| p.timeout_ms)
            .unwrap_or(self.timeout_ms)
    }

    fn mono_module<'a>(&'a self, profile: Option<&'a Profile>) -> Option<&'a str> {
        self.mono_module_hint
            .as_deref()
            .or_else(|| profile.and_then(|p| p.mono_module.as_deref()))
    }

    fn base_dir<'a>(&'a self, profile: Option<&'a Profile>) -> Option<&'a str> {
        self.base_dir
            .as_deref()
            .or_else(|| profile.and_then(|p| p.base_dir.as_deref()))
    }
}

pub(crate) fn injector_for(
    process: &ProcessInfo,
    runtime: &RuntimeArgs,
    profile: Option<&Profile>,
) -> Injector {
    Injector::with_config(process.pid, runtime.config(profile))
}

pub(crate) fn profile_name(positional: Option<&String>, alias: Option<&String>) -> Option<String> {
    alias.cloned().or_else(|| positional.cloned())
}

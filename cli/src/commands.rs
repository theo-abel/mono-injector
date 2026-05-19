pub(crate) mod clean;
pub(crate) mod eject;
pub(crate) mod inject;
pub(crate) mod list;
pub(crate) mod profile;
pub(crate) mod status;

use clap::Args;
use mono_injector_core::runtime::RuntimeOptions;
use std::time::Duration;

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
    pub(crate) fn options(&self) -> RuntimeOptions {
        RuntimeOptions {
            timeout_ms: self.timeout_ms,
            mono_module_hint: self.mono_module_hint.clone(),
            base_dir: self.base_dir.clone(),
        }
    }
}

pub(crate) fn profile_name(positional: Option<&String>, alias: Option<&String>) -> Option<String> {
    alias.cloned().or_else(|| positional.cloned())
}

pub(crate) fn parse_duration_millis(raw: &str) -> std::result::Result<Duration, String> {
    parse_duration_with_default_unit(raw, "ms")
}

pub(crate) fn parse_duration_seconds(raw: &str) -> std::result::Result<Duration, String> {
    parse_duration_with_default_unit(raw, "s")
}

fn parse_duration_with_default_unit(
    raw: &str,
    unit: &str,
) -> std::result::Result<Duration, String> {
    let duration = if raw.chars().all(|c| c.is_ascii_digit()) {
        format!("{raw}{unit}")
    } else {
        raw.to_owned()
    };
    humantime::parse_duration(&duration).map_err(|error| error.to_string())
}

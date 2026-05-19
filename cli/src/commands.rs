pub(crate) mod eject;
pub(crate) mod inject;
pub(crate) mod list;

use clap::Args;
use mono_injector::{Config, Injector};

use crate::process::ProcessInfo;

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
    fn config(&self) -> Config {
        let mut builder = Config::builder().timeout_ms(self.timeout_ms);

        if let Some(hint) = &self.mono_module_hint {
            builder = builder.mono_module_hint(hint);
        }

        if let Some(dir) = &self.base_dir {
            builder = builder.base_dir(dir);
        }

        builder.build()
    }
}

pub(crate) fn injector_for(process: &ProcessInfo, runtime: &RuntimeArgs) -> Injector {
    Injector::with_config(process.pid, runtime.config())
}

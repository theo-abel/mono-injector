mod eject;
mod inject;
mod shared;

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::process::ProcessInfo;
use crate::runtime::RuntimeOptions;

pub use eject::{eject, resolve_eject};
pub use inject::{inject, resolve_inject};

/// Default loader class used when neither args nor profile provide one.
pub const DEFAULT_CLASS_NAME: &str = "Loader";
/// Default injection method used when neither args nor profile provide one.
pub const DEFAULT_INJECT_METHOD: &str = "Init";
/// Default ejection method used when neither args nor profile provide one.
pub const DEFAULT_EJECT_METHOD: &str = "Unload";
/// Default readiness module used for Steam-launched Unity games.
pub const DEFAULT_STEAM_WAIT_MODULE: &str = "d3d11.dll";
/// Default post-readiness delay used for Steam launches.
pub const DEFAULT_STEAM_SETTLE_MS: u64 = 8_000;

/// Profile-aware options for resolving or executing an injection.
#[derive(Debug, Clone)]
pub struct InjectOptions {
    pub profile_name: Option<String>,
    pub process: Option<String>,
    pub assembly: Option<PathBuf>,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
    pub inject_method: Option<String>,
    pub eject_method: Option<String>,
    pub wait_for_process: bool,
    pub wait_timeout: Duration,
    pub poll_interval: Duration,
    pub wait_module: Option<String>,
    pub disable_wait_module: bool,
    pub settle_delay: Option<Duration>,
    pub steam_app: Option<u32>,
    pub runtime: RuntimeOptions,
}

/// Profile-aware options for resolving or executing an ejection.
#[derive(Debug, Clone)]
pub struct EjectOptions {
    pub profile_name: Option<String>,
    pub process: Option<String>,
    pub handle: Option<String>,
    pub raw_handle: Option<String>,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub latest: bool,
    pub force: bool,
    pub runtime: RuntimeOptions,
}

/// Fully resolved injection plan suitable for dry-run output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedInjectPlan {
    pub process: ProcessInfo,
    pub assembly: PathBuf,
    pub namespace: String,
    pub class_name: String,
    pub method_name: String,
    pub eject_method: String,
    pub entry: String,
    pub wait_module: Option<String>,
    pub settle_ms: u64,
    pub steam_app: Option<u32>,
    pub profile: Option<String>,
}

/// Result returned after injection or dry-run planning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InjectOutput {
    pub status: String,
    pub process: ProcessInfo,
    pub assembly: PathBuf,
    pub entry: String,
    pub handle: Option<String>,
    pub profile: Option<String>,
}

/// Fully resolved ejection plan suitable for dry-run output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedEjectPlan {
    pub process: ProcessInfo,
    pub handle: String,
    pub namespace: String,
    pub class_name: String,
    pub method_name: String,
    pub entry: String,
    pub profile: Option<String>,
    pub raw: bool,
}

/// Result returned after ejection or dry-run planning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EjectOutput {
    pub status: String,
    pub process: ProcessInfo,
    pub handle: String,
    pub entry: String,
    pub profile: Option<String>,
}

impl ResolvedInjectPlan {
    /// Converts the plan into the dry-run output shape used by frontends.
    #[must_use]
    pub fn dry_run_output(self) -> InjectOutput {
        InjectOutput {
            status: "dry-run".to_owned(),
            process: self.process,
            assembly: self.assembly,
            entry: self.entry,
            handle: None,
            profile: self.profile,
        }
    }
}

impl ResolvedEjectPlan {
    /// Converts the plan into the dry-run output shape used by frontends.
    #[must_use]
    pub fn dry_run_output(self) -> EjectOutput {
        EjectOutput {
            status: "dry-run".to_owned(),
            process: self.process,
            handle: self.handle,
            entry: self.entry,
            profile: self.profile,
        }
    }
}

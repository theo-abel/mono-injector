use mono_injector::Injector;

use crate::error::{Error, Result};
use crate::process::ProcessInfo;
use crate::profiles::{Profile, get_profile};
use crate::runtime::RuntimeOptions;

use super::{EjectOptions, InjectOptions};

pub(super) fn injector_for(
    options: &impl RuntimeConfig,
    process: &ProcessInfo,
    profile: Option<&Profile>,
) -> Injector {
    Injector::with_config(process.pid, options.runtime().to_config(profile))
}

pub(super) trait RuntimeConfig {
    fn runtime(&self) -> &RuntimeOptions;
}

impl RuntimeConfig for InjectOptions {
    fn runtime(&self) -> &RuntimeOptions {
        &self.runtime
    }
}

impl RuntimeConfig for EjectOptions {
    fn runtime(&self) -> &RuntimeOptions {
        &self.runtime
    }
}

pub(super) fn load_profile(name: Option<&str>) -> Result<Option<Profile>> {
    name.map(get_profile).transpose()
}

pub(super) fn profile_process(profile: Option<&Profile>) -> Option<&String> {
    profile.and_then(|p| p.process.as_ref())
}

pub(super) fn required(
    value: Option<&String>,
    name: &'static str,
    flag: &'static str,
) -> Result<String> {
    value.cloned().ok_or(Error::MissingArgument { name, flag })
}

pub(super) fn duration_ms(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

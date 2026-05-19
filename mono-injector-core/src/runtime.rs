use mono_injector::Config;
use serde::{Deserialize, Serialize};

use crate::profiles::Profile;

/// Runtime settings that affect how the low-level injector finds Mono and waits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeOptions {
    pub timeout_ms: u32,
    pub mono_module_hint: Option<String>,
    pub base_dir: Option<String>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 5_000,
            mono_module_hint: None,
            base_dir: None,
        }
    }
}

impl RuntimeOptions {
    /// Converts profile-aware runtime options into the low-level injector config.
    #[must_use]
    pub fn to_config(&self, profile: Option<&Profile>) -> Config {
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

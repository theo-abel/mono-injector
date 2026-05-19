/// Session-level configuration for an injection operation.
///
/// Construct via [`Config::builder`]:
/// ```no_run
/// # use mono_injector::Config;
/// let cfg = Config::builder()
///     .timeout_ms(10_000)
///     .mono_module_hint("mono-2.0-bdwgc")
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Milliseconds passed to `WaitForSingleObject` for each remote call. Default: `5_000`.
    pub(crate) timeout_ms: u32,
    /// Case-insensitive fragment to match the Mono module path. Default: `"mono"`.
    pub(crate) mono_module_hint: String,
    /// Base directory hint for `mono_assembly_load_from_full`. Default: `""`.
    pub(crate) base_dir: String,
}

impl Config {
    /// Returns a builder with all defaults set.
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_ms: 5_000,
            mono_module_hint: "mono".to_owned(),
            base_dir: String::new(),
        }
    }
}

/// Builder for [`Config`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    /// Sets the remote thread wait timeout in milliseconds.
    #[must_use]
    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.0.timeout_ms = ms;
        self
    }

    /// Fragment matched (case-insensitive) against module paths to find the Mono DLL.
    ///
    /// Use `"mono-2.0-bdwgc"` for Unity 2018+ (Boehm GC) or `"mono"` for older Unity versions.
    #[must_use]
    pub fn mono_module_hint(mut self, hint: impl Into<String>) -> Self {
        self.0.mono_module_hint = hint.into();
        self
    }

    /// Base directory hint passed to `mono_assembly_load_from_full`.
    #[must_use]
    pub fn base_dir(mut self, dir: impl Into<String>) -> Self {
        self.0.base_dir = dir.into();
        self
    }

    /// Consumes the builder and returns the [`Config`].
    #[must_use]
    pub fn build(self) -> Config {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_timeout_is_5000() {
        assert_eq!(Config::default().timeout_ms, 5_000);
    }

    #[test]
    fn default_hint_is_mono() {
        assert_eq!(Config::default().mono_module_hint, "mono");
    }

    #[test]
    fn builder_overrides_all_fields() {
        let cfg = Config::builder()
            .timeout_ms(9_000)
            .mono_module_hint("mono-2.0-bdwgc")
            .base_dir("/tmp")
            .build();
        assert_eq!(cfg.timeout_ms, 9_000);
        assert_eq!(cfg.mono_module_hint, "mono-2.0-bdwgc");
        assert_eq!(cfg.base_dir, "/tmp");
    }
}

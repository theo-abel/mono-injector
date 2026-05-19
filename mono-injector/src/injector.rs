use crate::config::Config;
use crate::error::{Error, Result};
use crate::mono::api::{MonoSession, RemoteMonoApi};
use crate::mono::module::find_mono_module;
use crate::mono::{eject_steps, inject_steps};
use crate::pe;
use crate::process::memory;

/// An opaque handle to a managed assembly loaded in the target process.
///
/// Obtained from [`Injector::inject`] and required for a later [`Injector::eject`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssemblyHandle(u64);

impl AssemblyHandle {
    /// Creates a handle from a raw pointer value returned by the remote process.
    ///
    /// Returns `None` if `ptr` is zero (Mono returns null on failure).
    #[must_use]
    pub fn from_raw(ptr: u64) -> Option<Self> {
        if ptr == 0 { None } else { Some(Self(ptr)) }
    }

    pub(crate) fn new(ptr: u64) -> Option<Self> {
        Self::from_raw(ptr)
    }

    /// Returns the underlying pointer value, e.g. for serialising across process invocations.
    #[must_use]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    pub(crate) fn as_ptr(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for AssemblyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Parameters for a single injection.
#[derive(Debug, Clone, Copy)]
pub struct InjectRequest<'a> {
    pub assembly: &'a [u8],
    pub namespace: &'a str,
    pub class_name: &'a str,
    pub method_name: &'a str,
}

/// Parameters for ejecting a previously injected assembly.
#[derive(Debug, Clone, Copy)]
pub struct EjectRequest<'a> {
    pub handle: AssemblyHandle,
    pub namespace: &'a str,
    pub class_name: &'a str,
    pub method_name: &'a str,
}

/// Remote Mono assembly injector for a single Windows process.
///
/// # Examples
///
/// ```no_run
/// # use mono_injector::{Config, Injector, InjectRequest, EjectRequest};
/// let injector = Injector::with_config(12345, Config::builder()
///     .timeout_ms(10_000)
///     .mono_module_hint("mono-2.0-bdwgc")
///     .build());
///
/// # let dll_bytes: &[u8] = &[];
/// let handle = injector.inject(&InjectRequest {
///     assembly:    dll_bytes,
///     namespace:   "MyMod",
///     class_name:  "Loader",
///     method_name: "Initialize",
/// })?;
///
/// injector.eject(&EjectRequest {
///     handle,
///     namespace:   "MyMod",
///     class_name:  "Loader",
///     method_name: "Shutdown",
/// })?;
/// # Ok::<(), mono_injector::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Injector {
    pid: u32,
    config: Config,
}

impl Injector {
    /// Creates an injector for `pid` with default configuration.
    #[must_use]
    pub fn for_process(pid: u32) -> Self {
        Self {
            pid,
            config: Config::default(),
        }
    }

    /// Creates an injector for `pid` with a custom [`Config`].
    #[must_use]
    pub fn with_config(pid: u32, config: Config) -> Self {
        Self { pid, config }
    }

    /// Injects the assembly described by `req` and invokes its entry-point method.
    ///
    /// Returns an [`AssemblyHandle`] that identifies the loaded assembly; keep it to call
    /// [`eject`](Self::eject) later.
    ///
    /// # Errors
    ///
    /// See [`Error`] for all failure modes.
    pub fn inject(&self, req: &InjectRequest<'_>) -> Result<AssemblyHandle> {
        validate_inject_request(req)?;
        let session = open_session(self.pid, &self.config)?;
        inject_steps(session, req)
    }

    /// Ejects the assembly identified by `req.handle` and calls its cleanup method.
    ///
    /// # Errors
    ///
    /// See [`Error`] for all failure modes.
    pub fn eject(&self, req: &EjectRequest<'_>) -> Result<()> {
        validate_eject_request(req)?;
        let session = open_session(self.pid, &self.config)?;
        eject_steps(session, req)
    }
}

fn open_session(pid: u32, config: &Config) -> Result<MonoSession> {
    let process = memory::open(pid, config)?;
    let module = find_mono_module(&process, config)?;
    let exports = pe::parse_exports(&module)?;
    let api = RemoteMonoApi::resolve(&exports)?;

    Ok(MonoSession::new(api, process, config))
}

fn validate_inject_request(req: &InjectRequest<'_>) -> Result<()> {
    if req.assembly.is_empty() {
        return Err(Error::EmptyAssembly);
    }

    validate_strings(&[
        (req.namespace, "namespace"),
        (req.class_name, "class_name"),
        (req.method_name, "method_name"),
    ])
}

fn validate_eject_request(req: &EjectRequest<'_>) -> Result<()> {
    validate_strings(&[
        (req.namespace, "namespace"),
        (req.class_name, "class_name"),
        (req.method_name, "method_name"),
    ])
}

fn validate_strings(pairs: &[(&str, &str)]) -> Result<()> {
    for &(s, label) in pairs {
        if s.contains('\0') {
            return Err(Error::NullByteInString(label.to_owned()));
        }
    }
    Ok(())
}

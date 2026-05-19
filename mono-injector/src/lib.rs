//! Remote Mono assembly injector for Windows.
//!
//! Injects and ejects managed assemblies into Unity games and other Mono-hosted processes
//! **from outside** the target process: no DLL injection, no loader stub. Each Mono API call
//! is executed via a tiny x86/x64 shellcode stub run in a remote thread.
//!
//! # Quick start
//!
//! ```no_run
//! use mono_injector::{Config, EjectRequest, InjectRequest, Injector};
//!
//! let injector = Injector::with_config(
//!     12345,
//!     Config::builder().mono_module_hint("mono-2.0-bdwgc").build(),
//! );
//!
//! let handle = injector.inject(&InjectRequest {
//!     assembly:    &std::fs::read("MyMod.dll")?,
//!     namespace:   "MyMod",
//!     class_name:  "Loader",
//!     method_name: "Initialize",
//! })?;
//!
//! injector.eject(&EjectRequest {
//!     handle,
//!     namespace:   "MyMod",
//!     class_name:  "Loader",
//!     method_name: "Shutdown",
//! })?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod asm;
mod config;
mod error;
mod injector;
mod mono;
mod pe;
mod process;

pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use injector::{AssemblyHandle, EjectRequest, InjectRequest, Injector};

/// Commonly used types as a single glob import.
///
/// ```rust,no_run
/// use mono_injector::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        AssemblyHandle, Config, ConfigBuilder, EjectRequest, Error, InjectRequest, Injector, Result,
    };
}

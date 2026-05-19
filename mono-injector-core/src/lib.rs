//! Reusable application services for Mono injection workflows.
//!
//! This crate sits above the low-level `mono-injector` crate and below any UI.
//! It owns profile loading, local injection state, process discovery, metadata
//! inspection, and inject/eject orchestration without terminal output.

pub mod dotnet;
pub mod error;
pub mod operations;
pub mod process;
pub mod profiles;
pub mod runtime;
pub mod state;

pub use error::{Error, Result};

/// Common imports for frontends that use the service layer.
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::operations::{
        EjectOptions, EjectOutput, InjectOptions, InjectOutput, ResolvedEjectPlan,
        ResolvedInjectPlan, eject, inject, resolve_eject, resolve_inject,
    };
    pub use crate::process::{ListOptions, ModuleFilter, ProcessInfo, ProcessListing};
    pub use crate::profiles::{Profile, ProfileSummary, ProfilesFile};
    pub use crate::runtime::RuntimeOptions;
    pub use crate::state::{CleanMode, InjectionInput, InjectionRecord};
}

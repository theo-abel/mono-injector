use std::fs;

use goblin::pe::PE;

use crate::error::{Error, Result};
use crate::mono::module::MonoModule;

/// A single named export from the target's Mono module.
#[derive(Debug, Clone)]
pub(crate) struct Export {
    pub(crate) name: String,
    /// Absolute virtual address in the target process: `module_base + rva`.
    pub(crate) address: u64,
}

/// Reads the Mono DLL file from disk and extracts all named exports with their in-process VAs.
///
/// Reading the on-disk file (not the in-memory mapping) lets goblin resolve RVAs correctly
/// without dealing with section-alignment differences in a mapped PE image.
///
/// # Errors
///
/// Returns `Error::ModuleFileRead` if the file cannot be read, or `Error::PeParse` if goblin
/// fails to parse the PE headers.
pub(crate) fn parse_exports(module: &MonoModule) -> Result<Vec<Export>> {
    let bytes = fs::read(&module.path).map_err(|e| Error::ModuleFileRead {
        path: module.path.clone(),
        source: e,
    })?;

    let pe = PE::parse(&bytes).map_err(|e| Error::PeParse(e.to_string()))?;

    let exports = pe
        .exports
        .iter()
        .filter_map(|e| {
            e.name.map(|name| Export {
                name: name.to_owned(),
                address: module.base + e.rva as u64,
            })
        })
        .collect();

    Ok(exports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn kernel32_module() -> Option<MonoModule> {
        let path = PathBuf::from(r"C:\Windows\System32\kernel32.dll");
        if path.exists() {
            Some(MonoModule {
                base: 0x7fff_0000_0000,
                path,
            })
        } else {
            None
        }
    }

    #[test]
    fn parses_known_exports_from_kernel32() {
        let Some(module) = kernel32_module() else {
            return;
        };
        let exports = parse_exports(&module).expect("parse should succeed");
        assert!(!exports.is_empty(), "kernel32 has exports");
        let has_load_lib = exports.iter().any(|e| e.name == "LoadLibraryA");
        assert!(has_load_lib, "LoadLibraryA must be present");
    }

    #[test]
    fn addresses_are_nonzero_with_nonzero_base() {
        let Some(module) = kernel32_module() else {
            return;
        };
        let exports = parse_exports(&module).expect("parse should succeed");
        assert!(
            exports.iter().all(|e| e.address >= module.base),
            "all addresses must be >= module base"
        );
    }
}

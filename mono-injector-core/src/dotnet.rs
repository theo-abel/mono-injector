use std::collections::BTreeMap;

use dotnetdll::prelude::{ReadOptions, Resolution};

use crate::error::{Error, Result};

/// Validates that the assembly can be inspected as .NET metadata.
///
/// # Errors
///
/// Returns an error when the bytes are not readable by `dotnetdll`.
pub fn validate_assembly(assembly: &[u8]) -> Result<()> {
    parse(assembly).map(|_| ())
}

/// Infers the namespace for a loader class from .NET metadata.
///
/// The class-specific namespace is preferred. If the class is absent or ambiguous,
/// the most common non-empty namespace in the assembly is used.
///
/// # Errors
///
/// Returns an error when the assembly metadata cannot be parsed.
pub fn infer_namespace(assembly: &[u8], class_name: &str) -> Result<Option<String>> {
    let resolution = parse(assembly)?;
    Ok(namespace_for_class(&resolution, class_name)
        .or_else(|| most_common_namespace(namespace_counts(&resolution))))
}

fn parse(assembly: &[u8]) -> Result<Resolution<'_>> {
    Resolution::parse(assembly, ReadOptions::default())
        .map_err(|source| Error::AssemblyMetadata(source.to_string()))
}

fn namespace_for_class(resolution: &Resolution<'_>, class_name: &str) -> Option<String> {
    let mut matches = resolution
        .type_definitions
        .iter()
        .filter(|type_def| type_def.name == class_name)
        .filter_map(|type_def| type_def.namespace.as_deref())
        .filter(|namespace| !namespace.is_empty());
    let namespace = matches.next()?;
    matches.next().is_none().then(|| namespace.to_owned())
}

fn namespace_counts(resolution: &Resolution<'_>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for type_def in &resolution.type_definitions {
        add_namespace_count(&mut counts, type_def.namespace.as_deref());
    }
    counts
}

fn add_namespace_count(counts: &mut BTreeMap<String, usize>, namespace: Option<&str>) {
    if let Some(namespace) = namespace
        && !namespace.is_empty()
    {
        *counts.entry(namespace.to_owned()).or_default() += 1;
    }
}

fn most_common_namespace(counts: BTreeMap<String, usize>) -> Option<String> {
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(namespace, _)| namespace)
}

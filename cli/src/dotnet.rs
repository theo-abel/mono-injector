use std::collections::BTreeMap;

use dotnetdll::prelude::{ReadOptions, Resolution};

#[must_use]
pub(crate) fn infer_namespace(assembly: &[u8], class_name: &str) -> Option<String> {
    let resolution = Resolution::parse(assembly, ReadOptions::default()).ok()?;
    namespace_for_class(&resolution, class_name)
        .or_else(|| most_common_namespace(namespace_counts(&resolution)))
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
        if let Some(namespace) = type_def.namespace.as_deref()
            && !namespace.is_empty()
        {
            *counts.entry(namespace.to_owned()).or_default() += 1;
        }
    }
    counts
}

fn most_common_namespace(counts: BTreeMap<String, usize>) -> Option<String> {
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(namespace, _)| namespace)
}

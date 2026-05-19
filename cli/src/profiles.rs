use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const CONFIG_DIR: &str = "mono-injector";
const PROFILES_FILE: &str = "profiles.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Profile {
    pub(crate) process: Option<String>,
    pub(crate) assembly: Option<PathBuf>,
    pub(crate) namespace: Option<String>,
    #[serde(rename = "class")]
    pub(crate) class_name: Option<String>,
    pub(crate) inject_method: Option<String>,
    pub(crate) eject_method: Option<String>,
    pub(crate) mono_module: Option<String>,
    pub(crate) base_dir: Option<String>,
    pub(crate) timeout_ms: Option<u32>,
    pub(crate) wait_module: Option<String>,
    pub(crate) steam_app: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ProfilesFile {
    #[serde(default)]
    pub(crate) profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProfileSummary {
    pub(crate) name: String,
    pub(crate) profile: Profile,
}

pub(crate) fn load() -> Result<ProfilesFile> {
    match fs::read_to_string(path()) {
        Ok(content) => toml::from_str(&content).map_err(Error::ProfilesParse),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ProfilesFile::default()),
        Err(e) => Err(Error::Profiles(e)),
    }
}

pub(crate) fn get(name: &str) -> Result<Profile> {
    load()?
        .profiles
        .get(name)
        .cloned()
        .ok_or_else(|| Error::ProfileNotFound(name.to_owned()))
}

pub(crate) fn list() -> Result<Vec<ProfileSummary>> {
    Ok(load()?
        .profiles
        .into_iter()
        .map(|(name, profile)| ProfileSummary { name, profile })
        .collect())
}

#[must_use]
pub(crate) fn path() -> PathBuf {
    config_dir().join(CONFIG_DIR).join(PROFILES_FILE)
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(std::env::temp_dir)
}

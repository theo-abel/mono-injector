use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const CONFIG_DIR: &str = "mono-injector";
const PROFILES_FILE: &str = "profiles.toml";

/// A named set of injection defaults shared by command-line and graphical UIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub process: Option<String>,
    pub assembly: Option<PathBuf>,
    pub namespace: Option<String>,
    #[serde(rename = "class")]
    pub class_name: Option<String>,
    pub inject_method: Option<String>,
    pub eject_method: Option<String>,
    pub mono_module: Option<String>,
    pub base_dir: Option<String>,
    pub timeout_ms: Option<u32>,
    pub wait_module: Option<String>,
    pub settle_ms: Option<u64>,
    pub steam_app: Option<u32>,
}

/// On-disk profile document loaded from the user configuration directory.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProfilesFile {
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

/// Serializable profile list entry used by frontends.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSummary {
    pub name: String,
    pub profile: Profile,
}

/// Loads all configured profiles, returning an empty file if none exists.
///
/// # Errors
///
/// Returns an error when the profile file cannot be read or parsed.
pub fn load_profiles() -> Result<ProfilesFile> {
    match fs::read_to_string(profiles_path()?) {
        Ok(content) => toml::from_str(&content).map_err(Error::ProfilesParse),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ProfilesFile::default()),
        Err(e) => Err(Error::Profiles(e)),
    }
}

/// Loads one profile by name.
///
/// # Errors
///
/// Returns an error when profiles cannot be loaded or the name is unknown.
pub fn get_profile(name: &str) -> Result<Profile> {
    load_profiles()?
        .profiles
        .get(name)
        .cloned()
        .ok_or_else(|| Error::ProfileNotFound(name.to_owned()))
}

/// Lists configured profiles in deterministic name order.
///
/// # Errors
///
/// Returns an error when the profile file cannot be loaded.
pub fn list_profiles() -> Result<Vec<ProfileSummary>> {
    Ok(load_profiles()?
        .profiles
        .into_iter()
        .map(|(name, profile)| ProfileSummary { name, profile })
        .collect())
}

/// Persists the given profiles file to the user configuration directory.
///
/// # Errors
///
/// Returns an error when the configuration directory is unavailable, or the
/// file cannot be serialized or written.
pub fn save_profiles(file: &ProfilesFile) -> Result<()> {
    let path = profiles_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(Error::Profiles)?;
    }
    let content =
        toml::to_string_pretty(file).map_err(|e| Error::ProfilesSerialize(e.to_string()))?;
    fs::write(path, content).map_err(Error::Profiles)
}

/// Returns the default profile file path for the current user.
///
/// # Errors
///
/// Returns an error when the operating system does not expose a user configuration directory.
pub fn profiles_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_DIR).join(PROFILES_FILE))
}

fn config_dir() -> Result<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.config_dir().to_owned())
        .ok_or(Error::UserDirectoryUnavailable { kind: "config" })
}

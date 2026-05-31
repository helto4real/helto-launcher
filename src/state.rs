use crate::error::{LauncherError, Result};
use crate::favorites::Favorites;
use crate::APP_NAME;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const STATE_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LauncherState {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub favorites: Favorites,
    #[serde(default)]
    pub launch_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub last_selected_app: Option<String>,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            favorites: Favorites::default(),
            launch_counts: BTreeMap::new(),
            last_selected_app: None,
        }
    }
}

impl LauncherState {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(path).map_err(|source| LauncherError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;

        toml::from_str(&contents).map_err(|source| LauncherError::ParseToml {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| LauncherError::WriteFile {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(|source| LauncherError::SerializeToml {
                path: path.to_path_buf(),
                source,
            })?;

        std::fs::write(path, contents).map_err(|source| LauncherError::WriteFile {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn record_launch(&mut self, app_id: &str) {
        *self.launch_counts.entry(app_id.to_string()).or_default() += 1;
        self.last_selected_app = Some(app_id.to_string());
    }
}

pub fn state_path() -> PathBuf {
    state_path_with_env(|key| std::env::var_os(key))
}

pub fn state_path_with_env<F>(get_env: F) -> PathBuf
where
    F: Fn(&str) -> Option<std::ffi::OsString>,
{
    if let Some(state_home) = get_env("XDG_STATE_HOME") {
        return PathBuf::from(state_home).join(APP_NAME).join("state.toml");
    }

    let home = get_env("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/state").join(APP_NAME).join("state.toml")
}

fn default_version() -> u32 {
    STATE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn serializes_and_deserializes_state() {
        let mut state = LauncherState::default();
        state.favorites.add("firefox.desktop");
        state.record_launch("firefox.desktop");

        let encoded = toml::to_string(&state).unwrap();
        let decoded: LauncherState = toml::from_str(&encoded).unwrap();

        assert_eq!(decoded.version, STATE_VERSION);
        assert_eq!(decoded.favorites.items, ["firefox.desktop"]);
        assert_eq!(decoded.launch_counts["firefox.desktop"], 1);
    }

    #[test]
    fn uses_xdg_state_home_when_available() {
        let path = state_path_with_env(|key| match key {
            "XDG_STATE_HOME" => Some(OsString::from("/tmp/state")),
            "HOME" => Some(OsString::from("/home/me")),
            _ => None,
        });

        assert_eq!(path, PathBuf::from("/tmp/state/helto-launcher/state.toml"));
    }

    #[test]
    fn falls_back_to_local_state() {
        let path = state_path_with_env(|key| match key {
            "HOME" => Some(OsString::from("/home/me")),
            _ => None,
        });

        assert_eq!(
            path,
            PathBuf::from("/home/me/.local/state/helto-launcher/state.toml")
        );
    }
}

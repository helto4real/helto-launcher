use crate::error::{LauncherError, Result};
use crate::APP_NAME;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub commands: PowerCommands,
    #[serde(default)]
    pub privileged_apps: BTreeMap<String, bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            commands: PowerCommands::default(),
            privileged_apps: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PowerCommands {
    #[serde(default = "default_logout")]
    pub logout: Vec<String>,
    #[serde(default = "default_restart")]
    pub restart: Vec<String>,
    #[serde(default = "default_poweroff")]
    pub poweroff: Vec<String>,
}

impl Default for PowerCommands {
    fn default() -> Self {
        Self {
            logout: default_logout(),
            restart: default_restart(),
            poweroff: default_poweroff(),
        }
    }
}

impl Config {
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

    pub fn is_privileged(&self, app_id: &str) -> bool {
        self.privileged_apps.get(app_id).copied().unwrap_or(false)
    }
}

pub fn config_path() -> PathBuf {
    config_path_with_env(|key| std::env::var_os(key))
}

pub fn config_path_with_env<F>(get_env: F) -> PathBuf
where
    F: Fn(&str) -> Option<std::ffi::OsString>,
{
    if let Some(config_home) = get_env("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home)
            .join(APP_NAME)
            .join("config.toml");
    }

    let home = get_env("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join(APP_NAME).join("config.toml")
}

fn default_theme() -> String {
    "catppuccin-mocha".to_string()
}

fn default_logout() -> Vec<String> {
    vec!["systemctl".into(), "--user".into(), "exit".into()]
}

fn default_restart() -> Vec<String> {
    vec!["systemctl".into(), "reboot".into()]
}

fn default_poweroff() -> Vec<String> {
    vec!["systemctl".into(), "poweroff".into()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn default_config_has_expected_commands() {
        let config = Config::default();

        assert_eq!(config.theme, "catppuccin-mocha");
        assert_eq!(config.commands.logout, ["systemctl", "--user", "exit"]);
        assert_eq!(config.commands.restart, ["systemctl", "reboot"]);
        assert_eq!(config.commands.poweroff, ["systemctl", "poweroff"]);
    }

    #[test]
    fn parses_privileged_apps() {
        let config: Config = toml::from_str(
            r#"
theme = "catppuccin-mocha"

[privileged_apps]
"org.example.AdminTool.desktop" = true
"normal.desktop" = false
"#,
        )
        .unwrap();

        assert!(config.is_privileged("org.example.AdminTool.desktop"));
        assert!(!config.is_privileged("normal.desktop"));
    }

    #[test]
    fn uses_xdg_config_home_when_available() {
        let path = config_path_with_env(|key| match key {
            "XDG_CONFIG_HOME" => Some(OsString::from("/tmp/config")),
            "HOME" => Some(OsString::from("/home/me")),
            _ => None,
        });

        assert_eq!(
            path,
            PathBuf::from("/tmp/config/helto-launcher/config.toml")
        );
    }
}

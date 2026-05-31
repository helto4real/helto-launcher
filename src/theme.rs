use crate::error::{LauncherError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Theme {
    pub name: String,
    pub window_background: String,
    pub panel_background: String,
    pub search_background: String,
    pub search_text: String,
    pub list_item_background: String,
    pub list_item_hover: String,
    pub list_item_selected: String,
    pub text: String,
    pub muted_text: String,
    pub accent: String,
    pub warning: String,
    pub error: String,
    pub border: String,
    pub favorite_active: String,
    pub favorite_inactive: String,
}

impl Theme {
    pub fn load(theme_name: &str, theme_dirs: &[PathBuf]) -> Result<Self> {
        for dir in theme_dirs {
            let path = dir.join(format!("{theme_name}.toml"));
            if path.exists() {
                return Self::load_path(theme_name, &path);
            }
        }

        Err(LauncherError::Theme {
            theme: theme_name.to_string(),
            reason: "theme file was not found".to_string(),
        })
    }

    pub fn load_path(theme_name: &str, path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|source| LauncherError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        let theme: Self = toml::from_str(&contents).map_err(|source| LauncherError::ParseToml {
            path: path.to_path_buf(),
            source,
        })?;
        theme.validate(theme_name)?;
        Ok(theme)
    }

    pub fn validate(&self, theme_name: &str) -> Result<()> {
        let colors = [
            ("window_background", &self.window_background),
            ("panel_background", &self.panel_background),
            ("search_background", &self.search_background),
            ("search_text", &self.search_text),
            ("list_item_background", &self.list_item_background),
            ("list_item_hover", &self.list_item_hover),
            ("list_item_selected", &self.list_item_selected),
            ("text", &self.text),
            ("muted_text", &self.muted_text),
            ("accent", &self.accent),
            ("warning", &self.warning),
            ("error", &self.error),
            ("border", &self.border),
            ("favorite_active", &self.favorite_active),
            ("favorite_inactive", &self.favorite_inactive),
        ];

        for (name, value) in colors {
            if !is_hex_color(value) {
                return Err(LauncherError::Theme {
                    theme: theme_name.to_string(),
                    reason: format!("missing or invalid color `{name}`"),
                });
            }
        }

        Ok(())
    }

    pub fn css(&self) -> String {
        format!(
            r#"
.launcher-window {{
    background-color: {window_background};
    color: {text};
}}

.launcher-root {{
    background-color: {window_background};
    border: 1px solid {border};
    border-radius: 16px;
    padding: 8px;
}}

.launcher-content,
.launcher-panel,
.launcher-bottom,
.launcher-actions-panel,
.launcher-results-frame,
.launcher-results-frame viewport,
.launcher-results {{
    background-color: {window_background};
    color: {text};
}}

.launcher-panel {{
    background-color: {panel_background};
    border: 1px solid {panel_background};
    border-radius: 8px;
    padding: 8px 7px;
}}

.launcher-actions-panel {{
    background-color: {panel_background};
    border: 1px solid {border};
    border-radius: 10px;
    padding: 7px 8px;
    min-height: 44px;
}}

.launcher-search {{
    background-color: {search_background};
    background-image: none;
    color: {search_text};
    border: 1px solid {border};
    border-radius: 10px;
    padding: 8px 10px;
}}

.launcher-search text,
.launcher-search selection {{
    color: {search_text};
}}

.launcher-result-row {{
    background-color: transparent;
    background-image: none;
    padding: 0;
}}

.launcher-result-surface,
.launcher-row {{
    background-color: {list_item_background};
    background-image: none;
    color: {text};
    border-radius: 8px;
    padding: 6px 10px;
}}

.launcher-results row:hover .launcher-result-surface,
.launcher-row:hover {{
    background-color: {list_item_hover};
}}

.launcher-results row:selected,
.launcher-results row:selected .launcher-result-surface,
.launcher-results row:selected .launcher-row,
.launcher-row:selected {{
    background-image: none;
    border-radius: 8px;
}}

.launcher-results row:selected .launcher-result-surface,
.launcher-results row:selected .launcher-row,
.launcher-row:selected {{
    background-color: {list_item_selected};
    color: {text};
}}

.launcher-row-actions {{
    min-width: 48px;
}}

button.launcher-button {{
    background-color: {search_background};
    background-image: none;
    color: {text};
    border: 1px solid {border};
    border-radius: 9px;
    padding: 6px 12px;
    box-shadow: none;
    text-shadow: none;
}}

button.launcher-button label {{
    color: {text};
    text-shadow: none;
}}

button.launcher-button:hover {{
    background-color: {list_item_hover};
    background-image: none;
    border-color: {accent};
}}

button.launcher-button:active {{
    background-color: {list_item_selected};
    background-image: none;
}}

button.launcher-button:disabled {{
    background-color: {panel_background};
    background-image: none;
    color: {muted_text};
    border-color: {border};
    opacity: 0.55;
}}

button.launcher-button:disabled label {{
    color: {muted_text};
}}

button.launcher-icon-button {{
    min-width: 34px;
    min-height: 34px;
    margin: 0;
    padding: 4px;
}}

button.launcher-power-button {{
    min-width: 86px;
    font-weight: 600;
}}

button.launcher-logout-button {{
    background-color: alpha({accent}, 0.18);
    border-color: alpha({accent}, 0.55);
}}

button.launcher-logout-button label {{
    color: {accent};
}}

button.launcher-restart-button {{
    background-color: alpha({warning}, 0.14);
    border-color: alpha({warning}, 0.50);
}}

button.launcher-restart-button label {{
    color: {warning};
}}

button.launcher-poweroff-button {{
    background-color: alpha({error}, 0.14);
    border-color: alpha({error}, 0.50);
}}

button.launcher-poweroff-button label {{
    color: {error};
}}

button.launcher-favorite-button {{
    background-color: alpha({accent}, 0.14);
    border-color: alpha({accent}, 0.45);
    color: {accent};
}}

button.launcher-favorite-button label {{
    color: {accent};
}}

button.launcher-favorite-slot {{
    background-color: transparent;
    background-image: none;
    border: 1px solid transparent;
    border-radius: 10px;
    min-width: 48px;
    min-height: 48px;
    padding: 2px;
    box-shadow: none;
}}

button.launcher-favorite-slot:hover {{
    background-color: alpha({accent}, 0.12);
    background-image: none;
    border-color: alpha({accent}, 0.35);
}}

.launcher-favorite-tile,
.launcher-favorite-placeholder {{
    background-color: alpha({search_background}, 0.50);
    border: 1px solid alpha({border}, 0.65);
    border-radius: 10px;
    min-width: 42px;
    min-height: 42px;
}}

.launcher-favorite-placeholder {{
    background-color: alpha({panel_background}, 0.55);
    border-color: alpha({border}, 0.40);
}}

.launcher-favorite-empty {{
    min-width: 48px;
    min-height: 48px;
    margin: 2px;
}}

.launcher-favorite-badge {{
    background-color: {accent};
    color: {window_background};
    border-radius: 999px;
    min-width: 17px;
    min-height: 17px;
    padding: 0;
    margin: 0;
    font-size: 10px;
    font-weight: 700;
}}

.launcher-muted {{
    color: {muted_text};
}}

.launcher-accent {{
    color: {accent};
}}

.launcher-favorite-active {{
    color: {favorite_active};
}}

.launcher-favorite-inactive {{
    color: {favorite_inactive};
}}

.launcher-danger {{
    color: {error};
}}

.launcher-warning {{
    color: {warning};
}}
"#,
            window_background = self.window_background,
            panel_background = self.panel_background,
            search_background = self.search_background,
            search_text = self.search_text,
            list_item_background = self.list_item_background,
            list_item_hover = self.list_item_hover,
            list_item_selected = self.list_item_selected,
            text = self.text,
            muted_text = self.muted_text,
            accent = self.accent,
            warning = self.warning,
            error = self.error,
            border = self.border,
            favorite_active = self.favorite_active,
            favorite_inactive = self.favorite_inactive,
        )
    }
}

pub fn theme_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        dirs.push(
            PathBuf::from(config_home)
                .join(crate::APP_NAME)
                .join("themes"),
        );
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(
            PathBuf::from(home)
                .join(".config")
                .join(crate::APP_NAME)
                .join("themes"),
        );
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(repo_theme_dir) = theme_dir_from_exe(&current_exe) {
            push_unique(&mut dirs, repo_theme_dir);
        }
    }
    dirs.push(PathBuf::from("themes"));
    dirs
}

fn theme_dir_from_exe(exe: &Path) -> Option<PathBuf> {
    let profile_dir = exe.parent()?;
    let target_dir = profile_dir.parent()?;
    if target_dir.file_name()? != "target" {
        return None;
    }

    let profile = profile_dir.file_name()?.to_str()?;
    if !matches!(profile, "debug" | "release") {
        return None;
    }

    Some(target_dir.parent()?.join("themes"))
}

fn push_unique(values: &mut Vec<PathBuf>, value: PathBuf) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn is_hex_color(value: &str) -> bool {
    let value = value.strip_prefix('#').unwrap_or(value);
    matches!(value.len(), 6 | 8) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_theme() -> Theme {
        Theme {
            name: "Catppuccin Mocha".into(),
            window_background: "#1e1e2e".into(),
            panel_background: "#181825".into(),
            search_background: "#313244".into(),
            search_text: "#cdd6f4".into(),
            list_item_background: "#1e1e2e".into(),
            list_item_hover: "#313244".into(),
            list_item_selected: "#45475a".into(),
            text: "#cdd6f4".into(),
            muted_text: "#a6adc8".into(),
            accent: "#89b4fa".into(),
            warning: "#f9e2af".into(),
            error: "#f38ba8".into(),
            border: "#45475a".into(),
            favorite_active: "#f5c2e7".into(),
            favorite_inactive: "#6c7086".into(),
        }
    }

    #[test]
    fn validates_theme_colors() {
        valid_theme().validate("catppuccin-mocha").unwrap();
    }

    #[test]
    fn rejects_invalid_theme_color() {
        let mut theme = valid_theme();
        theme.accent = "blue".into();

        let err = theme.validate("catppuccin-mocha").unwrap_err();
        assert!(err.to_string().contains("accent"));
    }

    #[test]
    fn derives_theme_dir_from_release_binary_path() {
        let path = Path::new("/home/thhel/git/helto-launcher/target/release/helto-launcher");

        assert_eq!(
            theme_dir_from_exe(path),
            Some(PathBuf::from("/home/thhel/git/helto-launcher/themes"))
        );
    }

    #[test]
    fn derives_theme_dir_from_debug_binary_path() {
        let path = Path::new("/home/thhel/git/helto-launcher/target/debug/helto-launcher");

        assert_eq!(
            theme_dir_from_exe(path),
            Some(PathBuf::from("/home/thhel/git/helto-launcher/themes"))
        );
    }

    #[test]
    fn ignores_non_cargo_binary_path() {
        let path = Path::new("/usr/bin/helto-launcher");

        assert_eq!(theme_dir_from_exe(path), None);
    }
}

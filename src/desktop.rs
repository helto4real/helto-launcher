use crate::error::{LauncherError, Result};
use crate::exec::{parse_exec, CommandLine, ExecContext};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopApp {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub command: CommandLine,
    pub icon: Option<String>,
    pub terminal: bool,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
}

pub fn parse_desktop_entry(
    id: String,
    path: PathBuf,
    contents: &str,
) -> Result<Option<DesktopApp>> {
    let fields = parse_desktop_fields(contents);

    if bool_field(fields.get("NoDisplay")) || bool_field(fields.get("Hidden")) {
        return Ok(None);
    }

    let name = required_field(&fields, "Name", &path)?;
    let exec = required_field(&fields, "Exec", &path)?;
    let icon = fields
        .get("Icon")
        .cloned()
        .filter(|value| !value.is_empty());
    let context = ExecContext {
        name: &name,
        icon: icon.as_deref(),
        desktop_path: &path,
    };
    let command = parse_exec(&exec, &context)?;

    Ok(Some(DesktopApp {
        id,
        path,
        name,
        generic_name: fields
            .get("GenericName")
            .cloned()
            .filter(|value| !value.is_empty()),
        comment: fields
            .get("Comment")
            .cloned()
            .filter(|value| !value.is_empty()),
        exec,
        command,
        icon,
        terminal: bool_field(fields.get("Terminal")),
        categories: split_list(fields.get("Categories")),
        keywords: split_list(fields.get("Keywords")),
    }))
}

fn parse_desktop_fields(contents: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    let mut in_desktop_entry = false;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if !key.contains('[') {
            fields.insert(key.to_string(), value.trim().to_string());
        }
    }

    fields
}

fn required_field(fields: &BTreeMap<String, String>, key: &str, path: &Path) -> Result<String> {
    fields
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| LauncherError::DesktopEntry {
            path: path.to_path_buf(),
            reason: format!("missing required `{key}` field"),
        })
}

fn bool_field(value: Option<&String>) -> bool {
    value.is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn split_list(value: Option<&String>) -> Vec<String> {
    value
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_desktop_entry() {
        let app = parse_desktop_entry(
            "firefox.desktop".to_string(),
            PathBuf::from("/usr/share/applications/firefox.desktop"),
            r#"
[Desktop Entry]
Name=Firefox
GenericName=Web Browser
Comment=Browse the Web
Exec=firefox %u
Icon=firefox
Terminal=false
Categories=Network;WebBrowser;
Keywords=web;browser;
"#,
        )
        .unwrap()
        .unwrap();

        assert_eq!(app.name, "Firefox");
        assert_eq!(app.command.as_vec(), ["firefox"]);
        assert_eq!(app.categories, ["Network", "WebBrowser"]);
        assert_eq!(app.keywords, ["web", "browser"]);
    }

    #[test]
    fn skips_hidden_entries() {
        let app = parse_desktop_entry(
            "hidden.desktop".to_string(),
            PathBuf::from("hidden.desktop"),
            "[Desktop Entry]\nName=Hidden\nExec=hidden\nHidden=true\n",
        )
        .unwrap();

        assert!(app.is_none());
    }

    #[test]
    fn fails_without_name() {
        let err = parse_desktop_entry(
            "invalid.desktop".to_string(),
            PathBuf::from("invalid.desktop"),
            "[Desktop Entry]\nExec=invalid\n",
        )
        .unwrap_err();

        assert!(err.to_string().contains("missing required `Name`"));
    }
}

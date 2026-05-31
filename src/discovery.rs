use crate::desktop::{parse_desktop_entry, DesktopApp};
use crate::error::{LauncherError, Result};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use tracing::debug;

pub fn discover_apps() -> Vec<DesktopApp> {
    discover_apps_from_dirs(&application_dirs())
}

pub fn discover_apps_from_dirs(dirs: &[PathBuf]) -> Vec<DesktopApp> {
    let mut apps = BTreeMap::new();

    for dir in dirs {
        for path in desktop_files(dir) {
            let id = desktop_file_id(dir, &path);
            match read_desktop_app(id.clone(), path) {
                Ok(Some(app)) => {
                    apps.entry(id).or_insert(app);
                }
                Ok(None) => {}
                Err(error) => debug!("{error}"),
            }
        }
    }

    apps.into_values().collect()
}

pub fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data_home).join("applications"));
    }

    if let Some(home) = env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    if let Some(data_dirs) = env::var_os("XDG_DATA_DIRS") {
        for dir in env::split_paths(&data_dirs) {
            dirs.push(dir.join("applications"));
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }

    push_unique(&mut dirs, PathBuf::from("/usr/local/share/applications"));
    push_unique(&mut dirs, PathBuf::from("/usr/share/applications"));
    dirs
}

fn read_desktop_app(id: String, path: PathBuf) -> Result<Option<DesktopApp>> {
    let contents = std::fs::read_to_string(&path).map_err(|source| LauncherError::ReadFile {
        path: path.clone(),
        source,
    })?;

    parse_desktop_entry(id, path, &contents)
}

fn desktop_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            paths.extend(desktop_files(&path));
        } else if path
            .extension()
            .is_some_and(|extension| extension == "desktop")
        {
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

fn desktop_file_id(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .ok()
        .and_then(|relative| relative.to_str())
        .map(|relative| relative.replace('/', "-"))
        .or_else(|| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn push_unique(values: &mut Vec<PathBuf>, value: PathBuf) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_desktop_id_uses_dash_separator() {
        let id = desktop_file_id(
            Path::new("/usr/share/applications"),
            Path::new("/usr/share/applications/kde/org.kde.foo.desktop"),
        );

        assert_eq!(id, "kde-org.kde.foo.desktop");
    }
}

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("Could not read {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not write {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not parse TOML at {path}: {source}")]
    ParseToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Could not serialize TOML for {path}: {source}")]
    SerializeToml {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },

    #[error("Could not parse desktop entry: {path}: {reason}")]
    DesktopEntry { path: PathBuf, reason: String },

    #[error("Could not parse Exec line `{exec}`: {reason}")]
    ExecLine { exec: String, reason: String },

    #[error("Could not launch {app_name}: executable `{program}` was not found")]
    MissingExecutable { app_name: String, program: String },

    #[error("Could not launch {app_name}: {source}")]
    Launch {
        app_name: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not load theme `{theme}`: {reason}")]
    Theme { theme: String, reason: String },
}

pub type Result<T> = std::result::Result<T, LauncherError>;

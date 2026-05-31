use crate::config::Config;
use crate::desktop::DesktopApp;
use crate::error::{LauncherError, Result};
use crate::exec::CommandLine;
use std::process::{Command, Stdio};

pub fn command_for_app(app: &DesktopApp, config: &Config) -> CommandLine {
    if config.is_privileged(&app.id) {
        let mut args = Vec::with_capacity(app.command.args.len() + 1);
        args.push(app.command.program.clone());
        args.extend(app.command.args.clone());
        CommandLine {
            program: "pkexec".to_string(),
            args,
        }
    } else {
        app.command.clone()
    }
}

pub fn launch_app(app: &DesktopApp, config: &Config) -> Result<()> {
    let command_line = command_for_app(app, config);
    spawn_command(&app.name, &command_line)
}

pub fn spawn_command(app_name: &str, command_line: &CommandLine) -> Result<()> {
    let mut command = Command::new(&command_line.program);
    command
        .args(&command_line.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.spawn().map(|_| ()).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            LauncherError::MissingExecutable {
                app_name: app_name.to_string(),
                program: command_line.program.clone(),
            }
        } else {
            LauncherError::Launch {
                app_name: app_name.to_string(),
                source,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::DesktopApp;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn app() -> DesktopApp {
        DesktopApp {
            id: "admin.desktop".into(),
            path: PathBuf::from("admin.desktop"),
            name: "Admin".into(),
            generic_name: None,
            comment: None,
            exec: "admin --flag".into(),
            command: CommandLine {
                program: "admin".into(),
                args: vec!["--flag".into()],
            },
            icon: None,
            terminal: false,
            categories: Vec::new(),
            keywords: Vec::new(),
        }
    }

    #[test]
    fn wraps_privileged_apps_with_pkexec() {
        let mut config = Config::default();
        config
            .privileged_apps
            .extend(BTreeMap::from([("admin.desktop".to_string(), true)]));

        let command = command_for_app(&app(), &config);

        assert_eq!(command.as_vec(), ["pkexec", "admin", "--flag"]);
    }
}

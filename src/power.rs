use crate::config::PowerCommands;
use crate::error::{LauncherError, Result};
use crate::exec::CommandLine;
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerAction {
    Logout,
    Restart,
    Poweroff,
}

impl PowerAction {
    pub fn command_name(self) -> &'static str {
        match self {
            Self::Logout => "logout",
            Self::Restart => "restart",
            Self::Poweroff => "poweroff",
        }
    }

    pub fn needs_confirmation(self) -> bool {
        true
    }
}

pub fn command_for_action(action: PowerAction, commands: &PowerCommands) -> Option<CommandLine> {
    let values = match action {
        PowerAction::Logout => &commands.logout,
        PowerAction::Restart => &commands.restart,
        PowerAction::Poweroff => &commands.poweroff,
    };
    let (program, args) = values.split_first()?;

    Some(CommandLine {
        program: program.clone(),
        args: args.to_vec(),
    })
}

pub fn run_power_action(action: PowerAction, commands: &PowerCommands) -> Result<()> {
    let Some(command_line) = command_for_action(action, commands) else {
        return Err(LauncherError::ExecLine {
            exec: action.command_name().to_string(),
            reason: "power command is empty".to_string(),
        });
    };

    Command::new(&command_line.program)
        .args(&command_line.args)
        .spawn()
        .map(|_| ())
        .map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                LauncherError::MissingExecutable {
                    app_name: action.command_name().to_string(),
                    program: command_line.program,
                }
            } else {
                LauncherError::Launch {
                    app_name: action.command_name().to_string(),
                    source,
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_power_command() {
        let commands = PowerCommands::default();
        let restart = command_for_action(PowerAction::Restart, &commands).unwrap();

        assert_eq!(restart.as_vec(), ["systemctl", "reboot"]);
    }

    #[test]
    fn power_actions_need_confirmation() {
        assert!(PowerAction::Logout.needs_confirmation());
        assert!(PowerAction::Restart.needs_confirmation());
        assert!(PowerAction::Poweroff.needs_confirmation());
    }
}

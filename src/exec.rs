use crate::error::{LauncherError, Result};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecContext<'a> {
    pub name: &'a str,
    pub icon: Option<&'a str>,
    pub desktop_path: &'a Path,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandLine {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandLine {
    pub fn as_vec(&self) -> Vec<String> {
        let mut values = Vec::with_capacity(self.args.len() + 1);
        values.push(self.program.clone());
        values.extend(self.args.clone());
        values
    }
}

pub fn parse_exec(exec: &str, context: &ExecContext<'_>) -> Result<CommandLine> {
    let tokens = tokenize(exec)?;
    let mut expanded = Vec::new();

    for token in tokens {
        if let Some(values) = expand_token(&token, context)? {
            expanded.extend(values);
        }
    }

    let mut iter = expanded.into_iter().filter(|part| !part.is_empty());
    let Some(program) = iter.next() else {
        return Err(LauncherError::ExecLine {
            exec: exec.to_string(),
            reason: "Exec did not contain a runnable command".to_string(),
        });
    };

    Ok(CommandLine {
        program,
        args: iter.collect(),
    })
}

fn tokenize(exec: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = exec.chars().peekable();
    let mut single_quote = false;
    let mut double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !double_quote => single_quote = !single_quote,
            '"' if !single_quote => double_quote = !double_quote,
            '\\' if !single_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push('\\');
                }
            }
            ch if ch.is_whitespace() && !single_quote && !double_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if single_quote || double_quote {
        return Err(LauncherError::ExecLine {
            exec: exec.to_string(),
            reason: "unterminated quote".to_string(),
        });
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn expand_token(token: &str, context: &ExecContext<'_>) -> Result<Option<Vec<String>>> {
    if matches!(token, "%f" | "%F" | "%u" | "%U") {
        return Ok(None);
    }

    if token == "%i" {
        return Ok(context
            .icon
            .filter(|icon| !icon.is_empty())
            .map(|icon| vec!["--icon".to_string(), icon.to_string()]));
    }

    let mut output = String::new();
    let mut chars = token.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            output.push(ch);
            continue;
        }

        let Some(code) = chars.next() else {
            return Err(LauncherError::ExecLine {
                exec: token.to_string(),
                reason: "dangling `%` placeholder".to_string(),
            });
        };

        match code {
            '%' => output.push('%'),
            'f' | 'F' | 'u' | 'U' => {}
            'i' => {
                if let Some(icon) = context.icon.filter(|icon| !icon.is_empty()) {
                    output.push_str(icon);
                }
            }
            'c' => output.push_str(context.name),
            'k' => output.push_str(&context.desktop_path.to_string_lossy()),
            _ => {
                return Err(LauncherError::ExecLine {
                    exec: token.to_string(),
                    reason: format!("unsupported desktop Exec placeholder `%{code}`"),
                });
            }
        }
    }

    Ok(Some(vec![output]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn context<'a>() -> ExecContext<'a> {
        ExecContext {
            name: "Firefox",
            icon: Some("firefox"),
            desktop_path: Path::new("/usr/share/applications/firefox.desktop"),
        }
    }

    #[test]
    fn removes_file_and_url_placeholders() {
        let command = parse_exec("firefox %u --new-window %F", &context()).unwrap();
        assert_eq!(command.program, "firefox");
        assert_eq!(command.args, ["--new-window"]);
    }

    #[test]
    fn handles_quotes_and_percent_escape() {
        let command = parse_exec("app --name \"Hello World\" --ratio 100%%", &context()).unwrap();
        assert_eq!(
            command.as_vec(),
            ["app", "--name", "Hello World", "--ratio", "100%"]
        );
    }

    #[test]
    fn expands_name_icon_and_desktop_path() {
        let command = parse_exec("app %i --class %c --desktop %k", &context()).unwrap();
        assert_eq!(
            command.as_vec(),
            [
                "app",
                "--icon",
                "firefox",
                "--class",
                "Firefox",
                "--desktop",
                "/usr/share/applications/firefox.desktop"
            ]
        );
    }
}

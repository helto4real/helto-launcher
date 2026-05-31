use anyhow::Context;
use clap::Parser;
use helto_launcher::config::{config_path, Config};
use helto_launcher::discovery::discover_apps;
use helto_launcher::state::{state_path, LauncherState};
use helto_launcher::theme::{theme_dirs, Theme};
use helto_launcher::{ui, APP_ID};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(version, about = "A minimal GTK4 app launcher for Hyprland")]
struct Args {
    /// Read configuration from this path.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Read and write state at this path.
    #[arg(long)]
    state: Option<PathBuf>,

    /// Add a theme search directory. Later values are searched after XDG config themes.
    #[arg(long = "theme-dir")]
    theme_dirs: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(config_path);
    let state_path = args.state.unwrap_or_else(state_path);
    let config = Config::load(&config_path).context("loading configuration")?;
    let state = LauncherState::load(&state_path).context("loading launcher state")?;
    let apps = discover_apps();

    let mut theme_search_dirs = theme_dirs();
    theme_search_dirs.extend(args.theme_dirs);
    let theme = Theme::load(&config.theme, &theme_search_dirs).context("loading theme")?;

    ui::run(ui::UiInput {
        app_id: APP_ID,
        apps,
        config,
        state,
        state_path,
        theme,
    })
}

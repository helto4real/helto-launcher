# AGENTS.md

## Project Context

This repository contains Rust-based tools for Arch Linux desktop environments, with a primary focus on Wayland and Hyprland.

The tools in this project may interact with:

* Hyprland IPC
* Wayland-related user sessions
* desktop portals
* systemd user services
* configuration files under `~/.config`
* CLI utilities commonly available on Arch Linux
* status bars, launchers, notification daemons, and shell integrations

The goal is to build small, reliable, composable tools that feel native in a modern Hyprland setup.

Prefer simple, explicit Rust code over clever abstractions.

---

## Core Principles

When working in this repository:

1. Prioritize correctness, robustness, and maintainability.
2. Keep tools fast, predictable, and script-friendly.
3. Avoid unnecessary dependencies.
4. Respect the user’s system and configuration.
5. Never assume root access unless explicitly required.
6. Prefer user-level integration over system-wide changes.
7. Support Arch Linux conventions, but avoid hard-coding fragile paths where possible.
8. Treat Hyprland, Wayland, and desktop IPC as potentially unavailable or changing at runtime.
9. Fail clearly, with actionable error messages.
10. Keep CLI behavior stable and documented.

---

## Rust Guidelines

Use idiomatic stable Rust.

Prefer:

* `Result<T, E>` over panics
* `thiserror` for library/application errors
* `anyhow` for top-level CLI error handling when appropriate
* `clap` for CLI parsing
* `serde` / `serde_json` / `toml` for structured data
* `tracing` for logging
* `tokio` only when async is clearly useful
* `std::process::Command` only when a proper API or IPC interface is unavailable

Avoid:

* unnecessary macros
* global mutable state
* excessive trait abstraction
* over-engineered generic code
* shelling out when a stable API or socket protocol exists
* silently ignoring errors

Code should compile cleanly with:

```bash
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

Before finishing a change, run at least:

```bash
cargo fmt --all
cargo check
cargo test
```

If the repository has stricter commands in a Makefile, Justfile, CI file, or README, follow those instead.

---

## Project Structure

Prefer this structure unless the repository already uses another clear layout:

```text
src/
  main.rs
  cli.rs
  config.rs
  error.rs
  hyprland.rs
  wayland.rs
  systemd.rs
  command.rs
  output.rs
tests/
examples/
docs/
```

For larger projects, split reusable logic into a library:

```text
src/
  lib.rs
  main.rs
```

Keep `main.rs` thin. It should mostly parse arguments, initialize logging, call application logic, and handle top-level errors.

---

## CLI Design

CLI tools should be pleasant to use from terminals, scripts, Waybar modules, launchers, and keybindings.

Use `clap` derive unless there is a strong reason not to.

All commands should support:

```bash
--help
--version
```

When useful, support machine-readable output:

```bash
--json
```

Prefer stable, parseable output for scripts.

Do not print noisy logs to stdout. Use:

* stdout for actual command output
* stderr for diagnostics, warnings, and logs

Exit codes should be meaningful:

* `0` success
* `1` general failure
* `2` invalid usage or configuration
* higher codes only when clearly documented

---

## Hyprland Integration

When interacting with Hyprland, prefer Hyprland IPC over shell commands where possible.

Common IPC locations may involve environment variables such as:

```bash
HYPRLAND_INSTANCE_SIGNATURE
XDG_RUNTIME_DIR
```

Do not assume these are always set.

When Hyprland is unavailable, return a clear error such as:

```text
Hyprland IPC is not available. Are you running inside a Hyprland session?
```

Avoid hard dependencies on a running compositor in unit tests.

Design Hyprland-related code so it can be mocked or tested with fixture data.

Do not break if:

* Hyprland is not running
* the IPC socket is missing
* the compositor restarts
* monitors are hot-plugged
* workspaces change while the tool is running
* window metadata is incomplete
* JSON output changes slightly between Hyprland versions

When parsing Hyprland output, use tolerant deserialization where appropriate.

---

## Wayland and Desktop Environment Assumptions

This project targets Wayland-first workflows.

Do not assume X11 unless explicitly required.

Avoid relying on X11-only tools such as:

```bash
xdotool
xprop
wmctrl
```

Prefer Wayland-compatible approaches.

For clipboard integration, prefer tools commonly used on Wayland, such as:

```bash
wl-copy
wl-paste
```

For notifications, prefer standards-compatible tools or libraries.

For portals, consider desktop portal behavior and environment-specific quirks.

Always document external runtime dependencies.

---

## Arch Linux Conventions

This project is optimized for Arch Linux and Arch-based systems.

Assume the user may install tools via:

```bash
pacman
paru
yay
cargo install
```

But do not require an AUR helper.

When adding documentation, prefer examples using `pacman` for official packages and clearly mark AUR packages separately.

Do not modify files under `/etc`, `/usr`, or `/var` without explicit user intent.

Prefer user-level paths:

```text
~/.config
~/.local/bin
~/.local/share
~/.cache
```

Use XDG base directory conventions when possible.

Respect:

```bash
XDG_CONFIG_HOME
XDG_DATA_HOME
XDG_CACHE_HOME
XDG_STATE_HOME
XDG_RUNTIME_DIR
```

Do not assume the user’s shell is Bash. Commands should work in POSIX shell where possible, unless specifically documented as Bash, Zsh, or Fish.

---

## Configuration

Configuration should be explicit, documented, and stable.

Prefer TOML for user-editable configuration files.

Example path preference:

```text
$XDG_CONFIG_HOME/<tool-name>/config.toml
```

Fallback:

```text
~/.config/<tool-name>/config.toml
```

Do not overwrite existing configuration without confirmation unless the command explicitly says it will.

If generating config files, preserve comments where possible.

Configuration errors should include:

* file path
* offending key if known
* expected value
* suggested fix

---

## systemd User Services

If the tool runs as a background service, prefer systemd user services over system-wide services.

Use:

```bash
systemctl --user
```

Do not use `sudo systemctl` unless explicitly required.

User service files should be suitable for:

```text
~/.config/systemd/user/
```

After creating or changing a user service, document the expected commands:

```bash
systemctl --user daemon-reload
systemctl --user enable --now <service-name>.service
systemctl --user status <service-name>.service
journalctl --user -u <service-name>.service -f
```

If environment variables from the Wayland session are required, document them clearly.

---

## Logging and Diagnostics

Use `tracing` for logging.

Default output should be quiet.

Support increased verbosity where useful:

```bash
-v
-vv
--verbose
--quiet
```

Do not log secrets, tokens, full environment dumps, or private user data.

Diagnostic errors should be useful, not cryptic.

Bad:

```text
No such file or directory
```

Better:

```text
Could not connect to Hyprland IPC socket at /run/user/1000/hypr/...: file does not exist
```

---

## Error Handling

Never use `unwrap()` or `expect()` in production code unless the invariant is obvious and impossible to violate.

Acceptable cases:

```rust
let regex = Regex::new("...").expect("hard-coded regex must be valid");
```

Unacceptable cases:

```rust
let path = std::env::var("XDG_RUNTIME_DIR").unwrap();
```

Errors should usually be represented with a project-specific error enum for library code.

Top-level CLI errors may use `anyhow`.

---

## Testing

Write tests for:

* config parsing
* CLI argument behavior
* Hyprland IPC parsing
* JSON parsing
* error handling
* path resolution
* output formatting

Avoid tests that require a live Hyprland session.

Use fixture files for sample Hyprland responses.

Suggested fixture layout:

```text
tests/fixtures/
  hyprctl_clients.json
  hyprctl_monitors.json
  hyprctl_workspaces.json
```

For command-line integration tests, prefer `assert_cmd` and `predicates`.

For snapshot-style output tests, prefer `insta` if the project already uses it. Do not introduce snapshot testing casually for tiny projects.

---

## Dependency Policy

Before adding a dependency, consider whether the standard library is enough.

Acceptable common dependencies:

```toml
anyhow
thiserror
clap
serde
serde_json
toml
tracing
tracing-subscriber
tokio
dirs
directories
xdg
```

Avoid dependencies that:

* are unmaintained
* pull in large trees unnecessarily
* require system libraries without documentation
* are nightly-only
* make packaging harder on Arch Linux

When adding dependencies, explain why they are needed.

---

## Formatting and Style

Use standard Rust formatting:

```bash
cargo fmt --all
```

Prefer readable names.

Avoid abbreviations unless they are common in the domain:

Good:

```rust
workspace_id
monitor_name
socket_path
```

Avoid:

```rust
wsid
mon
sockp
```

Keep functions small when practical.

A function should usually do one thing:

* parse input
* query IPC
* transform data
* render output
* perform command action

Avoid mixing all of these in one function.

---

## Security and Safety

This project may run commands, read config files, interact with IPC sockets, and integrate with desktop automation.

Be conservative.

Never execute user-provided shell strings through a shell.

Avoid:

```rust
Command::new("sh").arg("-c").arg(user_input)
```

Prefer:

```rust
Command::new("hyprctl")
    .arg("dispatch")
    .arg("workspace")
    .arg(workspace.to_string())
```

Validate paths before writing.

Do not follow symlinks for sensitive file writes unless intentional and documented.

Do not change permissions broadly.

Do not recursively delete files unless the command is explicitly designed for cleanup and has safeguards.

---

## Performance

Most tools should start quickly and exit quickly.

Avoid unnecessary async runtimes for simple one-shot commands.

Avoid polling when event-based integration is possible.

For long-running tools:

* handle signals gracefully
* reconnect after Hyprland restarts if practical
* avoid busy loops
* use backoff on repeated failures
* keep memory usage modest

---

## Packaging

When adding packaging support, prefer simple Arch-friendly packaging.

Useful files may include:

```text
PKGBUILD
.install
LICENSE
README.md
CHANGELOG.md
```

PKGBUILD should avoid unnecessary post-install behavior.

Do not install user configuration files directly into a user’s home directory from a package.

For cargo-based installation, document:

```bash
cargo install --path .
```

For local development, document:

```bash
cargo run -- <args>
```

---

## Documentation

Every tool should have a README or section documenting:

* what the tool does
* requirements
* installation
* basic usage
* configuration
* examples
* known limitations
* troubleshooting

Examples should be realistic for Hyprland users.

Useful example categories:

```text
Waybar integration
Hyprland keybind
systemd user service
launcher integration
JSON output for scripts
```

Example Hyprland bind format:

```text
bind = SUPER, X, exec, <tool-name> <args>
```

Example Waybar module format:

```json
"custom/example": {
  "exec": "<tool-name> --json",
  "return-type": "json",
  "interval": 5
}
```

---

## Agent Workflow

When making changes as an AI coding agent:

1. Inspect the existing repository structure first.
2. Read README, Cargo.toml, Justfile, Makefile, CI files, and existing tests.
3. Follow the project’s existing patterns before introducing new ones.
4. Make the smallest coherent change that solves the task.
5. Update tests when behavior changes.
6. Update documentation when CLI behavior, config, or dependencies change.
7. Run formatting and relevant checks.
8. Summarize what changed and what was tested.

Do not rewrite unrelated code.

Do not introduce a new architecture unless the current one blocks the requested change.

Do not rename public commands, config keys, or output fields without explicit instruction.

---

## Agent Coding Rules

When implementing a feature:

* Start with the public behavior.
* Add or update tests for that behavior.
* Implement the smallest internal change needed.
* Keep errors user-friendly.
* Keep output stable.
* Avoid breaking existing scripts.

When fixing a bug:

* Add a regression test if practical.
* Explain the root cause.
* Avoid broad rewrites.
* Verify the fix with relevant commands.

When refactoring:

* Preserve behavior.
* Keep commits logically small.
* Avoid mixing refactors with feature changes unless necessary.

---

## Hyprland Tooling Examples

Useful tool ideas this repository may contain:

* workspace switchers
* monitor layout helpers
* window rule helpers
* scratchpad utilities
* Waybar modules
* active window inspectors
* idle/session helpers
* keybinding generators
* config validators
* notification helpers
* theme switchers
* portal diagnostics
* screenshot workflow helpers
* launcher integrations

Any such tool should work well in a composable Linux desktop workflow.

---

## Preferred External Commands

If shelling out is necessary, prefer common Wayland/Hyprland-friendly tools:

```text
hyprctl
systemctl --user
journalctl --user
notify-send
wl-copy
wl-paste
grim
slurp
swappy
jq
```

Do not assume all of them are installed.

If a command is missing, return a clear error with an installation hint.

Example:

```text
Required command `grim` was not found. Install it with: pacman -S grim
```

---

## Compatibility

Target stable Rust.

Avoid nightly features unless explicitly approved.

Support current Arch Linux packages.

Do not overfit to one exact Hyprland version unless the task requires it.

When Hyprland behavior differs by version, document the difference and handle it defensively where practical.

---

## Final Response Expectations for Agents

When completing a task, respond with:

* summary of changes
* files changed
* tests/checks run
* any known limitations
* any follow-up suggestions, if relevant

Do not claim tests passed unless they were actually run.

If checks could not be run, say why.

For this launcher repository, always run a release build after each code change:

```bash
cargo build --release
```

The user tests the live launcher from:

```text
./target/release/helto-launcher
```

Do this even when debug checks pass, so the release binary is ready for immediate live testing.

Example:

```text
Summary:
- Added Hyprland IPC client for workspace queries.
- Added JSON output mode for Waybar integration.
- Added config parsing tests.

Checks:
- cargo fmt --all
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings

Notes:
- Live Hyprland integration was not tested because no compositor session was available.
```

---

## Non-Goals

Do not turn this project into a full desktop environment framework.

Do not build a general-purpose plugin system unless explicitly requested.

Do not require root privileges for normal desktop tooling.

Do not make assumptions about the user’s personal Hyprland configuration.

Do not silently modify Hyprland config files.

---

## Tone and Maintainability

Code should feel boring in the best possible way.

Readable beats clever.

Small tools should remain small.

Every dependency, background process, config file, and IPC interaction should have a clear reason to exist.

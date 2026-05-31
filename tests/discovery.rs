use helto_launcher::discovery::discover_apps_from_dirs;
use std::path::PathBuf;

#[test]
fn discovers_valid_desktop_entries_from_fixture_dir() {
    let apps = discover_apps_from_dirs(&[PathBuf::from("tests/fixtures/desktop_entries")]);
    let ids: Vec<_> = apps.iter().map(|app| app.id.as_str()).collect();

    assert!(ids.contains(&"firefox.desktop"));
    assert!(ids.contains(&"terminal.desktop"));
    assert!(!ids.contains(&"hidden.desktop"));
    assert!(!ids.contains(&"nodisplay.desktop"));
    assert!(!ids.contains(&"invalid.desktop"));
}

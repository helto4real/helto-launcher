use crate::config::Config;
use crate::desktop::DesktopApp;
use crate::favorites::MAX_FAVORITES;
use crate::launch::launch_app;
use crate::power::{run_power_action, PowerAction};
use crate::search::filter_and_rank;
use crate::state::LauncherState;
use crate::theme::Theme;
use anyhow::Context;
use gtk::gdk;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{error, warn};

pub struct UiInput {
    pub app_id: &'static str,
    pub apps: Vec<DesktopApp>,
    pub config: Config,
    pub state: LauncherState,
    pub state_path: PathBuf,
    pub theme: Theme,
}

struct UiModel {
    apps: Vec<DesktopApp>,
    apps_by_id: HashMap<String, DesktopApp>,
    visible_ids: Vec<String>,
    config: Config,
    state: LauncherState,
    state_path: PathBuf,
    pending_power: Option<PowerAction>,
}

#[derive(Clone)]
struct UiWidgets {
    window: gtk::ApplicationWindow,
    search: gtk::Entry,
    results: gtk::ListBox,
    favorites: gtk::Box,
    remove_target: gtk::Label,
    status: gtk::Label,
}

pub fn run(input: UiInput) -> anyhow::Result<()> {
    let app = gtk::Application::builder()
        .application_id(input.app_id)
        .build();
    let state_path = input.state_path.clone();
    let theme = input.theme.clone();
    let app_data = Rc::new(RefCell::new(Some(input)));

    app.connect_activate(move |application| {
        let Some(input) = app_data.borrow_mut().take() else {
            return;
        };

        if let Err(err) = build_ui(application, input, theme.clone(), state_path.clone()) {
            error!("{err:?}");
        }
    });

    app.run();
    Ok(())
}

fn build_ui(
    application: &gtk::Application,
    input: UiInput,
    theme: Theme,
    state_path: PathBuf,
) -> anyhow::Result<()> {
    apply_theme(&theme)?;

    let apps_by_id = input
        .apps
        .iter()
        .cloned()
        .map(|app| (app.id.clone(), app))
        .collect();

    let model = Rc::new(RefCell::new(UiModel {
        apps: input.apps,
        apps_by_id,
        visible_ids: Vec::new(),
        config: input.config,
        state: input.state,
        state_path,
        pending_power: None,
    }));

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Helto Launcher")
        .default_width(760)
        .default_height(500)
        .resizable(false)
        .build();
    window.add_css_class("launcher-window");
    window.set_decorated(false);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("launcher-root");
    set_margin_all(&root, 10);
    window.set_child(Some(&root));

    let search = gtk::Entry::new();
    search.add_css_class("launcher-search");
    search.set_placeholder_text(Some("Search applications"));
    search.set_margin_bottom(4);
    root.append(&search);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.add_css_class("launcher-content");
    content.set_margin_top(2);
    content.set_margin_bottom(2);
    content.set_margin_start(4);
    content.set_margin_end(4);
    root.append(&content);

    let favorites = gtk::Box::new(gtk::Orientation::Vertical, 8);
    favorites.add_css_class("launcher-panel");
    favorites.set_width_request(64);
    content.append(&favorites);

    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .min_content_height(340)
        .build();
    scrolled.add_css_class("launcher-results-frame");
    let results = gtk::ListBox::new();
    results.add_css_class("launcher-results");
    results.set_selection_mode(gtk::SelectionMode::Single);
    scrolled.set_child(Some(&results));
    content.append(&scrolled);

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    bottom.add_css_class("launcher-bottom");
    bottom.set_margin_top(0);
    bottom.set_margin_start(4);
    bottom.set_margin_end(4);
    root.append(&bottom);

    let actions_panel = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions_panel.add_css_class("launcher-actions-panel");
    actions_panel.set_hexpand(true);
    bottom.append(&actions_panel);

    let remove_target = gtk::Label::new(Some("Drop favorite here to remove"));
    remove_target.add_css_class("launcher-muted");
    remove_target.set_valign(gtk::Align::Center);
    actions_panel.append(&remove_target);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    actions_panel.append(&spacer);

    let status = gtk::Label::new(None);
    status.set_xalign(0.0);
    status.add_css_class("launcher-muted");
    status.set_visible(false);

    for (label, action) in [
        ("Logout", PowerAction::Logout),
        ("Restart", PowerAction::Restart),
        ("Power", PowerAction::Poweroff),
    ] {
        let button = gtk::Button::with_label(label);
        button.add_css_class("launcher-button");
        button.add_css_class("launcher-power-button");
        button.add_css_class(match action {
            PowerAction::Logout => "launcher-logout-button",
            PowerAction::Restart => "launcher-restart-button",
            PowerAction::Poweroff => "launcher-poweroff-button",
        });
        button.set_valign(gtk::Align::Center);
        let model = model.clone();
        let widgets = UiWidgets {
            window: window.clone(),
            search: search.clone(),
            results: results.clone(),
            favorites: favorites.clone(),
            remove_target: remove_target.clone(),
            status: status.clone(),
        };
        button.connect_clicked(move |_| trigger_power(&model, &widgets, action));
        actions_panel.append(&button);
    }

    root.append(&status);

    let widgets = UiWidgets {
        window: window.clone(),
        search: search.clone(),
        results: results.clone(),
        favorites: favorites.clone(),
        remove_target: remove_target.clone(),
        status: status.clone(),
    };

    {
        let model = model.clone();
        let widgets = widgets.clone();
        search.connect_changed(move |entry| {
            model.borrow_mut().pending_power = None;
            clear_status(&widgets);
            refresh_results(&model, &widgets, entry.text().as_str());
        });
    }

    {
        let model = model.clone();
        let widgets = widgets.clone();
        results.connect_row_activated(move |_, row| {
            launch_visible_index(&model, &widgets, row.index() as usize);
        });
    }

    let key_controller = gtk::EventControllerKey::new();
    {
        let model = model.clone();
        let widgets = widgets.clone();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            handle_key(&model, &widgets, key, modifiers)
        });
    }
    search.add_controller(key_controller);

    install_remove_drop_target(&model, &widgets);
    refresh_favorites(&model, &widgets);
    refresh_results(&model, &widgets, "");

    window.present();
    search.grab_focus();

    Ok(())
}

fn apply_theme(theme: &Theme) -> anyhow::Result<()> {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(&theme.css());
    let display = gdk::Display::default().context("GTK display was not available")?;
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    Ok(())
}

fn refresh_results(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets, query: &str) {
    clear_list_box(&widgets.results);

    let results = {
        let model = model.borrow();
        filter_and_rank(&model.apps, query, &model.state.launch_counts)
            .into_iter()
            .map(|result| result.app.id.clone())
            .collect::<Vec<_>>()
    };

    model.borrow_mut().visible_ids = results.clone();

    for (index, app_id) in results.iter().enumerate() {
        let Some(app) = model.borrow().apps_by_id.get(app_id).cloned() else {
            continue;
        };

        let row = gtk::ListBoxRow::new();
        row.add_css_class("launcher-result-row");
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        row_box.add_css_class("launcher-result-surface");
        row_box.set_hexpand(true);
        set_margin_all(&row_box, 4);

        let icon = app_icon(&app);
        icon.set_pixel_size(28);
        icon.set_valign(gtk::Align::Center);
        row_box.append(&icon);

        let labels = gtk::Box::new(gtk::Orientation::Vertical, 2);
        labels.set_hexpand(true);
        labels.set_valign(gtk::Align::Center);
        let name = gtk::Label::new(Some(&format!("Alt+{}  {}", (index % 9) + 1, app.name)));
        name.set_xalign(0.0);
        name.set_hexpand(true);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        labels.append(&name);
        if let Some(generic_name) = app.generic_name.as_deref().or(app.comment.as_deref()) {
            let subtitle = gtk::Label::new(Some(generic_name));
            subtitle.set_xalign(0.0);
            subtitle.set_hexpand(true);
            subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
            subtitle.add_css_class("launcher-muted");
            labels.append(&subtitle);
        }
        row_box.append(&labels);

        let action_area = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        action_area.add_css_class("launcher-row-actions");
        action_area.set_valign(gtk::Align::Center);
        let favorite = favorite_button(model, widgets, &app);
        action_area.append(&favorite);
        row_box.append(&action_area);

        row.set_child(Some(&row_box));
        widgets.results.append(&row);
    }

    if let Some(row) = widgets.results.row_at_index(0) {
        widgets.results.select_row(Some(&row));
    }
}

fn refresh_favorites(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets) {
    clear_box(&widgets.favorites);

    let available: HashSet<String> = model.borrow().apps_by_id.keys().cloned().collect();
    let favorite_ids = model.borrow().state.favorites.items.clone();

    for (index, app_id) in favorite_ids.iter().enumerate() {
        let Some(app) = model.borrow().apps_by_id.get(app_id).cloned() else {
            continue;
        };
        if !available.contains(app_id) {
            continue;
        }

        let slot = gtk::Button::new();
        slot.add_css_class("launcher-favorite-slot");
        slot.set_tooltip_text(Some(&app.name));

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&favorite_icon_tile(&app)));
        overlay.add_overlay(&favorite_badge(index + 1));
        slot.set_child(Some(&overlay));

        {
            let model = model.clone();
            let widgets = widgets.clone();
            let app_id = app_id.clone();
            slot.connect_clicked(move |_| launch_by_id(&model, &widgets, &app_id));
        }

        install_favorite_drag_and_drop(&slot, model, widgets, index);
        widgets.favorites.append(&slot);
    }

    for index in favorite_ids.len()..MAX_FAVORITES {
        let empty = gtk::Overlay::new();
        empty.add_css_class("launcher-favorite-empty");
        empty.set_tooltip_text(Some("Empty favorite slot"));
        let placeholder = gtk::Box::new(gtk::Orientation::Vertical, 0);
        placeholder.add_css_class("launcher-favorite-placeholder");
        empty.set_child(Some(&placeholder));
        empty.add_overlay(&favorite_badge(index + 1));
        widgets.favorites.append(&empty);
    }
}

fn favorite_button(
    model: &Rc<RefCell<UiModel>>,
    widgets: &UiWidgets,
    app: &DesktopApp,
) -> gtk::Button {
    let is_favorite = model.borrow().state.favorites.contains(&app.id);
    let can_add = model.borrow().state.favorites.can_add(&app.id);
    let button = gtk::Button::with_label(if is_favorite { "*" } else { "+" });
    button.add_css_class("launcher-button");
    button.add_css_class("launcher-icon-button");
    button.add_css_class("launcher-favorite-button");
    button.set_tooltip_text(Some(if is_favorite {
        "Remove favorite"
    } else {
        "Add favorite"
    }));
    button.set_sensitive(can_add);
    button.add_css_class(if is_favorite {
        "launcher-favorite-active"
    } else {
        "launcher-favorite-inactive"
    });

    let model = model.clone();
    let widgets = widgets.clone();
    let app_id = app.id.clone();
    button.connect_clicked(move |_| {
        model.borrow_mut().state.favorites.toggle(&app_id);
        save_state(&model);
        refresh_favorites(&model, &widgets);
        refresh_results(&model, &widgets, widgets.search.text().as_str());
    });

    button
}

fn install_favorite_drag_and_drop<W>(
    row: &W,
    model: &Rc<RefCell<UiModel>>,
    widgets: &UiWidgets,
    index: usize,
) where
    W: IsA<gtk::Widget>,
{
    let drag = gtk::DragSource::builder()
        .actions(gdk::DragAction::MOVE)
        .build();
    drag.connect_prepare(move |_, _, _| {
        Some(gdk::ContentProvider::for_value(&(index as u32).to_value()))
    });
    row.add_controller(drag);

    let drop = gtk::DropTarget::new(u32::static_type(), gdk::DragAction::MOVE);
    {
        let model = model.clone();
        let widgets = widgets.clone();
        drop.connect_drop(move |_, value, _, _| {
            let Ok(from) = value.get::<u32>() else {
                return false;
            };

            if model
                .borrow_mut()
                .state
                .favorites
                .reorder(from as usize, index)
            {
                save_state(&model);
                refresh_favorites(&model, &widgets);
            }
            true
        });
    }
    row.add_controller(drop);
}

fn install_remove_drop_target(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets) {
    let drop = gtk::DropTarget::new(u32::static_type(), gdk::DragAction::MOVE);
    {
        let model = model.clone();
        let widgets = widgets.clone();
        drop.connect_drop(move |_, value, _, _| {
            let Ok(from) = value.get::<u32>() else {
                return false;
            };

            let app_id = model
                .borrow()
                .state
                .favorites
                .items
                .get(from as usize)
                .cloned();
            if let Some(app_id) = app_id {
                model.borrow_mut().state.favorites.remove(&app_id);
                save_state(&model);
                refresh_favorites(&model, &widgets);
                refresh_results(&model, &widgets, widgets.search.text().as_str());
            }
            true
        });
    }
    widgets.remove_target.add_controller(drop);
}

fn handle_key(
    model: &Rc<RefCell<UiModel>>,
    widgets: &UiWidgets,
    key: gdk::Key,
    modifiers: gdk::ModifierType,
) -> Propagation {
    if key == gdk::Key::Escape {
        widgets.window.close();
        return Propagation::Stop;
    }

    if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
        let index = widgets
            .results
            .selected_row()
            .map(|row| row.index() as usize)
            .unwrap_or(0);
        launch_visible_index(model, widgets, index);
        return Propagation::Stop;
    }

    let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
    let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

    if ctrl {
        if let Some(number) = key.to_unicode().and_then(number_key) {
            if (1..=5).contains(&number) {
                launch_favorite_slot(model, widgets, number - 1);
                return Propagation::Stop;
            }
        }
    }

    if key == gdk::Key::Down || (ctrl && key.to_unicode() == Some('n')) {
        move_selection(&widgets.results, 1);
        return Propagation::Stop;
    }

    if key == gdk::Key::Up || (ctrl && key.to_unicode() == Some('p')) {
        move_selection(&widgets.results, -1);
        return Propagation::Stop;
    }

    if alt {
        if let Some(number) = key.to_unicode().and_then(number_key) {
            if (1..=9).contains(&number) {
                launch_visible_index(model, widgets, number - 1);
                return Propagation::Stop;
            }
        }
    }

    let search_empty = widgets.search.text().is_empty();
    if search_empty {
        match key.to_unicode() {
            Some('q') => {
                trigger_power(model, widgets, PowerAction::Logout);
                return Propagation::Stop;
            }
            Some('r') => {
                trigger_power(model, widgets, PowerAction::Restart);
                return Propagation::Stop;
            }
            Some('Q') => {
                trigger_power(model, widgets, PowerAction::Poweroff);
                return Propagation::Stop;
            }
            Some(_) => {}
            None => {}
        }
    }

    Propagation::Proceed
}

fn move_selection(results: &gtk::ListBox, offset: i32) {
    let current = results.selected_row().map(|row| row.index()).unwrap_or(0);
    let next = (current + offset).max(0);
    if let Some(row) = results.row_at_index(next) {
        results.select_row(Some(&row));
    }
}

fn launch_visible_index(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets, index: usize) {
    let app_id = model.borrow().visible_ids.get(index).cloned();
    let Some(app_id) = app_id else {
        return;
    };
    launch_by_id(model, widgets, &app_id);
}

fn launch_favorite_slot(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets, index: usize) {
    let available: HashSet<String> = model.borrow().apps_by_id.keys().cloned().collect();
    let visible_favorites: Vec<_> = model
        .borrow()
        .state
        .favorites
        .items
        .iter()
        .filter(|app_id| available.contains(*app_id))
        .cloned()
        .collect();
    let Some(app_id) = visible_favorites.get(index) else {
        return;
    };
    launch_by_id(model, widgets, app_id);
}

fn launch_by_id(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets, app_id: &str) {
    let (app, config, state_path) = {
        let model = model.borrow();
        let Some(app) = model.apps_by_id.get(app_id).cloned() else {
            return;
        };
        (app, model.config.clone(), model.state_path.clone())
    };

    if config.is_privileged(&app.id) {
        set_status(
            widgets,
            "Privilege elevation requested through pkexec/polkit.",
        );
    }

    match launch_app(&app, &config) {
        Ok(()) => {
            {
                let mut model = model.borrow_mut();
                model.state.record_launch(&app.id);
                if let Err(err) = model.state.save(&state_path) {
                    warn!("{err}");
                }
            }
            widgets.window.close();
        }
        Err(err) => {
            set_status(widgets, &err.to_string());
            error!("{err}");
        }
    }
}

fn trigger_power(model: &Rc<RefCell<UiModel>>, widgets: &UiWidgets, action: PowerAction) {
    if action.needs_confirmation() && model.borrow().pending_power != Some(action) {
        model.borrow_mut().pending_power = Some(action);
        set_status(
            widgets,
            &format!("Press {} again to confirm.", action.command_name()),
        );
        return;
    }

    let commands = model.borrow().config.commands.clone();
    match run_power_action(action, &commands) {
        Ok(()) => widgets.window.close(),
        Err(err) => {
            set_status(widgets, &err.to_string());
            error!("{err}");
        }
    }
}

fn set_status(widgets: &UiWidgets, message: &str) {
    widgets.status.set_text(message);
    widgets.status.set_visible(true);
}

fn clear_status(widgets: &UiWidgets) {
    widgets.status.set_text("");
    widgets.status.set_visible(false);
}

fn app_icon(app: &DesktopApp) -> gtk::Image {
    if let Some(icon) = app.icon.as_deref() {
        if icon.starts_with('/') {
            return gtk::Image::from_file(icon);
        }
        return gtk::Image::from_icon_name(icon);
    }

    gtk::Image::from_icon_name("application-x-executable")
}

fn favorite_icon_tile(app: &DesktopApp) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Vertical, 0);
    tile.add_css_class("launcher-favorite-tile");
    let icon = app_icon(app);
    icon.set_pixel_size(34);
    icon.set_halign(gtk::Align::Center);
    icon.set_valign(gtk::Align::Center);
    tile.append(&icon);
    tile
}

fn favorite_badge(slot: usize) -> gtk::Label {
    let badge = gtk::Label::new(Some(&slot.to_string()));
    badge.add_css_class("launcher-favorite-badge");
    badge.set_halign(gtk::Align::End);
    badge.set_valign(gtk::Align::End);
    badge
}

fn number_key(ch: char) -> Option<usize> {
    ch.to_digit(10).map(|value| value as usize)
}

fn save_state(model: &Rc<RefCell<UiModel>>) {
    let model = model.borrow();
    if let Err(err) = model.state.save(&model.state_path) {
        warn!("{err}");
    }
}

fn clear_list_box(list: &gtk::ListBox) {
    while let Some(row) = list.row_at_index(0) {
        list.remove(&row);
    }
}

fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn set_margin_all<W>(widget: &W, margin: i32)
where
    W: IsA<gtk::Widget>,
{
    widget.set_margin_top(margin);
    widget.set_margin_bottom(margin);
    widget.set_margin_start(margin);
    widget.set_margin_end(margin);
}

//! Preferences: appearance and content on the General page; custom
//! sites and hidden boards on the Sites page.

use adw::prelude::*;
use channex::data::store::{CatalogViewMode, ReplyStyle};
use gtk::glib;

use super::{accent_picker, keep_synced, setting_switch, style_toggle};
use crate::state::state;
use crate::theme::{self, SKINS};
use crate::window::ChannexWindow;

/// Opens Preferences, optionally on a page (`"general"` or `"sites"`).
pub fn show_preferences(window: &ChannexWindow, page: Option<&str>) -> adw::PreferencesDialog {
    let dialog = adw::PreferencesDialog::builder()
        .search_enabled(true)
        .build();
    dialog.add(&general_page(window, &dialog));
    dialog.add(&sites_page(window));
    if let Some(page) = page {
        dialog.set_visible_page_name(page);
    }
    dialog.present(Some(window));
    dialog
}

fn general_page(window: &ChannexWindow, dialog: &adw::PreferencesDialog) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .name("general")
        .title("General")
        .icon_name("emblem-system-symbolic")
        .build();

    // ---- Appearance ----
    let appearance = adw::PreferencesGroup::builder().title("Appearance").build();

    let style = adw::ActionRow::builder().title("Style").build();
    style.add_suffix(&style_toggle());
    appearance.add(&style);

    let accent_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let accent_title = gtk::Label::builder()
        .label("Accent Color")
        .xalign(0.0)
        .build();
    let accent_hint = gtk::Label::builder()
        .xalign(0.0)
        .wrap(true)
        .css_classes(["dim-label", "caption"])
        .build();
    keep_synced(&accent_hint, |hint| {
        let skin = theme::skin(&state().data().settings.theme).is_some();
        hint.set_label(if skin {
            "Classic skins use their own traditional colors."
        } else {
            "Used for buttons, switches, selection, and links."
        });
    });
    accent_box.append(&accent_title);
    accent_box.append(&accent_hint);
    accent_box.append(&accent_picker());
    // A plain PreferencesRow holding a custom layout.
    let accent = adw::PreferencesRow::builder()
        .title("Accent Color")
        .activatable(false)
        .child(&accent_box)
        .build();
    appearance.add(&accent);

    let skin_names: Vec<&str> = std::iter::once("None")
        .chain(SKINS.iter().map(|s| s.name))
        .collect();
    let skin_row = adw::ComboRow::builder()
        .title("Imageboard Skin")
        .subtitle("Classic 4chan-style colors")
        .model(&gtk::StringList::new(&skin_names))
        .build();
    keep_synced(&skin_row, |row| {
        let theme = state().data().settings.theme.clone();
        let index = SKINS
            .iter()
            .position(|s| s.id == theme)
            .map_or(0, |i| i + 1) as u32;
        if row.selected() != index {
            row.set_selected(index);
        }
    });
    skin_row.connect_selected_notify(|row| {
        let theme = match row.selected() {
            0 => None,
            i => SKINS.get(i as usize - 1).map(|s| s.id.to_owned()),
        };
        let current = state().data().settings.theme.clone();
        let current_is_skin = theme::skin(&current).is_some();
        match theme {
            Some(id) if id != current => state().update_settings(|d| d.settings.theme = id),
            None if current_is_skin => {
                state().update_settings(|d| d.settings.theme = "system".into())
            }
            _ => {}
        }
    });
    appearance.add(&skin_row);

    let views = [
        ("Grid", CatalogViewMode::Grid),
        ("Compact Grid", CatalogViewMode::Compact),
        ("List", CatalogViewMode::List),
    ];
    let view_row = adw::ComboRow::builder()
        .title("Catalog View")
        .model(&gtk::StringList::new(&views.map(|(label, _)| label)))
        .build();
    keep_synced(&view_row, move |row| {
        let mode = state().data().settings.catalog_view_mode;
        let index = views.iter().position(|(_, m)| *m == mode).unwrap_or(0) as u32;
        if row.selected() != index {
            row.set_selected(index);
        }
    });
    view_row.connect_selected_notify(move |row| {
        let mode = views[row.selected() as usize].1;
        if state().data().settings.catalog_view_mode != mode {
            state().update_settings(|d| d.settings.catalog_view_mode = mode);
        }
    });
    appearance.add(&view_row);

    let styles = [
        ("Compact", ReplyStyle::Compact),
        ("Standard", ReplyStyle::Standard),
        ("Modern", ReplyStyle::Modern),
    ];
    let reply_row = adw::ComboRow::builder()
        .title("Post Style")
        .subtitle("How posts look in threads")
        .model(&gtk::StringList::new(&styles.map(|(label, _)| label)))
        .build();
    keep_synced(&reply_row, move |row| {
        let style = state().data().settings.reply_display_style;
        let index = styles.iter().position(|(_, s)| *s == style).unwrap_or(1) as u32;
        if row.selected() != index {
            row.set_selected(index);
        }
    });
    reply_row.connect_selected_notify(move |row| {
        let style = styles[row.selected() as usize].1;
        if state().data().settings.reply_display_style != style {
            state().update_settings(|d| d.settings.reply_display_style = style);
        }
    });
    appearance.add(&reply_row);
    page.add(&appearance);

    // ---- Content ----
    let content = adw::PreferencesGroup::builder().title("Content").build();
    content.add(&setting_switch(
        "Blur NSFW Images and Spoilers",
        "Click a thumbnail to reveal it",
        |d| d.settings.blur_nsfw,
        |d, v| d.settings.blur_nsfw = v,
    ));
    content.add(&setting_switch(
        "Hide NSFW Sites",
        "Keep them out of the site switcher",
        |d| d.settings.hide_nsfw_sites,
        |d, v| d.settings.hide_nsfw_sites = v,
    ));
    content.add(&setting_switch(
        "Mute Videos by Default",
        "Videos start without sound",
        |d| d.settings.mute_webms_by_default,
        |d, v| d.settings.mute_webms_by_default = v,
    ));
    content.add(&setting_switch(
        "Birthday Hats",
        "Party hats on post images every October 1st, 4chan's birthday",
        |d| d.settings.birthday_hats,
        |d, v| d.settings.birthday_hats = v,
    ));
    page.add(&content);

    // ---- Downloads ----
    let downloads_group = adw::PreferencesGroup::builder().title("Downloads").build();
    let folder = adw::ActionRow::builder()
        .title("Download Folder")
        .use_markup(false)
        .build();
    keep_synced(&folder, |row| {
        let custom = !state().data().settings.download_dir.trim().is_empty();
        let path = crate::downloads::base_dir();
        row.set_subtitle(&if custom {
            path.display().to_string()
        } else {
            format!("{} (default)", path.display())
        });
    });
    let reset = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Use Default Folder")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    keep_synced(&reset, |button| {
        button.set_visible(!state().data().settings.download_dir.trim().is_empty())
    });
    reset.connect_clicked(|_| state().update_settings(|d| d.settings.download_dir.clear()));
    let choose = gtk::Button::builder()
        .label("Choose…")
        .valign(gtk::Align::Center)
        .build();
    choose.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            let dialog = gtk::FileDialog::builder()
                .title("Download Folder")
                .initial_folder(&gtk::gio::File::for_path(crate::downloads::base_dir()))
                .build();
            dialog.select_folder(Some(&window), gtk::gio::Cancellable::NONE, |result| {
                if let Some(path) = result.ok().and_then(|f| f.path()) {
                    let path = path.to_string_lossy().into_owned();
                    state().update_settings(|d| d.settings.download_dir = path);
                }
            });
        }
    ));
    folder.add_suffix(&reset);
    folder.add_suffix(&choose);
    downloads_group.add(&folder);

    let namings = [
        (
            "Site, Board and Number",
            channex::data::store::FolderNaming::Id,
        ),
        ("Thread Subject", channex::data::store::FolderNaming::Name),
    ];
    let naming_row = adw::ComboRow::builder()
        .title("Thread Folder Names")
        .model(&gtk::StringList::new(&namings.map(|(label, _)| label)))
        .build();
    keep_synced(&naming_row, move |row| {
        let naming = state().data().settings.download_folder_naming;
        let index = namings.iter().position(|(_, n)| *n == naming).unwrap_or(0) as u32;
        row.set_subtitle(if index == 0 {
            "For example, 4chan-g-105076684"
        } else {
            "Named after the thread, or the start of its first post"
        });
        if row.selected() != index {
            row.set_selected(index);
        }
    });
    naming_row.connect_selected_notify(move |row| {
        let naming = namings[row.selected() as usize].1;
        if state().data().settings.download_folder_naming != naming {
            state().update_settings(|d| d.settings.download_folder_naming = naming);
        }
    });
    downloads_group.add(&naming_row);
    page.add(&downloads_group);

    // ---- Storage ----
    page.add(&storage_group(dialog));

    // ---- Welcome tour ----
    let tour = adw::PreferencesGroup::new();
    let tour_row = adw::ButtonRow::builder().title("Show Welcome Tour").build();
    tour_row.connect_activated(glib::clone!(
        #[weak]
        window,
        #[weak]
        dialog,
        move |_| {
            dialog.close();
            super::show_welcome(&window);
        }
    ));
    tour.add(&tour_row);
    page.add(&tour);

    page
}

fn sites_page(window: &ChannexWindow) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .name("sites")
        .title("Sites")
        .icon_name("web-browser-symbolic")
        .build();

    let custom = adw::PreferencesGroup::builder()
        .title("Custom Sites")
        .description("Any 4chan-API-compatible (vichan, etc.) or LynxChan imageboard.")
        .build();
    let custom_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    custom.add(&custom_list);
    let add_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .margin_top(12)
        .build();
    let add = adw::ButtonRow::builder()
        .title("Add Site…")
        .start_icon_name("list-add-symbolic")
        .build();
    add.connect_activated(glib::clone!(
        #[weak]
        window,
        move |row| super::show_add_site(&window, Some(row.upcast_ref()))
    ));
    add_list.append(&add);
    custom.add(&add_list);
    keep_synced(&custom_list, |list| {
        list.remove_all();
        let sites = state().data().settings.custom_sites.clone();
        list.set_visible(!sites.is_empty());
        for site in sites {
            let row = adw::ActionRow::builder()
                .title(&site.name)
                .subtitle(&site.site_origin)
                .use_markup(false)
                .build();
            let remove = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text("Remove Site")
                .valign(gtk::Align::Center)
                .css_classes(["flat"])
                .build();
            let id = site.id.clone();
            remove.connect_clicked(move |_| {
                state().update_settings(|d| d.settings.custom_sites.retain(|s| s.id != id));
            });
            row.add_suffix(&remove);
            list.append(&row);
        }
    });
    page.add(&custom);

    let hidden = adw::PreferencesGroup::builder()
        .title("Hidden Boards")
        .description("Right-click a board in the sidebar to hide it.")
        .build();
    let hidden_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    hidden.add(&hidden_list);
    keep_synced(&hidden_list, |list| {
        list.remove_all();
        let st = state();
        let hidden: Vec<(String, Vec<String>)> = {
            let data = st.data();
            let mut entries: Vec<_> = data
                .settings
                .hidden_boards
                .iter()
                .filter(|(_, b)| !b.is_empty())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            entries.sort();
            entries
        };
        if hidden.is_empty() {
            list.append(
                &adw::ActionRow::builder()
                    .title("No hidden boards")
                    .sensitive(false)
                    .build(),
            );
            return;
        }
        for (site_id, boards) in hidden {
            let name = st
                .site(&site_id)
                .map(|s| s.name)
                .unwrap_or_else(|| site_id.clone());
            let expander = adw::ExpanderRow::builder()
                .title(&name)
                .subtitle(if boards.len() == 1 {
                    "1 hidden board".to_owned()
                } else {
                    format!("{} hidden boards", boards.len())
                })
                .build();
            for code in boards {
                let row = adw::ActionRow::builder()
                    .title(format!("/{code}/"))
                    .use_markup(false)
                    .build();
                let show = gtk::Button::builder()
                    .label("Show")
                    .valign(gtk::Align::Center)
                    .build();
                let site_id = site_id.clone();
                show.connect_clicked(move |_| {
                    state()
                        .update_settings(|d| d.settings.set_board_hidden(&site_id, &code, false));
                });
                row.add_suffix(&show);
                expander.add_row(&row);
            }
            list.append(&expander);
        }
    });
    page.add(&hidden);
    page
}

/// Preferences → Storage: the image cache's size, a Clear button, and
/// when it's cleared automatically.
fn storage_group(dialog: &adw::PreferencesDialog) -> adw::PreferencesGroup {
    use channex::data::cache::{ClearInterval, SIZE_LIMITS_MB, format_size};

    let group = adw::PreferencesGroup::builder()
        .title("Storage")
        .description("Thumbnails and site icons are kept on disk so they load faster. Videos are streamed and never stored.")
        .build();

    let usage = adw::ActionRow::builder()
        .title("Image Cache")
        .subtitle("Calculating…")
        .build();
    let clear = gtk::Button::builder()
        .label("Clear")
        .valign(gtk::Align::Center)
        .build();
    usage.add_suffix(&clear);
    let refresh = glib::clone!(
        #[weak]
        usage,
        move || {
            glib::spawn_future_local(async move {
                let bytes = crate::cache::usage().await;
                usage.set_subtitle(&if bytes == 0 {
                    "Empty".to_owned()
                } else {
                    format_size(bytes)
                });
            });
        }
    );
    refresh();
    let refresh_after_clear = refresh.clone();
    clear.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        move |button| {
            button.set_sensitive(false);
            let refresh = refresh_after_clear.clone();
            let button = button.clone();
            glib::spawn_future_local(async move {
                crate::cache::clear().await;
                refresh();
                button.set_sensitive(true);
                dialog.add_toast(adw::Toast::new("Cache cleared"));
            });
        }
    ));
    group.add(&usage);

    let interval_row = adw::ComboRow::builder()
        .title("Clear Automatically")
        .model(&gtk::StringList::new(
            &ClearInterval::ALL.map(ClearInterval::label),
        ))
        .build();
    keep_synced(&interval_row, |row| {
        let interval = state().data().settings.cache_clear_interval;
        let index = ClearInterval::ALL
            .iter()
            .position(|i| *i == interval)
            .unwrap_or(2) as u32;
        if row.selected() != index {
            row.set_selected(index);
        }
    });
    interval_row.connect_selected_notify(|row| {
        let interval = ClearInterval::ALL[row.selected() as usize];
        if state().data().settings.cache_clear_interval != interval {
            state().update_settings(|d| d.settings.cache_clear_interval = interval);
        }
    });
    group.add(&interval_row);

    let labels = SIZE_LIMITS_MB.map(|mb| match mb {
        0 => "No Limit".to_owned(),
        mb if mb >= 1024 => format!("{} GB", mb / 1024),
        mb => format!("{mb} MB"),
    });
    let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    let limit_row = adw::ComboRow::builder()
        .title("Size Limit")
        .subtitle("The least recently used images are removed first")
        .model(&gtk::StringList::new(&label_refs))
        .build();
    keep_synced(&limit_row, |row| {
        let mb = state().data().settings.cache_size_limit_mb;
        let index = SIZE_LIMITS_MB.iter().position(|m| *m == mb).unwrap_or(3) as u32;
        if row.selected() != index {
            row.set_selected(index);
        }
    });
    limit_row.connect_selected_notify(move |row| {
        let mb = SIZE_LIMITS_MB[row.selected() as usize];
        if state().data().settings.cache_size_limit_mb != mb {
            state().update_settings(|d| d.settings.cache_size_limit_mb = mb);
            // Apply a lower limit right away.
            crate::cache::maintain();
            let refresh = refresh.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(500), refresh);
        }
    });
    group.add(&limit_row);
    group
}

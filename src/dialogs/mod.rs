//! Dialogs, and the settings controls they share.

mod add_site;
mod download_files;
mod preferences;
mod reply;
mod welcome;

pub use add_site::show_add_site;
pub use download_files::show_download_files;
pub use preferences::show_preferences;
pub use reply::{ReplyTarget, show_reply};
pub use welcome::show_welcome;

use adw::prelude::*;
use channex::data::store::AppData;

use crate::state::state;
use crate::theme;

/// Runs `sync` now and whenever settings change, until `widget` is
/// destroyed.
fn keep_synced<W: IsA<gtk::Widget>>(widget: &W, sync: impl Fn(&W) + 'static) {
    sync(widget);
    let weak = widget.downgrade();
    let handler = state().connect_local("settings-changed", false, move |_| {
        if let Some(widget) = weak.upgrade() {
            sync(&widget);
        }
        None
    });
    let handler = std::cell::RefCell::new(Some(handler));
    widget.connect_destroy(move |_| {
        if let Some(id) = handler.take() {
            state().disconnect(id);
        }
    });
}

/// System / Light / Dark. Insensitive while a classic skin (always
/// light) is active.
pub fn style_toggle() -> adw::ToggleGroup {
    let group = adw::ToggleGroup::builder()
        .valign(gtk::Align::Center)
        .build();
    for (name, label) in [("system", "System"), ("light", "Light"), ("dark", "Dark")] {
        group.add(adw::Toggle::builder().name(name).label(label).build());
    }
    keep_synced(&group, |group| {
        let theme = state().data().settings.theme.clone();
        let skin = theme::skin(&theme).is_some();
        group.set_sensitive(!skin);
        let name = if skin { "light" } else { theme.as_str() };
        if group.active_name().as_deref() != Some(name) {
            group.set_active_name(Some(name));
        }
    });
    group.connect_active_name_notify(|group| {
        let Some(name) = group.active_name() else {
            return;
        };
        if theme::skin(&state().data().settings.theme).is_some()
            || state().data().settings.theme == name
        {
            return;
        }
        state().update_settings(|d| d.settings.theme = name.to_string());
    });
    group
}

/// Round color swatches: "Follow System", GNOME's nine accents, and the
/// user's own color if it isn't one of them (e.g. imported from the Qt
/// version).
pub fn accent_picker() -> gtk::Box {
    let row = gtk::Box::builder()
        .spacing(10)
        .halign(gtk::Align::Start)
        .css_classes(["accent-picker"])
        .build();
    let current = state().data().settings.accent_color.clone();
    let custom = current.clone().filter(|hex| {
        !theme::ACCENTS
            .iter()
            .any(|(_, _, h)| h.eq_ignore_ascii_case(hex))
    });

    let mut swatches: Vec<(Option<String>, String, &str)> =
        vec![(None, "system".into(), "Follow System")];
    swatches.extend(
        theme::ACCENTS
            .iter()
            .map(|(id, label, hex)| (Some((*hex).to_owned()), (*id).to_owned(), *label)),
    );
    if let Some(hex) = &custom {
        swatches.push((Some(hex.clone()), "custom".into(), "Your Color"));
    }

    let mut first: Option<gtk::ToggleButton> = None;
    for (value, class, label) in swatches {
        let check = gtk::Image::builder()
            .icon_name("object-select-symbolic")
            .build();
        let button = gtk::ToggleButton::builder()
            .child(&check)
            .tooltip_text(label)
            .css_classes(["accent-swatch", class.as_str()])
            .build();
        button.update_property(&[gtk::accessible::Property::Label(label)]);
        if let Some(first) = &first {
            button.set_group(Some(first));
        } else {
            first = Some(button.clone());
        }
        let value_for_sync = value.clone();
        keep_synced(&button, move |button| {
            let st = state();
            let data = st.data();
            let active = data
                .settings
                .accent_color
                .as_deref()
                .map(str::to_ascii_lowercase)
                == value_for_sync.as_deref().map(str::to_ascii_lowercase);
            let skin = theme::skin(&data.settings.theme).is_some();
            drop(data);
            button.set_sensitive(!skin);
            if button.is_active() != active {
                button.set_active(active);
            }
        });
        button.connect_toggled(move |button| {
            if button.is_active() && state().data().settings.accent_color != value {
                let value = value.clone();
                state().update_settings(|d| d.settings.accent_color = value);
            }
        });
        row.append(&button);
    }
    row
}

/// A switch bound to one boolean setting.
pub fn setting_switch(
    title: &str,
    subtitle: &str,
    get: fn(&AppData) -> bool,
    set: fn(&mut AppData, bool),
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    keep_synced(&row, move |row| {
        let value = get(&state().data());
        if row.is_active() != value {
            row.set_active(value);
        }
    });
    row.connect_active_notify(move |row| {
        let value = row.is_active();
        if get(&state().data()) != value {
            state().update_settings(|d| set(d, value));
        }
    });
    row
}

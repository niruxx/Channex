//! "Add Site": a custom imageboard by address and engine.

use adw::prelude::*;
use channex::data::model::{MediaLayout, Schema};
use channex::data::sites::{CustomSiteInput, custom_site};
use gtk::glib;

use crate::state::state;
use crate::window::ChannexWindow;

/// Opens the dialog over `parent` (or the window).
pub fn show_add_site(window: &ChannexWindow, parent: Option<&gtk::Widget>) {
    let name = adw::EntryRow::builder().title("Name").build();
    let address = adw::EntryRow::builder()
        .title("Address")
        .input_purpose(gtk::InputPurpose::Url)
        .build();
    let engine = adw::ComboRow::builder()
        .title("Engine")
        .subtitle("vichan, Lainchan and 8kun are 4chan-style")
        .model(&gtk::StringList::new(&["4chan-Style", "LynxChan"]))
        .build();
    let layout = adw::ComboRow::builder()
        .title("Media Layout")
        .subtitle("Where the site keeps its images")
        .model(&gtk::StringList::new(&[
            "Board Folders (vichan)",
            "Flat (4chan CDN)",
            "File Store (8kun)",
        ]))
        .build();
    let thumbs = adw::ComboRow::builder()
        .title("Thumbnail Format")
        .model(&gtk::StringList::new(&["JPEG", "PNG"]))
        .build();
    let nsfw = adw::SwitchRow::builder()
        .title("NSFW Site")
        .subtitle("Blurs its images when that setting is on")
        .active(true)
        .build();
    engine
        .bind_property("selected", &layout, "visible")
        .transform_to(|_, selected: u32| Some(selected == 0))
        .sync_create()
        .build();
    engine
        .bind_property("selected", &thumbs, "visible")
        .transform_to(|_, selected: u32| Some(selected == 0))
        .sync_create()
        .build();

    let group = adw::PreferencesGroup::builder()
        .description("Any imageboard with a 4chan-compatible JSON API (vichan, Lainchan, …) or running LynxChan.")
        .build();
    for row in [
        name.upcast_ref::<gtk::Widget>(),
        address.upcast_ref(),
        engine.upcast_ref(),
        layout.upcast_ref(),
        thumbs.upcast_ref(),
        nsfw.upcast_ref(),
    ] {
        group.add(row);
    }
    let error = gtk::Label::builder()
        .css_classes(["error"])
        .wrap(true)
        .visible(false)
        .margin_top(12)
        .build();
    let content = adw::PreferencesPage::new();
    content.add(&group);
    let error_group = adw::PreferencesGroup::new();
    error_group.add(&error);
    content.add(&error_group);

    let cancel = gtk::Button::builder().label("Cancel").build();
    let add = gtk::Button::builder()
        .label("Add")
        .css_classes(["suggested-action"])
        .sensitive(false)
        .build();
    let header = adw::HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();
    header.pack_start(&cancel);
    header.pack_end(&add);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&content));

    let dialog = adw::Dialog::builder()
        .title("Add Site")
        .content_width(460)
        .child(&toolbar)
        .default_widget(&add)
        .build();

    let update_add = glib::clone!(
        #[weak]
        name,
        #[weak]
        address,
        #[weak]
        add,
        move || add
            .set_sensitive(!name.text().trim().is_empty() && !address.text().trim().is_empty())
    );
    let u = update_add.clone();
    name.connect_changed(move |_| u());
    address.connect_changed(move |_| update_add());
    address.set_activates_default(true);

    cancel.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));
    add.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        #[weak]
        window,
        move |_| {
            let input = CustomSiteInput {
                name: name.text().to_string(),
                origin: address.text().to_string(),
                media_origin: None,
                schema: if engine.selected() == 1 {
                    Schema::Lynxchan
                } else {
                    Schema::Yotsuba
                },
                media_layout: match layout.selected() {
                    1 => MediaLayout::FlatCdn,
                    2 => MediaLayout::FileStore,
                    _ => MediaLayout::BoardDirs,
                },
                nsfw: nsfw.is_active(),
            };
            let id = format!("custom-{}", glib::real_time() / 1000);
            match custom_site(id.clone(), &input) {
                Ok(mut site) => {
                    if input.schema == Schema::Yotsuba && thumbs.selected() == 1 {
                        site.thumb_ext = Some(".png".into());
                    }
                    state().update_settings(|d| d.settings.custom_sites.push(site));
                    dialog.close();
                    window.show_site(&id, None);
                    window.toast(&format!("Added {}", input.name.trim()));
                }
                Err(err) => {
                    error.set_label(&err.to_string());
                    error.set_visible(true);
                }
            }
        }
    ));

    match parent {
        Some(parent) => dialog.present(Some(parent)),
        None => dialog.present(Some(window)),
    }
}

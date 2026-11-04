//! The sidebar header's site picker: a menu button showing the current
//! site (favicon + name) that opens a list of every site. NSFW sites
//! are left out when "Hide NSFW sites" is on, unless revealed or
//! currently open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use channex::data::model::Site;
use gtk::glib;

use crate::state::state;
use crate::thumbnails;

type SelectedCallback = Box<dyn Fn(&str)>;

#[derive(Clone)]
pub struct SiteSwitcher {
    inner: Rc<Inner>,
}

struct Inner {
    button: gtk::MenuButton,
    avatar: adw::Avatar,
    label: gtk::Label,
    list: gtk::ListBox,
    popover: gtk::Popover,
    current: RefCell<String>,
    reveal_hidden: Cell<bool>,
    on_selected: RefCell<Option<SelectedCallback>>,
}

impl SiteSwitcher {
    pub fn new() -> Self {
        let avatar = adw::Avatar::builder().size(20).show_initials(true).build();
        let label = gtk::Label::builder()
            .css_classes(["heading"])
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        let content = gtk::Box::builder().spacing(8).build();
        content.append(&avatar);
        content.append(&label);

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["navigation-sidebar"])
            .build();
        let popover = gtk::Popover::builder().child(&list).has_arrow(true).build();
        popover.add_css_class("menu");
        let button = gtk::MenuButton::builder()
            .child(&content)
            .popover(&popover)
            .always_show_arrow(true)
            .tooltip_text("Switch Site")
            .build();
        button.add_css_class("flat");

        let this = Self {
            inner: Rc::new(Inner {
                button,
                avatar,
                label,
                list,
                popover: popover.clone(),
                current: RefCell::default(),
                reveal_hidden: Cell::new(false),
                on_selected: RefCell::default(),
            }),
        };

        // Rebuilt each time it opens, so settings changes apply.
        let weak = Rc::downgrade(&this.inner);
        popover.connect_show(move |_| {
            if let Some(inner) = weak.upgrade() {
                Self { inner }.rebuild();
            }
        });
        let weak = Rc::downgrade(&this.inner);
        this.inner.list.connect_row_activated(move |_, row| {
            let Some(inner) = weak.upgrade() else { return };
            let this = Self { inner };
            match row.widget_name().as_str() {
                "add-site" => {
                    this.inner.popover.popdown();
                    let _ = this.inner.button.activate_action("win.add-site", None);
                }
                "toggle-hidden" => {
                    this.inner
                        .reveal_hidden
                        .set(!this.inner.reveal_hidden.get());
                    this.rebuild();
                }
                site_id => {
                    let site_id = site_id.to_owned();
                    this.inner.popover.popdown();
                    if let Some(cb) = this.inner.on_selected.borrow().as_ref() {
                        cb(&site_id);
                    }
                }
            }
        });
        this
    }

    pub fn widget(&self) -> &gtk::MenuButton {
        &self.inner.button
    }

    pub fn connect_site_selected(&self, f: impl Fn(&str) + 'static) {
        self.inner.on_selected.replace(Some(Box::new(f)));
    }

    /// Shows `site_id` as the current site.
    pub fn set_current(&self, site_id: &str) {
        self.inner.current.replace(site_id.to_owned());
        let Some(site) = state().site(site_id) else {
            return;
        };
        self.inner.label.set_label(&site.name);
        set_avatar(&self.inner.avatar, &site);
    }

    fn rebuild(&self) {
        let inner = &self.inner;
        inner.list.remove_all();
        let current = inner.current.borrow().clone();
        let hide_nsfw = state().data().settings.hide_nsfw_sites;
        let sites = state().sites();
        let shown =
            |s: &Site| inner.reveal_hidden.get() || !hide_nsfw || !s.nsfw || s.id == current;
        let hidden_count = sites.iter().filter(|s| !shown(s)).count();

        for site in sites.iter().filter(|s| shown(s)) {
            let avatar = adw::Avatar::builder().size(24).show_initials(true).build();
            set_avatar(&avatar, site);
            let row_box = gtk::Box::builder().spacing(10).build();
            row_box.append(&avatar);
            row_box.append(
                &gtk::Label::builder()
                    .label(&site.name)
                    .xalign(0.0)
                    .hexpand(true)
                    .build(),
            );
            if site.nsfw {
                row_box.append(
                    &gtk::Label::builder()
                        .label("NSFW")
                        .css_classes(["caption", "dim-label"])
                        .build(),
                );
            }
            let check = gtk::Image::from_icon_name("object-select-symbolic");
            check.set_opacity(if site.id == current { 1.0 } else { 0.0 });
            row_box.append(&check);
            let row = gtk::ListBoxRow::builder()
                .child(&row_box)
                .name(&site.id)
                .build();
            inner.list.append(&row);
        }

        if hidden_count > 0 || inner.reveal_hidden.get() {
            let label = if inner.reveal_hidden.get() {
                "Hide NSFW Sites".to_owned()
            } else if hidden_count == 1 {
                "Show 1 Hidden Site".to_owned()
            } else {
                format!("Show {hidden_count} Hidden Sites")
            };
            let row_label = gtk::Label::builder()
                .label(label)
                .xalign(0.0)
                .css_classes(["dim-label"])
                .build();
            inner.list.append(
                &gtk::ListBoxRow::builder()
                    .child(&row_label)
                    .name("toggle-hidden")
                    .build(),
            );
        }

        let add_label = gtk::Box::builder().spacing(10).build();
        add_label.append(&gtk::Image::from_icon_name("list-add-symbolic"));
        add_label.append(&gtk::Label::builder().label("Add Site…").xalign(0.0).build());
        inner.list.append(
            &gtk::ListBoxRow::builder()
                .child(&add_label)
                .name("add-site")
                .build(),
        );
    }
}

/// Initials right away, the site's favicon once it loads.
fn set_avatar(avatar: &adw::Avatar, site: &Site) {
    avatar.set_text(Some(&site.name));
    avatar.set_custom_image(gtk::gdk::Paintable::NONE);
    if site.favicon.is_empty() {
        return;
    }
    let (favicon, name) = (site.favicon.clone(), site.name.clone());
    let avatar = avatar.downgrade();
    glib::spawn_future_local(async move {
        if let (Some(texture), Some(avatar)) = (thumbnails::load(&favicon).await, avatar.upgrade())
        {
            // Skip if the avatar was switched to another site meanwhile.
            if avatar.text().as_deref() == Some(name.as_str()) {
                avatar.set_custom_image(Some(&texture));
            }
        }
    });
}

//! First-run welcome tour: a short carousel (welcome, appearance,
//! content, shortcuts). Closing it or finishing marks onboarding done;
//! it can be replayed from Preferences.

use adw::prelude::*;
use gtk::glib;

use super::{accent_picker, setting_switch, style_toggle};
use crate::config;
use crate::state::state;
use crate::window::ChannexWindow;

pub fn show_welcome(window: &ChannexWindow) {
    let carousel = adw::Carousel::builder()
        .vexpand(true)
        .allow_scroll_wheel(false)
        .build();

    // 1. Welcome
    carousel.append(&page(
        Some(config::APP_ID),
        "Welcome to Channex",
        "Browse imageboards in a native app. Pick a site and a board in the sidebar; threads open with a click.",
        None,
    ));

    // 2. Appearance
    let appearance = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .halign(gtk::Align::Center)
        .build();
    appearance.append(&style_toggle());
    appearance.append(&accent_picker());
    carousel.append(&page(Some("applications-graphics-symbolic"), "Make It Yours", "Choose a style and an accent color. There are classic imageboard skins in Preferences too.", Some(appearance.upcast_ref())));

    // 3. Content
    let content = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    content.append(&setting_switch(
        "Blur NSFW Images and Spoilers",
        "Click a thumbnail to reveal it",
        |d| d.settings.blur_nsfw,
        |d, v| d.settings.blur_nsfw = v,
    ));
    content.append(&setting_switch(
        "Hide NSFW Sites",
        "Keep them out of the site switcher",
        |d| d.settings.hide_nsfw_sites,
        |d, v| d.settings.hide_nsfw_sites = v,
    ));
    carousel.append(&page(
        Some("view-conceal-symbolic"),
        "Sensitive Content",
        "Choose how sensitive content is handled. You can change this any time in Preferences.",
        Some(content.upcast_ref()),
    ));

    // 4. Done
    let tips = gtk::Label::builder()
        .use_markup(true)
        .justify(gtk::Justification::Center)
        .label(
            "<b>Ctrl+F</b> search · <b>F5</b> refresh · <b>Ctrl+D</b> bookmark · <b>F9</b> sidebar",
        )
        .css_classes(["dim-label"])
        .build();
    carousel.append(&page(
        Some("object-select-symbolic"),
        "You're All Set",
        "Bookmark threads to track new replies. A few handy shortcuts:",
        Some(tips.upcast_ref()),
    ));

    let dots = adw::CarouselIndicatorDots::builder()
        .carousel(&carousel)
        .build();
    let back = gtk::Button::builder().label("Back").build();
    let next = gtk::Button::builder()
        .label("Next")
        .css_classes(["suggested-action"])
        .build();
    let bottom = gtk::CenterBox::builder()
        .margin_start(18)
        .margin_end(18)
        .margin_top(6)
        .margin_bottom(18)
        .build();
    bottom.set_start_widget(Some(&back));
    bottom.set_center_widget(Some(&dots));
    bottom.set_end_widget(Some(&next));

    let header = adw::HeaderBar::builder().show_title(false).build();
    let skip = gtk::Button::builder()
        .label("Skip")
        .css_classes(["flat"])
        .build();
    header.pack_start(&skip);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&carousel));
    toolbar.add_bottom_bar(&bottom);

    let dialog = adw::Dialog::builder()
        .content_width(560)
        .content_height(600)
        .child(&toolbar)
        .title("Welcome")
        .build();
    dialog.connect_closed(|_| {
        if !state().data().settings.has_completed_onboarding {
            state().update_settings(|d| d.settings.has_completed_onboarding = true);
        }
    });

    let pages = carousel.n_pages();
    let update = glib::clone!(
        #[weak]
        carousel,
        #[weak]
        back,
        #[weak]
        next,
        #[weak]
        skip,
        move || {
            let index = carousel.position().round() as u32;
            back.set_sensitive(index > 0);
            skip.set_visible(index + 1 < pages);
            next.set_label(if index + 1 >= pages {
                "Start Browsing"
            } else {
                "Next"
            });
        }
    );
    update();
    let u = update.clone();
    carousel.connect_page_changed(move |_, _| u());

    back.connect_clicked(glib::clone!(
        #[weak]
        carousel,
        move |_| {
            let index = carousel.position().round() as u32;
            if index > 0 {
                carousel.scroll_to(&carousel.nth_page(index - 1), true);
            }
        }
    ));
    next.connect_clicked(glib::clone!(
        #[weak]
        carousel,
        #[weak]
        dialog,
        move |_| {
            let index = carousel.position().round() as u32;
            if index + 1 < pages {
                carousel.scroll_to(&carousel.nth_page(index + 1), true);
            } else {
                dialog.close();
            }
        }
    ));
    skip.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));

    dialog.present(Some(window));
}

fn page(
    icon: Option<&str>,
    title: &str,
    description: &str,
    child: Option<&gtk::Widget>,
) -> adw::StatusPage {
    let status = adw::StatusPage::builder()
        .title(title)
        .description(description)
        .hexpand(true)
        .css_classes(["welcome-page"])
        .build();
    if let Some(icon) = icon {
        status.set_icon_name(Some(icon));
    }
    if let Some(child) = child {
        status.set_child(Some(child));
    }
    status
}

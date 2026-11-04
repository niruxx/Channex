//! Birthday hats: on October 1st (4chan's anniversary), a party hat sits
//! on post images, as the site does it. Off with the "Birthday Hats"
//! switch in Preferences.

use gtk::glib;
use gtk::prelude::*;

use crate::state::state;

/// Whether hats are showing right now.
pub fn hats_today() -> bool {
    if !state().data().settings.birthday_hats {
        return false;
    }
    // Debug builds: CHANNEX_DEV_BIRTHDAY=1 pretends it's October 1st.
    if cfg!(debug_assertions) && std::env::var_os("CHANNEX_DEV_BIRTHDAY").is_some() {
        return true;
    }
    glib::DateTime::now_local().is_ok_and(|now| now.month() == 10 && now.day_of_month() == 1)
}

/// A party hat to overlay on a thumbnail's top-left corner. `size` is
/// its width and height in pixels.
pub fn hat(size: i32) -> gtk::Picture {
    let hat = gtk::Picture::for_resource("/io/github/niruxx/Channex/party-hat.svg");
    hat.set_size_request(size, size);
    hat.set_can_shrink(true);
    hat.set_halign(gtk::Align::Start);
    hat.set_valign(gtk::Align::Start);
    hat.set_can_target(false);
    hat.set_tooltip_text(Some("Happy birthday, 4chan!"));
    hat.add_css_class("party-hat");
    hat
}

mod application;
mod config;
#[cfg(debug_assertions)]
mod devtools;
mod dialogs;
mod downloads;
mod pages;
mod runtime;
mod state;
mod theme;
mod thumbnails;
mod viewer;
mod widgets;
mod window;
mod window_state;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::application::ChannexApplication;

fn main() -> glib::ExitCode {
    gio::resources_register_include!("channex.gresource")
        .expect("failed to register bundled resources");

    glib::set_application_name(config::APP_NAME);

    ChannexApplication::new().run()
}

//! The `AdwApplication` subclass: owns app-wide actions (quit, about,
//! shortcuts) and their accelerators, and opens the main window.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::window::ChannexWindow;
use crate::{config, state, thumbnails};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ChannexApplication {
        /// What happened opening the data file, shown once in the first
        /// window.
        pub load_outcome: std::cell::RefCell<Option<channex::data::store::LoadOutcome>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannexApplication {
        const NAME: &'static str = "ChannexApplication";
        type Type = super::ChannexApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for ChannexApplication {}

    impl ApplicationImpl for ChannexApplication {
        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            self.load_outcome.replace(Some(state::init()));
            thumbnails::prune_disk_cache();
            crate::theme::apply();
            state::state().connect_closure(
                "settings-changed",
                false,
                glib::closure_local!(|_: state::State| crate::theme::apply()),
            );
            app.setup_actions();
        }

        fn activate(&self) {
            let app = self.obj();
            // Single-instance: re-activating just presents the window.
            if let Some(window) = app.active_window() {
                window.present();
                return;
            }
            let window = ChannexWindow::new(&*app);
            if let Some(outcome) = self.load_outcome.take() {
                window.report_load_outcome(&outcome);
            }
            window.present();
            if !state::state().data().settings.has_completed_onboarding {
                crate::dialogs::show_welcome(&window);
            }
            #[cfg(debug_assertions)]
            crate::devtools::schedule_screenshot(&window);
        }
    }

    impl GtkApplicationImpl for ChannexApplication {}
    impl AdwApplicationImpl for ChannexApplication {}
}

glib::wrapper! {
    pub struct ChannexApplication(ObjectSubclass<imp::ChannexApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl ChannexApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", config::RESOURCE_BASE)
            .build()
    }

    fn setup_actions(&self) {
        let quit = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| app.quit_all())
            .build();
        let about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| app.show_about())
            .build();
        let shortcuts = gio::ActionEntry::builder("shortcuts")
            .activate(|app: &Self, _, _| app.show_shortcuts())
            .build();
        self.add_action_entries([quit, about, shortcuts]);

        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("app.shortcuts", &["<Control>question"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
        self.set_accels_for_action("win.toggle-sidebar", &["F9"]);
        self.set_accels_for_action("win.search", &["<Control>f"]);
        self.set_accels_for_action("win.refresh", &["F5", "<Control>r"]);
        self.set_accels_for_action("win.bookmark", &["<Control>d"]);
        self.set_accels_for_action("win.preferences", &["<Control>comma"]);
    }

    /// Closes windows through their normal close path (so they save
    /// their size) rather than tearing the app down directly.
    fn quit_all(&self) {
        for window in self.windows() {
            window.close();
        }
    }

    fn show_about(&self) {
        let dialog = adw::AboutDialog::builder()
            .application_name(config::APP_NAME)
            .application_icon(config::APP_ID)
            .developer_name("niruxxdaboi")
            .version(config::VERSION)
            .website("https://github.com/niruxx/Channex")
            .issue_url("https://github.com/niruxx/Channex/issues")
            .comments("A native imageboard browser.")
            .build();
        dialog.add_legal_section(
            "AI Usage",
            None,
            gtk::License::Custom,
            Some("Parts of this project (code, documentation, and/or assets) were written or assisted by AI tools."),
        );
        dialog.present(self.active_window().as_ref());
    }

    fn show_shortcuts(&self) {
        let builder =
            gtk::Builder::from_resource(&format!("{}/ui/shortcuts.ui", config::RESOURCE_BASE));
        let dialog: adw::ShortcutsDialog = builder
            .object("shortcuts_dialog")
            .expect("shortcuts.ui is missing shortcuts_dialog");
        dialog.present(self.active_window().as_ref());
    }
}

impl Default for ChannexApplication {
    fn default() -> Self {
        Self::new()
    }
}

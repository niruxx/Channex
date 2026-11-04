//! App-wide state shared by every window: the settings/bookmarks store,
//! the HTTP client, and each site's board list. Widgets read from it and
//! listen to its signals to stay in sync:
//!
//! - `settings-changed` - any setting (including custom sites)
//! - `bookmarks-changed`
//! - `boards-changed(site_id)` - a site's board list loaded or failed

use std::cell::{OnceCell, Ref, RefCell};
use std::collections::HashMap;
use std::sync::OnceLock;

use adw::prelude::*;
use channex::data::adapters;
use channex::data::http::HttpClient;
use channex::data::model::{Board, Site};
use channex::data::sites;
use channex::data::store::{AppData, LoadOutcome, Store, legacy_qt_data_file};
use gtk::glib;
use gtk::glib::subclass::Signal;
use gtk::subclass::prelude::*;

use crate::runtime;

/// One site's board list as the sidebar sees it.
#[derive(Debug, Clone, Default)]
pub struct BoardList {
    /// Discovered boards merged with the site's built-in ones.
    pub boards: Vec<Board>,
    pub loading: bool,
    /// Why discovery failed (the built-in boards are shown instead).
    pub error: Option<String>,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct State {
        pub store: OnceCell<RefCell<Store>>,
        pub http: OnceCell<HttpClient>,
        pub boards: RefCell<HashMap<String, BoardList>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for State {
        const NAME: &'static str = "ChannexState";
        type Type = super::State;
    }

    impl ObjectImpl for State {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("settings-changed").build(),
                    Signal::builder("bookmarks-changed").build(),
                    Signal::builder("boards-changed")
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct State(ObjectSubclass<imp::State>);
}

thread_local! {
    static STATE: OnceCell<State> = const { OnceCell::new() };
}

/// The app's state. Panics if called before [`init`].
pub fn state() -> State {
    STATE.with(|s| s.get().expect("state::init wasn't called").clone())
}

/// Opens the store (importing the Qt version's data on first run) and
/// creates the state. Returns what happened when loading, for a notice.
pub fn init() -> LoadOutcome {
    let path = glib::user_data_dir().join("channex").join("channex.json");
    let legacy = legacy_qt_data_file();
    let (store, outcome) = match Store::open(&path, legacy.as_deref()) {
        Ok(result) => result,
        Err(err) => {
            glib::g_warning!("channex", "Couldn't open {}: {err}", path.display());
            // Keep running with defaults; saving will retry the path.
            let (store, _) = Store::open(std::env::temp_dir().join("channex-fallback.json"), None)
                .expect("temp store");
            (store, LoadOutcome::Loaded)
        }
    };
    let state: State = glib::Object::new();
    let _ = state.imp().store.set(RefCell::new(store));
    let _ = state
        .imp()
        .http
        .set(HttpClient::new().expect("couldn't create the HTTP client"));
    STATE.with(|s| {
        let _ = s.set(state);
    });
    outcome
}

impl State {
    pub fn http(&self) -> HttpClient {
        self.imp().http.get().expect("http").clone()
    }

    fn store(&self) -> &RefCell<Store> {
        self.imp().store.get().expect("store")
    }

    /// Read access to settings and bookmarks. Don't hold it across a
    /// call that updates the state.
    pub fn data(&self) -> Ref<'_, AppData> {
        Ref::map(self.store().borrow(), Store::data)
    }

    /// Changes settings, saves, and emits `settings-changed`.
    pub fn update_settings<R>(&self, f: impl FnOnce(&mut AppData) -> R) -> R {
        let result = self.update(f);
        self.emit_by_name::<()>("settings-changed", &[]);
        result
    }

    /// Changes bookmarks, saves, and emits `bookmarks-changed`.
    pub fn update_bookmarks<R>(&self, f: impl FnOnce(&mut AppData) -> R) -> R {
        let result = self.update(f);
        self.emit_by_name::<()>("bookmarks-changed", &[]);
        result
    }

    fn update<R>(&self, f: impl FnOnce(&mut AppData) -> R) -> R {
        let mut store = self.store().borrow_mut();
        let mut result = None;
        if let Err(err) = store.update(|d| result = Some(f(d))) {
            glib::g_warning!("channex", "Couldn't save settings: {err}");
        }
        result.expect("update closure ran")
    }

    /// Presets followed by custom sites.
    pub fn sites(&self) -> Vec<Site> {
        sites::all_sites(&self.data().settings.custom_sites)
    }

    pub fn site(&self, id: &str) -> Option<Site> {
        self.sites().into_iter().find(|s| s.id == id)
    }

    /// The site's board list as last loaded. Starts a load the first
    /// time a site is asked for.
    pub fn boards(&self, site_id: &str) -> BoardList {
        let cached = self.imp().boards.borrow().get(site_id).cloned();
        match cached {
            Some(list) => list,
            None => {
                self.load_boards(site_id);
                self.imp()
                    .boards
                    .borrow()
                    .get(site_id)
                    .cloned()
                    .unwrap_or_default()
            }
        }
    }

    /// (Re)fetches a site's boards in the background.
    pub fn load_boards(&self, site_id: &str) {
        let Some(site) = self.site(site_id) else {
            return;
        };
        {
            let mut boards = self.imp().boards.borrow_mut();
            let entry = boards.entry(site_id.to_owned()).or_default();
            if entry.loading {
                return;
            }
            entry.loading = true;
            if entry.boards.is_empty() {
                entry.boards = sites::merge_default_boards(Vec::new(), &site.default_boards);
            }
        }
        self.emit_by_name::<()>("boards-changed", &[&site_id.to_owned()]);

        let client = self.http();
        let this = self.clone();
        glib::spawn_future_local(async move {
            let fetch_site = site.clone();
            let result =
                runtime::spawn(async move { adapters::fetch_boards(&client, &fetch_site).await })
                    .await;
            let list = match result {
                Ok(found) => BoardList {
                    boards: sites::merge_default_boards(found, &site.default_boards),
                    loading: false,
                    error: None,
                },
                Err(err) => BoardList {
                    boards: sites::merge_default_boards(Vec::new(), &site.default_boards),
                    loading: false,
                    error: Some(err.to_string()),
                },
            };
            this.imp().boards.borrow_mut().insert(site.id.clone(), list);
            this.emit_by_name::<()>("boards-changed", &[&site.id]);
        });
    }
}

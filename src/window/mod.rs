//! Main window: a sidebar (site switcher, board filter, Library, Boards)
//! beside an `AdwNavigationView` that shows the catalog, bookmarks, and
//! pushed thread pages. Layout lives in data/ui/window.blp.

mod site_switcher;

use std::cell::{OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use channex::data::model::Board;
use channex::data::store::LoadOutcome;
use gtk::{gio, glib};

use self::site_switcher::SiteSwitcher;
use crate::downloads::{DownloadManager, downloads};
use crate::pages::{CatalogPage, ThreadPage, bookmarks_page, downloads_page};
use crate::state::state;
use crate::window_state::WindowState;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/niruxx/Channex/ui/window.ui")]
    pub struct ChannexWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub sidebar_header: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub board_filter: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub library_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub boards_spinner: TemplateChild<adw::Spinner>,
        #[template_child]
        pub board_count: TemplateChild<gtk::Label>,
        #[template_child]
        pub boards_error: TemplateChild<gtk::Label>,
        #[template_child]
        pub board_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub boards_empty: TemplateChild<gtk::Label>,
        #[template_child]
        pub nav_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub main_overlay: TemplateChild<gtk::Overlay>,
        pub viewer: OnceCell<crate::viewer::MediaViewer>,

        pub site_switcher: OnceCell<SiteSwitcher>,
        pub bookmarks_badge: OnceCell<gtk::Label>,
        pub downloads_badge: OnceCell<gtk::Label>,
        /// The site whose boards the sidebar shows.
        pub site_id: RefCell<String>,
        /// Boards currently listed, in row order.
        pub boards: RefCell<Vec<Board>>,
        /// What's open in the content pane, to restore selection.
        pub current_board: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannexWindow {
        const NAME: &'static str = "ChannexWindow";
        type Type = super::ChannexWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChannexWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_actions();
            let viewer = crate::viewer::MediaViewer::new(&obj);
            self.main_overlay.add_overlay(&viewer);
            let _ = self.viewer.set(viewer);
            obj.setup_sidebar();
            obj.restore_state();
        }
    }

    impl WidgetImpl for ChannexWindow {}

    impl WindowImpl for ChannexWindow {
        fn close_request(&self) -> glib::Propagation {
            self.obj().save_state();
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for ChannexWindow {}
    impl AdwApplicationWindowImpl for ChannexWindow {}
}

glib::wrapper! {
    pub struct ChannexWindow(ObjectSubclass<imp::ChannexWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ChannexWindow {
    pub fn new<P: IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn add_toast(&self, toast: adw::Toast) {
        self.imp().toast_overlay.add_toast(toast);
    }

    pub fn toast(&self, message: &str) {
        self.add_toast(adw::Toast::new(message));
    }

    /// A toggle for the page header bars that shows/hides the sidebar.
    pub fn sidebar_toggle(&self) -> gtk::ToggleButton {
        let button = gtk::ToggleButton::builder()
            .icon_name("sidebar-show-symbolic")
            .tooltip_text("Show Sidebar")
            .build();
        self.imp()
            .split_view
            .bind_property("show-sidebar", &button, "active")
            .bidirectional()
            .sync_create()
            .build();
        button
    }

    /// Tells the user about a one-time event from opening the data file.
    pub fn report_load_outcome(&self, outcome: &LoadOutcome) {
        match outcome {
            LoadOutcome::Loaded => {}
            LoadOutcome::ImportedFromQt => {
                self.toast("Settings and bookmarks imported from the previous version")
            }
            LoadOutcome::Corrupt { backup } => {
                let toast = adw::Toast::new(&format!(
                    "Settings file was damaged and has been reset. The old copy is at {}",
                    backup.display()
                ));
                toast.set_timeout(0);
                self.add_toast(toast);
            }
        }
    }

    // ------------------------------------------------------------------
    // Actions
    // ------------------------------------------------------------------

    fn setup_actions(&self) {
        let toggle_sidebar = gio::ActionEntry::builder("toggle-sidebar")
            .activate(|win: &Self, _, _| {
                let split = &win.imp().split_view;
                split.set_show_sidebar(!split.shows_sidebar());
            })
            .build();
        let search = gio::ActionEntry::builder("search")
            .activate(|win: &Self, _, _| {
                if let Some(page) = win.visible_catalog() {
                    page.start_search();
                }
            })
            .build();
        let refresh = gio::ActionEntry::builder("refresh")
            .activate(|win: &Self, _, _| {
                let page = win.imp().nav_view.visible_page();
                if let Some(catalog) = page.clone().and_downcast::<CatalogPage>() {
                    catalog.reload();
                } else if let Some(thread) = page.and_downcast::<ThreadPage>() {
                    thread.reload();
                }
            })
            .build();
        let bookmark = gio::ActionEntry::builder("bookmark")
            .activate(|win: &Self, _, _| {
                if let Some(thread) = win
                    .imp()
                    .nav_view
                    .visible_page()
                    .and_downcast::<ThreadPage>()
                {
                    thread.toggle_bookmark();
                }
            })
            .build();
        let hide_board = gio::ActionEntry::builder("hide-board")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|win: &Self, _, param| {
                if let Some(code) = param.and_then(|p| p.get::<String>()) {
                    win.set_board_hidden(&code, true);
                }
            })
            .build();
        let unhide_board = gio::ActionEntry::builder("unhide-board")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|win: &Self, _, param| {
                if let Some(code) = param.and_then(|p| p.get::<String>()) {
                    win.set_board_hidden(&code, false);
                }
            })
            .build();
        let preferences = gio::ActionEntry::builder("preferences")
            .activate(|win: &Self, _, _| crate::dialogs::show_preferences(win, None))
            .build();
        let add_site = gio::ActionEntry::builder("add-site")
            .activate(|win: &Self, _, _| crate::dialogs::show_add_site(win, None))
            .build();
        let open_folder = gio::ActionEntry::builder("open-folder")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|win: &Self, _, param| {
                if let Some(path) = param.and_then(|p| p.get::<String>()) {
                    let file = gio::File::for_path(path);
                    gtk::FileLauncher::new(Some(&file)).launch(
                        Some(win),
                        gio::Cancellable::NONE,
                        |_| {},
                    );
                }
            })
            .build();
        let show_downloads = gio::ActionEntry::builder("show-downloads")
            .activate(|win: &Self, _, _| win.open_downloads())
            .build();
        self.add_action_entries([
            open_folder,
            show_downloads,
            toggle_sidebar,
            search,
            refresh,
            bookmark,
            hide_board,
            unhide_board,
            preferences,
            add_site,
        ]);
    }

    fn visible_catalog(&self) -> Option<CatalogPage> {
        self.imp()
            .nav_view
            .visible_page()
            .and_downcast::<CatalogPage>()
    }

    fn set_board_hidden(&self, code: &str, hidden: bool) {
        let site_id = self.imp().site_id.borrow().clone();
        state().update_settings(|d| d.settings.set_board_hidden(&site_id, code, hidden));
        if hidden {
            let toast = adw::Toast::builder()
                .title(format!("/{code}/ hidden from the sidebar"))
                .button_label("Undo")
                .action_name("win.unhide-board")
                .action_target(&code.to_variant())
                .build();
            self.add_toast(toast);
        }
    }

    // ------------------------------------------------------------------
    // Sidebar
    // ------------------------------------------------------------------

    fn setup_sidebar(&self) {
        let imp = self.imp();

        let switcher = SiteSwitcher::new();
        imp.sidebar_header.set_title_widget(Some(switcher.widget()));
        switcher.connect_site_selected(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |site_id| win.show_site(site_id, None)
        ));
        let _ = imp.site_switcher.set(switcher);

        // Library rows.
        let badge = gtk::Label::builder()
            .css_classes(["sidebar-badge"])
            .valign(gtk::Align::Center)
            .build();
        imp.library_list.append(&library_row(
            "user-bookmarks-symbolic",
            "Bookmarks",
            Some(&badge),
        ));
        let _ = imp.bookmarks_badge.set(badge);
        self.update_bookmark_badge();
        let downloads_badge = gtk::Label::builder()
            .css_classes(["sidebar-badge", "accent-badge"])
            .valign(gtk::Align::Center)
            .tooltip_text("Downloads in progress")
            .build();
        imp.library_list.append(&library_row(
            "folder-download-symbolic",
            "Downloads",
            Some(&downloads_badge),
        ));
        let _ = imp.downloads_badge.set(downloads_badge);
        self.update_downloads_badge();
        imp.library_list.connect_row_activated(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_, row| if row.index() == 0 {
                win.open_bookmarks()
            } else {
                win.open_downloads()
            }
        ));

        let manager = downloads();
        manager.connect_closure(
            "changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = win)]
                self,
                move |_: DownloadManager, _id: u64| win.update_downloads_badge()
            ),
        );
        manager.connect_closure(
            "job-finished",
            false,
            glib::closure_local!(
                #[watch(rename_to = win)]
                self,
                move |_: DownloadManager, id: u64| win.announce_finished_job(id)
            ),
        );

        imp.board_list.connect_row_activated(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_, row| {
                let board = win.imp().boards.borrow().get(row.index() as usize).cloned();
                if let Some(board) = board {
                    let site_id = win.imp().site_id.borrow().clone();
                    win.open_board(&site_id, &board.code);
                }
            }
        ));

        // Board filter.
        imp.board_list.set_filter_func(glib::clone!(
            #[weak(rename_to = win)]
            self,
            #[upgrade_or]
            true,
            move |row| win.board_row_matches(row)
        ));
        imp.board_filter.connect_search_changed(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                win.imp().board_list.invalidate_filter();
                win.update_board_empty_state();
            }
        ));
        imp.board_filter
            .connect_stop_search(|entry| entry.set_text(""));

        let st = state();
        st.connect_closure(
            "boards-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = win)]
                self,
                move |_: crate::state::State, site_id: String| {
                    if *win.imp().site_id.borrow() == site_id {
                        win.rebuild_board_list();
                    }
                }
            ),
        );
        st.connect_closure(
            "settings-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = win)]
                self,
                move |_: crate::state::State| win.rebuild_board_list()
            ),
        );
        st.connect_closure(
            "bookmarks-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = win)]
                self,
                move |_: crate::state::State| win.update_bookmark_badge()
            ),
        );
    }

    fn update_downloads_badge(&self) {
        if let Some(badge) = self.imp().downloads_badge.get() {
            let active = downloads().active_count();
            badge.set_label(&active.to_string());
            badge.set_visible(active > 0);
        }
    }

    /// "Downloaded 12 files to …" with an Open Folder button.
    fn announce_finished_job(&self, id: u64) {
        let Some(job) = downloads().job(id) else {
            return;
        };
        let folder = job
            .dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let title = match (job.done(), job.failed()) {
            (done, 0) => format!(
                "Downloaded {done} {} to {folder}",
                if done == 1 { "file" } else { "files" }
            ),
            (_, failed) => format!(
                "{failed} {} couldn't be downloaded",
                if failed == 1 { "file" } else { "files" }
            ),
        };
        let toast = adw::Toast::builder()
            .title(title)
            .button_label("Open Folder")
            .action_name("win.open-folder")
            .action_target(&job.dir.to_string_lossy().to_variant())
            .build();
        self.add_toast(toast);
    }

    fn update_bookmark_badge(&self) {
        if let Some(badge) = self.imp().bookmarks_badge.get() {
            let count = state().data().bookmarks.len();
            badge.set_label(&count.to_string());
            badge.set_visible(count > 0);
        }
    }

    fn board_row_matches(&self, row: &gtk::ListBoxRow) -> bool {
        let needle = self
            .imp()
            .board_filter
            .text()
            .trim()
            .trim_matches('/')
            .to_lowercase();
        if needle.is_empty() {
            return true;
        }
        let boards = self.imp().boards.borrow();
        boards.get(row.index() as usize).is_some_and(|b| {
            b.code.to_lowercase().contains(&needle) || b.title.to_lowercase().contains(&needle)
        })
    }

    fn update_board_empty_state(&self) {
        let imp = self.imp();
        let mut any = false;
        let mut child = imp.board_list.first_child();
        while let Some(widget) = child {
            if let Some(row) = widget.downcast_ref::<gtk::ListBoxRow>() {
                any |= self.board_row_matches(row);
            }
            child = widget.next_sibling();
        }
        imp.boards_empty
            .set_visible(!any && !imp.boards.borrow().is_empty());
    }

    /// Switches the sidebar to a site, then opens `board` (or, without
    /// one, asks the user to pick).
    pub fn show_site(&self, site_id: &str, board: Option<&str>) {
        let imp = self.imp();
        if *imp.site_id.borrow() != site_id {
            imp.site_id.replace(site_id.to_owned());
            imp.board_filter.set_text("");
            if let Some(switcher) = imp.site_switcher.get() {
                switcher.set_current(site_id);
            }
            self.rebuild_board_list();
        }
        match board {
            Some(code) => self.open_board(site_id, code),
            None => self.show_pick_board(),
        }
    }

    fn rebuild_board_list(&self) {
        let imp = self.imp();
        let st = state();
        let site_id = imp.site_id.borrow().clone();
        let list = st.boards(&site_id);
        let visible: Vec<Board> = {
            let data = st.data();
            list.boards
                .into_iter()
                .filter(|b| !data.settings.is_board_hidden(&site_id, &b.code))
                .collect()
        };

        imp.board_list.remove_all();
        for board in &visible {
            imp.board_list.append(&board_row(board));
        }
        imp.boards.replace(visible);

        imp.boards_spinner.set_visible(list.loading);
        imp.board_count.set_visible(!list.loading);
        imp.board_count
            .set_label(&imp.boards.borrow().len().to_string());
        // Sites like Lainchan have no board index at all; their built-in
        // list is expected, so only mention a failure when there's no
        // built-in list to fall back on.
        let has_defaults = st
            .site(&site_id)
            .is_some_and(|s| !s.default_boards.is_empty());
        imp.boards_error
            .set_visible(list.error.is_some() && !has_defaults);
        if let Some(err) = &list.error {
            imp.boards_error.set_tooltip_text(Some(err));
        }
        self.sync_selection();
        self.update_board_empty_state();
    }

    /// Highlights the sidebar row for whatever the content pane shows.
    fn sync_selection(&self) {
        let imp = self.imp();
        let current = imp.current_board.borrow().clone();
        match current.as_deref() {
            Some(library @ ("__bookmarks" | "__downloads")) => {
                imp.board_list.unselect_all();
                let index = if library == "__bookmarks" { 0 } else { 1 };
                imp.library_list
                    .select_row(imp.library_list.row_at_index(index).as_ref());
            }
            Some(code) => {
                imp.library_list.unselect_all();
                let index = imp.boards.borrow().iter().position(|b| b.code == code);
                match index.and_then(|i| imp.board_list.row_at_index(i as i32)) {
                    Some(row) => imp.board_list.select_row(Some(&row)),
                    None => imp.board_list.unselect_all(),
                }
            }
            None => {
                imp.library_list.unselect_all();
                imp.board_list.unselect_all();
            }
        }
    }

    // ------------------------------------------------------------------
    // Navigation
    // ------------------------------------------------------------------

    /// Makes `page` the only page in the content pane.
    fn show_root(&self, page: &impl IsA<adw::NavigationPage>) {
        let imp = self.imp();
        imp.nav_view.replace(&[page.clone().upcast()]);
        // On narrow windows the sidebar overlays the content; get it
        // out of the way once something's been picked.
        if imp.split_view.is_collapsed() {
            imp.split_view.set_show_sidebar(false);
        }
    }

    pub fn open_board(&self, site_id: &str, board: &str) {
        let imp = self.imp();
        if *imp.site_id.borrow() != site_id {
            self.show_site(site_id, Some(board));
            return;
        }
        let Some(site) = state().site(site_id) else {
            return;
        };
        imp.current_board.replace(Some(board.to_owned()));
        self.sync_selection();
        self.show_root(&CatalogPage::new(self, site, board));
    }

    fn show_pick_board(&self) {
        let imp = self.imp();
        imp.current_board.replace(None);
        self.sync_selection();
        let status = adw::StatusPage::builder()
            .icon_name("view-grid-symbolic")
            .title("Choose a Board")
            .description("Pick a board from the sidebar to see its threads.")
            .build();
        self.show_root(&self.simple_page("Channex", &status));
    }

    pub fn open_bookmarks(&self) {
        self.imp().current_board.replace(Some("__bookmarks".into()));
        self.sync_selection();
        self.show_root(&bookmarks_page(self));
    }

    /// Shows files full-window, starting at `index`.
    pub fn open_media(
        &self,
        files: Vec<channex::data::model::PostFile>,
        index: usize,
        context: crate::viewer::ViewerContext,
    ) {
        if let Some(viewer) = self.imp().viewer.get() {
            viewer.open(files, index, context);
        }
    }

    #[cfg(debug_assertions)]
    pub fn dev_media_viewer(&self) -> crate::viewer::MediaViewer {
        self.imp().viewer.get().unwrap().clone()
    }

    /// Gives focus back to the page once the viewer closes.
    pub fn media_viewer_closed(&self) {
        if let Some(page) = self.imp().nav_view.visible_page() {
            page.child_focus(gtk::DirectionType::TabForward);
        }
    }

    /// The page showing in the content pane.
    pub fn visible_page(&self) -> Option<adw::NavigationPage> {
        self.imp().nav_view.visible_page()
    }

    pub fn open_downloads(&self) {
        self.imp().current_board.replace(Some("__downloads".into()));
        self.sync_selection();
        self.show_root(&downloads_page(self));
    }

    /// Opens a thread on top of the current page.
    pub fn open_thread(&self, site_id: &str, board: &str, thread_id: &str) {
        let Some(site) = state().site(site_id) else {
            return;
        };
        self.imp()
            .nav_view
            .push(&ThreadPage::new(self, site, board, thread_id));
    }

    /// A page with a sidebar toggle in its header and `child` below.
    pub fn simple_page(&self, title: &str, child: &impl IsA<gtk::Widget>) -> adw::NavigationPage {
        let header = adw::HeaderBar::new();
        header.pack_start(&self.sidebar_toggle());
        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(child));
        adw::NavigationPage::builder()
            .title(title)
            .child(&toolbar)
            .build()
    }

    // ------------------------------------------------------------------
    // State
    // ------------------------------------------------------------------

    fn restore_state(&self) {
        let saved = WindowState::load();
        self.set_default_size(saved.width, saved.height);
        if saved.maximized {
            self.maximize();
        }
        let site = if state().site(&saved.last_site).is_some() {
            saved.last_site
        } else {
            "4chan".into()
        };
        let board = (!saved.last_board.is_empty()).then_some(saved.last_board);
        self.show_site(&site, board.as_deref());
    }

    fn save_state(&self) {
        let imp = self.imp();
        // default_size() tracks the unmaximized size, which is what
        // should be restored next time.
        let (width, height) = self.default_size();
        let last_board = imp
            .current_board
            .borrow()
            .clone()
            .filter(|b| !b.starts_with("__"))
            .unwrap_or_default();
        let saved = WindowState {
            width,
            height,
            maximized: self.is_maximized(),
            last_site: imp.site_id.borrow().clone(),
            last_board,
        };
        if let Err(err) = saved.save() {
            glib::g_warning!("channex", "Couldn't save window state: {err}");
        }
    }
}

fn library_row(icon: &str, label: &str, suffix: Option<&gtk::Label>) -> gtk::ListBoxRow {
    let content = gtk::Box::builder().spacing(12).build();
    content.append(&gtk::Image::from_icon_name(icon));
    content.append(
        &gtk::Label::builder()
            .label(label)
            .xalign(0.0)
            .hexpand(true)
            .build(),
    );
    if let Some(suffix) = suffix {
        content.append(suffix);
    }
    gtk::ListBoxRow::builder().child(&content).build()
}

fn board_row(board: &Board) -> gtk::ListBoxRow {
    let content = gtk::Box::builder().spacing(8).build();
    content.append(
        &gtk::Label::builder()
            .label(format!("/{}/", board.code))
            .css_classes(["heading"])
            .build(),
    );
    content.append(
        &gtk::Label::builder()
            .label(&board.title)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(["dim-label"])
            .build(),
    );
    let row = gtk::ListBoxRow::builder()
        .child(&content)
        .tooltip_text(&board.description)
        .build();
    if board.description.is_empty() {
        row.set_has_tooltip(false);
    }

    // Right-click (or long-press) → "Hide from Sidebar".
    let menu = gio::Menu::new();
    menu.append(
        Some("_Hide from Sidebar"),
        Some(&format!(
            "win.hide-board('{}')",
            board.code.replace('\'', "\\'")
        )),
    );
    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    popover.set_parent(&row);
    popover.set_has_arrow(false);
    popover.set_halign(gtk::Align::Start);
    let click = gtk::GestureClick::builder()
        .button(gtk::gdk::BUTTON_SECONDARY)
        .build();
    click.connect_pressed(glib::clone!(
        #[weak]
        popover,
        move |_, _, x, y| {
            popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.popup();
        }
    ));
    row.add_controller(click);
    let long_press = gtk::GestureLongPress::new();
    long_press.connect_pressed(glib::clone!(
        #[weak]
        popover,
        move |_, x, y| {
            popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.popup();
        }
    ));
    row.add_controller(long_press);
    // A popover parented with set_parent() isn't freed with the row.
    row.connect_destroy(move |_| popover.unparent());
    row
}

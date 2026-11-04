//! A board's catalog: every thread as a card grid (or compact grid, or
//! list), with search, sort, and refresh in the header bar.

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use channex::data::adapters;
use channex::data::catalog::{SortMode, filter_and_sort};
use channex::data::model::{Post, Site};
use channex::data::sites::resolve_thread_url;
use channex::data::store::{Bookmark, CatalogViewMode};
use gtk::{gio, glib};

use crate::runtime;
use crate::state::state;
use crate::widgets::{CardOptions, PostRef, ThreadCard, ThreadRow};
use crate::window::ChannexWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct CatalogPage {
        pub window: glib::WeakRef<ChannexWindow>,
        pub site: OnceCell<Site>,
        pub board: OnceCell<String>,
        /// Every thread, in the site's (bump) order.
        pub posts: RefCell<Vec<PostRef>>,
        /// What the views show: filtered and sorted `posts`.
        pub model: OnceCell<gio::ListStore>,
        pub sort: Cell<SortMode>,
        /// Threads whose blurred thumbnail was clicked.
        pub revealed: RefCell<HashSet<String>>,
        pub loading: Cell<bool>,

        pub title: OnceCell<adw::WindowTitle>,
        pub stack: OnceCell<gtk::Stack>,
        pub error_page: OnceCell<adw::StatusPage>,
        pub search_bar: OnceCell<gtk::SearchBar>,
        pub search_entry: OnceCell<gtk::SearchEntry>,
        pub view_toggle: OnceCell<adw::ToggleGroup>,
        pub refresh: OnceCell<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CatalogPage {
        const NAME: &'static str = "ChannexCatalogPage";
        type Type = super::CatalogPage;
        type ParentType = adw::NavigationPage;
    }

    impl ObjectImpl for CatalogPage {}
    impl WidgetImpl for CatalogPage {}
    impl NavigationPageImpl for CatalogPage {}
}

glib::wrapper! {
    pub struct CatalogPage(ObjectSubclass<imp::CatalogPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CatalogPage {
    pub fn new(window: &ChannexWindow, site: Site, board: &str) -> Self {
        let page: Self = glib::Object::new();
        let imp = page.imp();
        imp.window.set(Some(window));
        let _ = imp.site.set(site);
        let _ = imp.board.set(board.to_owned());
        let _ = imp.model.set(gio::ListStore::new::<glib::BoxedAnyObject>());
        page.build(window);
        page.reload();
        page
    }

    fn site(&self) -> &Site {
        self.imp().site.get().unwrap()
    }

    fn board(&self) -> &str {
        self.imp().board.get().unwrap()
    }

    fn model(&self) -> &gio::ListStore {
        self.imp().model.get().unwrap()
    }

    fn board_title(&self) -> String {
        let code = self.board();
        state()
            .boards(&self.site().id)
            .boards
            .into_iter()
            .find(|b| b.code == code)
            .map(|b| b.title)
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| format!("/{code}/"))
    }

    // ------------------------------------------------------------------
    // Layout
    // ------------------------------------------------------------------

    fn build(&self, window: &ChannexWindow) {
        let imp = self.imp();
        self.set_title(&self.board_title());
        self.set_tag(Some("catalog"));

        // Header bar.
        let title = adw::WindowTitle::new(&self.board_title(), "");
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(&window.sidebar_toggle());

        let search_toggle = gtk::ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search (Ctrl+F)")
            .build();
        header.pack_start(&search_toggle);

        let refresh_button = gtk::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Refresh (F5)")
            .action_name("win.refresh")
            .build();
        let refresh = gtk::Stack::new();
        refresh.add_named(&refresh_button, Some("button"));
        refresh.add_named(&adw::Spinner::new(), Some("spinner"));
        header.pack_end(&refresh);

        let view_toggle = adw::ToggleGroup::new();
        for (name, icon, tip) in [
            ("grid", "view-grid-symbolic", "Grid"),
            ("compact", "view-app-grid-symbolic", "Compact Grid"),
            ("list", "view-list-symbolic", "List"),
        ] {
            view_toggle.add(
                adw::Toggle::builder()
                    .name(name)
                    .icon_name(icon)
                    .tooltip(tip)
                    .build(),
            );
        }
        view_toggle.set_active_name(Some(view_mode_name(
            state().data().settings.catalog_view_mode,
        )));
        view_toggle.connect_active_name_notify(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |group| {
                let mode = match group.active_name().as_deref() {
                    Some("compact") => CatalogViewMode::Compact,
                    Some("list") => CatalogViewMode::List,
                    _ => CatalogViewMode::Grid,
                };
                if state().data().settings.catalog_view_mode != mode {
                    state().update_settings(|d| d.settings.catalog_view_mode = mode);
                }
                page.show_results();
            }
        ));
        header.pack_end(&view_toggle);

        let sort_menu = gio::Menu::new();
        for mode in SortMode::ALL {
            sort_menu.append(
                Some(mode.label()),
                Some(&format!("catalog.sort('{}')", sort_mode_name(mode))),
            );
        }
        let sort_section = gio::Menu::new();
        sort_section.append_section(Some("Sort By"), &sort_menu);
        header.pack_end(
            &gtk::MenuButton::builder()
                .icon_name("view-sort-descending-symbolic")
                .tooltip_text("Sort")
                .menu_model(&sort_section)
                .build(),
        );
        self.setup_actions();

        // Search bar.
        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text("Search threads, or paste a thread link")
            .hexpand(true)
            .build();
        let clamp = adw::Clamp::builder()
            .maximum_size(560)
            .child(&search_entry)
            .build();
        let search_bar = gtk::SearchBar::builder().child(&clamp).build();
        search_bar.connect_entry(&search_entry);
        search_bar.set_key_capture_widget(Some(self));
        search_bar
            .bind_property("search-mode-enabled", &search_toggle, "active")
            .bidirectional()
            .sync_create()
            .build();
        search_entry.connect_search_changed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.apply_filter()
        ));
        search_entry.connect_activate(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |entry| page.open_pasted_link(&entry.text())
        ));

        // Content.
        let stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::Crossfade)
            .build();
        stack.add_named(
            &adw::Spinner::builder()
                .width_request(32)
                .height_request(32)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build(),
            Some("loading"),
        );

        let retry = gtk::Button::builder()
            .label("Try Again")
            .halign(gtk::Align::Center)
            .css_classes(["pill", "suggested-action"])
            .build();
        retry.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.reload()
        ));
        let error_page = adw::StatusPage::builder()
            .icon_name("network-error-symbolic")
            .title("Couldn't Load This Board")
            .child(&retry)
            .build();
        stack.add_named(&error_page, Some("error"));

        stack.add_named(
            &adw::StatusPage::builder()
                .icon_name("system-search-symbolic")
                .title("No Results Found")
                .description("Try a different search.")
                .build(),
            Some("empty"),
        );
        stack.add_named(&self.build_grid(), Some("grid"));
        stack.add_named(&self.build_list(), Some("list"));

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.add_top_bar(&search_bar);
        toolbar.set_content(Some(&stack));
        self.set_child(Some(&toolbar));

        let _ = imp.title.set(title);
        let _ = imp.stack.set(stack);
        let _ = imp.error_page.set(error_page);
        let _ = imp.search_bar.set(search_bar);
        let _ = imp.search_entry.set(search_entry);
        let _ = imp.view_toggle.set(view_toggle);
        let _ = imp.refresh.set(refresh);

        // Bookmarks and the blur setting change what cards show.
        let st = state();
        st.connect_closure(
            "bookmarks-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = page)]
                self,
                move |_: crate::state::State| page.rebind_all()
            ),
        );
        st.connect_closure(
            "settings-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = page)]
                self,
                move |_: crate::state::State| {
                    let mode = view_mode_name(state().data().settings.catalog_view_mode);
                    let toggle = page.imp().view_toggle.get().unwrap();
                    if toggle.active_name().as_deref() != Some(mode) {
                        toggle.set_active_name(Some(mode));
                    }
                    page.rebind_all();
                }
            ),
        );
        st.connect_closure(
            "boards-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = page)]
                self,
                move |_: crate::state::State, _site: String| page.update_titles()
            ),
        );
    }

    fn setup_actions(&self) {
        let group = gio::SimpleActionGroup::new();
        let sort = gio::SimpleAction::new_stateful(
            "sort",
            Some(glib::VariantTy::STRING),
            &"bump".to_variant(),
        );
        sort.connect_activate(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |action, param| {
                let Some(name) = param.and_then(|p| p.get::<String>()) else {
                    return;
                };
                action.set_state(&name.to_variant());
                page.imp().sort.set(sort_mode_from_name(&name));
                page.apply_filter();
            }
        ));
        group.add_action(&sort);
        self.insert_action_group("catalog", Some(&group));
    }

    fn build_grid(&self) -> gtk::ScrolledWindow {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, item| {
                let card = ThreadCard::new();
                card.connect_bookmark_clicked(glib::clone!(
                    #[weak]
                    page,
                    move |post| page.toggle_bookmark(post)
                ));
                item.downcast_ref::<gtk::ListItem>()
                    .unwrap()
                    .set_child(Some(&card));
            }
        ));
        factory.connect_bind(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let (Some(card), Some(post)) =
                    (item.child().and_downcast::<ThreadCard>(), post_of(item))
                else {
                    return;
                };
                card.bind(&post, page.card_options(&post));
            }
        ));
        factory.connect_unbind(|_, item| {
            if let Some(card) = item
                .downcast_ref::<gtk::ListItem>()
                .unwrap()
                .child()
                .and_downcast::<ThreadCard>()
            {
                card.unbind();
            }
        });

        let grid = gtk::GridView::builder()
            .model(&gtk::NoSelection::new(Some(self.model().clone())))
            .factory(&factory)
            .single_click_activate(true)
            .max_columns(12)
            .css_classes(["catalog"])
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        grid.connect_activate(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, position| page.activate(position)
        ));
        gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&grid)
            .build()
    }

    fn build_list(&self) -> gtk::ScrolledWindow {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            item.downcast_ref::<gtk::ListItem>()
                .unwrap()
                .set_child(Some(&ThreadRow::new()));
        });
        factory.connect_bind(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let (Some(row), Some(post)) =
                    (item.child().and_downcast::<ThreadRow>(), post_of(item))
                else {
                    return;
                };
                row.bind(&post, page.card_options(&post));
            }
        ));
        factory.connect_unbind(|_, item| {
            if let Some(row) = item
                .downcast_ref::<gtk::ListItem>()
                .unwrap()
                .child()
                .and_downcast::<ThreadRow>()
            {
                row.unbind();
            }
        });

        let list = gtk::ListView::builder()
            .model(&gtk::NoSelection::new(Some(self.model().clone())))
            .factory(&factory)
            .single_click_activate(true)
            .css_classes(["catalog"])
            .margin_top(8)
            .margin_bottom(8)
            .build();
        list.connect_activate(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, position| page.activate(position)
        ));
        let clamp = adw::ClampScrollable::builder()
            .maximum_size(900)
            .child(&list)
            .build();
        gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&clamp)
            .build()
    }

    // ------------------------------------------------------------------
    // Data
    // ------------------------------------------------------------------

    /// Fetches the catalog. Existing threads stay on screen while a
    /// refresh runs; a failed refresh is reported as a toast.
    pub fn reload(&self) {
        let imp = self.imp();
        if imp.loading.replace(true) {
            return;
        }
        let first_load = imp.posts.borrow().is_empty();
        if first_load {
            imp.stack.get().unwrap().set_visible_child_name("loading");
        }
        imp.refresh.get().unwrap().set_visible_child_name("spinner");

        let client = state().http();
        let site = self.site().clone();
        let board = self.board().to_owned();
        let page = self.downgrade();
        glib::spawn_future_local(async move {
            let result =
                runtime::spawn(
                    async move { adapters::fetch_catalog(&client, &site, &board).await },
                )
                .await;
            let Some(page) = page.upgrade() else { return };
            let imp = page.imp();
            imp.loading.set(false);
            imp.refresh.get().unwrap().set_visible_child_name("button");
            match result {
                Ok(posts) => {
                    imp.posts.replace(posts.into_iter().map(Rc::new).collect());
                    page.apply_filter();
                }
                Err(err) if first_load => {
                    imp.error_page
                        .get()
                        .unwrap()
                        .set_description(Some(&err.to_string()));
                    imp.stack.get().unwrap().set_visible_child_name("error");
                }
                Err(err) => {
                    if let Some(window) = imp.window.upgrade() {
                        window.toast(&format!("Couldn't refresh: {err}"));
                    }
                }
            }
            page.update_titles();
        });
    }

    /// Rebuilds the shown list from `posts`, the search text, and sort.
    fn apply_filter(&self) {
        let imp = self.imp();
        let posts = imp.posts.borrow();
        let query = imp
            .search_entry
            .get()
            .map(|e| e.text().to_string())
            .unwrap_or_default();
        // A pasted link isn't a search term.
        let query = if query.trim_start().starts_with("http") {
            String::new()
        } else {
            query
        };
        let indices = filter_and_sort(&posts, &query, imp.sort.get());
        let items: Vec<glib::BoxedAnyObject> = indices
            .into_iter()
            .map(|i| glib::BoxedAnyObject::new(posts[i].clone()))
            .collect();
        drop(posts);
        let model = self.model();
        model.splice(0, model.n_items(), &items);
        self.show_results();
        self.update_titles();
    }

    fn show_results(&self) {
        let imp = self.imp();
        if imp.posts.borrow().is_empty() {
            return; // still loading, or the error page is showing
        }
        let stack = imp.stack.get().unwrap();
        if self.model().n_items() == 0 {
            stack.set_visible_child_name("empty");
            return;
        }
        let mode = state().data().settings.catalog_view_mode;
        stack.set_visible_child_name(if mode == CatalogViewMode::List {
            "list"
        } else {
            "grid"
        });
        self.rebind_all();
    }

    /// Re-renders every visible cell (after a setting or bookmark change).
    fn rebind_all(&self) {
        let model = self.model();
        let n = model.n_items();
        model.items_changed(0, n, n);
    }

    fn update_titles(&self) {
        let imp = self.imp();
        let title = self.board_title();
        self.set_title(&title);
        let total = imp.posts.borrow().len();
        let shown = self.model().n_items() as usize;
        let mut subtitle = format!("{} · /{}/", self.site().name, self.board());
        if total > 0 {
            if shown == total {
                subtitle.push_str(&format!(" · {total} threads"));
            } else {
                subtitle.push_str(&format!(" · {shown} of {total} threads"));
            }
        }
        let header_title = imp.title.get().unwrap();
        header_title.set_title(&title);
        header_title.set_subtitle(&subtitle);
    }

    fn card_options(&self, post: &Post) -> CardOptions {
        let st = state();
        let data = st.data();
        let spoiler = post.files.first().is_some_and(|f| f.spoiler);
        CardOptions {
            compact: data.settings.catalog_view_mode == CatalogViewMode::Compact,
            blurred: (self.site().nsfw || spoiler)
                && data.settings.blur_nsfw
                && !post.files.is_empty()
                && !self.imp().revealed.borrow().contains(&post.id),
            bookmarked: data.is_bookmarked(&self.site().id, self.board(), &post.thread_id),
        }
    }

    // ------------------------------------------------------------------
    // User actions
    // ------------------------------------------------------------------

    /// Clicking a thread: reveal its blurred thumbnail first, otherwise
    /// open it.
    fn activate(&self, position: u32) {
        let Some(post) = self
            .model()
            .item(position)
            .and_downcast::<glib::BoxedAnyObject>()
            .map(|o| o.borrow::<PostRef>().clone())
        else {
            return;
        };
        if self.card_options(&post).blurred {
            self.imp().revealed.borrow_mut().insert(post.id.clone());
            self.model().items_changed(position, 1, 1);
            return;
        }
        if let Some(window) = self.imp().window.upgrade() {
            window.open_thread(&self.site().id, self.board(), &post.thread_id);
        }
    }

    fn toggle_bookmark(&self, post: &PostRef) {
        let bookmark = Bookmark {
            site_id: self.site().id.clone(),
            board_code: self.board().to_owned(),
            thread_id: post.thread_id.clone(),
            subject: post.title(),
            thumb_url: post
                .files
                .first()
                .map(|f| f.thumb_url.clone())
                .unwrap_or_default(),
            reply_count: post.reply_count,
            ..Default::default()
        };
        let now = glib::real_time() / 1000;
        let added = state().update_bookmarks(|d| d.toggle_bookmark(bookmark, now));
        if let Some(window) = self.imp().window.upgrade() {
            window.toast(if added {
                "Bookmarked"
            } else {
                "Bookmark removed"
            });
        }
    }

    pub fn start_search(&self) {
        let imp = self.imp();
        imp.search_bar.get().unwrap().set_search_mode(true);
        imp.search_entry.get().unwrap().grab_focus();
    }

    /// Enter in the search field: if it's a thread link for a known
    /// site, open that thread.
    fn open_pasted_link(&self, text: &str) {
        let Some(target) = resolve_thread_url(text, &state().sites()) else {
            return;
        };
        let imp = self.imp();
        if let Some(window) = imp.window.upgrade() {
            imp.search_entry.get().unwrap().set_text("");
            imp.search_bar.get().unwrap().set_search_mode(false);
            window.open_thread(&target.site_id, &target.board, &target.thread_id);
        }
    }
}

fn post_of(item: &gtk::ListItem) -> Option<PostRef> {
    item.item()
        .and_downcast::<glib::BoxedAnyObject>()
        .map(|o| o.borrow::<PostRef>().clone())
}

fn view_mode_name(mode: CatalogViewMode) -> &'static str {
    match mode {
        CatalogViewMode::Grid => "grid",
        CatalogViewMode::Compact => "compact",
        CatalogViewMode::List => "list",
    }
}

fn sort_mode_name(mode: SortMode) -> &'static str {
    match mode {
        SortMode::Bump => "bump",
        SortMode::Replies => "replies",
        SortMode::Images => "images",
        SortMode::Oldest => "oldest",
    }
}

fn sort_mode_from_name(name: &str) -> SortMode {
    SortMode::ALL
        .into_iter()
        .find(|m| sort_mode_name(*m) == name)
        .unwrap_or_default()
}

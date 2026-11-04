//! A thread: every post in a width-clamped list. Quote links scroll to
//! (and briefly highlight) the quoted post, and hovering one previews it.
//! Header actions: bookmark, open in browser, refresh.

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use channex::data::adapters::{self, thread_web_url};
use channex::data::comment::Palette;
use channex::data::downloads::thread_folder_name;
use channex::data::model::{PostFile, Site, Thread};
use channex::data::store::Bookmark;
use gtk::{gdk, gio, glib};

use crate::runtime;
use crate::state::state;
use crate::theme;
use crate::widgets::{PostContext, PostRef, PostWidget};
use crate::window::ChannexWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ThreadPage {
        pub window: glib::WeakRef<ChannexWindow>,
        pub site: OnceCell<Rc<Site>>,
        pub board: OnceCell<String>,
        pub thread_id: OnceCell<String>,

        pub thread: RefCell<Option<Thread>>,
        pub posts: RefCell<Vec<PostRef>>,
        pub backlinks: RefCell<HashMap<String, Vec<String>>>,
        pub model: OnceCell<gio::ListStore>,
        pub palette: RefCell<Option<Rc<Palette>>>,
        pub highlighted: RefCell<Option<String>>,
        /// Unsent reply text.
        pub draft: Rc<RefCell<String>>,
        /// A post to scroll to once the next load finishes (a new reply).
        pub pending_jump: RefCell<Option<String>>,
        pub loading: Cell<bool>,

        pub title: OnceCell<adw::WindowTitle>,
        pub stack: OnceCell<gtk::Stack>,
        pub error_page: OnceCell<adw::StatusPage>,
        pub list: OnceCell<gtk::ListView>,
        pub bookmark: OnceCell<gtk::Button>,
        pub download: OnceCell<gtk::Button>,
        pub refresh: OnceCell<gtk::Stack>,
        pub preview: OnceCell<gtk::Popover>,
        pub preview_post: OnceCell<PostWidget>,
        pub preview_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThreadPage {
        const NAME: &'static str = "ChannexThreadPage";
        type Type = super::ThreadPage;
        type ParentType = adw::NavigationPage;
    }

    impl ObjectImpl for ThreadPage {
        fn dispose(&self) {
            if let Some(preview) = self.preview.get() {
                preview.unparent();
            }
        }
    }

    impl WidgetImpl for ThreadPage {}
    impl NavigationPageImpl for ThreadPage {}
}

glib::wrapper! {
    pub struct ThreadPage(ObjectSubclass<imp::ThreadPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ThreadPage {
    pub fn new(window: &ChannexWindow, site: Site, board: &str, thread_id: &str) -> Self {
        let page: Self = glib::Object::builder()
            .property("title", format!("Thread {thread_id}"))
            .property("tag", "thread")
            .build();
        let imp = page.imp();
        imp.window.set(Some(window));
        let _ = imp.site.set(Rc::new(site));
        let _ = imp.board.set(board.to_owned());
        let _ = imp.thread_id.set(thread_id.to_owned());
        let _ = imp.model.set(gio::ListStore::new::<glib::BoxedAnyObject>());
        page.build(window);
        page.reload();
        page
    }

    fn site(&self) -> &Rc<Site> {
        self.imp().site.get().unwrap()
    }

    fn board(&self) -> &str {
        self.imp().board.get().unwrap()
    }

    fn thread_id(&self) -> &str {
        self.imp().thread_id.get().unwrap()
    }

    fn model(&self) -> &gio::ListStore {
        self.imp().model.get().unwrap()
    }

    fn web_url(&self) -> String {
        thread_web_url(self.site(), self.board(), self.thread_id())
    }

    // ------------------------------------------------------------------
    // Layout
    // ------------------------------------------------------------------

    fn build(&self, window: &ChannexWindow) {
        let imp = self.imp();

        let title =
            adw::WindowTitle::new(&format!("Thread {}", self.thread_id()), &self.subtitle());
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(&window.sidebar_toggle());

        let refresh_button = gtk::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Refresh (F5)")
            .action_name("win.refresh")
            .build();
        let refresh = gtk::Stack::new();
        refresh.add_named(&refresh_button, Some("button"));
        refresh.add_named(&adw::Spinner::new(), Some("spinner"));
        header.pack_end(&refresh);

        let open = gtk::Button::builder()
            .icon_name("web-browser-symbolic")
            .tooltip_text("Open in Browser")
            .build();
        open.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.open_in_browser()
        ));
        header.pack_end(&open);

        let download = gtk::Button::builder()
            .icon_name("folder-download-symbolic")
            .tooltip_text("Download Files…")
            .sensitive(false)
            .build();
        download.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.download_files(None)
        ));
        header.pack_end(&download);

        let reply = gtk::Button::builder()
            .icon_name("mail-reply-sender-symbolic")
            .tooltip_text("Reply…")
            .build();
        reply.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.reply(None)
        ));
        header.pack_start(&reply);

        let bookmark = gtk::Button::builder().action_name("win.bookmark").build();
        header.pack_end(&bookmark);

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
            .title("Couldn't Load This Thread")
            .child(&retry)
            .build();
        stack.add_named(&error_page, Some("error"));

        let list = self.build_list();
        let clamp = adw::ClampScrollable::builder()
            .maximum_size(960)
            .child(&list)
            .build();
        stack.add_named(
            &gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .child(&clamp)
                .build(),
            Some("posts"),
        );

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&stack));
        self.set_child(Some(&toolbar));

        // Quote preview: a read-only post in a popover.
        let preview_post = PostWidget::new();
        preview_post.set_width_request(420);
        let preview_scroll = gtk::ScrolledWindow::builder()
            .child(&preview_post)
            .propagate_natural_height(true)
            .max_content_height(420)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();
        let preview = gtk::Popover::builder()
            .child(&preview_scroll)
            .autohide(false)
            .can_focus(false)
            .can_target(false)
            .build();
        preview.add_css_class("quote-preview");
        preview.set_parent(self);

        let _ = imp.title.set(title);
        let _ = imp.stack.set(stack);
        let _ = imp.error_page.set(error_page);
        let _ = imp.list.set(list);
        let _ = imp.bookmark.set(bookmark);
        let _ = imp.download.set(download);
        let _ = imp.refresh.set(refresh);
        let _ = imp.preview.set(preview);
        let _ = imp.preview_post.set(preview_post);
        self.update_bookmark_button();

        let st = state();
        st.connect_closure(
            "bookmarks-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = page)]
                self,
                move |_: crate::state::State| page.update_bookmark_button()
            ),
        );
        // Reply style and theme changes re-render the posts.
        st.connect_closure(
            "settings-changed",
            false,
            glib::closure_local!(
                #[watch(rename_to = page)]
                self,
                move |_: crate::state::State| page.restyle()
            ),
        );
        adw::StyleManager::default().connect_dark_notify(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.restyle()
        ));
    }

    fn build_list(&self) -> gtk::ListView {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, item| {
                let post = PostWidget::new();
                post.set_margin_top(5);
                post.set_margin_bottom(5);
                post.set_margin_start(12);
                post.set_margin_end(12);
                post.connect_quote_clicked(glib::clone!(
                    #[weak]
                    page,
                    move |id| page.jump_to(id)
                ));
                post.connect_quote_hovered(glib::clone!(
                    #[weak]
                    page,
                    move |target| page.preview(target)
                ));
                post.connect_file_clicked(glib::clone!(
                    #[weak]
                    page,
                    move |file| page.open_file(file)
                ));
                post.connect_reply_clicked(glib::clone!(
                    #[weak]
                    page,
                    move |post| page.reply(Some(format!(">>{}", post.id)))
                ));
                post.connect_download_clicked(glib::clone!(
                    #[weak]
                    page,
                    move |post| page.download_files(Some(post))
                ));
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                item.set_activatable(false);
                item.set_focusable(false);
                item.set_child(Some(&post));
            }
        ));
        factory.connect_bind(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(widget) = item.child().and_downcast::<PostWidget>() else {
                    return;
                };
                let Some(post) = item
                    .item()
                    .and_downcast::<glib::BoxedAnyObject>()
                    .map(|o| o.borrow::<PostRef>().clone())
                else {
                    return;
                };
                widget.bind(&post, &page.context_for(&post.id, true));
            }
        ));

        gtk::ListView::builder()
            .model(&gtk::NoSelection::new(Some(self.model().clone())))
            .factory(&factory)
            .css_classes(["thread"])
            .margin_top(8)
            .margin_bottom(24)
            .build()
    }

    fn context_for(&self, post_id: &str, interactive: bool) -> PostContext {
        let imp = self.imp();
        let palette = imp
            .palette
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(theme::comment_palette()))
            .clone();
        PostContext {
            site: self.site().clone(),
            board: self.board().to_owned(),
            backlinks: imp
                .backlinks
                .borrow()
                .get(post_id)
                .cloned()
                .unwrap_or_default(),
            style: state().data().settings.reply_display_style,
            highlighted: imp.highlighted.borrow().as_deref() == Some(post_id),
            palette,
            interactive,
        }
    }

    fn subtitle(&self) -> String {
        let mut subtitle = format!("{} · /{}/", self.site().name, self.board());
        if let Some(thread) = self.imp().thread.borrow().as_ref() {
            let posts = thread.post_count();
            let files = thread.all_files().len();
            subtitle.push_str(&format!(
                " · {posts} {} · {files} {}",
                plural(posts, "post", "posts"),
                plural(files, "file", "files")
            ));
        }
        subtitle
    }

    fn restyle(&self) {
        self.imp().palette.replace(None);
        let model = self.model();
        let n = model.n_items();
        model.items_changed(0, n, n);
    }

    fn update_bookmark_button(&self) {
        let Some(button) = self.imp().bookmark.get() else {
            return;
        };
        let bookmarked =
            state()
                .data()
                .is_bookmarked(&self.site().id, self.board(), self.thread_id());
        button.set_icon_name(if bookmarked {
            "user-bookmarks-symbolic"
        } else {
            "bookmark-new-symbolic"
        });
        button.set_tooltip_text(Some(if bookmarked {
            "Remove Bookmark (Ctrl+D)"
        } else {
            "Bookmark Thread (Ctrl+D)"
        }));
    }

    // ------------------------------------------------------------------
    // Data
    // ------------------------------------------------------------------

    pub fn reload(&self) {
        let imp = self.imp();
        if imp.loading.replace(true) {
            return;
        }
        let first_load = imp.thread.borrow().is_none();
        if first_load {
            imp.stack.get().unwrap().set_visible_child_name("loading");
        }
        imp.refresh.get().unwrap().set_visible_child_name("spinner");

        let client = state().http();
        let site = (**self.site()).clone();
        let (board, thread_id) = (self.board().to_owned(), self.thread_id().to_owned());
        let page = self.downgrade();
        glib::spawn_future_local(async move {
            let result = runtime::spawn(async move {
                adapters::fetch_thread(&client, &site, &board, &thread_id).await
            })
            .await;
            let Some(page) = page.upgrade() else { return };
            let imp = page.imp();
            imp.loading.set(false);
            imp.refresh.get().unwrap().set_visible_child_name("button");
            match result {
                Ok(thread) => page.show_thread(thread),
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
        });
    }

    fn show_thread(&self, thread: Thread) {
        let imp = self.imp();
        let posts: Vec<PostRef> = thread.posts().cloned().map(Rc::new).collect();
        imp.backlinks.replace(thread.backlinks());
        let title = thread.op.title();
        let replies = thread.replies.len() as u32;
        imp.thread.replace(Some(thread));

        let items: Vec<glib::BoxedAnyObject> = posts
            .iter()
            .cloned()
            .map(glib::BoxedAnyObject::new)
            .collect();
        imp.posts.replace(posts);
        let model = self.model();
        model.splice(0, model.n_items(), &items);
        imp.stack.get().unwrap().set_visible_child_name("posts");
        let has_files = imp.posts.borrow().iter().any(|p| !p.files.is_empty());
        imp.download.get().unwrap().set_sensitive(has_files);
        if let Some(id) = imp.pending_jump.take() {
            self.jump_to(&id);
        }

        self.set_title(&title);
        let header = imp.title.get().unwrap();
        header.set_title(&title);
        header.set_subtitle(&self.subtitle());

        // Opening a bookmarked thread counts its replies as seen.
        let (site, board, id) = (
            self.site().id.clone(),
            self.board().to_owned(),
            self.thread_id().to_owned(),
        );
        let seen_changed = state()
            .data()
            .bookmark(&site, &board, &id)
            .is_some_and(|b| b.last_seen_reply_count != replies || b.reply_count != replies);
        if seen_changed {
            state().update_bookmarks(|d| d.mark_seen(&site, &board, &id, replies));
        }
    }

    // ------------------------------------------------------------------
    // User actions
    // ------------------------------------------------------------------

    /// Scrolls to a quoted post and highlights it for a moment.
    fn jump_to(&self, post_id: &str) {
        let imp = self.imp();
        self.preview(None);
        let Some(index) = imp.posts.borrow().iter().position(|p| p.id == post_id) else {
            if let Some(window) = imp.window.upgrade() {
                window.toast(&format!("Post {post_id} isn't in this thread"));
            }
            return;
        };
        let index = index as u32;
        imp.list
            .get()
            .unwrap()
            .scroll_to(index, gtk::ListScrollFlags::NONE, None);
        imp.highlighted.replace(Some(post_id.to_owned()));
        self.model().items_changed(index, 1, 1);

        let id = post_id.to_owned();
        glib::timeout_add_local_once(
            Duration::from_millis(1600),
            glib::clone!(
                #[weak(rename_to = page)]
                self,
                move || {
                    let imp = page.imp();
                    if imp.highlighted.borrow().as_deref() == Some(id.as_str()) {
                        imp.highlighted.replace(None);
                        if let Some(i) = imp.posts.borrow().iter().position(|p| p.id == id) {
                            page.model().items_changed(i as u32, 1, 1);
                        }
                    }
                }
            ),
        );
    }

    /// Shows (or hides, with `None`) the preview for a hovered quote.
    fn preview(&self, target: Option<(String, gtk::Label, f64, f64)>) {
        let imp = self.imp();
        let popover = imp.preview.get().unwrap();
        let Some((id, label, x, y)) = target else {
            imp.preview_id.replace(None);
            popover.popdown();
            return;
        };
        if imp.preview_id.borrow().as_deref() == Some(id.as_str()) && popover.is_visible() {
            return;
        }
        let Some(post) = imp.posts.borrow().iter().find(|p| p.id == id).cloned() else {
            return;
        };
        imp.preview_post
            .get()
            .unwrap()
            .bind(&post, &self.context_for(&post.id, false));
        imp.preview_id.replace(Some(id));

        // Point at the hovered spot, in this page's coordinates.
        let Some(point) = label.compute_point(self, &gtk::graphene::Point::new(x as f32, y as f32))
        else {
            return;
        };
        popover.set_pointing_to(Some(&gdk::Rectangle::new(
            point.x() as i32,
            point.y() as i32,
            1,
            1,
        )));
        popover.popup();
    }

    /// "Download Files": every file in the thread, or just `post`'s.
    /// Opens the reply dialog, optionally quoting a post.
    pub fn reply(&self, quote: Option<String>) {
        let imp = self.imp();
        let Some(window) = imp.window.upgrade() else {
            return;
        };
        let page = self.downgrade();
        let target = crate::dialogs::ReplyTarget {
            site: self.site().clone(),
            board: self.board().to_owned(),
            thread_id: self.thread_id().to_owned(),
            draft: imp.draft.clone(),
            on_posted: Box::new(move |id| {
                let Some(page) = page.upgrade() else {
                    return;
                };
                if let Some(window) = page.imp().window.upgrade() {
                    window.toast("Reply posted");
                }
                page.imp()
                    .pending_jump
                    .replace((!id.is_empty()).then_some(id));
                page.reload();
            }),
        };
        crate::dialogs::show_reply(&window, target, quote);
    }

    pub fn download_files(&self, post: Option<&PostRef>) {
        let imp = self.imp();
        let Some(window) = imp.window.upgrade() else {
            return;
        };
        let files: Vec<PostFile> = {
            let thread = imp.thread.borrow();
            let Some(thread) = thread.as_ref() else {
                return;
            };
            match post {
                Some(p) => p.files.clone(),
                None => thread.all_files().into_iter().cloned().collect(),
            }
        };
        let Some(folder) = self.download_folder() else {
            return;
        };
        if files.is_empty() {
            return;
        }
        let dir = crate::downloads::base_dir().join(&folder);
        crate::dialogs::show_download_files(&window, &folder, dir, files);
    }

    /// Opens the thread's files in the media viewer, at `file`.
    /// Opens the viewer at the thread's `index`th file.
    pub fn open_media_at(&self, index: usize) {
        let file = self.imp().thread.borrow().as_ref().and_then(|t| t.all_files().get(index).map(|f| (*f).clone()));
        if let Some(file) = file {
            self.open_file(&file);
        }
    }

    fn open_file(&self, file: &PostFile) {
        let imp = self.imp();
        let Some(window) = imp.window.upgrade() else {
            return;
        };
        let Some(folder) = self.download_folder() else {
            return;
        };
        let files: Vec<PostFile> = match imp.thread.borrow().as_ref() {
            Some(thread) => thread.all_files().into_iter().cloned().collect(),
            None => return,
        };
        let index = files.iter().position(|f| f.url == file.url).unwrap_or(0);
        let context = crate::viewer::ViewerContext {
            save_dir: crate::downloads::base_dir().join(&folder),
            save_label: folder,
        };
        window.open_media(files, index, context);
    }

    /// This thread's download folder name, per the folder-naming setting.
    fn download_folder(&self) -> Option<String> {
        let thread = self.imp().thread.borrow();
        let thread = thread.as_ref()?;
        let title = if thread.op.subject.is_empty() {
            thread.op.comment.excerpt(80)
        } else {
            thread.op.subject.clone()
        };
        let naming = state().data().settings.download_folder_naming;
        Some(thread_folder_name(naming, &self.site().id, self.board(), self.thread_id(), &title))
    }

    fn open_in_browser(&self) {
        let window = self.imp().window.upgrade();
        gtk::UriLauncher::new(&self.web_url()).launch(
            window.as_ref(),
            gio::Cancellable::NONE,
            |_| {},
        );
    }

    pub fn toggle_bookmark(&self) {
        let imp = self.imp();
        let thread = imp.thread.borrow();
        let bookmark = Bookmark {
            site_id: self.site().id.clone(),
            board_code: self.board().to_owned(),
            thread_id: self.thread_id().to_owned(),
            subject: thread
                .as_ref()
                .map(|t| t.op.title())
                .unwrap_or_else(|| format!("Thread {}", self.thread_id())),
            thumb_url: thread
                .as_ref()
                .and_then(|t| t.op.files.first())
                .map(|f| f.thumb_url.clone())
                .unwrap_or_default(),
            reply_count: thread.as_ref().map(|t| t.replies.len() as u32).unwrap_or(0),
            ..Default::default()
        };
        drop(thread);
        let added =
            state().update_bookmarks(|d| d.toggle_bookmark(bookmark, glib::real_time() / 1000));
        if let Some(window) = imp.window.upgrade() {
            window.toast(if added {
                "Bookmarked"
            } else {
                "Bookmark removed"
            });
        }
    }
}

fn plural<'a>(n: usize, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 { one } else { many }
}

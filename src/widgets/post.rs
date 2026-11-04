//! One post in a thread: a card with the poster line, subject (OP),
//! files, the comment, and "Replies:" backlinks. Quote links and
//! backlinks are `quote:<id>` links handled by the thread page; hovering
//! one asks the page for a preview. Right-click offers Copy Text, Copy
//! Link, and (on 4chan) Report Post.

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use channex::data::adapters::thread_web_url;
use channex::data::comment::Palette;
use channex::data::html::escape_markup;
use channex::data::model::{PostFile, Site};
use channex::data::store::ReplyStyle;
use gtk::{gdk, gio, glib};

use super::PostRef;
use crate::thumbnails;

/// What the thread page tells a post widget when binding it.
#[derive(Clone)]
pub struct PostContext {
    pub site: Rc<Site>,
    pub board: String,
    pub backlinks: Vec<String>,
    pub style: ReplyStyle,
    pub highlighted: bool,
    pub palette: Rc<Palette>,
    /// Previews are read-only: no hover previews or menus inside them.
    pub interactive: bool,
}

type QuoteCallback = Box<dyn Fn(&str)>;
type HoverCallback = Box<dyn Fn(Option<(String, gtk::Label, f64, f64)>)>;
type FileCallback = Box<dyn Fn(&PostFile)>;
type DownloadCallback = Box<dyn Fn(&PostRef)>;
type ReplyCallback = Box<dyn Fn(&PostRef)>;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct PostWidget {
        pub avatar: OnceCell<adw::Avatar>,
        pub name: OnceCell<gtk::Label>,
        pub op_badge: OnceCell<gtk::Label>,
        pub trip: OnceCell<gtk::Label>,
        pub capcode: OnceCell<gtk::Label>,
        pub date: OnceCell<gtk::Label>,
        pub number: OnceCell<gtk::Label>,
        pub subject: OnceCell<gtk::Label>,
        pub files: OnceCell<gtk::FlowBox>,
        pub comment: OnceCell<gtk::Label>,
        pub backlinks: OnceCell<gtk::Label>,
        pub menu: OnceCell<gtk::PopoverMenu>,
        pub actions: OnceCell<gio::SimpleActionGroup>,

        pub post: RefCell<Option<PostRef>>,
        pub context: RefCell<Option<PostContext>>,
        pub on_quote: RefCell<Option<QuoteCallback>>,
        pub on_hover: RefCell<Option<HoverCallback>>,
        pub on_file: RefCell<Option<FileCallback>>,
        pub on_download: RefCell<Option<DownloadCallback>>,
        pub on_reply: RefCell<Option<ReplyCallback>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PostWidget {
        const NAME: &'static str = "ChannexPostWidget";
        type Type = super::PostWidget;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for PostWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build();
        }

        fn dispose(&self) {
            if let Some(menu) = self.menu.get() {
                menu.unparent();
            }
        }
    }

    impl WidgetImpl for PostWidget {}
    impl BoxImpl for PostWidget {}
}

glib::wrapper! {
    pub struct PostWidget(ObjectSubclass<imp::PostWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for PostWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl PostWidget {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 8)
            .build()
    }

    fn build(&self) {
        let imp = self.imp();
        self.set_css_classes(&["card", "post"]);

        // Poster line.
        let header = gtk::Box::builder().spacing(8).build();
        let avatar = adw::Avatar::builder()
            .size(32)
            .show_initials(true)
            .visible(false)
            .build();
        let name = gtk::Label::builder()
            .css_classes(["heading"])
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        let op_badge = gtk::Label::builder()
            .label("OP")
            .css_classes(["op-badge"])
            .visible(false)
            .build();
        let trip = gtk::Label::builder()
            .css_classes(["dim-label", "monospace"])
            .build();
        let capcode = gtk::Label::builder().css_classes(["capcode-badge"]).build();
        let date = gtk::Label::builder()
            .css_classes(["dim-label", "caption"])
            .build();
        let spacer = gtk::Box::builder().hexpand(true).build();
        let number = gtk::Label::builder()
            .css_classes(["dim-label", "caption", "numeric"])
            .build();
        for w in [
            avatar.upcast_ref::<gtk::Widget>(),
            name.upcast_ref(),
            op_badge.upcast_ref(),
            trip.upcast_ref(),
            capcode.upcast_ref(),
            date.upcast_ref(),
            spacer.upcast_ref(),
            number.upcast_ref(),
        ] {
            header.append(w);
        }
        self.append(&header);

        let subject = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .css_classes(["title-4"])
            .build();
        self.append(&subject);

        let files = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .homogeneous(false)
            .column_spacing(10)
            .row_spacing(10)
            .max_children_per_line(8)
            .build();
        self.append(&files);

        let comment = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .use_markup(true)
            .css_classes(["post-comment"])
            .build();
        comment.set_width_chars(1);
        self.append(&comment);

        let backlinks = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .use_markup(true)
            .css_classes(["caption"])
            .build();
        backlinks.set_width_chars(1);
        self.append(&backlinks);

        for label in [&comment, &backlinks] {
            label.connect_activate_link(glib::clone!(
                #[weak(rename_to = post)]
                self,
                #[upgrade_or]
                glib::Propagation::Proceed,
                move |_, uri| match uri.strip_prefix("quote:") {
                    Some(id) => {
                        if let Some(cb) = post.imp().on_quote.borrow().as_ref() {
                            cb(id);
                        }
                        glib::Propagation::Stop
                    }
                    None => glib::Propagation::Proceed,
                }
            ));
            self.add_hover_preview(label);
        }

        // Right-click menu.
        let menu_model = gio::Menu::new();
        menu_model.append(Some("_Reply"), Some("post.reply"));
        menu_model.append(Some("_Copy Text"), Some("post.copy-text"));
        menu_model.append(Some("Copy _Link"), Some("post.copy-link"));
        let download = gio::MenuItem::new(Some("_Download Files"), Some("post.download"));
        download.set_attribute_value("hidden-when", Some(&"action-disabled".to_variant()));
        menu_model.append_item(&download);
        let report = gio::MenuItem::new(Some("_Report Post…"), Some("post.report"));
        // Only 4chan has a report page to send people to.
        report.set_attribute_value("hidden-when", Some(&"action-disabled".to_variant()));
        menu_model.append_item(&report);
        let menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        menu.set_parent(self);
        menu.set_has_arrow(false);
        menu.set_halign(gtk::Align::Start);
        let click = gtk::GestureClick::builder()
            .button(gdk::BUTTON_SECONDARY)
            .build();
        click.connect_pressed(glib::clone!(
            #[weak(rename_to = post)]
            self,
            move |gesture, _, x, y| {
                if !post
                    .imp()
                    .context
                    .borrow()
                    .as_ref()
                    .is_some_and(|c| c.interactive)
                {
                    return;
                }
                gesture.set_state(gtk::EventSequenceState::Claimed);
                let menu = post.imp().menu.get().unwrap();
                menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                menu.popup();
            }
        ));
        self.add_controller(click);
        self.setup_actions();

        let _ = imp.avatar.set(avatar);
        let _ = imp.name.set(name);
        let _ = imp.op_badge.set(op_badge);
        let _ = imp.trip.set(trip);
        let _ = imp.capcode.set(capcode);
        let _ = imp.date.set(date);
        let _ = imp.number.set(number);
        let _ = imp.subject.set(subject);
        let _ = imp.files.set(files);
        let _ = imp.comment.set(comment);
        let _ = imp.backlinks.set(backlinks);
        let _ = imp.menu.set(menu);
    }

    /// Hovering a `quote:` link asks for a preview; leaving it (or the
    /// label) dismisses it.
    fn add_hover_preview(&self, label: &gtk::Label) {
        let motion = gtk::EventControllerMotion::new();
        motion.connect_motion(glib::clone!(
            #[weak(rename_to = post)]
            self,
            #[weak]
            label,
            move |_, x, y| {
                // The label updates its hovered link in its own handler;
                // read it once that has run.
                glib::idle_add_local_once(glib::clone!(
                    #[weak]
                    post,
                    #[weak]
                    label,
                    move || {
                        let hovered = label
                            .current_uri()
                            .and_then(|u| u.strip_prefix("quote:").map(str::to_owned));
                        post.notify_hover(hovered.map(|id| (id, label.clone(), x, y)));
                    }
                ));
            }
        ));
        motion.connect_leave(glib::clone!(
            #[weak(rename_to = post)]
            self,
            move |_| post.notify_hover(None)
        ));
        label.add_controller(motion);
    }

    fn notify_hover(&self, target: Option<(String, gtk::Label, f64, f64)>) {
        if !self
            .imp()
            .context
            .borrow()
            .as_ref()
            .is_some_and(|c| c.interactive)
        {
            return;
        }
        if let Some(cb) = self.imp().on_hover.borrow().as_ref() {
            cb(target);
        }
    }

    fn setup_actions(&self) {
        let group = gio::SimpleActionGroup::new();
        let copy_text = gio::ActionEntry::builder("copy-text")
            .activate(glib::clone!(
                #[weak(rename_to = post)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| {
                    if let Some(p) = post.imp().post.borrow().as_ref() {
                        post.clipboard().set_text(&p.comment.plain_text());
                    }
                }
            ))
            .build();
        let copy_link = gio::ActionEntry::builder("copy-link")
            .activate(glib::clone!(
                #[weak(rename_to = post)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| {
                    if let Some(link) = post.post_link() {
                        post.clipboard().set_text(&link);
                    }
                }
            ))
            .build();
        let report = gio::ActionEntry::builder("report")
            .activate(glib::clone!(
                #[weak(rename_to = post)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| {
                    let (Some(p), Some(ctx)) = (
                        post.imp().post.borrow().clone(),
                        post.imp().context.borrow().clone(),
                    ) else {
                        return;
                    };
                    // 4chan has no report API; its report page asks for
                    // a reason and a captcha in the browser.
                    let url = format!(
                        "https://sys.4chan.org/{}/imgboard.php?mode=report&no={}",
                        ctx.board, p.id
                    );
                    let window = post.root().and_downcast::<gtk::Window>();
                    gtk::UriLauncher::new(&url).launch(
                        window.as_ref(),
                        gio::Cancellable::NONE,
                        |_| {},
                    );
                }
            ))
            .build();
        let reply = gio::ActionEntry::builder("reply")
            .activate(glib::clone!(
                #[weak(rename_to = post)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| {
                    let current = post.imp().post.borrow().clone();
                    if let (Some(p), Some(cb)) = (current, post.imp().on_reply.borrow().as_ref()) {
                        cb(&p);
                    }
                }
            ))
            .build();
        let download = gio::ActionEntry::builder("download")
            .activate(glib::clone!(
                #[weak(rename_to = post)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| {
                    let current = post.imp().post.borrow().clone();
                    if let (Some(p), Some(cb)) = (current, post.imp().on_download.borrow().as_ref())
                    {
                        cb(&p);
                    }
                }
            ))
            .build();
        group.add_action_entries([reply, copy_text, copy_link, report, download]);
        self.insert_action_group("post", Some(&group));
        let _ = self.imp().actions.set(group);
    }

    fn post_link(&self) -> Option<String> {
        let post = self.imp().post.borrow().clone()?;
        let ctx = self.imp().context.borrow().clone()?;
        let page = thread_web_url(&ctx.site, &ctx.board, &post.thread_id);
        let anchor = if ctx.site.id == "4chan" {
            format!("#p{}", post.id)
        } else {
            format!("#{}", post.id)
        };
        Some(format!("{page}{anchor}"))
    }

    pub fn connect_quote_clicked(&self, f: impl Fn(&str) + 'static) {
        self.imp().on_quote.replace(Some(Box::new(f)));
    }

    pub fn connect_quote_hovered(
        &self,
        f: impl Fn(Option<(String, gtk::Label, f64, f64)>) + 'static,
    ) {
        self.imp().on_hover.replace(Some(Box::new(f)));
    }

    pub fn connect_reply_clicked(&self, f: impl Fn(&PostRef) + 'static) {
        self.imp().on_reply.replace(Some(Box::new(f)));
    }

    pub fn connect_download_clicked(&self, f: impl Fn(&PostRef) + 'static) {
        self.imp().on_download.replace(Some(Box::new(f)));
    }

    pub fn connect_file_clicked(&self, f: impl Fn(&PostFile) + 'static) {
        self.imp().on_file.replace(Some(Box::new(f)));
    }

    pub fn bind(&self, post: &PostRef, ctx: &PostContext) {
        let imp = self.imp();
        imp.post.replace(Some(post.clone()));
        imp.context.replace(Some(ctx.clone()));

        for (class, on) in [
            ("op", post.is_op),
            ("highlighted", ctx.highlighted),
            ("compact", ctx.style == ReplyStyle::Compact),
            ("modern", ctx.style == ReplyStyle::Modern),
        ] {
            if on {
                self.add_css_class(class);
            } else {
                self.remove_css_class(class);
            }
        }

        let avatar = imp.avatar.get().unwrap();
        avatar.set_visible(ctx.style == ReplyStyle::Modern);
        avatar.set_text(Some(post.display_name()));
        imp.name.get().unwrap().set_label(post.display_name());
        imp.op_badge.get().unwrap().set_visible(post.is_op);
        let trip = imp.trip.get().unwrap();
        trip.set_label(&post.tripcode);
        trip.set_visible(!post.tripcode.is_empty());
        let capcode = imp.capcode.get().unwrap();
        capcode.set_label(&post.capcode);
        capcode.set_visible(!post.capcode.is_empty());
        let date = imp.date.get().unwrap();
        date.set_label(&format_time(post.timestamp));
        imp.number
            .get()
            .unwrap()
            .set_label(&format!("No.{}", post.id));

        let subject = imp.subject.get().unwrap();
        subject.set_label(&post.subject);
        subject.set_visible(!post.subject.is_empty());

        let comment = imp.comment.get().unwrap();
        comment.set_markup(&post.comment.to_pango(&ctx.palette));
        comment.set_visible(!post.comment.is_empty());

        let backlinks = imp.backlinks.get().unwrap();
        if ctx.backlinks.is_empty() {
            backlinks.set_visible(false);
        } else {
            let links: Vec<String> = ctx
                .backlinks
                .iter()
                .map(|id| format!("<a href=\"quote:{0}\">&gt;&gt;{0}</a>", escape_markup(id)))
                .collect();
            backlinks.set_markup(&format!("Replies: {}", links.join(" ")));
            backlinks.set_visible(true);
        }

        if let Some(report) = imp
            .actions
            .get()
            .and_then(|g| g.lookup_action("report"))
            .and_downcast::<gio::SimpleAction>()
        {
            report.set_enabled(ctx.site.id == "4chan");
        }
        if let Some(download) = imp
            .actions
            .get()
            .and_then(|g| g.lookup_action("download"))
            .and_downcast::<gio::SimpleAction>()
        {
            download.set_enabled(!post.files.is_empty());
        }

        self.bind_files(post, ctx);
    }

    fn bind_files(&self, post: &PostRef, ctx: &PostContext) {
        let files = self.imp().files.get().unwrap();
        files.remove_all();
        files.set_visible(!post.files.is_empty());
        let size = match ctx.style {
            ReplyStyle::Compact => 110,
            ReplyStyle::Standard => 160,
            ReplyStyle::Modern => 190,
        };
        let hats = crate::birthday::hats_today();
        for (i, file) in post.files.iter().enumerate() {
            let tile = self.file_tile(
                post,
                file,
                size,
                ctx.style != ReplyStyle::Compact,
                hats && i == 0,
            );
            files.append(&tile);
        }
    }

    fn file_tile(
        &self,
        post: &PostRef,
        file: &PostFile,
        size: i32,
        caption: bool,
        hat: bool,
    ) -> gtk::Widget {
        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .can_shrink(true)
            .build();
        let clamp = |orientation, child: &gtk::Widget| {
            adw::Clamp::builder()
                .orientation(orientation)
                .maximum_size(size)
                .tightening_threshold(size)
                .unit(adw::LengthUnit::Px)
                .child(child)
                .build()
        };
        picture.set_size_request(size, size);
        let square = clamp(
            gtk::Orientation::Vertical,
            clamp(gtk::Orientation::Horizontal, picture.upcast_ref()).upcast_ref(),
        );
        let frame = gtk::Box::builder()
            .css_classes(["thumbnail-frame"])
            .overflow(gtk::Overflow::Hidden)
            .build();
        frame.append(&square);
        let overlay = gtk::Overlay::builder().child(&frame).build();
        if hat {
            overlay.add_overlay(&crate::birthday::hat(size / 4 + 8));
        }
        if file.is_video {
            overlay.add_overlay(
                &gtk::Image::builder()
                    .icon_name("media-playback-start-symbolic")
                    .pixel_size(24)
                    .css_classes(["osd", "thumbnail-badge"])
                    .halign(gtk::Align::Center)
                    .valign(gtk::Align::Center)
                    .can_target(false)
                    .build(),
            );
        }

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        content.append(&overlay);
        if caption {
            // "PNG · 120 KB · 800×600"
            let mut parts = vec![file.ext.trim_start_matches('.').to_uppercase()];
            if file.size > 0 {
                parts.push(format_size(file.size));
            }
            if file.width > 0 {
                parts.push(format!("{}×{}", file.width, file.height));
            }
            parts.retain(|p| !p.is_empty());
            content.append(
                &gtk::Label::builder()
                    .label(parts.join(" · "))
                    .css_classes(["caption", "dim-label"])
                    .xalign(0.0)
                    .build(),
            );
        }
        let button = gtk::Button::builder()
            .child(&content)
            .css_classes(["flat", "file-button"])
            .tooltip_text(&file.name)
            .build();
        let file_for_click = file.clone();
        button.connect_clicked(glib::clone!(
            #[weak(rename_to = post_widget)]
            self,
            move |_| {
                if let Some(cb) = post_widget.imp().on_file.borrow().as_ref() {
                    cb(&file_for_click);
                }
            }
        ));

        let url = file.thumb_url.clone();
        let expected = post.clone();
        let this = self.downgrade();
        glib::spawn_future_local(async move {
            let texture = thumbnails::load(&url).await;
            let Some(this) = this.upgrade() else { return };
            let still_same = this
                .imp()
                .post
                .borrow()
                .as_ref()
                .is_some_and(|p| Rc::ptr_eq(p, &expected));
            if let (true, Some(texture)) = (still_same, texture) {
                picture.set_paintable(Some(&texture));
            }
        });
        button.upcast()
    }
}

/// "Nov 4, 2026 · 14:05" in local time.
fn format_time(ms: i64) -> String {
    glib::DateTime::from_unix_local(ms / 1000)
        .and_then(|t| t.format("%b %e, %Y · %H:%M"))
        .map(|s| s.replace("  ", " "))
        .unwrap_or_default()
}

/// "120 KB", "2.4 MB".
fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        String::new()
    } else if bytes < 1024 * 1024 {
        format!("{} KB", bytes.div_ceil(1024))
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

//! A thread in the catalog grid: an Adwaita card with the thumbnail
//! (sticky/closed badges, a bookmark button on hover, and a blur for
//! NSFW/spoiler images), then subject, excerpt, and reply/image counts.

use std::cell::{OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use super::{CardOptions, PostRef, stats_text};
use crate::thumbnails;

type BookmarkCallback = Box<dyn Fn(&PostRef)>;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ThreadCard {
        pub picture: OnceCell<gtk::Picture>,
        pub placeholder: OnceCell<gtk::Image>,
        pub sticky: OnceCell<gtk::Image>,
        pub closed: OnceCell<gtk::Image>,
        pub spoiler: OnceCell<gtk::Box>,
        pub bookmark: OnceCell<gtk::Button>,
        pub subject: OnceCell<gtk::Label>,
        pub excerpt: OnceCell<gtk::Label>,
        pub stats: OnceCell<gtk::Label>,
        pub post: RefCell<Option<PostRef>>,
        pub on_bookmark: RefCell<Option<BookmarkCallback>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThreadCard {
        const NAME: &'static str = "ChannexThreadCard";
        type Type = super::ThreadCard;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ThreadCard {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build();
        }
    }

    impl WidgetImpl for ThreadCard {}
    impl BoxImpl for ThreadCard {}
}

glib::wrapper! {
    pub struct ThreadCard(ObjectSubclass<imp::ThreadCard>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for ThreadCard {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadCard {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .build()
    }

    fn build(&self) {
        let imp = self.imp();
        self.set_css_classes(&["card", "activatable", "thread-card"]);

        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .can_shrink(true)
            .build();
        let frame = gtk::Box::builder()
            .css_classes(["thumbnail-frame"])
            .overflow(gtk::Overflow::Hidden)
            .build();
        picture.set_hexpand(true);
        frame.append(&picture);

        let overlay = gtk::Overlay::builder().child(&frame).build();

        let placeholder = gtk::Image::builder()
            .icon_name("image-x-generic-symbolic")
            .pixel_size(32)
            .css_classes(["dim-label"])
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .can_target(false)
            .build();
        overlay.add_overlay(&placeholder);

        let badges = gtk::Box::builder()
            .spacing(4)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Start)
            .margin_start(12)
            .margin_top(12)
            .can_target(false)
            .build();
        let badge = |icon: &str, tip: &str| {
            gtk::Image::builder()
                .icon_name(icon)
                .tooltip_text(tip)
                .css_classes(["osd", "thumbnail-badge"])
                .build()
        };
        let sticky = badge("view-pin-symbolic", "Sticky");
        let closed = badge("changes-prevent-symbolic", "Closed");
        badges.append(&sticky);
        badges.append(&closed);
        overlay.add_overlay(&badges);

        let spoiler = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .can_target(false)
            .build();
        spoiler.append(
            &gtk::Image::builder()
                .icon_name("view-conceal-symbolic")
                .pixel_size(24)
                .build(),
        );
        spoiler.append(
            &gtk::Label::builder()
                .label("Click to Reveal")
                .css_classes(["caption-heading"])
                .build(),
        );
        spoiler.add_css_class("osd");
        spoiler.add_css_class("spoiler-badge");
        overlay.add_overlay(&spoiler);

        let bookmark = gtk::Button::builder()
            .icon_name("bookmark-new-symbolic")
            .css_classes(["osd", "circular", "bookmark-button"])
            .halign(gtk::Align::End)
            .valign(gtk::Align::Start)
            .margin_end(12)
            .margin_top(12)
            .build();
        bookmark.connect_clicked(glib::clone!(
            #[weak(rename_to = card)]
            self,
            move |_| {
                let post = card.imp().post.borrow().clone();
                if let (Some(post), Some(cb)) = (post, card.imp().on_bookmark.borrow().as_ref()) {
                    cb(&post);
                }
            }
        ));
        overlay.add_overlay(&bookmark);
        self.append(&overlay);

        let text = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .margin_start(12)
            .margin_end(12)
            .margin_top(10)
            .margin_bottom(12)
            .vexpand(true)
            .build();
        let subject = gtk::Label::builder()
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(["heading"])
            .build();
        let excerpt = gtk::Label::builder()
            .xalign(0.0)
            .yalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .lines(3)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .vexpand(true)
            .css_classes(["dim-label"])
            .build();
        // Natural width of a wrapping label is its unwrapped text; keep
        // it from widening the card.
        excerpt.set_width_chars(1);
        excerpt.set_max_width_chars(1);
        let stats = gtk::Label::builder()
            .xalign(0.0)
            .css_classes(["caption", "dim-label"])
            .build();
        text.append(&subject);
        text.append(&excerpt);
        text.append(&stats);
        self.append(&text);

        let _ = imp.picture.set(picture);
        let _ = imp.placeholder.set(placeholder);
        let _ = imp.sticky.set(sticky);
        let _ = imp.closed.set(closed);
        let _ = imp.spoiler.set(spoiler);
        let _ = imp.bookmark.set(bookmark);
        let _ = imp.subject.set(subject);
        let _ = imp.excerpt.set(excerpt);
        let _ = imp.stats.set(stats);
    }

    /// Called when the bookmark button is clicked, with the shown post.
    pub fn connect_bookmark_clicked(&self, f: impl Fn(&PostRef) + 'static) {
        self.imp().on_bookmark.replace(Some(Box::new(f)));
    }

    pub fn bind(&self, post: &PostRef, options: CardOptions) {
        let imp = self.imp();
        imp.post.replace(Some(post.clone()));

        let (width, thumb_height) = if options.compact {
            (170, 130)
        } else {
            (230, 190)
        };
        self.set_width_request(width);
        let picture = imp.picture.get().unwrap();
        picture.set_height_request(thumb_height);

        imp.subject.get().unwrap().set_label(&post.title());
        let excerpt = imp.excerpt.get().unwrap();
        excerpt.set_visible(!options.compact);
        excerpt.set_label(&post.comment.excerpt(220));
        imp.stats.get().unwrap().set_label(&stats_text(post));
        imp.sticky.get().unwrap().set_visible(post.sticky);
        imp.closed.get().unwrap().set_visible(post.closed);

        let bookmark = imp.bookmark.get().unwrap();
        bookmark.set_icon_name(if options.bookmarked {
            "user-bookmarks-symbolic"
        } else {
            "bookmark-new-symbolic"
        });
        bookmark.set_tooltip_text(Some(if options.bookmarked {
            "Remove Bookmark"
        } else {
            "Bookmark"
        }));
        if options.bookmarked {
            bookmark.add_css_class("bookmarked");
        } else {
            bookmark.remove_css_class("bookmarked");
        }

        if options.blurred {
            picture.add_css_class("blurred");
        } else {
            picture.remove_css_class("blurred");
        }
        imp.spoiler.get().unwrap().set_visible(options.blurred);

        let thumb = post
            .files
            .first()
            .map(|f| f.thumb_url.clone())
            .unwrap_or_default();
        picture.set_paintable(gtk::gdk::Paintable::NONE);
        imp.placeholder.get().unwrap().set_visible(true);
        if thumb.is_empty() {
            return;
        }
        let card = self.downgrade();
        let expected = post.clone();
        glib::spawn_future_local(async move {
            let texture = thumbnails::load(&thumb).await;
            let Some(card) = card.upgrade() else { return };
            // The cell may have been recycled for another thread.
            let still_same = card
                .imp()
                .post
                .borrow()
                .as_ref()
                .is_some_and(|p| std::rc::Rc::ptr_eq(p, &expected));
            if let (true, Some(texture)) = (still_same, texture) {
                card.imp()
                    .picture
                    .get()
                    .unwrap()
                    .set_paintable(Some(&texture));
                card.imp().placeholder.get().unwrap().set_visible(false);
            }
        });
    }

    pub fn unbind(&self) {
        self.imp().post.replace(None);
    }
}

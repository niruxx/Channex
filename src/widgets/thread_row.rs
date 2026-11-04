//! A thread in the catalog's list mode: a card row with a square
//! thumbnail, subject, a two-line excerpt, and counts.

use std::cell::{OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use super::{CardOptions, PostRef, stats_text};
use crate::thumbnails;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ThreadRow {
        pub picture: OnceCell<gtk::Picture>,
        pub placeholder: OnceCell<gtk::Image>,
        pub subject: OnceCell<gtk::Label>,
        pub sticky: OnceCell<gtk::Image>,
        pub excerpt: OnceCell<gtk::Label>,
        pub stats: OnceCell<gtk::Label>,
        pub post: RefCell<Option<PostRef>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThreadRow {
        const NAME: &'static str = "ChannexThreadRow";
        type Type = super::ThreadRow;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ThreadRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build();
        }
    }

    impl WidgetImpl for ThreadRow {}
    impl BoxImpl for ThreadRow {}
}

glib::wrapper! {
    pub struct ThreadRow(ObjectSubclass<imp::ThreadRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for ThreadRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadRow {
    pub fn new() -> Self {
        glib::Object::builder().property("spacing", 14).build()
    }

    fn build(&self) {
        let imp = self.imp();
        self.set_css_classes(&["card", "activatable", "thread-row"]);
        self.set_margin_top(4);
        self.set_margin_bottom(4);

        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .can_shrink(true)
            .build();
        picture.set_size_request(72, 72);
        let frame = gtk::Box::builder()
            .css_classes(["thumbnail-frame"])
            .overflow(gtk::Overflow::Hidden)
            .valign(gtk::Align::Center)
            .margin_start(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();
        // A picture's natural size is its image's; clamp it to a fixed
        // square so every row's thumbnail lines up.
        let clamp = |orientation, child: &gtk::Widget| {
            adw::Clamp::builder()
                .orientation(orientation)
                .maximum_size(72)
                .tightening_threshold(72)
                .unit(adw::LengthUnit::Px)
                .child(child)
                .build()
        };
        let square = clamp(
            gtk::Orientation::Vertical,
            clamp(gtk::Orientation::Horizontal, picture.upcast_ref()).upcast_ref(),
        );
        frame.append(&square);
        let placeholder = gtk::Image::builder()
            .icon_name("image-x-generic-symbolic")
            .pixel_size(24)
            .css_classes(["dim-label"])
            .can_target(false)
            .build();
        let overlay = gtk::Overlay::builder().child(&frame).build();
        overlay.add_overlay(&placeholder);
        self.append(&overlay);

        let text = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(3)
            .valign(gtk::Align::Center)
            .hexpand(true)
            .margin_end(14)
            .build();
        let subject = gtk::Label::builder()
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(["heading"])
            .build();
        let excerpt = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .lines(2)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(["dim-label"])
            .build();
        excerpt.set_width_chars(1);
        let stats = gtk::Label::builder()
            .xalign(0.0)
            .css_classes(["caption", "dim-label"])
            .build();
        let sticky = gtk::Image::builder()
            .icon_name("view-pin-symbolic")
            .tooltip_text("Sticky")
            .build();
        let title_row = gtk::Box::builder().spacing(6).build();
        title_row.append(&sticky);
        title_row.append(&subject);
        text.append(&title_row);
        text.append(&excerpt);
        text.append(&stats);
        self.append(&text);

        let _ = imp.picture.set(picture);
        let _ = imp.placeholder.set(placeholder);
        let _ = imp.subject.set(subject);
        let _ = imp.sticky.set(sticky);
        let _ = imp.excerpt.set(excerpt);
        let _ = imp.stats.set(stats);
    }

    pub fn bind(&self, post: &PostRef, options: CardOptions) {
        let imp = self.imp();
        imp.post.replace(Some(post.clone()));
        imp.subject.get().unwrap().set_label(&post.title());
        imp.sticky.get().unwrap().set_visible(post.sticky);
        imp.excerpt
            .get()
            .unwrap()
            .set_label(&post.comment.excerpt(300));
        imp.stats.get().unwrap().set_label(&stats_text(post));

        let picture = imp.picture.get().unwrap();
        if options.blurred {
            picture.add_css_class("blurred");
        } else {
            picture.remove_css_class("blurred");
        }
        picture.set_paintable(gtk::gdk::Paintable::NONE);
        imp.placeholder.get().unwrap().set_visible(true);
        let thumb = post
            .files
            .first()
            .map(|f| f.thumb_url.clone())
            .unwrap_or_default();
        if thumb.is_empty() {
            return;
        }
        let row = self.downgrade();
        let expected = post.clone();
        glib::spawn_future_local(async move {
            let texture = thumbnails::load(&thumb).await;
            let Some(row) = row.upgrade() else { return };
            let still_same = row
                .imp()
                .post
                .borrow()
                .as_ref()
                .is_some_and(|p| std::rc::Rc::ptr_eq(p, &expected));
            if let (true, Some(texture)) = (still_same, texture) {
                row.imp()
                    .picture
                    .get()
                    .unwrap()
                    .set_paintable(Some(&texture));
                row.imp().placeholder.get().unwrap().set_visible(false);
            }
        });
    }

    pub fn unbind(&self) {
        self.imp().post.replace(None);
    }
}

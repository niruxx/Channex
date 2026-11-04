//! An animated GIF, decoded with gdk-pixbuf and drawn frame by frame.
//!
//! GTK's media player (GStreamer) can show GIFs but can't seek in them,
//! so its looping - a seek back to the start - leaves them stuck on the
//! last frame. Here each frame is shown for its own delay, and the
//! animation always starts over at the end (whatever loop count the file
//! asks for). The file is decoded from memory; nothing touches the disk.

use std::cell::{OnceCell, RefCell};
use std::time::{Duration, SystemTime};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gdk_pixbuf::{Pixbuf, PixbufAnimation, PixbufAnimationIter, PixbufLoader};
use gtk::{gdk, glib};

/// Shortest frame delay honoured; GIFs often say 0 or 10 ms, which
/// browsers also stretch.
const MIN_DELAY: Duration = Duration::from_millis(20);

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AnimatedImage {
        pub picture: OnceCell<gtk::Picture>,
        pub animation: OnceCell<PixbufAnimation>,
        pub iter: RefCell<Option<PixbufAnimationIter>>,
        pub timer: RefCell<Option<glib::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AnimatedImage {
        const NAME: &'static str = "ChannexAnimatedImage";
        type Type = super::AnimatedImage;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for AnimatedImage {
        fn dispose(&self) {
            if let Some(timer) = self.timer.take() {
                timer.remove();
            }
        }
    }

    impl WidgetImpl for AnimatedImage {}
    impl BinImpl for AnimatedImage {}
}

glib::wrapper! {
    pub struct AnimatedImage(ObjectSubclass<imp::AnimatedImage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AnimatedImage {
    /// Decodes a GIF (or any format gdk-pixbuf animates). Paused until
    /// [`AnimatedImage::play`].
    pub fn new(bytes: &[u8]) -> Result<Self, glib::Error> {
        let loader = PixbufLoader::new();
        loader.write(bytes)?;
        loader.close()?;
        let animation = loader.animation().ok_or_else(|| {
            glib::Error::new(
                gtk::gdk_pixbuf::PixbufError::CorruptImage,
                "not an animation",
            )
        })?;

        let this: Self = glib::Object::new();
        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Contain)
            .can_shrink(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        this.set_child(Some(&picture));
        let _ = this.imp().picture.set(picture);
        let iter = animation.iter(Some(SystemTime::now()));
        this.show(&iter.pixbuf());
        this.imp().iter.replace(Some(iter));
        let _ = this.imp().animation.set(animation);
        Ok(this)
    }

    pub fn play(&self) {
        if self.imp().timer.borrow().is_none() {
            self.schedule_next();
        }
    }

    pub fn pause(&self) {
        if let Some(timer) = self.imp().timer.take() {
            timer.remove();
        }
    }

    fn schedule_next(&self) {
        let imp = self.imp();
        let Some(animation) = imp.animation.get() else {
            return;
        };
        if animation.is_static_image() {
            return;
        }
        // A negative delay means "this frame forever" (the file's loop
        // count is used up): start over instead.
        let delay = imp
            .iter
            .borrow()
            .as_ref()
            .and_then(PixbufAnimationIter::delay_time);
        let delay = match delay {
            Some(delay) => delay.max(MIN_DELAY),
            None => {
                let iter = animation.iter(Some(SystemTime::now()));
                self.show(&iter.pixbuf());
                imp.iter.replace(Some(iter));
                imp.iter
                    .borrow()
                    .as_ref()
                    .and_then(PixbufAnimationIter::delay_time)
                    .unwrap_or(MIN_DELAY)
                    .max(MIN_DELAY)
            }
        };
        let weak = self.downgrade();
        let id = glib::timeout_add_local_once(delay, move || {
            let Some(this) = weak.upgrade() else {
                return;
            };
            this.imp().timer.replace(None);
            let frame = {
                let iter = this.imp().iter.borrow();
                iter.as_ref().map(|iter| {
                    iter.advance(SystemTime::now());
                    iter.pixbuf()
                })
            };
            if let Some(frame) = frame {
                this.show(&frame);
            }
            this.schedule_next();
        });
        imp.timer.replace(Some(id));
    }

    fn show(&self, pixbuf: &Pixbuf) {
        let format = if pixbuf.has_alpha() {
            gdk::MemoryFormat::R8g8b8a8
        } else {
            gdk::MemoryFormat::R8g8b8
        };
        let texture = gdk::MemoryTexture::new(
            pixbuf.width(),
            pixbuf.height(),
            format,
            &pixbuf.read_pixel_bytes(),
            pixbuf.rowstride() as usize,
        );
        if let Some(picture) = self.imp().picture.get() {
            picture.set_paintable(Some(&texture));
        }
    }
}

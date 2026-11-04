//! An image that fits the view, and zooms on click (to actual size, or
//! 2× for small images), pinch, or Ctrl+scroll. While zoomed it scrolls,
//! and dragging pans it.

use std::cell::{Cell, OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, glib};

const MAX_SCALE: f64 = 8.0;

type ZoomCallback = Box<dyn Fn(bool)>;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ZoomableImage {
        pub scrolled: OnceCell<gtk::ScrolledWindow>,
        pub picture: OnceCell<gtk::Picture>,
        pub texture: RefCell<Option<gdk::Texture>>,
        /// 0 = fit to the view; otherwise image pixels per screen pixel.
        pub scale: Cell<f64>,
        pub drag_start: Cell<(f64, f64)>,
        pub dragged: Cell<bool>,
        pub pinch_start: Cell<f64>,

        pub on_zoom: RefCell<Option<ZoomCallback>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ZoomableImage {
        const NAME: &'static str = "ChannexZoomableImage";
        type Type = super::ZoomableImage;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for ZoomableImage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build();
        }
    }

    impl WidgetImpl for ZoomableImage {}
    impl BinImpl for ZoomableImage {}
}

glib::wrapper! {
    pub struct ZoomableImage(ObjectSubclass<imp::ZoomableImage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ZoomableImage {
    pub fn new(texture: &gdk::Texture) -> Self {
        let this: Self = glib::Object::new();
        this.imp().texture.replace(Some(texture.clone()));
        this.imp()
            .picture
            .get()
            .unwrap()
            .set_paintable(Some(texture));
        this
    }

    /// Called with `true` when the image becomes zoomed, `false` when it
    /// returns to fitting the view.
    pub fn connect_zoom_changed(&self, f: impl Fn(bool) + 'static) {
        self.imp().on_zoom.replace(Some(Box::new(f)));
    }

    pub fn is_zoomed(&self) -> bool {
        self.imp().scale.get() > 0.0
    }

    pub fn reset(&self) {
        self.set_scale(0.0, None);
    }

    fn build(&self) {
        let imp = self.imp();
        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Contain)
            .can_shrink(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        let scrolled = gtk::ScrolledWindow::builder()
            .child(&picture)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .build();
        self.set_child(Some(&scrolled));
        self.set_cursor_from_name(Some("zoom-in"));

        // Click toggles zoom; a drag (while zoomed) pans instead.
        let drag = gtk::GestureDrag::new();
        drag.connect_drag_begin(glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _, _| {
                let s = this.imp().scrolled.get().unwrap();
                this.imp()
                    .drag_start
                    .set((s.hadjustment().value(), s.vadjustment().value()));
                this.imp().dragged.set(false);
            }
        ));
        drag.connect_drag_update(glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |gesture, dx, dy| {
                if dx.abs() + dy.abs() > 6.0 {
                    this.imp().dragged.set(true);
                }
                if !this.is_zoomed() {
                    return;
                }
                gesture.set_state(gtk::EventSequenceState::Claimed);
                let s = this.imp().scrolled.get().unwrap();
                let (x, y) = this.imp().drag_start.get();
                s.hadjustment().set_value(x - dx);
                s.vadjustment().set_value(y - dy);
            }
        ));
        drag.connect_drag_end(glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |gesture, dx, dy| {
                if this.imp().dragged.get() {
                    return;
                }
                if let Some((x, y)) = gesture.start_point() {
                    this.toggle_zoom(x + dx, y + dy);
                }
            }
        ));
        self.add_controller(drag);

        let pinch = gtk::GestureZoom::new();
        pinch.connect_begin(glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _| this.imp().pinch_start.set(this.effective_scale())
        ));
        pinch.connect_scale_changed(glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |_, delta| {
                let scale = this.imp().pinch_start.get() * delta;
                this.set_scale(
                    if scale <= this.fit_scale() {
                        0.0
                    } else {
                        scale
                    },
                    None,
                );
            }
        ));
        self.add_controller(pinch);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.connect_scroll(glib::clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |controller, _, dy| {
                if !controller
                    .current_event_state()
                    .contains(gdk::ModifierType::CONTROL_MASK)
                {
                    return glib::Propagation::Proceed;
                }
                let factor = if dy < 0.0 { 1.25 } else { 0.8 };
                let scale = this.effective_scale() * factor;
                this.set_scale(
                    if scale <= this.fit_scale() {
                        0.0
                    } else {
                        scale
                    },
                    None,
                );
                glib::Propagation::Stop
            }
        ));
        self.add_controller(scroll);

        let _ = imp.scrolled.set(scrolled);
        let _ = imp.picture.set(picture);
    }

    #[cfg(debug_assertions)]
    pub fn dev_toggle_zoom(&self) {
        self.toggle_zoom(
            f64::from(self.width()) / 2.0,
            f64::from(self.height()) / 2.0,
        );
    }

    fn image_size(&self) -> (f64, f64) {
        self.imp()
            .texture
            .borrow()
            .as_ref()
            .map_or((1.0, 1.0), |t| {
                (f64::from(t.width().max(1)), f64::from(t.height().max(1)))
            })
    }

    /// Scale at which the whole image fits the view.
    fn fit_scale(&self) -> f64 {
        let (w, h) = self.image_size();
        let (vw, vh) = (
            f64::from(self.width().max(1)),
            f64::from(self.height().max(1)),
        );
        (vw / w).min(vh / h).min(1.0)
    }

    fn effective_scale(&self) -> f64 {
        let scale = self.imp().scale.get();
        if scale > 0.0 { scale } else { self.fit_scale() }
    }

    fn toggle_zoom(&self, x: f64, y: f64) {
        if self.is_zoomed() {
            self.set_scale(0.0, None);
        } else {
            // Actual size, or 2× when actual size already fits.
            let target = if self.fit_scale() < 1.0 { 1.0 } else { 2.0 };
            self.set_scale(target, Some((x, y)));
        }
    }

    /// Applies a zoom level (0 = fit), keeping `anchor` (a point in this
    /// widget) over the same spot of the image where possible.
    fn set_scale(&self, scale: f64, anchor: Option<(f64, f64)>) {
        let imp = self.imp();
        let scale = scale.min(MAX_SCALE);
        let was_zoomed = self.is_zoomed();
        let scrolled = imp.scrolled.get().unwrap();
        let picture = imp.picture.get().unwrap();

        // Where the anchor falls in the image, as fractions.
        let (fx, fy) = match anchor {
            Some((x, y)) => {
                let (w, h) = self.image_size();
                let fit = self.effective_scale();
                let (vw, vh) = (f64::from(self.width()), f64::from(self.height()));
                let (iw, ih) = (w * fit, h * fit);
                let (ox, oy) = ((vw - iw).max(0.0) / 2.0, (vh - ih).max(0.0) / 2.0);
                let h_adj = scrolled.hadjustment();
                let v_adj = scrolled.vadjustment();
                (
                    ((x + h_adj.value() - ox) / iw).clamp(0.0, 1.0),
                    ((y + v_adj.value() - oy) / ih).clamp(0.0, 1.0),
                )
            }
            None => (0.5, 0.5),
        };

        imp.scale.set(scale);
        if scale > 0.0 {
            let (w, h) = self.image_size();
            picture.set_can_shrink(false);
            picture.set_size_request((w * scale) as i32, (h * scale) as i32);
            scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
            // Scroll to the anchor once the zoomed size has actually been
            // laid out (GTK resets the position while switching the view
            // to scrolling), checking on each frame for up to ~half a second.
            let (target_w, target_h) = (w * scale, h * scale);
            let frames = Cell::new(0);
            scrolled.add_tick_callback(move |scrolled, _| {
                frames.set(frames.get() + 1);
                let (h_adj, v_adj) = (scrolled.hadjustment(), scrolled.vadjustment());
                let laid_out = h_adj.upper() + 1.0 >= target_w.min(h_adj.page_size().max(target_w))
                    && v_adj.upper() + 1.0 >= target_h.min(v_adj.page_size().max(target_h))
                    && h_adj.page_size() > 0.0;
                if laid_out && frames.get() > 1 {
                    h_adj.set_value(fx * h_adj.upper() - h_adj.page_size() / 2.0);
                    v_adj.set_value(fy * v_adj.upper() - v_adj.page_size() / 2.0);
                    return glib::ControlFlow::Break;
                }
                if frames.get() > 30 {
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        } else {
            picture.set_size_request(-1, -1);
            picture.set_can_shrink(true);
            scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
        }
        self.set_cursor_from_name(Some(if scale > 0.0 { "grab" } else { "zoom-in" }));

        if was_zoomed != (scale > 0.0) {
            if let Some(cb) = imp.on_zoom.borrow().as_ref() {
                cb(scale > 0.0);
            }
        }
    }
}

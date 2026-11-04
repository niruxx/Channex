//! The media viewer: every file of a thread, full-window over the app.
//!
//! An `AdwCarousel` holds one slot per file (swipe, arrow buttons, or the
//! Left/Right keys to move); only the current file and its neighbours are
//! loaded, so a thread with hundreds of files stays light. Images show
//! their thumbnail while the full file downloads, then zoom and pan (see
//! [`zoom`]); GIFs animate; videos stream with GTK's built-in controls.
//! A filmstrip along the bottom jumps to any file. Esc closes.

mod zoom;

use std::cell::{Cell, OnceCell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use channex::data::model::PostFile;
use gtk::{gdk, gio, glib};

use self::zoom::ZoomableImage;
use crate::downloads::downloads;
use crate::runtime;
use crate::state::state;
use crate::thumbnails;
use crate::window::ChannexWindow;

/// Where the files come from, for saving them.
#[derive(Debug, Clone)]
pub struct ViewerContext {
    /// Folder that "Save" downloads into.
    pub save_dir: PathBuf,
    pub save_label: String,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MediaViewer {
        pub window: glib::WeakRef<ChannexWindow>,
        pub files: RefCell<Rc<Vec<PostFile>>>,
        pub context: RefCell<Option<ViewerContext>>,
        pub index: Cell<u32>,
        /// One per file; filled when near the current position.
        pub slots: RefCell<Vec<gtk::Box>>,
        pub zoomed: Cell<bool>,

        pub title: OnceCell<adw::WindowTitle>,
        pub carousel: OnceCell<adw::Carousel>,
        pub prev: OnceCell<gtk::Button>,
        pub next: OnceCell<gtk::Button>,
        pub strip_model: OnceCell<gio::ListStore>,
        pub strip_selection: OnceCell<gtk::SingleSelection>,
        pub strip: OnceCell<gtk::ListView>,
        pub strip_bar: OnceCell<gtk::Widget>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MediaViewer {
        const NAME: &'static str = "ChannexMediaViewer";
        type Type = super::MediaViewer;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for MediaViewer {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build();
        }
    }

    impl WidgetImpl for MediaViewer {}
    impl BinImpl for MediaViewer {}
}

glib::wrapper! {
    pub struct MediaViewer(ObjectSubclass<imp::MediaViewer>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MediaViewer {
    pub fn new(window: &ChannexWindow) -> Self {
        let viewer: Self = glib::Object::builder().property("visible", false).build();
        viewer.imp().window.set(Some(window));
        viewer
    }

    // ------------------------------------------------------------------
    // Layout
    // ------------------------------------------------------------------

    fn build(&self) {
        let imp = self.imp();
        self.set_focusable(true);
        self.add_css_class("media-viewer");

        let back = gtk::Button::builder().icon_name("go-previous-symbolic").tooltip_text("Back (Esc)").build();
        back.connect_clicked(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_| viewer.close()
        ));
        let title = adw::WindowTitle::new("", "");
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(&back);

        let open = gtk::Button::builder().icon_name("web-browser-symbolic").tooltip_text("Open in Browser").build();
        open.connect_clicked(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_| viewer.open_in_browser()
        ));
        let save = gtk::Button::builder().icon_name("folder-download-symbolic").tooltip_text("Save to Downloads Folder").build();
        save.connect_clicked(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_| viewer.save_current()
        ));
        header.pack_end(&open);
        header.pack_end(&save);

        let carousel = adw::Carousel::builder().hexpand(true).vexpand(true).allow_scroll_wheel(false).build();
        carousel.connect_page_changed(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_, index| viewer.on_page_changed(index)
        ));

        let arrow = |icon: &str, tip: &str, halign: gtk::Align| {
            gtk::Button::builder()
                .icon_name(icon)
                .tooltip_text(tip)
                .halign(halign)
                .valign(gtk::Align::Center)
                .margin_start(12)
                .margin_end(12)
                .css_classes(["osd", "circular", "viewer-arrow"])
                .build()
        };
        let prev = arrow("go-previous-symbolic", "Previous (Left)", gtk::Align::Start);
        let next = arrow("go-next-symbolic", "Next (Right)", gtk::Align::End);
        prev.connect_clicked(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_| viewer.step(-1)
        ));
        next.connect_clicked(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_| viewer.step(1)
        ));
        let overlay = gtk::Overlay::builder().child(&carousel).build();
        overlay.add_overlay(&prev);
        overlay.add_overlay(&next);

        // Filmstrip.
        let strip_model = gio::ListStore::new::<glib::BoxedAnyObject>();
        let strip_selection = gtk::SingleSelection::builder().model(&strip_model).autoselect(false).build();
        strip_selection.connect_selected_notify(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |selection| {
                let index = selection.selected();
                if index != gtk::INVALID_LIST_POSITION && index != viewer.imp().index.get() {
                    viewer.go_to(index);
                }
            }
        ));
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let picture = gtk::Picture::builder().content_fit(gtk::ContentFit::Cover).can_shrink(true).build();
            picture.set_size_request(56, 56);
            let frame = gtk::Box::builder()
                .css_classes(["thumbnail-frame", "filmstrip-thumb"])
                .overflow(gtk::Overflow::Hidden)
                .width_request(56)
                .height_request(56)
                .build();
            frame.append(&picture);
            item.downcast_ref::<gtk::ListItem>().unwrap().set_child(Some(&frame));
        });
        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let (Some(frame), Some(file)) = (
                item.child().and_downcast::<gtk::Box>(),
                item.item().and_downcast::<glib::BoxedAnyObject>().map(|o| o.borrow::<PostFile>().clone()),
            ) else {
                return;
            };
            let Some(picture) = frame.first_child().and_downcast::<gtk::Picture>() else { return };
            picture.set_paintable(gdk::Paintable::NONE);
            frame.set_tooltip_text(Some(&file.name));
            let url = file.thumb_url.clone();
            glib::spawn_future_local(async move {
                if let Some(texture) = thumbnails::load(&url).await {
                    picture.set_paintable(Some(&texture));
                }
            });
        });
        let strip = gtk::ListView::builder()
            .orientation(gtk::Orientation::Horizontal)
            .model(&strip_selection)
            .factory(&factory)
            .css_classes(["filmstrip"])
            .build();
        let strip_scroll = gtk::ScrolledWindow::builder()
            .child(&strip)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .height_request(76)
            .build();
        let strip_bar = adw::Clamp::builder().maximum_size(1200).child(&strip_scroll).build();

        let toolbar = adw::ToolbarView::builder().top_bar_style(adw::ToolbarStyle::Raised).build();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&overlay));
        toolbar.add_bottom_bar(&strip_bar);
        self.set_child(Some(&toolbar));

        // Esc closes; Left/Right move (unless an image is zoomed, where
        // they'd fight with panning).
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| match key {
                gdk::Key::Escape => {
                    viewer.close();
                    glib::Propagation::Stop
                }
                gdk::Key::Left if !viewer.imp().zoomed.get() => {
                    viewer.step(-1);
                    glib::Propagation::Stop
                }
                gdk::Key::Right if !viewer.imp().zoomed.get() => {
                    viewer.step(1);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        ));
        self.add_controller(keys);

        let _ = imp.title.set(title);
        let _ = imp.carousel.set(carousel);
        let _ = imp.prev.set(prev);
        let _ = imp.next.set(next);
        let _ = imp.strip_model.set(strip_model);
        let _ = imp.strip_selection.set(strip_selection);
        let _ = imp.strip.set(strip);
        let _ = imp.strip_bar.set(strip_bar.upcast());
    }

    // ------------------------------------------------------------------
    // Opening and moving
    // ------------------------------------------------------------------

    /// Shows `files`, starting at `index`.
    pub fn open(&self, files: Vec<PostFile>, index: usize, context: ViewerContext) {
        let imp = self.imp();
        self.clear();
        let index = index.min(files.len().saturating_sub(1)) as u32;
        let carousel = imp.carousel.get().unwrap();

        let mut slots = Vec::with_capacity(files.len());
        for _ in &files {
            let slot = gtk::Box::builder().hexpand(true).vexpand(true).build();
            carousel.append(&slot);
            slots.push(slot);
        }
        let items: Vec<glib::BoxedAnyObject> = files.iter().cloned().map(glib::BoxedAnyObject::new).collect();
        imp.strip_model.get().unwrap().splice(0, 0, &items);
        imp.strip_bar.get().unwrap().set_visible(files.len() > 1);

        imp.slots.replace(slots);
        imp.files.replace(Rc::new(files));
        imp.context.replace(Some(context));
        imp.index.set(index);

        self.set_visible(true);
        self.grab_focus();
        // The carousel only knows its page size once it's allocated.
        glib::idle_add_local_once(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move || {
                let imp = viewer.imp();
                let carousel = imp.carousel.get().unwrap();
                if let Some(slot) = imp.slots.borrow().get(index as usize) {
                    carousel.scroll_to(slot, false);
                }
                viewer.on_page_changed(index);
            }
        ));
        self.on_page_changed(index);
    }

    pub fn close(&self) {
        self.clear();
        self.set_visible(false);
        if let Some(window) = self.imp().window.upgrade() {
            window.media_viewer_closed();
        }
    }

    /// Drops every slot (stopping any video).
    fn clear(&self) {
        let imp = self.imp();
        let carousel = imp.carousel.get().unwrap();
        for slot in imp.slots.take() {
            unload(&slot);
            carousel.remove(&slot);
        }
        imp.strip_model.get().unwrap().remove_all();
        imp.zoomed.set(false);
        carousel.set_interactive(true);
    }

    fn step(&self, delta: i32) {
        let target = self.imp().index.get() as i64 + i64::from(delta);
        if target >= 0 && (target as usize) < self.imp().files.borrow().len() {
            self.go_to(target as u32);
        }
    }

    fn go_to(&self, index: u32) {
        let imp = self.imp();
        if let Some(slot) = imp.slots.borrow().get(index as usize) {
            imp.carousel.get().unwrap().scroll_to(slot, true);
        }
    }

    fn on_page_changed(&self, index: u32) {
        let imp = self.imp();
        let files = imp.files.borrow().clone();
        let Some(file) = files.get(index as usize) else { return };
        let previous = imp.index.replace(index);

        // Leaving a page: reset its zoom, pause its video.
        if previous != index {
            if let Some(slot) = imp.slots.borrow().get(previous as usize) {
                pause(slot);
                reset_zoom(slot);
            }
        }
        imp.zoomed.set(false);
        imp.carousel.get().unwrap().set_interactive(true);

        // Load this file and its neighbours; free the rest.
        let slots = imp.slots.borrow().clone();
        for (i, slot) in slots.iter().enumerate() {
            let distance = (i as i64 - index as i64).unsigned_abs();
            if distance <= 1 {
                if slot.first_child().is_none() {
                    self.fill(slot, &files[i], i as u32 == index);
                }
            } else {
                unload(slot);
            }
        }
        if let Some(slot) = slots.get(index as usize) {
            play(slot);
        }

        // Header, arrows, filmstrip.
        let total = files.len();
        let mut details = vec![format!("{} of {total}", index + 1), file.ext.trim_start_matches('.').to_uppercase()];
        if file.width > 0 {
            details.push(format!("{}×{}", file.width, file.height));
        }
        let title = imp.title.get().unwrap();
        title.set_title(&file.name);
        title.set_subtitle(&details.join(" · "));
        imp.prev.get().unwrap().set_visible(index > 0);
        imp.next.get().unwrap().set_visible((index as usize) + 1 < total);
        let selection = imp.strip_selection.get().unwrap();
        if selection.selected() != index {
            selection.set_selected(index);
        }
        imp.strip.get().unwrap().scroll_to(index, gtk::ListScrollFlags::NONE, None);
    }

    // ------------------------------------------------------------------
    // Slots
    // ------------------------------------------------------------------

    fn fill(&self, slot: &gtk::Box, file: &PostFile, current: bool) {
        let ext = file.ext.to_ascii_lowercase();
        if file.is_video {
            self.fill_video(slot, file, current);
        } else if ext == ".gif" {
            self.fill_animation(slot, file, current);
        } else if matches!(ext.as_str(), ".jpg" | ".jpeg" | ".png" | ".webp" | ".bmp" | ".avif" | ".jxl") {
            self.fill_image(slot, file);
        } else {
            // PDFs, SWFs, …: nothing to show here.
            self.show_error(slot, file, "Channex can't show this kind of file. You can open it in your browser or save it.", false);
        }
    }

    /// Thumbnail + spinner while the full image downloads.
    fn fill_image(&self, slot: &gtk::Box, file: &PostFile) {
        let preview = gtk::Picture::builder().content_fit(gtk::ContentFit::Contain).can_shrink(true).hexpand(true).vexpand(true).build();
        let spinner = adw::Spinner::builder().width_request(32).height_request(32).halign(gtk::Align::Center).valign(gtk::Align::Center).build();
        let loading = gtk::Overlay::builder().child(&preview).build();
        loading.add_overlay(&spinner);
        slot.append(&loading);

        let thumb_url = file.thumb_url.clone();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            preview,
            async move {
                if let Some(texture) = thumbnails::load(&thumb_url).await {
                    preview.set_paintable(Some(&texture));
                }
            }
        ));

        let client = state().http();
        let url = file.url.clone();
        let file = file.clone();
        let viewer = self.downgrade();
        let slot_weak = slot.downgrade();
        glib::spawn_future_local(async move {
            let bytes = runtime::spawn(async move { client.get_bytes(&url).await }).await;
            let (Some(viewer), Some(slot)) = (viewer.upgrade(), slot_weak.upgrade()) else { return };
            // The slot may have been unloaded (or refilled) meanwhile.
            if slot.first_child().as_ref() != Some(loading.upcast_ref()) {
                return;
            }
            let texture = bytes
                .map_err(|e| e.to_string())
                .and_then(|b| gdk::Texture::from_bytes(&glib::Bytes::from_owned(b)).map_err(|_| "The image couldn't be decoded".to_owned()));
            unload(&slot);
            match texture {
                Ok(texture) => {
                    let image = ZoomableImage::new(&texture);
                    image.connect_zoom_changed(glib::clone!(
                        #[weak]
                        viewer,
                        move |zoomed| {
                            viewer.imp().zoomed.set(zoomed);
                            viewer.imp().carousel.get().unwrap().set_interactive(!zoomed);
                        }
                    ));
                    slot.append(&image);
                }
                Err(message) => viewer.show_error(&slot, &file, &message, true),
            }
        });
    }

    /// GIFs play as a looping, silent media stream with no controls.
    fn fill_animation(&self, slot: &gtk::Box, file: &PostFile, current: bool) {
        let media = gtk::MediaFile::for_file(&gio::File::for_uri(&file.url));
        media.set_loop(true);
        media.set_muted(true);
        let picture = gtk::Picture::builder()
            .paintable(&media)
            .content_fit(gtk::ContentFit::Contain)
            .can_shrink(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        slot.append(&picture);
        self.watch_errors(slot, &media, file);
        if current {
            media.play();
        }
    }

    fn fill_video(&self, slot: &gtk::Box, file: &PostFile, current: bool) {
        let media = gtk::MediaFile::for_file(&gio::File::for_uri(&file.url));
        media.set_loop(true);
        media.set_muted(state().data().settings.mute_webms_by_default);
        let video = gtk::Video::builder().media_stream(&media).hexpand(true).vexpand(true).build();
        slot.append(&video);
        self.watch_errors(slot, &media, file);
        if current {
            media.play();
        }
    }

    fn watch_errors(&self, slot: &gtk::Box, media: &gtk::MediaFile, file: &PostFile) {
        let file = file.clone();
        media.connect_error_notify(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            #[weak]
            slot,
            move |media| {
                if let Some(err) = media.error() {
                    unload(&slot);
                    viewer.show_error(&slot, &file, &format!("It couldn't be played: {}", err.message()), true);
                }
            }
        ));
    }

    fn show_error(&self, slot: &gtk::Box, file: &PostFile, message: &str, retry: bool) {
        let buttons = gtk::Box::builder().spacing(12).halign(gtk::Align::Center).build();
        if retry {
            let again = gtk::Button::builder().label("Try Again").css_classes(["pill"]).build();
            let file_again = file.clone();
            again.connect_clicked(glib::clone!(
                #[weak(rename_to = viewer)]
                self,
                #[weak]
                slot,
                move |_| {
                    unload(&slot);
                    viewer.fill(&slot, &file_again, true);
                }
            ));
            buttons.append(&again);
        }
        let open = gtk::Button::builder().label("Open in Browser").css_classes(["pill", "suggested-action"]).build();
        open.connect_clicked(glib::clone!(
            #[weak(rename_to = viewer)]
            self,
            move |_| viewer.open_in_browser()
        ));
        buttons.append(&open);
        let status = adw::StatusPage::builder()
            .icon_name("dialog-warning-symbolic")
            .title("Couldn't Show This File")
            .description(message)
            .child(&buttons)
            .hexpand(true)
            .build();
        slot.append(&status);
    }

    // ------------------------------------------------------------------
    // Actions
    // ------------------------------------------------------------------

    /// Debug builds: zoom the current image as if its centre were clicked.
    #[cfg(debug_assertions)]
    pub fn dev_toggle_zoom(&self) {
        let imp = self.imp();
        let slot = imp.slots.borrow().get(imp.index.get() as usize).cloned();
        if let Some(image) = slot.and_then(|s| s.first_child()).and_downcast::<ZoomableImage>() {
            image.dev_toggle_zoom();
        }
    }

    fn current_file(&self) -> Option<PostFile> {
        let imp = self.imp();
        imp.files.borrow().get(imp.index.get() as usize).cloned()
    }

    fn open_in_browser(&self) {
        let (Some(file), Some(window)) = (self.current_file(), self.imp().window.upgrade()) else { return };
        gtk::UriLauncher::new(&file.url).launch(Some(&window), gio::Cancellable::NONE, |_| {});
    }

    fn save_current(&self) {
        let imp = self.imp();
        let (Some(file), Some(context)) = (self.current_file(), imp.context.borrow().clone()) else { return };
        downloads().start_job(&context.save_label, context.save_dir, vec![(file.url, file.name)]);
    }
}

// Helpers over a slot's (single) child.

fn unload(slot: &gtk::Box) {
    pause(slot);
    while let Some(child) = slot.first_child() {
        slot.remove(&child);
    }
}

fn media_of(slot: &gtk::Box) -> Option<gtk::MediaStream> {
    let child = slot.first_child()?;
    if let Some(video) = child.downcast_ref::<gtk::Video>() {
        return video.media_stream();
    }
    child.downcast_ref::<gtk::Picture>()?.paintable().and_downcast::<gtk::MediaStream>()
}

fn pause(slot: &gtk::Box) {
    if let Some(media) = media_of(slot) {
        media.pause();
    }
}

fn play(slot: &gtk::Box) {
    if let Some(media) = media_of(slot) {
        media.play();
    }
}

fn reset_zoom(slot: &gtk::Box) {
    if let Some(image) = slot.first_child().and_downcast::<ZoomableImage>() {
        image.reset();
    }
}

//! "Download Files": every file in a thread as a toggleable thumbnail
//! (all on by default), so some can be left out before downloading.

use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use channex::data::model::PostFile;
use gtk::glib;

use crate::downloads::downloads;
use crate::thumbnails;
use crate::window::ChannexWindow;

pub fn show_download_files(
    window: &ChannexWindow,
    label: &str,
    dir: PathBuf,
    files: Vec<PostFile>,
) {
    let files = Rc::new(files);
    let flow = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .homogeneous(true)
        .min_children_per_line(3)
        .max_children_per_line(8)
        .column_spacing(8)
        .row_spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .valign(gtk::Align::Start)
        .build();

    let download = gtk::Button::builder()
        .css_classes(["suggested-action"])
        .build();
    let mut toggles: Vec<gtk::ToggleButton> = Vec::new();
    for file in files.iter() {
        let toggle = file_tile(file);
        flow.append(&toggle);
        toggles.push(toggle);
    }
    let toggles = Rc::new(toggles);

    let update = glib::clone!(
        #[strong]
        toggles,
        #[weak]
        download,
        move || {
            let n = toggles.iter().filter(|t| t.is_active()).count();
            download.set_label(&match n {
                0 => "Download".to_owned(),
                1 => "Download 1 File".to_owned(),
                n => format!("Download {n} Files"),
            });
            download.set_sensitive(n > 0);
        }
    );
    update();
    for toggle in toggles.iter() {
        let update = update.clone();
        toggle.connect_toggled(move |_| update());
    }

    let select_all = gtk::Button::builder()
        .label("Select All")
        .css_classes(["flat"])
        .build();
    let select_none = gtk::Button::builder()
        .label("Select None")
        .css_classes(["flat"])
        .build();
    for (button, value) in [(&select_all, true), (&select_none, false)] {
        let toggles = toggles.clone();
        button.connect_clicked(move |_| toggles.iter().for_each(|t| t.set_active(value)));
    }
    let destination = gtk::Label::builder()
        .label(format!("Saving to {}", dir.display()))
        .ellipsize(gtk::pango::EllipsizeMode::Start)
        .hexpand(true)
        .xalign(1.0)
        .css_classes(["dim-label", "caption"])
        .build();
    let bar = gtk::Box::builder()
        .spacing(6)
        .margin_start(6)
        .margin_end(12)
        .margin_top(6)
        .margin_bottom(6)
        .build();
    bar.append(&select_all);
    bar.append(&select_none);
    bar.append(&destination);

    let cancel = gtk::Button::builder().label("Cancel").build();
    let header = adw::HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();
    header.pack_start(&cancel);
    header.pack_end(&download);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.add_top_bar(&bar);
    toolbar.set_content(Some(
        &gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&flow)
            .vexpand(true)
            .build(),
    ));

    let dialog = adw::Dialog::builder()
        .title("Download Files")
        .content_width(760)
        .content_height(620)
        .child(&toolbar)
        .build();
    cancel.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));
    let label = label.to_owned();
    download.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        #[weak]
        window,
        move |_| {
            let chosen: Vec<(String, String)> = files
                .iter()
                .zip(toggles.iter())
                .filter(|(_, t)| t.is_active())
                .map(|(f, _)| (f.url.clone(), f.name.clone()))
                .collect();
            let count = chosen.len();
            downloads().start_job(&label, dir.clone(), chosen);
            dialog.close();
            let toast = adw::Toast::builder()
                .title(if count == 1 {
                    "Downloading 1 file".to_owned()
                } else {
                    format!("Downloading {count} files")
                })
                .button_label("Show")
                .action_name("win.show-downloads")
                .build();
            window.add_toast(toast);
        }
    ));
    dialog.present(Some(window));
}

fn file_tile(file: &PostFile) -> gtk::ToggleButton {
    let picture = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Cover)
        .can_shrink(true)
        .build();
    picture.set_size_request(120, 120);
    let clamp = |orientation, child: &gtk::Widget| {
        adw::Clamp::builder()
            .orientation(orientation)
            .maximum_size(120)
            .tightening_threshold(120)
            .unit(adw::LengthUnit::Px)
            .child(child)
            .build()
    };
    let square = clamp(
        gtk::Orientation::Vertical,
        clamp(gtk::Orientation::Horizontal, picture.upcast_ref()).upcast_ref(),
    );
    let frame = gtk::Box::builder()
        .css_classes(["thumbnail-frame"])
        .overflow(gtk::Overflow::Hidden)
        .build();
    frame.append(&square);
    let check = gtk::Image::builder()
        .icon_name("object-select-symbolic")
        .css_classes(["download-check"])
        .halign(gtk::Align::End)
        .valign(gtk::Align::Start)
        .margin_top(6)
        .margin_end(6)
        .can_target(false)
        .build();
    let overlay = gtk::Overlay::builder().child(&frame).build();
    overlay.add_overlay(&check);
    if file.is_video {
        overlay.add_overlay(
            &gtk::Image::builder()
                .icon_name("media-playback-start-symbolic")
                .css_classes(["osd", "thumbnail-badge"])
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .can_target(false)
                .build(),
        );
    }
    let toggle = gtk::ToggleButton::builder()
        .child(&overlay)
        .active(true)
        .tooltip_text(&file.name)
        .css_classes(["flat", "download-tile"])
        .build();

    let url = file.thumb_url.clone();
    glib::spawn_future_local(glib::clone!(
        #[weak]
        picture,
        async move {
            if let Some(texture) = thumbnails::load(&url).await {
                picture.set_paintable(Some(&texture));
            }
        }
    ));
    toggle
}

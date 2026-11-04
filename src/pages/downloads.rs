//! This session's download jobs, as cards: status, "N of M files", a
//! progress bar, Retry Failed / Open Folder / Remove, and the job's
//! files on demand. Clearing the list never deletes files.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::glib;

use crate::downloads::{FileStatus, Job, downloads};
use crate::window::ChannexWindow;

/// Most file rows shown per job (big threads can have hundreds).
const MAX_FILE_ROWS: usize = 400;

pub fn downloads_page(window: &ChannexWindow) -> adw::NavigationPage {
    let cards = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    let clamp = adw::Clamp::builder()
        .maximum_size(820)
        .child(&cards)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build();
    let empty = adw::StatusPage::builder()
        .icon_name("folder-download-symbolic")
        .title("No Downloads")
        .description("Use Download Files in a thread to save its images and videos. Clearing this list never deletes files.")
        .build();
    let stack = gtk::Stack::new();
    stack.add_named(&scrolled, Some("list"));
    stack.add_named(&empty, Some("empty"));

    let clear = gtk::Button::builder()
        .label("Clear History")
        .tooltip_text("Remove finished downloads from this list. Files are kept.")
        .build();
    clear.connect_clicked(|_| downloads().clear_finished());

    let header = adw::HeaderBar::new();
    header.pack_start(&window.sidebar_toggle());
    header.pack_end(&clear);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&stack));
    let page = adw::NavigationPage::builder()
        .title("Downloads")
        .tag("downloads")
        .child(&toolbar)
        .build();

    let expanded: Rc<RefCell<HashSet<u64>>> = Rc::default();
    let rebuild = Rc::new(glib::clone!(
        #[weak]
        cards,
        #[weak]
        stack,
        #[weak]
        clear,
        #[strong]
        expanded,
        move || {
            while let Some(child) = cards.first_child() {
                cards.remove(&child);
            }
            let jobs = downloads().jobs();
            stack.set_visible_child_name(if jobs.is_empty() { "empty" } else { "list" });
            clear.set_visible(jobs.iter().any(Job::finished));
            for job in jobs.iter().rev() {
                cards.append(&job_card(job, &expanded));
            }
        }
    ));
    rebuild();

    // Many files can finish in a burst; redraw at most every 150 ms.
    let pending = Rc::new(Cell::new(false));
    downloads().connect_closure(
        "changed",
        false,
        glib::closure_local!(
            #[watch]
            cards,
            move |_: crate::downloads::DownloadManager, _id: u64| {
                let _ = &cards;
                if pending.replace(true) {
                    return;
                }
                let (pending, rebuild) = (pending.clone(), rebuild.clone());
                glib::timeout_add_local_once(Duration::from_millis(150), move || {
                    pending.set(false);
                    rebuild();
                });
            }
        ),
    );
    page
}

fn job_card(job: &Job, expanded: &Rc<RefCell<HashSet<u64>>>) -> gtk::Box {
    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .css_classes(["card", "download-card"])
        .build();

    let (icon, icon_class) = if !job.finished() {
        ("folder-download-symbolic", "accent")
    } else if job.failed() > 0 {
        ("dialog-warning-symbolic", "warning")
    } else {
        ("object-select-symbolic", "success")
    };
    let top = gtk::Box::builder().spacing(14).build();
    top.append(
        &gtk::Image::builder()
            .icon_name(icon)
            .pixel_size(20)
            .css_classes(["download-status", icon_class])
            .valign(gtk::Align::Center)
            .build(),
    );

    let text = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .build();
    text.append(
        &gtk::Label::builder()
            .label(&job.label)
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .css_classes(["heading"])
            .build(),
    );
    let state = if !job.finished() {
        "Downloading"
    } else if job.failed() > 0 {
        "Finished with errors"
    } else {
        "Complete"
    };
    let files_word = if job.total() == 1 { "file" } else { "files" };
    let mut subtitle = format!("{state} · {} of {} {files_word}", job.done(), job.total());
    if job.failed() > 0 {
        subtitle.push_str(&format!(" · {} failed", job.failed()));
    }
    text.append(
        &gtk::Label::builder()
            .label(subtitle)
            .xalign(0.0)
            .css_classes(["dim-label", "caption"])
            .build(),
    );
    top.append(&text);

    let id = job.id;
    if job.finished() && job.failed() > 0 {
        let retry = gtk::Button::builder()
            .label("Retry Failed")
            .valign(gtk::Align::Center)
            .build();
        retry.connect_clicked(move |_| downloads().retry_failed(id));
        top.append(&retry);
    }
    let open = gtk::Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text("Open Folder")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .action_name("win.open-folder")
        .action_target(&job.dir.to_string_lossy().to_variant())
        .build();
    top.append(&open);
    if job.finished() {
        let remove = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text("Remove from List (files are kept)")
            .valign(gtk::Align::Center)
            .css_classes(["flat"])
            .build();
        remove.connect_clicked(move |_| downloads().remove_job(id));
        top.append(&remove);
    }
    card.append(&top);

    if !job.finished() {
        card.append(&gtk::ProgressBar::builder().fraction(job.progress()).build());
    }

    let files = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    for file in job.files.iter().take(MAX_FILE_ROWS) {
        let (status_icon, status_text) = match &file.status {
            FileStatus::Pending => ("content-loading-symbolic", "Waiting"),
            FileStatus::Done => ("object-select-symbolic", "Saved"),
            FileStatus::Failed(err) => ("dialog-warning-symbolic", err.as_str()),
        };
        let row = adw::ActionRow::builder()
            .title(&file.name)
            .subtitle(status_text)
            .use_markup(false)
            .build();
        row.add_prefix(&gtk::Image::from_icon_name(status_icon));
        files.append(&row);
    }
    if job.files.len() > MAX_FILE_ROWS {
        files.append(
            &adw::ActionRow::builder()
                .title(format!("…and {} more", job.files.len() - MAX_FILE_ROWS))
                .sensitive(false)
                .build(),
        );
    }
    let expander = gtk::Expander::builder()
        .label(format!(
            "Show {} {}",
            job.total(),
            if job.total() == 1 { "File" } else { "Files" }
        ))
        .child(&files)
        .expanded(expanded.borrow().contains(&id))
        .build();
    let expanded = expanded.clone();
    expander.connect_expanded_notify(move |e| {
        if e.is_expanded() {
            expanded.borrow_mut().insert(id);
        } else {
            expanded.borrow_mut().remove(&id);
        }
    });
    card.append(&expander);
    card
}

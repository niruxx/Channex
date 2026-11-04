//! Bookmarked threads as a boxed list: thumbnail, subject, where it's
//! from, a "N new" pill for unseen replies, and a remove button.

use adw::prelude::*;
use channex::data::store::Bookmark;
use gtk::glib;

use crate::state::state;
use crate::thumbnails;
use crate::window::ChannexWindow;

pub fn bookmarks_page(window: &ChannexWindow) -> adw::NavigationPage {
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .valign(gtk::Align::Start)
        .build();
    let clamp = adw::Clamp::builder()
        .maximum_size(820)
        .child(&list)
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
        .icon_name("user-bookmarks-symbolic")
        .title("No Bookmarks")
        .description("Bookmark threads from a catalog to keep track of their new replies here.")
        .build();
    let stack = gtk::Stack::new();
    stack.add_named(&scrolled, Some("list"));
    stack.add_named(&empty, Some("empty"));

    let page = window.simple_page("Bookmarks", &stack);
    page.set_tag(Some("bookmarks"));

    let rebuild = glib::clone!(
        #[weak]
        list,
        #[weak]
        stack,
        #[weak]
        window,
        move || {
            list.remove_all();
            let bookmarks = state().data().bookmarks.clone();
            stack.set_visible_child_name(if bookmarks.is_empty() {
                "empty"
            } else {
                "list"
            });
            for bookmark in bookmarks.iter().rev() {
                list.append(&bookmark_row(&window, bookmark));
            }
        }
    );
    rebuild();
    state().connect_closure(
        "bookmarks-changed",
        false,
        glib::closure_local!(
            #[watch]
            list,
            move |_: crate::state::State| {
                let _ = &list;
                rebuild()
            }
        ),
    );
    page
}

fn bookmark_row(window: &ChannexWindow, bookmark: &Bookmark) -> adw::ActionRow {
    let site_name = state()
        .site(&bookmark.site_id)
        .map(|s| s.name)
        .unwrap_or_else(|| bookmark.site_id.clone());
    let title = if bookmark.subject.is_empty() {
        format!("Thread {}", bookmark.thread_id)
    } else {
        bookmark.subject.clone()
    };
    let row = adw::ActionRow::builder()
        .title(&title)
        .subtitle(format!(
            "{site_name} · /{}/ · No.{}",
            bookmark.board_code, bookmark.thread_id
        ))
        .use_markup(false)
        .activatable(true)
        .build();

    // Thumbnail (or a placeholder glyph).
    let picture = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Cover)
        .can_shrink(true)
        .build();
    picture.set_size_request(40, 40);
    let frame = gtk::Box::builder()
        .css_classes(["thumbnail-frame"])
        .overflow(gtk::Overflow::Hidden)
        .valign(gtk::Align::Center)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    frame.append(&picture);
    let fallback = gtk::Image::builder()
        .icon_name("user-bookmarks-symbolic")
        .css_classes(["dim-label"])
        .can_target(false)
        .build();
    let overlay = gtk::Overlay::builder().child(&frame).build();
    overlay.add_overlay(&fallback);
    row.add_prefix(&overlay);
    if !bookmark.thumb_url.is_empty() {
        let url = bookmark.thumb_url.clone();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            picture,
            #[weak]
            fallback,
            async move {
                if let Some(texture) = thumbnails::load(&url).await {
                    picture.set_paintable(Some(&texture));
                    fallback.set_visible(false);
                }
            }
        ));
    }

    let new_replies = bookmark.new_replies();
    if new_replies > 0 {
        row.add_suffix(
            &gtk::Label::builder()
                .label(format!("{new_replies} new"))
                .css_classes(["new-badge"])
                .valign(gtk::Align::Center)
                .build(),
        );
    }

    let remove = gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Remove Bookmark")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    let (site_id, board, thread_id) = (
        bookmark.site_id.clone(),
        bookmark.board_code.clone(),
        bookmark.thread_id.clone(),
    );
    remove.connect_clicked(glib::clone!(
        #[strong]
        site_id,
        #[strong]
        board,
        #[strong]
        thread_id,
        move |_| {
            state().update_bookmarks(|d| d.remove_bookmark(&site_id, &board, &thread_id));
        }
    ));
    row.add_suffix(&remove);
    row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));

    row.connect_activated(glib::clone!(
        #[weak]
        window,
        move |_| window.open_thread(&site_id, &board, &thread_id)
    ));
    row
}

//! Replying to a thread.
//!
//! LynxChan sites post from here: name, options, comment, and the
//! site's captcha. Everywhere else the reply is written here and then
//! handed to the site's own page (copied to the clipboard, thread opened
//! in the browser), since those sites' captchas only work there.
//! Unsent text is kept as a per-thread draft.

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use channex::data::adapters::thread_web_url;
use channex::data::model::{PostEngine, Site};
use channex::data::posting::{Reply, captcha_url, post_lynxchan_reply, session_password};
use gtk::{gdk, gio, glib};

use crate::runtime;
use crate::state::state;
use crate::window::ChannexWindow;

/// Where a reply goes, and what to do once it's posted.
pub struct ReplyTarget {
    pub site: Rc<Site>,
    pub board: String,
    pub thread_id: String,
    /// Kept between openings of the dialog for this thread.
    pub draft: Rc<RefCell<String>>,
    /// Called with the new post's id after a successful in-app post.
    pub on_posted: Box<dyn Fn(String)>,
}

thread_local! {
    static PASSWORD: OnceCell<String> = const { OnceCell::new() };
}

/// One deletion password per session, like the Qt version.
fn password() -> String {
    PASSWORD.with(|p| {
        p.get_or_init(|| session_password(glib::real_time() as u64))
            .clone()
    })
}

/// Opens the reply dialog. `quote` (e.g. ">>123") is added to the draft.
pub fn show_reply(window: &ChannexWindow, target: ReplyTarget, quote: Option<String>) {
    if let Some(quote) = quote {
        let mut draft = target.draft.borrow_mut();
        if !draft.is_empty() && !draft.ends_with('\n') {
            draft.push('\n');
        }
        draft.push_str(&quote);
        draft.push('\n');
    }
    match target.site.post_engine {
        PostEngine::Lynxchan => lynxchan_dialog(window, target),
        PostEngine::External => external_dialog(window, target),
    }
}

/// A multi-line comment field in a card.
fn comment_view(text: &str) -> (gtk::TextView, gtk::Widget) {
    let view = gtk::TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .top_margin(10)
        .bottom_margin(10)
        .left_margin(12)
        .right_margin(12)
        .accepts_tab(false)
        .css_classes(["reply-text"])
        .build();
    view.buffer().set_text(text);
    view.update_property(&[gtk::accessible::Property::Label("Comment")]);
    let scrolled = gtk::ScrolledWindow::builder()
        .child(&view)
        .min_content_height(180)
        .max_content_height(360)
        .propagate_natural_height(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .css_classes(["card"])
        .overflow(gtk::Overflow::Hidden)
        .build();
    (view, scrolled.upcast())
}

fn text_of(view: &gtk::TextView) -> String {
    let buffer = view.buffer();
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string()
}

fn dialog_shell(
    title: &str,
    primary: &gtk::Button,
    content: &impl IsA<gtk::Widget>,
) -> adw::Dialog {
    let cancel = gtk::Button::builder().label("Cancel").build();
    let header = adw::HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();
    header.pack_start(&cancel);
    header.pack_end(primary);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(content));
    let dialog = adw::Dialog::builder()
        .title(title)
        .content_width(560)
        .child(&toolbar)
        .build();
    cancel.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));
    dialog
}

// ----------------------------------------------------------------------
// Captcha-gated sites: write here, finish on the site.
// ----------------------------------------------------------------------

fn external_dialog(window: &ChannexWindow, target: ReplyTarget) {
    let (view, comment) = comment_view(&target.draft.borrow());
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::builder()
        .description(
            "This site checks replies with its own captcha, which only works on its page. \
             Write your reply here, then Continue on Site: it's copied to your clipboard and \
             the thread opens in your browser, ready to paste.",
        )
        .build();
    group.add(&comment);
    page.add(&group);

    if target.site.id == "4chan" {
        let pass_group = adw::PreferencesGroup::new();
        let pass = adw::ButtonRow::builder()
            .title("Sign In with 4chan Pass")
            .end_icon_name("adw-external-link-symbolic")
            .build();
        pass.connect_activated(glib::clone!(
            #[weak]
            window,
            move |_| {
                gtk::UriLauncher::new("https://sys.4chan.org/auth").launch(
                    Some(&window),
                    gio::Cancellable::NONE,
                    |_| {},
                );
            }
        ));
        pass_group.add(&pass);
        page.add(&pass_group);
    }

    let continue_button = gtk::Button::builder()
        .label("Continue on Site")
        .css_classes(["suggested-action"])
        .build();
    let dialog = dialog_shell("Reply", &continue_button, &page);
    let url = thread_web_url(&target.site, &target.board, &target.thread_id);
    let draft = target.draft.clone();
    continue_button.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        #[weak]
        window,
        #[weak]
        view,
        move |_| {
            let text = text_of(&view);
            if !text.trim().is_empty() {
                window.clipboard().set_text(&text);
            }
            gtk::UriLauncher::new(&url).launch(Some(&window), gio::Cancellable::NONE, |_| {});
            // Handed off: the draft has done its job.
            view.buffer().set_text("");
            dialog.close();
            window.toast(if text.trim().is_empty() {
                "Opened the thread in your browser"
            } else {
                "Reply copied - paste it on the site, then refresh here"
            });
        }
    ));
    dialog.connect_closed(glib::clone!(
        #[weak]
        view,
        move |_| {
            draft.replace(text_of(&view));
        }
    ));
    dialog.present(Some(window));
    view.grab_focus();
}

// ----------------------------------------------------------------------
// LynxChan: post from here.
// ----------------------------------------------------------------------

fn lynxchan_dialog(window: &ChannexWindow, target: ReplyTarget) {
    let target = Rc::new(target);
    let name = adw::EntryRow::builder().title("Name").build();
    let options = adw::EntryRow::builder()
        .title("Options")
        .tooltip_text("For example, \"sage\"")
        .build();
    let details = adw::PreferencesGroup::new();
    details.add(&name);
    details.add(&options);

    let (view, comment) = comment_view(&target.draft.borrow());
    let comment_group = adw::PreferencesGroup::builder().title("Comment").build();
    comment_group.add(&comment);

    let captcha_picture = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Contain)
        .height_request(100)
        .build();
    let captcha_spinner = adw::Spinner::builder()
        .width_request(24)
        .height_request(24)
        .build();
    let captcha_stack = gtk::Stack::new();
    captcha_stack.add_named(&captcha_spinner, Some("loading"));
    captcha_stack.add_named(&captcha_picture, Some("image"));
    captcha_stack.add_named(
        &gtk::Label::builder()
            .label("Couldn't load the captcha")
            .css_classes(["dim-label"])
            .build(),
        Some("error"),
    );
    let captcha_card = gtk::Box::builder()
        .css_classes(["card", "captcha-card"])
        .build();
    captcha_stack.set_hexpand(true);
    captcha_card.append(&captcha_stack);
    let refresh = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("New Captcha")
        .valign(gtk::Align::Center)
        .css_classes(["flat", "circular"])
        .margin_end(6)
        .build();
    captcha_card.append(&refresh);
    let answer = adw::EntryRow::builder()
        .title("Captcha Answer")
        .activates_default(true)
        .build();
    let captcha_group = adw::PreferencesGroup::builder().title("Captcha").build();
    captcha_group.add(&captcha_card);
    let answer_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .margin_top(12)
        .build();
    answer_list.append(&answer);
    captcha_group.add(&answer_list);

    let error = gtk::Label::builder()
        .wrap(true)
        .xalign(0.0)
        .css_classes(["error"])
        .visible(false)
        .build();
    let error_group = adw::PreferencesGroup::new();
    error_group.add(&error);

    let page = adw::PreferencesPage::new();
    page.add(&details);
    page.add(&comment_group);
    page.add(&captcha_group);
    page.add(&error_group);

    let post = gtk::Button::builder()
        .label("Post")
        .css_classes(["suggested-action"])
        .sensitive(false)
        .build();
    let dialog = dialog_shell("Reply", &post, &page);
    dialog.set_default_widget(Some(&post));

    let load_captcha = Rc::new(glib::clone!(
        #[weak]
        captcha_stack,
        #[weak]
        captcha_picture,
        #[weak]
        answer,
        #[strong]
        target,
        move || {
            captcha_stack.set_visible_child_name("loading");
            answer.set_text("");
            let url = captcha_url(&target.site, &target.board, glib::real_time() as u64);
            let client = state().http();
            glib::spawn_future_local(async move {
                let bytes = runtime::spawn(async move { client.get_bytes(&url).await }).await;
                let texture = bytes
                    .ok()
                    .and_then(|b| gdk::Texture::from_bytes(&glib::Bytes::from_owned(b)).ok());
                match texture {
                    Some(texture) => {
                        captcha_picture.set_paintable(Some(&texture));
                        captcha_stack.set_visible_child_name("image");
                    }
                    None => captcha_stack.set_visible_child_name("error"),
                }
            });
        }
    ));
    load_captcha();
    let reload = load_captcha.clone();
    refresh.connect_clicked(move |_| reload());

    let update = Rc::new(glib::clone!(
        #[weak]
        view,
        #[weak]
        answer,
        #[weak]
        post,
        move || post
            .set_sensitive(!text_of(&view).trim().is_empty() && !answer.text().trim().is_empty())
    ));
    let u = update.clone();
    view.buffer().connect_changed(move |_| u());
    let u = update.clone();
    answer.connect_changed(move |_| u());

    let posted = Rc::new(std::cell::Cell::new(false));
    post.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        #[weak]
        view,
        #[weak]
        name,
        #[weak]
        options,
        #[weak]
        answer,
        #[weak]
        error,
        #[strong]
        target,
        #[strong]
        posted,
        #[strong]
        load_captcha,
        move |button| {
            let reply = Reply {
                board: target.board.clone(),
                thread_id: target.thread_id.clone(),
                message: text_of(&view),
                name: name.text().to_string(),
                options: options.text().to_string(),
                captcha: answer.text().trim().to_string(),
                password: password(),
                spoiler: false,
            };
            button.set_sensitive(false);
            button.set_label("Posting…");
            error.set_visible(false);
            let client = state().http();
            let site = (*target.site).clone();
            let button = button.clone();
            let (target, posted, load_captcha) =
                (target.clone(), posted.clone(), load_captcha.clone());
            glib::spawn_future_local(async move {
                let result =
                    runtime::spawn(
                        async move { post_lynxchan_reply(&client, &site, &reply).await },
                    )
                    .await;
                button.set_label("Post");
                match result {
                    Ok(id) => {
                        posted.set(true);
                        target.draft.replace(String::new());
                        dialog.close();
                        (target.on_posted)(id);
                    }
                    Err(message) => {
                        error.set_label(&message);
                        error.set_visible(true);
                        // A captcha only works once.
                        load_captcha();
                        button.set_sensitive(true);
                    }
                }
            });
        }
    ));

    let draft = target.draft.clone();
    dialog.connect_closed(glib::clone!(
        #[weak]
        view,
        move |_| {
            if !posted.get() {
                draft.replace(text_of(&view));
            }
        }
    ));
    dialog.present(Some(window));
    view.grab_focus();
}

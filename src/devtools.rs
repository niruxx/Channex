//! Developer helpers, compiled into debug builds only.
//!
//! `CHANNEX_SCREENSHOT=/path/out.png cargo run` renders the main window
//! to a PNG after `CHANNEX_SCREENSHOT_DELAY_MS` (default 3000) and
//! closes the window - for checking UI changes headlessly, e.g. inside
//! `dbus-run-session -- mutter --headless --wayland --virtual-monitor 1440x900`.
//!
//! `CHANNEX_DEV_PAGE` opens a screen first so it can be captured:
//! `bookmarks`, `preferences`, `preferences:sites`, `add-site`, or
//! `<what>:<site>:<board>:<thread>` with `thread`, `reply`,
//! `download-dialog`, or `download-test` (downloads six files), or
//! `media:<site>:<board>:<thread>:<file index>[:zoom]`.

use gtk::prelude::*;
use gtk::{glib, graphene};

pub fn schedule_screenshot(window: &crate::window::ChannexWindow) {
    match std::env::var("CHANNEX_DEV_PAGE").ok().as_deref() {
        Some("bookmarks") => window.open_bookmarks(),
        Some("preferences") => crate::dialogs::show_preferences(window, None),
        Some("preferences:sites") => crate::dialogs::show_preferences(window, Some("sites")),
        Some("add-site") => crate::dialogs::show_add_site(window, None),
        // Downloads the first 6 files of a thread, then shows Downloads.
        Some(page) if page.starts_with("download-test:") => {
            if let [_, site, board, id] = page.splitn(4, ':').collect::<Vec<_>>()[..] {
                let (Some(site), board, id) = (
                    crate::state::state().site(site),
                    board.to_owned(),
                    id.to_owned(),
                ) else {
                    return;
                };
                let client = crate::state::state().http();
                let window = window.clone();
                glib::spawn_future_local(async move {
                    let (s, b, i) = (site.clone(), board.clone(), id.clone());
                    let thread = crate::runtime::spawn(async move {
                        channex::data::adapters::fetch_thread(&client, &s, &b, &i).await
                    })
                    .await;
                    let Ok(thread) = thread else { return };
                    let files: Vec<(String, String)> = thread
                        .all_files()
                        .into_iter()
                        .take(6)
                        .map(|f| (f.url.clone(), f.name.clone()))
                        .collect();
                    let dir =
                        crate::downloads::base_dir().join(format!("{}-{board}-{id}", site.id));
                    crate::downloads::downloads().start_job("Test download", dir, files);
                    window.open_downloads();
                });
            }
        }
        // media:<site>:<board>:<thread>:<file index>[:zoom]
        Some(page) if page.starts_with("media:") => {
            let parts: Vec<String> = page.split(':').map(str::to_owned).collect();
            if parts.len() >= 5 {
                window.open_thread(&parts[1], &parts[2], &parts[3]);
                let index: usize = parts[4].parse().unwrap_or(0);
                let zoom = parts.get(5).is_some_and(|p| p == "zoom");
                let window = window.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(3000), move || {
                    if let Some(page) = window
                        .visible_page()
                        .and_downcast::<crate::pages::ThreadPage>()
                    {
                        page.open_media_at(index);
                    }
                    if zoom {
                        glib::timeout_add_local_once(
                            std::time::Duration::from_millis(3000),
                            move || {
                                window.dev_media_viewer().dev_toggle_zoom();
                            },
                        );
                    }
                });
            }
        }
        Some(page) if page.starts_with("reply:") => {
            if let [_, site, board, id] = page.splitn(4, ':').collect::<Vec<_>>()[..] {
                window.open_thread(site, board, id);
                let window = window.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(2500), move || {
                    if let Some(page) = window
                        .visible_page()
                        .and_downcast::<crate::pages::ThreadPage>()
                    {
                        page.reply(Some(">>12345".into()));
                    }
                });
            }
        }
        Some(page) if page.starts_with("download-dialog:") => {
            if let [_, site, board, id] = page.splitn(4, ':').collect::<Vec<_>>()[..] {
                window.open_thread(site, board, id);
                let window = window.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(2500), move || {
                    if let Some(page) = window
                        .visible_page()
                        .and_downcast::<crate::pages::ThreadPage>()
                    {
                        page.download_files(None);
                    }
                });
            }
        }
        Some(page) if page.starts_with("thread:") => {
            if let [_, site, board, id] = page.splitn(4, ':').collect::<Vec<_>>()[..] {
                window.open_thread(site, board, id);
            }
        }
        _ => {}
    }
    let Ok(path) = std::env::var("CHANNEX_SCREENSHOT") else {
        return;
    };
    let delay = std::env::var("CHANNEX_SCREENSHOT_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let window = window.clone().upcast::<gtk::Window>();
    glib::timeout_add_local_once(std::time::Duration::from_millis(delay), move || {
        let paintable = gtk::WidgetPaintable::new(Some(&window));
        let (w, h) = (window.width() as f64, window.height() as f64);
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, w, h);
        if let (Some(node), Some(renderer)) = (
            snapshot.to_node(),
            window.native().and_then(|n| n.renderer()),
        ) {
            let texture = renderer.render_texture(
                node,
                Some(&graphene::Rect::new(0.0, 0.0, w as f32, h as f32)),
            );
            if let Err(err) = texture.save_to_png(&path) {
                eprintln!("screenshot failed: {err}");
            }
        }
        // Close normally (not app.quit()) so close handlers such as
        // window-state saving run, as they would for a user.
        window.close();
    });
}

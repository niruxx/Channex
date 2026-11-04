//! Appearance: light/dark, the accent color, and the classic imageboard
//! skins, all applied by overriding libadwaita's CSS color variables
//! (`--window-bg-color`, `--accent-bg-color`, ...). Re-applied whenever
//! settings change.

use std::cell::OnceCell;

use channex::data::comment::Palette;
use gtk::gdk;

use crate::state::state;

/// GNOME's accent colors (Settings → Appearance), as (id, label, hex).
pub const ACCENTS: [(&str, &str, &str); 9] = [
    ("blue", "Blue", "#3584e4"),
    ("teal", "Teal", "#2190a4"),
    ("green", "Green", "#3a944a"),
    ("yellow", "Yellow", "#c88800"),
    ("orange", "Orange", "#ed5b00"),
    ("red", "Red", "#e62d42"),
    ("pink", "Pink", "#d56199"),
    ("purple", "Purple", "#9141ac"),
    ("slate", "Slate", "#6f8396"),
];

/// A classic 4chan-style skin.
pub struct Skin {
    pub id: &'static str,
    pub name: &'static str,
    /// Page background.
    canvas: &'static str,
    /// Lists and text areas.
    surface: &'static str,
    /// Posts, cards, sidebar.
    post: &'static str,
    /// Slightly deeper tone (selected rows).
    deep: &'static str,
    ink: &'static str,
    accent: &'static str,
}

pub const SKINS: [Skin; 5] = [
    Skin {
        id: "yotsuba",
        name: "Yotsuba",
        canvas: "#ffffee",
        surface: "#fffff2",
        post: "#f0e0d6",
        deep: "#e9d3c7",
        ink: "#000000",
        accent: "#34345c",
    },
    Skin {
        id: "yotsubab",
        name: "Yotsuba B",
        canvas: "#eef2ff",
        surface: "#f4f7ff",
        post: "#d6daf0",
        deep: "#c7cce6",
        ink: "#000000",
        accent: "#34345c",
    },
    Skin {
        id: "futaba",
        name: "Futaba",
        canvas: "#ffeef2",
        surface: "#fff5f7",
        post: "#f5d9df",
        deep: "#eec6cf",
        ink: "#000000",
        accent: "#a8375f",
    },
    Skin {
        id: "photon",
        name: "Photon",
        canvas: "#ffffff",
        surface: "#ffffff",
        post: "#f4f6f8",
        deep: "#e9edf1",
        ink: "#1a1a1a",
        accent: "#3a8fd9",
    },
    Skin {
        id: "burichan",
        name: "Burichan",
        canvas: "#eef2ff",
        surface: "#f5f7fc",
        post: "#d6daf0",
        deep: "#c3c9e3",
        ink: "#000000",
        accent: "#3355aa",
    },
];

pub fn skin(id: &str) -> Option<&'static Skin> {
    SKINS.iter().find(|s| s.id == id)
}

thread_local! {
    static PROVIDER: OnceCell<gtk::CssProvider> = const { OnceCell::new() };
}

/// Applies the saved appearance settings. Call at startup and whenever
/// settings change.
pub fn apply() {
    let (theme, accent) = {
        let st = state();
        let data = st.data();
        (
            data.settings.theme.clone(),
            data.settings.accent_color.clone(),
        )
    };
    let active_skin = skin(&theme);

    let scheme = match theme.as_str() {
        "dark" => adw::ColorScheme::ForceDark,
        "light" => adw::ColorScheme::ForceLight,
        _ if active_skin.is_some() => adw::ColorScheme::ForceLight,
        _ => adw::ColorScheme::Default,
    };
    adw::StyleManager::default().set_color_scheme(scheme);

    let mut css = String::new();
    if let Some(s) = active_skin {
        css.push_str(&format!(
            ":root {{
  --window-bg-color: {canvas}; --window-fg-color: {ink};
  --view-bg-color: {surface}; --view-fg-color: {ink};
  --headerbar-bg-color: {canvas}; --headerbar-fg-color: {ink};
  --sidebar-bg-color: {post}; --sidebar-fg-color: {ink};
  --secondary-sidebar-bg-color: {deep};
  --card-bg-color: {post}; --card-fg-color: {ink};
  --popover-bg-color: {surface}; --popover-fg-color: {ink};
  --dialog-bg-color: {canvas}; --dialog-fg-color: {ink};
  --accent-bg-color: {accent};
}}\n",
            canvas = s.canvas,
            surface = s.surface,
            post = s.post,
            deep = s.deep,
            ink = s.ink,
            accent = s.accent,
        ));
    } else if let Some(hex) = accent.as_deref().filter(|h| gdk::RGBA::parse(*h).is_ok()) {
        css.push_str(&format!(":root {{ --accent-bg-color: {hex}; }}\n"));
    }
    // Swatch for a custom (non-GNOME) accent in Preferences.
    if let Some(hex) = accent.as_deref().filter(|h| gdk::RGBA::parse(*h).is_ok()) {
        css.push_str(&format!(
            ".accent-swatch.custom {{ background-color: {hex}; }}\n"
        ));
    }

    PROVIDER.with(|cell| {
        let provider = cell.get_or_init(|| {
            let provider = gtk::CssProvider::new();
            if let Some(display) = gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
                );
            }
            provider
        });
        provider.load_from_string(&css);
    });
}

/// Colors for rendering comments (greentext, dead quotes, spoilers) in
/// the current theme.
pub fn comment_palette() -> Palette {
    let skin_active = skin(&state().data().settings.theme).is_some();
    let dark = adw::StyleManager::default().is_dark();
    if skin_active {
        // The classic imageboard greentext.
        Palette {
            greentext: "#789922".into(),
            dead_quote: "#707070".into(),
            spoiler: "#000000".into(),
        }
    } else if dark {
        Palette {
            greentext: "#8fd16b".into(),
            dead_quote: "#8a8a8f".into(),
            spoiler: "#5e5e64".into(),
        }
    } else {
        Palette {
            greentext: "#3f8a24".into(),
            dead_quote: "#77777c".into(),
            spoiler: "#2e2e33".into(),
        }
    }
}

//! Naming for downloaded files and per-thread folders.

use std::collections::HashSet;

use super::store::FolderNaming;

/// Characters Windows (and some Linux filesystems) refuse in names.
const FORBIDDEN: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// Makes `name` safe as a single path component on Linux and Windows:
/// no separators or reserved characters, no control characters, no
/// trailing dots or spaces, not empty, and at most 150 characters
/// (keeping the extension).
pub fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_end_matches(['.', ' ']).to_owned();
    let cleaned = if cleaned.is_empty() || cleaned.chars().all(|c| c == '.') {
        "file".to_owned()
    } else {
        cleaned
    };
    truncate_keep_extension(&cleaned, 150)
}

fn truncate_keep_extension(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        return name.to_owned();
    }
    let (stem, ext) = split_extension(name);
    let keep = max.saturating_sub(ext.chars().count()).max(1);
    let stem: String = stem.chars().take(keep).collect();
    format!("{}{ext}", stem.trim_end())
}

/// `photo.jpg` → (`photo`, `.jpg`); no extension → (`name`, ``).
fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(i) if i > 0 && i < name.len() - 1 => (&name[..i], &name[i..]),
        _ => (name, ""),
    }
}

/// The folder a thread's files go in, per the "Thread folder names"
/// setting: `site-board-thread`, or the thread's subject (else the start
/// of its first post) made filesystem-safe, falling back to the id form.
pub fn thread_folder_name(
    naming: FolderNaming,
    site_id: &str,
    board: &str,
    thread_id: &str,
    title_text: &str,
) -> String {
    let id_name = sanitize_file_name(&format!("{site_id}-{board}-{thread_id}"));
    if naming == FolderNaming::Id {
        return id_name;
    }
    let collapsed = title_text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = collapsed.chars().take(80).collect();
    let name = sanitize_file_name(cut.trim());
    if name == "file" { id_name } else { name }
}

/// `name`, or `name (1)`, `name (2)`, … before the extension, whichever
/// is first neither on disk (`exists`) nor already `taken` by this batch.
pub fn unique_name(name: &str, taken: &HashSet<String>, exists: impl Fn(&str) -> bool) -> String {
    let is_free =
        |candidate: &str| !taken.contains(&candidate.to_lowercase()) && !exists(candidate);
    if is_free(name) {
        return name.to_owned();
    }
    let (stem, ext) = split_extension(name);
    (1..)
        .map(|n| format!("{stem} ({n}){ext}"))
        .find(|candidate| is_free(candidate))
        .expect("an unused name exists")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_names() {
        assert_eq!(sanitize_file_name("a/b\\c:d*e?.png"), "a_b_c_d_e_.png");
        assert_eq!(sanitize_file_name("  notes.  "), "notes");
        assert_eq!(sanitize_file_name("tab\there"), "tab_here");
        assert_eq!(sanitize_file_name(""), "file");
        assert_eq!(sanitize_file_name("..."), "file");
        let long = format!("{}.webm", "x".repeat(300));
        let cut = sanitize_file_name(&long);
        assert_eq!(cut.chars().count(), 150);
        assert!(cut.ends_with(".webm"));
    }

    #[test]
    fn folder_names() {
        assert_eq!(
            thread_folder_name(FolderNaming::Id, "4chan", "g", "123", "Rust"),
            "4chan-g-123"
        );
        assert_eq!(
            thread_folder_name(
                FolderNaming::Name,
                "4chan",
                "g",
                "123",
                "  /g/ - Desktop:  Thread?  "
            ),
            "_g_ - Desktop_ Thread_"
        );
        assert_eq!(
            thread_folder_name(FolderNaming::Name, "4chan", "g", "123", "   "),
            "4chan-g-123"
        );
        assert_eq!(
            thread_folder_name(FolderNaming::Name, "4chan", "g", "1", &"word ".repeat(40))
                .chars()
                .count(),
            79
        );
    }

    #[test]
    fn unique_names_skip_taken_and_existing() {
        let mut taken = HashSet::new();
        taken.insert("image.png".to_owned());
        let on_disk = ["image (1).png"];
        let exists = |n: &str| on_disk.contains(&n);
        assert_eq!(unique_name("image.png", &taken, exists), "image (2).png");
        assert_eq!(unique_name("other.png", &taken, exists), "other.png");
        assert_eq!(
            unique_name("IMAGE.PNG", &taken, |_| false),
            "IMAGE (1).PNG",
            "case-insensitive, for Windows"
        );
        taken.insert("readme".to_owned());
        assert_eq!(unique_name("readme", &taken, |_| false), "readme (1)");
    }
}

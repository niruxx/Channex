//! Rules for clearing the on-disk image cache: on a schedule, and by
//! evicting the oldest files once it grows past a size limit.

use serde::{Deserialize, Serialize};

/// How often the whole cache is cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClearInterval {
    Never,
    Daily,
    Monthly,
    #[default]
    #[serde(other)]
    Weekly,
}

impl ClearInterval {
    pub const ALL: [ClearInterval; 4] = [
        ClearInterval::Never,
        ClearInterval::Daily,
        ClearInterval::Weekly,
        ClearInterval::Monthly,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ClearInterval::Never => "Never",
            ClearInterval::Daily => "Every Day",
            ClearInterval::Weekly => "Every Week",
            ClearInterval::Monthly => "Every Month",
        }
    }

    fn seconds(self) -> Option<i64> {
        const DAY: i64 = 24 * 3600;
        match self {
            ClearInterval::Never => None,
            ClearInterval::Daily => Some(DAY),
            ClearInterval::Weekly => Some(7 * DAY),
            ClearInterval::Monthly => Some(30 * DAY),
        }
    }
}

/// Size limits offered in Preferences, in megabytes (0 = no limit).
pub const SIZE_LIMITS_MB: [u32; 6] = [0, 100, 250, 500, 1024, 2048];

/// Whether a scheduled clear is due. `last_cleared` and `now` are Unix
/// seconds; a cache that has never been cleared (0) starts its clock now
/// rather than being wiped on first launch.
pub fn clear_due(interval: ClearInterval, last_cleared: i64, now: i64) -> bool {
    match interval.seconds() {
        Some(period) => last_cleared > 0 && now - last_cleared >= period,
        None => false,
    }
}

/// One cached file, for eviction decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedFile {
    pub size: u64,
    /// Last modified (or used), Unix seconds.
    pub modified: i64,
}

/// Indices of the files to delete, oldest first, so the cache fits in
/// `limit` bytes. It's trimmed to 90% of the limit, so it isn't back at
/// the limit (and trimming again) after the next few thumbnails.
pub fn files_to_evict(files: &[CachedFile], limit: u64) -> Vec<usize> {
    let total: u64 = files.iter().map(|f| f.size).sum();
    if limit == 0 || total <= limit {
        return Vec::new();
    }
    let target = limit / 10 * 9;
    let mut order: Vec<usize> = (0..files.len()).collect();
    order.sort_by_key(|&i| files[i].modified);
    let mut remaining = total;
    let mut evict = Vec::new();
    for i in order {
        if remaining <= target {
            break;
        }
        remaining -= files[i].size;
        evict.push(i);
    }
    evict
}

/// "123 MB", "1.2 GB", "850 KB".
pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let b = bytes as f64;
    if b < KB * KB {
        format!("{:.0} KB", b / KB)
    } else if b < KB * KB * KB {
        format!("{:.0} MB", b / (KB * KB))
    } else {
        format!("{:.1} GB", b / (KB * KB * KB))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule() {
        let day = 24 * 3600;
        assert!(
            !clear_due(ClearInterval::Weekly, 0, 10 * day),
            "never cleared: start the clock"
        );
        assert!(!clear_due(ClearInterval::Weekly, day, 7 * day));
        assert!(clear_due(ClearInterval::Weekly, day, 8 * day));
        assert!(clear_due(ClearInterval::Daily, day, 2 * day));
        assert!(!clear_due(ClearInterval::Monthly, day, 20 * day));
        assert!(!clear_due(ClearInterval::Never, 1, 1000 * day));
    }

    #[test]
    fn eviction_removes_oldest_until_under_ninety_percent() {
        let files = vec![
            CachedFile {
                size: 40,
                modified: 30,
            },
            CachedFile {
                size: 40,
                modified: 10,
            },
            CachedFile {
                size: 40,
                modified: 20,
            },
        ];
        assert!(files_to_evict(&files, 0).is_empty(), "no limit");
        assert!(files_to_evict(&files, 200).is_empty(), "under the limit");
        // 120 bytes, limit 100 -> trim to 90: dropping the oldest leaves 80.
        assert_eq!(files_to_evict(&files, 100), vec![1]);
        // Limit 70 -> trim to 63: the two oldest go, oldest first.
        assert_eq!(files_to_evict(&files, 70), vec![1, 2]);
    }

    #[test]
    fn sizes() {
        assert_eq!(format_size(500 * 1024), "500 KB");
        assert_eq!(format_size(123 * 1024 * 1024), "123 MB");
        assert_eq!(format_size(3 * 1024 * 1024 * 1024 / 2), "1.5 GB");
    }

    #[test]
    fn interval_json() {
        assert_eq!(
            serde_json::to_string(&ClearInterval::Daily).unwrap(),
            "\"daily\""
        );
        assert_eq!(
            serde_json::from_str::<ClearInterval>("\"bogus\"").unwrap(),
            ClearInterval::Weekly
        );
    }
}

use std::path::Path;

use serde::Serialize;

use crate::error::CoreError;
use crate::utils::text::{find_match, lowercase_chars, make_snippet};

use super::repository::Timeline;

#[derive(Debug, Clone, Serialize)]
pub struct TimelineSearchHit {
    /// "%Y-%m-%d" — matches the timeline date listing.
    pub date: String,
    /// "%H:%M:%S", empty when the entry has no timestamp prefix.
    pub time: String,
    pub snippet: String,
}

/// Case-insensitive substring search over timeline entry text. The device
/// context JSON appended to each line is excluded so hostnames and battery
/// fields never pollute results. Hits come newest date first, in entry
/// order within a date.
pub fn search(base_dir: &Path, query: &str) -> Result<Vec<TimelineSearchHit>, CoreError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let query_chars = lowercase_chars(query);

    let timeline = Timeline::new(base_dir.to_path_buf());
    let mut hits = Vec::new();
    for date in timeline.list_dates()? {
        for entry in timeline.read(date)? {
            let (time, text) = split_entry(&entry);
            let text_chars: Vec<char> = text.chars().collect();
            let Some(index) = find_match(&text_chars, &query_chars) else {
                continue;
            };
            hits.push(TimelineSearchHit {
                date: date.format("%Y-%m-%d").to_string(),
                time,
                snippet: make_snippet(&text_chars, index, query_chars.len()),
            });
        }
    }

    Ok(hits)
}

/// Splits a raw entry line into (time, text), dropping the "- [HH:MM:SS] "
/// prefix and the trailing device-context JSON when present.
fn split_entry(raw: &str) -> (String, String) {
    let (time, rest) = match raw.strip_prefix("- [") {
        Some(after) => match after.split_once("] ") {
            Some((time, rest)) if time.len() == 8 => (time.to_string(), rest),
            _ => (String::new(), raw),
        },
        None => (String::new(), raw),
    };

    let text = match rest.rfind(" {") {
        Some(pos) if serde_json::from_str::<serde_json::Value>(&rest[pos + 1..]).is_ok() => {
            &rest[..pos]
        }
        _ => rest,
    };

    (time, text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_timeline(tmp: &TempDir, date: &str, lines: &[&str]) {
        let dir = tmp.path().join("data/timeline");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{date}.md")), lines.join("\n") + "\n").unwrap();
    }

    #[test]
    fn test_search_empty_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(search(tmp.path(), "anything").unwrap().is_empty());
    }

    #[test]
    fn test_search_empty_query_returns_empty() {
        let tmp = TempDir::new().unwrap();
        write_timeline(&tmp, "2026-01-01", &["- [09:00:00] some entry"]);
        assert!(search(tmp.path(), "").unwrap().is_empty());
        assert!(search(tmp.path(), "  ").unwrap().is_empty());
    }

    #[test]
    fn test_search_single_match_returns_date_time_snippet() {
        let tmp = TempDir::new().unwrap();
        write_timeline(&tmp, "2026-01-01", &["- [09:15:30] bought fresh basil"]);
        let hits = search(tmp.path(), "basil").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].date, "2026-01-01");
        assert_eq!(hits[0].time, "09:15:30");
        assert!(hits[0].snippet.contains("basil"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        write_timeline(&tmp, "2026-01-01", &["- [09:00:00] Visited KYOTO"]);
        assert_eq!(search(tmp.path(), "kyoto").unwrap().len(), 1);
    }

    #[test]
    fn test_search_excludes_context_json() {
        let tmp = TempDir::new().unwrap();
        write_timeline(
            &tmp,
            "2026-01-01",
            &[r#"- [09:00:00] plain entry {"battery":80,"hostname":"my-laptop"}"#],
        );
        assert!(search(tmp.path(), "my-laptop").unwrap().is_empty());
        assert!(search(tmp.path(), "battery").unwrap().is_empty());
        assert_eq!(search(tmp.path(), "plain entry").unwrap().len(), 1);
    }

    #[test]
    fn test_search_multiple_dates_newest_first() {
        let tmp = TempDir::new().unwrap();
        write_timeline(&tmp, "2026-01-01", &["- [09:00:00] match older"]);
        write_timeline(&tmp, "2026-01-02", &["- [09:00:00] match newer"]);
        let hits = search(tmp.path(), "match").unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].date, "2026-01-02");
        assert_eq!(hits[1].date, "2026-01-01");
    }

    #[test]
    fn test_search_japanese() {
        let tmp = TempDir::new().unwrap();
        write_timeline(&tmp, "2026-01-01", &["- [09:00:00] 京都で打ち合わせ"]);
        let hits = search(tmp.path(), "京都").unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snippet.contains("京都"));
    }

    #[test]
    fn test_split_entry_without_context() {
        let (time, text) = split_entry("- [12:00:00] no context here");
        assert_eq!(time, "12:00:00");
        assert_eq!(text, "no context here");
    }

    #[test]
    fn test_split_entry_without_time_prefix() {
        let (time, text) = split_entry("bare text line");
        assert_eq!(time, "");
        assert_eq!(text, "bare text line");
    }

    #[test]
    fn test_split_entry_keeps_braces_that_are_not_json() {
        let (_, text) = split_entry("- [12:00:00] set {a, b} notation");
        assert_eq!(text, "set {a, b} notation");
    }
}

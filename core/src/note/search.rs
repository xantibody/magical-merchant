use std::path::Path;

use serde::Serialize;

use crate::error::CoreError;
use crate::utils::frontmatter::{self, NoteFrontmatter};
use crate::utils::fs::list_md_files;
use crate::utils::paths::notes_dir;

use super::title::extract_title;

/// How much context to keep around a match when building a snippet.
const SNIPPET_BEFORE: usize = 30;
const SNIPPET_AFTER: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub filename: String,
    pub title: String,
    pub snippet: String,
}

/// Case-insensitive substring search over note bodies (frontmatter
/// excluded; titles are part of the body, so they match too). Results keep
/// the directory listing order (newest first). Empty or whitespace-only
/// queries return nothing.
pub fn search(base_dir: &Path, query: &str) -> Result<Vec<SearchHit>, CoreError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let query_chars = lowercase_chars(query);

    let mut hits = Vec::new();
    for entry in list_md_files(&notes_dir(base_dir))? {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let body = match frontmatter::parse::<NoteFrontmatter>(&content) {
            Ok((_, body)) => body,
            Err(_) => content,
        };

        let body_chars: Vec<char> = body.chars().collect();
        let Some(index) = find_match(&body_chars, &query_chars) else {
            continue;
        };

        hits.push(SearchHit {
            filename: entry.file_name().to_string_lossy().to_string(),
            title: extract_title(&body).unwrap_or_default(),
            snippet: make_snippet(&body_chars, index, query_chars.len()),
        });
    }

    Ok(hits)
}

/// Lowercases a string into chars for use with `find_match`.
pub(super) fn lowercase_chars(s: &str) -> Vec<char> {
    s.chars().flat_map(|c| c.to_lowercase()).collect()
}

/// Finds the first case-insensitive occurrence of `query` in `haystack`,
/// comparing char by char so multi-byte text stays safe.
pub(super) fn find_match(haystack: &[char], query: &[char]) -> Option<usize> {
    if query.is_empty() || haystack.len() < query.len() {
        return None;
    }
    (0..=haystack.len() - query.len()).find(|&start| {
        haystack[start..start + query.len()]
            .iter()
            .flat_map(|c| c.to_lowercase())
            .eq(query.iter().copied())
    })
}

/// Cuts a window around the match, collapsing newlines and adding ellipses
/// when text was dropped on either side.
fn make_snippet(chars: &[char], match_index: usize, match_len: usize) -> String {
    let start = match_index.saturating_sub(SNIPPET_BEFORE);
    let end = (match_index + match_len + SNIPPET_AFTER).min(chars.len());

    let mut snippet: String = chars[start..end]
        .iter()
        .map(|&c| if c == '\n' { ' ' } else { c })
        .collect();
    if start > 0 {
        snippet = format!("…{snippet}");
    }
    if end < chars.len() {
        snippet = format!("{snippet}…");
    }
    snippet
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use std::fs;
    use tempfile::TempDir;

    fn write_note(tmp: &TempDir, filename: &str, body: &str) {
        let fm = NoteFrontmatter {
            time: FixedOffset::east_opt(9 * 3600)
                .unwrap()
                .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
                .unwrap(),
            tags: vec!["secret-tag".to_string()],
            context: None,
        };
        let content = frontmatter::render(&fm, body).unwrap();
        let dir = tmp.path().join("data/notes");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn test_search_empty_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(search(tmp.path(), "anything").unwrap().is_empty());
    }

    #[test]
    fn test_search_empty_query_returns_empty() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "some body");
        assert!(search(tmp.path(), "").unwrap().is_empty());
        assert!(search(tmp.path(), "   ").unwrap().is_empty());
    }

    #[test]
    fn test_search_single_match_returns_hit_with_title_and_snippet() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Recipe\nadd fresh basil now");
        let hits = search(tmp.path(), "basil").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].filename, "20260101_000001.md");
        assert_eq!(hits[0].title, "Recipe");
        assert!(hits[0].snippet.contains("basil"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "Rust is GREAT");
        assert_eq!(search(tmp.path(), "great").unwrap().len(), 1);
        assert_eq!(search(tmp.path(), "RUST").unwrap().len(), 1);
    }

    #[test]
    fn test_search_multiple_notes_sorted_newest_first() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "match older");
        write_note(&tmp, "20260102_000001.md", "match newer");
        let hits = search(tmp.path(), "match").unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].filename, "20260102_000001.md");
        assert_eq!(hits[1].filename, "20260101_000001.md");
    }

    #[test]
    fn test_search_no_match_returns_empty() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "nothing relevant");
        assert!(search(tmp.path(), "zebra").unwrap().is_empty());
    }

    #[test]
    fn test_search_does_not_match_frontmatter() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "plain body");
        assert!(search(tmp.path(), "secret-tag").unwrap().is_empty());
    }

    #[test]
    fn test_search_japanese() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# 旅行計画\n京都へ行く予定");
        let hits = search(tmp.path(), "京都").unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snippet.contains("京都"));
    }

    #[test]
    fn test_search_snippet_ellipsis_for_long_body() {
        let tmp = TempDir::new().unwrap();
        let body = format!("{}NEEDLE{}", "a".repeat(100), "b".repeat(100));
        write_note(&tmp, "20260101_000001.md", &body);
        let hits = search(tmp.path(), "needle").unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snippet.starts_with('…'));
        assert!(hits[0].snippet.ends_with('…'));
        assert!(hits[0].snippet.contains("NEEDLE"));
    }

    #[test]
    fn test_search_matches_title_when_body_does_not() {
        let tmp = TempDir::new().unwrap();
        // The title comes from the heading; search for a word only in it.
        write_note(&tmp, "20260101_000001.md", "# Unique Heading\nbody text");
        let hits = search(tmp.path(), "unique").unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_search_snippet_collapses_newlines() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "first\nNEEDLE\nlast");
        let hits = search(tmp.path(), "needle").unwrap();
        assert!(!hits[0].snippet.contains('\n'));
    }
}

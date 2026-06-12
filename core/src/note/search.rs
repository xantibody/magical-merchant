use std::path::Path;

use serde::Serialize;

use crate::error::CoreError;
use crate::utils::frontmatter::{self, NoteFrontmatter};
use crate::utils::fs::list_md_files;
use crate::utils::paths::notes_dir;
use crate::utils::text::{find_match, lowercase_chars, make_snippet};

use super::title::extract_title;

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

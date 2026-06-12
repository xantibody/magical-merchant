use std::path::Path;

use crate::error::CoreError;
use crate::utils::frontmatter::{self, NoteFrontmatter};
use crate::utils::validated::NoteFilename;

use super::repository::Notes;
use super::search::{find_match, lowercase_chars};
use super::summary::Summary as NoteSummary;
use super::wikilink::{extract_wikilinks, resolve_in};

/// Lists notes that mention the target note's title in plain text without
/// wikilinking to it — knowledge that connected itself while writing.
/// Notes that already wikilink to the target are excluded (they are
/// backlinks), as is the target itself. Matching is case-insensitive.
pub fn mentions(base_dir: &Path, target: &NoteFilename) -> Result<Vec<NoteSummary>, CoreError> {
    let repo = Notes::new(base_dir.to_path_buf());
    let notes = repo.list()?;

    let Some(target_note) = notes.iter().find(|n| n.filename == target.as_str()) else {
        return Ok(Vec::new());
    };
    let title = target_note.title.trim();
    if title.is_empty() {
        return Ok(Vec::new());
    }
    let title_chars = lowercase_chars(title);

    let mut result = Vec::new();
    for note in &notes {
        if note.filename == target.as_str() {
            continue;
        }
        let Ok(content) = repo.read(&NoteFilename::parse(&note.filename)?) else {
            continue;
        };
        let body = match frontmatter::parse::<NoteFrontmatter>(&content) {
            Ok((_, body)) => body,
            Err(_) => content,
        };

        let already_linked = extract_wikilinks(&body).iter().any(|link| {
            resolve_in(&notes, link)
                .map(|f| f == target.as_str())
                .unwrap_or(false)
        });
        if already_linked {
            continue;
        }

        let body_chars: Vec<char> = body.chars().collect();
        if find_match(&body_chars, &title_chars).is_some() {
            result.push(note.clone());
        }
    }

    Ok(result)
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
            tags: Vec::new(),
            context: None,
        };
        let content = frontmatter::render(&fm, body).unwrap();
        let dir = tmp.path().join("data/notes");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), content).unwrap();
    }

    fn target() -> NoteFilename {
        NoteFilename::parse("20260101_000001.md").unwrap()
    }

    #[test]
    fn test_mentions_empty_when_no_notes() {
        let tmp = TempDir::new().unwrap();
        assert!(mentions(tmp.path(), &target()).unwrap().is_empty());
    }

    #[test]
    fn test_mentions_empty_when_nothing_mentions_title() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Rust Tips\nbody");
        write_note(&tmp, "20260101_000002.md", "unrelated text");
        assert!(mentions(tmp.path(), &target()).unwrap().is_empty());
    }

    #[test]
    fn test_mentions_finds_plain_text_mention() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Rust Tips\nbody");
        write_note(&tmp, "20260101_000002.md", "I should reread Rust Tips soon");
        let result = mentions(tmp.path(), &target()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].filename, "20260101_000002.md");
    }

    #[test]
    fn test_mentions_multiple_notes() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Rust Tips\nbody");
        write_note(&tmp, "20260101_000002.md", "see rust tips");
        write_note(&tmp, "20260101_000003.md", "more Rust Tips here");
        assert_eq!(mentions(tmp.path(), &target()).unwrap().len(), 2);
    }

    #[test]
    fn test_mentions_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Rust Tips\nbody");
        write_note(&tmp, "20260101_000002.md", "RUST TIPS in caps");
        assert_eq!(mentions(tmp.path(), &target()).unwrap().len(), 1);
    }

    #[test]
    fn test_mentions_excludes_notes_that_wikilink() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Rust Tips\nbody");
        // Already a backlink — must not double-report as a mention.
        write_note(&tmp, "20260101_000002.md", "see [[Rust Tips]]");
        assert!(mentions(tmp.path(), &target()).unwrap().is_empty());
    }

    #[test]
    fn test_mentions_excludes_target_itself() {
        let tmp = TempDir::new().unwrap();
        write_note(
            &tmp,
            "20260101_000001.md",
            "# Rust Tips\nRust Tips inside itself",
        );
        assert!(mentions(tmp.path(), &target()).unwrap().is_empty());
    }

    #[test]
    fn test_mentions_empty_for_untitled_target() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "");
        write_note(&tmp, "20260101_000002.md", "anything at all");
        assert!(mentions(tmp.path(), &target()).unwrap().is_empty());
    }

    #[test]
    fn test_mentions_missing_target_returns_empty() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000002.md", "some text");
        assert!(mentions(tmp.path(), &target()).unwrap().is_empty());
    }

    #[test]
    fn test_mentions_japanese_title() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# 設計メモ\n本文");
        write_note(&tmp, "20260101_000002.md", "昨日の設計メモを見直す");
        assert_eq!(mentions(tmp.path(), &target()).unwrap().len(), 1);
    }
}

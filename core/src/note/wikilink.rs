use std::path::Path;

use crate::error::CoreError;
use crate::utils::validated::NoteFilename;

use super::repository::Notes;
use super::summary::Summary as NoteSummary;

/// Extracts `[[Title]]` targets from a note body in order of appearance,
/// trimmed and deduplicated. Links inside fenced code blocks or inline code
/// spans are ignored, as are empty titles and ones containing brackets.
pub fn extract_wikilinks(body: &str) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    let mut in_fence = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        scan_line(line, &mut links);
    }

    links
}

/// Scans one line for wikilinks, skipping inline code spans.
fn scan_line(line: &str, links: &mut Vec<String>) {
    let chars: Vec<char> = line.chars().collect();
    let mut in_code = false;
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if in_code || !(chars[i] == '[' && chars.get(i + 1) == Some(&'[')) {
            i += 1;
            continue;
        }
        let Some(close) = find_close(&chars, i + 2) else {
            i += 2;
            continue;
        };
        let title: String = chars[i + 2..close].iter().collect();
        let title = title.trim();
        if !title.is_empty()
            && !title.contains('[')
            && !title.contains(']')
            && !links.iter().any(|l| l == title)
        {
            links.push(title.to_string());
        }
        i = close + 2;
    }
}

/// Finds the index of the next `]]`, starting at `from`.
fn find_close(chars: &[char], from: usize) -> Option<usize> {
    (from..chars.len().saturating_sub(1)).find(|&j| chars[j] == ']' && chars[j + 1] == ']')
}

/// Resolves a wikilink title to a note filename by exact title match
/// (after trimming). When multiple notes share the title, the oldest
/// (smallest filename) wins so existing links stay stable as notes are added.
pub fn resolve(base_dir: &Path, title: &str) -> Result<Option<String>, CoreError> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(None);
    }
    let notes = Notes::new(base_dir.to_path_buf()).list()?;
    Ok(notes
        .into_iter()
        .filter(|n| n.title == title)
        .map(|n| n.filename)
        .min())
}

/// Lists notes that contain a wikilink resolving to `target`, excluding the
/// target note itself.
pub fn backlinks(base_dir: &Path, target: &NoteFilename) -> Result<Vec<NoteSummary>, CoreError> {
    let repo = Notes::new(base_dir.to_path_buf());
    let notes = repo.list()?;

    let mut result = Vec::new();
    for note in &notes {
        if note.filename == target.as_str() {
            continue;
        }
        let Ok(content) = repo.read(&NoteFilename::parse(&note.filename)?) else {
            continue;
        };
        let body = match crate::utils::frontmatter::parse::<
            crate::utils::frontmatter::NoteFrontmatter,
        >(&content)
        {
            Ok((_, body)) => body,
            Err(_) => content,
        };
        let links_to_target = extract_wikilinks(&body).iter().any(|link| {
            resolve_in(&notes, link)
                .map(|f| f == target.as_str())
                .unwrap_or(false)
        });
        if links_to_target {
            result.push(note.clone());
        }
    }

    Ok(result)
}

/// Resolves a title against an already-loaded note list (same policy as
/// `resolve`, without re-reading the directory per link).
pub(super) fn resolve_in(notes: &[NoteSummary], title: &str) -> Option<String> {
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    notes
        .iter()
        .filter(|n| n.title == title)
        .map(|n| n.filename.clone())
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::frontmatter::{self, NoteFrontmatter};
    use chrono::{FixedOffset, TimeZone};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_wikilinks_none() {
        assert!(extract_wikilinks("no links here").is_empty());
        assert!(extract_wikilinks("").is_empty());
    }

    #[test]
    fn test_extract_wikilinks_single() {
        assert_eq!(extract_wikilinks("see [[Other Note]]"), vec!["Other Note"]);
    }

    #[test]
    fn test_extract_wikilinks_multiple_in_order() {
        assert_eq!(
            extract_wikilinks("[[B]] then [[A]]\nand [[C]]"),
            vec!["B", "A", "C"]
        );
    }

    #[test]
    fn test_extract_wikilinks_dedupes() {
        assert_eq!(extract_wikilinks("[[X]] and [[X]] again"), vec!["X"]);
    }

    #[test]
    fn test_extract_wikilinks_ignores_code_fence() {
        let body = "```\n[[Not A Link]]\n```\n[[Real]]";
        assert_eq!(extract_wikilinks(body), vec!["Real"]);
    }

    #[test]
    fn test_extract_wikilinks_ignores_inline_code() {
        assert_eq!(extract_wikilinks("`[[code]]` but [[Real]]"), vec!["Real"]);
    }

    #[test]
    fn test_extract_wikilinks_ignores_empty_and_whitespace() {
        assert!(extract_wikilinks("[[]] [[  ]]").is_empty());
    }

    #[test]
    fn test_extract_wikilinks_ignores_unclosed() {
        assert!(extract_wikilinks("[[unclosed").is_empty());
    }

    #[test]
    fn test_extract_wikilinks_trims_title() {
        assert_eq!(
            extract_wikilinks("[[ Padded Title ]]"),
            vec!["Padded Title"]
        );
    }

    #[test]
    fn test_extract_wikilinks_japanese() {
        assert_eq!(
            extract_wikilinks("詳細は[[設計メモ]]を参照"),
            vec!["設計メモ"]
        );
    }

    /// Writes a fixture note directly so tests control the filename ordering.
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

    #[test]
    fn test_resolve_no_notes_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(resolve(tmp.path(), "Anything").unwrap(), None);
    }

    #[test]
    fn test_resolve_finds_note_by_title() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Design Memo\nbody");
        assert_eq!(
            resolve(tmp.path(), "Design Memo").unwrap(),
            Some("20260101_000001.md".to_string())
        );
    }

    #[test]
    fn test_resolve_duplicate_titles_picks_oldest() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260102_000000.md", "# Same\nnewer");
        write_note(&tmp, "20260101_000000.md", "# Same\nolder");
        assert_eq!(
            resolve(tmp.path(), "Same").unwrap(),
            Some("20260101_000000.md".to_string())
        );
    }

    #[test]
    fn test_resolve_trims_title_argument() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Padded\nbody");
        assert_eq!(
            resolve(tmp.path(), "  Padded  ").unwrap(),
            Some("20260101_000001.md".to_string())
        );
    }

    #[test]
    fn test_resolve_empty_title_returns_none() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "");
        assert_eq!(resolve(tmp.path(), "").unwrap(), None);
        assert_eq!(resolve(tmp.path(), "   ").unwrap(), None);
    }

    #[test]
    fn test_backlinks_empty_when_no_links() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Target\nbody");
        write_note(&tmp, "20260101_000002.md", "# Other\nno links");
        let target = NoteFilename::parse("20260101_000001.md").unwrap();
        assert!(backlinks(tmp.path(), &target).unwrap().is_empty());
    }

    #[test]
    fn test_backlinks_finds_single_linker() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Target\nbody");
        write_note(&tmp, "20260101_000002.md", "# Linker\nsee [[Target]]");
        let target = NoteFilename::parse("20260101_000001.md").unwrap();
        let result = backlinks(tmp.path(), &target).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].filename, "20260101_000002.md");
        assert_eq!(result[0].title, "Linker");
    }

    #[test]
    fn test_backlinks_multiple_linkers() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Target\nbody");
        write_note(&tmp, "20260101_000002.md", "[[Target]]");
        write_note(&tmp, "20260101_000003.md", "also [[Target]]");
        let target = NoteFilename::parse("20260101_000001.md").unwrap();
        assert_eq!(backlinks(tmp.path(), &target).unwrap().len(), 2);
    }

    #[test]
    fn test_backlinks_excludes_self_link() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Target\nself [[Target]]");
        let target = NoteFilename::parse("20260101_000001.md").unwrap();
        assert!(backlinks(tmp.path(), &target).unwrap().is_empty());
    }

    #[test]
    fn test_backlinks_ignores_unresolved_links() {
        let tmp = TempDir::new().unwrap();
        write_note(&tmp, "20260101_000001.md", "# Target\nbody");
        write_note(&tmp, "20260101_000002.md", "[[Nonexistent]]");
        let target = NoteFilename::parse("20260101_000001.md").unwrap();
        assert!(backlinks(tmp.path(), &target).unwrap().is_empty());
    }
}

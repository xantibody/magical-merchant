use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use serde::Serialize;

use crate::utils::frontmatter::{self, NoteFrontmatter};

use super::title::extract_title;

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub path: PathBuf,
    pub filename: String,
    pub time: Option<DateTime<FixedOffset>>,
    pub tags: Vec<String>,
    pub preview: String,
    pub title: String,
}

impl Summary {
    pub fn from_file(path: PathBuf, filename: String, content: &str) -> Self {
        let (time, tags, body) = match frontmatter::parse::<NoteFrontmatter>(content) {
            Ok((fm, body)) => (Some(fm.time), fm.tags, body),
            Err(_) => (None, Vec::new(), content.to_string()),
        };
        let preview: String = body.chars().take(100).collect();
        let title = extract_title(&body).unwrap_or_default();

        Self {
            path,
            filename,
            time,
            tags,
            preview,
            title,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::device::Context;
    use crate::utils::frontmatter::NoteFrontmatter;
    use chrono::{FixedOffset, TimeZone};
    use std::path::PathBuf;

    #[test]
    fn test_from_file_with_valid_frontmatter() {
        let fm = NoteFrontmatter {
            time: FixedOffset::east_opt(9 * 3600)
                .unwrap()
                .with_ymd_and_hms(2026, 3, 20, 14, 30, 45)
                .unwrap(),
            tags: vec!["a".to_string(), "b".to_string()],
            context: Some(Context {
                battery: Some(50),
                is_charging: Some(false),
                ..Context::default()
            }),
        };
        let content = frontmatter::render(&fm, "# Title\nBody").unwrap();
        let summary = Summary::from_file(
            PathBuf::from("/test/note.md"),
            "note.md".to_string(),
            &content,
        );
        assert!(summary.time.is_some());
        assert_eq!(summary.tags, vec!["a", "b"]);
        assert!(summary.preview.contains("# Title"));
        assert_eq!(summary.title, "Title");
    }

    #[test]
    fn test_from_file_with_invalid_content() {
        let summary = Summary::from_file(
            PathBuf::from("/test/note.md"),
            "note.md".to_string(),
            "no frontmatter here",
        );
        assert!(summary.time.is_none());
        assert!(summary.tags.is_empty());
        assert_eq!(summary.preview, "no frontmatter here");
        assert_eq!(summary.title, "no frontmatter here");
    }

    #[test]
    fn test_from_file_title_empty_body_is_empty_string() {
        let fm = NoteFrontmatter {
            time: FixedOffset::east_opt(9 * 3600)
                .unwrap()
                .with_ymd_and_hms(2026, 3, 20, 14, 30, 45)
                .unwrap(),
            tags: Vec::new(),
            context: None,
        };
        let content = frontmatter::render(&fm, "").unwrap();
        let summary = Summary::from_file(
            PathBuf::from("/test/note.md"),
            "note.md".to_string(),
            &content,
        );
        assert_eq!(summary.title, "");
    }
}

/// Extracts a human-facing title from a note body (frontmatter already
/// stripped by the caller). The first ATX heading wins even if plain text
/// precedes it; otherwise the first non-empty line is used. Lines inside
/// fenced code blocks and bare `---` lines are never picked.
pub fn extract_title(body: &str) -> Option<String> {
    let mut first_line: Option<String> = None;
    let mut in_fence = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.is_empty() || trimmed.chars().all(|c| c == '-') {
            continue;
        }
        if let Some(heading) = parse_heading(trimmed) {
            return Some(heading);
        }
        if first_line.is_none() {
            first_line = Some(trimmed.to_string());
        }
    }

    first_line
}

/// Returns the text of an ATX heading (`# ` through `###### `), or None.
fn parse_heading(line: &str) -> Option<String> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with(' ') && !rest.is_empty() {
        return None;
    }
    let text = rest.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_empty_body_returns_none() {
        assert_eq!(extract_title(""), None);
        assert_eq!(extract_title("\n\n  \n"), None);
    }

    #[test]
    fn test_extract_title_from_heading() {
        assert_eq!(extract_title("# Hello\nbody"), Some("Hello".to_string()));
    }

    #[test]
    fn test_extract_title_from_first_nonempty_line() {
        assert_eq!(
            extract_title("\n\nplain text\nmore"),
            Some("plain text".to_string())
        );
    }

    #[test]
    fn test_extract_title_heading_wins_over_earlier_text() {
        assert_eq!(
            extract_title("intro\n# Title\nbody"),
            Some("Title".to_string())
        );
    }

    #[test]
    fn test_extract_title_strips_hash_levels() {
        assert_eq!(extract_title("### Deep"), Some("Deep".to_string()));
        assert_eq!(extract_title("###### Six"), Some("Six".to_string()));
    }

    #[test]
    fn test_extract_title_ignores_heading_in_code_fence() {
        let body = "```sh\n# comment in code\n```\nactual first line";
        assert_eq!(extract_title(body), Some("actual first line".to_string()));
    }

    #[test]
    fn test_extract_title_skips_dashes_line() {
        assert_eq!(
            extract_title("---\nreal title"),
            Some("real title".to_string())
        );
    }

    #[test]
    fn test_extract_title_japanese() {
        assert_eq!(
            extract_title("# 設計メモ\n本文"),
            Some("設計メモ".to_string())
        );
        assert_eq!(
            extract_title("買い物リスト"),
            Some("買い物リスト".to_string())
        );
    }

    #[test]
    fn test_extract_title_hash_without_space_is_not_heading() {
        assert_eq!(
            extract_title("#hashtag text"),
            Some("#hashtag text".to_string())
        );
    }
}

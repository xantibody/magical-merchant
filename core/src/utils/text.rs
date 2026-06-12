//! Char-based text search helpers shared by note and timeline search.
//! Working in chars (not bytes) keeps multi-byte text safe to slice.

/// How much context to keep around a match when building a snippet.
const SNIPPET_BEFORE: usize = 30;
const SNIPPET_AFTER: usize = 50;

/// Lowercases a string into chars for use with `find_match`.
pub fn lowercase_chars(s: &str) -> Vec<char> {
    s.chars().flat_map(|c| c.to_lowercase()).collect()
}

/// Finds the first case-insensitive occurrence of `query` in `haystack`,
/// comparing char by char so multi-byte text stays safe.
pub fn find_match(haystack: &[char], query: &[char]) -> Option<usize> {
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
pub fn make_snippet(chars: &[char], match_index: usize, match_len: usize) -> String {
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

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_find_match_empty_query_returns_none() {
        assert_eq!(find_match(&chars("abc"), &[]), None);
    }

    #[test]
    fn test_find_match_finds_case_insensitive() {
        assert_eq!(find_match(&chars("Hello World"), &chars("world")), Some(6));
    }

    #[test]
    fn test_find_match_no_match() {
        assert_eq!(find_match(&chars("abc"), &chars("zzz")), None);
    }

    #[test]
    fn test_make_snippet_short_text_no_ellipsis() {
        let c = chars("short text");
        assert_eq!(make_snippet(&c, 0, 5), "short text");
    }

    #[test]
    fn test_make_snippet_adds_ellipses_both_sides() {
        let text = format!("{}NEEDLE{}", "a".repeat(100), "b".repeat(100));
        let c = chars(&text);
        let snippet = make_snippet(&c, 100, 6);
        assert!(snippet.starts_with('…'));
        assert!(snippet.ends_with('…'));
        assert!(snippet.contains("NEEDLE"));
    }
}

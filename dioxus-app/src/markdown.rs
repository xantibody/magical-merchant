use pulldown_cmark::{html, Options, Parser};

/// Convert Markdown source to HTML string.
pub fn render_markdown(source: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(source, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(render_markdown(""), "");
    }

    #[test]
    fn heading() {
        let html = render_markdown("# Hello");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn bold_and_italic() {
        let html = render_markdown("**bold** and *italic*");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn unordered_list() {
        let html = render_markdown("- one\n- two\n- three");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>one</li>"));
    }

    #[test]
    fn code_block() {
        let html = render_markdown("```rust\nfn main() {}\n```");
        assert!(html.contains("<code"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn strikethrough() {
        let html = render_markdown("~~deleted~~");
        assert!(html.contains("<del>deleted</del>"));
    }

    #[test]
    fn inline_code() {
        let html = render_markdown("use `code` here");
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn link() {
        let html = render_markdown("[text](https://example.com)");
        assert!(html.contains("<a href=\"https://example.com\">text</a>"));
    }
}

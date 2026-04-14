use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// Markdown editor with inline preview.
///
/// Lines above the cursor are rendered as HTML.
/// The current line and below remain as raw Markdown in a textarea.
/// Clicking the preview area switches back to full textarea editing.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    // Track how many lines from the top are "committed" to preview
    let mut preview_lines = use_signal(|| 0usize);
    let placeholder = placeholder.unwrap_or_default();

    let lines: Vec<&str> = value.split('\n').collect();
    let committed = preview_lines().min(lines.len());

    // The preview portion (lines 0..committed)
    let preview_text = if committed > 0 {
        lines[..committed].join("\n")
    } else {
        String::new()
    };

    // The editing portion (lines committed..)
    let edit_text = if committed < lines.len() {
        lines[committed..].join("\n")
    } else {
        String::new()
    };

    let preview_html = if committed > 0 {
        render_markdown(&preview_text)
    } else {
        String::new()
    };

    let value_for_input = value.clone();
    let handle_input = move |new_edit: String| {
        let committed_count = preview_lines();
        let top: Vec<&str> = value_for_input.split('\n').take(committed_count).collect();
        let mut full = top.join("\n");
        if !full.is_empty() {
            full.push('\n');
        }
        full.push_str(&new_edit);
        oninput(full);
    };

    let handle_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Enter {
            let all_lines: Vec<&str> = value.split('\n').collect();
            preview_lines.set(all_lines.len());
        }
    };

    let handle_preview_click = move |_| {
        // Click on preview: reset to full editing mode
        preview_lines.set(0);
    };

    rsx! {
        div { class: "markdown-editor",
            if committed > 0 {
                div {
                    class: "md-preview",
                    onclick: handle_preview_click,
                    dangerous_inner_html: "{preview_html}",
                }
            }

            textarea {
                class: "memo-input md-edit-area",
                placeholder: if committed == 0 { placeholder } else { String::new() },
                value: "{edit_text}",
                autofocus: true,
                oninput: move |e| handle_input(e.value()),
                onkeydown: handle_keydown,
            }
        }
    }
}

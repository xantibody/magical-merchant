use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// Markdown editor that switches between edit (textarea) and preview (rendered HTML).
///
/// When focused: shows raw Markdown in a textarea.
/// When blurred: shows rendered HTML. Click to re-enter edit mode.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let mut editing = use_signal(|| true);
    let placeholder = placeholder.unwrap_or_default();

    if editing() || value.is_empty() {
        rsx! {
            div { class: "markdown-editor",
                textarea {
                    class: "memo-input markdown-editor-input",
                    placeholder: "{placeholder}",
                    value: "{value}",
                    autofocus: true,
                    oninput: move |e| oninput(e.value()),
                    onblur: move |_| {
                        if !value.is_empty() {
                            editing.set(false);
                        }
                    },
                }
            }
        }
    } else {
        let html = render_markdown(&value);
        rsx! {
            div {
                class: "markdown-editor markdown-editor-preview",
                onclick: move |_| editing.set(true),
                dangerous_inner_html: "{html}",
            }
        }
    }
}

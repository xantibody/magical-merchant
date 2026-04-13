use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// A Markdown editor with live preview below the input area.
/// The preview updates in real-time as the user types.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let html = render_markdown(&value);
    let placeholder = placeholder.unwrap_or_default();

    rsx! {
        div { class: "markdown-editor",
            textarea {
                class: "memo-input markdown-editor-input",
                placeholder: "{placeholder}",
                value: "{value}",
                oninput: move |e| oninput(e.value()),
            }
            if !value.is_empty() {
                div { class: "markdown-preview",
                    dangerous_inner_html: "{html}",
                }
            }
        }
    }
}

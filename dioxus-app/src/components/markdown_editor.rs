use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// Typora-style Markdown editor with inline rendering.
///
/// Lines not being edited are rendered as HTML.
/// The currently active line shows raw Markdown for editing.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let active_line = use_signal(|| 0usize);
    let placeholder = placeholder.unwrap_or_default();

    let lines: Vec<String> = if value.is_empty() {
        vec![String::new()]
    } else {
        value.split('\n').map(String::from).collect()
    };
    let total_lines = lines.len();

    rsx! {
        div { class: "markdown-editor",
            if value.is_empty() {
                div {
                    class: "md-line md-line-edit",
                    input {
                        class: "md-line-input",
                        r#type: "text",
                        placeholder: "{placeholder}",
                        value: "",
                        autofocus: true,
                        oninput: move |e| oninput(e.value()),
                    }
                }
            } else {
                for (idx, line) in lines.iter().enumerate() {
                    if idx == active_line() {
                        EditLine {
                            key: "{idx}",
                            idx: idx,
                            line: line.clone(),
                            total_lines: total_lines,
                            value: value.clone(),
                            active_line: active_line,
                            oninput: oninput,
                        }
                    } else {
                        ViewLine {
                            key: "{idx}",
                            idx: idx,
                            line: line.clone(),
                            active_line: active_line,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditLine(
    idx: usize,
    line: String,
    total_lines: usize,
    value: String,
    mut active_line: Signal<usize>,
    oninput: EventHandler<String>,
) -> Element {
    let value_for_replace = value.clone();
    let replace_line = move |new_content: &str| {
        let mut parts: Vec<&str> = value_for_replace.split('\n').collect();
        if idx < parts.len() {
            parts[idx] = new_content;
        }
        oninput(parts.join("\n"));
    };

    let mut insert_newline = move || {
        let mut parts: Vec<String> = value.split('\n').map(String::from).collect();
        if idx < parts.len() {
            parts.insert(idx + 1, String::new());
        }
        active_line.set(idx + 1);
        oninput(parts.join("\n"));
    };

    rsx! {
        div { class: "md-line md-line-edit",
            input {
                class: "md-line-input",
                r#type: "text",
                value: "{line}",
                autofocus: true,
                oninput: move |e: Event<FormData>| {
                    replace_line(&e.value());
                },
                onkeydown: move |e: Event<KeyboardData>| {
                    match e.key() {
                        Key::Enter => {
                            e.prevent_default();
                            insert_newline();
                        }
                        Key::ArrowUp => {
                            if idx > 0 {
                                active_line.set(idx - 1);
                            }
                        }
                        Key::ArrowDown => {
                            if idx < total_lines - 1 {
                                active_line.set(idx + 1);
                            }
                        }
                        _ => {}
                    }
                },
            }
        }
    }
}

#[component]
fn ViewLine(idx: usize, line: String, mut active_line: Signal<usize>) -> Element {
    if line.trim().is_empty() {
        return rsx! {
            div {
                class: "md-line md-line-empty",
                onclick: move |_| active_line.set(idx),
                "\u{00A0}"
            }
        };
    }

    let html = render_markdown(&line);

    rsx! {
        div {
            class: "md-line md-line-view",
            onclick: move |_| active_line.set(idx),
            dangerous_inner_html: "{html}",
        }
    }
}

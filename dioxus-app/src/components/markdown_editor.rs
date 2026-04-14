use std::collections::HashMap;

use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// Markdown editor with per-line inline rendering.
///
/// The active line shows raw Markdown. Other lines show rendered HTML.
/// When the active line changes, the previously active line is rendered
/// asynchronously so editing is never blocked.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let mut active_line = use_signal(|| 0usize);
    // Cache: line_index -> (source_text, rendered_html)
    let mut render_cache = use_signal(HashMap::<usize, (String, String)>::new);
    // Tracks the previous active line so we know what to render on change
    let mut prev_active = use_signal(|| 0usize);
    let placeholder = placeholder.unwrap_or_default();

    let lines: Vec<String> = if value.is_empty() {
        vec![String::new()]
    } else {
        value.split('\n').map(String::from).collect()
    };
    let total_lines = lines.len();

    // When active line changes, async-render the line we just left
    let value_for_effect = value.clone();
    use_effect(move || {
        let current = active_line();
        let prev = prev_active();
        if current != prev {
            prev_active.set(current);
            // Render the line we just left
            let all_lines: Vec<String> = value_for_effect.split('\n').map(String::from).collect();
            if prev < all_lines.len() {
                let src = all_lines[prev].clone();
                if !src.trim().is_empty() {
                    spawn(async move {
                        let src_for_render = src.clone();
                        let html =
                            tokio::task::spawn_blocking(move || render_markdown(&src_for_render))
                                .await
                                .unwrap_or_default();
                        render_cache.write().insert(prev, (src, html));
                    });
                } else {
                    render_cache.write().remove(&prev);
                }
            }
        }
    });

    let update_line = {
        let value = value.clone();
        move |idx: usize, new_content: String| {
            let mut parts: Vec<String> = value.split('\n').map(String::from).collect();
            while parts.len() <= idx {
                parts.push(String::new());
            }
            parts[idx] = new_content;
            oninput(parts.join("\n"));
        }
    };

    let insert_line_after = {
        let value = value.clone();
        move |idx: usize| {
            let mut parts: Vec<String> = value.split('\n').map(String::from).collect();
            parts.insert(idx + 1, String::new());
            active_line.set(idx + 1);
            oninput(parts.join("\n"));
        }
    };

    let cache = render_cache();

    rsx! {
        div {
            class: "markdown-editor",
            for (idx , line) in lines.iter().enumerate() {
                if idx == active_line() {
                    div {
                        key: "line-{idx}",
                        class: "md-line md-line-edit",
                        input {
                            class: "md-line-input",
                            r#type: "text",
                            placeholder: if idx == 0 && value.is_empty() { placeholder.clone() } else { String::new() },
                            value: "{line}",
                            autofocus: true,
                            oninput: {
                                let update = update_line.clone();
                                move |e: Event<FormData>| update(idx, e.value())
                            },
                            onkeydown: {
                                let mut insert = insert_line_after.clone();
                                move |e: Event<KeyboardData>| {
                                    match e.key() {
                                        Key::Enter => {
                                            e.prevent_default();
                                            insert(idx);
                                        }
                                        Key::ArrowUp if idx > 0 => {
                                            active_line.set(idx - 1);
                                        }
                                        Key::ArrowDown if idx < total_lines - 1 => {
                                            active_line.set(idx + 1);
                                        }
                                        _ => {}
                                    }
                                }
                            },
                        }
                    }
                } else if line.trim().is_empty() {
                    div {
                        key: "line-{idx}",
                        class: "md-line md-line-empty",
                        onclick: move |_| active_line.set(idx),
                        "\u{00A0}"
                    }
                } else {
                    match cache.get(&idx) {
                        Some((src, html)) if src == line => rsx! {
                            div {
                                key: "line-{idx}",
                                class: "md-line md-line-view",
                                onclick: move |_| active_line.set(idx),
                                dangerous_inner_html: "{html}",
                            }
                        },
                        _ => rsx! {
                            div {
                                key: "line-{idx}",
                                class: "md-line md-line-raw",
                                onclick: move |_| active_line.set(idx),
                                "{line}"
                            }
                        },
                    }
                }
            }
        }
    }
}

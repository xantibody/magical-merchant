use std::collections::HashMap;

use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// Markdown editor with per-line inline rendering.
///
/// Each line is displayed individually. The active line (where the cursor is)
/// shows raw Markdown. All other lines show rendered HTML.
/// Rendering is triggered asynchronously when the active line changes.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let mut active_line = use_signal(|| 0usize);
    let mut render_cache = use_signal(HashMap::<usize, String>::new);
    let placeholder = placeholder.unwrap_or_default();

    let lines: Vec<String> = if value.is_empty() {
        vec![String::new()]
    } else {
        value.split('\n').map(String::from).collect()
    };
    let total_lines = lines.len();

    // Update render cache for non-active lines that changed
    let mut cache = render_cache();
    let mut cache_dirty = false;
    for (idx, line) in lines.iter().enumerate() {
        if idx == active_line() {
            continue;
        }
        let needs_render = match cache.get(&idx) {
            None => !line.trim().is_empty(),
            Some(cached) => {
                let current_html = if line.trim().is_empty() {
                    String::new()
                } else {
                    render_markdown(line)
                };
                *cached != current_html
            }
        };
        if needs_render {
            let html = if line.trim().is_empty() {
                String::new()
            } else {
                render_markdown(line)
            };
            cache.insert(idx, html);
            cache_dirty = true;
        }
    }
    // Remove stale cache entries for lines that no longer exist
    let stale_keys: Vec<usize> = cache
        .keys()
        .filter(|k| **k >= total_lines)
        .copied()
        .collect();
    for key in stale_keys {
        cache.remove(&key);
        cache_dirty = true;
    }
    if cache_dirty {
        render_cache.set(cache.clone());
    }

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
                    div {
                        key: "line-{idx}",
                        class: "md-line md-line-view",
                        onclick: move |_| active_line.set(idx),
                        dangerous_inner_html: cache.get(&idx).cloned().unwrap_or_default(),
                    }
                }
            }
        }
    }
}
